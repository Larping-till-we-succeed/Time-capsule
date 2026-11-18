"""Storage engine for TimeCapsule.

Projects live in a small JSON index, file contents go into a content addressed
object store (one copy per unique blob, shared by every snapshot) and each
snapshot records the path -> blob mapping it saw.
"""

import fnmatch
import hashlib
import json
import os
import shutil
import time
import uuid
from datetime import date, datetime, timedelta

HOME = os.environ.get("TIMECAPSULE_HOME") or os.path.join(os.path.expanduser("~"), ".timecapsule")
PROJECTS_FILE = os.path.join(HOME, "projects.json")
SETTINGS_FILE = os.path.join(HOME, "settings.json")

IGNORE_DIRS = {
    ".git", ".hg", ".svn", "node_modules", "__pycache__", ".venv", "venv", "env", ".idea",
    ".vscode", "dist", "build", ".mypy_cache", ".pytest_cache", ".ruff_cache", "target",
    ".next", ".nuxt", ".cache", ".gradle", ".terraform", "vendor", ".tox", "site-packages",
}
IGNORE_FILES = {".DS_Store", "Thumbs.db", "desktop.ini"}
IGNORE_EXT = {".pyc", ".pyo", ".pyd", ".o", ".obj", ".class", ".so", ".dylib", ".dll", ".a",
              ".lib", ".exe", ".swp", ".swo", ".tmp", ".part"}
MAX_FILE = 2 * 1024 * 1024
MAX_FILES = 20000
DEFAULT_SETTINGS = {
    "max_file": MAX_FILE,
    "extra_ignore": "",
    "auto_save": False,
    "auto_save_minutes": 30,
    "auto_save_only_changes": True,
}


class StoreError(Exception):
    pass


# --------------------------------------------------------------------------- io

def _ensure():
    os.makedirs(HOME, exist_ok=True)


def _read_json(path, default):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return default


def _write_json(path, data):
    _ensure()
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=1)
    os.replace(tmp, path)


def settings():
    s = dict(DEFAULT_SETTINGS)
    data = _read_json(SETTINGS_FILE, {})
    if isinstance(data, dict):
        for k in DEFAULT_SETTINGS:
            if k in data:
                s[k] = data[k]
    return s


def save_settings(data):
    _write_json(SETTINGS_FILE, {k: data[k] for k in DEFAULT_SETTINGS if k in data})


def autosave_config():
    """Normalised auto-save settings: enabled, interval in minutes, skip unchanged."""
    cfg = settings()
    try:
        minutes = int(cfg.get("auto_save_minutes") or 30)
    except (TypeError, ValueError):
        minutes = 30
    return {
        "enabled": bool(cfg.get("auto_save")),
        "minutes": max(1, min(1440, minutes)),
        "only_changes": bool(cfg.get("auto_save_only_changes", True)),
    }


# --------------------------------------------------------------------- projects

def load_projects():
    data = _read_json(PROJECTS_FILE, [])
    return [p for p in data if isinstance(p, dict) and p.get("id") and p.get("path")]


def save_projects(projects):
    _write_json(PROJECTS_FILE, projects)


def get_project(pid):
    for p in load_projects():
        if p["id"] == pid:
            return p
    return None


def add_project(path, name=None):
    path = os.path.abspath(os.path.expanduser(path))
    if not os.path.isdir(path):
        raise StoreError("That folder does not exist.")
    for p in load_projects():
        if os.path.abspath(p["path"]) == path:
            return p
    proj = {
        "id": uuid.uuid4().hex[:10],
        "name": (name or os.path.basename(path.rstrip(os.sep)) or path).strip(),
        "path": path,
        "created": time.time(),
    }
    projects = load_projects()
    projects.append(proj)
    save_projects(projects)
    _ensure()
    os.makedirs(project_dir(proj["id"]), exist_ok=True)
    return proj


def remove_project(pid):
    save_projects([p for p in load_projects() if p["id"] != pid])
    _STORAGE_CACHE.pop(pid, None)
    shutil.rmtree(project_dir(pid), ignore_errors=True)


