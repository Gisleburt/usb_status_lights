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

use core::convert::TryFrom;

mod response;
mod request;

pub use response::{Response, ResponseError};
pub use request::{Request, RequestError};

type RawMessage = [u8; 8];

pub trait Message: TryFrom<RawMessage> + Into<RawMessage> {}

/// Version number represents the version of the software running on the USB device.
#[derive(PartialEq, Debug)]
pub struct VersionNumber {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl VersionNumber {
    pub fn new(major: u8, minor: u8, patch: u8) -> VersionNumber {
        VersionNumber {
            major,
            minor,
            patch,
        }
    }
}

/// Represents the color of a specific led
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LedColor {
    pub led: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl LedColor {
    pub fn new(led: u8, red: u8, green: u8, blue: u8) -> LedColor {
        LedColor {
            led,
            red,
            green,
            blue,
        }
    }
}

/// Represents the color of a specific led and how long it will remain that color
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LedColorTimed {
    pub led: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub seconds: u8,
}

impl LedColorTimed {
    pub fn new(led: u8, red: u8, green: u8, blue: u8, seconds: u8) -> LedColorTimed {
        LedColorTimed {
            led,
            red,
            green,
            blue,
            seconds,
        }
    }
}


