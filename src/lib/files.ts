// Helpers for building and ordering the list of files to rename

import { getFileInfo } from '$lib/ipc';
import type { FileInfo } from '$lib/types';

export type SortMode = 'selection' | 'name-asc' | 'name-desc';

/**
 * Look up info for `paths` and append any files not already in `existing`.
 */
export async function mergeFiles(existing: FileInfo[], paths: string[]): Promise<FileInfo[]> {
  const fileInfos = await getFileInfo(paths);
  const known = new Set(existing.map((f) => f.path));
  return [...existing, ...fileInfos.filter((f) => !known.has(f.path))];
}

/**
 * Return a copy of `files` ordered by `mode`. "selection" keeps the order
 * files were added in. Names compare naturally (file2 before file10).
 */
export function sortFiles(files: FileInfo[], mode: SortMode): FileInfo[] {
  if (mode === 'selection') return files;
  const dir = mode === 'name-asc' ? 1 : -1;
  return [...files].sort(
    (a, b) => dir * a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' })
  );
}

/**
 * Split `newName` into the unchanged original part and the parts added around
 * it, so the UI can highlight what changed. When the original name is not
 * found intact, the whole new name counts as changed.
 */
export function diffName(
  originalName: string,
  newName: string
): { before: string; kept: string; after: string } {
  const idx = originalName ? newName.indexOf(originalName) : -1;
  if (idx === -1) return { before: newName, kept: '', after: '' };
  return {
    before: newName.slice(0, idx),
    kept: originalName,
    after: newName.slice(idx + originalName.length),
  };
}
