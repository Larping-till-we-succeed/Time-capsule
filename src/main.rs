//! TimeCapsule 0.0 — a time machine for project folders, in Rust.

mod cli;
mod diff;
mod preview;
mod store;
mod theme;
mod ui;

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};
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
    pub title: String,
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
        self.title.clear();
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
    /// Largest file a snapshot may capture, in megabytes.
    pub max_mb: u64,
    /// Comma separated names or globs to leave out of every snapshot.
    pub extra_ignore: String,
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
        keep_newer: bool,
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
    RenameProject {
        pid: String,
        name: String,
    },
    ConfirmProject {
        pid: String,
        name: String,
    },
}

pub enum TaskOutcome {
    Snapshot {
        pid: String,
        result: store::Result<(Snapshot, Vec<store::Skipped>)>,
    },
    Restore {
        pid: String,
        whole: bool,
        rel: Option<String>,
        result: store::Result<(Vec<(String, String)>, Option<Snapshot>)>,
    },
    Gc {
        dry_run: bool,
        result: store::Result<store::GcReport>,
    },
    Verify(store::VerifyReport),
}

/// A job running on a worker thread; the UI keeps drawing while it works.
pub struct Task {
    pub label: String,
    pub progress: store::Progress,
    rx: Receiver<TaskOutcome>,
    pub started: f64,
}

impl Task {
    pub fn counts(&self) -> (usize, usize) {
        self.progress.counts()
    }
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
    /// Whether the left navigation panel is shown (the top bar's menu button).
    pub sidebar_open: bool,
    pub project_head: Option<(String, String, usize, String, String)>,
    pub status_storage: (u64, u64),
    pub gc_preview: Option<store::GcReport>,
    pub verify_report: Option<store::VerifyReport>,
    /// Files the last capture had to leave out; shown in the snapshot detail.
    pub skipped: Vec<store::Skipped>,
    pub task: Option<Task>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        install_style(&cc.egui_ctx);
        let cfg = store::settings();
        let auto = store::autosave_config();
        let max_file = if cfg.max_file == 0 {
            store::MAX_FILE
        } else {
            cfg.max_file
        };
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
                max_mb: (max_file / (1024 * 1024)).clamp(1, 200),
                extra_ignore: cfg.extra_ignore.clone(),
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
            sidebar_open: true,
            project_head: None,
            status_storage: (0, 0),
            gc_preview: None,
            verify_report: None,
            skipped: Vec::new(),
            task: None,
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

    pub fn page_title(&self) -> &'static str {
        match self.page {
            Page::Dashboard => "Dashboard",
            Page::Timeline => "Timeline",
            Page::Compare => "Compare",
            Page::Stats => "Statistics",
            Page::Settings => "Settings",
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
            self.timeline.title.clear();
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
            self.timeline.title = snap.title.clone();
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

    // ------------------------------------------------------------- background
    pub fn busy(&self) -> bool {
        self.task.is_some()
    }

    fn spawn_task(
        &mut self,
        label: impl Into<String>,
        work: impl FnOnce(&store::Progress) -> TaskOutcome + Send + 'static,
    ) {
        let (tx, rx) = std::sync::mpsc::channel();
        let progress = store::Progress::new();
        let worker = progress.clone();
        std::thread::spawn(move || {
            let outcome = work(&worker);
            let _ = tx.send(outcome);
        });
        self.task = Some(Task {
            label: label.into(),
            progress,
            rx,
            started: store::now(),
        });
    }

    pub fn cancel_task(&mut self) {
        if let Some(task) = &self.task {
            task.progress.cancel();
        }
        self.task = None;
        self.notify(
            "Cancelled",
            "The background job stopped. Anything it had not finished was left untouched.",
        );
    }

    pub fn poll_task(&mut self) {
        let received = self.task.as_ref().and_then(|task| match task.rx.try_recv() {
            Ok(outcome) => Some(Ok(outcome)),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(())),
        });
        match received {
            Some(Ok(outcome)) => {
                self.task = None;
                self.handle_task(outcome);
            }
            Some(Err(())) => self.task = None,
            None => {}
        }
    }

