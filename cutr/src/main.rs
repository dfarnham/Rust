use anyhow::{Context, Result};
use clap::Parser;
use itertools::Itertools;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::str::FromStr;

// splits and trims (if requested) the input on a separator character and returns a Vec of the supplied type
pub fn split_on<T>(input: &str, sep: char, trim: bool) -> Result<Vec<T>, Box<dyn Error>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    let splits = input.split(sep);
    Ok(match trim {
        true => splits.map(|s| s.trim().parse::<T>().unwrap()).collect::<Vec<T>>(),
        false => splits.map(|s| s.parse::<T>().unwrap()).collect::<Vec<T>>(),
    })
}

fn fields(text: &str, delim: Option<char>, trim: bool) -> Result<Vec<String>, Box<dyn Error>> {
    match delim {
        Some(c) => split_on::<String>(text, c, trim),
        _ => Ok(text.split_whitespace().map(String::from).collect()),
    }
}

// ==============================================================

// Enum over the input -f arg types
// "-f 1" or "-f1"      =>  FieldType::Index(1)
// "-f 1,3" or "-f1,3"  =>  FieldType::Index(1), FieldType::Index(3)
// "-f 3-" or "-f3-"    =>  FieldType::StartRange(3)
// "-f 3-7" or "-f3-7"  =>  FieldType::FullRange(3, 7)
// "-f-1"               =>  FieldType::Last(1)
// "-fr." or "-f r.     =>  FieldType::Index(computed index on Regex header match)
#[derive(Debug)]
enum FieldType {
    Index(usize),
    Last(usize),
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
            FieldType::Last(a) => indices(n + 1 - *a, n + 1 - *a),
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
        #[clap(short, required = true)]
        field: Option<Vec<String>>,

        /// Use <DELIM> as the input field separator character (defaults to whitespace).
        /// -d '\t' will be interpreted as a TAB
        #[clap(short)]
        delim: Option<String>,

        /// Use <OUTDELIM> as the output field separator (defaults to input delimeter).
        /// If the input delimeter is absent the output delimeter is set to TAB '\t'
        #[clap(short)]
        outdelim: Option<String>,

        /*
        /// <REGEX> applied against each field in the "file header" after
        /// splitting on the input delimeter, where "file header" is simply
        /// the first line of input.
        #[clap(short)]
        regex: Option<Vec<String>>,
        */
        /// Output only unique fields
        #[clap(short)]
        uniq: bool,

        /// Trim whitespace on selected field data
        #[clap(short)]
        trim: bool,
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

    // "file header" is simply the first line of input
    let file_header = match lines.is_empty() {
        true => return Ok(()),
        false => fields(&lines[0], delim, args.trim)?,
    };

    // a capturing regex for "start-end" ranges and open ended ranges: ex "3-5", "3-"
    let farg_re = Regex::new(r"^(:?r(?P<regex>.+)|(?P<start>\d+)-(?P<end>\d+)?|-(?P<last>\d+))$")?;

    // sub-split all -f args on ',' and classify each as one of:
    //   FieldType::Index
    //   FieldType::StartRange
    //   FieldType::FullRange
    let mut field_types = vec![];
    if let Some(ref fargs) = args.field {
        let fstrs = fargs
            .iter()
            .flat_map(|f| split_on::<String>(f, ',', true).unwrap_or_default());
        for s in fstrs {
            match farg_re.captures(&s) {
                Some(capture) => {
                    if let Some(regex) = capture.name("regex") {
                        let re = Regex::new(regex.as_str())?;
                        file_header
                            .iter()
                            .enumerate()
                            .filter(|(_, txt)| re.is_match(txt))
                            .for_each(|(i, _)| field_types.push(FieldType::Index(i + 1)));
                    } else if let Some(last) = capture.name("last") {
                        let last_index = last
                            .as_str()
                            .parse::<usize>()
                            .with_context(|| format!("regex capture error? -f -{:?}", last))?;
                        field_types.push(FieldType::Last(last_index));
                    } else if let Some(start) = capture.name("start") {
                        let start_index = start
                            .as_str()
                            .parse::<usize>()
                            .with_context(|| format!("regex capture error? -f {:?}-", start))?;
                        if let Some(end) = capture.name("end") {
                            let end_index = end
                                .as_str()
                                .parse::<usize>()
                                .with_context(|| format!("regex capture error? -f {}-{:?}", start_index, end))?;
                            field_types.push(FieldType::FullRange(start_index, end_index));
                        } else {
                            field_types.push(FieldType::StartRange(start_index));
                        }
                    }
                }
                // simple string number
                None => field_types.push(FieldType::Index(
                    s.parse::<usize>().with_context(|| format!("-f {:?}", s))?,
                )),
            }
        }
    }

    // process the input lines emitting output-delimeter joined fields
    for line in &lines {
        let line_fields = fields(line, delim, args.trim)?;
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
