use crate::{share::create_udp_socket, MspErr, SocketConf};
use serde::Serialize;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::mpsc,
};

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 2, 60);
const MULTICAST_PORT: u16 = 4445;
const BROADCAST_MUST_CONTAIN: [&'static str; 4] = ["[MOTD]", "[/MOTD]", "[AD]", "[/AD]"];

/// LAN server info structure.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct LanServer {
    /// SocketAddrV4 information for the target server from `recv_from`.
    pub addr: SocketAddrV4,
    /// MOTD of the target server.
    pub motd: String,
    /// Open port of the target server.
    pub port: u16,
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

/// Get the host information of other open servers in the current LAN.
///
/// Currently, it only prints the host information cyclically, and does not return [LanServer] information.
/// # TODO Get host information for a period of time by passing in duration control.
///
/// # Example
///
/// ```no_run
/// use msp::{get_lan_server_status, MspErr, SocketConf};
///
/// fn main() -> Result<(), MspErr> {
///    let info =  get_lan_server_status(&SocketConf::default())?;
///
///     Ok(())
/// }
/// ```
pub fn get_lan_server_status(
    socket_conf: &SocketConf,
) -> Result<(impl Fn(), mpsc::Receiver<Result<Option<LanServer>, MspErr>>), MspErr> {
    let mut buffer = [0u8; 256];
    let (tx, rx) = mpsc::channel::<Result<Option<LanServer>, MspErr>>();
    let (t_sender, t_receiver) = mpsc::channel::<()>();
    let socket = create_udp_socket(&SocketConf {
        rep_udp_port: MULTICAST_PORT,
        ..socket_conf.clone()
    })?;

    socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;

    std::thread::spawn(move || {
        let send_err = |err: MspErr| {
            tx.send(Err(err))
                .expect("An error occurred while sending an error message");
        };

        'socket_receive_loop: loop {
            match t_receiver.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                    socket
                        .leave_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)
                        .expect("An error occurred while leaving multicast");

                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            let src_addr = match socket.recv_from(&mut buffer) {
                Ok((_, addr)) => addr,
                Err(err) => match err.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        tx.send(Ok(None))
                            .expect("An error occurred while sending an None message");

                        continue 'socket_receive_loop;
                    }
                    _ => {
                        send_err(MspErr::IoErr(err));

                        break 'socket_receive_loop;
                    }
                },
            };

            let (motd, port) = match std::str::from_utf8(&buffer) {
                Ok(str) => {
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

                    match abstract_broadcast_message(str) {
                        Ok((m, p)) => (m, p),
                        Err(err) => {
                            send_err(err);

                            return;
                        }
                    }
                }
                Err(_) => {
                    send_err(MspErr::InternalErr(format!(
                        "invalid utf-8: corrupt contents: {:?}",
                        buffer
                    )));

                    return;
                }
            };

            match src_addr {
                std::net::SocketAddr::V4(v4) => {
                    tx.send(Ok(Some(LanServer::create(v4, motd.into(), port))))
                        .expect("An error occurred while sending an LanServer message");
                }
                std::net::SocketAddr::V6(_) => {
                    tx.send(Err(MspErr::NoImpl(format!("Not impl for ipv6."))))
                        .expect("An error occurred while sending an error message");

                    return;
                }
            }
        }
    });

    Ok((
        move || {
            t_sender
                .send(())
                .expect("An error occurred while terminating child thread");
        },
        rx,
    ))
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
