use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;

use crate::prelude::*;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TriggerFetchClipsJob {
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub recorded_at_most_hours_ago: Option<usize>,
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub recorded_at_least_hours_ago: Option<usize>,
}
pub async fn trigger_fetch(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Form(TriggerFetchClipsJob {
        recorded_at_most_hours_ago: at_most,
        recorded_at_least_hours_ago: at_least,
    }): Form<TriggerFetchClipsJob>,
) -> Result<Redirect> {
    let at_most = at_most.unwrap_or_default();
    let at_least = at_least.unwrap_or_default();
    if at_most != 0 && at_most <= at_least {
        return Err(AppError::bad_request(
            "Recorded 'at most' must be greater than to 'at least'",
        ));
    }

    info!(
        "Triggering fetch new game clips job for game {game_id} in the \
        last {at_most} hours, at least {at_least} hours ago"
    );

    let mut worker = s.worker.lock().await;
    worker
        .trigger_fetch_new_game_clips(
            worker::rpc::TriggerFetchNewGameClipsRequest {
                game_id: Some(game_id.to_string()),
                recorded_at_most_hours_ago: at_most as i64,
                recorded_at_least_hours_ago: at_least as i64,
            },
        )
        .await?;

    Ok(Redirect::to(&format!("/game/{game_id}")))
}

pub async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Query(query): Query<g::clips::ShowParams>,
) -> Result<Html<String>> {
    let g::clips::ShowParams {
        page_size,
        page_offset,
        sort_direction_asc,
        broadcaster_name,
        title_like,
        langs,
        sort_by,
        view_count_max,
        view_count_min,
        min_recorded_at,
        max_recorded_at,
    } = &query;

    let (total_count, clips) = {
        let mut worker = s.worker.lock().await;
        let worker::rpc::ListClipsResponse { total_count, clips } = worker
            .list_clips(worker::rpc::ListClipsRequest {
                game_id: game_id.to_string(),
                page_size: *page_size as i64,
                page_offset: *page_offset as i64,
                sort_direction_asc: *sort_direction_asc,
                broadcaster_name: broadcaster_name.clone(),
                title_like: title_like.clone(),
                langs: langs.clone(),
                sort_by: worker::rpc::ListClipsSortBy::from(*sort_by).into(),
                view_count_max: view_count_max.map(|v| v as i64),
                view_count_min: *view_count_min as i64,
                min_recorded_at: min_recorded_at.clone(),
                max_recorded_at: max_recorded_at.clone(),
            })
            .await?
            .into_inner();
        let clips = clips
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<AnyResult<Vec<_>>>()?;

        (total_count as usize, clips)
    };

    let db = s.db.lock().await;
    s.views.clips(&db, &game_id, total_count, clips, query)
}
