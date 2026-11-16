<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { readDir } from '@tauri-apps/plugin-fs';
  import { mergeFiles } from '$lib/files';
  import type { FileInfo } from '$lib/types';

  // Props
  let { selectedFiles = $bindable([]) }: { selectedFiles?: FileInfo[] } = $props();

  let isSelecting = $state(false);
  let error = $state<string | undefined>(undefined);
  let menuOpen = $state(false);

  /** Add files by path (used by the pickers and by drag-and-drop). */
  export async function addPaths(paths: string[]) {
    try {
      error = undefined;
      selectedFiles = await mergeFiles(selectedFiles, paths);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to add files';
    }
  }

  export async function selectFiles() {
    menuOpen = false;
    try {
      isSelecting = true;
      error = undefined;
      const selected = await open({ multiple: true, directory: false });
      if (selected === null || selected === undefined) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) await addPaths(paths);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to select files';
    } finally {
      isSelecting = false;
    }
  }

  export async function selectFolder() {
    menuOpen = false;
    try {
      isSelecting = true;
      error = undefined;
      const selected = await open({ multiple: false, directory: true });
      if (selected === null || selected === undefined) return;
      const folderPath = Array.isArray(selected) ? selected[0] : selected;

      // Top-level files only, in natural name order
      const entries = await readDir(folderPath);
      const filePaths = entries
        .filter((e) => e.isFile)
        .sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' }))
        .map((e) => `${folderPath}/${e.name}`);

      if (filePaths.length === 0) {
        error = 'No files found in the selected folder.';
        return;
      }
      await addPaths(filePaths);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to read folder';
    } finally {
      isSelecting = false;
    }
  }
</script>

<div class="file-selector">
  <div class="add-wrap">
    <button class="add-button" onclick={() => (menuOpen = !menuOpen)} disabled={isSelecting} aria-haspopup="menu" aria-expanded={menuOpen}>
      + Add
    </button>
    {#if menuOpen}
      <div class="menu" role="menu">
        <button class="menu-item" role="menuitem" onclick={selectFiles}>Files…</button>
        <button class="menu-item" role="menuitem" onclick={selectFolder}>Folder…</button>
      </div>
    {/if}
  </div>
  {#if error}
    <span class="error-message" role="alert">{error}</span>
  {/if}
</div>

<style>
  .file-selector {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .add-wrap {
    position: relative;
  }

  .add-button {
    padding: 0.375rem 0.875rem;
    font-size: 0.8125rem;
    font-weight: 600;
    background-color: #4f46e5;
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
  }

  .add-button:hover:not(:disabled) {
    background-color: #4338ca;
  }

  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 0.25rem);
    z-index: 20;
    min-width: 8rem;
    display: flex;
    flex-direction: column;
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.375rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    overflow: hidden;
  }

  .menu-item {
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
    text-align: left;
    background: none;
    border: none;
    color: #374151;
    cursor: pointer;
  }

  .menu-item:hover {
    background-color: #f3f4f6;
  }

  .error-message {
    font-size: 0.75rem;
    color: #dc2626;
  }

  @media (prefers-color-scheme: dark) {
    .menu {
      background-color: #1f2937;
      border-color: #374151;
    }

    .menu-item {
      color: #d1d5db;
    }

    .menu-item:hover {
      background-color: #374151;
    }
  }
</style>
