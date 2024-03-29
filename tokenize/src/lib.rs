use regex::Regex;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[macro_use]
extern crate lazy_static;

pub mod error;
use error::TokenizeError;

pub mod tokenizer;
use tokenizer::Tokenizer;

//================================================
// TokenizationSpec describes a rule set for
// transforming text, tokenizing, and filtering
//================================================
pub mod tokenization_spec;
pub use tokenization_spec::TokenizationSpec;

//================================================
//            Implemented Tokenizers
//================================================
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TokenizerType {
    SplitStr,
    UnicodeSegment,
    UnicodeWord,
    Whitespace,
    RegexBoundary,
}

//================================================
// A Tokenizer holds a TokenizationConfig which is
// built from fields in the TokenizationSpec
//================================================
#[derive(Clone, Debug)]
pub struct TokenizationConfig {
    downcase_text: bool,
    trimmed_tokens: bool,
    filter_tokens_re: Option<Regex>,
}

//================================================
// returns a Tokenizer given a TokenizationSpec
//================================================
pub fn tokenizer_from_spec(spec: &TokenizationSpec) -> Result<Tokenizer, TokenizeError> {
    // `param` is an Option<String> and is passed as a parameter
    // to WordTokenizers requiring some form of initialization
    //
    // 1. SplitStr supplies `param` as the String pattern to split()
    // 2. RegexBoundary interprets `param` as additional boundary chars
    let param = spec.tokenizer_init_param.clone();

    let word_tokenizer = match spec.tokenizer_type {
        TokenizerType::SplitStr => WordTokenizer::SplitStr(SplitStrTokenizer::new(param)),
        TokenizerType::UnicodeSegment => WordTokenizer::UnicodeSegment(UnicodeSegmentTokenizer),
        TokenizerType::UnicodeWord => WordTokenizer::UnicodeWord(UnicodeWordTokenizer),
        TokenizerType::Whitespace => WordTokenizer::Whitespace(WhitespaceTokenizer),
        TokenizerType::RegexBoundary => WordTokenizer::RegexBoundary(RegexBoundaryTokenizer::new(param)),
    };

    // build a Tokenizer from the `config` and instantiated WordTokenizer
    let config = TokenizationConfig {
        downcase_text: spec.downcase_text,
        trimmed_tokens: spec.trimmed_tokens,
        filter_tokens_re: spec.filter_tokens_re.as_ref().map(|re| Regex::new(re).unwrap()),
    };
    Ok(Tokenizer::Spec(config, word_tokenizer))
}

//================================================
//         WordTokenizer, WordTokens trait
// WordTokenizer: an object with a words() method
//                which returns a list of String
//================================================
#[enum_delegate::register]
trait WordTokens {
    // default implementation is Whitespace
    fn words(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }
}

#[enum_delegate::implement(WordTokens)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WordTokenizer {
    SplitStr(SplitStrTokenizer),
    UnicodeSegment(UnicodeSegmentTokenizer),
    UnicodeWord(UnicodeWordTokenizer),
    Whitespace(WhitespaceTokenizer),
    RegexBoundary(RegexBoundaryTokenizer),
}

// *********************************************************
//      WordTokens Trait impls for all TokenizerTypes
// *********************************************************

//================================================
//              SplitStr Tokenizer
//            TokenizerType::SplitStr
//================================================
pub mod splitstr;
use splitstr::SplitStrTokenizer;
impl WordTokens for SplitStrTokenizer {
    fn words(&self, text: &str) -> Vec<String> {
        self.words(text)
    }
}

//================================================
//           Unicode Segment Tokenizer
//         TokenizerType::UnicodeSegment
//================================================
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnicodeSegmentTokenizer;
impl WordTokens for UnicodeSegmentTokenizer {
    fn words(&self, text: &str) -> Vec<String> {
        text.split_word_bounds().map(String::from).collect()
    }
}

//================================================
//            Unicode Word Tokenizer
//          TokenizerType::UnicodeWord
//================================================
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnicodeWordTokenizer;
impl WordTokens for UnicodeWordTokenizer {
    fn words(&self, text: &str) -> Vec<String> {
        text.unicode_words().map(String::from).collect()
    }
}

//================================================
//             Whitespace Tokenizer
//           TokenizerType::Whitespace
//================================================
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WhitespaceTokenizer;
impl WordTokens for WhitespaceTokenizer {
    /* default trait implementation */
}

//================================================
//            Regex Boundary Tokenizer
//          TokenizerType::RegexBoundary
//================================================
pub mod regexboundary;
use regexboundary::RegexBoundaryTokenizer;
impl WordTokens for RegexBoundaryTokenizer {
    fn words(&self, text: &str) -> Vec<String> {
        self.words(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_normal<T: Clone + Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<TokenizerType>();
        is_normal::<TokenizationConfig>();
        is_normal::<WordTokenizer>();
        is_normal::<SplitStrTokenizer>();
        is_normal::<UnicodeSegmentTokenizer>();
        is_normal::<UnicodeWordTokenizer>();
        is_normal::<WhitespaceTokenizer>();
        is_normal::<RegexBoundaryTokenizer>();
        is_normal::<Tokenizer>()
    }
}
