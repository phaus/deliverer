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

- [ ] **Specifications Folder**
  - Create a `specs/` directory to house technical documentation and protocol definitions for the upcoming implementations.
