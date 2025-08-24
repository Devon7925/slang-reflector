pub(crate) fn unescape(input: &str) -> Result<String, String> {
    let mut chars = input.chars().peekable();

    // expect surrounding quotes
    if chars.next() != Some('"') || chars.clone().last() != Some('"') {
        return Err("missing surrounding quotes".into());
    }

    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '"' && chars.peek().is_none() {
            break; // closing quote
        }
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('u') => {
                if chars.next() != Some('{') {
                    return Err("bad unicode escape".into());
                }
                let mut hex = String::new();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '}' {
                        break;
                    }
                    hex.push(ch);
                }
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|_| "invalid hex")?;
                match char::from_u32(code) {
                    Some(ch) => out.push(ch),
                    None => return Err("invalid codepoint".into()),
                }
            }
            Some(c) => return Err(format!("unsupported escape: {}", c)),
            None => return Err("unterminated escape".into()),
        }
    }
    Ok(out)
}