//! Discriminating probe + regression tests for PB-RS1 (OOS-RS-1): reconcile which
//! end of `Zone::Ordered` the library-scanning effects (Scry, Surveil,
//! RevealAndRoute, LookAtTopThenPlace) read/write against the end `draw_card`
//! actually takes.
//!
//! CR 121.1 is definitional: a draw is "putting the top card of the library
//! into the hand." `draw_card` uses `Zone::top()` = `v.last()`, so the LAST
//! element of the backing vector is the top, and the FIRST element (index 0)
//! is the bottom. Before this PB, Scry/Surveil/RevealAndRoute/LookAtTopThenPlace
//! all read `object_ids().take(n)` — indices `0..n` — which is the BOTTOM under
//! this convention, not the top. These tests are the canary that
//! `reveal_and_route.rs`'s 2-card/`count:4` shape failed to be (it exercises the
//! whole library, so read-end drift never surfaces there).
//!
//! These probe tests are kept permanently as regressions once the defect is fixed.

use mtg_engine::cards::card_definition::{TargetFilter, ZoneTarget};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::turn_actions::draw_card;
use mtg_engine::state::game_object::ObjectId;
use mtg_engine::state::turn::Step;
use mtg_engine::state::types::CardType;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{GameStateBuilder, ObjectSpec, PlayerId};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command,
    Completeness, Effect, EffectAmount, GameEvent, GameState, KeywordAbility, LibraryPosition,
    ManaCost, PlayerTarget, TypeLine,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn ec(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

/// Build a 3-card library for `p(1)`: "Card Alpha", "Card Beta", "Card Gamma",
/// declared in that order. `GameStateBuilder::object(...)` appends, so the
/// resulting `Zone::Ordered` vector is `[Alpha, Beta, Gamma]`. Under CR 121.1
/// (`Zone::top()` = last element = what `draw_card` takes), `Gamma` is the top
/// card and `Alpha` is the bottom.
fn three_card_library() -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Card Alpha")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Card Beta")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Card Gamma")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap()
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| "<missing>".to_string())
}

// ── Step-0 discriminating probe (CR 121.1 / 701.22a / 701.25a) ─────────────────

#[test]
/// CR 121.1, CR 701.22a — draw and Scry must agree on which card is "the top."
/// `draw_card` takes `Zone::top()` (last element). A `Scry 1` always bottoms the
/// card it looked at (deterministic fallback, CR 701.22a "any number"), so the
/// card Scry examines ends up at index 0 (the new bottom). That card must be the
/// SAME one `draw_card` would have taken. Pre-fix, Scry read indices `0..1` (the
/// existing bottom, "Card Alpha") and wrote back via `push_back` (append/top) —
/// so index 0 after a pre-fix Scry 1 is neither the drawn card nor the original
/// bottom; it is whatever was second-from-bottom ("Card Beta"), demonstrating
/// that both the read and the write end were wrong.
fn test_probe_draw_and_scry_agree_on_top() {
    // Draw side: fresh clone, draw one card, record its name.
    let mut draw_state = three_card_library();
    draw_card(&mut draw_state, p(1)).expect("draw should succeed");
    let hand_id = draw_state
        .objects()
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("a card should have been drawn to hand");
    let drawn_name = name_of(&draw_state, hand_id);
    assert_eq!(
        drawn_name, "Card Gamma",
        "CR 121.1: draw takes the last element (Zone::top())"
    );

    // Scry side: fresh clone of the SAME pre-draw setup, Scry 1.
    let mut scry_state = three_card_library();
    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);
    execute_effect(
        &mut scry_state,
        &Effect::Scry {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        &mut ctx,
    );

    // Scry's deterministic fallback bottoms every card it looked at, so the
    // card now at index 0 is the card Scry examined. It must be the drawn card.
    let lib_ids = scry_state
        .zones()
        .get(&ZoneId::Library(p(1)))
        .expect("library zone exists")
        .object_ids();
    let bottom_name = name_of(&scry_state, lib_ids[0]);
    assert_eq!(
        bottom_name, drawn_name,
        "CR 701.22a: Scry 1 must look at (and re-bottom) the same card draw_card takes"
    );
}

