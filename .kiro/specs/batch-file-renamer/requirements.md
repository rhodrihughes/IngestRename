# Requirements Document

## Introduction

The Batch File Renamer is a cross-platform desktop application that enables post-production teams to rename files with a customizable naming convention. The tool prepends a user-defined prefix followed by a sequential 4-digit number and optional additional text to filenames while preserving the original filename, supporting batch operations for efficient workflow.

## Glossary

- **Renamer**: The batch file renaming application
- **Source_File**: An original file selected by the user for renaming
- **Target_File**: The renamed file with the custom prefix, sequential number, optional additional text, and original filename
- **Sequential_Number**: A 4-digit zero-padded number (0001-9999) that increments for each file
- **Batch_Operation**: The process of renaming multiple files in a single operation
- **Custom_Prefix**: A user-defined text string prepended to all renamed files (e.g., "BB", "KCOC4", "ProjectX")
- **Additional_Text**: Optional user-defined text inserted between the Sequential_Number and the original filename (e.g., "DailyRush", "Scene5")
- **Naming_Convention**: The format [Custom_Prefix]_[NNNN]_[Additional_Text]_[original_filename] where NNNN is the Sequential_Number and Additional_Text is optional

## Requirements

### Requirement 1: File Selection

**User Story:** As a post-production team member, I want to select multiple files for renaming, so that I can process an entire batch of received files at once.

#### Acceptance Criteria

1. THE Renamer SHALL provide a file selection interface that allows multiple file selection
2. THE Renamer SHALL display the list of selected Source_Files with their current names
3. THE Renamer SHALL support file selection on both Windows and macOS operating systems
4. WHEN a user selects files, THE Renamer SHALL preserve the selection order for Sequential_Number assignment

### Requirement 2: Sequential Number Generation

**User Story:** As a post-production team member, I want files to be numbered sequentially, so that I can track the order in which files were received.

#### Acceptance Criteria

1. THE Renamer SHALL generate Sequential_Numbers starting from 0001
2. THE Renamer SHALL increment the Sequential_Number by 1 for each subsequent file in a Batch_Operation
3. THE Renamer SHALL format Sequential_Numbers as 4-digit zero-padded strings (0001, 0002, 0003, etc.)
4. THE Renamer SHALL support Sequential_Numbers up to 9999

### Requirement 3: Starting Number Configuration

**User Story:** As a post-production team member, I want to specify the starting number for a batch, so that I can continue numbering from where I left off in previous batches.

#### Acceptance Criteria

1. THE Renamer SHALL allow the user to specify a custom starting Sequential_Number
2. WHEN a custom starting number is provided, THE Renamer SHALL validate that it is between 0001 and 9999
3. IF the starting number is invalid, THEN THE Renamer SHALL display an error message and prevent the Batch_Operation
4. WHERE no custom starting number is specified, THE Renamer SHALL default to 0001

### Requirement 3.1: Prefix and Additional Text Configuration

**User Story:** As a post-production team member, I want to customize the prefix and add optional text to filenames, so that I can adapt the naming convention to different projects and workflows.

#### Acceptance Criteria

1. THE Renamer SHALL allow the user to specify a Custom_Prefix for the Batch_Operation
2. THE Renamer SHALL allow the user to specify optional Additional_Text for the Batch_Operation
3. WHERE no Custom_Prefix is specified, THE Renamer SHALL default to "BB"
4. WHERE no Additional_Text is specified, THE Renamer SHALL omit the Additional_Text component from Target_File names
5. THE Renamer SHALL validate that Custom_Prefix contains only characters valid for filenames on the target operating system
6. IF the Custom_Prefix contains invalid characters, THEN THE Renamer SHALL display an error message and prevent the Batch_Operation

### Requirement 4: Filename Transformation

**User Story:** As a post-production team member, I want files renamed with a customizable prefix, sequential number, and optional additional text, so that all received files follow our flexible logging convention.

#### Acceptance Criteria

1. WHEN renaming a Source_File, THE Renamer SHALL construct the Target_File name using the Custom_Prefix, Sequential_Number, Additional_Text (if provided), and original filename separated by underscores
2. THE Renamer SHALL preserve the complete original filename including file extension
3. THE Renamer SHALL preserve the file extension case (e.g., .MP4, .mp4, .Mp4)
4. WHERE Additional_Text is provided, THE Renamer SHALL insert it between the Sequential_Number and the original filename
5. WHERE Additional_Text is not provided, THE Renamer SHALL omit it from the Target_File name
6. FOR ALL Source_Files, THE Renamer SHALL generate Target_File names matching the Naming_Convention format

### Requirement 5: Preview Functionality

**User Story:** As a post-production team member, I want to preview the new filenames before applying changes, so that I can verify the renaming will be correct.

#### Acceptance Criteria

