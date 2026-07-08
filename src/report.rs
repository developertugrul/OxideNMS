//! Export helpers for security and operations reports.

use crate::network::security::{Category, Finding, FindingCode, Level, summarize};

pub fn security_markdown_report(source: &str, findings: &[Finding]) -> String {
    let summary = summarize(findings);
    let mut out = String::new();

    out.push_str("# OxideNMS Security Audit Report\n\n");
    out.push_str(&format!("- Source: `{}`\n\n", source));

    // Yönetici özeti: skor, not, seviye dağılımı.
    out.push_str("## Executive Summary\n\n");
    out.push_str(&format!(
        "- **Security posture score: {} / 100 (grade {})**\n",
        summary.score, summary.grade
    ));
    out.push_str(&format!(
        "- Critical: {} · Warning: {} · Info: {}\n",
        summary.critical, summary.warning, summary.info
    ));
    out.push_str(&format!("- Total findings: {}\n\n", summary.total));

    if findings.is_empty() {
        out.push_str(
            "No security issues found. The configuration meets the checked hardening baseline.\n",
        );
        return out;
    }

    // Kategoriye göre gruplu bulgular; referans + düzeltme sütunlu.
    out.push_str("## Findings by Category\n\n");
    for &category in Category::all() {
        let group: Vec<&Finding> = findings
            .iter()
            .filter(|f| f.code.category() == category)
            .collect();
        if group.is_empty() {
            continue;
        }
        out.push_str(&format!("### {}\n\n", category.label()));
        out.push_str("| Severity | Code | Line | Detail | Reference | Remediation |\n");
        out.push_str("| --- | --- | --- | --- | --- | --- |\n");
        for finding in group {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | `{}` |\n",
                level_name(finding.level),
                code_name(finding.code),
                finding
                    .line
                    .map(|line| line.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                md_escape(finding.detail.as_deref().unwrap_or("-")),
                finding.code.reference(),
                finding.code.remediation(),
            ));
        }
        out.push('\n');
    }

    out
}

pub fn security_csv_report(findings: &[Finding]) -> String {
    let mut out = String::from("severity,code,category,line,detail,reference,remediation\n");
    for finding in findings {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            csv_escape(level_name(finding.level)),
            csv_escape(code_name(finding.code)),
            csv_escape(finding.code.category().label()),
            finding
                .line
                .map(|line| line.to_string())
                .unwrap_or_default(),
            csv_escape(finding.detail.as_deref().unwrap_or("")),
            csv_escape(finding.code.reference()),
            csv_escape(finding.code.remediation()),
        ));
    }
    out
}

/// Filo geneli uyum raporu (Markdown): filo özeti + cihaz başına skor tablosu.
pub fn fleet_markdown_report(fleet: &crate::network::compliance::FleetPosture) -> String {
    let mut out = String::new();
    out.push_str("# OxideNMS Fleet Compliance Report\n\n");

    out.push_str("## Fleet Summary\n\n");
    out.push_str(&format!(
        "- **Average posture score: {} / 100 (grade {})**\n",
        fleet.average_score,
        fleet.grade()
    ));
    out.push_str(&format!("- Devices audited: {}\n", fleet.device_count()));
    out.push_str(&format!(
        "- Critical: {} · Warning: {} · Info: {}\n\n",
        fleet.total_critical, fleet.total_warning, fleet.total_info
    ));

    if fleet.devices.is_empty() {
        out.push_str("No device configurations available to audit.\n");
        return out;
    }

    out.push_str("## Devices (highest risk first)\n\n");
    out.push_str("| Device | Score | Grade | Critical | Warning | Info |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for d in fleet.sorted_by_risk() {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            md_escape(&d.device),
            d.summary.score,
            d.summary.grade,
            d.summary.critical,
            d.summary.warning,
            d.summary.info,
        ));
    }

    out
}

/// Markdown tablo hücresi için '|' ve satır sonlarını kaçırır.
fn md_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn level_name(level: Level) -> &'static str {
    match level {
        Level::Critical => "critical",
        Level::Warning => "warning",
        Level::Info => "info",
    }
}

fn code_name(code: FindingCode) -> &'static str {
    match code {
        FindingCode::TelnetEnabled => "telnet_enabled",
        FindingCode::NoEnableSecret => "no_enable_secret",
        FindingCode::SnmpPublic => "snmp_public",
        FindingCode::SnmpPrivate => "snmp_private",
        FindingCode::SnmpRw => "snmp_rw",
        FindingCode::NoPasswordEncryption => "no_password_encryption",
        FindingCode::HttpServerEnabled => "http_server_enabled",
        FindingCode::WeakPassword => "weak_password",
        FindingCode::SshV1 => "ssh_v1",
        FindingCode::Type7Password => "type7_password",
        FindingCode::LinePasswordless => "line_passwordless",
        FindingCode::NoLogging => "no_logging",
        FindingCode::NoNtpAuth => "no_ntp_auth",
        FindingCode::ExecTimeoutDisabled => "exec_timeout_disabled",
        FindingCode::NoAaaNewModel => "no_aaa_new_model",
        FindingCode::NoLoginBanner => "no_login_banner",
        FindingCode::WeakEnableSecretType => "weak_enable_secret_type",
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_report_contains_findings() {
        let findings = crate::network::security::audit(
            "line vty 0 4\n transport input telnet\n password cisco\n",
        );
        let report = security_markdown_report("unit-test", &findings);
        assert!(report.contains("OxideNMS Security Audit Report"));
        assert!(report.contains("telnet_enabled"));
    }

    #[test]
    fn markdown_report_has_executive_summary_and_score() {
        let findings = crate::network::security::audit("ip ssh version 1\n");
        let report = security_markdown_report("unit-test", &findings);
        assert!(report.contains("Executive Summary"));
        assert!(report.contains("Security posture score"));
        assert!(report.contains("Remediation"));
        assert!(report.contains("CIS IOS"));
    }

    #[test]
    fn markdown_report_clean_config_score_100() {
        let report = security_markdown_report("unit-test", &[]);
        assert!(report.contains("100 / 100 (grade A)"));
        assert!(report.contains("No security issues found"));
    }

    #[test]
    fn csv_report_escapes_detail() {
        let findings = vec![Finding {
            level: Level::Critical,
            code: FindingCode::WeakPassword,
            line: Some(7),
            detail: Some("username admin password \"cisco\"".to_string()),
        }];
        let report = security_csv_report(&findings);
        assert!(report.contains("\"username admin password \"\"cisco\"\"\""));
    }
}