#[test]
/// CR 121.1 — RevealAndRoute must see the same card `draw_card` yields.
fn test_probe_reveal_and_route_sees_the_drawn_card() {
    let mut state = three_card_library();
    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    execute_effect(
        &mut state,
        &Effect::RevealAndRoute {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: TargetFilter::default(),
            matched_dest: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            unmatched_dest: ZoneTarget::Library {
                owner: PlayerTarget::Controller,
                position: LibraryPosition::Bottom,
            },
        },
        &mut ctx,
    );

    let hand_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("a card should have been routed to hand");
    assert_eq!(
        name_of(&state, hand_id),
        "Card Gamma",
        "CR 121.1: RevealAndRoute count:1 must see the same card draw_card yields"
    );
}

#[test]
/// CR 701.25a, CR 121.1 — Surveil must mill the same card `draw_card` yields.
fn test_probe_surveil_mills_the_drawn_card() {
    let mut state = three_card_library();
    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    execute_effect(
        &mut state,
        &Effect::Surveil {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        &mut ctx,
    );

    let gy_names: Vec<String> = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Graveyard(p(1)))
        .map(|o| o.characteristics.name.clone())
        .collect();
    assert!(
        gy_names.contains(&"Card Gamma".to_string()),
        "CR 701.25a: Surveil 1 should mill the top card (Card Gamma), got {gy_names:?}"
    );
    assert!(
        !gy_names.contains(&"Card Alpha".to_string()),
        "CR 701.25a: Surveil 1 must not mill the bottom (Card Alpha), got {gy_names:?}"
    );
}

// ── Test 3 — cascade round-trip (CR 702.85a + CR 701.22a + CR 121.1) ───────────

