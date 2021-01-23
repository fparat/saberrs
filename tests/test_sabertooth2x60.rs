use std::io::Read;

use saberrs::sabertooth2x60::{PacketizedSerial, Sabertooth2x60};
use saberrs::{Result, SabertoothPort};
use serialport::TTYPort;

#[macro_use]
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

#[test]
fn test_example_in_doc() {
    // Manual page 23 "Example of Packetized Serial"
    let (mut saber, mut tty) = saber2x60_harness(130).unwrap();
    let vectors = [(1, 0.51, vec![130u8, 0, 64, 0b01000010])];
    test_set_method!(saber, set_drive_motor, vectors, tty);
}

#[test]
fn test_set_drive_motor() {
    let (mut saber, mut tty) = saber2x60_harness(128).unwrap();
    let vectors = [
        (1, 1.0, vec![128, 0, 127, 127]),
        (1, -1.0, vec![128, 1, 127, 0]),
        (1, 0.0, vec![128, 0, 0, 0]),
        (1, -0.25, vec![128, 1, 31, 32]),
        (2, 1.0, vec![128, 4, 127, 3]),
        (2, -1.0, vec![128, 5, 127, 4]),
        (2, 0.0, vec![128, 4, 0, 4]),
        (2, -0.25, vec![128, 5, 31, 36]),
    ];
    test_set_method!(saber, set_drive_motor, vectors, tty);
}
