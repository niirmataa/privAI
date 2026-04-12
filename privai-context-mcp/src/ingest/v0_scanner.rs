use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Result;

pub fn scan_v0_markdown(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with("TASK_") || name.ends_with("_WORKING_CONTEXT.md") {
            continue;
        }
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}
