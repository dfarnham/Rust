use clap::{crate_description, crate_name, crate_version, value_parser, Arg, ArgMatches, ColorChoice, Command};
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
                .help("File to read, use '-' for standard input")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("downcase")
                .short('d')
                .long("downcase")
                .action(clap::ArgAction::SetTrue)
                .help("Downcase text prior to tokenization"),
        )
        .arg(
            Arg::new("trimmed")
                .short('T')
                .long("trimmed")
                .action(clap::ArgAction::SetTrue)
                .help("Trim leading and trailing whitespace on the tokens"),
        )
        .arg(
            Arg::new("regex")
                .short('r')
                .long("re")
                .value_name("str")
                //.default_value(r"^[\p{Z}]+$")
                //.default_value("^$")
                .help("Discard tokens matching RE"),
        )
        .arg(
            Arg::new("tokenizer")
                .short('t')
                .long("tokenizer")
                .value_name("str")
                .default_value("whitespace")
                .help("Use <str> as the tokenizer (ss, us, uw, ws, rb)"),
        )
        .arg(
            Arg::new("tokenizer_param")
                .short('p')
                .long("param")
                .value_name("str")
                .help("Use <str> to initialize the tokenizer"),
        );
    app.get_matches_from(env::args().collect::<Vec<String>>())
}
