//! Tests for SR-34: composite-cost mana abilities (CR 605.1a).
//!
//! `enrich_spec_from_def` used to lower only a bare `Cost::Tap` activated ability into a
//! `ManaAbility` — so every mana source with an additional cost (a Signet's `{1}`, a
//! horizon land's `Pay 1 life`) was treated as a stack-using activated ability: it could
//! not be found by `Command::TapForMana` and could not be activated while paying for
//! another spell (CR 605.3a), which is what a Signet is *for*. `ManaAbility` gained a
//! `mana_cost` and `life_cost` component, `handle_tap_for_mana` gained a cost-payment
//! step, and the mana-ability lowering gate widened from `matches!(cost, Cost::Tap)` to
//! any cost payable through `Command::TapForMana` — see `mana_ability_lowering` and
//! `mana_ability_cost_components` in `crates/engine/src/testing/replay_harness.rs`.
//!
//! Pattern follows `tests/core/effect_choose_gate.rs` (SR-33): activate the ability and
//! assert the mana that comes out, not just that a `ManaAbility` exists (SF-5 — "a
//! data-model test can pin a defect as a requirement").

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardRegistry, Command, Cost, Effect, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn make_spec(
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

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn pool_amount(state: &GameState, player: PlayerId, color: ManaColor) -> u32 {
    let pool = &state.player(player).expect("player exists").mana_pool;
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

/// Build a two-player state with one battlefield permanent (by name), priority held by
/// p(1), at pre-combat main.
fn build_with_permanent(name: &str, defs: &HashMap<String, CardDefinition>) -> GameState {
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), name, ZoneId::Battlefield, defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));
    state
}

/// Minimal `Command::CastSpell` — only the fields that matter for a non-targeting,
/// non-modal, non-alt-cost spell are set; the rest default per
/// `tests/rules/grant_flash.rs`'s `cast_spell_cmd` pattern.
fn cast_spell_cmd(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: None,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
        face_down_kind: None,
        additional_costs: vec![],
    }))
}

// ── T1 / T2 / T3 / T4 — Signets: cost, no stack, insufficient mana ────────────────

/// CR 605.1a: a `{1},{T}: Add {C}{C}`-shaped ability (Boros Signet: `{1},{T}: Add
/// {W}{R}`) lowers to a `ManaAbility`, not a stack-using activated ability.
#[test]
fn signet_registers_a_mana_ability_not_an_activated_ability() {
    let defs = defs_map();
    let spec = make_spec(p(1), "Boros Signet", ZoneId::Battlefield, &defs);
    assert_eq!(
        spec.mana_abilities.len(),
        1,
        "Boros Signet's {{1}},{{T}}: Add {{W}}{{R}} must register as a ManaAbility (CR 605.1a)"
    );
    assert!(
        spec.activated_abilities.is_empty(),
        "the same ability must not ALSO appear in activated_abilities (SF-6 exclusion)"
    );
}

/// CR 605.1a / CR 118.3a: activating the Signet pays its `{1}` from the pool (generic,
/// any color) and produces `{W}{R}`.
#[test]
fn signet_tap_for_mana_pays_generic_and_produces_two() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), "Boros Signet", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 1,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let signet = find_by_name(&state, "Boros Signet");
    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: signet,
            ability_index: 0,
        },
    )
    .expect("Boros Signet activation should succeed with {1} available (CR 118.3a)");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        0,
        "the {{1}} generic cost must be paid from the pool"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::White),
        1,
        "Boros Signet adds {{W}}"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Red),
        1,
        "Boros Signet adds {{R}}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaCostPaid { player, cost } if *player == p(1) && cost.generic == 1
        )),
        "ManaCostPaid{{generic:1}} should be emitted (CR 602.2b/601.2f)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::White,
                amount: 1,
                ..
            }
        )),
        "ManaAdded(White, 1) should be emitted"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::Red,
                amount: 1,
                ..
            }
        )),
        "ManaAdded(Red, 1) should be emitted"
    );
}

