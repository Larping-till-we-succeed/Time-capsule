//! Visual language for TimeCapsule: palette, fonts and painting helpers.

use std::path::PathBuf;
use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Mesh, Painter,
    Pos2, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
};

pub const BG: Color32 = Color32::from_rgb(0x1b, 0x23, 0x35);
pub const SIDEBAR: Color32 = Color32::from_rgb(0x19, 0x20, 0x30);
pub const SURF: Color32 = Color32::from_rgb(0x1f, 0x29, 0x3d);
pub const SURF2: Color32 = Color32::from_rgb(0x25, 0x30, 0x48);
pub const BORDER: Color32 = Color32::from_rgb(0x28, 0x33, 0x4c);
pub const BORDER2: Color32 = Color32::from_rgb(0x34, 0x40, 0x5e);
pub const TEXT: Color32 = Color32::from_rgb(0xee, 0xf1, 0xf8);
pub const MUTED: Color32 = Color32::from_rgb(0x9a, 0xa4, 0xbc);
pub const DIM: Color32 = Color32::from_rgb(0x6c, 0x78, 0x94);
pub const ACCENT: Color32 = Color32::from_rgb(0x5b, 0x8c, 0xff);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x3a, 0x62, 0xd6);
pub const GREEN: Color32 = Color32::from_rgb(0x2b, 0xc4, 0x8a);
pub const RED: Color32 = Color32::from_rgb(0xf2, 0x6b, 0x6b);
pub const BLUE: Color32 = Color32::from_rgb(0x38, 0xa8, 0xe0);
pub const VIOLET: Color32 = Color32::from_rgb(0x7a, 0x6b, 0xf0);
pub const AMBER: Color32 = Color32::from_rgb(0xf0, 0xa9, 0x3b);

/// The three gradient tiles on the dashboard.
pub const TILE_GREEN_A: Color32 = Color32::from_rgb(0x25, 0xc4, 0x92);
pub const TILE_GREEN_B: Color32 = Color32::from_rgb(0x14, 0x9e, 0x74);
pub const TILE_BLUE_A: Color32 = Color32::from_rgb(0x35, 0xab, 0xe2);
pub const TILE_BLUE_B: Color32 = Color32::from_rgb(0x1f, 0x8a, 0xc8);
pub const TILE_VIOLET_A: Color32 = Color32::from_rgb(0x6a, 0x6c, 0xc8);
pub const TILE_VIOLET_B: Color32 = Color32::from_rgb(0x4c, 0x4f, 0x9c);

