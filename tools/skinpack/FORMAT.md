# RemixClient Skin Pack Format Specification

This specification defines the directory structure, metadata files, and binary constraints for community skin packs compatible with RemixClient.

## Directory Structure

A skin pack source directory has the following layout:

```text
my-skin-pack/
├── pack.json
└── skins/
    ├── 069a79f4-44e9-4726-a5be-fca90e38aaf5/
    │   ├── skin.png
    │   ├── cape.png     (optional)
    │   └── meta.json
    └── <uuid-v4-or-v3>/
        ├── skin.png
        └── meta.json
```

---

## 1. Pack Manifest (`pack.json`)

Located at the root of the skin pack directory.

```json
{
  "name": "Community Classic Pack",
  "author": "RemixCommunity",
  "license": "MIT",
  "version": "1.0.0",
  "skins": []
}
```

- **`name`** (`string`): Display name of the skin pack.
- **`author`** (`string`): Author or creator group.
- **`license`** (`string`): SPDX license identifier (e.g. `MIT`, `CC0-1.0`).
- **`version`** (`string`): Pack version.
- **`skins`** (`array`): Automatically populated by the `skinpack pack` CLI tool with cryptographic hashes and metadata.

---

## 2. Individual Skin Directory (`skins/<uuid>/`)

Each skin entry resides in a subdirectory named after the player's UUID (e.g., `069a79f4-44e9-4726-a5be-fca90e38aaf5`).

### `meta.json`
```json
{
  "name": "Notch",
  "model": "classic"
}
```
- **`name`** (`string`): In-game display name for the skin (max 64 chars, no path separators).
- **`model`** (`string`): Either `"classic"` (Steve 4px arms) or `"slim"` (Alex 3px arms).

### `skin.png`
- Must be a valid PNG binary file starting with signature `89 50 4E 47 0D 0A 1A 0A`.
- Allowed dimensions:
  - **64×32** (Legacy Java Edition)
  - **64×64** (Modern Java Edition standard)
  - **128×64** (HD skin standard)
  - **128×128** (High-res HD skin standard)

### `cape.png` (Optional)
- Optional custom cape texture.
- Must follow the same PNG validation rules (signature and allowed dimensions).

---

## 3. Verification & Security

The `skinpack` CLI tool enforces strict security checks:
1. **Path Traversal Protection**: Rejects any zip entries or source paths containing `..`, absolute paths (`/`), or Windows backslashes (`\`).
2. **Cryptographic Integrity**: Computes SHA-256 checksums for all skin/cape textures and validates them against `pack.json`.
3. **PNG Header Verification**: Inspects magic bytes and chunk header dimensions directly from binary data before packing or verifying.