/// CR 605.3b: a mana ability resolves immediately and never uses the stack, even with a
/// composite cost.
#[test]
fn signet_tap_for_mana_does_not_use_the_stack() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), "Boros Signet", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 1,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let signet = find_by_name(&state, "Boros Signet");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: signet,
            ability_index: 0,
        },
    )
    .expect("activation should succeed");

    assert!(
        state.stack_objects().is_empty(),
        "CR 605.3b: a mana ability must not use the stack"
    );
}

/// CR 118.3 / CR 601.2h: an unaffordable Signet activation is rejected and touches
/// nothing — `process_command` takes `GameState` by value, so a rejected command's
/// mutations (if any had happened) are unobservable; here none should happen at all.
#[test]
fn signet_with_empty_pool_is_insufficient_mana() {
    let defs = defs_map();
    let pre_command_state = build_with_permanent("Boros Signet", &defs);
    let signet = find_by_name(&pre_command_state, "Boros Signet");
    let probe_state = pre_command_state.clone();

    let result = process_command(
        pre_command_state,
        Command::TapForMana {
            player: p(1),
            source: signet,
            ability_index: 0,
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "an empty pool cannot pay Boros Signet's {{1}} (CR 118.3, 601.2h): {result:?}"
    );
    assert!(
        !probe_state.objects()[&signet].status.tapped,
        "the source must still be untapped in the caller's pre-command state"
    );
}

// ── T5 / T6 / T7 — horizon lands: life payment legality (CR 119.4) ────────────────

/// CR 119.4: a player at exactly 1 life CAN pay 1 life (">=", not ">") — they go to 0
/// and die to the SBA separately; that is correct and out of scope here.
#[test]
fn horizon_land_pays_life_and_at_exactly_one_life_is_legal() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), "Fiery Islet", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .player_life(p(1), 1)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let land = find_by_name(&state, "Fiery Islet");
    // Fiery Islet's mana_abilities[0] is its {U} arm (see fiery_islet.rs: {U} then {R},
    // one activated ability per printed colour, tainted_field.rs pattern).
    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land,
            ability_index: 0,
        },
    )
    .expect("paying exactly 1 life at 1 life must be legal (CR 119.4: '>=', not '>')");

    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        0,
        "life goes to exactly 0"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Blue),
        1,
        "Fiery Islet's {{U}} arm adds {{U}}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::LifeLost { player, amount: 1 } if *player == p(1)
        )),
        "LifeLost{{amount:1}} should be emitted (CR 119.4)"
    );
}

/// CR 119.4 / CR 118.3: a player at 0 life cannot pay 1 life.
#[test]
fn horizon_land_at_zero_life_cannot_pay() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), "Fiery Islet", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .player_life(p(1), 0)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let land = find_by_name(&state, "Fiery Islet");
    let probe_state = state.clone();
    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land,
            ability_index: 0,
        },
    );

    assert!(
        matches!(
            result,
            Err(GameStateError::InsufficientLife {
                required: 1,
                actual: 0,
                ..
            })
        ),
        "a player at 0 life cannot pay 1 life: {result:?}"
    );
    assert!(
        !probe_state.objects()[&land].status.tapped,
        "the land must still be untapped"
    );
}

/// CR 119.4b: players can ALWAYS pay 0 life, no matter their life total — the check
/// must short-circuit on `life_cost > 0` rather than reading `>=` unguarded, or a
/// `life_cost: 0` mana ability wrongly rejects at a negative life total (reachable
/// transiently before the CR 704.5a SBA runs). Every mana ability with no life
/// component (e.g. a basic land) takes this branch on every activation.
#[test]
fn zero_life_cost_ability_is_legal_at_negative_life() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), "Forest", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .player_life(p(1), -1)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let forest = find_by_name(&state, "Forest");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: forest,
            ability_index: 0,
        },
    )
    .expect("a life_cost:0 mana ability must activate at any life total, including negative (CR 119.4b)");

    assert_eq!(pool_amount(&state, p(1), ManaColor::Green), 1);
}

// ── T8 / T9 — funds a spell / cost paid from another mana ability, no recursion ───

