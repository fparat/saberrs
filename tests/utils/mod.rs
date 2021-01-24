#![allow(unused)]
use std::cell::RefCell;
use std::rc::Rc;

use serialport::SerialPort;
use serialport::TTYPort;

use saberrs::sabertooth2x32::{PacketSerial, PacketType, PlainText};
use saberrs::{SabertoothPort, SabertoothPortShared, SabertoothSerial};

mod responder;
pub use responder::*;

/// Return a (master, slave) tuple. The slave is set to non-exclusive and
/// can be used to connect a SabertoothDevice, then the master may be used
/// for interacting with it.
pub fn tty_pair() -> (TTYPort, TTYPort) {
    let (master, mut slave) = TTYPort::pair().expect("Unable to create pseudo-terminal pair");
    slave
        .set_exclusive(false)
        .expect("Cannot unset exclusivity of salve tty.");
    (master, slave)
}

/// Return a new SabertoothDevice, and a TTY for talking to it.
pub fn saberdevice_harness() -> (SabertoothPort, TTYPort) {
    let (master, slave) = tty_pair();
    let slave_name = &slave.name().expect("TTY has no name");
    let saber = SabertoothPort::new(slave_name).expect("Cannot open the sabertooth device");
    (saber, master)
}

pub fn saberdevice_harness_shared() -> (SabertoothPortShared, TTYPort) {
    let (master, slave) = tty_pair();
    let slave_name = &slave.name().expect("TTY has no name");
    let saber = SabertoothPortShared::new(slave_name).expect("Cannot open the sabertooth device");
    (saber, master)
}

/// Float equality assertion that is good enough for our use-case
#[macro_export]
macro_rules! assert_eq_float {
    ($x:expr, $y:expr) => {
        if (($x - $y) as f64).abs() > 0.001 {
            panic!("{} and {} are not (nearly) equal", $x, $y);
        }
    };
}

macro_rules! test_set_method {
    ($saber:expr, $setter:ident, $vectors:expr, $tty:expr) => {
        for (channel, value, expected) in $vectors.iter() {
            $saber.$setter(*channel, *value).expect("Set value failure");
            let mut buf = [0u8; 32];
            let read_len = $tty.read(&mut buf).expect("Read fail");
            assert_eq!(&buf[0..read_len], &expected[..], "Wrong frame content");
        }
    };
}

macro_rules! test_set_method_no_channel {
    ($saber:expr, $setter:ident, $vectors:expr, $tty:expr) => {
        for (value, expected) in $vectors.iter() {
            $saber.$setter(*value).expect("Set value failure");
            let mut buf = [0u8; 32];
            let read_len = $tty.read(&mut buf).expect("Read fail");
            assert_eq!(&buf[0..read_len], &expected[..], "Wrong frame content");
        }
    };
}

macro_rules! check_responder_error {
    ($responder: expr, $res: expr) => {
        if $responder.is_alive() {
            if let Err(e) = $res {
                panic!("{}", e);
            }
        } else {
            panic!($responder.join_panic().unwrap_err());
        }
    };
}

macro_rules! test_get_method_float_with_channel {
    ($saber:expr, $getter:ident, $vectors:expr, $responder:expr) => {
        for (channel, expected, response, value) in $vectors.iter() {
            $responder.set_expected(expected.as_ref());
            $responder.set_response(response.as_ref());

            let res = $saber.$getter(*channel);

            if let Ok(ret) = res {
                assert_eq_float!(value, ret);
            }

            check_responder_error!($responder, res)
        }
    };
}

macro_rules! test_get_method_no_channel {
    ($saber:expr, $getter:ident, $vectors:expr, $responder:expr) => {
        for (expected, response, value) in $vectors.iter() {
            $responder.set_expected(expected.as_ref());
            $responder.set_response(response.as_ref());

            let res = $saber.$getter();

            if let Ok(ret) = res {
                assert_eq!(value, &ret);
            }

            check_responder_error!($responder, res)
        }
    };
}

macro_rules! test_get_method_float_no_channel {
    ($saber:expr, $getter:ident, $vectors:expr, $responder:expr) => {
        for (expected, response, value) in $vectors.iter() {
            $responder.set_expected(expected.as_ref());
            $responder.set_response(response.as_ref());

            let res = $saber.$getter();

            if let Ok(ret) = res {
                assert_eq_float!(value, &ret);
            }

            check_responder_error!($responder, res)
        }
    };
}
