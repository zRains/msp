use crate::{
    share::{create_tcp_socket, get_server_current_time},
    varint::{decode_varint_from_socket, encode_varint},
    Conf, MspErr,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

const DEFAULT_SERVER_PORT: u16 = 25565;

/// Regular Server info type.
#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    /// Server version. Includes version name and protocol number.
    pub version: Version,
    /// Server Player info.
    pub players: Players,
    /// Server description, similar to MOTD.
    pub description: Description,
    /// Server icon, base64 encoding.
    pub favicon: String,

    /// Forge plugin server unique information, including mod and channel info.
    #[serde(
        alias = "forgeData",
        rename = "forgeData",
        skip_serializing_if = "Option::is_none"
    )]
    pub forge_data: Option<ForgeData>,

    /// Whether the server enables enforces secure chat, default is false.
    #[serde(
        alias = "enforcesSecureChat",
        rename = "enforcesSecureChat",
        default = "enforces_secure_chat_default"
    )]
    pub enforces_secure_chat: bool,

    /// Server latency, in milliseconds.
    #[serde(default = "ping_default")]
    pub ping: u64,
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<Player>,
}

impl Default for Players {
    fn default() -> Self {
        Players {
            max: 0,
            online: 0,
            sample: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Description {
    pub extra: Vec<DescriptionExtra>,
    pub text: String,
}

impl Default for Description {
    fn default() -> Self {
        Description {
            extra: vec![],
            text: "".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct DescriptionExtra {
    pub color: String,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
    pub obfuscated: bool,
    pub text: String,
    pub extra: Vec<DescriptionExtra>,
}

impl Default for DescriptionExtra {
    fn default() -> Self {
        Self {
            color: "".into(),
            bold: false,
            italic: false,
            underlined: false,
            strikethrough: false,
            obfuscated: false,
            text: "".into(),
            extra: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgeData {
    pub mods: Vec<ForgeMod>,
    pub channels: Vec<ForgeChannel>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ForgeMod {
    #[serde(alias = "modId", rename = "modId")]
    pub mod_id: String,
    pub modmarker: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ForgeChannel {
    pub res: String,
    pub version: String,
    pub required: bool,
}

pub fn get_server_status(conf: &Conf) -> Result<Server, MspErr> {
    let mut socket = create_tcp_socket(conf)?;
    let hand_shake_packet = build_handshake_packet(&conf);
    let status_request_packet = build_status_request_packet();

    socket.write(&hand_shake_packet)?;
    socket.write(&status_request_packet)?;

    // Decode data or data size from response buffers
    //
    // TODO Packet cannot be larger than 2^21, therefore,
    // it is necessary to handle the case of multiple packets.
    let (_p_buf_len, _p_size) = decode_varint_from_socket(&mut socket)?;
    let (_id_buf_len, _id) = decode_varint_from_socket(&mut socket)?;
    let (_d_buf_len, d_size) = decode_varint_from_socket(&mut socket)?;

    let mut data_buffer = Vec::new();

    Read::by_ref(&mut socket)
        .take(d_size as u64)
        .read_to_end(&mut data_buffer)?;

    // Debug block
    //
    // let mut demo_result_file = std::fs::File::create(".demo.json").expect("fail");
    // demo_result_file.write(&data_buffer).unwrap();
    // println!("{:?}", std::str::from_utf8(&data_buffer));

    match std::str::from_utf8(&data_buffer) {
        Ok(str) => match serde_json::from_str::<Server>(str) {
            Ok(mut server) => {
                // Get server ping
                let ping = get_server_ping(&mut socket)?;

                server.ping = ping;
                Ok(server)
            }
            Err(err) => Err(MspErr::DataErr(err.to_string())),
        },
        Err(err) => {
            return Err(MspErr::InternalErr(err.to_string()));
        }
    }
}

/// Build handshake packet buffer.
fn build_handshake_packet(conf: &Conf) -> Vec<u8> {
    let mut packet = Vec::<u8>::new();
    let mut packet_data = Vec::<u8>::new();
    let mut server_addr_bytes = conf.host.as_bytes().to_vec();

    // See protocol version [numbers](https://wiki.vg/Protocol_version_numbers).
    //
    // If the client is pinging to determine what version to use,
    // by convention -1 should be set.
    packet_data.append(&mut encode_varint(-1));
    // Server address
    //
    // UTF-8 string prefixed with its size in bytes as a VarInt.
    packet_data.append(&mut encode_varint(server_addr_bytes.len() as i32));
    packet_data.append(&mut server_addr_bytes);
    // Server port
    packet_data.append(&mut DEFAULT_SERVER_PORT.to_be_bytes().to_vec());
    // Next state, should be 1 for status, but could also be 2 for login.
    packet_data.append(&mut encode_varint(1));

    // Build [packet](https://wiki.vg/Protocol#Packet_format)
    packet.append(&mut encode_varint(1 + packet_data.len() as i32));
    packet.append(&mut encode_varint(0x00));
    packet.append(&mut packet_data);

    packet
}

/// Build status request packet buffer.
fn build_status_request_packet() -> Vec<u8> {
    let mut packet = Vec::<u8>::new();

    // Status Request
    packet.append(&mut encode_varint(1));
    packet.append(&mut encode_varint(0x00));

    packet
}

/// Build ping request packet buffer.
fn build_ping_request_packet() -> Result<(u64, Vec<u8>), MspErr> {
    let mut packet = Vec::<u8>::new();
    let now_millis = get_server_current_time()?;

    packet.append(&mut encode_varint(9));
    packet.push(0x01);
    packet.append(&mut now_millis.to_be_bytes().to_vec());

    Ok((now_millis, packet))
}

fn get_server_ping(socket: &mut TcpStream) -> Result<u64, MspErr> {
    let (req_t, ping_request_packet) = build_ping_request_packet()?;
    let mut time_bytes = [0u8; 8];

    socket.write(&ping_request_packet)?;
    decode_varint_from_socket(socket)?;
    decode_varint_from_socket(socket)?;

    // Why  take 8 buffers?
    //
    // Because server should response the same as sent by the client.
    Read::by_ref(socket).take(8).read(&mut time_bytes)?;

    let receive_t = u64::from_be_bytes(time_bytes);

    if receive_t == req_t {
        let res_t = get_server_current_time()?;

        return Ok(res_t - req_t);
    }

    Err(MspErr::DataErr(format!("Server's response time does not match the sending time(send: {}, receive: {}), indicating that the latency is not reliable.", req_t, receive_t)))
}

/// Set enforces secure chat option to false default
fn enforces_secure_chat_default() -> bool {
    false
}

/// Set ping to 0 default
fn ping_default() -> u64 {
    0
}
