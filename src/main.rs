use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SntpConf, SyncStatus},
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
    sys,
};
use log::{info, warn, error};
use std::ptr;
use std::time::Duration;

mod config;
use config::*;

mod wireguard {
    use super::*;
    use std::ffi::CString;

    /// Safe wrapper around the WireGuard C context.
    /// Stores CStrings to ensure they outlive the C config pointers.
    pub struct WireGuardTunnel {
        ctx: sys::wireguard::wireguard_ctx_t,
        _private_key: CString,
        _public_key: CString,
        _allowed_ip: CString,
        _allowed_ip_mask: CString,
        _endpoint: CString,
    }

    impl WireGuardTunnel {
        pub fn new(
            private_key: &str,
            peer_public_key: &str,
            local_ip: &str,
            local_netmask: &str,
            endpoint: &str,
            port: u16,
            keepalive: u16,
        ) -> Result<Self, &'static str> {
            let private_key_c = CString::new(private_key).map_err(|_| "Invalid private key")?;
            let public_key_c = CString::new(peer_public_key).map_err(|_| "Invalid public key")?;
            let allowed_ip_c = CString::new(local_ip).map_err(|_| "Invalid local IP")?;
            let allowed_ip_mask_c = CString::new(local_netmask).map_err(|_| "Invalid netmask")?;
            let endpoint_c = CString::new(endpoint).map_err(|_| "Invalid endpoint")?;

            let mut config = sys::wireguard::wireguard_config_t {
                private_key: private_key_c.as_ptr() as *mut _,
                listen_port: 0,
                fw_mark: 0,
                public_key: public_key_c.as_ptr() as *mut _,
                preshared_key: ptr::null_mut(),
                allowed_ip: allowed_ip_c.as_ptr() as *mut _,
                allowed_ip_mask: allowed_ip_mask_c.as_ptr() as *mut _,
                endpoint: endpoint_c.as_ptr() as *mut _,
                port: port as i32,
                persistent_keepalive: keepalive as i32,
            };

            let mut ctx = sys::wireguard::wireguard_ctx_t::default();

            let ret = unsafe {
                sys::wireguard::esp_wireguard_init(&mut config, &mut ctx)
            };

            if ret != sys::ESP_OK as i32 {
                return Err("esp_wireguard_init failed");
            }

            Ok(WireGuardTunnel {
                ctx,
                _private_key: private_key_c,
                _public_key: public_key_c,
                _allowed_ip: allowed_ip_c,
                _allowed_ip_mask: allowed_ip_mask_c,
                _endpoint: endpoint_c,
            })
        }

        pub fn connect(&mut self) -> Result<(), &'static str> {
            let ret = unsafe {
                sys::wireguard::esp_wireguard_connect(&mut self.ctx)
            };

            if ret != sys::ESP_OK as i32 {
                return Err("esp_wireguard_connect failed");
            }

