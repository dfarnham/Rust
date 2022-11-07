use anyhow::{Context, Result};
use itertools::Itertools;
use regex::{Match, Regex};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::str::FromStr;

mod app;

mod utilities;
use crate::utilities::{reset_sigpipe, tokens};

mod field_spec;
use crate::field_spec::FieldSpec;

// ==============================================================
// helper function to return the <usize> in a Regex captured match
fn captured_index(cap: Match) -> Result<usize, Box<dyn Error>> {
    Ok(cap
        .as_str()
        .parse::<usize>()
        .with_context(|| format!("regex capture error? -f {:?}", cap))?)
}
// ==============================================================

fn main() -> Result<(), Box<dyn Error>> {
    // behave like a typical unix utility
    reset_sigpipe()?;

    // parse command line arguments
    let args = app::get_args();

    // extract state switches, all default to false
    let (tab, trim, uniq, sorted, number, compliment, zero) = (
        args.get_flag("tab"),        // -T
        args.get_flag("trim"),       // -t
        args.get_flag("uniq"),       // -u
        args.get_flag("sorted"),     // -s
        args.get_flag("number"),     // -n
        args.get_flag("compliment"), // -c
        args.get_flag("zero"),       // -z
    );

    // a capturing regex:
    //   Label             -f Arg         Captured Text
    //   ----------------------------------------------
    //   {r_hdr}        |  -frPattern  |  "Pattern"
    //   {r_data}       |  -fRPattern  |  "Pattern"
    //   {start}-{end}  |  -f N-M      |  captures: "N" "M"
    //   {start}-       |  -f N        |  captures: "N"
    //   {last}         |  -f-N        |  captures: "N"
    //let farg_re = Regex::new(r"^(:?r(?P<r_hdr>.+)|R(?P<r_data>.+)|(?P<start>\d+)-(?P<end>\d+)?|-(?P<last>\d+))$")?;
    let farg_re = Regex::new(
        r"(?x)
        ^(?:
            r (?P<r_hdr>.+) |                 # header pattern
            R (?P<r_data>.+) |                # data pattern
            (?P<start>\d+) - (?P<end>\d+)? |  # ranges N-M or N-
            -(?P<last>\d+)                    # last index -N
        )$",
    )?;

    // a capturing regex for [rR] expressions between slashes (/). e.g. -fr/foo/
    //   Label        -f Arg                           Captured Text
    //   -----------------------------------------------------------
    //   {r_type}  |  -fr/Pattern/ or -fR/Pattern/  |  [rR]
    //   {r_exp}   |  -fr/Pattern/ or -fR/Pattern/  |  "Pattern"
    //let farg_slash_re = Regex::new(r"^(?P<r_type>[rR])/(?P<r_exp>.+)/$")?;
    let farg_slash_re = Regex::new(
        r"(?x)
        ^ (?P<r_type>[rR])  # starts with [rR]
        / (?P<r_exp>.+) /$  # ends with /pattern/",
    )?;

    // normalize isolated -f[rR] or sub-split on comma (,)
    let mut fargs = vec![];
    for fstr in args.get_many::<String>("fields").expect("required") {
        match farg_slash_re.captures(fstr) {
            Some(capture) => fargs.push(capture["r_type"].to_owned() + &capture["r_exp"]),
            _ => fargs.extend(fstr.split(',').map(String::from).collect::<Vec<_>>()),
        }
    }

    // set `input_delim` to Option<char>
    let input_delim = match args.get_one::<String>("input_delim") {
        Some(delim) if delim == "\\t" => Some('\t'),
        Some(delim) => delim.chars().next(),
        // check option -T
        _ => match tab {
            true => Some('\t'),
            false => None,
        },
    };

    // set `output_delim` to String
    //   handle special inputs representing TAB, NL
    //   Use <str> as the output field separator.
    //   Default is to use -d, or '\t'
    let output_delim = match args.get_one::<String>("output_delim") {
        Some(o) if o == "\\t" => "\t".to_string(),
        Some(o) if o == "\\n" => "\n".to_string(),
        Some(o) => o.to_string(),
        // copy the input delimeter or set to a tab
        None => match input_delim {
            Some(d) => d.to_string(),
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
    let line_tokens = |n: usize| tokens(&lines[n], input_delim, trim);

    // set `file_header` to the field tokens of the first line or return
    let file_header = match lines.is_empty() {
        true => return Ok(()),
        false => line_tokens(0)?,
    };

    // convert `fargs` to a list of field classifications (enums)
    //   FieldSpec::Index
    //   FieldSpec::OpenRange
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
                } else if let Some(regex) = capture.name("r_data") {
                    let re = Regex::new(regex.as_str())?;
                    field_enums.push(FieldSpec::RegularExpression(re));
                // -f-N => FieldSpec::Last
                } else if let Some(last) = capture.name("last") {
                    let last_index = captured_index(last)?;
                    field_enums.push(FieldSpec::Last(last_index));
                // -f N-M => FieldSpec::ClosedRange
                // -f N- => FieldSpec::OpenRange
                } else if let Some(start) = capture.name("start") {
                    let start_index = captured_index(start)?;
                    if let Some(end) = capture.name("end") {
                        let end_index = captured_index(end)?;
                        field_enums.push(FieldSpec::ClosedRange(start_index, end_index));
                    } else {
                        field_enums.push(FieldSpec::OpenRange(start_index));
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
        let line_tokens = line_tokens(i)?;

        // generate indices into `line_tokens` to extract
        let indices = field_enums.iter().flat_map(|f| f.indices(&line_tokens));

        // collect the (unique)? (sorted)? set of indices
        let indices = match uniq {
            true => match sorted {
                true => indices.unique().sorted().collect::<Vec<_>>(),
                false => indices.unique().collect::<Vec<_>>(),
            },
            false => match sorted {
                true => indices.sorted().collect::<Vec<_>>(),
                false => indices.collect::<Vec<_>>(),
            },
        };

        let indices = match compliment {
            true => (0..line_tokens.len()).filter(|i| !indices.contains(i)).collect(),
            false => indices,
        };

        // current line number + output_delim
        let line_number = match number {
            true => format!("{}{output_delim}", i + 1),
            false => "".into(),
        };

        // output a line of joined fields
        if !indices.is_empty() || !zero {
            println!(
                "{}{}",
                line_number,
                indices
                    .into_iter()
                    .map(|i| line_tokens[i].to_owned())
                    .collect::<Vec<_>>()
                    .join(&output_delim)
            );
        }
    }

    Ok(())
}
