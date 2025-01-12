use std::error::Error as StdError;
use std::fmt;

use http::StatusCode;
/// The boxed [Error], the desired way to pass [Error]
pub type BError = Box<Error>;

pub trait ErrorTrait: StdError + Sized + Send + Sync {}

#[derive(Debug)]
pub struct Error {
    /// the type of error
    pub etype: ErrorType,
    /// the source of error: from upstream, downstream or internal
    pub esource: ErrorSource,
    // chain to the cause of this error
    // pub cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>,
    // pub context: Option<Imm>
}

impl Error {
    fn create(
        etype: ErrorType,
        esource: ErrorSource,
        //cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>,
    ) -> Error {
        Error {
            etype,
            esource,
            //cause,
        }
    }

    fn do_new(e: ErrorType, s: ErrorSource) -> Error {
        Self::create(e, s)
    }

    pub fn new(e: ErrorType) -> Error {
        Self::do_new(e, ErrorSource::Unset)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
impl ErrorTrait for Error {}

#[derive(Debug)]
pub enum ErrorSource {
    /// The error is caused by the remote server
    Upstream,
    /// The error is caused by the remote client
    Downstream,
    /// The error is caused by the internal logic
    Internal,
    /// Error source unknown or to be set
    Unset,
}

impl ErrorSource {
    /// for displaying the error source
    pub fn as_str(&self) -> &str {
        match self {
            Self::Upstream => "Upstream",
            Self::Downstream => "Downstream",
            Self::Internal => "Internal",
            Self::Unset => "",
        }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    // IO Error for connection
    ReadError,
    WriteError,
    ReadTimeout,
    WriteTimeout,
    ConnectionClosed,
    // Connect Error
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
    // other errors
    InternalError,
    StateMachineError,
    // catch all
    UnknownError,
    /// Custom error with static string.
    /// this field is to allow users to extend the types of errors. If runtime generated string
    /// is needed, it is more likely to be treated as "context" rather than "type".
    Custom(&'static str),
    /// Custom error with static string and code.
    /// this field allows users to extend error further with error codes.
    CustomCode(&'static str, u16),
}

impl ErrorType {
    pub const fn new(name: &'static str, code: u16) -> Self {
        ErrorType::CustomCode(name, code)
    }

    /// create a new type of error. Users should try to make `name` unique.
    pub const fn new_code(name: &'static str, code: u16) -> Self {
        ErrorType::CustomCode(name, code)
    }

    pub fn as_str(&self) -> &str {
        match self {
            ErrorType::ReadError => "ConnectTimeout",
            ErrorType::WriteError => "WriteError",
            ErrorType::ReadTimeout => "ReadTimeout",
            ErrorType::WriteTimeout => "WriteTimeout",
            ErrorType::ConnectionClosed => "ConnectionClosed",
            ErrorType::ConnectTimeout => "ConnectTimeout",
            ErrorType::ConnectRefused => "ConnectRefused",
            ErrorType::ConnectNoRoute => "ConnectNoRoute",
            ErrorType::TLSHandshakeFailure => "TLSHandshakeFailure",
            ErrorType::TLSHandshakeTimeout => "TLSHandshakeTimeout",
            ErrorType::InvalidCert => "InvalidCert",
            ErrorType::HandshakeError => "HandshakeError",
            ErrorType::ConnectError => "ConnectError",
            ErrorType::BindError => "BindError",
            ErrorType::AcceptError => "AcceptError",
            ErrorType::SocketError => "SocketError",
            ErrorType::InternalError => "InternalError",
            ErrorType::UnknownError => "UnknownError",
            ErrorType::StateMachineError => "StateMachineError",
            ErrorType::Custom(s) => s,
            ErrorType::CustomCode(s, _) => s,
        }
    }

    pub fn as_code(&self) -> u16 {
        return match self {
            ErrorType::ReadError | ErrorType::WriteError => {
                StatusCode::INTERNAL_SERVER_ERROR.as_u16()
            }
            ErrorType::ReadTimeout | ErrorType::WriteTimeout => {
                StatusCode::REQUEST_TIMEOUT.as_u16()
            }
            ErrorType::ConnectionClosed => StatusCode::SERVICE_UNAVAILABLE.as_u16(),
            ErrorType::ConnectTimeout | ErrorType::ConnectRefused | ErrorType::ConnectNoRoute => {
                StatusCode::GATEWAY_TIMEOUT.as_u16()
            }
            ErrorType::TLSHandshakeFailure
            | ErrorType::TLSHandshakeTimeout
            | ErrorType::InvalidCert
            | ErrorType::HandshakeError => StatusCode::BAD_GATEWAY.as_u16(),
            ErrorType::ConnectError
            | ErrorType::BindError
            | ErrorType::AcceptError
            | ErrorType::SocketError => StatusCode::BAD_GATEWAY.as_u16(),
            ErrorType::InternalError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrorType::UnknownError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrorType::StateMachineError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ErrorType::Custom(_) => todo!(),
            ErrorType::CustomCode(_, _) => todo!(),
        };
    }
}