/// Build a cascade sorcery definition (mirrors `mechanics_a_d/cascade.rs`).
fn cascade_sorcery(id: &str, name: &str, mv: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: Some(ManaCost {
            generic: mv,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: format!("Cascade (Mana value {mv})"),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cascade),
            AbilityDefinition::Spell {
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

/// Build a plain (non-cascade) sorcery definition.
fn plain_sorcery(id: &str, name: &str, mv: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: Some(ManaCost {
            generic: mv,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Plain sorcery".into(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

#[test]
/// CR 702.85a + CR 701.22a + CR 121.1 — a card cascade bottoms must not be seen
/// (read OR moved) by a subsequent Scry. This is the cross-subsystem check the
/// spec calls out: pre-fix, cascade correctly bottoms a card via
/// `move_object_to_bottom_of_zone`, but a following Scry's `object_ids().take(n)`
/// read would land exactly on that just-bottomed card (index 0) and re-move it
/// via the old buggy `push_back` write — undoing the cascade's bottoming and
/// minting the bottomed card a NEW ObjectId (CR 400.7) in the process.
///
/// Discriminator: capture the bottomed card's ObjectId right after cascade
/// resolves, then assert that exact ObjectId is still present (same object,
/// untouched) after the following Scry. Pre-fix this fails (Scry's read hits
/// the bottomed card and moves it, minting a new ID); post-fix it holds (Scry
/// only reads the true top, never touching the bottomed card).
fn test_cascade_bottomed_card_is_not_seen_by_next_scry() {
    let p1 = p(1);
    let p2 = p(2);

    // MV=5 cascade spell.
    let cascade_def = cascade_sorcery("rs1-cascade-spell", "Cascade Spell", 5);
    // MV=5 sorcery (equal MV, doesn't qualify -- exiled, then bottomed).
    let reject_def = plain_sorcery("rs1-cascade-reject", "Cascade Reject", 5);
    // MV=2 sorcery (qualifies -- cast for free, removed from library).
    let small_def = plain_sorcery("rs1-small-spell", "Small Spell", 2);
    // Untouched filler that stays at the true bottom throughout.
    let filler_def = plain_sorcery("rs1-bottom-filler", "Bottom Filler", 1);

    let cascade_id = cascade_def.card_id.clone();
    let reject_id = reject_def.card_id.clone();
    let small_id = small_def.card_id.clone();
    let filler_id = filler_def.card_id.clone();

    let registry = CardRegistry::new(vec![cascade_def, reject_def, small_def, filler_def]);

    // Push order (last pushed = top, per Zone::top() = last element):
    // Bottom Filler (bottom), Small Spell, Cascade Reject (top -- examined first).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Cascade Spell")
                .with_card_id(cascade_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 5,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Cascade)
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Bottom Filler")
                .with_card_id(filler_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Small Spell")
                .with_card_id(small_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Cascade Reject")
                .with_card_id(reject_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 5,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state.players_mut().get_mut(&p1).unwrap().mana_pool.colorless = 5;

    let cascade_hand_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Cascade Spell")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: cascade_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap();

    // Resolve the cascade trigger -- this runs `resolve_cascade`, which exiles
    // "Cascade Reject" (MV5, doesn't qualify), then "Small Spell" (MV2,
    // qualifies -- cast for free), then bottoms "Cascade Reject" (CR 702.85a).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    let cascade_cast = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::CascadeCast { .. }));
    assert!(cascade_cast, "cascade should have cast Small Spell");

    // Library should now hold exactly 2 cards: Cascade Reject (bottomed,
    // index 0) and Bottom Filler (now the top, index 1).
    let lib_ids = state
        .zones()
        .get(&ZoneId::Library(p1))
        .unwrap()
        .object_ids();
    assert_eq!(
        lib_ids.len(),
        2,
        "library should hold Cascade Reject + Bottom Filler after cascade"
    );
    assert_eq!(
        name_of(&state, lib_ids[0]),
        "Cascade Reject",
        "CR 702.85a: the non-cast exiled card must be bottomed (index 0)"
    );
    let reject_id_after_cascade = lib_ids[0];

    // Run Scry 1. Under the fix, this reads the TRUE top (Bottom Filler) and
    // never touches "Cascade Reject" at all.
    let mut state = state;
    let source_id = ObjectId(999);
    let mut ctx = ec(p1, source_id);
    execute_effect(
        &mut state,
        &Effect::Scry {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        &mut ctx,
    );

    // The exact object cascade bottomed must still exist, untouched, in the
    // library. Pre-fix, Scry's `object_ids().take(1)` read hits index 0 (the
    // just-bottomed Cascade Reject), moves it, and mints it a NEW ObjectId
    // (CR 400.7) -- so this lookup would fail pre-fix.
    let still_present = state
        .objects()
        .get(&reject_id_after_cascade)
        .map(|o| o.zone == ZoneId::Library(p1))
        .unwrap_or(false);
    assert!(
        still_present,
        "CR 701.22a: Scry 1 must not read or move the card cascade just bottomed \
         (object identity should survive -- CR 400.7 says a NEW id means it was moved)"
    );
}

// ── Test 4 — scry-to-bottom ordering (CR 701.22a + CR 121.1) ───────────────────

#[test]
/// CR 701.22a + CR 121.1 — Scry 2, both to the bottom (deterministic fallback),
/// must land below every pre-existing card, and the new top (via `draw_card`)
/// must be the card that was NOT looked at. This is the cross-check that the
/// write side and the read side agree: a compensating double-inversion (wrong
/// read + wrong write cancelling out) would pass the step-0 probe alone.
fn test_scry_two_to_bottom_lands_below_everything() {
    // Vector order (declared bottom to top): Bottom1, Bottom2, Bottom3, Top2, Top1.
    // Top1 is topmost (last element / Zone::top()).
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Bottom1")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Bottom2")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Bottom3")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Top2")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Top1")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);
    execute_effect(
        &mut state,
        &Effect::Scry {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(2),
        },
        &mut ctx,
    );

    // Deterministic fallback bottoms both looked-at cards, sorted by ObjectId
    // ascending. Top1 and Top2 (declared first, so lower ObjectIds) must now
    // occupy indices 0 and 1 -- below every pre-existing card.
    let lib_ids = state
        .zones()
        .get(&ZoneId::Library(p(1)))
        .unwrap()
        .object_ids();
    assert_eq!(lib_ids.len(), 5, "no cards should be lost");
    let names: Vec<String> = lib_ids.iter().map(|&id| name_of(&state, id)).collect();
    assert_eq!(
        &names[0..2],
        &["Top1".to_string(), "Top2".to_string()],
        "CR 701.22a: both scried cards must land below every pre-existing card, got {names:?}"
    );
    assert_eq!(
        &names[2..5],
        &[
            "Bottom1".to_string(),
            "Bottom2".to_string(),
            "Bottom3".to_string()
        ],
        "CR 121.1: the untouched cards must keep their original relative order, got {names:?}"
    );

    // Cross-check: draw_card must now yield "Bottom3", the new topmost card.
    let mut draw_state = state;
    let events = draw_card(&mut draw_state, p(1)).expect("draw should succeed");
    assert!(!events.is_empty(), "draw should produce an event");
    let hand_id = draw_state
        .objects()
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("a card should have been drawn to hand");
    assert_eq!(
        name_of(&draw_state, hand_id),
        "Bottom3",
        "CR 121.1: draw_card must agree with Scry's write side on the new top"
    );
}
