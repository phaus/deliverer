use dhcplease::{
    options::{DhcpOption, MessageType},
    packet::DhcpPacket,
};
use std::env;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let port: u16 = if args.len() > 1 {
        args[1].parse().expect("Please provide a valid port number")
    } else {
        67
    };

    let server_ip = Ipv4Addr::new(192, 168, 1, 10);
    let next_server = Ipv4Addr::new(192, 168, 1, 10);
    let offered_ip = Ipv4Addr::new(192, 168, 1, 100);

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Listening for DHCP on port {}...", port);

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
                        let mut options = vec![
                            DhcpOption::MessageType(MessageType::Offer),
                            DhcpOption::SubnetMask(Ipv4Addr::new(255, 255, 255, 0)),
                            DhcpOption::Router(vec![Ipv4Addr::new(192, 168, 1, 1)]),
                            DhcpOption::ServerIdentifier(server_ip),
                        ];

                        let reply = DhcpPacket::create_reply(
                            &request,
                            MessageType::Offer,
                            server_ip,
                            next_server,
                            options,
                        );

                        let mut reply_bytes = reply.encode();
                        
                        inject_raw_option(&mut reply_bytes, 50, &offered_ip.octets());
                        inject_pxe_options(&mut reply_bytes, "192.168.1.10", "bootx64.efi");

                        socket.send_to(&reply_bytes, remote_addr).await?;
                        println!("Sent OFFER to {}", remote_addr);
                    }
                    Some(MessageType::Request) => {
                        println!("Handling REQUEST -> Sending ACK");
                        let mut options = vec![
                            DhcpOption::MessageType(MessageType::Ack),
                            DhcpOption::SubnetMask(Ipv4Addr::new(255, 255, 255, 0)),
                            DhcpOption::Router(vec![Ipv4Addr::new(192, 168, 1, 1)]),
                            DhcpOption::ServerIdentifier(server_ip),
                        ];

                        let reply = DhcpPacket::create_reply(
                            &request,
                            MessageType::Ack,
                            server_ip,
                            next_server,
                            options,
                        );

                        let mut reply_bytes = reply.encode();
                        inject_pxe_options(&mut reply_bytes, "192.168.1.10", "bootx64.efi");
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
