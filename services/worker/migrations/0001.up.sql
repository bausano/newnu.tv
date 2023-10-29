CREATE TABLE IF NOT EXISTS games (
    -- twitch id of the game
    id TEXT NOT NULL UNIQUE,
    -- how's the game named on twitch
    name TEXT NOT NULL,
    -- if paused, the worker will not fetch clips for the game
    is_paused INTEGER DEFAULT TRUE,
    -- the last time we queried clips for this game
    -- we start with a past value to populate the db
    last_queried_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-24 hours'))
);
