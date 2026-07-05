//! Lightweight device discovery primitives.

use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::Duration;

use crate::error::NetworkError;
use crate::network::Subnet;

const MAX_DISCOVERY_HOSTS: u64 = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryProbe {
    pub ip: Ipv4Addr,
    pub ssh_open: bool,
    pub snmp_open: bool,
}

impl DiscoveryProbe {
    pub fn service_summary(&self) -> String {
        match (self.ssh_open, self.snmp_open) {
            (true, true) => "SSH,SNMP".to_string(),
            (true, false) => "SSH".to_string(),
            (false, true) => "SNMP".to_string(),
            (false, false) => "-".to_string(),
        }
    }

    pub fn is_candidate(&self) -> bool {
        self.ssh_open || self.snmp_open
    }
}

pub fn scan_cidr(cidr: &str, timeout: Duration) -> Result<Vec<DiscoveryProbe>, NetworkError> {
    let subnet = Subnet::parse(cidr)?;
    let start = u32::from(subnet.first_host().unwrap_or(subnet.network()));
    let end = u32::from(subnet.last_host().unwrap_or(subnet.broadcast()));
    let total = u64::from(end.saturating_sub(start)) + 1;
    let capped_end = if total > MAX_DISCOVERY_HOSTS {
        start + MAX_DISCOVERY_HOSTS as u32 - 1
    } else {
        end
    };

    let mut probes = Vec::new();
    for value in start..=capped_end {
        let ip = Ipv4Addr::from(value);
        let ssh_open = tcp_open(ip, 22, timeout);
        let snmp_open = tcp_open(ip, 161, timeout);
        if ssh_open || snmp_open {
            probes.push(DiscoveryProbe {
                ip,
                ssh_open,
                snmp_open,
            });
        }
    }

    Ok(probes)
}

fn tcp_open(ip: Ipv4Addr, port: u16, timeout: Duration) -> bool {
    let addr = SocketAddrV4::new(ip, port);
    TcpStream::connect_timeout(&addr.into(), timeout).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_summary_is_stable() {
        let probe = DiscoveryProbe {
            ip: Ipv4Addr::new(192, 0, 2, 1),
            ssh_open: true,
            snmp_open: false,
        };
        assert_eq!(probe.service_summary(), "SSH");
        assert!(probe.is_candidate());
    }

    #[test]
    fn invalid_cidr_returns_error() {
        assert!(scan_cidr("not-a-cidr", Duration::from_millis(1)).is_err());
    }
}
