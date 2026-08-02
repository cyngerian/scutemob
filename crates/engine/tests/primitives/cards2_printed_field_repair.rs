//! CARDS-2 (scutemob-181) — the printed-field repair batch, behaviour half.
//!
//! `core::cards2_printed_field_fidelity` is the *gate*: it proves every def's four printed
//! fields match the card. This file is the *proof the repairs mean something* — it drives the
//! repaired definitions through the real engine, because a def can carry the right numbers and
//! still do nothing with them.
//!
//! The batch's headline card is Boon Satyr, playtest finding F1: a human cast it off one green
//! source and noticed. Four defects in one file — cost `{2}{G}` for a printed `{1}{G}{G}`,
//! bestow `{4}{G}{G}` for `{3}{G}{G}`, no Enchantment card type, and the printed
//! "Enchanted creature gets +4/+2" clause **never authored at all**, on a def declaring
//! `Completeness::Complete`. The first three are numbers the gate now pins. The fourth is a
//! behaviour, and only T5 can see it: before the repair the def had no `Static` abilities, so
//! a bestowed Boon Satyr attached correctly and granted nothing.
//!
//! T1-T4 read the real def out of `all_cards()` and assert its shape. T5/T6 are the
//! discriminating pair: T5 fails against the pre-repair corpus (the bear stays 2/2), T6 pins
//! the other side of `EffectFilter::AttachedCreature` — an *unbestowed* Boon Satyr is a plain
//! creature attached to nothing (CR 702.103f) and must confer nothing, which is what makes the
//! two statics safe on a card that spends most of its life not being an Aura.
//!
//! T7/T8 cover the two mana-cost classes the gate found that are not merely typos: the
//! dropped-`{X}` class (four defs whose `x_count` was 0 for a printed `{X}`, so the spell was
//! castable but X was structurally unavailable) and Tyrranax Rex, which was payable for four
//! mana on a seven-drop.

use mtg_card_types::cards::card_definition::ContinuousEffectDef;
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, enrich_spec_from_def, process_command,
    AbilityDefinition, CardDefinition, CardRegistry, CardType, Command, Effect, EffectDuration,
    EffectFilter, EffectLayer, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    LayerModification, ManaColor, ObjectId, ObjectSpec, PlayerId, Step, SubType, Target, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn def_named(name: &str) -> CardDefinition {
    load_defs()
        .remove(name)
        .unwrap_or_else(|| panic!("no card definition named {name:?}"))
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Power/toughness of a battlefield object after the layer system has run (never read
/// `obj.characteristics.power` directly for a battlefield permanent — W3-LC).
fn pt_on_battlefield(state: &GameState, name: &str) -> (Option<i32>, Option<i32>) {
    let id = find_object(state, name);
    let ch = calculate_characteristics(state, id)
        .unwrap_or_else(|| panic!("'{name}' has no computed characteristics"));
    (ch.power, ch.toughness)
}

// ── T1-T4: the repaired Boon Satyr definition ─────────────────────────────────

#[test]
/// T1 — CR 202.1: printed mana cost `{1}{G}{G}`. The pre-repair def said `{2}{G}`, which is
/// castable off a single green source; that is the exact symptom the playtester reported.
fn t1_boon_satyr_mana_cost_is_one_gg() {
    let def = def_named("Boon Satyr");
    let cost = def.mana_cost.expect("Boon Satyr has a printed mana cost");
    assert_eq!(cost.generic, 1, "printed {{1}}{{G}}{{G}}: one generic");
    assert_eq!(cost.green, 2, "printed {{1}}{{G}}{{G}}: two green pips");
    assert_eq!(cost.mana_value(), 3, "CR 202.3: mana value 3");
    // The defect was a transposition, so mana value alone would NOT have caught it: {2}{G} is
    // also 3. Only the pip split distinguishes them, which is why the gate compares structure.
}

#[test]
/// T2 — CR 702.103a: bestow cost `{3}{G}{G}`. The pre-repair def charged `{4}{G}{G}`; the
/// engine's own bestow test (`mechanics_a_d/bestow.rs`) has always hardcoded the right number
/// against a mock, so nothing connected the two.
fn t2_boon_satyr_bestow_cost_is_three_gg() {
    let def = def_named("Boon Satyr");
    let bestow = def
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Bestow { cost } => Some(cost.clone()),
            _ => None,
        })
        .expect("Boon Satyr declares AbilityDefinition::Bestow");
    assert_eq!(bestow.generic, 3);
    assert_eq!(bestow.green, 2);
    assert_eq!(bestow.mana_value(), 5);
}

#[test]
/// T3 — CR 205.2a: "Enchantment Creature — Satyr". The missing Enchantment type is not
/// cosmetic: it is what a bestowed permanent falls back to being (CR 702.103b) and what
/// enchantment-matters cards see.
fn t3_boon_satyr_is_an_enchantment_creature() {
    let def = def_named("Boon Satyr");
    assert!(def.types.card_types.contains(&CardType::Enchantment));
    assert!(def.types.card_types.contains(&CardType::Creature));
    assert_eq!(def.types.card_types.len(), 2);
    assert!(def.types.subtypes.contains(&SubType("Satyr".to_string())));
    assert!(def
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Flash))));
}

