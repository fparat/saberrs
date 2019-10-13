# Changelog

## v0.2.1 (2019-10-13)
- Add "serialport" feature, optional and enabled by default. The external crate
  "serialport" is a dependency only if the feature "serialport" is enabled.
- The `From` implementations for `PacketSerial` and `PlainText` are generic for
  any `SabertoothSerial` object.
- Remove useless "cargo-readme" dependency.
- Add logging using "log" external crate.
- Improve documentation.

## v0.2.0 (2019-08-27)
First release. Support of Sabertooth 2x32.

