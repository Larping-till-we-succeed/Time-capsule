//! Headless commands over the same store as the window, so snapshots can run
//! from cron or a systemd timer and the data can be inspected from a script.

use serde_json::json;

use crate::store;

const HELP: &str = "\
TimeCapsule — command line

USAGE:
  timecapsule snapshot  [--project <id|name|path>] [--title TEXT] [--note TEXT]
  timecapsule list      [--project <id|name|path>] [--json]
  timecapsule restore   <snapshot-id> [--project <...>] [--file <path>] [--no-safety]
  timecapsule gc        [--project <...>] [--apply]
  timecapsule verify    [--project <...>] [--json]
  timecapsule --help

Without --project a single tracked project is used automatically.
gc only reports by default; pass --apply to delete unreferenced blobs.
Data lives in $TIMECAPSULE_HOME (default: the app's data folder).
";

#[derive(Default)]
struct Opts {
    project: Option<String>,
    json: bool,
    apply: bool,
    no_safety: bool,
    file: Option<String>,
    title: Option<String>,
    note: Option<String>,
    positional: Vec<String>,
}

pub fn run(args: &[String]) -> i32 {
    let mut opts = Opts::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--project" | "-p" => opts.project = iter.next().cloned(),
            "--json" => opts.json = true,
            "--apply" => opts.apply = true,
            "--no-safety" => opts.no_safety = true,
            "--file" => opts.file = iter.next().cloned(),
            "--title" => opts.title = iter.next().cloned(),
            "--note" => opts.note = iter.next().cloned(),
            "--dry-run" => opts.apply = false,
            "-h" | "--help" => {
                print!("{}", HELP);
                return 0;
            }
            other if other.starts_with('-') => {
                eprintln!("timecapsule: unknown option {other}\n\n{HELP}");
                return 2;
            }
            other => opts.positional.push(other.to_string()),
        }
    }

    let Some(command) = opts.positional.first().cloned() else {
        print!("{}", HELP);
        return 2;
    };

    let result = match command.as_str() {
        "snapshot" => cmd_snapshot(&opts),
        "list" => cmd_list(&opts),
        "restore" => cmd_restore(&opts),
        "gc" => cmd_gc(&opts),
        "verify" => cmd_verify(&opts),
        other => Err(format!("unknown command {other}\n\n{HELP}")),
    };

    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("timecapsule: {message}");
            1
        }
    }
}

fn resolve_project(opts: &Opts) -> Result<store::Project, String> {
    let projects = store::load_projects();
    if projects.is_empty() {
        return Err("no tracked projects yet — add one in the app first".into());
    }
    let Some(selected) = opts.project.as_deref() else {
        if projects.len() == 1 {
            return Ok(projects.into_iter().next().unwrap());
        }
        let mut message = String::from("several projects are tracked, pass --project:\n");
        for p in &projects {
            message.push_str(&format!("  {}  {}  {}\n", p.id, p.name, p.path));
        }
        return Err(message.trim_end().to_string());
    };
    let wanted = selected.trim();
    let found = projects.iter().find(|p| {
        p.id == wanted
            || p.name == wanted
            || p.path == wanted
            || p.name.eq_ignore_ascii_case(wanted)
            || store::expand_path(&p.path) == store::expand_path(wanted)
    });
    found
        .cloned()
        .ok_or_else(|| format!("no project matches {wanted:?}"))
}

fn cmd_snapshot(opts: &Opts) -> Result<i32, String> {
    let project = resolve_project(opts)?;
    let note = opts.note.clone().unwrap_or_default();
    let title = opts.title.clone().unwrap_or_default();
    let snap = store::take_snapshot(&project.id, &note, &title, "manual", None)
        .map_err(|e| e.to_string())?;
    if opts.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "id": snap.id,
                "ts": snap.ts,
                "time": store::fmt_stamp(snap.ts),
                "title": snap.title,
                "note": snap.note,
                "branch": snap.branch_id(),
                "files": snap.count,
                "bytes": snap.bytes,
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "Captured {} · {} files · {} · {}",
            snap.id,
            snap.count,
            store::fmt_bytes(snap.bytes),
            store::fmt_stamp(snap.ts)
        );
    }
    Ok(0)
}

