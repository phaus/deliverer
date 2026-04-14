# Technical Specification: Apple Netboot

## Overview
Apple Netboot is a mechanism used by Apple hardware to boot an operating system over a network. It is based on the Boot Server Discovery Protocol (BSDP), which is loosely derived from DHCP and BootP. The process allows a client to locate a Netboot server and select an available OS image.

## BSDP Protocol Flow
The Netboot process typically follows these steps:
1. **DHCP Request**: The computer uses DHCP to request an IP address and related information.
2. **BSDP LIST**: The computer broadcasts a `BSDP: LIST` request on the local subnet to locate a Netboot server and available OS images.
3. **BSDP SELECT**: The computer informs the Netboot server about the selected image.
4. **TFTP Download**: The computer uses TFTP to download the boot file and the Mac boot process initiates.

## DHCP Options

To support Netboot, the DHCP server must facilitate the BSDP handshake, often by carrying vendor-specific options.

### Required Options

| Option Code | Name | Description |
| :--- | :--- | :--- |
| 43 | Vendor Specific Information | Used to carry BSDP/Netboot vendor-specific options. |
| 54 | Server Identifier | The IP address of the DHCP server. |
| 66 | TFTP Server Name | The hostname or IP address of the TFTP server used to download boot files. |
| 67 | Boot File Name | The name of the boot file to be downloaded from the TFTP server. |

### Vendor-Specific Options (Option 43) for BSDP

The BSDP protocol is embedded in the DHCP frame using **Option 43**.

#### BSDP LIST Response
When a client sends a `BSDP: LIST` request, the server should respond with a packet containing vendor-encapsulated options that describe the available services.

#### BSDP SELECT Response
When a client selects an image, the server responds with the necessary boot information. According to common implementations (e.g., for Mac Mini booting), the `vendor-encapsulated-options` might include specific byte sequences to indicate the boot service and the image details.

For example, a response might include:
- `08:04:81:00:00:89`: Indicating specific BSDP service parameters.
- Specific image identifiers within the encapsulated data.

## Implementation Notes

1. **Message Type**: The server must handle the full DHCP lifecycle (`DISCOVER` $\rightarrow$ `OFFER` $\rightarrow$ `REQUEST` $\rightarrow$ `ACK`) while being aware of the BSDP state machine.
2. **Option 43 Handling**: The server must be able to parse and correctly construct `Option 43` payloads containing the BSDP-specific Type-Length-Value (TLV) structures.
3. **End of Options**: All DHCP options must be terminated by the end-of-options marker (`0xFF` or `255`).
4. **Byte Ordering**: DHCP options follow a Type-Length-Value (TLV) format. For BSDP, the `Length` field in Option 43 refers to the total length of the encapsulated vendor-specific data.
