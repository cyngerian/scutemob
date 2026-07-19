//! Tests for PB-EF12: granted/intrinsic `any_color` ManaAbility colour choice
//! (EF-W-PB2-3, CR 605.3b / 106.1b / 111.10a).
//!
//! A mana ability resolves immediately and never uses the stack (CR 605.3b), so any
//! colour choice for an `any_color: true` ManaAbility's production must be made at
//! activation, on the command itself: `Command::TapForMana` gains
//! `chosen_color: Option<ManaColor>`. `handle_tap_for_mana` (`rules/mana.rs`) validates it:
//! - `any_color == true`: requires `Some(c)` with `c` one of White/Blue/Black/Red/Green.
//!   `Some(Colorless)` is rejected (CR 106.1b: colorless is a mana TYPE, not a colour) and
//!   `None` is rejected (no silent default) — both via `GameStateError::InvalidCommand`.
//! - `any_color == false`: `chosen_color` must be `None`.
//!
//! This closes EF-W-PB2-3 and the EF queue. `PROTOCOL_VERSION` bumped 17 -> 18
//! (`Command::TapForMana` gained a field, a wire-frame shape change).
//! `HASH_SCHEMA_VERSION` does NOT bump — `Command` is not in the `GameState` hash
//! closure, and the colour lands in `ManaPool`, already per-colour.

use imbl::OrdMap;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, CardRegistry,
    Command, ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent,
    GameState, GameStateBuilder, GameStateError, LayerModification, ManaAbility, ManaColor,
    ObjectId, ObjectSpec, PlayerId, Step, PROTOCOL_VERSION,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
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

/// A synthetic `any_color: true` mana source ("Test Rock" — a tap-only artifact) for
/// controlled, card-independent tests of the colour-choice channel itself.
fn any_color_source_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    let mut ma = ManaAbility::tap_for(ManaColor::White);
    ma.any_color = true;
    ma.produces = Default::default();
    ObjectSpec::artifact(owner, name).with_mana_ability(ma)
}

fn defs_map() -> HashMap<String, mtg_engine::CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

// ── Happy path: each of WUBRG, stack stays empty (CR 605.3b) ──────────────────

/// CR 111.10a / CR 605.3b — an `any_color: true` mana ability, activated with
/// `chosen_color: Some(c)`, adds exactly `c` to the pool for every legal colour, and the
/// stack stays empty (mana abilities never use the stack).
#[test]
fn test_ef12_any_color_choice_produces_each_legal_color() {
    for color in [
        ManaColor::White,
        ManaColor::Blue,
        ManaColor::Black,
        ManaColor::Red,
        ManaColor::Green,
    ] {
        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .object(any_color_source_spec(p(1), "Test Rock"))
            .build()
            .expect("state builds");

        let source = find_by_name(&state, "Test Rock");
        let (state, events) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source,
                ability_index: 0,
                chosen_color: Some(color),
            },
        )
        .unwrap_or_else(|e| panic!("TapForMana with chosen_color {color:?} should succeed: {e:?}"));

        assert_eq!(
            pool_amount(&state, p(1), color),
            1,
            "chosen_color {color:?} must add exactly 1 mana of that colour"
        );
        assert!(
            state.stack_objects().is_empty(),
            "CR 605.3b: a mana ability must never use the stack"
        );
        assert!(
            events.iter().any(
                |e| matches!(e, GameEvent::ManaAdded { color: c, amount: 1, .. } if *c == color)
            ),
            "ManaAdded event must carry the chosen colour, not Colorless"
        );
    }
}

// ── Granted happy path: end-to-end via a real card def + process_command ──────

/// PB-EF12 (EF-W-PB2-3): Elven Chorus's card def now authors a real
/// `LayerModification::AddManaAbility(ManaAbility{any_color:true, ..})` grant to
/// `EffectFilter::CreaturesYouControl` (previously a `partial` TODO — see
/// `crates/card-defs/src/defs/elven_chorus.rs`). Programmatic check (not eyeball, per plan
/// criterion (a)): the def's serialized shape must contain the grant.
#[test]
fn test_ef12_elven_chorus_def_authors_the_any_color_grant() {
    let defs = defs_map();
    let chorus = defs.get("Elven Chorus").expect("Elven Chorus has a def");
    assert!(
        chorus.completeness.is_complete(),
        "Elven Chorus should be Complete post-PB-EF12"
    );
    let json = serde_json::to_value(chorus).expect("serializes");
    let text = json.to_string();
    assert!(
        text.contains("AddManaAbility") && text.contains("CreaturesYouControl"),
        "Elven Chorus's def must contain a granted any_color mana ability to creatures you \
         control: {text}"
    );
}

