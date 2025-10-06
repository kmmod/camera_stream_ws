use anyhow::Result;
use colored::*;
use futures_util::{SinkExt, StreamExt};
use opencv::core::{Mat, MatTraitConst, Size, Vector, VectorToVec};
use opencv::highgui::wait_key;
use opencv::imgproc::INTER_LINEAR;
use opencv::videoio::{VideoCaptureTrait, VideoCaptureTraitConst};
use opencv::{highgui, imgcodecs, imgproc, videoio};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load_config();

    let listener = TcpListener::bind(&config.url).await?;
    println!(
        "{} {}",
        "Websocket server started on ws:://".green().bold(),
        config.url.yellow()
    );

    // Create broadcast channel for frames
    let (tx, _rx) = broadcast::channel::<Vec<u8>>(10);

    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    let mut cap = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    if !cap.is_opened()? {
        panic!("{}", "Unable to open default camera!".red());
    }

    let mut frame = Mat::default();

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(33)); // ~30 fps

    println!(
        "{} {}",
        "Starting capture loop.".green().bold(),
        "ESC to close window frame".red()
    );

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Frame processing
                cap.read(&mut frame)?;

                let src_width = frame.cols();
                let src_height = frame.rows();
                let target_height = config.frame_height;

                // Calculate width to maintain aspect ratio
                let aspect_ratio = src_width as f64 / src_height as f64;
                let target_width = (target_height as f64 * aspect_ratio) as i32;

                // Resize
                let mut resized = Mat::default();
                imgproc::resize(
                    &frame,
                    &mut resized,
                    Size::new(target_width, target_height),
                    0.0,
                    0.0,
                    INTER_LINEAR
                )?;

                // Encode frame as JPEG
                let mut buf = Vector::new();
                let params = Vector::new(); // Can add quality params here
                imgcodecs::imencode(".jpg", &resized, &mut buf, &params)?;
                let jpeg_data = buf.to_vec();

                // Broadcast frame to all connected clients (ignore if no receivers)
                let _ = tx.send(jpeg_data);

                highgui::imshow(window, &resized)?;

                if wait_key(10)? == 27 {
                    break;
                }
            }

            Ok((stream, _)) = listener.accept() => {
                let rx = tx.subscribe();
                tokio::spawn(handle_connection(stream, rx));
            }
        }
    }

    highgui::destroy_all_windows()?;
    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    mut frame_rx: broadcast::Receiver<Vec<u8>>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    println!("{}", "WebSocket connection established".green().bold());

    // Spawn a task to receive frames and send them
    let send_task = tokio::spawn(async move {
        while let Ok(frame_data) = frame_rx.recv().await {
            if write.send(Message::Binary(frame_data)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (optional)
    while let Some(msg) = read.next().await {
        if let Ok(msg) = msg {
            if msg.is_text() {
                println!(
                    "{} {}",
                    "Received message: ".green(),
                    msg.to_text().unwrap_or("").cyan()
                );
            } else if msg.is_close() {
                break;
            }
        } else {
            break;
        }
    }

    send_task.abort();

    Ok(())
}
