use crossbeam_channel::{Receiver, Sender};
use regex::Regex;
use shared_child::SharedChild;
use std::{
    error::Error,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::Arc,
};
use tokio::net::windows::named_pipe::PipeEnd;
mod cpu;
mod read;
mod utils;

#[tokio::main]
async fn main() {
    let gpu_types = cpu::detect_gpu();
    println!("GPU Types: {:#?}", gpu_types);
    for gpu in gpu_types {
        match gpu {
            Some(cpu::GpuType::Nvidia(name)) => println!("Nvidia GPU: {}", name),
            Some(cpu::GpuType::Amd(name)) => println!("Amd GPU: {}", name),
            Some(cpu::GpuType::Intel(name)) => println!("Intel GPU: {}", name),
            None => println!("No GPU detected"),
        }
    }
    // let input = "input.mp4";
    // let output = "output12.mp4";
    // let getVideoDuration = utils::get_total_duration(input);
    // println!("getVideoDuration:{:?}", getVideoDuration);
    // if let Err(err) = compress_video_with_progress(input, output, 50, getVideoDuration).await {
    //     eprintln!("Error: {}", err);
    // }
}
async fn compress_video_with_progress(
    input: &str,
    output: &str,
    quality: u16,
    getVideoDuration: Result<f64, Box<dyn Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // let quality = quality.clamp(0, 100);
    let codec = "h264_nvenc";
    // Start FFmpeg process
    let max_crf: u16 = 36;
    let min_crf: u16 = 24; // Lower the CRF, higher the quality
    let default_crf: u16 = 28;
    let compression_quality = if (0..=100).contains(&quality) {
        let diff = (max_crf - min_crf) - ((max_crf - min_crf) * quality) / 100;
        format!("{}", min_crf + diff)
    } else {
        format!("{default_crf}")
    };
    let compression_quality_str = compression_quality.as_str();
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&[
        "-i",
        input,
        "-hide_banner",
        "-progress",
        "-",
        "-nostats",
        "-loglevel",
        "error",
        "-pix_fmt",
        "yuv420p",
        "-c:v",
        codec,
        // "-b:v",
        // "0",
        "-movflags",
        "+faststart",
        "-preset",
        "medium",
        "-qp",
        "0",
        "-crf",
        compression_quality_str,
        "-vf",
        "pad=ceil(iw/2)*2:ceil(ih/2)*2",
        output,
    ])
    .stdout(Stdio::piped());
    // .stderr(Stdio::piped());
    match SharedChild::spawn(&mut cmd) {
        Ok(child) => {
            // let cp = Arc::new(child);
            // let cp_clone1 = cp.clone();
            // let mut s: String = String::new();
            let (tx, rx): (Sender<String>, Receiver<String>) = crossbeam_channel::unbounded();
            let thread: tokio::task::JoinHandle<u8> = tokio::spawn(async move {
                if let Some(stdout) = child.take_stdout() {
                    let mut reader = BufReader::new(stdout);

                    loop {
                        let mut buf: Vec<u8> = Vec::new();
                        match read::read_line(&mut reader, &mut buf) {
                            Ok(n) => {
                                if n == 0 {
                                    break;
                                }
                                if let Ok(output) = std::str::from_utf8(&buf) {
                                    // log::debug!("[ffmpeg] stdout: {:?}", output);
                                    // println!("{}---------", output);
                                    let re = Regex::new("out_time=(?<out_time>.*?)\\n").unwrap();
                                    if let Some(cap) = re.captures(output) {
                                        let out_time = &cap["out_time"];
                                        if !out_time.is_empty() {
                                            tx.try_send(String::from(out_time)).ok();
                                        }
                                    }
                                }
                                // println!("=====================================");
                                // println!("当前数据:{}", n);
                            }
                            Err(_) => {
                                eprintln!("Error: Failed to read progress from ffmpeg");
                            }
                        }
                    }
                }

                if child.wait().is_ok() {
                    0
                } else {
                    1
                }
            });
            let video_duration = match getVideoDuration {
                Ok(duration) => duration,
                Err(e) => {
                    eprintln!("Failed to get video duration: {}", e);
                    return Err(e);
                }
            };
            let progress_handle = tokio::spawn(async move {
                while let Ok(current_duration) = rx.recv() {
                    println!("current_duration:{:?}", current_duration);
                    let current_time = parse_time_to_seconds(&current_duration);
                    if current_time.is_some() {
                        let progress = (current_time.unwrap() / video_duration) * 100.0;
                        println!("Progress: {:.2}%", progress);
                    }
                }
            });
            let message: String = match thread.await {
                Ok(exit_status) => {
                    if exit_status == 1 {
                        String::from("Video is corrupted.")
                    } else {
                        String::from("")
                    }
                }
                Err(err) => err.to_string(),
            };
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    Ok(())
}

fn parse_time_to_seconds(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let hours = parts[0].parse::<f64>().ok()?;
    let minutes = parts[1].parse::<f64>().ok()?;
    let seconds = parts[2].parse::<f64>().ok()?;

    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}
