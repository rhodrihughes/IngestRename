<script lang="ts">
  import type { FileInfo, PreviewResult } from '$lib/types';
  import { diffName } from '$lib/files';

  let {
    files,
    previews,
    skipped,
    dragging = false,
    ontoggle,
    onremove,
    ontoggleall,
  }: {
    files: FileInfo[];
    previews: PreviewResult[];
    skipped: Set<string>;
    dragging?: boolean;
    ontoggle: (path: string) => void;
    onremove: (path: string) => void;
    ontoggleall: (include: boolean) => void;
  } = $props();

  const byPath = $derived(new Map(previews.map((p) => [p.originalPath, p])));
  const includedCount = $derived(files.filter((f) => !skipped.has(f.path)).length);
  const allIncluded = $derived(files.length > 0 && includedCount === files.length);
</script>

<div class="table" class:dragging role="region" aria-label="Files and rename preview">
  {#if files.length === 0}
    <div class="empty" role="status">
      <p class="empty-title">Drop files here</p>
      <p class="empty-sub">or use + Add (⌘O) to choose files or a folder.</p>
    </div>
  {:else}
    <div class="head">
      <label class="cell-check">
        <input
          type="checkbox"
          checked={allIncluded}
          aria-label="Include all files"
          onchange={(e) => ontoggleall((e.target as HTMLInputElement).checked)}
        />
      </label>
      <span class="col-orig">Original</span>
      <span class="col-arrow"></span>
      <span class="col-new">New name</span>
      <span class="col-count">{files.length} file{files.length === 1 ? '' : 's'}</span>
    </div>

    <ul class="rows">
      {#each files as file (file.path)}
        {@const isSkipped = skipped.has(file.path)}
        {@const preview = byPath.get(file.path)}
        <li class="row" class:row-skipped={isSkipped} class:row-error={!isSkipped && preview?.status === 'error'} class:row-warning={!isSkipped && preview?.status === 'warning'}>
          <label class="cell-check">
            <input type="checkbox" checked={!isSkipped} aria-label="Include {file.name}" onchange={() => ontoggle(file.path)} />
          </label>
          <span class="col-orig name" title={file.path}>{file.name}</span>
          <span class="col-arrow">→</span>
          <span class="col-new name">
            {#if isSkipped}
              <span class="muted">skipped</span>
            {:else if preview}
              {@const d = diffName(preview.originalName, preview.newName)}
              <span title={preview.newPath}>{#if d.before}<mark>{d.before}</mark>{/if}{d.kept}{#if d.after}<mark>{d.after}</mark>{/if}</span>
            {:else}
              <span class="muted">…</span>
            {/if}
          </span>
          <span class="col-count status">
            {#if !isSkipped && preview && preview.status !== 'valid' && preview.message}
              <span class="msg" class:msg-error={preview.status === 'error'}>
                {preview.status === 'error' ? '⚠' : '!'} {preview.message}
              </span>
            {/if}
            <button class="remove" type="button" aria-label="Remove {file.name}" onclick={() => onremove(file.path)}>✕</button>
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .table {
    display: flex;
    flex-direction: column;
    min-height: 12rem;
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .table.dragging {
    border-color: #4f46e5;
    box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.2);
  }

  .head,
  .row {
    display: grid;
    grid-template-columns: 2rem minmax(0, 1fr) 1.5rem minmax(0, 1.4fr) minmax(8rem, auto);
    align-items: center;
    gap: 0.5rem;
    padding: 0 0.75rem;
  }

  .head {
    height: 2.125rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: #374151;
    background-color: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
  }

  .col-count {
    text-align: right;
    color: #6b7280;
    font-weight: 400;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
  }

  .row {
    min-height: 2rem;
    font-size: 0.8125rem;
    border-bottom: 1px solid #f3f4f6;
  }

  .row:hover {
    background-color: #f9fafb;
  }

  .row-skipped .name,
  .row-skipped .col-arrow {
    color: #9ca3af;
  }

  .row-error {
    background-color: #fef2f2;
  }

  .row-warning {
    background-color: #fffbeb;
  }

  .name {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.8125rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #374151;
  }

  .col-new {
    color: #111827;
  }

  .col-arrow {
    color: #9ca3af;
    text-align: center;
  }

  mark {
    background: none;
    color: #16a34a;
    font-weight: 600;
  }

  .muted {
    color: #9ca3af;
  }

  .status {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.25rem;
    min-width: 0;
  }

  .msg {
    font-size: 0.75rem;
    color: #92400e;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .msg-error {
    color: #b91c1c;
  }

  .remove {
    background: none;
    border: none;
    padding: 0.125rem 0.25rem;
    font-size: 0.75rem;
    color: #9ca3af;
    cursor: pointer;
    border-radius: 0.25rem;
    opacity: 0;
  }

  .row:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }

  .remove:hover {
    color: #ef4444;
    background-color: #fee2e2;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    padding: 2rem;
    margin: 0.75rem;
    border: 2px dashed #d1d5db;
    border-radius: 0.5rem;
    color: #6b7280;
  }

  .empty-title {
    margin: 0;
    font-weight: 600;
    color: #374151;
  }

  .empty-sub {
    margin: 0;
    font-size: 0.8125rem;
  }

  @media (prefers-color-scheme: dark) {
    .table {
      background-color: #1f2937;
      border-color: #374151;
    }

    .head {
      background-color: #111827;
      border-bottom-color: #374151;
      color: #d1d5db;
    }

    .row {
      border-bottom-color: #374151;
    }

    .row:hover {
      background-color: #111827;
    }

    .name {
      color: #d1d5db;
    }

    .col-new {
      color: #f9fafb;
    }

    .row-error {
      background-color: #3b1111;
    }

    .row-warning {
      background-color: #33290c;
    }

    .empty {
      border-color: #4b5563;
      color: #9ca3af;
    }

    .empty-title {
      color: #d1d5db;
    }
  }
</style>
