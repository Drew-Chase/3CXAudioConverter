# Audio Converter CLI

This CLI application converts audio files to the 3CX audio specifications using `FFmpeg`. It automatically handles the download of `FFmpeg` if it is not already present.

## Features

- Converts audio files to 3CX specifications (`mono`, `8kHz`, `16-bit`).
- Automatically downloads and extracts `FFmpeg` if not present.
- Supports batch processing of files in a given directory.

## Prerequisites

- Rust (2021 edition)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Installation

Clone the repository and navigate to the project directory.

```sh
git clone <repository-url>
cd <repository-directory>
```

## Building

To build the project, run the following command in the project directory:

```sh
cargo build --release
```

The executable can be found in the `target/release` directory.

## Usage

After building the project, use the following command to run the application:

```sh
./target/release/<executable-name> <input-directory>
```

Replace `<executable-name>` with the actual name of the compiled executable and `<input-directory>` with the directory containing the audio files to be converted.

### Example

```sh
./target/release/phone_audio_converter.exe ./audio_files
```

## How It Works

1. The application checks if `FFmpeg` is available locally. If not, it downloads and extracts the latest version.
2. It reads all files in the specified input directory.
3. For each audio file, it generates the appropriate `FFmpeg` command to convert the file to the desired specifications.
4. It executes the `FFmpeg` commands concurrently using `tokio` for asynchronous processing.

## Dependencies

- `futures`: For handling asynchronous computations.
- `serde`: For serialization and deserialization.
- `serde_json`: For JSON parsing.
- `reqwest`: For making HTTP requests.
- `tokio`: For asynchronous runtime.
- `zip`: For handling ZIP files.

## Contributing

Feel free to open issues and submit pull requests for bug fixes and enhancements.

## License

This project is open-source and available under the MIT License.

## Acknowledgements

- This project uses the [FFmpeg](https://ffmpeg.org/) library for audio processing, which is licensed under the LGPL.

## Contact

For any inquiries or feedback, please contact [Your Name or Contact Information].

---

Thank you for using our audio converter CLI application!

