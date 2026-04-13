# Agent Instructions for Deliverer

## Development Workflow

- **Build/Run**: Use `cargo run --manifest-path dhcp-server/Cargo.toml` to interact with the project.
- **Testing**: Use the provided `test_client.py` to verify DHCP responses.
- **Privileged Ports**: Running on port 67 requires `sudo`. For testing, pass a port > 1024 as an argument (e.g., `cargo run -- 1067`).

## Project Structure & Quirks

- **Nested Package**: The core logic resides in the `dhcp-server/` directory. Ensure you use the `--manifest-path` flag when running cargo commands from the root.
- **Manual Option Injection**: The project uses `dhcplease` for the core protocol, but due to crate limitations, PXE-specific options (66 and 67) are injected manually via byte manipulation in `src/main.rs`.
- **Testing Requirement**: The `test_client.py` expects a listening UDP socket. When testing, ensure the server is running before executing the script.

## Verification Commands

- **Run Server (Test mode)**: `cargo run --manifest-path dhcp-server/Cargo.toml --bin dhcp-server 1067`
- **Run Mock Client**: `python3 test_client.py`
