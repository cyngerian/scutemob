//! Buyback keyword ability tests (CR 702.27).
//!
//! Buyback is a static ability that functions while the spell is on the stack.
//! "Buyback [cost]" means "You may pay an additional [cost] as you cast this
//! spell" and "If the buyback cost was paid, put this spell into its owner's
//! hand instead of into that player's graveyard as it resolves."
//! (CR 702.27a)
//!
//! Key rules verified:
//! - Buyback is an optional additional cost (CR 702.27a).
//! - If paid and spell resolves, card returns to hand (not graveyard) (CR 702.27a).
//! - If NOT paid, spell goes to graveyard normally (CR 702.27a).
//! - Countered buyback spell goes to graveyard, not hand (CR 701.6a).
//! - Buyback adds its cost to the total (CR 601.2f, CR 118.8).
//! - Insufficient mana for buyback is rejected (CR 601.2f-h).
//! - Spell without buyback rejects cast_with_buyback: true (engine validation).
//! - Flashback exile overrides buyback return-to-hand (CR 702.34a).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec,
    PlayerId, Step, Target, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<mtg_engine::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

/// Pass priority for all listed players once.
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Searing Touch: Instant {R}. Buyback {4}. Deal 1 damage to any target.
///
/// This is a synthetic card modelled after actual buyback instants.
/// Base cost: {R}. Buyback cost: {4}. Total with buyback: {4}{R}.
fn searing_touch_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("searing-touch".to_string()),
        name: "Searing Touch".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Buyback {4} (You may pay an additional {4} as you cast this spell. If you do, put this card into your hand as it resolves.)\nSearing Touch deals 1 damage to any target."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Buyback),
            AbilityDefinition::Buyback {
                cost: ManaCost {
                    generic: 4,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Lightning Bolt: Instant {R}. Deal 3 damage to any target. No buyback.
fn lightning_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Counterspell: Instant {U}{U}. Counter target spell.
fn counterspell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("counterspell".to_string()),
        name: "Counterspell".to_string(),
        mana_cost: Some(ManaCost {
            blue: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Counter target spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Flashback Buyback: Instant {R}. Flashback {2}{R}. Buyback {4}. Draw a card.
///
/// Synthetic card for testing the flashback-beats-buyback interaction.
/// No such printed card exists. Used to verify CR 702.34a overrides CR 702.27a.
fn flashback_buyback_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("flashback-buyback".to_string()),
        name: "Flashback Buyback".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Flashback {2}{R}. Buyback {4}. (Synthetic test card — no printed equivalent.)"
                .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility { kind: AltCostKind::Flashback, details: None,
                cost: ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Buyback),
            AbilityDefinition::Buyback {
                cost: ManaCost {
                    generic: 4,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

// ── Test 1: Basic buyback — paid, spell resolves → card returns to hand ───────

/// CR 702.27a — If the buyback cost was paid, the card goes to its owner's hand
/// instead of the graveyard as the spell resolves.
#[test]
fn test_buyback_basic_return_to_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def()]);

    let spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {4}{R} — base {R} + buyback {4}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let initial_p2_life = state.players[&p2].life_total;
    let spell_id = find_object(&state, "Searing Touch");

    // Cast Searing Touch with buyback, targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with buyback failed: {:?}", e));

    // `was_buyback_paid` should be true on the stack object.
    assert!(
        state.stack_objects[0].was_buyback_paid,
        "CR 702.27a: was_buyback_paid should be true on stack object"
    );

    // Mana pool should be empty — {4}{R} consumed.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.27a: {{4}}{{R}} total cost should be deducted from mana pool"
    );

    // Both players pass priority — spell resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have taken 1 damage.
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life - 1,
        "CR 702.27a: Searing Touch should deal 1 damage to p2"
    );

    // Searing Touch should be back in p1's hand (NOT graveyard).
    let in_hand = find_object_in_zone(&state, "Searing Touch", ZoneId::Hand(p1));
    assert!(
        in_hand.is_some(),
        "CR 702.27a: buyback spell should return to owner's hand after resolving"
    );

    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Searing Touch" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_graveyard,
        "CR 702.27a: buyback spell must NOT be in graveyard when buyback was paid"
    );
}

