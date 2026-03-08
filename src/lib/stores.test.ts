// Unit tests for reactive stores
// Validates: Requirements 3.1.3, 3.4, 5.2, 9.4

import { describe, test, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  selectedFiles,
  config,
  previewResults,
  operationStatus,
  hasSelectedFiles,
  selectedFileCount,
  isConfigValid,
  hasPreviewErrors,
  hasPreviewWarnings,
  validPreviewCount,
  canExecute,
  isOperationRunning,
  operationProgress,
  successCount,
  failureCount,
  failedOperations,
  resetStores,
  resetOperationStatus,
  updateOperationProgress,
  setOperationState,
} from './stores';
import type { FileInfo, PreviewResult, RenameResult } from './types';

describe('Core Writable Stores', () => {
  beforeEach(() => {
    resetStores();
  });

  test('selectedFiles initializes as empty array', () => {
    expect(get(selectedFiles)).toEqual([]);
  });

  test('config initializes with default values', () => {
    const defaultConfig = get(config);
    expect(defaultConfig.prefix).toBe('BFR');
    expect(defaultConfig.startingNumber).toBe(1);
    expect(defaultConfig.additionalText).toBeUndefined();
    expect(defaultConfig.operations).toEqual([]);
  });

  test('previewResults initializes as empty array', () => {
    expect(get(previewResults)).toEqual([]);
  });

  test('operationStatus initializes with idle state', () => {
    const status = get(operationStatus);
    expect(status.state).toBe('idle');
    expect(status.progress).toBe(0);
    expect(status.total).toBe(0);
    expect(status.results).toEqual([]);
  });
});

describe('selectedFiles store', () => {
  beforeEach(() => {
    resetStores();
  });

  test('can update selectedFiles', () => {
    const files: FileInfo[] = [
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
      { path: '/test/file2.txt', name: 'file2.txt', extension: 'txt', size: 200 },
    ];
    
    selectedFiles.set(files);
    expect(get(selectedFiles)).toEqual(files);
  });
});

describe('config store', () => {
  beforeEach(() => {
    resetStores();
  });

  test('can update config prefix', () => {
    config.update((c) => ({ ...c, prefix: 'TEST' }));
    expect(get(config).prefix).toBe('TEST');
  });

  test('can update config starting number', () => {
    config.update((c) => ({ ...c, startingNumber: 100 }));
    expect(get(config).startingNumber).toBe(100);
  });

  test('can update config additional text', () => {
    config.update((c) => ({ ...c, additionalText: 'Scene1' }));
    expect(get(config).additionalText).toBe('Scene1');
  });
});

describe('Derived Store: hasSelectedFiles', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns false when no files selected', () => {
    expect(get(hasSelectedFiles)).toBe(false);
  });

  test('returns true when files are selected', () => {
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    expect(get(hasSelectedFiles)).toBe(true);
  });
});

describe('Derived Store: selectedFileCount', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns 0 when no files selected', () => {
    expect(get(selectedFileCount)).toBe(0);
  });

  test('returns correct count when files are selected', () => {
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
      { path: '/test/file2.txt', name: 'file2.txt', extension: 'txt', size: 200 },
      { path: '/test/file3.txt', name: 'file3.txt', extension: 'txt', size: 300 },
    ]);
    expect(get(selectedFileCount)).toBe(3);
  });
});

describe('Derived Store: isConfigValid', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns true for default config', () => {
    expect(get(isConfigValid)).toBe(true);
  });

  test('returns false for starting number below 1', () => {
    config.update((c) => ({ ...c, startingNumber: 0 }));
    expect(get(isConfigValid)).toBe(false);
  });

  test('returns false for starting number above 9999', () => {
    config.update((c) => ({ ...c, startingNumber: 10000 }));
    expect(get(isConfigValid)).toBe(false);
  });

  test('returns false for empty prefix', () => {
    config.update((c) => ({ ...c, prefix: '' }));
    expect(get(isConfigValid)).toBe(false);
  });

  test('returns false for prefix with invalid characters', () => {
    config.update((c) => ({ ...c, prefix: 'test<>file' }));
    expect(get(isConfigValid)).toBe(false);
  });

  test('returns true for valid custom config', () => {
    config.update((c) => ({ ...c, prefix: 'PROJECT', startingNumber: 500 }));
    expect(get(isConfigValid)).toBe(true);
  });
});

