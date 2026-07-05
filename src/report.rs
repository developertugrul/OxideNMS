//! Export helpers for security and operations reports.

use crate::network::security::{Finding, FindingCode, Level};

pub fn security_markdown_report(source: &str, findings: &[Finding]) -> String {
    let mut out = String::new();
    out.push_str("# OxideNMS Security Audit Report\n\n");
    out.push_str(&format!("- Source: {}\n", source));
    out.push_str(&format!("- Findings: {}\n\n", findings.len()));
    out.push_str("| Severity | Code | Line | Detail |\n");
    out.push_str("| --- | --- | --- | --- |\n");

    for finding in findings {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            level_name(finding.level),
            code_name(finding.code),
            finding
                .line
                .map(|line| line.to_string())
                .unwrap_or_else(|| "-".to_string()),
            finding.detail.as_deref().unwrap_or("-").replace('|', "\\|")
        ));
    }

    out
}

pub fn security_csv_report(findings: &[Finding]) -> String {
    let mut out = String::from("severity,code,line,detail\n");
    for finding in findings {
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv_escape(level_name(finding.level)),
            csv_escape(code_name(finding.code)),
            finding
                .line
                .map(|line| line.to_string())
                .unwrap_or_default(),
            csv_escape(finding.detail.as_deref().unwrap_or(""))
        ));
    }
    out
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
