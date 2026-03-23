//! Ward keyword ability tests (CR 702.21).
//!
//! Ward is a triggered ability: "Whenever this permanent becomes the target of a
//! spell or ability an opponent controls, counter that spell or ability unless
//! that player pays [cost]."
//!
//! In the deterministic engine, `MayPayOrElse` always applies the `or_else` branch
//! (interactive payment is deferred to M10+). This means ward ALWAYS counters the
//! targeting spell or ability in these tests, which is the correct behavior for a
//! non-interactive engine.

use mtg_engine::cards::card_definition::ForEachTarget;
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec,
    PlayerId, Step, Target, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

/// A spell that targets a creature (e.g., simplified Doom Blade).
fn targeting_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("doom-blade".to_string()),
        name: "Doom Blade".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "can't be countered" version of the targeting spell.
fn uncounterable_targeting_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("raze-to-the-ground".to_string()),
        name: "Raze to the Ground".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "This spell can't be countered. Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: true,
        }],
        ..Default::default()
    }
}

/// A non-targeting "destroy all creatures" spell (no ward trigger).
fn wrath_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("wrath-of-god".to_string()),
        name: "Wrath of God".to_string(),
        mana_cost: Some(ManaCost {
            white: 2,
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Destroy all creatures.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // ForEach all creatures on the battlefield — same pattern as the real Wrath of God.
            effect: Effect::ForEach {
                over: ForEachTarget::EachCreature,
                effect: Box::new(Effect::DestroyPermanent {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                }),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Ward basic counter ────────────────────────────────────────────────

#[test]
/// CR 702.21a — Ward {2}: when an opponent casts a spell targeting this permanent,
/// ward triggers and counters the spell (deterministic: always counters).
fn test_ward_basic_counter_on_targeting() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![targeting_spell_def()]);

    // p1 has a creature with Ward {2} on the battlefield.
    let ward_creature =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p2 has Doom Blade in hand.
    let spell = ObjectSpec::card(p2, "Doom Blade")
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(ward_creature)
        .object(spell)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p2 mana to cast Doom Blade {1B}.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p2);

    let creature_id = find_object(&state, "Ward Creature");
    let spell_id = find_object(&state, "Doom Blade");

    // p2 casts Doom Blade targeting the ward creature.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p2)),
        "SpellCast event expected"
    );

    // Ward trigger fires — PermanentTargeted event emitted.
    assert!(
        cast_events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTargeted { target_id, targeting_controller, .. }
            if *target_id == creature_id && *targeting_controller == p2
        )),
        "PermanentTargeted event expected for ward trigger"
    );

    // Stack: Doom Blade + Ward trigger (ward trigger goes on top, resolves first).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "stack should have Doom Blade + ward trigger"
    );

    // Both players pass priority. The active player (p2) has priority first after the cast.
    // After p2 and p1 both pass, the ward trigger (top of stack) resolves.
    let (state, resolve_events) = pass_all(state, &[p2, p1]);

    // Ward resolution: MayPayOrElse always fires or_else = CounterSpell.
    // Doom Blade should be countered. SpellCountered event emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 702.21a: ward should counter Doom Blade (deterministic: always counters)"
    );

    // Ward creature survives (was not destroyed).
    assert!(
        state.objects.values().any(|o| {
            o.characteristics.name == "Ward Creature" && o.zone == ZoneId::Battlefield
        }),
        "Ward creature should still be on the battlefield"
    );

    // Doom Blade is in graveyard (countered, moved there).
    assert!(
        state.objects.values().any(|o| {
            o.characteristics.name == "Doom Blade" && matches!(o.zone, ZoneId::Graveyard(_))
        }),
        "Doom Blade should be in graveyard after being countered"
    );
}

// ── Test 2: Ward does not trigger for controller ───────────────────────────────

#[test]
/// CR 702.21a — Ward only triggers for spells/abilities controlled by opponents.
/// The permanent's controller targeting their own ward creature does NOT trigger ward.
fn test_ward_does_not_trigger_for_controller() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![targeting_spell_def()]);

    // p1 has a ward creature on the battlefield.
    let ward_creature =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p1 also has Doom Blade in hand (controller targeting own creature).
    let spell = ObjectSpec::card(p1, "Doom Blade")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let creature_id = find_object(&state, "Ward Creature");
    let spell_id = find_object(&state, "Doom Blade");

    // p1 casts Doom Blade targeting their own ward creature.
    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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

    // PermanentTargeted IS emitted (it signals that a permanent was targeted),
    // but since the targeting controller is the permanent's own controller,
    // check_triggers should NOT queue a ward trigger.
    // Verify this by checking the stack — only Doom Blade, no ward trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.21a: only Doom Blade should be on stack — ward does NOT trigger for controller targeting own permanent"
    );

    // No ward trigger in pending_triggers either.
    assert!(
        state.pending_triggers.is_empty(),
        "CR 702.21a: no pending ward trigger when controller targets own permanent"
    );
}

