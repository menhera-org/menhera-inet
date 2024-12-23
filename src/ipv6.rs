use std::net::Ipv6Addr;
use std::str::FromStr;
use std::fmt::Display;

type RawIpv6Addr = [u8; 16];

fn ipv6_subnet_mask(prefix_len: u8) -> RawIpv6Addr {
    let mut mask = [0u8; 16];
    for i in 0..(prefix_len / 8) {
        mask[i as usize] = 0xff;
    }
    if prefix_len % 8 != 0 {
        mask[(prefix_len / 8) as usize] = 0xff << (8 - (prefix_len % 8));
    }
    mask
}

/// First address in the network
fn ipv6_network_address(ip: RawIpv6Addr, prefix_len: u8) -> RawIpv6Addr {
    let mask = ipv6_subnet_mask(prefix_len);
    let mut addr = [0; 16];
    for i in 0..16 {
        addr[i] = ip[i] & mask[i];
    }
    addr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv6Error;

impl Display for Ipv6Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IPv6 address error")
    }
}

impl std::error::Error for Ipv6Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv6Target {
    pub(crate) ip: RawIpv6Addr,
    pub(crate) prefix_len: Option<u8>,
}

impl Ipv6Target {
    pub fn new(ip: Ipv6Addr, prefix_len: Option<u8>) -> Result<Self, Ipv6Error> {
        let ip = ip.octets();
        if let Some(prefix_len) = prefix_len {
            if prefix_len > 128 {
                return Err(Ipv6Error);
            }
            let network = ipv6_network_address(ip, prefix_len);
            if ip != network {
                return Err(Ipv6Error);
            }
        }
        Ok(Ipv6Target { ip, prefix_len })
    }

    pub fn ip(&self) -> Ipv6Addr {
        Ipv6Addr::from(self.ip)
    }

    pub fn prefix_len(&self) -> Option<u8> {
        self.prefix_len
    }

    pub fn is_net(&self) -> bool {
        self.prefix_len.is_some()
    }

    pub fn net(&self) -> Option<ipnet::Ipv6Net> {
        self.prefix_len.map(|prefix_len| {
            let network = ipv6_network_address(self.ip, prefix_len);
            ipnet::Ipv6Net::new(Ipv6Addr::from(network), prefix_len).unwrap()
        })
    }
}

impl Display for Ipv6Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ip = Ipv6Addr::from(self.ip);
        match self.prefix_len {
            Some(prefix_len) => write!(f, "{}/{}", ip, prefix_len),
            None => write!(f, "{}", ip),
        }
    }
}

impl FromStr for Ipv6Target {
    type Err = Ipv6Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');
        let ip = parts.next().ok_or(Ipv6Error)?;
        let prefix_len = parts.next().map(|s| s.parse().ok()).flatten();
        match prefix_len {
            Some(prefix_len) if prefix_len > 128 => return Err(Ipv6Error),
            _ => (),
        }
        if parts.next().is_some() {
            return Err(Ipv6Error);
        }
        let ip: Ipv6Addr = ip.parse().map_err(|_| Ipv6Error)?;
        let ip = ip.octets();
        if let Some(prefix_len) = prefix_len {
            let network = ipv6_network_address(ip, prefix_len);
            if ip != network {
                return Err(Ipv6Error);
            }
        }
        Ok(Ipv6Target {
            ip,
            prefix_len,
        })
    }
}
