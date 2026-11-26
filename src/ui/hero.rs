//! The banner at the top: planet limb, glowing brand, search and actions.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_hero(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 62.0), Sense::hover());
        let __painter = ui.painter().clone();
        let p = &__painter;

        // menu button
        let menu_rect =
            Rect::from_min_size(Pos2::new(rect.left(), rect.top() + 13.0), Vec2::splat(36.0));
        let menu = ui.interact(menu_rect, ui.id().with("menu"), Sense::click());
        if menu.hovered() {
            p.rect_filled(menu_rect, 9, SURF2);
        }
        let cx = menu_rect.center().x;
        let cy = menu_rect.center().y;
        for i in 0..3 {
            let y = cy - 6.0 + i as f32 * 6.0;
            p.line_segment(
                [Pos2::new(cx - 8.0, y), Pos2::new(cx + 8.0, y)],
                Stroke::new(1.7, if menu.hovered() { TEXT } else { MUTED }),
            );
        }
        if menu.on_hover_text("Toggle navigation · Ctrl + B").clicked() {
            self.sidebar_open = !self.sidebar_open;
        }

        // page title, with the project it belongs to
        let title = self.page_title();
        let galley = p.layout_no_wrap(title.to_string(), bold(23.0), TEXT);
        let title_pos = Pos2::new(rect.left() + 48.0, rect.center().y - galley.size().y * 0.5);
        p.galley(title_pos, galley.clone(), TEXT);
        let after_title = title_pos.x + galley.size().x + 12.0;
        if let Some(proj) = self.project() {
            let sub = format!("·  {}", proj.name);
            let sub_galley = p.layout_no_wrap(sub, regular(12.5), MUTED);
            if after_title + sub_galley.size().x < rect.right() - 600.0 {
                p.galley(
                    Pos2::new(after_title, rect.center().y - sub_galley.size().y * 0.5),
                    sub_galley,
                    MUTED,
                );
            }
        }

        // right side: settings, new snapshot, new project, search
        let gear_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 36.0, rect.top() + 13.0),
            Vec2::splat(36.0),
        );
        let gear = ui.interact(gear_rect, ui.id().with("top_gear"), Sense::click());
        if gear.hovered() {
            p.rect_filled(gear_rect, 9, SURF2);
        }
        p.text(
            gear_rect.center(),
            Align2::CENTER_CENTER,
            "⚙",
            icon(15.0),
            if gear.hovered() { TEXT } else { MUTED },
        );
        if gear.on_hover_text("Settings").clicked() {
            self.select_page(crate::Page::Settings);
        }

        let snap_w = 152.0;
        let snap_rect = Rect::from_min_size(
            Pos2::new(gear_rect.left() - 10.0 - snap_w, rect.top() + 15.0),
            Vec2::new(snap_w, 34.0),
        );
        let mut new_snap = false;
        common::area(ui, snap_rect, |ui| {
            if button(ui, "＋  New snapshot", Btn::Primary, self.pid.is_some()).clicked() {
                new_snap = true;
            }
        });
        if new_snap {
            self.dialog_new_snapshot();
        }

        let proj_w = 128.0;
        let proj_rect = Rect::from_min_size(
            Pos2::new(snap_rect.left() - 8.0 - proj_w, rect.top() + 15.0),
            Vec2::new(proj_w, 34.0),
        );
        let show_proj = proj_rect.left() - (rect.left() + 320.0) > 110.0;
        let mut new_project = false;
        if show_proj {
            common::area(ui, proj_rect, |ui| {
                if button(ui, "＋  Project", Btn::Ghost, true).clicked() {
                    new_project = true;
                }
            });
        }
        if new_project {
            self.dialog_new_project();
        }

        let search_right = if show_proj {
            proj_rect.left() - 10.0
        } else {
            snap_rect.left() - 10.0
        };
        let min_left = rect.left() + 48.0 + galley.size().x + 240.0;
        let search_w = (search_right - min_left).clamp(150.0, 290.0);
        let search_rect = Rect::from_min_size(
            Pos2::new(search_right - search_w, rect.top() + 15.0),
            Vec2::new(search_w, 34.0),
        );
        p.rect_filled(search_rect, 9, SURF);
        p.rect_stroke(search_rect, 9, Stroke::new(1.0, BORDER), StrokeKind::Inside);
        p.text(
            Pos2::new(search_rect.left() + 12.0, search_rect.center().y),
            Align2::LEFT_CENTER,
            "⌕",
            icon(15.0),
            DIM,
        );
        let edit_rect = Rect::from_min_max(
            Pos2::new(search_rect.left() + 30.0, search_rect.top() + 6.0),
            Pos2::new(search_rect.right() - 10.0, search_rect.bottom() - 6.0),
        );
        common::area(ui, edit_rect, |ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(
                        egui::RichText::new("Search")
                            .font(regular(12.5))
                            .color(DIM),
                    )
                    .font(regular(12.5))
                    .desired_width(edit_rect.width())
                    .frame(false)
                    .margin(Vec2::new(0.0, 3.0)),
            );
            if self.focus_search {
                response.request_focus();
                self.focus_search = false;
            }
            if response.changed() {
                self.rebuild_dashboard();
            }
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.select_page(crate::Page::Dashboard);
            }
        });

        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left() - 24.0, rect.bottom() - 1.0),
                Vec2::new(rect.width() + 48.0, 1.0),
            ),
            0,
            BORDER,
        );
        ui.advance_cursor_after_rect(rect);
    }

    pub fn ui_project_head(&mut self, ui: &mut egui::Ui) {
        let Some((name, path, count, latest, storage)) = self.project_head.clone() else {
            return;
        };
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 62.0),
            Sense::hover(),
        );
        let __painter = ui.painter().clone();
    let p = &__painter;
        p.text(
            Pos2::new(rect.left() + 2.0, rect.top() + 2.0),
            Align2::LEFT_TOP,
            &name,
            bold(22.0),
            TEXT,
        );
        p.text(
            Pos2::new(rect.left() + 2.0, rect.top() + 33.0),
            Align2::LEFT_TOP,
            &path,
            regular(12.5),
            MUTED,
        );
        let chip_y = rect.top() + 48.0;
        let mut x = rect.left() + 2.0;
        let chips: Vec<(String, Color32, Option<Color32>)> = {
            let mut v = vec![(format!("{} snapshots", count), ACCENT, Some(Color32::from_rgb(0x0e, 0x33, 0x50)))];
            if count > 0 {
                v.push((latest, MUTED, None));
                v.push((storage, MUTED, None));
            } else {
                v.push(("no snapshots yet".into(), MUTED, None));
            }
            v
        };
        for (text, color, bg) in chips {
            let font = FontId::new(11.5, bold_family());
            let galley = p.layout_no_wrap(text.clone(), font, color);
            let w = galley.size().x + 18.0;
            let chip_rect = Rect::from_min_size(Pos2::new(x, chip_y), Vec2::new(w, 22.0));
            p.rect_filled(chip_rect, 8, bg.unwrap_or(SURF2));
            p.rect_stroke(
                chip_rect,
                8,
                Stroke::new(1.0, darker(color, 3.4)),
                StrokeKind::Inside,
            );
            p.galley(
                chip_rect.min + Vec2::new(9.0, (22.0 - galley.size().y) * 0.5),
                galley,
                color,
            );
            x += w + 8.0;
        }
    }

    pub fn ui_welcome(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let mut clicked = false;
        common::area(ui, rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((rect.height() * 0.22).max(40.0));
                let (r, _) = ui.allocate_exact_size(Vec2::splat(84.0), Sense::hover());
                ui.painter().rect_filled(r, 26, Color32::from_rgb(0x1c, 0x25, 0x32));
                ui.painter().rect_stroke(r, 26, Stroke::new(1.0, BORDER2), StrokeKind::Inside);
                ui.painter().text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    "⏳",
                    icon(36.0),
                    Color32::from_rgb(0xd0, 0xdc, 0xea),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("A time machine for your projects")
                        .font(bold(19.0))
                        .color(TEXT),
                );
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(
                            "Create a project for a folder, capture snapshots whenever something matters, and travel back to any moment — whole projects or single files.",
                        )
                        .font(regular(13.5))
                        .color(MUTED),
                    )
                    .wrap()
                    .halign(egui::Align::Center),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space((rect.width() - 210.0).max(0.0) * 0.5);
                    if button(ui, "Create your first project", Btn::Primary, true).clicked() {
                        clicked = true;
                    }
                });
            });
        });
        if clicked {
            self.dialog_new_project();
        }
    }

    pub fn ui_statusbar(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink2(Vec2::new(18.0, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let ui = &mut child;
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);
        let busy = self.busy();
        ui.painter().circle_filled(
            Pos2::new(ui.min_rect().left() + 4.0, rect.center().y),
            4.0,
            if busy { ACCENT } else { GREEN },
        );
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(if busy { "Working" } else { "Ready" })
                .font(bold(12.0))
                .color(Color32::from_rgb(0xd7, 0xe8, 0xf8)),
        );
        ui.add_space(16.0);
        ui.label(egui::RichText::new("Auto snapshots:").font(regular(12.0)).color(MUTED));
        if common::toggle(ui, self.auto.enabled).clicked() {
            let on = !self.auto.enabled;
            self.set_autosave(on);
            return;
        }
        ui.label(
            egui::RichText::new(if self.auto.enabled { "ON" } else { "OFF" })
                .font(bold(12.0))
                .color(Color32::from_rgb(0xd7, 0xe8, 0xf8)),
        );
        ui.add_space(12.0);
        ui.label(egui::RichText::new("◷").font(icon(13.0)).color(DIM));
        let next = if !self.auto.enabled {
            "Next snapshot: paused".to_string()
        } else {
            let remaining = self
                .auto_deadline
                .map(|d| (d - store::now()).max(0.0))
                .unwrap_or(self.auto.minutes as f64 * 60.0);
            let mins = remaining / 60.0;
            if mins >= 60.0 {
                format!("Next snapshot: in {}h {}m", (mins / 60.0) as i64, (mins % 60.0) as i64)
            } else {
                format!("Next snapshot: in {} min", (mins.round() as i64).max(1))
            }
        };
        ui.label(egui::RichText::new(next).font(regular(12.0)).color(MUTED));

        let running = self.task.as_ref().map(|task| {
            let (done, total) = task.counts();
            (task.label.clone(), done, total)
        });
        if let Some((label, done, total)) = running {
            ui.add_space(14.0);
            let glyph = ["◐", "◓", "◑", "◒"][((store::now() * 4.0) as usize) % 4];
            let text = if total > 0 {
                format!("{} {} · {}/{}", glyph, label, done, total)
            } else {
                format!("{} {}", glyph, label)
            };
            ui.label(egui::RichText::new(text).font(bold(12.0)).color(ACCENT));
            if button(ui, "Cancel", Btn::Ghost, true).clicked() {
                self.cancel_task();
            }
        }

        // right side
        let right = ui.available_rect_before_wrap().right();
        let gear_rect = Rect::from_min_size(
            Pos2::new(right - 30.0, rect.center().y - 15.0),
            Vec2::splat(30.0),
        );
        let storage_rect = Rect::from_min_size(
            Pos2::new(right - 30.0 - 12.0 - 120.0 - 10.0, rect.center().y - 3.0),
            Vec2::new(120.0, 6.0),
        );
        let (used, captured) = self.status_storage;
        let text = if self.pid.is_some() {
            format!("{} / {}", store::fmt_bytes(used), store::fmt_bytes(captured))
        } else {
            "No project storage yet".into()
        };
        ui.painter().text(
            Pos2::new(storage_rect.left() - 10.0, rect.center().y),
            Align2::RIGHT_CENTER,
            text,
            regular(12.0),
            MUTED,
        );
        let frac = if captured == 0 {
            0.0
        } else {
            (used as f32 / captured as f32).clamp(0.02, 1.0)
        };
        common::mini_bar(
            ui.painter(),
            storage_rect,
            frac,
            Color32::from_rgb(0x3b, 0x82, 0xf6),
        );

        let response = ui.interact(gear_rect, ui.id().with("gear"), Sense::click());
        let hovered = response.hovered();
        if hovered {
            ui.painter().rect_filled(gear_rect, 8, SURF2);
        }
        ui.painter().text(
            gear_rect.center(),
            Align2::CENTER_CENTER,
            "⚙",
            icon(14.0),
            if hovered { TEXT } else { MUTED },
        );
        if response.clicked() {
            self.select_page(crate::Page::Settings);
        }
    }
}
