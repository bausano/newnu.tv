use crate::prelude::*;
use axum::response::Html;
use handlebars::Handlebars;
use serde_json::json;
use std::sync::Arc;
use twitch::models::GameId;

#[derive(Clone)]
pub struct Views {
    handlebars: Arc<Handlebars<'static>>,
}

impl Views {
    pub fn new() -> AnyResult<Self> {
        let mut h = Handlebars::new();
        h.set_strict_mode(false);

        // contains the head imports
        h.register_template_string("base", include_str!("views/base.hbs"))?;

        h.register_template_string(
            "homepage",
            include_str!("views/homepage.hbs"),
        )?;

        h.register_template_string(
            "search_game",
            include_str!("views/search_game.hbs"),
        )?;

        h.register_template_string("game", include_str!("views/game.hbs"))?;

        h.register_template_string(
            "settings",
            include_str!("views/settings.hbs"),
        )?;

        h.register_template_string("clips", include_str!("views/clips.hbs"))?;

        Ok(Self {
            handlebars: Arc::new(h),
        })
    }

    /// Pull all necessary data to render homepage from db.
    pub fn homepage(&self, db: &DbConn) -> Result<Html<String>> {
        let games: Vec<_> = db::game::select_all(db)?;

        self.handlebars
            .render("homepage", &json!({ "parent": "base", "games": games }))
            .map(Html)
            .map_err(From::from)
    }

    pub fn search_game(
        &self,
        games: &[twitch::models::Game],
    ) -> Result<Html<String>> {
        self.handlebars
            .render(
                "search_game",
                &json!({
                    "parent": "base",
                    "games": games
                }),
            )
            .map(Html)
            .map_err(From::from)
    }

    /// Pull all necessary data to render game info from db.
    pub fn game(&self, db: &DbConn, game: &GameId) -> Result<Html<String>> {
        let game = db::game::select_by_id(db, game)?;

        self.handlebars
            .render("game", &json!({ "parent": "base", "game": game }))
            .map(Html)
            .map_err(From::from)
    }

    /// View global settings.
    pub fn settings(&self, db: &DbConn) -> Result<Html<String>> {
        let fetch_new_game_clips_cron =
            db::setting::fetch_new_game_clips_cron(db)?;
        let recorded_at_least_hours_ago =
            db::setting::recorded_at_least_hours_ago(db)?;

        self.handlebars
            .render(
                "settings",
                &json!(
                    {
                        "parent": "base",
                        "settings": {
                            "fetch_new_game_clips_cron": fetch_new_game_clips_cron,
                            "recorded_at_least_hours_ago": recorded_at_least_hours_ago,
                        }
                    }
                ),
            )
            .map(Html)
            .map_err(From::from)
    }

    pub fn clips(
        &self,
        db: &DbConn,
        game_id: &GameId,
        total_count: usize,
        clips: Vec<worker::models::clip::Clip>,
    ) -> Result<Html<String>> {
        let game = db::game::select_by_id(db, game_id)?;

        self.handlebars
            .render(
                "clips",
                &json!({
                    "parent": "base",
                    "game": game,
                    "total_count": total_count,
                    "clips": clips,
                }),
            )
            .map(Html)
            .map_err(From::from)
    }
}
