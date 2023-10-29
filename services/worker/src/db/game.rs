use crate::prelude::*;
use chrono::Utc;
use rusqlite::{named_params, ErrorCode};

pub fn insert(
    db: &DbConn,
    id: &twitch::models::GameId,
    name: impl AsRef<str>,
) -> Result<()> {
    let res = db.execute(
        "INSERT INTO
            games (id, name)
        VALUES
            (:id, :name);",
        named_params! {
            ":id": id,
            ":name": name.as_ref(),
        },
    );
    let Err(e) = res else { return Ok(()) };

    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => Err(AppError::already_exists(
            format!("Game with id {id} already exists"),
        )),
        _ => Err(AppError::internal(e.to_string())),
    }
}

pub fn delete(db: &DbConn, game_id: &twitch::models::GameId) -> Result<()> {
    db.execute(
        "DELETE FROM games WHERE id = :id;",
        named_params! { ":id": game_id },
    )
    .map(drop)
    .map_err(From::from)
}

pub fn set_is_paused(
    db: &DbConn,
    game_id: &twitch::models::GameId,
    is_paused: bool,
) -> Result<()> {
    db.execute(
        "UPDATE games SET is_paused = :is_paused WHERE id = :game_id;",
        named_params! { ":game_id": game_id, ":is_paused": is_paused },
    )
    .map(drop)
    .map_err(From::from)
}

pub fn select_all_active_that_have_last_clip_older_than(
    db: &DbConn,
    duration: chrono::Duration,
) -> Result<Vec<(twitch::models::GameId, Option<chrono::DateTime<Utc>>)>> {
    db.prepare(
        "
            SELECT
                games.id as game_id,
                MAX(clips.recorded_at) as latest_clip_recorded_at
            FROM games
            LEFT JOIN clips ON clips.game_id = games.id
            WHERE games.is_paused = FALSE
            AND games.id NOT IN (
                SELECT game_id
                FROM clips
                WHERE recorded_at > :last_queried
            )
            GROUP BY games.id
        ",
    )?
    .query_map(
        named_params! { ":last_queried": chrono::Utc::now() - duration },
        |row| Ok((row.get("game_id")?, row.get("latest_clip_recorded_at")?)),
    )?
    .map(|res| res.map_err(AppError::from))
    .collect()
}

pub fn select_latest_clip_recorded_at(
    db: &DbConn,
    game_id: &twitch::models::GameId,
) -> Result<Option<chrono::DateTime<Utc>>> {
    db.prepare(
        "
            SELECT
                MAX(clips.recorded_at) as latest_clip_recorded_at
            FROM clips
            WHERE clips.game_id = :game_id
        ",
    )?
    .query_row(named_params! { ":game_id": game_id }, |row| {
        row.get("latest_clip_recorded_at")
    })
    .map_err(AppError::from)
}
