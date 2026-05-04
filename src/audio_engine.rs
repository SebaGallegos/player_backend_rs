use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError};  // for receiving messages (as command) from main thread for CLI and future GUI
use std::thread;
use std::time::Duration;    // to define a timeout
use std::process;

// audio commands will be sent from main thread (CLI or GUI)
// to the audio engine thread via this channel
pub enum AudioCommand {
    Pause,
    Play,
    Stop,
}

pub fn init_engine(song_path: String, command_receiver: Receiver<AudioCommand>) {
    // thread: run audio engine in a separate thread
    // move: move song_path and command_receiver ownership into the thread
    thread::spawn(move || {
        // 1. find the default output device
        let mut stream_handle = DeviceSinkBuilder::open_default_sink()
            .expect("CRITICAL: Default audio hardware output device not found.");

        // optional: to prevent the log message "Stream dropped while still playing" when the player is dropped
        stream_handle.log_on_drop(false);

        // 2. create a new player
        let player = Player::connect_new(stream_handle.mixer());

        // 3. open the audio file
        let file = File::open(&song_path)   // '&' to borrow the string, not take ownership
            .expect("Error: Cannot open audio file, please check the path."); // TODO: improve error message
        let reader = BufReader::new(file);
        let source_song = Decoder::new(reader)
            .expect("Error: The file does not have a supported audio format or is corrupted.");

        // This calculates the total duration of the song, which is used to display the progress in the CLI
        let total_duration = source_song.total_duration().unwrap_or(Duration::from_secs(0));
        let total_sec = total_duration.as_secs();

        // 4. play the audio
        player.append(source_song);

        // IMPORTANT
        // 5. engine while loop: listen for commands from main thread
        // and control the player accordingly
        loop {
            // wait a message with a timeout of 200ms
            match command_receiver.recv_timeout(Duration::from_millis(200)){
                // If this receives a command, execute the corresponding action on the player
                Ok(command) => match command {
                    AudioCommand::Pause => {
                        player.pause();
                        println!();
                    },
                    AudioCommand::Play => {
                        player.play();
                        println!();
                    },
                    AudioCommand::Stop => {
                        player.stop();
                        println!();
                        break;  // exit the loop and end the thread
                    }
                },
                // If the timeout is reached without receiving a command,
                // check if the player if still playing, if not, it means
                // the song has finished, so we can exit the thread
                Err(RecvTimeoutError::Timeout) => {
                    if !player.empty() && !player.is_paused() {
                        // This gets the current position of the player in seconds and prints the progress in the CLI
                        let actual_second = player.get_pos().as_secs();

                        // \r: delete the current line and print the new progress, without creating a new line
                        print!("\r Progress: {:02}:{:02} / {:02}:{:02} [write a command: p/r/q] ",
                            actual_second / 60, actual_second % 60,
                            total_sec / 60, total_sec % 60
                        );
                        
                        // flush the output to ensure the progress is printed immediately
                        io::stdout().flush().unwrap();
                    }

                    if player.empty() {
                        println!("Song finished, exiting audio engine...");
                        process::exit(0);
                    }
                }
                // If the channel is disconnected for other reasons,
                // also exit the thread
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}
