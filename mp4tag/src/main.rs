use anyhow::{Context, Result};
use mp4ameta::Tag;

// clap arg parser
mod argparse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = argparse::get_args();

    let m4a_file = args.get_one::<std::path::PathBuf>("FILE");

    // create a Tag object from the m4a file
    let mut tag = match m4a_file {
        Some(file) => {
            Tag::read_from_path(file).with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?
        }
        _ => unreachable!(),
    };

    // Artist
    if let Some(artist) = args.get_one::<String>("artist") {
        match artist.is_empty() {
            true => tag.remove_artists(),
            false => tag.set_artist(artist),
        }
    }

    // Album
    if let Some(album) = args.get_one::<String>("album") {
        match album.is_empty() {
            true => tag.remove_album(),
            false => tag.set_album(album),
        }
    }

    // Album Artist
    if let Some(album_artist) = args.get_one::<String>("album-artist") {
        match album_artist.is_empty() {
            true => tag.remove_album_artists(),
            false => tag.set_album_artist(album_artist),
        }
    }

    // Title
    if let Some(title) = args.get_one::<String>("title") {
        match title.is_empty() {
            true => tag.remove_title(),
            false => tag.set_title(title),
        }
    }

    // Track Number
    if let Some(track_number) = args.get_one::<u16>("track-number") {
        match track_number == &0 {
            true => tag.remove_track_number(),
            false => tag.set_track_number(*track_number),
        }
    }

    // Total Tracks
    if let Some(total_tracks) = args.get_one::<u16>("total-tracks") {
        match total_tracks == &0 {
            true => tag.remove_total_tracks(),
            false => tag.set_total_tracks(*total_tracks),
        }
    }

    // Disc Number
    if let Some(disc_number) = args.get_one::<u16>("disc-number") {
        match disc_number == &0 {
            true => tag.remove_disc_number(),
            false => tag.set_disc_number(*disc_number),
        }
    }

    // Total Discs
    if let Some(total_discs) = args.get_one::<u16>("total-discs") {
        match total_discs == &0 {
            true => tag.remove_total_discs(),
            false => tag.set_total_discs(*total_discs),
        }
    }

    // Year
    if let Some(year) = args.get_one::<String>("year") {
        match year.is_empty() || year == "0" {
            true => tag.remove_year(),
            false => tag.set_year(year),
        }
    }

    // Genre
    if let Some(genre) = args.get_one::<String>("genre") {
        match genre.is_empty() {
            true => tag.remove_genres(),
            false => tag.set_genre(genre),
        }
    }

    // Compilation Flag
    if args.get_flag("compilation") {
        tag.set_compilation();
    }
    if args.get_flag("no-compilation") {
        tag.remove_compilation();
    }

    // Zero -- remove all fields and metatdata
    if args.get_flag("zero") {
        tag.remove_advisory_rating();
        tag.remove_album();
        tag.remove_album_artists();
        tag.remove_artists();
        tag.remove_artworks();
        tag.remove_categories();
        tag.remove_comments();
        tag.remove_compilation();
        tag.remove_composers();
        tag.remove_copyright();
        tag.remove_custom_genres();
        tag.remove_descriptions();
        tag.remove_disc();
        tag.remove_disc_number();
        tag.remove_encoder();
        tag.remove_gapless_playback();
        tag.remove_genres();
        tag.remove_groupings();
        tag.remove_isrc();
        tag.remove_keywords();
        tag.remove_lyricists();
        tag.remove_lyrics();
        tag.remove_media_type();
        tag.remove_movement();
        tag.remove_movement_count();
        tag.remove_movement_index();
        tag.remove_show_movement();
        tag.remove_standard_genres();
        tag.remove_title();
        tag.remove_total_discs();
        tag.remove_total_tracks();
        tag.remove_track_number();
        tag.remove_tv_episode();
        tag.remove_tv_episode_name();
        tag.remove_tv_network_name();
        tag.remove_tv_season();
        tag.remove_tv_show_name();
        tag.remove_work();
        tag.remove_year();
    }

    // Write tags to the file
    Ok(tag.write_to_path(m4a_file.expect("write file error").clone().into_os_string())?)
}