describe('Derived Store: hasPreviewErrors', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns false when no preview results', () => {
    expect(get(hasPreviewErrors)).toBe(false);
  });

  test('returns false when all results are valid', () => {
    const results: PreviewResult[] = [
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
    ];
    previewResults.set(results);
    expect(get(hasPreviewErrors)).toBe(false);
  });

  test('returns true when any result has error status', () => {
    const results: PreviewResult[] = [
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
      {
        originalPath: '/test/file2.txt',
        originalName: 'file2.txt',
        newName: 'BFR_0002_file2.txt',
        newPath: '/test/BFR_0002_file2.txt',
        status: 'error',
        message: 'File already exists',
      },
    ];
    previewResults.set(results);
    expect(get(hasPreviewErrors)).toBe(true);
  });
});

describe('Derived Store: hasPreviewWarnings', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns false when no preview results', () => {
    expect(get(hasPreviewWarnings)).toBe(false);
  });

  test('returns false when all results are valid', () => {
    const results: PreviewResult[] = [
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
    ];
    previewResults.set(results);
    expect(get(hasPreviewWarnings)).toBe(false);
  });

  test('returns true when any result has warning status', () => {
    const results: PreviewResult[] = [
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
      {
        originalPath: '/test/file2.txt',
        originalName: 'file2.txt',
        newName: 'BFR_0002_file2.txt',
        newPath: '/test/BFR_0002_file2.txt',
        status: 'warning',
        message: 'Path exceeds 260 characters',
      },
    ];
    previewResults.set(results);
    expect(get(hasPreviewWarnings)).toBe(true);
  });
});

describe('Derived Store: validPreviewCount', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns 0 when no preview results', () => {
    expect(get(validPreviewCount)).toBe(0);
  });

  test('returns correct count of valid results', () => {
    const results: PreviewResult[] = [
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
      {
        originalPath: '/test/file2.txt',
        originalName: 'file2.txt',
        newName: 'BFR_0002_file2.txt',
        newPath: '/test/BFR_0002_file2.txt',
        status: 'warning',
        message: 'Path exceeds 260 characters',
      },
      {
        originalPath: '/test/file3.txt',
        originalName: 'file3.txt',
        newName: 'BFR_0003_file3.txt',
        newPath: '/test/BFR_0003_file3.txt',
        status: 'valid',
      },
    ];
    previewResults.set(results);
    expect(get(validPreviewCount)).toBe(2);
  });
});

describe('Derived Store: canExecute', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns false when no files selected', () => {
    expect(get(canExecute)).toBe(false);
  });

  test('returns false when config is invalid', () => {
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    config.update((c) => ({ ...c, startingNumber: 0 }));
    expect(get(canExecute)).toBe(false);
  });

  test('returns false when preview has errors', () => {
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    previewResults.set([
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'error',
        message: 'File already exists',
      },
    ]);
    expect(get(canExecute)).toBe(false);
  });

  test('returns false when operation is running', () => {
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    previewResults.set([
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
    ]);
    operationStatus.update((s) => ({ ...s, state: 'running' }));
    expect(get(canExecute)).toBe(false);
  });

  test('returns true when all conditions are met', () => {
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    previewResults.set([
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'BFR_0001_file1.txt',
        newPath: '/test/BFR_0001_file1.txt',
        status: 'valid',
      },
    ]);
    expect(get(canExecute)).toBe(true);
  });
});

describe('Derived Store: isOperationRunning', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns false when operation is idle', () => {
    expect(get(isOperationRunning)).toBe(false);
  });

  test('returns true when operation is running', () => {
    operationStatus.update((s) => ({ ...s, state: 'running' }));
    expect(get(isOperationRunning)).toBe(true);
  });

  test('returns false when operation is completed', () => {
    operationStatus.update((s) => ({ ...s, state: 'completed' }));
    expect(get(isOperationRunning)).toBe(false);
  });
});

describe('Derived Store: operationProgress', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns 0 when total is 0', () => {
    expect(get(operationProgress)).toBe(0);
  });

  test('returns correct percentage', () => {
    operationStatus.update((s) => ({ ...s, progress: 5, total: 10 }));
    expect(get(operationProgress)).toBe(50);
  });

  test('returns 100 when progress equals total', () => {
    operationStatus.update((s) => ({ ...s, progress: 10, total: 10 }));
    expect(get(operationProgress)).toBe(100);
  });
});

