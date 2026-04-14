# Implementation Plan

## Proposed Features

- [ ] **MAC-Address to Boot Config Mapping**
  - Implement a way to map specific MAC addresses to unique boot configurations (e.g., different boot files or TFTP servers for different hardware).
  - This will allow granular control over which files are served to specific clients.

- [ ] **Netboot Support**
  - **Netboot 1**: Implement support for standard Netboot protocol.
  - **Netboot 2 (Apple)**: Implement support for the Netboot 2 protocol (used by Apple devices).
  - *Reference*: [kea-mboot](https://github.com/saybur/kea-mboot)

## Documentation & Research

- [ ] **Vagrant Test Environment**
  - Create a `Vagrantfile` to set up a local test network.
  - Configure one VM as the DHCP server (running the `dhcp-server` service).
  - Configure at least one VM as a DHCP client to verify PXE/DHCP functionality.
  - Ensure the network configuration allows communication between the server and client.
