# Changelog

All notable changes to OxideNMS will be documented in this file.

The project follows Conventional Commits and GitHub Releases. Versions use
semantic versioning.

## [1.0.5] - 2026-07-05

### Fixed

- Moved release verification to the Windows runner to avoid Linux-only clippy
  variance blocking cross-platform artifact builds.

## [1.0.4] - 2026-07-05

### Added

- Device Discovery screen for bounded CIDR scanning and inventory import.
- Audit Log screen with filtering and CSV copy support.

### Fixed

- Expanded Linux CI dependencies for egui/eframe release verification.

## [1.0.3] - 2026-07-05

### Added

- Mandatory GitHub-hosted update manifest at `latest.json`.
- Vault verification, OS data-directory database storage, and audit logging.
- Operator dashboard backed by inventory, config history, audit, and security data.
- Extended device lifecycle inventory fields.
- Markdown and CSV security audit report export.

### Changed

- New releases now lock older application builds through `latest_version`.
- The update manifest URL is fixed in code and cannot be changed from user settings.
- Release automation is triggered by pushing a new `v*` tag.

## [1.0.2] - 2026-07-05

### Added

- Professional product roadmap for Cisco NMS/NCCCM and cybersecurity workflows.
- GitHub Release documentation and release checklist.
- Security and contribution policy documents.

### Changed

- Updated project metadata for OxideNMS.
- Rewrote English and Turkish README files with clean UTF-8 content.
- Standardized release artifact naming and verification workflow.

### Notes

- This release line still contains foundation-level screens. Future releases
  will harden vault verification, audit logging, device lifecycle inventory,
  compliance reporting, and topology workflows.
