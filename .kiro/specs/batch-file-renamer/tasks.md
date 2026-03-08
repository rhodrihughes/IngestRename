# Implementation Plan: Batch File Renamer

## Overview

This implementation plan breaks down the Batch File Renamer application into discrete, implementable tasks. The application is built with Tauri 2.0, combining a Rust backend for file system operations with a Svelte + TypeScript frontend. Tasks are organized to build incrementally, with early validation of core functionality through testing.

## Tasks

- [x] 1. Project setup and scaffolding
  - Initialize Tauri 2.0 project with Svelte + TypeScript template
  - Configure Vite build system
  - Set up shadcn-svelte with Tailwind CSS
  - Configure Tauri permissions for file system access and dialog API
  - Create basic project structure (frontend components, backend modules)
  - Set up testing frameworks (vitest for frontend, cargo test for backend)
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 2. Backend core data models and validation
  - [x] 2.1 Create Rust data models
    - Implement `RenameConfig`, `FileMetadata`, `PreviewResult`, `RenameResult` structs
    - Implement `RenameOperation`, `FormatType`, `TextPosition` enums
    - Add serde serialization/deserialization
    - _Requirements: 1.1, 3.1.1, 3.1.2, 4.1_

  - [ ]* 2.2 Write property test for sequential number formatting
    - **Property 2: Sequential Number Formatting and Incrementing**
    - **Validates: Requirements 2.2, 2.3**

  - [x] 2.3 Implement validation module
    - Write `validate_prefix` function for OS-specific character validation
    - Write `validate_starting_number` function (range 1-9999)
    - Write `validate_path_length` function with OS-specific limits
    - Write `check_file_exists` and `check_file_permissions` functions
    - _Requirements: 3.1.5, 3.1.6, 3.2, 3.3, 11.4, 11.5_

  - [ ]* 2.4 Write property test for starting number validation
    - **Property 3: Starting Number Validation**
    - **Validates: Requirements 3.2, 3.3**

  - [ ]* 2.5 Write property test for prefix character validation
    - **Property 5: Prefix Character Validation**
    - **Validates: Requirements 3.1.5, 3.1.6**

  - [ ]* 2.6 Write unit tests for validation edge cases
    - Test invalid starting numbers (0, 10000, negative)
    - Test OS-specific invalid prefix characters
    - Test path length boundaries
    - _Requirements: 3.2, 3.3, 3.1.5, 3.1.6, 11.4, 11.5_

- [ ] 3. Backend file operations module
  - [x] 3.1 Implement file operations module
    - Write `get_file_info` function to extract file metadata
    - Write `rename_file` function with error handling
    - Implement Windows long path support (\\?\ prefix handling)
    - Add file permission checking
    - _Requirements: 1.2, 6.1, 6.3, 7.2, 8.4, 11.1, 11.2_

  - [ ]* 3.2 Write property test for file location preservation
    - **Property 13: File Location Preservation**
    - **Validates: Requirements 6.3**

  - [ ]* 3.3 Write property test for long path access on Windows
    - **Property 23: Long Path Access on Windows**
    - **Validates: Requirements 11.2**

  - [ ]* 3.4 Write unit tests for file operations
    - Test file metadata extraction
    - Test permission error handling
    - Test file locking scenarios
    - _Requirements: 1.2, 7.2, 8.4_

