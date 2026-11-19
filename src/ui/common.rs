//! Shared painting pieces: cards, charts, brand marks and small controls.

use egui::{
    Align2, Color32, FontId, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind,
    Ui, Vec2,
};

use crate::preview::{ext_of, snapshot_highlight};
use crate::store::{self, Snapshot};
use crate::theme::*;

pub fn area(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect);
        add(ui);
    });
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(SURF)
        .stroke(Stroke::new(1.0, Color32::from_rgb(0x14, 0x30, 0x49)))
        .corner_radius(16)
        .inner_margin(egui::Margin::symmetric(16, 12))
}

pub fn card_at(painter: &egui::Painter, rect: Rect) {
    card(painter, rect, 16, SURF);
}

pub fn hgap(ui: &mut Ui, width: f32) {
    ui.add_space(width);
}

pub fn caption(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).font(bold(10.0)).color(DIM));
}

pub fn clock_logo(painter: &egui::Painter, rect: Rect) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.5 - 7.0;
    for (extra, a) in [(9.0, 16u8), (6.0, 26), (3.0, 44)] {
        painter.circle_stroke(center, r + extra, Stroke::new(2.2, alpha(ACCENT, a)));
    }
    painter.circle_stroke(center, r, Stroke::new(1.4, alpha(ACCENT, 70)));
    painter.circle_stroke(center, r - 1.5, Stroke::new(2.4, Color32::from_rgb(0xa9, 0xe6, 0xff)));
    let hand = Stroke::new(2.2, Color32::from_rgb(0xe6, 0xf7, 0xff));
    painter.line_segment([center, Pos2::new(center.x, center.y - r * 0.52)], hand);
    painter.line_segment(
        [center, Pos2::new(center.x + r * 0.36, center.y + r * 0.22)],
        hand,
    );
    painter.circle_filled(center, 2.1, Color32::WHITE);
}

fn sample_cubic(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let mt = 1.0 - t;
            Pos2::new(
                mt * mt * mt * p0.x + 3.0 * mt * mt * t * p1.x + 3.0 * mt * t * t * p2.x + t * t * t * p3.x,
                mt * mt * mt * p0.y + 3.0 * mt * mt * t * p1.y + 3.0 * mt * t * t * p2.y + t * t * t * p3.y,
            )
        })
        .collect()
}

