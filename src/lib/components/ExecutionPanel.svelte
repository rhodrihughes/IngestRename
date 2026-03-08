<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { OperationStatus, PreviewResult } from '$lib/types';

  // Props
  export let status: OperationStatus = {
    state: 'idle',
    progress: 0,
    total: 0,
    results: [],
  };

  export let canExecute: boolean = false;
  export let previewResults: PreviewResult[] = [];

  const dispatch = createEventDispatcher<{ execute: void }>();

  // Two-step confirmation state
  let confirmPending = false;

  // Copy state
  let copyState: 'idle' | 'copied' | 'error' = 'idle';
  let copyTimer: ReturnType<typeof setTimeout>;

  async function copyNewNames() {
    const text = previewResults.map((r) => r.newName).join('\n');
    try {
      await navigator.clipboard.writeText(text);
      copyState = 'copied';
    } catch {
      copyState = 'error';
    }
    clearTimeout(copyTimer);
    copyTimer = setTimeout(() => (copyState = 'idle'), 2000);
  }

  // Derived values
  $: progressPercent = status.total > 0
    ? Math.round((status.progress / status.total) * 100)
    : 0;

  $: successCount = status.results.filter((r) => r.success).length;

  $: failedResults = status.results.filter((r) => !r.success);

  $: isRunning = status.state === 'running';

  // Reset confirmation state when operation starts or status changes
  $: if (isRunning || status.state === 'idle') {
    confirmPending = false;
  }

  function handleExecuteClick() {
    if (!confirmPending) {
      // First click: enter confirmation state
      confirmPending = true;
    } else {
      // Second click: confirmed — dispatch execute
      confirmPending = false;
      dispatch('execute');
    }
  }

  function cancelConfirm() {
    confirmPending = false;
  }

  /**
   * Extract just the filename from a full path for display
   */
  function basename(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }
</script>

