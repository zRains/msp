use crate::{util::create_udp_socket, Msp, MspErr};
use serde::Serialize;
use std::net::{Ipv4Addr, UdpSocket};

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

fn send_query_request(msp: &Msp, full_query: bool) -> Result<QueryReader, MspErr> {
    let udp_socket = create_udp_socket(Ipv4Addr::UNSPECIFIED, 8000)?;
    let mut bufs = [0u8; 17];
    // Construct init packet
    //
    // Magic num: 0xFE, 0xFD
    // Type: 0x09 - for handshake, 0x00 - for status request
    // Session ID: for convenience, set the session_id to 1([0x00, 0x00, 0x00, 0x01])
    let init_packet: &mut [u8] = &mut [0xFE, 0xFD, 0x09, 0x00, 0x00, 0x00, 0x01];

    udp_socket.connect(msp.to_string())?;
    udp_socket.send(init_packet)?;
    udp_socket.recv(&mut bufs)?;

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
    udp_socket.send(
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
    udp_socket.peek(&mut bufs)?;

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
            Ok(QueryReader::create_with_idx(udp_socket, 5))
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
pub fn query_basic_status(msp: &Msp) -> Result<QueryBasic, MspErr> {
    let mut nt_str_reader = send_query_request(msp, false)?;

    Ok(QueryBasic {
        motd: nt_str_reader.read_str()?,
        game_type: nt_str_reader.read_str()?,
        map: nt_str_reader.read_str()?,
        numplayers: nt_str_reader.read_str()?,
        maxplayers: nt_str_reader.read_str()?,
        hostport: nt_str_reader.read_port()?,
        hostip: nt_str_reader.read_str()?,
    })
}

/// Get full [status](https://wiki.vg/Query#Full_stat)
pub fn query_full_status(msp: &Msp) -> Result<QueryFull, MspErr> {
    let mut nt_str_reader = send_query_request(msp, true)?;

    // Drop meaningless byte padding
    nt_str_reader.set_current_idx_forward(11);

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
        hostname: nt_str_reader.read_kv()?.1,
        gametype: nt_str_reader.read_kv()?.1,
        game_id: nt_str_reader.read_kv()?.1,
        version: nt_str_reader.read_kv()?.1,
        plugins: resolve_plugin(nt_str_reader.read_kv()?.1)?,
        map: nt_str_reader.read_kv()?.1,
        numplayers: nt_str_reader.read_kv()?.1,
        maxplayers: nt_str_reader.read_kv()?.1,
        hostport: nt_str_reader.read_kv()?.1,
        hostip: nt_str_reader.read_kv()?.1,
        players: {
            // Because there are two null-terminated tokens at the end of the KV section,
            // only one was consumed previously.
            nt_str_reader.set_current_idx_forward(10 + 1);
            nt_str_reader.read_str_group()?
        },
    })
}

pub struct QueryReader {
    socket: UdpSocket,
    current_idx: usize,
}

impl QueryReader {
    pub fn create_with_idx(socket: UdpSocket, current_idx: usize) -> Self {
        Self {
            socket,
            current_idx,
        }
    }

    #[allow(dead_code)]
    pub fn set_current_idx(&mut self, idx: usize) {
        self.current_idx = idx;
    }

    pub fn set_current_idx_forward(&mut self, idx: usize) {
        self.current_idx += idx;
    }

    #[allow(dead_code)]
    pub fn set_current_idx_retreat(&mut self, idx: usize) -> Result<(), MspErr> {
        if self.current_idx < idx {
            return Err(MspErr::DataErr(format!(
                "retreat({}) cannot be greater than current index({})",
                idx, self.current_idx
            )));
        }

        self.current_idx -= idx;

        Ok(())
    }

    pub fn read(&mut self, consume: bool) -> Result<u8, MspErr> {
        let mut bufs = vec![0u8; self.current_idx + 1];

        match self.socket.peek(&mut bufs) {
            Ok(_) => {
                if consume {
                    self.current_idx += 1;
                }

                match bufs.last() {
                    Some(&buf) => Ok(buf),
                    None => {
                        return Err(MspErr::DataErr("Incomplete data".into()));
                    }
                }
            }
            Err(err) => Err(MspErr::IoErr(err)),
        }
    }

    pub fn read_port(&mut self) -> Result<u16, MspErr> {
        Ok(u16::from_be_bytes([self.read(true)?, self.read(true)?]))
    }

    pub fn read_str(&mut self) -> Result<String, MspErr> {
        let mut result = Vec::new();

        loop {
            let buf = self.read(true)?;
            // Compatible with special characters: § © ®...
            //
            // FIXME This is only intended to address situations where the server.properties
            // file is not properly formatted, and it does not cover all special characters
            // larger than one byte. For more details, please refer to
            // [UTF-8 encoding table and Unicode characters](https://www.utf8-chartable.de/unicode-utf8-table.pl)
            match buf {
                // Check the Null-terminated string
                0x00 => break,
                special_buf
                    if special_buf >= 0x80
                        && 0xBF >= special_buf
                        && result.last() != Some(&0xC2) =>
                {
                    result.append(&mut vec![0xC2, special_buf]);
                }
                common_buf => result.push(common_buf),
            }
        }

        Ok(String::from_utf8_lossy(result.as_slice()).into())
    }

    pub fn read_str_group(&mut self) -> Result<Vec<String>, MspErr> {
        let mut result = Vec::new();
        let mut str_group = Vec::<String>::new();

        loop {
            let buf = self.read(true)?;

            match buf {
                0x00 => {
                    let next_buf = self.read(true)?;

                    if next_buf == 0x00 {
                        break;
                    }

                    str_group.push(String::from_utf8_lossy(result.as_slice()).into());
                    result.clear();
                    result.push(next_buf);
                }
                common_buf => result.push(common_buf),
            }
        }

        Ok(str_group)
    }

    pub fn read_kv(&mut self) -> Result<(String, String), MspErr> {
        let mut result = Vec::new();
        let mut kv: (Option<String>, Option<String>) = (None, None);

        loop {
            let buf = self.read(true)?;

            match buf {
                0x00 => {
                    if kv.0.is_none() {
                        kv.0 = Some(String::from_utf8_lossy(result.as_slice()).into());
                        result.clear();
                        continue;
                    }

                    kv.1 = Some(String::from_utf8_lossy(result.as_slice()).into());
                    break;
                }
                special_buf
                    if special_buf >= 0x80
                        && 0xBF >= special_buf
                        && result.last() != Some(&0xC2) =>
                {
                    result.append(&mut vec![0xC2, special_buf]);
                }
                common_buf => result.push(common_buf),
            }
        }

        Ok((kv.0.unwrap_or("".into()), kv.1.unwrap_or("".into())))
    }
}
