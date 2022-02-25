use anyhow::{bail, Result};
use std::{io::BufReader, u8};

mod packets {
    #![allow(clippy::all)]
    #![allow(warnings)]
    protospec::include_spec!("packets");
}

#[derive(Debug)]
pub struct ServerQueryResponse {
    pub protocol_version: u32,
    pub minecraft_version: String,
    pub motd: String,
    pub online_players: u32,
    pub max_players: u32,
}

impl ServerQueryResponse {
    pub fn read(buf: &[u8]) -> Result<Self> {
        let mut br = BufReader::new(buf);
        let packet = packets::ServerQueryResponse::decode_sync(&mut br)?;
        if let Some(data) = packet.body {
            let data = String::from_utf16(&data.data)?;
            let mut data = data.split('\0').map(|a| a.to_string());
            data.next();
            Ok(Self {
                protocol_version: data.next().unwrap_or_default().parse::<u32>()?,
                minecraft_version: data.next().unwrap_or_default(),
                motd: data.next().unwrap_or_default(),
                online_players: data.next().unwrap_or_default().parse::<u32>()?,
                max_players: data.next().unwrap_or_default().parse::<u32>()?,
            })
        } else {
            bail!("Not a minecraft packet.");
        }
    }
}

#[test]
fn test_server_query_response() {
    let data = [
        0xff, 0x00, 0x23, 0x00, 0xa7, 0x00, 0x31, 0x00, 0x00, 0x00, 0x34, 0x00, 0x37, 0x00, 0x00,
        0x00, 0x31, 0x00, 0x2e, 0x00, 0x34, 0x00, 0x2e, 0x00, 0x32, 0x00, 0x00, 0x00, 0x41, 0x00,
        0x20, 0x00, 0x4d, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x65, 0x00, 0x63, 0x00, 0x72, 0x00, 0x61,
        0x00, 0x66, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x65, 0x00, 0x72, 0x00, 0x76, 0x00,
        0x65, 0x00, 0x72, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x32, 0x00, 0x30,
    ];
    let packet = ServerQueryResponse::read(&data[..]);
    assert!(packet.is_ok());
}
