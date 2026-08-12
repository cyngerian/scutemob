//! **PB-DX7 (`scutemob-207`) — the OOS-DP9-10 residual gate.**
//!
//! # What this exists for
//!
//! PB-DP9 made a spell's resolution *suspendable*: `EffectChoiceQuestion` pauses inside
//! `execute_effect`, the player answers, and the engine **re-executes the whole resolution**
//! from the top. That re-execution is only sound if the resolution is deterministic — if the
//! second pass can reach a different outcome than the first, the answer the player gave is
//! applied to a state they were never shown.
//!
//! `OOS-DP9-10` is that hazard. It was filed as "`EffectContext.target_remaps` is a
//! `HashMap`", audited clean on that specific claim, and then **re-scoped by PB-DP9's closing
//! review**: the premise is RESOLUTION-scoped, not effect-scoped, so *any* statement anywhere
//! in a resolution that reaches an outcome through `HashMap`/`HashSet` iteration order can
//! diverge between passes. Rust's `RandomState` re-keys per map, so this diverges **within a
//! single process**, not merely across processes or machines.
//!
//! The widened audit ran in PB-DP9's fix cycle and found and fixed five live sites
//! (`Effect::ChooseCreatureType` and its ETB twin in `replacement.rs`, both `max_by_key` over
//! a `HashMap` where a tie is the common case; `abilities.rs`'s
//! `AnyCreatureYouControlBatchCombatDamage` map, which queued CR 603.3b triggers in *map*
//! order, i.e. stack order; `turn_actions.rs`'s CR 603.7b delayed-trigger map; and
//! `replacement.rs`'s `PendingZoneChange.already_applied`, built from a `HashSet` without the
//! sort its own sibling site documents as "load-bearing, not cosmetic" — and that field is fed
//! element-by-element into the state hash).
//!
//! **The residual the seed left open, verbatim: "there is no gate for the shape."** A new
//! unordered-iteration-to-outcome site is a live hazard the moment a card co-locates it with a
//! search/scry/surveil. This file is that gate.
//!
//! # What it actually checks, and what it deliberately does not
//!
//! A source scan cannot do dataflow, so it does **not** try to decide "does this container's
//! iteration order reach an outcome?" — that is the judgement call this gate exists to
//! *force a human to make*, not to make itself. What it pins is the **surface**: the number of
//! `HashMap<` / `HashSet<` type occurrences in the resolution path, per file, as a ratchet.
//!
//! - A count may only ever go **down**. Introducing an unordered container in the resolution
//!   path exceeds its file's ceiling and fails with the classification question.
//! - Converting one to `BTreeMap`/`BTreeSet` (or deleting it) leaves the count below the
//!   ceiling and fails asking you to tighten the number, so the ratchet cannot rust.
//! - **Every other `.rs` file** under the scanned roots is pinned at **zero**, so a new
//!   container in a *new* file is caught too. This is the load-bearing half: a ceiling table
//!   listing only the files that have containers today is a gate that checks the channel it
//!   was written for, and a new channel is invisible to it — the exact failure mode
//!   `MR-M11-01` and `OOS-DP7-11` are both instances of.
//!
//! This is the same source-scan ratchet technique as SR-25's `bare_lookup_ratchet`, SR-5's
//! keyword registry and SR-8's protocol fingerprint. As there, the needle strings live here in
//! the test, never in the scanned files, so the scan cannot match its own source; `//` line
//! comments are stripped and all whitespace removed before counting, so the numbers are
//! insensitive to rustfmt line-wrapping.
//!
//! Known limitation, shared with every sibling source-scan gate: only `//` line comments are
//! stripped, not block comments, and a type aliased elsewhere (`type Foo = HashMap<..>;` in
//! another crate, then `Foo` used here) would not be seen. Both take deliberately obscure code
//! that review would reject; they are stated rather than defended against, because a gate that
//! overclaims its reach is what PB-DX7 exists to fix.
//!
//! # The classification as re-verified at HEAD by PB-DX7
//!
//! All 27 occurrences below were re-inspected (not assumed from the seed's summary), and every
//! one is still clean:
//!
//! - `replacement.rs` — the only file that genuinely *iterates* a `HashSet` to an outcome. All
//!   three `already_applied.into_iter()` sites collect into a `Vec` and `sort_by_key(|id| id.0)`
//!   immediately (the fix-cycle repair, and the source comments say so). The five
//!   `.iter().copied().collect()` sites read the *`Vec`* field into a set — construction, not
//!   unordered iteration. Inside `find_applicable` the set is `contains`-only (`:54`).
//! - `effects/mod.rs` — `top_ids` is iterated at `:5958`, but only to partition into
//!   `matched_ids` / `unmatched_ids`, both of which are `sort_by_key(|id| id.0)`'d before use.
//!   `target_remaps` is `insert`/`get`-only (the seed's original claim, still true).
//!   `seen_names` is a membership filter.
//! - `sba.rs` (`chars_map`), `abilities.rs` (`seen`, `left_battlefield`,
//!   `arrived_in_graveyard_this_batch`), `commander.rs` (`seen`, `reported_incomplete`),
//!   `engine.rs` (`sources_on_bf`) — all `contains`/`get`-only membership or lookup tables;
//!   none is iterated at all.
//!
//! **This gate does not close OOS-DP9-10's determinism question for all time**, and must not be
//! read as doing so: it cannot see an outcome reached through iteration of a container that
//! already exists within its ceiling. What it closes is the residual as filed — the absence of
//! any mechanism at all that makes a *new* site loud.

