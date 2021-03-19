use anyhow::{Context, Result};
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use structopt::StructOpt;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn Error>> {
    // =================================
    // start command line option parsing

    #[derive(StructOpt)]
    #[structopt(
        name = "uuid",
        about = "Outputs a Version 5 uuid using namespace OID on the input or a Version 4 random uuid"
    )]
    struct Cli {
        // Input file
        #[structopt(parse(from_os_str))]
        input: Option<std::path::PathBuf>,

        // Version 4
        #[structopt(long, help = "Version 4 uuid -- output a random v4 uuid and exit")]
        v4: bool,

        // Version 5
        #[allow(dead_code)]
        #[structopt(long, help = "Version 5 uuid (namespace OID) on the input -- default")]
        v5: bool,

        // Quiet mode
        #[structopt(short, long, help = "Quiet mode - only the checksum is printed out")]
        quiet: bool,
    };

    let args = Cli::from_args();

    // end command line option parsing
    // ===============================

    // option --v4 -- output a version 4 random uuid and exit
    if args.v4 {
        match args.quiet {
            true => println!("{}", Uuid::new_v4()),
            false => println!("uuid4:\t{}", Uuid::new_v4()),
        }
        return Ok(());
    }

    // allocate a buffer to receive data from stdin|file, note a filename of "-" implies stdin
    let mut buffer = Vec::new();
    let input_name: String = match args.input {
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
            "-".into()
        }
    };

    // compute a version 5 uuid using namespace OID on the input
    let uuid5 = Uuid::new_v5(&Uuid::NAMESPACE_OID, &buffer);
    match args.quiet {
        true => println!("{}", uuid5),
        false => println!("uuid5 ({}) = {}", input_name, uuid5),
    }

    Ok(())
}
