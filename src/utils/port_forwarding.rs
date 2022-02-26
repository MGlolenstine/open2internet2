use anyhow::{bail, Result};
use tracing::debug;

use portforwarder_rs::port_forwarder::Forwarder;

pub fn redirect_minecraft_to_a_port(
    forwarder: &mut Forwarder,
    mc_port: u16,
    wanted_port: u16,
) -> Result<()> {
    if let Err(e) = forwarder.forward_port(
        mc_port,
        wanted_port,
        portforwarder_rs::port_forwarder::PortMappingProtocol::TCP,
        "MinecraftLAN",
    ) {
        bail!("Failed to open the port!\n{}", e);
    }
    debug!("It worked! Got port {}, hopefully!", wanted_port);
    Ok(())
}
