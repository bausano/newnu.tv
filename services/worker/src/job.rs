pub mod fetch_new_game_clips;

use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::prelude::*;

#[derive(Clone)]
pub struct Jobs {
    pub scheduler: JobScheduler,
    pub fetch_new_game_clips: uuid::Uuid,
}

const CRON_ONCE_AN_HOUR: &str =
    // sec   min   hour   day of month   month   day of week   year
    " *      *     0      *              *       *             *";

pub async fn schedule_all(g: AppState) -> AnyResult<Jobs> {
    let scheduler = JobScheduler::new().await?;

    let fetch_new_game_clips = scheduler
        .add(Job::new_async(CRON_ONCE_AN_HOUR, move |_, _| {
            let db = Arc::clone(&g.db);
            let tc = Arc::clone(&g.twitch);
            Box::pin(async move {
                if let Err(e) = fetch_new_game_clips::once_for_all(db, tc).await
                {
                    error!("Error in fetch_new_game_clips job: {e}");
                }
            })
        })?)
        .await?;

    let shed = scheduler.clone();
    tokio::spawn(async move {
        if let Err(e) = shed.start().await {
            error!("Error in scheduler: {e}");
        }
    });

    Ok(Jobs {
        scheduler,
        fetch_new_game_clips,
    })
}
