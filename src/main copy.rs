use crossbeam_channel::{Receiver, Sender};
use regex::Regex;
use shared_child::SharedChild;
use std::f32::consts::E;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
/// Compresses a video using FFmpeg and prints progress to the console.
async fn compress_video_with_progress(
    input: &str,
    output: &str,
    quality: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate quality
    let quality = quality.clamp(0, 100);
    let crf = calculate_crf(quality);

    // Start FFmpeg process
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&[
        "-i",
        input,
        "-c:v",
        "libx264",
        "-crf",
        &crf,
        "-preset",
        "medium",
        "-progress",
        "-", // Output progress to stderr
        "-loglevel",
        "error",
        output,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    match SharedChild::spawn(&mut cmd) {
        Ok(child) => {
            let cp = Arc::new(child);
            let cp_clone1 = cp.clone();
            let (tx, rx): (Sender<String>, Receiver<String>) = crossbeam_channel::unbounded();
            let thread: tokio::task::JoinHandle<u8> = tokio::spawn(async move {
                if let Some(stdout) = cp_clone1.take_stdout() {
                    let mut reader = BufReader::new(stdout);
                    let mut buf = Vec::new();
                    loop {
                        buf.clear();
                        println!("[ffmpeg] Progress:11");

                        match reader.read_until(b'\n', &mut buf) {
                            Ok(n) => {
                                println!("{}", String::from_utf8_lossy(&buf));
                                if n == 0 {
                                    break;
                                }
                                if let Ok(output) = std::str::from_utf8(&buf) {
                                    // log::debug!("[ffmpeg] stdout: {:?}", output);
                                    let re = Regex::new("out_time=(?<out_time>.*?)\n").unwrap();
                                    if let Some(cap) = re.captures(output) {
                                        let out_time = &cap["out_time"];
                                        if !out_time.is_empty() {
                                            tx.try_send(String::from(out_time)).ok();
                                        }
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }

                if cp_clone1.wait().is_ok() {
                    0
                } else {
                    1
                }
            });

            // let result = thread.await?;

            // 例如，您可以尝试等待子进程完成
            // match cp_clone1.wait() {
            //     Ok(status) => {
            //         println!("FFmpeg process exited with status: {}", status);
            //     }
            //     Err(err) => {
            //         eprintln!("Failed to wait for FFmpeg process: {}", err);
            //     }
            // }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    // cmd.wait()?;
    Ok(())
}

/// Convert quality (0-100) to CRF value (hybrid method).
fn calculate_crf(quality: u16) -> String {
    let min_crf = 18; // High quality
    let max_crf = 28; // Lower quality
    let crf = max_crf - ((max_crf - min_crf) * quality) / 100;
    crf.to_string()
}

/// Convert time string (HH:MM:SS.mmm) to seconds.
fn parse_time(time_str: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = time_str.split([':', '.']).collect();
    let h = parts[0].parse::<u64>()?;
    let m = parts[1].parse::<u64>()?;
    let s = parts[2].parse::<u64>()?;
    let ms = if parts.len() > 3 {
        parts[3].parse::<u64>()?
    } else {
        0
    };
    Ok(h as f64 * 3600.0 + m as f64 * 60.0 + s as f64 + ms as f64 / 1000.0)
}
#[tokio::main]
async fn main() {
    let input = "input.mp4";
    let output = "output.mp4";

    if let Err(err) = compress_video_with_progress(input, output, 50).await {
        eprintln!("Error: {}", err);
    }
}
