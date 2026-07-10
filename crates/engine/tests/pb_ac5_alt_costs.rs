//! PB-AC5 — Alt-costs & timing keywords: Warp (CR 702.185), Transmute (CR 702.53),
//! Exert (CR 701.43), Pitch (CR 118.9).
//!
//! Key rules verified:
//! - Warp: alternative cost from hand; delayed exile trigger at next end step targets
//!   "the permanent this spell becomes" (never the spell); recast gated to a strictly
//!   later turn; graveyard-cast permission is per-card (Timeline Culler).
//! - Transmute: activated ability functioning only from hand; discards itself; searches
//!   for a card of equal mana value; sorcery-speed only.
//! - Exert: keyword action usable as an optional attack cost (508.1g) and as a plain
//!   activation cost; one-shot "doesn't untap next untap step" that expires regardless
//!   of how many times it was set before the next untap step (701.43a/b); can't exert
//!   off the battlefield (701.43c).
//! - Pitch: alternative cost exiling a card of a required color from hand (+ optional
//!   life payment); only one alternative cost per spell (118.9a); doesn't change the
//!   spell's mana value (118.9c).

use mtg_engine::cards::card_definition::{ActivationZone, AltCastDetails, Cost};
use mtg_engine::cards::helpers::mana_pool;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::PlayerId;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CardType, Color, Command, Designations, Effect, EffectAmount, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerTarget,
    Step, TargetRequirement, TypeLine, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Hand(owner)).is_some()
}

/// Pass priority for all listed players once.
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

fn empty_cast_spell(player: PlayerId, card: ObjectId, alt_cost: Option<AltCostKind>) -> Command {
    Command::CastSpell {
        player,
        card,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

// ── Card definitions (mocks, unless noted "real card") ────────────────────────

/// Warp Test Creature: {B}{B}, 2/2. Haste. Warp—{B}, Pay 2 life. Castable from
/// graveyard via warp (mirrors the real Timeline Culler; kept as a local mock so
/// warp mechanism tests are independent of card-def churn).
fn warp_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("warp-test-creature".to_string()),
        name: "Warp Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Warp),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Warp,
                cost: ManaCost {
                    black: 1,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Warp {
                    costs: vec![Cost::PayLife(2)],
                    from_graveyard: true,
                }),
            },
        ],
        ..Default::default()
    }
}

/// A trivial counter spell used to test that a countered warp spell is never exiled
/// by the warp delayed trigger (it never becomes a permanent — CR 400.7 / 702.185a).
fn mock_counter_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-counter".to_string()),
        name: "Mock Counter".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                exile_instead: false,
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A trivial targeted spell used as a legal counter/target for Force of Will / Force
/// of Negation pitch tests (both have a mandatory single spell/noncreature-spell target).
fn mock_threat_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-threat-spell".to_string()),
        name: "Mock Threat Spell".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A mock card with BOTH Warp and Flashback so the mutual-exclusion check
/// (CR 118.9a) can be exercised: Flashback is auto-detected from graveyard + keyword,
/// independent of the `alt_cost` parameter.
fn warp_and_flashback_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("warp-flashback-hybrid".to_string()),
        name: "Warp Flashback Hybrid".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Warp),
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Warp,
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Warp {
                    costs: vec![],
                    from_graveyard: true,
                }),
            },
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Flashback,
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                details: None,
            },
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
        ..Default::default()
    }
}

/// A vanilla creature used as a MV=2 library find for transmute searches.
fn mv2_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mv2-quarry".to_string()),
        name: "MV2 Quarry".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// A MV=1 creature used to prove transmute doesn't grab the wrong mana value.
