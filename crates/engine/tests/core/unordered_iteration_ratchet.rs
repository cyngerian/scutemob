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
//! whole-token `HashMap` / `HashSet` occurrences in the resolution path, per file, as a
//! ratchet (widened from the `<`-suffixed annotation spelling to whole-token matching in the
//! PB-DX7 review's H1 fix — see below).
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
//! another crate, then `Foo` used here) would not be seen. The block-comment gap is genuinely
//! obscure code review would likely reject. **The alias gap is not, and neither is
//! type-inferred construction, which this doc previously (wrongly) called equally obscure --
//! `let x = HashSet::new();` with the type inferred from use is completely ordinary Rust, and
//! before the H1 fix it (along with imports, parameter restatements and empty-literal
//! arguments -- everything that is not the `<`-suffixed annotation spelling) accounted for 58
//! of the tree's 85 real occurrences, undercounted to 0 in three files entirely.** Stated
//! rather than defended against, because a gate that overclaims its reach is what PB-DX7
//! exists to fix -- and this file itself was found, by execution, to be exactly such a gate
//! before the widened needle.
//!
//! # The classification as re-verified at HEAD by PB-DX7 (and re-verified again for review H1)
//!
//! **Corrected 2026-08-11 (review H1)**: the count below is now **85**, not 27 — the original
//! needle (`HashMap<` / `HashSet<`, the type-ANNOTATION spelling only) missed every
//! CONSTRUCTION-style occurrence (`HashSet::new()`, `HashMap::with_capacity(n)`, an empty
//! `&HashSet::new()` literal argument, a `use` import, a turbofish), which is the majority
//! idiom in this tree — `rules/casting.rs` alone has 9 `HashSet::new()` sites and matched the
//! old needle exactly zero times. All 85 occurrences were traced to a named
//! variable/field/parameter and classified (not assumed from the original 27's summary); every
//! one is still clean:
//!
//! - `replacement.rs` (21) — the only file that genuinely *iterates* a `HashSet` to an
//!   outcome, via the `already_applied` family. All three `already_applied.into_iter()` sites
//!   collect into a `Vec` and `sort_by_key(|id| id.0)` immediately (the fix-cycle repair, and
//!   the source comments say so); the remaining declarations, parameter restatements and two
//!   `.clone()`s into `applied` are the same family. 7 empty `&HashSet::new()` arguments feed
//!   `find_applicable`, which is `contains`-only (`:54`).
//! - `effects/mod.rs` (19) — `top_ids` is iterated at `:5958`, but only to partition into
//!   `matched_ids` / `unmatched_ids`, both `sort_by_key(|id| id.0)`'d before use.
//!   `target_remaps` is `insert`/`get`-only across its 1 declaration + 1 field annotation + 5
//!   construction sites (the seed's original claim, still true). `seen_names` is a membership
//!   filter. The rest are 1 `use` import and 9 empty `&HashSet::new()` arguments.
//! - `abilities.rs` (**6**, was 11 — PB-DX15a converted `left_battlefield` and the
//!   CR 603.10a look-back set to `BTreeSet`), `sba.rs` (8), `commander.rs` (4),
//!   `engine.rs` (3) — `seen`,
//!   `left_battlefield`, `arrived_in_graveyard_this_batch`, `chars_map`, `reported_incomplete`,
//!   `sources_on_bf` are all `contains`/`get`-only membership or lookup tables, each counted
//!   once per declaration/parameter/construction site rather than once per file; the rest are
//!   `use` imports and empty-literal arguments.
//! - `casting.rs` (9), `resolution.rs` (7), `turn_actions.rs` (3) — **entirely newly visible
//!   under the widened needle** (the old counter measured all three at 0). `casting.rs` is 9
//!   independent `let mut seen(_x) = HashSet::new();` dedup sites, each `insert`-only.
//!   `resolution.rs` is 7 empty `&HashSet::new()` arguments. `turn_actions.rs` is 1
//!   construction + 2 empty-literal arguments. All inspected individually; none iterated to an
//!   outcome.
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

