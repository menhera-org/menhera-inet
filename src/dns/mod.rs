

use regex::Regex;
use hickory_proto::rr;

use std::fmt::Display;
use std::sync::OnceLock;
use std::str::FromStr;
use std::net::ToSocketAddrs;
use std::sync::Arc;

static RE_DNS_LABEL: OnceLock<Regex> = OnceLock::new();
static RE_DIGITS_ONLY: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsError {
    InvalidInput,
    ProtocolError,
}

impl Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsError::InvalidInput => write!(f, "Invalid input"),
            DnsError::ProtocolError => write!(f, "Protocol error"),
        }
    }
}

impl std::error::Error for DnsError {}

fn get_dns_label_re() -> &'static Regex {
    RE_DNS_LABEL.get_or_init(|| {
        Regex::new(r"(?i)^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$").unwrap()
    })
}

fn get_digits_only_re() -> &'static Regex {
    RE_DIGITS_ONLY.get_or_init(|| {
        Regex::new(r"^\d+$").unwrap()
    })
}

fn is_valid_dns_host_label(label: &str) -> bool {
    get_dns_label_re().is_match(label) && !get_digits_only_re().is_match(label)
}

fn is_valid_dns_host(host: &str) -> bool {
    let host = host.trim_end_matches('.');
    if host.is_empty() {
        return false;
    }
    host.split('.').all(is_valid_dns_host_label)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedAddrs {
    v4: Vec<std::net::Ipv4Addr>,
    v6: Vec<std::net::Ipv6Addr>,
}

impl ResolvedAddrs {
    pub fn v4(&self) -> &[std::net::Ipv4Addr] {
        &self.v4
    }

    pub fn v6(&self) -> &[std::net::Ipv6Addr] {
        &self.v6
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DnsHostname {
    hostname: Arc<String>,
}

impl DnsHostname {
    pub fn new(hostname: &str) -> Result<Self, DnsError> {
        let name = rr::Name::from_str(hostname).map_err(|_| DnsError::InvalidInput)?;
        let hostname = name.to_string();
        if is_valid_dns_host(&hostname) {
            Ok(DnsHostname {
                hostname: Arc::new(hostname),
            })
        } else {
            Err(DnsError::InvalidInput)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.hostname
    }

    fn to_socket_addrs(&self) -> Result<std::vec::IntoIter<std::net::SocketAddr>, std::io::Error> {
        format!("{}:0", self.hostname).to_socket_addrs()
    }

    pub fn resolve_blocking(&self) -> Result<ResolvedAddrs, DnsError> {
        let mut v4 = Vec::new();
        let mut v6 = Vec::new();
        for addr in self.to_socket_addrs().map_err(|_| DnsError::ProtocolError)? {
            match addr {
                std::net::SocketAddr::V4(v4_addr) => {
                    v4.push(*v4_addr.ip());
                }
                std::net::SocketAddr::V6(v6_addr) => {
                    v6.push(*v6_addr.ip());
                }
            }
        }
        Ok(ResolvedAddrs {
            v4: v4,
            v6: v6,
        })
    }

    #[cfg(feature = "tokio")]
    pub async fn resolve(instance: &DnsHostname) -> Result<ResolvedAddrs, DnsError> {
        let instance = instance.clone();
        tokio::task::spawn_blocking(move || {
            instance.resolve_blocking()
        }).await.map_err(|_| DnsError::ProtocolError)?
    }
}

impl Display for DnsHostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hostname)
    }
}