use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub enum ShowSortBy {
    RecordedAt,
    ViewCount,
}

#[derive(Deserialize, Serialize, Debug)]
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

fn default_page_size() -> usize {
    50
}
fn default_sort_by() -> ShowSortBy {
    ShowSortBy::ViewCount
}

impl From<ShowSortBy> for worker::rpc::ListClipsSortBy {
    fn from(s: ShowSortBy) -> Self {
        match s {
            ShowSortBy::RecordedAt => worker::rpc::ListClipsSortBy::RecordedAt,
            ShowSortBy::ViewCount => worker::rpc::ListClipsSortBy::ViewCount,
        }
    }
}
