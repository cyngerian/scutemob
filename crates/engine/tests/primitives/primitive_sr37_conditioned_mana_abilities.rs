//! SR-37 (`scutemob-93`): SF-10 — a mana ability with an "activate only if ..." restriction
//! must honour that restriction. Finding: `memory/card-authoring/sr34-engine-findings-2026-07-17.md`
//! §SF-10.
//!
//! CR 605.1a is explicit that an activation restriction does *not* disqualify an ability from
//! being a mana ability, so Tainted Field's coloured arms are still lowered into
//! `ManaAbility` (rather than kept on the stack — that would regress SR-33's
//! `every_complete_land_registers_each_printed_tap_mana_color` gate). But before SR-37 the
//! lowering loop's `..` silently dropped `activation_condition`, and `handle_tap_for_mana`
//! never checked one, so Tainted Field tapped for `{W}`/`{B}` with **no Swamp controlled**.
//!
//! Every behavioural test here *activates* the ability and asserts the outcome (mana produced
//! or an error), never registration shape alone — the SF-8 lesson (a data-model test can pin
//! a defect as a requirement), per `tests/core/effect_choose_gate.rs` and
//! `primitive_sr36_scaled_mana_and_life_costs.rs`.

use std::collections::HashMap;

use mtg_engine::cards::{AbilityDefinition, Cost};
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition, CardRegistry,
    Command, Effect, GameState, GameStateBuilder, ManaColor, ObjectId, ObjectSpec, PlayerId, Step,
    ZoneId,
};

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
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Battlefield)
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

/// Build a state holding a Tainted Field controlled by p(1), plus (optionally) a Swamp, with
/// p(1) holding priority in a main phase. Each caller gets a fresh state so activating one
/// ability's `{T}` cost never taps the land under test for the next assertion.
fn tainted_field_state(with_swamp: bool) -> GameState {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(make_spec(p(1), "Tainted Field", &defs));
    if with_swamp {
        builder = builder.object(make_spec(p(1), "Swamp", &defs));
    }
    let mut state = builder
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));
    state
}

// Tainted Field's abilities (source order): [0] `{T}: Add {C}` (unconditioned),
// [1] `{T}: Add {W}` (Swamp condition), [2] `{T}: Add {B}` (Swamp condition). All three are
// bare-`Cost::Tap` single-colour `AddMana`, so all three lower into mana abilities in the
// same order — `ability_index` matches the source index.
const COLORLESS_ARM: usize = 0;
const WHITE_ARM: usize = 1;
const BLACK_ARM: usize = 2;

/// The unconditioned `{C}` arm is always legal — it must not be collateral damage of the
/// new condition check.
#[test]
fn tainted_field_colorless_arm_needs_no_swamp() {
    let state = tainted_field_state(false);
    let land = find_by_name(&state, "Tainted Field");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land,
            ability_index: COLORLESS_ARM,
        },
    )
    .expect("the unconditioned {C} arm must be legal with no Swamp");
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        1,
        "the {{C}} arm carries no activation_condition and must produce {{C}}"
    );
}

/// SF-10, the load-bearing case: the coloured arms are rejected with no Swamp controlled
/// (CR 602.5b). Pre-fix these produced `{W}`/`{B}` regardless.
#[test]
fn tainted_field_colored_arms_rejected_without_a_swamp() {
    for arm in [WHITE_ARM, BLACK_ARM] {
        let state = tainted_field_state(false);
        let land = find_by_name(&state, "Tainted Field");
        let result = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: land,
                ability_index: arm,
            },
        );
        assert!(
            result.is_err(),
            "ability_index {arm} is 'Activate only if you control a Swamp' — with no Swamp it \
             must be rejected (CR 602.5b), not produce mana"
        );
    }
}

/// With a Swamp controlled, each coloured arm activates and produces exactly its colour.
#[test]
fn tainted_field_colored_arms_work_with_a_swamp() {
    for (arm, color) in [(WHITE_ARM, ManaColor::White), (BLACK_ARM, ManaColor::Black)] {
        let state = tainted_field_state(true);
        let land = find_by_name(&state, "Tainted Field");
        let (state, _events) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: land,
                ability_index: arm,
            },
        )
        .unwrap_or_else(|e| panic!("arm {arm} must be legal while controlling a Swamp: {e:?}"));
        assert_eq!(
            pool_amount(&state, p(1), color),
            1,
            "arm {arm} must add exactly one {color:?} when a Swamp is controlled"
        );
    }
}

/// Structural companion: the condition must actually reach the registered `ManaAbility`.
/// The `{C}` arm carries none; both coloured arms carry `Some`. This pins the threading in
/// `mana_ability_lowering` so the `..`-drop regression cannot silently return.
#[test]
fn tainted_field_registers_condition_on_colored_arms_only() {
    let defs = defs_map();
    let spec = make_spec(p(1), "Tainted Field", &defs);
    assert_eq!(
        spec.mana_abilities.len(),
        3,
        "Tainted Field must register all three tap-mana arms"
    );
    assert!(
        spec.mana_abilities[COLORLESS_ARM]
            .activation_condition
            .is_none(),
        "the {{C}} arm has no activation condition"
    );
    for arm in [WHITE_ARM, BLACK_ARM] {
        assert!(
            spec.mana_abilities[arm].activation_condition.is_some(),
            "coloured arm {arm} must carry its 'control a Swamp' condition into the ManaAbility"
        );
    }
}

/// Corpus backstop (CR 605.1a + 602.5b): every bare-`Cost::Tap`, no-target, single-effect
/// `AddMana` activated ability carrying an `activation_condition` must surface a registered
/// `ManaAbility` that carries that exact condition. A bare `Cost::Tap` always lowers, so this
/// predicate never over-counts a conditioned ability that legitimately stays on the stack.
/// Non-vacuous: Tainted Field's two coloured arms are counted, asserted below.
#[test]
fn conditioned_bare_tap_mana_abilities_carry_their_condition() {
    let defs = defs_map();
    let mut conditioned_seen = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for def in all_cards() {
        // Def-side: the conditions carried by bare-Tap, no-target, AddMana activated abilities.
        let def_conditions: Vec<_> = def
            .abilities
            .iter()
            .filter_map(|ab| match ab {
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana { .. },
                    targets,
                    activation_condition: Some(c),
                    ..
                } if targets.is_empty() => Some(c.clone()),
                _ => None,
            })
            .collect();
        if def_conditions.is_empty() {
            continue;
        }
        conditioned_seen += def_conditions.len();

        // Registered-side: the conditions carried by the enriched mana abilities.
        let spec = make_spec(p(1), &def.name, &defs);
        let registered_conditions: Vec<_> = spec
            .mana_abilities
            .iter()
            .filter_map(|ma| ma.activation_condition.clone())
            .collect();

        for cond in &def_conditions {
            if !registered_conditions.contains(cond) {
                failures.push(format!(
                    "{}: def declares a conditioned tap-mana ability ({cond:?}) but no \
                     registered ManaAbility carries that condition — the condition was dropped \
                     in lowering (SF-10)",
                    def.name
                ));
            }
        }
    }

    assert!(
        conditioned_seen >= 2,
        "expected at least Tainted Field's 2 conditioned coloured arms; found {conditioned_seen} \
         — the gate has gone vacuous (a serde/def change hid the corpus case)"
    );
    assert!(failures.is_empty(), "{failures:#?}");
}
