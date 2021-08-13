use core::convert::TryFrom;
use crate::{LedColor, LedColorTimed, Message};
use super::RawMessage;

// #[repr(u8)]
// enum RequestId {
//     VersionRequest = 1,
//     BackgroundRequest = 2,
//     ForegroundRequest = 3,
// }

/// A request that can be made of a usb device
#[derive(PartialEq, Debug)]
#[non_exhaustive]
pub enum Request {
    VersionRequest,
    BackgroundRequest(LedColor),
    ForegroundRequest(LedColorTimed),
}

impl Request {
    fn get_id(&self) -> u8 {
        match self {
            Request::VersionRequest => 1,
            Request::BackgroundRequest { .. } => 2,
            Request::ForegroundRequest { .. } => 3,
        }
    }

    pub fn to_bytes(&self) -> RawMessage {
        match self {
            Self::VersionRequest => [self.get_id(), 0, 0, 0, 0, 0, 0, 0],
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
        }
    }
}

/// Possible errors that might result from a potential request
#[derive(PartialEq, Debug)]
pub enum RequestError {
    InvalidResponseId(u8),
}

impl TryFrom<RawMessage> for Request {
    type Error = RequestError;

    fn try_from(value: RawMessage) -> Result<Self, Self::Error> {
        match value[0] {
            1 => Ok(Self::VersionRequest),
            2 => Ok(Self::BackgroundRequest(LedColor::new(
                value[1], value[2], value[3], value[4],
            ))),
            3 => Ok(Self::ForegroundRequest(LedColorTimed::new(
                value[1], value[2], value[3], value[4], value[5],
            ))),
            x => Err(RequestError::InvalidResponseId(x)),
        }
    }
}

impl Into<RawMessage> for Request {
    fn into(self) -> [u8; 8] {
        self.to_bytes()
    }
}

impl Message for Request {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_request_to_bytes() {
        let message = Request::VersionRequest;
        assert_eq!(message.to_bytes(), [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_version_request_from_bytes() {
        let raw_message: [u8; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
        let message = Request::try_from(raw_message).unwrap();
        assert_eq!(message, Request::VersionRequest);
    }

    #[test]
    fn test_background_request_to_bytes() {
        let message = Request::BackgroundRequest(LedColor::new(1, 255, 255, 255));
        assert_eq!(message.to_bytes(), [2, 1, 255, 255, 255, 0, 0, 0]);
    }

    #[test]
    fn test_background_request_from_bytes() {
        let raw_message: [u8; 8] = [2, 1, 255, 255, 255, 0, 0, 0];
        let message = Request::try_from(raw_message).unwrap();
        assert_eq!(
            message,
            Request::BackgroundRequest(LedColor::new(1, 255, 255, 255))
        );
    }

    #[test]
    fn test_foreground_request_to_bytes() {
        let message = Request::ForegroundRequest(LedColorTimed::new(1, 255, 255, 255, 10));
        assert_eq!(message.to_bytes(), [3, 1, 255, 255, 255, 10, 0, 0]);
    }

    #[test]
    fn test_foreground_request_from_bytes() {
        let raw_message: [u8; 8] = [3, 1, 255, 255, 255, 10, 0, 0];
        let message = Request::try_from(raw_message).unwrap();
        assert_eq!(
            message,
            Request::ForegroundRequest(LedColorTimed::new(1, 255, 255, 255, 10))
        );
    }
}
