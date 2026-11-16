// Unit tests for IPC wrapper functions
// Validates: Requirements 8.3

import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  getFileInfo,
  generatePreview,
  validateConfig,
  executeBatchRename,
  executeBatchRenameWithProgress,
  onRenameProgress,
  IPCError,
  type ValidationResult,
  type ProgressEvent,
} from './ipc';
import type { FileInfo, RenameConfig, PreviewResult, RenameResult } from './types';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Import mocked functions
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

describe('IPC Wrapper Functions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getFileInfo', () => {
    test('should call invoke with correct command and parameters', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
        { path: '/test/file2.txt', name: 'file2.txt', extension: 'txt', size: 2048 },
      ];

      vi.mocked(invoke).mockResolvedValue(mockFiles);

      const paths = ['/test/file1.txt', '/test/file2.txt'];
      const result = await getFileInfo(paths);

      expect(invoke).toHaveBeenCalledWith('get_file_info', { paths });
      expect(result).toEqual(mockFiles);
    });

    test('should throw IPCError when invoke fails', async () => {
      const errorMessage = 'File not found';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      const paths = ['/nonexistent/file.txt'];

      await expect(getFileInfo(paths)).rejects.toThrow(IPCError);
      await expect(getFileInfo(paths)).rejects.toThrow(errorMessage);
    });

    test('should wrap non-Error objects in IPCError', async () => {
      vi.mocked(invoke).mockRejectedValue('String error');

      await expect(getFileInfo(['/test/file.txt'])).rejects.toThrow(IPCError);
      await expect(getFileInfo(['/test/file.txt'])).rejects.toThrow('String error');
    });
  });

  describe('generatePreview', () => {
    test('should call invoke with correct command and parameters', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockPreview: PreviewResult[] = [
        {
          originalPath: '/test/file1.txt',
          originalName: 'file1.txt',
          newName: 'TEST_0001_file1.txt',
          newPath: '/test/TEST_0001_file1.txt',
          status: 'valid',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockPreview);

      const result = await generatePreview(mockFiles, mockConfig);

      expect(invoke).toHaveBeenCalledWith('generate_preview', {
        files: mockFiles,
        config: mockConfig,
      });
      expect(result).toEqual(mockPreview);
    });

    test('should throw IPCError when configuration is invalid', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 10000, // Invalid: exceeds 9999
        additionalText: undefined,
        operations: [],
      };

      vi.mocked(invoke).mockRejectedValue(new Error('Invalid configuration'));

      await expect(generatePreview(mockFiles, mockConfig)).rejects.toThrow(IPCError);
    });
  });

  describe('validateConfig', () => {
    test('should call invoke with correct command and parameters', async () => {
      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockValidation: ValidationResult = {
        valid: true,
        errors: [],
        warnings: [],
      };

      vi.mocked(invoke).mockResolvedValue(mockValidation);

      const result = await validateConfig(mockConfig);

      expect(invoke).toHaveBeenCalledWith('validate_config', { config: mockConfig });
      expect(result).toEqual(mockValidation);
    });

    test('should return validation errors for invalid config', async () => {
      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 0, // Invalid
        additionalText: undefined,
        operations: [],
      };

      const mockValidation: ValidationResult = {
        valid: false,
        errors: ['Starting number must be between 1 and 9999'],
        warnings: [],
      };

      vi.mocked(invoke).mockResolvedValue(mockValidation);

      const result = await validateConfig(mockConfig);

      expect(result.valid).toBe(false);
      expect(result.errors).toHaveLength(1);
    });
  });

  describe('executeBatchRename', () => {
    test('should call invoke with correct command and parameters', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockResults: RenameResult[] = [
        {
          originalPath: '/test/file1.txt',
          newPath: '/test/TEST_0001_file1.txt',
          success: true,
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockResults);

      const result = await executeBatchRename(mockFiles, mockConfig);

      expect(invoke).toHaveBeenCalledWith('execute_batch_rename', {
        files: mockFiles,
        config: mockConfig,
        overwrite: [],
      });
      expect(result).toEqual(mockResults);
    });

    test('should handle partial failures', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
        { path: '/test/file2.txt', name: 'file2.txt', extension: 'txt', size: 2048 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockResults: RenameResult[] = [
        {
          originalPath: '/test/file1.txt',
          newPath: '/test/TEST_0001_file1.txt',
          success: true,
        },
        {
          originalPath: '/test/file2.txt',
          newPath: '/test/TEST_0002_file2.txt',
          success: false,
          error: 'Permission denied',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockResults);

      const result = await executeBatchRename(mockFiles, mockConfig);

      expect(result).toHaveLength(2);
      expect(result[0].success).toBe(true);
      expect(result[1].success).toBe(false);
      expect(result[1].error).toBe('Permission denied');
    });
  });

  describe('onRenameProgress', () => {
    test('should set up event listener for progress events', async () => {
      const mockUnlisten = vi.fn();
      vi.mocked(listen).mockResolvedValue(mockUnlisten);

      const callback = vi.fn();
      const unlisten = await onRenameProgress(callback);

      expect(listen).toHaveBeenCalledWith('rename-progress', expect.any(Function));
      expect(unlisten).toBe(mockUnlisten);
    });

    test('should call callback with progress event payload', async () => {
      const mockEvent: ProgressEvent = {
        current: 5,
        total: 10,
        file_name: 'test.txt',
      };

      let capturedCallback: ((event: { payload: ProgressEvent }) => void) | undefined;

      vi.mocked(listen).mockImplementation(async (_event, callback) => {
        capturedCallback = callback as (event: { payload: ProgressEvent }) => void;
        return vi.fn() as UnlistenFn;
      });

      const callback = vi.fn();
      await onRenameProgress(callback);

      // Simulate event emission
      if (capturedCallback) {
        capturedCallback({ payload: mockEvent });
      }

      expect(callback).toHaveBeenCalledWith(mockEvent);
    });
  });

  describe('executeBatchRenameWithProgress', () => {
    test('should execute rename and track progress', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockResults: RenameResult[] = [
        {
          originalPath: '/test/file1.txt',
          newPath: '/test/TEST_0001_file1.txt',
          success: true,
        },
      ];

      const mockUnlisten = vi.fn();
      vi.mocked(listen).mockResolvedValue(mockUnlisten);
      vi.mocked(invoke).mockResolvedValue(mockResults);

      const progressCallback = vi.fn();
      const result = await executeBatchRenameWithProgress(
        mockFiles,
        mockConfig,
        progressCallback
      );

      expect(listen).toHaveBeenCalledWith('rename-progress', expect.any(Function));
      expect(invoke).toHaveBeenCalledWith('execute_batch_rename', {
        files: mockFiles,
        config: mockConfig,
        overwrite: [],
      });
      expect(result).toEqual(mockResults);
      expect(mockUnlisten).toHaveBeenCalled();
    });

    test('should work without progress callback', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockResults: RenameResult[] = [
        {
          originalPath: '/test/file1.txt',
          newPath: '/test/TEST_0001_file1.txt',
          success: true,
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockResults);

      const result = await executeBatchRenameWithProgress(mockFiles, mockConfig);

      expect(listen).not.toHaveBeenCalled();
      expect(result).toEqual(mockResults);
    });

    test('should clean up listener even if rename fails', async () => {
      const mockFiles: FileInfo[] = [
        { path: '/test/file1.txt', name: 'file1.txt', extension: 'txt', size: 1024 },
      ];

      const mockConfig: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 1,
        additionalText: undefined,
        operations: [],
      };

      const mockUnlisten = vi.fn();
      vi.mocked(listen).mockResolvedValue(mockUnlisten);
      vi.mocked(invoke).mockRejectedValue(new Error('Rename failed'));

      const progressCallback = vi.fn();

      await expect(
        executeBatchRenameWithProgress(mockFiles, mockConfig, progressCallback)
      ).rejects.toThrow(IPCError);

      expect(mockUnlisten).toHaveBeenCalled();
    });
  });

  describe('IPCError', () => {
    test('should create error with command context', () => {
      const error = new IPCError('Test error', 'test_command');

      expect(error.message).toBe('Test error');
      expect(error.command).toBe('test_command');
      expect(error.name).toBe('IPCError');
    });

    test('should include cause if provided', () => {
      const cause = new Error('Original error');
      const error = new IPCError('Test error', 'test_command', cause);

      expect(error.cause).toBe(cause);
    });
  });
});

