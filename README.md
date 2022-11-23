# Rust
Rust Tools / Playground

## Build/Install

~~~
Build everything
$ cargo build --bins -r

Lint check everything
$ cargo clippy

Install a tool to ~/.cargo/bin
$ cd <tool>
$ cargo install --path .
~~~

## tokenize - Library to acquire/configure text tokenizers given a specification

~~~
This library takes as input a TokenizationSpec and returns a configured Tokenizer.

The TokenizationSpec instructs text transformations and token filtering rules.

The "text" to tokens[] recipe:
	1. downcase the input text (true/false)
	2. apply WordTokenizer(TokenizerType) to text
	3. whitespace trim() tokens (true/false)
	4. discard tokens matching a Regular Expression

Tokenizer.tokens(&str) -> Vec<String>

The TokenizerType is one of:
	* SplitStr (Option<String>) -- String to split on
	* UnicodeSegment
	* UnicodeWord
	* Whitespace
	* WordBoundary (Option<String>) -- String containing boundary chars to exclude.
		Overrides the standard \b assertion for that char. e.g. "-'"

pub struct TokenizationSpec {
    pub tokenizer_type: TokenizerType,
    pub tokenizer_init_param: Option<String>,
    pub downcase_text: bool,
    pub trimmed_tokens: bool,
    pub filter_tokens_re: Option<String>,
}
impl TokenizationSpec {
    pub fn default() -> Self {
        Self {
            tokenizer_type: TokenizerType::Whitespace,
            tokenizer_init_param: None,
            downcase_text: false,
            trimmed_tokens: false,
            filter_tokens_re: None,
        }
    }
}
~~~

### Example
~~~
use tokenize::{tokenizer_from_spec, TokenizationSpec, TokenizerType};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let utf8str = "\u{201F}THE-BIG-RIPOFF\u{201D} Mr\u{FE52} & Mrs\u{2024} John B. Smith, cheapsite.com, 1.5 million, i\u{FF0E}e\u{2024}, 🍺+🍕, na\u{00EF}ve, stressed vowels: \u{00E9}, \u{00ED}, \u{00F3}, \u{00FA}, \u{2026}";

    println!("utf8str = {utf8str}\n");

    // Whitespace (default)
    let mut tokenizer_spec = TokenizationSpec::default();

    for toker in [TokenizerType::Whitespace, TokenizerType::SplitStr, TokenizerType::UnicodeSegment,
                  TokenizerType::UnicodeWord, TokenizerType::WordBoundary] {
        tokenizer_spec.tokenizer_type = toker;
        let tokenizer = tokenizer_from_spec(&tokenizer_spec)?;
        println!("{:?}:\t{:?}\n", tokenizer_spec.tokenizer_type, tokenizer.tokens(utf8str));
    }

    Ok(())
}

utf8str = ‟THE-BIG-RIPOFF” Mr﹒ & Mrs․ John B. Smith, cheapsite.com, 1.5 million, i．e․, 🍺+🍕, naïve, stressed vowels: é, í, ó, ú, …

Whitespace:	["‟THE-BIG-RIPOFF”", "Mr﹒", "&", "Mrs․", "John", "B.", "Smith,", "cheapsite.com,", "1.5", "million,", "i．e․,", "🍺+🍕,", "naïve,", "stressed", "vowels:", "é,", "í,", "ó,", "ú,", "…"]

SplitStr:	["", "‟", "T", "H", "E", "-", "B", "I", "G", "-", "R", "I", "P", "O", "F", "F", "”", " ", "M", "r", "﹒", " ", "&", " ", "M", "r", "s", "․", " ", "J", "o", "h", "n", " ", "B", ".", " ", "S", "m", "i", "t", "h", ",", " ", "c", "h", "e", "a", "p", "s", "i", "t", "e", ".", "c", "o", "m", ",", " ", "1", ".", "5", " ", "m", "i", "l", "l", "i", "o", "n", ",", " ", "i", "．", "e", "․", ",", " ", "🍺", "+", "🍕", ",", " ", "n", "a", "ï", "v", "e", ",", " ", "s", "t", "r", "e", "s", "s", "e", "d", " ", "v", "o", "w", "e", "l", "s", ":", " ", "é", ",", " ", "í", ",", " ", "ó", ",", " ", "ú", ",", " ", "…", ""]

UnicodeSegment:	["‟", "THE", "-", "BIG", "-", "RIPOFF", "”", " ", "Mr", "﹒", " ", "&", " ", "Mrs", "․", " ", "John", " ", "B", ".", " ", "Smith", ",", " ", "cheapsite.com", ",", " ", "1.5", " ", "million", ",", " ", "i．e", "․", ",", " ", "🍺", "+", "🍕", ",", " ", "naïve", ",", " ", "stressed", " ", "vowels", ":", " ", "é", ",", " ", "í", ",", " ", "ó", ",", " ", "ú", ",", " ", "…"]

