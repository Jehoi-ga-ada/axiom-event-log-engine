use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::storage::LogStore;

pub struct TcpIngestServer {
    listener: TcpListener,
    store: Arc<Mutex<dyn LogStore>>,
}

impl TcpIngestServer {
    pub async fn new(addr: &str, store: Arc<Mutex<dyn LogStore>>) -> Self {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind TCP listener");
        Self { listener, store }
    }

    pub async fn run(&self) {
        loop {
            let (mut socket, _) = match self.listener.accept().await {
                Ok(s) => s,
                Err(_) => continue,
            };

            let store = self.store.clone();

            tokio::spawn(async move {
                let mut len_buf = [0u8; 4];

                loop {
                    if socket.read_exact(&mut len_buf).await.is_err() {
                        break; // Connection closed
                    }
                    let len = u32::from_be_bytes(len_buf) as usize;

                    let mut data = vec![0u8; len];
                    if socket.read_exact(&mut data).await.is_err() {
                        break;
                    }

                    let mut lock = store.lock().await;
                    if let Err(e) = lock.append(&data) {
                        eprintln!("Failed to append to log: {}", e);
                        break;
                    }
                    
                    // Slice 1 Optimization: We could send an ACK here 
                    // to notify Go that the data is durable.
                }
            });
        }
    }
}