## ledger-sdk-rust

Modular Rust SDK for communicating with Ledger devices.

Modules:

- `ledger-apdu`: APDU types and helpers
- `ledger-transport`: `Exchange` trait and abstractions
- `ledger-transport-hid`: HID transport (hidapi)
- `ledger-device-base`: Device/app info helpers (version, device info, app info)

Build:

```
cargo build
```

Typical flow:

1. Create a transport (e.g., HID)
2. Send APDUs or use helpers in `ledger-device-base`

License and attribution:

- Apache-2.0 (see `LICENSE`). Portions derived from Zondax `ledger-rs` (Apache-2.0); see `NOTICE`.
