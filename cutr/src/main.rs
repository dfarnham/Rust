use anyhow::{Context, Result};
use clap::{crate_description, crate_name, crate_version, value_parser, Arg, ArgAction, ColorChoice, Command};
use itertools::Itertools;
use regex::{Match, Regex};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::str::FromStr;

// splits and optionally trims the input String on a separator character
// returns a Vec of parse::<T>() over the splits
fn split_on<T>(text: &str, sep: char, trim: bool) -> Result<Vec<T>, Box<dyn Error>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: 'static,
{
    let mut parsed_splits = vec![];
    for mut s in text.split(sep) {
        if trim {
            s = s.trim();
        }
        parsed_splits.push(s.parse::<T>()?)
    }
    Ok(parsed_splits)
}

fn tokens(text: &str, delim: Option<char>, trim: bool) -> Result<Vec<String>, Box<dyn Error>> {
    match delim {
        Some(c) => split_on::<String>(text, c, trim),
        _ => Ok(text.split_whitespace().map(String::from).collect()),
    }
}

// helper function to return the number held in the RE captured match
fn cap_to_index(cap: Match) -> Result<usize, Box<dyn Error>> {
    Ok(cap
        .as_str()
        .parse::<usize>()
        .with_context(|| format!("regex capture error? -f {:?}", cap))?)
}

// ==============================================================

// Enum over the input -f arg types
// "-f 1" or "-f1"      =>  FieldSpec::Index(1)
// "-f 1,3" or "-f1,3"  =>  FieldSpec::Index(1), FieldSpec::Index(3)
// "-f 3-" or "-f3-"    =>  FieldSpec::StartRange(3)
// "-f 3-7" or "-f3-7"  =>  FieldSpec::ClosedRange(3, 7)
// "-f-1"               =>  FieldSpec::Last(1)
// "-fr." or "-f r.     =>  computed indices on Regex header matches into => List(FieldSpec::Index)
// "-fR." or "-f R.     =>  FieldSpec::RegularExpression(re), computed indices on Regex data matches into => List(FieldSpec::Index)
#[derive(Debug)]
enum FieldSpec {
    Index(usize),
    Last(usize),
    StartRange(usize),
    ClosedRange(usize, usize),
    RegularExpression(Regex),
}
impl FieldSpec {
    fn indices(&self, tokens: &[String]) -> Vec<usize> {
        let indices = |start: usize, end: usize| -> Vec<usize> {
            (match start <= end {
                true => (start..=end).collect::<Vec<_>>(),
                false => (end..=start).rev().collect::<Vec<_>>(),
            })
            .into_iter()
            .filter(|i| *i > 0 && *i <= tokens.len())
            .map(|i| i - 1)
            .collect()
        };
        match self {
            FieldSpec::Index(a) => indices(*a, *a),
            FieldSpec::StartRange(a) => indices(*a, tokens.len()),
            FieldSpec::ClosedRange(a, b) => indices(*a, *b),
            FieldSpec::Last(a) => indices(tokens.len() + 1 - *a, tokens.len() + 1 - *a),
            FieldSpec::RegularExpression(re) => tokens
                .iter()
                .enumerate()
                .filter(|(_, txt)| re.is_match(txt))
                .flat_map(|(i, _)| indices(i + 1, i + 1))
                .collect(),
        }
    }
}

// ==============================================================