pub fn hourglass(painter: &egui::Painter, rect: Rect) {
    let w = rect.width();
    let h = rect.height();
    let cx = rect.center().x;
    let top = rect.top() + 14.0;
    let bottom = rect.bottom() - 22.0;
    painter.circle_filled(
        Pos2::new(cx, rect.top() + h * 0.5),
        w * 0.46,
        alpha(ACCENT, 26),
    );

    let mut left: Vec<Pos2> = Vec::new();
    left.extend(sample_cubic(
        Pos2::new(cx - 25.0, top),
        Pos2::new(cx - 17.0, top + 26.0),
        Pos2::new(cx - 4.0, top + 30.0),
        Pos2::new(cx - 3.0, rect.top() + h * 0.5),
        16,
    ));
    left.extend(sample_cubic(
        Pos2::new(cx - 3.0, rect.top() + h * 0.5),
        Pos2::new(cx - 4.0, bottom - 30.0),
        Pos2::new(cx - 17.0, bottom - 26.0),
        Pos2::new(cx - 25.0, bottom),
        16,
    ));
    let mut outline = left.clone();
    outline.push(Pos2::new(cx + 25.0, bottom));
    let right_side = sample_cubic(
        Pos2::new(cx + 25.0, bottom),
        Pos2::new(cx + 17.0, bottom - 26.0),
        Pos2::new(cx + 4.0, bottom - 30.0),
        Pos2::new(cx + 3.0, rect.top() + h * 0.5),
        16,
    );
    outline.extend(right_side);
    let right_top = sample_cubic(
        Pos2::new(cx + 3.0, rect.top() + h * 0.5),
        Pos2::new(cx + 4.0, top + 30.0),
        Pos2::new(cx + 17.0, top + 26.0),
        Pos2::new(cx + 25.0, top),
        16,
    );
    outline.extend(right_top);
    painter.add(Shape::convex_polygon(
        outline.clone(),
        alpha(ACCENT, 22),
        Stroke::new(1.3, Color32::from_rgba_unmultiplied(140, 224, 255, 175)),
    ));

    painter.add(Shape::convex_polygon(
        vec![
            Pos2::new(cx - 15.0, top + 6.0),
            Pos2::new(cx + 15.0, top + 6.0),
            Pos2::new(cx + 7.0, top + 19.0),
            Pos2::new(cx - 7.0, top + 19.0),
        ],
        Color32::from_rgb(0x5e, 0xc8, 0xf5),
        Stroke::NONE,
    ));
    let mut mound = sample_cubic(
        Pos2::new(cx - 16.0, bottom - 5.0),
        Pos2::new(cx - 8.0, bottom - 18.0),
        Pos2::new(cx + 8.0, bottom - 18.0),
        Pos2::new(cx + 16.0, bottom - 5.0),
        18,
    );
    mound.push(Pos2::new(cx + 16.0, bottom - 5.0));
    painter.add(Shape::convex_polygon(
        mound,
        Color32::from_rgba_unmultiplied(94, 200, 245, 190),
        Stroke::NONE,
    ));
    painter.line_segment(
        [Pos2::new(cx, top + 22.0), Pos2::new(cx, bottom - 16.0)],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 240, 255, 190)),
    );
    for (x, y, r) in [
        (cx - 34.0, rect.top() + h * 0.32, 1.2),
        (cx + 33.0, rect.top() + h * 0.42, 1.0),
        (cx - 29.0, rect.top() + h * 0.68, 0.9),
        (cx + 30.0, rect.top() + h * 0.72, 1.3),
    ] {
        painter.circle_filled(
            Pos2::new(x, y),
            r,
            Color32::from_rgba_unmultiplied(150, 225, 255, 150),
        );
    }
    for y in [top, bottom] {
        let bar = Rect::from_min_size(Pos2::new(cx - 16.5, y - 3.0), Vec2::new(33.0, 3.4));
        painter.rect_filled(bar, 1.7, Color32::from_rgba_unmultiplied(74, 160, 205, 200));
    }
}

pub fn sparkline(painter: &egui::Painter, rect: Rect, data: &[f64], color: Color32) {
    if data.len() < 2 {
        return;
    }
    let lo = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = if (hi - lo).abs() < f64::EPSILON { 1.0 } else { hi - lo };
    let pts: Vec<Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, v)| {
            Pos2::new(
                rect.left() + 2.0 + i as f32 * (rect.width() - 4.0) / (data.len() - 1) as f32,
                rect.bottom() - 3.0 - ((v - lo) / span) as f32 * (rect.height() - 8.0),
            )
        })
        .collect();
    let mut fill = pts.clone();
    fill.push(Pos2::new(rect.right() - 2.0, rect.bottom()));
    fill.push(Pos2::new(rect.left() + 2.0, rect.bottom()));
    painter.add(Shape::convex_polygon(fill, alpha(color, 40), Stroke::NONE));
    painter.add(Shape::line(pts.clone(), Stroke::new(1.5, color)));
    if let Some(last) = pts.last() {
        painter.circle_filled(*last, 1.9, color);
    }
}

pub fn mini_bar(painter: &egui::Painter, rect: Rect, frac: f32, color: Color32) {
    painter.rect_filled(rect, 3, Color32::from_rgb(0x0f, 0x22, 0x33));
    let f = frac.clamp(0.0, 1.0);
    let w = (rect.width() * f).max(3.0);
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(w, rect.height())),
        3,
        color,
    );
}

