use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("worker_descriptor.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/worker.proto"], &["proto"])
        .unwrap();
}
