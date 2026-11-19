//! Storage engine for TimeCapsule.
//!
//! Projects live in a small JSON index, file contents go into a content addressed
//! object store (one copy per unique blob, shared by every snapshot) and each
//! snapshot records the path -> blob mapping it saw.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_FILE: u64 = 2 * 1024 * 1024;
pub const MAX_FILES: usize = 20000;

pub const IGNORE_DIRS: &[&str] = &[
    ".git", ".hg", ".svn", "node_modules", "__pycache__", ".venv", "venv", "env", ".idea",
    ".vscode", "dist", "build", ".mypy_cache", ".pytest_cache", ".ruff_cache", "target",
    ".next", ".nuxt", ".cache", ".gradle", ".terraform", "vendor", ".tox", "site-packages",
];
pub const IGNORE_FILES: &[&str] = &[".DS_Store", "Thumbs.db", "desktop.ini"];
pub const IGNORE_EXT: &[&str] = &[
    ".pyc", ".pyo", ".pyd", ".o", ".obj", ".class", ".so", ".dylib", ".dll", ".a", ".lib",
    ".exe", ".swp", ".swo", ".tmp", ".part",
];

#[derive(Debug, Clone)]
pub struct StoreError(pub String);

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StoreError {}

pub type Result<T> = std::result::Result<T, StoreError>;

