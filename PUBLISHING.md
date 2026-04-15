# Publishing Guide for FPGAd

This document explains how to release tagged versions on GitHub and publish the FPGAd packages to crates.io.

## GitHub Release Automation

GitHub releases are handled by `.github/workflows/release.yml`.

### Trigger Modes

1. Push a tag (`v*`) to build and publish a release automatically.
2. Run the workflow manually (`workflow_dispatch`) with:
    - `version` (for example `0.1.1`)
    - `commit_sha` (the exact commit to release)

The release workflow validates that:

- The version is semver-like
- The commit exists
- The requested version matches `[workspace.package].version` in `Cargo.toml` at that commit

If validation passes, the workflow builds in release mode and uploads these assets to the GitHub release:

- `fpgad-<version>-linux-amd64.tar.gz` (contains `fpgad` and `fpgad_cli`)
- `fpgad-<version>-linux-amd64.tar.gz.sha256`

The same workflow also publishes crates to crates.io by default by invoking `.github/workflows/cargo-publish.yml` in `publish` mode, in dependency order:

- `fpgad_macros`
- `fpgad`
- `fpgad_cli`

Publishing uses crates.io Trusted Publishing (OIDC) instead of a long-lived API token.

On crates.io, configure Trusted Publishing for this repository and workflow file (`cargo-publish.yml`) before first use.

### Changelog Strategy

This repository currently uses GitHub-generated release notes (`--generate-notes`) as the release changelog.

This approach keeps release notes aligned with merged PRs and labels without requiring a manually maintained `CHANGELOG.md`.

If a manually curated changelog is preferred later, add `CHANGELOG.md` and include an explicit update step in the release checklist.

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
- [ ] GitHub release notes reviewed (or CHANGELOG updated, if adopted)
- [ ] Git repository is clean

## Cargo Publish Automation

Cargo publishing is handled by `.github/workflows/cargo-publish.yml` (manual trigger and reusable workflow).

Inputs:

- `version` (for example `0.1.1`)
- `commit_sha`
- `mode`: `dry-run` or `publish`

Behavior:

- Publishes packages in dependency order: `fpgad_macros` -> `fpgad` -> `fpgad_cli`
- Validates that `Cargo.toml` version matches the requested version
- For `mode=publish`, requires crates.io Trusted Publishing configuration for this workflow file (`cargo-publish.yml`)

Use `mode=dry-run` to validate the publishing flow safely.

Use the manual workflow when you want to validate publish behavior before creating a GitHub release.

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

Or use the `cargo-publish.yml` workflow with `mode=dry-run`.

## Version Updates

When releasing new versions, update the version in the workspace `Cargo.toml`:

```toml
[workspace.package]
version = "X.Y.Z"  # Update this
```

All packages will inherit this version. 

Note that the CI pipeline will enforce an increase in the version whenever contents of a .rs file inside <part>/src is changed, before the commits can be merged.
