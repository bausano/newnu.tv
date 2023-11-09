use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub enum ShowSortBy {
    RecordedAt,
    #[default]
    ViewCount,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub struct ShowParams {
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub page_offset: usize,
    #[serde(default)]
    pub sort_direction_asc: bool,
    #[serde(default)]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub broadcaster_name: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub title_like: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "g::csv_string_is_vec")]
    pub langs: Vec<String>,
    #[serde(default = "default_sort_by")]
    pub sort_by: ShowSortBy,
    pub view_count_max: Option<usize>,
    #[serde(default)]
    pub view_count_min: usize,
    #[serde(default)]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub min_recorded_at: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub max_recorded_at: Option<String>,
}

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

fn default_page_size() -> usize {
    50
}
fn default_sort_by() -> ShowSortBy {
    ShowSortBy::ViewCount
}

impl From<ShowSortBy> for &'static str {
    fn from(s: ShowSortBy) -> Self {
        match s {
            ShowSortBy::RecordedAt => "recorded_at",
            ShowSortBy::ViewCount => "view_count",
        }
    }
}
