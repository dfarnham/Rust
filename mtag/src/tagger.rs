use id3::{Frame, TagLike};
use lofty::prelude::*;
use regex::Regex;
use std::ffi::OsStr;
use std::fs::{metadata, File};
use std::path::Path;

// Struct AudioInfo
use crate::AudioInfo;

#[derive(Clone)]
pub enum Tagger {
    // (tag, path, extension, bitrate, seconds)
    M4a(mp4ameta::Tag, String, String, usize, usize),
    Mp3(id3::Tag, String, String, usize, usize),
    Flac(metaflac::Tag, String, String, usize, usize),
    Ogg(lofty::tag::Tag, String, String, usize, usize),
}
impl Tagger {
    // Artist
    // ======
    pub fn artist(&self) -> String {
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
    pub fn remove_artist(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_artists(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_artist(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("artist"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackArtist),
        }
    }
    pub fn set_artist(&mut self, artist: &str) {
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
    pub fn album(&self) -> String {
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
    pub fn remove_album(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_album(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_album(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("album"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::AlbumTitle),
        }
    }
    pub fn set_album(&mut self, album: &str) {
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
    pub fn album_artist(&self) -> String {
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
    pub fn remove_album_artist(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_album_artists(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_album_artist(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("albumartist"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::AlbumArtist),
        }
    }
    pub fn set_album_artist(&mut self, album_artist: &str) {
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
    pub fn title(&self) -> String {
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
    pub fn remove_title(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_title(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_title(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("title"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackTitle),
        }
    }
    pub fn set_title(&mut self, title: &str) {
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
    pub fn track_number(&self) -> usize {
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
    pub fn remove_track_number(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_track_number(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_track(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("tracknumber"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackNumber),
        }
    }
    pub fn set_track_number(&mut self, track_number: usize) {
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
    pub fn track_total(&self) -> usize {
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
    pub fn remove_track_total(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_total_tracks(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_total_tracks(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("tracktotal"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::TrackTotal),
        }
    }
    pub fn set_track_total(&mut self, track_total: usize) {
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
    pub fn disc_number(&self) -> usize {
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
    pub fn remove_disc_number(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_disc_number(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_disc(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("discnumber"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::DiscNumber),
        }
    }
    pub fn set_disc_number(&mut self, disc_number: usize) {
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
    pub fn disc_total(&self) -> usize {
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
    pub fn remove_disc_total(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_total_discs(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_total_discs(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("disctotal"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::DiscTotal),
        }
    }
    pub fn set_disc_total(&mut self, disc_total: usize) {
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
    pub fn year(&self) -> usize {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.year().unwrap_or("0").parse::<usize>().unwrap_or(0),
            Tagger::Mp3(tag, _, _, _, _) => tag.year().unwrap_or(0) as usize,
            Tagger::Flac(tag, _, _, _, _) => match tag.get_vorbis("date") {
                Some(iter) => iter.collect::<Vec<_>>()[0].parse::<usize>().unwrap(),
                None => 0,
            },
            Tagger::Ogg(tag, _, _, _, _) => tag
                .get_string(&ItemKey::RecordingDate)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0),
        }
    }
    pub fn remove_year(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_year(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_year(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("date"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::RecordingDate),
        }
    }
    pub fn set_year(&mut self, year: usize) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.set_year(year.to_string()),
            Tagger::Mp3(tag, _, _, _, _) => tag.set_year(year as i32),
            Tagger::Flac(tag, _, _, _, _) => tag.set_vorbis("date", vec![year.to_string()]),
            Tagger::Ogg(tag, _, _, _, _) => {
                tag.insert_text(ItemKey::RecordingDate, year.to_string());
            }
        }
    }

    // Genre
    // =====
    pub fn genre(&self) -> String {
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
    pub fn remove_genre(&mut self) {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.remove_genres(),
            Tagger::Mp3(tag, _, _, _, _) => tag.remove_genre(),
            Tagger::Flac(tag, _, _, _, _) => tag.remove_vorbis("genre"),
            Tagger::Ogg(tag, _, _, _, _) => tag.remove_key(&ItemKey::Genre),
        }
    }
    pub fn set_genre(&mut self, genre: &str) {
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
    pub fn compilation(&self) -> bool {
        match self {
            Tagger::M4a(tag, _, _, _, _) => tag.compilation(),
            Tagger::Mp3(tag, _, _, _, _) => tag.get("TCMP").is_some() || tag.get("TCP").is_some(),
            Tagger::Flac(tag, _, _, _, _) => tag.get_vorbis("compilation").is_some(),
            Tagger::Ogg(tag, _, _, _, _) => tag.get_string(&ItemKey::FlagCompilation).is_some(),
        }
    }
    pub fn remove_compilation(&mut self) {
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
    pub fn set_compilation(&mut self) {
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
    pub fn encoder(&self) -> String {
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
    pub fn version(&self) -> String {
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
    pub fn zero(&mut self) {
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
                    ItemKey::RecordingDate,
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
    pub fn save(&mut self, file: &Path) {
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
    pub fn path(&self) -> String {
        match self {
            Tagger::M4a(_, path, _, _, _) => path.to_string(),
            Tagger::Mp3(_, path, _, _, _) => path.to_string(),
            Tagger::Flac(_, path, _, _, _) => path.to_string(),
            Tagger::Ogg(_, path, _, _, _) => path.to_string(),
        }
    }

    // Extension
    // =========
    pub fn extension(&self) -> String {
        match self {
            Tagger::M4a(_, _, extension, _, _) => extension.to_string(),
            Tagger::Mp3(_, _, extension, _, _) => extension.to_string(),
            Tagger::Flac(_, _, extension, _, _) => extension.to_string(),
            Tagger::Ogg(_, _, extension, _, _) => extension.to_string(),
        }
    }

    // Bitrate
    // =======
    pub fn bitrate(&self) -> usize {
        match self {
            Tagger::M4a(_, _, _, bitrate, _) => *bitrate,
            Tagger::Mp3(_, _, _, bitrate, _) => *bitrate,
            Tagger::Flac(_, _, _, bitrate, _) => *bitrate,
            Tagger::Ogg(_, _, _, bitrate, _) => *bitrate,
        }
    }

    // Seconds
    // =======
    pub fn seconds(&self) -> usize {
        match self {
            Self::M4a(_, _, _, _, seconds) => *seconds,
            Self::Mp3(_, _, _, _, seconds) => *seconds,
            Self::Flac(_, _, _, _, seconds) => *seconds,
            Self::Ogg(_, _, _, _, seconds) => *seconds,
        }
    }

    // AudioInfo
    // =========
    pub fn info(&mut self) -> AudioInfo {
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

pub fn tagger_from_file(file: &Path) -> Tagger {
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
