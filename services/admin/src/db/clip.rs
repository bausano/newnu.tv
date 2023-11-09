use itertools::Itertools;
use rusqlite::named_params;
use std::{rc::Rc, time::Duration};
use twitch::models::GameId;

use crate::models::clip::{Clip, ShowParams};
use crate::prelude::*;

pub fn list(
    db: &DbConn,
    game_id: &GameId,
    request: &ShowParams,
) -> Result<(usize, Vec<Clip>)> {
    let ShowParams {
        broadcaster_name,
        langs,
        max_recorded_at,
        min_recorded_at,
        page_offset,
        page_size,
        sort_by,
        sort_direction_asc,
        title_like,
        view_count_max,
        view_count_min,
    } = request;

    if *page_size == 0 {
        return Err(AppError::bad_request(
            "page_size must be greater than 0".to_string(),
        ));
    }

    let sort_by: &str = From::from(*sort_by);
    let sort_direction = if *sort_direction_asc { "ASC" } else { "DESC" };
    // array feature of sqlite
    let langs = Rc::new(
        langs
            .into_iter()
            .cloned()
            .map(rusqlite::types::Value::from)
            .collect_vec(),
    );
    let where_clause = "WHERE
        game_id = :game_id
        AND (:broadcaster_name IS NULL OR broadcaster_name = :broadcaster_name)
        AND (:title_like IS NULL OR title LIKE '%' || :title_like || '%')
        AND (:skip_langs OR lang IN rarray(:langs))
        AND (:view_count_max IS NULL OR view_count <= :view_count_max)
        AND view_count >= :view_count_min
        AND (:min_recorded_at IS NULL OR recorded_at >= :min_recorded_at)
        AND (:max_recorded_at IS NULL OR recorded_at <= :max_recorded_at)";

    let total_count_sql = format!("SELECT COUNT(*) FROM clips {where_clause}");
    let params = named_params! {
        ":broadcaster_name": broadcaster_name,
        ":game_id": game_id,
        ":langs": langs,
        ":max_recorded_at": max_recorded_at,
        ":min_recorded_at": min_recorded_at,
        ":skip_langs": langs.is_empty(),
        ":title_like": title_like,
        ":view_count_max": view_count_max,
        ":view_count_min": view_count_min,
    };
    let total_count: i64 = db
        .prepare(&total_count_sql)?
        .query_row(params, |row| row.get(0))?;

    let paginated_sql = format!(
        "SELECT
            broadcaster_id,
            broadcaster_name,
            created_at,
            creator_name,
            duration,
            game_id,
            id,
            lang,
            recorded_at,
            thumbnail_url,
            title,
            updated_at,
            url,
            view_count
        FROM clips
        {where_clause}
        ORDER BY {sort_by} {sort_direction}
        LIMIT :page_size
        OFFSET :page_offset"
    );
    let params = named_params! {
        ":broadcaster_name": broadcaster_name,
        ":game_id": game_id,
        ":langs": langs,
        ":max_recorded_at": max_recorded_at,
        ":min_recorded_at": min_recorded_at,
        ":page_offset": page_offset,
        ":page_size": page_size,
        ":skip_langs": langs.is_empty(),
        ":title_like": title_like,
        ":view_count_max": view_count_max,
        ":view_count_min": view_count_min,
    };
    let clips = db
        .prepare(&paginated_sql)?
        .query_map(params, |row| Clip::try_from(row))?
        .map(|row| row.map_err(AppError::from))
        .try_collect()?;

    Ok((total_count as usize, clips))
}

