# clog

Tool helps to spot new behavior in your logs so you don't have to analyse 1000s
of lines yourself. Create log filters from known logs. Use filters to flag unknown logs.

# Prerequisites

You need to have `Rust` installed in your system. For further details please see:
[Rust documentation](https://doc.rust-lang.org/)

You will also need `cargo` which should normally be part of `Rust` installation.

# Installation

Build binary from sources:
`cargo build --release`

Once built copy binary from following location:
`./target/release/clog`

If you want to you can also run tests to see if everything works as expected:
`cargo test`

# Usage

Analyse logs and save filters to a file (example with systemd):
`journalctl --since "10 years ago" -nall | clog -m -s clog.result`

Filter today's logs to see if there is anything that would require attention:
`journalctl --since "1 day ago" -nall | clog -l clog.result -p`

# Thanks
Big thank-you to:
- [Rust team](https://rust-lang.org/)
- [stackedit.io for online MarkDown editor](https://stackedit.io/app#)