use std::fs;
use std::path::{Path, PathBuf};

/// Roots scanned. These are the resolution path: everything a suspended-and-replayed
/// resolution can execute. `crates/engine/src/testing/` is deliberately excluded — the replay
/// harness and script schema are test infrastructure, not resolution, and they carry 20
/// containers between them that would drown the signal.
const SCAN_DIRS: [&str; 3] = [
    "crates/engine/src/rules",
    "crates/engine/src/effects",
    "crates/engine/src/state",
];

/// Per-file ceilings on `HashMap<` / `HashSet<` occurrences, comment-stripped and
/// whitespace-insensitive. **A count may only decrease.** Any file under [`SCAN_DIRS`] not
/// listed here is pinned at zero.
///
/// Measured at HEAD by PB-DX7 (2026-08-11); see the module docs for the per-site
/// classification behind each number.
const UNORDERED_CEILINGS: &[(&str, usize)] = &[
    // 11 — the only genuine iteration site in the resolution path, and all three
    // `into_iter()` collections are sorted by `ReplacementId` immediately (PB-DP9 fix cycle).
    // The remaining eight are `Vec` -> set construction and `contains`-only reads.
    ("rules/replacement.rs", 11),
    // 5 — `chars_map: HashMap<ObjectId, Characteristics>`, threaded through four SBA helpers
    // as a lookup table. Never iterated.
    ("rules/sba.rs", 5),
    // 5 — `seen` (x2), `left_battlefield`, `arrived_in_graveyard_this_batch` (declaration +
    // parameter). All membership filters; none iterated.
    ("rules/abilities.rs", 5),
    // 3 — `target_remaps` (insert/get-only, the audited-clean subject of the seed's original
    // narrower claim), `top_ids` (iterated, but both output vectors are sorted before use),
    // `seen_names` (membership filter).
    ("effects/mod.rs", 3),
    // 2 — `reported_incomplete` and `seen`, both membership filters in deck validation.
    ("rules/commander.rs", 2),
    // 1 — `sources_on_bf`, a membership filter.
    ("rules/engine.rs", 1),
];

/// Non-vacuity floor: the scan must find a real codebase. Deliberately below the live 27.
const MIN_TOTAL_FOUND: usize = 20;
/// The scan must actually walk a meaningful number of files.
const MIN_FILES_SCANNED: usize = 20;

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/engine -> crates -> workspace root
    p.pop();
    p.pop();
    p
}

fn walk_rs(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_rs(&p, acc);
        } else if p.extension().is_some_and(|x| x == "rs") {
            acc.push(p);
        }
    }
}

/// `HashMap<` / `HashSet<` occurrences in `src`, with `//` line comments stripped and all
/// whitespace removed (so a rustfmt-wrapped `HashMap<\n    K,\n    V,\n>` counts once, and a
/// comment quoting the needle counts zero).
fn unordered_container_count(src: &str) -> usize {
    let decommented: String = src
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let squeezed: String = decommented.chars().filter(|c| !c.is_whitespace()).collect();
    // Built at runtime so this file's own source cannot match the needle if it is ever
    // brought under a scan root.
    let map_needle = format!("Hash{}<", "Map");
    let set_needle = format!("Hash{}<", "Set");
    squeezed.matches(&map_needle).count() + squeezed.matches(&set_needle).count()
}

/// Every scanned file → its live count, keyed by path relative to `crates/engine/src`.
fn live_counts() -> Vec<(String, usize)> {
    let root = workspace_root();
    let mut out = Vec::new();
    for dir in SCAN_DIRS {
        let mut files = Vec::new();
        walk_rs(&root.join(dir), &mut files);
        files.sort();
        for f in files {
            let src = fs::read_to_string(&f).expect("readable engine source");
            let rel = f
                .strip_prefix(root.join("crates/engine/src"))
                .expect("under crates/engine/src")
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, unordered_container_count(&src)));
        }
    }
    out
}

