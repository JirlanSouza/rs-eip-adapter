# rs-eip-adapter

A minimal EtherNet/IP (ENIP) adapter implemented in Rust. This project provides a lightweight ENIP encapsulation handler that responds to discovery requests (e.g., `ListIdentity`) over UDP and a small CIP model (Identity and TCP/IP Interface) as a foundation for further development.

Quick summary
- Purpose: Respond to ENIP discovery (`ListIdentity`) via UDP.
- Status: Prototype — discovery support implemented; other ENIP/CIP features partial or missing.
- Bind address: 0.0.0.0:44818 (EIP reserved port).

Build and run
1. Build:
   ```
   cargo build --release
   ```
2. Run:
   ```
   RUST_LOG=info cargo run --release
   ```

What it does
- Binds a UDP socket to the EtherNet/IP reserved port and listens for ENIP encapsulation packets.
- Parses ENIP encapsulation header, validates requests, and dispatches supported commands.
- Implements a CPF encoder used to construct `ListIdentity` responses based on registered CIP instances.

Project layout (brief)
- `src/`
  - `cip/` — CIP model (traits, Identity, TCP/IP Interface, Registry).
  - `encap/` — ENIP helpers (commands, errors, CPF encoder, list_identity handler).
  - `encap.rs` — `EncapsulationHandler` and header encoding/decoding.
  - `transport/udp.rs` — UDP listener that forwards datagrams to the handler.
  - `eip_stack.rs` — bootstrap: creates `Registry`, registers classes and starts transport.
  - `main.rs` — application entry point (creates `EipStack` with identity info).

Development notes
- The code uses `tokio` for async I/O and `bytes` for buffer manipulation.
- The current handler supports `ListIdentity` and `Nop`. Additional ENIP commands and CIP services should be added as needed.
- Consider adding:
  - Unit tests for binary encoders/decoders (Encapsulation header, CPF).
  - Stronger error types and consistent endianness handling.
  - Session management for `RegisterSession`.
  - Per-packet concurrency to avoid blocking the UDP receive loop.
  - Graceful shutdown and configuration options.

License
- This project is licensed under the MIT License. See `LICENSE` in the repository root.

Contributing
- Fork the repository, create a feature or fix branch, and open a pull request. Include tests and a short description of the change.

Contact
- Open an issue in the repository for questions, bugs, or proposals.