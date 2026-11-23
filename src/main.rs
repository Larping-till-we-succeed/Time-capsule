//! TimeCapsule 0.0 — a time machine for project folders, in Rust.

mod diff;
mod preview;
mod store;
mod theme;
mod ui;

use std::collections::HashMap;
use std::time::Duration;

use eframe::egui;
use egui::{Key, Modifiers, Pos2, Rect, Vec2};

use preview::Preview;
use store::{Branch, Overview, Project, ProjectStats, Snapshot};
use theme::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Timeline,
    Compare,
    Stats,
    Settings,
}

impl Page {
    pub fn key(&self) -> &'static str {
        match self {
            Page::Dashboard => "dashboard",
            Page::Timeline => "timeline",
            Page::Compare => "compare",
            Page::Stats => "stats",
            Page::Settings => "settings",
        }
    }
}

#[derive(Default)]
pub struct DashData {
    pub chain: Vec<Snapshot>,
    pub series: Vec<f64>,
    pub sizes: Vec<f64>,
    pub total_snaps: usize,
    pub project_count: usize,
    pub new_projects: usize,
    pub used: u64,
    pub pct: i64,
    pub space_delta: i64,
    pub dirty: bool,
    pub changed: (usize, usize, usize),
    pub recent: Vec<Snapshot>,
    pub strip: Vec<(Snapshot, Option<Preview>)>,
    pub current_preview: Option<Preview>,
    pub project_path: String,
    pub latest: Option<Snapshot>,
}

#[derive(Default)]
pub struct TimelineState {
    pub chain: Vec<Snapshot>,
    pub all_snaps: Vec<Snapshot>,
    pub branches: Vec<Branch>,
    pub search: String,
    pub pinned_only: bool,
    pub selected: Option<String>,
    pub detail: Option<Snapshot>,
    pub prev: Option<Snapshot>,
    pub note: String,
    pub note_sid: Option<String>,
    pub note_dirty: bool,
    pub note_status: String,
    pub filter: String,
}

impl TimelineState {
    pub fn reset(&mut self) {
        self.chain.clear();
        self.all_snaps.clear();
        self.branches.clear();
        self.selected = None;
        self.detail = None;
        self.prev = None;
        self.note.clear();
        self.note_sid = None;
        self.note_dirty = false;
        self.note_status.clear();
        self.filter.clear();
        self.pinned_only = false;
    }
}

#[derive(Default)]
pub struct CompareState {
    pub chain: Vec<Snapshot>,
    pub a: Option<String>,
    pub b: Option<String>,
    pub filter: String,
    pub active: Option<String>,
    pub diff: Option<store::Diff>,
    pub diff_lines: Vec<(char, String)>,
    pub line_meta: String,
    pub binary: bool,
    pub empty_reason: String,
}

pub struct SettingsForm {
    pub enabled: bool,
    pub minutes: i64,
    pub only_changes: bool,
    pub saved_at: Option<f64>,
}

pub enum Dialog {
    NewProject {
        name: String,
        path: String,
        create: bool,
        error: Option<String>,
    },
    NewSnapshot {
        title: String,
        note: String,
        preview: String,
        error: Option<String>,
    },
    NewBranch {
        name: String,
        from: String,
        from_ts: f64,
        error: Option<String>,
    },
    Restore {
        sid: String,
        write: Vec<String>,
        delete: Vec<String>,
        whole: bool,
        rel: Option<String>,
        safety: bool,
    },
    Confirm {
        title: String,
        text: String,
        ok_text: String,
        sid: String,
    },
    Message {
        title: String,
        text: String,
    },
}