- [x] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 5. Backend preview generator module
  - [x] 5.1 Implement preview generator
    - Write `PreviewGenerator` struct with `generate_preview` method
    - Implement `construct_target_name` function following naming convention
    - Implement `apply_operations` function for Requirement 12 operations
    - Handle optional additional text correctly (omit when not provided)
    - Preserve original filename and extension case
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 12.1-12.10_

  - [ ]* 5.2 Write property test for naming convention format
    - **Property 6: Naming Convention Format**
    - **Validates: Requirements 4.1, 4.6**

  - [ ]* 5.3 Write property test for original filename preservation
    - **Property 7: Original Filename Preservation**
    - **Validates: Requirements 4.2**

  - [ ]* 5.4 Write property test for extension case preservation
    - **Property 8: Extension Case Preservation**
    - **Validates: Requirements 4.3**

  - [ ]* 5.5 Write property test for additional text omission
    - **Property 4: Additional Text Omission**
    - **Validates: Requirements 3.1.4, 4.5**

  - [ ]* 5.6 Write property test for additional text positioning
    - **Property 9: Additional Text Positioning**
    - **Validates: Requirements 4.4**

  - [ ]* 5.7 Write property test for find-and-replace operation
    - **Property 27: Find-and-Replace Operation**
    - **Validates: Requirements 12.1, 12.2**

  - [ ]* 5.8 Write property test for add-text operation
    - **Property 28: Add-Text Operation**
    - **Validates: Requirements 12.3, 12.4**

  - [ ]* 5.9 Write property test for multiple operations composition
    - **Property 33: Multiple Operations Composition**
    - **Validates: Requirements 12.9**

  - [ ]* 5.10 Write unit tests for preview generator
    - Test default prefix "BFR"
    - Test files without extensions
    - Test special characters in filenames
    - Test maximum filename lengths
    - _Requirements: 3.1.3, 4.1, 10.2, 10.3, 10.4_

- [ ] 6. Backend batch renamer module
  - [x] 6.1 Implement batch renamer
    - Write `BatchRenamer` struct with `execute` method
    - Implement `rename_single` function with error handling
    - Handle duplicate target filename errors (skip and continue)
    - Handle permission errors (skip and continue)
    - Preserve file order during batch operations
    - Return detailed results with success/failure information
    - _Requirements: 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4_

  - [ ]* 6.2 Write property test for file selection order preservation
    - **Property 1: File Selection Order Preservation**
    - **Validates: Requirements 1.4, 6.2**

  - [ ]* 6.3 Write property test for batch rename completeness
    - **Property 12: Batch Rename Completeness**
    - **Validates: Requirements 6.1**

  - [ ]* 6.4 Write property test for duplicate target name handling
    - **Property 15: Duplicate Target Name Handling**
    - **Validates: Requirements 7.1**

  - [ ]* 6.5 Write property test for permission error handling
    - **Property 16: Permission Error Handling**
    - **Validates: Requirements 7.2**

  - [ ]* 6.6 Write property test for error continuation
    - **Property 17: Error Continuation**
    - **Validates: Requirements 7.3**

  - [ ]* 6.7 Write unit tests for batch renamer
    - Test single file operation
    - Test empty file list
    - Test complete batch failure scenario
    - Test partial batch failure scenario
    - _Requirements: 6.1, 7.1, 7.2, 7.3, 7.4_

- [x] 7. Checkpoint - Ensure all backend tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Tauri command handlers
  - [x] 8.1 Implement Tauri commands
    - Write `get_file_info` command handler
    - Write `generate_preview` command handler
    - Write `validate_config` command handler
    - Write `execute_batch_rename` command handler with progress events
    - Configure command permissions in Tauri capabilities
    - _Requirements: 1.1, 5.1, 5.2, 6.1, 9.4_

  - [ ]* 8.2 Write integration tests for IPC commands
    - Test command invocation from frontend
    - Test error propagation across IPC boundary
    - Test progress event emission
    - _Requirements: 8.3, 9.4_

- [ ] 9. Frontend TypeScript models and utilities
  - [x] 9.1 Create TypeScript type definitions
    - Define `RenameConfig`, `FileInfo`, `PreviewResult`, `RenameResult` interfaces
    - Define `RenameOperation`, `FormatType`, `TextPosition` types
    - Define `OperationStatus` interface
    - Create type guards and validation utilities
    - _Requirements: 1.1, 3.1.1, 3.1.2, 4.1, 9.4_

  - [x] 9.2 Create IPC wrapper functions
    - Write typed wrappers for all Tauri commands
    - Implement error handling and type conversion
    - Create event listener utilities for progress updates
    - _Requirements: 5.2, 9.4_

  - [ ]* 9.3 Write unit tests for TypeScript utilities
    - Test type guards
    - Test error handling in IPC wrappers
    - _Requirements: 8.3_