#[test]
/// T4 — CR 613.4c: the "+4/+2" clause exists as two layer-7c statics filtered to the attached
/// creature, the same shape Rancor uses. Pinned by shape, not by count, so that a future edit
/// which keeps two `Static` abilities but points them at the wrong filter still fails.
fn t4_boon_satyr_grants_plus_four_plus_two_to_the_attached_creature() {
    let def = def_named("Boon Satyr");
    let statics: Vec<&ContinuousEffectDef> = def
        .abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Static { continuous_effect } => Some(continuous_effect),
            _ => None,
        })
        .collect();
    assert_eq!(
        statics.len(),
        2,
        "expected exactly the +4 power and +2 toughness statics, found {statics:?}"
    );
    for ce in &statics {
        assert_eq!(ce.layer, EffectLayer::PtModify, "CR 613.4c: layer 7c");
        assert_eq!(
            ce.filter,
            EffectFilter::AttachedCreature,
            "the bonus goes to the enchanted creature, nothing else"
        );
        assert_eq!(ce.duration, EffectDuration::WhileSourceOnBattlefield);
        assert!(ce.condition.is_none());
    }
    let mods: Vec<&LayerModification> = statics.iter().map(|ce| &ce.modification).collect();
    assert!(
        mods.contains(&&LayerModification::ModifyPower(4)),
        "printed +4 power, found {mods:?}"
    );
    assert!(
        mods.contains(&&LayerModification::ModifyToughness(2)),
        "printed +2 toughness, found {mods:?}"
    );
}

// ── T5/T6: the discriminating behaviour pair ──────────────────────────────────

