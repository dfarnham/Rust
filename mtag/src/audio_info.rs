use crate::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: usize,
    pub track_number: usize,
    pub track_total: usize,
    pub disc_number: usize,
    pub disc_total: usize,
    pub compilation: bool,
    pub encoder: String,
    pub version: String,
    pub seconds: usize,
    pub extension: String,
    pub bitrate: usize,
    pub path: String,
}

impl AudioInfo {
    pub fn json(&self) -> String {
        serde_json::to_string(self).unwrap_or("{}".into())
    }
}

impl fmt::Display for AudioInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.title.is_empty() {
            writeln!(f, "title:        {}", self.title)?;
        }

        if !self.artist.is_empty() {
            writeln!(f, "artist:       {}", self.artist)?;
        }

        if !self.album.is_empty() {
            writeln!(f, "album:        {}", self.album)?;
        }

        if !self.album_artist.is_empty() {
            writeln!(f, "albumartist:  {}", self.album_artist)?;
        }

        if !self.genre.is_empty() {
            writeln!(f, "genre:        {}", self.genre)?;
        }

        if self.year > 0 {
            writeln!(f, "year:         {}", self.year)?;
        }

        if self.track_number > 0 {
            if self.track_total > 0 {
                writeln!(f, "track:        {}/{}", self.track_number, self.track_total)?;
            } else {
                writeln!(f, "track:        {}", self.track_number)?;
            }
        }

        if self.disc_total > 0 {
            writeln!(f, "disc:         {}/{}", self.disc_number, self.disc_total)?;
        } else if self.disc_number > 0 {
            writeln!(f, "disc:         {}", self.disc_number)?;
        }

        writeln!(
            f,
            "time:         {}",
            match self.seconds {
                0 => "0:00".into(),
                s => format!("{}:{:02}", s / 60, s % 60),
            }
        )?;

        if self.compilation {
            writeln!(f, "comp:         yes")?;
        }

        if !self.encoder.is_empty() {
            writeln!(f, "encoder:      {}", self.encoder)?;
        }

        if !self.extension.is_empty() {
            writeln!(f, "extension:    {}", self.extension)?;
        }

        if !self.version.is_empty() {
            writeln!(f, "version:      {}", self.version)?;
        }

        if self.bitrate > 0 {
            writeln!(f, "bitrate:      {}", self.bitrate)?;
        }

        writeln!(f, "path:         {}", self.path)
    }
}
