use std::io::Read;

use serialport::SerialPort;

use saberrs::sabertooth2x32::Sabertooth2x32;

#[macro_use]
mod utils;

mod checksum {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn startup() {
        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        let mut buf = [0u8; 32];

        saberchecksum.startup(1).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\x80\x28\x20\x48\x00\x00\x4d\x31\x7e";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        saberchecksum.startup(2).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\x80\x28\x20\x48\x00\x00\x4d\x32\x7f";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        saberchecksum.startup(0).expect_err("Channel 0 should fail");
        saberchecksum.startup(3).expect_err("Channel 3 should fail");
    }

    #[test]
    #[rustfmt::skip]
    fn shutdown() {
        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        let mut buf = [0u8; 32];

        saberchecksum.shutdown(1).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\x80\x28\x20\x48\x01\x00\x4d\x31\x7f";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        saberchecksum.shutdown(2).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\x80\x28\x20\x48\x01\x00\x4d\x32\x00";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        saberchecksum
            .shutdown(0)
            .expect_err("Channel 0 should fail");
        saberchecksum
            .shutdown(3)
            .expect_err("Channel 3 should fail");
    }

    #[test]
    #[rustfmt::skip]
    fn set_speed() {
        let vectors = [
            (1, -1.0, b"\x80\x28\x01\x29\x7f\x0f\x4d\x31\x0c".to_vec()),
            (2, -0.5, b"\x80\x28\x01\x29\x7f\x07\x4d\x32\x05".to_vec()),
            (1, 0.0, b"\x80\x28\x00\x28\x00\x00\x4d\x31\x7e".to_vec()),
            (1, 0.25, b"\x80\x28\x00\x28\x7f\x03\x4d\x31\x00".to_vec()),
            (2, 0.5, b"\x80\x28\x00\x28\x7f\x07\x4d\x32\x05".to_vec()),
            (1, 0.75, b"\x80\x28\x00\x28\x7f\x0b\x4d\x31\x08".to_vec()),
            (2, 1.0, b"\x80\x28\x00\x28\x7f\x0f\x4d\x32\x0d".to_vec()),
        ];

        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        test_set_method!(saberchecksum, set_speed, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_speed_errs() {
        let (mut saberchecksum, tty) = utils::saberchecksum_harness();
        saberchecksum.set_speed(0, 0.0).expect_err("Channel <1 should fail");
        saberchecksum.set_speed(3, 0.0).expect_err("Channel >2 should fail");
        saberchecksum.set_speed(1, 1.01).expect_err("Values >100.0 should fail");
        saberchecksum.set_speed(1, -1.01).expect_err("Values <-100.0 should fail");

        // nothing should have been sent over serial
        assert_eq!(0, tty.bytes_to_read().unwrap());
    }

    #[test]
    #[rustfmt::skip]
    fn set_drive() {
        let vectors = [
            (-0.5, b"\x80\x28\x01\x29\x7f\x07\x4d\x44\x17".to_vec()),
            (1.0, b"\x80\x28\x00\x28\x7f\x0f\x4d\x44\x1f".to_vec()),
        ];

        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        test_set_method_no_channel!(saberchecksum, set_drive, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_turn() {
        let vectors = [
            (-1.0, b"\x80\x28\x01\x29\x7f\x0f\x4d\x54\x2f".to_vec()),
            (0.25, b"\x80\x28\x00\x28\x7f\x03\x4d\x54\x23".to_vec()),
        ];
        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        test_set_method_no_channel!(saberchecksum, set_turn, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_power() {
        let vectors = [(1, -1.0, b"\x80\x28\x01\x29\x7f\x0f\x50\x31\x0f".to_vec())];

        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        test_set_method!(saberchecksum, set_power, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_ramp() {
        let vectors = [(1, 0.25, b"\x80\x28\x00\x28\x7f\x03\x52\x31\x05".to_vec())];

        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        test_set_method!(saberchecksum, set_ramp, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_aux() {
        let vectors = [(2, 0.5, b"\x80\x28\x00\x28\x7f\x07\x51\x32\x09".to_vec())];

        let (mut saberchecksum, mut tty) = utils::saberchecksum_harness();
        test_set_method!(saberchecksum, set_aux, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn get_speed() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\x80\x29\x00\x29\x4D\x31\x7E".to_vec(), b"\x80\x49\x00\x49\x7F\x03\x4D\x31\x00".to_vec(), 0.24963),
            (2, b"\x80\x29\x00\x29\x4D\x32\x7F".to_vec(), b"\x80\x49\x01\x4A\x2E\x08\x4D\x32\x35".to_vec(), -0.522_716),
        ];

        let (mut saberchecksum, responder) = utils::saberchecksum_responder_harness();
        test_get_method!(saberchecksum, get_speed, vectors, responder);
        responder.stop();
    }

    #[test]
    #[rustfmt::skip]
    fn get_voltage() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\x80\x29\x10\x39\x4D\x31\x7E".to_vec(), b"\x80\x49\x10\x59\x78\x00\x4D\x31\x76".to_vec(), 12.0),
            (2, b"\x80\x29\x10\x39\x4D\x32\x7F".to_vec(), b"\x80\x49\x10\x59\x78\x00\x4D\x32\x77".to_vec(), 12.0),
        ];

        let (mut saberchecksum, responder) = utils::saberchecksum_responder_harness();
        test_get_method!(saberchecksum, get_voltage, vectors, responder);
        responder.stop();
    }

    #[test]
    #[rustfmt::skip]
    fn get_current() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\x80\x29\x20\x49\x4D\x31\x7E".to_vec(), b"\x80\x49\x20\x69\x0B\x00\x4D\x31\x09".to_vec(), 11.0),
            (2, b"\x80\x29\x20\x49\x4D\x32\x7F".to_vec(), b"\x80\x49\x20\x69\x03\x00\x4D\x32\x02".to_vec(), 3.0),
        ];

        let (mut saberchecksum, responder) = utils::saberchecksum_responder_harness();
        test_get_method!(saberchecksum, get_current, vectors, responder);
        responder.stop();
    }

    #[test]
    #[rustfmt::skip]
    fn get_temperature() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\x80\x29\x40\x69\x4D\x31\x7E".to_vec(), b"\x80\x49\x40\x09\x1C\x00\x4D\x31\x1A".to_vec(), 28.0),
            (2, b"\x80\x29\x40\x69\x4D\x32\x7F".to_vec(), b"\x80\x49\x40\x09\x1D\x00\x4D\x32\x1C".to_vec(), 29.0),
        ];

        let (mut saberchecksum, responder) = utils::saberchecksum_responder_harness();
        test_get_method!(saberchecksum, get_temperature, vectors, responder);
        responder.stop();
    }
}

mod crc {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn startup() {
        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        let mut buf = [0u8; 32];

        sabercrc.startup(1).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\xf0\x28\x20\x67\x00\x00\x4d\x31\x66\x5c";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        sabercrc.startup(2).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\xf0\x28\x20\x67\x00\x00\x4d\x32\x14\x4c";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        sabercrc.startup(0).expect_err("Channel 0 should fail");
        sabercrc.startup(3).expect_err("Channel 3 should fail");
    }

    #[test]
    #[rustfmt::skip]
    fn shutdown() {
        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        let mut buf = [0u8; 32];

        sabercrc.shutdown(1).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\xf0\x28\x20\x67\x01\x00\x4d\x31\x3b\x22";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        sabercrc.shutdown(2).expect("Startup failure");
        let read_len = tty.read(&mut buf).expect("Read fail");
        let expected = b"\xf0\x28\x20\x67\x01\x00\x4d\x32\x49\x32";
        assert_eq!(expected.len(), read_len, "Wrong length");
        assert_eq!(expected, &buf[0..read_len], "Wrong data");

        sabercrc.shutdown(0).expect_err("Channel 0 should fail");
        sabercrc.shutdown(3).expect_err("Channel 3 should fail");
    }

    #[test]
    #[rustfmt::skip]
    fn set_speed() {
        let vectors = [
            (1,  -1.0, b"\xf0\x28\x01\x20\x7f\x0f\x4d\x31\x51\x3b".to_vec()),
            (2,  -0.5, b"\xf0\x28\x01\x20\x7f\x07\x4d\x32\x65\x6c".to_vec()),
            (1,  0.0,  b"\xf0\x28\x00\x0c\x00\x00\x4d\x31\x66\x5c".to_vec()),
            (1,  0.25, b"\xf0\x28\x00\x0c\x7f\x03\x4d\x31\x74\x5f".to_vec()),
            (2,  0.5,  b"\xf0\x28\x00\x0c\x7f\x07\x4d\x32\x65\x6c".to_vec()),
            (1,  0.75, b"\xf0\x28\x00\x0c\x7f\x0b\x4d\x31\x32\x18".to_vec()),
            (2,  1.0,  b"\xf0\x28\x00\x0c\x7f\x0f\x4d\x32\x23\x2b".to_vec()),
        ];

        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        test_set_method!(sabercrc, set_speed, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_speed_errs() {
        let (mut sabercrc, tty) = utils::sabercrc_harness();
        sabercrc.set_speed(0, 0.0).expect_err("Channel <1 should fail");
        sabercrc.set_speed(3, 0.0).expect_err("Channel >2 should fail");
        sabercrc.set_speed(1, 1.0001).expect_err("Values >100.0 should fail");
        sabercrc.set_speed(1, -1.0001).expect_err("Values <-100.0 should fail");

        // nothing should have been sent over serial
        assert_eq!(0, tty.bytes_to_read().unwrap());
    }

    #[test]
    #[rustfmt::skip]
    fn set_drive() {
        let vectors = [
            (-0.5, b"\xf0\x28\x01\x20\x7f\x07\x4d\x44\x1b\x76".to_vec()),
            (1.0,  b"\xf0\x28\x00\x0c\x7f\x0f\x4d\x44\x5d\x31".to_vec()),
        ];

        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        test_set_method_no_channel!(sabercrc, set_drive, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_turn() {
        let vectors = [
            (-1.0, b"\xF0\x28\x01\x20\x7f\x0f\x4d\x54\x03\x39".to_vec()),
            (0.25, b"\xF0\x28\x00\x0c\x7f\x03\x4d\x54\x26\x5d".to_vec()),
        ];
        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        test_set_method_no_channel!(sabercrc, set_turn, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_power() {
        let vectors = [(1, -1.0, b"\xf0\x28\x01\x20\x7f\x0f\x50\x31\x6e\x1a".to_vec())];

        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        test_set_method!(sabercrc, set_power, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_ramp() {
        let vectors = [(1, 0.25, b"\xf0\x28\x00\x0c\x7f\x03\x52\x31\x0a\x6e".to_vec())];

        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        test_set_method!(sabercrc, set_ramp, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn set_aux() {
        let vectors = [(2, 0.5, b"\xf0\x28\x00\x0c\x7f\x07\x51\x32\x0a\x00".to_vec())];

        let (mut sabercrc, mut tty) = utils::sabercrc_harness();
        test_set_method!(sabercrc, set_aux, vectors, tty);
    }

    #[test]
    #[rustfmt::skip]
    fn get_speed() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\xF0\x29\x00\x6D\x4D\x31\x06\x24".to_vec(), b"\xF0\x49\x00\x15\x00\x0C\x4D\x31\x43\x38".to_vec(), 0.750_366_4),
            (2, b"\xF0\x29\x00\x6D\x4D\x32\x74\x34".to_vec(), b"\xF0\x49\x01\x39\x6B\x05\x4D\x32\x4C\x58".to_vec(), -0.364_924_28),
        ];

        let (mut sabercrc, responder) = utils::sabercrc_responder_harness();
        test_get_method!(sabercrc, get_speed, vectors, responder);
        responder.stop();
    }

    #[test]
    #[rustfmt::skip]
    fn get_voltage() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\xF0\x29\x10\x2E\x4D\x31\x06\x24".to_vec(), b"\xF0\x49\x10\x56\x78\x00\x4D\x31\x54\x0A".to_vec(), 12.0),
            (2, b"\xF0\x29\x10\x2E\x4D\x32\x74\x34".to_vec(), b"\xF0\x49\x10\x56\x78\x00\x4D\x32\x26\x1A".to_vec(), 12.0),
        ];

        let (mut sabercrc, responder) = utils::sabercrc_responder_harness();
        test_get_method!(sabercrc, get_voltage, vectors, responder);
        responder.stop();
    }

    #[test]
    #[rustfmt::skip]
    fn get_current() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\xF0\x29\x20\x06\x4D\x31\x06\x24".to_vec(), b"\xF0\x49\x21\x52\x02\x00\x4D\x31\x3D\x2A".to_vec(), -2.0),
            (2, b"\xF0\x29\x20\x06\x4D\x32\x74\x34".to_vec(), b"\xF0\x49\x20\x7E\x12\x00\x4D\x32\x30\x3C".to_vec(), 18.0),
        ];

        let (mut sabercrc, responder) = utils::sabercrc_responder_harness();
        test_get_method!(sabercrc, get_current, vectors, responder);
        responder.stop();
    }

    #[test]
    #[rustfmt::skip]
    fn get_temperature() {
        #[rustfmt::skip]
            let vectors = [
            (1, b"\xF0\x29\x40\x56\x4D\x31\x06\x24".to_vec(), b"\xF0\x49\x40\x2E\x1C\x00\x4D\x31\x01\x7A".to_vec(), 28.0),
            (2, b"\xF0\x29\x40\x56\x4D\x32\x74\x34".to_vec(), b"\xF0\x49\x40\x2E\x1D\x00\x4D\x32\x2E\x14".to_vec(), 29.0),
        ];

        let (mut sabercrc, responder) = utils::sabercrc_responder_harness();
        test_get_method!(sabercrc, get_temperature, vectors, responder);
        responder.stop();
    }
}
