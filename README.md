# OxideNMS

OxideNMS is a Rust desktop application for Cisco-focused network management,
configuration control, and cybersecurity operations.

The target users are CCNA, CCNP, and CCIE-level network engineers and security
operators who need a fast local tool for device inventory, configuration
backup, configuration diff, hardening audits, syslog visibility, SNMP checks,
and controlled operational workflows.

## Current Capabilities

- Device inventory with local SQLite storage.
- AES-256-GCM credential encryption using a master password.
- SSH-based running-config collection.
- Configuration history and side-by-side diff workflow.
- Cisco configuration hardening audit rules.
- Bulk command deployment workflow.
- Automatic backup worker foundation.
- SNMP status map foundation.
- Built-in syslog UDP listener.
- VLAN, subnet, template, firmware, and utility screens.
- Multi-language UI infrastructure with embedded fonts.
- Mandatory and optional update manifest support.

Some screens are still foundation-level and are tracked in the roadmap.
OxideNMS is being hardened toward professional NMS/NCCCM usage.

## Product Direction

OxideNMS is being shaped around the core expectations of professional network
configuration management tools:

- Discovery and device lifecycle inventory.
- Automated configuration backup and retention.
- Change tracking, diff, and rollback preparation.
- Compliance and cybersecurity posture reporting.
- Topology visibility through SNMP, CDP, and LLDP.
- Safe bulk operations with preview, approval, and audit logging.
- GitHub Release-based desktop distribution.

## Build and Run

Install Rust from <https://rustup.rs/>, then run:

```bash
cargo run --release
```

Run checks before publishing a change:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

## Release Builds

GitHub Releases are created from version tags:

```bash
git tag v1.0.2
git push origin v1.0.2
```

The release workflow builds Windows, Linux, and macOS binaries and uploads
assets named with the `OxideNMS-{platform}-{arch}` convention.

## Update Manifest

OxideNMS can check a JSON manifest to decide whether a newer release is
optional or mandatory. See [assets/latest.example.json](assets/latest.example.json).

Example:

```json
{
  "latest_version": "1.0.2",
  "minimum_version": "1.0.2",
  "download_url": "https://github.com/developertugrul/OxideNMS/releases/latest",
  "notes": "Security and reliability release."
}
```

`minimum_version` should only be raised when older clients must be blocked.

## Documentation

- [Roadmap](docs/ROADMAP.md)
- [Release process](docs/RELEASE.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

OxideNMS is licensed under the MIT License. See [LICENSE](LICENSE).
