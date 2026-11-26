//! Dashboard: the mission-control overview of everything TimeCapsule holds.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::preview;
use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

fn kernel() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

fn uptime() -> String {
    if let Ok(text) = std::fs::read_to_string("/proc/uptime") {
        if let Some(first) = text.split_whitespace().next() {
            if let Ok(secs) = first.parse::<f64>() {
                return format!("{}h {}m", (secs / 3600.0) as u64, (secs % 3600.0 / 60.0) as u64);
            }
        }
    }
    "—".into()
}

fn os_name() -> String {
    let mut s = std::env::consts::OS.to_string();
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

fn card_header(p: &egui::Painter, rect: Rect, glyph: &str, color: Color32, title: &str) {
    p.text(
        Pos2::new(rect.left() + 16.0, rect.top() + 13.0),
        Align2::LEFT_TOP,
        glyph,
        icon(14.0),
        color,
    );
    p.text(
        Pos2::new(rect.left() + 38.0, rect.top() + 11.0),
        Align2::LEFT_TOP,
        title,
        bold(14.5),
        TEXT,
    );
}


fn smooth_path(points: &[Pos2], samples: usize) -> Vec<Pos2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut out: Vec<Pos2> = Vec::with_capacity(points.len() * samples + 1);
    for i in 0..points.len() - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() { points[i + 2] } else { p2 };
        for s in 0..samples {
            let t = s as f32 / samples as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            out.push(Pos2::new(
                0.5 * ((2.0 * p1.x)
                    + (-p0.x + p2.x) * t
                    + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
                    + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3),
                0.5 * ((2.0 * p1.y)
                    + (-p0.y + p2.y) * t
                    + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
                    + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3),
            ));
        }
    }
    out.push(*points.last().unwrap());
    out
}

/// One of the three coloured tiles at the top of the dashboard.
fn paint_tile(
    ui: &mut egui::Ui,
    rect: Rect,
    label: &str,
    value: &str,
    delta: &str,
    from: Color32,
    to: Color32,
) -> bool {
    let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
    gradient_rect(p, rect, from, to);
    p.rect_stroke(
        rect,
        14,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 26)),
        StrokeKind::Inside,
    );
    p.text(
        Pos2::new(rect.left() + 18.0, rect.top() + 16.0),
        Align2::LEFT_TOP,
        label,
        regular(12.5),
        Color32::from_rgba_unmultiplied(255, 255, 255, 220),
    );
    p.text(
        Pos2::new(rect.left() + 18.0, rect.top() + 40.0),
        Align2::LEFT_TOP,
        elide(p, value, &bold(28.0), rect.width() - 36.0),
        bold(28.0),
        Color32::WHITE,
    );
    let galley = p.layout_no_wrap(delta.to_string(), bold(11.0), Color32::WHITE);
    let chip = Rect::from_min_size(
        Pos2::new(rect.left() + 18.0, rect.bottom() - 35.0),
        Vec2::new(galley.size().x + 20.0, 24.0),
    );
    p.rect_filled(
        chip,
        8,
        Color32::from_rgba_unmultiplied(255, 255, 255, 34),
    );
    p.galley(
        chip.min + Vec2::new(10.0, (24.0 - galley.size().y) * 0.5),
        galley,
        Color32::WHITE,
    );

    let detail = Rect::from_min_size(
        Pos2::new(rect.right() - 18.0 - 68.0, rect.bottom() - 37.0),
        Vec2::new(68.0, 26.0),
    );
    let resp = ui.interact(detail, ui.id().with(("tile_detail", label)), Sense::click());
    let bg = if resp.hovered() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 70)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 34)
    };
    p.rect_filled(detail, 8, bg);
    p.text(
        detail.center(),
        Align2::CENTER_CENTER,
        "Detail",
        bold(11.0),
        Color32::WHITE,
    );
    resp.clicked()
}

impl App {
    pub fn open_timeline_at(&mut self, sid: Option<String>) {
        if self.pid.is_none() {
            return;
        }
        self.page = crate::Page::Timeline;
        self.rebuild_timeline(sid);
    }

