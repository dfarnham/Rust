use anyhow::{Context, Result};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

const MSB: u32 = 0x80000000; // Most significant bit
const CRCPOLY: u32 = 0x80050000; // x^16 + x^15 + x^2 + 1 (shifted << 16)

// CRC-16, the input must be 2-byte zero padded
fn crc16_padded(msg: &[u8]) -> u16 {
    let mut crc;

    // Check the last 2 bytes of the input are 0
    assert!(msg.len() > 1, "Invalid input");
    assert!(msg[msg.len() - 1] == 0, "Invalid input last 2 bytes not 0");
    assert!(msg[msg.len() - 2] == 0, "Invalid input last 2 bytes not 0");

    // Load the first 2-bytes as the MSB is not set yet
    crc = (msg[0] as u32) << 24;
    crc |= (msg[1] as u32) << 16;

    for byte in msg.iter().skip(2) {
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
        let mut msg = vec![];
        let input_name: String = match file.as_os_str() != "-" {
            true => {
                File::open(&file)
                    .with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?
                    .read_to_end(&mut msg)
                    .with_context(|| format!("could not read file `{:?}`", file.as_os_str()))?;
                file.to_string_lossy().into()
            }
            false => {
                io::stdin()
                    .read_to_end(&mut msg)
                    .with_context(|| "could not read `stdin`")?;
                "<stdin>".into()
            }
        };

        // Add 2-bytes of zero padding
        msg.push(0);
        msg.push(0);
        println!("{input_name}: {}", crc16_padded(&msg));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_bad_input_1() {
        crc16_padded(&[]);
    }

    #[test]
    #[should_panic]
    fn test_bad_input_2() {
        crc16_padded(&[1]);
    }

    #[test]
    #[should_panic]
    fn test_bad_input_3() {
        crc16_padded(&[1, 2]);
    }

    #[test]
    #[should_panic]
    fn test_bad_input_4() {
        crc16_padded(&[1, 0]);
    }

    #[test]
    fn test0() {
        assert_eq!(0, crc16_padded(&[0, 0]));
    }

    #[test]
    fn test1() {
        assert_eq!(
            35273,
            crc16_padded(&[5, 0, 255, 255, 255, 255, 0, 0, 0, 0, 2, 0, 1, 1, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test2() {
        assert_eq!(
            43036,
            crc16_padded(&[170, 170, 170, 170, 170, 170, 170, 170, 204, 204, 204, 204, 204, 204, 204, 204, 0, 0])
        );
    }

    #[test]
    fn test3() {
        assert_eq!(25309, crc16_padded(b"dave\0\0"))
    }
}
