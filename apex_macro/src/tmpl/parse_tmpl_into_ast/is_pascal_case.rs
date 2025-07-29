pub(crate) fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start with uppercase letter
    let mut chars = s.chars();

    if let Some(first_char) = chars.next() {
        if !first_char.is_ascii_uppercase() {
            return false;
        }
    }

    // Check rest of string: only letters and digits, no underscores/hyphens
    for ch in chars {
        if !ch.is_ascii_alphanumeric() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pascal_case() {
        assert!(is_pascal_case("HelloWorld"));
    }

    #[test]
    fn invalid_pascal_case() {
        assert!(!is_pascal_case("helloWorld"));
    }
}
