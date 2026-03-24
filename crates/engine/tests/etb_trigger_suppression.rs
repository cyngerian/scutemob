//! Tests for ETB trigger suppression: Layer 6 ability removal (IG-1) and
//! Torpor Orb-style static suppression (IG-2).
//!
//! # IG-1: Layer 6 ability removal suppresses ETB triggers (CR 603.2, 613)
//!
//! When a continuous effect applies `RemoveAllAbilities` (Layer 6) to a permanent
//! that is entering the battlefield, that permanent's ETB triggered abilities from
//! its CardDefinition must NOT be queued. The "Dress Down + Evoked Solitude" ruling
//! is the canonical example.
//!
//! # IG-2: Torpor Orb-style static suppression (CR 614.16a)
//!
//! A permanent with `AbilityDefinition::SuppressCreatureETBTriggers` prevents
//! creatures entering the battlefield from having their ETB triggered abilities
//! queued. This is a replacement effect — the trigger never fires, rather than
//! being countered after firing. Non-creature permanents are unaffected when
//! `ETBSuppressFilter::CreaturesOnly` is used.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    ContinuousEffect, ETBSuppressFilter, Effect, EffectAmount, EffectDuration, EffectFilter,
    EffectId, EffectLayer, GameStateBuilder, LayerModification, ManaCost, ObjectSpec, PlayerId,
    PlayerTarget, Step, TriggerCondition, TypeLine, ZoneId,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// A creature with an ETB "draw a card" triggered ability.
fn etb_draw_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("etb-draw-creature".to_string()),
        name: "ETB Draw Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature enters, draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
        }],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}

/// A mock "Torpor Orb" creature — its static ability suppresses ETB triggers on
/// creatures entering the battlefield.
fn torpor_orb_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("torpor-orb".to_string()),
        name: "Torpor Orb".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Creatures entering the battlefield don't cause abilities to trigger."
            .to_string(),
        abilities: vec![AbilityDefinition::SuppressCreatureETBTriggers {
            filter: ETBSuppressFilter::CreaturesOnly,
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
    }
}

/// An enchantment with an ETB "gain 3 life" triggered ability (non-creature, used to test
/// that IG-2's `CreaturesOnly` filter does NOT suppress non-creature ETB triggers).
fn etb_gain_life_enchantment_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("etb-gain-life-enchantment".to_string()),
        name: "ETB Gain Life Enchantment".to_string(),
        mana_cost: Some(ManaCost {
            white: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this enchantment enters, gain 3 life.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            },
            intervening_if: None,
            targets: vec![],
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
    }
}

// ── IG-1 Tests ───────────────────────────────────────────────────────────────

/// CR 603.2, 613 Layer 6 — Dress Down pattern (IG-1, positive case):
///
/// When a continuous effect with `RemoveAllAbilities` (Layer 6) applies to a creature
/// entering the battlefield, the creature's ETB triggered ability should NOT be queued.
/// Verified by checking that `state.pending_triggers` is empty after the creature enters.
#[test]
fn test_ig1_layer6_remove_all_abilities_suppresses_etb_trigger() {
    let registry = CardRegistry::new(vec![etb_draw_creature_def()]);

    let creature = ObjectSpec::card(p1(), "ETB Draw Creature")
        .with_card_id(CardId("etb-draw-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    // The RemoveAllAbilities effect has no source (simulates an always-active "Dress Down"
    // with WhileSourceOnBattlefield from a source that doesn't exist in this test state,
    // so we use duration Indefinite to keep it always-active).
    let remove_all_eff = ContinuousEffect {
        id: EffectId(999),
        source: None,
        timestamp: 1000,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::RemoveAllAbilities,
        is_cda: false,
        condition: None,
    };

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .with_registry(registry)
        .object(creature)
        .add_continuous_effect(remove_all_eff)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "ETB Draw Creature" && obj.zone == ZoneId::Hand(p1())
        })
        .map(|(id, _)| *id)
        .expect("ETB Draw Creature not found in hand");

    // Add mana and cast the creature.
    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: card_id,
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

    // Pass priority for both players to resolve the spell from the stack.
    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // The creature has now entered the battlefield. Because RemoveAllAbilities was active,
    // the ETB trigger should NOT have been queued (IG-1 fix).
    assert!(
        state.pending_triggers.is_empty(),
        "IG-1: ETB draw trigger should be suppressed by RemoveAllAbilities (Layer 6). \
         Got {} pending triggers.",
        state.pending_triggers.len()
    );

    // The creature should be on the battlefield.
    let on_bf = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "ETB Draw Creature" && o.zone == ZoneId::Battlefield);
    assert!(on_bf, "ETB Draw Creature should be on the battlefield");
}