pub fn stat_card(
    ui: &mut Ui,
    rect: Rect,
    value: &str,
    caption_text: &str,
    icon_glyph: &str,
    color: Color32,
    delta: &str,
    trend: i64,
    data: &[f64],
    hint: &str,
) {
    let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
    card(p, rect, 16, SURF);
    // soft radial glow in the corner
    let glow_center = Pos2::new(rect.right() - 12.0, rect.top() - 18.0);
    if let Some(mesh) = radial_mesh(glow_center, rect.width().max(110.0), alpha(color, 40)) {
        let clipped = p.with_clip_rect(rect);
        clipped.add(Shape::mesh(mesh));
    }
    let tile_rect = Rect::from_min_size(
        Pos2::new(rect.left() + 16.0, rect.top() + 14.0),
        Vec2::splat(40.0),
    );
    p.rect_filled(tile_rect, 11, darker(color, 4.3));
    p.rect_stroke(
        tile_rect,
        11,
        Stroke::new(1.0, darker(color, 3.2)),
        StrokeKind::Inside,
    );
    p.text(
        tile_rect.center(),
        Align2::CENTER_CENTER,
        icon_glyph,
        icon(17.0),
        color,
    );

    let narrow = rect.width() < 200.0;
    p.text(
        Pos2::new(rect.left() + 68.0, rect.top() + 12.0),
        Align2::LEFT_TOP,
        elide(p, value, &bold(if narrow { 20.0 } else { 24.0 }), rect.width() - 82.0),
        bold(if narrow { 20.0 } else { 24.0 }),
        TEXT,
    );
    p.text(
        Pos2::new(rect.left() + 68.0, rect.top() + 45.0),
        Align2::LEFT_TOP,
        elide(p, caption_text, &regular(12.0), rect.width() - 82.0),
        regular(12.0),
        MUTED,
    );
    if !hint.is_empty() {
        p.text(
            Pos2::new(rect.left() + 68.0, rect.top() + 63.0),
            Align2::LEFT_TOP,
            elide(p, hint, &regular(10.5), rect.width() - 82.0),
            regular(10.5),
            DIM,
        );
    }
    if !delta.is_empty() && rect.width() >= 210.0 {
        p.text(
            Pos2::new(rect.right() - 14.0, rect.top() + 14.0),
            Align2::RIGHT_TOP,
            delta,
            bold(11.0),
            if trend >= 0 { GREEN } else { RED },
        );
    }
    if rect.width() >= 200.0 {
        let spark = Rect::from_min_size(
            Pos2::new(rect.right() - 14.0 - 98.0, rect.bottom() - 14.0 - 34.0),
            Vec2::new(98.0, 34.0),
        );
        sparkline(p, spark, data, color);
    }
}

fn radial_mesh(center: Pos2, radius: f32, color: Color32) -> Option<egui::Mesh> {
    let mut mesh = egui::Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    mesh.vertices.push(egui::epaint::Vertex { pos: center, uv, color });
    let n = 40;
    for i in 0..=n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + Vec2::new(a.cos(), a.sin()) * radius,
            uv,
            color: Color32::TRANSPARENT,
        });
    }
    for i in 0..n {
        mesh.indices.push(0);
        mesh.indices.push(1 + i);
        mesh.indices.push(2 + i);
    }
    Some(mesh)
}

