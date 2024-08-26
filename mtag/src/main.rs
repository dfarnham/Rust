use general::split_on;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// clap arg parser
mod argparse;

// Struct AudioInfo
mod audio_info;
use audio_info::AudioInfo;

// Tagger
mod tagger;
use tagger::Tagger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = argparse::get_args();

    let mut max_title_len = 0;
    let mut total_seconds = 0;
    let mut song_info = vec![];

    let audio_files: Vec<_> = args
        .get_many::<std::path::PathBuf>("FILE")
        .expect("argparse to enforce 1 or more input files")
        .collect();

    let audio_files = match audio_files.len() == 1 && audio_files[0].is_dir() {
        true => {
            let mut files = std::fs::read_dir(audio_files[0])?
                .map(|res| res.unwrap().path().to_string_lossy().to_string())
                .filter(|f| f.ends_with(".m4a") || f.ends_with(".mp3") || f.ends_with(".flac") || f.ends_with(".ogg"))
                .map(PathBuf::from)
                .collect::<Vec<_>>();
            files.sort();
            files
        }
        false => audio_files.into_iter().cloned().collect::<Vec<_>>(),
    };

    for file in audio_files {
        let mut modified = false;
        let mut tagger = Tagger::new(&file);

        // Zero -- remove all fields and metatdata
        if args.get_flag("zero") {
            modified = true;
            tagger.zero();
        }

        if let Some(json_str) = args.get_one::<String>("from-json") {
            let mut audio_info: AudioInfo = serde_json::from_str(json_str)?;
            audio_info.path = file.to_string_lossy().to_string();
            modified = tagger.update_from_audio_info(&audio_info) || modified;
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
            tagger.set_compilation();
        } else if args.get_flag("no-compilation") {
            modified = modified || tagger.compilation();
            tagger.remove_compilation();
        }

        if modified {
            println!("*** MODIFIED ***");
            tagger.save(&file);
        }

        let audio_info = tagger.info();
        song_info.push(audio_info.clone());

        if args.get_flag("summary") {
            max_title_len = max_title_len.max(audio_info.title.len());
            total_seconds += audio_info.seconds;
        } else if args.get_flag("json") {
            println!("{}", audio_info.json());
        } else {
            println!("{audio_info}");
        }
    }

    if args.get_flag("summary") && !song_info.is_empty() {
        let header = format!(
            "{} ({}) [{}]",
            song_info[0].album, song_info[0].year, song_info[0].genre
        );
        let space = " ";
        let field_len = max_title_len + 14; // "00. ".len() + " ... ".len() + "00:00".len();
        let art_len = match field_len > song_info[0].artist.len() {
            true => (field_len - song_info[0].artist.len()) / 2,
            false => 0,
        };
        let alb_len = match field_len > header.len() {
            true => (field_len - header.len()) / 2,
            false => 0,
        };
        let playing_time = format!("Playing Time: {}:{:02}", total_seconds / 60, total_seconds % 60);
        let play_len = match field_len > playing_time.len() {
            true => (field_len - playing_time.len()) / 2,
            false => 0,
        };
        println!("{space:>art_len$}{}", song_info[0].artist);
        println!("{space:>alb_len$}{header}");
        println!("{space:>play_len$}{playing_time}\n");

        for f in song_info {
            let song_len = max_title_len - f.title.len();
            println!(
                "{:2}. {} {:.^song_len$}... {}:{:02}",
                f.track_number,
                f.title,
                "",
                f.seconds / 60,
                f.seconds % 60
            );
        }
    }

    Ok(())
}
