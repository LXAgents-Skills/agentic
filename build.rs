//! Embed `content/` in the binary.
//!
//! Walks `content/` and writes `$OUT_DIR/content_files.rs`: one `(path, text)` pair per `.md`
//! file, with the path set-relative and forward-slashed, sorted. The text is pulled in with
//! `include_str!`, so it is the file byte for byte. Cargo re-runs this script whenever anything
//! under `content/` changes, so adding a file and rebuilding is adding a tool.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn markdown_files(dir: &Path, base: &Path, found: &mut Vec<String>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            markdown_files(&path, base, found);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let relative = path.strip_prefix(base).expect("path under content/");
            let parts: Vec<_> = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect();
            found.push(parts.join("/"));
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let content = root.join("content");
    println!("cargo::rerun-if-changed=content");

    let mut files = Vec::new();
    if content.is_dir() {
        markdown_files(&content, &content, &mut files);
    }
    files.sort();

    let mut out = String::from("/// Every `.md` file in `content/`: set-relative path, text.\n");
    out.push_str("pub static CONTENT_FILES: &[(&str, &str)] = &[\n");
    for path in &files {
        let full = content.join(path);
        out.push_str(&format!(
            "    ({path:?}, include_str!({:?})),\n",
            full.display().to_string()
        ));
    }
    out.push_str("];\n");

    let target = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("content_files.rs");
    fs::write(target, out).expect("write content_files.rs");
}
