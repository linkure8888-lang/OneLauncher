# skinpack — Community Skin Pack CLI

`skinpack` is a standalone Rust command-line tool designed for creating, verifying, and indexing community skin packs for RemixClient.

---

## Installation

```bash
cd tools/skinpack
cargo build --release
```

The compiled binary will be located at `target/release/skinpack.exe` (or `target/release/skinpack` on Unix).

---

## Commands

### 1. `pack`
Packs a skin pack source directory into a `.zip` archive, validates all PNG files and metadata, and computes SHA-256 checksums into `pack.json`.

```bash
skinpack pack <DIR> -o <OUT.ZIP>
```

### 2. `verify`
Verifies the integrity of a `.zip` skin pack, checking path traversal safety, PNG signatures/dimensions, and SHA-256 checksums against `pack.json`.

```bash
skinpack verify <PACK.ZIP>
```

### 3. `index`
Scans a directory of `.zip` skin packs and generates an aggregated `index.json` containing metadata, sizes, and checksums for each pack.

```bash
skinpack index <DIR> -o <INDEX.JSON>
```

---

## Pack Structure

Refer to [FORMAT.md](FORMAT.md) for detailed layout requirements, `pack.json` schema, and PNG dimension specifications.
