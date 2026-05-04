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

// fn render_status(actual_sec: u64, total_sec: u64, status: &str)
// this renders the progress of the song in the CLI and his current status,
// flushing stdout to update the progress in real time
// without creating a new line of each update
fn render_status(actual_sec: u64, total_sec: u64, status: &str) {
    print!(
        "\r Progress: {:02}:{:02} / {:02}:{:02} [{}] [write a command: p/r/q] ",
        actual_sec / 60,
        actual_sec % 60,
        total_sec / 60,
        total_sec % 60,
        status
    );

    // (:) oh, unwrap() triggers a panic if the flush fails
    // well, I changes it to expect()
    // to provide a custom message in case of error
    // and "panic" is a very scary word too :D
    io::stdout().flush().expect("Error: Failed to flush stdout.");
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
                        render_status(player.get_pos().as_secs(), total_sec, "PAUSED");
                    },
                    AudioCommand::Play => {
                        player.play();
                        render_status(player.get_pos().as_secs(), total_sec, "PLAYING");
                    },
                    AudioCommand::Stop => {
                        player.stop();
                        print!("\r");
                        io::stdout().flush().expect("Error: Failed to flush stdout.");
                        break;  // exit the loop and end the thread
                    }
                },
                // If the timeout is reached without receiving a command,
                // check if the player if still playing, if not, it means
                // the song has finished, so we can exit the thread
                Err(RecvTimeoutError::Timeout) => {
                    if !player.empty() {
                        // get the current position of the song in seconds
                        let actual_sec = player.get_pos().as_secs();
                        // get the current status of the player
                        let status = if player.is_paused() { "PAUSED" } else { "PLAYING" };
                        // finally render the status in the CLI
                        render_status(actual_sec, total_sec, status);
                    }

                    if player.empty() {
                        println!("\nSong finished, exiting audio engine...");
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
