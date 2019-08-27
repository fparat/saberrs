use super::*;

pub const PACKET_SET_SIZE: usize = 9;
pub const PACKET_GET_SIZE: usize = 7;
pub const PACKET_REPLY_SIZE: usize = 9;

fn checksum(data: &[u8]) -> u8 {
    let s: u32 = data.iter().map(|&b| u32::from(b)).sum();
    (s & 0x7f) as u8
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
        buf[0] = address;
        buf[1] = CMD_NUM_SET;
        buf[2] = command_value as u8;
        buf[3] = checksum(&buf[..3]);
        buf[4..6].copy_from_slice(&pack_data_value(data_value as u16));
        buf[6..8].copy_from_slice(&target[..2]);
        buf[8] = checksum(&buf[4..8]);

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
        buf[0] = address;
        buf[1] = CMD_NUM_GET;
        buf[2] = command_value as u8;
        buf[3] = checksum(&buf[..3]);
        buf[4..6].copy_from_slice(&source[..2]);
        buf[6] = checksum(&buf[4..6]);

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
    } else if resp[3] != checksum(&resp[..3]) || resp[8] != checksum(&resp[4..8]) {
        Err(ParseError::ChecksumError)
    } else if resp[0] != address {
        Err(ParseError::AddressError)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(0x15, checksum(b"\x80\x81\x04\x07\x09"));
    }
}
