use std::{
    error::Error, io::BufReader, path::Path, process::{Command, Stdio}, str, sync::Arc
};

use regex::Regex;
use shared_child::SharedChild;
// mod read;
use crate::read;
// pub async fn get_video_duration(video_path: &str) -> Result<Option<String>, String> {
//     if !Path::exists(Path::new(video_path)) {
//         return Err(String::from("给定路径中不存在文件."));
//     }
//     let mut command = Command::new("ffprobe");
//     command
//         .args([
//             "-v",
//             "error",
//             "-show_entries",
//             "format=duration",
//             "-of",
//             "default=noprint_wrappers=1:nokey=1",
//             video_path,
//         ])
//         .stdout(Stdio::piped());

//     // 解析输出
//     let duration_str = str::from_utf8(&mut command.stdout)?;
//     // let duration: f64 = duration_str.trim().parse()?;

//     Ok(duration)
//     // match SharedChild::spawn(&mut command) {
//     //     Ok(child) => {
//     //         let cp = Arc::new(child);
//     //         let cp_clone1 = cp.clone();
//     //         let cp_clone2 = cp.clone();

//     //         let thread: tokio::task::JoinHandle<(u8, Option<String>)> = tokio::spawn(async move {
//     //             let mut duration: Option<String> = None;
//     //             if let Some(stderr) = cp_clone1.take_stderr() {
//     //                 let mut reader = BufReader::new(stderr);
//     //                 println!("stderr: {:?}", reader);
//     //                 loop {
//     //                     let mut buf: Vec<u8> = Vec::new();
//     //                     match read::read_line(&mut reader, &mut buf) {
//     //                         Ok(n) => {
//     //                             if n == 0 {
//     //                                 break;
//     //                             }
//     //                             let line = std::str::from_utf8(&buf).unwrap();
//     //                             println!("line: {}-------", line);
//     //                             let re = Regex::new("Duration: (?<duration>.*?),").unwrap();
//     //                             if let Some(cap) = re.captures(line) {
//     //                                 let matched_duration = &cap["duration"];
//     //                                 println!("matched_duration: {}", matched_duration);
//     //                                 // FFMPEG might return duration as N/A for files with invalid or unknown encoding
//     //                                 duration = Some(String::from(matched_duration));
//     //                                 println!("duration: {}", duration.as_ref().unwrap());
//     //                             };
//     //                         }
//     //                         Err(_) => {
//     //                             break;
//     //                         }
//     //                     };
//     //                 }
//     //             }
//     //             if cp_clone1.wait().is_ok() {
//     //                 return (0, duration);
//     //             }
//     //             (1, duration)
//     //         });

//     //         let result: Result<Option<String>, String> = match thread.await {
//     //             Ok((exit_status, duration)) => {
//     //                 if exit_status == 1 {
//     //                     Err(String::from("Video file is corrupted"))
//     //                 } else {
//     //                     Ok(duration)
//     //                 }
//     //             }
//     //             Err(err) => Err(err.to_string()),
//     //         };

//     //         // Cleanup
//     //         match cp_clone2.kill() {
//     //             Ok(_) => {
//     //                 println!("child process killed.");
//     //             }
//     //             Err(err) => {
//     //                 println!("child process could not be killed {}", err.to_string());
//     //             }
//     //         }
//     //         match result {
//     //             Ok(duration) => Ok(duration),
//     //             Err(err) => Err(err),
//     //         }
//     //     }
//     //     Err(err) => Err(err.to_string()),
//     // }
// }
// 获取输入视频的总时长
pub fn get_total_duration(input: &str) -> Result<f64, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input,
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let duration_str = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    let duration = duration_str.parse::<f64>()?;

    Ok(duration)
}