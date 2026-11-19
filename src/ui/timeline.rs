//! Timeline page: a day-grouped rail of snapshots next to the detail panel.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::store::{self, Snapshot};
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_timeline(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let rail_rect = Rect::from_min_size(
            rect.min,
            Vec2::new(rect.width(), 176.0),
        );
        self.paint_rail_card(ui, rail_rect);

        let body = Rect::from_min_max(
            Pos2::new(rect.left(), rail_rect.bottom() + 14.0),
            rect.max,
        );
        let left_w = 424.0_f32.min(body.width() * 0.42);
        let left = Rect::from_min_size(body.min, Vec2::new(left_w, body.height()));
        let detail = Rect::from_min_max(
            Pos2::new(left.right() + 16.0, body.top()),
            body.max,
        );
        self.paint_history(ui, left);
        if self.timeline.chain.is_empty() {
            self.paint_empty_detail(ui, detail);
        } else {
            self.paint_detail(ui, detail);
        }
    }

    fn paint_rail_card(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        p.text(
            Pos2::new(rect.left() + 20.0, rect.top() + 15.0),
            Align2::LEFT_TOP,
            "PROJECT TIMELINE",
            bold(10.0),
            DIM,
        );
        let span_text = if self.timeline.chain.is_empty() {
            "nothing captured yet".to_string()
        } else if self.timeline.chain.len() > 1 {
            let span = (self.timeline.chain.last().unwrap().ts
                - self.timeline.chain[0].ts)
                / 86400.0;
            if self.timeline.branches.len() > 1 {
                format!(
                    "{} moments · {:.1} days · {} branches",
                    self.timeline.chain.len(),
                    span,
                    self.timeline.branches.len()
                )
            } else {
                format!("{} moments · {:.1} days of history", self.timeline.chain.len(), span)
            }
        } else if self.timeline.branches.len() == 1 {
            "1 moment".into()
        } else {
            "1 moment on this branch".into()
        };
        let narrow = rect.width() < 860.0;
        if !narrow {
            p.text(
                Pos2::new(rect.left() + 152.0, rect.top() + 16.0),
                Align2::LEFT_TOP,
                span_text,
                regular(12.0),
                MUTED,
            );
        }

        // legend + branch picker on the right
        let mut x = rect.right() - 20.0;
        let font = FontId::new(11.5, bold_family());
        let legend = [
            ("● manual snapshot", Color32::from_rgb(0x8f, 0xa6, 0xc0)),
            ("● auto-save", Color32::from_rgb(0x45, 0x4b, 0x57)),
        ];
        for (text, color) in legend.iter().rev() {
            if narrow {
                break;
            }
            let galley = p.layout_no_wrap(text.to_string(), font.clone(), *color);
            let w = galley.size().x + 18.0;
            let chip_rect = Rect::from_min_size(
                Pos2::new(x - w, rect.top() + 12.0),
                Vec2::new(w, 22.0),
            );
            p.rect_filled(chip_rect, 8, Color32::from_rgb(0x0f, 0x24, 0x37));
            p.rect_stroke(
                chip_rect,
                8,
                Stroke::new(1.0, darker(*color, 3.4)),
                StrokeKind::Inside,
            );
            p.galley(
                chip_rect.min + Vec2::new(9.0, (22.0 - galley.size().y) * 0.5),
                galley,
                *color,
            );
            x -= w + 8.0;
        }
        x -= 10.0;
        let new_branch_rect = Rect::from_min_size(
            Pos2::new(x - 116.0, rect.top() + 10.0),
            Vec2::new(116.0, 26.0),
        );
        let new_branch_resp = ui.interact(
            new_branch_rect,
            ui.id().with("newbranch"),
            Sense::click(),
        );
        let hovered = new_branch_resp.hovered();
        p.rect_filled(
            new_branch_rect,
            8,
            if hovered { Color32::from_rgb(0x14, 0x2c, 0x44) } else { Color32::from_rgb(0x0e, 0x21, 0x35) },
        );
        p.rect_stroke(
            new_branch_rect,
            8,
            Stroke::new(1.0, if hovered { Color32::from_rgb(0x2a, 0x5d, 0x85) } else { Color32::from_rgb(0x1b, 0x3f, 0x5c) }),
            StrokeKind::Inside,
        );
        p.text(
            new_branch_rect.center(),
            Align2::CENTER_CENTER,
            "⑃   New branch",
            bold(12.0),
            TEXT,
        );
        if new_branch_resp.clicked() {
            self.dialog_new_branch();
        }
        x = new_branch_rect.left() - 10.0;
        let combo_w = if narrow { 190.0 } else { 210.0 };
        let combo_rect = Rect::from_min_size(
            Pos2::new(x - combo_w, rect.top() + 8.0),
            Vec2::new(combo_w, 30.0),
        );
        if !narrow {
            p.text(
                Pos2::new(combo_rect.left() - 12.0, rect.top() + 15.0),
                Align2::RIGHT_TOP,
                "BRANCH",
                bold(10.0),
                DIM,
            );
        }
        let label = self
            .timeline
            .branches
            .iter()
            .find(|b| b.id == self.branch)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "main".into());
        let count = self.timeline.all_snaps.iter().filter(|s| s.branch_id() == self.branch).count();
        common::area(ui, combo_rect, |ui| {
            egui::ComboBox::from_id_salt("branch_combo")
                .selected_text(format!("{}  ·  {} snapshots", label, count))
                .width(combo_w)
                .show_ui(ui, |ui| {
                    let branches = self.timeline.branches.clone();
                    for b in branches {
                        let n = self
                            .timeline
                            .all_snaps
                            .iter()
                            .filter(|s| s.branch_id() == b.id)
                            .count();
                        let text = format!(
                            "{}  ·  {} snapshot{}",
                            b.name,
                            n,
                            if n == 1 { "" } else { "s" }
                        );
                        if ui.selectable_label(b.id == self.branch, text).clicked() {
                            self.select_branch(&b.id);
                        }
                    }
                });
        });

        let rail = Rect::from_min_max(
            Pos2::new(rect.left() + 20.0, rect.top() + 50.0),
            Pos2::new(rect.right() - 20.0, rect.bottom() - 10.0),
        );
        self.paint_rail(ui, rail);
    }

    fn paint_rail(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        if self.timeline.all_snaps.is_empty() {
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Every capture you make lands on this line",
                regular(12.0),
                MUTED,
            );
            return;
        }
        let mut ordered = self.timeline.branches.clone();
        ordered.sort_by(|a, b| {
            (a.id != "main")
                .cmp(&(b.id != "main"))
                .then(a.created.partial_cmp(&b.created).unwrap_or(std::cmp::Ordering::Equal))
        });
        let lane_of = |id: &str| ordered.iter().position(|b| b.id == id).unwrap_or(0);
        let count = ordered.len().max(1);
        let lane_y = |index: usize| {
            let span = (rect.height() - 6.0 - 20.0).max(1.0);
            rect.top() + 6.0 + span * (index as f32 + 0.5) / count as f32
        };
        let proj_created = self
            .project()
            .map(|p| p.created)
            .unwrap_or_else(|| self.timeline.all_snaps[0].ts);
        let left = rect.left() + 16.0;
        let edge = rect.right() - 16.0;
        let label_pad = ordered
            .iter()
            .map(|b| p.layout_no_wrap(b.name.clone(), regular(10.0), TEXT).size().x)
            .fold(0.0_f32, f32::max)
            + 28.0;
        let right = edge - label_pad;
        let t0 = proj_created.min(self.timeline.all_snaps[0].ts);
        let t1 = store::now().max(self.timeline.all_snaps.last().unwrap().ts);
        let t1 = if t1 <= t0 { t0 + 1.0 } else { t1 };
        let x_of = |ts: f64| {
            let frac = ((ts - t0) / (t1 - t0)).clamp(0.0, 1.0) as f32;
            left + (right - left) * frac
        };
        let main_y = lane_y(0);
        p.line_segment(
            [Pos2::new(left, main_y - 5.0), Pos2::new(left, main_y + 5.0)],
            Stroke::new(1.0, Color32::from_rgb(0x1c, 0x40, 0x5e)),
        );
        p.line_segment(
            [Pos2::new(right, main_y - 5.0), Pos2::new(right, main_y + 5.0)],
            Stroke::new(1.0, Color32::from_rgb(0x1c, 0x40, 0x5e)),
        );
        p.text(
            Pos2::new(left, rect.bottom() - 2.0),
            Align2::LEFT_BOTTOM,
            store::fmt_date(t0),
            regular(10.0),
            DIM,
        );
        p.text(
            Pos2::new(edge, rect.bottom() - 2.0),
            Align2::RIGHT_BOTTOM,
            "today",
            regular(10.0),
            DIM,
        );

        let mut clicked: Option<String> = None;
        for (idx, branch) in ordered.clone().iter().enumerate() {
            let y = lane_y(idx);
            let active = branch.id == self.branch;
            let mut own: Vec<&Snapshot> = self
                .timeline
                .all_snaps
                .iter()
                .filter(|s| s.branch_id() == branch.id)
                .collect();
            own.sort_by(|a, b| a.ts.partial_cmp(&b.ts).unwrap_or(std::cmp::Ordering::Equal));
            let line_color = if active {
                Color32::from_rgb(0x6f, 0x93, 0xb8)
            } else {
                Color32::from_rgb(0x1b, 0x3b, 0x57)
            };
            let base = branch
                .base
                .as_ref()
                .and_then(|id| self.timeline.all_snaps.iter().find(|s| &s.id == id));
            if let Some(base) = base {
                let bx = x_of(base.ts);
                let by = lane_y(lane_of(base.branch_id()));
                p.line_segment(
                    [Pos2::new(left, y), Pos2::new(bx, y)],
                    Stroke::new(1.0, Color32::from_rgb(0x13, 0x2c, 0x44)),
                );
                p.line_segment(
                    [Pos2::new(bx, by), Pos2::new(bx, y)],
                    Stroke::new(2.0, line_color),
                );
                p.line_segment(
                    [Pos2::new(bx, y), Pos2::new(right, y)],
                    Stroke::new(2.0, line_color),
                );
            } else {
                p.line_segment(
                    [Pos2::new(left, y), Pos2::new(right, y)],
                    Stroke::new(if active { 2.0 } else { 1.0 }, line_color),
                );
            }
            let label = branch.name.clone();
            let galley = p.layout_no_wrap(label.clone(), regular(10.0), TEXT);
            let w = galley.size().x + 18.0;
            let label_rect = Rect::from_center_size(
                Pos2::new(edge - w / 2.0, y),
                Vec2::new(w, 16.0),
            );
            p.rect_filled(
                label_rect,
                8,
                if active { Color32::from_rgb(0x10, 0x2a, 0x42) } else { Color32::from_rgb(0x07, 0x15, 0x23) },
            );
            p.rect_stroke(
                label_rect,
                8,
                Stroke::new(1.0, Color32::from_rgb(0x1a, 0x3b, 0x58)),
                StrokeKind::Inside,
            );
            p.galley(
                label_rect.min + Vec2::new(9.0, (16.0 - galley.size().y) * 0.5),
                galley,
                if active { ACCENT } else { DIM },
            );
            if y < rect.top() + 2.0 {
                continue;
            }
            let radius = if own.len() <= 40 { 4.5 } else { 3.5 };
            for snap in own {
                let x = x_of(snap.ts);
                let chosen = self.timeline.selected.as_deref() == Some(snap.id.as_str());
                let color = if chosen {
                    ACCENT
                } else if active {
                    if snap.kind == "auto" {
                        Color32::from_rgb(0x6f, 0x93, 0xb8)
                    } else {
                        Color32::from_rgb(0xa8, 0xc0, 0xd8)
                    }
                } else {
                    Color32::from_rgb(0x34, 0x59, 0x7a)
                };
                if chosen {
                    p.circle_filled(
                        Pos2::new(x, y),
                        (radius + 1.2) * 2.4,
                        alpha(ACCENT, 46),
                    );
                }
                p.circle_filled(Pos2::new(x, y), radius + if chosen { 1.2 } else { 0.0 }, color);
                let hit = Rect::from_center_size(Pos2::new(x, y), Vec2::splat(22.0));
                let response = ui.interact(
                    hit,
                    ui.id().with(("dot", idx, &snap.id)),
                    Sense::click(),
                );
                if response.clicked() {
                    clicked = Some(snap.id.clone());
                }
                if response.hovered() {
                    let kind = if snap.kind == "auto" { "Auto-save" } else { "Snapshot" };
                    response.on_hover_text(format!(
                        "{} · {} · {}",
                        kind,
                        store::fmt_stamp(snap.ts),
                        branch.name
                    ));
                }
            }
        }
        if let Some(sid) = clicked {
            self.select_snapshot(&sid);
        }
    }

    fn paint_history(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        p.text(
            Pos2::new(rect.left() + 16.0, rect.top() + 14.0),
            Align2::LEFT_TOP,
            "HISTORY",
            bold(10.0),
            DIM,
        );
        let query = self.timeline.search.trim().to_lowercase();
        let shown: Vec<Snapshot> = self
            .timeline
            .chain
            .iter()
            .rev()
            .filter(|s| {
                query.is_empty()
                    || s.title.to_lowercase().contains(&query)
                    || s.note.to_lowercase().contains(&query)
                    || s.files.keys().any(|f| f.to_lowercase().contains(&query))
            })
            .cloned()
            .collect();
        let count_text = if query.is_empty() {
            format!("{} snapshots", self.timeline.chain.len())
        } else {
            format!("{} of {}", shown.len(), self.timeline.chain.len())
        };
        p.text(
            Pos2::new(rect.left() + 82.0, rect.top() + 15.0),
            Align2::LEFT_TOP,
            count_text,
            regular(12.0),
            MUTED,
        );
        let search_rect = Rect::from_min_max(
            Pos2::new(rect.left() + 16.0, rect.top() + 38.0),
            Pos2::new(rect.right() - 12.0, rect.top() + 66.0),
        );
        common::area(ui, search_rect, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.timeline.search)
                    .hint_text(
                        egui::RichText::new("Search notes and files…")
                            .font(regular(12.0))
                            .color(DIM),
                    )
                    .font(regular(12.0))
                    .desired_width(search_rect.width()),
            );
        });

        let list = Rect::from_min_max(
            Pos2::new(rect.left() + 8.0, rect.top() + 76.0),
            Pos2::new(rect.right() - 8.0, rect.bottom() - 10.0),
        );
        if self.timeline.chain.is_empty() {
            let mut clicked = false;
            common::area(ui, list, |ui| {
                if common::empty_state(
                    ui,
                    "🕰",
                    "No snapshots yet",
                    "Capture the current state of your project and it will appear here, grouped by day, ready to revisit.",
                    "Capture the first snapshot",
                )
                .map(|r| r.clicked())
                .unwrap_or(false)
                {
                    clicked = true;
                }
            });
            if clicked {
                self.dialog_new_snapshot();
            }
            return;
        }

        let mut clicked: Option<String> = None;
        let prev_by_id: std::collections::HashMap<String, Option<Snapshot>> = {
            let mut m = std::collections::HashMap::new();
            for (i, s) in self.timeline.chain.iter().enumerate() {
                m.insert(s.id.clone(), if i == 0 { None } else { Some(self.timeline.chain[i - 1].clone()) });
            }
            m
        };
        let points = self.branch_points();
        let mut row_y = list.top();
        common::area(ui, list, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("history")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if shown.is_empty() {
                        ui.add_space(28.0);
                        ui.label(
                            egui::RichText::new("No snapshot matches that search.")
                                .font(regular(13.0))
                                .color(MUTED),
                        );
                        return;
                    }
                    let mut last_day: Option<String> = None;
                    let width = ui.available_width();
                    for snap in &shown {
                        let day = store::day_key(snap.ts);
                        if last_day.as_deref() != Some(day.as_str()) {
                            ui.add_space(10.0);
                            let (r, _) = ui.allocate_exact_size(
                                Vec2::new(width, 20.0),
                                Sense::hover(),
                            );
                            ui.painter().text(
                                Pos2::new(r.left() + 40.0, r.center().y),
                                Align2::LEFT_CENTER,
                                store::day_label(snap.ts).to_uppercase(),
                                bold(10.0),
                                Color32::from_rgb(0x6f, 0x8a, 0xa6),
                            );
                            last_day = Some(day);
                        }
                        let (r, response) = ui.allocate_exact_size(
                            Vec2::new(width, 86.0),
                            Sense::click(),
                        );
                        let selected = self.timeline.selected.as_deref() == Some(snap.id.as_str());
                        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
                        if selected {
                            p.rect_filled(r, egui::CornerRadius::same(10), Color32::from_rgb(0x10, 0x29, 0x3f));
                            p.rect_stroke(
                                r,
                                10,
                                Stroke::new(1.0, Color32::from_rgb(0x1d, 0x5c, 0x8c)),
                                StrokeKind::Inside,
                            );
                            p.rect_filled(
                                Rect::from_min_size(r.min, Vec2::new(3.0, r.height())),
                                0,
                                ACCENT,
                            );
                        } else if response.hovered() {
                            p.rect_filled(r, 10, Color32::from_rgb(0x0c, 0x20, 0x33));
                        }
                        let x = r.left() + 24.0;
                        p.line_segment(
                            [Pos2::new(x, r.top()), Pos2::new(x, r.bottom())],
                            Stroke::new(2.0, Color32::from_rgb(0x1a, 0x3b, 0x58)),
                        );
                        let center = Pos2::new(x, r.center().y);
                        if selected {
                            p.circle_filled(center, 13.0, alpha(ACCENT, 40));
                        }
                        p.circle_filled(
                            center,
                            5.0,
                            if selected { ACCENT } else { Color32::from_rgb(0x2f, 0x55, 0x75) },
                        );
                        if selected {
                            p.circle_filled(center, 2.0, Color32::from_rgb(0x10, 0x29, 0x3f));
                        }
                        let tx = r.left() + 46.0;
                        p.text(
                            Pos2::new(tx, r.top() + 12.0),
                            Align2::LEFT_TOP,
                            store::fmt_time(snap.ts),
                            bold(15.0),
                            TEXT,
                        );
                        let title = if snap.title.is_empty() { "Snapshot" } else { &snap.title };
                        p.text(
                            Pos2::new(tx + 58.0, r.top() + 16.0),
                            Align2::LEFT_TOP,
                            elide(p, title, &bold(12.0), r.width() - 130.0),
                            bold(12.0),
                            MUTED,
                        );
                        let note = if !snap.note.trim().is_empty() {
                            snap.note.trim()
                        } else {
                            "No note"
                        };
                        p.text(
                            Pos2::new(tx, r.top() + 36.0),
                            Align2::LEFT_TOP,
                            elide(p, note, &regular(12.0), r.width() - 60.0),
                            regular(12.0),
                            if snap.note.trim().is_empty() { DIM } else { Color32::from_rgb(0xc8, 0xcc, 0xd4) },
                        );
                        let mut cx = tx;
                        let meta_y = r.top() + 58.0;
                        let prev = prev_by_id.get(&snap.id).and_then(|p| p.clone());
                        if let Some(prev) = prev {
                            let d = store::compare(&prev, snap);
                            for (kind, mark, color) in [
                                ("added", "+", GREEN),
                                ("modified", "~", ACCENT),
                                ("deleted", "−", RED),
                            ] {
                                let n = match kind {
                                    "added" => d.added.len(),
                                    "modified" => d.modified.len(),
                                    _ => d.deleted.len(),
                                };
                                if n > 0 {
                                    let w = common::paint_chip(
                                        p,
                                        Pos2::new(cx, meta_y),
                                        &format!("{}{}", mark, n),
                                        color,
                                        Some(darker(color, 3.4)),
                                    );
                                    cx += w + 6.0;
                                }
                            }
                        } else {
                            let w = common::paint_chip(
                                p,
                                Pos2::new(cx, meta_y),
                                "first snapshot",
                                MUTED,
                                None,
                            );
                            cx += w + 6.0;
                        }
                        if let Some(names) = points.get(&snap.id) {
                            for name in names {
                                let w = common::paint_chip(
                                    p,
                                    Pos2::new(cx, meta_y),
                                    &format!("⑃ {}", name),
                                    BLUE,
                                    None,
                                );
                                cx += w + 6.0;
                            }
                        }
                        p.text(
                            Pos2::new(r.right() - 14.0, meta_y + 2.0),
                            Align2::RIGHT_TOP,
                            store::fmt_bytes(snap.bytes),
                            regular(11.0),
                            DIM,
                        );
                        if response.clicked() {
                            clicked = Some(snap.id.clone());
                        }
                    }
                    ui.add_space(row_y * 0.0 + 6.0);
                });
        });
        if let Some(sid) = clicked {
            self.select_snapshot(&sid);
        }
    }

    fn paint_empty_detail(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
        card(p, rect, 16, SURF);
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Snapshots you capture show their notes, files and restore actions here.",
            regular(12.5),
            MUTED,
        );
    }

    fn paint_detail(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let Some(snap) = self.timeline.detail.clone() else {
            self.paint_empty_detail(ui, rect);
            return;
        };
        let Some(pid) = self.pid.clone() else { return };
        let prev = self.timeline.prev.clone();
        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;

        // header
        let head = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 104.0));
        card(p, head, 16, SURF);
        p.text(
            Pos2::new(head.left() + 18.0, head.top() + 12.0),
            Align2::LEFT_TOP,
            store::fmt_time(snap.ts),
            bold(19.0),
            TEXT,
        );
        p.text(
            Pos2::new(head.left() + 18.0, head.top() + 38.0),
            Align2::LEFT_TOP,
            format!("{} · {}", store::fmt_date(snap.ts), store::ago(snap.ts)),
            regular(12.0),
            MUTED,
        );
        let title = if snap.title.is_empty() { "Untitled moment" } else { &snap.title };
        p.text(
            Pos2::new(head.left() + 18.0, head.top() + 56.0),
            Align2::LEFT_TOP,
            format!(
                "{}  ·  {} files  ·  {}",
                title,
                snap.count,
                store::fmt_bytes(snap.bytes)
            ),
            regular(12.0),
            MUTED,
        );
        let base = prev.clone().unwrap_or_default();
        let d = store::compare(&base, &snap);
        let mut cx = head.left() + 18.0;
        let chip_y = head.top() + 76.0;
        if snap.kind == "auto" {
            cx += common::paint_chip(p, Pos2::new(cx, chip_y), "auto-save", MUTED, None) + 8.0;
        }
        for (kind, n) in [
            ("modified", d.modified.len()),
            ("added", d.added.len()),
            ("deleted", d.deleted.len()),
        ] {
            if n > 0 {
                let color = status_color(kind);
                cx += common::paint_chip(
                    p,
                    Pos2::new(cx, chip_y),
                    &format!("{} {}", n, kind),
                    color,
                    Some(darker(color, 3.4)),
                ) + 8.0;
            }
        }
        if d.added.is_empty() && d.modified.is_empty() && d.deleted.is_empty() {
            common::paint_chip(p, Pos2::new(cx, chip_y), "identical to previous", MUTED, None);
        } else if prev.is_none() {
            common::paint_chip(p, Pos2::new(cx, chip_y), "first snapshot", MUTED, None);
        } else {
            common::paint_chip(p, Pos2::new(cx, chip_y), "vs previous", MUTED, None);
        }

        // note card
        let note_card = Rect::from_min_size(
            Pos2::new(rect.left(), head.bottom() + 12.0),
            Vec2::new(rect.width(), 142.0),
        );
        card(p, note_card, 16, SURF);
        p.text(
            Pos2::new(note_card.left() + 18.0, note_card.top() + 12.0),
            Align2::LEFT_TOP,
            "Note",
            bold(14.0),
            TEXT,
        );
        let dirty = self.timeline.note.trim() != snap.note.trim();
        self.timeline.note_dirty = dirty;
        p.text(
            Pos2::new(note_card.left() + 70.0, note_card.top() + 15.0),
            Align2::LEFT_TOP,
            if !self.timeline.note_status.is_empty() {
                self.timeline.note_status.clone()
            } else if dirty {
                "unsaved changes".into()
            } else {
                String::new()
            },
            regular(12.0),
            MUTED,
        );
        let save_rect = Rect::from_min_size(
            Pos2::new(note_card.right() - 18.0 - 100.0, note_card.top() + 8.0),
            Vec2::new(100.0, 30.0),
        );
        let mut save_clicked = false;
        common::area(ui, save_rect, |ui| {
            if button(ui, "Save note", Btn::Ghost, dirty).clicked() {
                save_clicked = true;
            }
        });
        let editor = Rect::from_min_max(
            Pos2::new(note_card.left() + 18.0, note_card.top() + 40.0),
            Pos2::new(note_card.right() - 18.0, note_card.bottom() - 12.0),
        );
        common::area(ui, editor, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.timeline.note)
                    .hint_text(
                        egui::RichText::new("What happened at this moment? Why did you save it?")
                            .font(regular(13.0))
                            .color(DIM),
                    )
                    .font(regular(13.0))
                    .desired_rows(3)
                    .desired_width(editor.width()),
            );
        });

        // changes card fills the rest
        let changes = Rect::from_min_max(
            Pos2::new(rect.left(), note_card.bottom() + 12.0),
            Pos2::new(rect.right(), rect.bottom() - 46.0),
        );
        card(p, changes, 16, SURF);
        let filter_w: f32 = if changes.width() < 560.0 { 120.0 } else { 180.0 };
        p.text(
            Pos2::new(changes.left() + 18.0, changes.top() + 12.0),
            Align2::LEFT_TOP,
            elide(
                p,
                "Changes in this snapshot",
                &bold(14.0),
                changes.width() - filter_w - 60.0,
            ),
            bold(14.0),
            TEXT,
        );
        if changes.width() >= 620.0 {
            p.text(
                Pos2::new(changes.left() + 184.0, changes.top() + 15.0),
                Align2::LEFT_TOP,
                format!(
                    "{} added · {} modified · {} deleted",
                    d.added.len(),
                    d.modified.len(),
                    d.deleted.len()
                ),
                regular(12.0),
                MUTED,
            );
        }
        let filter_rect = Rect::from_min_size(
            Pos2::new(changes.right() - 18.0 - filter_w, changes.top() + 8.0),
            Vec2::new(filter_w, 28.0),
        );
        common::area(ui, filter_rect, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.timeline.filter)
                    .hint_text(
                        egui::RichText::new("Filter files…")
                            .font(regular(11.5))
                            .color(DIM),
                    )
                    .font(regular(11.5))
                    .desired_width(filter_w),
            );
        });

        let query = self.timeline.filter.trim().to_lowercase();
        let mut rows = store::ordered_changes(&d);
        if !query.is_empty() {
            rows.retain(|(rel, _)| rel.to_lowercase().contains(&query));
        }
        p.text(
            Pos2::new(changes.left() + 18.0, changes.bottom() - 20.0),
            Align2::LEFT_TOP,
            format!(
                "{} unchanged file{} not shown",
                d.same.len(),
                if d.same.len() == 1 { "" } else { "s" }
            ),
            regular(12.0),
            MUTED,
        );
        let list = Rect::from_min_max(
            Pos2::new(changes.left() + 12.0, changes.top() + 40.0),
            Pos2::new(changes.right() - 12.0, changes.bottom() - 26.0),
        );
        let mut restore_file: Option<String> = None;
        common::area(ui, list, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("changes")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if rows.is_empty() {
                        ui.add_space(14.0);
                        ui.label(
                            egui::RichText::new(if query.is_empty() {
                                "No file changes here."
                            } else {
                                "Nothing matches that filter."
                            })
                            .font(regular(13.0))
                            .color(MUTED),
                        );
                        return;
                    }
                    for (rel, kind) in &rows {
                        let old = base.files.get(rel).map(|f| f.size).unwrap_or(0);
                        let new = snap.files.get(rel).map(|f| f.size).unwrap_or(0);
                        let (r, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 42.0),
                            Sense::hover(),
                        );
                        let __painter = ui.painter().with_clip_rect(rect);
    let p = &__painter;
                        p.rect_filled(r, 11, Color32::from_rgb(0x0d, 0x1e, 0x2f));
                        p.rect_stroke(
                            r,
                            11,
                            Stroke::new(1.0, Color32::from_rgb(0x14, 0x31, 0x4a)),
                            StrokeKind::Inside,
                        );
                        let color = status_color(kind);
                        let galley = p.layout_no_wrap(
                            status_label(kind).to_string(),
                            bold(11.0),
                            color,
                        );
                        let pill = Rect::from_min_size(
                            Pos2::new(r.left() + 10.0, r.center().y - 10.0),
                            Vec2::new(galley.size().x + 16.0, 20.0),
                        );
                        p.rect_filled(pill, 8, darker(color, 3.2));
                        p.galley(
                            pill.min + Vec2::new(8.0, (20.0 - galley.size().y) * 0.5),
                            galley,
                            color,
                        );
                        let text_x = pill.right() + 10.0;
                        p.text(
                            Pos2::new(text_x, r.center().y),
                            Align2::LEFT_CENTER,
                            elide(p, rel, &mono(12.0), r.right() - text_x - 120.0),
                            mono(12.0),
                            TEXT,
                        );
                        let (delta, dcolor) = if *kind == "added" {
                            (format!("+{}", store::fmt_bytes(new)), GREEN)
                        } else if *kind == "deleted" {
                            (format!("−{}", store::fmt_bytes(old)), RED)
                        } else if new == old {
                            ("same size".into(), DIM)
                        } else if new > old {
                            (format!("+{}", store::fmt_bytes(new - old)), GREEN)
                        } else {
                            (format!("−{}", store::fmt_bytes(old - new)), RED)
                        };
                        p.text(
                            Pos2::new(r.right() - 56.0, r.center().y),
                            Align2::RIGHT_CENTER,
                            delta,
                            bold(11.0),
                            dcolor,
                        );
                        let btn = Rect::from_center_size(
                            Pos2::new(r.right() - 24.0, r.center().y),
                            Vec2::splat(26.0),
                        );
                        let resp = ui.interact(
                            btn,
                            ui.id().with(("restore", rel)),
                            Sense::click(),
                        );
                        let hovered = resp.hovered();
                        p.rect_filled(
                            btn,
                            7,
                            if hovered { Color32::from_rgb(0x0e, 0x2a, 0x44) } else { Color32::from_rgb(0x10, 0x2a, 0x42) },
                        );
                        p.rect_stroke(
                            btn,
                            7,
                            Stroke::new(
                                1.0,
                                if hovered { ACCENT } else { Color32::from_rgb(0x1c, 0x40, 0x5e) },
                            ),
                            StrokeKind::Inside,
                        );
                        p.text(
                            btn.center(),
                            Align2::CENTER_CENTER,
                            "↺",
                            icon(13.0),
                            if hovered { ACCENT } else { Color32::from_rgb(0xc8, 0xcc, 0xd4) },
                        );
                        if resp.on_hover_text(format!("Restore {} from this snapshot", rel)).clicked() {
                            restore_file = Some(rel.clone());
                        }
                        ui.add_space(6.0);
                    }
                });
        });

        // footer
        let foot = Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() - 38.0),
            Vec2::new(rect.width(), 38.0),
        );
        let mut compare_clicked = false;
        let mut delete_clicked = false;
        let mut restore_clicked = false;
        common::area(ui, foot, |ui| {
            let compact = foot.width() < 640.0;
            let restore_label = if compact { "↺  Restore" } else { "↺  Restore this snapshot" };
            let delete_label = if compact { "Delete" } else { "Delete snapshot" };
            let compare_label = if compact { "⇄  Compare" } else { "⇄  Compare with previous" };
            let restore_w: f32 = if compact { 104.0 } else { 190.0 };
            let delete_w: f32 = if compact { 82.0 } else { 132.0 };
            let compare_w: f32 = if compact { 106.0 } else { 200.0 };
            let restore_rect = Rect::from_min_size(
                Pos2::new(foot.right() - restore_w, foot.top()),
                Vec2::new(restore_w, 38.0),
            );
            let delete_rect = Rect::from_min_size(
                Pos2::new(restore_rect.left() - 8.0 - delete_w, foot.top()),
                Vec2::new(delete_w, 38.0),
            );
            let compare_rect = Rect::from_min_size(
                Pos2::new(foot.left(), foot.top()),
                Vec2::new(compare_w.min(delete_rect.left() - foot.left() - 8.0), 38.0),
            );
            common::area(ui, compare_rect, |ui| {
                if button(ui, compare_label, Btn::Ghost, prev.is_some()).clicked() {
                    compare_clicked = true;
                }
            });
            common::area(ui, delete_rect, |ui| {
                if button(ui, delete_label, Btn::Danger, true).clicked() {
                    delete_clicked = true;
                }
            });
            common::area(ui, restore_rect, |ui| {
                if button(ui, restore_label, Btn::Primary, true).clicked() {
                    restore_clicked = true;
                }
            });
        });

        if save_clicked {
            self.save_note();
        }
        if compare_clicked {
            if let Some(prev) = prev {
                let (a, b) = (prev.id.clone(), snap.id.clone());
                self.page = crate::Page::Compare;
                self.rebuild_compare(Some(a), Some(b));
                return;
            }
        }
        if delete_clicked {
            self.confirm_delete(&snap.id);
        }
        if restore_clicked {
            self.start_restore(snap.id.clone(), true, None);
        }
        if let Some(rel) = restore_file {
            self.start_restore(snap.id.clone(), false, Some(rel));
        }
        let _ = pid;
    }
}
