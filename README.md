# ctrl-z

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Anders429/ctrl-z/CI)](https://github.com/Anders429/ctrl-z/actions)
[![codecov.io](https://img.shields.io/codecov/c/gh/Anders429/ctrl-z)](https://codecov.io/gh/Anders429/ctrl-z)
[![crates.io](https://img.shields.io/crates/v/ctrl-z)](https://crates.io/crates/ctrl-z)
[![docs.rs](https://docs.rs/ctrl-z/badge.svg)](https://docs.rs/ctrl-z)
[![MSRV](https://img.shields.io/badge/rustc-1.0.0+-yellow.svg)](#minimum-supported-rust-version)
[![License](https://img.shields.io/crates/l/ctrl-z)](#license)

A composable reader to treat `0x1A` as an end-of-file marker.

Historically, `0x1A` (commonly referred to as `CTRL-Z`, `^Z`, or a "substitute character") was used
in old systems to explicitly mark the end of a file. While modern systems no longer require this
practice, some legacy files still contain this byte to mark the end of a file. This library
provides a reader to treat `0x1A` as the end of a file, rather than reading it as a regular byte.

## Usage
This library provides a reader in the form of a `struct` named `ReadToCtrlZ`. As is common
practice, this reader is composable with other types implementing the
[`Read`](https://doc.rust-lang.org/std/io/trait.Read.html) or
[`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) traits. The reader checks the
returned bytes for the presence of the EOF marker `0x1A` and stops reading when it is encountered.

### Example
For example, the reader defined below only reads until the `0x1A` byte, at which point it stops
reading.

``` rust
use ctrl_z::ReadToCtrlZ;

let mut output = String::new();

let mut reader = ReadToCtrlZ::new(b"foo\x1abar".as_slice());

assert_eq!(reader.read_to_string(&mut output), Ok(3));
assert_eq!(output, "foo");
```

## Minimum Supported Rust Version
This crate is guaranteed to compile on stable `rustc 1.0.0` and up.

## License
This project is licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](https://github.com/Anders429/ctrl-z/blob/HEAD/LICENSE-APACHE) or
http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](https://github.com/Anders429/ctrl-z/blob/HEAD/LICENSE-MIT) or
http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