fn err<T>(msg: impl Into<String>) -> Result<T> {
    Err(StoreError(msg.into()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileInfo {
    #[serde(default)]
    pub h: String,
    #[serde(default)]
    pub size: u64,
}

pub type FileMap = BTreeMap<String, FileInfo>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Snapshot {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub ts: f64,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub files: FileMap,
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    pub bytes: u64,
}

impl Snapshot {
    pub fn branch_id(&self) -> &str {
        if self.branch.is_empty() {
            "main"
        } else {
            &self.branch
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub created: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Branch {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub created: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_max_file")]
    pub max_file: u64,
    #[serde(default)]
    pub extra_ignore: String,
    #[serde(default)]
    pub auto_save: bool,
    #[serde(default = "default_minutes")]
    pub auto_save_minutes: i64,
    #[serde(default = "default_true")]
    pub auto_save_only_changes: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_file: MAX_FILE,
            extra_ignore: String::new(),
            auto_save: false,
            auto_save_minutes: 30,
            auto_save_only_changes: true,
        }
    }
}

fn default_max_file() -> u64 {
    MAX_FILE
}
fn default_minutes() -> i64 {
    30
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy)]
pub struct AutoConfig {
    pub enabled: bool,
    pub minutes: i64,
    pub only_changes: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Overview {
    pub counts: HashMap<String, usize>,
    pub per_day: HashMap<String, usize>,
    pub used: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Diff {
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub modified: Vec<String>,
    pub same: Vec<String>,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

impl Diff {
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        (self.added.len(), self.modified.len(), self.deleted.len(), self.same.len())
    }
}

#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub snapshots: usize,
    pub files: usize,
    pub live_bytes: u64,
    pub storage: u64,
    pub per_day: Vec<(String, usize)>,
    pub top_changed: Vec<(String, usize)>,
    pub span_days: f64,
    pub first: Option<f64>,
    pub last: Option<f64>,
    pub branch: String,
    pub branch_name: String,
    pub branches: usize,
}

pub fn home() -> PathBuf {
    if let Ok(v) = std::env::var("TIMECAPSULE_HOME") {
        if !v.is_empty() {
            return PathBuf::from(v);
        }
    }
    let h = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(h).join(".timecapsule")
}

pub fn projects_file() -> PathBuf {
    home().join("projects.json")
}
pub fn settings_file() -> PathBuf {
    home().join("settings.json")
}

fn ensure() {
    let _ = fs::create_dir_all(home());
}

fn read_json<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> T {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

fn write_json<T: Serialize>(path: &Path, data: &T) {
    ensure();
    let tmp = PathBuf::from(format!("{}.tmp", path.display()));
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    if data.serialize(&mut ser).is_ok() {
        if let Ok(mut f) = fs::File::create(&tmp) {
            if f.write_all(&buf).is_ok() {
                let _ = f.flush();
                drop(f);
                let _ = fs::rename(&tmp, path);
            }
        }
    }
}

// ------------------------------------------------------------------- settings

pub fn settings() -> Settings {
    let mut s = Settings::default();
    let data: serde_json::Value = read_json(&settings_file());
    if let Some(obj) = data.as_object() {
        if let Some(v) = obj.get("max_file").and_then(|v| v.as_u64()) {
            s.max_file = v;
        }
        if let Some(v) = obj.get("extra_ignore").and_then(|v| v.as_str()) {
            s.extra_ignore = v.to_string();
        }
        if let Some(v) = obj.get("auto_save").and_then(|v| v.as_bool()) {
            s.auto_save = v;
        }
        if let Some(v) = obj.get("auto_save_minutes").and_then(|v| v.as_i64()) {
            s.auto_save_minutes = v;
        }
        if let Some(v) = obj.get("auto_save_only_changes").and_then(|v| v.as_bool()) {
            s.auto_save_only_changes = v;
        }
    }
    s
}

pub fn save_settings(data: &Settings) {
    write_json(&settings_file(), data);
}

pub fn autosave_config() -> AutoConfig {
    let cfg = settings();
    AutoConfig {
        enabled: cfg.auto_save,
        minutes: cfg.auto_save_minutes.clamp(1, 1440),
        only_changes: cfg.auto_save_only_changes,
    }
}

// ------------------------------------------------------------------- projects

pub fn load_projects() -> Vec<Project> {
    let data: serde_json::Value = read_json(&projects_file());
    let mut out = Vec::new();
    if let Some(arr) = data.as_array() {
        for item in arr {
            if let Ok(p) = serde_json::from_value::<Project>(item.clone()) {
                if !p.id.is_empty() && !p.path.is_empty() {
                    out.push(p);
                }
            }
        }
    }
    out
}

pub fn save_projects(projects: &[Project]) {
    write_json(&projects_file(), &projects);
}

pub fn get_project(pid: &str) -> Option<Project> {
    load_projects().into_iter().find(|p| p.id == pid)
}

pub fn add_project(path: &str, name: Option<&str>) -> Result<Project> {
    let expanded = expand_path(path);
    if !expanded.is_dir() {
        return err("That folder does not exist.");
    }
    let abs = fs::canonicalize(&expanded).unwrap_or(expanded);
    let abs_str = abs.to_string_lossy().to_string();
    let mut projects = load_projects();
    if let Some(p) = projects.iter().find(|p| {
        let ep = expand_path(&p.path);
        fs::canonicalize(&ep).unwrap_or(ep).to_string_lossy() == abs_str
    }) {
        return Ok(p.clone());
    }
    let base = abs
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| abs_str.clone());
    let display = name.map(|n| n.trim()).filter(|n| !n.is_empty()).unwrap_or(&base);
    let proj = Project {
        id: uuid::Uuid::new_v4().simple().to_string()[..10].to_string(),
        name: display.to_string(),
        path: abs_str,
        created: now(),
        branch: None,
    };
    projects.push(proj.clone());
    save_projects(&projects);
    ensure();
    let _ = fs::create_dir_all(project_dir(&proj.id));
    Ok(proj)
}

pub fn expand_path(path: &str) -> PathBuf {
    let p = path.trim();
    if let Some(rest) = p.strip_prefix("~/") {
        let h = std::env::var("HOME").unwrap_or_default();
        return PathBuf::from(h).join(rest);
    }
    if p == "~" {
        return PathBuf::from(std::env::var("HOME").unwrap_or_default());
    }
    PathBuf::from(p)
}

pub fn remove_project(pid: &str) {
    let projects: Vec<Project> = load_projects().into_iter().filter(|p| p.id != pid).collect();
    save_projects(&projects);
    storage_cache().lock().unwrap().remove(pid);
    let _ = fs::remove_dir_all(project_dir(pid));
}

pub fn rename_project(pid: &str, name: &str) {
    let mut projects = load_projects();
    for p in projects.iter_mut() {
        if p.id == pid {
            let n = name.trim();
            if !n.is_empty() {
                p.name = n.to_string();
            }
        }
    }
    save_projects(&projects);
}

pub fn project_dir(pid: &str) -> PathBuf {
    home().join("projects").join(pid)
}
pub fn objects_dir(pid: &str) -> PathBuf {
    project_dir(pid).join("objects")
}
pub fn snapshots_file(pid: &str) -> PathBuf {
    project_dir(pid).join("snapshots.json")
}
pub fn branches_file(pid: &str) -> PathBuf {
    project_dir(pid).join("branches.json")
}

// -------------------------------------------------------------------- scanning

fn glob_match(pat: &str, name: &str) -> bool {
    fn inner(p: &[u8], n: &[u8]) -> bool {
        if p.is_empty() {
            return n.is_empty();
        }
        match p[0] {
            b'*' => {
                if inner(&p[1..], n) {
                    return true;
                }
                if !n.is_empty() {
                    return inner(p, &n[1..]);
                }
                false
            }
            b'?' => !n.is_empty() && inner(&p[1..], &n[1..]),
            b'[' => {
                if n.is_empty() {
                    return false;
                }
                let mut i = 1;
                let mut negate = false;
                if i < p.len() && (p[i] == b'!' || p[i] == b'^') {
                    negate = true;
                    i += 1;
                }
                let mut matched = false;
                let mut first = true;
                while i < p.len() && (p[i] != b']' || first) {
                    first = false;
                    if i + 2 < p.len() && p[i + 1] == b'-' && p[i + 2] != b']' {
                        if n[0] >= p[i] && n[0] <= p[i + 2] {
                            matched = true;
                        }
                        i += 3;
                    } else {
                        if p[i] == n[0] {
                            matched = true;
                        }
                        i += 1;
                    }
                }
                if i >= p.len() {
                    return false;
                }
                if matched != negate {
                    inner(&p[i + 1..], &n[1..])
                } else {
                    false
                }
            }
            c => !n.is_empty() && c == n[0] && inner(&p[1..], &n[1..]),
        }
    }
    inner(pat.as_bytes(), name.as_bytes())
}

fn ignored(name: &str, extra: &[String]) -> bool {
    extra.iter().any(|pat| glob_match(pat, name))
}

fn file_ext(name: &str) -> String {
    match name.rfind('.') {
        Some(i) if i > 0 || name.starts_with('.') => name[i..].to_lowercase(),
        _ => String::new(),
    }
}

pub fn scan_project(path: &Path) -> Vec<(String, PathBuf, u64)> {
    let cfg = settings();
    let extra: Vec<String> = cfg
        .extra_ignore
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    let max_file = if cfg.max_file == 0 { MAX_FILE } else { cfg.max_file };
    let mut found = Vec::new();
    let walker = walkdir::WalkDir::new(path)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !IGNORE_DIRS.contains(&name.as_ref()) && !ignored(&name, &extra)
            } else {
                true
            }
        });
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.depth() == 0 {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if IGNORE_FILES.contains(&name.as_str()) || ignored(&name, &extra) {
            continue;
        }
        if IGNORE_EXT.contains(&file_ext(&name).as_str()) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();
        if size > max_file {
            continue;
        }
        let ap = entry.path().to_path_buf();
        let rel = match ap.strip_prefix(path) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        found.push((rel, ap, size));
        if found.len() >= MAX_FILES {
            break;
        }
    }
    found
}

fn collect(pid: &str, store_objects: bool) -> Result<(FileMap, u64)> {
    let proj = get_project(pid).ok_or_else(|| StoreError("Unknown project.".into()))?;
    let path = PathBuf::from(&proj.path);
    if !path.is_dir() {
        return err(format!("The project folder is missing:\n{}", proj.path));
    }
    let mut files = FileMap::new();
    let mut total = 0u64;
    let objects = objects_dir(pid);
    for (rel, ap, _size) in scan_project(&path) {
        let data = match fs::read(&ap) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let digest = hex(&hasher.finalize());
        if store_objects {
            let blob = objects.join(&digest[..2]).join(&digest);
            if !blob.exists() {
                if let Some(parent) = blob.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&blob, &data);
            }
        }
        total += data.len() as u64;
        files.insert(rel, FileInfo { h: digest, size: data.len() as u64 });
    }
    Ok((files, total))
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ------------------------------------------------------------------- snapshots

pub fn load_snapshots(pid: &str) -> Vec<Snapshot> {
    let data: serde_json::Value = read_json(&snapshots_file(pid));
    let mut out = Vec::new();
    if let Some(arr) = data.as_array() {
        for item in arr {
            if let Ok(s) = serde_json::from_value::<Snapshot>(item.clone()) {
                if !s.id.is_empty() {
                    out.push(s);
                }
            }
        }
    }
    out.sort_by(|a, b| a.ts.partial_cmp(&b.ts).unwrap_or(std::cmp::Ordering::Equal));
    out
}

pub fn save_snapshots(pid: &str, snaps: &[Snapshot]) {
    write_json(&snapshots_file(pid), &snaps);
}

pub fn get_snapshot(pid: &str, sid: &str) -> Option<Snapshot> {
    load_snapshots(pid).into_iter().find(|s| s.id == sid)
}

pub fn previous_snapshot(pid: &str, sid: &str, branch: Option<&str>) -> Option<Snapshot> {
    let snap = get_snapshot(pid, sid)?;
    let bid = branch
        .map(|b| b.to_string())
        .unwrap_or_else(|| snap.branch_id().to_string());
    let chain = branch_chain(pid, Some(&bid));
    let idx = chain.iter().position(|s| s.id == sid)?;
    if idx == 0 {
        None
    } else {
        Some(chain[idx - 1].clone())
    }
}

pub fn take_snapshot(
    pid: &str,
    note: &str,
    title: &str,
    kind: &str,
    branch: Option<&str>,
) -> Result<Snapshot> {
    let (files, total) = collect(pid, true)?;
    if files.is_empty() {
        return err("Nothing to capture: no trackable files were found.");
    }
    let bid = branch
        .map(|b| b.to_string())
        .unwrap_or_else(|| active_branch(pid));
    let chain = branch_chain(pid, Some(&bid));
    let snap = Snapshot {
        id: uuid::Uuid::new_v4().simple().to_string()[..12].to_string(),
        ts: now(),
        note: note.trim().to_string(),
        title: title.trim().to_string(),
        kind: kind.to_string(),
        branch: bid,
        parent: chain.last().map(|s| s.id.clone()),
        count: files.len(),
        bytes: total,
        files,
    };
    let mut snaps = load_snapshots(pid);
    snaps.push(snap.clone());
    save_snapshots(pid, &snaps);
    Ok(snap)
}

pub fn update_snapshot(
    pid: &str,
    sid: &str,
    note: Option<&str>,
    title: Option<&str>,
) -> Option<Snapshot> {
    let mut snaps = load_snapshots(pid);
    let mut found = None;
    for s in snaps.iter_mut() {
        if s.id == sid {
            if let Some(n) = note {
                s.note = n.trim().to_string();
            }
            if let Some(t) = title {
                s.title = t.trim().to_string();
            }
            found = Some(s.clone());
            break;
        }
    }
    if found.is_some() {
        save_snapshots(pid, &snaps);
    }
    found
}

pub fn delete_snapshot(pid: &str, sid: &str) {
    let snaps = load_snapshots(pid);
    let parent = snaps
        .iter()
        .find(|s| s.id == sid)
        .and_then(|s| s.parent.clone());
    let mut rest: Vec<Snapshot> = snaps.into_iter().filter(|s| s.id != sid).collect();
    for s in rest.iter_mut() {
        if s.parent.as_deref() == Some(sid) {
            s.parent = parent.clone();
        }
    }
    save_snapshots(pid, &rest);
}

pub fn has_changes(pid: &str, branch: Option<&str>) -> bool {
    let bid = branch
        .map(|b| b.to_string())
        .unwrap_or_else(|| active_branch(pid));
    let chain = branch_chain(pid, Some(&bid));
    if chain.is_empty() {
        return true;
    }
    match collect(pid, false) {
        Ok((current, _)) => current != chain.last().unwrap().files,
        Err(_) => false,
    }
}

// -------------------------------------------------------------------- branches

pub fn load_branches(pid: &str) -> Vec<Branch> {
    let data: serde_json::Value = read_json(&branches_file(pid));
    let mut branches: Vec<Branch> = Vec::new();
    if let Some(arr) = data.as_array() {
        for item in arr {
            if let Ok(b) = serde_json::from_value::<Branch>(item.clone()) {
                if !b.id.is_empty() && !b.name.is_empty() {
                    branches.push(b);
                }
            }
        }
    }
    if !branches.iter().any(|b| b.id == "main") {
        let created = get_project(pid).map(|p| p.created).unwrap_or_else(now);
        branches.insert(
            0,
            Branch { id: "main".into(), name: "main".into(), base: None, created },
        );
    }
    branches
}

pub fn save_branches(pid: &str, branches: &[Branch]) {
    write_json(&branches_file(pid), &branches);
}

pub fn get_branch(pid: &str, bid: &str) -> Option<Branch> {
    load_branches(pid).into_iter().find(|b| b.id == bid)
}

pub fn active_branch(pid: &str) -> String {
    let bid = get_project(pid)
        .and_then(|p| p.branch)
        .unwrap_or_else(|| "main".into());
    if get_branch(pid, &bid).is_some() {
        bid
    } else {
        "main".into()
    }
}

pub fn set_active_branch(pid: &str, bid: &str) {
    let mut projects = load_projects();
    for p in projects.iter_mut() {
        if p.id == pid {
            p.branch = Some(bid.to_string());
        }
    }
    save_projects(&projects);
}

pub fn add_branch(pid: &str, name: &str, from_sid: &str) -> Result<Branch> {
    let name = name.trim();
    if name.is_empty() {
        return err("Give the branch a name.");
    }
    let mut branches = load_branches(pid);
    if branches.iter().any(|b| b.name.eq_ignore_ascii_case(name)) {
        return err(format!("A branch called “{name}” already exists."));
    }
    if get_snapshot(pid, from_sid).is_none() {
        return err("Pick the moment the branch should start from.");
    }
    let branch = Branch {
        id: uuid::Uuid::new_v4().simple().to_string()[..8].to_string(),
        name: name.to_string(),
        base: Some(from_sid.to_string()),
        created: now(),
    };
    branches.push(branch.clone());
    save_branches(pid, &branches);
    Ok(branch)
}

pub fn branch_chain(pid: &str, bid: Option<&str>) -> Vec<Snapshot> {
    let bid = bid.map(|b| b.to_string()).unwrap_or_else(|| active_branch(pid));
    let snaps = load_snapshots(pid);
    let by_id: HashMap<String, Snapshot> =
        snaps.iter().map(|s| (s.id.clone(), s.clone())).collect();
    let branch = get_branch(pid, &bid).unwrap_or_default();
    let heads: Vec<&Snapshot> = snaps.iter().filter(|s| s.branch_id() == bid).collect();
    let mut head_id = heads
        .last()
        .map(|s| s.id.clone())
        .or_else(|| branch.base.clone());
    let mut seen: HashSet<String> = HashSet::new();
    let mut chain: Vec<Snapshot> = Vec::new();
    while let Some(hid) = head_id.clone() {
        if seen.contains(&hid) {
            break;
        }
        seen.insert(hid.clone());
        let snap = match by_id.get(&hid) {
            Some(s) => s.clone(),
            None => break,
        };
        chain.push(snap.clone());
        let mut parent = snap.parent.clone();
        if parent.is_none() {
            let own = snap.branch_id().to_string();
            let earlier = snaps
                .iter()
                .filter(|s| {
                    !seen.contains(&s.id) && s.ts <= snap.ts && s.branch_id() == own
                })
                .max_by(|a, b| a.ts.partial_cmp(&b.ts).unwrap_or(std::cmp::Ordering::Equal));
            parent = earlier
                .map(|s| s.id.clone())
                .or_else(|| get_branch(pid, &own).and_then(|b| b.base));
        }
        head_id = parent;
    }
    chain.sort_by(|a, b| a.ts.partial_cmp(&b.ts).unwrap_or(std::cmp::Ordering::Equal));
    chain
}

// ------------------------------------------------------------------------ blobs

pub fn blob_path(pid: &str, digest: &str) -> PathBuf {
    if digest.len() < 2 {
        return objects_dir(pid).join(digest);
    }
    objects_dir(pid).join(&digest[..2]).join(digest)
}

pub fn read_blob(pid: &str, digest: &str) -> Vec<u8> {
    fs::read(blob_path(pid, digest)).unwrap_or_default()
}

static STORAGE_CACHE: OnceLock<Mutex<HashMap<String, ((u64, u64), u64)>>> = OnceLock::new();

fn storage_cache() -> &'static Mutex<HashMap<String, ((u64, u64), u64)>> {
    STORAGE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn file_key(path: &Path) -> Option<(u64, u64)> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let dur = modified.duration_since(UNIX_EPOCH).ok()?;
    Some((dur.as_nanos() as u64, meta.len()))
}