/// The scanner works: it finds a real tree, the counter has positive and negative controls,
/// and the comment/whitespace handling behaves as the module docs claim.
#[test]
fn unordered_scanner_is_not_vacuous() {
    let live = live_counts();
    assert!(
        live.len() >= MIN_FILES_SCANNED,
        "only {} files scanned under {:?}; the walker is broken (expected >= {})",
        live.len(),
        SCAN_DIRS,
        MIN_FILES_SCANNED
    );
    let total: usize = live.iter().map(|(_, n)| n).sum();
    assert!(
        total >= MIN_TOTAL_FOUND,
        "the scan found only {total} unordered containers across the resolution path; the \
         counter is broken (expected >= {MIN_TOTAL_FOUND})"
    );

    // Positive control.
    assert_eq!(
        unordered_container_count("let x: HashMap<A, B> = ...; let y: HashSet<C> = ...;"),
        2,
        "positive control failed: the counter missed a bare declaration"
    );
    // Line-wrap control (the whole reason whitespace is squeezed).
    assert_eq!(
        unordered_container_count("let x: HashMap<\n    ObjectId,\n    Characteristics,\n> = z;"),
        1,
        "line-wrap control failed: a rustfmt-wrapped declaration must count once"
    );
    // Comment control.
    assert_eq!(
        unordered_container_count("// this mentions HashMap< and HashSet< in prose\nlet a = 1;"),
        0,
        "comment control failed: a commented needle must not count"
    );
    // Negative control: the ordered replacements must not count.
    assert_eq!(
        unordered_container_count("let x: BTreeMap<A, B> = ...; let y: BTreeSet<C> = ...;"),
        0,
        "negative control failed: BTreeMap/BTreeSet are the FIX, not the hazard"
    );

    // Every ceiling entry names a file that exists (a dead entry would silently weaken the
    // zero-pin below by excusing a path that is never scanned).
    let paths: Vec<&str> = live.iter().map(|(p, _)| p.as_str()).collect();
    for (file, _) in UNORDERED_CEILINGS {
        assert!(
            paths.contains(file),
            "UNORDERED_CEILINGS names `{file}`, which the scan does not reach — dead entry \
             (file renamed or moved?). Fix the path; do not delete the entry, or its \
             containers silently leave the gate."
        );
    }
}

/// **OOS-DP9-10 residual.** The unordered-container surface of the resolution path is pinned
/// per file and may only shrink; every unlisted file is pinned at zero.
#[test]
fn unordered_container_surface_is_ratcheted() {
    let live = live_counts();
    let mut over: Vec<String> = Vec::new();
    let mut under: Vec<String> = Vec::new();

    for (path, count) in &live {
        let ceiling = UNORDERED_CEILINGS
            .iter()
            .find(|(f, _)| f == path)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        if *count > ceiling {
            over.push(format!("{path}: {count} > ceiling {ceiling}"));
        } else if *count < ceiling {
            under.push(format!("{path}: {count} < ceiling {ceiling}"));
        }
    }

    assert!(
        over.is_empty(),
        "\n\nNew `HashMap`/`HashSet` in the resolution path:\n  {}\n\n\
         PB-DP9 re-executes a WHOLE resolution after a suspended `EffectChoiceQuestion` is \
         answered, so any statement in it that reaches an outcome through map/set iteration \
         order can diverge between passes — and Rust's `RandomState` re-keys per map, so this \
         diverges within a single process, not merely across machines (OOS-DP9-10).\n\n\
         Classify the container you just added:\n  \
         (a) `contains`/`get`-only, or a lookup table that is never iterated → raise this \
         file's ceiling in UNORDERED_CEILINGS and say which, in the entry itself;\n  \
         (b) iterated, but every consumer sorts before use → same, and name the sort site;\n  \
         (c) iterated to an outcome (a `max_by_key`, a trigger queue order, a field that \
         reaches the state hash) → this is the defect. Use `BTreeMap`/`BTreeSet`, or sort \
         explicitly before use. `replacement.rs`'s `already_applied` is the worked example, \
         sort included.\n",
        over.join("\n  ")
    );

    assert!(
        under.is_empty(),
        "\n\nThese ceilings are now loose:\n  {}\n\n\
         A container was converted or deleted — good. Lower the ceiling in \
         UNORDERED_CEILINGS to the live count so the ratchet cannot rust back open. (Run \
         `emits_the_live_unordered_counts` below with --nocapture for the numbers.)\n",
        under.join("\n  ")
    );
}

/// Prints the live per-file counts, for pasting into [`UNORDERED_CEILINGS`] when a ceiling is
/// tightened. Always passes; it exists to be read, in the shape SR-25's ratchet established.
#[test]
fn emits_the_live_unordered_counts() {
    println!("live unordered-container counts (resolution path):");
    let mut total = 0usize;
    for (path, count) in live_counts() {
        if count > 0 {
            println!("    (\"{path}\", {count}),");
            total += count;
        }
    }
    println!("  total: {total}");
}
