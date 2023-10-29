use std::time::Duration;

use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct Clip {
    /// Twitch assigned id
    pub id: String,
    /// Twitch assigned id
    pub broadcaster_id: String,
    /// Display name is case insensitively equal to the twitch broadcaster
    /// login
    pub broadcaster_name: String,
    /// The name of the user who recorded the clip
    pub creator_name: String,
    /// When the clip was created on Twitch side
    pub recorded_at: String,
    /// We store only seconds in db
    pub duration: Duration,
    /// Title of the clip as set by the creator, hence unsafe
    pub title: String,
    /// Where to download the clip
    pub url: String,
    /// The image url for thumbnail
    pub thumbnail_url: String,
    /// The number of times the clip has been viewed.
    /// This number depends on when we queried the APIs.
    pub view_count: usize,
    /// Can be empty if Twitch doesn't recognize the language
    pub lang: String,
    /// Twitch assigned id
    pub game_id: String,
}

impl Clip {
    pub fn file_name(&self) -> String {
        format!("{}_{}.mp4", self.broadcaster_name, self.id)
    }
}

/// Bind clips to it and execute all inserts at once.
#[cfg(feature = "sqlite")]
pub struct InsertClipStatement<'a>(rusqlite::Statement<'a>);

#[cfg(feature = "sqlite")]
impl<'a> InsertClipStatement<'a> {
    pub fn new(db: &'a rusqlite::Connection) -> Result<Self, anyhow::Error> {
        // replace will update view count
        let stmt = db.prepare(
            "
            INSERT OR REPLACE INTO
            clips(
                id,
                broadcaster_id,
                broadcaster_name,
                creator_name,
                recorded_at,
                duration,
                title,
                thumbnail_url,
                url,
                view_count,
                lang,
                game_id
            )
            VALUES (
                :id,
                :broadcaster_id,
                :broadcaster_name,
                :creator_name,
                :recorded_at,
                :duration,
                :title,
                :thumbnail_url,
                :url,
                :view_count,
                :lang,
                :game_id
            );",
        )?;

        Ok(Self(stmt))
    }

    pub fn execute(&mut self, clip: &Clip) -> Result<(), anyhow::Error> {
        self.0
            .execute(rusqlite::named_params! {
                ":id": clip.id,
                ":broadcaster_id": clip.broadcaster_id,
                ":broadcaster_name": clip.broadcaster_name,
                ":creator_name": clip.creator_name,
                ":recorded_at": clip.recorded_at,
                ":duration": (clip.duration.as_secs() as i64),
                ":title": clip.title,
                ":thumbnail_url": clip.thumbnail_url,
                ":url": clip.url,
                ":view_count": (clip.view_count as i64),
                ":lang": clip.lang,
                ":game_id": clip.game_id,
            })
            .map_err(|e| anyhow!("Cannot bind clip: {e}"))?;

        Ok(())
    }
}