impl TryFrom<&rusqlite::Row<'_>> for Clip {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> StdResult<Self, Self::Error> {
        Ok(Self {
            broadcaster_id: row.get("broadcaster_id")?,
            broadcaster_name: row.get("broadcaster_name")?,
            created_at: row.get("created_at")?,
            creator_name: row.get("creator_name")?,
            duration: Duration::from_secs(row.get::<_, i64>("duration")? as u64),
            thumbnail_url: row.get("thumbnail_url")?,
            game_id: row.get("game_id")?,
            id: row.get("id")?,
            lang: row.get("lang")?,
            recorded_at: row.get("recorded_at")?,
            title: row.get("title")?,
            updated_at: row.get("updated_at")?,
            url: row.get("url")?,
            view_count: row.get("view_count")?,
        })
    }
}

#[cfg(test)]
mod tests {
    //! We have an SQL with some sample data in tests/assets/clips.sql.
    //! Load that data, insert it into in-memory sqlite.

    use crate::models::clip::ShowSortBy;

    use super::*;

    #[test]
    fn it_lists_clips_sorted_asc() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 5,
                sort_by: ShowSortBy::RecordedAt,
                sort_direction_asc: true,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 5);
        assert_eq!(clips[0].id, "FlirtyTenuousCasetteHoneyBadger");
        assert_eq!(clips[1].id, "MoistUnsightlyBatteryTwitchRPG");

        Ok(())
    }

    #[test]
    fn it_lists_clips_sorted_desc() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 1,
                sort_by: ShowSortBy::RecordedAt,
                sort_direction_asc: false,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].id, "DeterminedShyGullKappaWealth");

        Ok(())
    }

    #[test]
    fn it_lists_clips_by_views() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 2,
                sort_by: ShowSortBy::ViewCount,
                sort_direction_asc: true,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "DeterminedShyGullKappaWealth");
        assert_eq!(clips[1].id, "SuaveHonestWeaselJKanStyle");

        Ok(())
    }

    #[test]
    fn it_lists_clips_in_views_range() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 2,
                sort_by: ShowSortBy::ViewCount,
                sort_direction_asc: false,
                view_count_max: Some(1500),
                view_count_min: 500,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 4);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "ToughZealousFungusDancingBanana");
        assert_eq!(clips[1].id, "EsteemedShinyAsteriskBibleThump");

        Ok(())
    }

    #[test]
    fn it_lists_clips_with_title_like() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                title_like: Some("pytlik".to_string()),
                page_size: 1,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 1);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].id, "ToughZealousFungusDancingBanana");

        Ok(())
    }

    #[test]
    fn it_lists_clips_with_offset() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 1,
                page_offset: 1,
                sort_by: ShowSortBy::RecordedAt,
                sort_direction_asc: true,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].id, "MoistUnsightlyBatteryTwitchRPG");

        Ok(())
    }

    #[test]
    fn it_lists_clips_based_on_langs() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 10,
                langs: vec!["en".to_string()],
                ..Default::default()
            },
        )?;
        assert_eq!(clips.len(), 7);
        assert_eq!(total_count, 7);

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 10,
                langs: vec!["nl".to_string()],
                ..Default::default()
            },
        )?;
        assert_eq!(clips.len(), 1);
        assert_eq!(total_count, 1);

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 10,
                langs: vec!["nl".to_string(), "cs".to_string()],
                ..Default::default()
            },
        )?;
        assert_eq!(clips.len(), 2);
        assert_eq!(total_count, 2);

        Ok(())
    }

    #[test]
    fn it_lists_clips_based_on_broadcaster() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                page_size: 2,
                broadcaster_name: Some("Davaeorn".to_string()),
                sort_direction_asc: true,
                sort_by: ShowSortBy::RecordedAt,
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 5);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "MoistUnsightlyBatteryTwitchRPG");
        assert_eq!(clips[1].id, "EsteemedShinyAsteriskBibleThump");

        Ok(())
    }

    fn prepare_db() -> Result<DbConn> {
        pretty_env_logger::try_init_timed().ok();

        let db = db::open(":memory:")?;

        db.execute_batch(include_str!("../../../../tests/assets/clips.sql"))?;

        Ok(db)
    }
}
