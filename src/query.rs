use crate::{
    conf::Conf,
    share::{create_udp_socket, UdpReader},
    MspErr,
};
use serde::Serialize;
use std::net::Ipv4Addr;

const TOKEN_MASK: i32 = 0x0F0F0F0F;
const PENDDING_BUFS: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

#[derive(Serialize, Debug)]
pub struct QueryBasic {
    motd: String,
    game_type: String,
    map: String,
    numplayers: String,
    maxplayers: String,
    hostport: u16,
    hostip: String,
}

impl std::fmt::Display for QueryBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

#[derive(Serialize, Debug)]
pub struct QueryFull {
    hostname: String,
    gametype: String,
    game_id: String,
    version: String,
    plugins: Vec<ModPlugin>,
    map: String,
    numplayers: String,
    maxplayers: String,
    hostport: String,
    hostip: String,
    players: Vec<String>,
}

impl std::fmt::Display for QueryFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

#[derive(Serialize, Debug)]
pub struct ModPlugin {
    mod_name: String,
    plugins: Vec<String>,
}

impl std::fmt::Display for ModPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

fn send_query_request(conf: &Conf, full_query: bool) -> Result<UdpReader, MspErr> {
    let socket = create_udp_socket(&conf.socket_conf)?;
    let mut bufs = [0u8; 17];
    // Construct init packet
    //
    // Magic num: 0xFE, 0xFD
    // Type: 0x09 - for handshake, 0x00 - for status request
    // Session ID: for convenience, set the session_id to 1([0x00, 0x00, 0x00, 0x01])
    let init_packet: &mut [u8] = &mut [0xFE, 0xFD, 0x09, 0x00, 0x00, 0x00, 0x01];

    socket.connect(conf)?;
    socket.send(init_packet)?;
    socket.recv(&mut bufs)?;

    let (session_id, token) = get_challenge_token(&mut bufs)?;

    if session_id != 1 {
        return Err(MspErr::DataErr(format!(
            "Response session_id({}) is inconsistent with the client(1).",
            session_id
        )));
    }

    // Send stat request [packet](https://wiki.vg/Query#Request_2)
    //
    // Should change the type into 0x00
    init_packet[2] = 0x00;
    // Reset to 5 bytes
    let mut bufs = [0u8; 5];

    // Full query except the payload must be padded to 8 bytes.
    // Sending [0x00, 0x00, 0x00, 0x00] at the end works.
    socket.send(
        &[
            init_packet,
            token.to_be_bytes().as_slice(),
            &mut (match full_query {
                true => PENDDING_BUFS.as_slice(),
                false => [].as_slice(),
            }),
        ]
        .concat(),
    )?;
    // Use peek instand of recv cause unknown response packet size
    socket.peek(&mut bufs)?;

    if bufs.get(0) != Some(&0x00) {
        return Err(MspErr::DataErr(format!(
            "Response packet invalid, expected start with 0x00, but got: {}",
            bufs[0]
        )));
    }

    match bufs[1..].try_into() {
        Ok(bs) => {
            let receive_token = i32::from_be_bytes(bs) & TOKEN_MASK;

            if receive_token != 1 {
                return Err(MspErr::DataErr(format!(
                    "Query session ID mismatch, expected: {}, but got: {}",
                    token, receive_token
                )));
            }

            // Set Reader index to 5. We don't need Type and Session ID anymore.
            Ok(UdpReader::create_with_idx(socket, 5))
        }
        Err(err) => Err(MspErr::InternalErr(err.to_string())),
    }
}

