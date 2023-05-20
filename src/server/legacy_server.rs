use crate::{
    conf::Conf,
    share::{bufs_to_utf16_str, create_tcp_socket},
    MspErr,
};
use serde::Serialize;
use std::io::{Read, Write};

/// Legacy server info type.
#[derive(Serialize, Debug)]
pub struct LegacyServer {
    /// Protocol version.
    pub protocol_version: i32,
    /// Server version.
    pub server_version: String,
    /// MOTD of the target server.
    pub motd: String,
    /// Online players.
    pub online_players: i32,
    /// Max players.
    pub max_players: i32,
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

/// Legacy beta server info type.
///
/// A server older than Legacy, used by not many people anymore.
#[derive(Serialize, Debug)]
pub struct LegacyBetaServer {
    /// MOTD of the target server.
    pub motd: String,
    /// Online players.
    pub online_players: i32,
    /// Max players.
    pub max_players: i32,
}

impl std::fmt::Display for LegacyBetaServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}

/// Server [before 1.5](https://wiki.vg/Server_List_Ping#1.4_to_1.5)
pub fn get_legacy_server_status(conf: &Conf) -> Result<LegacyServer, MspErr> {
    let mut socket = create_tcp_socket(conf)?;
    let mut bufs = Vec::<u8>::new();

    socket.write(&mut Vec::from([0xFE, 0x01]))?;
    socket.read_to_end(&mut bufs)?;

    process_legacy_server_bufs(bufs.as_slice())
}

pub fn get_beta_legacy_server_status(conf: &Conf) -> Result<LegacyBetaServer, MspErr> {
    let mut socket = create_tcp_socket(conf)?;
    let mut bufs = [0u8; 1];

    // Prior to Minecraft 1.4, the client only sends 0xFE.
    socket.write(&[0xFE])?;
    socket.read(&mut bufs)?;

    if bufs.get(0) != Some(&0xFF) {
        return Err(MspErr::DataErr(format!(
            "Packet response excepted start with: 0xFF, but got: 0x{:02X}",
            bufs[0]
        )));
    }

    // Read packet data length
    //
    // For unknown reasons (likely due to encoding), this needs to be divided by 2.
    let mut bufs = [0u8; 2];
    socket.read(&mut bufs)?;

    let mut bufs = vec![0u8; (u16::from_be_bytes(bufs) * 2) as usize];

    socket.read(&mut bufs)?;

    let server_info = bufs_to_utf16_str(bufs.as_slice())?;
    let server_split = server_info.split("§").collect::<Vec<_>>();

    build_beta_legacy_server(server_split)
}

fn build_legacy_server(data: Vec<&str>) -> Result<LegacyServer, MspErr> {
    // 5 parts of server information need to be returned.
    //
    // 1.Protocol version (e.g. 47)
    // 2.Minecraft server version (e.g. 1.4.2)
    // 3.Message of the day (e.g. A Minecraft Server)
    // 4.Current player count
    // 5.Max players
    if data.len() != 5 {
        return Err(MspErr::DataErr(format!(
            "Expected return is 5 parts of server information, but {}  were obtained.",
            data.len()
        )));
    }

    Ok(LegacyServer {
        protocol_version: data[0].parse::<i32>()?,
        server_version: data[1].into(),
        motd: data[2].into(),
        online_players: data[3].parse::<i32>()?,
        max_players: data[4].parse::<i32>()?,
    })
}

fn build_beta_legacy_server(data: Vec<&str>) -> Result<LegacyBetaServer, MspErr> {
    // 3 parts of server information need to be returned.
    //
    // 1.MOTD
    // 2.Current player count
    // 3.Max players
    if data.len() != 3 {
        return Err(MspErr::DataErr(format!(
            "Expected return is 3 parts of server information, but {}  were obtained.",
            data.len()
        )));
    }

    Ok(LegacyBetaServer {
        motd: data[2].into(),
        online_players: data[3].parse::<i32>()?,
        max_players: data[4].parse::<i32>()?,
    })
}

pub fn process_legacy_server_bufs(bufs: &[u8]) -> Result<LegacyServer, MspErr> {
    if bufs.get(0) != Some(&0xFF) {
        return Err(MspErr::DataErr(format!(
            "Packet response excepted start with: 0xFF, but got: 0x{:02X}",
            bufs[0]
        )));
    }

    let server_info = bufs_to_utf16_str(&bufs[3..])?;

    if !server_info.starts_with("§1") {
        return Err(MspErr::DataErr(
            "Server info excepted start with: §1.".into(),
        ));
    }

    Ok(build_legacy_server(
        server_info.split("\0").skip(1).collect::<Vec<_>>(),
    )?)
}
