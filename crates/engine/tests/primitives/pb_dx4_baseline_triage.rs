//! PB-DX4 (OOS-DP10-8): the oracle-text triage of PB-DP10's 97-entry decision
//! `BASELINE`.
//!
//! PB-DP10 froze 97 `Complete` defs that still carry an engine-made choice into
//! `crates/engine/tests/core/decision_gate.rs`'s `BASELINE`, but populated it
//! **mechanically** — the plan §5.3 class-B (the def is faithful; the engine merely
//! auto-picks among legal options) vs class-D (the def is simply wrong) triage was never
//! performed. A closing-review spot-check of 5 of the 97 found 2 class-D members and the
//! seed itself flagged that "2-of-5 is a very noisy sample".
//!
//! It was. All 97 were read against MCP printed text this batch and the split is
//! **86 B / 11 D**, so the spot-check overstated the D rate roughly fivefold. Per-def
//! findings with oracle citations: `memory/primitives/pb-dx4-baseline-triage.md`.
//!
//! This file pins the eleven. Five defs were demoted and five repaired in place; the
//! eleventh (Staff of Compleation) was deliberately left `Complete` and allowlisted in
//! `completeness_deviation_scan.rs`, matching the shipped `nether_traitor` decision for
//! the identical owner-vs-controller class.
//!
//! ## Pre-fix observation, and which tests have one
//!
//! The standard PB-DX3's review MEDIUM established: a "pre-fix, X happened" sentence must
//! be **run, not reasoned to**, because a true statement arrived at by inference is
//! indistinguishable in a document from one arrived at by observation, and only the second
//! survives the next refactor. So this section says exactly which claims were executed and
//! declines to manufacture the rest.
//!
//! **`shambling_ghast_minus_one_minus_one_wears_off_at_end_of_turn` — OBSERVED.** Mode 1
//! was reverted in the shipped def to
//! `Effect::AddCounter { counter: CounterType::MinusOneMinusOne, count: 1 }`, this test
//! re-run with its two assertions instrumented as prints so execution could reach both
//! reads, and the numbers read off the run: the 3/3 victim was **2/2 after the effect and
//! still 2/2 after the turn boundary** — the counter survived cleanup, which is the whole
//! defect. The def was then restored and the same drive gives 2/2 during the turn and
//! **3/3 after cleanup**. Both halves executed, in both directions.
//!
//! **Every other test here is a claim about the authored def, and has no pre-fix drive by
//! design.** `put_at_most_one_reveals_use_the_put_one_primitive` is the one worth naming:
//! it asserts the choice of PRIMITIVE rather than an outcome, because a drive cannot
//! separate `LookAtTopThenPlace` from `RevealAndRoute` on a fixture with one matching card
//! on top — both put exactly that card in hand — and a drive with several matching cards
//! would be asserting arity, which is what reading the primitive already does directly.
//! Claiming a pre-fix number for it would have meant running a fixture chosen to make the
//! claim true, which is the failure mode this section exists to avoid. The data pins
//! (`metastatic_evangel_...`, `radstorm_...`, `sword_of_truth_and_justice_...`,
//! `shambling_ghast_does_not_have_the_decayed_keyword`) compare fields against MCP printed
//! values; their "pre-fix value" is simply the field's old contents, recorded in each
//! def's own diff and in the triage doc.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, CardDefinition,
    Command, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, SpellTarget, Step,
    SubType, Target, TargetRequirement, ZoneId,
};
use std::collections::HashMap;

// ── Shared helpers ───────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Load a card def from the real corpus by exact name. Never re-declared inline — the
/// probes must exercise the shipped def.
fn card_def(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("{name} should be in the corpus"))
}

fn defs_map(defs: &[&CardDefinition]) -> HashMap<String, CardDefinition> {
    defs.iter()
        .map(|d| (d.name.clone(), (*d).clone()))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found in state"))
}

fn on_battlefield(owner: PlayerId, def: &CardDefinition, all: &[&CardDefinition]) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs_map(all),
    )
}

fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (stuck at {:?}, wanted {:?})",
            state.turn().step,
            target
        );
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder at step {:?}", state.turn().step));
        let (next, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {holder:?} failed: {e:?}"));
        state = next;
    }
}

// ── T1: Shambling Ghast's -1/-1 lasts until end of turn, not forever ─────────

