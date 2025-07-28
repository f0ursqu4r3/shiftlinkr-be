use regex::Regex;

pub fn clean_sql(sql: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    let replaced = re.replace_all(sql, " ").to_string();
    replaced.trim().to_string()
}

pub fn sql(query: &str) -> String {
    let cleaned = clean_sql(query);
    let re = Regex::new(r"\?").unwrap();
    let mut param_index = 1;
    let mut result = cleaned.clone();
    while let Some(mat) = re.find(&result) {
        let replacement = format!("${}", param_index);
        result.replace_range(mat.range(), &replacement);
        param_index += 1;
    }
    result
}
