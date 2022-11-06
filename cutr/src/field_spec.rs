use crate::Regex;

// Enum over the input -f arg types
// "-f 1" or "-f1"      =>  FieldSpec::Index(1)
// "-f 1,3" or "-f1,3"  =>  FieldSpec::Index(1), FieldSpec::Index(3)
// "-f 3-" or "-f3-"    =>  FieldSpec::OpenRange(3)
// "-f 3-7" or "-f3-7"  =>  FieldSpec::ClosedRange(3, 7)
// "-f-1"               =>  FieldSpec::Last(1)
// "-fr." or "-f r.     =>  computed indices on Regex header matches into => List(FieldSpec::Index)
// "-fR." or "-f R.     =>  FieldSpec::RegularExpression(re), computed indices on Regex data matches into => List(FieldSpec::Index)
#[derive(Debug)]
pub enum FieldSpec {
    Index(usize),
    Last(usize),
    OpenRange(usize),
    ClosedRange(usize, usize),
    RegularExpression(Regex),
}
impl FieldSpec {
    pub fn indices(&self, tokens: &[String]) -> Vec<usize> {
        let indices = |start: usize, end: usize| -> Vec<usize> {
            (match start <= end {
                true => (start..=end).collect::<Vec<_>>(),
                false => (end..=start).rev().collect::<Vec<_>>(),
            })
            .into_iter()
            .filter(|i| *i > 0 && *i <= tokens.len())
            .map(|i| i - 1)
            .collect()
        };
        match self {
            FieldSpec::Index(a) => indices(*a, *a),
            FieldSpec::OpenRange(a) => indices(*a, tokens.len()),
            FieldSpec::ClosedRange(a, b) => indices(*a, *b),
            FieldSpec::Last(a) => indices(tokens.len() + 1 - *a, tokens.len() + 1 - *a),
            FieldSpec::RegularExpression(re) => tokens
                .iter()
                .enumerate()
                .filter(|(_, txt)| re.is_match(txt))
                .flat_map(|(i, _)| indices(i + 1, i + 1))
                .collect(),
        }
    }
}
