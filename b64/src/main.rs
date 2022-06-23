use anyhow::{Context, Result};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};

// Base64 alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
#[rustfmt::skip]
const B64TABLE: [u8; 64] = [
     65,  66,  67,  68,  69,  70,  71,  72,  73,  74,  75,  76,  77,  // "ABCDEFGHIJKLM"
     78,  79,  80,  81,  82,  83,  84,  85,  86,  87,  88,  89,  90,  // "NOPQRSTUVWXYZ"
     97,  98,  99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109,  // "abcdefghijklm"
    110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122,  // "nopqrstuvwxyz"
     48,  49,  50,  51,  52,  53,  54,  55,  56,  57,                 // "0123456789"
     43,                                                              // "+"
     47                                                               // "/"
];

// Reverse index that yields the 6 bit value (position in the alphabet)
#[rustfmt::skip]
const R_B64TABLE: [u8; 80] = [
    62,                                                  // "+"
     0,  0,  0,                                          // unused
    63,                                                  // "/"
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61,              // "0" .. "9"
     0,  0,  0,  0,  0,  0,  0,                          // unused
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12,  // "A" - "M"
    13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,  // "N" - "Z"
     0,  0,  0,  0,  0,  0,                              // unused
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,  // "a" - "m"
    39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51   // "n" - "z"
];

// values in R_B64TABLE are offset by the minimum value ("+") in B64TABLE
const TABLE_OFFSET: u8 = 43;

// Base64 pad character ("=")
const PAD_CHAR: u8 = 61;

/* Algorithm using shifting:
 *
 * bytes[0] = 'A'
 * bytes[1] = 'B'
 * bytes[2] = 'C'
 *
 * b64table[bytes[0] >> 2]                               //  b64table[16] == 'Q'
 * b64table[((bytes[0] << 4) & 0x30) | (bytes[1] >> 4)]  //  b64table[20] == 'U'
 * b64table[((bytes[1] << 2) & 0x3c) | (bytes[2] >> 6)]  //  b64table[9]  == 'J'
 * b64table[bytes[2] & 0x3f]                             //  b64table[3]  == 'D'
 *
 * --------------------------------------------------------------------------------
 *
 * Algorithm using a C union
 *
 * typedef unsigned char uchar;
 *
 * typedef union {
 *     uchar bytes[3];
 *     struct {
 *         unsigned d:6;
 *         unsigned c:6;
 *         unsigned b:6;
 *         unsigned a:6;
 *     } u;
 * } B64;
 *
 * Union of 24 bits (3 bytes) and four 6-bit ints for Base64 encoding
 *
 *     'A'       'B'       'C'
 *     65        66        67
 *   bytes[2]  bytes[1]  bytes[0]  <===== "ABC" loaded in reverse of shifting technique
 *   --------  --------  --------
 *   01000001  01000010  01000011
 *   ||||||||  ||||||||  ||||||||
 *   ||||||||  ||||||||  ||++++++ u.d == 000011 == b64table[3] == 'D'
 *   ||||||||  ||||++++  ++ u.c == 001001 == b64table[9] == 'J'
 *   ||||||++  ++++ u.b == 010100 == b64table[20] == 'U'
 *   ++++++ u.a == 010000 == b64table[16] == 'Q'
 *
 */

fn b64_encode(src: [u8; 3], dst: &mut [u8; 4], n: usize) {
    // assert!(0x30 == 0b0011_0000);
    // assert!(0x3c == 0b0011_1100);
    // assert!(0x3f == 0b0011_1111);

    dst[0] = B64TABLE[(src[0] >> 2) as usize];
    match n {
        1 => {
            dst[1] = B64TABLE[(src[0] << 4 & 0b0011_0000) as usize];
            dst[2] = PAD_CHAR;
            dst[3] = PAD_CHAR;
        }

        2 => {
            dst[1] = B64TABLE[((src[0] << 4 & 0b0011_0000) | src[1] >> 4) as usize];
            dst[2] = B64TABLE[(src[1] << 2 & 0b0011_1100) as usize];
            dst[3] = PAD_CHAR;
        }

        _ => {
            dst[1] = B64TABLE[((src[0] << 4 & 0b0011_0000) | src[1] >> 4) as usize];
            dst[2] = B64TABLE[((src[1] << 2 & 0b0011_1100) | src[2] >> 6) as usize];
            dst[3] = B64TABLE[(src[2] & 0b0011_1111) as usize];
        }
    }
}

