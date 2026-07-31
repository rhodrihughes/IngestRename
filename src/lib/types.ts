// Type definitions for the IngestRename application
// These types mirror the Rust backend models for type-safe IPC communication
// Validates: Requirements 1.1, 3.1.1, 3.1.2, 4.1, 9.4

// ============================================================================
// Core Configuration Types
// ============================================================================

/**
 * Configuration for a batch rename operation
 * Validates: Requirements 1.1, 3.1.1, 3.1.2, 4.1
 */
export interface RenameConfig {
  /** User-defined prefix prepended to all renamed files (default: "BFR") */
  prefix: string;
  
  /** Starting sequential number for the batch (range: 1-9999, default: 1) */
  startingNumber: number;
  
  /** Optional text inserted between sequential number and original filename */
  additionalText?: string;

  /**
   * Where the incrementing sequence number is placed (default: 'start').
   *
   * - 'start': the number after the prefix increments per file
   *   (e.g. BFR_0001_..., BFR_0002_...).
   * - 'end': the number after the prefix stays fixed at startingNumber for
   *   every file, and an incrementing counter is appended to the end of each
   *   filename (e.g. BFR_0001_..._01, BFR_0001_..._02).
   */
  numberPosition?: NumberPosition;

  /** Starting value for the appended counter when numberPosition is 'end' (default: 1) */
  endNumberStart?: number;

  /** Zero-padding width for the appended counter when numberPosition is 'end' (default: 2) */
  endNumberPadding?: number;

  /** List of rename operations to apply (for Requirement 12) */
  operations: RenameOperation[];
}

/**
 * Position of the incrementing sequence number in the generated filename
 */
export type NumberPosition = 'start' | 'end';

/**
 * Position for text insertion
 */
export type TextPosition = 'before' | 'after';

/**
 * Types of rename operations (for Requirement 12)
 */
/** Separator character placed between the formatted value and the filename */
export type FormatSeparator = 'none' | '_' | '-';

export type CaseType = 'lowercase' | 'uppercase' | 'title-case' | 'camel-case' | 'snake-case';

export type RenameOperation =
  | { type: 'find-replace'; find: string; replace: string; useRegex?: boolean }
  | { type: 'add-text'; text: string; position: TextPosition }
  | { type: 'format'; format: FormatType; position: TextPosition; separator: FormatSeparator }
  | { type: 'change-case'; case: CaseType }
  | { type: 'remove-text'; text: string };

/**
 * Types of format operations
 */
export type FormatType =
  | { type: 'sequential'; start: number; increment: number }
  | { type: 'counter'; start: number; increment: number; padding: number }
  | { type: 'date'; format: string };

// ============================================================================
// File Metadata Types
// ============================================================================

/**
 * Metadata about a file to be renamed
 * Validates: Requirements 1.1, 4.1
 */
export interface FileInfo {
  /** Full path to the file */
  path: string;
  
  /** Filename without directory path */
  name: string;
  
  /** File extension (if present) */
  extension?: string;
  
  /** File size in bytes */
  size: number;
}

// ============================================================================
// Preview and Result Types
// ============================================================================

/**
 * Status of a preview result
 */
export type PreviewStatus = 'valid' | 'warning' | 'error';

/**
 * Result of a preview operation showing the proposed rename
 * Validates: Requirements 5.1, 5.2
 */
export interface PreviewResult {
  /** Original full path */
  originalPath: string;
  
  /** Original filename */
  originalName: string;
  
  /** Proposed new filename */
  newName: string;
  
  /** Proposed new full path */
  newPath: string;
  
  /** Status of the preview (valid, warning, or error) */
  status: PreviewStatus;
  
  /** Optional message explaining warnings or errors */
  message?: string;
}

/**
 * Result of a single file rename operation
 * Validates: Requirements 6.1, 7.1, 7.2, 7.3
 */
export interface RenameResult {
  /** Original file path */
  originalPath: string;
  
  /** New file path (if successful) */
  newPath: string;
  
  /** Whether the rename was successful */
  success: boolean;
  
  /** Error message if the rename failed */
  error?: string;
}

/** A record of a completed batch rename, used for undo */
export interface UndoRecord {
  label: string;
  /** [currentPath, originalPath] pairs for successfully renamed files */
  entries: [string, string][];
}

/** Response from execute_batch_rename */
export interface BatchRenameResponse {
  results: RenameResult[];
  undoRecord: UndoRecord;
}

/** A saved named preset */
export interface Preset {
  name: string;
  config: RenameConfig;
}

// ============================================================================
// Operation Status Types
// ============================================================================

/**
 * State of a batch operation
 */
export type OperationState = 'idle' | 'running' | 'completed' | 'error';

/**
 * Status of a batch rename operation
 * Validates: Requirements 9.4
 */
export interface OperationStatus {
  /** Current state of the operation */
  state: OperationState;
  
  /** Number of files processed */
  progress: number;
  
  /** Total number of files to process */
  total: number;
  
  /** Results of individual file operations */
  results: RenameResult[];
}

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Type guard to check if a value is a valid TextPosition
 */
export function isTextPosition(value: unknown): value is TextPosition {
  return value === 'before' || value === 'after';
}

/**
 * Type guard to check if a value is a valid PreviewStatus
 */
export function isPreviewStatus(value: unknown): value is PreviewStatus {
  return value === 'valid' || value === 'warning' || value === 'error';
}

/**
 * Type guard to check if a value is a valid OperationState
 */
export function isOperationState(value: unknown): value is OperationState {
  return value === 'idle' || value === 'running' || value === 'completed' || value === 'error';
}

/**
 * Type guard to check if a value is a valid RenameOperation
 */
