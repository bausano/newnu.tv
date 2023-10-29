CREATE TABLE IF NOT EXISTS games (
    -- twitch id of the game
    id TEXT NOT NULL UNIQUE,
    -- how's the game named on twitch
    name TEXT NOT NULL,
    -- the url to a small thumbnail image
    box_art_url TEXT NOT NULL,
    -- if paused, the worker will not fetch clips for the game
    is_paused INTEGER DEFAULT TRUE,
    -- when was the game added to the db
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