fn mv1_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mv1-decoy".to_string()),
        name: "MV1 Decoy".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Arena of Glory (activation-cost exert shape, standalone mock — mirrors the primitive
/// without the (separately DSL-gapped) mana-spend haste rider): a land with
/// `{T}: Add {R}` and `{R}, {T}, Exert this land: Add {R}{R}`.
fn exert_land_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("exert-land".to_string()),
        name: "Exert Land".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        red: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                    Cost::Exert,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 2, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

/// A creature with Exert usable BOTH as an attack cost (KeywordAbility::Exert) and as
/// a plain activation cost (`Cost::Exert`, no restriction) -- used to prove CR 701.43b
/// (exerting the same permanent via two different sources still expires in ONE untap
/// step, since Designations::EXERTED is a boolean, not a counter).
fn dual_exert_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dual-exert-creature".to_string()),
        name: "Dual Exert Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Exert),
            AbilityDefinition::Activated {
                cost: Cost::Exert,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

/// A card with `Cost::Exert` activated ONLY from the graveyard (nonsensical combo,
/// but exercises the CR 701.43c "can't exert off the battlefield" guard, which is
/// otherwise unreachable through the normal battlefield-only activation paths).
fn graveyard_exert_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("graveyard-exert".to_string()),
        name: "Graveyard Exert".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Exert,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: Some(ActivationZone::Graveyard),
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

// ── A: Hash schema sentinel ────────────────────────────────────────────────────

#[test]
/// Strict-equality hash schema sentinel (conventions.md hash-sentinel rule).
fn test_hash_schema_version_is_32() {
    assert_eq!(HASH_SCHEMA_VERSION, 35u8);
}

#[test]
/// PB-AC5 H2 regression: `ActivationCost.exert: true` and `exert: false` must produce
/// different hashes. Guards against the exact "PB-S H1 failure mode" the review flagged —
/// `exert` was omitted from `impl HashInto for ActivationCost` and had to be added back.
fn test_exert_field_participates_in_hash() {
    use mtg_engine::state::game_object::{ActivatedAbility, ActivationCost};

    let p1 = p(1);

    let cost_no_exert = ActivationCost {
        exert: false,
        ..Default::default()
    };
    let cost_with_exert = ActivationCost {
        exert: true,
        ..Default::default()
    };

    let make_state = |cost: ActivationCost| {
        let source = ObjectSpec::artifact(p1, "Exert Hash Stone")
            .in_zone(ZoneId::Battlefield)
            .with_activated_ability(ActivatedAbility {
                targets: vec![],
                cost,
                description: "Test ability".to_string(),
                effect: None,
                sorcery_speed: false,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            });
        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![]))
            .object(source)
            .at_step(Step::PreCombatMain)
            .active_player(p1)
            .build()
            .unwrap()
    };

    let hash_no_exert = make_state(cost_no_exert).public_state_hash();
    let hash_with_exert = make_state(cost_with_exert).public_state_hash();

    assert_ne!(
        hash_no_exert, hash_with_exert,
        "PB-AC5 H2 defense: exert field must participate in ActivationCost hash (two costs \
         differing only in exert must produce distinct public hashes)"
    );
}

#[test]
/// PB-AC5 H1 regression: `StackObject.was_warped: true` and `was_warped: false` must
/// produce different hashes. Guards against a stack state where a warp-cast spell (whose
/// resulting permanent gets exiled at the next end step) hashes identically to a normally-
/// cast spell (whose permanent stays) — the exact replay/rewind corruption class the
/// `HashInto for StackObject` impl exists to prevent.
fn test_was_warped_field_participates_in_hash() {
    use mtg_engine::state::stack::{StackObject, StackObjectKind};

    let p1 = p(1);

    let make_stack_spell = |was_warped: bool| StackObject {
        id: ObjectId(9001),
        controller: p1,
        kind: StackObjectKind::Spell {
            source_object: ObjectId(9001),
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_powers: vec![],
        lki_counters: im::OrdMap::new(),
        lki_power: None,
    };

    let make_state = |was_warped: bool| {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![]))
            .at_step(Step::PreCombatMain)
            .active_player(p1)
            .build()
            .unwrap();
        state.stack_objects.push_back(make_stack_spell(was_warped));
        state
    };

    let hash_not_warped = make_state(false).public_state_hash();
    let hash_warped = make_state(true).public_state_hash();

    assert_ne!(
        hash_not_warped, hash_warped,
        "PB-AC5 H1 defense: was_warped field must participate in StackObject hash (two stack \
         objects differing only in was_warped must produce distinct public hashes)"
    );
}

// ── B: Warp (CR 702.185) ────────────────────────────────────────────────────────

