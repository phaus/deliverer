use dhcplease::{
    options::{DhcpOption, MessageType},
    packet::DhcpPacket,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeviceConfig {
    interface: String,
    server_ip: Ipv4Addr,
    subnet_mask: Ipv4Addr,
    next_server: Ipv4Addr,
    offered_ip_start: Ipv4Addr,
    offered_ip_end: Ipv4Addr,
    tftp_server: String,
    boot_file: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
struct AppConfig {
    port: u16,
    devices: Vec<DeviceConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 1067,
            devices: vec![DeviceConfig {
                interface: "eth0".to_string(),
                server_ip: Ipv4Addr::new(192, 168, 1, 10),
                subnet_mask: Ipv4Addr::new(255, 255, 255, 0),
                next_server: Ipv4Addr::new(192, 168, 1, 10),
                offered_ip_start: Ipv4Addr::new(192, 168, 1, 100),
                offered_ip_end: Ipv4Addr::new(192, 168, 1, 150),
                tftp_server: "192.168.1.10".to_string(),
                boot_file: "bootx64.efi".to_string(),
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
struct AppConfig {
    port: u16,
    devices: Vec<DeviceConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 1067,
            devices: vec![DeviceConfig {
                interface: "eth0".to_string(),
                server_ip: Ipv4Addr::new(192, 168, 1, 10),
                subnet_mask: Ipv4Addr::new(255, 255, 255, 0),
                next_server: Ipv4Addr::new(192, 168, 1, 10),
                offered_ip_start: Ipv4Addr::new(192, 168, 1, 100),
                offered_ip_end: Ipv4Addr::new(192, 168, 1, 150),
                tftp_server: "192.168.1.10".to_string(),
                boot_file: "bootx64.efi".to_string(),
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
struct AppConfig {
    port: u16,
    devices: Vec<DeviceConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 1067,
            devices: vec![DeviceConfig {
                interface: "eth0".to_string(),
                server_ip: Ipv4Addr::new(192, 168, 1, 10),
                next_server: Ipv4Addr::new(192, 168, 1, 10),
                offered_ip_start: Ipv4Addr::new(192, 168, 1, 100),
                offered_ip_end: Ipv4Addr::new(192, 168, 1, 150),
                tftp_server: "192.168.1.10".to_string(),
                boot_file: "bootx64.efi".to_string(),
            }],
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allow overriding port via CLI argument for testing or showing help
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!("Deliverer DHCP Server");
                println!("\nUsage:");
                println!("  deliverer [PORT]");
                println!("\nArguments:");
                println!("  PORT    UDP port to listen on (default: 1067)");
                println!("\nOptions:");
                println!("  -h, --help    Print help information");
                return Ok(());
            }
            arg => {
                 if let Ok(_p) = arg.parse::<u16>() {
                    // Valid port, will be applied after config load
                } else {
                    eprintln!("Error: Invalid argument '{}'", arg);
                    eprintln!("Use --help for usage information.");
                    return Err("Invalid argument".into());
                }
            }
        }
    }

    let config_path = "config.json";
    
    // Load or create AppConfig
    let mut app_config: AppConfig = if std::path::Path::new(config_path).exists() {
        let content = fs::read_to_string(config_path)?;
        serde_json::from_str(&content)?
    } else {
        let default_config = AppConfig::default();
        let content = serde_json::to_string_pretty(&default_config)?;
        fs::write(config_path, content)?;
        println!("Created default config.json");
        default_config
    };

    // Apply port override if provided
    if args.len() > 1 {
        if let Ok(p) = args[1].parse::<u16>() {
            app_config.port = p;
        }
    }

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", app_config.port)).await?;

    println!("Listening for DHCP on port {}...", app_config.port);
    println!("Configured devices: {}", app_config.devices.len());

    let mut buf = [0u8; 1500];

    loop {
        let (len, remote_addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];

        match DhcpPacket::parse(data) {
            Ok(request) => {
                let msg_type_opt = request.message_type();
                println!("Received packet from {} (Type: {:?})", remote_addr, msg_type_opt);

                // Find the matching device configuration
                let matching_device = app_config.devices.iter().find(|dev| {
                    // Simple matching: check if the remote address is within the device's subnet
                    let remote_ip = remote_addr.ip().try_into().ok();
                    if let Some(ip) = remote_ip {
                        let ip_u32: u32 = ip.into();
                        let subnet_u32: u32 = dev.server_ip.into(); // This should be the subnet IP or we use server_ip
                        // Wait, the subnet mask check should be: (remote_ip & subnet_mask) == (server_ip & subnet_mask)
                        // But we need to be careful with how we represent subnet mask.
                        // Let's assume server_ip is the gateway/server IP on that subnet.
                        // A more robust way is to use the subnet mask provided in DeviceConfig.
                        
                        // For now, let's just match based on the server_ip being in the same subnet as the remote_addr
                        // But that's not quite right. We should check if the remote_addr's subnet matches.
                        // Actually, a simpler way for this minimal implementation is to match based on the server_ip.
                        // If the packet is a broadcast, it's harder.
                        
                        // Let's use the subnet mask:
                        let remote_ip_u32: u32 = ip.into();
                        let server_ip_u32: u32 = dev.server_ip.into();
                        let mask_u32: u32 = dev.subnet_mask.into();
                        
                        (remote_ip_u32 & mask_u32) == (server_ip_u32 & mask_u32)
                    } else {
                        false
                    }
                });

                if let Some(dev) = matching_device {
                    match msg_type_opt {
                        Some(MessageType::Discover) => {
                            println!("Handling DISCOVER for device {} -> Sending OFFER", dev.interface);
                            let options = vec![
                                DhcpOption::MessageType(MessageType::Offer),
                                DhcpOption::SubnetMask(dev.subnet_mask),
                                DhcpOption::Router(vec![dev.next_server]),
                                DhcpOption::ServerIdentifier(dev.server_ip),
                            ];

                            let reply = DhcpPacket::create_reply(
                                &request,
                                MessageType::Offer,
                                dev.server_ip,
                                dev.next_server,
                                options,
                            );

                            let mut reply_bytes = reply.encode();
                            
                            // Injecting offered IP range (simplified: just use the start)
                            inject_raw_option(&mut reply_bytes, 50, &dev.offered_ip_start.octets());
                            inject_pxe_options(&mut reply_bytes, &dev.tftp_server, &dev.boot_file);

                            socket.send_to(&reply_bytes, remote_addr).await?;
                            println!("Sent OFFER to {} via {}", remote_addr, dev.interface);
                        }
                        Some(MessageType::Request) => {
                            println!("Handling REQUEST for device {} -> Sending ACK", dev.interface);
                            let options = vec![
                                DhcpOption::MessageType(MessageType::Ack),
                                DhcpOption::SubnetMask(dev.subnet_mask),
                                DhcpOption::Router(vec![dev.next_server]),
                                DhcpOption::ServerIdentifier(dev.server_ip),
                            ];

                            let reply = DhcpPacket::create_reply(
                                &request,
                                MessageType::Ack,
                                dev.server_ip,
                                dev.next_server,
                                options,
                            );

                            let mut reply_bytes = reply.encode();
                            inject_pxe_options(&mut reply_bytes, &dev.tftp_server, &dev.boot_file);
                            socket.send_to(&reply_bytes, remote_addr).await?;
                            println!("Sent ACK to {} via {}", remote_addr, dev.interface);
                        }
                        _ => println!("Unhandled message type: {:?}", msg_type_opt),
                    }
                } else {
                    println!("No matching device configuration found for {}", remote_addr);
                }
            }
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
        }
    }
}

fn inject_raw_option(bytes: &mut Vec<u8>, code: u8, data: &[u8]) {
    if let Some(pos) = bytes.iter().position(|&b| b == 255) {
        let mut opt = vec![code, data.len() as u8];
        opt.extend_from_slice(data);
        
        let mut new_bytes = bytes[..pos].to_vec();
        new_bytes.extend_from_slice(&opt);
        new_bytes.push(255);
        new_bytes.extend_from_slice(&bytes[pos + 1..]);
        *bytes = new_bytes;
    }
}

fn inject_pxe_options(bytes: &mut Vec<u8>, tftp_server: &str, boot_file: &str) {
    if let Some(pos) = bytes.iter().position(|&b| b == 255) {
        let mut opt66 = vec![66, tftp_server.len() as u8];
        opt66.extend_from_slice(tftp_server.as_bytes());

        let mut opt67 = vec![67, boot_file.len() as u8];
        opt67.extend_from_slice(boot_file.as_bytes());

        let mut new_bytes = bytes[..pos].to_vec();
        new_bytes.extend_from_slice(&opt66);
        new_bytes.extend_from_slice(&opt67);
        new_bytes.push(255);
        new_bytes.extend_from_slice(&bytes[pos + 1..]);
        *bytes = new_bytes;
    }
}
