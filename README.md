# OxideNMS

OxideNMS is a professional desktop network management and cybersecurity operations
tool for Cisco-focused teams.

It is designed for CCNA, CCNP, CCIE, NOC, NetOps, and security operations users
who need a local, fast, and controlled application for day-to-day Cisco network
operations.

## What OxideNMS Does

OxideNMS brings the most important network configuration management workflows
into one desktop application:

- Device inventory and lifecycle tracking.
- Encrypted local credential vault.
- SSH-based running-config collection.
- Configuration backup history.
- Configuration comparison and change review.
- Cisco configuration hardening checks.
- Bulk command deployment with dry-run and confirmation flow.
- Operational audit log.
- Device discovery scan.
- Syslog listener for live operational visibility.
- SNMP topology/status map foundation.
- VLAN, subnet, template, and firmware operation screens.
- Mandatory update enforcement from the official release manifest.

## Who It Is For

OxideNMS is built for engineers and operators who manage Cisco environments:

- Network engineers working at CCNA, CCNP, or CCIE level.
- NOC and NetOps teams responsible for operational consistency.
- Cybersecurity teams reviewing network device posture.
- Consultants who need a portable but controlled Cisco operations workstation.

## Security Model

OxideNMS stores operational data locally and protects device credentials with a
master password. The application also keeps an audit trail for sensitive actions
such as vault initialization, device changes, backups, discovery imports, and
bulk deploy operations.

Update checks are mandatory. When a newer official release exists, older builds
are locked until the user installs the current version. The update manifest URL
is fixed in the application and cannot be changed by end users.

## Installation

Windows users should install OxideNMS from the official installer in GitHub
Releases:

[Download the latest OxideNMS release](https://github.com/developertugrul/OxideNMS/releases/latest)

The Windows installer installs OxideNMS into Program Files, creates Start Menu
shortcuts, registers an uninstaller, and can optionally create a desktop
shortcut.

Linux and macOS release assets are currently distributed as application binaries.

## Product Direction

OxideNMS is being developed toward professional NMS/NCCCM workflows:

- Scheduled configuration backup and retention.
- Change tracking and rollback preparation.
- Compliance policy sets and exportable reports.
- SNMPv3 credential profiles.
- CDP/LLDP topology enrichment.
- Safer job queues for bulk operations.
- Security posture dashboards and alerting.

## Documentation

- [Roadmap](docs/ROADMAP.md)
- [Release process](docs/RELEASE.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

OxideNMS is licensed under the MIT License. See [LICENSE](LICENSE).
