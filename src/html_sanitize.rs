//! HTML sanitization utils

/// Replace quote literals ('"') with "&quot;"
pub fn encode_quotes(str: &str) -> String {
    let mut out = String::with_capacity(str.len());
    for char in str.chars() {
        if char == '"' {
            out.push_str("&quot;");
        } else {
            out.push(char);
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_escape_string() {
        let result = encode_quotes(r#"What is up "man?""#);
        assert_eq!(result, r#"What is up &quot;man?&quot;"#);
    }
}
