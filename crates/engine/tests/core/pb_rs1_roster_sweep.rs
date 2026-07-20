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
fn contains_key(v: &serde_json::Value, key: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == key || contains_key(child, key)),
        serde_json::Value::Array(items) => items.iter().any(|i| contains_key(i, key)),
        _ => false,
    }
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