/// CR 603.2, 613 Layer 6 — Negative case (IG-1):
///
/// Without any `RemoveAllAbilities` effect, the ETB triggered ability should fire
/// normally. This validates the positive path is untouched by the IG-1 fix.
#[test]
fn test_ig1_without_layer6_effect_etb_trigger_fires_normally() {
    let registry = CardRegistry::new(vec![etb_draw_creature_def()]);

    let creature = ObjectSpec::card(p1(), "ETB Draw Creature")
        .with_card_id(CardId("etb-draw-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    // Library card so DrawCards has something to draw.
    let library_card = ObjectSpec::card(p1(), "Library Card").in_zone(ZoneId::Library(p1()));

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .with_registry(registry)
        .object(creature)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "ETB Draw Creature" && obj.zone == ZoneId::Hand(p1())
        })
        .map(|(id, _)| *id)
        .expect("ETB Draw Creature not found in hand");

    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: card_id,
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

    // Pass priority to resolve the spell.
    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // The creature entered the battlefield. Without RemoveAllAbilities, the ETB trigger
    // should have been queued (it will appear as a pending trigger or on the stack).
    let has_pending_or_stack = !state.pending_triggers.is_empty()
        || state
            .stack_objects
            .iter()
            .any(|s| matches!(s.kind, mtg_engine::StackObjectKind::TriggeredAbility { .. }));

    assert!(
        has_pending_or_stack,
        "IG-1 negative case: ETB draw trigger should fire when no RemoveAllAbilities is active"
    );
}

// ── IG-2 Tests ───────────────────────────────────────────────────────────────

