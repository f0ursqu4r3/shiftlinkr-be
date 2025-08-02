use regex::Regex;

pub fn sql(query: &str) -> String {
    let cleaned = query.split_whitespace().collect::<Vec<&str>>().join(" ");
    let re = Regex::new(r"\?").unwrap();
    let mut param_index = 1;
    let mut result = cleaned;
    while let Some(mat) = re.find(&result) {
        let replacement = format!("${}", param_index);
        result.replace_range(mat.range(), &replacement);
        param_index += 1;
    }
    result
}