pub fn storage_bytes(pid: &str) -> u64 {
    let key = file_key(&snapshots_file(pid));
    let mut cache = storage_cache().lock().unwrap();
    if let Some(k) = key {
        if let Some((cached_key, bytes)) = cache.get(pid) {
            if *cached_key == k {
                return *bytes;
            }
        }
    }
    let mut total = 0u64;
    if let Ok(walker) = fs::read_dir(objects_dir(pid)) {
        for entry in walker.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Ok(sub) = fs::read_dir(&p) {
                    for f in sub.flatten() {
                        if let Ok(m) = f.metadata() {
                            if m.is_file() {
                                total += m.len();
                            }
                        }
                    }
                }
            } else if let Ok(m) = entry.metadata() {
                total += m.len();
            }
        }
    }
    if let Some(k) = key {
        cache.insert(pid.to_string(), (k, total));
    }
    total
}

pub fn library_stats() -> Overview {
    let mut out = Overview::default();
    for proj in load_projects() {
        let snaps = load_snapshots(&proj.id);
        out.counts.insert(proj.id.clone(), snaps.len());
        out.used += storage_bytes(&proj.id);
        for snap in &snaps {
            *out.per_day.entry(day_key(snap.ts)).or_insert(0) += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------- compare

pub fn compare(a: &Snapshot, b: &Snapshot) -> Diff {
    let mut d = Diff::default();
    for p in b.files.keys() {
        if !a.files.contains_key(p) {
            d.added.push(p.clone());
        }
    }
    for p in a.files.keys() {
        if !b.files.contains_key(p) {
            d.deleted.push(p.clone());
        }
    }
    for (p, info) in b.files.iter() {
        if let Some(old) = a.files.get(p) {
            if old.h != info.h {
                d.modified.push(p.clone());
            } else {
                d.same.push(p.clone());
            }
        }
    }
    d.added.sort();
    d.deleted.sort();
    d.modified.sort();
    d.bytes_in = b.files.values().map(|f| f.size).sum();
    d.bytes_out = a.files.values().map(|f| f.size).sum();
    d
}

pub fn ordered_changes(d: &Diff) -> Vec<(String, &'static str)> {
    let mut rows: Vec<(String, &'static str)> = Vec::new();
    for rel in &d.modified {
        rows.push((rel.clone(), "modified"));
    }
    for rel in &d.added {
        rows.push((rel.clone(), "added"));
    }
    for rel in &d.deleted {
        rows.push((rel.clone(), "deleted"));
    }
    rows
}

// ---------------------------------------------------------------------- restore

pub fn preview_restore(pid: &str, snap: &Snapshot) -> Result<(Vec<String>, Vec<String>)> {
    let (current, _) = collect(pid, false)?;
    let mut write: Vec<String> = snap
        .files
        .iter()
        .filter(|(p, info)| current.get(*p).map(|c| &c.h) != Some(&info.h))
        .map(|(p, _)| p.clone())
        .collect();
    write.sort();
    let mut delete: Vec<String> = current
        .keys()
        .filter(|p| !snap.files.contains_key(*p))
        .cloned()
        .collect();
    delete.sort();
    Ok((write, delete))
}

pub fn restore(
    pid: &str,
    snap: &Snapshot,
    whole: bool,
    rels: Option<&[String]>,
    safety: bool,
) -> Result<(Vec<(String, String)>, Option<Snapshot>)> {
    let proj = get_project(pid).ok_or_else(|| StoreError("Unknown project.".into()))?;
    let path = PathBuf::from(&proj.path);
    if !path.is_dir() {
        return err(format!("The project folder is missing:\n{}", proj.path));
    }
    let mut created = None;
    if safety && whole {
        created = take_snapshot(
            pid,
            "Safety copy before restoring an earlier snapshot",
            "",
            "manual",
            None,
        )
        .ok();
    }
    let mut actions = Vec::new();
    let targets: Vec<String> = if whole {
        snap.files.keys().cloned().collect()
    } else {
        rels.map(|r| r.to_vec()).unwrap_or_default()
    };
    for rel in targets {
        let info = match snap.files.get(&rel) {
            Some(i) => i,
            None => continue,
        };
        let dest = path.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if dest.is_file() {
            if let Ok(meta) = fs::metadata(&dest) {
                if meta.len() == info.size {
                    if let Ok(data) = fs::read(&dest) {
                        let mut hasher = Sha256::new();
                        hasher.update(&data);
                        if hex(&hasher.finalize()) == info.h {
                            continue;
                        }
                    }
                }
            }
        }
        let data = read_blob(pid, &info.h);
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::write(&dest, &data).is_ok() {
            actions.push(("restored".to_string(), rel));
        }
    }
    if whole {
        if let Ok((current, _)) = collect(pid, false) {
            for rel in current.keys() {
                if !snap.files.contains_key(rel) {
                    let p = path.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
                    if fs::remove_file(p).is_ok() {
                        actions.push(("removed".to_string(), rel.clone()));
                    }
                }
            }
        }
    }
    Ok((actions, created))
}

// ------------------------------------------------------------------------ stats

pub fn project_stats(pid: &str, branch: Option<&str>) -> ProjectStats {
    let bid = branch
        .map(|b| b.to_string())
        .unwrap_or_else(|| active_branch(pid));
    let snaps = branch_chain(pid, Some(&bid));
    let proj = get_project(pid);
    let mut live = FileMap::new();
    if let Some(p) = &proj {
        if Path::new(&p.path).is_dir() {
            if let Ok((files, _)) = collect(pid, false) {
                live = files;
            }
        }
    }
    let today = Local::now().date_naive();
    let mut per_day = Vec::new();
    for back in (0..14).rev() {
        let day = today - chrono::Duration::days(back);
        let start = Local
            .from_local_datetime(&day.and_hms_opt(0, 0, 0).unwrap())
            .single()
            .map(|d| d.timestamp() as f64)
            .unwrap_or(0.0);
        let cnt = snaps.iter().filter(|s| s.ts >= start && s.ts < start + 86400.0).count();
        per_day.push((day.format("%d/%m").to_string(), cnt));
    }
    let mut changed: HashMap<String, usize> = HashMap::new();
    let mut prev: Option<&Snapshot> = None;
    for s in &snaps {
        if let Some(p) = prev {
            let d = compare(p, s);
            for rel in &d.modified {
                *changed.entry(rel.clone()).or_insert(0) += 1;
            }
            for rel in &d.added {
                changed.entry(rel.clone()).or_insert(0);
            }
        } else {
            for rel in s.files.keys() {
                changed.entry(rel.clone()).or_insert(0);
            }
        }
        prev = Some(s);
    }
    let mut top: Vec<(String, usize)> = changed.into_iter().filter(|(_, v)| *v > 0).collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(8);
    let span = if snaps.len() > 1 {
        (snaps.last().unwrap().ts - snaps.first().unwrap().ts) / 86400.0
    } else {
        0.0
    };
    let branches = load_branches(pid);
    ProjectStats {
        snapshots: snaps.len(),
        files: live.len(),
        live_bytes: live.values().map(|f| f.size).sum(),
        storage: storage_bytes(pid),
        per_day,
        top_changed: top,
        span_days: span,
        first: snaps.first().map(|s| s.ts),
        last: snaps.last().map(|s| s.ts),
        branch_name: get_branch(pid, &bid).map(|b| b.name).unwrap_or_else(|| "main".into()),
        branch: bid,
        branches: branches.len(),
    }
}

// ------------------------------------------------------------------------- time

pub fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub fn fmt_bytes(n: u64) -> String {
    let mut v = n as f64;
    for unit in ["B", "KB", "MB", "GB"] {
        if v < 1024.0 || unit == "GB" {
            return if unit == "B" {
                format!("{:.0} {}", v, unit)
            } else {
                format!("{:.1} {}", v, unit)
            };
        }
        v /= 1024.0;
    }
    format!("{:.1} GB", v)
}

fn local(ts: f64) -> chrono::DateTime<Local> {
    Local
        .timestamp_opt(ts as i64, 0)
        .single()
        .unwrap_or_else(Local::now)
}

pub fn fmt_time(ts: f64) -> String {
    local(ts).format("%H:%M").to_string()
}
pub fn fmt_date(ts: f64) -> String {
    local(ts).format("%d %b %Y").to_string()
}
pub fn fmt_stamp(ts: f64) -> String {
    local(ts).format("%d %b %Y · %H:%M").to_string()
}
pub fn day_key(ts: f64) -> String {
    local(ts).format("%Y-%m-%d").to_string()
}
pub fn day_label(ts: f64) -> String {
    let d = local(ts).date_naive();
    let today = Local::now().date_naive();
    if d == today {
        "Today".into()
    } else if d == today - chrono::Duration::days(1) {
        "Yesterday".into()
    } else {
        local(ts).format("%A, %d %B %Y").to_string()
    }
}

pub fn clock_label(ts: f64) -> String {
    let d = local(ts);
    let hour = match d.hour() % 12 {
        0 => 12,
        h => h,
    };
    let ampm = if d.hour() < 12 { "AM" } else { "PM" };
    format!("{}:{:02} {}", hour, d.minute(), ampm)
}

pub fn short_day(ts: f64) -> String {
    let label = day_label(ts);
    if label.starts_with("Today") || label.starts_with("Yesterday") {
        label
    } else {
        local(ts).format("%b %d").to_string()
    }
}

pub fn ago(ts: f64) -> String {
    let diff = now() - ts;
    if diff < 60.0 {
        "just now".into()
    } else if diff < 3600.0 {
        format!("{} min ago", (diff / 60.0) as i64)
    } else if diff < 86400.0 {
        format!("{} h ago", (diff / 3600.0) as i64)
    } else {
        let days = (diff / 86400.0) as i64;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}