pub fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}
pub fn icon_family() -> FontFamily {
    FontFamily::Name("icon".into())
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, bold_family())
}
pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}
pub fn icon(size: f32) -> FontId {
    FontId::new(size, icon_family())
}
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn alpha(color: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

/// Python's QColor.darker(percent) equivalent for the chip borders.
pub fn darker(color: Color32, factor: f32) -> Color32 {
    let f = 1.0 / factor;
    Color32::from_rgb(
        (color.r() as f32 * f) as u8,
        (color.g() as f32 * f) as u8,
        (color.b() as f32 * f) as u8,
    )
}

fn fc_match(query: &str) -> Option<PathBuf> {
    let out = std::process::Command::new("fc-match")
        .args(["-f", "%{file}", query])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn read_font(path: Option<PathBuf>) -> Option<Vec<u8>> {
    let path = path?;
    std::fs::read(path).ok()
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    let mut add = |name: &str, bytes: Option<Vec<u8>>| -> bool {
        if let Some(data) = bytes {
            defs.font_data.insert(name.to_string(), Arc::new(FontData::from_owned(data)));
            true
        } else {
            false
        }
    };

    let mut have_ui = false;
    let mut have_bold = false;
    let mut have_mono = false;
    let mut icon_names: Vec<String> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        have_ui = add("ui", read_font(fc_match("Inter")));
        have_bold = add("ui-bold", read_font(fc_match("Inter:weight=bold")));
        have_mono = add("mono", read_font(fc_match("JetBrains Mono")));
        add("mono-bold", read_font(fc_match("JetBrains Mono:weight=bold")));
        if add("symbols", read_font(fc_match("Noto Sans Symbols 2"))) {
            icon_names.push("symbols".into());
        }
        if add("nerd", read_font(fc_match("Iosevka Nerd Font"))) {
            icon_names.insert(0, "nerd".into());
        }
        if add("cjk", read_font(fc_match("Noto Sans CJK SC"))) {
            icon_names.push("cjk".into());
        }
    }
    #[cfg(target_os = "windows")]
    {
        let win = PathBuf::from(r"C:\Windows\Fonts");
        have_ui = add("ui", read_font(Some(win.join("segoeui.ttf"))));
        have_bold = add("ui-bold", read_font(Some(win.join("segoeuib.ttf"))));
        have_mono = add("mono", read_font(Some(win.join("consola.ttf"))));
        add("mono-bold", read_font(Some(win.join("consolab.ttf"))));
        if add("symbols", read_font(Some(win.join("seguisym.ttf")))) {
            icon_names.push("symbols".into());
        }
    }

    let fallbacks: Vec<String> = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    let mut prop: Vec<String> = Vec::new();
    if have_ui {
        prop.push("ui".into());
    }
    let bold_list = if have_bold {
        vec!["ui-bold".to_string()]
    } else {
        prop.clone()
    };

    prop.extend(icon_names.iter().cloned());
    prop.extend(fallbacks.iter().cloned());
    defs.families.insert(FontFamily::Proportional, prop.clone());

    let mut bold_chain = bold_list;
    bold_chain.extend(icon_names.iter().cloned());
    bold_chain.extend(fallbacks.iter().cloned());
    defs.families.insert(bold_family(), bold_chain);

    let mut icons = icon_names.clone();
    icons.extend(fallbacks.iter().cloned());
    defs.families.insert(icon_family(), icons);

    let mut mono_chain: Vec<String> = Vec::new();
    if have_mono {
        mono_chain.push("mono".into());
        if defs.font_data.contains_key("mono-bold") {
            mono_chain.push("mono-bold".into());
        }
    }
    let mono_fallbacks = defs
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    mono_chain.extend(icon_names.iter().cloned());
    mono_chain.extend(mono_fallbacks);
    defs.families.insert(FontFamily::Monospace, mono_chain);

    ctx.set_fonts(defs);
}

pub fn install_style(ctx: &egui::Context) {
    use egui::{TextStyle, Visuals};
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, bold(22.0)),
        (TextStyle::Body, regular(14.0)),
        (TextStyle::Monospace, mono(12.5)),
        (TextStyle::Button, regular(14.0)),
        (TextStyle::Small, regular(11.5)),
    ]
    .into();

    let mut v = Visuals::dark();
    v.override_text_color = Some(TEXT);
    v.panel_fill = BG;
    v.window_fill = SURF;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.window_corner_radius = CornerRadius::same(14);
    v.extreme_bg_color = Color32::from_rgb(0x17, 0x1e, 0x2e);
    v.faint_bg_color = SURF2;
    v.selection.bg_fill = Color32::from_rgb(0x33, 0x4c, 0x86);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.widgets.noninteractive.bg_fill = SURF;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.noninteractive.corner_radius = CornerRadius::same(10);
    v.widgets.inactive.bg_fill = SURF2;
    v.widgets.inactive.weak_bg_fill = SURF2;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.corner_radius = CornerRadius::same(10);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x2c, 0x38, 0x54);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x2c, 0x38, 0x54);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER2);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.hovered.corner_radius = CornerRadius::same(10);
    v.widgets.active.bg_fill = Color32::from_rgb(0x33, 0x41, 0x60);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(0x33, 0x41, 0x60);
    v.widgets.active.bg_stroke = Stroke::new(1.0, BORDER2);
    v.widgets.active.corner_radius = CornerRadius::same(10);
    v.widgets.open.bg_fill = SURF2;
    v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    style.visuals = v;
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.interact_size.y = 28.0;
    ctx.set_style(style);
}

