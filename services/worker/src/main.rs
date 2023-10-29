mod conf;
mod db;
mod error;
mod g;
mod job;
mod prelude;
mod service;

use crate::prelude::*;
use rpc::worker_server::WorkerServer;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

mod rpc {
    tonic::include_proto!("worker");
}

struct RpcWorker {
    g: AppState,
    _jobs: job::Jobs,
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenvy::from_filename(".env.worker").ok();
    pretty_env_logger::try_init_timed().ok();

    info!("worker starting");

    let conf = Conf::from_env()?;
    let twitch = Arc::new(conf.construct_twitch_client().await?);
    let db = {
        let mut db = rusqlite::Connection::open(conf.db_path())?;
        db::up(&mut db)?;
        Arc::new(Mutex::new(db))
    };

    let g = AppState {
        db: Arc::clone(&db),
        conf: Arc::new(conf),
        twitch: Arc::clone(&twitch),
    };

    let jobs = job::schedule_all(g.clone()).await?;

    let addr = g.conf.rpc_addr;
    let server = RpcWorker { g, _jobs: jobs };

    Server::builder()
        .add_service(WorkerServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
