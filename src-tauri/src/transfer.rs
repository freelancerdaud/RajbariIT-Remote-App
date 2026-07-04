// ============================================
// Feature 5: File Transfer — transfer.rs
// ============================================
// Sends and receives files over TCP with chunked transfer,
// metadata headers, and progress tracking.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Default port for file transfer
const TRANSFER_PORT: u16 = 9096;

/// Chunk size for file transfer (64 KB)
const CHUNK_SIZE: usize = 65536;

/// Flag to track whether receive server is running
static RECEIVE_SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

/// File metadata sent before the actual file data
#[derive(Debug, Serialize, Deserialize)]
struct FileMetadata {
    filename: String,
    filesize: u64,
}

/// Send a file to a remote device over TCP.
#[tauri::command]
pub async fn send_file(
    device_ip: String,
    device_port: u16,
    file_path: String,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;

    let path = PathBuf::from(&file_path);

    // Validate file exists
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Read file metadata
    let metadata = std::fs::metadata(&path).map_err(|e| format!("Metadata error: {}", e))?;
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let filesize = metadata.len();

    // Connect to the target device
    let addr = format!("{}:{}", device_ip, device_port);
    let mut stream = tokio::net::TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    // Send file metadata as JSON + newline delimiter
    let meta = FileMetadata {
        filename: filename.clone(),
        filesize,
    };
    let meta_json = serde_json::to_string(&meta).map_err(|e| format!("JSON error: {}", e))?;
    let meta_bytes = format!("{}\n", meta_json);
    stream
        .write_all(meta_bytes.as_bytes())
        .await
        .map_err(|e| format!("Write metadata error: {}", e))?;

    // Read and send file data in chunks
    let file_data =
        std::fs::read(&path).map_err(|e| format!("Read file error: {}", e))?;
    let total = file_data.len();
    let mut sent = 0;

    for chunk in file_data.chunks(CHUNK_SIZE) {
        stream
            .write_all(chunk)
            .await
            .map_err(|e| format!("Write chunk error: {}", e))?;
        sent += chunk.len();
        let progress = (sent as f64 / total as f64 * 100.0) as u32;
        log::info!("Transfer progress: {}% ({}/{})", progress, sent, total);
    }

    stream
        .flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

    Ok(format!(
        "File '{}' sent successfully ({} bytes)",
        filename, filesize
    ))
}

/// Start a TCP server to receive incoming files.
#[tauri::command]
pub async fn start_receive_server(port: u16, save_dir: String) -> Result<String, String> {
    if RECEIVE_SERVER_RUNNING.load(Ordering::SeqCst) {
        return Ok("Receive server already running".to_string());
    }

    let actual_port = if port == 0 { TRANSFER_PORT } else { port };

    // Determine save directory
    let save_path = if save_dir.is_empty() {
        dirs_next::download_dir()
            .or_else(dirs_next::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(&save_dir)
    };

    RECEIVE_SERVER_RUNNING.store(true, Ordering::SeqCst);

    tokio::spawn(async move {
        let listener =
            match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", actual_port)).await {
                Ok(l) => l,
                Err(e) => {
                    log::error!("File receive server bind error: {}", e);
                    RECEIVE_SERVER_RUNNING.store(false, Ordering::SeqCst);
                    return;
                }
            };

        log::info!("File receive server listening on port {}", actual_port);

        while RECEIVE_SERVER_RUNNING.load(Ordering::SeqCst) {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("File transfer connection from: {}", addr);
                    let dir = save_path.clone();
                    tokio::spawn(handle_incoming_file(stream, dir));
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }
        }
    });

    Ok(format!("File receive server started on port {}", actual_port))
}

/// Handle an incoming file transfer connection
async fn handle_incoming_file(mut stream: tokio::net::TcpStream, save_dir: PathBuf) {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

    let mut reader = BufReader::new(&mut stream);

    // Read metadata line (JSON + newline)
    let mut meta_line = String::new();
    match reader.read_line(&mut meta_line).await {
        Ok(0) => {
            log::error!("Connection closed before metadata");
            return;
        }
        Ok(_) => {}
        Err(e) => {
            log::error!("Read metadata error: {}", e);
            return;
        }
    }

    let meta: FileMetadata = match serde_json::from_str(meta_line.trim()) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Parse metadata error: {}", e);
            return;
        }
    };

    log::info!(
        "Receiving file: {} ({} bytes)",
        meta.filename,
        meta.filesize
    );

    // Read file data
    let mut file_data = Vec::with_capacity(meta.filesize as usize);
    let mut remaining = meta.filesize as usize;
    let mut buf = vec![0u8; CHUNK_SIZE];

    while remaining > 0 {
        let to_read = std::cmp::min(remaining, CHUNK_SIZE);
        match reader.read_exact(&mut buf[..to_read]).await {
            Ok(_) => {
                file_data.extend_from_slice(&buf[..to_read]);
                remaining -= to_read;
                let progress =
                    ((meta.filesize as usize - remaining) as f64 / meta.filesize as f64 * 100.0)
                        as u32;
                log::info!("Receive progress: {}%", progress);
            }
            Err(e) => {
                log::error!("Read data error: {}", e);
                return;
            }
        }
    }

    // Save to disk
    let save_path = save_dir.join(&meta.filename);
    match std::fs::write(&save_path, &file_data) {
        Ok(_) => {
            log::info!("File saved: {:?}", save_path);
        }
        Err(e) => {
            log::error!("Save file error: {}", e);
        }
    }
}
