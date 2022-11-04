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

fn fields(text: &str, delim: Option<char>, trim: bool) -> Result<Vec<String>, Box<dyn Error>> {
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
// "-f 1" or "-f1"      =>  FieldType::Index(1)
// "-f 1,3" or "-f1,3"  =>  FieldType::Index(1), FieldType::Index(3)
// "-f 3-" or "-f3-"    =>  FieldType::StartRange(3)
// "-f 3-7" or "-f3-7"  =>  FieldType::ClosedRange(3, 7)
// "-f-1"               =>  FieldType::Last(1)
// "-fr." or "-f r.     =>  FieldType::Index(computed index on Regex header match)
#[derive(Debug)]
enum FieldType {
    Index(usize),
    Last(usize),
    StartRange(usize),
    ClosedRange(usize, usize),
}
impl FieldType {
    fn to_indices(&self, n: usize) -> Vec<usize> {
        let indices = |start: usize, end: usize| -> Vec<usize> {
            (match start <= end {
                true => (start..=end).collect::<Vec<_>>(),
                false => (end..=start).rev().collect::<Vec<_>>(),
            })
            .into_iter()
            .filter(|i| *i > 0 && *i <= n)
            .map(|i| i - 1)
            .collect()
        };
        match self {
            FieldType::Index(a) => indices(*a, *a),
            FieldType::StartRange(a) => indices(*a, n),
            FieldType::ClosedRange(a, b) => indices(*a, *b),
            FieldType::Last(a) => indices(n + 1 - *a, n + 1 - *a),
        }
    }
}

// ==============================================================

fn main() -> Result<(), Box<dyn Error>> {
    let app = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .color(ColorChoice::Auto)
        .arg(
            Arg::new("FILE")
                .help("File to read, use '-' for standard input")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("fields")
                .short('f')
                .help("Field number, range, or regex")
                .long_help(
                    "Field number, range, or regex.\n\
                        Duplicate field numbers will result in duplicate outputs.\n\
                        The output order is determined by the argument order.\n\
                        See option \"-u\" for unique field output\n\n\
                        -f N          # field position index starting at 1\n\
                        -f-N          # field position index from the end\n\
                        -f N-M        # field position range (increasing or decreasing)\n\
                        -f N-         # field position open range to the end\n\
                        -f rREGEX     # field matching regex on the \"file header\"\n\
                        -f LIST (comma separated N,-N,N-M,rREGEX) or use multiple -f\n\n\
                        * -f1,3       # [1,3]\n\
                        * -f1-3       # [1,2,3]\n\
                        * -f3-1       # [3,2,1]\n\
                        * -f1,1       # [1,1]\n\
                        * -f-1        # last field\n\
                        * -f-2        # second to last field\n\
                        * -f'r^.{3}$' # field with exactly 3 characters\n\n\
                        Indexing from the end (-f-N) requires no spaces between\n\
                        the argument and parameter (example: use -f-2 not -f -2)\n\n\
                        -f rREGEX will apply the REGEX to each field of the \"file header\"\n\
                        after splitting on the -d <delim> to determine the index to output,\n\
                        where \"file header\" is simply the first line of input",
                )
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("delim")
                .short('d')
                .help("Input field separator character (defaults to whitespace)")
                .long_help(
                    "Use <delim> as the input field separator character \
                        (defaults to whitespace). \nUse -d '\\t' for TAB",
                ),
        )
        .arg(
            Arg::new("outdelim")
                .short('o')
                .help("The output field separator [default: -d \"delim\" or '\\t']")
                .long_help(
                    "Use <outdelim> as the output field separator.\n\
                     If both input <delim> and <outdelim> are absent, <outdelim> will be '\\t'\n\n\
                     If <outdelim> is absent and <delim> was supplied (see -d), <delim> will be used",
                ),
        )
        .arg(
            Arg::new("uniq")
                .short('u')
                .action(clap::ArgAction::SetTrue)
                .help("Output only unique fields")
                .long_help(
                    "Example: -f 1,3,2,1-3 generates indicies [1,3,2,1,2,3]\n\
                    Using -u will generate indicies [1,3,2]",
                ),
        )
        .arg(
            Arg::new("trim")
                .short('t')
                .action(clap::ArgAction::SetTrue)
                .help("Trim whitespace on the output fields"),
        );
    let args = app.get_matches_from(env::args().collect::<Vec<String>>());

    // ==============================================================
    // ==============================================================

    // a capturing regex:
    //   Label              -f Arg        Captured Text
    //   ----------------------------------------------
    //   {regex}        #  -frPattern  #  captures: "Pattern"
    //   {start}-{end}  #  -f N-M      #  captures: "N" "M"
    //   {start}-       #  -f N        #  captures: "N"
    //   {last}         #  -f-N        #  captures: "N"
    let farg_re = Regex::new(r"^(:?r(?P<regex>.+)|(?P<start>\d+)-(?P<end>\d+)?|-(?P<last>\d+))$")?;

    // sub-split all -f args on ','
    let fargs = args
        .get_many::<String>("fields")
        .expect("required")
        .map(String::from)
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|f| fields(&f, Some(','), true).expect("wtf"));

    // convert the input delimiter from Option<String> to Option<char>
    let delim = match args.get_one::<String>("delim") {
        Some(delim) if delim == "\\t" => Some('\t'),
        Some(delim) => delim.chars().next(),
        _ => None,
    };

    // set `outdelim`
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
            _ => "\t".to_string(),
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
        _ => io::stdin().lock().lines().map(|line| line.expect("wtf")).collect::<Vec<_>>(),
    };

    // "file header" is simply the first line of input
    // return on no input
    let file_header = match lines.is_empty() {
        true => return Ok(()),
        false => fields(&lines[0], delim, args.get_flag("trim"))?,
    };

    // build a list of classified fields
    //   FieldType::Index
    //   FieldType::StartRange
    //   FieldType::ClosedRange
    let mut field_types = vec![];
    for s in fargs {
        match farg_re.captures(&s) {
            Some(capture) => {
                // -f 'rPattern' => List of FieldType::Index
                if let Some(regex) = capture.name("regex") {
                    let re = Regex::new(regex.as_str())?;
                    field_types.extend(
                        file_header
                            .iter()
                            .enumerate()
                            .filter(|(_, txt)| re.is_match(txt))
                            .map(|(i, _)| FieldType::Index(i + 1))
                            .collect::<Vec<_>>(),
                    );
                // -f-N => FieldType::Last
                } else if let Some(last) = capture.name("last") {
                    let last_index = cap_to_index(last)?;
                    field_types.push(FieldType::Last(last_index));
                // -f N-M => FieldType::ClosedRange
                // -f N- => FieldType::StartRange
                } else if let Some(start) = capture.name("start") {
                    let start_index = cap_to_index(start)?;
                    if let Some(end) = capture.name("end") {
                        let end_index = cap_to_index(end)?;
                        field_types.push(FieldType::ClosedRange(start_index, end_index));
                    } else {
                        field_types.push(FieldType::StartRange(start_index));
                    }
                }
            }
            // -f N => FieldType::Index or parse() Err
            None => field_types.push(FieldType::Index(
                s.parse::<usize>().with_context(|| format!("-f {:?}", s))?,
            )),
        }
    }

    // process the input lines
    for line in &lines {
        let line_fields = fields(line, delim, args.get_flag("trim"))?;
        let max_index = line_fields.len();

        // generate indices into `line_fields` to extract
        let indices = field_types.iter().flat_map(|ft| ft.to_indices(max_index));

        // collect the (unique)? set
        let indices = match args.get_flag("uniq") {
            true => indices.unique().collect::<Vec<_>>(),
            false => indices.collect::<Vec<_>>(),
        };

        // output a line of joined fields
        println!(
            "{}",
            indices
                .into_iter()
                .map(|i| line_fields[i].to_owned())
                .collect::<Vec<_>>()
                .join(&outdelim)
        );
    }

    Ok(())
}
