use anyhow::Result;
use general::split_on;
use id3::{Frame, TagLike};
use regex::Regex;
use serde::Serialize;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{metadata, File};
use std::path::Path;
use lofty::prelude::*;

// clap arg parser
mod argparse;

#[derive(Clone, Debug, Serialize)]
struct AudioInfo {
    title: String,
    artist: String,
    album: String,
    album_artist: String,
    genre: String,
    year: usize,
    track_number: usize,
    track_total: usize,
    disc_number: usize,
    disc_total: usize,
    compilation: bool,
    encoder: String,
    version: String,
    seconds: usize,
    extension: String,
    bitrate: usize,
    path: String,
}

impl AudioInfo {
    pub fn json(&self) -> String {
        serde_json::to_string(self).unwrap_or("{}".into())
    }
}

impl fmt::Display for AudioInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.title.is_empty() {
            writeln!(f, "title:       {}", self.title)?;
        }

        if !self.artist.is_empty() {
            writeln!(f, "artist:      {}", self.artist)?;
        }

        if !self.album.is_empty() {
            writeln!(f, "album:       {}", self.album)?;
        }

        if !self.album_artist.is_empty() {
            writeln!(f, "alb_artist:  {}", self.album_artist)?;
        }

        if !self.genre.is_empty() {
            writeln!(f, "genre:       {}", self.genre)?;
        }

        if self.year > 0 {
            writeln!(f, "year:        {}", self.year)?;
        }

        if self.track_number > 0 {
            if self.track_total > 0 {
                writeln!(f, "track:       {}/{}", self.track_number, self.track_total)?;
            } else {
                writeln!(f, "track:       {}", self.track_number)?;
            }
        }

        if self.disc_total > 0 {
            writeln!(f, "disc:        {}/{}", self.disc_number, self.disc_total)?;
        } else if self.disc_number > 0 {
            writeln!(f, "disc:        {}", self.disc_number)?;
        }

        writeln!(
            f,
            "time:        {}",
            match self.seconds {
                0 => "0:00".into(),
                s => format!("{}:{:02}", s / 60, s % 60),
            }
        )?;

        if self.compilation {
            writeln!(f, "comp:        yes")?;
        }

        if !self.encoder.is_empty() {
            writeln!(f, "encoder:     {}", self.encoder)?;
        }

        if !self.extension.is_empty() {
            writeln!(f, "extension:   {}", self.extension)?;
        }

        if !self.version.is_empty() {
            writeln!(f, "version:     {}", self.version)?;
        }

        if self.bitrate > 0 {
            writeln!(f, "bitrate:     {}", self.bitrate)?;
        }

        writeln!(f, "path:        {}", self.path)
    }
}

