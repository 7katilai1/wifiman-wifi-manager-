/// Parse a line separated by ':' in nmcli -t mode (supports escaped ':')
pub fn split_nmcli_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                current.push(next);
            }
        } else if c == ':' {
            parts.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    parts.push(current);
    parts
}

/// Clean nmcli "--" empty value
pub fn clean_dash(val: Option<&String>) -> String {
    match val {
        Some(s) if s != "--" && !s.is_empty() => s.clone(),
        _ => String::new(),
    }
}

/// Parse comma or space separated list, skip "--" and empty values
pub fn parse_nmcli_list(val: Option<&String>) -> Vec<String> {
    match val {
        Some(s) if s != "--" && !s.is_empty() => {
            s.split(|c| c == ',' || c == ' ')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s != "--")
                .collect()
        }
        _ => Vec::new(),
    }
}
