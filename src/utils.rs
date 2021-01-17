use crate::error::{Error, Result};

pub const RANGE_MAX: i32 = 2047;
pub const RANGE_MIN: i32 = -2047;

macro_rules! match_channel_to {
    ($channel:expr, $ch1:expr, $ch2:expr) => {
        match $channel {
            1 => $ch1,
            2 => $ch2,
            _ => {
                let msg = format!("channel should be 1 or 2 (was {})", $channel);
                return Err(crate::error::Error::InvalidInput(msg));
            }
        }
    };
}

pub fn ratio_to_value(ratio: f32) -> Result<i32> {
    if ratio > 1.0 || ratio < -1.0 {
        return Err(Error::InvalidInput(format!(
            "value ({}) out of range -1.0~1.0",
            ratio
        )));
    }

    let value = (ratio * RANGE_MAX as f32) as i32;

    if value > RANGE_MAX {
        Ok(RANGE_MAX)
    } else if value < RANGE_MIN {
        Ok(RANGE_MIN)
    } else {
        Ok(value)
    }
}

pub fn value_to_ratio(value: i32) -> f32 {
    value as f32 / RANGE_MAX as f32
}
