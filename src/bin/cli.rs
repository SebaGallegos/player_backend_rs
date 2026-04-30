use player_backend_rs::audio_engine::{self, AudioCommand};
use player_backend_rs::metadata;

use std::env;
use std::io;
use std::process;
use std::sync::mpsc;    // to create communication channel between main thread and audio engine thread

fn main() {
    // 1. get argument: song path
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --release -- <song_path>");
        process::exit(1);
    }

    let song_path = args[1].clone();  // clone to get ownership of the string

    // new feature: extract and display metadata
    let metadata = metadata::extract_metadata(&song_path);

    println!("\nNow playing: {} - {} ({} - {})\n", metadata.artist, metadata.title, metadata.album, metadata.year);

    // 2. create communication channel
    // tx: this main thread's sender, used to send commands to the audio engine thread in FIFO order
    // rx: the audio engine thread's receiver, used to receive commands from the main thread
    let (tx, rx) = mpsc::channel();

    // 3. initialize the audio engine in a separate thread
    println!("Initializing audio engine...");
    audio_engine::init_engine(song_path, rx);

    // -- CLI --
    println!("\nAudio player commands:");
    println!(" [p] Pause");
    println!(" [r] Resume");
    println!(" [q] Quit\n");

    let mut input = String::new();

    loop {
        input.clear();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "p" => {
                tx.send(AudioCommand::Pause).unwrap();
                println!("PAUSE");
            },
            "r" => {
                tx.send(AudioCommand::Play).unwrap();
                println!("RESUME");
            },
            "q" => {
                tx.send(AudioCommand::Stop).unwrap();
                println!("stopping audio engine...");
                break;
            },
            _ => println!("Invalid command."),
        }
    }
}