# OxideNMS Roadmap

OxideNMS is being developed into a professional Cisco-focused NMS/NCCCM and
cybersecurity operations desktop tool for CCNA, CCNP, and CCIE-level users.

## Phase 1: Release and Documentation

- Clean project metadata, README files, release workflow, and update manifest.
- Document release, contribution, security, and roadmap practices.
- Keep GitHub Release artifacts predictable across Windows, Linux, and macOS.

## Phase 2: Reliability and Security Foundation

- Move runtime database storage to the operating system data directory.
- Add real master password verification with vault metadata.
- Add audit logging for device operations.
- Add dry-run and approval behavior for bulk operations.
- Expand Cisco hardening checks.

## Phase 3: Professional Operator UI

- Make the main window resizable with sensible minimum dimensions.
- Use Dashboard as the operator landing page.
- Replace simulated dashboard charts with real inventory and operations metrics.
- Simplify navigation and label foundation-level tools clearly.
- Standardize page layout, status messages, and error handling.

## Phase 4: NCM/NCCCM Core

- Expand device lifecycle inventory.
- Add manual and scheduled configuration backup history.
- Add change tracking and rollback preparation.
- Add compliance policy sets and exportable reports.
- Add SNMP/CDP/LLDP topology models.

## Phase 5: Cybersecurity Operations

- Add security posture dashboard.
- Improve syslog filtering, severity views, and export.
- Track IOS/software lifecycle data.
- Add Markdown, HTML, and CSV security and change reports.
