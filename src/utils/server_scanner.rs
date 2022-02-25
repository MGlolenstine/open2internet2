use crate::packets::ServerQueryResponse;
use anyhow::{bail, Result};
use tracing::debug;

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use netstat::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const LOCAL_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const LOCAL_IP_V6: Ipv6Addr = Ipv6Addr::new(0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16, 0u16);

/// Scan ports in Minecraft range, to find the Minecraft one.
pub async fn scan_ports() -> Vec<(String, u16)> {
    let all_ports = get_used_ports();
    let mut ports = vec![];
    for (addr, port) in all_ports {
        if let Ok(a) = is_minecraft(addr, port).await {
            ports.push((a.motd, port));
        }
    }
    ports
}

pub fn get_used_ports() -> Vec<(IpAddr, u16)> {
    let af_flags = AddressFamilyFlags::all();
    let proto_flags = ProtocolFlags::all();
    let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap();
    let mut ports = vec![];
    for si in sockets_info {
        if let ProtocolSocketInfo::Tcp(tcp_si) = si.protocol_socket_info {
            if let netstat::TcpState::Listen = tcp_si.state {
                ports.push((tcp_si.local_addr, tcp_si.local_port));
            }
        }
    }
    ports
}

/// Check if the port at the local address is responding with correct data for Minecraft server.
#[tracing::instrument]
async fn is_minecraft(addr: IpAddr, port: u16) -> Result<ServerQueryResponse> {
    let stream = TcpStream::connect(&SocketAddr::new(addr, port)).await;
    let mut stream = if let Ok(stream) = stream {
        stream
    } else {
        debug!("Unable to connect!");
        bail!("Unable to connect!");
    };
    let req = [0xFE, 0x01];
    stream.write(&req).await.unwrap();
    let mut resp = vec![0u8; 1024];
    let buf_len = tokio::time::timeout(Duration::from_millis(100), stream.read(&mut resp)).await;
    if buf_len.is_err() {
        debug!("Timed out!");
        bail!("Timed out!");
    } else if let Ok(a) = buf_len {
        if a.is_err() {
            debug!("Failed writing to the TCP stream!");
            bail!("Failed writing to the TCP stream!");
        }
    }
    is_minecraft_response(&resp)
}

/// Check if the returned buffer equals to Minecraft's response.
#[tracing::instrument]
fn is_minecraft_response(buffer: &[u8]) -> Result<ServerQueryResponse> {
    if buffer.is_empty() {
        bail!("Buffer is empty!");
    }
    ServerQueryResponse::read(buffer)
}
