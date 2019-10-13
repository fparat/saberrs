use std::io;
use std::time::Duration;

use crate::error::Result;

/// Minimal serial port trait.
///
/// The Sabertooth interfaces will rely on this trait for low level
/// communications.
/// In most cases there is no need to handle this trait or its implementors
/// manually, it is better to use an interface constructor directly, for ex.
/// `PacketSerial::new("/dev/ttyS0")`.
/// A case where it would be useful to manipulate this trait is when a
/// particular serial setting is required.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use saberrs::{Result, SabertoothSerial, SabertoothPort};
/// use saberrs::sabertooth2x32::PacketSerial;
///
/// # fn example() -> Result<()> {
/// // Open a serial port with secific baud rate and timeout.
/// let mut dev = SabertoothPort::new("/dev/ttyS2")?;
/// dev.set_baud_rate(19200)?;
/// dev.set_timeout(Duration::from_secs(5))?;
///
/// // Use it with a PacketSerial interface.
/// let mut saber = PacketSerial::from(dev);
/// # Ok(())}
/// ```
pub trait SabertoothSerial: io::Write + io::Read {
    /// Set the timeout of the serial port.
    fn set_timeout(&mut self, timeout: Duration) -> Result<()>;

    /// Get the current timeout setting of the serial port.
    fn timeout(&self) -> Duration;

    /// Set the baud rate of the serial port.
    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()>;

    /// Get the current baud rate setting of the serial port.
    fn baud_rate(&self) -> Result<u32>;

    /// Clear the tx and rx buffer, remaining bytes will be lost.
    fn clear_all(&self) -> Result<()>;
}

/// `SabertoothPort` and `SabertoothPortShared` are optional concrete
/// implementations of the trait `SabertoothSerial`. Thay can be disabled for
/// cutting the dependency on the `serialport` external crate.
/// In this case the trait `SabertoothSerial` will need to be implemented
/// manually by the application.
#[cfg(feature = "serialport")]
pub mod sabertoothport {
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;
    use std::time::Duration;

    use serialport;
    use serialport::{ClearBuffer, DataBits, FlowControl, Parity, StopBits};
    use serialport::{SerialPort, SerialPortSettings};

    use crate::{Result, SabertoothSerial};

    /// Default baud rate setting when opening a `SabertoothPort`.
    const DEFAULT_BAUDRATE: u32 = 9600;

    /// Default timeout setting when opening a `SabertoothPort`.
    const DEFAULT_TIMEOUT_MS: u64 = 100;

    /// Default serial settings.
    const DEFAULT_SETTINGS: SerialPortSettings = SerialPortSettings {
        baud_rate: DEFAULT_BAUDRATE,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
    };

    /// Raw Sabertooth controller.
    ///
    /// It is a simple wrapper around a serial port handle and may be used for
    /// manually write and read bytes with the device.
    ///
    /// **Requires** the "serialport" feature (enabled by default).
    pub struct SabertoothPort {
        dev: Box<dyn SerialPort>,
    }

    impl SabertoothPort {
        /// Create a new `SabertoothPort` with a default configuration
        pub fn new(port: &str) -> Result<SabertoothPort> {
            let ser = serialport::open_with_settings(port, &DEFAULT_SETTINGS)?;
            Ok(SabertoothPort { dev: ser })
        }
    }

    impl SabertoothSerial for SabertoothPort {
        fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
            Ok(self.dev.set_timeout(timeout)?)
        }

        fn timeout(&self) -> Duration {
            self.dev.timeout()
        }

        fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()> {
            Ok(self.dev.set_baud_rate(baud_rate)?)
        }

        fn baud_rate(&self) -> Result<u32> {
            Ok(self.dev.baud_rate()?)
        }

        fn clear_all(&self) -> Result<()> {
            Ok(self.dev.clear(ClearBuffer::All)?)
        }
    }

    impl io::Read for SabertoothPort {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.dev.read(buf)
        }
    }

    impl io::Write for SabertoothPort {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.dev.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.dev.flush()
        }
    }

    impl std::fmt::Debug for SabertoothPort {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "SabertoothPort({:?})",
                self.dev.name().unwrap_or_else(|| String::from("_"))
            )
        }
    }

    /// Clonable variant of [SabertoothPort](struct.SabertoothPort.html).
    ///
    /// It is more flexible, for example it allows to mix several protocols
    /// (PlainText and PacketSerial) with the same port. However in most cases
    /// `SabertoothPort` is enough and recommended.
    ///
    /// The downside of `SabertoothPortShared`, besides possible performance loss,
    /// is that it is not
    /// [Send](https://doc.rust-lang.org/std/marker/trait.Send.html).
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use saberrs::{Result, SabertoothSerial, SabertoothPortShared};
    /// use saberrs::sabertooth2x32::{PacketSerial, PacketType, PlainText, Sabertooth2x32};
    ///
    /// # fn example() -> Result<()> {
    ///
    /// let mut dev = SabertoothPortShared::new("/dev/ttyS2")?;
    ///
    /// // The following interfaces will all communicate using the same port, but
    /// // using different protocols.
    /// let mut sabertext = PlainText::from(&dev);
    /// let mut saberchecksum = PacketSerial::from(&dev).with_packet_type(PacketType::Checksum);
    /// let mut sabercrc = PacketSerial::from(&dev).with_packet_type(PacketType::CRC);
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// **Requires** the "serialport" feature (enabled by default).
    #[derive(Clone)]
    pub struct SabertoothPortShared {
        dev: Rc<RefCell<Box<dyn SerialPort>>>,
    }

    impl SabertoothPortShared {
        /// Create a new `SabertoothPortShared` with a default configuration
        pub fn new(port: &str) -> Result<SabertoothPortShared> {
            let ser = serialport::open_with_settings(port, &DEFAULT_SETTINGS)?;
            Ok(SabertoothPortShared {
                dev: Rc::new(RefCell::new(ser)),
            })
        }
    }

    impl SabertoothSerial for SabertoothPortShared {
        fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
            Ok(self.dev.borrow_mut().set_timeout(timeout)?)
        }

        fn timeout(&self) -> Duration {
            self.dev.borrow_mut().timeout()
        }

        fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()> {
            Ok(self.dev.borrow_mut().set_baud_rate(baud_rate)?)
        }

        fn baud_rate(&self) -> Result<u32> {
            Ok(self.dev.borrow_mut().baud_rate()?)
        }

        fn clear_all(&self) -> Result<()> {
            Ok(self.dev.borrow_mut().clear(ClearBuffer::All)?)
        }
    }

    impl io::Read for SabertoothPortShared {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.dev.borrow_mut().read(buf)
        }
    }

    impl io::Write for SabertoothPortShared {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.dev.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.dev.borrow_mut().flush()
        }
    }

    impl std::fmt::Debug for SabertoothPortShared {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "SabertoothPortShared({:?})",
                self.dev
                    .borrow_mut()
                    .name()
                    .unwrap_or_else(|| String::from("_"))
            )
        }
    }
}
