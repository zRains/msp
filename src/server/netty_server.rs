use super::{process_legacy_server_bufs, LegacyServer};
use crate::{conf::Conf, share::create_tcp_socket, MspErr};
use std::io::{Read, Write};

/// The alias of [LegacyServer] is identical in content.
pub type NettyServer = LegacyServer;

pub fn get_netty_server_status(conf: &Conf) -> Result<NettyServer, MspErr> {
    let mut socket = create_tcp_socket(conf)?;
    let mut packet_data = Vec::<u8>::new();
    let host_u16 = conf.host.encode_utf16().collect::<Vec<_>>();

    packet_data.append(&mut vec![
        0xFE, 0x01, 0xFA, 0x00, 0x0B, 0x00, 0x4D, 0x00, 0x43, 0x00, 0x7C, 0x00, 0x50, 0x00, 0x69,
        0x00, 0x6E, 0x00, 0x67, 0x00, 0x48, 0x00, 0x6F, 0x00, 0x73, 0x00, 0x74,
    ]);
    packet_data.append(&mut ((7 + host_u16.len()) as u16).to_be_bytes().to_vec());
    // Protocol version
    packet_data.push(0x50);
    packet_data.append(&mut (conf.host.len() as u16).to_be_bytes().to_vec());
    packet_data.append(
        &mut host_u16
            .iter()
            .map(|x| x.to_be_bytes().to_vec())
            .flatten()
            .collect(),
    );
    // Server port
    packet_data.append(&mut (conf.port as u32).to_be_bytes().to_vec());
    socket.write(&mut vec![0xFE, 0x01])?;

    let mut bufs = Vec::new();

    socket.read_to_end(&mut bufs)?;

    process_legacy_server_bufs(bufs.as_slice())
}