def rename_project(pid, name):
    projects = load_projects()
    for p in projects:
        if p["id"] == pid:
            p["name"] = name.strip() or p["name"]
    save_projects(projects)


def project_dir(pid):
    return os.path.join(HOME, "projects", pid)


def objects_dir(pid):
    return os.path.join(project_dir(pid), "objects")


def snapshots_file(pid):
    return os.path.join(project_dir(pid), "snapshots.json")


def branches_file(pid):
    return os.path.join(project_dir(pid), "branches.json")


# --------------------------------------------------------------------- scanning

def _ignored(name, extra):
    return any(fnmatch.fnmatch(name, pat) for pat in extra)


def scan_project(path):
    """Return [(relpath, abspath, size)] for every file TimeCapsule tracks."""
    cfg = settings()
    extra = [p.strip() for p in (cfg.get("extra_ignore") or "").split(",") if p.strip()]
    max_file = int(cfg.get("max_file") or MAX_FILE)
    found = []
    for root, dirs, files in os.walk(path):
        dirs[:] = [d for d in sorted(dirs) if d not in IGNORE_DIRS and not _ignored(d, extra)]
        for fn in sorted(files):
            if fn in IGNORE_FILES or _ignored(fn, extra):
                continue
            if os.path.splitext(fn)[1].lower() in IGNORE_EXT:
                continue
            ap = os.path.join(root, fn)
            try:
                if os.path.islink(ap) or not os.path.isfile(ap):
                    continue
                size = os.path.getsize(ap)
            except OSError:
                continue
            if size > max_file:
                continue
            found.append((os.path.relpath(ap, path).replace(os.sep, "/"), ap, size))
            if len(found) >= MAX_FILES:
                return found
    return found


def _collect(pid, store_objects=True):
    proj = get_project(pid)
    if proj is None:
        raise StoreError("Unknown project.")
    path = proj["path"]
    if not os.path.isdir(path):
        raise StoreError(f"The project folder is missing:\n{path}")
    files = {}
    total = 0
    objects = objects_dir(pid)
    for rel, ap, _size in scan_project(path):
        try:
            with open(ap, "rb") as f:
                data = f.read()
        except OSError:
            continue
        digest = hashlib.sha256(data).hexdigest()
        if store_objects:
            blob = os.path.join(objects, digest[:2], digest)
            if not os.path.exists(blob):
                os.makedirs(os.path.dirname(blob), exist_ok=True)
                with open(blob, "wb") as f:
                    f.write(data)
        files[rel] = {"h": digest, "size": len(data)}
        total += len(data)
    return files, total


# -------------------------------------------------------------------- snapshots

def load_snapshots(pid):
    data = _read_json(snapshots_file(pid), [])
    snaps = [s for s in data if isinstance(s, dict) and s.get("id") and isinstance(s.get("files"), dict)]
    snaps.sort(key=lambda s: s.get("ts", 0))
    return snaps


def save_snapshots(pid, snaps):
    _write_json(snapshots_file(pid), snaps)


def get_snapshot(pid, sid):
    for s in load_snapshots(pid):
        if s["id"] == sid:
            return s
    return None


def previous_snapshot(pid, sid, branch=None):
    snap = get_snapshot(pid, sid)
    if snap is None:
        return None
    chain = branch_chain(pid, branch or snap.get("branch") or "main")
    ids = [s["id"] for s in chain]
    if sid not in ids:
        return None
    return chain[ids.index(sid) - 1] if ids.index(sid) else None


def take_snapshot(pid, note="", title="", kind="manual", branch=None):
    files, total = _collect(pid)
    if not files:
        raise StoreError("Nothing to capture: no trackable files were found.")
    bid = branch or active_branch(pid)
    chain = branch_chain(pid, bid)
    snap = {
        "id": uuid.uuid4().hex[:12],
        "ts": time.time(),
        "note": note.strip(),
        "title": title.strip(),
        "kind": kind,
        "branch": bid,
        "parent": chain[-1]["id"] if chain else None,
        "files": files,
        "count": len(files),
        "bytes": total,
    }
    snaps = load_snapshots(pid)
    snaps.append(snap)
    save_snapshots(pid, snaps)
    return snap


