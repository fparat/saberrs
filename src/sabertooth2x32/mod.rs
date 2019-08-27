use crate::Result;

mod packetserial;
mod plaintext;

pub use packetserial::{PacketSerial, PacketType, DEFAULT_ADDRESS, DEFAULT_PACKET_TYPE};
pub use plaintext::PlainText;

/// Trait exposing the available methods for controlling the Sabertooth 2x32.
/// Note: implementors may also provide additional methods.
pub trait Sabertooth2x32 {
    ///  Returns the motor channel from a shutdown state to normal operation.
    fn startup(&mut self, channel: usize) -> Result<()>;

    /// Shuts off the motor output. Using the shutdown command will put the motor
    /// in a hard brake state.
    fn shutdown(&mut self, channel: usize) -> Result<()>;

    /// Set the speed of the selected motor.
    /// *channel* is 1 or 2, *percent* is a percentage between -100.0 for full
    /// backward and 100.0 for full forward (so 0.0 stops the motor).
    fn set_speed(&mut self, channel: usize, percent: f32) -> Result<()>;

    /// Get the current speed of the motor. See set_motor() for the values range.
    fn get_speed(&mut self, channel: usize) -> Result<f32>;

    /// Stop the motors, ie. set both speeds to zero.
    fn stop_motors(&mut self) -> Result<()> {
        self.set_speed(1, 0.0)?;
        self.set_speed(2, 0.0)?;
        Ok(())
    }

    /// Set the drive. *percent* is a percentage between -100.0 for full backward
    /// and 100.0 for full forward.
    /// Note: Both set_drive() and set_turn() must have been set at least once
    /// for having an effect.
    fn set_drive(&mut self, percent: f32) -> Result<()>;

    /// Set the turn value. *percent* is a percentage between -100.0 for full
    /// left and 100.0 for full right.
    /// Note: Both set_drive() and set_turn() must have been set at least once
    /// for having an effect.
    fn set_turn(&mut self, percent: f32) -> Result<()>;

    /// Set the power output of the selected motor. *channel* is 1 or 2, and
    /// *percent* is a percentage between -100.0 and 100.0.
    fn set_power(&mut self, channel: usize, percent: f32) -> Result<()>;

    /// Return the current power output of the motor. *channel* is 1 or 2, and
    /// the returned value is a percentage between -100.0 and 100.0.
    fn get_power(&mut self, channel: usize) -> Result<f32>;

    /// Set the speed ramping of the motor.
    fn set_ramp(&mut self, channel: usize, percent: f32) -> Result<()>;

    fn set_aux(&mut self, channel: usize, percent: f32) -> Result<()>;

    /// Get the battery voltage on the selected motor, in volts.
    fn get_voltage(&mut self, channel: usize) -> Result<f32>;

    /// Get the motor current in amperes. Positive current values mean energy is
    /// being drawn from the battery, and negative values indicate energy is
    /// being regenerated into the battery. Note: this noisy signal may vary by
    /// several amps, this is normal.
    fn get_current(&mut self, channel: usize) -> Result<f32>;

    /// Get the temperature of the output transistors for this channel, in
    /// degrees celsius.
    fn get_temperature(&mut self, channel: usize) -> Result<f32>;
}
