//! Dialogs: create a project, describe a snapshot, confirm restores.

use egui::{Align2, Color32, Frame, Margin, RichText, Stroke, Vec2};

use crate::store;
use crate::theme::*;
use crate::{App, Dialog};

use super::common;

enum Action {
    NewProject { path: String, name: String, create: bool },
    NewSnapshot { title: String, note: String },
    NewBranch { name: String, from: String },
    Restore { sid: String, whole: bool, rel: Option<String>, safety: bool },
    Delete(String),
}

fn modal(
    ctx: &egui::Context,
    id: &str,
    width: f32,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Window::new(id)
        .id(egui::Id::new(id))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            Frame::NONE
                .fill(BG)
                .stroke(Stroke::new(1.0, BORDER2))
                .corner_radius(14)
                .inner_margin(Margin::same(22)),
        )
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);
            add(ui);
        });
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).font(bold(22.0)).color(TEXT));
    ui.add_space(2.0);
}

fn field_caption(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).font(bold(10.0)).color(DIM));
}

fn error_label(ui: &mut egui::Ui, error: &Option<String>) {
    if let Some(text) = error {
        ui.label(RichText::new(text).font(regular(12.5)).color(RED));
    }
}

fn sub_text(ui: &mut egui::Ui, text: &str) {
    ui.add(
        egui::Label::new(RichText::new(text).font(regular(12.5)).color(MUTED)).wrap(),
    );
}

fn footer(ui: &mut egui::Ui, ok_text: &str, danger: bool) -> (bool, bool) {
    let mut cancel = false;
    let mut ok = false;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if button(ui, ok_text, if danger { Btn::Primary } else { Btn::Primary }, true).clicked() {
                ok = true;
            }
            if button(ui, "Cancel", Btn::Ghost, true).clicked() {
                cancel = true;
            }
        });
    });
    (cancel, ok)
}