/// Two players, a real Boon Satyr in p1's hand and a vanilla 2/2 in play.
fn boon_satyr_scenario() -> (GameState, PlayerId, PlayerId) {
    let (p1, p2) = (p(1), p(2));
    let defs = load_defs();
    let registry = CardRegistry::new(all_cards());

    let satyr = enrich_spec_from_def(
        ObjectSpec::card(p1, "Boon Satyr")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Boon Satyr")),
        &defs,
    );
    let bear = ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(satyr)
        .object(bear)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

#[test]
/// T5 — THE discriminating test. Bestow the real Boon Satyr onto a 2/2 and the 2/2 becomes
/// 6/4. Against the pre-repair corpus this fails with the bear still 2/2: the def had no
/// `Static` abilities at all, so bestow attached a permanent that granted nothing, and every
/// other test in the suite was happy because the attachment itself worked.
fn t5_bestowed_boon_satyr_makes_a_two_two_into_a_six_four() {
    let (mut state, p1, p2) = boon_satyr_scenario();

    assert_eq!(
        pt_on_battlefield(&state, "Vanilla Bear"),
        (Some(2), Some(2)),
        "baseline before anything is attached"
    );

    // Bestow cost {3}{G}{G}.
    {
        let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Green, 2);
        pool.add(ManaColor::Colorless, 3);
    }

    let satyr_id = find_object(&state, "Boon Satyr");
    let bear_id = find_object(&state, "Vanilla Bear");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: satyr_id,
            targets: vec![Target::Object(bear_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Bestow),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .expect("CR 702.103a: casting Boon Satyr for its bestow cost");

    let (state, _) = pass_all(state, &[p1, p2]);

    let satyr_bf = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Boon Satyr" && o.zone == ZoneId::Battlefield)
        .expect("CR 702.103b: bestowed Boon Satyr resolves onto the battlefield");
    assert_eq!(
        satyr_bf.attached_to,
        Some(bear_id),
        "CR 303.4: the bestowed Aura is attached to its target"
    );

    assert_eq!(
        pt_on_battlefield(&state, "Vanilla Bear"),
        (Some(6), Some(4)),
        "CR 613.4c: 2/2 plus the printed +4/+2 — this is the clause the def never had"
    );
}

#[test]
/// T6 — the other side of `EffectFilter::AttachedCreature`. Put Boon Satyr onto the
/// battlefield as an ordinary creature (never bestowed, so `attached_to` is None, CR 702.103f)
/// and it must confer nothing on anything, including itself. Without this, T5 would also pass
/// for a def whose statics had a filter matching every creature.
fn t6_unbestowed_boon_satyr_confers_nothing() {
    let (p1, p2) = (p(1), p(2));
    let defs = load_defs();
    let satyr = enrich_spec_from_def(
        ObjectSpec::card(p1, "Boon Satyr")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Boon Satyr")),
        &defs,
    );
    let bear = ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(satyr)
        .object(bear)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    assert_eq!(
        pt_on_battlefield(&state, "Vanilla Bear"),
        (Some(2), Some(2)),
        "an unattached Boon Satyr pumps nothing"
    );
    assert_eq!(
        pt_on_battlefield(&state, "Boon Satyr"),
        (Some(4), Some(2)),
        "and does not pump itself — its printed body is 4/2"
    );
}

// ── T7/T8: the two mana-cost classes that are not simple typos ────────────────

#[test]
/// T7 — the dropped-`{X}` class. Four defs printed with `{X}` carried `x_count: 0`, so the
/// spell was castable at a fixed cost and X had no structural existence — Torment of Hailfire
/// for `{B}{B}` drained for zero. `x_count` is the whole fix; nothing else about these cards
/// changed.
fn t7_dropped_x_class_declares_its_x_symbol() {
    for name in [
        "Chord of Calling",
        "Green Sun's Zenith",
        "Torment of Hailfire",
        "Wake the Dead",
    ] {
        let cost = def_named(name)
            .mana_cost
            .unwrap_or_else(|| panic!("{name} has a printed mana cost"));
        assert_eq!(
            cost.x_count, 1,
            "CR 107.3: {name} is printed with one {{X}} symbol"
        );
    }
}

#[test]
/// T8 — Tyrranax Rex, the worst single cost error in the corpus: `{G}{G}{G}{G}` for a printed
/// `{4}{G}{G}{G}`. Three mana cheap, and deck-legal. Mana value is the right assertion here
/// (unlike T1) precisely because the error was not a transposition.
fn t8_tyrranax_rex_costs_seven() {
    let cost = def_named("Tyrranax Rex").mana_cost.expect("printed cost");
    assert_eq!(cost.generic, 4);
    assert_eq!(cost.green, 3);
    assert_eq!(cost.mana_value(), 7, "CR 202.3: a seven-drop");
}

// ── T9/T10: the two ABILITY repairs, which no gate can see ────────────────────

#[test]
/// T9 — Zulaport Cutthroat gains exactly ONE life per death, in a four-player game.
///
/// The review-cycle finding this pins is the sharpest correctness bug the batch found. The def
/// used `Effect::DrainLife { amount: Fixed(1) }`, and `effects/mod.rs` gives the controller the
/// **total** life lost across all opponents — so in the four-player Commander format this
/// engine targets, every creature death gained **3** life for a card that prints 1. It shipped
/// `Complete` and deck-legal.
///
/// SR-37 cannot catch this: R1–R8 check printed *fields*, and this is an ability. Nothing else
/// covered the card either — `grep -rl zulaport crates/engine/tests test-data/` was empty. So
/// the shape is pinned here, structurally, rather than left to the next re-read.
fn t9_zulaport_cutthroat_gains_one_life_not_one_per_opponent() {
    let def = def_named("Zulaport Cutthroat");
    let effect = def
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Triggered { effect, .. } => Some(effect),
            _ => None,
        })
        .expect("Zulaport Cutthroat has a death trigger");

    let debug = format!("{effect:?}");
    assert!(
        !debug.contains("DrainLife"),
        "CR 118.4 / the printed text: 'each opponent loses 1 life AND you gain 1 life' is two          effects, not a drain. `Effect::DrainLife` credits the controller with the TOTAL lost,          which is 3 at a four-player table. Effect was: {debug}"
    );
    assert!(
        debug.contains("EachOpponent"),
        "the life loss is per-opponent: {debug}"
    );
    // Exactly one GainLife, and it must be OUTSIDE the per-opponent loop — a GainLife nested
    // in the ForEach would reintroduce the 3-life bug with different spelling.
    assert_eq!(
        debug.matches("GainLife").count(),
        1,
        "expected exactly one GainLife: {debug}"
    );
    let Effect::Sequence(steps) = effect else {
        panic!("expected a Sequence of [per-opponent loss, single gain], got {debug}");
    };
    assert!(
        steps.iter().any(|s| matches!(s, Effect::GainLife { .. })),
        "the GainLife must be a sibling of the ForEach, not nested inside it: {debug}"
    );
}

#[test]
/// T10 — Tyrranax Rex has the four abilities it prints and not the one it does not.
///
/// The def declared `KeywordAbility::Ravenous`, which appears on **no printing of this card**,
/// and omitted haste, Toxic 4 and "this spell can't be countered" — while being the gate's own
/// motivating example. Repairing its mana cost fixed one defect of five.
///
/// Pinned both ways round. The positive half would pass on a def that also kept Ravenous; the
/// negative half is what encodes the actual finding, and it is why the golden script that
/// certified the invented keyword had to be retired.
fn t10_tyrranax_rex_has_its_printed_abilities_and_not_ravenous() {
    let def = def_named("Tyrranax Rex");
    let keywords: Vec<&KeywordAbility> = def
        .abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Keyword(k) => Some(k),
            _ => None,
        })
        .collect();
    for want in [
        KeywordAbility::Trample,
        KeywordAbility::Ward(4),
        KeywordAbility::Haste,
        KeywordAbility::Toxic(4),
    ] {
        assert!(
            keywords.contains(&&want),
            "printed keyword {want:?} missing; def has {keywords:?}"
        );
    }
    assert!(
        !keywords.contains(&&KeywordAbility::Ravenous),
        "CR 702.156 Ravenous is on NO printing of Tyrranax Rex — it was invented, along with an          oracle_text describing it and a golden script certifying it"
    );
    assert!(
        def.cant_be_countered,
        "printed 'This spell can't be countered' (CR 701.6a) lives on the definition, not in          `abilities`"
    );
}