#[test]
/// CR 702.185a — Cast from hand paying the warp mana cost + non-mana components
/// (Pay 2 life). Mana and life are deducted; the permanent enters the battlefield.
fn test_warp_cast_from_hand_pays_warp_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![warp_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Warp Test Creature")
                .with_card_id(CardId("warp-test-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.players.get_mut(&p1).unwrap().life_total = 20;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Warp Test Creature");

    let (state, _events) = process_command(
        state,
        empty_cast_spell(p1, card_id, Some(AltCostKind::Warp)),
    )
    .unwrap_or_else(|e| panic!("warp cast should succeed: {:?}", e));

    assert_eq!(
        state.players[&p1].life_total, 18,
        "warp cost's Pay 2 life should be deducted (CR 702.185a)"
    );
    assert_eq!(
        state.players[&p1].mana_pool.get(ManaColor::Black),
        0,
        "warp mana cost {{B}} should be paid from the pool"
    );
    assert!(
        !state.stack_objects.is_empty(),
        "warp-cast spell should be on the stack"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        on_battlefield(&state, "Warp Test Creature"),
        "warp-cast creature should resolve onto the battlefield"
    );
}

#[test]
/// CR 702.185a/b — A warp-cast permanent is exiled at the beginning of the next end
/// step, marked WARPED, with warped_turn recorded.
fn test_warp_exiled_at_next_end_step() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![warp_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Warp Test Creature")
                .with_card_id(CardId("warp-test-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.players.get_mut(&p1).unwrap().life_total = 20;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Warp Test Creature");
    let (state, _) = process_command(
        state,
        empty_cast_spell(p1, card_id, Some(AltCostKind::Warp)),
    )
    .unwrap();

    // Resolve the spell -> creature on battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(on_battlefield(&state, "Warp Test Creature"));

    // Advance PostCombatMain -> End step.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::End,
        "should have advanced to End step"
    );
    let warped_turn = state.turn.turn_number;

    // Resolve the delayed WarpExile trigger.
    let (state, _end_events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Warp Test Creature"),
        "CR 702.185a: warp-cast permanent should be exiled at the next end step"
    );
    let exile_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Warp Test Creature" && o.zone == ZoneId::Exile)
        .expect("warped card should be in exile");
    assert!(
        exile_obj.designations.contains(Designations::WARPED),
        "CR 702.185b: exiled card should be marked WARPED"
    );
    assert_eq!(
        exile_obj.warped_turn, warped_turn,
        "warped_turn should record the turn it was exiled"
    );
}

#[test]
/// CR 702.185a ("if this spell's warp cost was paid") — a normally-cast permanent
/// (no warp) is NOT exiled at end step.
fn test_warp_not_exiled_if_not_warp_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![warp_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Warp Test Creature")
                .with_card_id(CardId("warp-test-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Warp Test Creature");
    let (state, _) = process_command(state, empty_cast_spell(p1, card_id, None)).unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(on_battlefield(&state, "Warp Test Creature"));

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End);

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        on_battlefield(&state, "Warp Test Creature"),
        "normally-cast permanent should NOT be exiled at end step (warp cost not paid)"
    );
}

