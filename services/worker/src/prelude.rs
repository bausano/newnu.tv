pub(crate) use anyhow::{Error as AnyError, Result as AnyResult};
pub(crate) use log::{debug, error, info, trace, warn};
pub(crate) use std::result::Result as StdResult;

pub(crate) use crate::conf::Conf;
pub(crate) use crate::db;
pub(crate) use crate::error::AppError;
pub(crate) use crate::g::AppState;

pub(crate) type DbConn = rusqlite::Connection;
pub(crate) type DbLock = std::sync::Arc<tokio::sync::Mutex<DbConn>>;
pub(crate) type Result<T> = StdResult<T, AppError>;
