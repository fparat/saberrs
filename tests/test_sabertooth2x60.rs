use std::io::Read;
use std::time::Duration;

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

    saber
        .set_drive_motor(0, 0.5)
        .expect_err("expected invalid motor error");
    saber
        .set_drive_motor(3, 0.5)
        .expect_err("expected invalid motor error");
    saber
        .set_drive_motor(1, -1.1)
        .expect_err("expected out of range error");
    saber
        .set_drive_motor(1, 1.1)
        .expect_err("expected out of range error");
}

#[test]
fn test_set_min_voltage() {
    let (mut saber, mut tty) = saber2x60_harness(129).unwrap();
    let vectors = [
        (6., vec![129, 2, 0, 3]),
        (10., vec![129, 2, 20, 23]),
        (25., vec![129, 2, 95, 98]),
        (30., vec![129, 2, 120, 123]),
    ];
    test_set_method_no_channel!(saber, set_min_voltage, vectors, tty);

    saber
        .set_min_voltage(-0.1)
        .expect_err("expected out of range error");
    saber
        .set_min_voltage(120.1)
        .expect_err("expected out of range error");
}

#[test]
fn test_set_max_voltage() {
    let (mut saber, mut tty) = saber2x60_harness(130).unwrap();
    let vectors = [
        (0., vec![130, 3, 0, 5]),
        (10., vec![130, 3, 51, 56]),
        (17., vec![130, 3, 87, 92]),
        (25., vec![130, 3, 128, 5]),
    ];
    test_set_method_no_channel!(saber, set_max_voltage, vectors, tty);

    saber
        .set_max_voltage(-0.1)
        .expect_err("expected out of range error");
    saber
        .set_max_voltage(25.1)
        .expect_err("expected out of range error");
}

#[test]
fn test_set_drive_mixed() {
    let (mut saber, mut tty) = saber2x60_harness(131).unwrap();
    let vectors = [
        (0., vec![131, 8, 0, 11]),
        (1., vec![131, 8, 127, 10]),
        (-1., vec![131, 9, 127, 11]),
        (0.5, vec![131, 8, 63, 74]),
        (-0.3, vec![131, 9, 38, 50]),
    ];
    test_set_method_no_channel!(saber, set_drive_mixed, vectors, tty);

    saber
        .set_drive_mixed(1.1)
        .expect_err("expected out of range error");
    saber
        .set_drive_mixed(-1.1)
        .expect_err("expected out of range error");
}

#[test]
fn test_set_turn_mixed() {
    let (mut saber, mut tty) = saber2x60_harness(131).unwrap();
    let vectors = [
        (0., vec![131, 10, 0, 13]),
        (1., vec![131, 10, 127, 12]),
        (-1., vec![131, 11, 127, 13]),
        (0.5, vec![131, 10, 63, 76]),
        (-0.3, vec![131, 11, 38, 52]),
    ];
    test_set_method_no_channel!(saber, set_turn_mixed, vectors, tty);

    saber
        .set_turn_mixed(1.1)
        .expect_err("expected out of range error");
    saber
        .set_turn_mixed(-1.1)
        .expect_err("expected out of range error");
}

#[test]
fn test_set_serial_timeout() {
    let (mut saber, mut tty) = saber2x60_harness(132).unwrap();
    let vectors = [
        (Duration::from_millis(100), vec![132, 14, 1, 19]),
        (Duration::from_millis(12700), vec![132, 14, 127, 17]),
        (Duration::from_millis(0), vec![132, 14, 0, 18]),
        (Duration::from_millis(2000), vec![132, 14, 20, 38]),
        (Duration::from_millis(1), vec![132, 14, 1, 19]), // check ceil rounding
    ];
    test_set_method_no_channel!(saber, set_serial_timeout, vectors, tty);

    saber
        .set_serial_timeout(Duration::from_millis(12701))
        .expect_err("expected out of range error");
}

#[test]
fn test_set_baudrate() {
    use saberrs::sabertooth2x60::Baudrate;
    let (mut saber, mut tty) = saber2x60_harness(133).unwrap();
    let vectors = [
        (Baudrate::B2400, vec![133, 15, 1, 21]),
        (Baudrate::B9600, vec![133, 15, 2, 22]),
        (Baudrate::B19200, vec![133, 15, 3, 23]),
        (Baudrate::B38400, vec![133, 15, 4, 24]),
        (Baudrate::B115200, vec![133, 15, 5, 25]),
    ];
    test_set_method_no_channel!(saber, set_baudrate, vectors, tty);
}
