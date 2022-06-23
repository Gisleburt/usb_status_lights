use super::RawMessage;
use crate::{LedColor, LedColorTimed, Message};
use core::convert::TryFrom;

#[repr(u8)]
#[non_exhaustive]
enum RequestId {
    Version = 1,
    Background = 2,
    Foreground = 3,
}

/// A request that can be made of a usb device
#[derive(PartialEq, Debug)]
#[non_exhaustive]
pub enum Request {
    Version,
    Background(LedColor),
    Foreground(LedColorTimed),
}

impl Request {
    fn get_id(&self) -> u8 {
        match self {
            Request::Version => RequestId::Version as u8,
            Request::Background { .. } => RequestId::Background as u8,
            Request::Foreground { .. } => RequestId::Foreground as u8,
        }
    }

    pub fn to_bytes(&self) -> RawMessage {
        match self {
            Self::Version => [self.get_id(), 0, 0, 0, 0, 0, 0, 0],
            Self::Background(led) => [
                self.get_id(),
                led.led,
                led.red,
                led.green,
                led.blue,
                0,
                0,
                0,
            ],
            Self::Foreground(led) => [
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
#[non_exhaustive]
pub enum RequestError {
    InvalidRequest(RawMessage),
    MalformedRequest(RawMessage),
}

impl RequestError {
    pub fn get_id(&self) -> u8 {
        match self {
            RequestError::InvalidRequest(msg) => msg[0],
            RequestError::MalformedRequest(msg) => msg[0],
        }
    }
}

impl TryFrom<RawMessage> for Request {
    type Error = RequestError;

    fn try_from(msg: RawMessage) -> Result<Self, Self::Error> {
        match msg {
            [1, 0, 0, 0, 0, 0, 0, 0] => Ok(Self::Version),
            [1, _, _, _, _, _, _, _] => Err(RequestError::MalformedRequest(msg)),
            [2, led, red, green, blue, 0, 0, 0] => {
                Ok(Self::Background(LedColor::new(led, red, green, blue)))
            }
            [2, _, _, _, _, _, _, _] => Err(RequestError::MalformedRequest(msg)),
            [3, led, red, green, blue, seconds, 0, 0] => Ok(Self::Foreground(LedColorTimed::new(
                led, red, green, blue, seconds,
            ))),
            [3, _, _, _, _, _, _, _] => Err(RequestError::MalformedRequest(msg)),
            _ => Err(RequestError::InvalidRequest(msg)),
        }
    }
}

impl From<Request> for RawMessage {
    fn from(req: Request) -> Self {
        req.to_bytes()
    }
}

impl Message for Request {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_request_to_bytes() {
        let message = Request::Version;
        assert_eq!(message.to_bytes(), [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_version_request_from_bytes() {
        let raw_message: [u8; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
        let message = Request::try_from(raw_message).unwrap();
        assert_eq!(message, Request::Version);
    }

    #[test]
    fn test_background_request_to_bytes() {
        let message = Request::Background(LedColor::new(1, 255, 255, 255));
        assert_eq!(message.to_bytes(), [2, 1, 255, 255, 255, 0, 0, 0]);
    }

    #[test]
    fn test_background_request_from_bytes() {
        let raw_message: [u8; 8] = [2, 1, 255, 255, 255, 0, 0, 0];
        let message = Request::try_from(raw_message).unwrap();
        assert_eq!(
            message,
            Request::Background(LedColor::new(1, 255, 255, 255))
        );
    }

    #[test]
    fn test_foreground_request_to_bytes() {
        let message = Request::Foreground(LedColorTimed::new(1, 255, 255, 255, 10));
        assert_eq!(message.to_bytes(), [3, 1, 255, 255, 255, 10, 0, 0]);
    }

    #[test]
    fn test_foreground_request_from_bytes() {
        let raw_message: [u8; 8] = [3, 1, 255, 255, 255, 10, 0, 0];
        let message = Request::try_from(raw_message).unwrap();
        assert_eq!(
            message,
            Request::Foreground(LedColorTimed::new(1, 255, 255, 255, 10))
        );
    }
}