/// CR 605.3a / 605.3b: a Signet activated between land-tap and cast, in the same
/// priority window, never touches the stack — only the spell it funds does. This is
/// the CR 605.3b payload as this Command model can express it: no priority window for
/// opponents, and usable in the same window as the cast it funds (the model does not
/// have a mid-cost-payment interleave — see the SR-34 plan §1 scoping note).
#[test]
fn mana_ability_funds_a_spell_in_the_same_priority_window() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let mountain = make_spec(p(1), "Mountain", ZoneId::Battlefield, &defs);
    let signet = make_spec(p(1), "Boros Signet", ZoneId::Battlefield, &defs);
    let mind_stone = make_spec(p(1), "Mind Stone", ZoneId::Hand(p(1)), &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(mountain)
        .object(signet)
        .object(mind_stone)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let mountain_id = find_by_name(&state, "Mountain");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: mountain_id,
            ability_index: 0,
        },
    )
    .expect("tap Mountain for {R}");
    assert!(
        state.stack_objects().is_empty(),
        "tapping a land never uses the stack"
    );

    let signet_id = find_by_name(&state, "Boros Signet");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: signet_id,
            ability_index: 0,
        },
    )
    .expect("Boros Signet's {1} is paid from the {R} the Mountain just added (CR 605.3a)");
    assert!(
        state.stack_objects().is_empty(),
        "CR 605.3b: activating the Signet must not have put anything on the stack"
    );
    assert_eq!(pool_amount(&state, p(1), ManaColor::White), 1);
    assert_eq!(pool_amount(&state, p(1), ManaColor::Red), 1);

    let mind_stone_id = find_by_name(&state, "Mind Stone");
    let (state, _) = process_command(state, cast_spell_cmd(p(1), mind_stone_id))
        .expect("Mind Stone's {2} generic is paid from the Signet's {W}{R} (CR 601.2b)");

    assert_eq!(
        state.stack_objects().len(),
        1,
        "the stack must hold exactly the spell — never a mana ability at any point in this sequence"
    );
}

/// CR 605.3a: a Signet's `{1}` may legally come from a land tapped in a PRIOR Command —
/// the mana pool persists between Commands, so no recursion or loop-detection is
/// needed for `handle_tap_for_mana` (it never calls itself).
#[test]
fn signet_mana_cost_can_be_paid_from_another_mana_ability() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let mountain = make_spec(p(1), "Mountain", ZoneId::Battlefield, &defs);
    let signet = make_spec(p(1), "Boros Signet", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(mountain)
        .object(signet)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let mountain_id = find_by_name(&state, "Mountain");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: mountain_id,
            ability_index: 0,
        },
    )
    .expect("tap Mountain for {R} in the first Command");

    let signet_id = find_by_name(&state, "Boros Signet");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: signet_id,
            ability_index: 0,
        },
    )
    .expect("the Signet's {1} is already sitting in the pool from the prior Command");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Red),
        1,
        "Mountain's red was spent by the Signet's cost; the Signet's own red replaces it"
    );
    assert_eq!(pool_amount(&state, p(1), ManaColor::White), 1);
}

// ── T10 — SF-8 deleted the Finding-A exclusion: Cabal Coffers is now a real mana ability ──

/// SR-34 Finding A used to actively exclude `AddManaScaled` from every cost shape except
/// bare `Cost::Tap`, specifically to keep Cabal Coffers's `{2},{T}: Add {B} for each Swamp
/// you control` on the stack: widening the lowering gate without a way to resolve
/// `AddManaScaled`'s dynamic amount would have captured it and demoted it from "correct via
/// the stack" to "exactly one black mana" (the `{colour: 1}` marker, unmultiplied by Swamp
/// count). SF-8 (SR-36, `scutemob-92`) gave `handle_tap_for_mana` that resolution
/// (`ManaAbility::scaled_amount` + step 6c), so the exclusion is gone — this test now pins
/// the exact negation of what it pinned before SF-8 landed. The amount itself (N Swamps ->
/// N black, the `{2}` generic actually leaving the pool) is pinned by activation in
/// `primitive_sr36_scaled_mana_and_life_costs.rs::cabal_coffers_is_a_real_mana_ability`;
/// this test's job is the registration-shape half of that story.
#[test]
fn composite_cost_add_mana_scaled_is_now_a_real_mana_ability() {
    let defs = defs_map();
    let spec = make_spec(p(1), "Cabal Coffers", ZoneId::Battlefield, &defs);
    assert_eq!(
        spec.mana_abilities.len(),
        1,
        "Cabal Coffers must now be captured by the widened lowering (SF-8 deleted the \
         Finding-A exclusion)"
    );
    assert!(
        spec.activated_abilities.is_empty(),
        "the same ability must not ALSO appear in activated_abilities (SF-6 exclusion)"
    );
}

