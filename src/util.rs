use crate::MspErr;
use std::{
    net::{Ipv4Addr, TcpStream, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn get_server_current_time() -> Result<u64, MspErr> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(t) => {
            let tm = t.as_millis();

            // Time should not exceed `u64::MAX`
            match tm.cmp(&(u64::MAX as u128)) {
                std::cmp::Ordering::Greater => {
                    return Err(MspErr::InternalErr(format!(
                        "Current time is too large: {}",
                        tm
                    )));
                }
                _ => Ok(tm as u64),
            }
        }
        Err(err) => {
            return Err(MspErr::InternalErr(format!("{}", err)));
        }
    }
}

pub fn create_tcp_socket(conn_str: String) -> Result<TcpStream, MspErr> {
    Ok(TcpStream::connect(conn_str)?)
}

pub fn create_udp_socket(ip: Ipv4Addr, port: u16) -> Result<UdpSocket, MspErr> {
    Ok(UdpSocket::bind((ip, port))?)
}

pub fn is_valid_port(port: u16) -> bool {
    return port >= 1024;
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