def update_snapshot(pid, sid, note=None, title=None):
    snaps = load_snapshots(pid)
    for s in snaps:
        if s["id"] == sid:
            if note is not None:
                s["note"] = note.strip()
            if title is not None:
                s["title"] = title.strip()
            save_snapshots(pid, snaps)
            return s
    return None


def delete_snapshot(pid, sid):
    snaps = load_snapshots(pid)
    dead = next((s for s in snaps if s["id"] == sid), None)
    parent = dead.get("parent") if dead else None
    snaps = [s for s in snaps if s["id"] != sid]
    for s in snaps:
        if s.get("parent") == sid:
            s["parent"] = parent
    save_snapshots(pid, snaps)


def has_changes(pid, branch=None):
    """True when the folder differs from the newest snapshot (used by auto-save)."""
    chain = branch_chain(pid, branch or active_branch(pid))
    if not chain:
        return True
    try:
        current, _total = _collect(pid, store_objects=False)
    except StoreError:
        return False
    return current != chain[-1]["files"]


# --------------------------------------------------------------------- branches

def load_branches(pid):
    data = _read_json(branches_file(pid), [])
    branches = [b for b in data if isinstance(b, dict) and b.get("id") and b.get("name")]
    if not any(b["id"] == "main" for b in branches):
        created = (get_project(pid) or {}).get("created") or time.time()
        branches.insert(0, {"id": "main", "name": "main", "base": None, "created": created})
    return branches


def save_branches(pid, branches):
    _write_json(branches_file(pid), branches)


def get_branch(pid, bid):
    for b in load_branches(pid):
        if b["id"] == bid:
            return b
    return None


def active_branch(pid):
    bid = (get_project(pid) or {}).get("branch") or "main"
    return bid if get_branch(pid, bid) else "main"


def set_active_branch(pid, bid):
    projects = load_projects()
    for p in projects:
        if p["id"] == pid:
            p["branch"] = bid
    save_projects(projects)


def add_branch(pid, name, from_sid):
    """Start a new branch at `from_sid`; it inherits the history before that moment."""
    name = (name or "").strip()
    if not name:
        raise StoreError("Give the branch a name.")
    branches = load_branches(pid)
    if any(b["name"].lower() == name.lower() for b in branches):
        raise StoreError(f"A branch called “{name}” already exists.")
    if not get_snapshot(pid, from_sid):
        raise StoreError("Pick the moment the branch should start from.")
    branch = {
        "id": uuid.uuid4().hex[:8],
        "name": name,
        "base": from_sid,
        "created": time.time(),
    }
    branches.append(branch)
    save_branches(pid, branches)
    return branch


def branch_chain(pid, bid=None):
    """The moments of one branch: its own snapshots plus the history it branched from."""
    bid = bid or active_branch(pid)
    snaps = load_snapshots(pid)
    by_id = {s["id"]: s for s in snaps}
    branch = get_branch(pid, bid) or {}
    heads = [s for s in snaps if (s.get("branch") or "main") == bid]
    head_id = heads[-1]["id"] if heads else branch.get("base")
    seen = set()
    chain = []
    while head_id and head_id not in seen:
        seen.add(head_id)
        snap = by_id.get(head_id)
        if snap is None:
            break
        chain.append(snap)
        parent = snap.get("parent")
        if parent is None:
            # snapshots captured before branching have no parent link yet
            own = snap.get("branch") or "main"
            earlier = [s for s in snaps
                       if s["id"] not in seen and s["ts"] <= snap["ts"]
                       and (s.get("branch") or "main") == own]
            parent = earlier[-1]["id"] if earlier else (get_branch(pid, own) or {}).get("base")
        head_id = parent
    chain.sort(key=lambda s: s.get("ts", 0))
    return chain


