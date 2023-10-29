use crate::prelude::*;
use anyhow::Context;
use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub struct Conf {
    /// For example 0.0.0.0:8080
    pub rpc_addr: SocketAddr,
    pub sqlite_db_path: PathBuf,
    pub twitch_client_id: String,
    pub twitch_secret: String,
}

impl Conf {
    pub fn from_env() -> AnyResult<Self> {
        info!("Loading config from environment");

        let rpc_addr = env::var("RPC_ADDR").context("RPC_ADDR")?;
        debug!("RPC_ADDR: {rpc_addr}");

        let sqlite_db_path =
            env::var("SQLITE_DB_PATH").context("SQLITE_DB_PATH")?;
        debug!("SQLITE_DB_PATH: {sqlite_db_path}");

        let twitch_client_id =
            env::var("TWITCH_CLIENT_ID").context("TWITCH_CLIENT_ID")?;
        debug!(
            "TWITCH_CLIENT_ID: {}{}",
            &twitch_client_id[0..3],
            "*".repeat(twitch_client_id.len() - 3)
        );

        let twitch_secret =
            env::var("TWITCH_SECRET").context("TWITCH_SECRET")?;
        debug!(
            "TWITCH_SECRET: {}{}",
            &twitch_secret[0..3],
            "*".repeat(twitch_secret.len() - 3)
        );

        Ok(Self {
            rpc_addr: rpc_addr.parse()?,
            sqlite_db_path: sqlite_db_path.into(),
            twitch_client_id,
            twitch_secret,
        })
    }

    pub fn db_path(&self) -> &Path {
        &self.sqlite_db_path
    }

    pub async fn construct_twitch_client(&self) -> AnyResult<twitch::Client> {
        twitch::Client::new(
            self.twitch_client_id.as_str(),
            self.twitch_secret.as_str(),
        )
        .await
    }
}
