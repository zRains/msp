use std::{io::Read, net::TcpStream};

use crate::MspErr;

const SEGMENT_BITS: u32 = 0x7F;
const CHECKER_BIT: u8 = 0x80;

/// Encode the given number as a [VarInt](https://wiki.vg/Protocol#VarInt_and_VarLong).
pub fn encode_varint(num: i32) -> Vec<u8> {
    // Why do we need to cast `num` to the u32 type?
    //
    // The protocol documentation mentions: "negative values always use the maximum number of bytes."
    // This indicates that encoding negative numbers actually encodes the value corresponding
    // to its two's complement representation.
    let mut num = num as u32;
    let mut result = Vec::<u8>::new();

    loop {
        if (num & (!SEGMENT_BITS)) == 0 {
            result.push(num as u8);

            return result;
        }

        result.push(((num & SEGMENT_BITS) | (!SEGMENT_BITS)) as u8);
        num >>= 7;
    }
}

/// Decode the given VarInt as a number
pub fn decode_varint(arr: &Vec<u8>) -> Result<i32, MspErr> {
    // VarInts are never longer than 5 bytes
    //
    // Because VarInt encoding objects are of type i32,
    // they will produce a maximum of 5 bytes of data.
    if arr.len() > 5 {
        return Err(MspErr::DataErr(format!(
            "VarInts are never longer than 5 bytes, but got {}",
            arr.len()
        )));
    }

    match arr.last() {
        Some(&n) => {
            if n & CHECKER_BIT != 0 {
                return Err(MspErr::DataErr(format!(
                    "Invalid VarInt data: [{}]",
                    arr.iter()
                        .map(|&x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )));
            }

            let mut result = 0i32;

            for (i, &n) in arr.iter().enumerate() {
                result |= ((n as i32) & (SEGMENT_BITS as i32)) << (i * 7);
            }

            return Ok(result);
        }
        _ => Err(MspErr::DataErr(format!("VarInts is empty"))),
    }
}

pub fn decode_varint_from_socket(socket: &mut TcpStream) -> Result<(usize, i32), MspErr> {
    let mut buffer = Vec::<u8>::new();
    let mut temp_buffer = vec![0; 1];

    loop {
        socket.read(&mut temp_buffer)?;

        if let Some(&buf) = temp_buffer.get(0) {
            buffer.push(buf);

            if buf & CHECKER_BIT == 0 {
                break;
            }
        }
    }

    Ok((buffer.len(), decode_varint(&buffer)?))
}
