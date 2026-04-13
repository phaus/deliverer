use dhcplease::{
    options::{DhcpOption, MessageType},
    packet::DhcpPacket,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
struct AppConfig {
    port: u16,
    server_ip: Ipv4Addr,
    next_server: Ipv4Addr,
    offered_ip: Ipv4Addr,
    tftp_server: String,
    boot_file: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 1067,
            server_ip: Ipv4Addr::new(192, 168, 1, 10),
            next_server: Ipv4Addr::new(192, 168, 1, 10),
            offered_ip: Ipv4Addr::new(192, 168, 1, 100),
            tftp_server: "192.168.1.10".to_string(),
            boot_file: "bootx64.efi".to_string(),
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
    println!("Server IP: {}", app_config.server_ip);
    println!("TFTP Server: {}", app_config.tftp_server);
    println!("Boot File: {}", app_config.boot_file);

    let mut buf = [0u8; 1500];

    loop {
        let (len, remote_addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];

        match DhcpPacket::parse(data) {
            Ok(request) => {
                let msg_type_opt = request.message_type();
                println!("Received packet from {} (Type: {:?})", remote_addr, msg_type_opt);

                match msg_type_opt {
                    Some(MessageType::Discover) => {
                        println!("Handling DISCOVER -> Sending OFFER");
                        let options = vec![
                            DhcpOption::MessageType(MessageType::Offer),
                            DhcpOption::SubnetMask(Ipv4Addr::new(255, 255, 255, 0)),
                            DhcpOption::Router(vec![Ipv4Addr::new(192, 168, 1, 1)]),
                            DhcpOption::ServerIdentifier(app_config.server_ip),
                        ];

                        let reply = DhcpPacket::create_reply(
                            &request,
                            MessageType::Offer,
                            app_config.server_ip,
                            app_config.next_server,
                            options,
                        );

                        let mut reply_bytes = reply.encode();
                        
                        inject_raw_option(&mut reply_bytes, 50, &app_config.offered_ip.octets());
                        inject_pxe_options(&mut reply_bytes, &app_config.tftp_server, &app_config.boot_file);

                        socket.send_to(&reply_bytes, remote_addr).await?;
                        println!("Sent OFFER to {}", remote_addr);
                    }
                    Some(MessageType::Request) => {
                        println!("Handling REQUEST -> Sending ACK");
                        let options = vec![
                            DhcpOption::MessageType(MessageType::Ack),
                            DhcpOption::SubnetMask(Ipv4Addr::new(255, 255, 255, 0)),
                            DhcpOption::Router(vec![Ipv4Addr::new(192, 168, 1, 1)]),
                            DhcpOption::ServerIdentifier(app_config.server_ip),
                        ];

                        let reply = DhcpPacket::create_reply(
                            &request,
                            MessageType::Ack,
                            app_config.server_ip,
                            app_config.next_server,
                            options,
                        );

                        let mut reply_bytes = reply.encode();
                        inject_pxe_options(&mut reply_bytes, &app_config.tftp_server, &app_config.boot_file);
                        socket.send_to(&reply_bytes, remote_addr).await?;
                        println!("Sent ACK to {}", remote_addr);
                    }
                    _ => println!("Unhandled message type: {:?}", msg_type_opt),
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
