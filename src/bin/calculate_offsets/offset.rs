use itertools::Itertools;

#[derive(Debug)]
pub struct Offset {
    pub name: String,
    pub start: usize,
    pub excluded_end: usize,
    pub length: usize,
}

impl Offset {
    fn row_format(&self) -> String {
        format!(
            "| `{}` | {}..{} | {} |",
            self.name, self.start, self.excluded_end, self.length
        )
    }
}

pub fn to_markdown(offsets: Vec<Offset>) -> String {
    let mut res = String::from("| Name | Byte Range | Length (bytes) |\n|:--- |:--- |:--- |");
    for offset in offsets
        .iter()
        .sorted_by(|a, b| Ord::cmp(&a.start, &b.start))
    {
        res = format!("{}\n{}", res, offset.row_format())
    }
    res
}
