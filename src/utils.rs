use crate::error::Result;

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

macro_rules! ratio_to_value_range {
    ($ratio: expr, $range_min: expr, $range_max: expr) => {
        if $ratio > 1.0 || $ratio < -1.0 {
            return Err(crate::Error::InvalidInput(format!(
                "value ({}) out of range -1.0~1.0",
                $ratio
            )));
        } else {
            let value = ($ratio * $range_max as f32) as i32;

            if value > $range_max {
                Ok($range_max)
            } else if value < $range_min {
                Ok($range_min)
            } else {
                Ok(value)
            }
        }
    };
}

pub fn ratio_to_value(ratio: f32) -> Result<i32> {
    ratio_to_value_range!(ratio, RANGE_MIN, RANGE_MAX)
}

pub fn value_to_ratio(value: i32) -> f32 {
    value as f32 / RANGE_MAX as f32
}
