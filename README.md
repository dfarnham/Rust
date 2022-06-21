# Rust
Rust Tools / Playground

## num - Number/UTF Representation Converter

~~~sh
Number/UTF Representation Converter

USAGE:
    num [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --binary <binary>      Binary,         num -b 11111001101111010
    -c, --char <char>          UTF-8 Char,     num -c 🍺
    -d, --decimal <decimal>    Decimal,        num -d 127866
    -x, --hex <hex>            Hexadecimal,    num -x 1f37a
    -o, --octal <octal>        Octal,          num -o 371572
    -U, --utf16 <utf16>        UTF-16,         num -U 'd83c df7a'
    -u, --utf8 <utf8>          UTF-8,          num -u 'f0 9f 8d ba'

$ num -c 🍺
(Dec) 127866	(Oct) 371572	(Hex) 1f37a	(Bin[15]) 11111001101111010	(UTF-8) f0 9f 8d ba	(UTF-16) d83c df7a	(UTF-8 Char) 🍺
~~~

---

## b64 - Base64 encoder/decoder

~~~sh
Usage: b64 [-encode] [-decode] [-pretty] file|stdin

Options:
    -e, --encode        encode to Base64 (default)
    -d, --decode        decode from Base64
    -p, --pretty        break output into lines of length 76
    -h, --help          usage
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

~~~sh
Outputs a Version 5 uuid using namespace OID on the input or a Version 4 random uuid

USAGE:
    uuid [FLAGS] [input]

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Quiet mode - only the checksum is printed out
        --v4         Version 4 uuid -- output a random v4 uuid and exit
        --v5         Version 5 uuid (namespace OID) on the input -- default
    -V, --version    Prints version information

ARGS:
    <input>    file|stdin
~~~

---

## nom\_word\_boundary
experimenting with a nom based word boundary parser, custom parser remains 30% faster however.

---

## kennard-stone -- [Kennard Stone algorithm](http://wiki.eigenvector.com/index.php?title=Kennardstone)
