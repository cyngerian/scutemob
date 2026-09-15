#!/usr/bin/env python3
"""data-freshness — check and refresh scutemob's external rules/card data.

Sources (all gitignored; `refresh` rebuilds every one of them):

  CR        .scryfall-cache/MagicCompRules.txt       Wizards; a new file ships with each set
  oracle    .scryfall-cache/oracle-cards.jsonl.gz    Scryfall bulk data; rebuilt daily
  rulings   .scryfall-cache/rulings.jsonl.gz         Scryfall bulk data; rebuilt daily
  sqlite    cards.sqlite                             built from the three above by
                                                     `scryfall-import` + `mtg-mcp-server --import-only`
  fixture   test-data/card-fidelity/printed-fields.tsv  COMMITTED; SR-37 gate input, built
                                                     from cards.sqlite by refresh-card-fidelity-fixture.py

`.scryfall-cache/meta.json` records what each artifact was built from; `check` reads it
and `refresh` writes it. Without it (a cache older than 2026-09-14) sqlite/fixture read
UNKNOWN — run `refresh`.

Verbs:
  check    (default) local vs published. Exit 0 CURRENT, 1 STALE, 2 UNKNOWN or network error.
           --max-age-days N   Scryfall files older than N days count as stale (default 30);
                              the CR is stale on ANY effective-date difference.
           --offline          local dates only, no network.
           --json             machine output.
  refresh  download + rebuild everything, in order: cr, scryfall, rules, fixture.
           --only STEP        run one step (repeatable). --skip STEP likewise.
  cites    every `CR NNN[.N[a]]` token under crates/ docs/ test-data/ must name a rule that
           exists in the local CR. Existence only — the per-section coverage report is
           LL-2 (scutemob-256). Exit 1 on any unknown citation.

The /start skill runs `tools/start-check.sh` (= `check`) and offers `refresh` when STALE.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import sqlite3
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CACHE = ROOT / ".scryfall-cache"
META = CACHE / "meta.json"
CR_TXT = CACHE / "MagicCompRules.txt"
ORACLE = CACHE / "oracle-cards.jsonl.gz"
RULINGS = CACHE / "rulings.jsonl.gz"
SQLITE = ROOT / "cards.sqlite"
FIXTURE = ROOT / "test-data" / "card-fidelity" / "printed-fields.tsv"
CARGO = Path.home() / ".cargo" / "bin" / "cargo"

RULES_PAGE = "https://magic.wizards.com/en/rules"
CR_LINK_RE = re.compile(r"https://media\.wizards\.com/[^\"' ]*MagicCompRules[^\"']*?\.txt")
SCRYFALL_BULK = "https://api.scryfall.com/bulk-data/{}"
UA = {"User-Agent": "scutemob-data-freshness/1.0 (github.com/cyngerian/scutemob)", "Accept": "*/*"}

CITE_RE = re.compile(r"CR (\d{3}(?:\.\d+[a-z]?)?)")
RULE_RE = re.compile(r"^(\d{3}(?:\.\d+[a-z]?)?)\.?\s+(.*)$")
EFFECTIVE_RE = re.compile(r"effective as of ([A-Z][a-z]+ \d{1,2}, \d{4})")
MONTHS = {m: i for i, m in enumerate(
    ["January", "February", "March", "April", "May", "June", "July", "August",
     "September", "October", "November", "December"], 1)}


# ----------------------------------------------------------------------------- helpers

def now_iso() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()


def fetch(url: str, timeout: int = 60) -> bytes:
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read()


def read_meta() -> dict:
    if META.exists():
        try:
            return json.loads(META.read_text())
        except json.JSONDecodeError:
            pass
    return {}


def write_meta(meta: dict) -> None:
    CACHE.mkdir(exist_ok=True)
    META.write_text(json.dumps(meta, indent=2, sort_keys=True) + "\n")


def cr_text(path: Path) -> str:
    # UTF-8 with BOM, bare CR line endings (memory/gotchas-infra.md).
    return path.read_text(encoding="utf-8-sig", errors="replace").replace("\r\n", "\n").replace("\r", "\n")


def cr_effective(text: str) -> tuple[str | None, str | None]:
    """('August 7, 2026', '2026-08-07') or (None, None)."""
    m = EFFECTIVE_RE.search(text[:4000])
    if not m:
        return None, None
    human = m.group(1)
    mon, day, year = human.replace(",", "").split()
    return human, f"{int(year):04d}-{MONTHS[mon]:02d}-{int(day):02d}"


def parse_cr_rules(text: str) -> dict[str, str]:
    """rule number -> text, from the body (after the table of contents) up to the Glossary."""
    i = text.find("\n1. Game Concepts")
    j = text.find("\n1. Game Concepts", i + 1)
    body = text[j:] if j > 0 else text
    g = body.find("\nGlossary")
    body = body[:g] if g > 0 else body
    rules: dict[str, str] = {}
    for line in body.split("\n"):
        m = RULE_RE.match(line.strip())
        if m:
            rules[m.group(1)] = m.group(2).strip()
    return rules


def published_cr_link() -> str:
    page = fetch(RULES_PAGE, timeout=30).decode("utf-8", "replace")
    m = CR_LINK_RE.search(page)
    if not m:
        raise RuntimeError("no MagicCompRules*.txt link on " + RULES_PAGE)
    return m.group(0).replace(" ", "%20")


def scryfall_bulk_info(kind: str) -> dict:
    return json.loads(fetch(SCRYFALL_BULK.format(kind), timeout=30))


def sqlite_counts() -> dict | None:
    if not SQLITE.exists():
        return None
    con = sqlite3.connect(f"file:{SQLITE}?mode=ro", uri=True)
    try:
        c = {}
        for t in ("cards", "rulings", "rules"):
            try:
                c[t] = con.execute(f"SELECT COUNT(*) FROM {t}").fetchone()[0]
            except sqlite3.OperationalError:
                c[t] = 0
        return c
    finally:
        con.close()


def file_age_days(p: Path) -> float | None:
    if not p.exists():
        return None
    return (dt.datetime.now(dt.timezone.utc) - dt.datetime.fromtimestamp(p.stat().st_mtime, dt.timezone.utc)).days


def parse_iso(s: str | None) -> dt.datetime | None:
    if not s:
        return None
    try:
        return dt.datetime.fromisoformat(s.replace("Z", "+00:00"))
    except ValueError:
        return None


# ----------------------------------------------------------------------------- check

def do_check(args) -> int:
    meta = read_meta()
    rows: list[dict] = []
    worst = 0  # 0 current, 1 stale, 2 unknown

    def row(source, local, published, status, note=""):
        nonlocal worst
        rows.append({"source": source, "local": local, "published": published, "status": status, "note": note})
        worst = max(worst, {"CURRENT": 0, "STALE": 1, "UNKNOWN": 2}[status])

    # --- CR
    local_eff = local_iso = None
    if CR_TXT.exists():
        local_eff, local_iso = cr_effective(cr_text(CR_TXT))
    if args.offline:
        row("CR", local_eff or "missing", "(offline)", "CURRENT" if local_eff else "UNKNOWN")
    else:
        try:
            link = published_cr_link()
            if meta.get("cr", {}).get("url") == link and local_eff:
                row("CR", local_eff, local_eff, "CURRENT", "meta url matches published link")
            else:
                pub_eff, _ = cr_effective(fetch(link).decode("utf-8-sig", "replace"))
                if not local_eff:
                    row("CR", "missing", pub_eff or link, "STALE")
                elif pub_eff == local_eff:
                    row("CR", local_eff, pub_eff, "CURRENT")
                else:
                    row("CR", local_eff, pub_eff, "STALE", link)
        except Exception as e:  # noqa: BLE001
            row("CR", local_eff or "missing", "?", "UNKNOWN", f"fetch failed: {e}")

    # --- Scryfall bulk files
    for kind, path in (("oracle_cards", ORACLE), ("rulings", RULINGS)):
        local_ts = meta.get(kind, {}).get("updated_at")
        legacy = path.with_name(path.name.replace(".jsonl.gz", ".json"))
        if not path.exists() and legacy.exists():
            local_desc = f"legacy {legacy.name} (mtime {dt.date.fromtimestamp(legacy.stat().st_mtime)})"
            local_dt = dt.datetime.fromtimestamp(legacy.stat().st_mtime, dt.timezone.utc)
        elif path.exists():
            local_dt = parse_iso(local_ts) or dt.datetime.fromtimestamp(path.stat().st_mtime, dt.timezone.utc)
            local_desc = local_dt.date().isoformat()
        else:
            local_dt, local_desc = None, "missing"
        if args.offline:
            row(kind, local_desc, "(offline)", "CURRENT" if local_dt else "UNKNOWN")
            continue
        try:
            pub = parse_iso(scryfall_bulk_info(kind)["updated_at"])
            if local_dt is None:
                row(kind, "missing", pub.date().isoformat(), "STALE")
            else:
                age = (pub - local_dt).days
                st = "STALE" if age > args.max_age_days else "CURRENT"
                row(kind, local_desc, pub.date().isoformat(), st, f"{age} days behind (limit {args.max_age_days})")
        except Exception as e:  # noqa: BLE001
            row(kind, local_desc, "?", "UNKNOWN", f"fetch failed: {e}")

    # --- sqlite: built from the cache files above?
    counts = sqlite_counts()
    ms = meta.get("sqlite")
    if counts is None:
        row("sqlite", "missing", "-", "STALE", "run refresh")
    elif not ms:
        row("sqlite", f"{counts['cards']} cards / {counts['rules']} rules", "-", "UNKNOWN", "no meta.json; run refresh")
    else:
        problems = []
        if counts != ms.get("counts"):
            problems.append("row counts differ from meta")
        if ms.get("cr_effective_iso") != local_iso:
            problems.append(f"rules table built from CR {ms.get('cr_effective_iso')}, cache has {local_iso}")
        for kind in ("oracle_cards", "rulings"):
            if ms.get(kind + "_updated_at") != meta.get(kind, {}).get("updated_at"):
                problems.append(f"{kind} newer in cache than in sqlite")
        row("sqlite", f"imported {ms.get('imported_at', '?')[:10]}", "-",
            "STALE" if problems else "CURRENT", "; ".join(problems))

    # --- fixture (committed): built from the current sqlite?
    mf = meta.get("fixture")
    if not FIXTURE.exists():
        row("fixture", "missing", "-", "STALE", "run refresh --only fixture")
    elif not mf or not ms:
        row("fixture", "present", "-", "UNKNOWN", "no meta.json; run refresh --only fixture")
    elif mf.get("sqlite_imported_at") != ms.get("imported_at"):
        row("fixture", f"from sqlite {mf.get('sqlite_imported_at', '?')[:10]}", "-", "STALE",
            "sqlite was rebuilt since; run refresh --only fixture, then the SR-37 gate")
    else:
        row("fixture", f"from sqlite {mf.get('sqlite_imported_at', '?')[:10]}", "-", "CURRENT")

    overall = ["CURRENT", "STALE", "UNKNOWN"][worst]
    if args.json:
        print(json.dumps({"overall": overall, "rows": rows}, indent=2))
    else:
        w = max(len(r["source"]) for r in rows)
        for r in rows:
            note = f"  ({r['note']})" if r["note"] else ""
            print(f"{r['source']:<{w}}  {r['status']:<7}  local {r['local']}  published {r['published']}{note}")
        print(f"\nDATA {overall}")
        if worst:
            print("refresh with:  python3 tools/data-freshness.py refresh")
    return worst


# ----------------------------------------------------------------------------- refresh

def run(cmd: list[str], **kw) -> None:
    print("+", " ".join(str(c) for c in cmd), flush=True)
    subprocess.run(cmd, check=True, cwd=ROOT, **kw)


def step_cr(meta: dict) -> None:
    link = published_cr_link()
    print(f"CR: downloading {link}")
    data = fetch(link)
    text = data.decode("utf-8-sig", "replace")
    human, iso = cr_effective(text)
    if not iso:
        raise RuntimeError("downloaded CR has no 'effective as of' line; refusing to overwrite")
    n = len(parse_cr_rules(text))
    if n < 2000:
        raise RuntimeError(f"downloaded CR parses to only {n} rules; refusing to overwrite")
    CACHE.mkdir(exist_ok=True)
    tmp = CR_TXT.with_suffix(".txt.tmp")
    tmp.write_bytes(data)
    tmp.replace(CR_TXT)
    meta["cr"] = {"url": link, "effective": human, "effective_iso": iso, "rules": n, "fetched_at": now_iso()}
    print(f"CR: effective {human}, {n} rules")


def step_scryfall(meta: dict) -> None:
    infos = {k: scryfall_bulk_info(k) for k in ("oracle_cards", "rulings")}
    run([str(CARGO), "run", "-q", "-p", "scryfall-import", "--", "--db", str(SQLITE)])
    for k, info in infos.items():
        meta[k] = {"updated_at": info["updated_at"], "fetched_at": now_iso()}
    for legacy in (CACHE / "oracle-cards.json", CACHE / "rulings.json"):
        if legacy.exists():
            legacy.unlink()
            print(f"removed legacy {legacy.name} (superseded by the .jsonl.gz file)")


def step_rules(meta: dict) -> None:
    if not CR_TXT.exists():
        raise RuntimeError("no CR text in the cache; run --only cr first")
    run([str(CARGO), "run", "-q", "-p", "mtg-mcp-server", "--",
         "--db", str(SQLITE), "--rules", str(CR_TXT), "--import-only"])


def record_sqlite(meta: dict) -> None:
    _, iso = cr_effective(cr_text(CR_TXT)) if CR_TXT.exists() else (None, None)
    meta["sqlite"] = {
        "imported_at": now_iso(),
        "counts": sqlite_counts(),
        "cr_effective_iso": iso,
        "oracle_cards_updated_at": meta.get("oracle_cards", {}).get("updated_at"),
        "rulings_updated_at": meta.get("rulings", {}).get("updated_at"),
    }


def step_fixture(meta: dict) -> None:
    # Relative paths on purpose: the fixture script echoes its argv into the committed
    # file's header, and an absolute home-directory path there is churn.
    corpus = Path(tempfile.gettempdir()) / "corpus.tsv"
    with corpus.open("w") as out:
        run([str(CARGO), "run", "-q", "-p", "card-field-dump"], stdout=out)
    run([sys.executable, "tools/refresh-card-fidelity-fixture.py",
         "--corpus", str(corpus), "--db", SQLITE.name, "--out", str(FIXTURE.relative_to(ROOT))])
    corpus.unlink()
    meta["fixture"] = {"refreshed_at": now_iso(), "sqlite_imported_at": meta.get("sqlite", {}).get("imported_at")}


STEPS = ["cr", "scryfall", "rules", "fixture"]


def do_refresh(args) -> int:
    steps = [s for s in STEPS if (not args.only or s in args.only) and s not in (args.skip or [])]
    meta = read_meta()
    for s in steps:
        print(f"\n=== {s} ===")
        {"cr": step_cr, "scryfall": step_scryfall, "rules": step_rules, "fixture": step_fixture}[s](meta)
        if s in ("scryfall", "rules"):
            record_sqlite(meta)
        write_meta(meta)
    print("\nDone. Next:")
    print("  ~/.cargo/bin/cargo test -p mtg-engine --test core cards2_printed_field_fidelity   # SR-37 gate")
    print("  python3 tools/data-freshness.py cites                                           # citations still resolve")
    print("  A running mtg-rules MCP server reads cards.sqlite live; no restart needed.")
    return 0


# ----------------------------------------------------------------------------- cites

CITE_DIRS = ["crates", "docs", "test-data"]
CITE_EXT = {".rs", ".md", ".json", ".py", ".toml", ".txt", ".yaml", ".yml"}


def do_cites(args) -> int:
    if not CR_TXT.exists():
        print(f"no {CR_TXT.relative_to(ROOT)}; run `refresh --only cr`", file=sys.stderr)
        return 2
    rules = parse_cr_rules(cr_text(CR_TXT))
    unknown: dict[str, list[str]] = {}
    total = 0
    for d in args.dirs or CITE_DIRS:
        for p in (ROOT / d).rglob("*"):
            if not p.is_file() or p.suffix not in CITE_EXT or "target" in p.parts:
                continue
            try:
                lines = p.read_text(encoding="utf-8").split("\n")
            except UnicodeDecodeError:
                continue
            for i, line in enumerate(lines, 1):
                for m in CITE_RE.finditer(line):
                    total += 1
                    if m.group(1) not in rules:
                        unknown.setdefault(m.group(1), []).append(f"{p.relative_to(ROOT)}:{i}")
    print(f"{total} citations checked against CR {cr_effective(cr_text(CR_TXT))[0]} ({len(rules)} rules)")
    if not unknown:
        print("all citations resolve")
        return 0
    n = sum(len(v) for v in unknown.values())
    print(f"{n} citations name {len(unknown)} rule numbers that do not exist:")
    for k in sorted(unknown, key=lambda k: -len(unknown[k])):
        locs = unknown[k]
        shown = ", ".join(locs[:3]) + (f", +{len(locs) - 3} more" if len(locs) > 3 else "")
        print(f"  CR {k} x{len(locs)}: {shown}")
    return 1


# ----------------------------------------------------------------------------- main

def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="data-freshness", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="verb")
    c = sub.add_parser("check", help="local vs published (default)")
    c.add_argument("--max-age-days", type=int, default=30)
    c.add_argument("--offline", action="store_true")
    c.add_argument("--json", action="store_true")
    r = sub.add_parser("refresh", help="download and rebuild")
    r.add_argument("--only", action="append", choices=STEPS)
    r.add_argument("--skip", action="append", choices=STEPS)
    ci = sub.add_parser("cites", help="every CR citation must exist in the local CR")
    ci.add_argument("dirs", nargs="*", help=f"directories to scan (default {' '.join(CITE_DIRS)})")
    args = ap.parse_args(argv)
    if args.verb is None:
        args = ap.parse_args(["check"] + (argv or []))
    try:
        return {"check": do_check, "refresh": do_refresh, "cites": do_cites}[args.verb](args)
    except subprocess.CalledProcessError as e:
        print(f"step failed: {e}", file=sys.stderr)
        return 2
    except RuntimeError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
