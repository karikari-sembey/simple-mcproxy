use anyhow::{Context as _, Result, anyhow};

use crate::MAX_PACKET_SIZE;

pub enum State {
    Handshaking,
    Status,
    Login,
}

pub enum Packet {
    Handshake(HandshakePacket),
}

impl Packet {
    pub fn parse(state: State, bytes: &[u8]) -> Result<Self> {
        if bytes.len() == MAX_PACKET_SIZE {
            return Err(anyhow!("too big packets!"));
        }
        let mut bytes = bytes.iter();

        let _len = read_varint(&mut bytes)?; // read packet len (unused)
        let _packet_id = read_varint(&mut bytes)?; // read packet id (unused)

        match state {
            State::Handshaking => Self::parse_handshake(&mut bytes),
            _ => Err(anyhow!("unimplemented state")),
        }
    }

    fn parse_handshake<'a, T: Iterator<Item = &'a u8>>(mut bytes: &mut T) -> Result<Self> {
        let _protocol_version = read_varint(&mut bytes)?;
        let server = read_string(&mut bytes)?;
        let port = read_ushort(&mut bytes)?;
        let intent = match read_varint(&mut bytes)? {
            1 => State::Status,
            2 => State::Login,
            _ => return Err(anyhow!("wrong intent")),
        };
        Ok(Packet::Handshake(HandshakePacket {
            server,
            port,
            intent,
        }))
    }
}

pub struct HandshakePacket {
    pub server: String,
    #[allow(dead_code)]
    pub port: u16,
    #[allow(dead_code)]
    pub intent: State,
}

fn read_varint<'a, T: Iterator<Item = &'a u8>>(reader: &mut T) -> Result<i32> {
    const SEGMENT_BITS: u8 = 0x7F;
    const CONTINUE_BIT: u8 = 0x80;
    let mut ret: i32 = 0;
    let mut pos: i32 = 0;
    loop {
        let current = *reader.next().context("no bytes")?;
        ret |= ((current & SEGMENT_BITS) as i32) << pos;
        if (current & CONTINUE_BIT) == 0 {
            break;
        }
        pos += 7;

        if pos >= 32 {
            return Err(anyhow!("VarInt too long"));
        }
    }
    Ok(ret)
}

fn read_ushort<'a, T: Iterator<Item = &'a u8>>(reader: &mut T) -> Result<u16> {
    Ok((*reader.next().context("no bytes")? as u16) << 8
        | (*reader.next().context("no bytes")? as u16))
}

fn read_string<'a, T: Iterator<Item = &'a u8>>(reader: &mut T) -> Result<String> {
    let n = read_varint(reader)?;
    let mut string = Vec::<u8>::with_capacity(n as usize);
    for _ in 0..n {
        string.push(*reader.next().context("no bytes")?);
    }
    Ok(String::from_utf8_lossy(&string).to_string())
}
