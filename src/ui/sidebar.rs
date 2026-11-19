//! Left sidebar: brand, project switcher, navigation and the time-machine mark.

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

impl App {
    pub fn ui_sidebar(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let bottom_h = 236.0;
        let top_rect = Rect::from_min_max(
            rect.min,
            Pos2::new(rect.right(), rect.bottom() - bottom_h - 8.0),
        );
        common::area(ui, top_rect, |ui| self.sidebar_top(ui));
        let bottom = Rect::from_min_max(
            Pos2::new(rect.left(), rect.bottom() - bottom_h),
            rect.max,
        );
        common::area(ui, bottom, |ui| self.sidebar_bottom(ui));
    }

    fn sidebar_top(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
            common::clock_logo(ui.painter(), r);
            ui.add_space(1.0);
            ui.label(egui::RichText::new("TimeCapsule").font(bold(18.0)).color(TEXT));
        });
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("PROJECTS").font(bold(10.0)).color(DIM));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                let __painter = ui.painter().clone();
    let p = &__painter;
                let hovered = response.hovered();
                p.rect_filled(
                    rect,
                    7,
                    if hovered {
                        Color32::from_rgb(0x0e, 0x2a, 0x44)
                    } else {
                        Color32::from_rgb(0x0d, 0x24, 0x38)
                    },
                );
                p.rect_stroke(
                    rect,
                    7,
                    Stroke::new(
                        1.0,
                        if hovered {
                            ACCENT
                        } else {
                            Color32::from_rgb(0x1c, 0x40, 0x5e)
                        },
                    ),
                    StrokeKind::Inside,
                );
                p.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "＋",
                    icon(13.0),
                    if hovered { ACCENT } else { Color32::from_rgb(0xc8, 0xcc, 0xd4) },
                );
                if response.on_hover_text("Add a project folder").clicked() {
                    self.dialog_new_project();
                }
            });
        });

        ui.add_space(6.0);
        let list_height = 216.0_f32.min(ui.available_height() - 250.0).max(72.0);
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
                        if project_row(ui, &proj, count, active).clicked() {
                            self.select_project(&proj.id);
                        }
                        ui.add_space(6.0);
                    }
                });
        });

        ui.add_space(8.0);
        crate::theme::hline(ui, egui::Color32::from_rgb(0x0f, 0x24, 0x37));
        ui.add_space(6.0);
        let views = [
            ("dashboard", "⌂", "Dashboard"),
            ("timeline", "◷", "Timeline"),
            ("compare", "⚖", "Compare"),
            ("stats", "▤", "Statistics"),
        ];
        for (key, glyph, name) in views {
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
        }
        ui.add_space(4.0);
        crate::theme::hline(ui, egui::Color32::from_rgb(0x0f, 0x24, 0x37));
        ui.add_space(4.0);
        if common::nav_row(ui, "⚙", "Settings", self.page == crate::Page::Settings).clicked() {
            self.select_page(crate::Page::Settings);
        }
    }

    fn sidebar_bottom(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let art = Rect::from_min_size(
            Pos2::new(rect.center().x - 48.0, rect.top()),
            Vec2::splat(96.0),
        );
        common::hourglass(ui.painter(), art);

        let name_y = art.bottom() + 6.0;
        ui.painter().text(
            Pos2::new(rect.center().x, name_y),
            Align2::CENTER_TOP,
            "TimeCapsule",
            bold(15.0),
            ACCENT,
        );
        ui.painter().text(
            Pos2::new(rect.center().x, name_y + 22.0),
            Align2::CENTER_TOP,
            "Travel through time.\nNever lose progress.",
            regular(12.0),
            MUTED,
        );
        let storage = format!("Project storage · {}", store::fmt_bytes(self.overview.used));
        ui.painter().text(
            Pos2::new(rect.center().x, rect.bottom() - 26.0),
            Align2::CENTER_BOTTOM,
            &storage,
            bold(10.0),
            DIM,
        );
        ui.painter().text(
            Pos2::new(rect.center().x, rect.bottom()),
            Align2::CENTER_BOTTOM,
            "TimeCapsule 0.0",
            regular(9.0),
            Color32::from_rgb(0x3d, 0x51, 0x69),
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
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 70.0), Sense::click());
    let __painter = ui.painter().clone();
    let p = &__painter;
    let hovered = response.hovered();
    let (fill, stroke) = if active {
        (
            Color32::from_rgb(0x10, 0x2a, 0x44),
            Color32::from_rgb(0x1f, 0x5f, 0x92),
        )
    } else if hovered {
        (
            Color32::from_rgb(0x0f, 0x24, 0x37),
            BORDER2,
        )
    } else {
        (Color32::from_rgb(0x0a, 0x1b, 0x2b), BORDER)
    };
    p.rect_filled(rect, 11, fill);
    p.rect_stroke(rect, 11, Stroke::new(1.0, stroke), StrokeKind::Inside);
    p.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        elide(p, &proj.name, &bold(13.5), rect.width() - 24.0),
        bold(13.5),
        TEXT,
    );
    p.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 31.0),
        Align2::LEFT_TOP,
        elide(p, &proj.path, &regular(10.5), rect.width() - 24.0),
        regular(10.5),
        MUTED,
    );
    p.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 49.0),
        Align2::LEFT_TOP,
        format!("{} snapshot{}", count, if count == 1 { "" } else { "s" }),
        bold(10.5),
        if active {
            Color32::from_rgb(0x7d, 0xd3, 0xfc)
        } else {
            ACCENT
        },
    );
    response.on_hover_text(&proj.path)
}
