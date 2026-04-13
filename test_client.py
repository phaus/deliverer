import socket
import struct


def create_dhcp_discover():
    # Minimal DHCP Discover packet (RFC 2131)
    # Length must be at least 240 bytes (header + magic cookie)

    packet = bytearray(300)  # Padding to 300 bytes

    # op=1 (boot request)
    packet[0] = 0x01
    # htype=1 (Ethernet)
    packet[1] = 0x01
    # hlen=6
    packet[2] = 0x06
    # hops=0
    packet[3] = 0x00
    # xid (transaction id)
    packet[4:8] = b"\x12\x34\x56\x78"
    # secs=0
    packet[8:10] = b"\x00\x00"
    # flags=0 (unicast)
    packet[10:12] = b"\x00\x00"
    # ciaddr=0.0.0.0
    packet[12:16] = b"\x00\x00\x00\x00"
    # yiaddr=0.0.0.0
    packet[16:20] = b"\x00\x00\x00\x00"
    # siaddr=0.0.0.0
    packet[20:24] = b"\x00\x00\x00\x00"
    # giaddr=0.0.0.0
    packet[24:28] = b"\x00\x00\x00\x00"
    # chaddr (client hardware address) - 6 bytes
    packet[28:34] = b"\x00\x11\x22\x33\x44\x55"
    # htype=1, hlen=6, padding...
    # sname (64 bytes)
    # file (128 bytes)
    # Magic Cookie (at offset 236)
    packet[236:240] = b"\x63\x82\x53\x63"

    # Options
    # Option 53: DHCP Message Type (1 = Discover)
    # [Code, Length, Value]
    packet[240:243] = b"\x35\x01\x01"

    # Option 55: Parameter Request List (to ask for subnet, router, etc)
    # Let's just send some basic ones to be safe
    packet[243:246] = b"\x37\x03\x01\x01\x01"

    # End option
    packet[246] = 0xFF

    return packet


def run_test(port=1067):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    sock.settimeout(3.0)

    print(f"Sending DHCP Discover to localhost:{port}...")
    discover_packet = create_dhcp_discover()

    try:
        sock.sendto(discover_packet, ("127.0.0.1", port))
        data, addr = sock.recvfrom(1500)
        print(f"Received response from {addr}")
        print(f"Response length: {len(data)} bytes")

        # Check for Magic Cookie
        if b"\x63\x82\x53\x63" in data:
            print("Success: Magic Cookie found!")
            # Check if it's an OFFER (Option 53 = 2)
            if b"\x35\x01\x02" in data:
                print("Success: Message Type is OFFER!")
            # Check for PXE options (66 or 67)
            if b"\x42" in data or b"\x43" in data:
                print("Success: PXE Options (66/67) found in response!")
            else:
                print("Warning: PXE Options not found in response.")
        else:
            print("Failure: Magic Cookie not found!")

    except socket.timeout:
        print("Error: Test timed out. Is the server running?")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        sock.close()


if __name__ == "__main__":
    run_test()
