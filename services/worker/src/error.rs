use crate::prelude::*;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct AppError {
    pub message: Cow<'static, str>,
    pub kind: AppErrorKind,
}

#[derive(Default, Debug, Serialize)]
pub enum AppErrorKind {
    AlreadyExists,
    NotFound,
    /// Internal server error.
    ///
    /// We don't track the error kind for that error as it's too specific.
    /// Will result in console log.
    #[default]
    Other,
}

impl AppError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::Other,
        }
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::AlreadyExists,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::NotFound,
        }
    }
}

impl From<AnyError> for AppError {
    fn from(err: AnyError) -> Self {
        Self {
            message: err.to_string().into(),
            kind: AppErrorKind::Other,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self {
            message: err.to_string().into(),
            kind: AppErrorKind::Other,
        }
    }
}

impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        let code = match err.kind {
            AppErrorKind::AlreadyExists => tonic::Code::InvalidArgument,
            AppErrorKind::NotFound => tonic::Code::NotFound,
            AppErrorKind::Other => tonic::Code::Internal,
        };

        Self::new(code, err.message)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