pub fn bar_chart(ui: &mut Ui, height: f32, data: &[(String, usize)]) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let __painter = ui.painter().clone();
    let p = &__painter;
    if data.is_empty() {
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No history captured yet",
            regular(12.0),
            MUTED,
        );
        return;
    }
    let pad_b = 26.0;
    let pad_t = 16.0;
    let top = rect.top() + pad_t;
    let bottom = rect.bottom() - pad_b;
    let peak = data.iter().map(|d| d.1).max().unwrap_or(1).max(1) as f32;
    let step = rect.width() / data.len().max(1) as f32;
    let bar_w = (step * 0.46).min(26.0);
    for i in 0..4 {
        let y = top + (bottom - top) * i as f32 / 3.0;
        p.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(1.0, Color32::from_rgb(0x10, 0x22, 0x34)),
        );
    }
    for (i, (name, value)) in data.iter().enumerate() {
        let cx = rect.left() + step * i as f32 + step / 2.0;
        let bh = (bottom - top) * (*value as f32 / peak);
        if *value > 0 {
            let bar = Rect::from_min_size(
                Pos2::new(cx - bar_w / 2.0, bottom - bh),
                Vec2::new(bar_w, bh),
            );
            gradient_rect(p, bar, BLUE, ACCENT_DARK);
            p.text(
                Pos2::new(bar.center().x, bar.top() - 2.0),
                Align2::CENTER_BOTTOM,
                value.to_string(),
                regular(10.0),
                MUTED,
            );
        } else {
            p.rect_filled(
                Rect::from_min_size(
                    Pos2::new(cx - bar_w / 2.0, bottom - 3.0),
                    Vec2::new(bar_w, 3.0),
                ),
                1.5,
                Color32::from_rgb(0x10, 0x22, 0x34),
            );
        }
        p.text(
            Pos2::new(cx, bottom + 12.0),
            Align2::CENTER_CENTER,
            name,
            regular(10.0),
            if i % 2 == 0 { MUTED } else { DIM },
        );
    }
    p.line_segment(
        [Pos2::new(rect.left(), bottom), Pos2::new(rect.right(), bottom)],
        Stroke::new(1.0, Color32::from_rgb(0x17, 0x36, 0x4f)),
    );
}

pub fn toggle(ui: &mut Ui, on: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(38.0, 20.0), Sense::click());
    let track = Rect::from_min_size(
        Pos2::new(rect.left(), rect.top() + 1.0),
        Vec2::new(rect.width(), rect.height() - 2.0),
    );
    let __painter = ui.painter().clone();
    let p = &__painter;
    p.rect_filled(
        track,
        track.height() / 2.0,
        if on { ACCENT } else { Color32::from_rgb(0x16, 0x34, 0x4d) },
    );
    let cx = if on {
        track.right() - track.height() / 2.0
    } else {
        track.left() + track.height() / 2.0
    };
    p.circle_filled(
        Pos2::new(cx, track.center().y),
        track.height() / 2.0 - 3.0,
        Color32::from_rgb(0xf8, 0xfc, 0xff),
    );
    response
}

pub fn paint_badge(p: &egui::Painter, rect: Rect, snap: &Snapshot) {
    p.rect_filled(rect, 8, Color32::from_rgb(0x0a, 0x1c, 0x2b));
    p.rect_stroke(rect, 8, Stroke::new(1.0, BORDER), StrokeKind::Inside);
    match snapshot_highlight(snap) {
        None => {
            p.text(rect.center(), Align2::CENTER_CENTER, "—", mono(9.0), DIM);
        }
        Some(path) => {
            let ext = ext_of(&path);
            p.text(
                Pos2::new(rect.center().x, rect.top() + 11.0),
                Align2::CENTER_CENTER,
                ext.chars().take(4).collect::<String>(),
                mono(11.0),
                ACCENT,
            );
            p.text(
                Pos2::new(rect.center().x, rect.top() + 25.0),
                Align2::CENTER_CENTER,
                format!("{} files", snap.files.len()),
                mono(8.0),
                MUTED,
            );
        }
    }
}

pub fn badge(ui: &mut Ui, snap: &Snapshot) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(46.0, 36.0), Sense::hover());
    paint_badge(ui.painter(), rect, snap);
}

