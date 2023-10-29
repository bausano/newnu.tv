use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::prelude::*;

pub async fn search(
    State(s): State<g::HttpState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>> {
    let query = params
        .get("q")
        .ok_or_else(|| AppError::bad_request("Missing query param 'q'"))?;

    let games = s.twitch.search_for_game(query).await?;

    s.views.search_game(&games)
}

pub async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;
    s.views.game(&db, &game_id)
}

pub async fn add(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    let Some(game) = s.twitch.get_game(game_id.clone()).await? else {
        return Err(AppError::bad_request(format!(
            "Game with id {game_id} not found"
        )));
    };

    {
        let mut worker = s.worker.lock().await;
        worker
            .add_game(worker::rpc::AddGameRequest {
                game_id: game_id.to_string(),
                game_name: game.name.clone(),
            })
            .await?;
    }

    let db = s.db.lock().await;
    db::game::insert(&db, &game)?;

    Ok(Redirect::to(&format!("/game/{game_id}")))
}

pub async fn delete(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    {
        let mut worker = s.worker.lock().await;
        worker
            .delete_game(worker::rpc::DeleteGameRequest {
                game_id: game_id.to_string(),
            })
            .await?;
    }

    let db = s.db.lock().await;
    db::game::delete(&db, &game_id)?;

    Ok(Redirect::to("/"))
}

pub async fn pause(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    set_is_paused(&s, game_id.clone(), true).await?;

    Ok(Redirect::to(&format!("/game/{game_id}")))
}

pub async fn resume(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    set_is_paused(&s, game_id.clone(), false).await?;

    Ok(Redirect::to(&format!("/game/{game_id}")))
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TriggerFetchClipsJob {
    pub recorded_at_most_hours_ago: usize,
    pub recorded_at_least_hours_ago: usize,
}
pub async fn trigger_fetch_clips_job(
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
    use worker::rpc::trigger_fetch_new_game_clips_request;
    worker
        .trigger_fetch_new_game_clips(
            worker::rpc::TriggerFetchNewGameClipsRequest {
                game_id: Some(
                    trigger_fetch_new_game_clips_request::GameId::Id(
                        game_id.to_string(),
                    ),
                ),
                recorded_at_most_hours_ago: at_most as i64,
                recorded_at_least_hours_ago: at_least as i64,
            },
        )
        .await?;

    Ok(Redirect::to(&format!("/game/{game_id}")))
}

async fn set_is_paused(
    s: &g::HttpState,
    game_id: twitch::models::GameId,
    is_paused: bool,
) -> Result<()> {
    {
        let mut worker = s.worker.lock().await;
        worker
            .set_is_game_paused(worker::rpc::SetIsGamePausedRequest {
                game_id: game_id.to_string(),
                is_paused,
            })
            .await?;
    }

    let db = s.db.lock().await;
    db::game::set_is_paused(&db, &game_id, is_paused)?;

    Ok(())
}
