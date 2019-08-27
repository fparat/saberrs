use std::collections::VecDeque;
use std::fmt::Display;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serialport::SerialPort;

pub enum ResponderType {
    Text,     // response sent when b'\n' is received
    Checksum, // response is sent after the last expected byte is received
    CRC,      // same as Checksum
}

/// Structure used for mocking a Sabertooth (real) device.
/// It will check that the bytes it receives match what it expects. When all
/// expected data is received it will send back a predefined response.
/// Its start() method will activate it and return a "ResponderController" that
/// can be used for setting the expected data, the response, and stopping it.
pub struct Responder {
    type_: ResponderType,
    tty: Box<dyn SerialPort>,
    expected: VecDeque<u8>, // will be consumed during checking
    response: Vec<u8>,
}

impl Responder {
    pub fn new(tty: Box<dyn SerialPort>, type_: ResponderType) -> Responder {
        Responder {
            type_,
            tty,
            expected: VecDeque::new(),
            response: Vec::new(),
        }
    }

    pub fn start(mut self) -> ResponderController {
        let (tx, rx) = mpsc::sync_channel(0);
        let join_handle = thread::spawn(move || {
            // Init: set read timeout on tty
            self.tty
                .set_timeout(Duration::from_millis(10))
                .expect("Cannot set timeout");

            // Start listening
            loop {
                // Process command of parent: stop or update data
                match rx.try_recv() {
                    Ok(ResponderCmd::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                        if !self.expected.is_empty() {
                            panic!("Expected data were not received: {:?}", self.expected)
                        }
                        break;
                    }
                    Ok(ResponderCmd::SetExpected(exp)) => self.expected = exp,
                    Ok(ResponderCmd::SetResponse(resp)) => self.response = resp,
                    Ok(ResponderCmd::Ping) => {}
                    _ => {}
                }

                // Read and process from tty
                let mut buf = [0u8; 1];
                match self.tty.read_exact(&mut buf) {
                    Ok(_) => self.assert_next_byte(buf[0]),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                    Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                        // Parent thread most probably ended, this thread will
                        // gracefully stop during command processing next loop.
                    }
                    Err(e) => panic!("Read fail: {}", e),
                }
            }
        });

        ResponderController {
            join_handle,
            tx: Box::new(tx),
        }
    }

    fn assert_next_byte(&mut self, received: u8) {
        let expected_byte = self.expected.pop_front().expect("Received too many bytes");
        if received != expected_byte {
            panic!(format!(
                "Expected {:#02x} ({:?}) but received {:#02x} ({:?})",
                expected_byte, expected_byte as char, received, received as char
            ))
        }
        if self.must_respond(received) {
            self.tty
                .write_all(self.response.as_ref())
                .expect("Write fail");
        }
    }

    fn must_respond(&self, received: u8) -> bool {
        match self.type_ {
            ResponderType::Text => received == b'\n',
            ResponderType::Checksum => self.expected.is_empty(),
            ResponderType::CRC => self.expected.is_empty(),
        }
    }
}

/// Used for controlling a started Responder.
pub struct ResponderController {
    join_handle: thread::JoinHandle<()>,
    tx: Box<mpsc::SyncSender<ResponderCmd>>,
}

impl ResponderController {
    pub fn set_expected(&self, expected: &[u8]) {
        self.tx
            .send(ResponderCmd::SetExpected(VecDeque::from(expected.to_vec())))
            .unwrap();
    }

    pub fn set_response(&self, response: &[u8]) {
        self.tx
            .send(ResponderCmd::SetResponse(response.to_vec()))
            .unwrap();
    }

    pub fn stop(self) {
        self.tx.send(ResponderCmd::Stop).unwrap();
        self.join_handle
            .join()
            .expect("Error when stopping Responder")
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.join_handle.join()
    }

    pub fn is_alive(&self) -> bool {
        // Sending a dummy command will succeed if the responder is alive.
        self.tx.send(ResponderCmd::Ping).is_ok()
    }

    /// Join the responder thread, and in case this thread failed, extract the
    /// error string. Used for identifying assertion failures in the responder
    /// side. Should be called only when `self.is_alive()` is false.
    pub fn join_panic(self) -> std::result::Result<(), String> {
        match self.join() {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(e) = e.downcast_ref::<String>() {
                    Err(e.to_string())
                } else if let Some(e) = e.downcast_ref::<&'static str>() {
                    Err(e.to_string())
                } else {
                    Err(format!("Unknown error: {:?}", e))
                }
            }
        }
    }
}

enum ResponderCmd {
    Stop,
    Ping,
    SetExpected(VecDeque<u8>),
    SetResponse(Vec<u8>),
}
