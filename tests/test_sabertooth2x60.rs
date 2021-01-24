use std::io::Read;
use std::time::Duration;

use saberrs::sabertooth2x60::{ErrorConditions, PacketizedSerial, Sabertooth2x60};
use saberrs::{Result, SabertoothPort};
use serialport::TTYPort;

#[macro_use]
mod utils;

use utils::{Responder, ResponderController, ResponderType};

/// Return a new Sabertooth, and a TTY for talking to it.
pub fn saber2x60_harness(address: u8) -> Result<(PacketizedSerial<SabertoothPort>, TTYPort)> {
    let (saber, tty) = utils::saberdevice_harness();
    let pair = (PacketizedSerial::from_serial(saber, address)?, tty);
    Ok(pair)
}

/// Return a new Sabertooth connected to a responder, and the controller of the responder.
pub fn saber2x60_responder_harness(
    address: u8,
) -> Result<(PacketizedSerial<SabertoothPort>, ResponderController)> {
    let (dev, tty) = utils::saberdevice_harness();
    let saber = PacketizedSerial::from_serial(dev, address)?;
    let responder = Responder::new(Box::new(tty), ResponderType::Checksum).start();
    Ok((saber, responder))
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

    saber
        .set_drive_motor(1, f32::NAN)
        .expect_err("expected handling of NaN");
    saber
        .set_drive_motor(1, f32::INFINITY)
        .expect_err("expected handling of +Inf");
    saber
        .set_drive_motor(1, f32::NEG_INFINITY)
        .expect_err("expected handling of -Inf");
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

    saber
        .set_min_voltage(f32::NAN)
        .expect_err("expected handling of NaN");
    saber
        .set_min_voltage(f32::INFINITY)
        .expect_err("expected handling of +Inf");
    saber
        .set_min_voltage(f32::NEG_INFINITY)
        .expect_err("expected handling of -Inf");
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

    saber
        .set_max_voltage(f32::NAN)
        .expect_err("expected handling of NaN");
    saber
        .set_max_voltage(f32::INFINITY)
        .expect_err("expected handling of +Inf");
    saber
        .set_max_voltage(f32::NEG_INFINITY)
        .expect_err("expected handling of -Inf");
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

    saber
        .set_drive_mixed(f32::NAN)
        .expect_err("expected handling of NaN");
    saber
        .set_drive_mixed(f32::INFINITY)
        .expect_err("expected handling of +Inf");
    saber
        .set_drive_mixed(f32::NEG_INFINITY)
        .expect_err("expected handling of -Inf");
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

    saber
        .set_turn_mixed(f32::NAN)
        .expect_err("expected handling of NaN");
    saber
        .set_turn_mixed(f32::INFINITY)
        .expect_err("expected handling of +Inf");
    saber
        .set_turn_mixed(f32::NEG_INFINITY)
        .expect_err("expected handling of -Inf");
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

#[test]
fn test_set_ramp() {
    let (mut saber, mut tty) = saber2x60_harness(134).unwrap();
    let vectors = [
        // fast
        (Duration::from_millis(250), vec![134, 16, 1, 23]),
        (Duration::from_millis(125), vec![134, 16, 2, 24]),
        (Duration::from_millis(83), vec![134, 16, 3, 25]),
        // intermediate
        (Duration::from_millis(1526), vec![134, 16, 21, 43]),
        (Duration::from_millis(763), vec![134, 16, 32, 54]),
        (Duration::from_millis(373), vec![134, 16, 55, 77]),
        (Duration::from_millis(262), vec![134, 16, 74, 96]), // overlap of lower intermediate with fast
        // slow
        (Duration::from_millis(16787), vec![134, 16, 11, 33]),
        (Duration::from_millis(3357), vec![134, 16, 15, 37]),
        (Duration::from_millis(1679), vec![134, 16, 20, 42]),
    ];
    test_set_method_no_channel!(saber, set_ramp, vectors, tty);

    saber
        .set_ramp(Duration::from_millis(24))
        .expect_err("expected out of range");
    saber
        .set_ramp(Duration::from_secs(17))
        .expect_err("expected out of range");
}

#[test]
fn test_set_deadband() {
    let (mut saber, mut tty) = saber2x60_harness(135).unwrap();
    let vectors = [
        (0.0, vec![135, 17, 0, 24]),
        (0.2, vec![135, 17, 25, 49]),
        (0.5, vec![135, 17, 63, 87]),
        (0.85, vec![135, 17, 107, 3]),
        (1.0, vec![135, 17, 127, 23]),
    ];
    test_set_method_no_channel!(saber, set_deadband, vectors, tty);

    saber
        .set_deadband(-0.1)
        .expect_err("expected out of range error");
    saber
        .set_deadband(1.1)
        .expect_err("expected out of range error");

    saber
        .set_deadband(f32::NAN)
        .expect_err("expected handling of NaN");
    saber
        .set_deadband(f32::INFINITY)
        .expect_err("expected handling of +Inf");
    saber
        .set_deadband(f32::NEG_INFINITY)
        .expect_err("expected handling of -Inf");
}

#[test]
#[rustfmt::skip]
fn test_get_errors() {
    let (mut saber, responder) = saber2x60_responder_harness(128).unwrap();
    let vectors = [
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0000_0000], ErrorConditions(0)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0000_0001], ErrorConditions(1)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0000_0010], ErrorConditions(2)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0000_0100], ErrorConditions(4)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0000_1000], ErrorConditions(8)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0001_0000], ErrorConditions(16)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0010_0000], ErrorConditions(32)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b0100_0000], ErrorConditions(64)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b1000_0000], ErrorConditions(128)),
        (vec![128, 127, 2, 0, 0, 1], vec![0, 0b1010_0110], ErrorConditions(166)),
    ];
    test_get_method_no_channel!(saber, get_errors, vectors, responder);
}

