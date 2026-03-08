<script lang="ts">
  import type { PreviewResult } from '$lib/types';

  export let previewResults: PreviewResult[] = [];
</script>

<div class="preview-table-wrapper">
  {#if previewResults.length === 0}
    <div class="empty-state" role="status">
      <svg class="empty-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
      </svg>
      <p class="empty-title">No files selected</p>
      <p class="empty-subtitle">Select files to see a preview of the rename operation.</p>
    </div>
  {:else}
    <div class="table-container" role="region" aria-label="Rename preview">
      <table class="table">
        <thead>
          <tr>
            <th scope="col" class="th">Original Name</th>
            <th scope="col" class="th">New Name</th>
            <th scope="col" class="th th-status">Status</th>
          </tr>
        </thead>
        <tbody>
          {#each previewResults as result, i (result.originalPath)}
            <tr
              class="tr"
              class:tr-error={result.status === 'error'}
              class:tr-warning={result.status === 'warning'}
              aria-rowindex={i + 1}
            >
              <td class="td td-original">
                <span class="filename" title={result.originalPath}>{result.originalName}</span>
              </td>
              <td class="td td-new">
                <span class="filename" title={result.newPath}>{result.newName}</span>
                {#if result.message && result.status !== 'valid'}
                  <span class="message" role="note">{result.message}</span>
                {/if}
              </td>
              <td class="td td-status">
                {#if result.status === 'valid'}
                  <span class="badge badge-valid" aria-label="Valid">
                    <svg class="badge-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                      <path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 0 1 .143 1.052l-8 10.5a.75.75 0 0 1-1.127.075l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 0 1 1.05-.143Z" clip-rule="evenodd" />
                    </svg>
                    Valid
                  </span>
                {:else if result.status === 'warning'}
                  <span class="badge badge-warning" aria-label="Warning">
                    <svg class="badge-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                      <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                    </svg>
                    Warning
                  </span>
                {:else}
                  <span class="badge badge-error" aria-label="Error">
                    <svg class="badge-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                      <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                    </svg>
                    Error
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <p class="summary" aria-live="polite">
      {previewResults.length} file{previewResults.length === 1 ? '' : 's'} —
      {previewResults.filter(r => r.status === 'valid').length} valid,
      {previewResults.filter(r => r.status === 'warning').length} warning{previewResults.filter(r => r.status === 'warning').length === 1 ? '' : 's'},
      {previewResults.filter(r => r.status === 'error').length} error{previewResults.filter(r => r.status === 'error').length === 1 ? '' : 's'}
    </p>
  {/if}
</div>

<style>
  .preview-table-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  /* ── Empty state ─────────────────────────────────────────────────────── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 3rem 1.5rem;
    border: 1px dashed #d1d5db;
    border-radius: 0.5rem;
    text-align: center;
    color: #6b7280;
  }

  .empty-icon {
    width: 2.5rem;
    height: 2.5rem;
    color: #9ca3af;
  }

  .empty-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: #374151;
    margin: 0;
  }

  .empty-subtitle {
    font-size: 0.875rem;
    margin: 0;
  }

  /* ── Table container ─────────────────────────────────────────────────── */
  .table-container {
    overflow-x: auto;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  /* ── Header ──────────────────────────────────────────────────────────── */
  .th {
    padding: 0.625rem 0.875rem;
    text-align: left;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    background-color: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    white-space: nowrap;
  }

  .th-status {
    width: 7rem;
  }

  /* ── Rows ────────────────────────────────────────────────────────────── */
  .tr {
    border-bottom: 1px solid #f3f4f6;
    transition: background-color 0.1s;
  }

  .tr:last-child {
    border-bottom: none;
  }

  .tr:hover {
    background-color: #f9fafb;
  }

  .tr-warning {
    background-color: #fffbeb;
  }

  .tr-warning:hover {
    background-color: #fef3c7;
  }

  .tr-error {
    background-color: #fef2f2;
  }

  .tr-error:hover {
    background-color: #fee2e2;
  }

  /* ── Cells ───────────────────────────────────────────────────────────── */
  .td {
    padding: 0.625rem 0.875rem;
    vertical-align: top;
    color: #111827;
  }

  .td-original {
    color: #6b7280;
    max-width: 16rem;
  }

  .td-new {
    max-width: 20rem;
  }

  .td-status {
    white-space: nowrap;
  }

  .filename {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.8125rem;
  }

  .message {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #92400e;
    line-height: 1.4;
  }

  .tr-error .message {
    color: #991b1b;
  }

  /* ── Status badges ───────────────────────────────────────────────────── */
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.1875rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
    white-space: nowrap;
  }

  .badge-icon {
    width: 0.875rem;
    height: 0.875rem;
    flex-shrink: 0;
  }

  .badge-valid {
    background-color: #dcfce7;
    color: #166534;
  }

  .badge-warning {
    background-color: #fef9c3;
    color: #854d0e;
  }

  .badge-error {
    background-color: #fee2e2;
    color: #991b1b;
  }

  /* ── Summary line ────────────────────────────────────────────────────── */
  .summary {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
    padding: 0 0.25rem;
  }

  /* ── Dark mode ───────────────────────────────────────────────────────── */
  @media (prefers-color-scheme: dark) {
    .empty-state {
      border-color: #374151;
      color: #9ca3af;
    }

    .empty-title {
      color: #d1d5db;
    }

    .table-container {
      border-color: #374151;
    }

    .th {
      background-color: #1f2937;
      color: #9ca3af;
      border-bottom-color: #374151;
    }

    .tr {
      border-bottom-color: #1f2937;
    }

    .tr:hover {
      background-color: #1f2937;
    }

    .tr-warning {
      background-color: #1c1a0a;
    }

    .tr-warning:hover {
      background-color: #292510;
    }

    .tr-error {
      background-color: #1c0a0a;
    }

    .tr-error:hover {
      background-color: #291010;
    }

    .td {
      color: #f3f4f6;
    }

    .td-original {
      color: #9ca3af;
    }

    .message {
      color: #fbbf24;
    }

    .tr-error .message {
      color: #f87171;
    }

    .badge-valid {
      background-color: #14532d;
      color: #86efac;
    }

    .badge-warning {
      background-color: #422006;
      color: #fde68a;
    }

    .badge-error {
      background-color: #450a0a;
      color: #fca5a5;
    }

    .summary {
      color: #9ca3af;
    }
  }
</style>
