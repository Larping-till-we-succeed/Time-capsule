"""Diff helpers: decide whether a blob is text and build readable unified diffs."""

import difflib

TEXT_EXT = {
    ".py", ".pyw", ".js", ".jsx", ".ts", ".tsx", ".json", ".md", ".txt", ".rst", ".css", ".scss",
    ".html", ".htm", ".xml", ".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf", ".sh", ".bash",
    ".zsh", ".c", ".h", ".cpp", ".hpp", ".cc", ".cs", ".java", ".kt", ".rs", ".go", ".rb", ".php",
    ".sql", ".lua", ".swift", ".dart", ".ex", ".exs", ".r", ".m", ".mm", ".vue", ".svelte", ".env",
    ".gitignore", ".dockerfile", ".gradle", ".properties", ".plist", ".tex", ".csv",
}


def looks_text(name, data):
    """Cheap text heuristic: known extension or no NUL byte and valid UTF-8."""
    if not data:
        return True
    ext = ("." + name.rsplit(".", 1)[-1].lower()) if "." in name else ""
    if ext in TEXT_EXT:
        return True
    sample = data[:8192]
    if b"\x00" in sample:
        return False
    try:
        sample.decode("utf-8")
        return True
    except UnicodeDecodeError:
        return False


def decode(data):
    return data.decode("utf-8", "replace")


def unified(name, old, new, context=3):
    """Return a unified diff as a list of (kind, text) lines. kind is + - @ or ' '."""
    a = decode(old).splitlines()
    b = decode(new).splitlines()
    out = []
    for line in difflib.unified_diff(a, b, fromfile="a/" + name, tofile="b/" + name, lineterm="", n=context):
        if line.startswith("+++") or line.startswith("---"):
            kind = "@"
        elif line.startswith("@@"):
            kind = "@"
        elif line.startswith("+"):
            kind = "+"
        elif line.startswith("-"):
            kind = "-"
        else:
            kind = " "
        out.append((kind, line))
    return out


def line_counts(old, new):
    added = removed = 0
    for kind, _line in unified("x", old, new, context=0):
        if kind == "+":
            added += 1
        elif kind == "-":
            removed += 1
    return added, removed


def side_by_side(old, new, limit=400):
    """Simple paired view used when a unified diff would be too wide."""
    a = decode(old).splitlines()[:limit]
    b = decode(new).splitlines()[:limit]
    return a, b
