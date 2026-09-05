//! PB-RS1 roster sweep: which card defs use one of the four library-top-N effects
//! (`Effect::Scry`, `Effect::Surveil`, `Effect::RevealAndRoute`, `Effect::LookAtTopThenPlace`)
//! whose read/write ends were reconciled with `draw_card` (CR 121.1) in this PB.
//!
//! **Enumerated from `all_cards()`, not grep (SR-34/36).** A `grep` baseline (47 distinct
//! files: Scry 20, RevealAndRoute 18, Surveil 9, LookAtTopThenPlace 3, some overlapping) is
//! calibration only -- it misses macro-generated/re-exported defs and can over-count
//! comments. This test is the measured deliverable.
//!
//! **Nested walk, not a top-level match.** The four effects can appear anywhere in a def's
//! effect tree -- nested under `Sequence`, `ForEach`, `Conditional`, a triggered ability's
//! effect, an activated ability's effect, or a mode. A shallow top-level scan under-counts
//! (this is the exact "hole in the checker" pattern documented in `effect_choose_gate.rs`).
//! Walking `serde_json::to_value(&def)` reaches every field of the whole `CardDefinition` by
//! construction, so a new nesting site is covered the moment it exists.

use mtg_engine::all_cards;

/// True if `key` appears anywhere in the value tree as an object key (matches
/// `effect_choose_gate.rs`'s `contains_key` helper -- `Effect` is externally tagged, so a
/// variant name is an object key).
///
/// PB-DP10 rewire: delegates to the canonical walk in `decision_site_walk.rs` (plan §2.3).
/// Behavior-neutral -- `Scry`, `Surveil`, `RevealAndRoute`, `LookAtTopThenPlace` are all
/// struct variants, so the canonical walk's extra unit-variant matching never fires here.
fn contains_key(v: &serde_json::Value, key: &str) -> bool {
    crate::decision_site_walk::json_contains_variant(v, key)
}

const EFFECTS: [&str; 4] = ["Scry", "Surveil", "RevealAndRoute", "LookAtTopThenPlace"];

