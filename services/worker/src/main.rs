mod conf;
mod error;
mod g;
mod prelude;
mod service;

use crate::prelude::*;
use rpc::worker_server::WorkerServer;
use std::sync::Arc;
use tonic::transport::Server;

mod rpc {
    tonic::include_proto!("worker");
}

struct RpcWorker {
    g: AppState,
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenvy::from_filename(".env.worker").ok();
    pretty_env_logger::try_init_timed().ok();

    info!("worker starting");

    let conf = Conf::from_env()?;

    let g = AppState {
        conf: Arc::new(conf),
    };

    let addr = g.conf.rpc_addr;
    let server = RpcWorker { g };

    Server::builder()
        .add_service(WorkerServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
