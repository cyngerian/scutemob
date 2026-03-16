//! Cascade keyword tests (CR 702.85).
//!
//! Session 9 of M9.4 implements cascade:
//! - `KeywordAbility::Cascade` — triggers when the spell is cast
//! - `resolve_cascade` in `rules/copy.rs` — exiles library cards until finding
//!   a qualifying nonland card with mana value strictly less than the cascade spell,
//!   casts it for free, puts remaining exiled cards on the library bottom
//! - `GameEvent::CascadeExiled` and `GameEvent::CascadeCast`
//!
//! CC#29 (split card mana value) is documented but not implemented since the
//! engine doesn't support split cards yet (CR 708.4).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost,
    ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Pass priority for all listed players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Build a cascade sorcery definition.
///
/// The spell has mana value `mv` (generic), Cascade keyword, and a simple
/// GainLife(1) effect (the actual effect is irrelevant for cascade tests).
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
        oracle_text: format!("Cascade (Mana value {})", mv),
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
        meld_pair: None,
    }
}

/// Build a plain sorcery definition (no cascade).
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
        meld_pair: None,
    }
}

/// Build a land definition for the "stop at land" cascade test.
fn basic_land(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "{T}: Add {G}.".into(),
        abilities: vec![],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}

// ── CR 702.85b: Cascade exiles until finding a qualifying card ─────────────────

/// CR 702.85b — Casting a cascade spell exiles cards from top of library until
/// a nonland card with mana value strictly less than the cascade spell is found.
///
/// Setup: cascade spell with MV=5. Library (top to bottom): Forest (land, skip),
/// big sorcery MV=5 (equal, skip), small sorcery MV=2 (MV<5, cast this).
///
/// After cascade: small sorcery is on the stack (cast for free), Forest and
/// big sorcery are on the bottom of the library (or in exile if the put-back
/// logic uses move_object_to_zone — they may end up in library zone).
#[test]
fn test_cascade_exiles_until_hit() {
    let p1 = p1();
    let p2 = p2();

    // MV=5 cascade spell
    let cascade_def = cascade_sorcery("cascade-mv5", "Big Cascade Spell", 5);
    // MV=5 sorcery (equal MV, should be skipped)
    let equal_mv_def = plain_sorcery("equal-mv-spell", "Equal MV Spell", 5);
    // MV=2 sorcery (less than 5, should be cast)
    let small_def = plain_sorcery("small-mv-spell", "Small Spell", 2);
    // Basic land (should be skipped)
    let forest_def = basic_land("forest-cascade-test", "Forest Test");

    let cascade_card_id = cascade_def.card_id.clone();
    let small_card_id = small_def.card_id.clone();
    let forest_card_id = forest_def.card_id.clone();
    let equal_card_id = equal_mv_def.card_id.clone();

    let registry = CardRegistry::new(vec![cascade_def, equal_mv_def, small_def, forest_def]);

    // Library (push_back = appended; last pushed = top of library, drawn first):
    // We want top-to-bottom: Forest, EqualMV(5), SmallMV(2)
    // Push order: Forest first (bottom), EqualMV next, SmallMV last (top).
    // Actually: push_back appends, top() returns last().
    // So push order determines position with last pushed = top.
    // To get top=SmallMV, middle=EqualMV, bottom=Forest:
    //   Push Forest first, then EqualMV, then SmallMV.
    // But wait — we want cascade to exile SmallMV LAST (it's the qualifying card).
    // Cascade exiles from TOP until finding qualifying.
    // We want top=Forest (skip), second=EqualMV (skip, MV=5 not <5), third=SmallMV (cast).
    // So top = Forest → push last: Forest is last = top.
    // Push order: SmallMV, EqualMV, Forest (Forest = last = top).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        // Cascade spell in hand.
        .object(
            ObjectSpec::card(p1, "Big Cascade Spell")
                .with_card_id(cascade_card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 5,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Cascade)
                .in_zone(ZoneId::Hand(p1)),
        )
        // SmallMV on bottom of library (pushed first).
        .object(
            ObjectSpec::card(p1, "Small Spell")
                .with_card_id(small_card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        // EqualMV in middle (pushed second).
        .object(
            ObjectSpec::card(p1, "Equal MV Spell")
                .with_card_id(equal_card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 5,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        // Forest on top (pushed last = top = last element).
        .object(
            ObjectSpec::card(p1, "Forest Test")
                .with_card_id(forest_card_id)
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Give p1 enough mana to cast the MV=5 cascade spell.
    let mut state = state;
    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 5;

    let cascade_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Big Cascade Spell")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
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
        },
    )
    .unwrap();

    // CR 702.85a: After cast, the cascade trigger is on the stack above the spell.
    // Stack has: [cascade spell (bottom), cascade trigger (top)] = 2 entries.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: cascade spell + cascade trigger on stack; got {} stack objects",
        state.stack_objects.len()
    );

    // At cast time: 1 SpellCast (cascade spell) + 1 AbilityTriggered (cascade trigger).
    // No CascadeCast yet — that fires when the trigger resolves.
    let spell_cast_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        spell_cast_count, 1,
        "1 SpellCast event at cast time (the cascade spell itself); got {}",
        spell_cast_count
    );
    let cascade_cast_at_cast = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CascadeCast { .. }))
        .count();
    assert_eq!(
        cascade_cast_at_cast, 0,
        "No CascadeCast event at cast time (trigger hasn't resolved); got {}",
        cascade_cast_at_cast
    );

    // Resolve the cascade trigger (pass priority for both players).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 702.85b: After trigger resolves, the qualifying card (Small Spell) is on the stack.
    // Stack: [cascade spell (bottom), cascaded spell (top)] = 2 entries.
    assert!(
        state.stack_objects.len() >= 2,
        "Cascade spell + cascaded spell should both be on stack after trigger resolves; got {}",
        state.stack_objects.len()
    );

    // CascadeCast fires during trigger resolution.
    let cascade_cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CascadeCast { .. }))
        .count();
    assert_eq!(
        cascade_cast_count, 1,
        "Exactly 1 CascadeCast event when trigger resolves; got {}",
        cascade_cast_count
    );

    // SpellCast for the cascaded spell fires during trigger resolution.
    // (cascade IS a cast per CR 702.85c)
    let spell_cast_on_resolve = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        spell_cast_on_resolve, 1,
        "1 SpellCast event when trigger resolves (cascaded spell); got {}",
        spell_cast_on_resolve
    );
}