// ── Test 3: Ward does not trigger for non-targeting spell ─────────────────────

#[test]
/// CR 702.21a — Ward only triggers when the permanent BECOMES a target. Non-targeting
/// spells (e.g., "destroy all creatures") do NOT trigger ward.
fn test_ward_does_not_trigger_for_non_targeting_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![wrath_def()]);

    // p1 has a ward creature on the battlefield.
    let ward_creature =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p2 has Wrath of God in hand (no targeting).
    // `.with_card_id` is required so resolution.rs can look up the Spell effect
    // from the registry (without it, card_id is None and the effect is skipped).
    let spell = ObjectSpec::card(p2, "Wrath of God")
        .with_card_id(CardId("wrath-of-god".to_string()))
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            white: 2,
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward_creature)
        .object(spell)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 2);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p2);

    let spell_id = find_object(&state, "Wrath of God");

    // p2 casts Wrath of God (no targets).
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
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

    // No PermanentTargeted event (non-targeting spell).
    assert!(
        !cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTargeted { .. })),
        "CR 702.21a: ward should NOT trigger for non-targeting spells"
    );

    // Stack has only Wrath (no ward trigger).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "only Wrath of God should be on stack (no ward trigger)"
    );

    // After Wrath resolves, creature is destroyed (ward didn't save it).
    // Active player is p2 — they have priority after casting.
    let (state, _) = pass_all(state, &[p2, p1]); // Wrath resolves

    assert!(
        !state.objects.values().any(|o| {
            o.characteristics.name == "Ward Creature" && o.zone == ZoneId::Battlefield
        }),
        "Ward Creature should be destroyed by Wrath of God (ward did not trigger)"
    );
}

// ── Test 4: Ward triggers for activated ability targeting ──────────────────────

#[test]
/// CR 702.21a — Ward triggers for abilities too, not just spells.
/// An activated ability controlled by an opponent that targets the ward permanent
/// should trigger ward.
fn test_ward_triggers_for_activated_ability_targeting() {
    use mtg_engine::state::{ActivatedAbility, ActivationCost, StackObjectKind};
    use mtg_engine::{CardEffectTarget, Effect};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has a Ward {2} creature on the battlefield.
    let ward_creature =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p2 has a creature with an activated ability: "{T}: Destroy target creature."
    let ability = ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
        },
        description: "{T}: Destroy target creature".to_string(),
        effect: Some(Effect::DestroyPermanent {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        }),
        sorcery_speed: false,
        activation_condition: None,
    };
    let ability_creature =
        ObjectSpec::creature(p2, "Assassin", 1, 1).with_activated_ability(ability);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ward_creature)
        .object(ability_creature)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let ward_id = find_object(&state, "Ward Creature");
    let assassin_id = find_object(&state, "Assassin");

    // p2 activates their ability targeting the ward creature.
    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: assassin_id,
            ability_index: 0,
            targets: vec![Target::Object(ward_id)],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .unwrap();

    // PermanentTargeted event should be emitted.
    assert!(
        activate_events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTargeted { target_id, targeting_controller, .. }
            if *target_id == ward_id && *targeting_controller == p2
        )),
        "CR 702.21a: PermanentTargeted event expected for ward trigger (ability targeting)"
    );

    // Stack: activated ability + ward trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "stack should have activated ability + ward trigger"
    );

    // Ward trigger is the top entry (it goes on after the ability, resolves first).
    let top_kind = &state.stack_objects.back().unwrap().kind;
    assert!(
        matches!(top_kind, StackObjectKind::TriggeredAbility { .. }),
        "ward trigger should be on top of the stack (resolves first)"
    );
}

// ── Test 5: Ward + can't-be-countered spell ────────────────────────────────────

