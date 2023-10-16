//! This is a utility to extract otpauth strings from QR-images and display the 6-digit TOTP

use anyhow::{Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
use rqrr::PreparedImage;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::{self, Read};

// adopted from:
// https://github.com/Levminer/authme/tree/dev/core/crates/google_authenticator_converter
// https://alexbakker.me/post/parsing-google-auth-export-qr-code.html
mod google_authenticator_converter;

mod totp_token;
use crate::totp_token::generate_tokens;

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(
        author,
        version,
        about,
        long_about = "1. Extract the otpauth:// and TOTP from an image:\n    $ qr-otpauth -v my-saved-qr.jpg\n    file = my-saved-qr.jpg\n    otpauth = otpauth://totp/user@site.com?secret=SECRET&issuer=Site&algorithm=SHA1&digits=6&period=30\n    123456, Site\n\n2. TOTP from migration accounts:\n    $ qr-otpauth -a \"otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE\"\n    947627, Test1\n    958374, Test2\n    882973, Test3"
    )]
    struct Args {
        /// "otpauth-migration://offline?data=bHVja3kK..." or "otpauth://totp/...?secret=SECRET"
        #[arg(short, long)]
        auth: Option<String>,

        /// verbose output
        #[arg(short, long)]
        verbose: bool,

        /// image-file|stdin, filename of "-" implies stdin
        files: Vec<std::path::PathBuf>,
    }
    let args = Args::parse();

    // ===============================================================

    if let Some(otpauth) = args.auth {
        if args.verbose {
            println!("otpauth = {otpauth}");
        }
        // Display the 6 digit TOTP token and Issuer
        for (token, issuer) in generate_tokens(&otpauth)? {
            println!("{token}, {issuer}");
        }
        return Ok(());
    }

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
                // Decode the grid and obtain the otpauth string
                // e.g. otpauth://totp/Site:User?Secret=Base-32&period=30&digits=6&issuer=SiteName
                // e.g. otpauth-migration://offline?data=Base-64
                let (_meta, otpauth) = grids[0].decode()?;

                if args.verbose {
                    println!("file = {input_name}\notpauth = {otpauth}");
                }

                // Display the 6 digit TOTP token and Issuer
                for (token, issuer) in generate_tokens(&otpauth)? {
                    println!("{token}, {issuer}");
                }

                if args.verbose {
                    println!("{:~^20}", "");
                }
            }
            grids => println!(
                "\n** Error({input_name}) expected 1 image grid, found {} grids **\n",
                grids.len()
            ),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration() {
        let otpauth = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE";
        assert_eq!(generate_tokens(&otpauth).unwrap().len(), 3);
    }
}
