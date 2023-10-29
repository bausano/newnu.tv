use crate::prelude::*;
use axum::response::Html;

pub async fn page(State(s): State<g::HttpState>) -> Result<Html<String>> {
    let db = s.db.lock().await;
    s.views.homepage(&db)
}
