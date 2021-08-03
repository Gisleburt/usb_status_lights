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

type RawMessage = [u8; 8];

use core::convert::TryFrom;

#[cfg_attr(test, derive(PartialEq, Debug))]
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

#[cfg_attr(test, derive(PartialEq, Debug))]
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

#[cfg_attr(test, derive(PartialEq, Debug))]
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

#[cfg_attr(test, derive(PartialEq, Debug))]
#[non_exhaustive]
pub enum Message {
    VersionRequest,
    VersionResponse(VersionNumber),

    BackgroundRequest(LedColor),
    BackgroundResponse { error_code: u8 },

    ForegroundRequest(LedColorTimed),
    ForegroundResponse { error_code: u8 },
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
            Self::VersionRequest => [self.get_id(), 0, 0, 0, 0, 0, 0, 0],
            Self::VersionResponse(v) => [self.get_id(), v.major, v.minor, v.patch, 0, 0, 0, 0],
            Self::BackgroundRequest(led) => [
                self.get_id(),
                led.led,
                led.red,
                led.green,
                led.blue,
                0,
                0,
                0,
            ],
            Self::BackgroundResponse { error_code } => {
                [self.get_id(), *error_code, 0, 0, 0, 0, 0, 0]
            }
            Self::ForegroundRequest(led) => [
                self.get_id(),
                led.led,
                led.red,
                led.green,
                led.blue,
                led.seconds,
                0,
                0,
            ],
            Self::ForegroundResponse { error_code } => {
                [self.get_id(), *error_code, 0, 0, 0, 0, 0, 0]
            }
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum MessageError {
    EmptyMessage,
    InvalidMessageId(u8),
    InvalidMessageForId(u8, RawMessage),
}

impl TryFrom<RawMessage> for Message {
    type Error = MessageError;

    fn try_from(value: RawMessage) -> Result<Self, Self::Error> {
        match value[0] {
            1 => Ok(Self::VersionRequest),
            2 => Ok(Self::VersionResponse(VersionNumber::new(
                value[1], value[2], value[3],
            ))),
            x => Err(MessageError::InvalidMessageId(x)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Message, VersionNumber};
    use core::convert::TryFrom;

    #[test]
    fn test_version_request_to_bytes() {
        let message = Message::VersionRequest;
        assert_eq!(message.to_bytes(), [1, 0, 0, 0, 0, 0, 0, 0])
    }

    #[test]
    fn test_version_request_from_bytes() {
        let raw_message: [u8; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
        let message = Message::try_from(raw_message).unwrap();
        assert_eq!(message, Message::VersionRequest)
    }

    #[test]
    fn test_version_response_to_bytes() {
        let message = Message::VersionResponse(VersionNumber::new(3, 4, 5));
        assert_eq!(message.to_bytes(), [2, 3, 4, 5, 0, 0, 0, 0])
    }

    #[test]
    fn test_version_response_from_bytes() {
        let raw_message: [u8; 8] = [2, 3, 4, 5, 0, 0, 0, 0];
        let message = Message::try_from(raw_message).unwrap();
        assert_eq!(message, Message::VersionResponse(VersionNumber::new(3, 4, 5)))
    }
}