export function isRenameOperation(value: unknown): value is RenameOperation {
  if (typeof value !== 'object' || value === null) return false;
  
  const op = value as Record<string, unknown>;
  
  if (op.type === 'find-replace') {
    return typeof op.find === 'string' && typeof op.replace === 'string';
  }
  
  if (op.type === 'add-text') {
    return typeof op.text === 'string' && isTextPosition(op.position);
  }
  
  if (op.type === 'format') {
    const sep = op.separator;
    const validSep = sep === 'none' || sep === '_' || sep === '-';
    return isFormatType(op.format) && isTextPosition(op.position) && validSep;
  }
  
  return false;
}

/**
 * Type guard to check if a value is a valid FormatType
 */
export function isFormatType(value: unknown): value is FormatType {
  if (typeof value !== 'object' || value === null) return false;
  
  const format = value as Record<string, unknown>;
  
  if (format.type === 'sequential') {
    return typeof format.start === 'number' && typeof format.increment === 'number';
  }
  
  if (format.type === 'date') {
    return typeof format.format === 'string';
  }
  
  return false;
}

/**
 * Type guard to check if a value is a valid RenameConfig
 */
export function isRenameConfig(value: unknown): value is RenameConfig {
  if (typeof value !== 'object' || value === null) return false;
  
  const config = value as Record<string, unknown>;
  
  return (
    typeof config.prefix === 'string' &&
    typeof config.startingNumber === 'number' &&
    (config.additionalText === undefined || typeof config.additionalText === 'string') &&
    Array.isArray(config.operations) &&
    config.operations.every(isRenameOperation)
  );
}

/**
 * Type guard to check if a value is a valid FileInfo
 */
export function isFileInfo(value: unknown): value is FileInfo {
  if (typeof value !== 'object' || value === null) return false;
  
  const file = value as Record<string, unknown>;
  
  return (
    typeof file.path === 'string' &&
    typeof file.name === 'string' &&
    (file.extension === undefined || file.extension === null || typeof file.extension === 'string') &&
    typeof file.size === 'number'
  );
}

/**
 * Type guard to check if a value is a valid PreviewResult
 */
export function isPreviewResult(value: unknown): value is PreviewResult {
  if (typeof value !== 'object' || value === null) return false;
  
  const result = value as Record<string, unknown>;
  
  return (
    typeof result.originalPath === 'string' &&
    typeof result.originalName === 'string' &&
    typeof result.newName === 'string' &&
    typeof result.newPath === 'string' &&
    isPreviewStatus(result.status) &&
    (result.message === undefined || typeof result.message === 'string')
  );
}

/**
 * Type guard to check if a value is a valid RenameResult
 */
export function isRenameResult(value: unknown): value is RenameResult {
  if (typeof value !== 'object' || value === null) return false;
  
  const result = value as Record<string, unknown>;
  
  return (
    typeof result.originalPath === 'string' &&
    typeof result.newPath === 'string' &&
    typeof result.success === 'boolean' &&
    (result.error === undefined || typeof result.error === 'string')
  );
}

/**
 * Type guard to check if a value is a valid OperationStatus
 */
export function isOperationStatus(value: unknown): value is OperationStatus {
  if (typeof value !== 'object' || value === null) return false;
  
  const status = value as Record<string, unknown>;
  
  return (
    isOperationState(status.state) &&
    typeof status.progress === 'number' &&
    typeof status.total === 'number' &&
    Array.isArray(status.results) &&
    status.results.every(isRenameResult)
  );
}

// ============================================================================
// Validation Utilities
// ============================================================================

/**
 * Validates a starting number is within the valid range (1-9999)
 * Validates: Requirements 3.2, 3.3
 */
export function validateStartingNumber(num: number): { valid: boolean; error?: string } {
  if (!Number.isInteger(num)) {
    return { valid: false, error: 'Starting number must be an integer' };
  }
  
  if (num < 1 || num > 9999) {
    return { valid: false, error: 'Starting number must be between 1 and 9999' };
  }
  
  return { valid: true };
}

/**
 * Validates a prefix contains only valid characters
 * Note: Full OS-specific validation is done on the backend
 * Validates: Requirements 3.1.5, 3.1.6
 */
export function validatePrefix(prefix: string): { valid: boolean; error?: string } {
  if (prefix.length === 0) {
    return { valid: false, error: 'Prefix cannot be empty' };
  }
  
  // Basic validation - backend will do OS-specific validation
  const invalidChars = /[<>:"|?*\/\\]/;
  if (invalidChars.test(prefix)) {
    return { valid: false, error: 'Prefix contains invalid characters' };
  }
  
  return { valid: true };
}

/**
 * Validates a RenameConfig object
 * Validates: Requirements 3.1.1, 3.1.2, 3.2, 3.3
 */
export function validateRenameConfig(config: RenameConfig): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  const prefixValidation = validatePrefix(config.prefix);
  if (!prefixValidation.valid && prefixValidation.error) {
    errors.push(prefixValidation.error);
  }
  
  const numberValidation = validateStartingNumber(config.startingNumber);
  if (!numberValidation.valid && numberValidation.error) {
    errors.push(numberValidation.error);
  }
  
  return {
    valid: errors.length === 0,
    errors
  };
}

/**
 * Creates a default RenameConfig
 * Validates: Requirements 3.1.3, 3.4
 */
export function createDefaultConfig(): RenameConfig {
  return {
    prefix: 'BFR',
    startingNumber: 1,
    additionalText: undefined,
    numberPosition: 'start',
    endNumberStart: 1,
    endNumberPadding: 2,
    operations: []
  };
}
