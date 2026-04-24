use anyhow::Result;
use std::collections::HashMap;
use tokio::process::Command as AsyncCommand;

use crate::models::*;
use crate::utils::*;

// ─── Scan & List ─────────────────────────────────────────────────

pub async fn scan_networks() -> Result<()> {
    let _ = AsyncCommand::new("nmcli")
        .args(&["dev", "wifi", "rescan"])
        .output()
        .await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Ok(())
}

pub async fn get_networks() -> Result<Vec<Network>> {
    let mut networks = Vec::new();

    // Wi-Fi networks
    if let Ok(output) = AsyncCommand::new("nmcli")
        .args(&["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "dev", "wifi"])
        .output()
        .await
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let parts = split_nmcli_line(line);
                if parts.len() >= 4 {
                    networks.push(Network {
                        net_type: NetworkType::WiFi,
                        in_use: parts[0] == "*",
                        ssid: parts[1].to_string(),
                        signal: parts[2].parse().unwrap_or(0),
                        security: parts[3].to_string(),
                        uuid: None,
                    });
                }
            }
        }
    }

    // Saved connections
    if let Ok(output) = AsyncCommand::new("nmcli")
        .args(&["-t", "-f", "NAME,UUID,TYPE,ACTIVE", "connection", "show"])
        .output()
        .await
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let parts = split_nmcli_line(line);
                if parts.len() >= 4 {
                    let name = &parts[0];
                    let uuid = &parts[1];
                    let ctype = &parts[2];
                    let active = parts[3] == "yes";

                    if ctype == "802-3-ethernet" {
                        networks.push(Network {
                            net_type: NetworkType::Ethernet,
                            in_use: active,
                            ssid: name.to_string(),
                            signal: 100,
                            security: "".to_string(),
                            uuid: Some(uuid.to_string()),
                        });
                    } else if ctype == "802-11-wireless" {
                        let mut best_idx: Option<usize> = None;
                        let mut best_exact = false;
                        for (i, net) in networks.iter().enumerate() {
                            if net.net_type != NetworkType::WiFi {
                                continue;
                            }
                            let exact = net.ssid == *name;
                            let fuzzy = name.starts_with(&net.ssid);
                            if !exact && !fuzzy {
                                continue;
                            }
                            match best_idx {
                                None => {
                                    best_idx = Some(i);
                                    best_exact = exact;
                                }
                                Some(_) if exact && !best_exact => {
                                    best_idx = Some(i);
                                    best_exact = exact;
                                }
                                _ => {}
                            }
                        }
                        if let Some(idx) = best_idx {
                            let net = &mut networks[idx];
                            if net.uuid.is_none() || active {
                                net.uuid = Some(uuid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(networks)
}

/// List all saved connection profiles (for nmtui-style "Edit a connection")
pub async fn get_saved_connections() -> Result<Vec<SavedConnection>> {
    let output = AsyncCommand::new("nmcli")
        .args(&["-t", "-f", "NAME,UUID,TYPE,ACTIVE", "connection", "show"])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut connections = Vec::new();
    for line in stdout.lines() {
        let parts = split_nmcli_line(line);
        if parts.len() >= 4 {
            connections.push(SavedConnection {
                name: parts[0].clone(),
                uuid: parts[1].clone(),
                con_type: parts[2].clone(),
                active: parts[3] == "yes",
            });
        }
    }
    Ok(connections)
}

// ─── Device helpers ──────────────────────────────────────────────

pub async fn get_wifi_device() -> Option<String> {
    let output = AsyncCommand::new("nmcli")
        .args(&["-t", "-f", "DEVICE,TYPE", "dev"])
        .output()
        .await
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().find_map(|line| {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[1] == "wifi" {
            Some(parts[0].to_string())
        } else {
            None
        }
    })
}

pub async fn get_all_devices() -> Result<Vec<DeviceInfo>> {
    let output = AsyncCommand::new("nmcli")
        .args(&["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "dev"])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in stdout.lines() {
        let parts = split_nmcli_line(line);
        if parts.len() >= 4 {
            devices.push(DeviceInfo {
                name: parts[0].clone(),
                dev_type: parts[1].clone(),
                state: parts[2].clone(),
                connection: parts[3].clone(),
            });
        }
    }
    Ok(devices)
}

// ─── Connect / Disconnect ────────────────────────────────────────

pub async fn connect_to_network(
    ssid: &str,
    password: Option<&str>,
    uuid: Option<&str>,
) -> Result<()> {
    let ifname = get_wifi_device().await;

    if let Some(uuid) = uuid {
        if let Some(ref dev) = ifname {
            let _ = AsyncCommand::new("nmcli")
                .args(&["con", "modify", "uuid", uuid, "connection.interface-name", dev])
                .output()
                .await;
        }
    }

    let mut args = vec!["dev", "wifi", "connect"];
    // "--" separator: prevents SSID from being interpreted as an nmcli flag
    if ssid.starts_with('-') {
        args.push("--");
    }
    args.push(ssid);
    if let Some(pass) = password {
        args.push("password");
        args.push(pass);
    }
    if let Some(ref dev) = ifname {
        args.push("ifname");
        args.push(dev);
    }
    let output = AsyncCommand::new("nmcli").args(&args).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("nmcli connect failed: {}", stderr);
        return Err(anyhow::anyhow!("Connection failed: {}", stderr));
    }
    Ok(())
}

pub async fn connect_ethernet(uuid: &str) -> Result<()> {
    let output = AsyncCommand::new("nmcli")
        .args(&["con", "up", "uuid", uuid])
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Connection failed: {}", stderr));
    }
    Ok(())
}

pub async fn disconnect_network(net: &Network) {
    let wifi_dev = get_wifi_device().await;

    if net.net_type == NetworkType::Ethernet {
        if let Some(uuid) = &net.uuid {
            let _ = AsyncCommand::new("nmcli")
                .args(&["con", "down", "uuid", uuid])
                .output()
                .await;
        }
    } else if let Some(dev) = &wifi_dev {
        let _ = AsyncCommand::new("nmcli")
            .args(&["dev", "disconnect", dev])
            .output()
            .await;
    }
}

pub async fn delete_connection(uuid: &str) -> Result<()> {
    let output = AsyncCommand::new("nmcli")
        .args(&["con", "delete", "uuid", uuid])
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Delete failed: {}", stderr));
    }
    Ok(())
}

// ─── Connection Details / Edit ───────────────────────────────────

pub async fn get_connection_details(uuid: &str) -> Result<ConnectionDetails> {
    let output = AsyncCommand::new("nmcli")
        .args(&["-t", "con", "show", "uuid", uuid])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut map: HashMap<String, String> = HashMap::new();
    for line in stdout.lines() {
        let parts = split_nmcli_line(line);
        if parts.len() >= 2 {
            // First part is key, the rest is value (IPv6 addresses might contain ':')
            let key = parts[0].clone();
            let value = parts[1..].join(":");
            map.insert(key, value);
        }
    }

    let mtu_val = clean_dash(map.get("802-3-ethernet.mtu"));
    let mtu = if mtu_val.is_empty() {
        clean_dash(map.get("802-11-wireless.mtu"))
    } else {
        mtu_val
    };

    let mac_val = clean_dash(map.get("802-3-ethernet.cloned-mac-address"));
    let cloned_mac = if mac_val.is_empty() {
        clean_dash(map.get("802-11-wireless.cloned-mac-address"))
    } else {
        mac_val
    };

    Ok(ConnectionDetails {
        uuid: uuid.to_string(),
        name: map.get("connection.id").cloned().unwrap_or_default(),
        con_type: map.get("connection.type").cloned().unwrap_or_default(),
        autoconnect: map
            .get("connection.autoconnect")
            .map(|v| v == "yes")
            .unwrap_or(true),
        interface_name: clean_dash(map.get("connection.interface-name")),
        ipv4_method: map.get("ipv4.method").cloned().unwrap_or_default(),
        ipv4_addresses: parse_nmcli_list(map.get("ipv4.addresses")),
        ipv4_gateway: clean_dash(map.get("ipv4.gateway")),
        ipv4_dns: parse_nmcli_list(map.get("ipv4.dns")),
        ipv6_method: map.get("ipv6.method").cloned().unwrap_or_default(),
        ipv6_addresses: parse_nmcli_list(map.get("ipv6.addresses")),
        ipv6_gateway: clean_dash(map.get("ipv6.gateway")),
        ipv6_dns: parse_nmcli_list(map.get("ipv6.dns")),
        mtu,
        cloned_mac,
    })
}

pub async fn modify_connection(uuid: &str, settings: &[(&str, &str)]) -> Result<()> {
    let mut args: Vec<&str> = vec!["con", "modify", "uuid", uuid];
    for (key, value) in settings {
        args.push(key);
        args.push(value);
    }
    let output = AsyncCommand::new("nmcli").args(&args).output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Modify failed: {}", stderr));
    }
    Ok(())
}

// ─── Connection Creation ─────────────────────────────────────────

pub async fn add_connection(
    con_type: &str,
    name: &str,
    settings: &[(String, String)],
) -> Result<String> {
    let mut args: Vec<String> = vec![
        "con".into(),
        "add".into(),
        "type".into(),
        con_type.into(),
        "con-name".into(),
        name.into(),
    ];
    for (key, value) in settings {
        args.push(key.clone());
        args.push(value.clone());
    }
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = AsyncCommand::new("nmcli").args(&args_ref).output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Add failed: {}", stderr));
    }
    // Extract UUID from nmcli output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let uuid = stdout
        .split('(')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .unwrap_or("")
        .to_string();
    Ok(uuid)
}

// ─── Hostname ────────────────────────────────────────────────────

pub async fn get_hostname() -> Result<String> {
    let output = AsyncCommand::new("nmcli")
        .args(&["general", "hostname"])
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn set_hostname(name: &str) -> Result<()> {
    let output = AsyncCommand::new("nmcli")
        .args(&["general", "hostname", name])
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Set hostname failed: {}", stderr));
    }
    Ok(())
}
