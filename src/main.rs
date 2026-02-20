mod core;
mod domain;
mod infra;

use crate::infra::storage::mmap_handler::MmapStore;
use crate::infra::network::tcp_uds::TcpIngestServer;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let storage = MmapStore::new("active.log", 1024 * 1024 * 1024)
        .expect("Failed to initialize mmap storage");
    
    let shared_store = Arc::new(Mutex::new(storage));

    let server = TcpIngestServer::new("127.0.0.1:8080", shared_store).await;

    println!("Log Engine Online. Listening on 127.0.0.1:8080");

    server.run().await;
}