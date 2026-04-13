use dhcplease::packet::DhcpPacket;
use std::net::Ipv4Addr;

fn main() {
    // A minimal DHCP DISCOVER packet (hex representation for testing)
    // This is a very simplified mock of what a packet might look like
    // In a real scenario, we'd capture a real one.
    let mut mock_packet = vec![0u8; 300];

    // Set some basic fields to see if parsing works
    // op: 1 (boot request)
    mock_packet[0] = 1;
    // transaction id: 0x12345678
    mock_packet[4] = 0x12;
    mock_packet[5] = 0x34;
    mock_packet[6] = 0x56;
    mock_packet[7] = 0x78;
    // client hardware type: 1 (Ethernet)
    mock_packet[23] = 1;
    // client hardware address length: 6
    mock_packet[24] = 6;
    // client hardware address: 00:11:22:33:44:55
    mock_packet[28..34].copy_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

    // Try to parse it
    match DhcpPacket::from_bytes(&mock_packet) {
        Ok(packet) => println!("Successfully parsed mock packet!"),
        Err(e) => println!("Failed to parse mock packet: {:?}", e),
    }
}