/// Run Shambling Ghast's mode-1 effect directly against `target`, bypassing the stack.
///
/// The subject of this test is the effect's DURATION, not the death trigger or the modal
/// choice that reach it, so the effect is executed straight from the shipped def with the
/// victim as declared target. Everything upstream (CR 603.3d trigger targeting, the modal
/// announcement) is PB-DP8's and PB-DP3's territory and already has its own coverage.
fn run_ghast_mode_1(
    state: &mut GameState,
    controller: PlayerId,
    source: ObjectId,
    target: ObjectId,
) {
    let ghast = card_def("Shambling Ghast");
    let modes = ghast
        .abilities
        .iter()
        .find_map(|a| match a {
            mtg_engine::AbilityDefinition::Triggered { modes: Some(m), .. } => Some(m.clone()),
            _ => None,
        })
        .expect("Shambling Ghast has a modal triggered ability");
    // Taken from the SHIPPED def, not re-declared: if the authored effect regresses to a
    // counter, this test executes the regression and the cleanup assertion catches it.
    execute_effect(
        state,
        &modes.modes[1],
        &mut EffectContext::new(
            controller,
            source,
            vec![SpellTarget {
                target: Target::Object(target),
                zone_at_cast: Some(ZoneId::Battlefield),
            }],
        ),
    );
}

#[test]
/// **CR 613.1e / MCP printed text.** Shambling Ghast's mode 1 is "Target creature an
/// opponent controls gets **-1/-1 until end of turn**". It was authored as a permanent
/// `CounterType::MinusOneMinusOne` counter, which is a different game object entirely: a
/// counter persists past cleanup, is proliferate-able, and annihilates against +1/+1
/// counters under CR 122.3.
///
/// The discriminating assertion is the SECOND one. The first — that the victim is
/// weakened at all — passed before the fix too, which is exactly why it is not sufficient
/// on its own and why this test drives through a cleanup step rather than stopping at
/// "the -1/-1 applied".
fn shambling_ghast_minus_one_minus_one_wears_off_at_end_of_turn() {
    let ghast = card_def("Shambling Ghast");
    let all = [&ghast];

    // The victim is a plain 3/3 vanilla body rather than a corpus card: it must SURVIVE
    // -1/-1 or the wear-off half is unobservable, and it must carry no ability of its own
    // that could muddy the P/T reading.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(on_battlefield(p(1), &ghast, &all))
        .object(ObjectSpec::creature(p(2), "Test Bear", 3, 3))
        .build()
        .expect("state builds");

    let source = find_object(&state, "Shambling Ghast");
    let victim_id = find_object(&state, "Test Bear");
    let base = calculate_characteristics(&state, victim_id).expect("victim has characteristics");
    let (base_p, base_t) = (base.power, base.toughness);
    assert!(
        base_t.unwrap_or(0) > 1,
        "fixture: the victim must survive -1/-1 or the wear-off half is unobservable; \
         got toughness {base_t:?}"
    );

    run_ghast_mode_1(&mut state, p(1), source, victim_id);

    let weakened = calculate_characteristics(&state, victim_id).expect("still on the battlefield");
    assert_eq!(
        (weakened.power, weakened.toughness),
        (base_p.map(|v| v - 1), base_t.map(|v| v - 1)),
        "CR 613.1e: mode 1 gives the target -1/-1"
    );

    // The subject: advance into the NEXT turn, which necessarily passes through this
    // turn's cleanup step. An until-end-of-turn effect ends there (CR 514.2); a -1/-1
    // COUNTER would not.
    //
    // Targeting the next Upkeep rather than Cleanup itself is deliberate: no player
    // receives priority during a normal cleanup step (CR 514.3), so a priority-passing
    // walk never rests there and asking it to stop at Cleanup panics.
    let state = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state.turn().turn_number,
        2,
        "the walk must actually have crossed a turn boundary, or 'past cleanup' is a \
         claim about nothing"
    );
    let restored = calculate_characteristics(&state, victim_id).expect("still on the battlefield");
    assert_eq!(
        (restored.power, restored.toughness),
        (base_p, base_t),
        "CR 514.2 / 613.1e: 'until end of turn' ends during the cleanup step, so the \
         victim must be back to its printed P/T. If this fails with the victim still \
         weakened, mode 1 has regressed to a permanent -1/-1 COUNTER -- which survives \
         cleanup, is proliferate-able, and annihilates against +1/+1 counters (CR 122.3), \
         none of which the printed card does."
    );
}

// ── T2: Shambling Ghast no longer has a keyword the card does not have ───────

