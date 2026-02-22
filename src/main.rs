mod core;
mod domain;
mod infra;
mod util;

use crate::infra::storage::mmap_handler::MmapStore;
use crate::infra::network::tcp_uds::{TcpIngestServer, UdsIngestServer, disk_worker};
use std::env;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let storage = MmapStore::new("active.log", 1024 * 1024 * 1024)
        .expect("Failed to initialize mmap storage");    

    let (tx, rx) = mpsc::channel(50_000);

    let batch_size = 1000;

    tokio::spawn(async move {
        disk_worker(rx, Box::new(storage)).await;
    });

    let transport = env::var("TRANSPORT").unwrap_or_else(|_| "tcp".to_string());

    match transport.as_str() {
        "uds" => {
            let server = UdsIngestServer::new("/tmp/axiom.sock", tx, batch_size).await;
            println!("Log Engine Online [UDS]. Protocol: Batch ACK ({}).", batch_size);
            server.run().await;
        }
        "tcp" | _ => {
            let server = TcpIngestServer::new("127.0.0.1:8080", tx, batch_size).await;
            println!("Log Engine Online [TCP]. Protocol: Batch ACK ({}).", batch_size);
            server.run().await;
        }
    }
}