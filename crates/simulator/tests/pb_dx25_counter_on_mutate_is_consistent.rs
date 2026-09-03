//! PB-DX25 — the direct closure proof for `OOS-SIM3-5`'s headline claim, and the
//! Stage-2 fail-before asymmetry record (plan §12 risk 1).
//!
//! At HEAD, `Effect::CounterSpell` targeting a `MutatingCreatureSpell`'s card is a
//! silent no-op (shape (c), plan §2.1): `position()`'s second clause only matches
//! `StackObjectKind::Spell`, so nothing is found, nothing is removed, and the
//! mutate spell simply resolves and merges anyway. **This produces NO
//! `stack_consistency` divergence** — the card and its stack entry both survive,
//! consistently — so the instrument-level "zero violations" assertion below is
//! GREEN both before and after this batch's fix. That is not evidence the defect
//! is absent; it is evidence `stack_consistency` was never the right lens for shape
//! (c) (shape (a), the stranding, is what would redden it — see
//! `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs`'s T2).
//!
//! What DOES discriminate at HEAD is the BEHAVIOURAL fact: `test_dx25_...` (this
//! file) asserts that a Gemrazer "countered" by Counterspell does NOT merge with
//! its mutate target. At HEAD that assertion fails, honestly, because the mutate
//! resolves despite the counter. After PB-DX25's fix (Stage 4), it flips to
//! asserting the CORRECT end state — no merge, Gemrazer in the graveyard, zero
//! violations throughout. Both the "zero violations" and the "no merge" checks are
//! kept permanently: the first is a real regression guard against shape (a)
//! stranding (a partial fix that repairs the lookup but not the zone-move would
//! redden it), the second is the fix's own closure proof.

use std::collections::HashMap;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AdditionalCost,
    CardDefinition, CardId, CardRegistry, CardType, Command, GameState, GameStateBuilder,
    ManaColor, ObjectId, ObjectSpec, PlayerId, StackObjectKind, Step, SubType, Target, ZoneId,
};
use mtg_simulator::invariants;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn all_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn enrich(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

fn wolf_spec(owner: PlayerId) -> ObjectSpec {
    let mut wolf = ObjectSpec::card(owner, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);
    wolf
}

/// CR 701.6a / CR 702.140a — a real two-player game (`GameStateBuilder` +
/// `process_command`, the plan's own first option) in which p2 counters p1's
/// Gemrazer mutate cast. `invariants::check_all` is run after every command;
/// zero `stack_consistency` violations are expected throughout AND at the
/// terminal state (this half is non-discriminating for shape (c) at HEAD -- see
/// the module doc). The behavioural half (no merge) IS discriminating and is the
/// fail-before record for this file.
#[test]
fn test_dx25_counter_on_mutate_produces_no_stack_consistency_violations() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs_by_name();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich(p1, "Gemrazer", ZoneId::Hand(p1), &defs))
        .object(wolf_spec(p1))
        .object(enrich(p2, "Counterspell", ZoneId::Hand(p2), &defs))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    {
        let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Green, 3);
        pool.add(ManaColor::Colorless, 2);
    }
    {
        let pool = &mut state.players_mut().get_mut(&p2).unwrap().mana_pool;
        pool.add(ManaColor::Blue, 2);
    }
    state.turn_mut().priority_holder = Some(p1);

    let mut violations_seen = Vec::new();
    let check = |state: &GameState, violations_seen: &mut Vec<String>| {
        let v = invariants::check_all(state, None);
        for viol in v.into_iter().filter(|v| v.check == "stack_consistency") {
            violations_seen.push(format!("turn {}: {}", viol.turn_number, viol.description));
        }
    };

    let gemrazer_hand_id = find_object(&state, "Gemrazer");
    let wolf_id = find_object(&state, "Mock Wolf");
    let counterspell_hand_id = find_object(&state, "Counterspell");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: gemrazer_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate { target: wolf_id }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell (mutate) failed: {:?}", e));
    check(&state, &mut violations_seen);

    let gemrazer_stack_card_id = match &state.stack_objects()[0].kind {
        StackObjectKind::MutatingCreatureSpell { source_object, .. } => *source_object,
        other => panic!("expected MutatingCreatureSpell, got {:?}", other),
    };

    let (state, _) = process_command(state, Command::PassPriority { player: p1 })
        .unwrap_or_else(|e| panic!("PassPriority failed: {:?}", e));
    check(&state, &mut violations_seen);

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p2,
            card: counterspell_hand_id,
            targets: vec![Target::Object(gemrazer_stack_card_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell (Counterspell) failed: {:?}", e));
    check(&state, &mut violations_seen);

    let mut state = state;
    while !state.stack_objects().is_empty() {
        let holder = state
            .turn()
            .priority_holder
            .expect("priority holder must be set while the stack is non-empty");
        let (s, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        check(&state, &mut violations_seen);
    }

    // Terminal-state check, independent of the per-command walk above.
    let terminal_violations: Vec<_> = invariants::check_all(&state, None)
        .into_iter()
        .filter(|v| v.check == "stack_consistency")
        .collect();

    assert!(
        violations_seen.is_empty(),
        "CR 701.6a / OOS-SIM3-5: zero stack_consistency violations expected across \
         the whole game (this half is non-discriminating for shape (c) at HEAD -- \
         see this file's module doc); got {:?}",
        violations_seen
    );
    assert!(
        terminal_violations.is_empty(),
        "CR 701.6a / OOS-SIM3-5: zero stack_consistency violations expected at the \
         terminal state; got {:?}",
        terminal_violations
    );

    // The BEHAVIOURAL half -- discriminating at HEAD (fails before the fix; see
    // this file's module doc for the recorded asymmetry).
    let wolf_obj = state
        .objects()
        .get(&wolf_id)
        .expect("Wolf should still exist");
    assert!(
        wolf_obj.merged_components.is_empty(),
        "CR 701.6a / CR 729.2: Gemrazer was countered and must NOT have merged with \
         the Wolf -- merged_components should be empty, got {:?}",
        wolf_obj.merged_components
    );
    assert!(
        find_in_zone(&state, "Gemrazer", ZoneId::Graveyard(p1)).is_some(),
        "CR 701.6a: countered Gemrazer should be in p1's graveyard"
    );
}

/// Non-vacuity, mandatory (plan §6 File C): a hand-built state with a card in
/// `ZoneId::Stack` claimed by NO stack object DOES produce a `stack_consistency`
/// violation. Without this half, the "zero violations" assertion above is
/// satisfiable by a `check_all` that always returns nothing.
#[test]
fn test_dx25_an_unclaimed_stack_zone_card_is_a_real_violation() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Orphaned Card")
                .in_zone(ZoneId::Stack)
                .with_card_id(CardId("orphaned-card".to_string())),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let violations: Vec<_> = invariants::check_all(&state, None)
        .into_iter()
        .filter(|v| v.check == "stack_consistency")
        .collect();
    assert!(
        !violations.is_empty(),
        "a card sitting in ZoneId::Stack with no owning StackObject must be a real \
         stack_consistency violation -- the check_all()/GameStateBuilder wiring \
         used above is provably capable of detecting the class this test's sibling \
         asserts zero of"
    );
}
