use anyhow::{Context, Result};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

const MSB: u32 = 0x80000000; // Most significant bit
const CRCPOLY: u32 = 0x80050000; // x^16 + x^15 + x^2 + 1 (shifted << 16)

// CRC-16, the input is assumed to be 2-byte zero padded
fn compute_zero_padded_crc16(buffer: &[u8]) -> u16 {
    let mut crc;

    // Check the last 2 bytes of the input are 0
    assert!(buffer.len() > 1, "Invalid input");
    assert!(buffer[buffer.len() - 1] == 0, "Invalid input last 2 bytes not 0");
    assert!(buffer[buffer.len() - 2] == 0, "Invalid input last 2 bytes not 0");

    // Load the first 2-bytes as the MSB is not set yet
    crc = (buffer[0] as u32) << 24;
    crc |= (buffer[1] as u32) << 16;

    for byte in buffer.iter().skip(2) {
        crc ^= (*byte as u32) << 8;
        for _ in 0..8 {
            if crc & MSB == 0 {
                crc <<= 1;
            } else {
                crc = (crc << 1) ^ CRCPOLY;
            }
        }
    }
    ((crc >> 16) & 0x0000ffff) as u16
}

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// file|stdin, filename of "-" implies stdin
        files: Vec<std::path::PathBuf>,
    }
    let args = Args::parse();

    let files = match args.files.is_empty() {
        true => vec![std::path::PathBuf::from("-")],
        false => args.files,
    };

    for file in files {
        // Receive data from stdin|file, note a filename of "-" implies stdin
        let mut buffer = vec![];
        let input_name: String = match file.as_os_str() != "-" {
            true => {
                File::open(&file)
                    .with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?
                    .read_to_end(&mut buffer)
                    .with_context(|| format!("could not read file `{:?}`", file.as_os_str()))?;
                file.to_string_lossy().into()
            }
            false => {
                io::stdin()
                    .read_to_end(&mut buffer)
                    .with_context(|| "could not read `stdin`")?;
                "<stdin>".into()
            }
        };

        // Add 2-bytes of zero padding
        buffer.push(0);
        buffer.push(0);
        println!("{input_name}: {}", compute_zero_padded_crc16(&buffer));
    }

    Ok(())
}
