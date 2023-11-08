use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct Clip {
    pub broadcaster_id: String,
    pub broadcaster_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub creator_name: String,
    pub duration: Duration,
    pub game_id: String,
    pub id: String,
    pub lang: String,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
    pub thumbnail_url: String,
    pub title: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub url: String,
    pub view_count: usize,
}

impl TryFrom<crate::rpc::Clip> for Clip {
    type Error = anyhow::Error;

    fn try_from(clip: crate::rpc::Clip) -> Result<Self, Self::Error> {
        Ok(Self {
            broadcaster_id: clip.broadcaster_id,
            broadcaster_name: clip.broadcaster_name,
            created_at: clip.created_at.parse()?,
            creator_name: clip.creator_name,
            duration: Duration::from_secs(clip.duration_secs as u64),
            game_id: clip.game_id,
            id: clip.id,
            lang: clip.lang,
            recorded_at: clip.recorded_at.parse()?,
            thumbnail_url: clip.thumbnail_url,
            title: clip.title,
            updated_at: clip.updated_at.parse()?,
            url: clip.url,
            view_count: clip.view_count as usize,
        })
    }
}