fn main() -> Result<(), Box<dyn Error>> {
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
                .help("[-]number, range, or regex [--help for details]")
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
                        \tFieldSpec::StartRange(a)\n\
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
                        * -f-2        # second to last index\n\
                        * -f'r/^.{3}$/' # index of fields with exactly 3 characters in \"file header\"\n\
                        * -f'R/^.{3}$/' # index of fields with exactly 3 characters (matched against all data)\n\
                        \n\
                        More Information\n\
                        -f-N must be specified without spaces; use -f-2 not -f -2\n\
                        \n\
                        -fr, -fR, can optionally specifty the pattern between slashes (/) as -fr//, -fR//\n\
                        \n\
                        When using -f[rR] in a list, comma (,) is treated as a <field_spec> separator not a\n\
                        component of the Regular Expression. Use a separate -f[rR] if comma is in the pattern\n\
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
            Arg::new("delim")
                .short('d')
                .value_name("char")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Input field separator character. Defaults to whitespace")
                .long_help(
                    "Use <char> as the input field separator character, the default is whitespace \n\
                    where consecutive spaces and tabs count as one single field separator.\n\n\
                    Use -T or -d '\\t' for TAB",
                ),
        )
        .arg(
            Arg::new("outdelim")
                .short('o')
                .value_name("str")
                .help("Use <str> as the output field separator. Default is to use -d, or '\\t'"),
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
            Arg::new("sorted")
                .short('s')
                .action(clap::ArgAction::SetTrue)
                .help("Output fields in index-sorted order"),
        )
        .arg(
            Arg::new("tab")
                .short('T')
                .conflicts_with("delim")
                .action(clap::ArgAction::SetTrue)
                .help("Short for -d'\\t'"),
        )
        .arg(
            Arg::new("trim")
                .short('t')
                .action(clap::ArgAction::SetTrue)
                .help("Trim whitespace in data parsing"),
        )
        .arg(
            Arg::new("zero")
                .short('z')
                .action(clap::ArgAction::SetTrue)
                .help("Don't output empty lines"),
        );
    let args = app.get_matches_from(env::args().collect::<Vec<String>>());

    // ==============================================================
    // ==============================================================

    // a capturing regex:
    //   Label             -f Arg         Captured Text
    //   ----------------------------------------------
    //   {r_hdr}        |  -frPattern  |  "Pattern"
    //   {r_all}        |  -fRPattern  |  "Pattern"
    //   {start}-{end}  |  -f N-M      |  captures: "N" "M"
    //   {start}-       |  -f N        |  captures: "N"
    //   {last}         |  -f-N        |  captures: "N"
    let farg_re = Regex::new(r"^(:?r(?P<r_hdr>.+)|R(?P<r_all>.+)|(?P<start>\d+)-(?P<end>\d+)?|-(?P<last>\d+))$")?;

    // a capturing regex for [rR] expressions between slashes (/). e.g. -fr/foo/
    //   Label        -f Arg                           Captured Text
    //   -----------------------------------------------------------
    //   {r_type}  |  -fr/Pattern/ or -fR/Pattern/  |  [rR]
    //   {r_exp}   |  -fr/Pattern/ or -fR/Pattern/  |  "Pattern"
    let farg_slash_re = Regex::new(r"^(?P<r_type>[rR])/(?P<r_exp>.+)/$")?;

    // sub-split all non-regex -f args on comma (,)
    let mut fargs = vec![];
    for fstr in args.get_many::<String>("fields").expect("required") {
        match farg_slash_re.captures(fstr) {
            Some(capture) => {
                fargs.push(capture["r_type"].to_owned() + &capture["r_exp"]);
            }
            _ => {
                fargs.extend(fstr.split(',').map(String::from).collect::<Vec<_>>());
            }
        }
    }

    // set `delim` to Option<char>
    let delim = match args.get_one::<String>("delim") {
        Some(delim) if delim == "\\t" => Some('\t'),
        Some(delim) => delim.chars().next(),
        // check option -T
        _ => match args.get_flag("tab") {
            true => Some('\t'),
            false => None,
        },
    };

    // set `outdelim` to String
    //   handle special inputs TAB, NL
    //   if `outdelim` was provided use it
    //   else if `delim` was provided set to `delim`, otherwise set to TAB
    let outdelim = match args.get_one::<String>("outdelim") {
        Some(outdelim) if outdelim == "\\t" => "\t".to_string(),
        Some(outdelim) if outdelim == "\\n" => "\n".to_string(),
        Some(outdelim) => outdelim.to_string(),
        // copy the input delimeter or set to a tab
        None => match delim {
            Some(delim) => delim.to_string(),
            _ => '\t'.to_string(),
        },
    };

    // read input lines from a filename or stdin and collect into a Vec<String>
    let lines = match args.get_one::<PathBuf>("FILE") {
        Some(file) if file.as_os_str() != "-" => io::BufReader::new(
            File::open(file).with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?,
        )
        .lines()
        .map(|line| line.expect("wtf"))
        .collect::<Vec<_>>(),
        _ => io::stdin()
            .lock()
            .lines()
            .map(|line| line.expect("wtf"))
            .collect::<Vec<_>>(),
    };
    let field_tokens = |n: usize| tokens(&lines[n], delim, args.get_flag("trim"));

    // set `file_header` to the field tokens of the first line or return
    let file_header = match lines.is_empty() {
        true => return Ok(()),
        false => field_tokens(0)?,
    };

    // convert `fargs` to a list of field classifications (enums)
    //   FieldSpec::Index
    //   FieldSpec::StartRange
    //   FieldSpec::ClosedRange
    //   FieldSpec::RegularExpression
    let mut field_enums = vec![];
    for s in fargs {
        match farg_re.captures(&s) {
            Some(capture) => {
                // -f 'rPattern' => List of FieldSpec::Index
                if let Some(regex) = capture.name("r_hdr") {
                    let re = Regex::new(regex.as_str())?;
                    field_enums.extend(
                        file_header
                            .iter()
                            .enumerate()
                            .filter(|(_, txt)| re.is_match(txt))
                            .map(|(i, _)| FieldSpec::Index(i + 1))
                            .collect::<Vec<_>>(),
                    );
                // -f 'RPattern' => FieldSpec::RegularExpression
                } else if let Some(regex) = capture.name("r_all") {
                    let re = Regex::new(regex.as_str())?;
                    field_enums.push(FieldSpec::RegularExpression(re));
                // -f-N => FieldSpec::Last
                } else if let Some(last) = capture.name("last") {
                    let last_index = cap_to_index(last)?;
                    field_enums.push(FieldSpec::Last(last_index));
                // -f N-M => FieldSpec::ClosedRange
                // -f N- => FieldSpec::StartRange
                } else if let Some(start) = capture.name("start") {
                    let start_index = cap_to_index(start)?;
                    if let Some(end) = capture.name("end") {
                        let end_index = cap_to_index(end)?;
                        field_enums.push(FieldSpec::ClosedRange(start_index, end_index));
                    } else {
                        field_enums.push(FieldSpec::StartRange(start_index));
                    }
                }
            }
            // -f N => FieldSpec::Index or parse() Err
            None => field_enums.push(FieldSpec::Index(
                s.parse::<usize>().with_context(|| format!("-f {:?}", s))?,
            )),
        }
    }

    // process the input lines
    for i in 0..lines.len() {
        let field_tokens = field_tokens(i)?;

        // generate indices into `field_tokens` to extract
        let indices = field_enums.iter().flat_map(|f| f.indices(&field_tokens));

        // collect the (unique)? (sorted)? set of indices
        let indices = match args.get_flag("uniq") {
            true => match args.get_flag("sorted") {
                true => indices.unique().sorted().collect::<Vec<_>>(),
                false => indices.unique().collect::<Vec<_>>(),
            },
            false => match args.get_flag("sorted") {
                true => indices.sorted().collect::<Vec<_>>(),
                false => indices.collect::<Vec<_>>(),
            },
        };

        // output a line of joined fields
        if !indices.is_empty() || !args.get_flag("zero") {
            println!(
                "{}",
                indices
                    .into_iter()
                    .map(|i| field_tokens[i].to_owned())
                    .collect::<Vec<_>>()
                    .join(&outdelim)
            );
        }
    }

    Ok(())
}
