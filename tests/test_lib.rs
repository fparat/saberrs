use std::io::Write;

use saberrs::sabertooth2x32::{PacketSerial, PlainText, Sabertooth2x32};

mod utils;

#[test]
#[allow(unused)]
fn instantiate_multiple_protocols() {
    let (mut dev, tty) = utils::saberdevice_harness_shared();
    let mut sabertext = PlainText::from(&dev);
    let mut saberchecksum = PacketSerial::from(&dev);
    dev.write_all(b"gibberish").expect("Raw interface failed");
    sabertext.set_speed(1, 0.2).expect("Text interface failed");
    saberchecksum
        .set_speed(1, 0.2)
        .expect("Checksum interface failed");
}
