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
// each Token holds a reference into an input string which was
// parsed by a nom parser https://github.com/Geal/nom
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

#[derive(Debug)]
pub struct WordBoundaryTokenizer {
    exclude_boundary_chars: String,
}
impl WordBoundaryTokenizer {
    pub fn default() -> Self {
        Self::new("")
    }
    pub fn new(exclude_boundary_chars: &'static str) -> Self {
        Self {
            exclude_boundary_chars: exclude_boundary_chars.to_string(),
        }
    }

    fn is_regex_boundary(c: char) -> bool {
        lazy_static! {
            static ref WORD_BOUNDARY: Regex = Regex::new(r"^X\b").unwrap();
        }
        let xc = "X".to_string() + &c.to_string();
        WORD_BOUNDARY.is_match(&xc)
    }

    // return a list of enum Token<'a> of type B or T (Boundary or Token)
    // each Token holds a reference into the input string as found by the
    // nom parser https://github.com/Geal/nom
    //
    // joining the contents of the list would reproduce the input
    //    assert_eq!(Token::joined(&tokens), input);
    pub fn nom_tokens<'a>(&self, input: &'a str) -> Result<Vec<Token<'a>>, Box<dyn Error + 'a>> {
        let boundary_predicate =
            |c| !&self.exclude_boundary_chars.contains(c) && WordBoundaryTokenizer::is_regex_boundary(c);

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

    // this is 30% faster than nom_tokens()
    pub fn tokens<'a>(&self, input: &'a str) -> Result<Vec<Token<'a>>, Box<dyn Error>> {
        let boundary_predicate =
            |c| !&self.exclude_boundary_chars.contains(c) && WordBoundaryTokenizer::is_regex_boundary(c);

        let mut i = 0;
        let mut b = 0;
        let mut t = 0;
        let mut tokens = vec![];

        for c in input.chars() {
            // str references are being returned (indexed by utf8 units)
            let c_len = c.len_utf8();

            if boundary_predicate(c) {
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

    // filters the list on Token::T() and returns a list of their references
    pub fn words<'a>(&self, text: &'a str) -> Result<Vec<&'a str>, Box<dyn Error + 'a>> {
        Ok(self
            .tokens(text)?
            .into_iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.str_value())
            .collect::<Vec<_>>())
    }

    // filters the list on Token::T() and returns a list of their references.to_string()
    pub fn text_words<'a>(&self, text: &'a str) -> Result<Vec<String>, Box<dyn Error + 'a>> {
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
        let input = "";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn b() {
        let input = ",";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn t() {
        let input = "a";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bb() {
        let input = ",,";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",,")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tt() {
        let input = "aa";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("aa")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bt() {
        let input = ",a";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("a")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tb() {
        let input = "a,";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(",")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn btb() {
        let input = ",a;";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("a"), B(";")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tbt() {
        let input = "a,b";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(","), T("b")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bttb() {
        let input = ",ab;";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(","), T("ab"), B(";")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn tbbt() {
        let input = "a,;b";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("a"), B(",;"), T("b")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bbtbb() {
        let input = ",;a.!";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",;"), T("a"), B(".!")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn ttbtt() {
        let input = "ab,cd";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("ab"), B(","), T("cd")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn bbt() {
        let input = ",;a";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B(",;"), T("a")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn ttb() {
        let input = "ab,";
        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("ab"), B(",")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn test1() {
        let allowed = "'🍺🍕";
        let input = "Don't forget the 🍺+🍕 party!x";

        let wbt = WordBoundaryTokenizer::new(allowed);

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
    fn test2() {
        let allowed = "'";
        let input = "'";

        let wbt = WordBoundaryTokenizer::new(allowed);

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![T("'")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }

    #[test]
    fn test3() {
        let input = "'";

        let wbt = WordBoundaryTokenizer::default();

        let tokens = wbt.tokens(input);
        let nom_tokens = wbt.nom_tokens(input);
        assert_eq!(tokens.as_ref().unwrap(), nom_tokens.as_ref().unwrap());

        assert_eq!(tokens.as_ref().unwrap(), &vec![B("'")]);
        assert_eq!(input, Token::joined(&tokens.unwrap()));
    }
}
