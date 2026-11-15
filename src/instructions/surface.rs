//! The surface: one tool per embedded `.md` file, checked once at boot.
//!
//! Adding a markdown file to `content/` and rebuilding is adding a tool. There is no tool
//! module to write and no list to edit. A malformed set stops the process with a message
//! naming the file and the fix, instead of answering the first caller wrongly.

use std::sync::OnceLock;

use super::{
    frontmatter::parse_frontmatter,
    naming::{OVERRIDE_BASES, name_problem, tool_name_for},
};

include!(concat!(env!("OUT_DIR"), "/content_files.rs"));

/// One served file: its tool name, the description a client routes on, and the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub name: String,
    pub description: String,
    pub path: &'static str,
    pub text: &'static str,
}

/// Build the surface from a set of `(path, text)` files.
///
/// Each tool serves its file whole, frontmatter included, byte for byte. No tool takes an
/// argument, so there is no path for a caller to traverse with.
pub fn build_surface(
    files: &[(&'static str, &'static str)],
    overrides: &[(&str, &str)],
) -> Result<Vec<Instruction>, String> {
    for (key, _) in overrides {
        if !files.iter().any(|(path, _)| path == key) {
            return Err(format!(
                "OVERRIDE_BASES names {key}, which is not in the set. Remove the override or restore the file."
            ));
        }
    }

    let mut tools: Vec<Instruction> = Vec::with_capacity(files.len());
    for &(path, text) in files {
        let name = tool_name_for(path, overrides);
        if let Some(problem) = name_problem(&name) {
            return Err(format!(
                "{path}: {problem}. Shorten its base name in OVERRIDE_BASES (src/instructions/naming.rs)."
            ));
        }
        if let Some(other) = tools.iter().find(|tool| tool.name == name) {
            return Err(format!(
                "{path} and {} both derive \"{name}\". Give one an entry in OVERRIDE_BASES.",
                other.path
            ));
        }
        let description = parse_frontmatter(text)
            .and_then(|fields| fields.get("description").cloned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("{path} has no frontmatter `description:`. It would publish a tool no client can route on.")
            })?;
        tools.push(Instruction {
            name,
            description,
            path,
            text,
        });
    }

    if tools.is_empty() {
        return Err("No markdown files under content/. This server would expose nothing.".to_string());
    }
    Ok(tools)
}

/// The embedded set, built and checked once.
pub fn instructions() -> Result<&'static [Instruction], &'static str> {
    static SURFACE: OnceLock<Result<Vec<Instruction>, String>> = OnceLock::new();
    match SURFACE.get_or_init(|| build_surface(CONTENT_FILES, OVERRIDE_BASES)) {
        Ok(tools) => Ok(tools),
        Err(problem) => Err(problem),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::tool_id;

    const DOC: &str = "---\nname: x\ndescription: A file.\n---\n\nBody.\n";

    #[test]
    fn the_embedded_set_builds() {
        let tools = instructions().expect("content/ builds");
        assert_eq!(tools.len(), CONTENT_FILES.len());
    }

    #[test]
    fn a_file_with_no_description_does_not_build() {
        let error = build_surface(&[("a.md", "# no frontmatter\n")], &[]).unwrap_err();
        assert!(error.contains("no frontmatter `description:`"), "{error}");
    }

    #[test]
    fn two_files_deriving_one_id_do_not_build() {
        let error = build_surface(&[("one/same.md", DOC), ("two/same.md", DOC)], &[]).unwrap_err();
        assert!(error.contains("both derive"), "{error}");
    }

    #[test]
    fn an_override_that_matches_no_file_does_not_build() {
        let error = build_surface(&[("a.md", DOC)], &[("gone.md", "gone")]).unwrap_err();
        assert!(error.contains("not in the set"), "{error}");
    }

    #[test]
    fn a_filename_too_long_for_64_characters_does_not_build_until_it_is_overridden() {
        let path: &'static str = Box::leak(format!("{}name.md", "long-".repeat(12)).into_boxed_str());
        let error = build_surface(&[(path, DOC)], &[]).unwrap_err();
        assert!(error.contains("OVERRIDE_BASES"), "{error}");
        let tools = build_surface(&[(path, DOC)], &[(path, "long_name")]).unwrap();
        assert_eq!(tools[0].name, tool_id("long_name"));
    }

    #[test]
    fn an_empty_set_does_not_build() {
        let error = build_surface(&[], &[]).unwrap_err();
        assert!(error.contains("expose nothing"), "{error}");
    }
}
