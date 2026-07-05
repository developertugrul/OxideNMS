# Security Policy

OxideNMS handles network device credentials and can run operational commands.
Security-sensitive changes must be reviewed carefully.

## Supported Versions

Only the latest GitHub Release is considered supported.

## Reporting a Vulnerability

Do not open a public issue for credential handling, command execution, update,
or storage vulnerabilities. Report privately to the repository owner.

Include:

- OxideNMS version.
- Operating system.
- Reproduction steps.
- Impact assessment.
- Any relevant logs with secrets removed.

## Security Expectations

- Credentials must never be stored as plaintext.
- Master password verification must reject wrong passwords.
- Bulk operations must provide preview and explicit approval.
- Every device operation must create an audit trail.
- Update releases must be distributed through GitHub Releases and a controlled
  manifest.
