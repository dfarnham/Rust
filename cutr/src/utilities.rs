use crate::FromStr;

// splits and optionally trims the input String on a separator character
// returns a Vec of parse::<T>() over the splits
fn split_on<T>(text: &str, sep: char, trim: bool) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: 'static,
{
    let mut parsed_splits = vec![];
    for mut s in text.split(sep) {
        if trim {
            s = s.trim();
        }
        parsed_splits.push(s.parse::<T>()?)
    }
    Ok(parsed_splits)
}

// ==============================================================

// return a list of String tokens
pub fn tokens(text: &str, delim: Option<char>, trim: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    match delim {
        Some(c) => split_on::<String>(text, c, trim),
        _ => Ok(text.split_whitespace().map(String::from).collect()),
    }
}
