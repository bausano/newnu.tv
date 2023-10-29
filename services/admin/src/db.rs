pub mod game;

use crate::prelude::*;
use rusqlite_migration::{Migrations, M};

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
    Migrations::new(vec![M::up(include_str!("../migrations/0001.up.sql"))
        .down(include_str!("../migrations/0001.down.sql"))])
}
