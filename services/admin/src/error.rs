use crate::prelude::*;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use serde::Serialize;
use serde_json::json;
use std::borrow::Cow;

#[derive(Debug)]
pub struct AppError {
    pub message: Cow<'static, str>,
    pub kind: AppErrorKind,
    pub status: StatusCode,
}

#[derive(Default, Debug, Serialize)]
pub enum AppErrorKind {
    /// Something wrong with the user request, see message for more info.
    BadRequest,
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
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::BadRequest,
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::AlreadyExists,
            status: StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let Self {
            message,
            kind,
            status,
        } = self;

        if matches!(kind, AppErrorKind::Other) {
            error!("500: {message}");
        }

        let body = Json(json!({
            "kind": kind,
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl From<AnyError> for AppError {
    fn from(err: AnyError) -> Self {
        Self {
            message: err.to_string().into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<handlebars::RenderError> for AppError {
    fn from(err: handlebars::RenderError) -> Self {
        Self {
            message: err.to_string().into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<tonic::Status> for AppError {
    fn from(err: tonic::Status) -> Self {
        let (kind, status) = match err.code() {
            tonic::Code::InvalidArgument => {
                (AppErrorKind::BadRequest, StatusCode::BAD_REQUEST)
            }
            tonic::Code::NotFound => {
                (AppErrorKind::NotFound, StatusCode::NOT_FOUND)
            }
            _ => (AppErrorKind::Other, StatusCode::INTERNAL_SERVER_ERROR),
        };

        Self {
            message: err.message().to_string().into(),
            kind,
            status,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self {
            message: err.to_string().into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
