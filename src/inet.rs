
use std::str::FromStr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InetError {
    V4(crate::ipv4::Ipv4Error),
    V6(crate::ipv6::Ipv6Error),
    Other,
}

impl Display for InetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InetError::V4(e) => write!(f, "IPv4 error: {}", e),
            InetError::V6(e) => write!(f, "IPv6 error: {}", e),
            InetError::Other => write!(f, "Other error"),
        }
    }
}

impl std::error::Error for InetError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InetTarget {
    V4(crate::ipv4::Ipv4Target),
    V6(crate::ipv6::Ipv6Target),
}

impl FromStr for InetTarget {
    type Err = InetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v4) = s.parse() {
            return Ok(InetTarget::V4(v4));
        }
        if let Ok(v6) = s.parse() {
            return Ok(InetTarget::V6(v6));
        }
        Err(InetError::Other)
    }
}

impl Display for InetTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InetTarget::V4(v4) => write!(f, "{}", v4),
            InetTarget::V6(v6) => write!(f, "{}", v6),
        }
    }
}

impl InetTarget {
    pub fn ip(&self) -> std::net::IpAddr {
        match self {
            InetTarget::V4(v4) => std::net::IpAddr::V4(v4.ip()),
            InetTarget::V6(v6) => std::net::IpAddr::V6(v6.ip()),
        }
    }

    pub fn prefix_len(&self) -> Option<u8> {
        match self {
            InetTarget::V4(v4) => v4.prefix_len(),
            InetTarget::V6(v6) => v6.prefix_len(),
        }
    }

    pub fn is_net(&self) -> bool {
        match self {
            InetTarget::V4(v4) => v4.is_net(),
            InetTarget::V6(v6) => v6.is_net(),
        }
    }

    pub fn net(&self) -> Option<ipnet::IpNet> {
        match self {
            InetTarget::V4(v4) => v4.net().map(|n| n.into()),
            InetTarget::V6(v6) => v6.net().map(|n| n.into()),
        }
    }
}
