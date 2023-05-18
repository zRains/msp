use serde::Serialize;

use crate::{
    util::{create_udp_socket, UdpReader},
    Msp, MspErr,
};
use std::net::Ipv4Addr;

const MAGIC_BYTES: &[u8] = &[
    0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD, 0x12, 0x34, 0x56, 0x78,
];

#[derive(Serialize, Debug)]
pub struct BedrockServer {
    edition: String,
    motd_line_1: String,
    protocol_version: i32,
    version_name: String,
    online_players: i32,
    max_players: i32,
    server_id: String,
    motd_line_2: String,
    game_mode: String,
    game_mode_id: u8,
    port_ipv4: u16,
    port_ipv6: u16,
}

impl std::fmt::Display for BedrockServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

pub fn get_bedrock_server_status(msp: &Msp) -> Result<BedrockServer, MspErr> {
    let socket = create_udp_socket(Ipv4Addr::UNSPECIFIED, 8000)?;

    let packet = [
        // Packet ID
        &[0x01],
        // Time
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].as_slice(),
        // MAGIC
        MAGIC_BYTES,
    ]
    .concat();

    socket.send_to(packet.as_slice(), msp)?;

    let mut udp_reader = UdpReader::create_with_idx(socket, 0);

    match udp_reader.read_bufs(1)?.get(0) {
        Some(&first_buf) if first_buf != 0x1C => {
            return Err(MspErr::DataErr(format!(
                "Packet response excepted start with: 0x1C, but got: 0x{:02X}",
                first_buf
            )));
        }
        Some(_) | None => {}
    };

    // Drop time data(8 bytes)
    udp_reader.set_current_idx_forward(8);

    let _server_guid = udp_reader.read_bufs(8)?;
    let _magic_bytes = udp_reader.read_bufs(16)?;
    let server_info_len = match udp_reader.read_bufs(2)?.try_into() {
        Ok(len) => u16::from_be_bytes(len) as usize,
        Err(_) => {
            return Err(MspErr::DataErr("Cannot convert to u16.".into()));
        }
    };
    let server_info_buf = udp_reader.read_bufs(server_info_len)?;
    let server_info = String::from_utf8_lossy(server_info_buf.as_slice());

    println!("{:?}", server_info);

    let server_info_split = server_info.split(";").collect::<Vec<_>>();

    if server_info_split.len() < 10 {
        return Err(MspErr::DataErr(format!(
            "Expected return at least 10 parts of server information, but {}  were obtained.",
            server_info_split.len()
        )));
    }

    Ok(BedrockServer {
        edition: server_info_split[0].into(),
        motd_line_1: server_info_split[1].into(),
        protocol_version: server_info_split[2].parse()?,
        version_name: server_info_split[3].into(),
        online_players: server_info_split[4].parse()?,
        max_players: server_info_split[5].parse()?,
        server_id: server_info_split[6].into(),
        motd_line_2: server_info_split[7].into(),
        game_mode: server_info_split[8].into(),
        game_mode_id: server_info_split[9].parse()?,
        port_ipv4: if let Some(&p4) = server_info_split.get(10) {
            p4.parse()?
        } else {
            msp.port
        },
        port_ipv6: if let Some(&p6) = server_info_split.get(11) {
            p6.parse()?
        } else {
            0
        },
    })
}