<div class="execution-panel">
  <!-- ── Execute button + confirmation ──────────────────────────────────── -->
  <div class="execute-section">
    {#if !confirmPending}
      <button
        class="btn btn-primary"
        on:click={handleExecuteClick}
        disabled={isRunning || !canExecute}
        aria-busy={isRunning}
      >
        {#if isRunning}
          <span class="spinner" aria-hidden="true"></span>
          Renaming…
        {:else}
          Execute Rename
        {/if}
      </button>
      {#if previewResults.length > 0}
        <button
          class="btn btn-ghost copy-btn"
          class:copy-btn-success={copyState === 'copied'}
          class:copy-btn-error={copyState === 'error'}
          on:click={copyNewNames}
          aria-label="Copy new filenames to clipboard"
        >
          {#if copyState === 'copied'}
            <svg class="btn-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 0 1 .143 1.052l-8 10.5a.75.75 0 0 1-1.127.075l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 0 1 1.05-.143Z" clip-rule="evenodd" />
            </svg>
            Copied!
          {:else if copyState === 'error'}
            <svg class="btn-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
            </svg>
            Failed
          {:else}
            <svg class="btn-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path d="M7 3.5A1.5 1.5 0 0 1 8.5 2h3.879a1.5 1.5 0 0 1 1.06.44l3.122 3.12A1.5 1.5 0 0 1 17 6.622V12.5a1.5 1.5 0 0 1-1.5 1.5h-1v-3.379a3 3 0 0 0-.879-2.121L10.5 5.379A3 3 0 0 0 8.379 4.5H7v-1Z" />
              <path d="M4.5 6A1.5 1.5 0 0 0 3 7.5v9A1.5 1.5 0 0 0 4.5 18h7a1.5 1.5 0 0 0 1.5-1.5v-5.879a1.5 1.5 0 0 0-.44-1.06L9.44 6.439A1.5 1.5 0 0 0 8.378 6H4.5Z" />
            </svg>
            Copy new filenames
          {/if}
        </button>
      {/if}
    {:else}
      <div class="confirm-row" role="group" aria-label="Confirm rename operation">
        <span class="confirm-label">Are you sure? This will rename the files.</span>
        <button class="btn btn-danger" on:click={handleExecuteClick}>
          Yes, Rename
        </button>
        <button class="btn btn-ghost" on:click={cancelConfirm}>
          Cancel
        </button>
      </div>
    {/if}
  </div>

  <!-- ── Progress bar (visible while running) ───────────────────────────── -->
  {#if isRunning}
    <div class="progress-section" role="status" aria-live="polite">
      <div class="progress-header">
        <span class="progress-label">Renaming files…</span>
        <span class="progress-count">{status.progress} / {status.total}</span>
      </div>
      <div
        class="progress-track"
        role="progressbar"
        aria-valuenow={status.progress}
        aria-valuemin={0}
        aria-valuemax={status.total}
        aria-label="Rename progress"
      >
        <div class="progress-fill" style="width: {progressPercent}%"></div>
      </div>
      <span class="progress-percent">{progressPercent}%</span>
    </div>
  {/if}

  <!-- ── Completion summary ──────────────────────────────────────────────── -->
  {#if status.state === 'completed' || status.state === 'error'}
    <div class="results-section">
      <!-- Success message -->
      {#if successCount > 0}
        <div class="success-banner" role="status" aria-live="polite">
          <svg class="banner-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z" clip-rule="evenodd" />
          </svg>
          <span>
            Successfully renamed
            <strong>{successCount}</strong>
            {successCount === 1 ? 'file' : 'files'}.
          </span>
        </div>
      {/if}

      <!-- Error summary -->
      {#if failedResults.length > 0}
        <div class="error-summary" role="alert">
          <div class="error-summary-header">
            <svg class="banner-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
            </svg>
            <span>
              <strong>{failedResults.length}</strong>
              {failedResults.length === 1 ? 'file' : 'files'} failed to rename:
            </span>
          </div>
          <ul class="error-list">
            {#each failedResults as result}
              <li class="error-item">
                <span class="error-filename" title={result.originalPath}>
                  {basename(result.originalPath)}
                </span>
                <span class="error-reason">{result.error ?? 'Unknown error'}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .execution-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  /* ── Execute section ─────────────────────────────────────────────────── */
  .execute-section {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .confirm-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .confirm-label {
    font-size: 0.875rem;
    color: #92400e;
    font-weight: 500;
  }

  /* ── Buttons ─────────────────────────────────────────────────────────── */
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1.25rem;
    font-size: 0.875rem;
    font-weight: 600;
    border-radius: 0.375rem;
    border: 1px solid transparent;
    cursor: pointer;
    transition: background-color 0.15s, border-color 0.15s, opacity 0.15s;
    white-space: nowrap;
    line-height: 1.25;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background-color: #4f46e5;
    color: #fff;
    border-color: #4f46e5;
  }

  .btn-primary:not(:disabled):hover {
    background-color: #4338ca;
    border-color: #4338ca;
  }

  .btn-danger {
    background-color: #dc2626;
    color: #fff;
    border-color: #dc2626;
  }

  .btn-danger:not(:disabled):hover {
    background-color: #b91c1c;
    border-color: #b91c1c;
  }

  .btn-ghost {
    background-color: transparent;
    color: #374151;
    border-color: #d1d5db;
  }

  .btn-ghost:not(:disabled):hover {
    background-color: #f3f4f6;
  }

  .copy-btn-success {
    background-color: #f0fdf4 !important;
    border-color: #86efac !important;
    color: #166534 !important;
  }

  .copy-btn-error {
    background-color: #fef2f2 !important;
    border-color: #fca5a5 !important;
    color: #991b1b !important;
  }

  .btn-icon {
    width: 0.875rem;
    height: 0.875rem;
    flex-shrink: 0;
  }

  /* ── Spinner ─────────────────────────────────────────────────────────── */
  .spinner {
    display: inline-block;
    width: 0.875rem;
    height: 0.875rem;
    border: 2px solid rgba(255, 255, 255, 0.4);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* ── Progress section ────────────────────────────────────────────────── */
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .progress-label {
    font-size: 0.875rem;
    color: #374151;
    font-weight: 500;
  }

  .progress-count {
    font-size: 0.8125rem;
    color: #6b7280;
    font-variant-numeric: tabular-nums;
  }

  .progress-track {
    width: 100%;
    height: 0.5rem;
    background-color: #e5e7eb;
    border-radius: 9999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: #4f46e5;
    border-radius: 9999px;
    transition: width 0.2s ease;
    min-width: 0;
  }

  .progress-percent {
    font-size: 0.75rem;
    color: #6b7280;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  /* ── Results section ─────────────────────────────────────────────────── */
  .results-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .banner-icon {
    width: 1.125rem;
    height: 1.125rem;
    flex-shrink: 0;
  }

  /* Success banner */
  .success-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background-color: #f0fdf4;
    border: 1px solid #bbf7d0;
    border-radius: 0.375rem;
    color: #166534;
    font-size: 0.875rem;
  }

  /* Error summary */
  .error-summary {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.375rem;
    color: #991b1b;
    font-size: 0.875rem;
  }

  .error-summary-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .error-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding-left: 1.625rem; /* align under text after icon */
  }

  .error-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .error-filename {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.8125rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .error-reason {
    font-size: 0.75rem;
    color: #b91c1c;
  }

  /* ── Dark mode ───────────────────────────────────────────────────────── */
  @media (prefers-color-scheme: dark) {
    .confirm-label {
      color: #fbbf24;
    }

    .btn-ghost {
      color: #d1d5db;
      border-color: #374151;
    }

    .btn-ghost:not(:disabled):hover {
      background-color: #374151;
    }

    .copy-btn-success {
      background-color: #052e16 !important;
      border-color: #166534 !important;
      color: #86efac !important;
    }

    .copy-btn-error {
      background-color: #450a0a !important;
      border-color: #991b1b !important;
      color: #fca5a5 !important;
    }

    .progress-label {
      color: #d1d5db;
    }

    .progress-count,
    .progress-percent {
      color: #9ca3af;
    }

    .progress-track {
      background-color: #374151;
    }

    .success-banner {
      background-color: #052e16;
      border-color: #166534;
      color: #86efac;
    }

    .error-summary {
      background-color: #450a0a;
      border-color: #991b1b;
      color: #fca5a5;
    }

    .error-reason {
      color: #f87171;
    }
  }
</style>
