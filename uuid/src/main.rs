use anyhow::{Context, Result};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// Version 4, output a random v4 uuid
        #[clap(short = '4')]
        v4: bool,

        /// Version 5, namespace OID on the input -- this is the default
        #[allow(dead_code)]
        #[clap(short = '5')]
        v5: bool,

        /// Quiet mode, output only the UUID, suppress filename
        #[clap(short, long)]
        quiet: bool,

        /// file|stdin, filename of "-" implies stdin
        #[clap(parse(from_os_str), multiple_values = false)]
        file: Option<std::path::PathBuf>,
    }
    let args = Args::parse();

    // ===============================================================

    // option -4 -- output a version 4 random uuid and exit
    if args.v4 {
        match args.quiet {
            true => println!("{}", Uuid::new_v4()),
            false => println!("uuid4:\t{}", Uuid::new_v4()),
        }
        return Ok(());
    }

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

    // compute a version 5 uuid using namespace OID on the input
    let uuid5 = Uuid::new_v5(&Uuid::NAMESPACE_OID, &buffer);
    match args.quiet {
        true => println!("{uuid5}"),
        false => println!("uuid5 ({input_name}) = {uuid5}"),
    }

    Ok(())
}
