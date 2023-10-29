pub mod rpc {
    tonic::include_proto!("worker");
}

pub type Client = rpc::worker_client::WorkerClient<tonic::transport::Channel>;