/// Per-file ceilings on whole-token `HashMap` / `HashSet` occurrences, comment-stripped.
/// **A count may only decrease.** Any file under [`SCAN_DIRS`] not listed here is pinned at
/// zero.
///
/// **Re-measured for PB-DX7 review H1 (2026-08-11)**, after widening the counter from the
/// `<`-suffixed annotation spelling (which found 27) to whole-token matching (which finds
/// **85** — see `unordered_container_count`'s doc for why 27 undercounted). Every one of the
/// 85 occurrences was traced to a named variable/parameter/field and classified; see the
/// module doc's "classification as re-verified" section below for the full per-file account.
/// No genuinely new unordered-iteration-to-outcome site was found — every additional
/// occurrence beyond the original 27 is one of: a `use` import line, a function parameter
/// restating an already-classified set's type, a `.clone()` of an already-classified set, or
/// an empty-literal `&HashSet::new()` argument (which cannot have an iteration-order hazard —
/// it is always empty at the call site).
const UNORDERED_CEILINGS: &[(&str, usize)] = &[
    // 21 — the `already_applied` family (CR 616.1 replacement-ordering; sorted by
    // `ReplacementId` before use, PB-DP9 fix cycle) across its declarations, parameter
    // restatements, and two `.clone()`s into `applied`; plus 7 empty `&HashSet::new()`
    // arguments to `find_applicable` (contains-only there, per the original audit).
    ("rules/replacement.rs", 21),
    // 19 — `target_remaps` (insert/get-only; 1 declaration + 1 field annotation + 5
    // `HashMap::new()` construction sites across the functions that build `EffectContext`),
    // `top_ids` (iterated, but both output vectors are `sort_by_key`'d before use),
    // `seen_names` (declaration + construction), 1 `use` import, and 9 empty
    // `&HashSet::new()` arguments.
    ("effects/mod.rs", 19),
    // 6 — `seen` (x2 declaration+construction pairs across two helper fns) + 2 `use`
    // imports. All membership filters; none iterated.
    //
    // **11 → 6, PB-DX15a (`scutemob-216`).** `left_battlefield` and the CR 603.10a
    // look-back set became `BTreeSet`s. The look-back set had to be touched anyway
    // (rider `OOS-DX24-7` splits it into a whole-batch set and a strictly-earlier set),
    // and this ratchet fired on the first draft, which added three more `HashSet`s and
    // pushed the count to 15. They were `contains`-only, i.e. legitimately category (a),
    // and converting was still the better answer: it costs nothing at this size, it
    // moves the ceiling DOWN instead of asking for it to be raised, and it removes the
    // question entirely from a function PB-DP9 re-executes wholesale after every
    // suspended `EffectChoiceQuestion` (`OOS-DP9-10`).
    //
    // **6 -> 8, PB-DX35 (2026-09, `OOS-DX4-2`).** `pb_dx35_trigger_modal_plan_tests::
    // t9_site3_agrees_with_the_shared_plan_by_value` (an internal `#[cfg(test)]` unit
    // test -- `trigger_ability_target_requirements` is a bare private `fn`, unreachable
    // from an external integration test) builds a one-entry `HashMap<String,
    // CardDefinition>` to pass to `enrich_spec_from_def`, which only ever calls
    // `.get(&spec.name)` on it -- category (a), a lookup table never iterated. 1 `use`
    // import + 1 `HashMap::from(..)` construction site = +2.
    ("rules/abilities.rs", 8),
    // 9 — 9x `let mut seen(_x) = std::collections::HashSet::new();`, one per splice/dedup
    // site; each is `insert`-only, never iterated. **Newly visible under the widened
    // needle** (the `<`-suffixed counter found 0 here) — inspected individually, all clean.
    ("rules/casting.rs", 9),
    // 8 — `chars_map: HashMap<ObjectId, Characteristics>` (1 declaration + 4 parameter
    // restatements across SBA helpers, a lookup table, never iterated), 1 `use` import, 2
    // empty `&HashSet::new()` arguments.
    ("rules/sba.rs", 8),
    // 7 — 7x `&std::collections::HashSet::new()` empty-set arguments. **Newly visible under
    // the widened needle.** Inspected individually, all clean.
    ("rules/resolution.rs", 7),
    // 4 — `reported_incomplete` and `seen` (each declaration + construction), both membership
    // filters in deck validation.
    ("rules/commander.rs", 4),
    // 3 — `sources_on_bf` (a membership filter) + 2 empty `&HashSet::new()` arguments.
    ("rules/engine.rs", 3),
    // 3 — 1x `HashSet::new()` construction + 2x empty `&HashSet::new()` arguments. **Newly
    // visible under the widened needle.** Inspected individually, all clean.
    ("rules/turn_actions.rs", 3),
];

