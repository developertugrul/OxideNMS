# Release Process

OxideNMS releases are published through GitHub Releases.

## Pre-Release Checklist

1. Update `Cargo.toml` version.
2. Update `CHANGELOG.md`.
3. Update `latest.json`.
4. Update `assets/latest.example.json`.
5. Run local verification:

```bash
cargo fmt --check
cargo test
cargo clippy --locked -- -D warnings
```

## Tag and Publish

Create and push a version tag after the release commit is on `main`.

The GitHub Actions release workflow verifies formatting, tests, and clippy before
building release assets.

## Artifact Names

Current release assets use this convention:

- `OxideNMS-windows-amd64-setup.exe`
- `OxideNMS-linux-amd64`
- `OxideNMS-macos-arm64`

The Windows asset is an installer. It installs OxideNMS into Program Files,
creates Start Menu shortcuts, registers an uninstaller, and can optionally
create a desktop shortcut.

Linux and macOS assets are currently distributed as application binaries.

## Update Manifest Policy

`latest_version` advertises the newest release and locks older applications.

`minimum_version` remains in the manifest for compatibility and should normally
match the mandatory baseline.

`download_url` should point to the latest GitHub Release or a specific asset.
The manifest URL is compiled into the application and is not user-configurable.
