# Minimal DHCP Server for PXE Booting

A lightweight, minimal DHCP server written in Rust, designed specifically to facilitate PXE (Preboot Execution Environment) booting. This server allows you to redirect network clients to a specified netboot/TFTP server by injecting standard DHCP options.

## Features

- **PXE Support**: Automatically injects DHCP options 66 (TFTP Server Name) and 67 (Bootfile Name) into responses.
- **Full Handshake**: Handles the complete DHCP lifecycle: `DISCOVER` $\rightarrow$ `OFFER` $\rightarrow$ `REQUEST` $\rightarrow$ `ACK`.
- **Non-Privileged Testing**: Supports running on high ports (e.g., `1067`) to allow testing without `sudo` permissions.
- **Asynchronous**: Built on top of `tokio` for efficient, non-blocking network I/O.

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (Cargo)
- `python3` (for running the test client)

## Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/phaus/deliverer.git
   cd deliverer
   ```

2. Build the project:
   ```bash
   cd dhcp-server
   cargo build --release
   ```

## Usage

### Running with standard privileges (Port 67)
Since port 67 is a privileged port, you will need `sudo`:
```bash
sudo ./target/release/dhcp-server
```

### Running for testing (Custom Port)
To run without root privileges, specify a port greater than 1024:
```bash
./target/release/dhcp-server 1067
```

### Configuration
The server uses a `config.json` file for network settings (subnet, gateway, IP pool, etc.). A default configuration is generated automatically on the first run if one is not found.

## Testing

We have included a Python-based mock client to verify the server implementation.

1. In a separate terminal, start the server:
   ```bash
   cargo run --manifest-path dhcp-server/Cargo.toml --bin dhcp-server 1067
   ```

2. Run the test client:
   ```bash
   python3 test_client.py
   ```

The client will send a `DISCOVER` packet and verify that the server responds with an `OFFER` containing the correct Magic Cookie and PXE options.

## License

MIT