/// Process query handshake response [packet](https://wiki.vg/Query#Response),
/// and get challenge token.
fn get_challenge_token(mut bufs: &mut [u8]) -> Result<(i32, i32), MspErr> {
    // Remove the 0 element at the end of the array
    let mut buf_len = bufs.len();
    while let Some(&0) = bufs.last() {
        bufs = &mut bufs[..buf_len - 1];
        buf_len = bufs.len();
    }

    if buf_len <= 5 || buf_len > 17 {
        return Err(MspErr::DataErr(format!(
            "Query handshake response packet len invalid, current len: {}",
            buf_len
        )));
    }

    if bufs.get(0) != Some(&0x09) {
        return Err(MspErr::DataErr(format!(
            "Query handshake response packet invalid, expected start with 0x90, but got: {}",
            bufs[0]
        )));
    }

    let session_id = i32::from_be_bytes(match bufs[1..5].try_into() {
        Ok(id) => id,
        Err(err) => {
            return Err(MspErr::DataErr(format!(
                "Can not parse bufs into session_id, bufs: {:?}, reason: {}.",
                bufs[1..5].to_vec(),
                err.to_string()
            )));
        }
    }) & TOKEN_MASK;

    match std::str::from_utf8(&bufs[5..]) {
        Ok(token_str) => match token_str.parse::<i32>() {
            Ok(token) => Ok((session_id, token)),
            Err(err) => Err(MspErr::InternalErr(err.to_string())),
        },
        Err(err) => Err(MspErr::InternalErr(err.to_string())),
    }
}

/// Get basic [status](https://wiki.vg/Query#Basic_stat)
pub fn query_basic_status(conf: &Conf) -> Result<QueryBasic, MspErr> {
    let mut udp_reader = send_query_request(conf, false)?;

    Ok(QueryBasic {
        motd: udp_reader.read_nt_str()?,
        game_type: udp_reader.read_nt_str()?,
        map: udp_reader.read_nt_str()?,
        numplayers: udp_reader.read_nt_str()?,
        maxplayers: udp_reader.read_nt_str()?,
        hostport: u16::from_be_bytes([udp_reader.read(true)?, udp_reader.read(true)?]),
        hostip: udp_reader.read_nt_str()?,
    })
}

/// Get full [status](https://wiki.vg/Query#Full_stat)
pub fn query_full_status(conf: &Conf) -> Result<QueryFull, MspErr> {
    let mut udp_reader = send_query_request(conf, true)?;

    // Drop meaningless byte padding
    udp_reader.set_current_idx_forward(11);

    // Plugin format: [SERVER_MOD_NAME[: PLUGIN_NAME(; PLUGIN_NAME...)]]
    //
    // TODO So far, there have been no cases of multiple mod plugins.
    // Therefore, for now, we are considering a single mod plugin.
    let resolve_plugin = |plugin_str: String| -> Result<Vec<ModPlugin>, MspErr> {
        if plugin_str.len() == 0 {
            return Ok(vec![]);
        }

        let mut result = Vec::new();
        let plugin_collection = plugin_str.split(":").map(|x| x.trim()).collect::<Vec<_>>();

        match plugin_collection.len() {
            2 => {
                result.push(ModPlugin {
                    mod_name: plugin_collection[0].into(),
                    plugins: plugin_collection[1]
                        .split(";")
                        .map(|x| x.trim().into())
                        .collect::<Vec<_>>(),
                });
            }
            1 => {
                result.push(ModPlugin {
                    mod_name: plugin_collection[0].into(),
                    plugins: vec![],
                });
            }
            _ => {
                return Err(MspErr::DataErr("Multiple mod plugin formats have been detected. Please submit the server address to the issues section to help us improve.".into()));
            }
        };

        Ok(result)
    };

    Ok(QueryFull {
        hostname: udp_reader.read_nt_kv()?.1,
        gametype: udp_reader.read_nt_kv()?.1,
        game_id: udp_reader.read_nt_kv()?.1,
        version: udp_reader.read_nt_kv()?.1,
        plugins: resolve_plugin(udp_reader.read_nt_kv()?.1)?,
        map: udp_reader.read_nt_kv()?.1,
        numplayers: udp_reader.read_nt_kv()?.1,
        maxplayers: udp_reader.read_nt_kv()?.1,
        hostport: udp_reader.read_nt_kv()?.1,
        hostip: udp_reader.read_nt_kv()?.1,
        players: {
            // Because there are two null-terminated tokens at the end of the KV section,
            // only one was consumed previously.
            udp_reader.set_current_idx_forward(10 + 1);
            udp_reader.read_nt_str_group()?
        },
    })
}