    pub fn ui_dashboard(&mut self, ui: &mut egui::Ui) {
        let wide = ui.available_width() >= 940.0;
        let gap = 14.0;
        let tile_h = 118.0;
        let chart_h = 268.0;
        let strip_h = 236.0;
        let bottom_h = 292.0;
        let cols = if wide { 3usize } else { 1usize };
        let tile_rows = (3 + cols - 1) / cols;
        let content_h = if wide {
            tile_h + gap + chart_h + gap + strip_h + gap + bottom_h + 8.0
        } else {
            tile_rows as f32 * (tile_h + gap)
                + chart_h
                + gap
                + 300.0
                + gap
                + strip_h
                + gap
                + 236.0
                + gap
                + bottom_h
                + gap
                + bottom_h
                + 8.0
        };
        egui::ScrollArea::vertical()
            .id_salt("dashboard")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let width = ui.available_width();
                let (content, _) =
                    ui.allocate_exact_size(Vec2::new(width, content_h), Sense::hover());
                let tile_w = (width - gap * (cols - 1) as f32) / cols as f32;

                let snapshot_delta = format!(
                    "{} {}%",
                    if self.dash.pct >= 0 { "↑" } else { "↓" },
                    self.dash.pct.abs()
                );
                let project_delta = if self.dash.new_projects > 0 {
                    format!("↑ {}", self.dash.new_projects)
                } else {
                    "stable".to_string()
                };
                let space_delta = format!(
                    "{} {}%",
                    if self.dash.space_delta >= 0 { "↑" } else { "↓" },
                    self.dash.space_delta.abs()
                );
                let tiles: [(&str, String, String, Color32, Color32); 3] = [
                    (
                        "Total snapshots",
                        self.dash.total_snaps.to_string(),
                        snapshot_delta,
                        TILE_GREEN_A,
                        TILE_GREEN_B,
                    ),
                    (
                        "Tracked projects",
                        self.dash.project_count.to_string(),
                        project_delta,
                        TILE_BLUE_A,
                        TILE_BLUE_B,
                    ),
                    (
                        "Used space",
                        store::fmt_bytes(self.dash.used),
                        space_delta,
                        TILE_VIOLET_A,
                        TILE_VIOLET_B,
                    ),
                ];

                let mut open_timeline = false;
                let mut go_settings = false;
                for (i, (label, value, delta, from, to)) in tiles.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            content.left() + col as f32 * (tile_w + gap),
                            content.top() + row as f32 * (tile_h + gap),
                        ),
                        Vec2::new(tile_w, tile_h),
                    );
                    if paint_tile(ui, rect, label, value, delta, *from, *to) {
                        match i {
                            0 => open_timeline = true,
                            2 => go_settings = true,
                            _ => {}
                        }
                    }
                }

                let mut y = content.top() + tile_rows as f32 * (tile_h + gap);
                if wide {
                    let recent_w = 326.0;
                    let chart_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width - recent_w - gap, chart_h),
                    );
                    self.paint_chart_card(ui, chart_rect);
                    let recent_rect = Rect::from_min_size(
                        Pos2::new(content.right() - recent_w, y),
                        Vec2::new(recent_w, chart_h),
                    );
                    self.paint_recent(ui, recent_rect);
                } else {
                    let chart_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, chart_h),
                    );
                    self.paint_chart_card(ui, chart_rect);
                    y += chart_h + gap;
                    let recent_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, 300.0),
                    );
                    self.paint_recent(ui, recent_rect);
                }
                y += chart_h + gap;

                let strip_rect = Rect::from_min_size(
                    Pos2::new(content.left(), y),
                    Vec2::new(width, strip_h),
                );
                self.paint_timeline_card(ui, strip_rect);
                y += strip_h + gap;

                if wide {
                    let what_w = 300.0;
                    let quick_w = 330.0;
                    let current_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width - what_w - quick_w - gap * 2.0, bottom_h),
                    );
                    self.paint_current_state(ui, current_rect);
                    let what_rect = Rect::from_min_size(
                        Pos2::new(current_rect.right() + gap, y),
                        Vec2::new(what_w, bottom_h),
                    );
                    self.paint_what_changed(ui, what_rect);
                    let quick_rect = Rect::from_min_size(
                        Pos2::new(what_rect.right() + gap, y),
                        Vec2::new(quick_w, bottom_h),
                    );
                    self.paint_quick_actions(ui, quick_rect);
                } else {
                    let current_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, 236.0),
                    );
                    self.paint_current_state(ui, current_rect);
                    y += 236.0 + gap;
                    let what_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, bottom_h),
                    );
                    self.paint_what_changed(ui, what_rect);
                    y += bottom_h + gap;
                    let quick_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, bottom_h),
                    );
                    self.paint_quick_actions(ui, quick_rect);
                }
                let _ = y;

                if open_timeline {
                    self.open_timeline_at(None);
                }
                if go_settings {
                    self.select_page(crate::Page::Settings);
                }
            });
    }

    fn paint_chart_card(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, rect, 14, SURF);

        p.text(
            Pos2::new(rect.left() + 18.0, rect.top() + 15.0),
            Align2::LEFT_TOP,
            "Snapshots over time",
            bold(15.0),
            TEXT,
        );
        let (status, color) = if self.pid.is_none() {
            ("No project".to_string(), DIM)
        } else if self.dash.dirty {
            ("Changes pending".to_string(), AMBER)
        } else {
            ("All good".to_string(), GREEN)
        };
        let galley = p.layout_no_wrap(status.clone(), bold(11.0), color);
        let pill = Rect::from_min_size(
            Pos2::new(
                rect.right() - 18.0 - (galley.size().x + 20.0),
                rect.top() + 11.0,
            ),
            Vec2::new(galley.size().x + 20.0, 23.0),
        );
        p.rect_filled(pill, 8, darker(color, 4.4));
        p.galley(
            pill.min + Vec2::new(10.0, (23.0 - galley.size().y) * 0.5),
            galley,
            color,
        );

        let caption = if self.dash.total_snaps > 0 {
            format!("{} snapshots across all projects", self.dash.total_snaps)
        } else {
            "Capture a snapshot to start the chart".to_string()
        };
        p.text(
            Pos2::new(rect.left() + 18.0, rect.top() + 36.0),
            Align2::LEFT_TOP,
            caption,
            regular(11.0),
            MUTED,
        );
        let plot = Rect::from_min_max(
            Pos2::new(rect.left() + 46.0, rect.top() + 64.0),
            Pos2::new(rect.right() - 18.0, rect.bottom() - 56.0),
        );
        let series = self.dash.series.clone();
        let n = series.len().max(2);
        let hi = series.iter().cloned().fold(0.0_f64, f64::max).max(1.0);
        for i in 0..=3 {
            let y = plot.top() + plot.height() * i as f32 / 3.0;
            p.line_segment(
                [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
                Stroke::new(1.0, alpha(BORDER, 170)),
            );
            p.text(
                Pos2::new(plot.left() - 9.0, y),
                Align2::RIGHT_CENTER,
                format!("{}", (hi * (1.0 - i as f64 / 3.0)).round() as i64),
                regular(9.5),
                DIM,
            );
        }
        let pts: Vec<Pos2> = series
            .iter()
            .enumerate()
            .map(|(i, v)| {
                Pos2::new(
                    plot.left() + i as f32 * plot.width() / (n - 1) as f32,
                    plot.bottom() - (v / hi) as f32 * (plot.height() - 8.0),
                )
            })
            .collect();
        if pts.len() >= 2 {
            let curve: Vec<Pos2> = smooth_path(&pts, 14)
                .into_iter()
                .map(|pt| Pos2::new(pt.x, pt.y.clamp(plot.top(), plot.bottom())))
                .collect();
            // The area under the line is a strip of quads, which stays clean even
            // when the curve is not convex.
            let mut mesh = egui::Mesh::default();
            let uv = egui::epaint::WHITE_UV;
            for pair in curve.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let base = plot.bottom();
                let i = mesh.vertices.len() as u32;
                for (pos, color) in [
                    (a, alpha(AMBER, 78)),
                    (b, alpha(AMBER, 78)),
                    (Pos2::new(b.x, base), alpha(AMBER, 6)),
                    (Pos2::new(a.x, base), alpha(AMBER, 6)),
                ] {
                    mesh.vertices.push(egui::epaint::Vertex { pos, uv, color });
                }
                mesh.indices.extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
            }
            p.add(egui::Shape::mesh(mesh));
            p.add(egui::Shape::line(curve, Stroke::new(2.2, AMBER)));
            if let Some(last) = pts.last() {
                p.circle_filled(*last, 4.2, SURF);
                p.circle_stroke(*last, 4.2, Stroke::new(2.0, AMBER));
            }
        }
        let today = store::now();
        for i in 0..n {
            let ts = today - (n - 1 - i) as f64 * 86400.0;
            let x = plot.left() + i as f32 * plot.width() / (n - 1) as f32;
            p.text(
                Pos2::new(x, plot.bottom() + 10.0),
                Align2::CENTER_TOP,
                store::short_day(ts),
                regular(9.5),
                DIM,
            );
        }

        let detail = Rect::from_min_size(
            Pos2::new(rect.right() - 18.0 - 76.0, rect.bottom() - 31.0),
            Vec2::new(76.0, 24.0),
        );
        let resp = ui.interact(detail, ui.id().with("chart_detail"), Sense::click());
        p.rect_filled(
            detail,
            8,
            if resp.hovered() {
                SURF2
            } else {
                alpha(SURF2, 180)
            },
        );
        p.text(detail.center(), Align2::CENTER_CENTER, "Detail", bold(11.0), TEXT);
        if resp.clicked() {
            self.open_timeline_at(None);
        }
    }

    fn paint_timeline_card(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        card_header(p, rect, "◷", ACCENT, "Timeline");
        let link_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 16.0 - 76.0, rect.top() + 8.0),
            Vec2::new(76.0, 22.0),
        );
        let link = ui.interact(link_rect, ui.id().with("viewall"), Sense::click());
        p.text(
            Pos2::new(link_rect.right(), link_rect.center().y),
            Align2::RIGHT_CENTER,
            "View all →",
            bold(12.0),
            if link.hovered() { Color32::from_rgb(0x8f, 0xd6, 0xff) } else { ACCENT },
        );
        let mut open: Option<String> = None;
        if link.clicked() {
            open = Some(String::new());
        }

        let strip = Rect::from_min_max(
            Pos2::new(rect.left() + 16.0, rect.top() + 38.0),
            Pos2::new(rect.right() - 16.0, rect.bottom() - 8.0),
        );
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        if self.dash.strip.is_empty() {
            p.text(
                strip.center(),
                Align2::CENTER_CENTER,
                "Capture a snapshot to start the timeline.",
                regular(13.0),
                MUTED,
            );
        } else {
            let n = self.dash.strip.len();
            let gap = 12.0;
            let cw = (strip.width() - gap * (n - 1) as f32 - 4.0) / n as f32;
            let ch = strip.height() - 38.0;
            let selected = self.dash.chain.last().map(|s| s.id.clone());
            let line_y = strip.bottom() - 24.0;
            let first_x = strip.left() + 2.0 + cw / 2.0;
            let last_x = strip.left() + 2.0 + (n - 1) as f32 * (cw + gap) + cw / 2.0;
            p.line_segment(
                [Pos2::new(first_x, line_y), Pos2::new(last_x, line_y)],
                Stroke::new(2.0, Color32::from_rgb(0x1c, 0x46, 0x66)),
            );
            for (i, (snap, preview)) in self.dash.strip.clone().iter().enumerate() {
                let card_rect = Rect::from_min_size(
                    Pos2::new(strip.left() + 2.0 + i as f32 * (cw + gap), strip.top()),
                    Vec2::new(cw, ch),
                );
                let response = ui.interact(
                    card_rect,
                    ui.id().with(("strip", i)),
                    Sense::click(),
                );
                let chosen = selected.as_deref() == Some(snap.id.as_str());
                let fill = if chosen {
                    SURF2
                } else if response.hovered() {
                    SURF2
                } else {
                    SURF
                };
                p.rect_filled(card_rect, 12, fill);
                p.rect_stroke(
                    card_rect,
                    12,
                    Stroke::new(
                        if chosen { 1.4 } else { 1.0 },
                        if chosen { ACCENT } else { BORDER },
                    ),
                    StrokeKind::Inside,
                );
                p.text(
                    Pos2::new(card_rect.left() + 11.0, card_rect.top() + 9.0),
                    Align2::LEFT_TOP,
                    store::short_day(snap.ts),
                    bold(12.0),
                    TEXT,
                );
                p.text(
                    Pos2::new(card_rect.left() + 11.0, card_rect.top() + 26.0),
                    Align2::LEFT_TOP,
                    store::clock_label(snap.ts),
                    regular(11.0),
                    MUTED,
                );
                let thumb = Rect::from_min_max(
                    Pos2::new(card_rect.left() + 8.0, card_rect.top() + 42.0),
                    Pos2::new(card_rect.right() - 8.0, card_rect.bottom() - 8.0),
                );
                if thumb.height() > 12.0 {
                    match preview {
                        Some(pv) => preview::paint_preview(p, thumb, pv, 8),
                        None => preview::paint_empty_preview(ui, thumb, 8),
                    }
                }
                let dot = Pos2::new(card_rect.center().x, line_y);
                if chosen {
                    p.circle_filled(dot, 4.6 * 2.4, alpha(ACCENT, 46));
                    p.circle_filled(dot, 4.6, ACCENT);
                } else {
                    p.circle_filled(dot, 4.0, Color32::from_rgb(0x6f, 0x93, 0xb8));
                }
                if response.clicked() {
                    open = Some(snap.id.clone());
                }
            }
        }
        if let Some(sid) = open {
            if sid.is_empty() {
                self.open_timeline_at(None);
            } else {
                self.open_timeline_at(Some(sid));
            }
        }
    }

    fn paint_current_state(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        card_header(p, rect, "▣", ACCENT, "Current State");
        let preview_rect = Rect::from_min_max(
            Pos2::new(rect.left() + 16.0, rect.top() + 40.0),
            Pos2::new(rect.right() - 16.0, rect.top() + 158.0),
        );
        match &self.dash.current_preview {
            Some(pv) => preview::paint_preview(p, preview_rect, pv, 9),
            None => preview::paint_empty_preview(ui, preview_rect, 9),
        }
        let mut facts: Vec<(String, String)> = Vec::new();
        if self.dash.latest.is_some() {
            facts.push(("OS".into(), os_name()));
            facts.push(("Kernel".into(), kernel()));
            facts.push(("Uptime".into(), uptime()));
        } else {
            facts.push(("Folder".into(), self.dash.project_path.clone()));
            facts.push(("Captured".into(), "nothing yet".into()));
            facts.push(("Uptime".into(), uptime()));
        }
        let mut y = preview_rect.bottom() + 8.0;
        for (name, value) in facts {
            p.text(
                Pos2::new(rect.left() + 16.0, y),
                Align2::LEFT_TOP,
                name.to_uppercase(),
                bold(10.0),
                DIM,
            );
            p.text(
                Pos2::new(rect.left() + 78.0, y),
                Align2::LEFT_TOP,
                elide(p, &value, &regular(12.0), rect.width() - 94.0),
                regular(12.0),
                TEXT,
            );
            y += 18.0;
        }
    }

    fn paint_recent(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        card_header(p, rect, "★", AMBER, "Recent Snapshots");
        let link_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 16.0 - 76.0, rect.top() + 8.0),
            Vec2::new(76.0, 22.0),
        );
        let link = ui.interact(link_rect, ui.id().with("recentall"), Sense::click());
        p.text(
            Pos2::new(link_rect.right(), link_rect.center().y),
            Align2::RIGHT_CENTER,
            "View all →",
            bold(12.0),
            if link.hovered() { Color32::from_rgb(0x8f, 0xd6, 0xff) } else { ACCENT },
        );
        if link.clicked() {
            self.open_timeline_at(None);
        }

        let rows_top = rect.top() + 40.0;
        if self.dash.recent.is_empty() {
            p.text(
                Pos2::new(rect.left() + 16.0, rows_top + 18.0),
                Align2::LEFT_TOP,
                if self.query.trim().is_empty() {
                    "No snapshots yet — capture the first moment."
                } else {
                    "No snapshots match that search."
                },
                regular(13.0),
                MUTED,
            );
            return;
        }
        let mut open: Option<String> = None;
        for (i, snap) in self.dash.recent.clone().iter().enumerate() {
            let row = Rect::from_min_size(
                Pos2::new(rect.left() + 8.0, rows_top + i as f32 * 56.0),
                Vec2::new(rect.width() - 16.0, 56.0),
            );
            let response = ui.interact(row, ui.id().with(("recent", i)), Sense::click());
            if response.hovered() {
                p.rect_filled(row, 10, SURF2);
                p.rect_stroke(
                    row,
                    10,
                    Stroke::new(1.0, Color32::from_rgb(0x17, 0x34, 0x4d)),
                    StrokeKind::Inside,
                );
            }
            let badge_rect = Rect::from_min_size(
                Pos2::new(row.left() + 4.0, row.center().y - 18.0),
                Vec2::new(46.0, 36.0),
            );
            common::paint_badge(p, badge_rect, snap);
            let text_x = badge_rect.right() + 11.0;
            p.text(
                Pos2::new(text_x, row.top() + 10.0),
                Align2::LEFT_TOP,
                format!("{}, {}", store::short_day(snap.ts), store::clock_label(snap.ts)),
                bold(12.5),
                TEXT,
            );
            let note = if !snap.note.trim().is_empty() {
                snap.note.trim()
            } else if !snap.title.trim().is_empty() {
                snap.title.trim()
            } else {
                "No note"
            };
            p.text(
                Pos2::new(text_x, row.top() + 30.0),
                Align2::LEFT_TOP,
                elide(p, note, &regular(11.5), (row.width() - 250.0).max(80.0)),
                regular(11.5),
                MUTED,
            );
            let auto = snap.kind == "auto";
            let mut cx = row.right() - 8.0;
            p.text(
                Pos2::new(cx, row.center().y),
                Align2::RIGHT_CENTER,
                "›",
                regular(16.0),
                DIM,
            );
            cx -= 14.0;
            if snap.branch_id() != "main" {
                let font = FontId::new(11.5, bold_family());
                let text = format!("⑃ {}", snap.branch_id());
                let galley = p.layout_no_wrap(text, font, VIOLET);
                let w = galley.size().x + 18.0;
                let chip_rect = Rect::from_min_size(
                    Pos2::new(cx - w, row.center().y - 11.0),
                    Vec2::new(w, 22.0),
                );
                p.rect_filled(chip_rect, 8, SURF2);
                p.rect_stroke(
                    chip_rect,
                    8,
                    Stroke::new(1.0, darker(VIOLET, 3.4)),
                    StrokeKind::Inside,
                );
                p.galley(
                    chip_rect.min + Vec2::new(9.0, (22.0 - galley.size().y) * 0.5),
                    galley,
                    VIOLET,
                );
                cx -= w + 8.0;
            }
            let font = FontId::new(11.5, bold_family());
            let label = if auto { "Auto-save" } else { "manual" };
            let galley = p.layout_no_wrap(label.to_string(), font, if auto { MUTED } else { ACCENT });
            let w = galley.size().x + 18.0;
            let chip_rect = Rect::from_min_size(
                Pos2::new(cx - w, row.center().y - 11.0),
                Vec2::new(w, 22.0),
            );
            p.rect_filled(chip_rect, 8, SURF2);
            p.rect_stroke(
                chip_rect,
                8,
                Stroke::new(1.0, darker(if auto { MUTED } else { ACCENT }, 3.4)),
                StrokeKind::Inside,
            );
            p.galley(
                chip_rect.min + Vec2::new(9.0, (22.0 - galley.size().y) * 0.5),
                galley,
                if auto { MUTED } else { ACCENT },
            );
            if response.clicked() {
                open = Some(snap.id.clone());
            }
        }
        if let Some(sid) = open {
            self.open_timeline_at(Some(sid));
        }
    }

    fn paint_what_changed(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        card_header(p, rect, "◫", ACCENT, "What Changed?");
        let mut y = rect.top() + 44.0;
        if self.dash.chain.is_empty() {
            p.text(
                Pos2::new(rect.left() + 16.0, y),
                Align2::LEFT_TOP,
                "Nothing captured yet.",
                regular(13.0),
                MUTED,
            );
            return;
        }
        let (added, deleted, modified) = self.dash.changed;
        for (glyph, color, count, name) in [
            ("+", GREEN, added, "files added"),
            ("−", RED, deleted, "files removed"),
            ("▤", ACCENT, modified, "files modified"),
        ] {
            p.text(
                Pos2::new(rect.left() + 16.0, y),
                Align2::LEFT_TOP,
                glyph,
                bold(14.0),
                color,
            );
            p.text(
                Pos2::new(rect.left() + 38.0, y + 1.0),
                Align2::LEFT_TOP,
                format!("{} {}", count, name),
                regular(12.0),
                TEXT,
            );
            y += 24.0;
        }
        y += 2.0;
        p.rect_filled(
            Rect::from_min_size(Pos2::new(rect.left() + 16.0, y), Vec2::new(rect.width() - 32.0, 1.0)),
            0,
            BORDER,
        );
        y += 12.0;
        p.text(
            Pos2::new(rect.left() + 16.0, y),
            Align2::LEFT_TOP,
            "MOST CHANGED PROJECT",
            bold(10.0),
            DIM,
        );
        y += 20.0;
        let name = self
            .project()
            .map(|p| p.name)
            .unwrap_or_else(|| "—".into());
        p.text(
            Pos2::new(rect.left() + 16.0, y),
            Align2::LEFT_TOP,
            "▢",
            icon(14.0),
            ACCENT,
        );
        p.text(
            Pos2::new(rect.left() + 38.0, y),
            Align2::LEFT_TOP,
            elide(p, &name, &bold(12.0), rect.width() - 130.0),
            bold(12.0),
            TEXT,
        );
        p.text(
            Pos2::new(rect.right() - 16.0, y),
            Align2::RIGHT_TOP,
            format!("+{} changes", added + deleted + modified),
            bold(11.0),
            GREEN,
        );
    }

    fn paint_quick_actions(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        card_header(p, rect, "⚡", AMBER, "Quick Actions");
        let mut y = rect.top() + 42.0;
        let actions = [
            ("◉", "Create Snapshot", "Ctrl + S", 0u8),
            ("⚖", "Compare Snapshots", "Ctrl + Shift + C", 1),
            ("↺", "Restore Snapshot", "Ctrl + R", 2),
            ("◷", "Open Timeline", "Ctrl + T", 3),
        ];
        for (glyph, text, hint, action) in actions {
            let row = Rect::from_min_size(
                Pos2::new(rect.left() + 10.0, y),
                Vec2::new(rect.width() - 20.0, 40.0),
            );
            let response = ui.interact(row, ui.id().with(("quick", action)), Sense::click());
            if response.hovered() {
                p.rect_filled(row, 10, SURF2);
                p.rect_stroke(
                    row,
                    10,
                    Stroke::new(1.0, BORDER2),
                    StrokeKind::Inside,
                );
            }
            p.text(
                Pos2::new(row.left() + 12.0, row.center().y),
                Align2::LEFT_CENTER,
                glyph,
                icon(15.0),
                ACCENT,
            );
            p.text(
                Pos2::new(row.left() + 38.0, row.center().y),
                Align2::LEFT_CENTER,
                text,
                bold(12.5),
                Color32::from_rgb(0xdb, 0xe9, 0xf8),
            );
            p.text(
                Pos2::new(row.right() - 6.0, row.center().y),
                Align2::RIGHT_CENTER,
                hint,
                regular(10.5),
                DIM,
            );
            if response.clicked() {
                match action {
                    0 => self.dialog_new_snapshot(),
                    1 => self.compare_latest(),
                    2 => self.restore_latest(),
                    _ => self.open_timeline_at(None),
                }
            }
            y += 42.0;
        }
    }
}
