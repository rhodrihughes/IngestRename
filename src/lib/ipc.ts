// IPC wrapper functions for Tauri commands
// Provides type-safe access to backend commands with error handling
// Validates: Requirements 5.2, 9.4

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  FileInfo,
  PreviewResult,
  RenameConfig,
  RenameResult,
  UndoRecord,
  BatchRenameResponse,
  Preset,
  AppSettings,
} from './types';

// ============================================================================
// Error Handling
// ============================================================================

/**
 * Custom error class for IPC errors
 */
export class IPCError extends Error {
  constructor(
    message: string,
    public readonly command: string,
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = 'IPCError';
  }
}

/**
 * Wraps IPC errors with context
 */
function wrapIPCError(command: string, error: unknown): IPCError {
  if (error instanceof Error) {
    return new IPCError(error.message, command, error);
  }
  return new IPCError(String(error), command, error);
}

// ============================================================================
// Validation Result Type
// ============================================================================

/**
 * Result of configuration validation from backend
 */
export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

// ============================================================================
// Progress Event Type
// ============================================================================

/**
 * Progress event payload for batch rename operations
 * Validates: Requirements 9.4
 */
export interface ProgressEvent {
  current: number;
  total: number;
  file_name: string;
}

// ============================================================================
// IPC Command Wrappers
// ============================================================================

/**
 * Get file information for a list of file paths
 * 
 * Validates: Requirements 1.1, 1.2
 * 
 * @param paths - List of file paths to get information for
 * @returns Promise resolving to array of file metadata
 * @throws IPCError if the command fails
 */
export async function getFileInfo(paths: string[]): Promise<FileInfo[]> {
  try {
    const result = await invoke<FileInfo[]>('get_file_info', { paths });
    return result;
  } catch (error) {
    throw wrapIPCError('get_file_info', error);
  }
}

/**
 * Generate preview of rename operation
 * 
 * Validates: Requirements 5.1, 5.2
 * 
 * @param files - List of files to preview
 * @param config - Rename configuration
 * @returns Promise resolving to array of preview results
 * @throws IPCError if the command fails or configuration is invalid
 */
export async function generatePreview(
  files: FileInfo[],
  config: RenameConfig
): Promise<PreviewResult[]> {
  try {
    const result = await invoke<PreviewResult[]>('generate_preview', {
      files,
      config,
    });
    return result;
  } catch (error) {
    throw wrapIPCError('generate_preview', error);
  }
}

/**
 * Validate rename configuration
 * 
 * Validates: Requirements 3.1.5, 3.1.6, 3.2, 3.3
 * 
 * @param config - Rename configuration to validate
 * @returns Promise resolving to validation result
 * @throws IPCError if the command fails
 */
export async function validateConfig(
  config: RenameConfig
): Promise<ValidationResult> {
  try {
    const result = await invoke<ValidationResult>('validate_config', {
      config,
    });
    return result;
  } catch (error) {
    throw wrapIPCError('validate_config', error);
  }
}

/**
 * Execute batch rename operation
 * 
 * Validates: Requirements 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4, 9.4
 * 
 * @param files - List of files to rename
 * @param config - Rename configuration
 * @param overwrite - Original paths of files allowed to replace an existing file
 * @returns Promise resolving to array of rename results
 * @throws IPCError if the command fails or configuration is invalid
 */
export async function executeBatchRename(
  files: FileInfo[],
  config: RenameConfig,
  overwrite: string[] = []
): Promise<BatchRenameResponse> {
  try {
    const result = await invoke<BatchRenameResponse>('execute_batch_rename', {
      files,
      config,
      overwrite,
    });
    return result;
  } catch (error) {
    throw wrapIPCError('execute_batch_rename', error);
  }
}

// ============================================================================
// Event Listener Utilities
// ============================================================================

/**
 * Listen for progress updates during batch rename operations
 * 
 * Validates: Requirements 9.4
 * 
 * @param callback - Function to call when progress event is received
 * @returns Promise resolving to unlisten function
 * 
 * @example
 * ```typescript
 * const unlisten = await onRenameProgress((event) => {
 *   console.log(`Processing ${event.current}/${event.total}: ${event.file_name}`);
 * });
 * 
 * // Later, to stop listening:
 * unlisten();
 * ```
 */
export async function onRenameProgress(
  callback: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await listen<ProgressEvent>('rename-progress', (event) => {
    callback(event.payload);
  });
}

/**
 * Helper to execute batch rename with progress tracking
 * 
 * Validates: Requirements 6.1, 9.4
 * 
 * @param files - List of files to rename
 * @param config - Rename configuration
 * @param onProgress - Optional callback for progress updates
 * @param overwrite - Original paths of files allowed to replace an existing file
 * @returns Promise resolving to array of rename results
 * @throws IPCError if the command fails or configuration is invalid
 * 
 * @example
 * ```typescript
 * const results = await executeBatchRenameWithProgress(
 *   files,
 *   config,
 *   (event) => {
 *     updateProgressBar(event.current, event.total);
 *   }
 * );
 * ```
 */
export async function executeBatchRenameWithProgress(
  files: FileInfo[],
  config: RenameConfig,
  onProgress?: (event: ProgressEvent) => void,
  overwrite: string[] = []
): Promise<BatchRenameResponse> {
  let unlisten: UnlistenFn | undefined;
  
  try {
    if (onProgress) {
      unlisten = await onRenameProgress(onProgress);
    }
    const result = await executeBatchRename(files, config, overwrite);
    return result;
  } finally {
    if (unlisten) {
      unlisten();
    }
  }
}

export async function undoBatchRename(record: UndoRecord): Promise<RenameResult[]> {
  try {
    return await invoke<RenameResult[]>('undo_batch_rename', { record });
  } catch (error) {
    throw wrapIPCError('undo_batch_rename', error);
  }
}

export async function listPresets(): Promise<Preset[]> {
  try {
    return await invoke<Preset[]>('list_presets');
  } catch (error) {
    throw wrapIPCError('list_presets', error);
  }
}

export async function savePreset(preset: Preset): Promise<void> {
  try {
    await invoke('save_preset', { preset });
  } catch (error) {
    throw wrapIPCError('save_preset', error);
  }
}

export async function deletePreset(name: string): Promise<void> {
  try {
    await invoke('delete_preset', { name });
  } catch (error) {
    throw wrapIPCError('delete_preset', error);
  }
}

export async function getSettings(): Promise<AppSettings> {
  try {
    return await invoke<AppSettings>('get_settings');
  } catch (error) {
    throw wrapIPCError('get_settings', error);
  }
}

export async function setDefaultPreset(name: string | null): Promise<void> {
  try {
    await invoke('set_default_preset', { name });
  } catch (error) {
    throw wrapIPCError('set_default_preset', error);
  }
}

export async function saveLastConfig(config: RenameConfig): Promise<void> {
  try {
    await invoke('save_last_config', { config });
  } catch (error) {
    throw wrapIPCError('save_last_config', error);
  }
}