/// Emits the full sorted, de-duplicated roster plus per-effect counts and the total,
/// asserting a non-zero floor so the sweep cannot silently go vacuous (a serde rename or a
/// walk that stops finding nesting sites would otherwise let this test pass while reporting
/// nothing -- the same hazard `effect_choose_gate.rs`'s `stub_gates_are_not_vacuous` guards).
#[test]
fn pb_rs1_roster_sweep_reports_affected_cards() {
    let defs = all_cards();

    let mut per_effect_counts: Vec<(&str, usize)> = Vec::new();
    let mut affected: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for effect in EFFECTS {
        let mut count = 0usize;
        for def in &defs {
            let json = serde_json::to_value(def).expect("CardDefinition serializes");
            if contains_key(&json, effect) {
                count += 1;
                affected.insert(def.name.clone());
            }
        }
        per_effect_counts.push((effect, count));
    }

    let sorted: Vec<String> = affected.into_iter().collect();

    eprintln!("PB-RS1 roster sweep (from all_cards(), not grep):");
    for (effect, count) in &per_effect_counts {
        eprintln!("  {effect}: {count}");
    }
    eprintln!(
        "  TOTAL distinct cards (union across all 4 effects): {}",
        sorted.len()
    );
    eprintln!("  Full sorted list:");
    for name in &sorted {
        eprintln!("    {name}");
    }

    // Non-vacuity floor: the plan's grep baseline was 47 distinct files. Enumeration
    // should be >= that ballpark (nesting can only add cards a shallow scan misses, not
    // remove them) -- assert a conservative floor well below 47 so a real corpus change
    // (e.g. authoring wave demotions) does not make this gate flaky, while still catching
    // "the walk silently found nothing."
    //
    // DELIBERATELY a floor, not an exact-count pin (contrast
    // `pb_os1_gain_control_reversion_roster`, which pins an exact 2-card set): that
    // roster covers one narrow, historically-fixed combination (GainControl +
    // UntilEndOfTurn/UntilYourNextTurn duration) unlikely to grow via routine
    // authoring. This roster covers four of the engine's most common library-read
    // primitives (Scry/Surveil/RevealAndRoute/LookAtTopThenPlace) during an ACTIVE
    // card-authoring campaign -- the measured count (41 as of 2026-07-19, see
    // `memory/primitive-wip.md`) is expected to keep climbing as new defs are
    // authored, so an exact pin would need routine unrelated updates and would
    // erode into "just bump the number," defeating its own purpose. Reviewed and
    // recorded (not filed) in `memory/primitives/pb-review-RS1.md` item 12: "the
    // test asserts a floor of >= 30 rather than the measured 41, so a real 41->31
    // regression would pass silently -- acceptable anti-flake tradeoff." Left as a
    // floor, per that call; a large regression (e.g. an authoring wave silently
    // dropping Scry usage on a dozen defs) would still need to cross the >= 30 line
    // to go undetected, which is a coarse but real backstop.
    assert!(
        sorted.len() >= 30,
        "roster sweep reports only {} affected cards -- expected at least 30 (grep baseline \
         was 47 distinct files); this floor exists so a serde rename or a walk that stops \
         finding nesting sites cannot silently pass while reporting near-zero. Full list: \
         {:?}",
        sorted.len(),
        sorted
    );

    // Sanity-check a handful of known members (from the plan's "known members" list) --
    // if any of these is absent, the walk itself is broken, not just under-counting.
    //
    // "Six" (the plan's 10th name) is deliberately EXCLUDED here: its def carries a
    // `// TODO: DSL gap` for exactly this pattern (mill 3, route a land to hand) and is
    // `Completeness::partial(..)`, not wired to `Effect::RevealAndRoute` at all yet. The
    // plan's list named it as a card that SHOULD eventually use this primitive, not one
    // that does today -- asserting its presence would be wrong, not a walk defect.
    for known in [
        "Goblin Ringleader",
        "Coiling Oracle",
        "Sylvan Messenger",
        "Risen Reef",
        "Chaos Warp",
        "Satyr Wayfinder",
        "Birthing Ritual",
        "Growing Rites of Itlimoc",
        "Yuriko, the Tiger's Shadow",
    ] {
        assert!(
            sorted.iter().any(|n| n == known),
            "known member '{known}' is missing from the roster sweep -- the walk is broken, \
             not just under-counting. Full list: {sorted:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PB-DX57 (`OOS-DX28-1`) — [`EFFECTS`] pinned against `pub enum Effect`
// ─────────────────────────────────────────────────────────────────────────────

/// `Effect` variants in this file's vocabulary neighbourhood that PB-RS1's roster
/// deliberately does NOT sweep, with a reason each.
///
/// A named const rather than a sentence, so
/// [`effects_list_is_a_checked_subset_of_pub_enum_effect`] can require the
/// classification of the neighbourhood to be TOTAL. Without it, "the other library
/// effects are out of scope" is unbounded and a fifth in-scope variant joins in
/// silence — which is the whole of `OOS-DX28-1`.
///
/// * `MillCards` — CR 701.13a moves cards from the top of a library to a graveyard
///   without looking at them or re-ordering them. PB-RS1 reconciled the
///   look-at-top-then-*place* read/write pair with `draw_card` (CR 121.1); milling
///   has no such pair.
/// * `PutOnLibrary` — the WRITE end alone. It puts an object onto a library from
///   somewhere else; it never reads the top N, so there is nothing to reconcile.
/// * `SearchLibrary` — CR 701.23 searches the WHOLE library, with its own shuffle
///   and reveal rules; it is a different primitive, not a wider version of this one.
const LIBRARY_ADJACENT_EXCLUSIONS: [&str; 3] = ["MillCards", "PutOnLibrary", "SearchLibrary"];

/// **Census row 13 (`OOS-DX28-1`).** [`EFFECTS`] is a hand-written 4-name subset of a
/// **106-variant** `pub enum Effect`, and nothing compared it to that declaration.
/// Two things went wrong silently:
///
/// * a **rename** (or a `#[serde(rename)]`) on any of the four makes
///   `json_contains_variant` match nothing for it. The `>= 30` floor below is a floor
///   on the UNION of all four, so losing one of the smaller three — `Surveil` (9
///   defs), `LookAtTopThenPlace` (3) — leaves the union above 30 and this sweep
///   reports a clean, short roster. The file's own doc makes exactly this distinction
///   for the WALK and does not make it for the LIST.
/// * a **fifth** look-at-top-shaped `Effect` variant is outside the sweep entirely.
///
/// Two legs, and the second is what makes the first more than a spell-check:
/// `EFFECTS ⊆ declared`, and `EFFECTS ∪ LIBRARY_ADJACENT_EXCLUSIONS` must be exactly
/// the declared variants whose names carry this file's library vocabulary. So a new
/// `Effect::LookAtTopThenExile` is a red row rather than a silent omission, and adding
/// a fifth member becomes a deliberate act.
///
/// **Stated residual.** Leg 2's family is keyed on the variant NAME, which is a
/// convention rather than a declaration: a library-reading variant named with none of
/// these tokens escapes it. Leg 1 does not depend on the convention. This is a bound
/// on the class, not a proof that the class is closed — the honest reading the module
/// doc already applies to the `>= 30` floor.
///
/// **Revert to watch red**: remove `"Surveil"` from [`EFFECTS`] (leg 1 stays green —
/// a subset check cannot see a SHRINKING list — and leg 2 catches it, which is why
/// the neighbourhood classification has to be total rather than a subset).
#[test]
fn effects_list_is_a_checked_subset_of_pub_enum_effect() {
    use crate::pb_dx57_declared_source::{declared_enum_variants, CARD_DEFINITION_RS};
    use std::collections::BTreeSet;

    let declared = declared_enum_variants(CARD_DEFINITION_RS, "Effect");
    assert!(
        declared.len() >= 100,
        "non-vacuity: `pub enum Effect` parsed as only {} variant(s) (measured 106 at \
         HEAD); the declaration parser has broken and both legs below would be trivially \
         satisfiable",
        declared.len()
    );

    let swept: BTreeSet<String> = EFFECTS.iter().map(|s| (*s).to_string()).collect();
    let excluded: BTreeSet<String> = LIBRARY_ADJACENT_EXCLUSIONS
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // ── leg 1: nothing swept is undeclared ───────────────────────────────────
    let unknown: Vec<&String> = swept.difference(&declared).collect();
    assert!(
        unknown.is_empty(),
        "EFFECTS names {unknown:?}, which `pub enum Effect` does not declare. \
         `json_contains_variant` matches on the serialized variant NAME, so that needle \
         matches nothing at all: the sweep silently stops counting that effect's defs \
         while the >= 30 union floor keeps this test green."
    );
    let unknown_ex: Vec<&String> = excluded.difference(&declared).collect();
    assert!(
        unknown_ex.is_empty(),
        "LIBRARY_ADJACENT_EXCLUSIONS names {unknown_ex:?}, which `pub enum Effect` does \
         not declare -- the exclusion's reason has rotted"
    );

    // ── leg 2: the library-vocabulary neighbourhood is TOTALLY classified ────
    const VOCABULARY: [&str; 8] = [
        "Top", "Library", "Reveal", "Scry", "Surveil", "Mill", "Look", "Route",
    ];
    let neighbourhood: BTreeSet<String> = declared
        .iter()
        .filter(|n| VOCABULARY.iter().any(|v| n.contains(v)))
        .cloned()
        .collect();

    eprintln!(
        "PB-DX57 row 13: {} declared Effect variants; library-vocabulary neighbourhood \
         {neighbourhood:?}; swept {swept:?}; excluded {excluded:?}",
        declared.len()
    );

    assert!(
        neighbourhood.len() >= 7,
        "non-vacuity: the library-vocabulary filter matched only {neighbourhood:?} \
         (measured 7 at HEAD); the naming convention leg 2 rests on has changed"
    );
    assert!(
        swept.is_disjoint(&excluded),
        "{:?} is both swept and excluded",
        swept.intersection(&excluded).collect::<Vec<_>>()
    );
    let classified: BTreeSet<String> = swept.union(&excluded).cloned().collect();
    assert_eq!(
        classified,
        neighbourhood,
        "the library-vocabulary `Effect` variants are no longer totally classified. An \
         UNCLASSIFIED one is outside PB-RS1's roster sweep entirely, and a name that has \
         LEFT `EFFECTS` shrinks the sweep without moving the >= 30 union floor. Add it to \
         EFFECTS if the read/write reconciliation applies to it, or to \
         LIBRARY_ADJACENT_EXCLUSIONS with the reason it does not. \
         in-neighbourhood-but-unclassified = {:?}, classified-but-not-in-neighbourhood = {:?}",
        neighbourhood.difference(&classified).collect::<Vec<_>>(),
        classified.difference(&neighbourhood).collect::<Vec<_>>()
    );
}