// ── Test 2: Buyback NOT paid — spell goes to graveyard normally ───────────────

/// CR 702.27a — When buyback is NOT paid, the spell goes to its owner's graveyard
/// on resolution (same as any ordinary instant/sorcery).
#[test]
fn test_buyback_not_paid_goes_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def()]);

    let spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay only {R} — no buyback.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Searing Touch");

    // Cast WITHOUT buyback.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell without buyback failed: {:?}", e));

    // `was_buyback_paid` should be false on the stack object.
    assert!(
        !state.stack_objects[0].was_buyback_paid,
        "CR 702.27a: was_buyback_paid should be false when not paying buyback"
    );

    // Both players pass priority — spell resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Searing Touch should be in p1's graveyard (NOT hand).
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Searing Touch" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        in_graveyard,
        "CR 702.27a: spell without buyback paid should go to graveyard on resolution"
    );

    let in_hand = find_object_in_zone(&state, "Searing Touch", ZoneId::Hand(p1));
    assert!(
        in_hand.is_none(),
        "CR 702.27a: spell without buyback paid must NOT return to hand"
    );
}

// ── Test 3: Buyback paid, spell countered → graveyard (not hand) ──────────────

/// CR 701.6a — "A countered spell is put into its owner's graveyard."
/// Buyback only applies "as it resolves" — a countered spell does not resolve,
/// so buyback doesn't trigger. The card goes to graveyard regardless of whether
/// the buyback cost was paid.
#[test]
fn test_buyback_paid_spell_countered_goes_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def(), counterspell_def()]);

    let buyback_spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let counter_spell = ObjectSpec::card(p2, "Counterspell")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("counterspell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            blue: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(buyback_spell)
        .object(counter_spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {4}{R} for buyback cast; p2 has {U}{U} for counterspell.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let buyback_id = find_object(&state, "Searing Touch");

    // p1 casts Searing Touch with buyback, targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: buyback_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Buyback cast failed: {:?}", e));

    // Find Searing Touch on the stack (as a game object in zone Stack).
    let spell_on_stack = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Searing Touch" && obj.zone == ZoneId::Stack {
                Some(id)
            } else {
                None
            }
        })
        .expect("Searing Touch should be in Stack zone");

    let counter_id = find_object(&state, "Counterspell");

    // p1 passes priority — p2 gets priority.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 casts Counterspell targeting Searing Touch.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: counter_id,
            targets: vec![Target::Object(spell_on_stack)],
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
        },
    )
    .unwrap_or_else(|e| panic!("Counterspell cast failed: {:?}", e));

    // Both players pass — Counterspell resolves (counters Searing Touch), then Counterspell resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2, p1, p2]);

    // SpellCountered event emitted for Searing Touch.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p1)),
        "CR 701.6a: SpellCountered event expected for Searing Touch"
    );

    // Searing Touch should be in p1's GRAVEYARD (not hand), even though buyback was paid.
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Searing Touch" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        in_graveyard,
        "CR 701.6a: countered spell goes to graveyard even if buyback was paid (buyback only applies 'as it resolves')"
    );

    let in_hand = find_object_in_zone(&state, "Searing Touch", ZoneId::Hand(p1));
    assert!(
        in_hand.is_none(),
        "CR 701.6a: countered buyback spell must NOT return to hand"
    );
}

// ── Test 4: Buyback cost adds to total (mana consumption) ─────────────────────

