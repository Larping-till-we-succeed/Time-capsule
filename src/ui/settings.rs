//! Settings page: auto-save configuration as page content, not a dialog.

use egui::{Color32, Pos2, Rect, RichText, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_settings(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("settings_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| self.ui_settings_body(ui));
    }

    fn ui_settings_body(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let rect = Rect::from_min_size(ui.cursor().min, Vec2::new(width, 300.0));
        let __painter = ui.painter().with_clip_rect(rect);
        let p = &__painter;
        card(p, rect, 16, SURF);
        p.text(
            Pos2::new(rect.left() + 20.0, rect.top() + 18.0),
            egui::Align2::LEFT_TOP,
            "AUTOMATIC SNAPSHOTS",
            bold(10.0),
            DIM,
        );
        p.text(
            Pos2::new(rect.left() + 20.0, rect.top() + 38.0),
            egui::Align2::LEFT_TOP,
            "Auto-save",
            bold(17.0),
            TEXT,
        );

        let body = Rect::from_min_max(
            Pos2::new(rect.left() + 20.0, rect.top() + 76.0),
            Pos2::new(rect.right() - 20.0, rect.bottom() - 18.0),
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let ui = &mut child;
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

        let mut enabled = self.settings.enabled;
        ui.horizontal(|ui| {
            if ui
                .checkbox(
                    &mut enabled,
                    RichText::new("Capture snapshots automatically")
                        .font(bold(13.0))
                        .color(Color32::from_rgb(0xcf, 0xdf, 0xf0)),
                )
                .changed()
            {
                self.settings.enabled = enabled;
            }
            ui.add_space(12.0);
            ui.add_enabled(
                self.settings.enabled,
                egui::DragValue::new(&mut self.settings.minutes)
                    .range(1..=1440)
                    .speed(1.0)
                    .suffix(" min"),
            );
        });
        ui.checkbox(
            &mut self.settings.only_changes,
            RichText::new("Skip when the folder has not changed since the last snapshot")
                .font(bold(13.0))
                .color(Color32::from_rgb(0xcf, 0xdf, 0xf0)),
        );
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Automatic snapshots are marked “Auto-save” in the timeline and never replace the moments you capture by hand. They run for every project while TimeCapsule is open.",
                )
                .font(regular(12.5))
                .color(MUTED),
            )
            .wrap(),
        );

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let state = if !self.auto.enabled {
                "Auto-save is off — snapshots are only captured when you ask.".to_string()
            } else {
                let mut text = format!("Auto-save is on · every {} min", self.auto.minutes);
                if let Some(last) = self.last_auto {
                    text.push_str(&format!(" · last capture {}", store::fmt_time(last)));
                }
                text
            };
            ui.add(
                egui::Label::new(RichText::new(state).font(regular(12.0)).color(MUTED)).wrap(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if button(ui, "Save settings", Btn::Primary, true).clicked() {
                    self.save_settings();
                }
                let saved = self
                    .settings
                    .saved_at
                    .map(|t| store::now() - t < 3.0)
                    .unwrap_or(false);
                if saved {
                    ui.label(RichText::new("Saved").font(bold(12.0)).color(GREEN));
                }
            });
        });

        // ---------------------------------------------------- storage maintenance
        let maint = Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() + 14.0),
            Vec2::new(width, 222.0),
        );
        let __mp = ui.painter().with_clip_rect(maint);
        let mp = &__mp;
        card(mp, maint, 16, SURF);
        mp.text(
            Pos2::new(maint.left() + 20.0, maint.top() + 18.0),
            egui::Align2::LEFT_TOP,
            "STORAGE MAINTENANCE",
            bold(10.0),
            DIM,
        );
        mp.text(
            Pos2::new(maint.left() + 20.0, maint.top() + 38.0),
            egui::Align2::LEFT_TOP,
            "Object store",
            bold(17.0),
            TEXT,
        );
        mp.text(
            Pos2::new(maint.left() + 20.0, maint.top() + 66.0),
            egui::Align2::LEFT_TOP,
            elide(
                mp,
                "Verify that every captured file can still be read, and remove blobs no snapshot points at.",
                &regular(12.5),
                maint.width() - 40.0,
            ),
            regular(12.5),
            MUTED,
        );

        let has_project = self.pid.is_some();
        let check_rect = Rect::from_min_size(
            Pos2::new(maint.left() + 20.0, maint.top() + 94.0),
            Vec2::new(148.0, 32.0),
        );
        let mut check_clicked = false;
        common::area(ui, check_rect, |ui| {
            if button(ui, "Check integrity", Btn::Ghost, has_project).clicked() {
                check_clicked = true;
            }
        });
        let scan_rect = Rect::from_min_size(
            Pos2::new(check_rect.right() + 10.0, maint.top() + 94.0),
            Vec2::new(200.0, 32.0),
        );
        let scan_label = match &self.gc_preview {
            Some(report) if report.dry_run && report.removed > 0 => {
                format!("Remove {} unused", report.removed)
            }
            _ => "Scan for unused files".to_string(),
        };
        let mut scan_clicked = false;
        common::area(ui, scan_rect, |ui| {
            if button(ui, &scan_label, Btn::Ghost, has_project).clicked() {
                scan_clicked = true;
            }
        });
        if check_clicked {
            if let Some(pid) = self.pid.clone() {
                self.verify_report = Some(store::verify_store(&pid));
                self.gc_preview = None;
            }
        }
        if scan_clicked {
            if let Some(pid) = self.pid.clone() {
                let dry = self
                    .gc_preview
                    .as_ref()
                    .map(|r| !(r.dry_run && r.removed > 0))
                    .unwrap_or(true);
                if let Ok(report) = store::gc_objects(&pid, dry) {
                    if !dry {
                        self.notify(
                            "Storage cleaned up",
                            format!(
                                "Removed {} unused blob{} and freed {}.",
                                report.removed,
                                if report.removed == 1 { "" } else { "s" },
                                store::fmt_bytes(report.freed)
                            ),
                        );
                        self.verify_report = None;
                    }
                    self.gc_preview = Some(report);
                }
            }
        }

        let mut line = 0usize;
        let mut line_y = maint.top() + 138.0;
        if let Some(report) = &self.verify_report {
            let (text, color) = if report.missing.is_empty() && report.corrupt.is_empty() {
                (
                    format!(
                        "{} blob{} checked — every file is readable{}",
                        report.present,
                        if report.present == 1 { "" } else { "s" },
                        if report.unreferenced > 0 {
                            format!(" · {} unreferenced", report.unreferenced)
                        } else {
                            String::new()
                        }
                    ),
                    GREEN,
                )
            } else {
                (
                    format!(
                        "{} missing · {} corrupt · {} unreferenced",
                        report.missing.len(),
                        report.corrupt.len(),
                        report.unreferenced
                    ),
                    RED,
                )
            };
            mp.text(
                Pos2::new(maint.left() + 20.0, line_y),
                egui::Align2::LEFT_TOP,
                elide(mp, &text, &regular(12.0), maint.width() - 40.0),
                regular(12.0),
                color,
            );
            line += 1;
            line_y += 18.0;
        }
        if let Some(report) = &self.gc_preview {
            let text = if report.dry_run {
                if report.removed == 0 {
                    "Nothing to clean up — every blob belongs to a snapshot.".to_string()
                } else {
                    format!(
                        "{} unused blob{} would be removed and {} freed.",
                        report.removed,
                        if report.removed == 1 { "" } else { "s" },
                        store::fmt_bytes(report.freed)
                    )
                }
            } else {
                format!(
                    "Removed {} unused blob{} and freed {}.",
                    report.removed,
                    if report.removed == 1 { "" } else { "s" },
                    store::fmt_bytes(report.freed)
                )
            };
            mp.text(
                Pos2::new(maint.left() + 20.0, line_y),
                egui::Align2::LEFT_TOP,
                elide(mp, &text, &regular(12.0), maint.width() - 40.0),
                regular(12.0),
                MUTED,
            );
            line += 1;
        }
        if line == 0 {
            mp.text(
                Pos2::new(maint.left() + 20.0, line_y),
                egui::Align2::LEFT_TOP,
                "Checks run locally and never modify snapshots.",
                regular(12.0),
                DIM,
            );
        }
        let _ = line;
        ui.advance_cursor_after_rect(Rect::from_min_max(rect.min, maint.max));
    }
}
