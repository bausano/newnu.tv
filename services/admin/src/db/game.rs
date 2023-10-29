use crate::prelude::*;
use rusqlite::{named_params, ErrorCode};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Game {
    pub id: twitch::models::GameId,
    pub name: String,
    /// Note that this _might_ contain a placeholder for the size.
    ///
    /// https://static-cdn.jtvnw.net/ttv-boxart/{game_id}-{width}x{height}.jpg
    pub box_art_url: String,
    pub is_paused: bool,
}

pub fn insert(db: &DbConn, game: &twitch::models::Game) -> Result<()> {
    let res = db.execute(
        "INSERT INTO
            games (id, name, box_art_url)
        VALUES
            (:id, :name, :box_art_url);",
        named_params! {
            ":id": game.id,
            ":name": game.name,
            ":box_art_url": game.box_art_url,
        },
    );
    let Err(e) = res else { return Ok(()) };

    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => Err(AppError::already_exists(
            format!("Game with id {} already exists", game.id),
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

pub fn select_all(db: &DbConn) -> Result<Vec<Game>> {
    let mut stmt = db.prepare_cached(
        "SELECT id, name, box_art_url, is_paused
        FROM games ORDER BY is_paused ASC, name ASC;",
    )?;
    let games = stmt
        .query_map((), |row| Game::try_from(row))?
        .map(|res| res.map_err(AppError::from))
        .collect::<Result<_>>()?;
    Ok(games)
}

pub fn select_by_id(
    db: &DbConn,
    game_id: &twitch::models::GameId,
) -> Result<Game> {
    let mut stmt = db.prepare_cached(
        "SELECT id, name, box_art_url, is_paused
        FROM games WHERE id = :id;",
    )?;
    let game = stmt.query_row(named_params! { ":id": game_id }, |row| {
        Game::try_from(row)
    })?;
    Ok(game)
}

pub fn set_is_paused(
    db: &DbConn,
    game_id: &twitch::models::GameId,
    is_paused: bool,
) -> Result<()> {
    let res = db.execute(
        "UPDATE games SET is_paused = :is_paused WHERE id = :game_id;",
        named_params! { ":game_id": game_id, ":is_paused": is_paused },
    );
    let Err(e) = res else { return Ok(()) };

    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => Err(AppError::already_exists(
            format!("Game with id {game_id} does not exist"),
        )),
        _ => Err(AppError::internal(e.to_string())),
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Game {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> StdResult<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            box_art_url: twitch::models::into_standard_box_art_size(
                row.get::<_, String>("box_art_url")?,
            ),
            is_paused: row.get("is_paused")?,
        })
    }
}
