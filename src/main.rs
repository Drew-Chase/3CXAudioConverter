use futures::future::join_all;
use std::os::windows::process::CommandExt;
use std::{fs, io};

#[tokio::main]
async fn main() {
	let ffmpeg_path = match get_existing_ffmpeg() {
		Some(path) => path,
		None => {
			download_ffmpeg().await
		}
	};

	println!("Using ffmpeg at: {}", fs::canonicalize(ffmpeg_path.ffmpeg).unwrap().to_str().unwrap()[4..].to_string());
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		eprintln!("Usage: {} <input>", args[0]);
		std::process::exit(1);
	}

	let input_dir = &args[1];
	let input_dir = fs::canonicalize(input_dir).unwrap();

	let output_dir = input_dir.join("output");
	fs::create_dir_all(&output_dir).unwrap();


	let mut files = fs::read_dir(&input_dir).unwrap()
	                                        .map(|res| res.map(|e| e.path()))
	                                        .collect::<Result<Vec<_>, io::Error>>()
	                                        .unwrap();

	files.sort();

	// We create a Vec to store the ffmpeg commands as String
	let commands: Vec<(String, String)> = files.iter().filter_map(|file| {
		if !file.is_file() {
			return None;
		}
		let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
		let output_file = output_dir.join(&file_name);
		let output_file = output_file.with_extension("wav");
		let output_file_str = output_file.to_str().unwrap().to_string();
		let file_str = file.to_str().unwrap().to_string();
		let file_str = file_str[4..].to_string();
		let output_file_str = output_file_str[4..].to_string();
		let command = format!("-hide_banner -hwaccel auto -y -i \"{}\" -ac 1 -ar 8000 -sample_fmt s16 \"{}\"", file_str, output_file_str);
		Some((file_name, command))
	}).collect();

	// Now we create tasks, using the Strings from the commands Vec
	let tasks: Vec<_> = commands.iter().map(|(filename, command)| {
		println!("Processing File: {}", filename);
		execute_ffmpeg_command(command)
	}).collect();

	join_all(tasks).await;
}


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
pub async fn execute_ffmpeg_command(command: &str) -> Result<String, io::Error> {
	let path: FFmpegPath = match get_existing_ffmpeg() {
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


use serde::{Deserialize, Serialize};
use std::path::Path;
use zip::read::ZipArchive;

/// Downloads FFmpeg binary and extracts it to the current directory.
pub async fn download_ffmpeg() -> FFmpegPath {
	println!("Downloading ffmpeg for {}", std::env::consts::OS);
	let versions = match get_ffmpeg_version().await {
		Ok(versions) => versions,
		Err(error) => panic!("Failed to get ffmpeg versions: {:?}", error),
	};
	let os_versions = versions.bin;
	let os_version = match std::env::consts::OS {
		"windows" => os_versions.windows_64,
		"linux" => os_versions.linux_64,
		"macos" => os_versions.osx_64,
		_ => os_versions.linux_64,
	};
	let ffmpeg_url = os_version.ffmpeg;

	std::fs::create_dir_all("ffmpeg").unwrap();


	let exe_path = std::env::current_exe().unwrap();
	let exe_dir = exe_path.parent().unwrap();

	let ffmpeg_dir = exe_dir.join("ffmpeg");
	std::fs::create_dir_all(&ffmpeg_dir).unwrap();

	let ffmpeg_zip = ffmpeg_dir.join("ffmpeg.zip");

	let ffmpeg_dir = ffmpeg_dir.to_str().unwrap();


	let ffmpeg_zip: String = match download_file(ffmpeg_url, ffmpeg_zip.to_str().unwrap()).await {
		Ok(r) => r,
		_ => panic!("Failed to join ffmpeg download thread"),
	};

	let ffmpeg_path = match extract_zip(&ffmpeg_zip, ffmpeg_dir) {
		Ok(Some(path)) => path,
		_ => panic!("Failed to extract ffmpeg"),
	};

	std::fs::remove_file(&ffmpeg_zip).unwrap();

	println!("Downloaded ffmpeg version {}", versions.version);

	FFmpegPath {
		ffmpeg: ffmpeg_path,
	}
}

/// Downloads a file from the given URL and saves it with the specified name.
///
/// # Arguments
///
/// * `url` - The URL of the file to download.
/// * `name` - The name to use for the downloaded file.
async fn download_file(url: String, name: &str) -> Result<String, Box<dyn std::error::Error>> {
	println!("Downloading {} to {}", url, name);
	let response = reqwest::get(&url).await?;
	let mut file = match std::fs::File::create(name) {
		Ok(file) => file,
		Err(error) => return Err(Box::new(error)),
	};
	match std::io::copy(&mut response.bytes().await?.as_ref(), &mut file) {
		Ok(_) => Ok(name.to_string()),
		Err(error) => Err(Box::new(error)),
	}
}

/// Retrieves the paths to the existing FFmpeg and FFprobe executables based on the current operating system.
///
/// # Arguments
///
/// None
///
/// # Returns
///
/// - `Some(FFmpegPath)`: If both FFmpeg and FFprobe executables exist at the expected paths.
/// - `None`: If either FFmpeg or FFprobe executable is missing.
///
/// If the current operating system is `windows`, the expected paths for FFmpeg executable is `"ffmpeg/ffmpeg.exe"`
/// and for FFprobe executable is `"ffmpeg/ffprobe.exe"`. If either of these executables does not exist at the expected
/// paths, `None` is returned. Otherwise, a `Some(FFmpegPath)` is returned with the paths to the existing FFmpeg and
/// FFprobe executables.
pub fn get_existing_ffmpeg() -> Option<FFmpegPath> {
	let os = std::env::consts::OS;
	let ffmpeg_path = if os == "windows" {
		"ffmpeg/ffmpeg.exe"
	} else {
		"ffmpeg/ffmpeg"
	};

	let exe_path = std::env::current_exe().unwrap();
	let binding = exe_path.parent()?.join(ffmpeg_path);
	let ffmpeg_path = binding.to_str()?;

	if !Path::new(ffmpeg_path).exists() {
		return None;
	}

	Some(FFmpegPath {
		ffmpeg: ffmpeg_path.to_string(),
	})
}

/// Extracts a zip file to a specified output path.
///
/// # Arguments
///
/// * `zip_path` - The path to the zip file.
/// * `output_path` - The path where the files will be extracted.
///
/// # Returns
///
/// A `Result` containing either `Some(String)` if extraction was successful and the name of the extracted file matches the specified conditions,
/// or `None` if no file matching the conditions was found.
///
/// # Errors
///
/// Returns an `std::io::Error` if there was an error opening or reading the zip file, creating the output directory, or writing the extracted file to disk.
fn extract_zip(zip_path: &str, output_path: &str) -> Result<Option<String>, std::io::Error> {
	println!("Extracting {} to {}", zip_path, output_path);
	let file = std::fs::File::open(zip_path)?;
	let mut archive = ZipArchive::new(file)?;

	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		let filename = file.name().to_string();

		if filename.contains("ffprobe") || filename.contains("ffmpeg") {
			let outpath = Path::new(output_path).join(file.enclosed_name().unwrap());

			if (&*filename).ends_with('/') {
				std::fs::create_dir_all(&outpath)?;
			} else {
				if let Some(p) = outpath.parent() {
					if !p.exists() {
						std::fs::create_dir_all(&p)?;
					}
				}
				let mut outfile = std::fs::File::create(&outpath)?;
				println!("extracting: {}", outpath.display());
				std::io::copy(&mut file, &mut outfile)?;
			}
			return Ok(Some(filename));
		}
	}

	Ok(None)
}

/// Retrieves the latest version of FFmpeg from the ffbinaries API.
async fn get_ffmpeg_version() -> Result<FFmpegVersions, Box<dyn std::error::Error>> {
	println!("Getting latest ffmpeg version");
	let url = "https://ffbinaries.com/api/v1/version/latest";

	match reqwest::get(url).await {
		Err(error) => Err(Box::new(error)),
		Ok(response) => match response.text().await {
			Err(error) => Err(Box::new(error)),
			Ok(body) => match serde_json::from_str(&body) {
				Err(error) => Err(Box::new(error)),
				Ok(versions) => Ok(versions),
			},
		},
	}
}

/// Represents the paths to the FFmpeg and FFprobe executables.
pub struct FFmpegPath {
	pub ffmpeg: String,
}

/// Struct representing the FFmpeg version information for both ffmpeg and ffprobe.
#[derive(Serialize, Deserialize)]
struct FFmpegOSVersion {
	pub ffmpeg: String,
	pub ffprobe: String,
}

#[derive(Serialize, Deserialize)]
struct FFmpegOSVersions {
	#[serde(rename = "windows-64")]
	pub windows_64: FFmpegOSVersion,
	#[serde(rename = "linux-32")]
	pub linux_32: FFmpegOSVersion,
	#[serde(rename = "linux-64")]
	pub linux_64: FFmpegOSVersion,
	#[serde(rename = "linux-armhf")]
	pub linux_armhf: FFmpegOSVersion,
	#[serde(rename = "linux-armel")]
	pub linux_armel: FFmpegOSVersion,
	#[serde(rename = "linux-arm64")]
	pub linux_arm64: FFmpegOSVersion,
	#[serde(rename = "osx-64")]
	pub osx_64: FFmpegOSVersion,
}

/// Represents the versions of FFmpeg.
#[derive(Serialize, Deserialize)]
struct FFmpegVersions {
	pub version: String,
	pub permalink: String,
	pub bin: FFmpegOSVersions,
}
