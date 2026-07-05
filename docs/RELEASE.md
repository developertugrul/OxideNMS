# Release Process

OxideNMS releases are published through GitHub Releases.

## Pre-Release Checklist

1. Update `Cargo.toml` version.
2. Update `CHANGELOG.md`.
3. Update `assets/latest.example.json`.
4. Run:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

## Tag and Publish

```bash
git tag v1.0.2
git push origin v1.0.2
```

The release workflow verifies formatting, tests, and clippy before building
release binaries.

## Artifact Names

Artifacts use this convention:

- `OxideNMS-windows-amd64.exe`
- `OxideNMS-linux-amd64`
- `OxideNMS-macos-arm64`
- `OxideNMS-macos-x86_64`

## Update Manifest Policy

`latest_version` advertises the newest release and locks older applications.

`minimum_version` remains in the manifest for compatibility and should normally
match the mandatory baseline.

`download_url` should point to the latest GitHub Release or a specific asset.
The manifest URL is compiled into the application and is not user-configurable.
