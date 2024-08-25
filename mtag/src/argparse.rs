use clap::{
    crate_description, crate_name, crate_version, value_parser, Arg, ArgAction, ArgMatches, ColorChoice, Command,
};
use std::env;
use std::path::PathBuf;

pub fn get_args() -> ArgMatches {
    let app = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .color(ColorChoice::Auto)
        .max_term_width(100)
        .arg(
            Arg::new("FILE")
                .help("Audio file(s)")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .num_args(1..)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("artist")
                .short('a')
                .long("artist")
                .value_name("artist")
                .value_parser(clap::builder::StringValueParser::new())
                .help("Set <artist>, empty value removes <artist>"),
        )
        .arg(
            Arg::new("album")
                .short('A')
                .long("album")
                .value_name("album")
                .value_parser(clap::builder::StringValueParser::new())
                .help("Set <album>, empty value removes <album>"),
        )
        .arg(
            Arg::new("album-artist")
                .short('b')
                .long("album-artist")
                .value_name("album artist")
                .value_parser(clap::builder::StringValueParser::new())
                .help("Set <album artist>, empty value removes <album artist>"),
        )
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("title")
                .value_parser(clap::builder::StringValueParser::new())
                .help("Set <title>, empty value removes <title>"),
        )
        .arg(
            Arg::new("trkn")
                .short('T')
                .long("trkn")
                .value_name("trkn")
                .value_parser(clap::builder::StringValueParser::new())
                .conflicts_with_all(["track-number", "track-total"])
                .help("Sets both <track number> and <track total>, ex. -T 2/8"),
        )
        .arg(
            Arg::new("track-number")
                .short('n')
                .long("track-number")
                .value_name("track number")
                .value_parser(value_parser!(usize))
                .help("Set <track number>, 0 removes <track number>"),
        )
        .arg(
            Arg::new("track-total")
                .short('N')
                .long("track-total")
                .value_name("track total")
                .value_parser(value_parser!(usize))
                .help("Set <track total>, 0 removes <track total>"),
        )
        .arg(
            Arg::new("disc-number")
                .short('d')
                .long("disc-number")
                .value_name("disc number")
                .value_parser(value_parser!(usize))
                .help("Set <disc number>, 0 removes <disc number>"),
        )
        .arg(
            Arg::new("disc-total")
                .short('D')
                .long("dist-total")
                .value_name("disc total")
                .value_parser(value_parser!(usize))
                .help("Set <disc total>, 0 removes <disc total>"),
        )
        .arg(
            Arg::new("year")
                .short('y')
                .long("year")
                .value_name("year")
                .value_parser(value_parser!(usize))
                .help("Set <year>, 0 removes <year>"),
        )
        .arg(
            Arg::new("genre")
                .short('g')
                .long("genre")
                .value_name("genre")
                .value_parser(clap::builder::StringValueParser::new())
                .action(ArgAction::Append)
                .help("Set <genre>, empty value removes <genre>"),
        )
        .arg(
            Arg::new("compilation")
                .short('c')
                .long("compilation")
                .conflicts_with("no-compilation")
                .action(ArgAction::SetTrue)
                .help("Set <compilation flag>"),
        )
        .arg(
            Arg::new("no-compilation")
                .short('C')
                .long("no-compilation")
                .conflicts_with("compilation")
                .action(ArgAction::SetTrue)
                .help("Remove <compilation flag>"),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .action(ArgAction::SetTrue)
                .help("output tags as JSON"),
        )
        .arg(
            Arg::new("from-json")
                .short('J')
                .long("from-json")
                .value_name("JSON")
                .value_parser(clap::builder::StringValueParser::new())
                .conflicts_with_all([
                    "artist",
                    "album",
                    "album-artist",
                    "title",
                    "trkn",
                    "track-number",
                    "track-total",
                    "disc-number",
                    "disc-total",
                    "year",
                    "genre",
                    "compilation",
                    "no-compilation",
                    "json",
                ])
                .help("input tags from JSON"),
        )
        .arg(
            Arg::new("summary")
                .short('s')
                .long("summary")
                .action(ArgAction::SetTrue)
                .help("Display a summary"),
        )
        .arg(
            Arg::new("zero")
                .short('z')
                .long("zero")
                .action(ArgAction::SetTrue)
                .help("Remove all tags"),
        );
    app.get_matches_from(env::args().collect::<Vec<String>>())
}
