use crate::{Msp, MspErr};
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

pub fn create_tcp_socket(conn_msp: &Msp) -> Result<TcpStream, MspErr> {
    Ok(TcpStream::connect(conn_msp)?)
}

pub fn create_udp_socket(ip: Ipv4Addr, port: u16) -> Result<UdpSocket, MspErr> {
    Ok(UdpSocket::bind((ip, port))?)
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
