use std::time::Duration;

use crate::Result;

pub mod packetizedserial;

pub use packetizedserial::PacketizedSerial;

/// Possible serial baudrates for command 15
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Baudrate {
    B2400,
    B9600,
    B19200,
    B38400,
    B115200,
}

/// Trait exposing the available methods for controlling the Sabertooth 2x60.
pub trait Sabertooth2x60 {
    /// Set the drive value for a motor. *motor* is 1 or 2, and *ratio* is a
    /// ratio between -1.0 for full backward and 1.0 for full forward.
    fn set_drive_motor(&mut self, motor: usize, ratio: f32) -> Result<()>;

    /// Set the minimum voltage. If the battery voltage drops below this value
    /// the output will be shut down. This value is not persistant and must be
    /// set each run.
    fn set_min_voltage(&mut self, volts: f32) -> Result<()>;

    /// Set the maximum voltage. If the battery voltage increases above this
    /// value the moter will be put into a hard brake until the voltage drops
    /// below the set point again.
    fn set_max_voltage(&mut self, volts: f32) -> Result<()>;

    /// Set the drive value in mixed mode. *ratio* is the ratio between -1.0 for
    /// for full backward and 1.0 for full forward.
    /// The Sabertooth requires valid data for both drive mixed and turn mixed
    /// before it begins to operate.
    /// Caution, avoid mixing mixed mode commands with independant mode commands.
    fn set_drive_mixed(&mut self, ratio: f32) -> Result<()>;

    /// Set the turn value in mixed mode. *ratio* is the ratio between -1.0 for
    /// for full left and 1.0 for full right.
    /// The Sabertooth requires valid data for both drive mixed and turn mixed
    /// before it begins to operate.
    /// Caution, avoid mixing mixed mode commands with independant mode commands.
    fn set_turn_mixed(&mut self, ratio: f32) -> Result<()>;

    /// Set the time after which the motor driver will shut off if it has not
    /// received a command. 0 will disable the timeout. This setting is *not*
    /// persistent between power cycles.
    fn set_serial_timeout(&mut self, timeout: Duration) -> Result<()>;

    /// Set the serial baudrate. Valid values are: 2400, 9600 (default), 19200,
    /// 38400 and 115200. This setting *does* persist between power cycles.
    fn set_baudrate(&mut self, baudrate: Baudrate) -> Result<()>;

    /// Set the speed ramping value. This function estimates the command value
    /// that corresponds to the given ramp time according to the manual.
    /// From the manual it is not clear if zero is an accepted value, so in
    /// doubt the method allows it.
    fn set_ramp(&mut self, ramp: Duration) -> Result<()>;

    /// Set the deadband value. *ratio* is the deadband ratio between 0.0 and 1.0.
    fn set_deadband(&mut self, ratio: f32) -> Result<()>;

    /// Get error conditions
    fn get_errors(&mut self) -> Result<ErrorConditions>;

    /// Get the temperature of a motor, in degrees celsius.
    fn get_temperature(&mut self, motor: usize) -> Result<f32>;

    /// Get the battery voltage in volts.
    fn get_voltage(&mut self) -> Result<f32>;

    /// Get the motor duty-cycle. UNSTABLE: unsure about format.
    fn get_duty_cycle(&mut self, motor: usize) -> Result<f32>;
}

/// Combination of error conditions returned by the device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorConditions(pub u8);

#[rustfmt::skip]
impl ErrorConditions {
    /// The raw value returned by the device.
    #[inline] pub fn value(&self)           -> u8   { self.0 }
    /// If there is no error condition.
    #[inline] pub fn is_ok(&self)           -> bool { self.0 == 0 }
    /// Overcurrent.
    #[inline] pub fn overcurrent(&self)     -> bool { (self.0 & (1 << 0)) != 0 }
    /// Overvoltage.
    #[inline] pub fn overvoltage(&self)     -> bool { (self.0 & (1 << 1)) != 0 }
    /// Overtemperature.
    #[inline] pub fn overtemperature(&self) -> bool { (self.0 & (1 << 2)) != 0 }
    /// Undervoltage.
    #[inline] pub fn undervoltage(&self)    -> bool { (self.0 & (1 << 3)) != 0 }
    /// The motor 1 is depowered.
    #[inline] pub fn deadband_1(&self)      -> bool { (self.0 & (1 << 5)) != 0 }
    /// The motor 2 is depowered.
    #[inline] pub fn deadband_2(&self)      -> bool { (self.0 & (1 << 6)) != 0 }
    /// In timeout
    #[inline] pub fn timeout(&self)         -> bool { (self.0 & (1 << 7)) != 0 }
}