#[test]
/// CR 702.185a ("after the current turn has ended") — same-turn recast rejected;
/// recast on a later turn succeeds.
fn test_warp_recast_from_exile_after_turn() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![warp_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Warp Test Creature")
                .with_card_id(CardId("warp-test-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .turn_number(5)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.players.get_mut(&p1).unwrap().life_total = 20;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Warp Test Creature");
    let (state, _) = process_command(
        state,
        empty_cast_spell(p1, card_id, Some(AltCostKind::Warp)),
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]); // -> End step
    let (mut state, _) = pass_all(state, &[p1, p2]); // resolve exile trigger

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Warp Test Creature" && o.zone == ZoneId::Exile)
        .map(|(&id, _)| id)
        .expect("warped card should be in exile");

    // Same-turn recast must fail.
    state.turn.priority_holder = Some(p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.players.get_mut(&p1).unwrap().life_total = 20;
    let result = process_command(
        state.clone(),
        empty_cast_spell(p1, exile_id, Some(AltCostKind::Warp)),
    );
    assert!(
        result.is_err(),
        "recasting a warped card on the same turn must fail (CR 702.185a)"
    );

    // Advance to a later turn; recast should succeed.
    let mut state = state;
    state.turn.turn_number += 1;
    state.turn.step = Step::PreCombatMain;
    state.turn.priority_holder = Some(p1);
    let result = process_command(
        state,
        empty_cast_spell(p1, exile_id, Some(AltCostKind::Warp)),
    );
    assert!(
        result.is_ok(),
        "recasting a warped card on a later turn should succeed (CR 702.185a): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.185a / CR 400.7 — A countered warp spell never becomes a permanent, so the
/// delayed exile trigger has nothing to exile: the card stays in the graveyard.
fn test_warp_countered_spell_not_exiled() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![warp_creature_def(), mock_counter_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Warp Test Creature")
                .with_card_id(CardId("warp-test-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Mock Counter")
                .with_card_id(CardId("mock-counter".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p2)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.players.get_mut(&p1).unwrap().life_total = 20;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Warp Test Creature");
    let (state, _) = process_command(
        state,
        empty_cast_spell(p1, card_id, Some(AltCostKind::Warp)),
    )
    .unwrap();

    // p1 passes; p2 responds by casting Mock Counter targeting the warp spell.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let counter_id = find_object(&state, "Mock Counter");
    // CR 601.2c target validation looks up `state.objects` by the CARD's own ObjectId
    // (which moved to ZoneId::Stack when cast) -- i.e. `source_object`, not the
    // StackObject container's own `id`.
    let warp_stack_id = state
        .stack_objects
        .iter()
        .find_map(|so| match &so.kind {
            mtg_engine::StackObjectKind::Spell { source_object }
                if state
                    .objects
                    .get(source_object)
                    .map(|o| o.characteristics.name.as_str())
                    == Some("Warp Test Creature") =>
            {
                Some(*source_object)
            }
            _ => None,
        })
        .expect("warp spell should be on the stack");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: counter_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(warp_stack_id)],
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

    // Resolve Mock Counter -> counters the warp spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        in_graveyard(&state, "Warp Test Creature", p1),
        "countered warp spell should go to the graveyard (never became a permanent)"
    );

    // Advance through end step; no WarpExile trigger should fire (nothing on the
    // battlefield has cast_alt_cost == Warp).
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        in_graveyard(&state, "Warp Test Creature", p1),
        "CR 400.7 / 702.185a: countered spell must remain in graveyard, not be exiled"
    );
    assert!(
        state.stack_objects.is_empty(),
        "no delayed WarpExile trigger should have been queued for a countered spell"
    );
}

#[test]
/// Timeline Culler (real card) — `from_graveyard: true` grants graveyard-cast
/// permission via warp; without it, casting from graveyard would be rejected by the
/// standard zone check.
fn test_warp_timeline_culler_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::timeline_culler::card();
    let registry = CardRegistry::new(vec![def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Timeline Culler")
                .with_card_id(CardId("timeline-culler".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.players.get_mut(&p1).unwrap().life_total = 20;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Timeline Culler");
    let result = process_command(
        state,
        empty_cast_spell(p1, card_id, Some(AltCostKind::Warp)),
    );
    assert!(
        result.is_ok(),
        "Timeline Culler's warp ability should permit casting from the graveyard: {:?}",
        result.err()
    );
}

#[test]
/// CR 118.9a — Warp cannot combine with another alternative cost (Flashback,
/// auto-detected from graveyard + keyword).
fn test_warp_mutual_exclusion() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![warp_and_flashback_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Warp Flashback Hybrid")
                .with_card_id(CardId("warp-flashback-hybrid".to_string()))
                .with_types(vec![CardType::Instant])
                .with_keyword(KeywordAbility::Flashback)
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Warp Flashback Hybrid");
    let result = process_command(
        state,
        empty_cast_spell(p1, card_id, Some(AltCostKind::Warp)),
    );
    assert!(
        result.is_err(),
        "CR 118.9a: warp cast should be rejected when Flashback also auto-applies"
    );
}

// ── C: Transmute (CR 702.53) ────────────────────────────────────────────────────

#[test]
/// Dimir Infiltrator (real card) — Transmute searches for a card with the same mana
/// value (2), reveals it, puts it into hand, and shuffles.
fn test_transmute_searches_equal_mana_value() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::dimir_infiltrator::card();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def, mv2_creature_def(), mv1_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Dimir Infiltrator")
                .with_card_id(CardId("dimir-infiltrator".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs_map,
        ))
        .object(
            ObjectSpec::card(p1, "MV2 Quarry")
                .with_card_id(CardId("mv2-quarry".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .object(
            ObjectSpec::card(p1, "MV1 Decoy")
                .with_card_id(CardId("mv1-decoy".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Dimir Infiltrator");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap_or_else(|e| panic!("transmute activation should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_hand(&state, "MV2 Quarry", p1),
        "CR 702.53a: transmute should find the MV=2 card"
    );
    assert!(
        !in_hand(&state, "MV1 Decoy", p1),
        "transmute should NOT find the MV=1 card (wrong mana value)"
    );
}

#[test]
/// CR 702.53a — Transmute functions only while the card is in hand; not from the
/// battlefield.
fn test_transmute_only_from_hand() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::dimir_infiltrator::card();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Dimir Infiltrator")
                .with_card_id(CardId("dimir-infiltrator".to_string()))
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Dimir Infiltrator");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.53a: transmute must not be activatable from the battlefield"
    );
}

#[test]
/// CR 702.53a — "Activate only as a sorcery": rejected with a nonempty stack.
fn test_transmute_sorcery_timing() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::dimir_infiltrator::card();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def, mv2_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Dimir Infiltrator")
                .with_card_id(CardId("dimir-infiltrator".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs_map,
        ))
        .object(
            ObjectSpec::card(p1, "MV2 Quarry")
                .with_card_id(CardId("mv2-quarry".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    // Not p1's turn -- can't activate a sorcery-speed ability.
    state.turn.active_player = p2;

    let source_id = find_object(&state, "Dimir Infiltrator");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.53a: transmute must follow sorcery timing (not active player's turn)"
    );
}

#[test]
/// CR 702.53a — The source card itself is discarded as part of the transmute cost.
fn test_transmute_discards_self() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::dimir_infiltrator::card();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def, mv2_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Dimir Infiltrator")
                .with_card_id(CardId("dimir-infiltrator".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs_map,
        ))
        .object(
            ObjectSpec::card(p1, "MV2 Quarry")
                .with_card_id(CardId("mv2-quarry".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Dimir Infiltrator");
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    assert!(
        in_graveyard(&state, "Dimir Infiltrator", p1),
        "CR 702.53a: the transmuted card should be discarded (in graveyard)"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { .. })),
        "CardDiscarded event should be emitted for the discard-self cost"
    );
}

// ── D: Exert (CR 701.43) ────────────────────────────────────────────────────────

#[test]
/// Combat Celebrant (real card) — the linked "when you do" trigger untaps all OTHER
/// creatures you control and schedules an additional combat phase, but ONLY when the
/// player chose to exert (CR 701.43d / CR 607.2h).
fn test_exert_combat_celebrant_untaps_and_extra_combat() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = mtg_engine::all_cards();
    let def_map: std::collections::HashMap<String, CardDefinition> =
        defs.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(defs);

    let celebrant = mtg_engine::enrich_spec_from_def(
        ObjectSpec::card(p1, "Combat Celebrant")
            .with_card_id(mtg_engine::card_name_to_id("Combat Celebrant"))
            .in_zone(ZoneId::Battlefield),
        &def_map,
    );
    let ally = ObjectSpec::creature(p1, "Sleepy Ally", 1, 1)
        .in_zone(ZoneId::Battlefield)
        .tapped();

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(celebrant)
        .object(ally)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let celebrant_id = find_object(&state, "Combat Celebrant");
    let ally_id = find_object(&state, "Sleepy Ally");
    assert!(
        state.objects[&ally_id].status.tapped,
        "sanity: ally should start tapped"
    );

    state.turn.priority_holder = Some(p1);
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(celebrant_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![celebrant_id],
        },
    )
    .unwrap_or_else(|e| panic!("DeclareAttackers with exert should succeed: {:?}", e));

    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::PermanentExerted { object_id } if *object_id == celebrant_id)
        ),
        "PermanentExerted event should be emitted"
    );
    assert!(
        state.objects[&celebrant_id]
            .designations
            .contains(Designations::EXERTED),
        "CR 701.43a: exerted attacker should carry the EXERTED designation"
    );
    assert!(
        !state.stack_objects.is_empty(),
        "the linked 'when you do' trigger should be on the stack"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !state.objects[&ally_id].status.tapped,
        "CR 701.43d: the linked trigger should untap the ally (another creature you control)"
    );
    // Assert an exact count (not just `.contains`) so a double-fire of the linked trigger
    // (e.g. if a runtime `TriggerEvent` mapping were later added alongside the existing
    // card-registry-scan firing path) would be caught rather than silently passing.
    let combat_phase_count = state
        .turn
        .additional_phases
        .iter()
        .filter(|p| **p == mtg_engine::state::turn::Phase::Combat)
        .count();
    assert_eq!(
        combat_phase_count, 1,
        "the linked trigger should schedule exactly one additional combat phase, not double-fire"
    );
}

#[test]
/// CR 701.43a — An exerted permanent does not untap during the controller's next
/// untap step; the EXERTED designation is cleared at that point.
fn test_exert_does_not_untap_next_untap_step() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Exerted Beast", 3, 3)
                .in_zone(ZoneId::Battlefield)
                .tapped(),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let beast_id = find_object(&state, "Exerted Beast");
    state
        .objects
        .get_mut(&beast_id)
        .unwrap()
        .designations
        .insert(Designations::EXERTED);

    // First untap step: should skip untap and clear the designation.
    let _ = mtg_engine::rules::turn_actions::untap_active_player_permanents(&mut state);
    assert!(
        state.objects[&beast_id].status.tapped,
        "CR 701.43a: exerted permanent should not untap during the next untap step"
    );
    assert!(
        !state.objects[&beast_id]
            .designations
            .contains(Designations::EXERTED),
        "CR 701.43a/b: EXERTED should be cleared after the skipped untap step"
    );

    // Second untap step: no longer exerted, should untap normally.
    let _ = mtg_engine::rules::turn_actions::untap_active_player_permanents(&mut state);
    assert!(
        !state.objects[&beast_id].status.tapped,
        "permanent should untap normally on the FOLLOWING untap step"
    );
}

#[test]
/// CR 701.43b — Exerting a permanent through two different sources in the same turn
/// (attack cost + activation cost) still only skips ONE untap step (boolean flag).
fn test_exert_twice_expires_same_step() {
    let p1 = p(1);
    let p2 = p(2);
    let def = dual_exert_creature_def();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::creature(p1, "Dual Exert Creature", 2, 2)
                .with_card_id(CardId("dual-exert-creature".to_string()))
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Dual Exert Creature");
    state.turn.priority_holder = Some(p1);

    // Exert once via the attack cost.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(creature_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![creature_id],
        },
    )
    .unwrap();
    assert!(state.objects[&creature_id]
        .designations
        .contains(Designations::EXERTED));

    // Exert a SECOND time via the activation-cost shape (no "already exerted" guard
    // on this shape -- CR 701.43b permits re-exerting).
    let mut state = state;
    state.turn.priority_holder = Some(p1);
    let (mut state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap_or_else(|e| panic!("second exert (activation) should succeed: {:?}", e));

    assert!(state.objects[&creature_id]
        .designations
        .contains(Designations::EXERTED));

    // Only ONE untap step should be skipped, regardless of the double exert.
    let _ = mtg_engine::rules::turn_actions::untap_active_player_permanents(&mut state);
    assert!(
        state.objects[&creature_id].status.tapped,
        "should stay tapped through the first untap step after being exerted twice"
    );
    assert!(
        !state.objects[&creature_id]
            .designations
            .contains(Designations::EXERTED),
        "EXERTED should be cleared after exactly one skipped untap step (CR 701.43b)"
    );
    let _ = mtg_engine::rules::turn_actions::untap_active_player_permanents(&mut state);
    assert!(
        !state.objects[&creature_id].status.tapped,
        "should untap normally on the SECOND untap step (only one step was skipped)"
    );
}

#[test]
/// A creature already EXERTED this turn cannot be offered as an exert choice again
/// via the attack-cost shape (models Combat Celebrant's "hasn't been exerted this
/// turn" printed restriction).
fn test_exert_offer_requires_not_already_exerted() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Already Exerted", 2, 2)
                .with_keyword(KeywordAbility::Exert)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Already Exerted");
    state
        .objects
        .get_mut(&creature_id)
        .unwrap()
        .designations
        .insert(Designations::EXERTED);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(creature_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![creature_id],
        },
    );
    assert!(
        result.is_err(),
        "an already-exerted attacker should not be offered the exert choice again"
    );
}

