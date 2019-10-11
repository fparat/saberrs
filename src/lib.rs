//! `saberrs` is a library for interfacing with [Dimension Engineering]
//! Sabertooth motor driver.
//!
//! Currently only the Sabertooth 2x32 is supported.
//!
//! # Simple usage
//!
//! ```rust
//! use saberrs::sabertooth2x32;
//! use saberrs::Result;
//! use saberrs::sabertooth2x32::Sabertooth2x32;
//!
//! # fn example() -> Result<()> {
//! // Create a handle. This will use "PacketSerial" protocol.
//! let mut saber = sabertooth2x32::PacketSerial::new("/dev/ttyS0")?;
//!
//! // Go forward at half-speed (50.0%)
//! saber.set_drive(50.0)?;
//! saber.set_turn(0.0)?;
//!
//! // Request the battery voltage from motor 1.
//! let vbat : f32 = saber.get_voltage(1)?;
//!
//! // Stop the motors
//! saber.stop_motors()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! Other protocol variants can be used:
//!
//! ```rust
//!  use saberrs::sabertooth2x32;
//!  use saberrs::sabertooth2x32::{Sabertooth2x32, PacketType};
//!  use saberrs::Result;
//!
//! # fn example() -> Result<()> {
//! // "PacketSerial" with specified address and frame protection type.
//! let mut saber = sabertooth2x32::PacketSerial::new("/dev/ttyS0")?
//!     .with_packet_type(PacketType::Checksum)
//!     .with_address(129);
//!
//! // "PlainText" protocol
//! let mut sabertext = sabertooth2x32::PlainText::new("/dev/ttyS1")?;
//! # Ok(())
//! # }
//! ```
//!
//! # More advanced: serial settings and port sharing
//!
//! The serial port is encapsulated in a trait [SabertoothSerial], implemented
//! by [SabertoothPort] and [SabertoothPortShared], which allow to manually
//! specify some serial port settings (although there are far less options than
//! a raw serial port), or mix multiple protocols (PacketSerial or PlainText).
//!
//! # License
//!
//! Licensed under either of
//!
//! * Apache License, Version 2.0
//! ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
//! * MIT license
//! ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//! # Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
//! dual licensed as above, without any additional terms or conditions.
//!
//!
//! # Disclaimer
//!
//! This library is not affiliated or associated in any way with Dimension Engineering.
//!
//! All product and company names are trademarks or registered trademarks of
//! their respective holders. Use of them does not imply any affiliation with or
//! endorsement by them.
//!
//! [Dimension Engineering]: https://www.dimensionengineering.com
//! [Sabertooth 2x32]: https://www.dimensionengineering.com/products/sabertooth2x32
//! [SabertoothSerial]: trait.SabertoothSerial.html
//! [SabertoothPort]: struct.SabertoothPort.html
//! [SabertoothPortShared]: struct.SabertoothPortShared.html

pub use error::{Error, ErrorKind, Result};
pub use port::SabertoothSerial;

#[cfg(feature="serialport")]
pub use port::sabertoothport::{SabertoothPort, SabertoothPortShared};

#[macro_use]
mod utils;

mod error;
mod port;

/// Interface for the [Sabertooth 2x32].
///
/// [Sabertooth 2x32]: https://www.dimensionengineering.com/products/sabertooth2x32
pub mod sabertooth2x32;
