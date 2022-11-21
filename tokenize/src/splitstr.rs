//***********************************************
//             SplitStr Tokenizer
//***********************************************
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitStrTokenizer {
    split_pattern: String,
}

impl SplitStrTokenizer {
    pub fn default() -> Self {
        Self::new(None)
    }

    pub fn new(split_pattern: Option<String>) -> Self {
        Self {
            split_pattern: split_pattern.unwrap_or_else(|| "".into()),
        }
    }

    pub fn words(&self, text: &str) -> Vec<String> {
        text.split(&self.split_pattern).map(String::from).collect()
    }
}