/// CR 601.2f, CR 118.8 — Buyback adds its cost to the total mana cost to be paid.
/// Searing Touch: base {R}, buyback {4} → total {4}{R}.
/// Exactly {4}{R} must be consumed from the mana pool.
#[test]
fn test_buyback_cost_added_to_total() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def()]);

    let spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Exactly {4}{R} — just enough for buyback cast.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Searing Touch");

    // Cast with buyback — should succeed with exactly the right amount of mana.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with exact buyback mana failed: {:?}", e));

    // Mana pool must be empty — all {4}{R} consumed.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 601.2f: {{4}}{{R}} total cost (base {{R}} + buyback {{4}}) should empty the mana pool"
    );
}

// ── Test 5: Insufficient mana for buyback is rejected ─────────────────────────

/// CR 601.2f-h — If the player declares intent to pay buyback but cannot cover
/// the total cost (base + buyback), the cast is rejected with InsufficientMana.
/// Searing Touch: base {R}, buyback {4} → total {4}{R}. Only {R} in pool.
#[test]
fn test_buyback_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def()]);

    let spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only {R} — enough for base cost but NOT for buyback {4}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Searing Touch");

    // Attempt cast with buyback — should fail (not enough mana).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 601.2f-h: should reject buyback cast when mana pool lacks the buyback cost"
    );
}

// ── Test 6: No buyback ability → rejected ─────────────────────────────────────

/// Engine validation — a spell without the Buyback ability should reject
/// cast_with_buyback: true. The engine checks the card definition.
#[test]
fn test_buyback_no_buyback_ability_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("lightning-bolt".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Plenty of mana — only failure should be missing buyback ability.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 6);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt");

    // Attempt to use buyback on a non-buyback spell.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "engine validation: should reject cast_with_buyback: true on a non-buyback spell"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("buyback") || err.contains("InvalidCommand"),
        "engine validation: error should mention buyback, got: {err}"
    );
}

// ── Test 7: Flashback exile overrides buyback return-to-hand ──────────────────

/// CR 702.34a — Flashback says "exile this card instead of putting it anywhere
/// else any time it would leave the stack." This overrides buyback's
/// "put into owner's hand" destination. Flashback wins.
///
/// Note: No printed card has both Buyback and Flashback. This is a defensive
/// test for a hypothetical interaction using a synthetic test card.
#[test]
fn test_buyback_with_flashback_exile_wins() {
    let p1 = p(1);
    let p2 = p(2);

    // Use the synthetic flashback+buyback card.
    let registry = CardRegistry::new(vec![flashback_buyback_def()]);

    // Place it in graveyard — we'll cast it via flashback from there.
    let spell = ObjectSpec::card(p1, "Flashback Buyback")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flashback-buyback".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback)
        .with_keyword(KeywordAbility::Buyback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay flashback cost {2}{R} + buyback {4} = {6}{R} total.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 6);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Flashback Buyback");

    // Cast from graveyard via flashback with buyback also set.
    // The engine detects flashback from the zone (graveyard) + keyword.
    // cast_with_buyback: true should also be set (paying the buyback additional cost).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Flashback + buyback cast failed: {:?}", e));

    // Both players pass — spell resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // The card should be in EXILE (flashback wins), NOT in hand (buyback) or graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Flashback Buyback" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.34a: flashback exile overrides buyback return-to-hand — card should be in exile"
    );

    let in_hand = find_object_in_zone(&state, "Flashback Buyback", ZoneId::Hand(p1));
    assert!(
        in_hand.is_none(),
        "CR 702.34a: buyback must NOT win over flashback — card must not be in hand"
    );

    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Flashback Buyback" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_graveyard,
        "CR 702.34a: flashback card should be in exile, not graveyard"
    );
}

// ── Test 8: Buyback paid, spell fizzles → graveyard (NOT hand) ───────────────

