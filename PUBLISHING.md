# Publishing Guide for FPGAd

This document explains how to publish the FPGAd packages to crates.io.

## Package Structure

The workspace contains 3 packages:

1. **`fpgad_macros`** - Procedural macros (must be published first)
2. **`fpgad`** - The daemon (depends on fpgad_macros)
3. **`fpgad_cli`** - Command-line interface

## Publication Order

Packages must be published in this order due to dependencies:

1. `fpgad_macros` (no dependencies on other workspace crates)
2. `fpgad` (depends on fpgad_macros)
3. `fpgad_cli` (no dependencies on other workspace crates, but logically depends on daemon)

## Pre-publication Checklist

- [ ] All packages have proper metadata (version, license, description, etc.)
- [ ] All packages have README.md files
- [ ] Workspace-level metadata is shared across packages
- [ ] Dependencies have version requirements specified
- [ ] All packages compile successfully
- [ ] All tests pass (CI auto runs on PR)
- [ ] Documentation is complete
- [ ] CHANGELOG is up to date
- [ ] Git repository is clean

## Publishing Commands

### 1. Publish fpgad_macros

```bash
cd fpgad_macros
cargo publish
```

### 2. Publish fpgad

```bash
cd daemon
cargo publish
```

### 3. Publish fpgad_cli

```bash
cd cli
cargo publish
```

## Dry Run Testing

Before publishing, test with dry-run:

```bash
cargo publish --dry-run
```

## Version Updates

When releasing new versions, update the version in the workspace `Cargo.toml`:

```toml
[workspace.package]
version = "X.Y.Z"  # Update this
```

All packages will inherit this version. 

Note that the CI pipeline will enforce an increase in the version whenever contents of a .rs file inside <part>/src is changed, before the commits can be merged.