#[test]
/// CR 702.21a + rulings (Raze to the Ground): Ward still triggers even for
/// "can't be countered" spells. Ward resolves and tries to counter, but the
/// spell's cant_be_countered flag prevents it. The spell resolves normally.
///
/// In the deterministic engine, MayPayOrElse always fires the or_else (CounterSpell),
/// but CounterSpell checks cant_be_countered and skips. The creature is still destroyed.
fn test_ward_cant_be_countered_spell_resolves_normally() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uncounterable_targeting_spell_def()]);

    // p1 has a ward creature on the battlefield.
    let ward_creature =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p2 has the can't-be-countered targeting spell.
    // `.with_card_id` is required so casting.rs can read cant_be_countered from the registry.
    let spell = ObjectSpec::card(p2, "Raze to the Ground")
        .with_card_id(CardId("raze-to-the-ground".to_string()))
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward_creature)
        .object(spell)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p2);

    let creature_id = find_object(&state, "Ward Creature");
    let spell_id = find_object(&state, "Raze to the Ground");

    // p2 casts the cant-be-countered spell targeting the ward creature.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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

    // Ward trigger fires.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTargeted { .. })),
        "CR 702.21a: ward should still trigger for cant-be-countered spells"
    );

    // Both spell and ward trigger on stack.
    assert_eq!(state.stack_objects.len(), 2);

    // Both players pass — ward trigger resolves first (it's on top).
    // Active player is p2 — they have priority first.
    let (state, resolve_events) = pass_all(state, &[p2, p1]);

    // Ward tried to counter but the spell can't be countered.
    // SpellCountered should NOT be emitted (or the counter attempt had no effect).
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 101.6: cant-be-countered spell should NOT be countered by ward"
    );

    // Ward trigger resolved (AbilityResolved event emitted).
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "ward trigger should still resolve (even though counter had no effect)"
    );

    // Both players pass again — spell resolves. Active player has priority.
    let (state, spell_events) = pass_all(state, &[p2, p1]);

    // Spell resolved and destroyed the creature.
    assert!(
        spell_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { .. })),
        "cant-be-countered spell should resolve normally"
    );

    // Ward creature should be destroyed (spell resolved despite ward trigger).
    assert!(
        !state.objects.values().any(|o| {
            o.characteristics.name == "Ward Creature" && o.zone == ZoneId::Battlefield
        }),
        "CR 101.6 + 702.21a: cant-be-countered spell should destroy creature despite ward"
    );
}

// ── Test 6: Ward — multiple targets each trigger separately ──────────────────

#[test]
/// CR 702.21a (Ruling: Adrix and Nev, Purple Worm): If a spell targets multiple
/// permanents that each have ward, each ward triggers separately.
fn test_ward_multiple_targets_trigger_separately() {
    use mtg_engine::{CardEffectTarget, Effect, EffectAmount};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A spell that targets two creatures.
    let two_target_spell = CardDefinition {
        card_id: CardId("twin-bolt".to_string()),
        name: "Twin Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Twin Bolt deals 2 damage to each of up to two target creatures.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
                Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 1 },
                    amount: EffectAmount::Fixed(2),
                },
            ]),
            targets: vec![
                TargetRequirement::TargetCreature,
                TargetRequirement::TargetCreature,
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![two_target_spell]);

    // p1 has TWO ward creatures.
    let ward_a =
        ObjectSpec::creature(p1, "Ward Creature A", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let ward_b =
        ObjectSpec::creature(p1, "Ward Creature B", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p2 casts the two-target spell.
    let spell = ObjectSpec::card(p2, "Twin Bolt")
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward_a)
        .object(ward_b)
        .object(spell)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p2);

    let a_id = find_object(&state, "Ward Creature A");
    let b_id = find_object(&state, "Ward Creature B");
    let spell_id = find_object(&state, "Twin Bolt");

    // p2 casts Twin Bolt targeting both ward creatures.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(a_id), Target::Object(b_id)],
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

    // Two PermanentTargeted events — one for each ward creature.
    let targeted_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTargeted { .. }))
        .count();
    assert_eq!(
        targeted_count, 2,
        "CR 702.21a: two PermanentTargeted events expected (one per ward creature)"
    );

    // Stack: spell + two ward triggers.
    assert_eq!(
        state.stack_objects.len(),
        3,
        "stack should have the spell + 2 ward triggers (one per ward creature)"
    );
}

// ── Test 7: Ward — multiplayer opponent check ─────────────────────────────────

#[test]
/// CR 702.21a — In multiplayer (Commander), "an opponent" means any player other
/// than the permanent's controller. Ward triggers for any opponent's spell.
fn test_ward_multiplayer_opponent_check() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let registry = CardRegistry::new(vec![targeting_spell_def()]);

    // p1 has a ward creature.
    let ward_creature =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p3 (an opponent of p1) has Doom Blade.
    let spell = ObjectSpec::card(p3, "Doom Blade")
        .in_zone(ZoneId::Hand(p3))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(ward_creature)
        .object(spell)
        .active_player(p3)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p3)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p3)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p3);

    let creature_id = find_object(&state, "Ward Creature");
    let spell_id = find_object(&state, "Doom Blade");

    // p3 (opponent of p1) casts Doom Blade targeting p1's ward creature.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p3,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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

    // Ward triggers because p3 is an opponent of p1.
    assert!(
        cast_events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTargeted { target_id, targeting_controller, .. }
            if *target_id == creature_id && *targeting_controller == p3
        )),
        "CR 702.21a: ward should trigger when any opponent (p3) targets the ward permanent"
    );

    // Stack: spell + ward trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "spell + ward trigger should both be on stack"
    );
}