#[derive(Clone)]
enum Tagger {
    // (tag, path, extension, bitrate, seconds)
    M4a(mp4ameta::Tag, String, String, usize, usize),
    Mp3(id3::Tag, String, String, usize, usize),
    Flac(metaflac::Tag, String, String, usize, usize),
    Ogg(lofty::tag::Tag, String, String, usize, usize),
}
impl Tagger {
    // Artist
    // ======
    fn artist(&self) -> String {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.artist().unwrap_or("").into(),
            Tagger::Mp3(tag, _, _, _, _) => tag.artist().unwrap_or("").into(),
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("artist") {
                Some(iter) => iter.collect::<Vec<_>>()[0].to_string(),
                None => "".into(),
            },
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::TrackArtist).unwrap_or("").into(),
        }
    }
    fn remove_artist(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_artists(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_artist(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("artist"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackArtist),
        }
    }
    fn set_artist(&mut self, artist: &str) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_artist(artist),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_artist(artist),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("artist", vec![artist]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::TrackArtist, artist.into());
            }
        }
    }

    // Album
    // =====
    fn album(&self) -> String {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.album().unwrap_or("").into(),
            Tagger::Mp3(tag, _, _, _, _) => tag.album().unwrap_or("").into(),
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("album") {
                Some(iter) => iter.collect::<Vec<_>>()[0].to_string(),
                None => "".into(),
            },
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::AlbumTitle).unwrap_or("").into(),
        }
    }
    fn remove_album(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_album(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_album(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("album"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::AlbumTitle),
        }
    }
    fn set_album(&mut self, album: &str) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_album(album),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_album(album),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("album", vec![album]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::AlbumTitle, album.into());
            }
        }
    }

    // Album Artist
    // ============
    fn album_artist(&self) -> String {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.album_artist().unwrap_or("").into(),
            Tagger::Mp3(tag, _, _, _, _) => tag.album_artist().unwrap_or("").into(),
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("albumartist") {
                Some(iter) => iter.collect::<Vec<_>>()[0].to_string(),
                None => "".into(),
            },
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::AlbumArtist).unwrap_or("").into(),
        }
    }
    fn remove_album_artist(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_album_artists(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_album_artist(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("albumartist"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::AlbumArtist),
        }
    }
    fn set_album_artist(&mut self, album_artist: &str) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_album_artist(album_artist),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_album_artist(album_artist),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("albumartist", vec![album_artist]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::AlbumArtist, album_artist.into());
            }
        }
    }

    // Title
    // =====
    fn title(&self) -> String {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.title().unwrap_or("").into(),
            Tagger::Mp3(tag, _, _, _, _) => tag.title().unwrap_or("").into(),
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("title") {
                Some(iter) => iter.collect::<Vec<_>>()[0].to_string(),
                None => "".into(),
            },
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::TrackTitle).unwrap_or("").into(),
        }
    }
    fn remove_title(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_title(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_title(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("title"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackTitle),
        }
    }
    fn set_title(&mut self, title: &str) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_title(title),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_title(title),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("title", vec![title]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::TrackTitle, title.into());
            }
        }
    }

    // Track Number
    // ============
    fn track_number(&self) -> usize {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.track_number().unwrap_or(0) as usize,
            Tagger::Mp3(tag, _, _, _, _) => tag.track().unwrap_or(0) as usize,
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("tracknumber") {
                Some(iter) => iter.collect::<Vec<_>>()[0].parse::<usize>().unwrap(),
                None => 0,
            },
            Tagger::Ogg(tag, _, _, _, _) => tag
                .get_string(&ItemKey::TrackNumber)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0),
        }
    }
    fn remove_track_number(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_track_number(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_track(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("tracknumber"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackNumber),
        }
    }
    fn set_track_number(&mut self, track_number: usize) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_track_number(track_number as u16),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_track(track_number as u32),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("tracknumber", vec![track_number.to_string()]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::TrackNumber, track_number.to_string());
            }
        }
    }

    // Total Tracks
    // ============
    fn track_total(&self) -> usize {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.total_tracks().unwrap_or(0) as usize,
            Tagger::Mp3(tag, _, _, _, _) => tag.total_tracks().unwrap_or(0) as usize,
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("tracktotal") {
                Some(iter) => iter.collect::<Vec<_>>()[0].parse::<usize>().unwrap(),
                None => 0,
            },
            Tagger::Ogg(tag, _, _, _, _) => tag
                .get_string(&ItemKey::TrackTotal)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0),
        }
    }
    fn remove_track_total(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_total_tracks(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_total_tracks(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("tracktotal"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackTotal),
        }
    }
    fn set_track_total(&mut self, track_total: usize) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_total_tracks(track_total as u16),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_total_tracks(track_total as u32),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("tracktotal", vec![track_total.to_string()]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::TrackTotal, track_total.to_string());
            }
        }
    }

    // Disc Number
    // ===========
    fn disc_number(&self) -> usize {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.disc_number().unwrap_or(0) as usize,
            Tagger::Mp3(tag, _, _, _, _) => tag.disc().unwrap_or(0) as usize,
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("discnumber") {
                Some(iter) => iter.collect::<Vec<_>>()[0].parse::<usize>().unwrap(),
                None => 0,
            },
            Tagger::Ogg(tag, _, _, _, _) => tag
                .get_string(&ItemKey::DiscNumber)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0),
        }
    }
    fn remove_disc_number(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_disc_number(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_disc(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("discnumber"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::DiscNumber),
        }
    }
    fn set_disc_number(&mut self, disc_number: usize) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_disc_number(disc_number as u16),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_disc(disc_number as u32),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("discnumber", vec![disc_number.to_string()]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::DiscNumber, disc_number.to_string());
            }
        }
    }

    // Total Discs
    // ===========
    fn disc_total(&self) -> usize {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.total_discs().unwrap_or(0) as usize,
            Tagger::Mp3(tag, _, _, _, _) => tag.total_discs().unwrap_or(0) as usize,
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("disctotal") {
                Some(iter) => iter.collect::<Vec<_>>()[0].parse::<usize>().unwrap(),
                None => 0,
            },
            Tagger::Ogg(tag, _, _, _, _) => tag
                .get_string(&ItemKey::DiscTotal)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0),
        }
    }
    fn remove_disc_total(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_total_discs(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_total_discs(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("disctotal"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::DiscTotal),
        }
    }
    fn set_disc_total(&mut self, disc_total: usize) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_total_discs(disc_total as u16),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_total_discs(disc_total as u32),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("disctotal", vec![disc_total.to_string()]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::DiscTotal, disc_total.to_string());
            }
        }
    }

    // Year
    // ====
    fn year(&self) -> usize {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.year().unwrap_or("0").parse::<usize>().unwrap_or(0),
            Tagger::Mp3(tag, _, _, _, _) => tag.year().unwrap_or(0) as usize,
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("date") {
                Some(iter) => iter.collect::<Vec<_>>()[0].parse::<usize>().unwrap(),
                None => 0,
            },
            Tagger::Ogg(tag, _, _, _, _) => tag
                .get_string(&ItemKey::Year)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0),
        }
    }
    fn remove_year(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_year(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_year(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("date"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::Year),
        }
    }
    fn set_year(&mut self, year: usize) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_year(year.to_string()),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_year(year as i32),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("date", vec![year.to_string()]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::Year, year.to_string());
            }
        }
    }

    // Genre
    // =====
    fn genre(&self) -> String {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.genre().unwrap_or("").into(),
            Tagger::Mp3(tag, _, _, _, _) => {
                let re = Regex::new(r"^\([^)]+\)").unwrap();
                match re.replace(tag.genre().unwrap_or(""), "") {
                    g if g.is_empty() => tag.genre_parsed().unwrap_or(std::borrow::Cow::Borrowed("")).into(),
                    g => g.into(),
                }
            }
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("genre") {
                Some(iter) => iter.collect::<Vec<_>>()[0].to_string(),
                None => "".into(),
            },
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::Genre).unwrap_or("").into(),
        }
    }
    fn remove_genre(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_genres(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_genre(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("genre"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::Genre),
        }
    }
    fn set_genre(&mut self, genre: &str) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_genre(genre),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_genre(genre),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("genre", vec![genre]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::Genre, genre.into());
            }
        }
    }

    // Compilation
    // ===========
    fn compilation(&self) -> bool {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.compilation(),
            Tagger::Mp3(tag, _, _, _, _) => tag.get("TCMP").is_some() || tag.get("TCP").is_some(),
            Tagger::Flac(tag, _, _, _, _) => tag.get_vorbis("compilation").is_some(),
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::FlagCompilation).is_some(),
        }
    }
    fn remove_compilation(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_compilation(),
            Tagger::Mp3(tag, _, _, _, _) => {
                tag.remove("TCMP");
                tag.remove("TCP");
            }
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("compilation"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::FlagCompilation),
        }
    }
    fn set_compilation(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_compilation(),
            Tagger::Mp3(tag, _, _, _, _) => {
                tag.remove("TCP");
                tag.add_frame(Frame::text("TCMP", "1"));
            }
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("compilation", vec!["1"]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::FlagCompilation, "1".to_string());
            }
        }
    }

    // Encoder
    // =======
    fn encoder(&self) -> String {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.encoder().unwrap_or("").into(),
            Tagger::Mp3(tag, _, _, _, _) => match tag.get("TENC") {
                Some(encoder) => encoder.to_string().replace("Encoded by = ", ""),
                _ => "".into(),
            },
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("encoded-by") {
                Some(iter) => iter.collect::<Vec<_>>()[0].to_string(),
                None => "".into(),
            },
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::EncoderSoftware).unwrap_or("").into(),
        }
    }

    // Version
    // =======
    fn version(&self) -> String {
        match self {
            Tagger::M4a(_, _, _, _, _) => "".into(),
            Tagger::Mp3(tag, _, _, _, _) => tag.version().to_string(),
            Tagger::Flac(tag, _, _, _, _) => match tag.vorbis_comments() {
                Some(vc) => match tag.get_streaminfo() {
                    Some(si) => format!(
                        "{}, {} bits per sample @ {}Hz",
                        vc.vendor_string, si.bits_per_sample, si.sample_rate
                    ),
                    None => vc.vendor_string.clone(),
                },
                None => "".into(),
            },
            Tagger::Ogg(_, _, _, _, _) => "".into(),
        }
    }

    // Zero -- remove all fields and metatdata
    // =======================================
    fn zero(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.clear(),
            Tagger::Mp3(tag, _, _, _, _) => {
                tag.remove_album();
                tag.remove_album_artist();
                tag.remove_all_chapters();
                tag.remove_all_lyrics();
                tag.remove_all_pictures();
                tag.remove_all_synchronised_lyrics();
                tag.remove_all_tables_of_contents();
                tag.remove_all_unique_file_identifiers();
                tag.remove_artist();
                tag.remove_date_recorded();
                tag.remove_date_released();
                tag.remove_disc();
                tag.remove_duration();
                tag.remove_genre();
                tag.remove_original_date_released();
                tag.remove_title();
                tag.remove_total_discs();
                tag.remove_total_tracks();
                tag.remove_track();
                tag.remove_year();
            }
            Tagger::Flac(tag, _, _, _, _) => {
                for s in [
                    "artist",
                    "album",
                    "albumartist",
                    "title",
                    "tracknumber",
                    "tracktotal",
                    "discnumber",
                    "disctotal",
                    "date",
                    "genre",
                    "compilation",
                ] {
                    tag.remove_vorbis(s);
                }
            }
            Tagger::Ogg(tag, _, _, _, _) => {
                for key in [
                    ItemKey::TrackArtist,
                    ItemKey::AlbumTitle,
                    ItemKey::AlbumArtist,
                    ItemKey::TrackTitle,
                    ItemKey::TrackNumber,
                    ItemKey::TrackTotal,
                    ItemKey::DiscNumber,
                    ItemKey::DiscTotal,
                    ItemKey::Year,
                    ItemKey::Genre,
                    ItemKey::FlagCompilation,
                ] {
                    tag.remove_key(&key);
                }
            }
        }
    }

    // Save
    // ====
    fn save(&mut self, file: &Path) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.write_to_path(file).expect("M4a - write failed"),
            Tagger::Mp3(tag, _, _, _, _) => {
                if tag.write_to_path(file, tag.version()).is_err() {
                    match tag.version() {
                        id3::Version::Id3v22 => tag
                            .write_to_path(file, id3::Version::Id3v23)
                            .expect("Mp3 - write failed"),
                        _ => tag
                            .write_to_path(file, id3::Version::Id3v24)
                            .expect("Mp3 - write failed"),
                    }
                }
            }
            Tagger::Flac(tag, _, _, _, _) => tag.save().expect("Flac - write failed"),
            Tagger::Ogg(tag, _, _, _, _) => tag
                .save_to_path(file, lofty::config::WriteOptions::default())
                .expect("Ogg - write failed"),
        }
    }

    // Path
    // ====
    fn path(&self) -> String {
        match self {
            Tagger::M4a(_, path, _, _, _) => path.to_string(),
            Tagger::Mp3(_, path, _, _, _) => path.to_string(),
            Tagger::Flac(_, path, _, _, _) => path.to_string(),
            Tagger::Ogg(_, path, _, _, _) => path.to_string(),
        }
    }

    // Extension
    // =========
    fn extension(&self) -> String {
        match self {
            Tagger::M4a(_, _, extension, _, _) => extension.to_string(),
            Tagger::Mp3(_, _, extension, _, _) => extension.to_string(),
            Tagger::Flac(_, _, extension, _, _) => extension.to_string(),
            Tagger::Ogg(_, _, extension, _, _) => extension.to_string(),
        }
    }

    // Bitrate
    // =======
    fn bitrate(&self) -> usize {
        match self {
            Tagger::M4a(_, _, _, bitrate, _) => *bitrate,
            Tagger::Mp3(_, _, _, bitrate, _) => *bitrate,
            Tagger::Flac(_, _, _, bitrate, _) => *bitrate,
            Tagger::Ogg(_, _, _, bitrate, _) => *bitrate,
        }
    }

    // Seconds
    // =======
    fn seconds(&self) -> usize {
        match self {
            Self::M4a(_, _, _, _, seconds) => *seconds,
            Self::Mp3(_, _, _, _, seconds) => *seconds,
            Self::Flac(_, _, _, _, seconds) => *seconds,
            Self::Ogg(_, _, _, _, seconds) => *seconds,
        }
    }

    // AudioInfo
    // =========
    fn audio_info(&mut self) -> AudioInfo {
        AudioInfo {
            title: self.title(),
            artist: self.artist(),
            album: self.album(),
            album_artist: self.album_artist(),
            genre: self.genre(),
            year: self.year(),
            track_number: self.track_number(),
            track_total: self.track_total(),
            disc_number: self.disc_number(),
            disc_total: self.disc_total(),
            compilation: self.compilation(),
            encoder: self.encoder(),
            version: self.version(),
            seconds: self.seconds(),
            extension: self.extension(),
            bitrate: self.bitrate(),
            path: self.path(),
        }
    }
}

