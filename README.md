# saberrs

[![crates.io version badge](https://img.shields.io/crates/v/saberrs.svg)](https://crates.io/crates/saberrs)
[![Documentation](https://docs.rs/saberrs/badge.svg)](https://docs.rs/saberrs)
![License](https://img.shields.io/crates/l/log.svg)

`saberrs` is a library for interfacing with [Dimension Engineering]
Sabertooth motor driver.

Currently only the Sabertooth 2x32 is supported.

Full documentation: https://docs.rs/saberrs

## Simple usage

In `Cargo.toml`:

```toml
[dependencies]
saberrs = "0.2"
```

In application code:

```rust
use saberrs::sabertooth2x32::{Sabertooth2x32, PacketSerial};

// Create a handle. This will use "PacketSerial" protocol.
let mut saber = PacketSerial::new("/dev/ttyS0")?;

// Go forward at half-speed (50.0%)
saber.set_drive(50.0)?;
saber.set_turn(0.0)?;

// Request the battery voltage from motor 1.
let vbat : f32 = saber.get_voltage(1)?;

// Stop the motors
saber.stop_motors()?;

```

Other protocol variants can be used:

```rust
use saberrs::sabertooth2x32::{Sabertooth2x32, PacketSerial, PacketType, PlainText};

// "PacketSerial" with specified address and frame protection type.
let mut saber = PacketSerial::new("/dev/ttyS0")?
    .with_packet_type(PacketType::Checksum)
    .with_address(129);

// "PlainText" protocol
let mut sabertext = PlainText::new("/dev/ttyS1")?;
```

## Features and dependencies

Features:

- `serialport`, enabled by default, for providing default serial IO handlers.

Dependencies:

- [serialport] for the `serialport` feature.
- [log] for emitting logs.

## License

Licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


## Disclaimer

This library is not affiliated or associated in any way with Dimension Engineering.

All product and company names are trademarks or registered trademarks of
their respective holders. Use of them does not imply any affiliation with or
endorsement by them.

[Dimension Engineering]: https://www.dimensionengineering.com
[serialport]: https://crates.io/crates/serialport
[log]: https://crates.io/crates/log
