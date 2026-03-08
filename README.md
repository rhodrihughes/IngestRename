# Batch File Renamer

A desktop app for renaming files in bulk using a customizable naming convention. Built for post-production teams who need to log and track received files consistently.

**Supported platforms:** Windows 10/11 · macOS Ventura (13.0)+

---

## What it does

Batch File Renamer prepends a prefix, sequential number, and optional label to your files:

```
[prefix]_[NNNN]_[optional_text]_[original_filename]
```

Examples:
- `BFR_0001_DailyRush_video.mp4`
- `KCOC4_0042_scene5_audio.wav`
- `ProjectX_0100_document.pdf`

---

## Installation

### Windows

1. Download the `.msi` installer from the [Releases](../../releases) page
2. Run the installer and follow the prompts
3. Launch **Batch File Renamer** from the Start menu

### macOS

1. Download the `.dmg` file from the [Releases](../../releases) page
2. Open the `.dmg` and drag the app to your Applications folder
3. See [macOS first launch](#macos-first-launch) below before opening

#### macOS first launch

Builds distributed without an Apple Developer certificate will be blocked by Gatekeeper on first launch. To open the app:

1. Right-click (or Control-click) the app in Finder
2. Select **Open** from the context menu
3. Click **Open** in the confirmation dialog

This is a one-time step. After the first launch, the app opens normally with a double-click.

---

## Basic usage

The workflow takes four steps:

1. **Select files** — click "Select Files" and choose the files you want to rename (multi-select supported)
2. **Configure** — set your prefix, starting number, and optional additional text in the config panel
3. **Preview** — review the before/after table to confirm the new names look correct
4. **Execute** — click "Execute Rename" to apply the changes

The preview updates in real-time as you change the configuration, so you always see exactly what will happen before committing.

### Configuration options

| Option | Default | Description |
|---|---|---|
| Prefix | `BFR` | Text prepended to every filename |
| Starting number | `0001` | First sequential number (1–9999) |
| Additional text | *(empty)* | Optional label inserted between the number and original filename |

---

## Advanced features

In addition to the standard prefix/number format, you can apply extra rename operations to each file. These can be combined and stacked.

### Find & Replace

Replaces all occurrences of a text string within each filename.

- **Find:** text to search for
- **Replace:** text to substitute in

Example: find `_DRAFT` → replace with `` (empty) removes `_DRAFT` from every filename.

### Add Text

Inserts text at a specific position relative to the original filename.

- **Before** — inserts text before the filename (after the sequential number)
- **After** — inserts text after the filename stem, before the extension

### Format

Adds automatically generated values to filenames.

- **Sequential** — appends an incrementing counter with a configurable start value and step
- **Date** — appends a date stamp with a configurable format (e.g. `YYYY-MM-DD`)

For both sequential and date formats, you can choose whether the value appears **before** or **after** the original filename.

### Combining operations

Multiple operations can be active at the same time. The preview table shows the cumulative result of all selected operations before you execute.

---

## Troubleshooting

**"File already exists" error**
The target filename already exists in the same directory. The file is skipped and the rest of the batch continues. Rename or move the conflicting file, then re-run for the skipped files.

**"Permission denied" error**
The app doesn't have write access to the file or its directory. Check that the file isn't open in another application, and that you have write permissions for the folder.

**Long file paths on Windows**
Windows has a legacy 260-character path limit. The app supports paths beyond this limit, but you may see a warning if a target path approaches or exceeds it. If a path would exceed the system maximum (32,767 characters), the file is skipped with an error. To avoid issues, keep your folder nesting shallow or use shorter prefixes and labels.

**App won't open on macOS**
See [macOS first launch](#macos-first-launch) above. If you already tried right-click → Open and it still doesn't work, check System Settings → Privacy & Security for a prompt to allow the app.

---

## Platform support

| Platform | Minimum version | Installer |
|---|---|---|
| Windows | Windows 10 | `.msi` |
| macOS | Ventura 13.0 | `.dmg` (Universal — Intel + Apple Silicon) |
