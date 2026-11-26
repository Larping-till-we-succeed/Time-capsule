//! Left navigation: brand mark, workspace pages, projects and storage.

use egui::{Align2, Color32, Pos2, Rect, Sense, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_sidebar(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let bottom_h = 132.0;
        let top_rect = Rect::from_min_max(
            rect.min,
            Pos2::new(rect.right(), rect.bottom() - bottom_h - 10.0),
        );
        common::area(ui, top_rect, |ui| self.sidebar_top(ui));
        let bottom = Rect::from_min_max(
            Pos2::new(rect.left(), rect.bottom() - bottom_h),
            rect.max,
        );
        common::area(ui, bottom, |ui| self.sidebar_bottom(ui));
    }

    fn sidebar_top(&mut self, ui: &mut egui::Ui) {
        let (head, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 58.0), Sense::hover());
        let p = ui.painter().clone();
        let avatar = Rect::from_center_size(
            Pos2::new(head.left() + 19.0, head.center().y),
            Vec2::splat(38.0),
        );
        p.circle_filled(avatar.center(), 19.0, Color32::from_rgb(0xe7, 0xea, 0xf3));
        let initial = self
            .project()
            .and_then(|pr| pr.name.chars().next())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "T".into());
        p.text(
            avatar.center(),
            Align2::CENTER_CENTER,
            initial,
            bold(16.0),
            Color32::from_rgb(0x1b, 0x23, 0x35),
        );
        let name = self
            .project()
            .map(|pr| pr.name)
            .unwrap_or_else(|| "TimeCapsule".into());
        p.text(
            Pos2::new(avatar.right() + 11.0, head.center().y - 9.0),
            Align2::LEFT_CENTER,
            elide(&p, &name, &bold(13.0), head.width() - 64.0),
            bold(13.0),
            TEXT,
        );
        p.text(
            Pos2::new(avatar.right() + 11.0, head.center().y + 10.0),
            Align2::LEFT_CENTER,
            "TimeCapsule",
            regular(11.0),
            DIM,
        );

        ui.add_space(8.0);
        ui.label(egui::RichText::new("WORKSPACE").font(bold(9.5)).color(DIM));
        ui.add_space(4.0);
        for (key, glyph, name) in [
            ("dashboard", "⌂", "Dashboard"),
            ("timeline", "◷", "Timeline"),
            ("compare", "⚖", "Compare"),
            ("stats", "▤", "Statistics"),
        ] {
            let active = self.page.key() == key;
            if common::nav_row(ui, glyph, name, active).clicked() {
                let page = match key {
                    "dashboard" => crate::Page::Dashboard,
                    "timeline" => crate::Page::Timeline,
                    "compare" => crate::Page::Compare,
                    _ => crate::Page::Stats,
                };
                self.select_page(page);
            }
            ui.add_space(1.0);
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("PROJECTS").font(bold(9.5)).color(DIM));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
                let p = ui.painter().clone();
                p.rect_filled(
                    rect,
                    7,
                    if response.hovered() {
                        SURF2
                    } else {
                        Color32::TRANSPARENT
                    },
                );
                p.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "＋",
                    icon(13.0),
                    if response.hovered() { ACCENT } else { MUTED },
                );
                if response.on_hover_text("Add a project folder").clicked() {
                    self.dialog_new_project();
                }
            });
        });
        ui.add_space(4.0);
        let list_height = 200.0_f32.min(ui.available_height() - 64.0).max(56.0);
        ui.allocate_ui(Vec2::new(ui.available_width(), list_height), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("projects")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if self.projects.is_empty() {
                        ui.label(
                            egui::RichText::new("No projects yet.\nAdd a folder to start.")
                                .font(regular(12.0))
                                .color(MUTED),
                        );
                    }
                    let projects = self.projects.clone();
                    let current = self.pid.clone();
                    for proj in projects {
                        let count = *self.overview.counts.get(&proj.id).unwrap_or(&0);
                        let active = current.as_deref() == Some(proj.id.as_str());
                        let response = project_row(ui, &proj, count, active);
                        let mut wanted: Option<bool> = None;
                        response.context_menu(|ui| {
                            if ui.button("Rename project…").clicked() {
                                wanted = Some(true);
                                ui.close_menu();
                            }
                            if ui.button("Remove project…").clicked() {
                                wanted = Some(false);
                                ui.close_menu();
                            }
                        });
                        match wanted {
                            Some(true) => {
                                self.dialog = Some(crate::Dialog::RenameProject {
                                    pid: proj.id.clone(),
                                    name: proj.name.clone(),
                                })
                            }
                            Some(false) => {
                                self.dialog = Some(crate::Dialog::ConfirmProject {
                                    pid: proj.id.clone(),
                                    name: proj.name.clone(),
                                })
                            }
                            None => {}
                        }
                        if response.clicked() {
                            self.select_project(&proj.id);
                        }
                        ui.add_space(2.0);
                    }
                });
        });

        ui.add_space(6.0);
        if common::nav_row(ui, "⚙", "Settings", self.page == crate::Page::Settings).clicked() {
            self.select_page(crate::Page::Settings);
        }
    }

    fn sidebar_bottom(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let p = ui.painter().clone();
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.top()),
                Vec2::new(rect.width(), 1.0),
            ),
            0,
            BORDER,
        );
        p.text(
            Pos2::new(rect.left(), rect.top() + 18.0),
            Align2::LEFT_TOP,
            "STORAGE",
            bold(9.5),
            DIM,
        );
        let used = self.overview.used;
        let captured = self.status_storage.1.max(used);
        p.text(
            Pos2::new(rect.left(), rect.top() + 34.0),
            Align2::LEFT_TOP,
            format!(
                "{} of {} captured",
                store::fmt_bytes(used),
                store::fmt_bytes(captured)
            ),
            regular(11.5),
            MUTED,
        );
        let bar = Rect::from_min_size(
            Pos2::new(rect.left(), rect.top() + 54.0),
            Vec2::new(rect.width(), 5.0),
        );
        common::mini_bar(
            &p,
            bar,
            if captured == 0 {
                0.0
            } else {
                (used as f32 / captured as f32).clamp(0.0, 1.0)
            },
            ACCENT,
        );
        p.text(
            Pos2::new(rect.left(), rect.bottom()),
            Align2::LEFT_BOTTOM,
            if self.busy() { "Working…" } else { "Ready" },
            bold(11.0),
            if self.busy() { ACCENT } else { GREEN },
        );
        p.text(
            Pos2::new(rect.right(), rect.bottom()),
            Align2::RIGHT_BOTTOM,
            "TimeCapsule 0.0",
            regular(10.0),
            DIM,
        );
    }
}

fn project_row(
    ui: &mut egui::Ui,
    proj: &store::Project,
    count: usize,
    active: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 52.0), Sense::click());
    let p = ui.painter().clone();
    let hovered = response.hovered();
    let fill = if active {
        SURF2
    } else if hovered {
        alpha(SURF2, 140)
    } else {
        Color32::TRANSPARENT
    };
    p.rect_filled(rect, 10, fill);
    if active {
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.center().y - 9.0),
                Vec2::new(3.0, 18.0),
            ),
            2,
            ACCENT,
        );
    }
    p.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 11.0),
        Align2::LEFT_TOP,
        elide(&p, &proj.name, &bold(12.5), rect.width() - 62.0),
        bold(12.5),
        if active { TEXT } else { MUTED },
    );
    p.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 30.0),
        Align2::LEFT_TOP,
        elide(&p, &proj.path, &regular(10.5), rect.width() - 74.0),
        regular(10.5),
        DIM,
    );
    p.text(
        Pos2::new(rect.right() - 10.0, rect.center().y),
        Align2::RIGHT_CENTER,
        count.to_string(),
        bold(11.0),
        if active { ACCENT } else { DIM },
    );
    response.on_hover_text(&proj.path)
}
