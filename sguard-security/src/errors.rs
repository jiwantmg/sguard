use hyper::StatusCode;

#[derive(Debug)]
pub struct AuthError {
    pub etype: ErrorType,
}

impl AuthError {
    pub fn new(etype: ErrorType) -> AuthError {
        AuthError { etype }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    InvalidToken,
    WrongCredentials,
    TokenCreation,
    WriteTimeout,
    UnsupportedAuthentication,
    UnknownError,
    Unauthorized,
    /// Custom error with a static string for extensibility.
    Custom(&'static str),
    /// Custom error with a static string and an error code.
    CustomCode(&'static str, u16),
}

impl ErrorType {
    /// Creates a custom error with a static string and code.
    /// Use this method to extend error types with unique names and codes.
    pub const fn new(name: &'static str, code: u16) -> Self {
        Self::CustomCode(name, code)
    }

    /// Returns a human-readable string representation of the error.
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidToken => "Invalid token",
            Self::WrongCredentials => "Wrong credentials",
            Self::TokenCreation => "Token creation failed",
            Self::WriteTimeout => "Write timeout",
            Self::UnknownError => "Unknown error",
            Self::UnsupportedAuthentication => "Unsupported Authentication",
            Self::Unauthorized => "Unauthorized",
            Self::Custom(message) => message,
            Self::CustomCode(message, _) => message,
        }
    }

    /// Returns the associated status code for the error.
    pub fn as_code(&self) -> u16 {
        match self {
            Self::InvalidToken => StatusCode::UNAUTHORIZED.as_u16(),
            Self::WrongCredentials => StatusCode::FORBIDDEN.as_u16(),
            Self::TokenCreation => StatusCode::SERVICE_UNAVAILABLE.as_u16(),
            Self::WriteTimeout => StatusCode::REQUEST_TIMEOUT.as_u16(),
            Self::UnknownError => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            Self::UnsupportedAuthentication => StatusCode::UNAUTHORIZED.as_u16(),
            Self::Unauthorized => StatusCode::UNAUTHORIZED.as_u16(),
            Self::Custom(_) => StatusCode::BAD_REQUEST.as_u16(), // Default for custom errors
            Self::CustomCode(_, code) => *code,                  // Use provided code
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (code: {})",
            self.etype.as_str(),
            self.etype.as_code()
        )
    }
}

impl std::error::Error for AuthError {}
