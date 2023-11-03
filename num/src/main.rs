use anyhow::{Context, Result};
use clap::Parser;
use colored::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// Binary,         num -b 11111001101111010
        #[clap(short, long, group = "input", action = clap::ArgAction::SetTrue)]
        binary: bool,

        /// Decimal,        num -d 127866
        #[clap(short, long, group = "input", action = clap::ArgAction::SetTrue)]
        decimal: bool,

        /// Hexadecimal,    num -x 1f37a
        #[clap(short = 'x', group = "input", long, action = clap::ArgAction::SetTrue)]
        hex: bool,

        /// Octal,          num -o 371572
        #[clap(short, long, group = "input", action = clap::ArgAction::SetTrue)]
        octal: bool,

        /// Number (derive base from input form unless specified -[xbod])
        number: Option<String>,
    }
    let args = Args::parse();

    // ==============================================================
    //

    // -x, -b, -o, -d
    let n = if let Some(number) = args.number {
        match number {
            zero if zero.trim_start_matches('0').is_empty() => 0,
            hex if args.hex || hex.starts_with("0x") || hex.starts_with('x') => {
                u32::from_str_radix(hex.trim_start_matches("0x").trim_start_matches('x'), 16)
                    .with_context(|| format!("failed to parse '{hex}'"))?
            }
            binary if args.binary || binary.starts_with("0b") || binary.starts_with('b') => {
                u32::from_str_radix(binary.trim_start_matches("0b").trim_start_matches('b'), 2)
                    .with_context(|| format!("failed to parse '{binary}'"))?
            }
            octal if !args.decimal && (args.octal || octal.starts_with("0o") || octal.starts_with('0')) => {
                u32::from_str_radix(octal.trim_start_matches("0o").trim_start_matches('0'), 8)
                    .with_context(|| format!("failed to parse '{octal}'"))?
            }
            _ => number
                .trim_start_matches('0')
                .parse::<u32>()
                .with_context(|| format!("failed to parse '{number}'"))?,
        }
    } else {
        0
    };

    let n_oct = format!("{n:#o}").trim_start_matches("0o").to_owned();
    let n_hex = format!("{n:#x}").trim_start_matches("0x").to_owned();
    let n_bin = format!("{n:#b}").trim_start_matches("0b").to_owned();

    #[rustfmt::skip]
    println!("{} {}    {} {}    {} {}    {}{}{} {}",
        "(Dec)".yellow().bold(), n.to_string().green().bold(),
        "(Oct)".yellow().bold(), n_oct.green().bold(),
        "(Hex)".yellow().bold(), n_hex.green().bold(),
        "(Bin-".yellow().bold(), n_bin.len().to_string().green().bold(), ")".yellow().bold(), n_bin.green().bold(),
    );

    Ok(())
}
