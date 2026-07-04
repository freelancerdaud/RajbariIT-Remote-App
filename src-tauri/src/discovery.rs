// ============================================
// Feature 1: LAN Discovery — discovery.rs
// ============================================
// Uses mDNS (Multicast DNS) to discover and broadcast devices on the local network.

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Duration;

/// Service type for RajbariIT Remote discovery
const SERVICE_TYPE: &str = "_rjremote._tcp.local.";

/// Default port for screen streaming
const DEFAULT_PORT: u16 = 9095;

/// Represents a discovered device on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub os: String,
}

/// Global daemon handle for mDNS broadcasting
static DAEMON: Mutex<Option<ServiceDaemon>> = Mutex::new(None);

/// Get the current device's hostname
#[tauri::command]
pub fn get_device_name() -> Result<String, String> {
    hostname::get()
        .map(|name| name.to_string_lossy().to_string())
        .map_err(|e| format!("Could not get hostname: {}", e))
}

/// Start broadcasting this device on the LAN via mDNS
#[tauri::command]
pub fn start_broadcasting(device_name: String) -> Result<(), String> {
    let mdns = ServiceDaemon::new().map_err(|e| format!("mDNS error: {}", e))?;

    // Get the OS name
    let os_name = std::env::consts::OS.to_string();

    // Create a unique instance name
    let instance_name = format!("rjremote-{}", &device_name.replace(' ', "-").to_lowercase());

    // Build the service info
    let mut properties = HashMap::new();
    properties.insert("name".to_string(), device_name);
    properties.insert("os".to_string(), os_name);
    properties.insert("ver".to_string(), "1.0.0".to_string());

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance_name,
        &instance_name,
        "",
        DEFAULT_PORT,
        properties,
    )
    .map_err(|e| format!("Service info error: {}", e))?;

    // Register the service
    mdns.register(service_info)
        .map_err(|e| format!("Register error: {}", e))?;

    // Store daemon handle
    let mut daemon = DAEMON.lock().map_err(|e| e.to_string())?;
    *daemon = Some(mdns);

    Ok(())
}

/// Stop broadcasting this device
#[tauri::command]
pub fn stop_broadcasting() -> Result<(), String> {
    let mut daemon = DAEMON.lock().map_err(|e| e.to_string())?;
    if let Some(mdns) = daemon.take() {
        mdns.shutdown().map_err(|e| format!("Shutdown error: {}", e))?;
    }
    Ok(())
}

/// Scan for devices on the local network using mDNS
#[tauri::command]
pub async fn discover_devices() -> Result<Vec<DeviceInfo>, String> {
    let mdns = ServiceDaemon::new().map_err(|e| format!("mDNS error: {}", e))?;
    let browser = mdns
        .browse(SERVICE_TYPE)
        .map_err(|e| format!("Browse error: {}", e))?;

    let mut devices: Vec<DeviceInfo> = Vec::new();

    // Listen for events for a short duration
    let timeout = Duration::from_secs(3);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match browser.recv_timeout(Duration::from_millis(500)) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let name = info
                    .get_property_val_str("name")
                    .unwrap_or("Unknown Device")
                    .to_string();
                let os = info
                    .get_property_val_str("os")
                    .unwrap_or("unknown")
                    .to_string();
                let ip = info
                    .get_addresses()
                    .iter()
                    .next()
                    .map(|a| a.to_string())
                    .unwrap_or_default();
                let port = info.get_port();

                if !ip.is_empty() {
                    devices.push(DeviceInfo { name, ip, port, os });
                }
            }
            Ok(_) => {} // Ignore other events
            Err(_) => break, // Timeout or error, stop listening
        }
    }

    // Shutdown this temporary browser daemon
    let _ = mdns.shutdown();

    Ok(devices)
}