#[test]
/// **MCP printed keywords are `["Treasure"]` only.** The def granted
/// `KeywordAbility::Decayed` (CR 702.147a: can't block, and sacrifice itself at end of
/// combat after attacking), which the printed card does not have at all — a phantom
/// ability that made every Shambling Ghast in every deck unable to block and suicidal.
///
/// Pinned as a corpus-wide zero rather than a per-def check: after this batch, `Decayed`
/// is granted by exactly one def (`jadar_ghoulcaller_of_nephalia`, which grants it to the
/// *token* it creates and does not carry it itself), so any def carrying it directly is a
/// regression worth naming.
fn shambling_ghast_does_not_have_the_decayed_keyword() {
    let ghast = card_def("Shambling Ghast");
    let has_decayed = ghast.abilities.iter().any(|a| {
        matches!(
            a,
            mtg_engine::AbilityDefinition::Keyword(mtg_engine::KeywordAbility::Decayed)
        )
    });
    assert!(
        !has_decayed,
        "MCP printed keywords for Shambling Ghast are [\"Treasure\"] only; Decayed \
         (CR 702.147a) is not on the card"
    );
    assert!(
        !ghast.oracle_text.to_lowercase().contains("decayed"),
        "the stored oracle_text must not name Decayed either -- it was the reason the \
         phantom keyword survived review, and golden script baseline/112 cited this very \
         field as its authority for testing an ability the card does not have"
    );
    assert!(
        !ghast.oracle_text.contains("enters"),
        "the stored oracle_text said 'When Shambling Ghast enters' against the def's own \
         TriggerCondition::WhenDies; printed text is 'When this creature dies'"
    );
}

// ── T3: Metastatic Evangel's printed data ────────────────────────────────────

#[test]
/// **MCP: `{1}{W}`, `Creature — Phyrexian Human Cleric`, `3/1`, "Whenever another
/// **nontoken** creature you control enters, proliferate."** The def had four deviations
/// at once: `{2}{W}`, a missing `Human` subtype, a transposed `1/3`, and no nontoken
/// filter.
///
/// The nontoken half is the interesting one: the def carried a note claiming `is_token`
/// "is only checked in combat_damage_filter paths; for ETB trigger matching it is
/// silently ignored". That note was **stale** — PB-AC0 forwards the whole `TargetFilter`
/// as `triggering_creature_filter` through the creature-ETB lowering and
/// `rules/abilities.rs` honours `is_nontoken` explicitly on that path. Same class as
/// PB-DX3/PB-DX3b's stale blocker notes, found here by a different route.
fn metastatic_evangel_matches_its_printed_card() {
    let def = card_def("Metastatic Evangel");

    let cost = def.mana_cost.as_ref().expect("has a mana cost");
    assert_eq!(
        (cost.generic, cost.white),
        (1, 1),
        "MCP printed cost is {{1}}{{W}}"
    );

    for st in ["Phyrexian", "Human", "Cleric"] {
        assert!(
            def.types.subtypes.contains(&SubType(st.to_string())),
            "MCP type line is 'Creature — Phyrexian Human Cleric'; {st} is missing"
        );
    }

    assert_eq!(
        (def.power, def.toughness),
        (Some(3), Some(1)),
        "MCP printed P/T is 3/1 (the def had it transposed to 1/3)"
    );

    let trigger = def
        .abilities
        .iter()
        .find_map(|a| match a {
            mtg_engine::AbilityDefinition::Triggered {
                trigger_condition, ..
            } => Some(format!("{trigger_condition:?}")),
            _ => None,
        })
        .expect("has the ETB trigger");
    assert!(
        trigger.contains("is_nontoken: true"),
        "printed text is 'another NONTOKEN creature you control enters'; the filter must \
         carry is_nontoken (honoured on the creature-ETB path via \
         triggering_creature_filter, PB-AC0). Got: {trigger}"
    );
}

// ── T4: the two put-≤1 reveals put at most one card in hand ──────────────────

