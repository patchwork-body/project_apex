/// Convert a snake_case string to PascalCase
pub(crate) fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("counter"), "Counter");
        assert_eq!(to_pascal_case("my_component"), "MyComponent");
        assert_eq!(
            to_pascal_case("hello_world_component"),
            "HelloWorldComponent"
        );
        assert_eq!(to_pascal_case("a"), "A");
        assert_eq!(to_pascal_case(""), "");
    }
}