// ── CR 702.85b: Cascade over lands ────────────────────────────────────────────

/// CR 702.85b — Cascade skips over land cards when searching the library.
///
/// A land has no mana cost (mana value 0 in non-stack zones). However, lands
/// are excluded from cascade targets by the rule: "until you exile a nonland
/// card with mana value less than this spell's mana value."
///
/// This test verifies: a land at the top of the library is exiled (skipped)
/// before reaching the qualifying nonland card.
#[test]
fn test_cascade_skips_lands() {
    let p1 = p1();
    let p2 = p2();

    // MV=3 cascade spell
    let cascade_def = cascade_sorcery("cascade-mv3", "Cascade Spell MV3", 3);
    // MV=1 sorcery (qualifying)
    let small_def = plain_sorcery("small-spell-mv1", "Small Spell MV1", 1);
    // Land (should be skipped even though it has mana value 0)
    let land_def = basic_land("land-skip-test", "Land Skip Test");

    let cascade_card_id = cascade_def.card_id.clone();
    let small_card_id = small_def.card_id.clone();
    let land_card_id = land_def.card_id.clone();

    let registry = CardRegistry::new(vec![cascade_def, small_def, land_def]);

    // Library: Land on top (pushed last), Small on bottom (pushed first).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Cascade Spell MV3")
                .with_card_id(cascade_card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Cascade)
                .in_zone(ZoneId::Hand(p1)),
        )
        // SmallMV on bottom (pushed first).
        .object(
            ObjectSpec::card(p1, "Small Spell MV1")
                .with_card_id(small_card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        // Land on top (pushed last).
        .object(
            ObjectSpec::card(p1, "Land Skip Test")
                .with_card_id(land_card_id)
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;

    let cascade_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Cascade Spell MV3")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
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
        },
    )
    .unwrap();

    // CR 702.85a: After cast, the cascade trigger is on the stack above the spell.
    // Stack has: [cascade spell, cascade trigger] = 2 entries.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: cascade spell + cascade trigger on stack; got {}",
        state.stack_objects.len()
    );

    // At cast time: no CascadeCast yet (trigger hasn't resolved).
    let cascade_cast_at_cast = cast_events
        .iter()
        .find(|e| matches!(e, GameEvent::CascadeCast { .. }));
    assert!(
        cascade_cast_at_cast.is_none(),
        "No CascadeCast at cast time (trigger hasn't resolved); events: {:?}",
        cast_events
    );

    // Resolve the cascade trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CascadeCast should fire when the trigger resolves (Small Spell found after skipping land).
    let cascade_cast = resolve_events
        .iter()
        .find(|e| matches!(e, GameEvent::CascadeCast { .. }));
    assert!(
        cascade_cast.is_some(),
        "CascadeCast should fire when trigger resolves; events: {:?}",
        resolve_events
    );

    // The land should have been exiled (CascadeExiled fires during trigger resolution).
    let exiled_event = resolve_events
        .iter()
        .find(|e| matches!(e, GameEvent::CascadeExiled { .. }));
    assert!(
        exiled_event.is_some(),
        "CascadeExiled event should fire during trigger resolution; events: {:?}",
        resolve_events
    );

    // After trigger resolves: [cascade spell (bottom), cascaded sorcery (top)] = 2 entries.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After trigger resolves: cascade spell + cascaded sorcery on stack"
    );
}

