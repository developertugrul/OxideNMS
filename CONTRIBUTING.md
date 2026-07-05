# Contributing to OxideNMS

OxideNMS is a Cisco-focused Rust desktop application for network management,
configuration control, and cybersecurity operations.

## Development Rules

- Keep changes scoped and commit with Conventional Commits.
- Run `cargo fmt --check`, `cargo test`, and `cargo clippy -- -D warnings`
  before opening a pull request.
- Prefer service/domain modules for operational logic. GUI screens should not
  own SSH, database, backup, or audit behavior directly.
- Do not store secrets in tests, examples, commits, screenshots, or logs.
- Treat risky network operations as explicit workflows with preview, approval,
  result tracking, and audit logging.

## Commit Examples

```text
docs: prepare GitHub release documentation
feat: harden vault storage and operational audit logging
fix: reject invalid master password unlock attempts
ci: verify release builds before publishing assets
```

## Pull Request Checklist

- The change is limited to one coherent topic.
- User-facing behavior is documented.
- Security-sensitive behavior has tests.
- Release notes are added to `CHANGELOG.md` when user-visible behavior changes.
