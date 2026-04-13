# Technical Specification: Apple Netboot 2

## Overview
Apple Netboot 2 is a mechanism used by Apple hardware to boot an operating system over a network. This process relies on the DHCP protocol to provide the client with necessary boot information, such as the location of the boot files and the TFTP server.

## DHCP Options

To support Netboot 2, the DHCP server must provide specific options in its responses (OFFER and ACK).

### Required Options

| Option Code | Name | Description |
| :--- | :--- | :--- |
| 50 | Requested IP Address | In some implementations, specifically for PXE, the server may need to inject an IP address. |
| 66 | TFTP Server Name | The hostname or IP address of the TFTP server used to download boot files. |
| 67 | Boot File Name | The name of the boot file to be downloaded from the TFTP server. |
| 51 | IP Address Lease Time | The duration (in seconds) for which the offered IP address is valid. |
| 53 | DHCP Message Type | Indicates the type of DHCP message (e.g., OFFER, ACK). |
| 54 | Server Identifier | The IP address of the DHCP server. |

### Specific Implementations

#### PXE Option Injection
As seen in the current implementation in `dhcp-server/src/main.rs`, options 66 and 67 are often injected manually after the standard DHCP packet construction to ensure they are correctly placed before the end-of-options marker (`255`).

```rust
// Example of option injection logic
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
```

## Packet Requirements

1. **Message Type**: The server must respond to `DHCPDISCOVER` with a `DHCPOFFER` and to `DHCPREQUEST` with a `DHCPACK`.
2. **End of Options**: All DHCP options must be terminated by the end-of-options marker (`0xFF` or `255`).
3. **Byte Ordering**: DHCP options follow a Type-Length-Value (TLV) format, where the length is specified in bytes.
