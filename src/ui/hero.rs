//! The banner at the top: planet limb, glowing brand, search and actions.

use egui::{
    Align2, Color32, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2,
};

use crate::store;
use crate::theme::*;
use crate::App;

use super::common;

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

fn globe_color(dist: f32, radius: f32) -> Color32 {
    let t = (dist / (radius * 1.5)).clamp(0.0, 1.0);
    let stops = [
        (0.0, Color32::from_rgb(0x1d, 0x51, 0x73)),
        (0.30, Color32::from_rgb(0x0f, 0x2e, 0x45)),
        (0.62, Color32::from_rgb(0x08, 0x19, 0x27)),
        (1.0, Color32::from_rgb(0x04, 0x0b, 0x13)),
    ];
    for w in stops.windows(2) {
        if t <= w[1].0 {
            let span = w[1].0 - w[0].0;
            let f = if span > 0.0 { (t - w[0].0) / span } else { 0.0 };
            return lerp_color(w[0].1, w[1].1, f);
        }
    }
    stops[3].1
}

fn paint_planet(painter: &egui::Painter, rect: Rect) {
    let w = rect.width();
    let h = rect.height();
    let radius = w * 0.78;
    let cx = rect.left() + w * 1.24;
    let cy = rect.top() + h * 1.08 - radius;
    let center = Pos2::new(cx, cy);
    let glow = Pos2::new(cx - radius * 0.55, cy + radius * 1.05);
    painter.circle_filled(glow, radius * 0.9, alpha(ACCENT, 22));

    let gradient_center = Pos2::new(cx - radius * 0.8, cy + radius * 1.25);
    let rings = 10;
    let segs = 96;
    let mut mesh = Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    for ring in 0..=rings {
        let rr = radius * ring as f32 / rings as f32;
        for s in 0..=segs {
            let a = s as f32 / segs as f32 * std::f32::consts::TAU;
            let pos = center + Vec2::new(a.cos(), a.sin()) * rr;
            let dist = (pos - gradient_center).length();
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: globe_color(dist, radius),
            });
        }
    }
    let row = (segs + 1) as u32;
    for ring in 0..rings {
        for s in 0..segs {
            let i = ring * row + s;
            mesh.indices.extend_from_slice(&[i, i + row, i + 1, i + 1, i + row, i + row + 1]);
        }
    }
    painter.add(Shape::mesh(mesh));

    painter.circle_stroke(
        center,
        radius - 1.5,
        Stroke::new(2.0, Color32::from_rgba_unmultiplied(120, 214, 255, 120)),
    );
    painter.circle_stroke(center, radius + 5.0, Stroke::new(9.0, alpha(ACCENT, 46)));
    painter.circle_stroke(center, radius + 16.0, Stroke::new(22.0, alpha(ACCENT, 22)));
}

