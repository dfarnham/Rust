use anyhow::{Context, Result};
use getopts::Options;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::str;

/*
 * UTF-8
 *
 *  Number  | Bits for   |    First   |    Last    |          |          |          |          |
 * of bytes | code point | code point | code point |  Byte 1  |  Byte 2  |  Byte 3  |  Byte 4  |
 * ---------------------------------------------------------------------------------------------
 *     1    |     7      |   U+0000   |  U+007F    | 0xxxxxxx |          |          |          |
 *     2    |    11      |   U+0080   |  U+07FF    | 110xxxxx | 10xxxxxx |          |          |
 *     3    |    16      |   U+0800   |  U+FFFF    | 1110xxxx | 10xxxxxx | 10xxxxxx |          |
 *     4    |    21      |   U+10000  |  U+10FFFF  | 11110xxx | 10xxxxxx | 10xxxxxx | 10xxxxxx |
 */

// returns the number of bytes representing the utf8 char [1,4]
fn utf8_char_size(ptr: u8) -> u8 {
    match ptr {
        n if n >= 0xf0 => 4,
        n if n >= 0xe0 => 3,
        n if n >= 0xc0 => 2,
        _ => 1,
    }
}

// returns a number [1,4] representing the number of bytes in the valid utf8 char
fn utf8_char_validate(ptr: &[u8]) -> Result<u8, String> {
    let n = utf8_char_size(ptr[0]);

    for i in 1..n {
        if (ptr[i as usize] & 0xc0) != 0x80 {
            return Err(format!("utf8_char_validate() failed at byte {i}"));
        }
    }
    Ok(n)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("\nUsage: {program} [options] file|stdin");
    print!("{}", opts.usage(&brief));
    println!("Example: echo -n '🍺&🍕' | {program} -b '[' -a ']'\n[🍺][&][🍕]")
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("b", "prefix", "prefix string", "");
    opts.optopt("a", "postfix", "postfix string", "");
    opts.optflag("h", "help", "usage");

    let arg_match = match opts.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            print_usage(&args[0], opts);
            return Err(e.into());
        }
    };

    if arg_match.opt_present("h") {
        print_usage(&args[0], opts);
        std::process::exit(0)
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

    let bytes = buffer.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>();

    let mut i = 0;
    while i < bytes.len() {
        let n = match utf8_char_validate(&bytes[i..]) {
            Ok(n) => n,
            Err(e) => return Err(e.into()),
        };

        // optional prefix string
        if arg_match.opt_present("b") {
            io::stdout().write_all(arg_match.opt_str("b").unwrap().as_bytes())?;
        }

        // utf8 char
        let character_bytes = &bytes[i..i + n as usize];
        io::stdout().write_all(character_bytes)?;

        // optional postfix string
        if arg_match.opt_present("a") {
            io::stdout().write_all(arg_match.opt_str("a").unwrap().as_bytes())?;
        }

        // internal validation & debug
        let _character = match str::from_utf8(character_bytes) {
            Ok(ch) => ch,
            Err(e) => return Err(e.into()),
        };
        //println!("\n{} {:02x?}", _character, character_bytes);

        i += n as usize;
    }

    Ok(())
}
