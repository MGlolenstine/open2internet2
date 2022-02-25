use std::net::IpAddr;

/// Get host's local IP.
pub fn get_local_ip() -> Option<IpAddr> {
    for iface in get_if_addrs::get_if_addrs().unwrap() {
        let ip = iface.ip().to_string();
        //? Filter out local IPs for IPv4 and IPv6, and also Docker IPs.
        if !iface.is_loopback() && !ip.starts_with("172.") {
            return Some(iface.ip());
        }
    }
    None
}
