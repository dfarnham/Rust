use anyhow::{Context, Result};
use clap::Parser;
use itertools::Itertools;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::str::FromStr;

// splits the input on a separator character and returns a Vec of the supplied type
pub fn split_on<T>(input: &str, sep: char) -> Result<Vec<T>, Box<dyn Error>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    Ok(input
        .split(sep)
        .map(|s| s.trim().parse::<T>().unwrap())
        .collect::<Vec<T>>())
}

fn fields(text: &str, delim: Option<char>) -> Result<Vec<String>, Box<dyn Error>> {
    match delim {
        Some(c) => split_on::<String>(text, c),
        _ => Ok(text.split_whitespace().map(String::from).collect()),
    }
}

// ==============================================================

// Enum over the input -f arg types
// "-f 1" or "-f1"      =>  FieldType::Index(1)
// "-f 1,3" or "-f1,3"  =>  FieldType::Index(1), FieldType::Index(3)
// "-f 3-" or "-f3-"    =>  FieldType::StartRange(3)
// "-f 3-7" or "-f3-7"  =>  FieldType::FullRange(3, 7)
#[derive(Debug)]
enum FieldType {
    Index(usize),
    StartRange(usize),
    FullRange(usize, usize),
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
            FieldType::FullRange(a, b) => indices(*a, *b),
        }
    }
}

// ==============================================================

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about)]
    struct Args {
        /// file|stdin, filename of "-" implies stdin
        #[clap(multiple_values = false)]
        file: Option<std::path::PathBuf>,

        /// Field number, comma separated list, or range. Duplicate occurrences of
        /// a field number will result in duplicate outputs. The output order is determined
        /// by the argument order or field range. Supply -u for unique field output
        #[clap(short)]
        field: Option<Vec<String>>,

        /// Use <DELIM> as the input field separator character (defaults to whitespace).
        /// -d '\t' will be interpreted as a TAB
        #[clap(short)]
        delim: Option<String>,

        /// Use <OUTDELIM> as the output field separator (defaults to input delimeter).
        /// If the input delimeter is absent the output delimeter is set to TAB '\t'
        #[clap(short)]
        outdelim: Option<String>,

        /// <REGEX> applied against each field in the "file header" after
        /// splitting on the input delimeter, where "file header" is simply
        /// the first line of input. Processed after -f
        #[clap(short)]
        regex: Option<Vec<String>>,

        /// Output only unique fields
        #[clap(short)]
        uniq: bool,
    }
    let args = Args::parse();

    // ==============================================================
    // ==============================================================

    // convert the input delimiter from Option<String> to Option<char>
    let delim = match args.delim {
        Some(delim) if delim == "\\t" => Some('\t'),
        Some(delim) => delim.chars().next(),
        _ => None,
    };

    // `outdelim` is used on each println!() in a join() over the selected fields
    let outdelim = match args.outdelim {
        Some(outdelim) if outdelim == "\\t" => "\t".to_string(),
        Some(outdelim) if outdelim == "\\n" => "\n".to_string(),
        Some(outdelim) => outdelim,
        // copy the input delimeter or set to a tab
        None => match delim {
            Some(delim) => delim.to_string(),
            _ => "\t".to_string(),
        },
    };

    // a capturing regex for "start-end" ranges and open ended ranges: ex "3-5", "3-"
    let digit_range_re = Regex::new(r"^(?P<start>\d+)-(?P<end>\d+)?$")?;

    // sub-split all -f args on ',' and classify each as one of:
    //   FieldType::Index
    //   FieldType::StartRange
    //   FieldType::FullRange
    let mut field_types = match args.field {
        Some(ref fstrs) => fstrs
            .iter()
            .flat_map(|f| split_on::<String>(f, ',').unwrap_or_default())
            .filter_map(|s| match digit_range_re.captures(&s) {
                Some(capture) => {
                    let start = capture["start"].parse::<usize>().ok()?;
                    match capture.name("end") {
                        Some(c) => Some(FieldType::FullRange(start, c.as_str().parse::<usize>().ok()?)),
                        None => Some(FieldType::StartRange(start)),
                    }
                }
                // simple string number
                None => Some(FieldType::Index(s.parse::<usize>().ok()?)),
            })
            .collect::<Vec<_>>(),
        None => vec![],
    };

    // read input lines from a filename or stdin and convert the lines to a Vec<String>
    let lines = match args.file {
        Some(file) if file.as_os_str() != "-" => io::BufReader::new(
            File::open(&file).with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?,
        )
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<_>>(),
        _ => io::stdin().lock().lines().map(|line| line.unwrap()).collect::<Vec<_>>(),
    };

    if !lines.is_empty() {
        // test all -r args against the "file header" (first line of input)
        // adding the index of any matches as a FieldType::Index to `field_types`
        if let Some(ref args_regex) = args.regex {
            let header = fields(&lines[0], delim)?;
            for regex_str in args_regex {
                let re = Regex::new(regex_str)?;
                header
                    .iter()
                    .enumerate()
                    .filter(|(_, txt)| re.is_match(txt))
                    .for_each(|(i, _)| field_types.push(FieldType::Index(i + 1)));
            }
        }
    }

    // process the input lines emitting output-delimeter joined fields
    for line in &lines {
        let line_fields = fields(line, delim)?;
        let max_index = line_fields.len();

        // generate indices of `line_fields` to extract
        let indices = field_types.iter().flat_map(|ft| ft.to_indices(max_index));
        let indices = match args.uniq {
            true => indices.unique().collect::<Vec<_>>(),
            false => indices.collect::<Vec<_>>(),
        };

        // output a line
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