- [ ] 10. Frontend Svelte stores and state management
  - [x] 10.1 Create reactive stores
    - Create `selectedFiles` writable store
    - Create `config` writable store with default values
    - Create `previewResults` writable store
    - Create `operationStatus` writable store
    - Implement derived stores for computed values
    - _Requirements: 3.1.3, 3.4, 5.2, 9.4_

  - [ ]* 10.2 Write property test for preview reactivity
    - **Property 11: Preview Reactivity**
    - **Validates: Requirements 5.2**

- [ ] 11. Frontend FileSelector component
  - [x] 11.1 Implement FileSelector component
    - Create Svelte component with file selection button
    - Integrate Tauri dialog API for multi-file selection
    - Display selected files list with current names
    - Emit filesSelected event with file metadata
    - Style with shadcn-svelte Button component
    - _Requirements: 1.1, 1.2, 1.3, 9.1, 9.2_

  - [ ]* 11.2 Write unit tests for FileSelector
    - Test file selection event emission
    - Test empty selection handling
    - _Requirements: 1.1, 1.2_

- [x] 12. Frontend ConfigPanel component
  - [x] 12.1 Implement ConfigPanel component
    - Create input fields for prefix, starting number, additional text
    - Add validation feedback for invalid inputs
    - Implement operation builder UI for Requirement 12 features
    - Emit configChange event on input changes
    - Style with shadcn-svelte Input and Label components
    - _Requirements: 3.1.1, 3.1.2, 3.1.3, 3.1.4, 3.2, 3.3, 12.1-12.10_

  - [ ]* 12.3 Write unit tests for ConfigPanel
    - Test default values
    - Test validation error display
    - Test config change event emission
    - _Requirements: 3.1.3, 3.2, 3.3, 3.4_

- [x] 13. Frontend PreviewTable component
  - [x] 13.1 Implement PreviewTable component
    - Create table displaying original and new filenames
    - Show status indicators (valid, warning, error)
    - Display error/warning messages for each file
    - Update reactively when preview data changes
    - Style with shadcn-svelte Table components
    - _Requirements: 5.1, 5.2, 5.3, 11.3_

  - [ ]* 13.2 Write property test for preview generation
    - **Property 10: Preview Generation**
    - **Validates: Requirements 5.1**

  - [ ]* 13.3 Write property test for multi-operation preview
    - **Property 34: Multi-Operation Preview**
    - **Validates: Requirements 12.10**

  - [ ]* 13.4 Write unit tests for PreviewTable
    - Test empty preview display
    - Test error status rendering
    - Test warning status rendering
    - _Requirements: 5.1, 5.3_

- [ ] 14. Frontend ExecutionPanel component
  - [x] 14.1 Implement ExecutionPanel component
    - Create execute button with confirmation requirement
    - Display progress bar during batch operation
    - Show success message with file count on completion
    - Display error summary for failed files
    - Disable button during operation
    - Style with shadcn-svelte Button and Progress components
    - _Requirements: 5.4, 6.1, 6.4, 7.4, 9.3, 9.4_

  - [ ]* 14.2 Write property test for success message with count
    - **Property 14: Success Message with Count**
    - **Validates: Requirements 6.4**

  - [ ]* 14.3 Write property test for error summary reporting
    - **Property 18: Error Summary Reporting**
    - **Validates: Requirements 7.4**

  - [ ]* 14.4 Write property test for visual feedback during operations
    - **Property 20: Visual Feedback During Operations**
    - **Validates: Requirements 9.4**

  - [ ]* 14.5 Write unit tests for ExecutionPanel
    - Test button disabled state during operation
    - Test progress display
    - Test success message rendering
    - Test error summary rendering
    - _Requirements: 5.4, 6.4, 7.4, 9.4_