// ── T11 — the two lists can never disagree again (SF-6 / step 5's collapse) ───────

/// SR-34 §3 step 5: `mana_ability_lowering` is now the single predicate deciding both
/// `ManaAbility` registration AND the `activated_abilities` exclusion, so the two lists
/// cannot diverge again the way they silently did for `Effect::AddManaMatchingType`
/// (recognised by the `activated_abilities` exclusion's old `matches!` arm, but not by
/// `try_as_tap_mana_ability` — so a hypothetical `Cost::Tap` + `AddManaMatchingType`
/// ability would have been silently absent from BOTH lists). For every def whose
/// abilities are pure `AbilityDefinition::Activated` entries (excluding `Reconfigure`
/// and `Outlast`, which append additional non-`Activated`-sourced entries to
/// `activated_abilities` and would break a naive count), every `Activated` entry must
/// land in exactly one of `mana_abilities` / `activated_abilities` — never both, never
/// neither.
///
/// **What this test does and does not pin (SR-34 review Finding 6)**: both loops in
/// `enrich_spec_from_def` call the *same* `mana_ability_lowering` function, and the
/// second is the exact negation of the first, so this identity holds **by construction**
/// under the current structure — it would hold even if `mana_ability_lowering` returned
/// `None` for every input (everything would land in `activated_abilities`; the sum still
/// balances). It is a **partition** guard, not a lowering-correctness guard: it fails
/// only if the single predicate is ever re-split into two, re-opening the SF-6/`AddManaMatchingType`
/// divergence. The correctness of what actually lowers (which defs, into what
/// `ManaAbility`) is pinned separately by T1/T10/T13 and
/// `sr34_certified_defs_produce_exactly_their_printed_mana`.
#[test]
fn is_tap_mana_ability_agrees_with_the_lowering() {
    let defs = defs_map();
    let mut checked = 0usize;
    for def in all_cards() {
        if def.abilities.iter().any(|a| {
            matches!(
                a,
                AbilityDefinition::Reconfigure { .. } | AbilityDefinition::Outlast { .. }
            )
        }) {
            continue; // these append non-Activated-sourced activated_abilities entries
        }
        let activated_count = def
            .abilities
            .iter()
            .filter(|a| matches!(a, AbilityDefinition::Activated { .. }))
            .count();
        if activated_count == 0 {
            continue;
        }
        checked += 1;
        let spec = make_spec(p(1), &def.name, ZoneId::Battlefield, &defs);
        assert_eq!(
            spec.mana_abilities.len() + spec.activated_abilities.len(),
            activated_count,
            "{}: every AbilityDefinition::Activated entry must land in exactly one of \
             mana_abilities / activated_abilities (mana={}, activated={}, expected total={})",
            def.name,
            spec.mana_abilities.len(),
            spec.activated_abilities.len(),
            activated_count
        );
    }
    assert!(
        checked > 100,
        "the corpus scan found suspiciously few Activated-ability defs to check ({checked}) — \
         the filter or the corpus is probably broken (SR-5: assert the denominator)"
    );
}

// ── T12 — CR 106.12/106.12a/106.12b apply to composite-cost sources too ──────────

