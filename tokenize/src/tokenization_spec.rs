use crate::TokenizerType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
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
