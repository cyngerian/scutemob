#!/usr/bin/env python3
"""Authoring status report — regenerable source of truth for card authoring progress.

Run from anywhere:  python3 tools/authoring-report.py
Writes:             docs/authoring-status.md   (overwritten — never hand-edit)

Determinism: same git state + same authoring plan => same output. The "generated at"
timestamp is the only non-deterministic field.
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
import unicodedata
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
DEFS = REPO / "crates/engine/src/cards/defs"
PLAN = REPO / "test-data/test-cards/_authoring_plan.json"
OUT = REPO / "docs/authoring-status.md"
MISSING_OUT = REPO / "docs/authoring-status-missing.txt"
PREV_JSON = REPO / "docs/authoring-status-prev.json"  # baseline for run-over-run diff
SAMPLE_OTHER_N = 12  # how many raw OTHER TODO lines to show in the report

# Activity windows in days (for "added/modified in last N days" tables)
WINDOWS = (7, 30, 90, 365)

# TODO classification — pattern → human-readable bucket. First match wins, so put
# specific patterns BEFORE generic ones. Each bucket should map to a concrete engine
# primitive or named PB seed so the report directly informs the next-batch roadmap.
TODO_BUCKETS: list[tuple[re.Pattern, str]] = [
    # Specific named-primitive seeds (highest priority — reference our own queue)
    (re.compile(r"\bPB-CC-C-followup\b", re.I), "PB-CC-C-followup (Layer-7c CDA)"),
    (re.compile(r"\bPB-TS\b", re.I), "PB-TS (token-count → EffectAmount)"),

    # CR / engine concept buckets
    (re.compile(r"\bCDA\b|EffectAmount::PermanentCount|dynamic\s+P/T", re.I), "CDA / dynamic P/T"),
    (re.compile(r"can.t\s+(block|attack)|attack\s+if\s+able|attacks?\s+each\s+combat", re.I), "can't / must block-attack"),
    (re.compile(r"GainControl|exchange\s+control", re.I), "gain / exchange control"),
    (re.compile(r"WhenLeavesBattlefield|when\s+leaves|leave[s]?\s+the\s+battlefield", re.I), "leaves-battlefield trigger"),
    (re.compile(r"can.t\s+be\s+countered", re.I), "can't be countered"),
    (re.compile(r"play\s+lands?\s+from|additional\s+land", re.I), "play-lands-from-zone / extra land drops"),
    (re.compile(r"UntapAll|untap\s+all|WheneverPermanentUntaps|whenever.*untaps?", re.I), "untap-all / untap trigger"),
    (re.compile(r"return\s+(to\s+)?hand|bounce\s+land|ReturnPermanent", re.I), "return-to-hand / bounce"),
    (re.compile(r"hybrid\s+(mana|cost)", re.I), "hybrid mana"),
    (re.compile(r"activated.*from\s+graveyard|graveyard.*activat|activation_zone", re.I), "activate-from-graveyard"),
    (re.compile(r"TriggeringPermanent|triggering\s+(permanent|object)|EffectTarget::Triggering", re.I), "trigger-targets-triggerer"),
    (re.compile(r"hexproof|protection.*dynamic|conditional\s+protection", re.I), "dynamic hexproof / protection"),
    (re.compile(r"draw\s+a?\s*card.*trigger|whenever\s+(you|opponent|player).*draws?\s+a?\s*card", re.I), "draw trigger"),
    (re.compile(r"sacrifice.*additional\s+cost|AdditionalCost.*[Ss]acrifice|Cost::SacrificeAnyNumber|sacrifice\s+another", re.I), "sacrifice as cost"),
    (re.compile(r"AttachedCreature.*has|equipped.*has|attached.*has", re.I), "equipment grants ability"),
    (re.compile(r"WhenAttachedCreatureDies|equipped.*dies|attached.*dies", re.I), "equipped/attached dies"),
    (re.compile(r"control\s+\d|N\s+or\s+more.*control|count\s+of\s+\w+\s+you\s+control", re.I), "count-threshold static"),
    (re.compile(r"DevotionTo|devotion", re.I), "devotion"),
    (re.compile(r"additional\s+blocker", re.I), "additional blocker"),
    (re.compile(r"TriggerDoubling|drivnod|trigger.*double|doubl.*trigger|Isshin", re.I), "trigger doubling"),
    (re.compile(r"WheneverSubtypeDealsCombat|subtype.*combat\s+damage|subtype.*deals\s+damage", re.I), "subtype combat-damage trigger"),
    (re.compile(r"stun\s+counter", re.I), "stun counters"),
    (re.compile(r"no\s+max(imum)?\s+hand|maximum\s+hand\s+size", re.I), "no-maximum-hand-size"),
    (re.compile(r"\bExert\b", re.I), "exert"),
    (re.compile(r"as\s+(it\s+)?enters.*choose|choose.*enters|ETB.*choice", re.I), "ETB choice"),
    (re.compile(r"cost\s+reduction.*legend|per\s+legendary", re.I), "cost-reduction-per-legendary"),
    (re.compile(r"prevent.*noncombat|noncombat.*damage", re.I), "noncombat-damage prevent"),
    (re.compile(r"counter.*placed.*trigger|WhenCounterPlaced|when.*counter.*put\s+on", re.I), "counter-placed trigger"),
    (re.compile(r"\bWarp\b", re.I), "warp"),
    (re.compile(r"reanimate.*total\s+power|multi.*reanim|multi-target.*graveyard.*battlefield", re.I), "multi-target reanimate"),
    (re.compile(r"tap\s+untapped\s+creature.*cost|tap.*creature.*as\s+cost", re.I), "tap-creature cost"),
    (re.compile(r"protection.*chosen\s+color|color\s+choice.*protect", re.I), "chosen-color protection"),
    (re.compile(r"can.t\s+gain\s+life|life.*can.t\s+change", re.I), "can't-gain-life"),
    (re.compile(r"TokenSpec.*EffectAmount|X\s+(\d/\d|tokens?)|number\s+of.*tokens?|token.*equal\s+to", re.I), "X-scaled tokens"),
    (re.compile(r"delayed\s+trigger", re.I), "delayed triggers"),
    (re.compile(r"WheneverYouSacrifice|when.*you\s+sacrifice", re.I), "you-sacrifice trigger"),
    (re.compile(r"each\s+(player|opponent).*upkeep|each\s+upkeep", re.I), "per-opponent upkeep"),
    (re.compile(r"non.hand|free\s+(spell|cast)|cast\s+without\s+paying", re.I), "free-cast / non-hand cast"),
    (re.compile(r"loses?\s+all\s+abilit|GrantKeywordUntilEOT|gains?\s+\[?keyword\]?\s+until", re.I), "lose-abilities / grant-keyword EOT"),
    (re.compile(r"postcombat\s+main|second\s+main", re.I), "postcombat main"),
    (re.compile(r"\bConnive\b", re.I), "connive"),
    (re.compile(r"proliferate", re.I), "proliferate trigger"),
    (re.compile(r"valiant|becomes\s+target", re.I), "valiant / becomes-target"),
    (re.compile(r"double.*power|doubled\s+P/T", re.I), "double power"),
    (re.compile(r"\bExplore\b", re.I), "explore"),

    # Generic primitive-shape gaps (added round 2 from OTHER sample)
    (re.compile(r"TriggerCondition::\w+", re.I), "TriggerCondition::* missing variant"),
    (re.compile(r"TargetFilter\.\w+|TargetFilter\s+lacks|filter\s+lacks|supertype\s+constraint|legendary\s+(permanent|target)", re.I), "TargetFilter missing field"),
    (re.compile(r"EffectAmount::\w+|EffectAmount\s+lacks|HandSize", re.I), "EffectAmount::* missing variant"),
    (re.compile(r"EnchantTarget::\w+|enchant\s+(player|planeswalker)", re.I), "EnchantTarget missing variant"),
    (re.compile(r"Cost::\w+|alternative\s+cost|alt[\s-]cost", re.I), "Cost::* missing variant"),
    (re.compile(r"replacement\s+effect|ReplacementModification", re.I), "replacement effect missing"),
    (re.compile(r"impulse\s+draw|exile\s+top.*may\s+(play|cast)", re.I), "impulse draw"),
    (re.compile(r"discard.*hand.*draw|discard\s+(your|all).*draw", re.I), "discard-then-draw"),
    (re.compile(r"any\s+number\s+of|may\s+reveal|look\s+at\s+the\s+top", re.I), "interactive / hidden-info choice"),
    (re.compile(r"counter\s+target.*targets?|counter.*spell.*condition", re.I), "conditional counterspell"),
    (re.compile(r"\bcompleated\b|phyrexian\s+life", re.I), "compleated"),
    (re.compile(r"whenever.*attacks?\b|WhenAttacks|self.attack\s+trigger", re.I), "attack trigger (self / generic)"),
    (re.compile(r"combat\s+damage\s+to\s+a?\s*player", re.I), "combat-damage-to-player trigger"),
    (re.compile(r"opponent\s+casts?|opponent\s+plays?|whenever\s+an?\s+opponent", re.I), "opponent-action trigger"),
    (re.compile(r"conditional\s+(static|trigger|grant)|conditional\s+(double|first)\s+strike", re.I), "conditional static / grant"),
    (re.compile(r"copy\s+(target|spell)|except\s+it\s+has|retained\s+ability", re.I), "copy / retained-ability"),
    (re.compile(r"attach\s+all|attach\s+(every|each)", re.I), "mass-attach equipment"),
    (re.compile(r"AddCounterAmount|counter.*equal\s+to|counters?\s+for\s+each", re.I), "X-scaled counters"),
    (re.compile(r"top\s+\d+\s+cards|reveal\s+(the\s+)?top|surveil|library\s+manipulation", re.I), "top-of-library lookahead"),
    (re.compile(r"player\s+token|PlayerFilter|each\s+opponent\s+loses", re.I), "per-player effect dispatch"),
    (re.compile(r"\bDSL\s+gap\b", re.I), "DSL gap (unspecified)"),
]


def run(*args: str, cwd: Path = REPO) -> str:
    return subprocess.check_output(args, cwd=cwd, text=True, stderr=subprocess.DEVNULL)


def slugify(name: str) -> str:
    """Match the project's filename convention. Handles apostrophes, accents, and DFCs.

    - Curly + straight apostrophes are DELETED (not separators) so "Akroma's Will" → "akromas_will"
    - Accented characters are transliterated (é → e) before slugging.
    - For "Front // Back" DFC names, only the front face is slugged here; callers wanting all
      faces should use `all_face_slugs`.
    """
    n = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode("ascii")
    n = n.replace("'", "").replace("’", "")  # straight + curly apostrophes
    n = n.split("//")[0].strip()
    return re.sub(r"[^a-z0-9]+", "_", n.lower()).strip("_")


def all_face_slugs(name: str) -> set[str]:
    """Return every plausible on-disk slug for a card. Covers DFCs + combined naming."""
    faces = [f.strip() for f in name.split("//")]
    out = {slugify(faces[0])}
    if len(faces) > 1:
        out.add(slugify(name))            # combined ("front_back")
        out.add(slugify(faces[1]))        # back face alone
    return out


def classify_file(path: Path) -> tuple[str, list[str]]:
    """Return (bucket, todo_lines).
    Bucket ∈ {empty, todo, clean}."""
    text = path.read_text(encoding="utf-8", errors="replace")
    todos = [ln.strip() for ln in text.splitlines() if re.match(r"\s*//\s*TODO\b", ln)]
    if re.search(r"abilities:\s*vec!\[\s*\]\s*,", text):
        return "empty", todos
    if todos:
        return "todo", todos
    return "clean", todos


def classify_todo(line: str) -> str:
    for pat, name in TODO_BUCKETS:
        if pat.search(line):
            return name
    return "OTHER (unclassified)"


def git_count_added(since_days: int) -> int:
    out = run(
        "git",
        "log",
        f"--since={since_days}.days.ago",
        "--diff-filter=A",
        "--name-only",
        "--pretty=format:",
        "--",
        "crates/engine/src/cards/defs/*.rs",
    )
    return sum(1 for ln in out.splitlines() if ln.endswith(".rs"))


def git_count_modified(since_days: int) -> int:
    out = run(
        "git",
        "log",
        f"--since={since_days}.days.ago",
        "--diff-filter=M",
        "--name-only",
        "--pretty=format:",
        "--",
        "crates/engine/src/cards/defs/*.rs",
    )
    return len({ln for ln in out.splitlines() if ln.endswith(".rs")})


def git_recent_card_commits(limit: int = 25) -> list[str]:
    out = run(
        "git",
        "log",
        f"-{limit}",
        "--oneline",
        "--",
        "crates/engine/src/cards/defs/",
    )
    return out.splitlines()


def git_head() -> str:
    return run("git", "rev-parse", "--short", "HEAD").strip()


def git_branch() -> str:
    return run("git", "rev-parse", "--abbrev-ref", "HEAD").strip()


def main() -> int:
    if not DEFS.is_dir():
        print(f"ERROR: {DEFS} not found", file=sys.stderr)
        return 2

    # 1. File scan
    files = sorted(p for p in DEFS.glob("*.rs") if p.name != "mod.rs")
    buckets: dict[str, int] = Counter()
    todo_files: list[Path] = []
    todo_classes: Counter[str] = Counter()
    other_samples: list[tuple[str, str]] = []  # (slug, todo line) — only OTHER bucket
    total_todos = 0
    for f in files:
        bucket, todos = classify_file(f)
        buckets[bucket] += 1
        if todos:
            todo_files.append(f)
            total_todos += len(todos)
            for ln in todos:
                cls = classify_todo(ln)
                todo_classes[cls] += 1
                if cls == "OTHER (unclassified)":
                    other_samples.append((f.stem, ln))

    # 2. Plan correlation (any-face match for DFCs, accent + apostrophe tolerant slugs)
    # Per-group accumulator: counts AND list of authored cards (slug, name, bucket)
    plan_data: dict | None = None
    plan_total = plan_authored = plan_missing = 0
    plan_by_group: dict[str, list[int]] = {}
    plan_authored_cards: dict[str, list[tuple[str, str, str]]] = {}  # group → [(slug, name, bucket)]
    extras: list[str] = []
    missing_card_names: list[tuple[str, str]] = []  # (group, original name)
    plan_generated_at: str | None = None
    file_bucket_by_slug = {f.stem: classify_file(f)[0] for f in files}
    if PLAN.is_file():
        with PLAN.open() as fh:
            plan_data = json.load(fh)
        plan_generated_at = plan_data.get("generated")
        slugs_on_disk = {f.stem for f in files}
        plan_slug_universe: set[str] = set()
        for sess in plan_data.get("sessions", []):
            gid = sess.get("group_id", "?")
            row = plan_by_group.setdefault(gid, [0, 0, 0, 0, 0])  # tot, auth, clean, todo, empty
            cards_in_group = plan_authored_cards.setdefault(gid, [])
            for card in sess.get("cards", []):
                faces = all_face_slugs(card["name"])
                plan_slug_universe |= faces
                row[0] += 1
                hit_slug = next((s for s in faces if s in slugs_on_disk), None)
                if hit_slug:
                    row[1] += 1
                    bucket = file_bucket_by_slug[hit_slug]
                    row[2 + ["clean", "todo", "empty"].index(bucket)] += 1
                    cards_in_group.append((hit_slug, card["name"], bucket))
                else:
                    missing_card_names.append((gid, card["name"]))
        plan_total = sum(r[0] for r in plan_by_group.values())
        plan_authored = sum(r[1] for r in plan_by_group.values())
        plan_missing = plan_total - plan_authored
        extras = sorted(slugs_on_disk - plan_slug_universe)
    extras_count = len(extras)

    # 2b. Categorize extras by commit prefix (W2, W6-cards, W1-B*, etc.)
    extras_by_prefix: Counter[str] = Counter()
    extras_by_month: Counter[str] = Counter()
    if extras:
        for slug in extras:
            try:
                out = subprocess.check_output(
                    ["git", "log", "--diff-filter=A", "--format=%ad|%s", "--date=short",
                     "--", f"crates/engine/src/cards/defs/{slug}.rs"],
                    cwd=REPO, text=True, stderr=subprocess.DEVNULL,
                ).strip().splitlines()
                if not out:
                    extras_by_prefix["NO-GIT-RECORD"] += 1
                    continue
                date, subject = out[-1].split("|", 1)  # earliest add
                m = re.match(r"^([A-Za-z0-9_+\-/]+):", subject)
                prefix = m.group(1) if m else "no-prefix"
                # Bucket Wave-1 ability batches together
                if re.match(r"^W1-B\d+", prefix):
                    prefix = "W1-B* (ability batches)"
                extras_by_prefix[prefix] += 1
                extras_by_month[date[:7]] += 1
            except Exception:
                extras_by_prefix["ERROR"] += 1

    # 3. Git activity
    activity = [(d, git_count_added(d), git_count_modified(d)) for d in WINDOWS]
    recent_commits = git_recent_card_commits(25)

    # 3b. Load previous snapshot for run-over-run diff (8)
    prev: dict = {}
    if PREV_JSON.is_file():
        try:
            prev = json.loads(PREV_JSON.read_text())
        except Exception:
            prev = {}
    prev_buckets = prev.get("file_buckets", {})
    prev_classes = prev.get("todo_classes", {})
    prev_plan = prev.get("plan", {})

    def delta(now: int, then: int | None) -> str:
        if then is None:
            return "—"
        d = now - then
        if d == 0:
            return "·"
        return f"+{d}" if d > 0 else f"{d}"

    # 4. Emit markdown
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    head = git_head()
    branch = git_branch()
    total_files = len(files)
    clean_pct = 100 * buckets["clean"] / total_files if total_files else 0
    plan_cov = 100 * plan_authored / plan_total if plan_total else 0

    out = []
    out.append("<!-- DO NOT EDIT — regenerate via `python3 tools/authoring-report.py` -->")
    out.append("")
    out.append("# Card Authoring Status — Canonical Report")
    out.append("")
    out.append(f"**Generated:** {now}  ")
    out.append(f"**Git:** `{head}` on `{branch}`  ")
    out.append(f"**Source:** `tools/authoring-report.py`")
    out.append("")
    out.append("This document is the single source of truth for card authoring progress. ")
    out.append("It is fully derived from the filesystem, the authoring plan JSON, and `git log`. ")
    out.append("Discussions of authoring strategy should reference this report, not stale prose docs.")
    out.append("")
    out.append("**See [`authoring-status-guide.md`](authoring-status-guide.md) for how to read this report ")
    out.append("and what is intentionally NOT in it.**")
    out.append("")
    out.append("---")
    out.append("")

    # Headline numbers
    out.append("## Headline")
    out.append("")
    has_prev = bool(prev)
    out.append("| Metric | Count" + (" | Δ since last run" if has_prev else "") + " |")
    out.append("| --- | ---: " + ("| ---: " if has_prev else "") + "|")
    headline = lambda label, now, prev_v: (
        f"| {label} | {now:,} | {delta(now, prev_v)} |" if has_prev else f"| {label} | {now:,} |"
    )
    out.append(headline("Card def files on disk", total_files, prev.get("total_files")))
    if plan_data:
        eff_cov = 100 * (plan_authored + extras_count) / plan_total if plan_total else 0
        out.append(headline(f"Authoring-plan target universe (snapshot {plan_generated_at[:10] if plan_generated_at else '?'})",
                            plan_total, prev_plan.get("total")))
        out.append(headline("Plan cards with a def file (any-face match)", plan_authored, prev_plan.get("authored")))
        out.append(headline("Plan cards still missing a def file", plan_missing, prev_plan.get("missing")))
        out.append(headline("Bonus defs (on disk, outside plan)", extras_count, prev_plan.get("extras")))
        out.append(f"| Effective coverage vs plan target | **{eff_cov:.0f}%** ({plan_authored + extras_count:,} / {plan_total:,}) |" + (" — |" if has_prev else ""))
    out.append(headline(f"Clean (no TODO, non-empty abilities)  — {clean_pct:.1f}%", buckets["clean"], prev_buckets.get("clean")))
    out.append(headline("With TODO markers", buckets["todo"], prev_buckets.get("todo")))
    out.append(headline("Empty `abilities: vec![]` placeholders", buckets["empty"], prev_buckets.get("empty")))
    out.append(headline("Total TODO lines across all defs", total_todos, prev.get("total_todos")))
    out.append("")

    # Activity
    out.append("## Authoring activity (git, by window)")
    out.append("")
    out.append("| Window | New files added | Existing files modified |")
    out.append("| --- | ---: | ---: |")
    for d, a, m in activity:
        label = f"last {d} days" if d <= 90 else f"last {d // 365} year"
        out.append(f"| {label} | {a:,} | {m:,} |")
    out.append("")

    # Bonus defs (outside-plan) breakdown
    if extras and extras_by_prefix:
        out.append("## Bonus defs outside the plan")
        out.append("")
        out.append(f"The plan was a one-shot snapshot at {plan_generated_at[:10] if plan_generated_at else '?'}; ")
        out.append("any card authored before plan generation OR added since (without re-running the planner) ")
        out.append("appears here. These are real cards, not noise — typically EDH staples, ability-batch ")
        out.append("reference cards, or sample cards shipped alongside primitive batches.")
        out.append("")
        out.append("| Source (commit prefix) | Count |")
        out.append("| --- | ---: |")
        for prefix, n in extras_by_prefix.most_common(15):
            out.append(f"| `{prefix}` | {n} |")
        out.append("")
        if extras_by_month:
            out.append("**By month added:** " + ", ".join(
                f"{ym}: {n}" for ym, n in sorted(extras_by_month.items())
            ))
            out.append("")

    # Plan groups (with quality split: clean/todo/empty)
    if plan_data and plan_by_group:
        out.append("## Coverage by authoring-plan group")
        out.append("")
        out.append("\"Clean\" / \"TODO\" / \"Empty\" subdivide the *authored* count by file quality. ")
        out.append("Groups with high authored-but-not-clean ratios are TODO-debt — the cards exist but ")
        out.append("are blocked on engine primitives.")
        out.append("")
        out.append("| Group | Auth / Total | % | Clean | TODO | Empty |")
        out.append("| --- | ---: | ---: | ---: | ---: | ---: |")
        for gid, row in sorted(plan_by_group.items(), key=lambda kv: (-kv[1][0], kv[0])):
            tot, auth, clean, todo, empty = row
            pct = 100 * auth / tot if tot else 0
            out.append(f"| `{gid}` | {auth} / {tot} | {pct:.0f}% | {clean} | {todo} | {empty} |")
        out.append("")

        # Lagging groups callout — with quality-of-authored detail (9)
        laggards = sorted(
            ((gid, r) for gid, r in plan_by_group.items() if r[0] >= 5 and r[1] / r[0] < 0.5),
            key=lambda kv: kv[1][1] / kv[1][0],
        )
        if laggards:
            out.append("### Lagging groups (≥5 cards in plan, <50% authored)")
            out.append("")
            out.append("For each lagging group, the table below lists the cards that ARE authored ")
            out.append("with their quality bucket. If most are `todo` or `empty`, the group is ")
            out.append("**engine-blocked** (cards exist but need primitives). If they are `clean`, ")
            out.append("the group is just **unwritten** (need authoring effort). This split tells ")
            out.append("you which kind of next-step work would unblock the group.")
            out.append("")
            for gid, (tot, auth, clean, todo, empty) in laggards:
                pct = 100 * auth / tot if tot else 0
                verdict = (
                    "**engine-blocked**" if (todo + empty) > clean
                    else "**unwritten**" if clean >= todo + empty and auth < tot
                    else "mixed"
                )
                out.append(f"#### `{gid}` — {auth} / {tot} ({pct:.0f}%), authored split: {clean} clean / {todo} todo / {empty} empty — {verdict}")
                out.append("")
                cards = plan_authored_cards.get(gid, [])
                if cards:
                    out.append("| Card | Slug | Bucket |")
                    out.append("| --- | --- | --- |")
                    for slug, name, bucket in sorted(cards, key=lambda x: x[1].lower()):
                        out.append(f"| {name} | `{slug}` | {bucket} |")
                    out.append("")

    # TODO classification
    if todo_classes:
        out.append("## TODO classification (top 25)")
        out.append("")
        out.append("Each TODO line is matched against engine-gap patterns. \"OTHER\" means unclassified — ")
        out.append("either a stale TODO (primitive now exists), a card-specific note, or a gap not yet ")
        out.append("in the classifier (`tools/authoring-report.py` `TODO_BUCKETS`). The OTHER bucket is ")
        out.append("the next thing to triage when the classifier table is grown.")
        out.append("")
        if has_prev:
            out.append("| Gap bucket | TODO lines | Δ since last run |")
            out.append("| --- | ---: | ---: |")
            for name, n in todo_classes.most_common(25):
                out.append(f"| {name} | {n} | {delta(n, prev_classes.get(name))} |")
        else:
            out.append("| Gap bucket | TODO lines |")
            out.append("| --- | ---: |")
            for name, n in todo_classes.most_common(25):
                out.append(f"| {name} | {n} |")
        out.append("")
        if len(todo_classes) > 25:
            tail = sum(n for _, n in todo_classes.most_common()[25:])
            out.append(f"_…and {len(todo_classes) - 25} more buckets totaling {tail} lines._")
            out.append("")

    # OTHER samples (7) — raw lines so you can read patterns and propose new buckets
    if other_samples:
        out.append("### Raw OTHER samples (read these to design new classifier buckets)")
        out.append("")
        out.append(f"Showing {min(SAMPLE_OTHER_N, len(other_samples))} of {len(other_samples)} ")
        out.append("unclassified TODO lines. If two or three of these have a common theme, that's a ")
        out.append("new bucket to add to `TODO_BUCKETS` in `tools/authoring-report.py`. Sample is ")
        out.append("deterministic (sorted by slug).")
        out.append("")
        out.append("```")
        # Deterministic sample: sort by slug, take stride across the list
        sorted_samples = sorted(other_samples, key=lambda x: x[0])
        n = len(sorted_samples)
        if n <= SAMPLE_OTHER_N:
            chosen = sorted_samples
        else:
            stride = n / SAMPLE_OTHER_N
            chosen = [sorted_samples[int(i * stride)] for i in range(SAMPLE_OTHER_N)]
        for slug, line in chosen:
            out.append(f"{slug}: {line[:140]}")
        out.append("```")
        out.append("")

    # Recent commits
    out.append("## Recent card-touching commits")
    out.append("")
    out.append("```")
    out.extend(recent_commits[:25])
    out.append("```")
    out.append("")

    # NO-GIT-RECORD sentinel — uncommitted scratch files would land here
    no_git_count = extras_by_prefix.get("NO-GIT-RECORD", 0)
    err_count = extras_by_prefix.get("ERROR", 0)
    if no_git_count or err_count:
        out.append("## ⚠ Sentinel — files outside git")
        out.append("")
        out.append(f"- **NO-GIT-RECORD** extras: {no_git_count} (uncommitted scratch files — review and either commit or delete)")
        if err_count:
            out.append(f"- **ERROR** extras: {err_count} (git-log lookup failed — investigate)")
        out.append("")

    # Missing-from-plan list (sidecar file)
    if missing_card_names:
        MISSING_OUT.parent.mkdir(parents=True, exist_ok=True)
        with MISSING_OUT.open("w", encoding="utf-8") as fh:
            fh.write(f"# Authoring plan — cards still missing on disk\n")
            fh.write(f"# Generated: {now}  Git: {head}  Plan snapshot: {plan_generated_at[:10] if plan_generated_at else '?'}\n")
            fh.write(f"# Format: <group>\\t<card name>   ({len(missing_card_names)} cards)\n\n")
            for gid, name in sorted(missing_card_names):
                fh.write(f"{gid}\t{name}\n")
        out.append("## Missing card-defs sidecar")
        out.append("")
        out.append(f"The full list of {len(missing_card_names)} plan cards still missing on disk is at ")
        out.append(f"`{MISSING_OUT.relative_to(REPO)}` (tab-separated `group<TAB>name`, sorted by group). ")
        out.append("Use it as a batch-author worklist.")
        out.append("")

    # Footer
    out.append("---")
    out.append("")
    out.append("## How to update this report")
    out.append("")
    out.append("```")
    out.append("python3 tools/authoring-report.py")
    out.append("```")
    out.append("")
    out.append("To extend the TODO classifier, add `(re.compile(...), \"bucket name\")` tuples to ")
    out.append("`TODO_BUCKETS` in `tools/authoring-report.py` and re-run.")
    out.append("")
    out.append("To change the universe target or plan source, edit `PLAN` at the top of the script.")
    out.append("")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(out), encoding="utf-8")

    # Save snapshot for next run's diff (8). This OVERWRITES every run.
    # If you want a longer-window comparison, manually copy this file before re-running.
    snapshot = {
        "generated": now,
        "git_head": head,
        "total_files": total_files,
        "total_todos": total_todos,
        "file_buckets": dict(buckets),
        "todo_classes": dict(todo_classes),
        "plan": {
            "total": plan_total,
            "authored": plan_authored,
            "missing": plan_missing,
            "extras": extras_count,
        },
    }
    PREV_JSON.write_text(json.dumps(snapshot, indent=2), encoding="utf-8")

    # stdout summary
    print(f"Wrote {OUT.relative_to(REPO)}")
    if missing_card_names:
        print(f"Wrote {MISSING_OUT.relative_to(REPO)} ({len(missing_card_names)} missing)")
    print(f"  {total_files:,} files | clean {buckets['clean']:,} ({clean_pct:.1f}%) | "
          f"todo {buckets['todo']:,} | empty {buckets['empty']:,}")
    if plan_data:
        print(f"  plan: {plan_authored:,} / {plan_total:,} ({plan_cov:.1f}%) authored, "
              f"{plan_missing:,} missing, {extras_count:,} extras")
        if no_git_count or err_count:
            print(f"  ⚠ {no_git_count} NO-GIT-RECORD, {err_count} ERROR (see report sentinel section)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
