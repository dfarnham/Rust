use crate::TokenizationConfig;
use crate::WordTokenizer;
use crate::WordTokens;

//================================================
// Tokenizer holds an instantiated WordTokenizer
// and config rules built from a TokenizationSpec
//
// text to tokens recipe:
//    1. downcase the text (true/false)
//    2. apply WordTokenizer to text
//    3. whitespace trim tokens (true/false)
//    4. discard tokens matching a RE
//================================================
pub enum Tokenizer {
    Spec(TokenizationConfig, WordTokenizer),
}
impl Tokenizer {
    fn transform_filter(config: &TokenizationConfig, words: Vec<String>) -> Vec<String> {
        let tokens = match config.trimmed_tokens {
            true => words.into_iter().map(|t| t.trim().into()).collect(),
            false => words,
        };
        match &config.filter_tokens_re {
            Some(re) => tokens.into_iter().filter(|tok| !re.is_match(tok)).collect(),
            None => tokens,
        }
    }

    pub fn tokens(&self, text: &str) -> Vec<String> {
        match self {
            Self::Spec(config, tokenizer) => match config.downcase_text {
                true => Self::transform_filter(config, tokenizer.words(&text.to_lowercase())),
                false => Self::transform_filter(config, tokenizer.words(text)),
            },
        }
    }
}