#[test]
/// `Cost::Exert` as a plain activation cost (arena_of_glory shape): pay {R}, {T},
/// Exert -> mana is added and EXERTED is set (701.43a as an activation cost).
fn test_exert_arena_of_glory_activation() {
    let p1 = p(1);
    let p2 = p(2);
    let def = exert_land_def();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Exert Land")
                .with_card_id(CardId("exert-land".to_string()))
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ))
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
    state.turn.priority_holder = Some(p1);

    let land_id = find_object(&state, "Exert Land");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: land_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap_or_else(|e| panic!("exert activation should succeed: {:?}", e));

    assert!(
        state.objects[&land_id]
            .designations
            .contains(Designations::EXERTED),
        "CR 701.43a: activating the exert cost should set EXERTED"
    );
    assert!(
        state.objects[&land_id].status.tapped,
        "the Tap component of the sequence cost should tap the land"
    );
}

#[test]
/// CR 701.43c — An object not on the battlefield can't be exerted.
fn test_exert_cannot_exert_off_battlefield() {
    let p1 = p(1);
    let p2 = p(2);
    let def = graveyard_exert_def();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Graveyard Exert")
                .with_card_id(CardId("graveyard-exert".to_string()))
                .in_zone(ZoneId::Graveyard(p1)),
            &defs_map,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let source_id = find_object(&state, "Graveyard Exert");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "CR 701.43c: exert cost should be rejected when the source is off the battlefield"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.to_lowercase().contains("battlefield"),
        "error should mention battlefield: {}",
        err
    );
}

