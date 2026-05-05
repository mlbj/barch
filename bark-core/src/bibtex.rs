pub fn extract_field_bibtex(bibtex: &str, field: &str) -> Option<String> {
    for line in bibtex.lines() {
        let line = line.trim();

        if let Some((key, value)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case(field) {
                return Some(
                    value
                        .trim()
                        .trim_matches(|c| c == '{' || c == '}' || c == ',')
                        .trim()
                        .to_string(),
                );
            }
        }
    }
    None
}

pub fn parse_bibtex_header(bibtex: &str) -> Option<(String, String)> {
    let first_line = bibtex.lines().next()?.trim();

    // Expect something like: @book{key,
    if !first_line.starts_with('@') {
        return None;
    }

    let after_at = &first_line[1..];
    let mut parts = after_at.splitn(2, '{');

    let entry_type = parts.next()?.trim().to_string();
    let rest = parts.next()?;

    let entry_key = rest.split(',').next()?.trim().to_string();

    Some((entry_type, entry_key))
}

pub fn split_bibtex_entries(input: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut current = String::new();
    let mut brace_level = 0;
    let mut in_entry = false;

    for c in input.chars() {
        if c == '@' && !in_entry {
            in_entry = true;
            current.clear();
        }

        if in_entry { 
            current.push(c);

            if c == '{' {
                brace_level += 1;
            } else if c == '}' {
                brace_level -= 1;

                if brace_level == 0 {
                    entries.push(current.trim().to_string());
                    in_entry = false;
                }
            }
        }
    }

    entries
}
