// Unit tests for type definitions, type guards, and validation utilities
// Validates: Requirements 1.1, 3.1.1, 3.1.2, 3.2, 3.3, 4.1, 9.4

import { describe, it, expect } from 'vitest';
import {
  type RenameConfig,
  type FileInfo,
  type PreviewResult,
  type RenameResult,
  type OperationStatus,
  type RenameOperation,
  type FormatType,
  type TextPosition,
  type PreviewStatus,
  type OperationState,
  isTextPosition,
  isPreviewStatus,
  isOperationState,
  isRenameOperation,
  isFormatType,
  isRenameConfig,
  isFileInfo,
  isPreviewResult,
  isRenameResult,
  isOperationStatus,
  validateStartingNumber,
  validatePrefix,
  validateRenameConfig,
  createDefaultConfig
} from './types';

// ============================================================================
// Type Guard Tests
// ============================================================================

describe('Type Guards', () => {
  describe('isTextPosition', () => {
    it('should accept valid text positions', () => {
      expect(isTextPosition('before')).toBe(true);
      expect(isTextPosition('after')).toBe(true);
    });

    it('should reject invalid text positions', () => {
      expect(isTextPosition('middle')).toBe(false);
      expect(isTextPosition('')).toBe(false);
      expect(isTextPosition(null)).toBe(false);
      expect(isTextPosition(undefined)).toBe(false);
      expect(isTextPosition(123)).toBe(false);
    });
  });

  describe('isPreviewStatus', () => {
    it('should accept valid preview statuses', () => {
      expect(isPreviewStatus('valid')).toBe(true);
      expect(isPreviewStatus('warning')).toBe(true);
      expect(isPreviewStatus('error')).toBe(true);
    });

    it('should reject invalid preview statuses', () => {
      expect(isPreviewStatus('success')).toBe(false);
      expect(isPreviewStatus('')).toBe(false);
      expect(isPreviewStatus(null)).toBe(false);
      expect(isPreviewStatus(undefined)).toBe(false);
    });
  });

  describe('isOperationState', () => {
    it('should accept valid operation states', () => {
      expect(isOperationState('idle')).toBe(true);
      expect(isOperationState('running')).toBe(true);
      expect(isOperationState('completed')).toBe(true);
      expect(isOperationState('error')).toBe(true);
    });

    it('should reject invalid operation states', () => {
      expect(isOperationState('pending')).toBe(false);
      expect(isOperationState('')).toBe(false);
      expect(isOperationState(null)).toBe(false);
    });
  });

  describe('isFormatType', () => {
    it('should accept valid sequential format', () => {
      expect(isFormatType({ type: 'sequential', start: 1, increment: 1 })).toBe(true);
      expect(isFormatType({ type: 'sequential', start: 10, increment: 5 })).toBe(true);
    });

    it('should accept valid date format', () => {
      expect(isFormatType({ type: 'date', format: '%Y-%m-%d' })).toBe(true);
      expect(isFormatType({ type: 'date', format: '%Y%m%d' })).toBe(true);
    });

    it('should reject invalid format types', () => {
      expect(isFormatType({ type: 'sequential', start: '1', increment: 1 })).toBe(false);
      expect(isFormatType({ type: 'sequential', start: 1 })).toBe(false);
      expect(isFormatType({ type: 'date', format: 123 })).toBe(false);
      expect(isFormatType({ type: 'unknown' })).toBe(false);
      expect(isFormatType(null)).toBe(false);
      expect(isFormatType('sequential')).toBe(false);
    });
  });

  describe('isRenameOperation', () => {
    it('should accept valid find-replace operation', () => {
      expect(isRenameOperation({ type: 'find-replace', find: 'old', replace: 'new' })).toBe(true);
    });

    it('should accept valid add-text operation', () => {
      expect(isRenameOperation({ type: 'add-text', text: 'prefix', position: 'before' })).toBe(true);
      expect(isRenameOperation({ type: 'add-text', text: 'suffix', position: 'after' })).toBe(true);
    });

    it('should accept valid format operation', () => {
      expect(isRenameOperation({
        type: 'format',
        format: { type: 'sequential', start: 1, increment: 1 },
        position: 'before',
        separator: 'none'
      })).toBe(true);
      
      expect(isRenameOperation({
        type: 'format',
        format: { type: 'date', format: '%Y-%m-%d' },
        position: 'after',
        separator: '_'
      })).toBe(true);
    });

    it('should reject invalid operations', () => {
      expect(isRenameOperation({ type: 'find-replace', find: 'old' })).toBe(false);
      expect(isRenameOperation({ type: 'add-text', text: 'prefix', position: 'middle' })).toBe(false);
      expect(isRenameOperation({ type: 'unknown' })).toBe(false);
      expect(isRenameOperation(null)).toBe(false);
      expect(isRenameOperation('operation')).toBe(false);
    });
  });

  describe('isRenameConfig', () => {
    it('should accept valid rename config', () => {
      const config: RenameConfig = {
        prefix: 'BFR',
        startingNumber: 1,
        additionalText: undefined,
        operations: []
      };
      expect(isRenameConfig(config)).toBe(true);
    });

    it('should accept config with additional text', () => {
      const config: RenameConfig = {
        prefix: 'TEST',
        startingNumber: 42,
        additionalText: 'Scene1',
        operations: []
      };
      expect(isRenameConfig(config)).toBe(true);
    });

    it('should accept config with operations', () => {
      const config: RenameConfig = {
        prefix: 'BFR',
        startingNumber: 1,
        operations: [
          { type: 'find-replace', find: 'old', replace: 'new' },
          { type: 'add-text', text: 'prefix', position: 'before' }
        ]
      };
      expect(isRenameConfig(config)).toBe(true);
    });

    it('should reject invalid configs', () => {
      expect(isRenameConfig({ prefix: 'BFR', startingNumber: '1', operations: [] })).toBe(false);
      expect(isRenameConfig({ prefix: 123, startingNumber: 1, operations: [] })).toBe(false);
      expect(isRenameConfig({ prefix: 'BFR', startingNumber: 1 })).toBe(false);
      expect(isRenameConfig(null)).toBe(false);
      expect(isRenameConfig('config')).toBe(false);
    });
  });

  describe('isFileInfo', () => {
    it('should accept valid file info', () => {
      const file: FileInfo = {
        path: '/test/file.txt',
        name: 'file.txt',
        extension: 'txt',
        size: 1024
      };
      expect(isFileInfo(file)).toBe(true);
    });

    it('should reject invalid file info', () => {
      expect(isFileInfo({ path: '/test/file.txt', name: 'file.txt', extension: 'txt' })).toBe(false);
      expect(isFileInfo({ path: '/test/file.txt', name: 'file.txt', extension: 'txt', size: '1024' })).toBe(false);
      expect(isFileInfo(null)).toBe(false);
    });
  });

  describe('isPreviewResult', () => {
    it('should accept valid preview result', () => {
      const result: PreviewResult = {
        originalPath: '/test/old.txt',
        originalName: 'old.txt',
        newName: 'BFR_0001_old.txt',
        newPath: '/test/BFR_0001_old.txt',
        status: 'valid'
      };
      expect(isPreviewResult(result)).toBe(true);
    });

    it('should accept preview result with message', () => {
      const result: PreviewResult = {
        originalPath: '/test/old.txt',
        originalName: 'old.txt',
        newName: 'BFR_0001_old.txt',
        newPath: '/test/BFR_0001_old.txt',
        status: 'warning',
        message: 'Path exceeds 260 characters'
      };
      expect(isPreviewResult(result)).toBe(true);
    });

    it('should reject invalid preview results', () => {
      expect(isPreviewResult({ originalPath: '/test/old.txt', originalName: 'old.txt' })).toBe(false);
      expect(isPreviewResult(null)).toBe(false);
    });
  });

  describe('isRenameResult', () => {
    it('should accept valid rename result', () => {
      const result: RenameResult = {
        originalPath: '/test/old.txt',
        newPath: '/test/new.txt',
        success: true
      };
      expect(isRenameResult(result)).toBe(true);
    });

    it('should accept rename result with error', () => {
      const result: RenameResult = {
        originalPath: '/test/old.txt',
        newPath: '/test/new.txt',
        success: false,
        error: 'File already exists'
      };
      expect(isRenameResult(result)).toBe(true);
    });

    it('should reject invalid rename results', () => {
      expect(isRenameResult({ originalPath: '/test/old.txt', newPath: '/test/new.txt' })).toBe(false);
      expect(isRenameResult(null)).toBe(false);
    });
  });

  describe('isOperationStatus', () => {
    it('should accept valid operation status', () => {
      const status: OperationStatus = {
        state: 'idle',
        progress: 0,
        total: 0,
        results: []
      };
      expect(isOperationStatus(status)).toBe(true);
    });

    it('should accept operation status with results', () => {
      const status: OperationStatus = {
        state: 'completed',
        progress: 2,
        total: 2,
        results: [
          { originalPath: '/test/old1.txt', newPath: '/test/new1.txt', success: true },
          { originalPath: '/test/old2.txt', newPath: '/test/new2.txt', success: true }
        ]
      };
      expect(isOperationStatus(status)).toBe(true);
    });

    it('should reject invalid operation status', () => {
      expect(isOperationStatus({ state: 'idle', progress: 0, total: 0 })).toBe(false);
      expect(isOperationStatus({ state: 'unknown', progress: 0, total: 0, results: [] })).toBe(false);
      expect(isOperationStatus(null)).toBe(false);
    });
  });
});