/// CR 613.1f / 605.1a / PB-EF12 — a granted `any_color: true` ManaAbility (the same shape
/// Elven Chorus, Cryptolith Rite, and Paradise Mantle all author via
/// `LayerModification::AddManaAbility`) resolves through the exact same colour-choice
/// channel as an intrinsic one: tapping the granted ability with `chosen_color: Some(Red)`
/// adds red, not colorless. End-to-end through `process_command` and layer-resolved
/// characteristics (CR 613.1f), not a direct effect call — this is the dispatch path all
/// four granted-`any_color` cards share, so one test proves it for all of them.
#[test]
fn test_ef12_granted_any_color_choice_end_to_end() {
    let source = ObjectSpec::card(p(1), "Elven Chorus")
        .with_types(vec![mtg_engine::CardType::Enchantment])
        .in_zone(mtg_engine::ZoneId::Battlefield);
    let bear = ObjectSpec::creature(p(1), "Mana Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(bear)
        .build()
        .expect("state builds");

    let source_id = find_by_name(&state, "Elven Chorus");
    let bear_id = find_by_name(&state, "Mana Bear");

    // The same grant shape elven_chorus.rs / cryptolith_rite.rs author (CR 613.1f, Layer 6):
    // "Creatures you control have '{T}: Add one mana of any color.'"
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddManaAbility(ManaAbility {
            produces: OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
            ..Default::default()
        }),
        is_cda: false,
        condition: None,
    });

    let chars = calculate_characteristics(&state, bear_id).expect("bear characteristics resolve");
    assert!(
        chars.mana_abilities.iter().any(|ma| ma.any_color),
        "the grant must register a real any_color mana ability on the creature"
    );

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: bear_id,
            ability_index: 0,
            chosen_color: Some(ManaColor::Red),
        },
    )
    .expect("TapForMana on the granted ability with chosen_color Some(Red) should succeed");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Red),
        1,
        "the granted any_color ability must add the chosen colour (red), not colorless"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        0,
        "CR 106.1b: colorless must never be produced by an any_color ability"
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
        "ManaAdded event must carry red"
    );
}

// ── Decoys (non-vacuous): each proven to fail for exactly the field under test ─

/// Decoy A — CR 106.1b: `chosen_color: Some(Colorless)` on an any_color source is rejected.
/// Colorless is a mana TYPE, not a colour, so it is outside the legal option set "any color"
/// offers — not a degraded choice. Non-vacuous: removing the Colorless check in
/// `handle_tap_for_mana` (rules/mana.rs) makes this test pass with `Ok`, so this assertion
/// exercises exactly that check.
#[test]
fn test_ef12_decoy_colorless_choice_rejected() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(any_color_source_spec(p(1), "Test Rock"))
        .build()
        .expect("state builds");

    let source = find_by_name(&state, "Test Rock");
    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,
            chosen_color: Some(ManaColor::Colorless),
        },
    );

    assert!(
        result.is_err(),
        "CR 106.1b: an any_color ability must reject a Colorless choice"
    );
    assert!(
        matches!(result.err().unwrap(), GameStateError::InvalidCommand(_)),
        "the rejection must be InvalidCommand, naming the CR violation"
    );
}

/// Decoy B — CR 605.3b: `chosen_color: None` on an any_color source is rejected (no silent
/// default to Colorless, unlike the pre-PB-EF12 engine). Non-vacuous: removing the `None =>
/// Err` arm in `handle_tap_for_mana` and falling back to a default colour makes this test
/// pass, so this assertion exercises exactly that arm.
#[test]
fn test_ef12_decoy_missing_choice_rejected() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(any_color_source_spec(p(1), "Test Rock"))
        .build()
        .expect("state builds");

    let source = find_by_name(&state, "Test Rock");
    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,
            chosen_color: None,
        },
    );

    assert!(
        result.is_err(),
        "an any_color ability with no chosen_color must be rejected, not silently default"
    );
    assert!(
        matches!(result.err().unwrap(), GameStateError::InvalidCommand(_)),
        "the rejection must be InvalidCommand"
    );
}

/// Decoy C — a `chosen_color` supplied for a FIXED-colour source (Forest) is rejected. The
/// colour channel is scoped to `any_color` abilities only; supplying one for a fixed-colour
/// ability is a caller bug the engine catches rather than silently ignores. Non-vacuous:
/// removing the `chosen_color.is_some() => Err` branch for the `!any_color` arm makes this
/// test pass (the Forest would just add green, ignoring the bogus colour), so this assertion
/// exercises exactly that branch.
#[test]
fn test_ef12_decoy_fixed_color_source_rejects_a_choice() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let forest_spec = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Forest").in_zone(mtg_engine::ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(forest_spec)
        .build()
        .expect("state builds");

    let forest = find_by_name(&state, "Forest");
    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: forest,
            ability_index: 0,
            chosen_color: Some(ManaColor::Green),
        },
    );

    assert!(
        result.is_err(),
        "a fixed-colour ability (Forest, {{T}}: Add {{G}}) must reject a supplied chosen_color"
    );
    assert!(
        matches!(result.err().unwrap(), GameStateError::InvalidCommand(_)),
        "the rejection must be InvalidCommand"
    );
}

// ── Version sentinels ───────────────────────────────────────────────────────────

/// PB-EF12 wire bump: `Command::TapForMana` gained `chosen_color`, a wire-frame shape
/// change (SR-8 closure). This sentinel forces deliberate review of any further bump.
#[test]
fn test_ef12_protocol_version_sentinel() {
    assert_eq!(
        PROTOCOL_VERSION, 19,
        "PROTOCOL_VERSION changed. Update this sentinel and the History list in \
         rules/protocol.rs."
    );
}
