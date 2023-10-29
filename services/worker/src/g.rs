use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbLock,
    pub conf: Arc<Conf>,
    pub twitch: Arc<twitch::Client>,
}