// ── E: Pitch (CR 118.9) ─────────────────────────────────────────────────────────

#[test]
/// Force of Will (real card) — pitch cost: pay 1 life and exile a blue card from
/// hand. Mana pool is untouched; the printed spell resolves normally.
fn test_pitch_force_of_will_exile_blue_and_life() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::force_of_will::card();
    let registry = CardRegistry::new(vec![def, mock_threat_spell_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Force of Will")
                .with_card_id(CardId("force-of-will".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Blue Fodder")
                .with_colors(vec![Color::Blue])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Mock Threat Spell")
                .with_card_id(CardId("mock-threat-spell".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p2)),
        )
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().life_total = 20;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    // p2 casts a spell onto the stack for Force of Will to target.
    let threat_id = find_object(&state, "Mock Threat Spell");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: threat_id,
            targets: vec![mtg_engine::state::targeting::Target::Player(p1)],
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
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // CR 601.2c target validation looks up `state.objects` by the CARD's own ObjectId
    // (moved to ZoneId::Stack when cast) -- i.e. `source_object`, not the StackObject
    // container's own `id`.
    let threat_stack_id = match &state
        .stack_objects
        .last()
        .expect("Mock Threat Spell should be on the stack")
        .kind
    {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        other => panic!("expected a Spell on the stack, got {:?}", other),
    };

    let card_id = find_object(&state, "Force of Will");
    let fodder_id = find_object(&state, "Blue Fodder");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(
                threat_stack_id,
            )],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Force of Will pitch cast should succeed: {:?}", e));

    assert_eq!(
        state.players[&p1].life_total, 19,
        "CR 118.9: pitch cost's 1 life payment should be deducted"
    );
    assert!(
        find_in_zone(&state, "Blue Fodder", ZoneId::Exile).is_some(),
        "the pitched blue card should be exiled (CR 400.7: new ObjectId, old `fodder_id` is dead)"
    );
    assert!(
        !state.stack_objects.is_empty(),
        "Force of Will should be on the stack after casting"
    );
}

#[test]
/// CR 118.9 — Attempting to pitch a non-blue card for Force of Will is rejected.
fn test_pitch_wrong_color_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::force_of_will::card();
    let registry = CardRegistry::new(vec![def, mock_threat_spell_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Force of Will")
                .with_card_id(CardId("force-of-will".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Red Fodder")
                .with_colors(vec![Color::Red])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Mock Threat Spell")
                .with_card_id(CardId("mock-threat-spell".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p2)),
        )
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().life_total = 20;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let threat_id = find_object(&state, "Mock Threat Spell");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: threat_id,
            targets: vec![mtg_engine::state::targeting::Target::Player(p1)],
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
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let threat_stack_id = match &state
        .stack_objects
        .last()
        .expect("Mock Threat Spell should be on the stack")
        .kind
    {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        other => panic!("expected a Spell on the stack, got {:?}", other),
    };

    let card_id = find_object(&state, "Force of Will");
    let fodder_id = find_object(&state, "Red Fodder");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(
                threat_stack_id,
            )],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "CR 118.9: pitching a non-blue card for Force of Will should be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.to_lowercase().contains("blue") || err.to_lowercase().contains("color"),
        "error should mention the color mismatch: {}",
        err
    );
}

#[test]
/// Force of Vigor (real card) — pitch cost legal only when it's not the caster's turn.
fn test_pitch_force_of_vigor_opponents_turn_only() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::force_of_vigor::card();
    let registry = CardRegistry::new(vec![def]);

    let build = |active: PlayerId| {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Force of Vigor")
                    .with_card_id(CardId("force-of-vigor".to_string()))
                    .with_types(vec![CardType::Instant])
                    .in_zone(ZoneId::Hand(p1)),
            )
            .object(
                ObjectSpec::card(p1, "Green Fodder")
                    .with_colors(vec![Color::Green])
                    .in_zone(ZoneId::Hand(p1)),
            )
            .active_player(active)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state.turn.priority_holder = Some(p1);
        state
    };

    // On p1's own turn -- pitch should be rejected.
    let state = build(p1);
    let card_id = find_object(&state, "Force of Vigor");
    let fodder_id = find_object(&state, "Green Fodder");
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "Force of Vigor's pitch cost should be rejected on the caster's own turn"
    );

    // On p2's turn -- pitch should succeed.
    let state = build(p2);
    let card_id = find_object(&state, "Force of Vigor");
    let fodder_id = find_object(&state, "Green Fodder");
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Force of Vigor's pitch cost should succeed on an opponent's turn: {:?}",
        result.err()
    );
}

