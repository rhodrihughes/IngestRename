<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import FileSelector from '$lib/components/FileSelector.svelte';
  import ConfigPanel from '$lib/components/ConfigPanel.svelte';
  import PreviewTable from '$lib/components/PreviewTable.svelte';
  import ExecutionPanel from '$lib/components/ExecutionPanel.svelte';
  import {
    selectedFiles,
    config,
    previewResults,
    operationStatus,
    canExecute,
  } from '$lib/stores';
  import { generatePreview, executeBatchRenameWithProgress } from '$lib/ipc';
  import type { FileInfo, RenameConfig } from '$lib/types';

  // Local reactive state bound to child components
  // FileSelector uses $bindable, so we keep a local array and sync to the store
  let localFiles = $state<FileInfo[]>([]);
  let localConfig = $state<RenameConfig>(get(config));
  let previewError = $state<string | undefined>(undefined);

  // Sync store → local on mount (handles hot-reload / pre-populated state)
  onMount(() => {
    localFiles = get(selectedFiles);
    localConfig = get(config);
    if (localFiles.length > 0) {
      triggerPreview(localFiles, localConfig);
    }
  });

  // -------------------------------------------------------------------------
  // Preview generation
  // -------------------------------------------------------------------------

  let previewDebounceTimer: ReturnType<typeof setTimeout> | undefined;

  async function triggerPreview(files: FileInfo[], cfg: RenameConfig) {
    if (files.length === 0) {
      previewResults.set([]);
      previewError = undefined;
      return;
    }
    try {
      previewError = undefined;
      const results = await generatePreview(files, cfg);
      previewResults.set(results);
    } catch (err) {
      previewError = err instanceof Error ? err.message : 'Failed to generate preview';
      console.error('Preview generation error:', err);
    }
  }

  function schedulePreview(files: FileInfo[], cfg: RenameConfig) {
    if (previewDebounceTimer !== undefined) clearTimeout(previewDebounceTimer);
    previewDebounceTimer = setTimeout(() => triggerPreview(files, cfg), 150);
  }

  // -------------------------------------------------------------------------
  // Reactive effects: sync local state → stores and trigger preview
  // -------------------------------------------------------------------------

  // When localFiles changes (FileSelector updated it via bind:), sync store + preview
  $effect(() => {
    selectedFiles.set(localFiles);
    schedulePreview(localFiles, localConfig);
  });

  // -------------------------------------------------------------------------
  // Event handlers (Svelte 4 components use on:event)
  // -------------------------------------------------------------------------

  function onConfigChange(event: CustomEvent<RenameConfig>) {
    const newConfig = event.detail;
    localConfig = newConfig;
    config.set(newConfig);
    schedulePreview(get(selectedFiles), newConfig);
  }

  async function onExecute() {
    const files = get(selectedFiles);
    const cfg = get(config);
    if (files.length === 0) return;

    operationStatus.set({ state: 'running', progress: 0, total: files.length, results: [] });

    try {
      const results = await executeBatchRenameWithProgress(files, cfg, (event) => {
        operationStatus.update((s) => ({
          ...s,
          progress: event.current,
          total: event.total,
        }));
      });

      const hasErrors = results.some((r) => !r.success);
      operationStatus.set({
        state: hasErrors ? 'error' : 'completed',
        progress: results.length,
        total: results.length,
        results,
      });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Batch rename failed';
      operationStatus.update((s) => ({
        ...s,
        state: 'error',
        results: [
          ...s.results,
          { originalPath: '', newPath: '', success: false, error: errorMsg },
        ],
      }));
    }
  }
</script>

<div class="app-layout">
  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <header class="app-header">
    <h1 class="app-title">Batch File Renamer</h1>
    <p class="app-subtitle">Select files, configure the naming convention, preview, and rename.</p>
  </header>

  <!-- ── Main content ────────────────────────────────────────────────────── -->
  <div class="app-body">
    <!-- Left column: controls -->
    <aside class="controls-column">
      <!-- File selection -->
      <section class="panel">
        <h2 class="panel-title">1. Select Files</h2>
        <FileSelector bind:selectedFiles={localFiles} />
      </section>

      <!-- Configuration -->
      <section class="panel">
        <h2 class="panel-title">2. Configure</h2>
        <ConfigPanel
          config={localConfig}
          on:configChange={onConfigChange}
        />
      </section>

      <!-- Execution -->
      <section class="panel">
        <h2 class="panel-title">3. Execute</h2>
        {#if previewError}
          <div class="preview-error" role="alert">{previewError}</div>
        {/if}
        <ExecutionPanel
          status={$operationStatus}
          canExecute={$canExecute}
          previewResults={$previewResults}
          on:execute={onExecute}
        />
      </section>
    </aside>

    <!-- Right column: preview -->
    <main class="preview-column">
      <section class="panel panel-preview">
        <h2 class="panel-title">Preview</h2>
        <PreviewTable previewResults={$previewResults} />
      </section>
    </main>
  </div>
</div>

<style>
  /* ── Layout ──────────────────────────────────────────────────────────── */
  .app-layout {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background-color: #f3f4f6;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }

  /* ── Header ──────────────────────────────────────────────────────────── */
  .app-header {
    padding: 1rem 1.5rem 0.75rem;
    background-color: #fff;
    border-bottom: 1px solid #e5e7eb;
  }

  .app-title {
    font-size: 1.25rem;
    font-weight: 700;
    color: #111827;
    margin: 0 0 0.125rem;
  }

  .app-subtitle {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  /* ── Body ────────────────────────────────────────────────────────────── */
  .app-body {
    display: grid;
    grid-template-columns: 22rem 1fr;
    gap: 1rem;
    padding: 1rem 1.5rem;
    flex: 1;
    align-items: start;
  }

  /* ── Columns ─────────────────────────────────────────────────────────── */
  .controls-column {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .preview-column {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  /* ── Panels ──────────────────────────────────────────────────────────── */
  .panel {
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .panel-preview {
    min-height: 20rem;
  }

  .panel-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: #374151;
    margin: 0;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #f3f4f6;
  }

  /* ── Preview error ───────────────────────────────────────────────────── */
  .preview-error {
    padding: 0.625rem 0.875rem;
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.375rem;
    color: #991b1b;
    font-size: 0.8125rem;
  }

  /* ── Responsive: stack on narrow windows ─────────────────────────────── */
  @media (max-width: 800px) {
    .app-body {
      grid-template-columns: 1fr;
    }
  }

  /* ── Dark mode ───────────────────────────────────────────────────────── */
  @media (prefers-color-scheme: dark) {
    .app-layout {
      background-color: #111827;
    }

    .app-header {
      background-color: #1f2937;
      border-bottom-color: #374151;
    }

    .app-title {
      color: #f9fafb;
    }

    .app-subtitle {
      color: #9ca3af;
    }

    .panel {
      background-color: #1f2937;
      border-color: #374151;
    }

    .panel-title {
      color: #d1d5db;
      border-bottom-color: #374151;
    }

    .preview-error {
      background-color: #450a0a;
      border-color: #991b1b;
      color: #fca5a5;
    }
  }
</style>
