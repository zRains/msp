use crate::{share::create_udp_socket, MspErr, SocketConf};
use serde::Serialize;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
    time::{Duration, SystemTime},
};

// Unsigned Short ref to u16 in rust.

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 2, 60);
const MULTICAST_PORT: u16 = 4445;
const MAXIMUM_SERVERS: usize = 100;
const BROADCAST_MUST_CONTAIN: [&'static str; 4] = ["[MOTD]", "[/MOTD]", "[AD]", "[/AD]"];
const SERVER_OFFLINE_TIMEOUT: u64 = 2000;
const STORE_CHECK_INTERVAL: u64 = 4000;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct LanServer {
    addr: SocketAddrV4,
    motd: String,
    port: u16,
}

impl LanServer {
    fn create(addr: SocketAddrV4, motd: String, port: u16) -> Self {
        Self { addr, motd, port }
    }
}

impl std::hash::Hash for LanServer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

impl std::fmt::Display for LanServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

pub fn get_lan_server_status(socket_conf: &SocketConf) -> Result<Vec<LanServer>, MspErr> {
    let mut lan_server_map = HashMap::<LanServer, SystemTime>::new();
    let mut buffer = [0u8; 256];
    let mut outer_loop_time_starter = SystemTime::now();
    let socket = create_udp_socket(&SocketConf {
        rep_udp_port: MULTICAST_PORT,
        ..socket_conf.clone()
    })?;

    socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
    socket.set_read_timeout(Some(Duration::from_millis(SERVER_OFFLINE_TIMEOUT)))?;

    'socket_receive_loop: loop {
        let inner_loop_time_starter = SystemTime::now();

        // TODO more info
        println!("map: {:?}", lan_server_map);

        if inner_loop_time_starter
            .duration_since(outer_loop_time_starter)?
            .as_millis() as u64
            > STORE_CHECK_INTERVAL
        {
            lan_server_map.retain(|_, &mut time| {
                match inner_loop_time_starter.duration_since(time) {
                    Ok(t) => t.as_millis() as u64 <= SERVER_OFFLINE_TIMEOUT,
                    _ => false,
                }
            });

            outer_loop_time_starter = inner_loop_time_starter;
        }

        let src_addr = match socket.recv_from(&mut buffer) {
            Ok((_, addr)) => addr,
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => {
                    continue 'socket_receive_loop;
                }
                _ => {
                    return Err(MspErr::IoErr(err));
                }
            },
        };
        let (motd, port) = if let Ok(str) = std::str::from_utf8(&buffer) {
            // Check broadcast message is valid.
            //
            // If is not valid, it will continue outer loop immediately,
            // or throw an `MspErr` Error in strict mod(not impl):
            //
            // TODO Impl strict mod
            for str_must_contain in BROADCAST_MUST_CONTAIN {
                if !str.contains(str_must_contain) {
                    continue 'socket_receive_loop;
                }
            }

            if lan_server_map.len() == MAXIMUM_SERVERS {
                continue 'socket_receive_loop;
            }

            abstract_broadcast_message(str)?
        } else {
            return Err(MspErr::InternalErr(format!(
                "invalid utf-8: corrupt contents: {:?}",
                buffer
            )));
        };

        match src_addr {
            std::net::SocketAddr::V4(v4) => {
                lan_server_map.insert(LanServer::create(v4, motd.into(), port), SystemTime::now());
            }
            std::net::SocketAddr::V6(_) => {
                return Err(MspErr::NoImpl(format!("Not impl for ipv6.")));
            }
        }
    }
}

fn abstract_broadcast_message(message: &str) -> Result<(&str, u16), MspErr> {
    let motd_start = message.find(BROADCAST_MUST_CONTAIN[0]).unwrap();
    let motd_end = message.find(BROADCAST_MUST_CONTAIN[1]).unwrap();
    let port_start = message.find(BROADCAST_MUST_CONTAIN[2]).unwrap();
    let port_end = message.find(BROADCAST_MUST_CONTAIN[3]).unwrap();

    match (
        &message[motd_start + BROADCAST_MUST_CONTAIN[0].len()..motd_end],
        &message[port_start + BROADCAST_MUST_CONTAIN[2].len()..port_end],
    ) {
        (motd, port) => match port.parse::<u16>() {
            Ok(p) => Ok((motd, p)),
            Err(_) => Err(MspErr::DataErr(format!(
                "Can not parse {} into port number",
                port
            ))),
        },
    }
}
