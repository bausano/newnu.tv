use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;

use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TriggerFetchClipsJob {
    pub recorded_at_most_hours_ago: usize,
    pub recorded_at_least_hours_ago: usize,
}
pub async fn trigger_fetch(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Form(body): Form<TriggerFetchClipsJob>,
) -> Result<Redirect> {
    let TriggerFetchClipsJob {
        recorded_at_most_hours_ago: at_most,
        recorded_at_least_hours_ago: at_least,
    } = body;

    if at_most != 0 && at_most <= at_least {
        return Err(AppError::bad_request(
            "Recorded 'at most' must be greater than to 'at least'",
        ));
    }

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

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ShowParams {
    #[serde(default)]
    pub page_size: usize,
    #[serde(default)]
    pub page_offset: usize,
    #[serde(default)]
    pub sort_direction_asc: bool,
    pub broadcaster_name: Option<String>,
    pub title_like: Option<String>,
    #[serde(default)]
    pub langs: Vec<String>,
    pub sort_by: Option<ShowSortBy>,
    pub view_count_max: Option<usize>,
    #[serde(default)]
    pub view_count_min: usize,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShowSortBy {
    RecordedAt,
    ViewCount,
}
pub async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Query(params): Query<ShowParams>,
) -> Result<Html<String>> {
    let ShowParams {
        page_size,
        page_offset,
        sort_direction_asc,
        broadcaster_name,
        title_like,
        langs,
        sort_by,
        view_count_max,
        view_count_min,
    } = params;
    let page_size = if page_size == 0 { 10 } else { page_size }; // TODO: consts

    let sort_by = match sort_by {
        Some(ShowSortBy::RecordedAt) | None => {
            worker::rpc::ListClipsSortBy::RecordedAt.into()
        }
        Some(ShowSortBy::ViewCount) => {
            worker::rpc::ListClipsSortBy::ViewCount.into()
        }
    };

    let (total_count, clips) = {
        let mut worker = s.worker.lock().await;
        let worker::rpc::ListClipsResponse { total_count, clips } = worker
            .list_clips(worker::rpc::ListClipsRequest {
                game_id: game_id.to_string(),
                page_size: page_size as i64,
                page_offset: page_offset as i64,
                sort_direction_asc,
                broadcaster_name,
                title_like,
                langs,
                sort_by,
                view_count_max: view_count_max.map(|v| v as i64),
                view_count_min: view_count_min as i64,
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
    s.views.clips(&db, &game_id, total_count, clips)
}
