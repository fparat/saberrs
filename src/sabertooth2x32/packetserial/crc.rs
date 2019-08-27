use super::*;

pub const PACKET_SET_SIZE: usize = 10;
pub const PACKET_GET_SIZE: usize = 8;
pub const PACKET_REPLY_SIZE: usize = 10;
pub const PACKET_ADDR_OFFSET: u8 = 112;

fn crc7(data: &[u8]) -> u8 {
    let mut crc = 0x7fu8;

    for &b in data {
        crc ^= b;

        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc >>= 1;
                crc ^= 0x76;
            } else {
                crc >>= 1;
            }
        }
    }

    crc ^ 0x7fu8
}

fn crc14(data: &[u8]) -> u16 {
    let mut crc = 0x3fffu16;

    for &b in data {
        crc ^= u16::from(b);

        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc >>= 1;
                crc ^= 0x22f0;
            } else {
                crc >>= 1;
            }
        }
    }

    crc ^ 0x3fff
}

fn crc14_to_buf(data: &[u8]) -> [u8; 2] {
    let crc = crc14(data);
    [(crc & 127) as u8, ((crc >> 7) & 127) as u8]
}

#[derive(Clone, PartialEq, Debug)]
pub struct PacketSet([u8; PACKET_SET_SIZE]);

impl PacketSet {
    pub fn new(
        address: u8,
        command_value: CommandSet,
        data_value: i32,
        target: [u8; 2],
    ) -> Result<PacketSet> {
        let mut command_value = command_value as u8;
        let mut data_value = data_value;

        if data_value < 0 {
            data_value = -data_value;
            command_value += 1
        }

        let mut buf = [0u8; PACKET_SET_SIZE];
        buf[0] = address + PACKET_ADDR_OFFSET;
        buf[1] = CMD_NUM_SET;
        buf[2] = command_value as u8;
        buf[3] = crc7(&buf[..3]);
        buf[4..6].copy_from_slice(&pack_data_value(data_value as u16));
        buf[6..8].copy_from_slice(&target[..2]);
        let crcdata = crc14_to_buf(&buf[4..8]);
        buf[8] = crcdata[0];
        buf[9] = crcdata[1];

        Ok(PacketSet(buf))
    }
}

impl AsRef<[u8]> for PacketSet {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PacketGet([u8; PACKET_GET_SIZE]);

impl PacketGet {
    pub fn new(address: u8, command_value: CommandGet, source: [u8; 2]) -> Result<PacketGet> {
        let mut buf = [0u8; PACKET_GET_SIZE];
        buf[0] = address + PACKET_ADDR_OFFSET;
        buf[1] = CMD_NUM_GET;
        buf[2] = command_value as u8;
        buf[3] = crc7(&buf[..3]);
        buf[4..6].copy_from_slice(&source[..2]);
        let crcdata = crc14_to_buf(&buf[4..6]);
        buf[6] = crcdata[0];
        buf[7] = crcdata[1];

        Ok(PacketGet(buf))
    }
}

impl AsRef<[u8]> for PacketGet {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub fn packet_is_valid(resp: &[u8], address: u8) -> std::result::Result<(), ParseError> {
    if resp.len() != PACKET_REPLY_SIZE {
        Err(ParseError::PacketSize)
    } else if resp[3] != crc7(&resp[..3]) || resp[8..10] != crc14_to_buf(&resp[4..8]) {
        Err(ParseError::ChecksumError)
    } else if resp[0] != address + PACKET_ADDR_OFFSET {
        Err(ParseError::AddressError)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc7() {
        assert_eq!(0x12, crc7(&[0]));
        assert_eq!(0x09, crc7(&[255]));
        assert_eq!(0x40, crc7(&[14]));
        assert_eq!(0x7f, crc7(&[127]));
        assert_eq!(0x64, crc7(&[128]));
        assert_eq!(0x7C, crc7(&[203]));
    }

    #[test]
    fn test_crc14() {
        assert_eq!(0x3bb7, crc14(&[0, 255]));
        assert_eq!(0x1aa7, crc14(&[255, 0]));
        assert_eq!(0x2080, crc14(&[14, 127]));
        assert_eq!(0x20ee, crc14(&[203, 128]));
    }

    #[test]
    fn test_crc_packet() {
        assert_eq!(
            &[240, 41, 0, 109, 77, 49, 6, 36],
            PacketFrame::new_get_frame(PacketType::CRC, 128, CommandGet::Value, [77, 49])
                .unwrap()
                .as_ref()
        );
    }
}
