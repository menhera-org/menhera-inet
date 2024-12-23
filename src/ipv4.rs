
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::fmt::Display;

type RawIpv4Addr = [u8; 4];

fn ipv4_subnet_mask(prefix_len: u8) -> RawIpv4Addr {
    let mut mask = 0xffffffffu32;
    mask = mask.checked_shr(prefix_len as u32).unwrap_or(0);
    mask = !mask;
    let mask = mask.to_be_bytes();
    mask
}

/// First address in the network
fn ipv4_network_address(ip: RawIpv4Addr, prefix_len: u8) -> RawIpv4Addr {
    let mask = ipv4_subnet_mask(prefix_len);
    let mut addr = [0; 4];
    for i in 0..4 {
        addr[i] = ip[i] & mask[i];
    }
    addr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Error;

impl Display for Ipv4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IPv4 address error")
    }
}

impl std::error::Error for Ipv4Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Target {
    pub(crate) ip: RawIpv4Addr,
    pub(crate) prefix_len: Option<u8>,
}

impl Ipv4Target {
    pub fn new(ip: Ipv4Addr, prefix_len: Option<u8>) -> Result<Self, Ipv4Error> {
        let ip = ip.octets();
        if let Some(prefix_len) = prefix_len {
            if prefix_len > 32 {
                return Err(Ipv4Error);
            }
            let network = ipv4_network_address(ip, prefix_len);
            if ip != network {
                return Err(Ipv4Error);
            }
        }
        Ok(Ipv4Target { ip, prefix_len })
    }

    pub fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.ip)
    }

    pub fn prefix_len(&self) -> Option<u8> {
        self.prefix_len
    }

    pub fn is_net(&self) -> bool {
        self.prefix_len.is_some()
    }

    pub fn net(&self) -> Option<ipnet::Ipv4Net> {
        self.prefix_len.map(|prefix_len| {
            let network = ipv4_network_address(self.ip, prefix_len);
            ipnet::Ipv4Net::new(Ipv4Addr::from(network), prefix_len).unwrap()
        })
    }
}

impl Display for Ipv4Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ip = Ipv4Addr::from(self.ip);
        match self.prefix_len {
            Some(prefix_len) => write!(f, "{}/{}", ip, prefix_len),
            None => write!(f, "{}", ip),
        }
    }
}

impl FromStr for Ipv4Target {
    type Err = Ipv4Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');
        let ip = parts.next().ok_or(Ipv4Error)?;
        let prefix_len = parts.next().map(|s| s.parse().ok()).flatten();
        match prefix_len {
            Some(prefix_len) if prefix_len > 32 => return Err(Ipv4Error),
            _ => (),
        }
        if parts.next().is_some() {
            return Err(Ipv4Error);
        }
        let ip: Ipv4Addr = ip.parse().map_err(|_| Ipv4Error)?;
        let ip = ip.octets();
        if let Some(prefix_len) = prefix_len {
            let network = ipv4_network_address(ip, prefix_len);
            if ip != network {
                return Err(Ipv4Error);
            }
        }
        Ok(Ipv4Target {
            ip,
            prefix_len,
        })
    }
}
