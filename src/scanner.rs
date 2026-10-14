use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

const DOC_EXTS: &[&str] = &["md", "mdx", "rst", "txt", "adoc"];
const PRIORITY_FILES: &[&str] = &["AGENTS.md", "CLAUDE.md", "llms.txt", "README.md"];
const MAX_FILE_SIZE: u64 = 500_000;

#[derive(Debug)]
pub struct DocFile {
    pub path: String,
    pub content: String,
}

pub fn scan_docs(root: &Path) -> Result<Vec<DocFile>> {
    let mut docs = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        if !DOC_EXTS.contains(&ext) && !PRIORITY_FILES.contains(&name) {
            continue;
        }
        if entry.metadata().map(|m| m.len()).unwrap_or(0) > MAX_FILE_SIZE {
            continue;
        }

        let content = std::fs::read_to_string(path).unwrap_or_default();
        docs.push(DocFile {
            path: path.to_string_lossy().into_owned(),
            content,
        });
    }
    Ok(docs)
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if name.starts_with('.') && name != ".github" {
        return true;
    }
    matches!(
        name.as_ref(),
        "node_modules" | "target" | "dist" | "build" | "vendor" | "__pycache__" | "coverage"
    )
}
