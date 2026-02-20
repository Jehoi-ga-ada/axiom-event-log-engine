use tokio::net::{TcpListener, UnixListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::core::storage::LogStore;
use std::fs;
use tokio::sync::{mpsc, oneshot};
use bytes::{BytesMut, Bytes};

pub struct LogBatch {
    pub messages: Vec<Bytes>,
    pub ack_tx: oneshot::Sender<()>,
}

pub async fn disk_worker(mut receiver: mpsc::Receiver<LogBatch>, mut store: Box<dyn LogStore>) {
    while let Some(batch) = receiver.recv().await {
        for data in &batch.messages {
            let _ = store.append(data);
        }
        let _ = batch.ack_tx.send(());
    }
}

pub struct TcpIngestServer {
    listener: TcpListener,
    tx: mpsc::Sender<LogBatch>,
    batch_size: usize,
}

impl TcpIngestServer {
    pub async fn new(addr: &str, tx: mpsc::Sender<LogBatch>, batch_size: usize) -> Self {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind TCP");
        Self { listener, tx, batch_size }
    }

    pub async fn run(&self) {
        loop {
            let (socket, _) = match self.listener.accept().await {
                Ok(s) => s,
                Err(_) => continue,
            };
            tokio::spawn(handle_connection(socket, self.tx.clone(), self.batch_size));
        }
    }
}
pub struct UdsIngestServer {
    listener: UnixListener,
    tx: mpsc::Sender<LogBatch>,
    batch_size: usize,
}

impl UdsIngestServer {
    pub async fn new(path: &str, tx: mpsc::Sender<LogBatch>, batch_size: usize) -> Self {
        let _ = fs::remove_file(path);
        let listener = UnixListener::bind(path).expect("Failed to bind UDS");
        Self { listener, tx, batch_size }
    }

    pub async fn run(&self) {
        loop {
            let (socket, _) = match self.listener.accept().await {
                Ok(s) => s,
                Err(_) => continue,
            };
            tokio::spawn(handle_connection(socket, self.tx.clone(), self.batch_size));
        }
    }
}

async fn handle_connection<S>(mut socket: S, tx: mpsc::Sender<LogBatch>, batch_size: usize) 
where S: AsyncReadExt + AsyncWriteExt + Unpin 
{
    let mut len_buf = [0u8; 4];
    let mut buffer = BytesMut::with_capacity(65536); // Pre-allocated slab
    let mut current_batch = Vec::with_capacity(batch_size);

    loop {
        let read_result = tokio::time::timeout(
            std::time::Duration::from_millis(10), 
            socket.read_exact(&mut len_buf)
        ).await;

        match read_result {
            Ok(Ok(_)) => {
                let len = u32::from_be_bytes(len_buf) as usize;
                
                // Resize without losing capacity
                buffer.resize(len, 0);
                if socket.read_exact(&mut buffer).await.is_err() { break; }

                // Zero-copy split
                let data = buffer.split().freeze();
                current_batch.push(data);

                if current_batch.len() >= batch_size {
                    if dispatch_batch(&tx, &mut current_batch, &mut socket).await.is_err() { break; }
                }
            }
            Ok(Err(_)) => break,
            Err(_) => {
                // Handle partial batch on timeout (FlushInterval)
                if !current_batch.is_empty() {
                    if dispatch_batch(&tx, &mut current_batch, &mut socket).await.is_err() { break; }
                }
            }
        }
    }
}

async fn dispatch_batch<S>(
    tx: &mpsc::Sender<LogBatch>, 
    batch_vec: &mut Vec<Bytes>, 
    socket: &mut S
) -> Result<(), Box<dyn std::error::Error>> 
where S: AsyncWriteExt + Unpin {
    let (ack_tx, ack_rx) = oneshot::channel();
    
    // Efficiently move the vector out and replace it with a fresh pre-allocated one
    let messages = std::mem::replace(batch_vec, Vec::with_capacity(batch_vec.capacity()));
    
    tx.send(LogBatch { messages, ack_tx }).await?;
    ack_rx.await?;
    socket.write_all(&[1]).await?;
    Ok(())
}