1. WHEN files are selected, THE Renamer SHALL display a preview showing both original and Target_File names
2. THE Renamer SHALL update the preview in real-time when the Custom_Prefix, starting Sequential_Number, or Additional_Text changes
3. THE Renamer SHALL clearly indicate which files will be renamed in the preview
4. THE Renamer SHALL require user confirmation before executing the Batch_Operation

### Requirement 6: Batch Rename Execution

**User Story:** As a post-production team member, I want to rename all selected files with a single action, so that I can efficiently process large batches of files.

#### Acceptance Criteria

1. WHEN the user confirms the Batch_Operation, THE Renamer SHALL rename all Source_Files to their corresponding Target_File names
2. THE Renamer SHALL rename files in the order they were selected
3. THE Renamer SHALL preserve the file location (files remain in their original directory)
4. WHEN the Batch_Operation completes, THE Renamer SHALL display a success message with the count of renamed files

### Requirement 7: Error Handling

**User Story:** As a post-production team member, I want to be notified of any errors during renaming, so that I can address issues and ensure all files are properly renamed.

#### Acceptance Criteria

1. IF a Target_File name already exists in the directory, THEN THE Renamer SHALL display an error message and skip that file
2. IF a file cannot be renamed due to permissions, THEN THE Renamer SHALL display an error message and skip that file
3. IF an error occurs during a Batch_Operation, THEN THE Renamer SHALL continue processing remaining files
4. WHEN a Batch_Operation completes with errors, THE Renamer SHALL display a summary listing which files failed and why

### Requirement 8: Cross-Platform Compatibility

**User Story:** As a post-production team member, I want the tool to work identically on Windows and macOS, so that all team members can use the same workflow regardless of their operating system.

#### Acceptance Criteria

1. THE Renamer SHALL run on Windows 11 and later versions (also compatible with Windows 10)
2. THE Renamer SHALL run on macOS Ventura (13.0) and later versions
3. THE Renamer SHALL provide identical functionality on both Windows and macOS
4. THE Renamer SHALL use native file system operations appropriate for each operating system

### Requirement 9: User Interface Simplicity

**User Story:** As a post-production team member, I want a simple and intuitive interface, so that I can quickly rename files without extensive training.

#### Acceptance Criteria

1. THE Renamer SHALL provide a single-window interface with all controls visible
2. THE Renamer SHALL use clear labels for all interactive elements
3. THE Renamer SHALL complete a typical Batch_Operation in no more than 4 user actions (select files, optionally configure prefix/additional text/starting number, confirm)
4. THE Renamer SHALL provide visual feedback during file renaming operations

### Requirement 10: File Type Support

**User Story:** As a post-production team member, I want to rename any file type, so that I can use the tool for all received media files regardless of format.

#### Acceptance Criteria

1. THE Renamer SHALL support renaming files with any file extension
2. THE Renamer SHALL support renaming files without file extensions
3. THE Renamer SHALL handle filenames containing special characters permitted by the operating system
4. THE Renamer SHALL handle filenames up to the maximum length supported by the operating system

### Requirement 11: Long File Path Handling

**User Story:** As a post-production team member working on Windows, I want the tool to handle long file paths beyond the 260-character limit, so that I can rename files in deeply nested directory structures without errors.

#### Acceptance Criteria

1. WHEN running on Windows, THE Renamer SHALL support file paths longer than 260 characters
2. WHEN a Source_File path exceeds 260 characters on Windows, THE Renamer SHALL use long path support mechanisms to access the file
3. WHEN a Target_File path would exceed 260 characters on Windows, THE Renamer SHALL display a warning message before attempting the rename
4. THE Renamer SHALL validate that the complete Target_File path length does not exceed the operating system maximum (32,767 characters on Windows, 1,024 characters on macOS)
5. IF a Target_File path would exceed the operating system maximum, THEN THE Renamer SHALL display an error message and skip that file

### Requirement 12: macOS Finder Feature Parity

**User Story:** As a post-production team member familiar with macOS Finder's rename tool, I want the same renaming capabilities available in this tool, so that I can perform all common rename operations in one application across both platforms.

#### Acceptance Criteria

1. THE Renamer SHALL provide a find-and-replace operation that replaces specified text in Source_File names
2. WHEN performing find-and-replace, THE Renamer SHALL replace all occurrences of the search text in each filename
3. THE Renamer SHALL provide an add-text operation that inserts text before the filename
4. THE Renamer SHALL provide an add-text operation that inserts text after the filename (before the extension)
5. THE Renamer SHALL provide a format operation that adds sequential numbers to filenames
6. THE Renamer SHALL provide a format operation that adds counter values to filenames with configurable start value and increment
7. THE Renamer SHALL provide a format operation that adds date stamps to filenames with configurable date format
8. WHEN applying format operations, THE Renamer SHALL allow the user to specify the position (before or after the original filename)
9. THE Renamer SHALL allow combining multiple rename operations in a single Batch_Operation
10. THE Renamer SHALL update the preview to show the result of all selected rename operations before execution