pub struct App {
    pub page: Page,
    pub projects: Vec<Project>,
    pub pid: Option<String>,
    pub branch: String,
    pub overview: Overview,
    pub auto: store::AutoConfig,
    pub settings: SettingsForm,
    pub last_auto: Option<f64>,
    pub auto_deadline: Option<f64>,
    pub query: String,
    pub dash: DashData,
    pub previews: HashMap<(String, String, usize, Option<String>), Option<Preview>>,
    pub dirty_cache: HashMap<String, (Option<String>, f64, bool)>,
    pub timeline: TimelineState,
    pub compare: CompareState,
    pub stats: Option<ProjectStats>,
    pub stats_for: Option<(String, String)>,
    pub dialog: Option<Dialog>,
    pub toast: Option<(String, String, f64)>,
    pub focus_search: bool,
    pub project_head: Option<(String, String, usize, String, String)>,
    pub status_storage: (u64, u64),
    pub gc_preview: Option<store::GcReport>,
    pub verify_report: Option<store::VerifyReport>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        install_style(&cc.egui_ctx);
        let auto = store::autosave_config();
        let mut app = Self {
            page: Page::Dashboard,
            projects: Vec::new(),
            pid: None,
            branch: "main".into(),
            overview: Overview::default(),
            auto,
            settings: SettingsForm {
                enabled: auto.enabled,
                minutes: auto.minutes,
                only_changes: auto.only_changes,
                saved_at: None,
            },
            last_auto: None,
            auto_deadline: None,
            query: String::new(),
            dash: DashData::default(),
            previews: HashMap::new(),
            dirty_cache: HashMap::new(),
            timeline: TimelineState::default(),
            compare: CompareState::default(),
            stats: None,
            stats_for: None,
            dialog: None,
            toast: None,
            focus_search: false,
            project_head: None,
            status_storage: (0, 0),
            gc_preview: None,
            verify_report: None,
        };
        app.reset_autosave_clock();
        app.refresh(None);
        app
    }

    // ------------------------------------------------------------------ state
    pub fn project(&self) -> Option<Project> {
        self.pid.as_ref().and_then(|pid| store::get_project(pid))
    }

    pub fn reset_autosave_clock(&mut self) {
        if self.auto.enabled {
            self.auto_deadline = Some(store::now() + self.auto.minutes as f64 * 60.0);
        } else {
            self.auto_deadline = None;
        }
    }

    pub fn preview_for(
        &mut self,
        snap: &Snapshot,
        prev: Option<&Snapshot>,
        max_lines: usize,
    ) -> Option<Preview> {
        let pid = self.pid.clone()?;
        let key = (
            pid.clone(),
            snap.id.clone(),
            max_lines,
            prev.map(|p| p.id.clone()),
        );
        if let Some(hit) = self.previews.get(&key) {
            return hit.clone();
        }
        let loaded = preview::snapshot_preview(&pid, snap, max_lines, prev);
        if self.previews.len() > 400 {
            self.previews.clear();
        }
        self.previews.insert(key, loaded.clone());
        loaded
    }

    pub fn refresh(&mut self, select: Option<String>) {
        self.projects = store::load_projects();
        if self
            .pid
            .as_ref()
            .map(|p| self.projects.iter().all(|x| &x.id != p))
            .unwrap_or(true)
        {
            self.pid = self.projects.first().map(|p| p.id.clone());
        }
        self.overview = store::library_stats();

        let Some(pid) = self.pid.clone() else {
            self.page = Page::Dashboard;
            self.dash = DashData::default();
            self.project_head = None;
            self.timeline.reset();
            self.compare = CompareState::default();
            self.stats = None;
            self.status_storage = (0, 0);
            return;
        };

        self.branch = store::active_branch(&pid);
        let proj = store::get_project(&pid);
        let snaps = store::load_snapshots(&pid);
        let mut chips = Vec::new();
        chips.push(format!("{} snapshots", snaps.len()));
        if let Some(last) = snaps.last() {
            chips.push(format!("latest {}", store::ago(last.ts)));
            chips.push(store::fmt_bytes(store::storage_bytes(&pid)));
        } else {
            chips.push("no snapshots yet".into());
        }
        self.project_head = proj.as_ref().map(|p| {
            (
                p.name.clone(),
                p.path.clone(),
                snaps.len(),
                chips.get(1).cloned().unwrap_or_default(),
                chips.get(2).cloned().unwrap_or_default(),
            )
        });
        let captured: u64 = store::branch_chain(&pid, Some(&self.branch))
            .iter()
            .map(|s| s.bytes)
            .sum();
        self.status_storage = (store::storage_bytes(&pid), captured);

        match self.page {
            Page::Dashboard => self.rebuild_dashboard(),
            Page::Timeline => self.rebuild_timeline(select),
            Page::Compare => self.rebuild_compare(None, None),
            Page::Stats => self.rebuild_stats(),
            Page::Settings => {}
        }
    }

    pub fn select_page(&mut self, page: Page) {
        if self.pid.is_none() {
            return;
        }
        self.page = page;
        match page {
            Page::Dashboard => self.rebuild_dashboard(),
            Page::Timeline => {
                let sel = self.timeline.selected.clone();
                self.rebuild_timeline(sel);
            }
            Page::Compare => {
                let (a, b) = (self.compare.a.clone(), self.compare.b.clone());
                self.rebuild_compare(a, b);
            }
            Page::Stats => self.rebuild_stats(),
            Page::Settings => self.load_settings_form(),
        }
    }

    pub fn select_project(&mut self, pid: &str) {
        if self.pid.as_deref() == Some(pid) {
            return;
        }
        self.pid = Some(pid.to_string());
        self.page = Page::Dashboard;
        self.timeline.reset();
        self.compare = CompareState::default();
        self.refresh(None);
    }

    // ------------------------------------------------------------- dashboard
    pub fn rebuild_dashboard(&mut self) {
        let Some(pid) = self.pid.clone() else {
            self.dash = DashData::default();
            return;
        };
        let branch = self.branch.clone();
        let chain = store::branch_chain(&pid, Some(&branch));
        let stats = self.overview.clone();
        let total_snaps: usize = stats.counts.values().sum();
        let used = stats.used;
        let today = store::now();
        let mut series = Vec::new();
        for back in (0..8).rev() {
            let key = store::day_key(today - back as f64 * 86400.0);
            series.push(*stats.per_day.get(&key).unwrap_or(&0) as f64);
        }
        let recent_sum: f64 = series[5..].iter().sum();
        let earlier_sum: f64 = series[..5].iter().sum::<f64>().max(1.0);
        let pct = ((recent_sum - earlier_sum) / earlier_sum * 100.0).round() as i64;
        let new_projects = self
            .projects
            .iter()
            .filter(|p| p.created > today - 7.0 * 86400.0)
            .count();
        let sizes: Vec<f64> = chain
            .iter()
            .rev()
            .take(8)
            .rev()
            .map(|s| s.files.values().map(|f| f.size as f64).sum())
            .collect();
        let sizes = if sizes.is_empty() { vec![0.0, 0.0] } else { sizes };
        let space_delta = if sizes.len() > 1 && sizes[0] > 0.0 {
            ((sizes[sizes.len() - 1] - sizes[0]) / sizes[0] * 100.0).round() as i64
        } else {
            0
        };
        let dirty = self.is_dirty(&pid, &branch, chain.last().map(|s| s.id.clone()));

        let (added, deleted, modified) = if let Some(last) = chain.last() {
            if chain.len() > 1 {
                let d = store::compare(&chain[chain.len() - 2], last);
                (d.added.len(), d.deleted.len(), d.modified.len())
            } else {
                (last.count, 0, 0)
            }
        } else {
            (0, 0, 0)
        };

        let recent_all: Vec<Snapshot> = chain.iter().rev().take(6).cloned().collect();
        let query = self.query.trim().to_lowercase();
        let recent: Vec<Snapshot> = recent_all
            .into_iter()
            .filter(|s| {
                query.is_empty()
                    || format!("{} {}", s.note, s.title).to_lowercase().contains(&query)
            })
            .take(4)
            .collect();

        let strip: Vec<(Snapshot, Option<Preview>)> = if chain.is_empty() {
            Vec::new()
        } else {
            let start = chain.len().saturating_sub(4);
            chain[start..]
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let prev = if i == 0 {
                        if start > 0 {
                            Some(chain[start - 1].clone())
                        } else {
                            None
                        }
                    } else {
                        Some(chain[start + i - 1].clone())
                    };
                    let pv = self.preview_for(s, prev.as_ref(), 8);
                    (s.clone(), pv)
                })
                .collect()
        };
        let latest = chain.last().cloned();
        let prev = if chain.len() > 1 {
            Some(chain[chain.len() - 2].clone())
        } else {
            None
        };
        let current_preview = latest
            .as_ref()
            .and_then(|s| self.preview_for(s, prev.as_ref(), 9));
        let project_path = store::get_project(&pid).map(|p| p.path).unwrap_or_default();

        self.dash = DashData {
            chain,
            series,
            sizes,
            total_snaps,
            project_count: self.projects.len(),
            new_projects,
            used,
            pct,
            space_delta,
            dirty,
            changed: (added, deleted, modified),
            recent,
            strip,
            current_preview,
            project_path,
            latest,
        };
    }

    fn is_dirty(&mut self, pid: &str, branch: &str, latest: Option<String>) -> bool {
        let now = store::now();
        if let Some((cached_latest, at, dirty)) = self.dirty_cache.get(pid) {
            if *cached_latest == latest && now - at < 15.0 {
                return *dirty;
            }
        }
        let dirty = store::has_changes(pid, Some(branch));
        self.dirty_cache
            .insert(pid.to_string(), (latest, now, dirty));
        dirty
    }

    // -------------------------------------------------------------- timeline
    pub fn rebuild_timeline(&mut self, select: Option<String>) {
        let Some(pid) = self.pid.clone() else {
            self.timeline.reset();
            return;
        };
        let branch = self.branch.clone();
        self.timeline.chain = store::branch_chain(&pid, Some(&branch));
        self.timeline.all_snaps = store::load_snapshots(&pid);
        self.timeline.branches = store::load_branches(&pid);
        let target = select
            .or_else(|| self.timeline.selected.clone())
            .or_else(|| self.timeline.chain.last().map(|s| s.id.clone()));
        if target.as_ref().map(|t| self.timeline.chain.iter().any(|s| &s.id == t))
            == Some(true)
        {
            self.select_snapshot(&target.unwrap());
        } else {
            self.timeline.selected = None;
            self.timeline.detail = None;
            self.timeline.prev = None;
            self.timeline.note.clear();
            self.timeline.note_sid = None;
        }
    }

    pub fn select_snapshot(&mut self, sid: &str) {
        let Some(pid) = self.pid.clone() else { return };
        let snap = self
            .timeline
            .chain
            .iter()
            .find(|s| s.id == sid)
            .cloned();
        let Some(snap) = snap else { return };
        self.timeline.selected = Some(sid.to_string());
        let prev = if snap.branch_id() == self.branch
            || store::get_snapshot(&pid, sid).is_some()
        {
            store::previous_snapshot(&pid, sid, Some(&self.branch))
        } else {
            None
        };
        if self.timeline.note_sid.as_deref() != Some(sid) {
            self.timeline.note = snap.note.clone();
            self.timeline.note_sid = Some(sid.to_string());
            self.timeline.note_dirty = false;
        }
        self.timeline.note_status.clear();
        self.timeline.detail = Some(snap);
        self.timeline.prev = prev;
    }

    pub fn notify(&mut self, title: &str, text: impl Into<String>) {
        self.toast = Some((title.to_string(), text.into(), store::now() + 3.0));
    }

    pub fn toggle_pin(&mut self, sid: &str) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(pinned) = store::toggle_pin(&pid, sid) else { return };
        if let Some(detail) = self.timeline.detail.as_mut() {
            if detail.id == sid {
                detail.pinned = pinned;
            }
        }
        for list in [&mut self.timeline.all_snaps, &mut self.timeline.chain] {
            if let Some(s) = list.iter_mut().find(|s| s.id == sid) {
                s.pinned = pinned;
            }
        }
        self.notify(
            if pinned { "Snapshot pinned" } else { "Snapshot unpinned" },
            "Pinned moments stay at the top of your mind — use the Pinned filter in History.",
        );
    }

    /// Write a snapshot into a folder the user picks; never touches the project.
    pub fn export_restore(&mut self, sid: &str) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(snap) = store::get_snapshot(&pid, sid) else { return };
        let Some(base) = rfd::FileDialog::new()
            .set_title("Choose where the restored copy should go")
            .pick_folder()
        else {
            return;
        };
        let name = self.project().map(|p| p.name).unwrap_or_else(|| "TimeCapsule".into());
        let safe: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let dir = base.join(format!("{} restored {}", safe.trim(), sid));
        match store::restore_export(&pid, &snap, &dir) {
            Ok(n) => {
                self.dialog = Some(Dialog::Message {
                    title: "Restored to a new folder".into(),
                    text: format!(
                        "{} file{} written to\n{}",
                        n,
                        if n == 1 { "" } else { "s" },
                        dir.display()
                    ),
                });
            }
            Err(e) => {
                self.dialog = Some(Dialog::Message {
                    title: "Export failed".into(),
                    text: e.to_string(),
                });
            }
        }
    }

    pub fn branch_points(&self) -> HashMap<String, Vec<String>> {
        let mut points: HashMap<String, Vec<String>> = HashMap::new();
        for b in &self.timeline.branches {
            if let Some(base) = &b.base {
                points.entry(base.clone()).or_default().push(b.name.clone());
            }
        }
        points
    }

    // --------------------------------------------------------------- compare
    pub fn rebuild_compare(&mut self, a: Option<String>, b: Option<String>) {
        let Some(pid) = self.pid.clone() else {
            self.compare = CompareState::default();
            return;
        };
        let branch = self.branch.clone();
        self.compare.chain = store::branch_chain(&pid, Some(&branch));
        let ids: Vec<String> = self.compare.chain.iter().map(|s| s.id.clone()).collect();
        self.compare.a = a
            .filter(|x| ids.contains(x))
            .or_else(|| {
                self.compare.a.clone().filter(|x| ids.contains(x))
            })
            .or_else(|| ids.get(ids.len().saturating_sub(2)).cloned());
        self.compare.b = b
            .filter(|x| ids.contains(x))
            .or_else(|| {
                self.compare.b.clone().filter(|x| ids.contains(x))
            })
            .or_else(|| ids.last().cloned());
        self.load_compare_diff();
    }

    pub fn load_compare_diff(&mut self) {
        let Some(pid) = self.pid.clone() else { return };
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
        self.compare.diff = None;
        self.compare.diff_lines.clear();
        self.compare.empty_reason.clear();
        let (Some(a), Some(b)) = (a, b) else {
            self.compare.empty_reason =
                "Add at least two snapshots to compare a project with itself over time.".into();
            return;
        };
        let diff = store::compare(&a, &b);
        let rows = store::ordered_changes(&diff);
        let active = self
            .compare
            .active
            .clone()
            .filter(|r| rows.iter().any(|(p, _)| p == r))
            .or_else(|| rows.first().map(|(p, _)| p.clone()));
        self.compare.diff = Some(diff);
        if let Some(rel) = active {
            self.show_compare_file(&pid, &a, &b, &rel);
        }
    }

    pub fn show_compare_file(&mut self, pid: &str, a: &Snapshot, b: &Snapshot, rel: &str) {
        self.compare.active = Some(rel.to_string());
        let fa = a.files.get(rel);
        let fb = b.files.get(rel);
        let old = fa.map(|f| store::read_blob(pid, &f.h)).unwrap_or_default();
        let new = fb.map(|f| store::read_blob(pid, &f.h)).unwrap_or_default();
        let kind = if fa.is_some() && fb.is_some() {
            "modified"
        } else if fb.is_some() {
            "added"
        } else {
            "deleted"
        };
        self.compare.binary =
            !diff::looks_text(rel, &old) || !diff::looks_text(rel, &new);
        if self.compare.binary {
            self.compare.line_meta = "binary file".into();
            self.compare.diff_lines.clear();
            return;
        }
        let (added, removed) = diff::line_counts(&old, &new);
        self.compare.line_meta = format!("{} · +{} −{}", kind, added, removed);
        self.compare.diff_lines = diff::unified(rel, &old, &new, 3);
    }

    // ----------------------------------------------------------------- stats
    pub fn rebuild_stats(&mut self) {
        let Some(pid) = self.pid.clone() else {
            self.stats = None;
            return;
        };
        let branch = self.branch.clone();
        self.stats = Some(store::project_stats(&pid, Some(&branch)));
        self.stats_for = Some((pid, branch));
    }

    // -------------------------------------------------------------- settings
    pub fn load_settings_form(&mut self) {
        let cfg = store::settings();
        self.settings.enabled = cfg.auto_save;
        self.settings.minutes = cfg.auto_save_minutes.clamp(1, 1440);
        self.settings.only_changes = cfg.auto_save_only_changes;
    }

    pub fn save_settings(&mut self) {
        let mut cfg = store::settings();
        cfg.auto_save = self.settings.enabled;
        cfg.auto_save_minutes = self.settings.minutes.clamp(1, 1440);
        cfg.auto_save_only_changes = self.settings.only_changes;
        store::save_settings(&cfg);
        self.auto = store::autosave_config();
        self.reset_autosave_clock();
        self.settings.saved_at = Some(store::now());
    }

    pub fn set_autosave(&mut self, on: bool) {
        let mut cfg = store::settings();
        cfg.auto_save = on;
        store::save_settings(&cfg);
        self.auto = store::autosave_config();
        self.settings.enabled = on;
        self.reset_autosave_clock();
        self.refresh(None);
    }

    pub fn autosave_tick(&mut self) {
        let now = store::now();
        if !self.auto.enabled {
            return;
        }
        if let Some(deadline) = self.auto_deadline {
            if now < deadline {
                return;
            }
        }
        self.auto_deadline = Some(now + self.auto.minutes as f64 * 60.0);
        if self.projects.is_empty() {
            return;
        }
        let mut captured = false;
        let projects = self.projects.clone();
        for proj in projects {
            if self.auto.only_changes && !store::has_changes(&proj.id, None) {
                continue;
            }
            if store::take_snapshot(&proj.id, "", "Auto-save", "auto", None).is_ok() {
                captured = true;
            }
        }
        if captured {
            self.last_auto = Some(store::now());
            self.refresh(None);
        }
    }

    // ---------------------------------------------------------------- actions
    pub fn dialog_new_project(&mut self) {
        self.dialog = Some(Dialog::NewProject {
            name: String::new(),
            path: String::new(),
            create: true,
            error: None,
        });
    }

    pub fn dialog_new_snapshot(&mut self) {
        let Some(proj) = self.project() else { return };
        let files = store::scan_project(std::path::Path::new(&proj.path));
        let total: u64 = files.iter().map(|f| f.2).sum();
        let preview = format!(
            "{} tracked files · {} will be captured. Existing content is reused from the project store, so repeated snapshots stay small.",
            files.len(),
            store::fmt_bytes(total)
        );
        self.dialog = Some(Dialog::NewSnapshot {
            title: String::new(),
            note: String::new(),
            preview,
            error: None,
        });
    }

    pub fn capture_snapshot(&mut self, title: &str, note: &str) {
        let Some(pid) = self.pid.clone() else { return };
        let branch = self.branch.clone();
        match store::take_snapshot(&pid, note, title, "manual", Some(&branch)) {
            Ok(snap) => {
                self.page = Page::Timeline;
                self.refresh(Some(snap.id));
            }
            Err(e) => {
                self.dialog = Some(Dialog::Message {
                    title: "TimeCapsule".into(),
                    text: e.to_string(),
                });
            }
        }
    }

    pub fn compare_latest(&mut self) {
        let Some(pid) = self.pid.clone() else { return };
        let chain = store::branch_chain(&pid, Some(&self.branch));
        if chain.len() < 2 {
            return;
        }
        let a = chain[chain.len() - 2].id.clone();
        let b = chain[chain.len() - 1].id.clone();
        self.page = Page::Compare;
        self.rebuild_compare(Some(a), Some(b));
    }

    pub fn restore_latest(&mut self) {
        let Some(pid) = self.pid.clone() else { return };
        let chain = store::branch_chain(&pid, Some(&self.branch));
        if let Some(last) = chain.last() {
            self.start_restore(last.id.clone(), true, None);
        }
    }

    pub fn start_restore(&mut self, sid: String, whole: bool, rel: Option<String>) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(snap) = store::get_snapshot(&pid, &sid) else {
            return;
        };
        match store::preview_restore(&pid, &snap) {
            Ok((mut write, mut delete)) => {
                if !whole {
                    if let Some(r) = &rel {
                        write.retain(|p| p == r);
                        delete.retain(|p| p == r);
                    }
                }
                self.dialog = Some(Dialog::Restore {
                    sid,
                    write,
                    delete,
                    whole,
                    rel,
                    safety: true,
                });
            }
            Err(e) => {
                self.dialog = Some(Dialog::Message {
                    title: "TimeCapsule".into(),
                    text: e.to_string(),
                });
            }
        }
    }

    pub fn complete_restore(
        &mut self,
        sid: &str,
        whole: bool,
        rel: Option<&str>,
        safety: bool,
    ) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(snap) = store::get_snapshot(&pid, sid) else { return };
        let rels = rel.map(|r| vec![r.to_string()]);
        match store::restore(&pid, &snap, whole, rels.as_deref(), safety) {
            Ok((actions, safety_snap)) => {
                self.refresh(None);
                let text = if whole {
                    format!(
                        "{} file{} restored from {}.{}",
                        actions.len(),
                        if actions.len() == 1 { "" } else { "s" },
                        store::fmt_stamp(snap.ts),
                        if safety_snap.is_some() {
                            "\nThe previous state was kept as a safety snapshot."
                        } else {
                            ""
                        }
                    )
                } else {
                    format!(
                        "{}{}",
                        rel.unwrap_or(""),
                        if actions.is_empty() {
                            " already matches this snapshot."
                        } else {
                            " restored."
                        }
                    )
                };
                self.dialog = Some(Dialog::Message { title: "Restored".into(), text });
            }
            Err(e) => {
                self.dialog = Some(Dialog::Message {
                    title: "TimeCapsule".into(),
                    text: format!("Restore failed:\n{}", e),
                });
            }
        }
    }

    pub fn confirm_delete(&mut self, sid: &str) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(snap) = store::get_snapshot(&pid, sid) else { return };
        self.dialog = Some(Dialog::Confirm {
            title: "Delete snapshot".into(),
            text: format!(
                "Delete the snapshot from {}? Files already captured by other snapshots stay in the project; this moment’s note disappears.",
                store::fmt_stamp(snap.ts)
            ),
            ok_text: "Delete snapshot".into(),
            sid: sid.to_string(),
        });
    }

    pub fn delete_snapshot(&mut self, sid: &str) {
        let Some(pid) = self.pid.clone() else { return };
        store::delete_snapshot(&pid, sid);
        self.refresh(None);
    }

    pub fn dialog_new_branch(&mut self) {
        let Some(pid) = self.pid.clone() else { return };
        let chain = store::branch_chain(&pid, Some(&self.branch));
        let from = self
            .timeline
            .selected
            .clone()
            .or_else(|| chain.last().map(|s| s.id.clone()));
        let Some(from) = from else {
            self.dialog = Some(Dialog::Message {
                title: "New branch".into(),
                text: "Capture at least one snapshot first — a branch starts from a moment in the timeline.".into(),
            });
            return;
        };
        let ts = store::get_snapshot(&pid, &from).map(|s| s.ts).unwrap_or(0.0);
        self.dialog = Some(Dialog::NewBranch {
            name: String::new(),
            from,
            from_ts: ts,
            error: None,
        });
    }

    pub fn create_branch(&mut self, name: &str, from: &str) {
        let Some(pid) = self.pid.clone() else { return };
        match store::add_branch(&pid, name, from) {
            Ok(branch) => {
                store::set_active_branch(&pid, &branch.id);
                self.branch = branch.id.clone();
                self.page = Page::Timeline;
                self.refresh(None);
            }
            Err(e) => {
                self.dialog = Some(Dialog::Message {
                    title: "TimeCapsule".into(),
                    text: e.to_string(),
                });
            }
        }
    }

    pub fn select_branch(&mut self, bid: &str) {
        let Some(pid) = self.pid.clone() else { return };
        if bid == self.branch {
            return;
        }
        store::set_active_branch(&pid, bid);
        self.branch = bid.to_string();
        self.refresh(None);
    }

    pub fn save_note(&mut self) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(sid) = self.timeline.note_sid.clone() else { return };
        let note = self.timeline.note.clone();
        if let Some(updated) = store::update_snapshot(&pid, &sid, Some(&note), None) {
            self.timeline.detail = Some(updated);
            self.timeline.note_dirty = false;
            self.timeline.note_status = "saved".into();
            self.refresh(Some(sid));
        }
    }

    pub fn create_project(&mut self, path: &str, name: &str, create: bool) {
        let expanded = store::expand_path(path);
        if create && !expanded.exists() {
            if let Err(e) = std::fs::create_dir_all(&expanded) {
                self.dialog = Some(Dialog::Message {
                    title: "TimeCapsule".into(),
                    text: format!("Could not create that folder:\n{}", e),
                });
                return;
            }
        }
        match store::add_project(path, Some(name)) {
            Ok(proj) => {
                self.pid = Some(proj.id);
                self.page = Page::Timeline;
                self.refresh(None);
            }
            Err(e) => {
                self.dialog = Some(Dialog::Message {
                    title: "TimeCapsule".into(),
                    text: e.to_string(),
                });
            }
        }
    }

    // -------------------------------------------------------------- shortcuts
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let mut events: Vec<&str> = Vec::new();
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::CTRL, Key::S) {
                events.push("snapshot");
            }
            if i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::C) {
                events.push("compare");
            }
            if i.consume_key(Modifiers::CTRL, Key::R) {
                events.push("restore");
            }
            if i.consume_key(Modifiers::CTRL, Key::T) {
                events.push("timeline");
            }
            if i.consume_key(Modifiers::CTRL, Key::D) {
                events.push("dashboard");
            }
            if i.consume_key(Modifiers::CTRL, Key::K) {
                events.push("search");
            }
        });
        for ev in events {
            match ev {
                "snapshot" => self.dialog_new_snapshot(),
                "compare" => self.compare_latest(),
                "restore" => self.restore_latest(),
                "timeline" => self.select_page(Page::Timeline),
                "dashboard" => self.select_page(Page::Dashboard),
                "search" => {
                    self.select_page(Page::Dashboard);
                    self.focus_search = true;
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.autosave_tick();
        if let Some((_, _, until)) = &self.toast {
            if store::now() > *until {
                self.toast = None;
            }
        }
        ctx.request_repaint_after(Duration::from_secs(1));

        // Background: a deep space gradient across the whole window.
        let screen = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(screen, 0, BG);
        let mid = screen.top() + screen.height() * 0.5;
        gradient_rect(
            &painter,
            Rect::from_min_max(screen.min, Pos2::new(screen.right(), mid)),
            egui::Color32::from_rgb(0x0b, 0x24, 0x37),
            egui::Color32::from_rgb(0x06, 0x12, 0x20),
        );
        gradient_rect(
            &painter,
            Rect::from_min_max(Pos2::new(screen.left(), mid), screen.max),
            egui::Color32::from_rgb(0x06, 0x12, 0x20),
            BG,
        );

        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(46.0)
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(0x04, 0x0a, 0x12)))
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    Rect::from_min_size(ui.min_rect().left_top(), Vec2::new(ui.available_width(), 1.0)),
                    0,
                    egui::Color32::from_rgb(0x0f, 0x24, 0x37),
                );
                self.ui_statusbar(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let side_rect = Rect::from_min_size(full.min, Vec2::new(252.0, full.height()));
                ui.painter().rect_filled(side_rect, 0, SIDEBAR);
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        Pos2::new(side_rect.right() - 1.0, side_rect.top()),
                        Vec2::new(1.0, side_rect.height()),
                    ),
                    0,
                    egui::Color32::from_rgb(0x0f, 0x24, 0x37),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(side_rect.shrink2(Vec2::new(16.0, 14.0))),
                    |ui| self.ui_sidebar(ui),
                );

                let right = Rect::from_min_max(
                    Pos2::new(side_rect.right() + 22.0, full.top() + 18.0),
                    Pos2::new(full.right() - 22.0, full.bottom() - 12.0),
                );
                ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(0.0, 12.0);
                    if self.pid.is_none() {
                        self.ui_welcome(ui);
                        return;
                    }
                    self.ui_hero(ui);
                    if self.page != Page::Dashboard {
                        self.ui_project_head(ui);
                    }
                    match self.page {
                        Page::Dashboard => self.ui_dashboard(ui),
                        Page::Timeline => self.ui_timeline(ui),
                        Page::Compare => self.ui_compare(ui),
                        Page::Stats => self.ui_stats(ui),
                        Page::Settings => self.ui_settings(ui),
                    }
                });
            });

        self.ui_dialogs(ctx);
        self.ui_toast(ctx);
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1120.0, 720.0])
            .with_title("TimeCapsule"),
        ..Default::default()
    };
    eframe::run_native(
        "TimeCapsule",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
