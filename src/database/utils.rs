pub fn clean_sql(sql: &str) -> String {
    sql.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn sql(query: &str) -> String {
    let mut result = String::new();
    let mut param_index = 1;
    for ch in clean_sql(query).chars() {
        if ch == '?' {
            result.push_str(&format!("${}", param_index));
            param_index += 1;
        } else {
            result.push(ch);
        }
    }
    result
}