UnicodeWord:	["THE", "BIG", "RIPOFF", "Mr", "Mrs", "John", "B", "Smith", "cheapsite.com", "1.5", "million", "i．e", "naïve", "stressed", "vowels", "é", "í", "ó", "ú"]

WordBoundary:	["THE", "BIG", "RIPOFF", "Mr", "Mrs", "John", "B", "Smith", "cheapsite", "com", "1", "5", "million", "i", "e", "naïve", "stressed", "vowels", "é", "í", "ó", "ú"]
~~~

## cutr - Extract selected fields of each line of a file by index, range, or regular expression

~~~
Extract selected fields of each line of a file by index, range, or regular expression

Usage: cutr [OPTIONS] -f <field_spec> [FILE]

Arguments:
  [FILE]  File to read, use '-' for standard input

Options:
  -f <field_spec>      [-]number, range, or regex (use `--help` for more detail)
  -d <char>            Input field separator character, defaults to whitespace
  -T                   Short for -d'\t'
  -o <str>             Use <str> as the output field separator, default is to use -d, or '\t'
  -s                   Output fields in index-sorted order
  -u                   Output only unique fields
  -t                   Trim whitespace in data parsing
  -n                   Add a beginning field on output denoting the line number of the input
  -c                   Output the compliment of fields
  -z                   Don't output empty lines
  -h, --help           Print help information (use `--help` for more detail)
  -V, --version        Print version information
~~~

## b64 - Base64 encoder/decoder

~~~
Base64 Encoder/Decoder

USAGE:
    b64 [OPTIONS] [FILE]

ARGS:
    <FILE>    file|stdin, filename of "-" implies stdin

OPTIONS:
    -d, --decode     Decode from Base64
    -e, --encode     Encode to Base64 (default)
    -h, --help       Print help information
    -p, --pretty     Break output into lines of length 76
    -V, --version    Print version information
~~~

---

## num - Number/UTF Representation Converter

~~~
Number/UTF Representation Converter

USAGE:
    num [OPTIONS]

OPTIONS:
    -b, --binary <BINARY>      Binary,         num -b 11111001101111010
    -c, --char <CHAR>          UTF-8 Char,     num -c 🍺
    -d, --decimal <DECIMAL>    Decimal,        num -d 127866
    -h, --help                 Print help information
    -o, --octal <OCTAL>        Octal,          num -o 371572
    -u, --utf8 <UTF8>          UTF-8,          num -u 'f0 9f 8d ba'
    -U, --utf16 <UTF16>        UTF-16,         num -U 'd83c df7a'
    -V, --version              Print version information
    -x, --hex <HEX>            Hexadecimal,    num -x 1f37a

$ num -c 🍺
(Dec) 127866	(Oct) 371572	(Hex) 1f37a	(Bin[15]) 11111001101111010	(UTF-8) f0 9f 8d ba	(UTF-16) d83c df7a	(UTF-8 Char) 🍺
~~~

---

## SHA 1,256

~~~
USAGE:
    sha [OPTIONS] [FILE]

ARGS:
    <FILE>    file|stdin, filename of "-" implies stdin

OPTIONS:
    -1               The SHA-1 hash function should be considered cryptographically broken:
                     https://sha-mbles.github.io/
    -2               SHA-2,256 (default)
    -5               SHA-2,512
    -h, --help       Print help information
    -p               Pretty format which is broken up with whitespace
    -V, --version    Print version information
~~~

---

## utf8char - utf8 validator

~~~sh
Usage: utf8char [options] file|stdin

Options:
    -b, --prefix        prefix string
    -a, --postfix       postfix string
    -h, --help          usage
Example: echo -n '🍺&🍕' | utf8char -b '[' -a ']'
[🍺][&][🍕]
~~~

---

## uuid -- uuid version 4,5 utility

~~~
UUID v4,v5

USAGE:
    uuid [OPTIONS] [FILE]

ARGS:
    <FILE>    file|stdin, filename of "-" implies stdin

OPTIONS:
    -4               Version 4, output a random v4 uuid
    -5               Version 5, namespace OID on the input -- this is the default
    -h, --help       Print help information
    -q, --quiet      Quiet mode, output only the UUID, suppress filename
    -V, --version    Print version information
~~~

---

## nom\_word\_boundary
experimenting with a nom based word boundary parser, custom parser remains 30% faster however.

---

## kennard-stone -- [Kennard Stone algorithm](http://wiki.eigenvector.com/index.php?title=Kennardstone)
# 
