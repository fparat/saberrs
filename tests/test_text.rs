use std::io::Read;

use serialport::SerialPort;

use saberrs::sabertooth2x32::Sabertooth2x32;

#[macro_use]
mod utils;

#[test]
fn startup() {
    let (mut sabertext, mut tty) = utils::sabertext_harness();

    sabertext.startup(1).expect("Startup failure");
    let mut buf = [0u8; 32];
    let read_len = tty.read(&mut buf).expect("Read fail");
    let expected = b"M1: startup\r\n";
    assert_eq!(expected.len(), read_len);
    assert_eq!(expected, &buf[0..expected.len()]);

    sabertext.startup(0).expect_err("Channel 0 should fail");
}

#[test]
fn shutdown() {
    let (mut sabertext, mut tty) = utils::sabertext_harness();

    sabertext.shutdown(2).expect("Startup failure");
    let mut buf = [0u8; 32];
    let read_len = tty.read(&mut buf).expect("Read fail");
    let expected = b"M2: shutdown\r\n";
    assert_eq!(expected.len(), read_len);
    assert_eq!(expected, &buf[0..expected.len()]);
}

#[test]
fn set_speed() {
    let vectors = [
        (1, -1.0, b"M1: -2047\r\n".to_vec()),
        (2, -0.5, b"M2: -1023\r\n".to_vec()),
        (2, -0.2, b"M2: -409\r\n".to_vec()),
        (1, 0.0, b"M1: 0\r\n".to_vec()),
        (1, 0.25, b"M1: 511\r\n".to_vec()),
        (2, 0.5, b"M2: 1023\r\n".to_vec()),
        (1, 0.75, b"M1: 1535\r\n".to_vec()),
        (2, 1.0, b"M2: 2047\r\n".to_vec()),
    ];

    let (mut sabertext, mut tty) = utils::sabertext_harness();
    test_set_method!(sabertext, set_speed, vectors, tty);
}

#[test]
#[rustfmt::skip]
fn set_speed_errs() {
    let (mut sabertext, tty) = utils::sabertext_harness();
    sabertext.set_speed(0, 0.0).expect_err("Channel <1 should fail");
    sabertext.set_speed(3, 0.0).expect_err("Channel >2 should fail");
    sabertext.set_speed(1, 1.0001).expect_err("Values >1.000 should fail");
    sabertext.set_speed(1, -1.0001).expect_err("Values <-1.000 should fail");

    // nothing should have been sent over serial
    assert_eq!(0, tty.bytes_to_read().unwrap());
}

#[test]
fn set_drive() {
    let vectors = [
        (-0.5, b"MD: -1023\r\n".to_vec()),
        (1.000, b"MD: 2047\r\n".to_vec()),
    ];

    let (mut sabertext, mut tty) = utils::sabertext_harness();
    test_set_method_no_channel!(sabertext, set_drive, vectors, tty);
}

#[test]
fn set_turn() {
    let vectors = [
        (-1.000, b"MT: -2047\r\n".to_vec()),
        (0.25, b"MT: 511\r\n".to_vec()),
    ];
    let (mut sabertext, mut tty) = utils::sabertext_harness();
    test_set_method_no_channel!(sabertext, set_turn, vectors, tty);
}

#[test]
fn set_power() {
    let vectors = [
        (1, -1.000, b"P1: -2047\r\n".to_vec()),
        (2, -0.5, b"P2: -1023\r\n".to_vec()),
        (1, 0.0, b"P1: 0\r\n".to_vec()),
        (1, 0.25, b"P1: 511\r\n".to_vec()),
        (2, 0.5, b"P2: 1023\r\n".to_vec()),
        (1, 0.75, b"P1: 1535\r\n".to_vec()),
        (2, 1.000, b"P2: 2047\r\n".to_vec()),
    ];

    let (mut sabertext, mut tty) = utils::sabertext_harness();
    test_set_method!(sabertext, set_power, vectors, tty);
}

