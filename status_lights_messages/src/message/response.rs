use crate::{Message, VersionNumber};
use core::convert::TryFrom;
use super::RawMessage;

/// A response the device can give back to the host
#[derive(PartialEq, Debug)]
#[non_exhaustive]
pub enum Response {
    ErrorResponse { error_code: u8 },
    VersionResponse(VersionNumber),
    BackgroundResponse,
    ForegroundResponse,
}

impl Response {
    fn get_id(&self) -> u8 {
        match self {
            Self::ErrorResponse { .. } => 0,
            Self::VersionResponse { .. } => 1,
            Self::BackgroundResponse { .. } => 2,
            Self::ForegroundResponse { .. } => 3,
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        match self {
            Self::ErrorResponse { error_code } => [self.get_id(), *error_code, 0, 0, 0, 0, 0, 0],
            Self::VersionResponse(v) => [self.get_id(), v.major, v.minor, v.patch, 0, 0, 0, 0],
            Self::BackgroundResponse => { [self.get_id(), 0, 0, 0, 0, 0, 0, 0] }
            Self::ForegroundResponse => { [self.get_id(), 0, 0, 0, 0, 0, 0, 0] }
        }
    }
}

/// Possible errors that might result from a potential request
#[derive(PartialEq, Debug)]
pub enum ResponseError {
    InvalidResponseId(u8),
}

impl TryFrom<RawMessage> for Response {
    type Error = ResponseError;

    fn try_from(value: RawMessage) -> Result<Self, Self::Error> {
        match value[0] {
            0 => Ok(Self::ErrorResponse { error_code: value[1] }),

            1 => Ok(Self::VersionResponse(VersionNumber::new(
                value[1], value[2], value[3],
            ))),

            2 => Ok(Self::BackgroundResponse),

            3 => Ok(Self::ForegroundResponse),
            x => Err(ResponseError::InvalidResponseId(x)),
        }
    }
}

impl Into<RawMessage> for Response {
    fn into(self) -> RawMessage {
        self.to_bytes()
    }
}

impl Message for Response {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_response_to_bytes() {
        let message = Response::VersionResponse(VersionNumber::new(3, 4, 5));
        assert_eq!(message.to_bytes(), [1, 3, 4, 5, 0, 0, 0, 0]);
    }

    #[test]
    fn test_version_response_from_bytes() {
        let raw_message: [u8; 8] = [1, 3, 4, 5, 0, 0, 0, 0];
        let message = Response::try_from(raw_message).unwrap();
        assert_eq!(
            message,
            Response::VersionResponse(VersionNumber::new(3, 4, 5))
        );
    }

    #[test]
    fn test_background_response_to_bytes() {
        let message = Response::BackgroundResponse { error_code: 42 };
        assert_eq!(message.to_bytes(), [2, 42, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_background_response_from_bytes() {
        let raw_message: [u8; 8] = [2, 42, 0, 0, 0, 0, 0, 0];
        let message = Response::try_from(raw_message).unwrap();
        assert_eq!(message, Response::BackgroundResponse { error_code: 42 });
    }

    #[test]
    fn test_foreground_response_to_bytes() {
        let message = Response::ForegroundResponse { error_code: 101 };
        assert_eq!(message.to_bytes(), [3, 101, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_foreground_response_from_bytes() {
        let raw_message: [u8; 8] = [3, 101, 0, 0, 0, 0, 0, 0];
        let message = Response::try_from(raw_message).unwrap();
        assert_eq!(message, Response::ForegroundResponse { error_code: 101 });
    }
}
