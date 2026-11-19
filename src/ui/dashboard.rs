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

impl App {
    pub fn open_timeline_at(&mut self, sid: Option<String>) {
        if self.pid.is_none() {
            return;
        }
        self.page = crate::Page::Timeline;
        self.rebuild_timeline(sid);
    }

    pub fn ui_dashboard(&mut self, ui: &mut egui::Ui) {
        let wide = ui.available_width() >= 900.0;
        let content_h = if wide { 676.0 } else { 1352.0 };
        egui::ScrollArea::vertical()
            .id_salt("dashboard")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let width = ui.available_width();
                let (content, _) =
                    ui.allocate_exact_size(Vec2::new(width, content_h), Sense::hover());
                let gap = 12.0;
                let stat_h = 104.0;
                let cols = if wide { 4usize } else { 2usize };
                let stat_w = (width - gap * (cols - 1) as f32) / cols as f32;
                let rows = if wide { 1.0 } else { 2.0 };

                let series = self.dash.series.clone();
                let project_series: Vec<f64> = (0..7)
                    .map(|i| (self.dash.project_count as i64 - (6 - i)).max(0) as f64)
                    .collect();
                let sizes = self.dash.sizes.clone();
                let cards: [(&str, &str, &str, Color32, String, i64, Vec<f64>); 3] = [
                    (
                        "Total Snapshots",
                        "▤",
                        "accent",
                        ACCENT,
                        if self.dash.pct >= 0 {
                            format!("↑ +{}%", self.dash.pct)
                        } else {
                            format!("↓ {}%", self.dash.pct)
                        },
                        self.dash.pct,
                        series,
                    ),
                    (
                        "Tracked Projects",
                        "▢",
                        "blue",
                        Color32::from_rgb(0x60, 0xa5, 0xfa),
                        if self.dash.new_projects > 0 {
                            format!("↑ +{}", self.dash.new_projects)
                        } else {
                            "stable".into()
                        },
                        if self.dash.new_projects > 0 { 1 } else { 0 },
                        project_series,
                    ),
                    (
                        "Used Space",
                        "◍",
                        "violet",
                        VIOLET,
                        if self.dash.space_delta >= 0 {
                            format!("↑ +{}%", self.dash.space_delta)
                        } else {
                            format!("↓ {}%", self.dash.space_delta)
                        },
                        self.dash.space_delta,
                        sizes,
                    ),
                ];
                let values = [
                    self.dash.total_snaps.to_string(),
                    self.dash.project_count.to_string(),
                    store::fmt_bytes(self.dash.used),
                ];

                for i in 0..4 {
                    let col = i % cols;
                    let row = i / cols;
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            content.left() + col as f32 * (stat_w + gap),
                            content.top() + row as f32 * (stat_h + gap),
                        ),
                        Vec2::new(stat_w, stat_h),
                    );
                    if i == 3 {
                        self.paint_status_card(ui, rect);
                    } else {
                        let (caption, glyph, _, color, delta, trend, data) = &cards[i];
                        common::stat_card(
                            ui,
                            rect,
                            &values[i],
                            caption,
                            glyph,
                            *color,
                            delta,
                            *trend,
                            data,
                            "",
                        );
                    }
                }
                let mut y = content.top() + rows * stat_h + (rows - 1.0) * gap + gap;

                let mid_h = 252.0;
                if wide {
                    let current_w = 352.0;
                    let timeline_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width - current_w - gap, mid_h),
                    );
                    self.paint_timeline_card(ui, timeline_rect);
                    let current_rect = Rect::from_min_size(
                        Pos2::new(content.right() - current_w, y),
                        Vec2::new(current_w, mid_h),
                    );
                    self.paint_current_state(ui, current_rect);
                    y += mid_h + gap;
                } else {
                    let timeline_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, mid_h),
                    );
                    self.paint_timeline_card(ui, timeline_rect);
                    y += mid_h + gap;
                    let current_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, 236.0),
                    );
                    self.paint_current_state(ui, current_rect);
                    y += 236.0 + gap;
                }

                let bottom_h = 296.0;
                if wide {
                    let what_w = 276.0;
                    let quick_w = 312.0;
                    let recent_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width - what_w - quick_w - gap * 2.0, bottom_h),
                    );
                    self.paint_recent(ui, recent_rect);
                    let what_rect = Rect::from_min_size(
                        Pos2::new(recent_rect.right() + gap, y),
                        Vec2::new(what_w, bottom_h),
                    );
                    self.paint_what_changed(ui, what_rect);
                    let quick_rect = Rect::from_min_size(
                        Pos2::new(what_rect.right() + gap, y),
                        Vec2::new(quick_w, bottom_h),
                    );
                    self.paint_quick_actions(ui, quick_rect);
                } else {
                    let recent_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(width, bottom_h),
                    );
                    self.paint_recent(ui, recent_rect);
                    y += bottom_h + gap;
                    let half = (width - gap) / 2.0;
                    let what_rect = Rect::from_min_size(
                        Pos2::new(content.left(), y),
                        Vec2::new(half, bottom_h),
                    );
                    self.paint_what_changed(ui, what_rect);
                    let quick_rect = Rect::from_min_size(
                        Pos2::new(what_rect.right() + gap, y),
                        Vec2::new(half, bottom_h),
                    );
                    self.paint_quick_actions(ui, quick_rect);
                }
            });
    }

    fn paint_status_card(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let (ok, title, sub) = if self.pid.is_none() {
            (false, "No project yet", "Add a folder to start capturing.")
        } else if self.dash.dirty {
            (false, "Changes pending", "The folder changed since the last snapshot.")
        } else {
            (true, "All good!", "Everything is backed up.")
        };
        let color = if ok { GREEN } else { AMBER };
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        let tile = Rect::from_min_size(
            Pos2::new(rect.left() + 16.0, rect.top() + 14.0),
            Vec2::splat(44.0),
        );
        p.rect_filled(tile, 12, darker(color, 4.3));
        p.rect_stroke(
            tile,
            12,
            Stroke::new(1.0, darker(color, 3.2)),
            StrokeKind::Inside,
        );
        p.text(
            tile.center(),
            Align2::CENTER_CENTER,
            if ok { "✓" } else { "!" },
            icon(19.0),
            color,
        );
        p.text(
            Pos2::new(tile.right() + 12.0, rect.top() + 24.0),
            Align2::LEFT_TOP,
            title,
            bold(16.0),
            TEXT,
        );
        p.text(
            Pos2::new(tile.right() + 12.0, rect.top() + 48.0),
            Align2::LEFT_TOP,
            elide(p, sub, &regular(12.0), rect.right() - tile.right() - 26.0),
            regular(12.0),
            MUTED,
        );
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
                    Color32::from_rgb(0x0d, 0x20, 0x30)
                } else if response.hovered() {
                    Color32::from_rgb(0x0c, 0x1e, 0x2e)
                } else {
                    Color32::from_rgb(0x0a, 0x18, 0x26)
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
                p.rect_filled(row, 10, Color32::from_rgb(0x0e, 0x21, 0x35));
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
                p.rect_filled(chip_rect, 8, Color32::from_rgb(0x0f, 0x24, 0x37));
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
            p.rect_filled(chip_rect, 8, Color32::from_rgb(0x0f, 0x24, 0x37));
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
                p.rect_filled(row, 10, Color32::from_rgb(0x0e, 0x21, 0x35));
                p.rect_stroke(
                    row,
                    10,
                    Stroke::new(1.0, Color32::from_rgb(0x1b, 0x3f, 0x5c)),
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
