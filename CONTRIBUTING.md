# Contributing to IngestRename

Developer guide for building, testing, and extending the project.

---

## Project structure

```
IngestRename/
├── src/                          # SvelteKit frontend
│   ├── lib/
│   │   ├── components/           # Svelte UI components
│   │   │   ├── FileSelector.svelte
│   │   │   ├── ConfigPanel.svelte
│   │   │   ├── PreviewTable.svelte
│   │   │   └── ExecutionPanel.svelte
│   │   ├── stores.ts             # Svelte reactive stores
│   │   ├── ipc.ts                # Tauri IPC wrapper functions
│   │   └── types.ts              # TypeScript type definitions
│   └── routes/
│       └── +page.svelte          # Main application page
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # Tauri command handlers
│       ├── file_ops/             # File system operations
│       ├── validator/            # Input validation logic
│       ├── preview/              # Preview generation
│       ├── renamer/              # Batch rename execution
│       └── models.rs             # Shared Rust data models
├── .github/
│   └── workflows/
│       ├── build.yml             # Release builds (MSI + DMG)
│       └── test.yml              # CI test runs
└── README.md
```

---

## Development setup

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable toolchain)
- On macOS: Xcode Command Line Tools (`xcode-select --install`)
- On Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the C++ workload

### Install and run

```bash
# Install frontend dependencies
npm install

# Start the development server (hot-reload for both frontend and backend)
npm run tauri dev

# Build for production
npm run tauri build
```

The dev server starts a Vite dev server on `localhost:5173` and launches the Tauri window pointing at it. Changes to Svelte files hot-reload instantly; changes to Rust files trigger a recompile.

---

## Testing

### Frontend (vitest)

Tests are co-located with source files using the `.test.ts` suffix.

```bash
# Run all frontend tests once
npx vitest run

# Watch mode during development
npx vitest
```

### Backend (cargo test)

```bash
# Run all backend tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run a specific test module
cargo test --manifest-path src-tauri/Cargo.toml -- validator::tests
```

### Property-based tests

The backend uses [proptest](https://github.com/proptest-rs/proptest) for property-based testing. By default proptest runs 256 cases per property; set `PROPTEST_CASES` to override:

```bash
PROPTEST_CASES=100 cargo test --manifest-path src-tauri/Cargo.toml
```

Property tests are annotated with the requirement they validate (e.g. `// Validates: Requirements 4.1, 4.6`) and cover 34 correctness properties across the naming convention, validation, file operations, and error handling.

---

## Tauri commands API

The frontend communicates with the Rust backend through four IPC commands. All commands are invoked via `@tauri-apps/api/core`'s `invoke`.

### `get_file_info`

Retrieves metadata for a list of file paths.

```typescript
invoke<FileMetadata[]>('get_file_info', { paths: string[] })
```

| Parameter | Type | Description |
|---|---|---|
| `paths` | `string[]` | Absolute paths to the files |

Returns an array of `FileMetadata` objects (`path`, `name`, `extension`, `size`).

---

### `generate_preview`

Generates the target filenames for a set of files without touching the filesystem.

```typescript
invoke<PreviewResult[]>('generate_preview', { files: FileMetadata[], config: RenameConfig })
```

| Parameter | Type | Description |
|---|---|---|
| `files` | `FileMetadata[]` | Files to preview |
| `config` | `RenameConfig` | Rename configuration (prefix, starting number, operations, etc.) |

Returns an array of `PreviewResult` objects with `originalName`, `newName`, `status` (`valid` / `warning` / `error`), and an optional `message`.

---

### `validate_config`

Validates a `RenameConfig` without performing any file operations.

```typescript
invoke<ValidationResult>('validate_config', { config: RenameConfig })
```

Returns a `ValidationResult` with `valid: boolean`, `errors`, and `warnings` arrays.

---

### `execute_batch_rename`

Performs the batch rename. Emits `rename-progress` events during execution.

```typescript
invoke<RenameResult[]>('execute_batch_rename', { files: FileMetadata[], config: RenameConfig })
```

Progress events can be subscribed to with:

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<{ progress: number; total: number }>('rename-progress', (event) => {
  // update UI progress
});
```

Returns an array of `RenameResult` objects with `originalPath`, `newPath`, `success`, and an optional `error` string.

---

## CI/CD pipeline

### `test.yml` — continuous testing

Runs on every push and pull request. Executes the full test suite (frontend + backend, including property-based tests) on a matrix of `windows-latest` and `macos-13`.

### `build.yml` — release builds

Triggered by pushing a `v*` tag or via manual workflow dispatch. Produces:

- **Windows**: `.msi` installer (built on `windows-latest`)
- **macOS**: Universal `.dmg` (Intel + Apple Silicon, built on `macos-13`)

Artifacts are uploaded and a GitHub Release is created automatically. macOS signing and notarization are optional — add the relevant secrets (`APPLE_CERTIFICATE`, `APPLE_ID`, etc.) to the repository to enable them. Without signing, the app is fully functional but requires a right-click → Open on first launch due to Gatekeeper.

### Creating a release

```bash
git tag v1.0.0
git push origin v1.0.0
```

This triggers `build.yml`, which builds both platforms and publishes a GitHub Release with the installers attached.