    fn handle_task(&mut self, outcome: TaskOutcome) {
        match outcome {
            TaskOutcome::Snapshot { result, .. } => match result {
                Ok((snap, skipped)) => {
                    self.skipped = skipped;
                    self.page = Page::Timeline;
                    self.refresh(Some(snap.id));
                    self.notify(
                        "Snapshot captured",
                        format!(
                            "{} files · {}",
                            snap.count,
                            store::fmt_bytes(snap.bytes)
                        ),
                    );
                }
                Err(e) => {
                    let text = e.to_string();
                    if text != store::CANCELLED {
                        self.dialog = Some(Dialog::Message {
                            title: "TimeCapsule".into(),
                            text,
                        });
                    }
                }
            },
            TaskOutcome::Gc { dry_run, result } => match result {
                Ok(report) => {
                    if !dry_run {
                        self.verify_report = None;
                        self.notify(
                            "Storage cleaned up",
                            format!(
                                "Removed {} unused blob{} and freed {}.",
                                report.removed,
                                if report.removed == 1 { "" } else { "s" },
                                store::fmt_bytes(report.freed)
                            ),
                        );
                    }
                    self.gc_preview = Some(report);
                }
                Err(e) => {
                    let text = e.to_string();
                    if text != store::CANCELLED {
                        self.notify("Storage maintenance", text);
                    }
                }
            },
            TaskOutcome::Restore {
                pid,
                whole,
                rel,
                result,
            } => {
                if self.pid.as_deref() == Some(pid.as_str()) {
                    self.refresh(None);
                }
                match result {
                    Ok((actions, safety_snap)) => {
                        let count = |kind: &str| {
                            actions.iter().filter(|(k, _)| k == kind).count()
                        };
                        let restored = count("restored");
                        let removed = count("removed");
                        let kept = count("kept");
                        let text = if whole {
                            let mut text = format!(
                                "{} file{} restored.{}{}{}",
                                restored,
                                if restored == 1 { "" } else { "s" },
                                if removed > 0 {
                                    format!(
                                        "\n{} file{} this snapshot does not contain were removed.",
                                        removed,
                                        if removed == 1 { "" } else { "s" }
                                    )
                                } else {
                                    String::new()
                                },
                                if kept > 0 {
                                    format!(
                                        "\n{} newer file{} were kept.",
                                        kept,
                                        if kept == 1 { "" } else { "s" }
                                    )
                                } else {
                                    String::new()
                                },
                                if safety_snap.is_some() {
                                    "\nThe previous state was kept as a safety snapshot."
                                } else {
                                    ""
                                }
                            );
                            text.shrink_to_fit();
                            text
                        } else {
                            let name = rel.unwrap_or_default();
                            if actions.is_empty() {
                                format!("{} already matches this snapshot.", name)
                            } else {
                                format!("{} restored.", name)
                            }
                        };
                        self.dialog = Some(Dialog::Message {
                            title: "Restored".into(),
                            text,
                        });
                    }
                    Err(e) => {
                        let text = e.to_string();
                        if text == store::CANCELLED {
                            self.notify(
                                "Restore cancelled",
                                "Files already replaced stay as they are; the safety snapshot holds the state from before.",
                            );
                        } else {
                            self.dialog = Some(Dialog::Message {
                                title: "TimeCapsule".into(),
                                text: format!("Restore failed:\n{}", e),
                            });
                        }
                    }
                }
            }
            TaskOutcome::Verify(report) => {
                self.verify_report = Some(report);
                self.gc_preview = None;
            }
        }
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
        let max_file = if cfg.max_file == 0 {
            store::MAX_FILE
        } else {
            cfg.max_file
        };
        self.settings.max_mb = (max_file / (1024 * 1024)).clamp(1, 200);
        self.settings.extra_ignore = cfg.extra_ignore.clone();
    }