#[test]
/// CR 118.9c — After pitching, the spell's mana value is still the printed value
/// (the paid cost is {0}, but the stored mana cost is unaffected).
fn test_pitch_mana_value_unchanged() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::force_of_will::card();
    let defs_map: std::collections::HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def, mock_threat_spell_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Force of Will")
                .with_card_id(CardId("force-of-will".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs_map,
        ))
        .object(
            ObjectSpec::card(p1, "Blue Fodder")
                .with_colors(vec![Color::Blue])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Mock Threat Spell")
                .with_card_id(CardId("mock-threat-spell".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p2)),
        )
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().life_total = 20;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let threat_id = find_object(&state, "Mock Threat Spell");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: threat_id,
            targets: vec![mtg_engine::state::targeting::Target::Player(p1)],
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
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let threat_stack_id = match &state
        .stack_objects
        .last()
        .expect("Mock Threat Spell should be on the stack")
        .kind
    {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        other => panic!("expected a Spell on the stack, got {:?}", other),
    };

    let card_id = find_object(&state, "Force of Will");
    let fodder_id = find_object(&state, "Blue Fodder");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(
                threat_stack_id,
            )],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let stack_source = state
        .stack_objects
        .iter()
        .find_map(|so| match &so.kind {
            mtg_engine::StackObjectKind::Spell { source_object }
                if state
                    .objects
                    .get(source_object)
                    .map(|o| o.characteristics.name.as_str())
                    == Some("Force of Will") =>
            {
                Some(*source_object)
            }
            _ => None,
        })
        .expect("Force of Will should be on the stack");
    let mana_value = state.objects[&stack_source]
        .characteristics
        .mana_cost
        .as_ref()
        .map(|c| c.mana_value())
        .unwrap_or(0);
    assert_eq!(
        mana_value, 5,
        "CR 118.9c: Force of Will's mana value should remain 5 ({{3}}{{U}}{{U}}) even when pitch-cast"
    );
}

