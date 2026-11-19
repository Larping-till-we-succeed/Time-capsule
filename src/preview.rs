//! Real snapshot previews: the file a snapshot changed plus its first real lines.

use std::path::Path;

use egui::{Align2, Color32, FontId, Painter, Rect, Stroke, StrokeKind, Ui, Vec2};

use crate::store::{self, Snapshot};
use crate::theme::{self, ACCENT, DIM, MUTED};

pub const TEXT_EXT: &[&str] = &[
    ".py", ".pyi", ".js", ".jsx", ".ts", ".tsx", ".json", ".md", ".txt", ".yml", ".yaml",
    ".toml", ".ini", ".cfg", ".css", ".scss", ".html", ".htm", ".xml", ".sh", ".bash",
    ".zsh", ".sql", ".rs", ".go", ".java", ".kt", ".c", ".h", ".cpp", ".hpp", ".cs",
    ".rb", ".php", ".lua", ".vue", ".svelte", ".csv", ".log", ".env",
];
pub const TEXT_NAMES: &[&str] = &[
    "dockerfile", "makefile", "license", "readme", "procfile", ".gitignore",
    ".gitattributes", ".editorconfig", ".dockerignore",
];

#[derive(Clone, Debug)]
pub struct Preview {
    pub path: String,
    pub lines: Vec<String>,
    pub count: usize,
    pub bytes: u64,
    pub binary: bool,
}

fn is_text(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel).to_lowercase();
    if TEXT_NAMES.contains(&name.as_str()) {
        return true;
    }
    let ext = match name.rfind('.') {
        Some(i) => name[i..].to_string(),
        None => String::new(),
    };
    TEXT_EXT.contains(&ext.as_str())
}

fn text_rank(rel: &str) -> (usize, usize, String) {
    let name = rel.rsplit('/').next().unwrap_or(rel).to_lowercase();
    let ext = match name.rfind('.') {
        Some(i) => name[i..].to_string(),
        None => String::new(),
    };
    let prio = if name.starts_with("readme") {
        0
    } else if [
        ".py", ".js", ".ts", ".tsx", ".jsx", ".rs", ".go", ".java", ".c", ".cpp", ".cs",
        ".rb", ".kt",
    ]
    .contains(&ext.as_str())
    {
        1
    } else if [".md", ".txt", ".json", ".toml", ".yml", ".yaml"].contains(&ext.as_str()) {
        2
    } else {
        3
    };
    (rel.matches('/').count(), prio, rel.to_string())
}

pub fn snapshot_highlight(snap: &Snapshot) -> Option<String> {
    snap.files.keys().min_by_key(|rel| text_rank(rel)).cloned()
}

pub fn snapshot_preview(
    pid: &str,
    snap: &Snapshot,
    max_lines: usize,
    previous: Option<&Snapshot>,
) -> Option<Preview> {
    if pid.is_empty() {
        return None;
    }
    let files = &snap.files;
    if files.is_empty() {
        return None;
    }
    let mut rel: Option<String> = None;
    if let Some(prev) = previous {
        let mut touched: Vec<&String> = files
            .iter()
            .filter(|(r, info)| prev.files.get(*r).map(|b| &b.h) != Some(&info.h))
            .map(|(r, _)| r)
            .collect();
        touched.retain(|r| is_text(r));
        rel = touched.into_iter().min_by_key(|r| text_rank(r)).cloned();
    }
    if rel.is_none() {
        rel = snapshot_highlight(snap);
    }
    let rel = rel?;
    let info = files.get(&rel)?;
    let raw = store::read_blob(pid, &info.h);
    let raw = &raw[..raw.len().min(2400)];
    let binary = raw.contains(&0u8);
    let mut lines = Vec::new();
    if !binary {
        for line in String::from_utf8_lossy(raw).lines() {
            let line = line.replace('\t', "    ");
            let line = line.trim_end();
            if !line.trim().is_empty() {
                lines.push(line.chars().take(140).collect());
            }
            if lines.len() >= max_lines {
                break;
            }
        }
    }
    Some(Preview {
        path: rel,
        lines,
        count: files.len(),
        bytes: snap.bytes,
        binary,
    })
}

fn line_color(text: &str) -> Color32 {
    let s = text.trim_start();
    if s.starts_with('#')
        || s.starts_with("//")
        || s.starts_with("/*")
        || s.starts_with('*')
        || s.starts_with("--")
        || s.starts_with("<!--")
    {
        return Color32::from_rgb(0x4f, 0x7f, 0x9e);
    }
    for kw in [
        "def ", "class ", "import ", "from ", "function ", "const ", "let ", "var ", "return ",
        "if ", "for ", "while ", "export ",
    ] {
        if s.starts_with(kw) {
            return ACCENT;
        }
    }
    Color32::from_rgb(0xa8, 0xc6, 0xdd)
}

/// Paint a snapshot's real file path and the first real lines of its content.
pub fn paint_preview(painter: &Painter, rect: Rect, preview: &Preview, radius: u8) {
    let p = painter.with_clip_rect(rect);
    p.rect_filled(rect, radius, Color32::from_rgb(0x06, 0x0e, 0x18));
    let header = (rect.height() * 0.17).clamp(13.0, 19.0);
    p.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), header)),
        0,
        Color32::from_rgb(0x0b, 0x1d, 0x2d),
    );
    let path_font = FontId::new(if rect.height() > 52.0 { 9.0 } else { 7.0 }, egui::FontFamily::Monospace);
    let short = theme::elide(&p, &preview.path, &path_font, rect.width() - 12.0);
    p.text(
        egui::Pos2::new(rect.left() + 6.0, rect.top() + header * 0.5),
        Align2::LEFT_CENTER,
        short,
        path_font,
        Color32::from_rgb(0x7e, 0xa9, 0xc9),
    );

    let body = Rect::from_min_max(
        egui::Pos2::new(rect.left() + 6.0, rect.top() + header + 3.0),
        egui::Pos2::new(rect.right() - 6.0, rect.bottom() - 2.0),
    );
    if preview.lines.is_empty() {
        let msg = if preview.binary {
            "binary file".to_string()
        } else {
            format!("{} files", preview.count)
        };
        p.text(
            body.left_top(),
            Align2::LEFT_TOP,
            msg,
            FontId::new(8.0, egui::FontFamily::Monospace),
            DIM,
        );
    } else {
        let lh = (body.height() / preview.lines.len() as f32).clamp(6.0, 16.0);
        let font = FontId::new(lh * 0.92, egui::FontFamily::Monospace);
        let mut y = body.top();
        for line in &preview.lines {
            if y + lh > body.bottom() + 1.0 {
                break;
            }
            let short = theme::elide(&p, line, &font, body.width());
            p.text(
                egui::Pos2::new(body.left(), y + lh * 0.5),
                Align2::LEFT_CENTER,
                short,
                font.clone(),
                line_color(line),
            );
            y += lh;
        }
    }
    p.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, Color32::from_white_alpha(24)),
        StrokeKind::Inside,
    );
}

pub fn paint_empty_preview(ui: &Ui, rect: Rect, radius: u8) {
    ui.painter().rect_filled(rect, radius, Color32::from_rgb(0x08, 0x14, 0x1f));
    ui.painter().rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "No snapshot yet",
        FontId::new(12.0, egui::FontFamily::Proportional),
        MUTED,
    );
}

pub fn ext_of(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    match name.rfind('.') {
        Some(i) if i + 1 < name.len() => name[i + 1..].to_uppercase(),
        _ => "FILE".to_string(),
    }
}
