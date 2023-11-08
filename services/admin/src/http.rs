/// endpoints for clips management
mod clips;
/// endpoints which ease development
mod dev;
/// endpoints for game management
mod game;
/// homepage
mod home;
/// endpoints for global settings
mod settings;

use crate::prelude::*;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hyper::StatusCode;
use serde_json::json;
use std::net::SocketAddr;

pub async fn start(g: g::HttpState) -> AnyResult<()> {
    let http_addr = g.conf.http_addr;

    let routes = routes().with_state(g);

    info!("http://{http_addr}");
    axum::Server::bind(&http_addr)
        .serve(routes.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

fn routes() -> Router<g::HttpState> {
    async fn handler_404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Invalid path",
            })),
        )
    }

    // since data is being sent with <form> which only supports GET and POST,
    // we instead append the method verb to the path (with the exception of GET)
    //
    // this is a trade-off between ~dogma~ conventions and no-js life
    // i'll take the latter any day of the week
    let app = Router::new()
        .route("/", get(home::page))
        .route("/search/game", get(game::search))
        .route("/game/:game_id", get(game::show))
        .route("/game/:game_id/post", post(game::add))
        .route("/game/:game_id/delete", post(game::delete))
        .route("/game/:game_id/pause/post", post(game::pause))
        .route("/game/:game_id/pause/delete", post(game::resume))
        .route("/game/:game_id/clips", get(clips::show))
        .route(
            "/game/:game_id/clips/fetch/post",
            post(clips::trigger_fetch),
        )
        .route("/settings", get(settings::show))
        .route("/settings/put", post(settings::edit))
        .route("/dev/reset/post", post(dev::reset))
        .fallback(handler_404);

    debug!("Http routes constructed");
    app
}