#[test]
/// CR 118.9a — Pitch cannot combine with another alternative cost (Flashback,
/// auto-detected from graveyard + keyword).
fn test_pitch_mutual_exclusion() {
    let p1 = p(1);
    let p2 = p(2);

    let pitch_flashback_def = CardDefinition {
        card_id: CardId("pitch-flashback-hybrid".to_string()),
        name: "Pitch Flashback Hybrid".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Pitch,
                cost: ManaCost::default(),
                details: Some(AltCastDetails::Pitch {
                    costs: vec![Cost::ExileFromHand { color: Color::Blue }],
                    opponents_turn_only: false,
                }),
            },
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Flashback,
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                details: None,
            },
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
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![pitch_flashback_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pitch Flashback Hybrid")
                .with_card_id(CardId("pitch-flashback-hybrid".to_string()))
                .with_types(vec![CardType::Instant])
                .with_keyword(KeywordAbility::Flashback)
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Blue Fodder")
                .with_colors(vec![Color::Blue])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let card_id = find_object(&state, "Pitch Flashback Hybrid");
    let fodder_id = find_object(&state, "Blue Fodder");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "CR 118.9a: pitch cast should be rejected when Flashback also auto-applies"
    );
}

#[test]
/// The spell being cast cannot be its own pitch cost (CR 118.9 — it's on the stack,
/// not in hand, by the time the pitch cost is paid).
fn test_pitch_cannot_pitch_self() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::force_of_will::card();
    let registry = CardRegistry::new(vec![def, mock_threat_spell_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Force of Will")
                .with_card_id(CardId("force-of-will".to_string()))
                .with_types(vec![CardType::Instant])
                .with_colors(vec![Color::Blue])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Mock Threat Spell")
                .with_card_id(CardId("mock-threat-spell".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p2)),
        )
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().life_total = 20;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let threat_id = find_object(&state, "Mock Threat Spell");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: threat_id,
            targets: vec![mtg_engine::state::targeting::Target::Player(p1)],
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
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let threat_stack_id = match &state
        .stack_objects
        .last()
        .expect("Mock Threat Spell should be on the stack")
        .kind
    {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        other => panic!("expected a Spell on the stack, got {:?}", other),
    };

    let card_id = find_object(&state, "Force of Will");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(
                threat_stack_id,
            )],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: card_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    // Isolate the `pitch_card_id == card` guard (casting.rs:4178) specifically: by the time
    // the pitch cost is paid the spell is already on the stack (not in hand), so a weaker test
    // asserting only `is_err()` would also pass if this guard were deleted and the code fell
    // through to the later "chosen card is not in your hand" zone check instead. Assert the
    // guard's own message to make sure it — not some other rejection — is what fires.
    let err = result.expect_err("a spell cannot be pitched to pay for its own casting");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("cannot be its own pitch cost"),
        "expected the self-pitch guard's specific error message, got: {msg}"
    );
}

#[test]
/// Force of Negation (real card) — countering a noncreature spell exiles it instead
/// of putting it into its owner's graveyard (PB-AC5 add-on P5b).
fn test_force_of_negation_counters_and_exiles() {
    let p1 = p(1);
    let p2 = p(2);
    let neg_def = mtg_engine::cards::defs::force_of_negation::card();
    let bolt_def = CardDefinition {
        card_id: CardId("mock-bolt".to_string()),
        name: "Mock Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![neg_def, bolt_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Force of Negation")
                .with_card_id(CardId("force-of-negation".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Blue Fodder")
                .with_colors(vec![Color::Blue])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Mock Bolt")
                .with_card_id(CardId("mock-bolt".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p2)),
        )
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
    state.turn.priority_holder = Some(p2);

    let bolt_id = find_object(&state, "Mock Bolt");
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: bolt_id,
            targets: vec![mtg_engine::state::targeting::Target::Player(p1)],
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

    // p2 passes; p1 (not-active-player) responds with pitch-cast Force of Negation.
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let neg_id = find_object(&state, "Force of Negation");
    let fodder_id = find_object(&state, "Blue Fodder");
    let bolt_stack_id = match &state
        .stack_objects
        .last()
        .expect("Mock Bolt should be on the stack")
        .kind
    {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        other => panic!("expected a Spell on the stack, got {:?}", other),
    };

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: neg_id,
            targets: vec![mtg_engine::state::targeting::Target::Object(bolt_stack_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Pitch),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::state::types::AdditionalCost::ExileFromHand {
                card: fodder_id,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Force of Negation pitch cast should succeed: {:?}", e));

    // CR 601.2i: CastSpell resets priority to the ACTIVE player (p2 here), not the caster.
    let (state, _) = pass_all(state, &[p2, p1]);

    assert!(
        find_in_zone(&state, "Mock Bolt", ZoneId::Exile).is_some(),
        "Force of Negation: countered spell should be exiled instead of graveyard"
    );
    assert!(
        find_in_zone(&state, "Mock Bolt", ZoneId::Graveyard(p2)).is_none(),
        "countered spell should NOT be in the graveyard"
    );
}
