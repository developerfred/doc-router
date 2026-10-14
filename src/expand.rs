use crate::{parser::build_tree, scanner::DocFile};
use anyhow::{anyhow, Result};
use std::path::Path;

pub fn expand_section(root: &Path, section_id: &str) -> Result<String> {
    let parts: Vec<&str> = section_id.split("::").collect();
    if parts.len() < 3 {
        return Err(anyhow!("invalid section_id: {section_id}"));
    }
    let doc_path = parts[0];

    let full = if Path::new(doc_path).is_absolute() {
        Path::new(doc_path).to_path_buf()
    } else {
        root.join(doc_path)
    };

    let content = std::fs::read_to_string(&full)
        .map_err(|e| anyhow!("cannot read {}: {}", full.display(), e))?;

    let doc = DocFile {
        path: doc_path.to_string(),
        content: content.clone(),
    };
    let tree = build_tree(&[doc])?;
    let parsed = tree.first().ok_or_else(|| anyhow!("no sections"))?;

    let section = parsed
        .sections
        .iter()
        .find(|s| s.id == section_id)
        .ok_or_else(|| anyhow!("section not found: {section_id}"))?;

    Ok(
        content[section.byte_start.min(content.len())..section.byte_end.min(content.len())]
            .to_string(),
    )
}
