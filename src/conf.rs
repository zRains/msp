use crate::{
    query, server, BedrockServer, LegacyBetaServer, LegacyServer, MspErr, NettyServer, QueryBasic,
    QueryFull, Server,
};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs},
    time::Duration,
};

/// Main struct used for configuring the connection.
#[derive(Debug, Clone)]
pub struct Conf {
    pub host: String,
    pub port: u16,
    pub socket_conf: SocketConf,
}

#[derive(Debug, Clone)]
pub struct SocketConf {
    pub read_time_out: Option<Duration>,
    pub write_timeout: Option<Duration>,
    pub rep_udp_ipv4: Ipv4Addr,
    pub rep_udp_port: u16,
}

impl Default for SocketConf {
    fn default() -> Self {
        Self {
            read_time_out: None,
            write_timeout: None,
            rep_udp_ipv4: Ipv4Addr::UNSPECIFIED,
            rep_udp_port: 5000,
        }
    }
}

impl ToSocketAddrs for Conf {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (&*self.host, self.port).to_socket_addrs()
    }
}

impl Conf {
    pub fn create(host: &str) -> Self {
        Self {
            host: host.trim().into(),
            port: 25565,
            socket_conf: SocketConf::default(),
        }
    }

    pub fn create_with_port(host: &str, port: u16) -> Result<Self, MspErr> {
        Ok(Self {
            host: host.trim().into(),
            port,
            socket_conf: SocketConf::default(),
        })
    }

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

        Ok(Self {
            host: socket_addr.ip().to_string(),
            port: socket_addr.port(),
            socket_conf: SocketConf::default(),
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
        server::get_beta_legacy_server_status(&self)
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