    pub fn save_settings(&mut self) {
        let mut cfg = store::settings();
        cfg.auto_save = self.settings.enabled;
        cfg.auto_save_minutes = self.settings.minutes.clamp(1, 1440);
        cfg.auto_save_only_changes = self.settings.only_changes;
        cfg.max_file = self.settings.max_mb.clamp(1, 200) * 1024 * 1024;
        cfg.extra_ignore = self.settings.extra_ignore.trim().to_string();
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
        if !self.auto.enabled || self.busy() {
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
        let report = store::scan_project(std::path::Path::new(&proj.path));
        let mut preview = format!(
            "{} tracked files · {} will be captured. Existing content is reused from the project store, so repeated snapshots stay small.",
            report.files.len(),
            store::fmt_bytes(report.total_bytes())
        );
        if !report.skipped.is_empty() {
            preview.push_str(&format!(
                "\n{} — adjust the limit or the ignore patterns in Settings.",
                store::skip_summary(&report.skipped)
            ));
        }
        self.dialog = Some(Dialog::NewSnapshot {
            title: String::new(),
            note: String::new(),
            preview,
            error: None,
        });
    }

    pub fn capture_snapshot(&mut self, title: &str, note: &str) {
        if self.busy() {
            self.notify(
                "Still working",
                "Wait for the current job to finish, or cancel it in the status bar.",
            );
            return;
        }
        let Some(pid) = self.pid.clone() else { return };
        let branch = self.branch.clone();
        let title = title.to_string();
        let note = note.to_string();
        let name = self
            .project()
            .map(|p| p.name)
            .unwrap_or_else(|| "project".into());
        self.spawn_task(format!("Capturing {}", name), move |progress| {
            let result = store::take_snapshot_ctl(
                &pid,
                &note,
                &title,
                "manual",
                Some(&branch),
                Some(progress),
            );
            TaskOutcome::Snapshot { pid, result }
        });
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
                    keep_newer: false,
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

    /// Hand the restore to a worker thread so a large project cannot freeze the
    /// window; `handle_task` reports the result when it lands.
    pub fn complete_restore(
        &mut self,
        sid: &str,
        whole: bool,
        rel: Option<&str>,
        safety: bool,
        keep_newer: bool,
    ) {
        if self.busy() {
            self.notify(
                "Still working",
                "Wait for the current job to finish, or cancel it in the status bar.",
            );
            return;
        }
        let Some(pid) = self.pid.clone() else { return };
        let Some(snap) = store::get_snapshot(&pid, sid) else { return };
        let rels = rel.map(|r| vec![r.to_string()]);
        let rel = rel.map(|r| r.to_string());
        let label = match &rel {
            Some(name) => format!("Restoring {}", name),
            None => format!("Restoring snapshot {}", sid),
        };
        let worker_pid = pid.clone();
        self.spawn_task(label, move |progress| {
            let result = store::restore(
                &worker_pid,
                &snap,
                whole,
                rels.as_deref(),
                safety,
                !keep_newer,
                Some(progress),
            );
            TaskOutcome::Restore {
                pid: worker_pid,
                whole,
                rel,
                result,
            }
        });
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

    pub fn save_note_and_title(&mut self) {
        let Some(pid) = self.pid.clone() else { return };
        let Some(sid) = self.timeline.note_sid.clone() else { return };
        let note = self.timeline.note.clone();
        let title = self.timeline.title.clone();
        if let Some(updated) = store::update_snapshot(&pid, &sid, Some(&note), Some(&title)) {
            self.timeline.detail = Some(updated);
            self.timeline.note_dirty = false;
            self.refresh(Some(sid));
            self.timeline.note_status = "saved".into();
        }
    }

    pub fn rename_project(&mut self, pid: &str, name: &str) {
        if name.trim().is_empty() {
            return;
        }
        store::rename_project(pid, name);
        self.refresh(None);
        self.notify("Project renamed", name.trim().to_string());
    }

    /// Forget a project inside TimeCapsule. The folder on disk is never touched.
    pub fn remove_project(&mut self, pid: &str) {
        let name = store::get_project(pid)
            .map(|p| p.name)
            .unwrap_or_else(|| "The project".into());
        store::remove_project(pid);
        if self.pid.as_deref() == Some(pid) {
            self.pid = None;
            self.page = Page::Dashboard;
            self.timeline.reset();
            self.compare = CompareState::default();
            self.stats = None;
        }
        self.refresh(None);
        self.notify(
            "Project removed",
            format!(
                "{} was removed from TimeCapsule. Files in its folder are untouched.",
                name
            ),
        );
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
            if i.consume_key(Modifiers::CTRL, Key::B) {
                events.push("sidebar");
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
                "sidebar" => self.sidebar_open = !self.sidebar_open,
                _ => {}
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.poll_task();
        self.autosave_tick();
        if let Some((_, _, until)) = &self.toast {
            if store::now() > *until {
                self.toast = None;
            }
        }
        ctx.request_repaint_after(if self.busy() {
            Duration::from_millis(150)
        } else {
            Duration::from_secs(1)
        });

        // Flat app background; the sidebar paints its own panel over it.
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(ctx.screen_rect(), 0, BG);

        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(42.0)
            .frame(egui::Frame::NONE.fill(SIDEBAR))
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    Rect::from_min_size(ui.min_rect().left_top(), Vec2::new(ui.available_width(), 1.0)),
                    0,
                    BORDER,
                );
                self.ui_statusbar(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let side_w = if self.sidebar_open { 248.0 } else { 0.0 };
                if side_w > 0.0 {
                    let side_rect = Rect::from_min_size(full.min, Vec2::new(side_w, full.height()));
                    ui.painter().rect_filled(side_rect, 0, SIDEBAR);
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            Pos2::new(side_rect.right() - 1.0, side_rect.top()),
                            Vec2::new(1.0, side_rect.height()),
                        ),
                        0,
                        BORDER,
                    );
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .max_rect(side_rect.shrink2(Vec2::new(16.0, 18.0))),
                        |ui| self.ui_sidebar(ui),
                    );
                }

                let right = Rect::from_min_max(
                    Pos2::new(full.left() + side_w + if side_w > 0.0 { 24.0 } else { 20.0 }, full.top() + 14.0),
                    Pos2::new(full.right() - 24.0, full.bottom() - 14.0),
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        std::process::exit(cli::run(&args));
    }
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