// ============================================================================
// Validation Utility Tests
// ============================================================================

describe('Validation Utilities', () => {
  describe('validateStartingNumber', () => {
    it('should accept valid starting numbers', () => {
      expect(validateStartingNumber(1)).toEqual({ valid: true });
      expect(validateStartingNumber(100)).toEqual({ valid: true });
      expect(validateStartingNumber(9999)).toEqual({ valid: true });
    });

    it('should reject numbers below 1', () => {
      const result = validateStartingNumber(0);
      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
    });

    it('should reject numbers above 9999', () => {
      const result = validateStartingNumber(10000);
      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
    });

    it('should reject negative numbers', () => {
      const result = validateStartingNumber(-1);
      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
    });

    it('should reject non-integers', () => {
      const result = validateStartingNumber(1.5);
      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
    });
  });

  describe('validatePrefix', () => {
    it('should accept valid prefixes', () => {
      expect(validatePrefix('BB')).toEqual({ valid: true });
      expect(validatePrefix('KCOC4')).toEqual({ valid: true });
      expect(validatePrefix('ProjectX')).toEqual({ valid: true });
      expect(validatePrefix('Test-123')).toEqual({ valid: true });
    });

    it('should reject empty prefix', () => {
      const result = validatePrefix('');
      expect(result.valid).toBe(false);
      expect(result.error).toBeDefined();
    });

    it('should reject prefixes with invalid characters', () => {
      const invalidPrefixes = ['Test<', 'Test>', 'Test:', 'Test"', 'Test|', 'Test?', 'Test*', 'Test/', 'Test\\'];
      
      for (const prefix of invalidPrefixes) {
        const result = validatePrefix(prefix);
        expect(result.valid).toBe(false);
        expect(result.error).toBeDefined();
      }
    });
  });

  describe('validateRenameConfig', () => {
    it('should accept valid config', () => {
      const config: RenameConfig = {
        prefix: 'BFR',
        startingNumber: 1,
        operations: []
      };
      
      const result = validateRenameConfig(config);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should reject config with invalid prefix', () => {
      const config: RenameConfig = {
        prefix: 'Test<',
        startingNumber: 1,
        operations: []
      };
      
      const result = validateRenameConfig(config);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should reject config with invalid starting number', () => {
      const config: RenameConfig = {
        prefix: 'BFR',
        startingNumber: 10000,
        operations: []
      };
      
      const result = validateRenameConfig(config);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should collect multiple errors', () => {
      const config: RenameConfig = {
        prefix: '',
        startingNumber: 0,
        operations: []
      };
      
      const result = validateRenameConfig(config);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBe(2);
    });
  });

  describe('createDefaultConfig', () => {
    it('should create config with default values', () => {
      const config = createDefaultConfig();
      
      expect(config.prefix).toBe('BFR');
      expect(config.startingNumber).toBe(1);
      expect(config.additionalText).toBeUndefined();
      expect(config.operations).toEqual([]);
    });

    it('should create valid config', () => {
      const config = createDefaultConfig();
      expect(isRenameConfig(config)).toBe(true);
      
      const validation = validateRenameConfig(config);
      expect(validation.valid).toBe(true);
    });
  });
});
