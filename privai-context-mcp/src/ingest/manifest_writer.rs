use std::{fs, path::Path};

use crate::{models::KnowledgeItem, Result};

pub fn write_manifest(path: &Path, items: &[KnowledgeItem]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(items)?;
    fs::write(path, json)?;
    Ok(())
}
