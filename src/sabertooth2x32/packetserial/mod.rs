#[allow(unused_imports)]
use log::debug;

use crate::error::{Error, Result};
use crate::port::SabertoothSerial;
use crate::sabertooth2x32::Sabertooth2x32;
use crate::utils;

#[cfg(feature = "serialport")]
use crate::port::sabertoothport::SabertoothPort;

mod checksum;
mod crc;

#[cfg(debug_assertions)]
macro_rules! dbg_frame {
    ($head:ident, $frame:expr) => {
        debug!("{} = {:?}", stringify!($head), $frame);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! dbg_frame {
    ($head:ident, $frame:expr) => {};
}

/// Default address for packet communication.
pub const DEFAULT_ADDRESS: u8 = 128;

/// Default packet type when creating a [PacketSerial](struct.PacketSerial.html)
pub const DEFAULT_PACKET_TYPE: PacketType = PacketType::CRC;

const CMD_NUM_SET: u8 = 40;
const CMD_NUM_GET: u8 = 41;
const CMD_NUM_REPLY: u8 = 73;

const PACKET_MAX_REPLY_SIZE: usize = crc::PACKET_REPLY_SIZE;

/// Type of frame protection for [PacketSerial](struct.PacketSerial.html).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PacketType {
    /// Manual extract:
    /// > * good for most applications
    /// > * easier to implement
    /// > * faster updates (uses one less byte per command than a CRC)
    Checksum,

    /// Manual extract:
    /// > * good for safety-critical applications with noisier wiring
    /// > * harder to implement
    /// > * slower updates (uses one more byte per command than a checksum)
    /// > * provides a Hamming distance of 4
    CRC,
}

pub enum ParseError {
    PacketSize,
    ChecksumError,
    AddressError,
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommandSet {
    Value = 0,
    KeepAlive = 16,
    Shutdown = 32, // can also be used for startup (name follows the doc)
    Timeout = 64,
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommandGet {
    Value = 0,
    Battery = 16,
    Current = 32,
    Temperature = 64,
}

fn pack_data_value(value: u16) -> [u8; 2] {
    [(value & 127) as u8, ((value >> 7) & 127) as u8]
}

fn unpack_data_value(buf: &[u8]) -> u16 {
    u16::from(buf[0] & 127) + (u16::from(buf[1] & 127) << 7)
}

/// Interface using the "Packet Serial" protocol with checksum or CRC.
pub struct PacketSerial<T: SabertoothSerial> {
    dev: T,
    address: u8,
    packet_type: PacketType,
}

#[cfg(feature = "serialport")]
impl PacketSerial<SabertoothPort> {
    /// Open a serial port and return a new instance of `PacketSerial` with
    /// default settings. By default the address is `128` and the protection
    /// type is `PacketType::CRC`.
    ///
    /// # Example
    ///
    /// ```
    /// use saberrs::sabertooth2x32::PacketSerial;
    /// let saber = PacketSerial::new("/dev/ttyUSB0");
    /// ```
    pub fn new(port: &str) -> Result<PacketSerial<SabertoothPort>> {
        Ok(PacketSerial::from(SabertoothPort::new(port)?))
    }
}

impl<T: SabertoothSerial> PacketSerial<T> {
    /// Set the address of the Sabertooth.
    ///
    /// # Example
    ///
    /// ```
    /// use saberrs::sabertooth2x32::PacketSerial;
    /// # use saberrs::{Result, SabertoothPort};
    /// # fn new_saber() -> Result<PacketSerial<SabertoothPort>> {
    /// let saber = PacketSerial::new("/dev/ttyUSB0")?.with_address(129);
    /// # Ok(saber)
    /// # }
    /// ```
    pub fn with_address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }

    /// Set the integrity protection type used for the frames.
    ///
    /// # Example
    ///
    /// ```
    /// use saberrs::sabertooth2x32::{PacketSerial, PacketType};
    /// # use saberrs::{Result, SabertoothPort};
    /// # fn new_saber() -> Result<PacketSerial<SabertoothPort>> {
    /// let saber = PacketSerial::new("/dev/ttyUSB0")?.with_packet_type(PacketType::CRC);
    /// # Ok(saber)
    /// # }
    /// ```
    pub fn with_packet_type(mut self, packet_type: PacketType) -> Self {
        self.packet_type = packet_type;
        self
    }

    fn write_frame(&mut self, txdata: &[u8]) -> Result<()> {
        dbg_frame!(tx, txdata);
        Ok(self.dev.write_all(txdata)?)
    }

    fn read_frame(&mut self, mut buf: &mut [u8]) -> Result<()> {
        self.dev.read_exact(&mut buf)?;
        dbg_frame!(rx, buf);
        Ok(())
    }

    fn set(&mut self, cmd_value: CommandSet, value: i32, target: [u8; 2]) -> Result<()> {
        let packet =
            PacketFrame::new_set_frame(self.packet_type, self.address, cmd_value, value, target)?;
        self.write_frame(packet.as_ref())
    }

    fn set_ratio(&mut self, ratio: f32, target: [u8; 2]) -> Result<()> {
        let value = utils::ratio_to_value(ratio)?;
        self.set(CommandSet::Value, value, target)
    }

    fn reply_size(&self) -> usize {
        match self.packet_type {
            PacketType::Checksum => checksum::PACKET_REPLY_SIZE,
            PacketType::CRC => crc::PACKET_REPLY_SIZE,
        }
    }

    fn parse_response(
        &self,
        resp: &[u8],
        expected_cmdvalue: CommandGet,
        expected_source: [u8; 2],
    ) -> Result<i32> {
        let error = |s: &str| Err(Error::Response(s.to_string()));

        let resp_cmdnum = resp[1];
        let resp_cmdvalue = resp[2];
        let resp_data_value = &resp[4..6];
        let resp_data_source = &resp[6..8];

        let validity = match self.packet_type {
            PacketType::Checksum => checksum::packet_is_valid(resp, self.address),
            PacketType::CRC => crc::packet_is_valid(resp, self.address),
        };

        match validity {
            Ok(_) => {}
            Err(ParseError::PacketSize) => return error("invalid packet size"),
            Err(ParseError::ChecksumError) => return error("invalid checksum or CRC"),
            Err(ParseError::AddressError) => return error("invalid address"),
        }

        if resp_cmdnum != CMD_NUM_REPLY {
            return error("invalid command num");
        }

        let expected_cmdvalue = expected_cmdvalue as u8;
        let is_negative = match resp_cmdvalue {
            _ if resp_cmdvalue == (expected_cmdvalue + 1) => true,
            _ if resp_cmdvalue == expected_cmdvalue => false,
            _ => return error("invalid command value"),
        };

        let mut data_value = i32::from(unpack_data_value(resp_data_value));
        if is_negative {
            data_value = -data_value
        }

        if resp_data_source != &expected_source[..] {
            return error("invalid source");
        }

        Ok(data_value)
    }

    fn get(&mut self, cmd_value: CommandGet, source: [u8; 2]) -> Result<i32> {
        let packet = PacketFrame::new_get_frame(self.packet_type, self.address, cmd_value, source)?;
        self.dev.clear_all()?;
        self.write_frame(packet.as_ref())?;
        let mut buf = [0u8; PACKET_MAX_REPLY_SIZE];
        let resp = &mut buf[..self.reply_size()];
        self.read_frame(resp)?;
        self.parse_response(resp, cmd_value, source)
    }

    fn get_ratio(&mut self, cmd_value: CommandGet, source: [u8; 2]) -> Result<f32> {
        let value = self.get(cmd_value, source)?;
        let ratio = utils::value_to_ratio(value);
        Ok(ratio)
    }
}

impl<T: SabertoothSerial> From<T> for PacketSerial<T> {
    fn from(dev: T) -> Self {
        PacketSerial {
            dev,
            address: DEFAULT_ADDRESS,
            packet_type: DEFAULT_PACKET_TYPE,
        }
    }
}

impl<T> From<&T> for PacketSerial<T>
where
    T: SabertoothSerial + Clone,
{
    fn from(dev: &T) -> Self {
        PacketSerial {
            dev: dev.clone(),
            address: DEFAULT_ADDRESS,
            packet_type: DEFAULT_PACKET_TYPE,
        }
    }
}

impl<T: SabertoothSerial> Sabertooth2x32 for PacketSerial<T> {
    fn startup(&mut self, channel: usize) -> Result<()> {
        let target = [b'M', match_channel_to!(channel, b'1', b'2')];
        self.set(CommandSet::Shutdown, 0, target)
    }

    fn shutdown(&mut self, channel: usize) -> Result<()> {
        let target = [b'M', match_channel_to!(channel, b'1', b'2')];
        self.set(CommandSet::Shutdown, 1, target)
    }

    fn set_speed(&mut self, channel: usize, ratio: f32) -> Result<()> {
        self.set_ratio(ratio, [b'M', match_channel_to!(channel, b'1', b'2')])
    }

    fn get_speed(&mut self, channel: usize) -> Result<f32> {
        self.get_ratio(
            CommandGet::Value,
            [b'M', match_channel_to!(channel, b'1', b'2')],
        )
    }

    fn set_drive(&mut self, ratio: f32) -> Result<()> {
        self.set_ratio(ratio, [b'M', b'D'])
    }

    fn set_turn(&mut self, ratio: f32) -> Result<()> {
        self.set_ratio(ratio, [b'M', b'T'])
    }

    fn set_power(&mut self, channel: usize, ratio: f32) -> Result<()> {
        self.set_ratio(ratio, [b'P', match_channel_to!(channel, b'1', b'2')])
    }

    fn get_power(&mut self, channel: usize) -> Result<f32> {
        self.get_ratio(
            CommandGet::Value,
            [b'P', match_channel_to!(channel, b'1', b'2')],
        )
    }

    fn set_ramp(&mut self, channel: usize, ratio: f32) -> Result<()> {
        self.set_ratio(ratio, [b'R', match_channel_to!(channel, b'1', b'2')])
    }

    fn set_aux(&mut self, channel: usize, ratio: f32) -> Result<()> {
        self.set_ratio(ratio, [b'Q', match_channel_to!(channel, b'1', b'2')])
    }

    fn get_voltage(&mut self, channel: usize) -> Result<f32> {
        let value = self.get(
            CommandGet::Battery,
            [b'M', match_channel_to!(channel, b'1', b'2')],
        )?;
        Ok(value as f32 / 10.0)
    }

    fn get_current(&mut self, channel: usize) -> Result<f32> {
        let value = self.get(
            CommandGet::Current,
            [b'M', match_channel_to!(channel, b'1', b'2')],
        )?;
        Ok(value as f32)
    }

    fn get_temperature(&mut self, channel: usize) -> Result<f32> {
        let value = self.get(
            CommandGet::Temperature,
            [b'M', match_channel_to!(channel, b'1', b'2')],
        )?;
        Ok(value as f32)
    }
}

#[derive(Clone, PartialEq, Debug)]
enum PacketFrame {
    ChecksumSet(checksum::PacketSet),
    ChecksumGet(checksum::PacketGet),
    CRCSet(crc::PacketSet),
    CRCGet(crc::PacketGet),
}

impl PacketFrame {
    fn new_set_frame(
        packet_type: PacketType,
        address: u8,
        command_value: CommandSet,
        data_value: i32,
        target: [u8; 2],
    ) -> Result<PacketFrame> {
        let frame = match packet_type {
            PacketType::Checksum => PacketFrame::ChecksumSet(checksum::PacketSet::new(
                address,
                command_value,
                data_value,
                target,
            )?),
            PacketType::CRC => PacketFrame::CRCSet(crc::PacketSet::new(
                address,
                command_value,
                data_value,
                target,
            )?),
        };
        Ok(frame)
    }

    pub fn new_get_frame(
        packet_type: PacketType,
        address: u8,
        command_value: CommandGet,
        source: [u8; 2],
    ) -> Result<PacketFrame> {
        let frame = match packet_type {
            PacketType::Checksum => {
                PacketFrame::ChecksumGet(checksum::PacketGet::new(address, command_value, source)?)
            }
            PacketType::CRC => {
                PacketFrame::CRCGet(crc::PacketGet::new(address, command_value, source)?)
            }
        };
        Ok(frame)
    }
}

impl AsRef<[u8]> for PacketFrame {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            PacketFrame::ChecksumSet(ar) => ar.as_ref(),
            PacketFrame::ChecksumGet(ar) => ar.as_ref(),
            PacketFrame::CRCSet(ar) => ar.as_ref(),
            PacketFrame::CRCGet(ar) => ar.as_ref(),
        }
    }
}
