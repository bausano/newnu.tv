pub mod fetch_new_game_clips;

use anyhow::anyhow;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::prelude::*;

#[derive(Clone)]
pub struct Jobs {
    pub scheduler: JobScheduler,
    pub fetch_new_game_clips: uuid::Uuid,
}

pub async fn schedule_all(
    db: DbLock,
    tc: Arc<twitch::Client>,
) -> AnyResult<Jobs> {
    let mut scheduler = JobScheduler::new().await?;

    let (fetch_new_game_clips_cron, recorded_at_least_ago) = {
        let db = db.lock().await;
        let new_clips_cron = db::setting::fetch_new_game_clips_cron(&db)?;
        let at_least_ago = db::setting::recorded_at_least_hours_ago(&db)?;

        (new_clips_cron, Some(chrono::Duration::hours(at_least_ago)))
    };

    let fetch_new_game_clips = scheduler
        .add(Job::new_async(
            fetch_new_game_clips_cron.as_ref(),
            move |_, _| {
                debug!("Triggering fetch_new_game_clips");

                let db = Arc::clone(&db);
                let tc = Arc::clone(&tc);

                Box::pin(async move {
                    let res = fetch_new_game_clips::once_for_all(
                        Arc::clone(&db),
                        Arc::clone(&tc),
                        fetch_new_game_clips::Conf {
                            recorded_at_most_ago: None, // last clip
                            recorded_at_least_ago,
                        },
                    )
                    .await;
                    if let Err(e) = res {
                        error!("Cannot trigger fetching new game clips: {e}");
                    }
                })
            },
        )?)
        .await?;

    let next_tick = scheduler
        .next_tick_for_job(fetch_new_game_clips)
        .await?
        .ok_or_else(|| {
            anyhow!("Cannot find next tick for fetch_new_game_clips")
        })?;
    info!("Next tick for fetch_new_game_clips: {next_tick}");

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
