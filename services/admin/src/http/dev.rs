use crate::prelude::*;
use axum::response::Redirect;

pub async fn reset(State(s): State<g::HttpState>) -> Result<Redirect> {
    {
        let mut worker = s.worker.lock().await;
        worker.dev_reset(()).await?;
    }

    let mut db = s.db.lock().await;
    db::down(&mut db)?;
    db::up(&mut db)?;

    Ok(Redirect::to("/"))
}
