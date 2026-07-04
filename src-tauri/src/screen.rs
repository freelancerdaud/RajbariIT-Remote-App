// ============================================
// Feature 2: Screen Streaming — screen.rs
// ============================================
// Desktop: Captures the screen using scrap crate, encodes frames to JPEG,
//          and streams them over TCP for remote viewing.
// Android: Stub functions (screen capture not supported).

#[cfg(not(target_os = "android"))]
use base64::Engine;
#[cfg(not(target_os = "android"))]
use base64::engine::general_purpose::STANDARD as BASE64;
#[cfg(not(target_os = "android"))]
use image::{ImageBuffer, RgbaImage, ImageEncoder};
#[cfg(not(target_os = "android"))]
use image::codecs::jpeg::JpegEncoder;
#[cfg(not(target_os = "android"))]
use scrap::{Capturer, Display};
#[cfg(not(target_os = "android"))]
use std::io::Cursor;

#[cfg(not(target_os = "android"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(target_os = "android"))]
use std::sync::Mutex;

/// Flag to track whether hosting is active
#[cfg(not(target_os = "android"))]
static IS_HOSTING: AtomicBool = AtomicBool::new(false);

/// Cached screen dimensions
#[cfg(not(target_os = "android"))]
static SCREEN_SIZE: Mutex<(u32, u32)> = Mutex::new((0, 0));

// ======================================================
// Desktop Implementation (Windows, macOS, Linux)
// ======================================================

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn start_screen_host(port: u16) -> Result<String, String> {
    if IS_HOSTING.load(Ordering::SeqCst) {
        return Ok("Already hosting".to_string());
    }

    IS_HOSTING.store(true, Ordering::SeqCst);

    let display = Display::primary().map_err(|e| format!("Display error: {}", e))?;
    let w = display.width() as u32;
    let h = display.height() as u32;

    let mut size = SCREEN_SIZE.lock().map_err(|e| e.to_string())?;
    *size = (w, h);

    let _port = port;
    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", _port)).await {
            Ok(l) => l,
            Err(e) => {
                log::error!("TCP listener error: {}", e);
                IS_HOSTING.store(false, Ordering::SeqCst);
                return;
            }
        };

        log::info!("Screen host listening on port {}", _port);

        while IS_HOSTING.load(Ordering::SeqCst) {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("Client connected: {}", addr);
                    tokio::spawn(handle_viewer_client(stream));
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }
        }
    });

    Ok(format!("Hosting started on port {}", port))
}

#[cfg(not(target_os = "android"))]
async fn handle_viewer_client(mut stream: tokio::net::TcpStream) {
    use tokio::io::AsyncWriteExt;

    while IS_HOSTING.load(Ordering::SeqCst) {
        match capture_frame_raw() {
            Ok(jpeg_data) => {
                let len = (jpeg_data.len() as u32).to_be_bytes();
                if stream.write_all(&len).await.is_err() {
                    break;
                }
                if stream.write_all(&jpeg_data).await.is_err() {
                    break;
                }
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(66)).await;
    }
}

#[cfg(not(target_os = "android"))]
fn capture_frame_raw() -> Result<Vec<u8>, String> {
    let display = Display::primary().map_err(|e| format!("Display error: {}", e))?;
    let w = display.width();
    let h = display.height();
    let mut capturer = Capturer::new(display).map_err(|e| format!("Capturer error: {}", e))?;

    let frame = loop {
        match capturer.frame() {
            Ok(f) => break f,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                return Err(format!("Capture error: {}", e));
            }
        }
    };

    let mut rgba_data = Vec::with_capacity((w * h * 4) as usize);
    for chunk in frame.chunks(4) {
        if chunk.len() >= 4 {
            rgba_data.push(chunk[2]); // R
            rgba_data.push(chunk[1]); // G
            rgba_data.push(chunk[0]); // B
            rgba_data.push(chunk[3]); // A
        }
    }

    let img: RgbaImage = ImageBuffer::from_raw(w as u32, h as u32, rgba_data)
        .ok_or("Failed to create image buffer")?;

    let mut jpeg_buf = Cursor::new(Vec::new());
    let encoder = JpegEncoder::new_with_quality(&mut jpeg_buf, 60);
    encoder
        .write_image(
            img.as_raw(),
            w as u32,
            h as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("JPEG encode error: {}", e))?;

    Ok(jpeg_buf.into_inner())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn stop_screen_host() -> Result<(), String> {
    IS_HOSTING.store(false, Ordering::SeqCst);
    Ok(())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn capture_frame() -> Result<String, String> {
    let jpeg_data = capture_frame_raw()?;
    Ok(BASE64.encode(&jpeg_data))
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn connect_to_host(ip: String, port: u16) -> Result<String, String> {
    use tokio::io::AsyncReadExt;

    let addr = format!("{}:{}", ip, port);
    let mut stream = tokio::net::TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    let frame_len = u32::from_be_bytes(len_buf) as usize;

    if frame_len > 10_000_000 {
        return Err("Frame too large, invalid stream".to_string());
    }

    let mut frame_data = vec![0u8; frame_len];
    stream
        .read_exact(&mut frame_data)
        .await
        .map_err(|e| format!("Read frame error: {}", e))?;

    Ok(format!("Connected to {} — received first frame ({} bytes)", addr, frame_len))
}

// ======================================================
// Android Stub Implementation
// ======================================================

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn start_screen_host(_port: u16) -> Result<String, String> {
    Err("Screen hosting is not supported on Android".to_string())
}

#[cfg(target_os = "android")]
#[tauri::command]
pub fn stop_screen_host() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "android")]
#[tauri::command]
pub fn capture_frame() -> Result<String, String> {
    Err("Screen capture is not supported on Android".to_string())
}

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn connect_to_host(ip: String, port: u16) -> Result<String, String> {
    use tokio::io::AsyncReadExt;

    let addr = format!("{}:{}", ip, port);
    let mut stream = tokio::net::TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    let frame_len = u32::from_be_bytes(len_buf) as usize;

    if frame_len > 10_000_000 {
        return Err("Frame too large".to_string());
    }

    let mut frame_data = vec![0u8; frame_len];
    stream
        .read_exact(&mut frame_data)
        .await
        .map_err(|e| format!("Read frame error: {}", e))?;

    Ok(format!("Connected to {} — received first frame ({} bytes)", addr, frame_len))
}
