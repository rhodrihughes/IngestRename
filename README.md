# IngestRename

[![Latest Release](https://img.shields.io/github/v/release/rhodrihughes/IngestRename?style=flat-square&label=latest)](https://github.com/rhodrihughes/IngestRename/releases/latest)
[![Download for macOS](https://img.shields.io/badge/download-macOS-blue?style=flat-square&logo=apple)](https://github.com/rhodrihughes/IngestRename/releases/latest/download/IngestRename.dmg)
[![Download for Windows](https://img.shields.io/badge/download-Windows-blue?style=flat-square&logo=windows)](https://github.com/rhodrihughes/IngestRename/releases/latest/download/IngestRename_x64_en-US.msi)
[![Build](https://img.shields.io/github/actions/workflow/status/rhodrihughes/IngestRename/build.yml?style=flat-square&label=build)](https://github.com/rhodrihughes/IngestRename/actions/workflows/build.yml)

A desktop app for renaming files in bulk using a customizable naming convention. Built for post-production teams who need to log and track received files consistently.

**Supported platforms:** Windows 10/11 · macOS Ventura (13.0)+

---

## What it does

IngestRename prepends a prefix, sequential number, and optional label to your files:

```
[prefix]_[NNNN]_[optional_text]_[original_filename]
```

Examples:
- `BFR_0001_DailyRush_video.mp4`
- `ABC_0042_scene5_audio.wav`
- `ProjectX_0100_document.pdf`

---

## Installation

### Windows

1. Download the `.msi` installer from the [Releases](../../releases) page
2. Run the installer and follow the prompts
3. Launch **IngestRename** from the Start menu

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

1. **Select files** — click "Select Files" to pick individual files (multi-select supported), or "Select Folder" to load all files from a folder at once
2. **Configure** — set your prefix, starting number, and optional additional text
3. **Preview** — review the before/after table to confirm the new names look correct
4. **Execute** — click "Execute Rename" to apply the changes

The preview updates in real-time as you change the configuration.

### Configuration options

| Option | Default | Description |
|---|---|---|
| Prefix | `BFR` | Text prepended to every filename |
| Starting number | `0001` | First sequential number (1–9999) |
| Additional text | *(empty)* | Optional label inserted between the number and original filename |

---

## Rename operations

In addition to the standard prefix/number format, you can stack extra operations on each filename. Operations are applied in order and can be reordered by dragging or using the ↑/↓ buttons.

### Find & Replace

Replaces all occurrences of a string within each filename.

- **Find / Replace** — literal text substitution
- **Use regex** — toggle to treat the Find field as a regular expression (see the in-app Regex guide for syntax help)

### Add Text

Inserts text at a specific position:

- **Before** — before the filename stem
- **After** — after the filename stem, before the extension

### Change Case

Transforms the case of the filename stem:

| Option | Example |
|---|---|
| lowercase | `dailyrush_clip01` |
| UPPERCASE | `DAILYRUSH_CLIP01` |
| Title Case | `Dailyrush Clip01` |
| camelCase | `dailyrushClip01` |
| snake_case | `dailyrush_clip01` |

### Remove Text

Removes all occurrences of a specific string from the filename.

### Format

Appends or prepends an automatically generated value:

- **Sequential** — incrementing number with configurable start and step
- **Counter** — same as sequential but with configurable zero-padding (default: 4 digits)
- **Date stamp** — today's date in a configurable format

#### Date format tokens

| Token | Value |
|---|---|
| `YYYY` | 4-digit year |
| `YY` | 2-digit year |
| `MM` | Month (01–12) |
| `DD` | Day (01–31) |
| `HH` | Hour (00–23) |
| `mm` | Minute (00–59) |
| `ss` | Second (00–59) |

For all Format operations you can choose a separator (none, `_`, or `-`) and whether the value appears before or after the filename.

---

## Presets

Save your current configuration as a named preset to reuse it later. Presets are stored locally and persist between sessions. Load or delete them from the Presets section in the config panel.

---

## Undo

After a successful rename, an **Undo** button appears. Clicking it reverses all renames from that batch, restoring the original filenames.

---

## Preview table

The preview table shows original and new filenames side by side. Use the copy icon in each column header to copy all original or new names to the clipboard (newline-separated).

---

## Troubleshooting

**"File already exists" error**
The target filename already exists in the same directory. The file is skipped and the rest of the batch continues. Rename or move the conflicting file, then re-run for the skipped files.

**"Permission denied" error**
The app doesn't have write access to the file or its directory. Check that the file isn't open in another application and that you have write permissions for the folder.

**Long file paths on Windows**
Windows has a legacy 260-character path limit. The app supports paths beyond this limit, but you may see a warning if a target path approaches or exceeds it. If a path would exceed the system maximum (32,767 characters), the file is skipped with an error. Keep folder nesting shallow or use shorter prefixes to avoid issues.

**App won't open on macOS**
See [macOS first launch](#macos-first-launch) above. If you already tried right-click → Open and it still doesn't work, check System Settings → Privacy & Security for a prompt to allow the app.

**Can't select multiple files with Cmd+A in the file dialog**
This is a known limitation of the Tauri webview — the keyboard shortcut is intercepted before it reaches the OS dialog. Use "Select Folder" instead to load all files from a directory at once.

---

## Platform support

| Platform | Minimum version | Installer |
|---|---|---|
| Windows | Windows 10 | `.msi` |
| macOS | Ventura 13.0 | `.dmg` (Universal — Intel + Apple Silicon) |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Author

Built by [Rhodri Hughes](https://github.com/rhodrihughes) · [GitHub repo](https://github.com/rhodrihughes/IngestRename)
