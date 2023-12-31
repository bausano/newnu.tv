pub mod clip;
pub mod game;
/// Stores various settings in db instead of constants so that they can be
/// changed via dashboard
pub mod setting;

use crate::prelude::*;
use rusqlite_migration::{Migrations, M};
use std::ffi::OsStr;

/// Opens and runs migrations.
pub fn open(path: impl AsRef<OsStr>) -> AnyResult<DbConn> {
    let mut db = DbConn::open(path.as_ref())?;
    rusqlite::vtab::array::load_module(&db)?;
    db::up(&mut db)?;

    Ok(db)
}

pub fn up(db: &mut DbConn) -> AnyResult<()> {
    info!("Running db UP migrations");

    migrations().to_latest(db)?;

    Ok(())
}

pub fn down(db: &mut DbConn) -> AnyResult<()> {
    info!("Running db DOWN migrations");

    migrations().to_version(db, 0)?;

    Ok(())
}

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("../migrations/0001.up.sql"))
            .down(include_str!("../migrations/0001.down.sql")),
        M::up(include_str!("../migrations/0002.up.sql"))
            .down(include_str!("../migrations/0002.down.sql")),
        M::up(include_str!("../migrations/0003.up.sql"))
            .down(include_str!("../migrations/0003.down.sql")),
    ])
}
