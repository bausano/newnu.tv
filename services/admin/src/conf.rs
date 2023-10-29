use anyhow::Context;

use crate::prelude::*;
use std::{env, net::SocketAddr, path::PathBuf};

pub struct Conf {
    /// For example 0.0.0.0:8080
    pub http_addr: SocketAddr,
    pub sqlite_db_path: PathBuf,
    pub worker_addr: SocketAddr,
    pub twitch_client_id: String,
    pub twitch_secret: String,
}

impl Conf {
    pub fn from_env() -> AnyResult<Self> {
        info!("Loading config from environment");

        let http_addr = env::var("HTTP_ADDR").context("HTTP_ADDR")?;
        debug!("HTTP_ADDR: {http_addr}");

        let worker_addr = env::var("WORKER_ADDR").context("WORKER_ADDR")?;
        debug!("WORKER_ADDR: {worker_addr}");

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
            http_addr: http_addr.parse()?,
            worker_addr: worker_addr.parse()?,
            sqlite_db_path: sqlite_db_path.into(),
            twitch_client_id,
            twitch_secret,
        })
    }

    pub fn open_db(&self) -> AnyResult<DbConn> {
        DbConn::open(&self.sqlite_db_path).map_err(From::from)
    }

    /// The worker does not have to be running for this to succeed.
    pub async fn connect_lazy_worker_client(
        &self,
    ) -> AnyResult<worker::Client> {
        let dst = format!("http://{}", self.worker_addr);
        let conn = tonic::transport::Endpoint::new(dst)?.connect_lazy();
        Ok(worker::Client::new(conn))
    }

    pub async fn construct_twitch_client(&self) -> AnyResult<twitch::Client> {
        twitch::Client::new(
            self.twitch_client_id.as_str(),
            self.twitch_secret.as_str(),
        )
        .await
    }
}
