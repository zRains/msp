use crate::{
    query, server, BedrockServer, LegacyBetaServer, LegacyServer, MspErr, NettyServer, QueryBasic,
    QueryFull, Server,
};
use std::{
    net::{Ipv4Addr, SocketAddr, ToSocketAddrs},
    time::Duration,
};

/// Main struct used for configuring the connection.
///
/// By default, the port number for Java Edition is 25565,
/// and for Bedrock Edition (including Pocket Edition), it is 19132.
#[derive(Debug, Clone)]
pub struct Conf {
    /// Server IP address or a domain name.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// See [SocketConf].
    pub socket_conf: SocketConf,
}

/// Additional socket configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketConf {
    /// Set the read timeout for socket.
    pub read_time_out: Option<Duration>,
    /// Set the write timeout for socket.
    pub write_timeout: Option<Duration>,
    /// Specify the address for creating a UDP connection.
    /// The default value is [Ipv4Addr::UNSPECIFIED].
    pub rep_udp_ipv4: Ipv4Addr,
    /// Specify the port for creating a UDP connection.
    /// The default value is 8000.
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
    /// Create a connection configuration using the default port.
    ///
    /// Default port is based on Java Edition(25565), to create a default port based on
    /// Bedrock Edition(19132), use [Conf::create_with_port] to manually specify it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use msp::{Conf, SocketConf};
    /// #
    /// let conf = Conf::create("www.example.com");
    /// #
    /// # assert_eq!(conf.host, "www.example.com");
    /// # assert_eq!(conf.port, 25565);
    /// # assert_eq!(conf.socket_conf, SocketConf::default());
    /// ```
    pub fn create(host: &str) -> Self {
        Self {
            host: host.trim().into(),
            port: 25565,
            socket_conf: SocketConf::default(),
        }
    }

    /// Create a connection configuration using the specified port.
    ///
    /// # Example
    ///
    /// ```
    /// # use msp::{Conf};
    /// #
    /// let conf = Conf::create_with_port("www.example.com", 19132);
    /// #
    /// # assert_eq!(conf.port, 19132);
    /// ```
    pub fn create_with_port(host: &str, port: u16) -> Self {
        Self {
            host: host.trim().into(),
            port,
            socket_conf: SocketConf::default(),
        }
    }

    /// Create a connection configuration by using a string.
    ///
    /// Attempting to split the given string into two parts,
    /// with the first part being the host of the server and
    /// the second part being the port of the server. If the port
    /// cannot be converted to [u16], it will throw a [MspErr] error.
    ///
    /// # Example
    ///
    /// ```
    /// # use msp::{Conf, MspErr};
    /// #
    /// # fn main() -> Result<(), MspErr> {
    ///     let conf = Conf::create_from_str("www.example.com:25565")?;
    /// #
    /// #   assert_eq!(conf.host, "www.example.com");
    /// #   assert_eq!(conf.port, 25565);
    /// #
    /// #   let conf = Conf::create_from_str("25565");
    /// #   assert!(conf.is_err());
    /// #   let conf = Conf::create_from_str("www.example.com:-1");
    /// #   assert!(conf.is_err());
    /// #   Ok(())
    /// # }
    /// ```
    pub fn create_from_str(addr: &str) -> Result<Self, MspErr> {
        let addr_split = addr.split(":").map(|x| x.trim()).collect::<Vec<_>>();

        if addr_split.len() != 2 {
            return Err(MspErr::DataErr(format!(
                "Invalid IPv4 socket address syntax: {}",
                addr
            )));
        }

        match addr_split[1].parse::<u16>() {
            Ok(port) => Ok(Self {
                host: addr_split[0].into(),
                port,
                socket_conf: SocketConf::default(),
            }),
            Err(_) => Err(MspErr::DataErr(format!("Invalid port: {}", addr_split[1]))),
        }
    }

    /// Get info from a modern Java Edition server.
    ///
    /// Using the [Server List Ping](https://wiki.vg/Server_List_Ping#Current_.281.7.2B.29) protocol.
    /// Suitable for Java Edition servers version 1.7 and above. Return type is [Server].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create("www.example.com");
    ///     let info = server.get_bedrock_server_status()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get_server_status(&self) -> Result<Server, MspErr> {
        server::get_server_status(self)
    }

    /// Get info from a legacy Java Edition server.
    ///
    /// This uses a protocol which is compatible with the
    /// client-server protocol as it was before the Netty rewrite.
    /// Suitable for Java Edition servers version 1.6 and above. Return type is [NettyServer].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create("www.example.com");
    ///     let info = server.get_netty_server_status()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get_netty_server_status(&self) -> Result<NettyServer, MspErr> {
        server::get_netty_server_status(self)
    }

    /// Get info from a legacy Java Edition server.
    ///
    /// Suitable for Java Edition servers version 1.4 to 1.5. Return type is [LegacyServer].
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create("www.example.com");
    ///     let info = server.get_legacy_server_status()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get_legacy_server_status(&self) -> Result<LegacyServer, MspErr> {
        server::get_legacy_server_status(self)
    }

    /// Get info from a beta legacy Java Edition server in beta release.
    ///
    /// Suitable for Java Edition servers version beta 1.8 to 1.3.
    /// Return type is [LegacyBetaServer].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create("www.example.com");
    ///     let info = server.get_beta_legacy_server_status()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get_beta_legacy_server_status(&self) -> Result<LegacyBetaServer, MspErr> {
        server::get_beta_legacy_server_status(&self)
    }

    /// Get **basic** info from a modern Java Edition server using the [Query](https://wiki.vg/Query) protocol.
    ///
    /// To use this protocol, you need to enable the enable-query option on the server side.
    /// See [Server Config](https://wiki.vg/Query#Server_Config). Return type is [QueryBasic].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create_with_port("www.example.com", 25565);
    ///     let info = server.query()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn query(&self) -> Result<QueryBasic, MspErr> {
        query::query_basic_status(self)
    }

    /// Get **full** info from a modern Java Edition server using the [Query](https://wiki.vg/Query) protocol.
    ///
    /// To use this protocol, you need to enable the enable-query option on the server side.
    /// See [Server Config](https://wiki.vg/Query#Server_Config). Return type is [QueryFull].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create_with_port("www.example.com", 25565);
    ///     let info = server.query_full()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn query_full(&self) -> Result<QueryFull, MspErr> {
        query::query_full_status(self)
    }

    /// Get info from a modern Bedrock Edition servers using the [RakNet](https://wiki.vg/Raknet_Protocol) protocol
    ///
    /// Suitable for Bedrock Edition servers version 1.16.220(protocol 431) and above.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use msp::{Conf, MspErr};
    ///
    /// fn main() -> Result<(), MspErr> {
    ///     let server = Conf::create_with_port("www.example.com", 19132);
    ///     let info = server.get_bedrock_server_status()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get_bedrock_server_status(&self) -> Result<BedrockServer, MspErr> {
        server::get_bedrock_server_status(self)
    }
}
