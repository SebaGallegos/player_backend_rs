mod audio_engine;

use audio_engine::AudioCommand;
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

    // 2. create communication channel
    // tx: this main thread's sender, used to send commands to the audio engine thread in FIFO order
    // rx: the audio engine thread's receiver, used to receive commands from the main thread
    let (tx, rx) = mpsc::channel();

    // 3. initialize the audio engine in a separate thread
    println!("Initializing audio engine...");
    audio_engine::init_engine(song_path, rx);

    // -- CLI --
    println!("\n Reproductor de audio: ");
    println!(" [p] Pausar");
    println!(" [r] Reanudar");
    println!(" [q] Salir");
    println!("Escribe un comando...\n");

    let mut input = String::new();

    loop {
        input.clear();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "p" => {
                tx.send(AudioCommand::Pause).unwrap();
                println!("PAUSAR");
            },
            "r" => {
                tx.send(AudioCommand::Play).unwrap();
                println!("REANUDAR");
            },
            "q" => {
                tx.send(AudioCommand::Stop).unwrap();
                println!("stopping audio engine...");
                break;
            },
            _ => println!("Comando no válido"),
        }
    }
}