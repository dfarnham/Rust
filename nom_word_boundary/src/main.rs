use nom::{
    bytes::complete::{take_till, take_while},
    combinator::{map, verify},
    multi::many0,
    sequence::pair,
    IResult,
};
use regex::Regex;

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
    pub fn tokens<'a>(
        &self,
        input: &'a str,
    ) -> Result<Vec<Token<'a>>, Box<dyn std::error::Error + 'a>> {
        let boundary_predicate = |c| {
            !&self.exclude_boundary_chars.contains(c) && WordBoundaryTokenizer::is_regex_boundary(c)
        };

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
                pair(
                    take_while(boundary_predicate),
                    take_till(boundary_predicate),
                ),
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

    // filters the list on Token::T() and returns a list of their references
    pub fn words<'a>(
        &self,
        text: &'a str,
    ) -> Result<Vec<&'a str>, Box<dyn std::error::Error + 'a>> {
        Ok(self
            .tokens(text)?
            .into_iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.str_value())
            .collect::<Vec<_>>())
    }

    // filters the list on Token::T() and returns a list of their references.to_string()
    pub fn text_words<'a>(
        &self,
        text: &'a str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + 'a>> {
        Ok(self
            .tokens(text)?
            .iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.value())
            .collect::<Vec<_>>())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let allowed = "'🍺🍕";
    let input = "Don't forget the 🍺+🍕 party!x";
    println!("input = {input:?}");

    let wbt = WordBoundaryTokenizer::new(allowed);

    let words = wbt.words(input);
    println!("wbt words = {words:?}");

    let tokens = wbt.tokens(input);
    println!("wbt tokens = {tokens:?}");
    assert_eq!(input, Token::joined(&tokens.unwrap()));

    let text_words = wbt.text_words(input);
    println!("wbt text_words = {text_words:?}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Token::{B, T};

    #[test]
    fn test1() {
        let allowed = "'🍺🍕";
        let input = "Don't forget the 🍺+🍕 party!x";

        let wbt = WordBoundaryTokenizer::new(allowed);

        let words = wbt.words(input);
        assert_eq!(
            words.as_ref().unwrap(),
            &vec!["Don't", "forget", "the", "🍺", "🍕", "party", "x"]
        );

        let tokens = wbt.tokens(input);
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
}
