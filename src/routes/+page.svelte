<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import FileSelector from '$lib/components/FileSelector.svelte';
  import ConfigPanel from '$lib/components/ConfigPanel.svelte';
  import PresetMenu from '$lib/components/PresetMenu.svelte';
  import FilesTable from '$lib/components/FilesTable.svelte';
  import {
    selectedFiles,
    config,
    previewResults,
    operationStatus,
    isConfigValid,
    hasPreviewErrors,
  } from '$lib/stores';
  import {
    generatePreview,
    executeBatchRenameWithProgress,
    undoBatchRename,
    getSettings,
    listPresets,
    saveLastConfig,
  } from '$lib/ipc';
  import { sortFiles, type SortMode } from '$lib/files';
  import { createDefaultConfig } from '$lib/types';
  import type { FileInfo, RenameConfig, UndoRecord } from '$lib/types';

  // Local reactive state bound to child components
  let localFiles = $state<FileInfo[]>([]);
  let localConfig = $state<RenameConfig>(get(config));
  let previewError = $state<string | undefined>(undefined);
  let lastUndoRecord = $state<UndoRecord | undefined>(undefined);
  let toast = $state<{ text: string; undo: boolean } | undefined>(undefined);
  let skipped = $state<Set<string>>(new Set());
  let sortMode = $state<SortMode>('selection');
  let dragging = $state(false);
  // Preset currently applied (shown in the Presets menu button)
  let activePreset = $state<string | null>(null);
  // Bumped whenever the config is replaced from outside (startup, preset load)
  // so ConfigPanel re-initialises from it
  let configVersion = $state(0);
  let fileSelector: ReturnType<typeof FileSelector> | undefined = $state();

  // Files in numbering order, and the subset that will actually be renamed
  const displayFiles = $derived(sortFiles(localFiles, sortMode));
  const activeFiles = $derived(displayFiles.filter((f) => !skipped.has(f.path)));

  const isRunning = $derived($operationStatus.state === 'running');
  const canRename = $derived(
    activeFiles.length > 0 && $isConfigValid && !$hasPreviewErrors && !isRunning
  );
  const conflictCount = $derived($previewResults.filter((r) => r.status === 'error').length);
  const failedResults = $derived($operationStatus.results.filter((r) => !r.success));
  const progressPercent = $derived(
    $operationStatus.total > 0
      ? Math.round(($operationStatus.progress / $operationStatus.total) * 100)
      : 0
  );
  const examplePreview = $derived($previewResults[0]);

  // -------------------------------------------------------------------------
  // Startup config: the default preset wins, then the last used config.
  // Any failure (missing preset, unreadable settings) falls back silently.
  // -------------------------------------------------------------------------

  async function loadStartupConfig() {
    try {
      const settings = await getSettings();
      let startup: RenameConfig | undefined;
      let startupName: string | null = null;
      if (settings.defaultPreset) {
        const presets = await listPresets();
        startup = presets.find((p) => p.name === settings.defaultPreset)?.config;
        if (startup) startupName = settings.defaultPreset;
      }
      startup ??= settings.lastConfig ?? undefined;
      if (startup) {
        replaceConfig({ ...createDefaultConfig(), ...startup }, false);
        activePreset = startupName;
      }
    } catch {
      /* keep defaults */
    }
  }

  function replaceConfig(next: RenameConfig, remember = true) {
    config.set(next);
    localConfig = next;
    configVersion++;
    if (remember) rememberConfig(next);
  }

  let lastConfigTimer: ReturnType<typeof setTimeout> | undefined;

  function rememberConfig(cfg: RenameConfig) {
    if (lastConfigTimer !== undefined) clearTimeout(lastConfigTimer);
    lastConfigTimer = setTimeout(() => {
      saveLastConfig($state.snapshot(cfg) as RenameConfig).catch(() => {});
    }, 500);
  }

  onMount(() => {
    localFiles = get(selectedFiles);
    localConfig = get(config);
    loadStartupConfig();

    // Accept files dropped anywhere on the window
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        const p = event.payload;
        if (p.type === 'enter' || p.type === 'over') dragging = true;
        else if (p.type === 'leave') dragging = false;
        else if (p.type === 'drop') {
          dragging = false;
          fileSelector?.addPaths(p.paths);
        }
      })
      .then((fn) => (unlisten = fn))
      .catch(() => {});
    return () => unlisten?.();
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
      const results = await generatePreview(files, $state.snapshot(cfg) as RenameConfig);
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

  // Sync files → store (the store feeds validation helpers)
  $effect(() => {
    selectedFiles.set(activeFiles);
  });

  // Regenerate the preview when the files, their order or the config change
  $effect(() => {
    schedulePreview(activeFiles, localConfig);
  });

  // -------------------------------------------------------------------------
  // Table interactions
  // -------------------------------------------------------------------------

  function toggleFile(path: string) {
    const next = new Set(skipped);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    skipped = next;
  }

  function toggleAll(include: boolean) {
    skipped = include ? new Set() : new Set(localFiles.map((f) => f.path));
  }

  function removeFile(path: string) {
    localFiles = localFiles.filter((f) => f.path !== path);
  }

  function clearFiles() {
    localFiles = [];
    skipped = new Set();
  }

  function onConfigChange(event: CustomEvent<RenameConfig>) {
    const newConfig = event.detail;
    localConfig = newConfig;
    config.set(newConfig);
    rememberConfig(newConfig);
  }

  // -------------------------------------------------------------------------
  // Rename + undo
  // -------------------------------------------------------------------------

  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  function showToast(text: string, undo: boolean) {
    toast = { text, undo };
    if (toastTimer !== undefined) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = undefined), undo ? 15000 : 4000);
  }

  async function onExecute() {
    if (!canRename) return;
    const files = $state.snapshot(activeFiles) as FileInfo[];
    const cfg = $state.snapshot(localConfig) as RenameConfig;

    operationStatus.set({ state: 'running', progress: 0, total: files.length, results: [] });
    lastUndoRecord = undefined;
    toast = undefined;

    try {
      const response = await executeBatchRenameWithProgress(files, cfg, (event) => {
        operationStatus.update((s) => ({
          ...s,
          progress: event.current,
          total: event.total,
        }));
      });

      const { results, undoRecord } = response;
      const hasErrors = results.some((r) => !r.success);
      operationStatus.set({
        state: hasErrors ? 'error' : 'completed',
        progress: results.length,
        total: results.length,
        results,
      });

      // Renamed files no longer exist at their old paths, so drop them
      const done = new Set(results.filter((r) => r.success).map((r) => r.originalPath));
      localFiles = localFiles.filter((f) => !done.has(f.path));

      if (undoRecord.entries.length > 0) {
        lastUndoRecord = undoRecord;
        showToast(
          `Renamed ${done.size} file${done.size === 1 ? '' : 's'}`,
          true
        );
      }
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

  async function onUndo() {
    if (!lastUndoRecord) return;
    const record = lastUndoRecord;
    try {
      await undoBatchRename(record);
      lastUndoRecord = undefined;
      showToast('Undone', false);
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Undo failed', false);
    }
  }

  function basename(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function onKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (!mod) return;
    const inField = (e.target as HTMLElement)?.matches?.('input, textarea, select');
    if (e.key === 'o') {
      e.preventDefault();
      fileSelector?.selectFiles();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      onExecute();
    } else if (e.key === 'z' && !inField && lastUndoRecord) {
      e.preventDefault();
      onUndo();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app-layout">
  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <header class="app-header">
    <h1 class="app-title">IngestRename</h1>
    <div class="header-actions">
      <a href="/info" class="info-link" aria-label="Regex quick guide">Regex guide ↗</a>
      <PresetMenu config={localConfig} bind:activeName={activePreset} on:load={(e) => replaceConfig(e.detail)} />
      <FileSelector bind:this={fileSelector} bind:selectedFiles={localFiles} />
    </div>
  </header>

  <main class="app-body">
    <!-- ── Rules ─────────────────────────────────────────────────────────── -->
    <section class="card rules" aria-label="Rename rules">
      <div class="rules-main">
        {#key configVersion}
          <ConfigPanel config={localConfig} on:configChange={onConfigChange} />
        {/key}
      </div>

      <aside class="example">
        <h2 class="example-title">Example</h2>
        {#if examplePreview}
          <p class="example-old">{examplePreview.originalName}</p>
          <p class="example-new">{examplePreview.newName}</p>
        {:else}
          <p class="example-old">Add files to see an example.</p>
        {/if}

        <label class="example-label" for="sort-mode">Number files in this order</label>
        <select id="sort-mode" class="example-select" bind:value={sortMode}>
          <option value="selection">Order added</option>
          <option value="name-asc">Name A → Z</option>
          <option value="name-desc">Name Z → A</option>
        </select>
      </aside>
    </section>

    {#if previewError}
      <div class="alert" role="alert">{previewError}</div>
    {/if}

    <!-- ── Files + preview ───────────────────────────────────────────────── -->
    <FilesTable
      files={displayFiles}
      previews={$previewResults}
      {skipped}
      {dragging}
      ontoggle={toggleFile}
      onremove={removeFile}
      ontoggleall={toggleAll}
    />

    {#if failedResults.length > 0}
      <div class="alert" role="alert">
        <strong>{failedResults.length} file{failedResults.length === 1 ? '' : 's'} failed to rename:</strong>
        <ul class="fail-list">
          {#each failedResults as r}
            <li><code>{basename(r.originalPath)}</code> — {r.error ?? 'Unknown error'}</li>
          {/each}
        </ul>
      </div>
    {/if}

    <!-- ── Footer ────────────────────────────────────────────────────────── -->
    <footer class="footer">
      <div class="footer-status" aria-live="polite">
        {#if isRunning}
          <div class="progress" role="progressbar" aria-valuenow={$operationStatus.progress} aria-valuemin={0} aria-valuemax={$operationStatus.total}>
            <div class="progress-fill" style="width: {progressPercent}%"></div>
          </div>
          <span>Renaming {$operationStatus.progress} / {$operationStatus.total}…</span>
        {:else if conflictCount > 0}
          <span class="bad">{conflictCount} conflict{conflictCount === 1 ? '' : 's'} to fix</span>
        {:else if activeFiles.length > 0}
          <span>{activeFiles.length} of {localFiles.length} files will be renamed</span>
        {/if}
      </div>
      <button class="btn" type="button" onclick={clearFiles} disabled={localFiles.length === 0 || isRunning}>Clear</button>
      <button class="btn btn-primary" type="button" onclick={onExecute} disabled={!canRename} title="⌘Enter">
        Rename{activeFiles.length > 0 ? ` ${activeFiles.length}` : ''}
      </button>
    </footer>
  </main>

  <!-- ── Toast ───────────────────────────────────────────────────────────── -->
  {#if toast}
    <div class="toast" role="status">
      <span>{toast.text}</span>
      {#if toast.undo && lastUndoRecord}
        <button class="toast-undo" type="button" onclick={onUndo}>Undo</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background-color: #f3f4f6;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }

  /* ── Header ──────────────────────────────────────────────────────────── */
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.625rem 1rem;
    background-color: #fff;
    border-bottom: 1px solid #e5e7eb;
  }

  .app-title {
    font-size: 1rem;
    font-weight: 700;
    color: #111827;
    margin: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .info-link {
    font-size: 0.8125rem;
    color: #4f46e5;
    text-decoration: none;
    margin-right: 0.5rem;
  }

  .info-link:hover {
    text-decoration: underline;
  }

  /* ── Body ────────────────────────────────────────────────────────────── */
  .app-body {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
    padding: 1rem;
    flex: 1;
  }

  .card {
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    padding: 1rem;
  }

  .rules {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 17rem;
    gap: 1rem;
  }

  @media (max-width: 800px) {
    .rules {
      grid-template-columns: 1fr;
    }
  }

  /* ── Example box ─────────────────────────────────────────────────────── */
  .example {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-self: start;
    padding: 0.875rem;
    background-color: #eef2ff;
    border: 1px solid #c7d2fe;
    border-radius: 0.5rem;
  }

  .example-title {
    margin: 0 0 0.25rem;
    font-size: 0.8125rem;
    font-weight: 600;
    color: #3730a3;
  }

  .example-old,
  .example-new {
    margin: 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.8125rem;
    overflow-wrap: anywhere;
  }

  .example-old {
    color: #6b7280;
  }

  .example-new {
    color: #111827;
    font-weight: 600;
  }

  .example-label {
    margin-top: 0.75rem;
    font-size: 0.75rem;
    color: #4b5563;
  }

  .example-select {
    padding: 0.375rem 0.5rem;
    font-size: 0.8125rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #111827;
  }

  /* ── Alerts ──────────────────────────────────────────────────────────── */
  .alert {
    padding: 0.625rem 0.875rem;
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.375rem;
    color: #991b1b;
    font-size: 0.8125rem;
  }

  .fail-list {
    margin: 0.25rem 0 0;
    padding-left: 1.25rem;
  }

  /* ── Footer ──────────────────────────────────────────────────────────── */
  .footer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .footer-status {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: #6b7280;
  }

  .bad {
    color: #b91c1c;
    font-weight: 500;
  }

  .progress {
    width: 8rem;
    height: 0.375rem;
    background-color: #e5e7eb;
    border-radius: 9999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: #4f46e5;
    transition: width 0.2s ease;
  }

  .btn {
    padding: 0.4375rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #374151;
    cursor: pointer;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background-color: #4f46e5;
    border-color: #4f46e5;
    color: #fff;
    font-weight: 600;
  }

  .btn-primary:not(:disabled):hover {
    background-color: #4338ca;
  }

  /* ── Toast ───────────────────────────────────────────────────────────── */
  .toast {
    position: fixed;
    left: 50%;
    bottom: 1.25rem;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background-color: #111827;
    color: #fff;
    font-size: 0.875rem;
    border-radius: 9999px;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25);
  }

  .toast-undo {
    background: none;
    border: none;
    color: #a5b4fc;
    font-weight: 600;
    cursor: pointer;
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

    .info-link {
      color: #818cf8;
    }

    .card {
      background-color: #1f2937;
      border-color: #374151;
    }

    .example {
      background-color: #1e1b4b;
      border-color: #3730a3;
    }

    .example-title {
      color: #c7d2fe;
    }

    .example-new {
      color: #f9fafb;
    }

    .example-label {
      color: #9ca3af;
    }

    .example-select,
    .btn {
      background-color: #1f2937;
      border-color: #374151;
      color: #d1d5db;
    }

    .btn-primary {
      background-color: #4f46e5;
      border-color: #4f46e5;
      color: #fff;
    }

    .alert {
      background-color: #450a0a;
      border-color: #991b1b;
      color: #fca5a5;
    }

    .progress {
      background-color: #374151;
    }

    .toast {
      background-color: #f9fafb;
      color: #111827;
    }

    .toast-undo {
      color: #4f46e5;
    }
  }
</style>