fn tagger_from_file(file: &Path) -> Tagger {
    let path = file.to_string_lossy().to_string();
    let extension = file.extension().unwrap_or(OsStr::new("")).to_string_lossy().to_string();

    match extension.as_ref() {
        "ogg" => {
            let tagged_file = lofty::probe::Probe::open(file)
                .expect("ERROR: Bad path provided!")
                .read()
                .expect("ERROR: Failed to read file!");

            let tag = match tagged_file.primary_tag() {
                Some(primary_tag) => primary_tag,
                // If the "primary" tag doesn't exist, we just grab the
                // first tag we can find. Realistically, a tag reader would likely
                // iterate through the tags to find a suitable one.
                None => tagged_file.first_tag().expect("ERROR: No tags found!"),
            };

            let properties = tagged_file.properties();
            let duration = properties.duration();
            let seconds = duration.as_secs() as usize;
            let bitrate = properties.audio_bitrate().unwrap_or(0) as usize;
            Tagger::Ogg(tag.clone(), path, extension, bitrate, seconds)
        }
        "m4a" => {
            let tag = mp4ameta::Tag::read_from_path(file)
                .unwrap_or_else(|_| panic!("could not open file `{:?}`", file.as_os_str()));
            let bitrate = if let Some(avg_bitrate) = tag.avg_bitrate() {
                avg_bitrate as usize / 1000
            } else if let Some(duration) = tag.duration() {
                // Lossless bitrate:
                // Sum non-audio: (&DataIdent, &Data) to subtract from the audio byte length
                let non_audio_len = tag.data().fold(0, |acc, d| acc + d.1.len()) as usize;
                (metadata(file).expect("metadata").len() as usize - non_audio_len) * 8 / duration.as_millis() as usize
            } else {
                0
            };
            let seconds = tag.duration().unwrap_or(std::time::Duration::new(0, 0)).as_secs() as usize;
            Tagger::M4a(tag, path, extension, bitrate, seconds)
        }
        "mp3" => {
            let tag = match id3::Tag::read_from_path(file) {
                Ok(tag) => tag,
                Err(id3::Error {
                    kind: id3::ErrorKind::NoTag,
                    ..
                }) => id3::Tag::new(),
                Err(err) => panic!("{}", err),
            };

            let mut decoder = minimp3_fixed::Decoder::new(File::open(file).expect("failed to open media"));

            // Gather some vitals from the 1st frame [ data.len(), channels, sample_rate ]; then count
            // the remaining frames and finally estimate a duration.
            // Duration: 1152 time domain samples per channel in a typical PCM WAV file
            let duration = match decoder.next_frame() {
                Ok(minimp3_fixed::Frame {
                    data,
                    sample_rate,
                    channels,
                    ..
                }) => {
                    let mut frame_cnt = 1;
                    while decoder.next_frame().is_ok() {
                        frame_cnt += 1;
                    }
                    //assert_eq!(data.len() / channels, 1152);
                    std::time::Duration::new((frame_cnt * data.len() / channels / sample_rate as usize) as u64, 0)
                }
                _ => panic!("failed to read the first frame"),
            };

            let bitrate = match duration.as_millis() {
                n if n > 0 => (metadata(file).expect("metadata").len() as usize) * 8 / n as usize,
                _ => 0,
            };

            Tagger::Mp3(tag, path, extension, bitrate, duration.as_secs() as usize)
        }
        "flac" => {
            let tag = metaflac::Tag::read_from_path(file)
                .unwrap_or_else(|_| panic!("could not open file `{:?}`", file.as_os_str()));
            let seconds = match tag.get_streaminfo() {
                Some(si) => (si.total_samples / si.sample_rate as u64) as usize,
                None => 0,
            };
            let bitrate = match seconds > 0 {
                true => (metadata(file).expect("metadata").len() as usize) * 8 / seconds / 1000,
                false => seconds,
            };

            Tagger::Flac(tag, path, extension, bitrate, seconds)
        }
        _ => todo!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = argparse::get_args();

    let audio_files: Vec<_> = args
        .get_many::<std::path::PathBuf>("FILE")
        .expect("argparse to enforce 1 or more input files")
        .collect();

    for file in audio_files {
        let mut modified = false;
        let mut tagger = tagger_from_file(file);

        // Zero -- remove all fields and metatdata
        if args.get_flag("zero") {
            modified = true;
            println!("--zero: modified = {modified}");
            tagger.zero();
        }

        // Title
        if let Some(title) = args.get_one::<String>("title") {
            if tagger.title() != *title {
                modified = true;
                match title.is_empty() {
                    true => tagger.remove_title(),
                    false => tagger.set_title(title),
                }
            }
        }

        // Artist
        if let Some(artist) = args.get_one::<String>("artist") {
            if tagger.artist() != *artist {
                modified = true;
                match artist.is_empty() {
                    true => tagger.remove_artist(),
                    false => tagger.set_artist(artist),
                }
            }
        }

        // Album
        if let Some(album) = args.get_one::<String>("album") {
            if tagger.album() != *album {
                modified = true;
                match album.is_empty() {
                    true => tagger.remove_album(),
                    false => tagger.set_album(album),
                }
            }
        }

        // Album Artist
        if let Some(album_artist) = args.get_one::<String>("album-artist") {
            if tagger.album_artist() != *album_artist {
                modified = true;
                match album_artist.is_empty() {
                    true => tagger.remove_album_artist(),
                    false => tagger.set_album_artist(album_artist),
                }
            }
        }

        // Track Number
        if let Some(track_number) = args.get_one::<usize>("track-number") {
            if tagger.track_number() != *track_number {
                modified = true;
                match track_number == &0 {
                    true => tagger.remove_track_number(),
                    false => tagger.set_track_number(*track_number),
                }
            }
        }

        // Track Number + Track Total
        if let Some(trkn) = args.get_one::<String>("trkn") {
            match trkn.is_empty() {
                true => {
                    modified = modified || tagger.track_number() > 0 || tagger.track_total() > 0;
                    println!("--trkn: modified = {modified}");
                    tagger.remove_track_number();
                    tagger.remove_track_total();
                }
                false => {
                    let trim = true;
                    let components = split_on::<usize>(trkn, '/', trim)?;
                    assert_eq!(components.len(), 2, "expected track_number/track_total");
                    let (track_number, track_total) = (components[0], components[1]);
                    assert!(track_total >= track_number, "expected track_total >= track_number");
                    modified = modified || (tagger.track_number() != track_number);
                    modified = modified || (tagger.track_total() != track_total);
                    println!("--trkn: modified = {modified}");
                    if modified {
                        tagger.set_track_number(track_number);
                        tagger.set_track_total(track_total);
                    }
                }
            }
        }

        // Track Total
        if let Some(track_total) = args.get_one::<usize>("track-total") {
            if tagger.track_total() != *track_total {
                modified = true;
                match track_total == &0 {
                    true => tagger.remove_track_total(),
                    false => tagger.set_track_total(*track_total),
                }
            }
        }

        // Disc Number
        if let Some(disc_number) = args.get_one::<usize>("disc-number") {
            if tagger.disc_number() != *disc_number {
                modified = true;
                match disc_number == &0 {
                    true => tagger.remove_disc_number(),
                    false => tagger.set_disc_number(*disc_number),
                }
            }
        }

        // Disc Total
        if let Some(disc_total) = args.get_one::<usize>("disc-total") {
            if tagger.disc_total() != *disc_total {
                modified = true;
                match disc_total == &0 {
                    true => tagger.remove_disc_total(),
                    false => tagger.set_disc_total(*disc_total),
                }
            }
        }

        // Year
        if let Some(year) = args.get_one::<usize>("year") {
            if tagger.year() != *year {
                modified = true;
                match year == &0 {
                    true => tagger.remove_year(),
                    false => tagger.set_year(*year),
                }
            }
        }

        // Genre
        if let Some(genre) = args.get_one::<String>("genre") {
            if tagger.genre() != *genre {
                modified = true;
                match genre.is_empty() {
                    true => tagger.remove_genre(),
                    false => tagger.set_genre(genre),
                }
            }
        }

        // Compilation Flag
        if args.get_flag("compilation") {
            modified = modified || !tagger.compilation();
            println!("--compilation: modified = {modified}");
            tagger.set_compilation();
        } else if args.get_flag("no-compilation") {
            modified = modified || tagger.compilation();
            println!("--no-compilation: modified = {modified}");
            tagger.remove_compilation();
        }

        if modified {
            println!("*** MODIFIED ***");
            tagger.save(file);
        }

        let audio_info = tagger.audio_info();

        if args.get_flag("json") {
            println!("{}", audio_info.json());
        } else {
            println!("{audio_info}");
        }
    }

    Ok(())
}
