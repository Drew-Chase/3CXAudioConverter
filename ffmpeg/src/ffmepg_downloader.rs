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
	let ffprobe_url = os_version.ffprobe;

	std::fs::create_dir_all("ffmpeg").unwrap();

	let ffmpeg_download_thread =
		std::thread::spawn(|| download_file(ffmpeg_url, "ffmpeg/ffmpeg.zip"));
	let ffprobe_download_thread =
		std::thread::spawn(|| download_file(ffprobe_url, "ffmpeg/ffprobe.zip"));

	let ffmpeg_zip: String = match ffmpeg_download_thread.join() {
		Ok(r) => match r.await {
			Ok(file) => file,
			_ => panic!("Failed to download ffmpeg archive"),
		},
		_ => panic!("Failed to join ffmpeg download thread"),
	};
	let ffprobe_zip: String = match ffprobe_download_thread.join() {
		Ok(r) => match r.await {
			Ok(file) => file,
			_ => panic!("Failed to download ffprobe archive"),
		},
		_ => panic!("Failed to join ffprobe download thread"),
	};

	let ffmpeg_path = match extract_zip(&ffmpeg_zip, "ffmpeg") {
		Ok(Some(path)) => path,
		_ => panic!("Failed to extract ffmpeg"),
	};
	let ffprobe_path = match extract_zip(&ffprobe_zip, "ffmpeg") {
		Ok(Some(path)) => path,
		_ => panic!("Failed to extract ffprobe"),
	};

	std::fs::remove_file(&ffmpeg_zip).unwrap();
	std::fs::remove_file(&ffprobe_zip).unwrap();

	println!("Downloaded ffmpeg version {}", versions.version);

	FFmpegPath {
		ffmpeg: ffmpeg_path,
		ffprobe: ffprobe_path,
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
	let (ffmpeg_path, ffprobe_path) = if os == "windows" {
		("ffmpeg/ffmpeg.exe", "ffmpeg/ffprobe.exe")
	} else {
		("ffmpeg/ffmpeg", "ffmpeg/ffprobe")
	};

	if !Path::new(ffmpeg_path).exists() || !Path::new(ffprobe_path).exists() {
		return None;
	}

	Some(FFmpegPath {
		ffmpeg: ffmpeg_path.to_string(),
		ffprobe:ffprobe_path.to_string(),
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
	pub ffprobe: String,
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