impl App {
    pub fn ui_dialogs(&mut self, ctx: &egui::Context) {
        let Some(mut dlg) = self.dialog.take() else {
            return;
        };
        let dim = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("dialog_dim"),
        ));
        dim.rect_filled(
            ctx.screen_rect(),
            0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 150),
        );
        let mut action: Option<Action> = None;
        let mut close = false;

        match &mut dlg {
            Dialog::NewProject { name, path, create, error } => {
                modal(ctx, "dlg_new_project", 520.0, |ui| {
                    heading(ui, "New project");
                    field_caption(ui, "NAME");
                    ui.add(
                        egui::TextEdit::singleline(name)
                            .hint_text(
                                RichText::new("e.g. The importer rewrite")
                                    .font(regular(13.0))
                                    .color(DIM),
                            )
                            .font(regular(13.0)),
                    );
                    field_caption(ui, "PROJECT FOLDER");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(path)
                                .hint_text(
                                    RichText::new("/home/you/projects/my-app")
                                        .font(regular(13.0))
                                        .color(DIM),
                                )
                                .font(regular(13.0))
                                .desired_width(ui.available_width() - 90.0),
                        );
                        if button(ui, "Browse…", Btn::Ghost, true).clicked() {
                            if let Some(folder) = rfd::FileDialog::new()
                                .set_title("Choose a project folder")
                                .pick_folder()
                            {
                                let text = folder.to_string_lossy().to_string();
                                if name.trim().is_empty() {
                                    if let Some(base) = folder.file_name() {
                                        *name = base.to_string_lossy().to_string();
                                    }
                                }
                                *path = text;
                                *error = None;
                            }
                        }
                    });
                    ui.checkbox(
                        create,
                        RichText::new("Create the folder if it does not exist yet")
                            .font(bold(13.0))
                            .color(Color32::from_rgb(0xcf, 0xdf, 0xf0)),
                    );
                    sub_text(
                        ui,
                        "A project is a folder TimeCapsule keeps an eye on. File contents are copied into its own store, so snapshots survive later edits. Caches, build output and files over 2 MB are skipped.",
                    );
                    error_label(ui, error);
                    let (cancel, ok) = footer(ui, "Create project", false);
                    if cancel {
                        close = true;
                    }
                    if ok {
                        if path.trim().is_empty() {
                            *error = Some("Choose a folder for this project first.".into());
                        } else {
                            let folder = store::expand_path(path);
                            if folder.exists() && !folder.is_dir() {
                                *error = Some("That path is a file, not a folder.".into());
                            } else if !folder.is_dir() && !*create {
                                *error = Some(
                                    "That folder does not exist — tick the box to create it, or browse elsewhere."
                                        .into(),
                                );
                            } else {
                                action = Some(Action::NewProject {
                                    path: path.clone(),
                                    name: name.clone(),
                                    create: *create,
                                });
                                close = true;
                            }
                        }
                    }
                });
            }
            Dialog::NewSnapshot { title, note, preview, error } => {
                modal(ctx, "dlg_new_snapshot", 520.0, |ui| {
                    heading(ui, "Capture a snapshot");
                    field_caption(ui, "TITLE");
                    ui.add(
                        egui::TextEdit::singleline(title)
                            .hint_text(
                                RichText::new("e.g. Before refactoring the importer")
                                    .font(regular(13.0))
                                    .color(DIM),
                            )
                            .font(regular(13.0)),
                    );
                    field_caption(ui, "NOTE");
                    ui.add(
                        egui::TextEdit::multiline(note)
                            .hint_text(
                                RichText::new("What is worth remembering about this moment?")
                                    .font(regular(13.0))
                                    .color(DIM),
                            )
                            .font(regular(13.0))
                            .desired_rows(4),
                    );
                    sub_text(ui, preview);
                    error_label(ui, error);
                    let (cancel, ok) = footer(ui, "Capture snapshot", false);
                    if cancel {
                        close = true;
                    }
                    if ok {
                        action = Some(Action::NewSnapshot {
                            title: title.clone(),
                            note: note.clone(),
                        });
                        close = true;
                    }
                });
            }
            Dialog::NewBranch { name, from, from_ts, error } => {
                let branch_name = self
                    .timeline
                    .branches
                    .iter()
                    .find(|b| b.id == self.branch)
                    .map(|b| b.name.clone())
                    .unwrap_or_else(|| "main".into());
                let from = from.clone();
                let from_ts = *from_ts;
                modal(ctx, "dlg_new_branch", 500.0, |ui| {
                    heading(ui, "New branch");
                    field_caption(ui, "BRANCH NAME");
                    ui.add(
                        egui::TextEdit::singleline(name)
                            .hint_text(
                                RichText::new("e.g. try-a-different-approach")
                                    .font(regular(13.0))
                                    .color(DIM),
                            )
                            .font(regular(13.0)),
                    );
                    sub_text(
                        ui,
                        &format!(
                            "The branch starts at {} and keeps the history it grew from. Snapshots you capture on it are stored in the same project and never touch “{}”.",
                            store::fmt_stamp(from_ts),
                            branch_name
                        ),
                    );
                    sub_text(ui, "You can switch between branches at any time in the timeline.");
                    error_label(ui, error);
                    let (cancel, ok) = footer(ui, "Create branch", false);
                    if cancel {
                        close = true;
                    }
                    if ok {
                        if name.trim().is_empty() {
                            *error = Some("Give the branch a name.".into());
                        } else {
                            action = Some(Action::NewBranch {
                                name: name.clone(),
                                from: from.clone(),
                            });
                            close = true;
                        }
                    }
                });
            }
            Dialog::Restore { sid, write, delete, whole, rel, safety } => {
                let stamp = self
                    .pid
                    .as_ref()
                    .and_then(|pid| store::get_snapshot(pid, sid))
                    .map(|s| s.ts)
                    .unwrap_or(0.0);
                let what = if *whole {
                    "the whole project".to_string()
                } else {
                    rel.clone().unwrap_or_default()
                };
                let sid = sid.clone();
                let whole = *whole;
                let rel_opt = rel.clone();
                let mut safety_val = *safety;
                let mut write_clicked = false;
                let delete_clicked = false;
                modal(ctx, "dlg_restore", 580.0, |ui| {
                    heading(
                        ui,
                        if whole { "Restore snapshot" } else { "Restore file" },
                    );
                    ui.label(
                        RichText::new(format!(
                            "Restore {} to the state captured on {}.",
                            what,
                            store::fmt_stamp(stamp)
                        ))
                        .font(regular(13.0))
                        .color(TEXT),
                    );
                    ui.horizontal(|ui| {
                        common::status_pill(ui, "added");
                        ui.label(
                            RichText::new(format!(
                                "{} file{} will be written",
                                write.len(),
                                if write.len() == 1 { "" } else { "s" }
                            ))
                            .font(regular(12.5))
                            .color(TEXT),
                        );
                        ui.add_space(10.0);
                        common::status_pill(ui, "deleted");
                        ui.label(
                            RichText::new(format!(
                                "{} file{} will be removed",
                                delete.len(),
                                if delete.len() == 1 { "" } else { "s" }
                            ))
                            .font(regular(12.5))
                            .color(TEXT),
                        );
                    });
                    Frame::NONE
                        .fill(SURF)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x14, 0x30, 0x49)))
                        .corner_radius(12)
                        .inner_margin(Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("restore_list")
                                .max_height(170.0)
                                .show(ui, |ui| {
                                    let mut shown = 0usize;
                                    for rel in write.iter().take(80) {
                                        ui.label(
                                            RichText::new(format!(
                                                "{}  {}",
                                                if whole { "✎" } else { "↺" },
                                                rel
                                            ))
                                            .font(regular(12.0))
                                            .color(MUTED),
                                        );
                                        shown += 1;
                                    }
                                    for rel in delete.iter().take(80) {
                                        ui.label(
                                            RichText::new(format!("✕  {}", rel))
                                                .font(regular(12.0))
                                                .color(MUTED),
                                        );
                                        shown += 1;
                                    }
                                    if shown == 0 {
                                        ui.label(
                                            RichText::new(
                                                "The folder already matches this snapshot.",
                                            )
                                            .font(regular(12.0))
                                            .color(MUTED),
                                        );
                                    }
                                    if write.len() + delete.len() > shown {
                                        ui.label(
                                            RichText::new(format!(
                                                "… and {} more",
                                                write.len() + delete.len() - shown
                                            ))
                                            .font(regular(12.0))
                                            .color(MUTED),
                                        );
                                    }
                                });
                        });
                    ui.checkbox(
                        &mut safety_val,
                        RichText::new(
                            "Capture the current state first, so this restore can be undone",
                        )
                        .font(bold(13.0))
                        .color(Color32::from_rgb(0xcf, 0xdf, 0xf0)),
                    );
                    let (cancel, ok) = footer(ui, "Restore now", false);
                    if cancel {
                        close = true;
                    }
                    if ok {
                        write_clicked = true;
                    }
                    let _ = delete_clicked;
                });
                *safety = safety_val;
                if write_clicked {
                    action = Some(Action::Restore {
                        sid: sid.clone(),
                        whole,
                        rel: rel_opt.clone(),
                        safety: safety_val,
                    });
                    close = true;
                }
            }
            Dialog::Confirm { title, text, ok_text, sid } => {
                let sid = sid.clone();
                let ok_text = ok_text.clone();
                modal(ctx, "dlg_confirm", 460.0, |ui| {
                    heading(ui, title);
                    sub_text(ui, text);
                    let (cancel, ok) = footer(ui, &ok_text, true);
                    if cancel {
                        close = true;
                    }
                    if ok {
                        action = Some(Action::Delete(sid.clone()));
                        close = true;
                    }
                });
            }
            Dialog::Message { title, text } => {
                modal(ctx, "dlg_message", 460.0, |ui| {
                    heading(ui, title);
                    sub_text(ui, text);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if button(ui, "OK", Btn::Primary, true).clicked() {
                                    close = true;
                                }
                            },
                        );
                    });
                });
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if let Some(act) = action {
            match act {
                Action::NewProject { path, name, create } => {
                    self.create_project(&path, &name, create)
                }
                Action::NewSnapshot { title, note } => self.capture_snapshot(&title, &note),
                Action::NewBranch { name, from } => self.create_branch(&name, &from),
                Action::Restore { sid, whole, rel, safety } => {
                    self.complete_restore(&sid, whole, rel.as_deref(), safety)
                }
                Action::Delete(sid) => self.delete_snapshot(&sid),
            }
        } else if !close {
            self.dialog = Some(dlg);
        }
    }

    pub fn ui_toast(&mut self, ctx: &egui::Context) {
        let Some((title, text, _)) = self.toast.clone() else {
            return;
        };
        egui::Area::new(egui::Id::new("toast"))
            .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -66.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                Frame::NONE
                    .fill(Color32::from_rgb(0x0b, 0x1c, 0x2c))
                    .stroke(Stroke::new(1.0, BORDER2))
                    .corner_radius(10)
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(title).font(bold(12.0)).color(TEXT));
                        ui.label(RichText::new(text).font(regular(12.0)).color(MUTED));
                    });
            });
    }
}


