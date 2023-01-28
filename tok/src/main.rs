use anyhow::{Context, Result};
use general::reset_sigpipe;
use std::fs::File;
use std::io::{self, BufRead, Error, ErrorKind, Write};
use tokenize::{error::TokenizeError, tokenizer_from_spec, TokenizationSpec, TokenizerType};

// clap arg parser
mod argparse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // behave like a typical unix utility
    reset_sigpipe()?;
    let mut stdout = io::stdout().lock();

    // parse command line arguments
    let args = argparse::get_args();

    // build a TokenizationSpec from arg inputs
    let tokenizer_spec = TokenizationSpec {
        downcase_text: args.get_flag("downcase"),
        trimmed_tokens: args.get_flag("trimmed"),
        tokenizer_init_param: args.get_one::<String>("tokenizer_param").cloned(),
        filter_tokens_re: args.get_one::<String>("regex").cloned(),
        tokenizer_type: match args.get_one::<String>("tokenizer") {
            Some(name) => match name.as_ref() {
                "ss" | "splitstr" => TokenizerType::SplitStr,
                "us" | "unicode_segment" => TokenizerType::UnicodeSegment,
                "uw" | "unicode_word" => TokenizerType::UnicodeWord,
                "ws" | "whitespace" => TokenizerType::Whitespace,
                "rb" | "regexboundary" => TokenizerType::RegexBoundary,
                _ => {
                    return Err(Box::new(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Invalid tokenizer: {name}"),
                    )))
                }
            },
            None => return Err(Box::new(Error::new(ErrorKind::InvalidInput, "No tokenizer"))),
        },
    };

    // Build a tokenizer from a TokenizationSpec
    let tokenizer =
        tokenizer_from_spec(&tokenizer_spec).map_err(|e| TokenizeError::AcquireTokerError(e.to_string()))?;

    // read input lines from a filename or stdin and collect into a Vec<String>
    let lines = match args.get_one::<std::path::PathBuf>("FILE") {
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

    for line in lines {
        let tokens = tokenizer.tokens(&line);
        //writeln!(stdout, "{}", tokens.join(""))?;
        writeln!(stdout, "{tokens:?}")?;
    }
    Ok(())
}