fn b64_decode(src: [u8; 4], dst: &mut [u8; 3]) -> usize {
    // assert!(0x03 == 0b0000_0011);
    // assert!(0x0f == 0b0000_1111);

    let a = R_B64TABLE[(src[0] - TABLE_OFFSET) as usize];
    let b = R_B64TABLE[(src[1] - TABLE_OFFSET) as usize];
    dst[0] = (a << 2) | (b >> 4 & 0b0000_0011);

    match src[3] {
        PAD_CHAR => match src[2] {
            PAD_CHAR => 1,
            _ => {
                let c = R_B64TABLE[(src[2] - TABLE_OFFSET) as usize];
                dst[1] = (b << 4) | (c >> 2 & 0b0000_1111);
                2
            }
        },

        _ => {
            let c = R_B64TABLE[(src[2] - TABLE_OFFSET) as usize];
            let d = R_B64TABLE[(src[3] - TABLE_OFFSET) as usize];
            dst[1] = (b << 4) | (c >> 2 & 0b0000_1111);
            dst[2] = (c << 6) | d;
            3
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// encode to Base64 (default)
        #[clap(short, long)]
        encode: bool,

        /// decode from Base64
        #[clap(short, long)]
        decode: bool,

        /// break output into lines of length 76
        #[clap(short, long)]
        pretty: bool,

        /// file|stdin, filename of "-" implies stdin
        #[clap(multiple_values = false)]
        file: Option<String>,
    }
    let args = Args::parse();

    // --encode, --decode are mutually exclusive
    // not specifying a mode implies --encode
    if args.encode && args.decode {
        return Err("options --encode, --decode are mutually exclusive".into());
    }

    // allocate a buffer to receive data from stdin|file, note a filename of "-" implies stdin
    let mut buffer = vec![];
    if args.file.is_none() || args.file == Some("-".to_string()) {
        io::stdin()
            .read_to_end(&mut buffer)
            .with_context(|| "could not read `stdin`")?;
    } else if let Some(file) = args.file {
        File::open(&file)
            .with_context(|| format!("could not open file `{}`", file))?
            .read_to_end(&mut buffer)
            .with_context(|| format!("could not read file `{}`", file))?;
    } else {
        return Err("option parsing snafu".into());
    }

    let mut src = [0; 3]; // original bytes
    let mut dst = [0; 4]; // Base64 bytes
    let mut n = 0;
    if args.decode {
        for byte in buffer.bytes() {
            let ch = byte?;

            // formatted Base64 allows for embedded newlines ('\n', '\r') that are ignored
            if ch == 10 || ch == 13 {
                continue;
            }

            dst[n] = ch;
            n += 1;
            if n == 4 {
                let nbytes = b64_decode(dst, &mut src);
                io::stdout().write_all(&src[0..nbytes])?;
                n = 0;
            }
        }
        assert!(n == 0, "final {n} bytes were not decoded");
    } else {
        let mut pretty_counter = 0;
        for byte in buffer.bytes() {
            src[n] = byte?;
            n += 1;
            if n == 3 {
                b64_encode(src, &mut dst, 3);
                io::stdout().write_all(&dst)?;

                // output a newline every 76 bytes when pretty printing
                if args.pretty {
                    pretty_counter += 1;
                    if pretty_counter % 19 == 0 {
                        io::stdout().write_all(b"\n")?;
                    }
                }
                n = 0;
            }
        }
        if n > 0 {
            b64_encode(src, &mut dst, n);
            io::stdout().write_all(&dst)?
        }
        io::stdout().write_all(b"\n")?;
    }

    Ok(())
}
