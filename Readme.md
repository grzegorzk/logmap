# flog

Tool helps to spot new behavior in your logs so you don't have to analyze 1000s
of lines yourself. Create log filters from known logs. Use filters to flag unknown logs.

[![Build Status](https://travis-ci.com/grzegorzk/flog.svg?branch=master)](https://travis-ci.com/grzegorzk/flog)

# Prerequisites

You need to have `Rust` installed in your system. For further details please see:
[Rust documentation](https://doc.rust-lang.org/)

You will also need `cargo` which should normally be part of `Rust` installation.

# Installation

Build binary from sources:
`cargo build --release`

Once built copy binary from following location:
`./target/release/flog`

If you want to you can also run tests to see if everything works as expected:
`cargo test --features=tst_utils`

# Usage

Analyse logs and save filters to a file (example with systemd):
`journalctl --since "10 years ago" -nall | ./target/release/flog -m -s flog.result`

Filter today's logs to see if there is anything that would require attention:
`journalctl --since "1 day ago" -nall | ./target/release/flog -l flog.result -p`

# How it works

`flog` counts matching words across known filters.

If invoked in learning mode it will allow some words not to match and extend
best-matching filter to contain some word alternatives. If no best-matching filter
was found it will then add new filter.

If invoked in passive (analysis) mode it will not allow for any non-matching words
and print to standard error stream all lines with no matching filter.
Empty output means there are no unseen logs in the input stream.

# Thanks
Big thank-you to:
- [Rust team](https://rust-lang.org/)
- [stackedit.io for online MarkDown editor](https://stackedit.io/app#)