- [ ] 15. Frontend App component integration
  - [x] 15.1 Implement root App component
    - Wire together all child components
    - Implement data flow between components
    - Connect stores to component props
    - Handle preview generation on config changes
    - Handle batch execution on user confirmation
    - Implement single-window layout
    - _Requirements: 5.2, 6.1, 9.1, 9.2, 9.3_

  - [ ]* 15.2 Write integration tests for complete workflow
    - Test select → configure → preview → execute flow
    - Test error handling across components
    - _Requirements: 9.3_

- [x] 16. Checkpoint - Ensure all frontend tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 17. Cross-platform testing and validation
  - [ ]* 17.1 Write property test for cross-platform functional consistency
    - **Property 19: Cross-Platform Functional Consistency**
    - **Validates: Requirements 8.3**

  - [ ]* 17.2 Write property test for file extension support
    - **Property 21: File Extension Support**
    - **Validates: Requirements 10.1**

  - [ ]* 17.3 Write property test for special character handling
    - **Property 22: Special Character Handling**
    - **Validates: Requirements 10.3**

  - [ ]* 17.4 Write property test for long path warning
    - **Property 24: Long Path Warning**
    - **Validates: Requirements 11.3**

  - [ ]* 17.5 Write property test for path length validation
    - **Property 25: Path Length Validation**
    - **Validates: Requirements 11.4**

  - [ ]* 17.6 Write property test for path length error handling
    - **Property 26: Path Length Error Handling**
    - **Validates: Requirements 11.5**

  - [ ]* 17.7 Write property tests for format operations
    - **Property 29: Sequential Format Operation** (Requirements 12.5)
    - **Property 30: Counter Format Operation** (Requirements 12.6)
    - **Property 31: Date Format Operation** (Requirements 12.7)
    - **Property 32: Format Operation Positioning** (Requirements 12.8)

- [ ] 18. Tauri configuration and platform-specific setup
  - [x] 18.1 Configure Tauri for production
    - Update tauri.conf.json with app metadata
    - Configure file system permissions and scopes
    - Set up dialog API permissions
    - Configure bundle targets (MSI for Windows, DMG for macOS)
    - Set macOS minimum version to 13.0
    - Add Windows long path support to app manifest
    - Create application icons for all platforms
    - _Requirements: 8.1, 8.2, 11.1, 11.2_

  - [ ]* 18.2 Test production builds locally
    - Build and test Windows MSI installer
    - Build and test macOS DMG (universal binary)
    - Verify long path support on Windows
    - Verify functionality on macOS Ventura
    - _Requirements: 8.1, 8.2, 11.1, 11.2_

- [ ] 19. GitHub Actions CI/CD pipeline
  - [x] 19.1 Create build workflow
    - Set up Windows build job with MSI output
    - Set up macOS build job with universal DMG output
    - Configure artifact uploads
    - Add test execution before builds
    - Create release job that publishes artifacts
    - _Requirements: 8.1, 8.2_

  - [x] 19.2 Create test workflow
    - Run unit tests on push and PR
    - Run property-based tests (100+ iterations)
    - Generate code coverage reports
    - Run tests on both Windows and macOS runners
    - _Requirements: 8.1, 8.2, 8.3_

- [ ] 20. Documentation
  - [x] 20.1 Create user documentation
    - Write README with installation instructions
    - Document basic usage workflow
    - Document advanced features (Requirement 12 operations)
    - Add troubleshooting section for common issues
    - Document macOS Gatekeeper workaround for unsigned builds
    - _Requirements: 9.1, 9.2, 9.3_

  - [x] 20.2 Create developer documentation
    - Document project structure
    - Document build and development setup
    - Document testing strategy
    - Add API documentation for Tauri commands
    - Document CI/CD pipeline
    - _Requirements: 8.1, 8.2_

- [x] 21. Final checkpoint - Complete system validation
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation throughout development
- Property tests validate universal correctness properties (34 total properties)
- Unit tests validate specific examples, edge cases, and error conditions
- The implementation builds incrementally: backend → IPC → frontend → integration
- Cross-platform testing ensures identical behavior on Windows and macOS
