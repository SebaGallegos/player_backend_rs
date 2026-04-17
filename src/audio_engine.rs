use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::fs::File;
use std::io::BufReader;

pub fn play(file_path: &str) -> Result<(), String> {
    // find the default output device
    let mut stream_handle = DeviceSinkBuilder::open_default_sink()
        .map_err(|_| "Default audio hardware output device not found.")?;

    // to prevent the log message "Stream dropped while still playing" when the player is dropped
    stream_handle.log_on_drop(false);

    // create a new player
    let player = Player::connect_new(stream_handle.mixer());

    // open the audio file
    let file = File::open(file_path)
        .map_err(|_| format!("Cannot open audio file: '{}', please check the path.", file_path))?;

    // decode the audio file
    let reader = BufReader::new(file);
    let source = Decoder::new(reader)
        .map_err(|_| "The file not have a supported audio format or is corrupted.")?;

    // play the audio
    player.append(source);

    // keep the player alive until the audio finishes playing
    player.sleep_until_end();

    // return success
    Ok(())
}