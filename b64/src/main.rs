use anyhow::{Context, Result};
use getopts::Options;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};

// Base64 alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
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

fn b64_encode(src: [u8; 3], dst: &mut [u8; 4], n: u8) {
    // assert!(0x30 == 0b0011_0000);
    // assert!(0x3c == 0b0011_1100);
    // assert!(0x3f == 0b0011_1111);

    dst[0] = B64TABLE[usize::from(src[0] >> 2)];
    match n {
        1 => {
            dst[1] = B64TABLE[usize::from((src[0] << 4) & 0b0011_0000)];
            dst[2] = PAD_CHAR;
            dst[3] = PAD_CHAR;
        }

        2 => {
            dst[1] = B64TABLE[usize::from(((src[0] << 4) & 0b0011_0000) | (src[1] >> 4))];
            dst[2] = B64TABLE[usize::from((src[1] << 2) & 0b0011_1100)];
            dst[3] = PAD_CHAR;
        }

        _ => {
            dst[1] = B64TABLE[usize::from(((src[0] << 4) & 0b0011_0000) | (src[1] >> 4))];
            dst[2] = B64TABLE[usize::from(((src[1] << 2) & 0b0011_1100) | (src[2] >> 6))];
            dst[3] = B64TABLE[usize::from(src[2] & 0b0011_1111)];
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn b64_decode(src: [u8; 4], dst: &mut [u8; 3]) -> u8 {
    // assert!(0x03 == 0b0000_0011);
    // assert!(0x0f == 0b0000_1111);

    let n; // return value: 1, 2, 3

    let a = R_B64TABLE[usize::from(src[0] - TABLE_OFFSET)];
    let b = R_B64TABLE[usize::from(src[1] - TABLE_OFFSET)];
    dst[0] = (a << 2) | ((b >> 4) & 0b0000_0011);

    if src[3] == PAD_CHAR {
        if src[2] == PAD_CHAR {
            n = 1
        } else {
            let c = R_B64TABLE[usize::from(src[2] - TABLE_OFFSET)];
            dst[1] = (b << 4) | ((c >> 2) & 0b0000_1111);
            n = 2
        }
    } else {
        let c = R_B64TABLE[usize::from(src[2] - TABLE_OFFSET)];
        let d = R_B64TABLE[usize::from(src[3] - TABLE_OFFSET)];
        dst[1] = (b << 4) | ((c >> 2) & 0b0000_1111);
        dst[2] = (c << 6) | d;
        n = 3
    }

    n
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("e", "encode", "encode to Base64 (default)");
    opts.optflag("d", "decode", "decode from Base64");
    opts.optflag("p", "pretty", "break output into lines of length 76");
    opts.optflag("h", "help",   "usage");

    let arg_match = match opts.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            print_usage(&args[0], opts);
            return Err(e.into());
        }
    };

    let encode = arg_match.opt_present("e");
    let decode = arg_match.opt_present("d");
    let pretty = arg_match.opt_present("p");

    // --encode, --decode are mutually exclusive
    // not specifying a mode implies --encode
    if encode && decode || arg_match.opt_present("h") {
        print_usage(&args[0], opts);
        return Err("options --encode, --decode are mutually exclusive".into());
    }

    // allocate a buffer to receive data from stdin|file, note a filename of "-" implies stdin
    let mut buffer = Vec::new();
    if arg_match.free.is_empty() || arg_match.free[0] == "-" {
        io::stdin()
            .read_to_end(&mut buffer)
            .with_context(|| "could not read `stdin`")?;
    } else {
        File::open(&arg_match.free[0])
            .with_context(|| format!("could not open file `{}`", arg_match.free[0]))?
            .read_to_end(&mut buffer)
            .with_context(|| format!("could not read file `{}`", arg_match.free[0]))?;
    }

    let mut src = [0; 3]; // original bytes
    let mut dst = [0; 4]; // Base64 bytes
    let mut n = 0;
    if decode {
        for byte in buffer.bytes() {
            let ch = byte?;

            // formatted Base64 allows for embedded newlines ('\n', '\r') that are ignored
            if ch == 10 || ch == 13 {
                continue;
            }

            dst[n] = ch;
            n += 1;
            if n == 4 {
                let nbytes = usize::from(b64_decode(dst, &mut src));
                io::stdout().write_all(&src[0..nbytes])?;
                n = 0
            }
        }
        assert!(n == 0, "final {} bytes were not decoded", n)
    } else {
        let mut pretty_counter = 0;
        for byte in buffer.bytes() {
            src[n] = byte?;
            n += 1;
            if n == 3 {
                b64_encode(src, &mut dst, 3);
                io::stdout().write_all(&dst)?;

                // output a newline every 76 bytes when pretty printing
                if pretty {
                    pretty_counter += 1;
                    if pretty_counter == 19 {
                        io::stdout().write_all(b"\n")?;
                        pretty_counter = 0
                    }
                }
                n = 0
            }
        }
        if n != 0 {
            b64_encode(src, &mut dst, n as u8);
            io::stdout().write_all(&dst)?
        }
        io::stdout().write_all(b"\n")?;
    }

    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Usage: {} [-encode] [-decode] [-pretty] file|stdin",
        program
    );
    print!("{}", opts.usage(&brief))
}

#[cfg(test)]
mod test;
