use crate::ffmepg_downloader::{download_ffmpeg, FFmpegPath};
use std::os::windows::process::CommandExt;

pub mod ffmepg_downloader;

/// Executes an FFmpeg command asynchronously and returns the output as a string.
///
/// # Arguments
///
/// - `command`: A string slice representing the FFmpeg command to be executed.
///
/// # Returns
///
/// - If the command is executed successfully, it returns the output as a `String`.
/// - If the command fails to execute, it returns an `std::io::Error`.
pub async fn execute_ffmpeg_command(command: &str) -> Result<String, std::io::Error> {
	let path: FFmpegPath = match ffmepg_downloader::get_existing_ffmpeg() {
		Some(p) => p,
		None => download_ffmpeg().await,
	};


	// Create a new Command and add arguments one by one
	let child = std::process::Command::new(path.ffmpeg)
		.raw_arg(&command)
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()?;

	let output = match child.wait_with_output() {
		Ok(output) => output,
		Err(e) => {
			eprintln!("Failed to start ffmpeg command: {}", e);
			return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to start ffmpeg command: {}", e)));
		}
	};

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();
		eprintln!("Failed to execute ffmpeg command: {}", command);
		eprintln!("stderr: {}", stderr);
		return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to execute ffmpeg command: {}", stderr)));
	}

	let mut tmp = String::from_utf8_lossy(&output.stdout).to_string();
	tmp += &String::from_utf8_lossy(&output.stderr).to_string();
	Ok(tmp)
}

/// Executes an FFprobe command asynchronously and returns the output as a string.
/// # Arguments
/// - `command`: A string slice representing the FFprobe command to be executed.
/// # Returns
/// - If the command is executed successfully, it returns the output as a `String`.
/// - If the command fails to execute, it returns an `std::io::Error`.
pub async fn execute_ffprobe_command(command: &str) -> Result<String, std::io::Error> {
	let path = match ffmepg_downloader::get_existing_ffmpeg() {
		Some(p) => p,
		None => download_ffmpeg().await,
	};

	let child = std::process::Command::new(path.ffprobe)
		.raw_arg(command)
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()?;

	let output = match child.wait_with_output() {
		Ok(output) => output,
		Err(e) => {
			eprintln!("Failed to start ffprobe command: {}", e);
			return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to start ffprobe command: {}", e)));
		}
	};

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();
		eprintln!("Failed to execute ffprobe command: {}", command);
		eprintln!("stderr: {}", stderr);
		return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to execute ffprobe command: {}", stderr)));
	}

	let mut tmp = String::from_utf8_lossy(&output.stdout).to_string();
	tmp += &String::from_utf8_lossy(&output.stderr).to_string();
	Ok(tmp)
}