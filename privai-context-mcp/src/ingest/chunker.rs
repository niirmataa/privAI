#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownChunk {
    pub heading: String,
    pub content: String,
}

pub fn chunk_by_h2(markdown: &str) -> Vec<MarkdownChunk> {
    let mut chunks = Vec::new();
    let mut heading = String::from("document");
    let mut content = String::new();

    for line in markdown.lines() {
        if line.starts_with("## ") {
            if !content.trim().is_empty() {
                chunks.push(MarkdownChunk {
                    heading: heading.clone(),
                    content: content.trim().to_string(),
                });
                content.clear();
            }
            heading = line.trim_start_matches("## ").trim().to_string();
        } else {
            content.push_str(line);
            content.push('\n');
        }
    }

    if !content.trim().is_empty() {
        chunks.push(MarkdownChunk {
            heading,
            content: content.trim().to_string(),
        });
    }

    chunks
}
