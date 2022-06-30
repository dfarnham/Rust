use anyhow::{Context, Result};
use clap::Parser;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use sha2::Sha512;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// The SHA-1 hash function should be considered cryptographically broken: https://sha-mbles.github.io/
        #[clap(short = '1')]
        v1: bool,

        /// SHA-2,256 (default)
        #[clap(short = '2')]
        v256: bool,

        /// SHA-2,512
        #[clap(short = '5')]
        v512: bool,

        /// Pretty format which is broken up with whitespace
        #[clap(short)]
        pretty: bool,

        /// file|stdin, filename of "-" implies stdin
        #[clap(multiple_values = false)]
        file: Option<std::path::PathBuf>,
    }
    let args = Args::parse();

    // allocate a buffer to receive data from stdin|file, note a filename of "-" implies stdin
    let mut buffer = vec![];
    let input_name: String = match args.file {
        Some(file) if file.as_os_str() != "-" => {
            File::open(&file)
                .with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?
                .read_to_end(&mut buffer)
                .with_context(|| format!("could not read file `{:?}`", file.as_os_str()))?;
            file.to_string_lossy().into()
        }
        _ => {
            io::stdin()
                .read_to_end(&mut buffer)
                .with_context(|| "could not read `stdin`")?;
            "<stdin>".into()
        }
    };

    let n = match args.pretty {
        true => 8,
        false => usize::MAX,
    };

    let digest = if args.v1 {
        format!("{:x}", Sha1::digest(buffer))
    } else if args.v512 {
        format!("{:x}", Sha512::digest(buffer))
    } else {
        format!("{:x}", Sha256::digest(buffer))
    };

    for (i, c) in digest.chars().enumerate() {
        print!("{c}");
        if (i + 1) % n == 0 {
            print!(" ");
        }
    }
    println!("\t{input_name}");
    Ok(())
}
