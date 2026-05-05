use lofty::prelude::{Accessor, TaggedFileExt};
use lofty::probe::Probe;

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
    let tagged_file = match Probe::open(file_path).and_then(|p| p.read()) {
        Ok(f) => f,
        Err(_) => {
            return SongMetadata {
                title: "Local file (without metadata)".to_string(),
                artist: "Unknown".to_string(),
                album: "Unknown".to_string(),
                year: "----".to_string(),
            };
        }
    };

    let tag_opt = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    if let Some(tag) = tag_opt {
        SongMetadata {
            title: tag
                .title()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown".to_string()),
            artist: tag
                .artist()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown".to_string()),
            album: tag
                .album()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown".to_string()),
            year: tag
                .date()
                .map(|d| d.year.to_string())
                .unwrap_or_else(|| "----".to_string()),
        }
    } else {
        SongMetadata {
            title: "Local file (without metadata)".to_string(),
            artist: "Unknown".to_string(),
            album: "Unknown".to_string(),
            year: "----".to_string(),
        }
    }
}
