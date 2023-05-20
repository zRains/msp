use crate::{conf::Conf, MspErr, SocketConf};
use std::{
    net::{TcpStream, UdpSocket},
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
                        "Failed to obtain current time. It should not exceed u64::MAX, but got: {}",
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

pub fn create_tcp_socket(conf: &Conf) -> Result<TcpStream, MspErr> {
    let socket = TcpStream::connect(conf)?;

    socket.set_read_timeout(conf.socket_conf.read_time_out)?;
    socket.set_write_timeout(conf.socket_conf.write_timeout)?;

    Ok(socket)
}

pub fn create_udp_socket(socket_conf: &SocketConf) -> Result<UdpSocket, MspErr> {
    let socket = UdpSocket::bind((socket_conf.rep_udp_ipv4, socket_conf.rep_udp_port))?;

    socket.set_read_timeout(socket_conf.read_time_out)?;
    socket.set_write_timeout(socket_conf.write_timeout)?;

    Ok(socket)
}

pub fn is_valid_port(port: u16) -> bool {
    return port >= 1024;
}

pub fn bufs_to_utf16_str(bufs: &[u8]) -> Result<String, MspErr> {
    if bufs.len() % 2 != 0 {
        return Err(MspErr::DataErr(format!(
            "Conversion from UTF-16 to string failed. Expected length to be even, but got: {}",
            bufs.len()
        )));
    }

    Ok(String::from_utf16_lossy(
        bufs.chunks(2)
            .map(|x| u16::from_be_bytes([x[0], x[1]]))
            .collect::<Vec<_>>()
            .as_slice(),
    ))
}

pub struct UdpReader {
    socket: UdpSocket,
    current_idx: usize,
}

impl UdpReader {
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

    pub fn read_bufs(&mut self, mut size: usize) -> Result<Vec<u8>, MspErr> {
        let mut buf_vec = Vec::new();

        while size > 0 {
            buf_vec.push(self.read(true)?);
            size -= 1;
        }

        Ok(buf_vec)
    }

    pub fn read_nt_str(&mut self) -> Result<String, MspErr> {
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

    pub fn read_nt_str_group(&mut self) -> Result<Vec<String>, MspErr> {
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

    pub fn read_nt_kv(&mut self) -> Result<(String, String), MspErr> {
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
