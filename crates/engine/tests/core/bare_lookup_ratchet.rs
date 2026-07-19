//! SR-25 anti-regression ratchet for the SR-4 / SR-14 / SR-25 silent-failure discipline.
//!
//! Background: `state::diagnostics` gives every state lookup in the resolution path a way to
//! say *which* kind of absence it tolerates — an engine bug (`expect_object` / `expect_player`
//! / `expect_zone`, which `debug_assert!`) or a rules-correct CR 608.2b fizzle (`fizzle_object`
//! / `fizzle_move_object_to_zone`, a quiet `None`). SR-4 swept `effects/mod.rs` and
//! `rules/resolution.rs`; SR-14 swept ten more `rules/` files; SR-25 swept `rules/layers.rs`,
//! `rules/commander.rs`, `rules/miracle.rs`, the four small foretell/plot/priority/suspend/
//! turn_structure sites, and the non-primitive swallow-sites in `state/mod.rs`.
//!
//! CLAUDE.md says "new code in these files must pick a side", but until this gate that was
//! pure convention: nothing stopped a fresh `state.objects.get(&id)` from reappearing and
//! silently swallowing a lookup, regressing the ~760 classified sites invisibly. This test
//! pins a per-file ceiling on bare `.objects.get` / `.players.get` / `.zones.get` (`_mut`
//! included) lookups. A file's count may only ever go *down*: adding a bare lookup exceeds
//! the ceiling and fails with a pointer at the diagnostics vocabulary; converting one leaves
//! the count below the ceiling and fails asking you to tighten it. Either way the number can
//! never silently rise.
//!
//! The scan strips `//` line comments (so a comment that quotes `.objects.get(` doesn't
//! inflate the count) and then removes *all* whitespace before counting, so the number is
//! insensitive to rustfmt line-wrapping — a multi-line `state\n  .objects\n  .get(&id)` chain
//! counts exactly like the one-line form, and no one can slip a new lookup past the gate by
//! splitting it across lines. This is the same source-scan technique SR-5's keyword registry,
//! SR-8's protocol fingerprint, and SR-23's `lki_diagnostics_scan` all use; the counter's own
//! needle strings live here, in the test, never in the scanned files, so the scan cannot match
//! its own source.
//!
//! Known limitation (shared with the SR-5/SR-8 source-scan gates): only `//` line comments are
//! stripped, not block comments. A contrived `state.objects/**/.get(&id)` would slip past the
//! needle (the non-whitespace `/**/` breaks the `.objects.get(` substring). That is not a
//! realistic regression path — it takes deliberately grotesque code that clippy and review would
//! reject — so it is documented rather than defended against.
//!
//! The remaining (non-zero) ceilings are the classified-and-left-alone residue: NONSWALLOW
//! predicate reads (`state.objects.get(&id).map(|o| ..).unwrap_or(false)`, where a departed
//! object legitimately answers the predicate `false`), disjoint-borrow sites guarded by
//! `debug_assert_object_live!`, and — in `state/mod.rs` — the primitive accessors
//! (`object`, `player`, `zone`, `add_object`, `move_object_to_zone`) that the whole `expect_*`
//! / `fizzle_*` vocabulary is *built on top of*. Those are the foundation; they stay bare by
//! construction.

use std::fs;
use std::path::PathBuf;

