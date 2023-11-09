use crate::{prelude::*, rpc, RpcWorker};
use rpc::worker_server::Worker;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Worker for RpcWorker {
    async fn download_clip(
        &self,
        _request: Request<rpc::DownloadClipRequest>,
    ) -> StdResult<Response<()>, Status> {
        debug!("Download clip");

        // TODO

        Ok(Response::new(()))
    }
}
