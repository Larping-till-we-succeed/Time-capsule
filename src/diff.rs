//! Diff helpers: decide whether a blob is text and build readable unified diffs.

use similar::TextDiff;

pub const TEXT_EXT: &[&str] = &[
    ".py", ".pyw", ".js", ".jsx", ".ts", ".tsx", ".json", ".md", ".txt", ".rst", ".css",
    ".scss", ".html", ".htm", ".xml", ".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf",
    ".sh", ".bash", ".zsh", ".c", ".h", ".cpp", ".hpp", ".cc", ".cs", ".java", ".kt", ".rs",
    ".go", ".rb", ".php", ".sql", ".lua", ".swift", ".dart", ".ex", ".exs", ".r", ".m", ".mm",
    ".vue", ".svelte", ".env", ".gitignore", ".dockerfile", ".gradle", ".properties", ".plist",
    ".tex", ".csv",
];

/// Cheap text heuristic: known extension or no NUL byte and valid UTF-8.
pub fn looks_text(name: &str, data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }
    let ext = match name.rfind('.') {
        Some(i) => name[i..].to_lowercase(),
        None => String::new(),
    };
    if TEXT_EXT.contains(&ext.as_str()) {
        return true;
    }
    let sample = &data[..data.len().min(8192)];
    if sample.contains(&0u8) {
        return false;
    }
    std::str::from_utf8(sample).is_ok()
}

pub fn decode(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string()
}

/// Unified diff as (kind, text) lines; kind is '+', '-', '@' or ' '.
pub fn unified(name: &str, old: &[u8], new: &[u8], context: usize) -> Vec<(char, String)> {
    let a = decode(old);
    let b = decode(new);
    let text = TextDiff::from_lines(&a, &b)
        .unified_diff()
        .context_radius(context)
        .header(&format!("a/{}", name), &format!("b/{}", name))
        .to_string();
    let mut out = Vec::new();
    for line in text.lines() {
        let kind = if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
            '@'
        } else if line.starts_with('+') {
            '+'
        } else if line.starts_with('-') {
            '-'
        } else {
            ' '
        };
        out.push((kind, line.to_string()));
    }
    out
}

pub fn line_counts(old: &[u8], new: &[u8]) -> (usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    for (kind, _) in unified("x", old, new, 0) {
        if kind == '+' {
            added += 1;
        } else if kind == '-' {
            removed += 1;
        }
    }
    (added, removed)
}
