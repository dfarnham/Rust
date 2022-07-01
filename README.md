# Rust
Rust Tools / Playground

## b64 - Base64 encoder/decoder

~~~sh
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

~~~sh
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

~~~sh
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

~~~sh
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