impl App {
    pub fn ui_hero(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 134.0), Sense::hover());
        let __painter = ui.painter().clone();
    let p = &__painter;

        let mut clip = p.with_clip_rect(rect);
        gradient_rect(
            &clip,
            rect,
            Color32::from_rgb(0x07, 0x16, 0x24),
            Color32::from_rgb(0x05, 0x0e, 0x19),
        );
        // stars
        let mut seed: u64 = 1337;
        for i in 0..90 {
            seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) & 0x7FFF_FFFF;
            let x = rect.left() + (seed >> 7) as f32 % width.max(1.0);
            let y = rect.top() + (seed >> 3) as f32 % (134.0 * 0.8);
            let a = (26 + (seed >> 11) % 90) as u8;
            clip.circle_filled(
                Pos2::new(x, y),
                if i % 5 == 0 { 1.2 } else { 0.7 },
                Color32::from_rgba_unmultiplied(255, 255, 255, a),
            );
        }
        paint_planet(&clip, rect);
        clip.rect_stroke(
            rect,
            18,
            Stroke::new(1.0, Color32::from_rgb(0x12, 0x30, 0x49)),
            StrokeKind::Inside,
        );

        let cy = rect.center().y;
        let logo = Rect::from_center_size(Pos2::new(rect.left() + 54.0, cy), Vec2::splat(56.0));
        common::clock_logo(ui.painter(), logo);
        ui.painter().text(
            Pos2::new(rect.left() + 98.0, cy - 30.0),
            Align2::LEFT_TOP,
            "TimeCapsule",
            bold(27.0),
            Color32::from_rgb(0xf4, 0xf9, 0xff),
        );
        ui.painter().text(
            Pos2::new(rect.left() + 98.0, cy + 5.0),
            Align2::LEFT_TOP,
            "Your digital time machine.",
            regular(13.0),
            Color32::from_rgb(0x9d, 0xb4, 0xcd),
        );

        // right aligned controls, shrinking gracefully on narrow windows
        let right = rect.right() - 20.0;
        let btn_w = 150.0;
        let snap_w = 164.0;
        let x_snap = right - snap_w;
        let mut x_proj = x_snap - 10.0 - btn_w;
        let title_end = rect.left() + 300.0;
        let show_project_btn = x_proj - 16.0 - title_end >= 190.0;
        if !show_project_btn {
            x_proj = x_snap - 10.0;
        }
        let search_left = title_end;
        let search_w = (x_proj - 16.0 - search_left).clamp(130.0, 330.0);
        let search_rect = Rect::from_min_size(
            Pos2::new(search_left, cy - 21.0),
            Vec2::new(search_w, 42.0),
        );

        let proj_rect = Rect::from_min_size(Pos2::new(x_proj, cy - 20.0), Vec2::new(btn_w, 40.0));
        let snap_rect = Rect::from_min_size(Pos2::new(x_snap, cy - 20.0), Vec2::new(snap_w, 40.0));

        let mut new_project = false;
        if show_project_btn {
            common::area(ui, proj_rect, |ui| {
                if button(ui, "＋  New project", Btn::Ghost, true).clicked() {
                    new_project = true;
                }
            });
        }
        let mut new_snap = false;
        common::area(ui, snap_rect, |ui| {
            if button(ui, "＋  New snapshot", Btn::Primary, self.pid.is_some()).clicked() {
                new_snap = true;
            }
        });
        if new_project {
            self.dialog_new_project();
        }
        if new_snap {
            self.dialog_new_snapshot();
        }

        // search box
        ui.painter().rect_filled(
            search_rect,
            11,
            Color32::from_rgb(0x07, 0x13, 0x1f),
        );
        ui.painter().rect_stroke(
            search_rect,
            11,
            Stroke::new(1.0, Color32::from_rgb(0x15, 0x33, 0x4e)),
            StrokeKind::Inside,
        );
        ui.painter().text(
            Pos2::new(search_rect.left() + 13.0, search_rect.center().y),
            Align2::LEFT_CENTER,
            "⌕",
            icon(17.0),
            Color32::from_rgb(0x6d, 0x87, 0xa2),
        );
        let show_hint = search_w > 250.0;
        if show_hint {
            ui.painter().text(
                Pos2::new(search_rect.right() - 9.0, search_rect.center().y),
                Align2::RIGHT_CENTER,
                "Ctrl + K",
                bold(10.5),
                Color32::from_rgb(0x6d, 0x87, 0xa2),
            );
        }
        let edit_rect = Rect::from_min_max(
            Pos2::new(search_rect.left() + 34.0, search_rect.top() + 8.0),
            Pos2::new(
                search_rect.right() - if show_hint { 66.0 } else { 10.0 },
                search_rect.bottom() - 8.0,
            ),
        );
        common::area(ui, edit_rect, |ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(
                        egui::RichText::new("Search snapshots…")
                            .font(regular(13.0))
                            .color(DIM),
                    )
                    .font(regular(13.0))
                    .desired_width(edit_rect.width())
                    .frame(false)
                    .margin(Vec2::new(0.0, 4.0)),
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
            p.rect_filled(chip_rect, 8, bg.unwrap_or(Color32::from_rgb(0x0f, 0x24, 0x37)));
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
            ui.painter().rect_filled(gear_rect, 8, Color32::from_rgb(0x0e, 0x21, 0x35));
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
