#[derive(Debug, Clone, PartialEq)]
pub enum NetworkType {
    WiFi,
    Ethernet,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub net_type: NetworkType,
    pub in_use: bool,
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionDetails {
    pub uuid: String,
    pub name: String,
    pub con_type: String,
    pub autoconnect: bool,
    pub interface_name: String,
    // IPv4
    pub ipv4_method: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv4_gateway: String,
    pub ipv4_dns: Vec<String>,
    // IPv6
    pub ipv6_method: String,
    pub ipv6_addresses: Vec<String>,
    pub ipv6_gateway: String,
    pub ipv6_dns: Vec<String>,
    // Device-specific
    pub mtu: String,
    pub cloned_mac: String,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub dev_type: String,
    pub state: String,
    pub connection: String,
}

/// Saved connection profile (for nmtui "Edit a connection" list)
#[derive(Debug, Clone)]
pub struct SavedConnection {
    pub name: String,
    pub uuid: String,
    pub con_type: String,
    pub active: bool,
}