# ------------------------------------------------------------------------- blobs

def blob_path(pid, digest):
    return os.path.join(objects_dir(pid), digest[:2], digest)


def read_blob(pid, digest):
    try:
        with open(blob_path(pid, digest), "rb") as f:
            return f.read()
    except OSError:
        return b""


_STORAGE_CACHE = {}


def storage_bytes(pid):
    """Bytes in the object store, cached against the snapshot index.

    The store only grows when a snapshot is written (blobs are never deleted), and a
    snapshot rewrites the index afterwards, so the index timestamp is a cheap and
    reliable cache key for large object trees.
    """
    try:
        st = os.stat(snapshots_file(pid))
        key = (st.st_mtime_ns, st.st_size)
    except OSError:
        key = None
    hit = _STORAGE_CACHE.get(pid)
    if hit is not None and key is not None and hit[0] == key:
        return hit[1]
    total = 0
    root = objects_dir(pid)
    for dirpath, _dirs, files in os.walk(root):
        for fn in files:
            try:
                total += os.path.getsize(os.path.join(dirpath, fn))
            except OSError:
                pass
    if key is not None:
        _STORAGE_CACHE[pid] = (key, total)
    return total


def library_stats():
    """Counts, storage and a per-day histogram for every project, in one pass."""
    counts = {}
    per_day = {}
    used = 0
    for proj in load_projects():
        snaps = load_snapshots(proj["id"])
        counts[proj["id"]] = len(snaps)
        used += storage_bytes(proj["id"])
        for snap in snaps:
            key = day_key(snap["ts"])
            per_day[key] = per_day.get(key, 0) + 1
    return {"counts": counts, "per_day": per_day, "used": used}


# ---------------------------------------------------------------------- compare

def compare(a, b):
    """Compare two snapshot file maps -> dict of status lists and counters."""
    fa, fb = a.get("files", {}), b.get("files", {})
    added = sorted(p for p in fb if p not in fa)
    deleted = sorted(p for p in fa if p not in fb)
    modified = sorted(p for p in fb if p in fa and fa[p]["h"] != fb[p]["h"])
    same = [p for p in fb if p in fa and fa[p]["h"] == fb[p]["h"]]
    return {
        "added": added,
        "deleted": deleted,
        "modified": modified,
        "same": same,
        "counts": {"added": len(added), "deleted": len(deleted), "modified": len(modified), "same": len(same)},
        "bytes_in": sum(fb[p]["size"] for p in fb),
        "bytes_out": sum(fa[p]["size"] for p in fa),
    }


def ordered_changes(diff):
    rows = []
    for rel in diff["modified"]:
        rows.append((rel, "modified"))
    for rel in diff["added"]:
        rows.append((rel, "added"))
    for rel in diff["deleted"]:
        rows.append((rel, "deleted"))
    return rows


# ---------------------------------------------------------------------- restore

def preview_restore(pid, snap):
    """What restoring `snap` would change in the live folder, without touching it."""
    current, _ = _collect(pid, store_objects=False)
    write = sorted(p for p, info in snap["files"].items()
                   if p not in current or current[p]["h"] != info["h"])
    delete = sorted(p for p in current if p not in snap["files"])
    return write, delete


