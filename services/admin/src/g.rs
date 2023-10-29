use crate::job::Jobs;
use crate::prelude::*;
use crate::views::Views;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct HttpState {
    pub db: Arc<Mutex<DbConn>>,
    pub conf: Arc<Conf>,
    pub views: Views,
    pub worker: Arc<Mutex<worker::Client>>,
    pub twitch: Arc<twitch::Client>,
    pub _jobs: Jobs,
}