#[test]
/// **CR 701.20a / MCP printed text.** Grisly Salvage — "You may put **a** creature or
/// land card from among them into your hand" — and Satyr Wayfinder — "You may put **a**
/// land card from among them into your hand" — were both authored with
/// `Effect::RevealAndRoute`, which routes **every** matching card (see the
/// `Effect::RevealAndRoute` arm in `effects/mod.rs`, which partitions into matched /
/// unmatched and moves all of the matched). So a two-mana instant put 3-5 creature or
/// land cards into hand, mandatorily.
///
/// `Effect::LookAtTopThenPlace` is the shipped put-≤1 sibling — its own DSL doc calls it
/// exactly that — and carries the `optional` flag the printed "you may" needs.
///
/// Asserted on the authored shape rather than by driving the reveal, because the defect
/// is a choice of primitive and the two primitives differ in arity, not in outcome for
/// any single input: a drive with one matching card on top passes under BOTH, which is
/// precisely the vacuous-probe trap this campaign keeps finding.
fn put_at_most_one_reveals_use_the_put_one_primitive() {
    for (name, printed) in [
        (
            "Grisly Salvage",
            "You may put a creature or land card from among them into your hand",
        ),
        (
            "Satyr Wayfinder",
            "You may put a land card from among them into your hand",
        ),
    ] {
        let def = card_def(name);
        let rendered = format!("{:?}", def.abilities);
        assert!(
            rendered.contains("LookAtTopThenPlace"),
            "{name}: printed text is \"{printed}\" -- 'a card', singular, and optional. \
             That is Effect::LookAtTopThenPlace (put <= 1), not Effect::RevealAndRoute \
             (routes ALL matches)."
        );
        assert!(
            !rendered.contains("RevealAndRoute"),
            "{name}: Effect::RevealAndRoute routes every matching card and has no \
             optional flag; it cannot express \"{printed}\""
        );
        assert!(
            rendered.contains("optional: true"),
            "{name}: the printed clause is 'you MAY put', so `optional` must be true"
        );
    }
}

// ── T5: Sword of Truth and Justice targets a creature YOU control ────────────

#[test]
/// **MCP: "put a +1/+1 counter on **a creature you control**, then proliferate."** The
/// trigger declared a bare `TargetRequirement::TargetCreature`, so the counter could
/// legally be placed on an opponent's creature — a strictly worse outcome the controller
/// could be forced into by the engine's auto-target, and an illegal one by the printed
/// card either way.
fn sword_of_truth_and_justice_targets_only_your_creature() {
    let def = card_def("Sword of Truth and Justice");
    let targets = def
        .abilities
        .iter()
        .find_map(|a| match a {
            mtg_engine::AbilityDefinition::Triggered { targets, .. } if !targets.is_empty() => {
                Some(targets.clone())
            }
            _ => None,
        })
        .expect("the combat-damage trigger declares a target");

    assert_eq!(targets.len(), 1, "one target: 'a creature you control'");
    match &targets[0] {
        TargetRequirement::TargetCreatureWithFilter(filter) => {
            assert!(
                matches!(filter.controller, mtg_engine::TargetController::You),
                "printed text is 'a creature YOU CONTROL'; got controller {:?}",
                filter.controller
            );
            assert!(
                !filter.exclude_self,
                "printed text says 'a creature you control', not 'another' -- \
                 exclude_self would wrongly bar the equipped creature itself"
            );
        }
        other => panic!(
            "expected TargetCreatureWithFilter(controller: You); a bare TargetCreature \
             lets the counter land on an opponent's creature. Got {other:?}"
        ),
    }
}

// ── T6: Radstorm's printed mana cost ─────────────────────────────────────────

#[test]
/// **MCP: `{3}{U}`.** The def had `{2}{U}` — a Storm card castable one mana cheap, and
/// Storm compounds the error: every extra mana available a turn earlier is another spell
/// cast before it, hence another copy (CR 702.40a).
fn radstorm_costs_three_generic_and_one_blue() {
    let cost = card_def("Radstorm")
        .mana_cost
        .expect("Radstorm has a mana cost");
    assert_eq!(
        (cost.generic, cost.blue),
        (3, 1),
        "MCP printed cost is {{3}}{{U}}"
    );
}

// ── T7: the five demotions are real and machine-checked ──────────────────────

