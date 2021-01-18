//! `saberrs` is a library for interfacing with [Dimension Engineering]
//! Sabertooth motor driver.
//!
//! Currently only the Sabertooth 2x32 is supported.
//!
//! # Simple usage
//!
//! ```rust
//! # use saberrs::Result;
//! use saberrs::sabertooth2x32::{Sabertooth2x32, PacketSerial};
//!
//! # fn example() -> Result<()> {
//! // Create a handle. This will use "PacketSerial" protocol.
//! let mut saber = PacketSerial::new("/dev/ttyS0")?;
//!
//! // Go forward at half-speed (50.0%)
//! saber.set_drive(0.5)?;
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
//! use saberrs::sabertooth2x32::{Sabertooth2x32, PacketSerial, PacketType, PlainText};
//! # use saberrs::Result;
//!
//! # fn example() -> Result<()> {
//! // "PacketSerial" with specified address and frame protection type.
//! let mut saber = PacketSerial::new("/dev/ttyS0")?
//!     .with_packet_type(PacketType::Checksum)
//!     .with_address(129);
//!
//! // "PlainText" protocol
//! let mut sabertext = PlainText::new("/dev/ttyS1")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Customizing the IO: the `SabertoothSerial` trait
//!
//! The handles rely on the trait [SabertoothSerial], which abstract the
//! low-level IO communication with the device.
//!
//! By default, the library provides [SabertoothPort] and
//! [SabertoothPortShared]. In most cases the application writer shouldn't need
//! to care about those, but they may be used for applying custom baud rates or
//! timeout values for example.
//!
//! [SabertoothSerial] can be implemented manually for even more customization.
//! For example stubs can be implemented for debugging purpose:
//!
//! ```rust
//! use std::time::Duration;
//! use std::io::{self, Read, Write};
//! use saberrs::{SabertoothSerial};
//!
//! struct SerialStub {
//!     timeout: Duration,
//!     baudrate: u32,
//! }
//!
//! impl SerialStub {
//!     pub fn new() -> Self {
//!         SerialStub {
//!             timeout: Duration::from_millis(100),
//!             baudrate: 9600,
//!         }
//!     }
//! }
//!
//! impl Read for SerialStub {
//!     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//!         println!("SerialStub.read()");
//!         Ok(0)
//!     }
//! }
//!
//! impl Write for SerialStub {
//!     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//!         println!("SerialStub.write({:?})", buf);
//!         Ok(buf.len())
//!     }
//!
//!     fn flush(&mut self) -> io::Result<()> { Ok(()) }
//! }
//!
//! impl SabertoothSerial for SerialStub {
//!     fn set_timeout(&mut self, timeout: Duration) -> saberrs::Result<()> {
//!         println!("SerialStub.set_timeout({:?})", timeout);
//!         self.timeout = timeout;
//!         Ok(())
//!     }
//!
//!     fn timeout(&self) -> Duration {
//!         println!("SerialStub.timeout() -> {:?}", self.timeout);
//!         self.timeout
//!     }
//!
//!     fn set_baud_rate(&mut self, baud_rate: u32) -> saberrs::Result<()> {
//!         println!("SerialStub.set_baudrate({})", baud_rate);
//!         self.baudrate = baud_rate;
//!         Ok(())
//!     }
//!
//!     fn baud_rate(&self) -> saberrs::Result<u32> {
//!         println!("SerialStub.baud_rate() -> {}", self.baudrate);
//!         Ok(self.baudrate)
//!     }
//!
//!     fn clear_all(&self) -> saberrs::Result<()> { Ok(()) }
//! }
//! ```
//!
//!
//! # Features and dependencies
//!
//! Features:
//!
//! - `serialport`, enabled by default, allows the usage of the crate
//! [serialport] for providing [SabertoothPort] and [SabertoothPortShared].
//! If this feature is disabled [SabertoothSerial] needs to be implemented
//! manually.
//!
//! Dependencies:
//!
//! - [serialport] for the `serialport` feature.
//! - [log] for emitting logs.
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
//! [Sabertooth 2x60]: https://www.dimensionengineering.com/products/sabertooth2x60
//! [SabertoothSerial]: trait.SabertoothSerial.html
//! [SabertoothPort]: struct.SabertoothPort.html
//! [SabertoothPortShared]: struct.SabertoothPortShared.html
//! [serialport]: https://crates.io/crates/serialport
//! [log]: https://crates.io/crates/log

pub use error::{Error, Result};
pub use port::SabertoothSerial;

#[cfg(feature = "serialport")]
pub use port::sabertoothport::{SabertoothPort, SabertoothPortShared};

#[macro_use]
mod utils;

mod error;
mod port;

/// Interface for the [Sabertooth 2x32].
///
/// [Sabertooth 2x32]: https://www.dimensionengineering.com/products/sabertooth2x32
pub mod sabertooth2x32;

/// Interface for the [Sabertooth 2x60].
///
/// [Sabertooth 2x60]: https://www.dimensionengineering.com/products/sabertooth2x60
pub mod sabertooth2x60;
