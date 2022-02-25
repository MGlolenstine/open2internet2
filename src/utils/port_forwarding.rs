use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::{bail, Result};
use tracing::debug;

/// Uses UPnP to port-forward the automagically generated port to the defined external one.
pub fn redirect_minecraft_to_a_port(
    local_addr: Ipv4Addr,
    mc_port: u16,
    wanted_port: u16,
    lease: u32,
) -> Result<()> {
    // UPnP only works on local IPv4 addresses
    let local_addr = SocketAddrV4::new(local_addr, mc_port);
    match igd::search_gateway(Default::default()) {
        Err(ref err) => bail!("Error finding gateway: {}", err),
        Ok(gateway) => {
            match gateway.add_port(
                igd::PortMappingProtocol::TCP,
                wanted_port,
                local_addr,
                lease,
                "MinecraftLAN",
            ) {
                Err(ref err) => {
                    bail!("There was an error registering the port! {}", err);
                }
                Ok(_) => {
                    debug!("It worked! Got port {}, hopefully!", wanted_port);
                }
            }
        }
    }
    Ok(())
}
