# mgba_log

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Anders429/mgba_log/ci.yml?branch=master)](https://github.com/Anders429/mgba_log/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/mgba_log)](https://crates.io/crates/mgba_log)
[![docs.rs](https://docs.rs/mgba_log/badge.svg)](https://docs.rs/mgba_log)
[![License](https://img.shields.io/crates/l/mgba_log)](#license)

A logging implementation for mGBA.

Provides a logging implementation for the [`log`](https://docs.rs/log/latest/log/index.html) crate for logging when compiling for the Game Boy Advance and running within the [mGBA](https://mgba.io/) emulator.

mGBA supports logging at the following log levels using the associated logging macros. Every level provided by the `log` crate is supported except for `Trace`, as mGBA has no analog for the `Trace` log level, and this crate provides a macro for logging directly to `Fatal`.

| Level | Macro                | Notes                                                                             |
| ----- | -------------------- | --------------------------------------------------------------------------------- |
| Debug | [`log::debug!`](https://docs.rs/log/latest/log/macro.debug.html)      |                                                                                   |
| Info  | [`log::info!`](https://docs.rs/log/latest/log/macro.info.html)       |                                                                                   |
| Warn  | [`log::warn!`](https://docs.rs/log/latest/log/macro.warn.html)       |                                                                                   |
| Error | [`log::error!`](https://docs.rs/log/latest/log/macro.error.html)      |                                                                                   |
| Fatal | [`mgba_log::fatal!`](https://docs.rs/mgba_log/latest/mgba_log/macro.fatal.html) | Not a standard [`log`](https://docs.rs/log/latest/log/index.html) level. Only usable when using this logging implementation. |

## Usage

### In libraries
`mgba_log` should be used in binaries only. Libraries should instead use the logging facade provided by the [`log`](https://docs.rs/log/latest/log/index.html) crate directly.

### In binaries
When logging in a binary, only one logger may be enabled. Therefore, `mgba_log` cannot be used alongside any other logging implementations.

#### Installation
Add `mgba_log` as a dependency in your `Cargo.toml`:

``` toml
[dependencies]
mgba_log = "0.1.0"
```

Then call [`init()`](https://docs.rs/mgba_log/latest/mgba_log/fn.init.html) early in your binary. Any records logged before initialization will be silently dropped.

``` rust
fn main() {
    mgba_log::init().expect("unable to initialize mGBA logger");

    log::info!("Hello, world!");
}
```

Note that you may want to handle the returned [`Error`](https://docs.rs/mgba_log/latest/mgba_log/struct.Error.html) message from [`init()`](https://docs.rs/mgba_log/latest/mgba_log/fn.init.html) more robustly, unless you only want your project to be run in mGBA.

## Compatibility
This logger uses memory mapped IO registers specific to the Game Boy Advance. It is therefore only safe to use this library when building to run on the Game Boy Advance or a Game Boy Advance emulator.

If this logger is attempted to be initialized when not running on mGBA, it will fail to initialize with an [`Error`](https://docs.rs/mgba_log/latest/mgba_log/struct.Error.html) identifying the failure.

## License
This project is licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](https://github.com/Anders429/mgba_log/blob/HEAD/LICENSE-APACHE) or
http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](https://github.com/Anders429/mgba_log/blob/HEAD/LICENSE-MIT) or
http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