#[test]
fn set_ramp() {
    let vectors = [
        (1, -1.000, b"R1: -2047\r\n".to_vec()),
        (2, -0.5, b"R2: -1023\r\n".to_vec()),
        (1, 0.0, b"R1: 0\r\n".to_vec()),
        (1, 0.25, b"R1: 511\r\n".to_vec()),
        (2, 0.5, b"R2: 1023\r\n".to_vec()),
        (1, 0.75, b"R1: 1535\r\n".to_vec()),
        (2, 1.000, b"R2: 2047\r\n".to_vec()),
    ];

    let (mut sabertext, mut tty) = utils::sabertext_harness();
    test_set_method!(sabertext, set_ramp, vectors, tty);
}

#[test]
fn set_aux() {
    let vectors = [
        (1, -1.000, b"Q1: -2047\r\n".to_vec()),
        (2, -0.5, b"Q2: -1023\r\n".to_vec()),
        (1, 0.0, b"Q1: 0\r\n".to_vec()),
        (1, 0.25, b"Q1: 511\r\n".to_vec()),
        (2, 0.5, b"Q2: 1023\r\n".to_vec()),
        (1, 0.75, b"Q1: 1535\r\n".to_vec()),
        (2, 1.000, b"Q2: 2047\r\n".to_vec()),
    ];

    let (mut sabertext, mut tty) = utils::sabertext_harness();
    test_set_method!(sabertext, set_aux, vectors, tty);
}

#[test]
fn get_speed() {
    #[rustfmt::skip]
    let vectors = [
        (1, b"M1: get\r\n".to_vec(), b"M1: 1256\r\n".to_vec(), 0.61358),
        (2, b"M2: get\r\n".to_vec(), b"M2: -2047\r\n".to_vec(), -1.000),
    ];

    let (mut sabertext, responder) = utils::sabertext_responder_harness();
    test_get_method!(sabertext, get_speed, vectors, responder);
    responder.stop();
}

#[test]
fn get_power() {
    #[rustfmt::skip]
    let vectors = [
        (1, b"P1: get\r\n".to_vec(), b"P1: -1000\r\n".to_vec(), -0.48852),
        (2, b"P2: get\r\n".to_vec(), b"P2: 2047\r\n".to_vec(), 1.000),
    ];

    let (mut sabertext, responder) = utils::sabertext_responder_harness();
    test_get_method!(sabertext, get_power, vectors, responder);
    responder.stop();
}

#[test]
fn get_voltage() {
    #[rustfmt::skip]
    let vectors = [
        (1, b"M1: getb\r\n".to_vec(), b"M1: B125\r\n".to_vec(), 12.5),
        (2, b"M2: getb\r\n".to_vec(), b"M2:B240\r\n".to_vec(), 24.0),
    ];

    let (mut sabertext, responder) = utils::sabertext_responder_harness();
    test_get_method!(sabertext, get_voltage, vectors, responder);
    responder.stop();
}

#[test]
fn get_current() {
    #[rustfmt::skip]
    let vectors = [
        (1, b"M1: getc\r\n".to_vec(), b"M1: C320\r\n".to_vec(), 32.0),
        (2, b"M2: getc\r\n".to_vec(), b"M2:C-20\r\n".to_vec(), -2.0),
    ];

    let (mut sabertext, responder) = utils::sabertext_responder_harness();
    test_get_method!(sabertext, get_current, vectors, responder);
    responder.stop();
}

#[test]
fn get_temperature() {
    #[rustfmt::skip]
    let vectors = [
        (1, b"M1: gett\r\n".to_vec(), b"M1: T30\r\n".to_vec(), 30.0),
        (2, b"M2: gett\r\n".to_vec(), b"M2:T85\r\n".to_vec(), 85.0),
    ];

    let (mut sabertext, responder) = utils::sabertext_responder_harness();
    test_get_method!(sabertext, get_temperature, vectors, responder);
    responder.stop();
}

#[cfg(feature = "serialport")]
#[test]
fn test_from_serialport() {
    use saberrs::sabertooth2x32::PlainText;
    let (saber, _) = utils::saberdevice_harness();
    let _ = PlainText::from(saber);
}