// ── CC#29: Split card mana value (documented, not tested) ─────────────────────

/// CC#29 / CR 708.4 — Split card mana value during cascade.
///
/// This is a documentation test that verifies the rule is noted in code.
/// Split cards are not yet implemented in the engine (deferred to a future
/// milestone). When implemented, the combined mana value of both halves
/// must be used when checking cascade eligibility in non-stack zones.
///
/// The test simply confirms that cascade works for normal cards. When split
/// cards are added, a proper test for CC#29 should be written.
#[test]
fn test_cascade_combined_mana_value_skip() {
    // CR 708.4: In a non-stack zone, a split card's mana value is the sum of
    // both halves' mana values. Example: Fire // Ice has mana value 4 (2+2).
    //
    // For a cascade spell with MV=4, a split card with combined MV=4 would be
    // SKIPPED (equal, not strictly less than). A split card with combined MV=3
    // would be ELIGIBLE (strictly less than 4).
    //
    // This test documents the behavior; actual split card support is deferred.
    // For now, verify that cascade with a normal non-split card of matching MV
    // correctly skips it.

    let p1 = p1();
    let p2 = p2();

    // MV=4 cascade spell.
    let cascade_def = cascade_sorcery("cascade-mv4", "Cascade MV4 Spell", 4);
    // "Split card" simulated as a single card with combined MV=4 (equal, skip).
    let equal_def = plain_sorcery("split-sim-mv4", "Split Sim MV4", 4);
    // Qualifying card MV=3 (strictly less than 4).
    let qualifying_def = plain_sorcery("qualifying-mv3", "Qualifying MV3", 3);

    let cascade_id = cascade_def.card_id.clone();
    let equal_id = equal_def.card_id.clone();
    let qualifying_id = qualifying_def.card_id.clone();

    let registry = CardRegistry::new(vec![cascade_def, equal_def, qualifying_def]);

    // Library top-to-bottom: SplitSim(4), Qualifying(3).
    // Push order: Qualifying first (bottom), SplitSim last (top).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Cascade MV4 Spell")
                .with_card_id(cascade_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 4,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Cascade)
                .in_zone(ZoneId::Hand(p1)),
        )
        // Qualifying (bottom, pushed first).
        .object(
            ObjectSpec::card(p1, "Qualifying MV3")
                .with_card_id(qualifying_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        // SplitSim (top, pushed last).
        .object(
            ObjectSpec::card(p1, "Split Sim MV4")
                .with_card_id(equal_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 4,
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
    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 4;

    let cascade_hand_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Cascade MV4 Spell")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
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
        },
    )
    .unwrap();

    // CR 702.85a: After cast, cascade trigger is on the stack above the spell.
    // Stack has [cascade spell, cascade trigger] = 2 entries.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: cascade spell + cascade trigger; got {}",
        state.stack_objects.len()
    );

    // No CascadeCast at cast time.
    let cascade_cast_at_cast = cast_events
        .iter()
        .find(|e| matches!(e, GameEvent::CascadeCast { .. }));
    assert!(
        cascade_cast_at_cast.is_none(),
        "No CascadeCast at cast time; events: {:?}",
        cast_events
    );

    // Resolve the cascade trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CascadeCast should fire (found Qualifying MV3 after skipping SplitSim MV4).
    let cascade_cast = resolve_events
        .iter()
        .find(|e| matches!(e, GameEvent::CascadeCast { .. }));
    assert!(
        cascade_cast.is_some(),
        "CascadeCast should fire when trigger resolves; events: {:?}",
        resolve_events
    );

    // Both cascade spell and cascaded spell are on stack.
    assert!(
        state.stack_objects.len() >= 2,
        "Cascade spell + cascaded spell should be on stack after trigger resolves"
    );

    // CR 708.4 note: when split cards are implemented, add a test here that
    // verifies a split card with combined MV=4 is skipped (not < 4) and a
    // split card with combined MV=3 is cast by cascade.
}
