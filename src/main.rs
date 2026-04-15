mod audio_engine;

use std::env;
use std::process;

fn main() {
    // 1. get args from command line
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Incorrect usage :(");    // to print errors
        eprintln!("Usage: cargo run -- <path_to_audio_file>");  // instructions for dev, not production
        process::exit(1);
    }

    let file_path = &args[1];
    println!("Initializing audio engine...");
    println!("Playing: '{}'", file_path);

    // 2. call to audio engine to play the file
    if let Err(e) = audio_engine::play(file_path) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    println!("Finished playing '{}'", file_path);
}