fn cmd_list(opts: &Opts) -> Result<i32, String> {
    let projects = store::load_projects();
    if opts.project.is_none() && projects.len() > 1 {
        if opts.json {
            let rows: Vec<_> = projects
                .iter()
                .map(|p| {
                    let snaps = store::load_snapshots(&p.id);
                    json!({
                        "id": p.id,
                        "name": p.name,
                        "path": p.path,
                        "snapshots": snaps.len(),
                        "latest": snaps.last().map(|s| s.ts),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
        } else {
            println!("{:<12} {:<24} {:>9}  PATH", "ID", "NAME", "SNAPSHOTS");
            for p in &projects {
                let count = store::load_snapshots(&p.id).len();
                println!("{:<12} {:<24} {:>9}  {}", p.id, p.name, count, p.path);
            }
        }
        return Ok(0);
    }

    let project = resolve_project(opts)?;
    let mut snaps = store::load_snapshots(&project.id);
    snaps.sort_by(|a, b| a.ts.total_cmp(&b.ts));
    if opts.json {
        let rows: Vec<_> = snaps
            .iter()
            .map(|s| {
                json!({
                    "id": s.id,
                    "ts": s.ts,
                    "time": store::fmt_stamp(s.ts),
                    "title": s.title,
                    "note": s.note,
                    "branch": s.branch_id(),
                    "files": s.count,
                    "bytes": s.bytes,
                    "pinned": s.pinned,
                    "kind": s.kind,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
    } else if snaps.is_empty() {
        println!("{} has no snapshots yet.", project.name);
    } else {
        println!("{} · {}", project.name, project.path);
        for s in &snaps {
            println!(
                "{:<14} {:<20} {:>6} {:>10}  {}{}",
                s.id,
                store::fmt_stamp(s.ts),
                s.count,
                store::fmt_bytes(s.bytes),
                if s.pinned { "★ " } else { "" },
                if s.title.trim().is_empty() {
                    "(untitled)"
                } else {
                    s.title.trim()
                }
            );
            if !s.note.trim().is_empty() {
                println!("               note: {}", s.note.trim());
            }
        }
    }
    Ok(0)
}

fn cmd_restore(opts: &Opts) -> Result<i32, String> {
    let Some(sid) = opts.positional.get(1) else {
        return Err("restore needs a snapshot id (see: timecapsule list)".into());
    };
    let project = resolve_project(opts)?;
    let snap = store::get_snapshot(&project.id, sid)
        .ok_or_else(|| format!("no snapshot {sid} in {}", project.name))?;
    let whole = opts.file.is_none();
    let rels = opts.file.clone().map(|f| vec![f]);
    let (actions, safety) = store::restore(
        &project.id,
        &snap,
        whole,
        rels.as_deref(),
        !opts.no_safety,
    )
    .map_err(|e| e.to_string())?;
    if opts.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "project": project.id,
                "snapshot": snap.id,
                "actions": actions.iter().map(|(kind, path)| json!({ "action": kind, "path": path })).collect::<Vec<_>>(),
                "safety_snapshot": safety.map(|s| s.id),
            }))
            .unwrap_or_default()
        );
    } else {
        for (kind, path) in &actions {
            println!("{kind:<9} {path}");
        }
        println!(
            "{} change{} from {}",
            actions.len(),
            if actions.len() == 1 { "" } else { "s" },
            store::fmt_stamp(snap.ts)
        );
        if let Some(safety) = safety {
            println!("Previous state kept as safety snapshot {}.", safety.id);
        }
    }
    Ok(0)
}

fn cmd_gc(opts: &Opts) -> Result<i32, String> {
    let project = resolve_project(opts)?;
    let dry_run = !opts.apply;
    let report = store::gc_objects(&project.id, dry_run).map_err(|e| e.to_string())?;
    if opts.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "project": project.id,
                "dry_run": report.dry_run,
                "blobs": report.blobs,
                "referenced": report.referenced,
                "removed": report.removed,
                "freed": report.freed,
                "freed_human": store::fmt_bytes(report.freed),
            }))
            .unwrap_or_default()
        );
    } else if dry_run {
        println!(
            "{} blobs, {} referenced · {} unused ({}) would be removed — rerun with --apply",
            report.blobs,
            report.referenced,
            report.removed,
            store::fmt_bytes(report.freed)
        );
    } else {
        println!(
            "Removed {} unused blob{} and freed {}.",
            report.removed,
            if report.removed == 1 { "" } else { "s" },
            store::fmt_bytes(report.freed)
        );
    }
    Ok(0)
}

fn cmd_verify(opts: &Opts) -> Result<i32, String> {
    let project = resolve_project(opts)?;
    let report = store::verify_store(&project.id);
    let healthy = report.missing.is_empty() && report.corrupt.is_empty();
    if opts.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "project": project.id,
                "healthy": healthy,
                "referenced": report.referenced,
                "present": report.present,
                "missing": report.missing,
                "corrupt": report.corrupt,
                "unreferenced": report.unreferenced,
            }))
            .unwrap_or_default()
        );
    } else if healthy {
        println!(
            "{} blobs checked — every captured file is readable ({} unreferenced).",
            report.present, report.unreferenced
        );
    } else {
        println!(
            "{} missing · {} corrupt · {} unreferenced",
            report.missing.len(),
            report.corrupt.len(),
            report.unreferenced
        );
        for digest in &report.missing {
            println!("missing  {digest}");
        }
        for digest in &report.corrupt {
            println!("corrupt  {digest}");
        }
    }
    Ok(if healthy { 0 } else { 1 })
}
