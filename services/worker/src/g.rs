use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub conf: Arc<Conf>,
}
