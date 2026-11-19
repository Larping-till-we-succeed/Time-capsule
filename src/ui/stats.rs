//! Statistics page: how much history exists, where it grew and what changes most.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_stats(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let Some(stats) = self.stats.clone() else {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "No project selected.",
                regular(13.0),
                MUTED,
            );
            return;
        };
        let wide = rect.width() >= 900.0;
        let cols = if wide { 4usize } else { 2usize };
        let rows = if wide { 1.0 } else { 2.0 };
        let stat_w = (rect.width() - 12.0 * (cols - 1) as f32) / cols as f32;
        let saved = (stats.live_bytes * stats.snapshots.max(1) as u64).saturating_sub(stats.storage);
        let cards = [
            (
                stats.snapshots.to_string(),
                "SNAPSHOTS",
                "🕰",
                ACCENT,
                String::new(),
            ),
            (
                stats.files.to_string(),
                "TRACKED FILES",
                "▤",
                BLUE,
                "tracked in the live folder".into(),
            ),
            (
                store::fmt_bytes(stats.live_bytes),
                "PROJECT SIZE",
                "◈",
                GREEN,
                "sum of tracked files".into(),
            ),
            (
                store::fmt_bytes(stats.storage),
                "PROJECT STORAGE",
                "❖",
                BLUE,
                format!("{} saved by dedupe", store::fmt_bytes(saved)),
            ),
        ];
        for (i, (value, caption, glyph, color, hint)) in cards.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            common::stat_card(
                ui,
                Rect::from_min_size(
                    Pos2::new(
                        rect.left() + col as f32 * (stat_w + 12.0),
                        rect.top() + row as f32 * 124.0,
                    ),
                    Vec2::new(stat_w, 112.0),
                ),
                value,
                caption,
                glyph,
                *color,
                "",
                0,
                &[],
                hint,
            );
        }

        let chart_y = rect.top() + rows * 112.0 + (rows - 1.0) * 12.0 + 12.0;
        let chart_h = (rect.bottom() - chart_y - 16.0 - 190.0).clamp(168.0, 268.0);
        let chart_rect = Rect::from_min_size(
            Pos2::new(rect.left(), chart_y),
            Vec2::new(rect.width(), chart_h),
        );
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, chart_rect, 16, SURF);
        p.text(
            Pos2::new(chart_rect.left() + 18.0, chart_rect.top() + 14.0),
            Align2::LEFT_TOP,
            "Snapshots over the last 14 days",
            bold(15.0),
            TEXT,
        );
        let span = if let Some(first) = stats.first {
            format!("since {}", store::fmt_date(first))
        } else {
            "no history yet".into()
        };
        p.text(
            Pos2::new(chart_rect.left() + 250.0, chart_rect.top() + 17.0),
            Align2::LEFT_TOP,
            span,
            regular(12.0),
            MUTED,
        );
        let chart_body = Rect::from_min_max(
            Pos2::new(chart_rect.left() + 18.0, chart_rect.top() + 44.0),
            Pos2::new(chart_rect.right() - 18.0, chart_rect.bottom() - 12.0),
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(chart_body)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        common::bar_chart(&mut child, chart_body.height(), &stats.per_day);

        let bottom_y = chart_rect.bottom() + 16.0;
        let bottom_h = (rect.bottom() - bottom_y).max(120.0);
        let facts_w = 330.0;
        let left_rect = Rect::from_min_size(
            Pos2::new(rect.left(), bottom_y),
            Vec2::new(rect.width() - facts_w - 16.0, bottom_h),
        );
        let facts_rect = Rect::from_min_size(
            Pos2::new(rect.right() - facts_w, bottom_y),
            Vec2::new(facts_w, bottom_h),
        );

        // most reworked files
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, left_rect, 16, SURF);
        p.text(
            Pos2::new(left_rect.left() + 18.0, left_rect.top() + 12.0),
            Align2::LEFT_TOP,
            "Most reworked files",
            bold(15.0),
            TEXT,
        );
        let list = Rect::from_min_max(
            Pos2::new(left_rect.left() + 12.0, left_rect.top() + 38.0),
            Pos2::new(left_rect.right() - 12.0, left_rect.bottom() - 8.0),
        );
        let top = stats.top_changed.clone();
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(list)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        egui::ScrollArea::vertical()
            .id_salt("top_changed")
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                if top.is_empty() {
                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new(
                            "Files that get edited between snapshots will show up here.",
                        )
                        .font(regular(13.0))
                        .color(MUTED),
                    );
                    return;
                }
                let peak = top[0].1.max(1);
                for (rel, count) in &top {
                    let (r, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 32.0),
                        Sense::hover(),
                    );
                    let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
                    p.text(
                        Pos2::new(r.left() + 4.0, r.top() + 1.0),
                        Align2::LEFT_TOP,
                        elide(p, rel, &mono(12.0), r.width() - 54.0),
                        mono(12.0),
                        TEXT,
                    );
                    p.text(
                        Pos2::new(r.right() - 4.0, r.top() + 1.0),
                        Align2::RIGHT_TOP,
                        format!("{}×", count),
                        bold(11.0),
                        ACCENT,
                    );
                    let bar = Rect::from_min_size(
                        Pos2::new(r.left() + 4.0, r.bottom() - 8.0),
                        Vec2::new(r.width() - 8.0, 6.0),
                    );
                    common::mini_bar(p, bar, *count as f32 / peak as f32, ACCENT);
                    ui.add_space(6.0);
                }
            });

        // facts
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, facts_rect, 16, SURF);
        p.text(
            Pos2::new(facts_rect.left() + 18.0, facts_rect.top() + 12.0),
            Align2::LEFT_TOP,
            "Project facts",
            bold(15.0),
            TEXT,
        );
        let proj = self.project();
        let facts = [
            (
                "BRANCH",
                format!("{} · {} total", stats.branch_name, stats.branches),
            ),
            (
                "TIMELINE SPAN",
                if stats.span_days > 0.0 {
                    format!("{:.1} days", stats.span_days)
                } else {
                    "one moment".into()
                },
            ),
            (
                "FIRST CAPTURE",
                stats.first.map(store::fmt_stamp).unwrap_or_else(|| "—".into()),
            ),
            (
                "LATEST CAPTURE",
                stats.last.map(store::fmt_stamp).unwrap_or_else(|| "—".into()),
            ),
            (
                "FOLDER",
                proj.map(|p| p.path).unwrap_or_else(|| "—".into()),
            ),
        ];
        let mut y = facts_rect.top() + 44.0;
        for (name, value) in facts {
            p.text(
                Pos2::new(facts_rect.left() + 18.0, y),
                Align2::LEFT_TOP,
                name,
                bold(10.0),
                DIM,
            );
            p.text(
                Pos2::new(facts_rect.left() + 18.0, y + 14.0),
                Align2::LEFT_TOP,
                elide(p, &value, &bold(13.0), facts_rect.width() - 36.0),
                bold(13.0),
                Color32::from_rgb(0xe6, 0xe9, 0xef),
            );
            y += 46.0;
            if y > facts_rect.bottom() - 22.0 {
                break;
            }
        }
        let _ = ui.available_width();
        let _ = Stroke::NONE;
        let _ = StrokeKind::Inside;
    }
}