/// Non-vacuity floor: the scan must find a real codebase. Deliberately below the live 85.
const MIN_TOTAL_FOUND: usize = 60;
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

/// Whole-token `HashMap` / `HashSet` occurrences in `src`, with `//` line comments stripped.
///
/// **PB-DX7 review fix (H1, 2026-08-11).** The original counter matched only the `<`-suffixed
/// type-annotation spelling (`HashMap<`), which is the MINORITY idiom in this tree — measured:
/// 27 annotation-style occurrences vs 85 total whole-token occurrences across the same scan
/// roots. `rules/casting.rs` alone carries 9 `HashSet::new()` construction sites and matched
/// the old needle exactly zero times, so it was pinned at ceiling 0 while genuinely carrying 9 —
/// the exact `OOS-DP9-10` defect (a new unordered-iteration-to-outcome site going unnoticed),
/// reproduced inside this gate's own file by an executed revert (see `pb-DX7-execution-notes.md`
/// §14). This counts the bare identifier as a whole token regardless of what follows it —
/// annotation (`HashMap<K, V>`), construction (`HashMap::new()`, `HashMap::with_capacity(n)`),
/// turbofish (`.collect::<HashSet<_>>()`), or a bare mention anywhere else a `HashMap`/`HashSet`
/// name can occur in real (non-string, non-comment) source.
fn unordered_container_count(src: &str) -> usize {
    let decommented: String = src
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let b = decommented.as_bytes();
    // Built at runtime so this file's own source cannot match the needle if it is ever
    // brought under a scan root.
    let map_needle = format!("Hash{}", "Map");
    let set_needle = format!("Hash{}", "Set");
    let mut count = 0usize;
    for needle in [map_needle.as_str(), set_needle.as_str()] {
        let mut from = 0usize;
        while let Some(rel) = decommented[from..].find(needle) {
            let at = from + rel;
            let after = at + needle.len();
            let ok_before = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
            let ok_after =
                after >= b.len() || !(b[after].is_ascii_alphanumeric() || b[after] == b'_');
            if ok_before && ok_after {
                count += 1;
            }
            from = at + 1;
        }
    }
    count
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

    // Positive control: annotation spelling.
    assert_eq!(
        unordered_container_count("let x: HashMap<A, B> = ...; let y: HashSet<C> = ...;"),
        2,
        "positive control failed: the counter missed a bare declaration"
    );
    // PB-DX7 review H1: construction spelling, no annotation at all -- this is the shape
    // `rules/casting.rs`'s 9 sites use, and the old `<`-suffixed needle could never see it.
    assert_eq!(
        unordered_container_count("let x = std::collections::HashSet::new();"),
        1,
        "H1 positive control failed: a type-inferred construction (no `<` anywhere on the \
         line) must still count -- this is the majority idiom in the tree, not an edge case"
    );
    // H1: both spellings on the same line count separately -- this is a whole-token CENSUS,
    // not a per-variable dedup.
    assert_eq!(
        unordered_container_count("let mut m: HashMap<K, V> = HashMap::new();"),
        2,
        "H1 positive control failed: annotation + construction on one line must count as 2 \
         occurrences of the token, not 1 occurrence of the variable"
    );
    // H1: turbofish and `use` import spellings, both real shapes found in the tree.
    assert_eq!(
        unordered_container_count("let s = items.collect::<HashSet<_>>();"),
        1,
        "H1 positive control failed: turbofish construction must count"
    );
    assert_eq!(
        unordered_container_count("use std::collections::HashMap;"),
        1,
        "H1 positive control failed: a bare `use` import must count (it is real surface, even \
         though not itself a hazard)"
    );
    // H1 token-boundary control: a name that merely CONTAINS "HashMap"/"HashSet" as a
    // substring must not match -- the whole point of moving off blind substring counting.
    assert_eq!(
        unordered_container_count("let x: MyHashMapWrapper = MyHashMapWrapper::new();"),
        0,
        "H1 token-boundary control failed: `HashMap` matched inside `MyHashMapWrapper`"
    );
    assert_eq!(
        unordered_container_count("struct HashMapper { x: u8 }"),
        0,
        "H1 token-boundary control failed: `HashMap` matched inside `HashMapper`"
    );
    // Occurrences spanning a line wrap still count once each -- not because whitespace is
    // squeezed (it no longer is; whole-token matching does not need to be), but because the
    // token itself (`HashMap`) never contains a newline regardless of how its generic
    // arguments are wrapped.
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
