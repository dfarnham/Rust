use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
// a Token<'a> of type B or T (Boundary or Token)
// each Token holds a reference into an input string which was
pub enum Token<'a> {
    B(&'a str),
    T(&'a str),
}
#[allow(dead_code)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RegexBoundaryTokenizer {
    // chars in "excluded_boundary_chars" that would typically return true on Regex \b that will now return false
    excluded_boundary_chars: String,
}
impl RegexBoundaryTokenizer {
    pub fn new(excluded_boundary_chars: Option<String>) -> Self {
        Self {
            excluded_boundary_chars: excluded_boundary_chars.unwrap_or_else(|| "".into()),
        }
    }

    pub fn boundary_predicate(&self, c: char) -> bool {
        lazy_static! {
            static ref REGEX_BOUNDARY_CHAR: Regex = Regex::new(r"^X\b").unwrap();
        }
        !self.excluded_boundary_chars.contains(c) && REGEX_BOUNDARY_CHAR.is_match(&("X".to_string() + &c.to_string()))
    }

    // return a list of enum Token<'a> of type B or T (Boundary or Token)
    // each Token holds a reference into the input string
    //
    // joining the contents of the list would reproduce the input
    //    assert_eq!(Token::joined(&tokens), input);
    pub fn tokens<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
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

        tokens
    }

    // returns a string list of all tokens
    pub fn text_tokens(&self, text: &str) -> Vec<String> {
        self.tokens(text).iter().map(|t| t.value()).collect()
    }

    // filters the tokens on Token::T() and returns a reference list
    pub fn ref_words<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.tokens(text)
            .into_iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.str_value())
            .collect()
    }

    // filters the tokens on Token::T() and returns a string list
    pub fn words(&self, text: &str) -> Vec<String> {
        self.tokens(text)
            .iter()
            .filter(|t| matches!(t, Token::T(_)))
            .map(|t| t.value())
            .collect()
    }

    // filters the tokens on Token::B() and returns a string list
    pub fn boundaries(&self, text: &str) -> Vec<String> {
        self.tokens(text)
            .iter()
            .filter(|t| matches!(t, Token::B(_)))
            .map(|t| t.value())
            .collect()
    }
}

// ========================================================
// ========================================================

#[cfg(test)]
mod tests {
    use super::Token::{B, T};
    use super::*;

    #[test]
    fn empty() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "";
        let tokens = wbt.tokens(input);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn empty_excluded() {
        let excluded = Some("".into());
        let wbt = RegexBoundaryTokenizer::new(excluded);

        let input = "";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn b() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(",")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn t() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "a";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("a")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn bb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",,";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(",,")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn tt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "aa";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("aa")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn bt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",a";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(","), T("a")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn tb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "a,";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("a"), B(",")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn btb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",a;";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(","), T("a"), B(";")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn tbt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "a,b";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("a"), B(","), T("b")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn bbt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",;a";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(",;"), T("a")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn ttb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "ab,";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("ab"), B(",")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn btt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",ab";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(","), T("ab")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn tbb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "a,;";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("a"), B(",;")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn bttb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",ab;";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(","), T("ab"), B(";")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn tbbt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "a,;b";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("a"), B(",;"), T("b")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn bbtbb() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = ",;a.!";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![B(",;"), T("a"), B(".!")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn ttbtt() {
        let wbt = RegexBoundaryTokenizer::default();

        let input = "ab,cd";
        let tokens = wbt.tokens(input);

        assert_eq!(tokens, vec![T("ab"), B(","), T("cd")]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn emoji_excluded() {
        let excluded = Some("'🍺🍕".into());
        let wbt = RegexBoundaryTokenizer::new(excluded);

        let input = "Don't forget the 🍺+🍕 party!x";
        let tokens = wbt.tokens(input);

        let words = wbt.words(input);
        assert_eq!(words, vec!["Don't", "forget", "the", "🍺", "🍕", "party", "x"]);

        assert_eq!(
            tokens,
            vec![
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

        let words = wbt.words(input);
        assert_eq!(words, vec!["Don't", "forget", "the", "🍺", "🍕", "party", "x"]);
        assert_eq!(input, Token::joined(&tokens));
    }

    #[test]
    fn other() {
        let excluded = Some("'¡".into());
        let wbt = RegexBoundaryTokenizer::new(excluded);

        let input = "Thorbjørn Risager, Sinéad O'Connor, ¡Americano!";
        let tokens = wbt.tokens(input);

        let words = wbt.words(input);
        assert_eq!(words, vec!["Thorbjørn", "Risager", "Sinéad", "O'Connor", "¡Americano"]);
        assert_eq!(input, Token::joined(&tokens));
    }
}
