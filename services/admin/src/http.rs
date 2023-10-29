// homepage
mod home;
// endpoints for game management
mod game;
// endpoints which ease development
mod dev;

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
        .route("/game/:game_id", get(game::show))
        .route(
            "/game/:game_id/job/fetch_clips",
            post(game::trigger_fetch_clips_job),
        )
        .route("/game/:game_id/post", post(game::add))
        .route("/game/:game_id/delete", post(game::delete))
        .route("/game/:game_id/pause/post", post(game::pause))
        .route("/game/:game_id/pause/delete", post(game::resume))
        .route("/search/game", get(game::search))
        .route("/dev/reset", post(dev::reset))
        .fallback(handler_404);

    debug!("Http routes constructed");
    app
}
