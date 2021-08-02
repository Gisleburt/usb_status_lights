//! Messages are sent as a stream of `u8`s. This first byte tells the recipient what the message
//! will be, and subsequent bytes contain any additional information required by that message.
//!
//! Messages are always 8 bytes long, padded with 0s where the message contains less than 8 bytes
//! of information
//!
//! For example, a request for the version number of the software running on a USB device would be
//! ```text
//! 1 0 0 0 0 0 0 0
//! ```
//! 1 is the ID for Version Request
//!
//! The response would ve
//! ```text
//! 2 4 5 6 0 0 0 0
//! ````
//! 2 is the ID for Version Response
//! 4 is the major version
//! 5 is the minor version
//! 6 is the patch version

#![no_std]

// const EMPTY_MESSAGE: [u8; 8] = [0; 8];
//
// macro_rules! raw_message {
//     ($bytes:tt) => {
//         $bytes.iter().chain(iter::repeat(&0)).take(8).collect()
//     }
// }

type RawMessage = [u8; 8];

use core::convert::TryFrom;

pub struct VersionNumber {
    major: u8,
    minor: u8,
    patch: u8,
}

impl VersionNumber {
    fn new(major: u8, minor: u8, patch: u8) -> VersionNumber {
        VersionNumber {
            major,
            minor,
            patch,
        }
    }
}

pub struct LedColor {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
}

pub struct LedColorTimed {
    led: u8,
    red: u8,
    green: u8,
    blue: u8,
    seconds: u8,
}

#[non_exhaustive]
pub enum Message {
    VersionRequest,
    VersionResponse(VersionNumber),

    BackgroundRequest(LedColor),
    BackgroundResponse {
        error_code: u8,
    },

    ForegroundRequest(LedColorTimed),
    ForegroundResponse {
        error_code: u8,
    },
}

impl Message {
    fn get_id(&self) -> u8 {
        match self {
            Message::VersionRequest => 1,
            Message::VersionResponse { .. } => 2,
            Message::BackgroundRequest { .. } => 3,
            Message::BackgroundResponse { .. } => 4,
            Message::ForegroundRequest { .. } => 5,
            Message::ForegroundResponse { .. } => 6,
        }
    }

    pub fn to_bytes(&self) -> RawMessage {
        match self {
            Message::VersionRequest => {
                [self.get_id(), 0, 0, 0, 0, 0, 0, 0]
            }
            Message::VersionResponse(v) => {
                [self.get_id(), v.major, v.minor, v.patch, 0, 0, 0, 0]
            }
            Message::BackgroundRequest(led) => {
                [self.get_id(), led.led, led.red, led.green, led.blue, 0, 0, 0]
            }
            Message::BackgroundResponse { error_code } => {
                [self.get_id(), *error_code, 0, 0, 0, 0, 0, 0]
            }
            Message::ForegroundRequest(led) => {
                [self.get_id(), led.led, led.red, led.green, led.blue, led.seconds, 0, 0]
            }
            Message::ForegroundResponse { error_code } => {
                [self.get_id(), *error_code, 0, 0, 0, 0, 0, 0]
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Message;

    #[test]
    fn test_version_request() {
        let message = Message::VersionRequest;
        assert_eq!(message.to_bytes(), [1, 0, 0, 0, 0, 0, 0, 0])
    }
}
