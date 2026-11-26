//! Compare page: pick two moments, read the summary and inspect file diffs.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_compare(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        let bar = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 56.0));
        card(p, bar, 16, SURF);
        p.text(
            Pos2::new(bar.left() + 16.0, bar.center().y),
            Align2::LEFT_CENTER,
            "FROM",
            bold(10.0),
            DIM,
        );
        let combo_w = ((bar.width() - 160.0) / 2.0).clamp(140.0, 340.0);
        let combo_a = Rect::from_min_size(
            Pos2::new(bar.left() + 58.0, bar.top() + 13.0),
            Vec2::new(combo_w, 30.0),
        );
        p.text(
            Pos2::new(combo_a.right() + 14.0, bar.center().y),
            Align2::LEFT_CENTER,
            "TO",
            bold(10.0),
            DIM,
        );
        let combo_b = Rect::from_min_size(
            Pos2::new(combo_a.right() + 44.0, bar.top() + 13.0),
            Vec2::new(combo_w, 30.0),
        );
        let swap_rect = Rect::from_min_size(
            Pos2::new(combo_b.right() + 8.0, bar.top() + 11.0),
            Vec2::new(34.0, 34.0),
        );

        let snap_label = |s: &store::Snapshot| {
            let tail = if !s.title.is_empty() {
                s.title.clone()
            } else if !s.note.trim().is_empty() {
                s.note.trim().to_string()
            } else {
                s.id.chars().take(6).collect()
            };
            format!("{}  ·  {}", store::fmt_stamp(s.ts), tail)
        };
        let chain = self.compare.chain.clone();
        let mut pick_a: Option<String> = None;
        let mut pick_b: Option<String> = None;
        common::area(ui, combo_a, |ui| {
            let selected = self
                .compare
                .a
                .as_ref()
                .and_then(|id| chain.iter().find(|s| &s.id == id))
                .map(snap_label)
                .unwrap_or_else(|| "—".into());
            egui::ComboBox::from_id_salt("cmp_a")
                .selected_text(selected)
                .width(combo_w)
                .show_ui(ui, |ui| {
                    for s in &chain {
                        if ui
                            .selectable_label(
                                self.compare.a.as_deref() == Some(s.id.as_str()),
                                snap_label(s),
                            )
                            .clicked()
                        {
                            pick_a = Some(s.id.clone());
                        }
                    }
                });
        });
        common::area(ui, combo_b, |ui| {
            let selected = self
                .compare
                .b
                .as_ref()
                .and_then(|id| chain.iter().find(|s| &s.id == id))
                .map(snap_label)
                .unwrap_or_else(|| "—".into());
            egui::ComboBox::from_id_salt("cmp_b")
                .selected_text(selected)
                .width(combo_w)
                .show_ui(ui, |ui| {
                    for s in &chain {
                        if ui
                            .selectable_label(
                                self.compare.b.as_deref() == Some(s.id.as_str()),
                                snap_label(s),
                            )
                            .clicked()
                        {
                            pick_b = Some(s.id.clone());
                        }
                    }
                });
        });
        let swap = ui.interact(swap_rect, ui.id().with("swap"), Sense::click());
        let p = &ui.painter().clone();
        p.rect_filled(
            swap_rect,
            10,
            if swap.hovered() { Color32::from_rgb(0x16, 0x34, 0x4d) } else { SURF2 },
        );
        p.rect_stroke(
            swap_rect,
            10,
            Stroke::new(1.0, BORDER2),
            StrokeKind::Inside,
        );
        p.text(
            swap_rect.center(),
            Align2::CENTER_CENTER,
            "⇄",
            icon(15.0),
            TEXT,
        );
        if let Some(id) = pick_a {
            self.compare.a = Some(id);
            self.load_compare_diff();
        }
        if let Some(id) = pick_b {
            self.compare.b = Some(id);
            self.load_compare_diff();
        }
        if swap.clicked() {
            std::mem::swap(&mut self.compare.a, &mut self.compare.b);
            self.load_compare_diff();
        }

        // stat cards
        let stats_y = bar.bottom() + 12.0;
        let stat_w = (rect.width() - 36.0) / 4.0;
        let (added, modified, deleted, same, add_bytes, mod_bytes, del_bytes, files_in, files_out) =
            if let Some(d) = &self.compare.diff {
                let b = self.compare.b.as_ref().and_then(|id| {
                    self.compare.chain.iter().find(|s| &s.id == id)
                });
                let a = self.compare.a.as_ref().and_then(|id| {
                    self.compare.chain.iter().find(|s| &s.id == id)
                });
                let add_bytes: u64 = b
                    .map(|s| d.added.iter().filter_map(|p| s.files.get(p)).map(|f| f.size).sum())
                    .unwrap_or(0);
                let mod_bytes: u64 = b
                    .map(|s| d.modified.iter().filter_map(|p| s.files.get(p)).map(|f| f.size).sum())
                    .unwrap_or(0);
                let del_bytes: u64 = a
                    .map(|s| d.deleted.iter().filter_map(|p| s.files.get(p)).map(|f| f.size).sum())
                    .unwrap_or(0);
                (
                    d.added.len(),
                    d.modified.len(),
                    d.deleted.len(),
                    d.same.len(),
                    add_bytes,
                    mod_bytes,
                    del_bytes,
                    a.map(|s| s.files.len()).unwrap_or(0),
                    b.map(|s| s.files.len()).unwrap_or(0),
                )
            } else {
                (0, 0, 0, 0, 0, 0, 0, 0, 0)
            };
        let cards = [
            ("Added", added, "＋", GREEN, store::fmt_bytes(add_bytes)),
            ("Modified", modified, "~", ACCENT, store::fmt_bytes(mod_bytes)),
            ("Deleted", deleted, "−", RED, store::fmt_bytes(del_bytes)),
            ("Unchanged", same, "=", MUTED, format!("{} → {} files", files_in, files_out)),
        ];
        for (i, (name, value, glyph, color, hint)) in cards.iter().enumerate() {
            common::stat_card(
                ui,
                Rect::from_min_size(
                    Pos2::new(rect.left() + i as f32 * (stat_w + 12.0), stats_y),
                    Vec2::new(stat_w, 104.0),
                ),
                &value.to_string(),
                name,
                glyph,
                *color,
                "",
                0,
                &[],
                hint,
            );
        }

        // body
        let body = Rect::from_min_max(
            Pos2::new(rect.left(), stats_y + 104.0 + 16.0),
            rect.max,
        );
        let left_w = 480.0_f32.min(body.width() * 0.45);
        let left = Rect::from_min_size(body.min, Vec2::new(left_w, body.height()));
        let right = Rect::from_min_max(
            Pos2::new(left.right() + 16.0, body.top()),
            body.max,
        );
        self.paint_compare_list(ui, left);
        self.paint_diff(ui, right);
    }

    fn paint_compare_list(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, rect, 16, SURF);
        let search = Rect::from_min_max(
            Pos2::new(rect.left() + 14.0, rect.top() + 12.0),
            Pos2::new(rect.right() - 14.0, rect.top() + 40.0),
        );
        common::area(ui, search, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.compare.filter)
                    .hint_text(
                        egui::RichText::new("Filter changed files…")
                            .font(regular(12.0))
                            .color(DIM),
                    )
                    .font(regular(12.0))
                    .desired_width(search.width()),
            );
        });
        let rows: Vec<(String, &'static str)> = match &self.compare.diff {
            Some(d) => {
                let q = self.compare.filter.trim().to_lowercase();
                store::ordered_changes(d)
                    .into_iter()
                    .filter(|(rel, _)| q.is_empty() || rel.to_lowercase().contains(&q))
                    .collect()
            }
            None => Vec::new(),
        };
        let list = Rect::from_min_max(
            Pos2::new(rect.left() + 10.0, rect.top() + 48.0),
            Pos2::new(rect.right() - 10.0, rect.bottom() - 10.0),
        );
        let mut pick: Option<String> = None;
        common::area(ui, list, |ui| {
            if self.compare.diff.is_none() {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("Add at least two snapshots to compare.")
                        .font(regular(13.0))
                        .color(MUTED),
                );
                return;
            }
            if rows.is_empty() {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("No differences between these two snapshots.")
                        .font(regular(13.0))
                        .color(MUTED),
                );
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("cmp_rows")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (rel, kind) in &rows {
                        let (r, response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 40.0),
                            Sense::click(),
                        );
                        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
                        let active = self.compare.active.as_deref() == Some(rel.as_str());
                        if active {
                            p.rect_filled(r, 9, SURF2);
                            p.rect_stroke(
                                r,
                                9,
                                Stroke::new(1.0, Color32::from_rgb(0x1d, 0x5c, 0x8c)),
                                StrokeKind::Inside,
                            );
                        } else if response.hovered() {
                            p.rect_filled(r, 9, Color32::from_rgb(0x0c, 0x20, 0x33));
                        }
                        let color = status_color(kind);
                        let galley = p.layout_no_wrap(
                            status_label(kind).to_string(),
                            bold(11.0),
                            color,
                        );
                        let pill = Rect::from_min_size(
                            Pos2::new(r.left() + 8.0, r.center().y - 10.0),
                            Vec2::new(galley.size().x + 16.0, 20.0),
                        );
                        p.rect_filled(pill, 8, darker(color, 3.2));
                        p.galley(
                            pill.min + Vec2::new(8.0, (20.0 - galley.size().y) * 0.5),
                            galley,
                            color,
                        );
                        p.text(
                            Pos2::new(pill.right() + 9.0, r.center().y),
                            Align2::LEFT_CENTER,
                            elide(p, rel, &mono(12.0), r.right() - pill.right() - 40.0),
                            mono(12.0),
                            TEXT,
                        );
                        p.text(
                            Pos2::new(r.right() - 12.0, r.center().y),
                            Align2::RIGHT_CENTER,
                            "›",
                            regular(14.0),
                            DIM,
                        );
                        if response.clicked() {
                            pick = Some(rel.clone());
                        }
                        ui.add_space(1.0);
                    }
                });
        });
        if let Some(rel) = pick {
            let pid = self.pid.clone();
            let a = self
                .compare
                .a
                .as_ref()
                .and_then(|id| self.compare.chain.iter().find(|s| &s.id == id))
                .cloned();
            let b = self
                .compare
                .b
                .as_ref()
                .and_then(|id| self.compare.chain.iter().find(|s| &s.id == id))
                .cloned();
            if let (Some(pid), Some(a), Some(b)) = (pid, a, b) {
                self.show_compare_file(&pid, &a, &b, &rel);
            }
        }
    }

    fn paint_diff(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, rect, 16, SURF);
        let name = self
            .compare
            .active
            .clone()
            .unwrap_or_else(|| "Select a file".into());
        p.text(
            Pos2::new(rect.left() + 16.0, rect.top() + 12.0),
            Align2::LEFT_TOP,
            elide(p, &name, &mono(12.0), rect.width() - 200.0),
            mono(12.0),
            TEXT,
        );
        p.text(
            Pos2::new(rect.right() - 16.0, rect.top() + 12.0),
            Align2::RIGHT_TOP,
            &self.compare.line_meta,
            regular(12.0),
            MUTED,
        );
        let body = Rect::from_min_max(
            Pos2::new(rect.left() + 12.0, rect.top() + 36.0),
            Pos2::new(rect.right() - 12.0, rect.bottom() - 10.0),
        );
        if self.compare.active.is_none() {
            p.text(
                body.center(),
                Align2::CENTER_CENTER,
                &self.compare.empty_reason,
                regular(12.5),
                MUTED,
            );
            return;
        }
        if self.compare.binary {
            p.text(
                body.center(),
                Align2::CENTER_CENTER,
                "This is a binary file, so it cannot be shown as a text diff.",
                regular(12.5),
                MUTED,
            );
            return;
        }
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let ui = &mut child;
        egui::ScrollArea::both()
            .id_salt("diff_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.compare.diff_lines.is_empty() {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("The file content is unchanged.")
                            .font(regular(13.0))
                            .color(MUTED),
                    );
                    return;
                }
                for (kind, text) in &self.compare.diff_lines {
                    let (color, bg) = match kind {
                        '+' => (GREEN, Color32::from_rgb(0x10, 0x23, 0x1a)),
                        '-' => (RED, Color32::from_rgb(0x2a, 0x14, 0x17)),
                        '@' => (BLUE, Color32::from_rgb(0x0a, 0x20, 0x33)),
                        _ => (Color32::from_rgb(0x8f, 0xa6, 0xc0), Color32::TRANSPARENT),
                    };
                    let galley = ui.painter().layout_no_wrap(
                        text.clone(),
                        mono(12.0),
                        color,
                    );
                    let (r, _) = ui.allocate_exact_size(
                        Vec2::new(galley.size().x + 10.0, galley.size().y + 2.0),
                        Sense::hover(),
                    );
                    if bg != Color32::TRANSPARENT {
                        ui.painter().rect_filled(r, 0, bg);
                    }
                    ui.painter().galley(
                        r.min + Vec2::new(5.0, 1.0),
                        galley,
                        color,
                    );
                }
            });
    }
}
