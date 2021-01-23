#![allow(clippy::manual_range_contains)]

use crate::{Error, Result, SabertoothSerial};

#[cfg(feature = "serialport")]
use crate::SabertoothPort;

use super::{Baudrate, Sabertooth2x60};

pub const ADDRESS_MIN: u8 = 128;
pub const ADDRESS_MAX: u8 = 135;

pub const COMMAND_DRIVE_FORWARD_MOTOR_1: u8 = 0;
pub const COMMAND_DRIVE_BACKWARDS_MOTOR_1: u8 = 1;
pub const COMMAND_MIN_VOLTAGE: u8 = 2;
pub const COMMAND_MAX_VOLTAGE: u8 = 3;
pub const COMMAND_DRIVE_FORWARD_MOTOR_2: u8 = 4;
pub const COMMAND_DRIVE_BACKWARDS_MOTOR_2: u8 = 5;
pub const COMMAND_DRIVE_MOTOR_1: u8 = 6;
pub const COMMAND_DRIVE_MOTOR_2: u8 = 7;
pub const COMMAND_DRIVE_FORWARD_MIXED: u8 = 8;
pub const COMMAND_DRIVE_BACKWARDS_MIXED: u8 = 9;
pub const COMMAND_TURN_RIGHT_MIXED: u8 = 10;
pub const COMMAND_TURN_LEFT_MIXED: u8 = 11;
pub const COMMAND_DRIVE_FORWARDS_BACK: u8 = 12;
pub const COMMAND_TURN_7_BIT: u8 = 13;
pub const COMMAND_SERIAL_TIMEOUT: u8 = 14;
pub const COMMAND_BAUDRATE: u8 = 15;
pub const COMMAND_RAMPING: u8 = 16;
pub const COMMAND_DEADBAND: u8 = 17;

fn address_is_valid(address: u8) -> bool {
    address >= ADDRESS_MIN && address <= ADDRESS_MAX
}

fn checksum(address: u8, command: u8, data: u8) -> u8 {
    ((address as u32 + command as u32 + data as u32) & 0x7f) as u8
}

fn ratio_to_0_127(ratio: f32) -> Result<u8> {
    ratio_to_value_range!(ratio, 0, 127).map(|v| v as u8)
}

fn err_motor<T>(motor: usize) -> Result<T> {
    Err(Error::InvalidInput(format!(
        "invalid motor value {}; should be 1 or 2",
        motor
    )))
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

    fn make_packet(&self, command: u8, data: u8) -> [u8; 4] {
        let chk = checksum(self.address, command, data);
        [self.address, command, data, chk]
    }
}

#[allow(unused_variables)]
impl<T: SabertoothSerial> Sabertooth2x60 for PacketizedSerial<T> {
    fn set_drive_motor(&mut self, motor: usize, ratio: f32) -> Result<()> {
        let (command, data) = match (motor, ratio) {
            (1, ratio) if ratio >= 0. => (COMMAND_DRIVE_FORWARD_MOTOR_1, ratio_to_0_127(ratio)?),
            (1, ratio) if ratio < 0. => (COMMAND_DRIVE_BACKWARDS_MOTOR_1, ratio_to_0_127(-ratio)?),
            (2, ratio) if ratio >= 0. => (COMMAND_DRIVE_FORWARD_MOTOR_2, ratio_to_0_127(ratio)?),
            (2, ratio) if ratio < 0. => (COMMAND_DRIVE_BACKWARDS_MOTOR_2, ratio_to_0_127(-ratio)?),
            _ => return err_motor(motor),
        };
        let packet = self.make_packet(command, data);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_min_voltage(&mut self, volts: f32) -> Result<()> {
        let data = ((volts - 6.) * 5.) as i32;
        if data < 0 || data > 120 {
            let msg = format!("min voltage {} out of range, must within 6-30 volts", volts);
            return Err(Error::InvalidInput(msg));
        }
        let packet = self.make_packet(COMMAND_MIN_VOLTAGE, data as u8);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_max_voltage(&mut self, volts: f32) -> Result<()> {
        if volts < 0. || volts > 25. {
            let msg = format!("max voltage {} out of range, must within 0-25 volts", volts);
            return Err(Error::InvalidInput(msg));
        }
        let data = (volts * 5.12f32) as u8;
        let packet = self.make_packet(COMMAND_MAX_VOLTAGE, data as u8);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_drive_mixed(&mut self, ratio: f32) -> Result<()> {
        let (command, data) = match ratio {
            ratio if ratio >= 0. => (COMMAND_DRIVE_FORWARD_MIXED, ratio_to_0_127(ratio)?),
            ratio if ratio < 0. => (COMMAND_DRIVE_BACKWARDS_MIXED, ratio_to_0_127(-ratio)?),
            _ => return Err(Error::InvalidInput(format!("Invalid ratio {}", ratio))),
        };
        let packet = self.make_packet(command, data);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_turn_mixed(&mut self, ratio: f32) -> Result<()> {
        let (command, data) = match ratio {
            ratio if ratio >= 0. => (COMMAND_TURN_RIGHT_MIXED, ratio_to_0_127(ratio)?),
            ratio if ratio < 0. => (COMMAND_TURN_LEFT_MIXED, ratio_to_0_127(-ratio)?),
            _ => return Err(Error::InvalidInput(format!("Invalid ratio {}", ratio))),
        };
        let packet = self.make_packet(command, data);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_serial_timeout(&mut self, timeout: std::time::Duration) -> Result<()> {
        let command = COMMAND_SERIAL_TIMEOUT;
        let data = ((timeout.as_millis() + 99) / 100) as u8;
        if data > 127 {
            let msg = format!("Timeout {}ms out of range", timeout.as_millis());
            return Err(Error::InvalidInput(msg));
        }
        let packet = self.make_packet(command, data);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_baudrate(&mut self, baudrate: Baudrate) -> Result<()> {
        let data = match baudrate {
            Baudrate::B2400 => 1,
            Baudrate::B9600 => 2,
            Baudrate::B19200 => 3,
            Baudrate::B38400 => 4,
            Baudrate::B115200 => 5,
        };
        let packet = self.make_packet(COMMAND_BAUDRATE, data);
        self.write_frame(&packet)?;
        Ok(())
    }

    fn set_ramp(&mut self, ramp: std::time::Duration) -> Result<()> {
        todo!()
    }

    fn set_deadband(&mut self, ratio: f32) -> Result<()> {
        // check negativity, ratio_to_0_127() accept -1.0~1.0
        if ratio < 0.0 {
            let msg = "the deadband ratio must be positive".to_string();
            return Err(Error::InvalidInput(msg));
        }
        let data = ratio_to_0_127(ratio)?;
        let packet = self.make_packet(COMMAND_DEADBAND, data);
        self.write_frame(&packet)?;
        Ok(())
    }
}
