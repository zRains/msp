use crate::{util::create_tcp_socket, Msp, MspErr};
use serde::Serialize;
use std::io::{Read, Write};

#[derive(Serialize)]
pub struct LegacyServer {
    protocol_version: i32,
    server_version: String,
    motd: String,
    online_palyers: i32,
    max_players: i32,
}

impl std::fmt::Display for LegacyServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

/// Server [before 1.5](https://wiki.vg/Server_List_Ping#1.4_to_1.5)
pub fn get_legacy_server_status(msp: &Msp) -> Result<LegacyServer, MspErr> {
    let mut socket = create_tcp_socket(msp.to_string())?;
    let mut bufs = Vec::<u8>::new();

    socket.write(&mut Vec::from([0xFE, 0x01]))?;

    socket.read_to_end(&mut bufs)?;
    legacy_server_response_process(bufs)
}

pub fn legacy_server_response_process(bufs: Vec<u8>) -> Result<LegacyServer, MspErr> {
    if bufs.len() <= 3 || bufs[3..].len() % 2 != 0 {
        return Err(MspErr::DataErr(format!(
            "Server response data len invalid, len: {}",
            bufs.len()
        )));
    }

    if bufs.get(0) != Some(&0xFF) {
        return Err(MspErr::DataErr(format!(
            "Server response data is invalid, it should start with: 0xFF, but got: 0x{:02X}",
            bufs[0]
        )));
    }

    let mut utf16_packet = Vec::<u16>::new();

    for chunk in bufs[3..].chunks(2) {
        utf16_packet.push(u16::from_be_bytes(match chunk.try_into() {
            Ok(data) => data,
            Err(err) => {
                return Err(MspErr::DataErr(format!(
                    "Can not parse [{}] into string, reason: {}",
                    chunk
                        .iter()
                        .map(|c| format!("0x{:02X}", c))
                        .collect::<Vec<_>>()
                        .join(", "),
                    err
                )));
            }
        }))
    }

    match String::from_utf16(&utf16_packet) {
        Ok(res_str) => {
            // the packet is a UTF-16BE string. It begins with two characters: §1
            if res_str.starts_with("§1") {
                let res_split_with_delimiter = res_str.split("\0").skip(1).collect::<Vec<_>>();

                // 5 pieces of information need to be returned.
                //
                // 1.Protocol version (e.g. 47)
                // 2.Minecraft server version (e.g. 1.4.2)
                // 3.Message of the day (e.g. A Minecraft Server)
                // 4.Current player count
                // 5.Max players
                if res_split_with_delimiter.len() != 5 {
                    return Err(MspErr::DataErr(format!(
                        "Server response info len is invalid,it must be 5, but got {}",
                        res_split_with_delimiter.len()
                    )));
                }

                match build_legacy_server(res_split_with_delimiter) {
                    Ok(s) => Ok(s),
                    Err(err) => Err(MspErr::DataErr(format!(
                        "Server response info is can not parse, reason: {}",
                        err
                    ))),
                }
            } else {
                return Err(MspErr::DataErr(format!(
                    "Server response info must start with: §1, but got: {}",
                    &res_str[..2]
                )));
            }
        }
        Err(err) => {
            return Err(MspErr::DataErr(format!(
                "Can not parse response data to string, reason: {}",
                err
            )));
        }
    }
}

fn build_legacy_server(data: Vec<&str>) -> Result<LegacyServer, std::num::ParseIntError> {
    Ok(LegacyServer {
        protocol_version: data[0].parse::<i32>()?,
        server_version: data[1].into(),
        motd: data[2].into(),
        online_palyers: data[3].parse::<i32>()?,
        max_players: data[4].parse::<i32>()?,
    })
}