            Ok(())
        }

        pub fn is_peer_up(&mut self) -> bool {
            let ret = unsafe {
                sys::wireguard::esp_wireguardif_peer_is_up(&mut self.ctx)
            };
            ret == sys::ESP_OK as i32
        }

        pub fn set_default_gateway(&mut self) -> Result<(), &'static str> {
            let ret = unsafe {
                sys::wireguard::esp_wireguard_set_default(&mut self.ctx)
            };

            if ret != sys::ESP_OK as i32 {
                return Err("esp_wireguard_set_default failed");
            }

            Ok(())
        }

        pub fn disconnect(&mut self) -> Result<(), &'static str> {
            let ret = unsafe {
                sys::wireguard::esp_wireguard_disconnect(&mut self.ctx)
            };

            if ret != sys::ESP_OK as i32 {
                return Err("esp_wireguard_disconnect failed");
            }

            Ok(())
        }
    }

    impl Drop for WireGuardTunnel {
        fn drop(&mut self) {
            let _ = self.disconnect();
        }
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);

    info!("ESP32 WireGuard VPN Client");

    // Initialize peripherals, event loop, and NVS
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // WiFi setup
    info!("Initializing WiFi...");
    let wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(wifi, sysloop)?;

    let wifi_config = Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().expect("SSID too long"),
        password: WIFI_PASS.try_into().expect("Password too long"),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_config)?;
    wifi.start()?;
    info!("Connecting to WiFi: {}", WIFI_SSID);
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi connected! IP: {}, Gateway: {}", ip_info.ip, ip_info.subnet.gateway);

    // NTP time synchronization
    info!("Synchronizing time via NTP...");

    let sntp_conf = SntpConf {
        servers: [NTP_SERVER, "time.google.com", "time.cloudflare.com"],
        ..Default::default()
    };
    let sntp = EspSntp::new(&sntp_conf)?;

    let mut sync_attempts = 0;
    while sntp.get_sync_status() != SyncStatus::Completed && sync_attempts < 30 {
        sync_attempts += 1;
        std::thread::sleep(Duration::from_secs(1));
    }

    if sntp.get_sync_status() == SyncStatus::Completed {
        use std::time::SystemTime;
        if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            info!("Time synchronized: {} (Unix epoch)", duration.as_secs());
        }
    } else {
        warn!("NTP sync timeout - WireGuard may reject handshakes!");
    }

    let _sntp = sntp;

    // WireGuard tunnel setup
    info!("Initializing WireGuard tunnel...");
    info!("Peer: {}:{}", WG_PEER_ENDPOINT, WG_PEER_PORT);
    info!("Local IP: {}/{}", WG_LOCAL_IP, WG_LOCAL_NETMASK);

    if WG_PRIVATE_KEY == "YOUR_PRIVATE_KEY_BASE64" {
        error!("WireGuard not configured! Edit src/config.rs");
        loop {
            std::thread::sleep(Duration::from_secs(10));
        }
    }

    let mut tunnel = match wireguard::WireGuardTunnel::new(
        WG_PRIVATE_KEY,
        WG_PEER_PUBLIC_KEY,
        WG_LOCAL_IP,
        WG_LOCAL_NETMASK,
        WG_PEER_ENDPOINT,
        WG_PEER_PORT,
        WG_KEEPALIVE,
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to initialize WireGuard: {}", e);
            return Err(anyhow::anyhow!("WireGuard init failed"));
        }
    };

    info!("WireGuard initialized, starting connection...");

    if let Err(e) = tunnel.connect() {
        error!("Failed to start WireGuard: {}", e);
        return Err(anyhow::anyhow!("WireGuard connect failed"));
    }

    info!("Waiting for WireGuard handshake...");
    let mut handshake_attempts = 0;
    while !tunnel.is_peer_up() && handshake_attempts < 30 {
        handshake_attempts += 1;
        std::thread::sleep(Duration::from_secs(1));
    }

    if tunnel.is_peer_up() {
        info!("WireGuard Tunnel Established! VPN IP: {}", WG_LOCAL_IP);
        // Uncomment to route all traffic through VPN:
        // tunnel.set_default_gateway()?;
    } else {
        warn!("WireGuard handshake timeout - peer may be unreachable");
    }

    // Main loop - monitor connection status
    loop {
        let wifi_ok = wifi.is_connected().unwrap_or(false);
        let wg_ok = tunnel.is_peer_up();

        if wifi_ok && wg_ok {
            info!("Status: WiFi=OK, WireGuard=UP, VPN IP={}", WG_LOCAL_IP);
        } else if wifi_ok {
            warn!("Status: WiFi=OK, WireGuard=DOWN - reconnecting...");
            let _ = tunnel.connect();
        } else {
            warn!("Status: WiFi=DOWN - reconnecting...");
            let _ = wifi.connect();
        }

        std::thread::sleep(Duration::from_secs(30));
    }
}