/// Per-file ceilings on bare `.objects/.players/.zones.get[_mut](` lookups, comment-stripped
/// and whitespace-insensitive (see module docs). **A count may only decrease.** To lower a
/// ceiling after converting a lookup, run the `emits_the_live_counts` helper below (it prints
/// the current numbers) and paste them in.
///
/// Order: SR-4's two files, SR-14's ten, then SR-25's nine.
const SWEPT_FILES: &[(&str, usize)] = &[
    // SR-4
    // PB-EF2 (2026-07-18): 100 → 105. Five new NONSWALLOW predicate reads (same shape as
    // the file's existing residue: `state.objects.get(&id).map(|o| ..)` / `state.players
    // .get(p).map(|ps| !ps.has_lost).unwrap_or(false)`), added by PlayerTarget::
    // ControllerOfTriggeringObject's three resolution sites (Manifest, Cloak,
    // resolve_player_target_list) and ControllerOfCounteredSpell's has-lost filter in
    // resolve_player_target_list — a departed triggering object/player legitimately
    // falls back to `ctx.controller` or an empty recipient list, not an engine bug.
    // PB-EF3 (2026-07-18): 105 → 107. Two new NONSWALLOW predicate reads for
    // `EffectTarget::AttackTarget` / `PlayerTarget::DefendingPlayer`: (1)
    // `state.objects.get(pw_id)` checking whether an attacked planeswalker is still on
    // the battlefield (CR 506.4c: if removed, the attacker attacks nothing, so the
    // effect correctly resolves to empty rather than an engine bug); (2) `state.players
    // .get(&dp)` in the DefendingPlayer arm, an exact copy of the pre-existing
    // DamagedPlayer arm's has-lost filter a few lines above it.
    // PB-EF3 fix (scutemob-103, review Finding 5): confirmed accurate post-fix. The
    // `AttackTarget` arm no longer falls back to `ctx.defending_player` when this
    // lookup finds the planeswalker gone -- it fizzles immediately, matching what
    // this comment already claimed. The fallback is now reserved for the case where
    // the attacker itself has left the live `combat.attackers` map entirely.
    // PB-OS6 (2026-07-19): 107 → 109. Two new NONSWALLOW predicate reads, both exact
    // copies of pre-existing sibling `Condition` arms' shape in this same match:
    // (1) `Condition::TopCardIsInstantOrSorcery`'s `state.zones.get(&lib_zone)
    // .and_then(|z| z.top())` -- identical idiom to `TopCardIsCreatureOfChosenType`
    // a few lines above it (an empty library legitimately answers the peek `false`,
    // not an engine bug); (2) `Condition::YouAttackedWithNOrMore(n)`'s `state.players
    // .get(&ctx.controller).map(..).unwrap_or(false)` -- identical idiom to
    // `YouAttackedThisTurn` / `ControllerLifeAtLeast` / half the other `Condition`
    // arms in this file (a missing controller answers the predicate `false`).
    // PB-OS8 (2026-07-19): 109 → 110. One new NONSWALLOW predicate read:
    // `Effect::LookAtTopThenPlace`'s `state.zones.get(&lib_zone).map(|z| z.object_ids())
    // .unwrap_or_default()` -- an exact copy of the pre-existing `Effect::RevealAndRoute`
    // idiom a few lines above it (an empty/missing library legitimately yields an empty
    // top-N window, which falls through to `continue`, not an engine bug).
    ("src/effects/mod.rs", 110),
    // PB-OS4b (2026-07-19): 102 → 101. `apply_face_change` replaced several raw
    // `state.objects.get_mut(&id)` transform-flip sites with a single call, and one
    // `debug_assert_object_live!` + bare-lookup pair collapsed into a plain
    // `state.objects.get(&id).map(..)` NONSWALLOW read (turn_actions-style) at the
    // TransformTrigger/DayboundTransformTrigger/craft-return boundary sites — net one
    // fewer bare lookup in this file.
    ("src/rules/resolution.rs", 101),
    // SR-14
    // PB-EF3 (2026-07-18): 72 → 74. Two new NONSWALLOW predicate reads, both matching the
    // file's existing residue shape exactly: (1) `state.objects.get(pw_id).map(|obj| obj
    // .controller)` in the new `AnyCreatureYouControlAttacks` defending-player capture (B1),
    // an exact duplicate of the pre-existing `SelfAttacks` capture a few lines above it — a
    // departed attacked planeswalker legitimately falls back to `None` (CR 506.4c), not an
    // engine bug; (2) `state.objects.get(&trigger.source)` in the new `has_ability_targets`
    // presence check, an exact duplicate of the pre-existing lookup inside the
    // Normal/CardDefETB target-selection branch a few lines below it.
    ("src/rules/abilities.rs", 74),
    ("src/rules/casting.rs", 34),
    ("src/rules/combat.rs", 16),
    ("src/rules/sba.rs", 7),
    ("src/rules/replacement.rs", 24),
    ("src/rules/turn_actions.rs", 7),
    ("src/rules/mana.rs", 7),
    ("src/rules/copy.rs", 4),
    // PB-EF5 (2026-07-18): 24 → 22. `transform_permanent_in_place` (extracted from
    // handle_transform's tail) uses `fizzle_object`/`fizzle_object_mut` (CR 400.7 --
    // the source may have left its zone) instead of bare `.objects.get[_mut]`, and
    // collapses the old duplicate `.objects.get` re-read (used only to re-check
    // `is_transformed`) into a single upfront snapshot.
    ("src/rules/engine.rs", 22),
    ("src/rules/lands.rs", 3),
    // SR-25
    // PB-EF9 (2026-07-18): 51 → 54. Three new NONSWALLOW-shaped reads in
    // `expire_while_you_control_source_effects` / `recompute_object_controller`: the
    // source-existence check (`state.objects.get(&src).map(|o| ..).unwrap_or(true)` --
    // CR 400.7, a departed source legitimately means "ended"), the owner lookup
    // (`match state.objects.get(&object_id) { Some(o) => o.owner, None => return }` --
    // a departed borrowed object has nothing to revert), and the final
    // `state.objects.get_mut(&object_id)` controller write (same fizzle: nothing to
    // write to if the object is gone). All three are one-shot expiry-pass reads with
    // no downstream engine invariant depending on the object being live.
    ("src/rules/layers.rs", 54),
    ("src/rules/commander.rs", 6),
    ("src/rules/miracle.rs", 2),
    ("src/rules/foretell.rs", 0),
    ("src/rules/plot.rs", 0),
    ("src/rules/priority.rs", 0),
    ("src/rules/suspend.rs", 0),
    ("src/rules/turn_structure.rs", 0),
    ("src/state/mod.rs", 18),
];