#[test]
/// The five class-D defs whose defect is **not authorable today** are demoted, so
/// `validate_deck` refuses them (SR-2 / Architecture Invariant 9) rather than shipping a
/// legal-but-wrong card.
///
/// **What this test asserts and what it does not.** It asserts the marker, and nothing
/// else — deliberately. A marker is the entire remedy available to a card-def-only batch
/// for a defect that needs an engine change, so the marker is the whole deliverable and
/// pinning it is pinning the work. It is NOT a claim that the underlying defect is fixed;
/// each def's `completeness` note names the missing DSL surface and, where one exists,
/// the seed that owns closing it.
fn class_d_defs_without_a_dsl_expression_are_demoted() {
    for (name, why) in [
        (
            "Smuggler's Copter",
            "printed 'you MAY draw a card. If you do, discard a card' authored as an \
             unconditional Sequence(DrawCards, DiscardCards) on both triggers (audit \
             §5 DP-12; the other 19 instances of this class are already known_wrong)",
        ),
        (
            "Contaminant Grafter",
            "printed 'then you MAY put a land card from your hand onto the battlefield' \
             authored unconditionally, forcing the land-put every qualifying end step",
        ),
        (
            "Grateful Apparition",
            "printed 'deals combat damage to a player OR PLANESWALKER'; \
             WhenDealsCombatDamageToPlayer never fires on planeswalker damage",
        ),
        (
            "Thrasios, Triton Hero",
            "printed 'Otherwise, DRAW A CARD' authored as a Hand zone-move, which fires \
             no draw event and bypasses draw triggers, replacements and restrictions",
        ),
        (
            "Shambling Ghast",
            "its mode-1 target is declared flat rather than per-mode, so choosing the \
             Treasure mode still requires a legal opponent creature (CR 603.3d)",
        ),
    ] {
        let def = card_def(name);
        assert!(
            def.completeness != mtg_engine::cards::Completeness::Complete,
            "{name} must not be Complete -- {why}"
        );
    }

    // Non-vacuity, and the other half of the batch: the five defs REPAIRED in place must
    // still be Complete. Without this, demoting all eleven would pass the loop above and
    // look like success while destroying five cards' worth of coverage.
    for name in [
        "Metastatic Evangel",
        "Grisly Salvage",
        "Satyr Wayfinder",
        "Sword of Truth and Justice",
        "Radstorm",
    ] {
        let def = card_def(name);
        assert_eq!(
            def.completeness,
            mtg_engine::cards::Completeness::Complete,
            "{name}'s defect WAS authorable and was fixed in place; it must stay Complete"
        );
    }
}

// ── T8: the corpus-wide `#[default]` question PB-DX3b left open ──────────────

#[test]
/// PB-DX3b closed with an explicit hand-off: `#[default] Completeness::Complete` had by
/// then produced two live-wrong deck-legal defs (`aurelia_the_warleader`,
/// `emeria_the_sky_ruin`) by different routes, and "which defs never declare a marker at
/// all?" was named a cheap corpus-wide question nobody had asked.
///
/// This batch asked it. The answer is **not a handful**: a clear majority of the corpus's
/// `Complete` defs are `Complete` only because nobody wrote the field. Every one of this
/// batch's eleven class-D defs was in that group.
///
/// Pinned as a ratchet in the direction that matters. The number is large and will move
/// with ordinary authoring, so this is a ceiling, not an equality: it fails if the
/// unmarked population GROWS, which is the direction that adds silent-defect surface.
/// Lower it when a batch reduces the count.
fn defs_that_never_declare_a_completeness_marker_are_ratcheted() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .join("crates/card-defs/src/defs");

    let mut total = 0usize;
    let mut undeclared = 0usize;
    for entry in std::fs::read_dir(&root).expect("defs dir is readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }
        total += 1;
        let source = std::fs::read_to_string(&path).expect("def source is readable");
        if !source.contains("completeness") {
            undeclared += 1;
        }
    }

    assert!(
        total >= 1_700,
        "denominator guard: only {total} def files walked, so this gate would be \
         measuring almost nothing"
    );

    // Measured on this branch at PB-DX4 close: 970 of 1,804 def files never mention
    // `completeness` at all. (This batch's own five demotions each ADDED a marker, so the
    // count fell by five from the 975 that would otherwise stand.)
    const MAX_UNDECLARED: usize = 970;
    assert!(
        undeclared <= MAX_UNDECLARED,
        "{undeclared} of {total} card-def files never mention `completeness`, up from the \
         pinned {MAX_UNDECLARED}. Every one of those defs is `Complete` by the \
         `#[default] Completeness::Complete` derive rather than by anyone's decision -- \
         the mechanism that shipped `aurelia_the_warleader` (PB-DX1) and \
         `emeria_the_sky_ruin` (PB-DX3b) as live-wrong deck-legal cards, and that all \
         eleven of PB-DX4's class-D defs sat in. A new def is not required to be \
         non-Complete; it IS required to say which it is."
    );
}
