use anyhow::{Context, Result};
use clap::Parser;
use general::reset_sigpipe;
use image::io::Reader as ImageReader;
use rqrr::PreparedImage;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn Error>> {
    // Behave like a typical unix utility
    reset_sigpipe()?;
    let mut stdout = io::stdout().lock();

    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// file|stdin, filename of "-" implies stdin
        files: Vec<std::path::PathBuf>,
    }
    let args = Args::parse();

    // ===============================================================

    let files = match args.files.is_empty() {
        true => vec![std::path::PathBuf::from("-")],
        false => args.files,
    };

    for file in files {
        // Read stdin|file into a byte buffer, note a filename of "-" implies stdin
        let mut bytes = vec![];
        let input_name: String = match file.as_os_str() != "-" {
            true => {
                File::open(&file)
                    .with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?
                    .read_to_end(&mut bytes)
                    .with_context(|| format!("could not read file `{:?}`", file.as_os_str()))?;
                file.to_string_lossy().into()
            }
            false => {
                io::stdin()
                    .read_to_end(&mut bytes)
                    .with_context(|| "could not read `stdin`")?;
                "<stdin>".into()
            }
        };

        // Extract and display the otpauth URI
        // https://github.com/google/google-authenticator/wiki/Key-Uri-Format

        // Detect the image format and decode the bytes into a Luma image
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?
            .to_luma8();

        // Prepare for detection
        let mut img = PreparedImage::prepare(img);

        // Search for grids, without decoding
        match img.detect_grids() {
            grids if grids.len() == 1 => {
                // Decode the grid
                let (_meta, content) = grids[0].decode()?;
                writeln!(stdout, "{input_name} = {content}")?
            }
            grids => writeln!(
                stdout,
                "Error({input_name}) expected 1 image grid, found {} grids",
                grids.len()
            )?,
        }
    }

    Ok(())
}