/// CR 614.16a — Torpor Orb pattern (IG-2, positive case):
///
/// When a permanent with `SuppressCreatureETBTriggers` is on the battlefield, a
/// creature entering should NOT have its ETB triggered ability queued.
///
/// Note: `register_static_continuous_effects` is called when a permanent resolves from
/// the stack (not when built directly via builder). So we cast the Torpor Orb first to
/// ensure its suppressor is registered in `state.etb_suppressors`.
#[test]
fn test_ig2_torpor_orb_suppresses_creature_etb() {
    let registry = CardRegistry::new(vec![torpor_orb_def(), etb_draw_creature_def()]);

    let torpor_in_hand = ObjectSpec::card(p1(), "Torpor Orb")
        .with_card_id(CardId("torpor-orb".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    let creature = ObjectSpec::card(p1(), "ETB Draw Creature")
        .with_card_id(CardId("etb-draw-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .with_registry(registry)
        .object(torpor_in_hand)
        .object(creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    // Step 1: Cast Torpor Orb from hand so register_static_continuous_effects runs on entry.
    let orb_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Torpor Orb" && o.zone == ZoneId::Hand(p1()))
        .map(|(id, _)| *id)
        .expect("Torpor Orb not in hand");

    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: orb_id,
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

    // Resolve Torpor Orb (both players pass priority).
    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // Torpor Orb is on battlefield — etb_suppressors should now be populated.
    assert!(
        !state.etb_suppressors.is_empty(),
        "ETB suppressors should be registered after Torpor Orb enters the battlefield"
    );

    // Step 2: Cast ETB Draw Creature.
    let creature_id = state
        .objects
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "ETB Draw Creature" && o.zone == ZoneId::Hand(p1())
        })
        .map(|(id, _)| *id)
        .expect("ETB Draw Creature not in hand");

    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: creature_id,
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

    // Resolve the creature (both players pass).
    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // Creature is on battlefield. The ETB draw trigger should NOT have been queued.
    assert!(
        state.pending_triggers.is_empty(),
        "IG-2: Torpor Orb should suppress creature ETB trigger. Got {} pending triggers.",
        state.pending_triggers.len()
    );

    let creature_on_bf = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "ETB Draw Creature" && o.zone == ZoneId::Battlefield);
    assert!(
        creature_on_bf,
        "ETB Draw Creature should be on the battlefield"
    );
}

/// CR 614.16a — Non-creature permanents are NOT suppressed by Torpor Orb (IG-2):
///
/// The `ETBSuppressFilter::CreaturesOnly` filter must leave non-creature ETBs unaffected.
/// An enchantment with an ETB trigger should still fire when Torpor Orb is on the battlefield.
#[test]
fn test_ig2_torpor_orb_does_not_suppress_non_creature_etb() {
    let registry = CardRegistry::new(vec![torpor_orb_def(), etb_gain_life_enchantment_def()]);

    // Torpor Orb on battlefield (place directly and register manually).
    // We use the cast path to ensure register_static_continuous_effects is called.
    let torpor_in_hand = ObjectSpec::card(p1(), "Torpor Orb")
        .with_card_id(CardId("torpor-orb".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    let enchantment = ObjectSpec::card(p1(), "ETB Gain Life Enchantment")
        .with_card_id(CardId("etb-gain-life-enchantment".to_string()))
        .with_types(vec![CardType::Enchantment])
        .with_mana_cost(ManaCost {
            white: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .with_registry(registry)
        .object(torpor_in_hand)
        .object(enchantment)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    // Cast Torpor Orb.
    let orb_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Torpor Orb" && o.zone == ZoneId::Hand(p1()))
        .map(|(id, _)| *id)
        .expect("Torpor Orb not in hand");

    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: orb_id,
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

    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // Now cast the enchantment.
    let enc_id = state
        .objects
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "ETB Gain Life Enchantment" && o.zone == ZoneId::Hand(p1())
        })
        .map(|(id, _)| *id)
        .expect("Enchantment not in hand");

    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::White, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: enc_id,
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

    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // The enchantment entered the battlefield. Its ETB trigger should fire normally
    // because Torpor Orb only suppresses creatures (CreaturesOnly filter).
    let has_enchantment_trigger = !state.pending_triggers.is_empty()
        || state
            .stack_objects
            .iter()
            .any(|s| matches!(s.kind, mtg_engine::StackObjectKind::TriggeredAbility { .. }));

    assert!(
        has_enchantment_trigger,
        "IG-2: Torpor Orb (CreaturesOnly) should NOT suppress non-creature ETB triggers. \
         Enchantment's ETB gain-life trigger should have fired."
    );
}

/// CR 614.16a — Torpor Orb leaving the battlefield restores ETB triggers (IG-2):
///
/// After the Torpor Orb leaves the battlefield, creatures entering should have their
/// ETB triggered abilities fire normally. The lazy cleanup in `queue_carddef_etb_triggers`
/// prunes stale suppressor entries when their source is no longer on the battlefield.
///
/// We verify this by directly manipulating state: cast + resolve Torpor Orb, then
/// mutate the Orb's zone to simulate it being destroyed, then cast the ETB creature.
#[test]
fn test_ig2_removing_torpor_orb_restores_etb_triggers() {
    let registry = CardRegistry::new(vec![torpor_orb_def(), etb_draw_creature_def()]);

    let torpor_in_hand = ObjectSpec::card(p1(), "Torpor Orb")
        .with_card_id(CardId("torpor-orb".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    let creature1 = ObjectSpec::card(p1(), "ETB Draw Creature")
        .with_card_id(CardId("etb-draw-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1()));

    // One library card so DrawCards has something to draw once ETB fires.
    let library_card = ObjectSpec::card(p1(), "Library Card").in_zone(ZoneId::Library(p1()));

    let state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .with_registry(registry)
        .object(torpor_in_hand)
        .object(creature1)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .build()
        .unwrap();

    // Step 1: Cast and resolve Torpor Orb.
    let orb_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Torpor Orb" && o.zone == ZoneId::Hand(p1()))
        .map(|(id, _)| *id)
        .expect("Torpor Orb not in hand");

    let mut state = state;
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: orb_id,
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

    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // Torpor Orb is on battlefield — suppressor registered.
    assert!(
        !state.etb_suppressors.is_empty(),
        "Suppressor should be registered"
    );

    // Step 2: Simulate the Orb being destroyed by directly moving its zone in state.
    // This replicates the "source no longer on battlefield" condition that the lazy
    // cleanup checks. We move the orb object directly in the im-rs OrdMap.
    let orb_bf_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Torpor Orb" && o.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .expect("Torpor Orb not on battlefield after resolution");

    let mut state = state;
    // Simulate the Orb being destroyed by moving its zone field off-battlefield.
    // The etb_suppressors entry remains (lazy cleanup), but the retain check in
    // queue_carddef_etb_triggers looks at `obj.zone == ZoneId::Battlefield`, so
    // changing obj.zone is sufficient to trigger lazy pruning.
    if let Some(obj) = state.objects.get_mut(&orb_bf_id) {
        obj.zone = ZoneId::Graveyard(p1());
    }

    // The suppressor entry still exists, but its source is off-battlefield.
    // The lazy cleanup in queue_carddef_etb_triggers will prune it on the next ETB.

    // Step 3: Cast ETB Draw Creature — its ETB should now fire normally.
    let creature_id = state
        .objects
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "ETB Draw Creature" && o.zone == ZoneId::Hand(p1())
        })
        .map(|(id, _)| *id)
        .expect("ETB Draw Creature not in hand");

    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1())
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: creature_id,
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

    let (state, _) = process_command(state, Command::PassPriority { player: p1() }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2() }).unwrap();

    // The ETB trigger should now fire (Torpor Orb is gone, lazy cleanup removed the suppressor).
    let has_trigger = !state.pending_triggers.is_empty()
        || state
            .stack_objects
            .iter()
            .any(|s| matches!(s.kind, mtg_engine::StackObjectKind::TriggeredAbility { .. }));

    assert!(
        has_trigger,
        "IG-2: After Torpor Orb leaves, creature ETB trigger should fire. \
         Got {} pending triggers, {} stack objects.",
        state.pending_triggers.len(),
        state.stack_objects.len()
    );
}
