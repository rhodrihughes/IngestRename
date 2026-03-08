<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { getFileInfo } from '$lib/ipc';
  import type { FileInfo } from '$lib/types';
  import { Button } from 'bits-ui';

  // Props
  let { selectedFiles = $bindable([]) }: { selectedFiles?: FileInfo[] } = $props();

  // Local state
  let isSelecting = $state(false);
  let error = $state<string | undefined>(undefined);

  /**
   * Handle file selection using Tauri dialog API
   * Validates: Requirements 1.1, 1.2, 1.3, 1.4
   */
  async function selectFiles() {
    try {
      isSelecting = true;
      error = undefined;

      // Open native file picker with multi-file selection
      const selected = await open({
        multiple: true,
        directory: false,
      });

      // Handle selection result
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        
        // Get file metadata from backend
        const fileInfos = await getFileInfo(paths);
        
        // Update selected files (preserves selection order - Requirement 1.4)
        selectedFiles = fileInfos;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to select files';
      console.error('File selection error:', err);
    } finally {
      isSelecting = false;
    }
  }

  /**
   * Remove a file from the selection
   */
  function removeFile(index: number) {
    selectedFiles = selectedFiles.filter((_, i) => i !== index);
  }

  /**
   * Clear all selected files
   */
  function clearSelection() {
    selectedFiles = [];
  }

  /**
   * Format file size for display
   */
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  }
</script>

<div class="file-selector">
  <!-- File selection button -->
  <div class="selection-controls">
    <Button.Root
      onclick={selectFiles}
      disabled={isSelecting}
      class="select-button"
    >
      {isSelecting ? 'Selecting...' : 'Select Files'}
    </Button.Root>

    {#if selectedFiles.length > 0}
      <Button.Root
        onclick={clearSelection}
        variant="outline"
        class="clear-button"
      >
        Clear All ({selectedFiles.length})
      </Button.Root>
    {/if}
  </div>

  <!-- Error message -->
  {#if error}
    <div class="error-message" role="alert">
      {error}
    </div>
  {/if}

  <!-- Selected files list -->
  {#if selectedFiles.length > 0}
    <div class="files-list">
      <h3 class="list-title">Selected Files ({selectedFiles.length})</h3>
      <ul class="files">
        {#each selectedFiles as file, index (file.path)}
          <li class="file-item">
            <div class="file-info">
              <span class="file-number">{index + 1}.</span>
              <div class="file-details">
                <span class="file-name" title={file.path}>{file.name}</span>
                <span class="file-meta">
                  {formatFileSize(file.size)}
                  {#if file.extension}
                    · {file.extension}
                  {/if}
                </span>
              </div>
            </div>
            <Button.Root
              onclick={() => removeFile(index)}
              variant="ghost"
              size="sm"
              class="remove-button"
              aria-label="Remove {file.name}"
            >
              ×
            </Button.Root>
          </li>
        {/each}
      </ul>
    </div>
  {:else if !error}
    <div class="empty-state">
      <p>No files selected. Click "Select Files" to choose files to rename.</p>
    </div>
  {/if}
</div>

<style>
  .file-selector {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .selection-controls {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .error-message {
    padding: 0.75rem;
    background-color: #fee;
    border: 1px solid #fcc;
    border-radius: 0.375rem;
    color: #c00;
    font-size: 0.875rem;
  }

  .files-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: #374151;
    margin: 0;
  }

  .files {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    max-height: 400px;
    overflow-y: auto;
    border: 1px solid #e5e7eb;
    border-radius: 0.375rem;
    padding: 0.5rem;
  }

  .file-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background-color: #f9fafb;
    border-radius: 0.25rem;
    transition: background-color 0.15s;
  }

  .file-item:hover {
    background-color: #f3f4f6;
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-width: 0;
  }

  .file-number {
    font-size: 0.75rem;
    color: #6b7280;
    font-weight: 500;
    flex-shrink: 0;
  }

  .file-details {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }

  .file-name {
    font-size: 0.875rem;
    color: #111827;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    font-size: 0.75rem;
    color: #6b7280;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
    color: #6b7280;
    background-color: #f9fafb;
    border: 1px dashed #d1d5db;
    border-radius: 0.375rem;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.875rem;
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .list-title {
      color: #d1d5db;
    }

    .files {
      border-color: #374151;
    }

    .file-item {
      background-color: #1f2937;
    }

    .file-item:hover {
      background-color: #374151;
    }

    .file-name {
      color: #f9fafb;
    }

    .file-number,
    .file-meta {
      color: #9ca3af;
    }

    .empty-state {
      background-color: #1f2937;
      border-color: #4b5563;
      color: #9ca3af;
    }

    .error-message {
      background-color: #7f1d1d;
      border-color: #991b1b;
      color: #fecaca;
    }
  }
</style>
