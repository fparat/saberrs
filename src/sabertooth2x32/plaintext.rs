use std::str;

use super::Sabertooth2x32;
use crate::error::{Error, ErrorKind, Result};
use crate::port::SabertoothSerial;
use crate::utils;
use std::convert::From;

#[cfg(feature = "serialport")]
use crate::port::sabertoothport::{SabertoothPort, SabertoothPortShared};

macro_rules! make_cmd_str {
    ($token:expr, $channel:expr, $value:expr) => {
        format!("{}{}: {}\r\n", $token, $channel, $value)
    };
}

#[cfg(debug_assertions)]
macro_rules! dbg_frame {
    ($head:ident, $frame:expr) => {
        let $head = std::str::from_utf8($frame)
            .unwrap_or("<decode error>")
            .trim_matches(char::from(0));
        dbg!($head);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! dbg_frame {
    ($head:ident, $frame:expr) => {};
}

/// Interface using "Plain Text" protocol.
pub struct PlainText<T: SabertoothSerial> {
    dev: T,
}

#[cfg(feature = "serialport")]
impl PlainText<SabertoothPort> {
    /// Create a default new "Plain Text" interface.
    pub fn new(port: &str) -> Result<PlainText<SabertoothPort>> {
        Ok(PlainText {
            dev: SabertoothPort::new(port)?,
        })
    }
}

impl<T: SabertoothSerial> PlainText<T> {
    fn write_frame(&mut self, txdata: &[u8]) -> Result<()> {
        dbg_frame!(tx, txdata);
        Ok(self.dev.write_all(txdata)?)
    }

    fn read_response(&mut self, rxdata: &mut [u8]) -> Result<usize> {
        const ENDFLAG: u8 = b'\n';

        let mut bytebuf = [0u8; 1];
        let mut count: usize = 0;

        for byte in &mut rxdata[..] {
            if self.dev.read(&mut bytebuf)? == 1 {
                *byte = bytebuf[0];
                count += 1;
                if bytebuf[0] == ENDFLAG {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(count)
    }

    fn send_percent_to_channel(&mut self, token: char, channel: usize, percent: f32) -> Result<()> {
        let channel = match_channel_to!(channel, '1', '2');
        self.send_percent(token, channel, percent)
    }

    fn send_percent(&mut self, token: char, channel: char, percent: f32) -> Result<()> {
        let value = utils::percent_to_value(percent)?;
        let cmdstr = make_cmd_str!(token, channel, value);
        let buf = cmdstr.as_bytes();
        self.write_frame(buf)
    }

    fn request(&mut self, txdata: &[u8], rxdata: &mut [u8]) -> Result<usize> {
        self.dev.clear_all()?;
        self.write_frame(txdata)?;
        let read_len = self.read_response(rxdata)?;
        dbg_frame!(rx, rxdata);
        Ok(read_len)
    }

    fn get_value(&mut self, token: char, ch: char, prefix: Option<char>, req: &str) -> Result<i32> {
        let cmdstr = make_cmd_str!(token, ch, req);
        let mut rxbuf = [0u8; 32];
        self.request(cmdstr.as_bytes(), &mut rxbuf)?;
        let splitted = split_response(&rxbuf)?;
        if splitted.0 != token || splitted.1 != ch || splitted.2 != prefix {
            return Err(Error::new(ErrorKind::Response, "Invalid response"));
        }
        Ok(splitted.3)
    }
}

// should work with SabertoothPort
impl<T: SabertoothSerial> From<T> for PlainText<T> {
    fn from(dev: T) -> Self {
        PlainText { dev }
    }
}

// should work with SabertoothPortShared
impl<T> From<&T> for PlainText<T>
where
    T: SabertoothSerial + Clone,
{
    fn from(dev: &T) -> Self {
        PlainText {
            dev: (*dev).clone(),
        }
    }
}

impl<T: SabertoothSerial> Sabertooth2x32 for PlainText<T> {
    fn startup(&mut self, channel: usize) -> Result<()> {
        let ch = match_channel_to!(channel, '1', '2');
        let cmdstr = make_cmd_str!('M', ch, "startup");
        self.write_frame(cmdstr.as_bytes())
    }

    fn shutdown(&mut self, channel: usize) -> Result<()> {
        let ch = match_channel_to!(channel, '1', '2');
        let cmdstr = make_cmd_str!('M', ch, "shutdown");
        self.dev.write_all(cmdstr.as_bytes())?;
        Ok(())
    }

    fn set_speed(&mut self, channel: usize, percent: f32) -> Result<()> {
        self.send_percent_to_channel('M', channel, percent)
    }

    fn get_speed(&mut self, channel: usize) -> Result<f32> {
        let ch = match_channel_to!(channel, '1', '2');
        let value = self.get_value('M', ch, None, "get")?;
        Ok(utils::value_to_percent(value))
    }

    fn set_drive(&mut self, percent: f32) -> Result<()> {
        self.send_percent('M', 'D', percent)
    }

    fn set_turn(&mut self, percent: f32) -> Result<()> {
        self.send_percent('M', 'T', percent)
    }

    fn set_power(&mut self, channel: usize, percent: f32) -> Result<()> {
        self.send_percent_to_channel('P', channel, percent)
    }

    fn get_power(&mut self, channel: usize) -> Result<f32> {
        let ch = match_channel_to!(channel, '1', '2');
        let value = self.get_value('P', ch, None, "get")?;
        Ok(utils::value_to_percent(value))
    }

    fn set_ramp(&mut self, channel: usize, percent: f32) -> Result<()> {
        self.send_percent_to_channel('R', channel, percent)
    }

    fn set_aux(&mut self, channel: usize, percent: f32) -> Result<()> {
        self.send_percent_to_channel('Q', channel, percent)
    }

    fn get_voltage(&mut self, channel: usize) -> Result<f32> {
        let ch = match_channel_to!(channel, '1', '2');
        let value = self.get_value('M', ch, Some('B'), "getb")?;
        Ok(value as f32 * 0.1f32)
    }

    fn get_current(&mut self, channel: usize) -> Result<f32> {
        let ch = match_channel_to!(channel, '1', '2');
        let value = self.get_value('M', ch, Some('C'), "getc")?;
        Ok(value as f32 * 0.1f32)
    }

    fn get_temperature(&mut self, channel: usize) -> Result<f32> {
        let ch = match_channel_to!(channel, '1', '2');
        let value = self.get_value('M', ch, Some('T'), "gett")?;
        Ok(value as f32)
    }
}

/// (token, channel, Options<prefix>, value)
/// ex.: response: b"M1: C-23" -> ('M', '1', Some('C'), -23)
#[derive(PartialEq, Debug)]
struct SplitResponse(char, char, Option<char>, i32);

/// Split a response into its components.
fn split_response(rxdata: &[u8]) -> Result<SplitResponse> {
    // Get the a &str. ASCII is expected
    let resp = match str::from_utf8(rxdata) {
        Ok(r) => r,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Response,
                "Invalid response, not readable",
            ))
        }
    };

    // Prepare the error to return in case of failure. It is a closure so that
    // we can provide it to several ok_or_else().
    let error = || Error::new(ErrorKind::Response, "Parse failure");

    // Trim and create the iterator over the characters.
    let mut resp_iter = resp.trim_matches(char::from(0)).trim().chars();

    // Get the first to characters: token and channel.
    let token = resp_iter.next().ok_or_else(error)?;
    let channel = resp_iter.next().ok_or_else(error)?;

    // Skip until ':', which we check it exists
    let mut resp_iter = resp_iter.skip_while(|c| *c != ':');
    let _ = resp_iter.next().ok_or_else(error)?;

    // Skip until we reach the value. Enable peek because we don't know if there
    // is a prefix.
    let mut resp_iter = resp_iter
        .skip_while(|c| !c.is_ascii_alphanumeric() && *c != '-')
        .peekable();

    // Get the prefix.
    let prefix = if resp_iter.peek().ok_or_else(error)?.is_ascii_alphabetic() {
        resp_iter.next()
    } else {
        None
    };

    // Get the value.
    let value: i32 = resp_iter
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect::<String>()
        .parse::<i32>()
        .ok()
        .ok_or_else(error)?;

    Ok(SplitResponse(token, channel, prefix, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_response() {
        assert_eq!(
            split_response(b"M1: B123\r\n\0\0").ok(),
            Some(SplitResponse('M', '1', Some('B'), 123))
        );
        assert_eq!(
            split_response(b"M2:T99\r\n\0\0").ok(),
            Some(SplitResponse('M', '2', Some('T'), 99))
        );
        assert_eq!(
            split_response(b"M1: C-34\r\n\0\0").ok(),
            Some(SplitResponse('M', '1', Some('C'), -34))
        );
        assert_eq!(
            split_response(b"\0P1: 213").ok(),
            Some(SplitResponse('P', '1', None, 213))
        );
        assert_eq!(
            split_response(b"S2: -52\r\n\0\0").ok(),
            Some(SplitResponse('S', '2', None, -52))
        );
    }
}