#[test]
fn test_error_conditions() {
    assert!(ErrorConditions(0).is_ok());
    assert!(!ErrorConditions(1).is_ok());

    assert!(ErrorConditions(0x01).overcurrent());
    assert!(ErrorConditions(0x02).overvoltage());
    assert!(ErrorConditions(0x04).overtemperature());
    assert!(ErrorConditions(0x08).undervoltage());
    assert!(ErrorConditions(0x20).deadband_1());
    assert!(ErrorConditions(0x40).deadband_2());
    assert!(ErrorConditions(0x80).timeout());

    assert!(!ErrorConditions(0xA5).is_ok());
    assert!(ErrorConditions(0xA5).overcurrent());
    assert!(!ErrorConditions(0xA5).overvoltage());
    assert!(ErrorConditions(0xA5).overtemperature());
    assert!(!ErrorConditions(0xA5).undervoltage());
    assert!(ErrorConditions(0xA5).deadband_1());
    assert!(!ErrorConditions(0xA5).deadband_2());
    assert!(ErrorConditions(0xA5).timeout());
}

#[test]
#[rustfmt::skip]
fn test_get_temperature() {
    // TODO
    // let (mut saber, responder) = saber2x60_responder_harness(128).unwrap();
    // let vectors = [
    //     (1, vec![128, 127, 2, 0, 1, 2], vec![1, 0], 0.0),
    //     (2, vec![128, 127, 2, 0, 2, 3], vec![2, 0], 0.0),
    // ];
    // test_get_method!(saber, get_temperature, vectors, responder);
}

#[allow(non_snake_case)]
#[test]
fn test_print_all() {
    for value in 0..256 {
        let v = (value as f64) * 5.0 / 255.0;
        let v0 = 5.0;
        let r = 1100.0 * v / (v0 - v);
        let b = 3455.0f64;
        let r0 = 10000.0f64;
        let T0 = 298.0f64;
        let T = b / (r / (r0 * (-b / T0).exp())).ln() - 273.0;
        println!("{} -> {}", value, T);
    }
    //panic!();  // uncomment + cargo test to inspect values
}

#[test]
fn test_get_voltage() {
    let (mut saber, responder) = saber2x60_responder_harness(130).unwrap();
    let vectors = [
        (vec![130, 127, 2, 0, 3, 6], vec![3, 0], 0.0),
        (vec![130, 127, 2, 0, 3, 6], vec![3, 23], 4.51),
        (vec![130, 127, 2, 0, 3, 6], vec![3, 100], 19.608),
        (vec![130, 127, 2, 0, 3, 6], vec![3, 255], 50.0),
    ];
    test_get_method_float_no_channel!(saber, get_voltage, vectors, responder);
}