/// Denominator guard: the roster must not be silently gutted down to a few green files.
const MIN_FILES: usize = 21;

/// Denominator guard: the aggregate scan must keep *finding* the bulk of the residue. If the
/// counter were broken to return 0 (or the paths all went stale), the total would collapse and
/// this floor would catch it. Set well below the current live total (477).
const MIN_TOTAL: usize = 400;

/// The six needles that constitute a "bare lookup". `.get(` and `.get_mut(` are disjoint (the
/// latter has `_` where the former has `(`), so summing them never double-counts.
const NEEDLES: &[&str] = &[
    ".objects.get(",
    ".objects.get_mut(",
    ".players.get(",
    ".players.get_mut(",
    ".zones.get(",
    ".zones.get_mut(",
];

fn engine_src(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Strip `//`-to-end-of-line comments and all whitespace, then count the needles.
///
/// Whitespace removal makes the count rustfmt-stable (a line-wrapped method chain counts the
/// same as the inline form) and, more importantly, un-evadable by line-splitting. Comment
/// stripping keeps a doc comment that mentions `.objects.get(` from inflating the ceiling.
fn bare_lookup_count(src: &str) -> usize {
    let decommented: String = src
        .lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let collapsed: String = decommented.chars().filter(|c| !c.is_whitespace()).collect();
    NEEDLES.iter().map(|n| collapsed.matches(n).count()).sum()
}

/// The ratchet: every swept file's bare-lookup count must equal its pinned ceiling.
#[test]
fn bare_lookup_counts_are_pinned() {
    assert!(
        SWEPT_FILES.len() >= MIN_FILES,
        "SR-25 denominator guard: the swept-file roster shrank to {} (< {MIN_FILES}). \
         Files may be added to the ratchet, never removed — a dropped file stops governing \
         its bare lookups.",
        SWEPT_FILES.len()
    );

    let mut total = 0usize;
    for &(rel, ceiling) in SWEPT_FILES {
        let src = engine_src(rel);
        // Prove the file was actually read, so a mis-pathed 0 can't pass as "fully swept".
        assert!(
            src.len() > 200,
            "SR-25: {rel} is suspiciously small ({} bytes) — wrong path? A misread file would \
             report 0 lookups and pass a 0 ceiling vacuously.",
            src.len()
        );
        let count = bare_lookup_count(&src);
        total += count;

        if count > ceiling {
            panic!(
                "SR-25 ratchet: {rel} now has {count} bare `.objects/.players/.zones.get[_mut](` \
                 lookups, up from the pinned {ceiling}. A new bare lookup swallows its absence \
                 silently (SR-4/SR-14). Pick a side in crates/engine/src/state/diagnostics.rs: \
                 `expect_object` / `expect_player` / `expect_zone[_mut]` when a `None` is an \
                 engine bug (debug_assert), or `fizzle_object[_mut]` / `fizzle_move_object_to_zone` \
                 when a `None` is a rules-correct CR 608.2b fizzle. If this really is a new \
                 primitive/NONSWALLOW site, classify it in the SR-14 audit doc and raise the \
                 ceiling deliberately."
            );
        }
        if count < ceiling {
            panic!(
                "SR-25 ratchet: {rel} is down to {count} bare lookups from the pinned {ceiling} \
                 — good, you converted some. Lower its ceiling in SWEPT_FILES to {count} so the \
                 ratchet keeps the gain (a stale-high ceiling would let a future regression hide \
                 under the slack)."
            );
        }
    }

    assert!(
        total >= MIN_TOTAL,
        "SR-25 denominator guard: the whole scan found only {total} bare lookups (< {MIN_TOTAL}). \
         The counter or the file paths are probably broken — a real scan of these 21 files finds \
         hundreds. A silently-empty scan would pass every per-file check vacuously."
    );
}

/// Non-vacuity of the counter itself: it must actually see lookups, ignore comments, and be
/// blind to whitespace. If any of these regress, the ratchet above is measuring nothing.
#[test]
fn counter_is_non_vacuous() {
    // One of each needle, inline.
    assert_eq!(
        bare_lookup_count("state.objects.get(&a); s.players.get_mut(&b); z.zones.get(&c);"),
        3,
        "counter missed inline lookups"
    );
    // get vs get_mut are counted, and distinct.
    assert_eq!(
        bare_lookup_count("x.objects.get(&a); x.objects.get_mut(&b);"),
        2
    );
    // A comment quoting the pattern must NOT count.
    assert_eq!(
        bare_lookup_count("// prefer state.objects.get over raw access\nlet y = 1;"),
        0,
        "comment stripping failed — a quoted pattern inflated the count"
    );
    // Whitespace/line-splitting must NOT hide a lookup.
    assert_eq!(
        bare_lookup_count("state\n    .objects\n    .get(&a)\n    .map(|o| o.tapped)"),
        1,
        "whitespace insensitivity failed — a split chain evaded the counter"
    );
    // `stack_objects` / `lki_objects` (leading `_`, not `.`) must NOT match.
    assert_eq!(
        bare_lookup_count("self.stack_objects.get(&a); self.lki_objects.get(&b);"),
        0,
        "false match on a `_objects` field"
    );
}

/// Guard that the vocabulary this ratchet steers authors toward actually exists, so the
/// failure message never points at a getter that was renamed out from under it (the SR-23
/// hazard, one layer up).
#[test]
fn diagnostics_vocabulary_still_exists() {
    let diag = engine_src("src/state/diagnostics.rs");
    for anchor in [
        "fn expect_object(",
        "fn expect_player(",
        "fn expect_zone(",
        "fn fizzle_object(",
        "fn fizzle_move_object_to_zone(",
    ] {
        assert!(
            diag.contains(anchor),
            "SR-25: diagnostics.rs no longer defines `{anchor}` — the ratchet's failure message \
             steers authors at a vocabulary that moved. Update both together."
        );
    }
}
