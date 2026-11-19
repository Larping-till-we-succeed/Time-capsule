//! Settings page: auto-save configuration as page content, not a dialog.

use egui::{Color32, Pos2, Rect, RichText, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;


impl App {
    pub fn ui_settings(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let rect = Rect::from_min_size(
            ui.available_rect_before_wrap().min,
            Vec2::new(width, 300.0),
        );
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
    }
}
