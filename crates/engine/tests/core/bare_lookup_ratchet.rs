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
    // PB-RS1 fix cycle (2026-07-19): 110 → 111. One new NONSWALLOW predicate read:
    // `RestrictSearchTopN`'s `state.zones.get(&lib_id).map(|z| z.top_n(top_n as usize))
    // .unwrap_or_default()`, replacing the old `state.objects.iter().filter(..)`
    // ObjectId-proxy scan (Finding 1 of pb-review-RS1.md). Exact same idiom as the
    // PB-OS8 entry directly above: a missing/empty library legitimately yields an
    // empty top-N restriction set, which falls through to "nothing found," not an
    // engine bug.
    // PB-DP5 (2026-07-26): 111 → 110. `draw_one_card`'s three bare lookups were
    // consolidated into `replacement::perform_one_draw` (which already used
    // `expect_*` throughout); `draw_cards_for_player` now just calls it in a
    // loop, netting one fewer bare lookup site in this file.
    // PB-DX25c (2026-08-06): 110 → 108. `Effect::ChangeTargets`'s ~130-line
    // open-coded candidate scan (two `.get(...)` bare-lookup sites feeding the
    // player/object candidate builds) is deleted; the whole decision now
    // delegates to `rules::retarget::plan_target_change` in a separate file.
    // The ratchet only ever moves DOWN -- this lowers the ceiling to keep the
    // gain rather than leaving slack a future regression could hide in.
    // PB-DX53 (2026-09, `OOS-DX21-1`): 108 -> 109. One new NONSWALLOW predicate
    // read: `Condition::YouAttackedWithNOrMoreCreaturesThisTurn(n)`'s
    // `state.players.get(&ctx.controller).map(..).unwrap_or(false)` -- an exact
    // copy of the pre-existing sibling `YouAttackedWithNOrMoreThisDeclaration`
    // arm's idiom immediately above it (a missing controller legitimately
    // answers the predicate `false`, not an engine bug).
    ("src/effects/mod.rs", 109),
    // PB-DX25c fix cycle (2026-08-06, review Finding E8): the two lookups
    // above did not disappear, they RELOCATED into this new file -- and
    // without this entry the ratchet's denominator would have silently
    // shrunk by two, exactly the class this gate exists to prevent (a
    // conversion vs. a relocation look identical from the effects/mod.rs
    // ceiling alone). Measured ceiling is 0, not because the risk vanished
    // but because `retarget.rs`'s reads are spelled through the `objects()`
    // accessor method (`state.objects().get(id)`) rather than the bare
    // `.objects.get(` field-access idiom this ratchet's NEEDLES match -- the
    // exact same silent-`None` shape, invisible to this specific scan only
    // because of the parenthesized method call. Swept here so a FUTURE bare
    // `.objects.get(`/`.players.get(`/`.zones.get(` added to this file (e.g.
    // by a reviewer "simplifying" the accessor call) is caught rather than
    // silently absorbed into an unswept file.
    ("src/rules/retarget.rs", 0),
    // PB-OS4b (2026-07-19): 102 → 101. `apply_face_change` replaced several raw
    // `state.objects.get_mut(&id)` transform-flip sites with a single call, and one
    // `debug_assert_object_live!` + bare-lookup pair collapsed into a plain
    // `state.objects.get(&id).map(..)` NONSWALLOW read (turn_actions-style) at the
    // TransformTrigger/DayboundTransformTrigger/craft-return boundary sites — net one
    // fewer bare lookup in this file.
    // PB-DP6 fix cycle (2026-07-26): 101 → 100. The `is_carddef_etb` resolution-time
    // intervening-if re-check's `condition_holds` closure and the effect-execution
    // context just below it each had their own `state.objects.get(&source_object)`
    // for `kicker_times_paid`/`x_value`; hoisted into one shared
    // `state.objects.get(&source_object).map(|o| (o.kicker_times_paid, o.x_value))`
    // read above both, fixing the closure's `EffectContext::new` zero-fill bug
    // (review finding 1) at zero net new lookups.
    ("src/rules/resolution.rs", 100),
    // SR-14
    // PB-EF3 (2026-07-18): 72 → 74. Two new NONSWALLOW predicate reads, both matching the
    // file's existing residue shape exactly: (1) `state.objects.get(pw_id).map(|obj| obj
    // .controller)` in the new `AnyCreatureYouControlAttacks` defending-player capture (B1),
    // an exact duplicate of the pre-existing `SelfAttacks` capture a few lines above it — a
    // departed attacked planeswalker legitimately falls back to `None` (CR 506.4c), not an
    // engine bug; (2) `state.objects.get(&trigger.source)` in the new `has_ability_targets`
    // presence check, an exact duplicate of the pre-existing lookup inside the
    // Normal/CardDefETB target-selection branch a few lines below it.
    // PB-OS11 (2026-07-19): 74 → 75. One new NONSWALLOW predicate read:
    // `collect_triggers_for_event`'s new `ControllerAttacks` batch-filter branch reads
    // `state.objects.get(aid)` for each declared attacker in `state.combat.attackers`
    // inside an `.any(|aid| ...)` closure — a departed attacker (removed from the
    // battlefield between attack declaration and trigger collection) legitimately
    // answers the predicate `false` (does not match), the exact same shape as every
    // other predicate-read site already ceilinged in this file.
    // PB-DX35 (2026-09, `OOS-DX4-2`): 75 -> 72. `trigger_modal_plan` consolidates
    // three hand-rolled copies of the trigger-target-requirement lookup (sites 1/2,
    // plus the modes lookup) into one shared function, removing bare
    // `state.objects.get`/`state.card_registry.get` duplicates that `flush_sorted`
    // used to repeat at each site. Lowered rather than left stale-high (a
    // stale-high ceiling is slack a regression hides in).
    // PB-DX36 (2026-09, `OOS-CARDS2-6`): 72 -> 71. `queue_damage_source_triggers`
    // replaced the single bare `state.objects.get(&assignment.source)`
    // creature_on_bf check the old inlined Equipment/Aura walk used with
    // `state.fizzle_object(source)` (CR 113.7a: the damage source may have left
    // the battlefield between the damage event and this collector running — a
    // rules-correct fizzle, not an engine bug), and its own new
    // `state.objects.get(&attachment_id)` opponent-scoping read is likewise
    // `state.fizzle_object(attachment_id)`. Net -1, not left stale-high.
    ("src/rules/abilities.rs", 71),
    ("src/rules/casting.rs", 33),
    // PB-DP4 (2026-07-26): 16 -> 15. `has_uncosted_attack_target` (new, CR 508.1d)
    // deduplicates the two copy-pasted `has_cant_attack_owner` bare
    // `state.restrictions.iter().any(|r| ... state.objects.get(&r.source) ...)` lookups
    // (the goad block and the MustAttackEachCombat block) into a single site.
    // PB-DX6 (2026-08-02): 15 -> 16. `accumulate_attack_tax_total` (new, CR 508.1h --
    // shared by `handle_declare_attackers`'s own validation AND
    // `queries::attack_tax_total`, plan §5.3's anti-drift requirement) needs its OWN
    // `state.restrictions.iter().any(|r| ... state.objects.get(&r.source) ...
    // matches!(o.zone, ZoneId::Battlefield) ...)` source-on-battlefield check to
    // accumulate the payable total, independently of `handle_declare_attackers`'s own
    // local scan for `x_tax_defenders`/`taxed_defenders` (which the plan deliberately
    // keeps local, not shared -- "the X/taxed_defenders bookkeeping stays in
    // combat.rs; only the total is shared"). This is the file's own pre-existing
    // NONSWALLOW predicate-read idiom (a departed source legitimately answers
    // "not on the battlefield", not an engine bug) duplicated into a third site
    // rather than a new pattern -- the two functions cannot share it without
    // widening `accumulate_attack_tax_total`'s signature to also report X/taxed-
    // defender bookkeeping, which `queries::attack_tax_total` (a pure total, no new
    // public type) has no use for.
    // PB-DX55 (2026-09-05): 16 → 14. Extracting `check_block_pair` /
    // `validate_block_declaration` (`OOS-SIM5-3`) collapsed the per-pair blocker loop
    // and the provoke requirement's independent `continue`-shaped mirror of that same
    // loop into ONE function -- the mirror's own bare `state.objects.get(&provoked_id)`
    // (controller/zone match) and `state.objects.get(&pw_id)` (cross-player planeswalker
    // check inside the per-pair loop, now shared) collapsed from two call sites into
    // one apiece. Two bare lookups converted, so the ceiling comes down with them
    // rather than leaving slack a future regression can hide in.
    ("src/rules/combat.rs", 14),
    // PB-DX49 (2026-09-03): 7 → 6. `check_saga_sbas`'s chapter-still-on-stack guard used
    // to re-fetch the Saga with a bare `state.objects.get(&saga_id)` to read
    // `is_transformed` and `card_id`; it now asks `rules::saga::saga_view`, which resolves
    // the object through `fizzle_object` internally. One bare lookup converted, so the
    // ceiling comes down with it rather than leaving slack a future regression can hide in.
    ("src/rules/sba.rs", 6),
    ("src/rules/replacement.rs", 24),
    ("src/rules/turn_actions.rs", 7),
    // PB-OS11 (2026-07-19): 7 → 8. One new NONSWALLOW predicate read:
    // `handle_tap_for_mana`'s remove-counter legality pre-check (step 5b2) reads
    // `state.objects.get(&source).and_then(|o| o.counters.get(counter).copied())
    // .unwrap_or(0)` — `source` was already validated present a few lines above
    // with no intervening zone change, so this cannot actually miss; the
    // `unwrap_or(0)` is defensive, matching the shape of every other predicate
    // read in this file.
    ("src/rules/mana.rs", 8),
    ("src/rules/copy.rs", 4),
    // PB-EF5 (2026-07-18): 24 → 22. `transform_permanent_in_place` (extracted from
    // handle_transform's tail) uses `fizzle_object`/`fizzle_object_mut` (CR 400.7 --
    // the source may have left its zone) instead of bare `.objects.get[_mut]`, and
    // collapses the old duplicate `.objects.get` re-read (used only to re-check
    // `is_transformed`) into a single upfront snapshot.
    // PB-DP4 (2026-07-26): 22 → 24 during implement, then 24 → 22 in the fix cycle
    // (review finding E2). The implement phase added two bare `.players.get(` sites
    // where a non-bare equivalent was already in use elsewhere in this same PB: (1)
    // the CR 119.4 life-cost gate in `CumulativeUpkeepCost::Life`'s pay arm now reads
    // `state.player(player)?.life_total` -- `player()` is a primitive accessor (not
    // matched by the `.players.get(` needle), the same idiom `resolution.rs` uses
    // throughout; (2) the `force_resolve_overdue_payments` boundary-sweep hook in
    // `handle_all_passed` now reads `state.expect_player(active).map(|p| !p.has_lost
    // && !p.has_conceded).unwrap_or(false)` -- `expect_player` is the NONSWALLOW
    // predicate-read idiom this PB's own `combat.rs:` `has_uncosted_attack_target`
    // already uses for an identical "departed player answers false" read. Neither
    // conversion changes behavior (both are the same `state.players` lookup under a
    // different name); the ceiling is restored to 22 to lock in the reduction, per
    // the ratchet's own rule that a gate exists to stop a raise it can avoid.
    // PB-DX6 stage B (2026-08-02): 22 -> 21. `handle_turn_face_up`'s payment block was
    // rewritten to mirror `abilities.rs::handle_activate_ability`'s hybrid/Phyrexian
    // flatten-then-pay shape (CR 107.4e/107.4f), which replaced the single bare
    // `state.players.get_mut(&player)` borrow held across the whole payment with three
    // separate `state.player(player)?` / `state.player_mut(player)?` primitive-accessor
    // calls (life-check, mana gate, phyrexian-life deduction) -- exactly the idiom this
    // file's own craft/cumulative-upkeep sites already use a few hundred lines away. Net
    // one fewer bare lookup, not a new NONSWALLOW site.
    ("src/rules/engine.rs", 21),
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
    // PB-DX39 lowered this 54 -> 36 rather than leaving the slack (PB-DX49's rule: a
    // stale-high ceiling is where a regression hides). The 18 that went were the
    // per-arm `state.objects.get(&source_id)` reads in `effect_applies_to`'s twenty
    // source-relative filter arms, now served by the two `source_view_*` constructors --
    // which is 20 reads replaced by 2, hence exactly 18.
    ("src/rules/layers.rs", 36),
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