/// CR 608.2b + 702.27a — When a spell fizzles (all targets illegal at resolution),
/// it is removed from the stack without resolving. Buyback only applies "as it resolves"
/// (CR 702.27a), so a fizzled spell does NOT return to the owner's hand — it goes to the
/// graveyard normally, exactly as if buyback had never been paid.
///
/// Setup: P1 casts Searing Touch with buyback targeting a creature. P1 then casts
/// Lightning Bolt to kill that creature. Lightning Bolt resolves first (LIFO), the
/// creature dies, then Searing Touch fizzles because its only target is gone.
/// Searing Touch must end up in P1's graveyard, not hand.
#[test]
fn test_buyback_paid_spell_fizzles_goes_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def(), lightning_bolt_def()]);

    // A 1/1 creature on the battlefield owned by p2.
    let creature = ObjectSpec::creature(p2, "Goblin Token", 1, 1);

    let buyback_spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let bolt_spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("lightning-bolt".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .object(buyback_spell)
        .object(bolt_spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P1 needs {4}{R} for Searing Touch + buyback, plus {R} for Lightning Bolt = {4}{R}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    // Find the creature on the battlefield.
    let creature_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Goblin Token" && obj.zone == ZoneId::Battlefield {
                Some(id)
            } else {
                None
            }
        })
        .expect("Goblin Token should be on battlefield");

    let buyback_id = find_object(&state, "Searing Touch");
    let bolt_id = find_object(&state, "Lightning Bolt");

    // P1 casts Searing Touch with buyback, targeting the creature.
    // Stack: [Searing Touch]
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: buyback_id,
            targets: vec![Target::Object(creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell Searing Touch with buyback failed: {:?}", e));

    assert!(
        state.stack_objects[0].was_buyback_paid,
        "CR 702.27a: was_buyback_paid should be true on stack object"
    );

    // P1 casts Lightning Bolt targeting the same creature (still on battlefield).
    // Stack: [Lightning Bolt, Searing Touch] (LB on top — resolves first)
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bolt_id,
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell Lightning Bolt failed: {:?}", e));

    // Both players pass priority → Lightning Bolt resolves (3 damage to 1/1 → creature dies).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature should be dead (in graveyard or no longer on battlefield).
    let creature_on_bf = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Goblin Token" && o.zone == ZoneId::Battlefield);
    assert!(
        !creature_on_bf,
        "Goblin Token should have died from Lightning Bolt before Searing Touch resolves"
    );

    // Both players pass priority again → Searing Touch tries to resolve.
    // Its target (the creature) is gone — all targets illegal → fizzle.
    let (state, fizzle_events) = pass_all(state, &[p1, p2]);

    // SpellFizzled event should have been emitted.
    assert!(
        fizzle_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { player, .. } if *player == p1)),
        "CR 608.2b: SpellFizzled event expected when all targets are illegal"
    );

    // Searing Touch must be in P1's GRAVEYARD — NOT hand.
    // CR 702.27a: buyback only returns the card "as it resolves"; a fizzled spell
    // does not resolve, so buyback does not apply.
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Searing Touch" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        in_graveyard,
        "CR 608.2b + 702.27a: fizzled buyback spell should go to graveyard, not hand"
    );

    let in_hand = find_object_in_zone(&state, "Searing Touch", ZoneId::Hand(p1));
    assert!(
        in_hand.is_none(),
        "CR 702.27a: buyback does NOT apply when spell fizzles — must NOT be in hand"
    );
}

// ── Test 9: SpellCast event is emitted for buyback cast ───────────────────────

/// CR 702.27a — Casting a spell with buyback emits the standard SpellCast event.
/// The buyback cost does not suppress or replace the normal casting event.
#[test]
fn test_buyback_spell_cast_event_emitted() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![searing_touch_def()]);

    let spell = ObjectSpec::card(p1, "Searing Touch")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("searing-touch".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Buyback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
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
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Searing Touch");

    let (_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Buyback),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell buyback failed: {:?}", e));

    // SpellCast event should be emitted.
    let spell_cast_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1));
    assert!(
        spell_cast_event,
        "CR 702.27a: SpellCast event should be emitted for a buyback spell cast"
    );
}
