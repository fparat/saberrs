use std::io::{Read, Write};
use std::time::{Duration, Instant};

use saberrs::SabertoothSerial;

mod utils;

#[test]
fn write_with_device() {
    let (mut saber, mut stub) = utils::saberdevice_harness();

    let msg = b"Hello: From Sabertooth\r\n";
    saber.write_all(msg).expect("Write fail");

    let mut buf = [0u8; 32];
    let read_len = stub.read(&mut buf).expect("Read fail");
    assert_eq!(read_len, msg.len());
    assert_eq!(&buf[0..msg.len()], msg);
}

#[test]
fn read_with_device() {
    let (mut saber, mut stub) = utils::saberdevice_harness();

    let msg = b"Hello: To Sabertooth\r\n";
    stub.write_all(msg).expect("Write fail");

    let mut buf = [0u8; 32];
    let read_len = saber.read(&mut buf).expect("Read fail");
    assert_eq!(read_len, msg.len());
    assert_eq!(&buf[0..msg.len()], msg);
}

#[test]
fn timeout_default_setting() {
    let (saber, _) = utils::saberdevice_harness();
    assert_eq!(saber.timeout(), Duration::from_millis(100));
}

#[test]
fn timeout_setting() {
    let (mut saber, _) = utils::saberdevice_harness();

    let durations = [
        Duration::from_millis(0),
        Duration::from_millis(1000),
        Duration::from_secs(2),
    ];

    for &t in durations.iter() {
        saber.set_timeout(t).expect("Could not set timeout");
        assert_eq!(saber.timeout(), t);
    }
}

// Note: Desktop operating systems are often imprecise with timings in the order
// of milliseconds, so this test may occasionally fail.
#[test]
fn timeout_actual() {
    let (mut saber, _tty) = utils::saberdevice_harness();
    let mut buf = [0u8; 8];
    let delta = Duration::from_millis(5);

    let mut do_timeout = |t| {
        saber.set_timeout(t).expect("Could not set timeout");
        let tstart = Instant::now();
        saber.read(&mut buf).expect_err("Timeout did not occured");
        let tstop = Instant::now();
        let elapsed = tstop - tstart;
        assert!((t - delta) < elapsed);
        assert!(elapsed < (t + delta));
    };

    do_timeout(Duration::from_millis(5));
    do_timeout(Duration::from_millis(50));
    do_timeout(Duration::from_millis(100));
}
