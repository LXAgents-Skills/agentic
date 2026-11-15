//! How a file in `content/` gets its tool ID.
//!
//! ```text
//! {org}_{repo}_{base}
//! ```
//!
//! `{org}_{repo}` is the tool prefix from `identity`. `{base}` is the filename: folder dropped,
//! `.md` dropped, lowercased, `-` turned into `_`. Full rules, and when to add an override:
//! `.agents/rules/tool-ids.md`.

use crate::identity::tool_id;

/// The longest ID this server publishes. The protocol allows 128, but several clients cap a
/// tool name at 64, so every ID is held to 64 at boot.
pub const MAX_ID_LENGTH: usize = 64;

/// Base names that do not survive mechanical derivation, keyed by set-relative path.
///
/// Use one when the derived ID would be longer than `MAX_ID_LENGTH`, when two folders hold the
/// same filename, or when the filename says nothing about the file. **Only the base is ever
/// shortened; the prefix is added by `tool_id` and never edited.** A key that matches no file
/// is a startup error, so a rename cannot leave an override matching nothing.
///
/// Example: `("references/javascript-typescript-react-web-frontend-security.md",
/// "js_ts_react_web_frontend_security")`.
pub const OVERRIDE_BASES: &[(&str, &str)] = &[];

/// The base name a path derives on its own.
pub fn derived_base(relative_path: &str) -> String {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    file_name
        .strip_suffix(".md")
        .unwrap_or(file_name)
        .to_lowercase()
        .replace('-', "_")
}

/// The tool ID for a set-relative path.
pub fn tool_name_for(relative_path: &str, overrides: &[(&str, &str)]) -> String {
    match overrides.iter().find(|(path, _)| *path == relative_path) {
        Some((_, base)) => tool_id(base),
        None => tool_id(&derived_base(relative_path)),
    }
}

/// Why `name` cannot be published, or `None` when it can.
pub fn name_problem(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(|ch| ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_');
    if !valid {
        return Some(format!(
            "\"{name}\" is not a valid tool name (lowercase letters, digits and underscores, starting with a letter)"
        ));
    }
    if name.len() > MAX_ID_LENGTH {
        return Some(format!(
            "\"{name}\" is {} characters, over the {MAX_ID_LENGTH}-character cap",
            name.len()
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_is_the_filename_folder_and_md_dropped_lowercased_dash_to_underscore() {
        assert_eq!(derived_base("git/Branching-Strategy.md"), "branching_strategy");
        assert_eq!(
            tool_name_for("a/b/example-guide.md", &[]),
            tool_id("example_guide")
        );
    }

    #[test]
    fn an_override_replaces_the_base_only_never_the_prefix() {
        let overrides = [("references/very-long-name.md", "short")];
        assert_eq!(
            tool_name_for("references/very-long-name.md", &overrides),
            tool_id("short")
        );
    }

    #[test]
    fn an_id_over_64_characters_or_with_a_bad_character_is_refused() {
        assert_eq!(name_problem(&"a".repeat(MAX_ID_LENGTH)), None);
        assert!(
            name_problem(&"a".repeat(MAX_ID_LENGTH + 1))
                .unwrap()
                .contains("65 characters")
        );
        assert!(name_problem("1abc").unwrap().contains("not a valid tool name"));
        assert!(name_problem("ab.c").unwrap().contains("not a valid tool name"));
        assert!(name_problem("").unwrap().contains("not a valid tool name"));
    }
}