/// CR 106.12b: "If you tap a permanent for mana, it produces three times as much"
/// (Nyxbloom Ancient) applies to a composite-cost mana source exactly as it does to a
/// bare-`{T}` one — this is the positive consequence of `requires_tap`-gated steps 7b/8
/// in `handle_tap_for_mana` being correct as written (SR-34 plan §1): they were
/// unreachable for Boros Signet before SR-34 because it never entered this function at
/// all (it went through `ActivateAbility` + the stack instead).
#[test]
fn composite_cost_mana_source_is_multiplied_by_a_mana_production_replacement() {
    use mtg_engine::CardId;

    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let signet = make_spec(p(1), "Boros Signet", ZoneId::Battlefield, &defs);
    let nyxbloom = make_spec(p(1), "Nyxbloom Ancient", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(Arc::clone(&registry))
        .object(signet)
        .object(nyxbloom)
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 1,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    // GameStateBuilder does not run ETB hooks, so Nyxbloom Ancient's replacement effect
    // must be registered manually (pattern from tests/rules/mana_triggers.rs).
    let battlefield: Vec<(ObjectId, PlayerId, Option<CardId>)> = state
        .objects()
        .iter()
        .filter(|(_, obj)| matches!(obj.zone, ZoneId::Battlefield))
        .map(|(id, obj)| (*id, obj.controller, obj.card_id.clone()))
        .collect();
    for (obj_id, controller, card_id) in &battlefield {
        mtg_engine::rules::replacement::register_permanent_replacement_abilities(
            &mut state,
            *obj_id,
            controller.to_owned(),
            card_id.as_ref(),
            &registry,
        );
    }

    let signet_id = find_by_name(&state, "Boros Signet");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: signet_id,
            ability_index: 0,
        },
    )
    .expect("Boros Signet activation should succeed");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::White),
        3,
        "Nyxbloom Ancient triples a composite-cost mana source's production (CR 106.12b) \
         exactly as it would a bare-{{T}} one"
    );
    assert_eq!(pool_amount(&state, p(1), ManaColor::Red), 3);
}

/// CR 106.12a: "Whenever you tap a [permanent] for mana" fires for a composite-cost
/// mana source too — before SR-34 this trigger path (`fire_mana_triggered_abilities`,
/// only called from inside `handle_tap_for_mana`) was simply never reached for a
/// horizon land, because `TapForMana` could not be issued against it at all.
#[test]
fn composite_cost_mana_source_fires_a_when_tapped_for_mana_trigger() {
    use mtg_engine::cards::card_definition::ManaSourceFilter;
    use mtg_engine::TriggerCondition;

    let defs = defs_map();
    let mut cards = all_cards();
    // A synthetic "whenever you tap a land for mana, add {G}" trigger source — the
    // Crypt Ghast / Mirari's Wake shape (`TriggerCondition::WhenTappedForMana`), scoped
    // to ANY land (not Swamp-only) so it fires off Fiery Islet, which is not a Swamp.
    let trigger_source = CardDefinition {
        card_id: mtg_engine::CardId("sr34-trigger-probe".to_string()),
        name: "SR-34 Trigger Probe".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenTappedForMana {
                source_filter: ManaSourceFilter::Land,
            },
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    green: 1,
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    cards.push(trigger_source.clone());
    let registry = CardRegistry::new(cards);

    let fiery_islet = make_spec(p(1), "Fiery Islet", ZoneId::Battlefield, &defs);
    let probe = enrich_spec_from_def(
        ObjectSpec::card(p(1), "SR-34 Trigger Probe")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(trigger_source.card_id.clone()),
        &defs,
    );
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(fiery_islet)
        .object(probe)
        .player_life(p(1), 5)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let land = find_by_name(&state, "Fiery Islet");
    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land,
            ability_index: 0, // Fiery Islet's {U} arm — still a Land per CR 106.12a
        },
    )
    .expect("Fiery Islet activation should succeed");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Green),
        1,
        "the WhenTappedForMana trigger must fire off a composite-cost land tap (CR 106.12a) \
         and its triggered mana ability resolves immediately (CR 605.4a)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::Green,
                amount: 1,
                ..
            }
        )),
        "ManaAdded(Green, 1) from the trigger should be emitted"
    );
}

// ── T13 — the gates are not vacuous ───────────────────────────────────────────────

