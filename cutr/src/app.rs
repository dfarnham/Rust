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
                .help("File to read, use '-' for standard input")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("fields")
                .short('f')
                .value_name("field_spec")
                .help("[-]number, range, or regex (use `--help` for more detail)")
                .long_help(
                    "[-]number, range, or regex\n\
                        \n\
                        <field_spec> syntax:\n\
                        -f N          # position index starting at 1\n\
                        -f-N          # position index counting from the end\n\
                        -f N-M        # position range (increasing or decreasing)\n\
                        -f N-         # position open range to the end\n\
                        -f r/REGEX/   # regex match over fields on the \"file header\"\n\
                        -f R/REGEX/   # regex match over fields on all input\n\
                        -f <field_spec>[,<field_spec>,...]
                        \n\
                        Each <field_spec> is represented by one or more enumerations\n\
                        \tFieldSpec::Index(a)\n\
                        \tFieldSpec::OpenRange(a)\n\
                        \tFieldSpec::ClosedRange(a, b)\n\
                        \tFieldSpec::Last(a)\n\
                        \tFieldSpec::RegularExpression(re)\n\
                        \n\
                        The combinded list of enumerations operate over the tokenized input\n\
                        producing lists of indices, the combination may contain repeated elements.\n\
                        See option \"-u\" for applying a unique filter\n\
                        \n\
                        Examples\n\
                        * -f1,3       # [1,3]\n\
                        * -f1-3       # [1,2,3]\n\
                        * -f3-1       # [3,2,1]\n\
                        * -f1,1       # [1,1]\n\
                        * -f1 -f2     # [1,2]\n\
                        * -f-1        # last index\n\
                        * -f1,-1      # first and last index\n\
                        * -f-2        # second to last index\n\
                        * -f'r/^.{3}$/' # index of fields with exactly 3 characters in \"file header\"\n\
                        * -f'R/^.{3}$/' # index of fields with exactly 3 characters (matched against all data)\n\
                        \n\
                        More Information\n\
                        -f-N must be specified without spaces; use -f-2 not -f -2\n\
                        \n\
                        -fr, -fR, can optionally specify the pattern between slashes (/) as -fr//, -fR//\n\
                        \n\
                        When using -f[rR] in a list, comma (,) is treated as a <field_spec> separator not a\n\
                        component of the Regular Expression.\n\
                        Isolate the REGEX into a separate -f[rR]// to avoid <filed_spec> list splitting
                        \n\
                        Ex: select the first field, header fields beginning with \"foo\", and the last field\n\
                        \t-f1,r^foo,-1\n\
                        \t\tor\n\
                        \t-f1 -fr/^foo/ -f-1",
                )
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("input_delim")
                .short('d')
                .value_name("char")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Input field separator character, defaults to whitespace")
                .long_help(
                    "Use <char> as the input field separator character, the default is whitespace \n\
                    where consecutive spaces and tabs count as one single field separator.\n\n\
                    Use -T or -d'\\t' for TAB",
                ),
        )
        .arg(
            Arg::new("tab")
                .short('T')
                .conflicts_with("input_delim")
                .action(clap::ArgAction::SetTrue)
                .help("Short for -d'\\t'"),
        )
        .arg(
            Arg::new("output_delim")
                .short('o')
                .value_name("str")
                .help("Use <str> as the output field separator, default is to use -d, or '\\t'"),
        )
        .arg(
            Arg::new("sorted")
                .short('s')
                .action(clap::ArgAction::SetTrue)
                .help("Output fields in index-sorted order"),
        )
        .arg(
            Arg::new("uniq")
                .short('u')
                .action(clap::ArgAction::SetTrue)
                .help("Output only unique fields")
                .long_help(
                    "Example: -f 1,3,1,2,1-3 specifies indices [1,3,1,2,1,2,3]\n\
                    Using -u will yield indices [1,3,2]",
                ),
        )
        .arg(
            Arg::new("trim")
                .short('t')
                .action(clap::ArgAction::SetTrue)
                .help("Trim whitespace in data parsing"),
        )
        .arg(
            Arg::new("number")
                .short('n')
                .action(clap::ArgAction::SetTrue)
                .help("Add a beginning field on output denoting the line number of the input"),
        )
        .arg(
            Arg::new("zero")
                .short('z')
                .action(clap::ArgAction::SetTrue)
                .help("Don't output empty lines"),
        );
    app.get_matches_from(env::args().collect::<Vec<String>>())
}
