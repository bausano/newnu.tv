use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::prelude::*;

#[derive(Clone)]
pub struct Jobs {
    pub scheduler: JobScheduler,
    pub fetch_new_game_clips: uuid::Uuid,
}

pub async fn schedule_all(
    db: DbLock,
    worker: Arc<Mutex<worker::Client>>,
) -> AnyResult<Jobs> {
    let mut scheduler = JobScheduler::new().await?;

    let db = db.lock().await;
    let fetch_new_game_clips_cron =
        db::setting::fetch_new_game_clips_cron(&db)?;
    let recorded_at_least_hours_ago =
        db::setting::recorded_at_least_hours_ago(&db)?;
    drop(db);

    let fetch_new_game_clips = scheduler
        .add(Job::new_async(
            fetch_new_game_clips_cron.as_ref(),
            move |_, _| {
                debug!("Triggering fetch_new_game_clips");

                let worker = Arc::clone(&worker);
                Box::pin(async move {
                    let res = worker
                        .lock()
                        .await
                        .trigger_fetch_new_game_clips(
                            worker::rpc::TriggerFetchNewGameClipsRequest {
                                game_id: None,                 // for all
                                recorded_at_most_hours_ago: 0, // last clip
                                recorded_at_least_hours_ago,
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
