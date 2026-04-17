use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Receiver;  // for receiving messages (as command) from main thread for CLI and future GUI
use std::thread;

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
        let file = File::open(&song_path)   // & to borrow the string, not take ownership
            .expect("Error: Cannot open audio file, please check the path."); // TODO: improve error message
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)
            .expect("Error: The file does not have a supported audio format or is corrupted.");

        // 4. play the audio
        player.append(source);

        // IMPORTANT
        // 5. engine while loop: listen for commands from main thread
        // and control the player accordingly
        while let Ok(command) = command_receiver.recv() {
            match command {
                AudioCommand::Pause => player.pause(),
                AudioCommand::Play => player.play(),
                AudioCommand::Stop => {
                    player.stop();
                    break;  // exit the loop and end the thread
                }
            }
        }
    });
}

// pub fn play(file_path: &str) -> Result<(), String> {
//     // find the default output device
//     let mut stream_handle = DeviceSinkBuilder::open_default_sink()
//         .map_err(|_| "Default audio hardware output device not found.")?;

//     // to prevent the log message "Stream dropped while still playing" when the player is dropped
//     stream_handle.log_on_drop(false);

//     // create a new player
//     let player = Player::connect_new(stream_handle.mixer());

//     // open the audio file
//     let file = File::open(file_path)
//         .map_err(|_| format!("Cannot open audio file: '{}', please check the path.", file_path))?;

//     // decode the audio file
//     let reader = BufReader::new(file);
//     let source = Decoder::new(reader)
//         .map_err(|_| "The file not have a supported audio format or is corrupted.")?;

//     // play the audio
//     player.append(source);

//     // keep the player alive until the audio finishes playing
//     player.sleep_until_end();

//     // return success
//     Ok(())
// }