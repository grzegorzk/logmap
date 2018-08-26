`clog`

Simple tool written in `Rust` to create log filters from known logs.
`Rust` documentation: https://doc.rust-lang.org/

Run tests:
`cargo test`

Build release binary:
`cargo build --release`
Binary will be created here: `./target/release/clog`

Analyse logs (example with systemd):
journalctl --since "10 years ago" -nall | clog > clog.result