describe('Derived Store: successCount', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns 0 when no results', () => {
    expect(get(successCount)).toBe(0);
  });

  test('returns correct count of successful operations', () => {
    const results: RenameResult[] = [
      { originalPath: '/test/file1.txt', newPath: '/test/BFR_0001_file1.txt', success: true },
      { originalPath: '/test/file2.txt', newPath: '/test/BFR_0002_file2.txt', success: false, error: 'Permission denied' },
      { originalPath: '/test/file3.txt', newPath: '/test/BFR_0003_file3.txt', success: true },
    ];
    operationStatus.update((s) => ({ ...s, results }));
    expect(get(successCount)).toBe(2);
  });
});

describe('Derived Store: failureCount', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns 0 when no results', () => {
    expect(get(failureCount)).toBe(0);
  });

  test('returns correct count of failed operations', () => {
    const results: RenameResult[] = [
      { originalPath: '/test/file1.txt', newPath: '/test/BFR_0001_file1.txt', success: true },
      { originalPath: '/test/file2.txt', newPath: '/test/BFR_0002_file2.txt', success: false, error: 'Permission denied' },
      { originalPath: '/test/file3.txt', newPath: '/test/BFR_0003_file3.txt', success: false, error: 'File exists' },
    ];
    operationStatus.update((s) => ({ ...s, results }));
    expect(get(failureCount)).toBe(2);
  });
});

describe('Derived Store: failedOperations', () => {
  beforeEach(() => {
    resetStores();
  });

  test('returns empty array when no results', () => {
    expect(get(failedOperations)).toEqual([]);
  });

  test('returns only failed operations with error messages', () => {
    const results: RenameResult[] = [
      { originalPath: '/test/file1.txt', newPath: '/test/BFR_0001_file1.txt', success: true },
      { originalPath: '/test/file2.txt', newPath: '/test/BFR_0002_file2.txt', success: false, error: 'Permission denied' },
      { originalPath: '/test/file3.txt', newPath: '/test/BFR_0003_file3.txt', success: false, error: 'File exists' },
    ];
    operationStatus.update((s) => ({ ...s, results }));
    
    const failed = get(failedOperations);
    expect(failed).toHaveLength(2);
    expect(failed[0]).toEqual({ path: '/test/file2.txt', error: 'Permission denied' });
    expect(failed[1]).toEqual({ path: '/test/file3.txt', error: 'File exists' });
  });

  test('handles missing error message', () => {
    const results: RenameResult[] = [
      { originalPath: '/test/file1.txt', newPath: '/test/BFR_0001_file1.txt', success: false },
    ];
    operationStatus.update((s) => ({ ...s, results }));
    
    const failed = get(failedOperations);
    expect(failed).toHaveLength(1);
    expect(failed[0]).toEqual({ path: '/test/file1.txt', error: 'Unknown error' });
  });
});

describe('Store Actions', () => {
  test('resetStores clears all stores', () => {
    // Set some data
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    config.update((c) => ({ ...c, prefix: 'TEST' }));
    previewResults.set([
      {
        originalPath: '/test/file1.txt',
        originalName: 'file1.txt',
        newName: 'TEST_0001_file1.txt',
        newPath: '/test/TEST_0001_file1.txt',
        status: 'valid',
      },
    ]);
    operationStatus.update((s) => ({ ...s, state: 'completed' }));

    // Reset
    resetStores();

    // Verify all stores are reset
    expect(get(selectedFiles)).toEqual([]);
    expect(get(config).prefix).toBe('BFR');
    expect(get(config).startingNumber).toBe(1);
    expect(get(previewResults)).toEqual([]);
    expect(get(operationStatus).state).toBe('idle');
  });

  test('resetOperationStatus clears only operation status', () => {
    // Set some data
    selectedFiles.set([
      { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 100 },
    ]);
    operationStatus.update((s) => ({ ...s, state: 'completed', progress: 10, total: 10 }));

    // Reset operation status
    resetOperationStatus();

    // Verify only operation status is reset
    expect(get(selectedFiles)).toHaveLength(1);
    expect(get(operationStatus).state).toBe('idle');
    expect(get(operationStatus).progress).toBe(0);
    expect(get(operationStatus).total).toBe(0);
  });

  test('updateOperationProgress updates progress and total', () => {
    updateOperationProgress(5, 10);
    
    const status = get(operationStatus);
    expect(status.progress).toBe(5);
    expect(status.total).toBe(10);
  });

  test('setOperationState updates state', () => {
    setOperationState('running');
    expect(get(operationStatus).state).toBe('running');

    setOperationState('completed');
    expect(get(operationStatus).state).toBe('completed');
  });
});