pub fn gradient_rect(painter: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    mesh.vertices.push(egui::epaint::Vertex { pos: rect.left_top(), uv, color: top });
    mesh.vertices.push(egui::epaint::Vertex { pos: rect.right_top(), uv, color: top });
    mesh.vertices.push(egui::epaint::Vertex { pos: rect.right_bottom(), uv, color: bottom });
    mesh.vertices.push(egui::epaint::Vertex { pos: rect.left_bottom(), uv, color: bottom });
    mesh.indices = vec![0, 1, 2, 0, 2, 3];
    painter.add(Shape::mesh(mesh));
}

pub fn hline(ui: &mut Ui, color: Color32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, color);
}

pub fn elide(painter: &Painter, text: &str, font: &FontId, max_width: f32) -> String {
    let full = painter.layout_no_wrap(text.to_string(), font.clone(), TEXT);
    if full.size().x <= max_width {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let candidate: String = chars[..mid].iter().collect::<String>() + "…";
        let w = painter
            .layout_no_wrap(candidate, font.clone(), TEXT)
            .size()
            .x;
        if w <= max_width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars[..lo].iter().collect::<String>() + "…"
}

pub fn text_at(
    painter: &Painter,
    pos: Pos2,
    text: &str,
    font: FontId,
    color: Color32,
    align: Align2,
) {
    painter.text(pos, align, text, font, color);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Btn {
    Primary,
    Ghost,
    Danger,
    Link,
    Nav,
    Quick,
}

/// A hand-painted pill button so the dark theme stays exact.
pub fn button(ui: &mut Ui, label: &str, kind: Btn, enabled: bool) -> Response {
    let font = match kind {
        Btn::Link => FontId::new(12.0, bold_family()),
        Btn::Nav => FontId::new(14.0, bold_family()),
        Btn::Quick => FontId::new(12.5, bold_family()),
        _ => FontId::new(14.0, bold_family()),
    };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), TEXT);
    let (pad_x, pad_y) = match kind {
        Btn::Link => (2.0, 2.0),
        Btn::Nav => (12.0, 9.0),
        Btn::Quick => (10.0, 8.0),
        _ => (16.0, 9.0),
    };
    let desired = Vec2::new(galley.size().x + pad_x * 2.0, galley.size().y.max(18.0) + pad_y * 2.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    let hovered = enabled && response.hovered();
    let pressed = enabled && response.is_pointer_button_down_on();
    let painter = ui.painter();

    let (fill, stroke, fg) = match kind {
        Btn::Primary => {
            let fill = if !enabled {
                Color32::from_rgb(0x2a, 0x33, 0x4a)
            } else if pressed {
                Color32::from_rgb(0x34, 0x5c, 0xcf)
            } else if hovered {
                Color32::from_rgb(0x6d, 0x99, 0xff)
            } else {
                ACCENT
            };
            (fill, Color32::TRANSPARENT, Color32::from_rgb(0xf7, 0xf9, 0xff))
        }
        Btn::Ghost => {
            let fill = if !enabled {
                Color32::from_rgb(0x20, 0x28, 0x3b)
            } else if pressed {
                SURF2
            } else if hovered {
                Color32::from_rgb(0x2c, 0x38, 0x54)
            } else {
                Color32::from_rgb(0x23, 0x2d, 0x43)
            };
            let stroke = if !enabled {
                BORDER
            } else if hovered {
                BORDER2
            } else {
                BORDER
            };
            (fill, stroke, if enabled { TEXT } else { DIM })
        }
        Btn::Danger => {
            let fill = if hovered {
                Color32::from_rgb(0x48, 0x24, 0x2c)
            } else {
                Color32::from_rgb(0x38, 0x1d, 0x24)
            };
            (fill, Color32::from_rgb(0x6b, 0x36, 0x40), Color32::from_rgb(0xff, 0xa9, 0xa9))
        }
        Btn::Link => (Color32::TRANSPARENT, Color32::TRANSPARENT, if hovered { Color32::from_rgb(0xa9, 0xc2, 0xff) } else { ACCENT }),
        Btn::Nav => {
            if pressed || hovered {
                (SURF2, Color32::TRANSPARENT, TEXT)
            } else {
                (Color32::TRANSPARENT, Color32::TRANSPARENT, MUTED)
            }
        }
        Btn::Quick => {
            if hovered {
                (SURF2, BORDER, TEXT)
            } else {
                (Color32::TRANSPARENT, Color32::TRANSPARENT, TEXT)
            }
        }
    };

    if fill != Color32::TRANSPARENT || stroke != Color32::TRANSPARENT {
        painter.rect_filled(rect, 11, fill);
        if stroke != Color32::TRANSPARENT {
            painter.rect_stroke(rect, 11, Stroke::new(1.0, stroke), StrokeKind::Inside);
        }
    }
    let text_pos = rect.center() - galley.size() * 0.5;
    painter.galley(text_pos, galley, fg);

    if kind == Btn::Link && hovered {
        let underline = Rect::from_min_max(
            Pos2::new(text_pos.x, text_pos.y + 16.0),
            Pos2::new(text_pos.x + 40.0, text_pos.y + 17.0),
        );
        painter.rect_filled(underline, 0, alpha(ACCENT, 120));
    }
    response
}

/// A small chip used for counts, branches and statuses.
pub fn chip(ui: &mut Ui, text: &str, color: Color32, bg: Option<Color32>) {
    let font = FontId::new(11.5, bold_family());
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, color);
    let size = Vec2::new(galley.size().x + 18.0, galley.size().y + 7.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let bg = bg.unwrap_or(Color32::from_rgb(0x0f, 0x24, 0x37));
    ui.painter().rect_filled(rect, 8, bg);
    ui.painter().rect_stroke(
        rect,
        8,
        Stroke::new(1.0, darker(color, 3.4)),
        StrokeKind::Inside,
    );
    let pos = rect.min + Vec2::new(9.0, (rect.height() - galley.size().y) * 0.5);
    ui.painter().galley(pos, galley, color);
}

pub fn card(painter: &Painter, rect: Rect, radius: u8, fill: Color32) {
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
}

pub fn status_color(kind: &str) -> Color32 {
    match kind {
        "added" => GREEN,
        "modified" => ACCENT,
        "deleted" => RED,
        _ => DIM,
    }
}

pub fn status_label(kind: &str) -> &'static str {
    match kind {
        "added" => "Added",
        "modified" => "Modified",
        "deleted" => "Deleted",
        _ => "Unchanged",
    }
}

pub fn status_pill(ui: &mut Ui, kind: &str) {
    let color = status_color(kind);
    chip(ui, status_label(kind), color, Some(darker(color, 3.2)));
}

pub fn tile(ui: &mut Ui, glyph: &str, color: Color32, size: f32, radius: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().rect_filled(rect, radius, darker(color, 4.3));
    ui.painter().rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, darker(color, 3.2)),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        icon(size * 0.44),
        color,
    );
}

pub fn thin_scroll(ui: &mut Ui, id: &str, max_height: Option<f32>, add: impl FnOnce(&mut Ui)) {
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, false]);
    if let Some(h) = max_height {
        scroll = scroll.max_height(h);
    }
    scroll.show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui);
    });
}

pub fn rich(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text.into()).size(size).color(color)
}

pub fn bold_rich(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text.into()).font(bold(size)).color(color)
}
