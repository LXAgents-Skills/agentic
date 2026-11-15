//! Pull the single-line scalar fields out of a markdown file's frontmatter.
//!
//! Only `key: value` on one line is read. A folded or multi-line YAML value would need a real
//! parser, and a dependency added for one call site is not worth it; a `description:` this
//! misses is reported as missing at boot rather than served empty.

use std::collections::BTreeMap;

/// The block between the opening `---` line and the first line that starts with `---`.
fn block(text: &str) -> Option<&str> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let end = rest.find("\n---")?;
    let inner = &rest[..end];
    Some(inner.strip_suffix('\r').unwrap_or(inner))
}

/// `key: value` from one line, where the key is `[A-Za-z_][A-Za-z0-9_]*` and the value is
/// trimmed. A line whose value starts with whitespace other than spaces and tabs is not a field.
fn field(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    let mut chars = key.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    let value = value.trim_start_matches([' ', '\t']);
    match value.chars().next() {
        Some(ch) if ch.is_whitespace() => None,
        _ => Some((key, value.trim())),
    }
}

/// The fields, or `None` when there is no frontmatter.
pub fn parse_frontmatter(text: &str) -> Option<BTreeMap<String, String>> {
    let inner = block(text)?;
    let mut fields = BTreeMap::new();
    for line in inner.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some((key, value)) = field(line) {
            fields.insert(key.to_string(), value.to_string());
        }
    }
    Some(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_fields_are_read_and_trimmed() {
        let fields = parse_frontmatter("---\nname: x\ndescription:   A file.  \n---\n\nBody.\n").unwrap();
        assert_eq!(fields["name"], "x");
        assert_eq!(fields["description"], "A file.");
    }

    #[test]
    fn crlf_frontmatter_is_read() {
        let fields = parse_frontmatter("---\r\ndescription: Yes.\r\n---\r\n").unwrap();
        assert_eq!(fields["description"], "Yes.");
    }

    #[test]
    fn no_frontmatter_is_none() {
        assert_eq!(parse_frontmatter("# no frontmatter\n"), None);
        assert_eq!(parse_frontmatter("---\nunterminated: yes\n"), None);
    }

    #[test]
    fn a_folded_value_reads_as_empty() {
        let fields = parse_frontmatter("---\ndescription: >\n  folded\n---\n").unwrap();
        assert_eq!(fields["description"], ">");
        let fields = parse_frontmatter("---\ndescription:\n  folded\n---\n").unwrap();
        assert_eq!(fields["description"], "");
    }

    #[test]
    fn lines_that_are_not_fields_are_skipped() {
        let fields = parse_frontmatter("---\n1bad: no\n  indented: no\nok_1: yes\n---\n").unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields["ok_1"], "yes");
    }
}
