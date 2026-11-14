use bytes::Bytes;
use http::StatusCode;
use http_body_util::Full;
use std::error::Error as StdError;
use std::fmt;

/// The boxed Error, the desired way to pass Error.
pub type BError = Box<Error>;

pub trait ErrorTrait: StdError + Sized + Send + Sync {}

#[derive(Debug)]
pub struct Error {
    /// The type of error.
    pub etype: ErrorType,

    /// The source of error: upstream, downstream or internal.
    pub esource: ErrorSource,
}

impl Error {
    fn create(etype: ErrorType, esource: ErrorSource) -> Self {
        Self { etype, esource }
    }

    fn do_new(e: ErrorType, s: ErrorSource) -> Self {
        Self::create(e, s)
    }

    pub fn new(e: ErrorType) -> Self {
        Self::do_new(e, ErrorSource::Unset)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.etype.as_str())
    }
}

impl From<std::io::Error> for Error {
    fn from(_error: std::io::Error) -> Self {
        Error::new(ErrorType::Custom("Standard IO Error".to_string()))
    }
}

impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        std::io::Error::other(error.etype.as_str().to_string())
    }
}

impl From<hyper::Error> for Error {
    fn from(_err: hyper::Error) -> Self {
        Error::new(ErrorType::Custom("Hyper IO Error".to_string()))
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Error::new(ErrorType::ConnectTimeout)
    }
}

impl StdError for Error {}

impl ErrorTrait for Error {}

#[derive(Debug, Clone)]
pub enum ErrorSource {
    Upstream,
    Downstream,
    Internal,
    Unset,
}

impl ErrorSource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Upstream => "Upstream",
            Self::Downstream => "Downstream",
            Self::Internal => "Internal",
            Self::Unset => "",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    // IO errors
    ReadError,
    WriteError,
    ReadTimeout,
    WriteTimeout,
    ConnectionClosed,

    // Connection errors
    ConnectTimeout,
    ConnectRefused,
    ConnectNoRoute,
    TLSHandshakeFailure,
    TLSHandshakeTimeout,
    InvalidCert,
    HandshakeError,
    ConnectError,
    BindError,
    AcceptError,
    SocketError,

    // Other errors
    InternalError,
    StateMachineError,
    UnknownError,

    // Custom errors
    Custom(String),
    CustomCode(String, u16),
}

impl ErrorType {
    /// Creates a custom error with a default HTTP status code.
    pub fn new(name: impl Into<String>, code: u16) -> Self {
        Self::CustomCode(name.into(), code)
    }

    /// Creates a custom error without a specific status code.
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Returns a human-readable error description.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ReadError => "ReadError",
            Self::WriteError => "WriteError",
            Self::ReadTimeout => "ReadTimeout",
            Self::WriteTimeout => "WriteTimeout",
            Self::ConnectionClosed => "ConnectionClosed",
            Self::ConnectTimeout => "ConnectTimeout",
            Self::ConnectRefused => "ConnectRefused",
            Self::ConnectNoRoute => "ConnectNoRoute",
            Self::TLSHandshakeFailure => "TLSHandshakeFailure",
            Self::TLSHandshakeTimeout => "TLSHandshakeTimeout",
            Self::InvalidCert => "InvalidCert",
            Self::HandshakeError => "HandshakeError",
            Self::ConnectError => "ConnectError",
            Self::BindError => "BindError",
            Self::AcceptError => "AcceptError",
            Self::SocketError => "SocketError",
            Self::InternalError => "InternalError",
            Self::UnknownError => "UnknownError",
            Self::StateMachineError => "StateMachineError",
            Self::Custom(s) => s,
            Self::CustomCode(s, _) => s,
        }
    }

    /// Returns the associated HTTP status code.
    pub fn as_code(&self) -> u16 {
        match self {
            Self::ReadError | Self::WriteError => {
                StatusCode::INTERNAL_SERVER_ERROR.as_u16()
            }

            Self::ReadTimeout | Self::WriteTimeout => {
                StatusCode::REQUEST_TIMEOUT.as_u16()
            }

            Self::ConnectionClosed => {
                StatusCode::SERVICE_UNAVAILABLE.as_u16()
            }

            Self::ConnectTimeout | Self::ConnectRefused | Self::ConnectNoRoute => {
                StatusCode::GATEWAY_TIMEOUT.as_u16()
            }

            Self::TLSHandshakeFailure
            | Self::TLSHandshakeTimeout
            | Self::InvalidCert
            | Self::HandshakeError
            | Self::ConnectError
            | Self::BindError
            | Self::AcceptError
            | Self::SocketError => {
                StatusCode::BAD_GATEWAY.as_u16()
            }

            Self::InternalError
            | Self::UnknownError
            | Self::StateMachineError => {
                StatusCode::INTERNAL_SERVER_ERROR.as_u16()
            }

            Self::Custom(_) => StatusCode::BAD_REQUEST.as_u16(),

            Self::CustomCode(_, code) => *code,
        }
    }
}

// Convert Error into HTTP response body.
pub trait IntoHyperResponse {
    fn into_hyper_response(&self) -> Full<Bytes>;
}

impl IntoHyperResponse for Error {
    fn into_hyper_response(&self) -> Full<Bytes> {
        Full::new(Bytes::from(self.to_string()))
    }
}

impl IntoHyperResponse for hyper::http::Error {
    fn into_hyper_response(&self) -> Full<Bytes> {
        Full::new(Bytes::from(self.to_string()))
    }
}

impl IntoHyperResponse for String {
    fn into_hyper_response(&self) -> Full<Bytes> {
        Full::new(Bytes::from(self.clone()))
    }
}