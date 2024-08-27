use futures::future::join_all;
use std::{fs, io};

#[tokio::main]
async fn main() {
	let ffmpeg_path = match ffmpeg::ffmepg_downloader::get_existing_ffmpeg() {
		Some(path) => path,
		None => {
			ffmpeg::ffmepg_downloader::download_ffmpeg().await
		}
	};

	println!("Using ffmpeg at: {:?}", fs::canonicalize(ffmpeg_path.ffmpeg));
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
	let commands: Vec<String> = files.iter().filter_map(|file| {
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
		Some(command)
	}).collect();

	// Now we create tasks, using the Strings from the commands Vec
	let tasks: Vec<_> = commands.iter().map(|c| {
		println!("Executing command: {}", c);
		ffmpeg::execute_ffmpeg_command(c)
	}).collect();

	join_all(tasks).await;
}
