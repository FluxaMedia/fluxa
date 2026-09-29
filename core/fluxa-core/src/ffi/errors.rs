use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    UnknownMethod,
    InvalidArgs,
    NotFound,
    Internal,
}

impl ErrorKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ErrorKind::UnknownMethod => "unknown_method",
            ErrorKind::InvalidArgs => "invalid_args",
            ErrorKind::NotFound => "not_found",
            ErrorKind::Internal => "internal",
        }
    }
}

pub(crate) struct CallError {
    pub(crate) kind: ErrorKind,
    pub(crate) message: String,
}

pub(crate) fn fail(kind: ErrorKind, message: impl Into<String>) -> CallError {
    CallError {
        kind,
        message: message.into(),
    }
}

pub(crate) fn unknown_method() -> CallError {
    CallError {
        kind: ErrorKind::UnknownMethod,
        message: String::new(),
    }
}

pub(crate) type Outcome = Result<Value, CallError>;
