use anyhow::anyhow;

use crate::prelude::*;

pub fn update(db: &DbConn, name: &str, value: &str) -> Result<()> {
    let mut stmt = db.prepare(
        "INSERT OR REPLACE INTO settings (name, value) VALUES (:name, :value)",
    )?;
    stmt.execute(rusqlite::named_params! {
        ":name": name,
        ":value": value,
    })?;

    Ok(())
}

pub fn fetch_new_game_clips_cron(db: &DbConn) -> Result<String> {
    fetch(db, "fetch_new_game_clips_cron")
}

pub fn recorded_at_least_hours_ago(db: &DbConn) -> Result<i64> {
    Ok(fetch(db, "recorded_at_least_hours_ago")?
        .parse()
        .map_err(|e| {
            anyhow!("Cannot parse recorded_at_least_hours_ago as i64: {e}")
        })?)
}

fn fetch(db: &DbConn, name: &str) -> Result<String> {
    let mut stmt =
        db.prepare("SELECT value FROM settings WHERE name = :name")?;
    let value = stmt
        .query_row(rusqlite::named_params! { ":name": name }, |row| {
            row.get(0)
        })?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_loads_defaults_ok() -> Result<()> {
        let mut db = DbConn::open_in_memory()?;
        db::up(&mut db)?;

        assert!(!fetch_new_game_clips_cron(&db).unwrap().is_empty());
        assert!(recorded_at_least_hours_ago(&db).is_ok());

        Ok(())
    }
}
