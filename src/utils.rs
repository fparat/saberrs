use crate::error::{Error, ErrorKind, Result};

pub const RANGE_MAX: i32 = 2047;
pub const RANGE_MIN: i32 = -2047;

macro_rules! match_channel_to {
    ($channel:expr, $ch1:expr, $ch2:expr) => {
        match $channel {
            1 => $ch1,
            2 => $ch2,
            _ => {
                let msg = format!("Channel should be 1 or 2 (was {})", $channel);
                return Err(Error::new(ErrorKind::InvalidInput, msg));
            }
        }
    };
}

pub fn percent_to_value(percent: f32) -> Result<i32> {
    if percent > 100.0 || percent < -100.0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Percentage value ({}) out of range -100.0~100.0", percent),
        ));
    }

    let value = (percent * RANGE_MAX as f32 / 100.0) as i32;

    if value > RANGE_MAX {
        Ok(RANGE_MAX)
    } else if value < RANGE_MIN {
        Ok(RANGE_MIN)
    } else {
        Ok(value)
    }
}

pub fn value_to_percent(value: i32) -> f32 {
    value as f32 / RANGE_MAX as f32 * 100.0
}
