use anyhow::{Context, Result};
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(StructOpt)]
    #[structopt(name = "num", about = "Number/UTF Representation Converter")]
    struct Cli {
        // char
        #[structopt(short, long, help = "UTF-8 Char,     num -c 🍺")]
        char: Option<String>,

        // binary
        #[structopt(short, long, help = "Binary,         num -b 11111001101111010")]
        binary: Option<String>,

        // decimal
        #[structopt(short, long, help = "Decimal,        num -d 127866")]
        decimal: Option<u32>,

        // hex
        #[structopt(short = "x", long, help = "Hexadecimal,    num -x 1f37a")]
        hex: Option<String>,

        // octal
        #[structopt(short, long, help = "Octal,          num -o 371572")]
        octal: Option<String>,

        // UTF-16
        #[structopt(short = "U", long, help = "UTF-16,         num -U 'd83c df7a'")]
        utf16: Option<String>,

        // UTF-8
        #[structopt(short = "u", long, help = "UTF-8,          num -u 'f0 9f 8d ba'")]
        utf8: Option<String>,
    }
    let args = Cli::from_args();

    // ==============================================================
    //
    let mut bytes = vec![];
    let mut utf8hex = String::new();

    // -d, -x, -o, -b
    let mut n = if let Some(decimal) = args.decimal {
        decimal
    } else if let Some(hex) = args.hex {
        u32::from_str_radix(hex.trim_start_matches("U+").trim_start_matches("0x"), 16)
            .with_context(|| format!("failed to parse '{hex}'"))?
    } else if let Some(octal) = args.octal {
        u32::from_str_radix(octal.trim_start_matches("0o"), 8)
            .with_context(|| format!("failed to parse '{octal}'"))?
    } else if let Some(binary) = args.binary {
        u32::from_str_radix(binary.trim_start_matches("0b"), 2)
            .with_context(|| format!("failed to parse '{binary}'"))?
    } else {
        0
    };

    // -u, -U, -c
    if let Some(utf8) = args.utf8 {
        utf8hex = utf8
            .trim_start_matches("0x")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
    } else if let Some(utf16) = args.utf16 {
        let s: String = utf16
            .trim_start_matches("0x")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        n = match s.len() {
            // convert hex representations to integers and undo the encoding
            // described at: https://en.wikipedia.org/wiki/UTF-16
            len if len <= 4 => {
                u32::from_str_radix(&s, 16).with_context(|| format!("failed to parse '{s}'"))?
            }
            len if (5..=8).contains(&len) => {
                let a = u32::from_str_radix(&s[0..len - 4], 16)
                    .with_context(|| format!("failed to parse '{}'", &s[0..len - 4]))?
                    - 0xd800;
                let b = u32::from_str_radix(&s[len - 4..], 16)
                    .with_context(|| format!("failed to parse '{}'", &s[len - 4..]))?
                    - 0xdc00;
                0x10000 + (a << 10 | b)
            }
            _ => panic!("UTF-16 length error"),
        }
    } else if let Some(char) = args.char {
        bytes = char.bytes().collect();
        utf8hex = bytes
            .iter()
            .map(|b| format!("{:#02x}", b).trim_start_matches("0x").to_string())
            .collect::<Vec<_>>()
            .concat()
    }

    if !utf8hex.is_empty() {
        n = match utf8hex.len() {
            2 => u32::from_str_radix(&utf8hex, 16)
                .with_context(|| format!("failed to parse '{utf8hex}'"))?,
            4 => {
                let a = u32::from_str_radix(&utf8hex[0..=1], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[0..=1]))?;
                let b = u32::from_str_radix(&utf8hex[2..=3], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[2..=3]))?;
                ((a & 0x1f) << 6) | (b & 0x3f)
            }
            6 => {
                let a = u32::from_str_radix(&utf8hex[0..=1], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[0..=1]))?;
                let b = u32::from_str_radix(&utf8hex[2..=3], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[2..=3]))?;
                let c = u32::from_str_radix(&utf8hex[4..=5], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[4..=5]))?;
                ((a & 0xf) << 12) | ((b & 0x3f) << 6) | (c & 0x3f)
            }
            8 => {
                let a = u32::from_str_radix(&utf8hex[0..=1], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[0..=1]))?;
                let b = u32::from_str_radix(&utf8hex[2..=3], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[2..=3]))?;
                let c = u32::from_str_radix(&utf8hex[4..=5], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[4..=5]))?;
                let d = u32::from_str_radix(&utf8hex[6..=7], 16)
                    .with_context(|| format!("failed to parse '{}'", &utf8hex[6..=7]))?;
                ((a & 0x7) << 18) | ((b & 0x3f) << 12) | ((c & 0x3f) << 6) | (d & 0x3f)
            }
            _ => panic!("UTF-8 must be entered in 2-byte hex format"),
        }
    }

    if bytes.is_empty() {
        match n {
            n if n <= 0x7f => {
                bytes.push(n as u8);
            }
            n if (0x80..=0x7ff).contains(&n) => {
                bytes.push((n >> 6 | 0xc0) as u8);
                bytes.push(((n & 0x3f) | 0x80) as u8);
            }
            n if (0x800..=0xffff).contains(&n) => {
                bytes.push((n >> 12 | 0xe0) as u8);
                bytes.push(((n >> 6 & 0x3f) | 0x80) as u8);
                bytes.push(((n & 0x3f) | 0x80) as u8);
            }
            n if (0x10000..=0x10ffff).contains(&n) => {
                bytes.push((n >> 18 | 0xf0) as u8);
                bytes.push(((n >> 12 & 0x3f) | 0x80) as u8);
                bytes.push(((n >> 6 & 0x3f) | 0x80) as u8);
                bytes.push(((n & 0x3f) | 0x80) as u8);
            }
            _ => panic!("unable to convert to UTF-8: [{}]", n),
        }
    }

    let n_oct = format!("{n:#o}").trim_start_matches("0o").to_owned();
    let n_hex = format!("{n:#x}").trim_start_matches("0x").to_owned();
    let n_bin = format!("{n:#b}").trim_start_matches("0b").to_owned();
    let n_utf8 = bytes
        .iter()
        .map(|b| format!("{:#02x}", b).trim_start_matches("0x").to_owned())
        .collect::<Vec<_>>()
        .join(" ");
    let n_utf16 = match (0x10000..=0x10ffff).contains(&n) {
        true => {
            format!("{:#4x}", ((n - 0x10000) >> 10) + 0xd800)
                .trim_start_matches("0x")
                .to_owned()
                + " "
                + format!("{:#4x}", ((n - 0x10000) & 0x3ff) + 0xdc00).trim_start_matches("0x")
        }
        _ => "NA".to_owned(),
    };
    let n_char = std::str::from_utf8(&bytes)
        .with_context(|| format!("std::str::from_utf8 failed converting bytes '{bytes:?}'"))?;

    println!(
        "(Dec) {}\t(Oct) {}\t(Hex) {}\t(Bin[{}]) {}\t(UTF-8) {}\t(UTF-16) {}\t(UTF-8 Char) {}",
        n,
        n_oct,
        n_hex,
        n_bin.len() - 2,
        n_bin,
        n_utf8,
        n_utf16,
        n_char
    );

    Ok(())
}
