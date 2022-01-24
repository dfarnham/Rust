use nom::{
    bytes::complete::{take_till, take_while},
    combinator::{map, verify},
    multi::many0,
    sequence::pair,
    IResult,
};
use regex::Regex;
use std::error::Error;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, Clone, PartialEq)]
// a Token<'a> of type B or T (Boundary or Token)
// each Token holds a reference into an input string
pub enum Token<'a> {
    B(&'a str),
    T(&'a str),
}
impl<'a> Token<'a> {
    // create a new String from the reference
    fn value(&self) -> String {
        match self {
            Token::B(s) | Token::T(s) => s.to_string(),
        }
    }

    // reference value
    fn str_value(&self) -> &'a str {
        match self {
            Token::B(s) | Token::T(s) => s,
        }
    }

    // test if referenceing something empty
    fn is_empty(&self) -> bool {
        match self {
            Token::B(s) | Token::T(s) => s.is_empty(),
        }
    }

    // String from a list
    fn joined(tokens: &[Token]) -> String {
        tokens.iter().map(|t| t.value()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordBoundaryTokenizer {
    // chars in "excluded_boundary_chars" that would typically return true on Regex \b that will now return false
    excluded_boundary_chars: String,
}
impl WordBoundaryTokenizer {
    pub fn default() -> Self {
        Self::new("")
    }

    pub fn new(excluded_boundary_chars: &str) -> Self {
        Self {
            excluded_boundary_chars: excluded_boundary_chars.into(),
        }
    }

    pub fn boundary_predicate(&self, c: char) -> bool {
        lazy_static! {
            static ref REGEX_BOUNDARY_CHAR: Regex = Regex::new(r"^X\b").unwrap();
        }
        !self.excluded_boundary_chars.contains(c) && REGEX_BOUNDARY_CHAR.is_match(&("X".to_string() + &c.to_string()))
    }

    // this is 30% faster than the equivalent nom_tokens() implementation
    //
    // return a list of enum Token<'a> of type B or T (Boundary or Token)
    // each Token holds a reference into the input string
    //
    // joining the contents of the list would reproduce the input
    //    assert_eq!(Token::joined(&tokens), input);
    pub fn tokens<'a>(&self, input: &'a str) -> Result<Vec<Token<'a>>, Box<dyn Error>> {
        let mut i = 0;
        let mut b = 0;
        let mut t = 0;
        let mut tokens = vec![];

        for c in input.chars() {
            // str references are being returned (indexed by utf8 units)
            let c_len = c.len_utf8();

            if self.boundary_predicate(c) {
                // finalize previous token if needed
                if i > t {
                    tokens.push(Token::T(&input[t..i]));
                }
                i += c_len;
                t = i;
            } else {
                // finalize previous boundary if needed
                if i > b {
                    tokens.push(Token::B(&input[b..i]));
                }
                i += c_len;
                b = i;
            }
        }

        // finalize the token which was last being processed
        if i > b {
            tokens.push(Token::B(&input[b..i]));
        } else if i > t {
            tokens.push(Token::T(&input[t..i]));
        }

        Ok(tokens)
    }

    // return a list of enum Token<'a> of type B or T (Boundary or Token)
    // each Token holds a reference into the input string as found by the
    // nom parser https://github.com/Geal/nom
    //
    // joining the contents of the list would reproduce the input
    //    assert_eq!(Token::joined(&tokens), input);
    pub fn nom_tokens<'a>(&self, input: &'a str) -> Result<Vec<Token<'a>>, Box<dyn Error + 'a>> {
        let boundary_predicate = |c| self.boundary_predicate(c);

        // The parser walks the input emitting a pair (Token::B, Token::T)
        // either of which may be empty, but not both (the stopping condition)
        //
        // tuple (output of each pair parse)
        // -------------------------------+
        //                                |
        // many0: collect into a Vec,     |
        // verify pair parse not empty    |
        // ----------------------+        |
        //                       |        |
        // unparsed input        |     (B , T)
        // -----------------+    |     +-----+
        //                  |    |     |     |
        //                  v    v     v     v
        let parse: IResult<&str, Vec<(Token, Token)>> = many0(map(
            verify(
                pair(take_while(boundary_predicate), take_till(boundary_predicate)),
                |p: &(&str, &str)| !p.0.is_empty() || !p.1.is_empty(),
            ),
            |p: (&str, &str)| (Token::B(p.0), Token::T(p.1)),
        ))(input);

        let (unparsed, value) = parse?;
        // if this isn't true we don't understand our parser
        assert!(unparsed.is_empty(), "unparsed input = {}", unparsed);

        let mut tokens = vec![];
        for (b, t) in value.into_iter() {
            // technically, only the endpoints need to be tested for empty
            if !b.is_empty() {
                tokens.push(b);
            }
            if !t.is_empty() {
                tokens.push(t);
            }
        }

        // if this isn't true we don't understand our parser
        assert_eq!(Token::joined(&tokens), input);
        Ok(tokens)
    }

    // filters the tokens on Token::T() and returns a list of their references
    pub fn words<'a>(&self, text: &'a str) -> Result<Vec<&'a str>, Box<dyn Error>> {
        Ok(self
            .tokens(text)?
            .into_iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.str_value())
            .collect::<Vec<_>>())
    }

    // filters the tokens on Token::T() and returns a list of their references.to_string()
    pub fn text_words(&self, text: &str) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self
            .tokens(text)?
            .iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.value())
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Token::{B, T};

    #[test]
    fn empty() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn empty_excluded() {
        let excluded = "";
        let wbt = WordBoundaryTokenizer::new(excluded);

        let input = "";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn b() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn t() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "a";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",,";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",,")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "aa";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("aa")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",a";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("a")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "a,";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(",")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn btb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",a;";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("a"), B(";")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tbt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "a,b";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(","), T("b")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bbt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",;a";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",;"), T("a")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn ttb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "ab,";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("ab"), B(",")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn btt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",ab";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("ab")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tbb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "a,;";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(",;")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bttb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",ab;";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("ab"), B(";")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tbbt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "a,;b";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(",;"), T("b")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bbtbb() {
        let wbt = WordBoundaryTokenizer::default();

        let input = ",;a.!";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",;"), T("a"), B(".!")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn ttbtt() {
        let wbt = WordBoundaryTokenizer::default();

        let input = "ab,cd";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("ab"), B(","), T("cd")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn emoji_excluded() {
        let excluded = "'🍺🍕";
        let wbt = WordBoundaryTokenizer::new(excluded);

        let input = "Don't forget the 🍺+🍕 party!x";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        let words = wbt.words(input);
        assert_eq!(
            words.as_ref().unwrap(),
            &vec!["Don't", "forget", "the", "🍺", "🍕", "party", "x"]
        );

        assert_eq!(
            tokens.as_ref().unwrap(),
            &vec![
                T("Don't"),
                B(" "),
                T("forget"),
                B(" "),
                T("the"),
                B(" "),
                T("🍺"),
                B("+"),
                T("🍕"),
                B(" "),
                T("party"),
                B("!"),
                T("x")
            ]
        );
        assert_eq!(input, Token::joined(&tokens.unwrap()));

        let text_words = wbt.text_words(input);
        assert_eq!(
            text_words.as_ref().unwrap(),
            &vec!["Don't", "forget", "the", "🍺", "🍕", "party", "x"]
        );
    }

    #[test]
    fn other() {
        let excluded = "'¡";
        let wbt = WordBoundaryTokenizer::new(excluded);

        let input = "Thorbjørn Risager, Sinéad O'Connor, ¡Americano!";
        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        let text_words = wbt.text_words(input);
        assert_eq!(
            text_words.as_ref().unwrap(),
            &vec!["Thorbjørn", "Risager", "Sinéad", "O'Connor", "¡Americano"]
        );
    }
}
