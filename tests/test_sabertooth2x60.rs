use saberrs::sabertooth2x60::PacketizedSerial;
use saberrs::{Result, SabertoothPort};
use serialport::TTYPort;

mod utils;

/// Return a new SabertoothText, and a TTY for talking to it.
pub fn saber2x60_harness(address: u8) -> Result<(PacketizedSerial<SabertoothPort>, TTYPort)> {
    let (saber, tty) = utils::saberdevice_harness();
    let pair = (PacketizedSerial::from_serial(saber, address)?, tty);
    Ok(pair)
}

#[test]
fn test_instantiate() {
    for a in 0..128 {
        saber2x60_harness(a).expect_err(&format!("{} is an invalid address", a));
    }
    for a in 128..=135 {
        let (_saber, _tty) = saber2x60_harness(a).expect(&format!("address {} failed", a));
    }
    for a in 136..255 {
        saber2x60_harness(a).expect_err(&format!("{} is an invalid address", a));
    }
}
