use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() == 1 || args.len() == 2 && args[1] == "-" {
        let mut buffer = vec![];
        io::stdin()
            .read_to_end(&mut buffer)
            .with_context(|| "failed to read `stdin`")?;

        match args[0].ends_with("sha1") {
            true => println!("{:x}\tstdin", Sha1::digest(buffer)),
            false => println!("{:x}\tstdin", Sha256::digest(buffer)),
        };
    } else {
        for file in &args[1..] {
            let mut buffer = vec![];
            File::open(&file)
                .with_context(|| format!("failed to open file `{}`", file))?
                .read_to_end(&mut buffer)
                .with_context(|| format!("failed to read file `{}`", file))?;

            match args[0].ends_with("sha1") {
                true => println!("{:x}\t{file}", Sha1::digest(buffer)),
                false => println!("{:x}\t{file}", Sha256::digest(buffer)),
            };
        }
    }
    Ok(())
}
