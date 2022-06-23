use core::convert::TryFrom;

use super::RawMessage;
use crate::{Message, RequestError, VersionNumber};

/// A response the device can give back to the host
#[derive(PartialEq, Debug)]
#[non_exhaustive]
pub enum Response {
    ErrorResponse(ErrorResponse),
    Version(VersionNumber),
    Background,
    Foreground,
}

impl Response {
    fn get_id(&self) -> u8 {
        match self {
            Self::ErrorResponse { .. } => 0,
            Self::Version { .. } => 1,
            Self::Background { .. } => 2,
            Self::Foreground { .. } => 3,
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        match self {
            Self::ErrorResponse(error_code) => match error_code {
                ErrorResponse::UnknownRequestId(id) => [
                    self.get_id(),
                    ErrorResponseCodes::UnknownResponseId as u8,
                    *id,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                ErrorResponse::MalformedRequestForId(id) => [
                    self.get_id(),
                    ErrorResponseCodes::MalformedRequestForId as u8,
                    *id,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            },
            Self::Version(v) => [self.get_id(), v.major, v.minor, v.patch, 0, 0, 0, 0],
            Self::Background => [self.get_id(), 0, 0, 0, 0, 0, 0, 0],
            Self::Foreground => [self.get_id(), 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

/// Possible errors that might result from a potential request
#[derive(PartialEq, Debug)]
pub enum ResponseError {
    ErrorResponse(ErrorResponse),
    UnknownResponse(RawMessage),
}

impl From<ErrorResponse> for ResponseError {
    fn from(error: ErrorResponse) -> Self {
        ResponseError::ErrorResponse(error)
    }
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum ErrorResponseCodes {
    UnknownResponseId = 1,
    MalformedRequestForId = 2,
}

#[derive(PartialEq, Debug)]
pub enum ErrorResponse {
    UnknownRequestId(u8),
    MalformedRequestForId(u8),
}

impl From<RequestError> for ErrorResponse {
    fn from(error: RequestError) -> Self {
        match error {
            RequestError::InvalidRequest(msg) => ErrorResponse::UnknownRequestId(msg[0]),
            RequestError::MalformedRequest(msg) => ErrorResponse::MalformedRequestForId(msg[0]),
        }
    }
}

impl From<ErrorResponse> for Response {
    fn from(error: ErrorResponse) -> Self {
        Response::ErrorResponse(error)
    }
}

impl From<RawMessage> for ResponseError {
    fn from(msg: RawMessage) -> Self {
        match (msg[0], msg[1]) {
            (0, 1) => ErrorResponse::UnknownRequestId(msg[2]).into(),
            (0, 2) => ErrorResponse::MalformedRequestForId(msg[2]).into(),
            _ => Self::UnknownResponse(msg),
        }
    }
}

impl TryFrom<RawMessage> for Response {
    type Error = ResponseError;

    fn try_from(msg: RawMessage) -> Result<Self, Self::Error> {
        match msg[0] {
            1 => Ok(Self::Version(VersionNumber::new(msg[1], msg[2], msg[3]))),

            2 => Ok(Self::Background),

            3 => Ok(Self::Foreground),

            // This handles both errors returned from the device (messages starting with 0) and
            // errors from not understanding the response
            _ => Err(ResponseError::from(msg)),
        }
    }
}

impl From<Response> for RawMessage {
    fn from(response: Response) -> Self {
        response.to_bytes()
    }
}

impl Message for Response {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_response_to_bytes() {
        let message = Response::Version(VersionNumber::new(3, 4, 5));
        assert_eq!(message.to_bytes(), [1, 3, 4, 5, 0, 0, 0, 0]);
    }

    #[test]
    fn test_version_response_from_bytes() {
        let raw_message: [u8; 8] = [1, 3, 4, 5, 0, 0, 0, 0];
        let message = Response::try_from(raw_message).unwrap();
        assert_eq!(message, Response::Version(VersionNumber::new(3, 4, 5)));
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
