use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::str::FromStr;

// https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
//
// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
#[allow(dead_code)]
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// Reads the lines of a file, trims and returns them as a Vec of the supplied type
pub fn read_trimmed_data_lines<T>(
    filename: Option<&PathBuf>,
) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static,
    <T as FromStr>::Err: std::error::Error,
{
    let mut values = vec![];
    match filename {
        Some(file) if file.as_os_str() != "-" => {
            for line in read_lines(file)? {
                values.push(line?.trim().parse::<T>()?);
            }
            Ok(values)
        }
        _ => {
            // STDIN
            for line in io::BufReader::new(io::stdin()).lines() {
                values.push(line?.trim().parse::<T>()?);
            }
            Ok(values)
        }
    }
}

// splits and optionally trims the input String on a separator character
// returns a Vec of parse::<T>() over the splits
pub fn split_on<T>(text: &str, sep: char, trim: bool) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::error::Error,
    <T as std::str::FromStr>::Err: 'static,
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

// ==============================================================

// https://github.com/rust-lang/rust/issues/62569

/*
#[cfg(unix)]
pub fn reset_sigpipe() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    Ok(())
}
*/

// This should be called in cli apps
pub fn reset_sigpipe() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;

        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)?;
        }
    }

    Ok(())
}
