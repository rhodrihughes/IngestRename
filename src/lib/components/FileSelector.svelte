<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { readDir } from '@tauri-apps/plugin-fs';
  import { getFileInfo } from '$lib/ipc';
  import type { FileInfo } from '$lib/types';

  // Props
  let { selectedFiles = $bindable([]) }: { selectedFiles?: FileInfo[] } = $props();

  // Local state
  let isSelecting = $state(false);
  let error = $state<string | undefined>(undefined);

  async function addPaths(paths: string[]) {
    const fileInfos = await getFileInfo(paths);
    const existingPaths = new Set(selectedFiles.map(f => f.path));
    const newFiles = fileInfos.filter(f => !existingPaths.has(f.path));
    selectedFiles = [...selectedFiles, ...newFiles];
  }

  async function selectFiles() {
    try {
      isSelecting = true;
      error = undefined;

      const selected = await open({ multiple: true, directory: false });
      if (selected === null || selected === undefined) return;

      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) return;

      await addPaths(paths);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to select files';
      console.error('File selection error:', err);
    } finally {
      isSelecting = false;
    }
  }

  async function selectFolder() {
    try {
      isSelecting = true;
      error = undefined;

      const selected = await open({ multiple: false, directory: true });
      if (selected === null || selected === undefined) return;

      const folderPath = Array.isArray(selected) ? selected[0] : selected;

      // Read top-level entries, filter to files only (no subdirectories)
      const entries = await readDir(folderPath);
      const filePaths = entries
        .filter(e => e.isFile)
        .sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' }))
        .map(e => `${folderPath}/${e.name}`);

      if (filePaths.length === 0) {
        error = 'No files found in the selected folder.';
        return;
      }

      await addPaths(filePaths);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to read folder';
      console.error('Folder selection error:', err);
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
    <button class="select-button" onclick={selectFiles} disabled={isSelecting}>
      {isSelecting ? 'Selecting...' : 'Select Files'}
    </button>
    <button class="select-button select-button--folder" onclick={selectFolder} disabled={isSelecting}>
      Select Folder
    </button>
    {#if selectedFiles.length > 0}
      <button class="clear-button" onclick={clearSelection}>
        Clear All ({selectedFiles.length})
      </button>
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
            <button
              class="remove-button"
              onclick={() => removeFile(index)}
              aria-label="Remove {file.name}"
            >×</button>
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

  .select-button {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    background-color: #4f46e5;
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .select-button:hover:not(:disabled) {
    background-color: #4338ca;
  }

  .select-button--folder {
    background-color: #6b7280;
  }

  .select-button--folder:hover:not(:disabled) {
    background-color: #4b5563;
  }

  .clear-button {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    background-color: transparent;
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .clear-button:hover {
    background-color: #f3f4f6;
  }

  .remove-button {
    background: none;
    border: none;
    cursor: pointer;
    color: #9ca3af;
    font-size: 1rem;
    padding: 0.25rem 0.375rem;
    border-radius: 0.25rem;
    line-height: 1;
    transition: color 0.15s, background-color 0.15s;
    flex-shrink: 0;
  }

  .remove-button:hover {
    color: #ef4444;
    background-color: #fee2e2;
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

    .clear-button {
      color: #d1d5db;
      border-color: #374151;
    }

    .clear-button:hover {
      background-color: #374151;
    }
  }
</style>
