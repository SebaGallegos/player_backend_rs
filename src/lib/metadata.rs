use id3::{Tag, TagLike};

// Struct to hold metadata information
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
}

// Function to extract metadata from an audio file.
// Always returns SongMetadata, even if the file has no metadata,
// in that case the fields will be "Unknown"
pub fn extract_metadata(file_path: &str) -> SongMetadata {
    // 1. try to read the metadata tags from the file
    // if successful, extract the relevant fields and return a SongMetadata struct
    match Tag::read_from_path(file_path) {
        Ok(tags) => SongMetadata {
            title: tags.title().unwrap_or("Unknown").to_string(),
            artist: tags.artist().unwrap_or("Unknown").to_string(),
            album: tags.album().unwrap_or("Unknown").to_string(),
            year: tags.year().map(|y| y.to_string()).unwrap_or_else(|| "----".to_string()),
        },
        // 2. otherwise, if failes, return a default SongMetadata with generic title
        // and "Unknown" for artist and album, and "----" for year
        Err(_) => SongMetadata {
            title: format!("Local file (without metadata)"),
            artist: "Unknown".to_string(),
            album: "Unknown".to_string(),
            year: "----".to_string(),
        },
    }
}