/// Returns the width of the painted chip.
pub fn paint_chip(
    p: &egui::Painter,
    left_top: Pos2,
    text: &str,
    color: Color32,
    bg: Option<Color32>,
) -> f32 {
    let font = FontId::new(11.5, bold_family());
    let galley = p.layout_no_wrap(text.to_string(), font, color);
    let w = galley.size().x + 18.0;
    let h = galley.size().y + 7.0;
    let rect = Rect::from_min_size(left_top, Vec2::new(w, h));
    p.rect_filled(rect, 8, bg.unwrap_or(Color32::from_rgb(0x0f, 0x24, 0x37)));
    p.rect_stroke(
        rect,
        8,
        Stroke::new(1.0, darker(color, 3.4)),
        StrokeKind::Inside,
    );
    p.galley(
        rect.min + Vec2::new(9.0, (h - galley.size().y) * 0.5),
        galley,
        color,
    );
    w
}

pub fn empty_state(
    ui: &mut Ui,
    glyph: &str,
    title: &str,
    text: &str,
    button_text: &str,
) -> Option<Response> {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(84.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, 26, Color32::from_rgb(0x1c, 0x25, 0x32));
        ui.painter().rect_stroke(
            rect,
            26,
            Stroke::new(1.0, BORDER2),
            StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            icon(36.0),
            Color32::from_rgb(0xd0, 0xdc, 0xea),
        );
        ui.add_space(8.0);
        ui.label(RichText::new(title).font(bold(19.0)).color(TEXT));
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new(text)
                    .font(regular(13.5))
                    .color(MUTED),
            )
            .wrap()
            .halign(egui::Align::Center),
        );
        ui.add_space(8.0);
        let mut clicked = None;
        if !button_text.is_empty() {
            ui.horizontal(|ui| {
                let total = ui.available_width();
                let w = (button_text.len() as f32 * 8.0 + 40.0).min(300.0);
                ui.add_space((total - w) * 0.5);
                let r = button(ui, button_text, Btn::Primary, true);
                if r.clicked() {
                    clicked = Some(r);
                }
            });
        }
        clicked
    })
    .inner
}

pub fn fact_row(ui: &mut Ui, name: &str, value: &str, label_width: f32, value_width: f32) {
    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(label_width, 16.0),
            egui::Label::new(RichText::new(name.to_uppercase()).font(bold(10.0)).color(DIM)),
        );
        let short = {
            let __painter = ui.painter().clone();
    let p = &__painter;
            elide(p, value, &regular(12.0), value_width)
        };
        let label = ui.add(
            egui::Label::new(RichText::new(short).font(regular(12.0)).color(TEXT))
                .truncate(),
        );
        label.on_hover_text(value);
    });
}

pub fn nav_row(ui: &mut Ui, glyph: &str, label: &str, active: bool) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 38.0), Sense::click());
    let __painter = ui.painter().clone();
    let p = &__painter;
    let hovered = response.hovered();
    if active {
        gradient_rect(
            p,
            rect,
            Color32::from_rgb(0x17, 0x51, 0x7f),
            Color32::from_rgb(0x0d, 0x2a, 0x46),
        );
        p.rect_stroke(
            rect,
            10,
            Stroke::new(1.0, Color32::from_rgb(0x1f, 0x5f, 0x92)),
            StrokeKind::Inside,
        );
    } else if hovered {
        p.rect_filled(rect, 10, Color32::from_rgb(0x0e, 0x21, 0x35));
    }
    let fg = if active {
        Color32::from_rgb(0xc4, 0xe7, 0xff)
    } else {
        MUTED
    };
    p.text(
        Pos2::new(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        icon(14.0),
        fg,
    );
    p.text(
        Pos2::new(rect.left() + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        bold(14.0),
        fg,
    );
    response
}

pub fn status_pill(ui: &mut Ui, kind: &str) {
    let color = status_color(kind);
    chip(ui, status_label(kind), color, Some(darker(color, 3.2)));
}

pub fn btn_label_with_icon(ui: &mut Ui, glyph: &str, text: &str, kind: Btn, enabled: bool) -> Response {
    button(ui, &format!("{}   {}", glyph, text), kind, enabled)
}

pub fn snapshot_day_line(snap: &Snapshot) -> String {
    format!("{}, {}", store::short_day(snap.ts), store::clock_label(snap.ts))
}