def restore(pid, snap, whole=True, rels=None, safety=True):
    """Restore the whole project or single files. Returns (actions, safety_snapshot)."""
    proj = get_project(pid)
    if proj is None:
        raise StoreError("Unknown project.")
    path = proj["path"]
    if not os.path.isdir(path):
        raise StoreError(f"The project folder is missing:\n{path}")

    created = None
    if safety and whole:
        try:
            created = take_snapshot(pid, note="Safety copy before restoring an earlier snapshot")
        except StoreError:
            created = None

    actions = []
    targets = list(snap["files"].keys()) if whole else list(rels or [])
    for rel in targets:
        info = snap["files"].get(rel)
        if not info:
            continue
        dest = os.path.join(path, *rel.split("/"))
        if os.path.isfile(dest):
            try:
                if os.path.getsize(dest) == info["size"]:
                    with open(dest, "rb") as f:
                        if hashlib.sha256(f.read()).hexdigest() == info["h"]:
                            continue
            except OSError:
                pass
        data = read_blob(pid, info["h"])
        parent = os.path.dirname(dest)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(dest, "wb") as f:
            f.write(data)
        actions.append(("restored", rel))

    if whole:
        current, _ = _collect(pid, store_objects=False)
        for rel in sorted(current):
            if rel not in snap["files"]:
                try:
                    os.remove(os.path.join(path, *rel.split("/")))
                    actions.append(("removed", rel))
                except OSError:
                    pass
    return actions, created


# ------------------------------------------------------------------------ stats

def project_stats(pid, branch=None):
    bid = branch or active_branch(pid)
    snaps = branch_chain(pid, bid)
    proj = get_project(pid)
    live = {}
    if proj and os.path.isdir(proj["path"]):
        try:
            live, _ = _collect(pid, store_objects=False)
        except StoreError:
            live = {}
    today = date.today()
    per_day = []
    for back in range(13, -1, -1):
        day = today - timedelta(days=back)
        start = time.mktime(datetime(day.year, day.month, day.day).timetuple())
        cnt = sum(1 for s in snaps if start <= s["ts"] < start + 86400)
        per_day.append((day.strftime("%d/%m"), cnt))
    changed = {}
    prev = None
    for s in snaps:
        if prev is not None:
            d = compare(prev, s)
            for rel in d["modified"]:
                changed[rel] = changed.get(rel, 0) + 1
            for rel in d["added"]:
                changed.setdefault(rel, 0)
        # first snapshot: every file counts as introduced
        else:
            for rel in s["files"]:
                changed.setdefault(rel, 0)
        prev = s
    top = sorted(((k, v) for k, v in changed.items() if v > 0), key=lambda x: -x[1])[:8]
    span = (snaps[-1]["ts"] - snaps[0]["ts"]) / 86400 if len(snaps) > 1 else 0
    return {
        "snapshots": len(snaps),
        "files": len(live),
        "live_bytes": sum(f["size"] for f in live.values()),
        "storage": storage_bytes(pid),
        "per_day": per_day,
        "top_changed": top,
        "span_days": span,
        "first": snaps[0]["ts"] if snaps else None,
        "last": snaps[-1]["ts"] if snaps else None,
        "branch": bid,
        "branch_name": (get_branch(pid, bid) or {}).get("name", "main"),
        "branches": len(load_branches(pid)),
    }


def fmt_bytes(n):
    n = float(n or 0)
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024 or unit == "GB":
            return (f"{n:.0f} {unit}" if unit == "B" else f"{n:.1f} {unit}")
        n /= 1024.0
    return f"{n:.1f} GB"


def fmt_time(ts):
    return datetime.fromtimestamp(ts).strftime("%H:%M")


def fmt_date(ts):
    return datetime.fromtimestamp(ts).strftime("%d %b %Y")


def fmt_stamp(ts):
    return datetime.fromtimestamp(ts).strftime("%d %b %Y · %H:%M")


def day_key(ts):
    return datetime.fromtimestamp(ts).strftime("%Y-%m-%d")


def day_label(ts):
    d = datetime.fromtimestamp(ts).date()
    today = date.today()
    if d == today:
        return "Today"
    if d == today - timedelta(days=1):
        return "Yesterday"
    return d.strftime("%A, %d %B %Y")


def ago(ts):
    diff = time.time() - ts
    if diff < 60:
        return "just now"
    if diff < 3600:
        return f"{int(diff // 60)} min ago"
    if diff < 86400:
        return f"{int(diff // 3600)} h ago"
    days = int(diff // 86400)
    return f"{days} day{'s' if days != 1 else ''} ago"
