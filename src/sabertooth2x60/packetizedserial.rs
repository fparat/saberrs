use crate::{Error, Result, SabertoothSerial};

#[cfg(feature = "serialport")]
use crate::SabertoothPort;

pub const ADDRESS_MIN : u8 = 128;
pub const ADDRESS_MAX : u8 = 135;
pub const ADDRESS_DEFAULT: u8 = 128;

fn address_is_valid(address: u8) -> bool {
    address >= ADDRESS_MIN && address <= ADDRESS_MAX
}

#[derive(Debug)]
pub struct PacketizedSerial<T: SabertoothSerial> {
    dev: T,
    address: u8,
}

#[cfg(feature = "serialport")]
impl PacketizedSerial<SabertoothPort> {
    /// Create a new PacketizedSerial interface
    pub fn new(port: &str, address: u8) -> Result<Self> {
        if address_is_valid(address) {
            let dev = SabertoothPort::new(port)?;
            let saber = PacketizedSerial::from_serial(dev, address)?;
            Ok(saber)
        } else {
            let msg = format!("Invalid address {}, must be greater than 128", address);
            Err(Error::InvalidInput(msg))
        }
    }
}

impl<T: SabertoothSerial> PacketizedSerial<T> {
    pub fn from_serial(dev: T, address: u8) -> Result<Self> {
        if address_is_valid(address) {
            let saber = PacketizedSerial { dev, address };
            Ok(saber)
        } else {
            let msg = format!("Invalid address {}, must be greater than 128", address);
            Err(Error::InvalidInput(msg))
        }
    }

    fn write_frame(&mut self, txdata: &[u8]) -> Result<()> {
        Ok(self.dev.write_all(txdata)?)
    }
}
