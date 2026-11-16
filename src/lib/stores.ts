// Reactive stores for managing application state
// These stores provide reactive state management for IngestRename
// Validates: Requirements 3.1.3, 3.4, 5.2, 9.4

import { writable, derived, type Writable, type Readable } from 'svelte/store';
import type {
  FileInfo,
  RenameConfig,
  PreviewResult,
  OperationStatus,
} from './types';
import { createDefaultConfig, hasNamePart } from './types';

// ============================================================================
// Core Writable Stores
// ============================================================================

/**
 * Store for selected files
 * Contains the list of files selected by the user for renaming
 * Validates: Requirements 1.2, 1.4
 */
export const selectedFiles: Writable<FileInfo[]> = writable<FileInfo[]>([]);

/**
 * Store for rename configuration
 * Contains the user's configuration for the batch rename operation
 * Validates: Requirements 3.1.3, 3.4
 */
export const config: Writable<RenameConfig> = writable<RenameConfig>(createDefaultConfig());

/**
 * Store for preview results
 * Contains the preview of how files will be renamed
 * Validates: Requirements 5.1, 5.2
 */
export const previewResults: Writable<PreviewResult[]> = writable<PreviewResult[]>([]);

/**
 * Store for operation status
 * Tracks the status of the batch rename operation
 * Validates: Requirements 9.4
 */
export const operationStatus: Writable<OperationStatus> = writable<OperationStatus>({
  state: 'idle',
  progress: 0,
  total: 0,
  results: [],
});

// ============================================================================
// Derived Stores
// ============================================================================

/**
 * Derived store: whether any files are selected
 * Validates: Requirements 1.2
 */
export const hasSelectedFiles: Readable<boolean> = derived(
  selectedFiles,
  ($selectedFiles) => $selectedFiles.length > 0
);

/**
 * Derived store: count of selected files
 * Validates: Requirements 1.2
 */
export const selectedFileCount: Readable<number> = derived(
  selectedFiles,
  ($selectedFiles) => $selectedFiles.length
);

/**
 * Derived store: whether the configuration is valid
 * Validates: Requirements 3.2, 3.3, 3.1.5
 */
export const isConfigValid: Readable<boolean> = derived(
  config,
  ($config) => {
    // Validate starting number
    if ($config.useNumber !== false && ($config.startingNumber < 1 || $config.startingNumber > 9999)) {
      return false;
    }
    
    // Validate prefix is not empty
    if ($config.usePrefix !== false && $config.prefix.length === 0) {
      return false;
    }
    
    // Basic validation - backend will do OS-specific validation
    const invalidChars = /[<>:"|?*\/\\]/;
    if ($config.usePrefix !== false && invalidChars.test($config.prefix)) {
      return false;
    }
    
    return hasNamePart($config);
  }
);

/**
 * Derived store: whether the preview has any errors
 * Validates: Requirements 5.3
 */
export const hasPreviewErrors: Readable<boolean> = derived(
  previewResults,
  ($previewResults) => $previewResults.some((result) => result.status === 'error')
);

/**
 * Derived store: whether the preview has any warnings
 * Validates: Requirements 5.3, 11.3
 */
export const hasPreviewWarnings: Readable<boolean> = derived(
  previewResults,
  ($previewResults) => $previewResults.some((result) => result.status === 'warning')
);

/**
 * Derived store: count of valid preview results
 * Validates: Requirements 5.1
 */
export const validPreviewCount: Readable<number> = derived(
  previewResults,
  ($previewResults) => $previewResults.filter((result) => result.status === 'valid').length
);

/**
 * Derived store: whether the operation can be executed
 * Requires: files selected, valid config, no preview errors, and operation not running
 * Validates: Requirements 5.4, 6.1
 */
export const canExecute: Readable<boolean> = derived(
  [hasSelectedFiles, isConfigValid, hasPreviewErrors, operationStatus],
  ([$hasSelectedFiles, $isConfigValid, $hasPreviewErrors, $operationStatus]) => {
    return (
      $hasSelectedFiles &&
      $isConfigValid &&
      !$hasPreviewErrors &&
      $operationStatus.state !== 'running'
    );
  }
);

/**
 * Derived store: whether the operation is currently running
 * Validates: Requirements 9.4
 */
export const isOperationRunning: Readable<boolean> = derived(
  operationStatus,
  ($operationStatus) => $operationStatus.state === 'running'
);

/**
 * Derived store: operation progress percentage (0-100)
 * Validates: Requirements 9.4
 */
export const operationProgress: Readable<number> = derived(
  operationStatus,
  ($operationStatus) => {
    if ($operationStatus.total === 0) return 0;
    return Math.round(($operationStatus.progress / $operationStatus.total) * 100);
  }
);

/**
 * Derived store: count of successful operations
 * Validates: Requirements 6.4
 */
export const successCount: Readable<number> = derived(
  operationStatus,
  ($operationStatus) => $operationStatus.results.filter((result) => result.success).length
);

/**
 * Derived store: count of failed operations
 * Validates: Requirements 7.4
 */
export const failureCount: Readable<number> = derived(
  operationStatus,
  ($operationStatus) => $operationStatus.results.filter((result) => !result.success).length
);

/**
 * Derived store: list of failed operations with error messages
 * Validates: Requirements 7.4
 */
export const failedOperations: Readable<Array<{ path: string; error: string }>> = derived(
  operationStatus,
  ($operationStatus) =>
    $operationStatus.results
      .filter((result) => !result.success)
      .map((result) => ({
        path: result.originalPath,
        error: result.error || 'Unknown error',
      }))
);

// ============================================================================
// Store Actions
// ============================================================================

/**
 * Reset all stores to their initial state
 * Useful for clearing the application state after an operation
 */
export function resetStores(): void {
  selectedFiles.set([]);
  config.set(createDefaultConfig());
  previewResults.set([]);
  operationStatus.set({
    state: 'idle',
    progress: 0,
    total: 0,
    results: [],
  });
}

/**
 * Reset only the operation status
 * Useful for starting a new operation without clearing file selection
 */
export function resetOperationStatus(): void {
  operationStatus.set({
    state: 'idle',
    progress: 0,
    total: 0,
    results: [],
  });
}

/**
 * Update operation progress
 * Called during batch rename to update progress
 * Validates: Requirements 9.4
 */
export function updateOperationProgress(progress: number, total: number): void {
  operationStatus.update((status) => ({
    ...status,
    progress,
    total,
  }));
}

/**
 * Set operation state
 * Validates: Requirements 9.4
 */
export function setOperationState(state: OperationStatus['state']): void {
  operationStatus.update((status) => ({
    ...status,
    state,
  }));
}