/// A synthetic def with a bare `Cost::Tap` still lowers (positive direction); one with
/// `Cost::DiscardCard` + `Cost::Tap` does NOT (negative direction — `DiscardCard` needs
/// a caller-supplied card, which `Command::TapForMana` has no payload for).
#[test]
fn sr34_gates_are_not_vacuous() {
    let defs = defs_map();

    // Positive: bare Cost::Tap still lowers (unchanged pre-SR-34 behaviour).
    let bare_tap_def = CardDefinition {
        card_id: mtg_engine::CardId("sr34-vacuity-bare-tap".to_string()),
        name: "SR-34 Vacuity Probe Bare Tap".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    white: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let mut defs_pos = defs.clone();
    defs_pos.insert(bare_tap_def.name.clone(), bare_tap_def.clone());
    let spec_pos = enrich_spec_from_def(
        ObjectSpec::card(p(1), &bare_tap_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(bare_tap_def.card_id.clone()),
        &defs_pos,
    );
    assert_eq!(
        spec_pos.mana_abilities.len(),
        1,
        "a bare Cost::Tap mana ability must still lower (positive control)"
    );

    // Negative: Cost::Sequence([DiscardCard, Tap]) must NOT lower — DiscardCard needs a
    // caller-supplied card, which TapForMana has no payload for.
    let discard_tap_def = CardDefinition {
        card_id: mtg_engine::CardId("sr34-vacuity-discard-tap".to_string()),
        name: "SR-34 Vacuity Probe Discard Tap".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![Cost::DiscardCard, Cost::Tap]),
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    white: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let mut defs_neg = defs.clone();
    defs_neg.insert(discard_tap_def.name.clone(), discard_tap_def.clone());
    let spec_neg = enrich_spec_from_def(
        ObjectSpec::card(p(1), &discard_tap_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(discard_tap_def.card_id.clone()),
        &defs_neg,
    );
    assert_eq!(
        spec_neg.mana_abilities.len(),
        0,
        "Cost::DiscardCard + Cost::Tap must NOT lower into a ManaAbility (negative control)"
    );
    assert_eq!(
        spec_neg.activated_abilities.len(),
        1,
        "it must instead register as a stack-using activated ability"
    );

    // Negative (SR-34 review Finding 1, CR 601.2h): a second Cost::Mana component in
    // the same Cost::Sequence must NOT lower — `mana_ability_cost_components` cannot
    // merge two mana costs without risking a silent partial payment, so it declines
    // rather than letting the second overwrite the first.
    let double_mana_def = CardDefinition {
        card_id: mtg_engine::CardId("sr34-vacuity-double-mana".to_string()),
        name: "SR-34 Vacuity Probe Double Mana".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
                Cost::Tap,
                Cost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
            ]),
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    white: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let mut defs_double_mana = defs.clone();
    defs_double_mana.insert(double_mana_def.name.clone(), double_mana_def.clone());
    let spec_double_mana = enrich_spec_from_def(
        ObjectSpec::card(p(1), &double_mana_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(double_mana_def.card_id.clone()),
        &defs_double_mana,
    );
    assert_eq!(
        spec_double_mana.mana_abilities.len(),
        0,
        "a second Cost::Mana component must NOT lower into a ManaAbility (negative \
         control — a two-mana-component cost stays on the stack rather than silently \
         under-charging, CR 601.2h)"
    );
    assert_eq!(
        spec_double_mana.activated_abilities.len(),
        1,
        "it must instead register as a stack-using activated ability"
    );

    // Negative (SR-34 review Finding 2, CR 106.12): a cost with no Cost::Tap component
    // must NOT lower — `requires_tap: false` mana abilities have no test coverage of
    // `handle_tap_for_mana`'s tapped-status/exhaustion behaviour with that flag, so the
    // path stays unreachable rather than shipping unproven (see the doc comment on
    // `mana_ability_cost_components`).
    let no_tap_def = CardDefinition {
        card_id: mtg_engine::CardId("sr34-vacuity-no-tap".to_string()),
        name: "SR-34 Vacuity Probe No Tap".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::SacrificeSelf,
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    white: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let mut defs_no_tap = defs;
    defs_no_tap.insert(no_tap_def.name.clone(), no_tap_def.clone());
    let spec_no_tap = enrich_spec_from_def(
        ObjectSpec::card(p(1), &no_tap_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(no_tap_def.card_id.clone()),
        &defs_no_tap,
    );
    assert_eq!(
        spec_no_tap.mana_abilities.len(),
        0,
        "a Cost::SacrificeSelf-only (no Cost::Tap) cost must NOT lower into a \
         ManaAbility (negative control — requires_tap: false is unproven, kept \
         unreachable)"
    );
    assert_eq!(
        spec_no_tap.activated_abilities.len(),
        1,
        "it must instead register as a stack-using activated ability"
    );
}

// ── Roster reconciliation (scutemob-90, criterion 4767) ──────────────────────────

/// Every def SR-34's widened gate certified as working end-to-end, pinned by the exact
/// mana it produces — not by `!mana_abilities.is_empty()`, which is the SF-5/SF-8
/// anti-pattern that let `AddManaScaled` register a `produces: {colour: 1}` marker and
/// call itself covered.
///
/// Each row is `(card, generic cost that MUST be charged, exact mana produced)`. The
/// generic cost is written out here rather than read back off the spec: reading it from
/// the object under test makes the assertion self-fulfilling (a def with the wrong cost
/// gets a pool sized to its wrong cost and balances), which is how `magnifying_glass.rs`
/// shipped a `{1}` cost on a card whose oracle prints a bare `{T}` — that bug survived a
/// draft of this very test.
/// Evidence and the full 27-def verdict table: `memory/primitives/sr34-roster-reconciliation.md`.
///
/// The two lands are the Odyssey filter cycle *as currently errata'd* — Scryfall oracle
/// for both is a plain `{1}, {T}: Add {U}{B}` / `{B}{G}`, not the three-way
/// `{U}{U}, {U}{B}, or {B}{B}` choice the printed cards had. Verified via MCP; do not
/// "fix" them back into a choice.
/// One row of `sr34_certified_defs_produce_exactly_their_printed_mana`:
/// (card name, generic cost that must be charged, exact mana produced).
type CertifiedManaSource = (&'static str, u32, &'static [(ManaColor, u32)]);

#[test]
fn sr34_certified_defs_produce_exactly_their_printed_mana() {
    let cases: [CertifiedManaSource; 10] = [
        (
            "Boros Signet",
            1,
            &[(ManaColor::White, 1), (ManaColor::Red, 1)],
        ),
        (
            "Dimir Signet",
            1,
            &[(ManaColor::Blue, 1), (ManaColor::Black, 1)],
        ),
        (
            "Golgari Signet",
            1,
            &[(ManaColor::Black, 1), (ManaColor::Green, 1)],
        ),
        (
            "Izzet Signet",
            1,
            &[(ManaColor::Blue, 1), (ManaColor::Red, 1)],
        ),
        (
            "Orzhov Signet",
            1,
            &[(ManaColor::White, 1), (ManaColor::Black, 1)],
        ),
        (
            "Rakdos Signet",
            1,
            &[(ManaColor::Black, 1), (ManaColor::Red, 1)],
        ),
        (
            "Simic Signet",
            1,
            &[(ManaColor::Blue, 1), (ManaColor::Green, 1)],
        ),
        (
            "Darkwater Catacombs",
            1,
            &[(ManaColor::Blue, 1), (ManaColor::Black, 1)],
        ),
        (
            "Viridescent Bog",
            1,
            &[(ManaColor::Black, 1), (ManaColor::Green, 1)],
        ),
        // Magnifying Glass's oracle is a *bare* `{T}: Add {C}` (MCP-verified); its def
        // modelled a `{1}` cost and a `{3}` Investigate, both wrong. Fixed in
        // scutemob-90, so it pays nothing and is a real (free) {C} source.
        ("Magnifying Glass", 0, &[(ManaColor::Colorless, 1)]),
    ];

    let defs = defs_map();
    for (name, generic_cost, expected) in cases {
        let def = defs
            .get(name)
            .unwrap_or_else(|| panic!("{name} has no CardDefinition"));
        assert!(
            def.completeness.is_complete(),
            "{name} is certified by this test and must stay Complete"
        );

        let spec = make_spec(p(1), name, ZoneId::Battlefield, &defs);
        assert_eq!(
            spec.mana_abilities.len(),
            1,
            "{name} must register exactly one ManaAbility (CR 605.1a)"
        );

        // The lowered cost must be the one this table declares — not whatever the def
        // happens to say (see the doc comment: reading it back is self-fulfilling).
        let lowered_generic = spec.mana_abilities[0]
            .mana_cost
            .as_ref()
            .map(|mc| mc.generic)
            .unwrap_or(0);
        assert_eq!(
            lowered_generic, generic_cost,
            "{name}: printed activation cost is {{{generic_cost}}} (MCP-verified oracle) \
             but the def lowers a {{{lowered_generic}}} generic cost"
        );

        let registry = CardRegistry::new(all_cards());
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry)
            .object(spec)
            .player_mana(
                p(1),
                mtg_engine::ManaPool {
                    colorless: generic_cost,
                    ..Default::default()
                },
            )
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build");
        state.turn_mut().priority_holder = Some(p(1));

        let source = find_by_name(&state, name);
        let (state, _events) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source,
                ability_index: 0,
            },
        )
        .unwrap_or_else(|e| panic!("{name} TapForMana should succeed: {e:?}"));

        // CR 605.3b: a mana ability never uses the stack.
        assert!(
            state.stack_objects().is_empty(),
            "{name}: a mana ability must not use the stack (CR 605.3b)"
        );

        for color in [
            ManaColor::White,
            ManaColor::Blue,
            ManaColor::Black,
            ManaColor::Red,
            ManaColor::Green,
            ManaColor::Colorless,
        ] {
            let want = expected
                .iter()
                .find(|(c, _)| *c == color)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            assert_eq!(
                pool_amount(&state, p(1), color),
                want,
                "{name}: wrong {color:?} in pool after TapForMana — the {generic_cost} \
                 generic cost must be spent and exactly the printed mana added"
            );
        }
    }
}

/// The reconciliation's marker verdicts, pinned so a def cannot silently drift back to
/// `Complete` without someone re-running the probe.
///
/// `Partial` = every game state the card produces is correct; only CR 605.3b timing is
/// missing (it uses the stack). `KnownWrong` = it writes state outside the legal option
/// set — "Add one mana of any color" producing `{C}`, which CR 106.1a/106.1b says is not
/// a color at all. Reasoning and per-def evidence:
/// `memory/primitives/sr34-roster-reconciliation.md`.
///
/// **SR-36 (`scutemob-92`) graduated three rows off the `Partial` list**: Cabal Coffers,
/// Cabal Stronghold and Crypt of Agadeem are now `Complete`. Their sole recorded blocker
/// was the CR 605.3b mis-registration above, and SF-8 fixed exactly that — it deleted
/// SR-34's Finding-A exclusion and gave `handle_tap_for_mana` the `scaled_amount`
/// resolution the exclusion existed to work around. They are removed from the list rather
/// than left as a stale expectation; the CR 605.1a/605.3b properties they used to fail
/// (registered as a real mana ability, off the stack, correct scaled amount, generic cost
/// actually spent) are now asserted by activation in
/// `primitive_sr36_scaled_mana_and_life_costs.rs`. This gate's job is unchanged for the
/// rows that remain: stop a def drifting back to `Complete` without someone re-running the
/// probe.
#[test]
fn sr34_roster_markers_match_the_reconciliation() {
    let defs = defs_map();

    let partial = [
        "Ashnod's Altar",
        "Phyrexian Tower",
        "Temple of the Dragon Queen",
    ];
    let known_wrong = [
        "Mana Confluence",
        "Staff of Compleation",
        "Voldaren Estate",
        "Phyrexian Altar",
        "Goldhound",
        "Druids' Repository",
        "Gemstone Array",
        "Three Tree City",
        "Maelstrom of the Spirit Dragon",
        "Secluded Courtyard",
        "Unclaimed Territory",
    ];

    for name in partial {
        let def = defs
            .get(name)
            .unwrap_or_else(|| panic!("{name} has no def"));
        assert!(
            matches!(
                def.completeness,
                mtg_engine::cards::Completeness::Partial(_)
            ),
            "{name} must be Partial (right mana, wrong mechanism — CR 605.3b), got {:?}",
            def.completeness
        );
    }
    for name in known_wrong {
        let def = defs
            .get(name)
            .unwrap_or_else(|| panic!("{name} has no def"));
        assert!(
            matches!(
                def.completeness,
                mtg_engine::cards::Completeness::KnownWrong(_)
            ),
            "{name} must be KnownWrong (produces mana outside the legal option set — \
             CR 106.1b), got {:?}",
            def.completeness
        );
    }
}
