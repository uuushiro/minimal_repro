pub(crate) fn escape_like_pattern(name: &str) -> String {
    name.replace("\\", "\\\\")
        .replace("%", "\\%")
        .replace("_", "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_like_pattern() {
        let name = "test%\\n_";
        let escaped_name = escape_like_pattern(name);
        assert_eq!(escaped_name, "test\\%\\\\n\\_");
    }
}
