mod error;
mod query;
mod server;
mod util;
mod varint;

pub use error::MspErr;
pub use query::{QueryBasic, QueryFull};
use serde::Serialize;
pub use server::{BedrockServer, LanServer, LegacyBetaServer, LegacyServer, NettyServer, Server};
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};
use util::{create_tcp_socket, is_valid_port};

/// Msp config struct
#[derive(Serialize)]
pub struct Msp {
    host: String,
    port: u16,
}

impl ToSocketAddrs for Msp {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (&*self.host, self.port).to_socket_addrs()
    }
}

impl std::fmt::Display for Msp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

impl Msp {
    /// Create Msp with default port(25565).
    pub fn create(host: &str) -> Self {
        Self {
            host: host.trim().into(),
            port: 25565,
        }
    }

    /// Create Msp using a custom port.
    pub fn create_with_port(host: &str, port: u16) -> Result<Self, MspErr> {
        if !is_valid_port(port) {
            return Err(MspErr::DataErr(format!(
                "Invalid port: {}, expected between 1024 and 65535",
                port
            )));
        }

        Ok(Self {
            host: host.trim().into(),
            port,
        })
    }

    /// Create Msp using an addr.
    pub fn create_from_str(addr: &str) -> Result<Self, MspErr> {
        let socket_addr = match addr.parse::<SocketAddrV4>() {
            Ok(parsed_addr) => parsed_addr,
            Err(err) => {
                return Err(MspErr::DataErr(format!(
                    "{} can not parse into socket addr, reason: {}",
                    addr,
                    err.to_string()
                )));
            }
        };

        Ok(Msp {
            host: socket_addr.ip().to_string(),
            port: socket_addr.port(),
        })
    }

    pub fn get_server_status(&self) -> Result<Server, MspErr> {
        server::get_server_status(self)
    }

    pub fn get_netty_server_status(&self) -> Result<NettyServer, MspErr> {
        server::get_netty_server_status(self)
    }

    pub fn get_legacy_server_status(&self) -> Result<LegacyServer, MspErr> {
        server::get_legacy_server_status(self)
    }

    pub fn get_beta_legacy_server_status(&self) -> Result<LegacyBetaServer, MspErr> {
        server::get_beta_legacy_server_status(self)
    }

    pub fn get_lan_server_status() -> Result<Vec<LanServer>, MspErr> {
        server::get_lan_server_status()
    }

    pub fn query(&self) -> Result<QueryBasic, MspErr> {
        query::query_basic_status(self)
    }

    pub fn query_full(&self) -> Result<QueryFull, MspErr> {
        query::query_full_status(self)
    }

    pub fn get_bedrock_server_status(&self) -> Result<BedrockServer, MspErr> {
        server::get_bedrock_server_status(self)
    }
}
