use std::time::Duration;

use crate::Result;

mod packetizedserial;

pub use packetizedserial::PacketizedSerial;

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
    fn set_baudrate(&mut self, baudrate: u32) -> Result<()>;

    /// Set the speed ramping value. This function estimates the command value
    /// that corresponds to the given ramp time according to the manual.
    fn set_ramp(&mut self, ramp: Duration) -> Result<()>;

    /// Set the deadband value. *ratio* is the deadband ratio between 0.0 and 1.0.
    fn set_deadband(&mut self, ratio: f32) -> Result<()>;
}
