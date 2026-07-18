//! PB-EF1 (scutemob-99): `TargetFilter.exclude_self` enforcement sweep.
//!
//! `TargetFilter.exclude_self` (PB-XS) is honored by the declarative target-validation
//! path and the trigger auto-target picker. PB-EF1 extends it to every *executor* that
//! matches a filter without a threaded source ObjectId. Each test below is a decoy test
//! in the SR-36 sense: the setup is arranged so that if `exclude_self` were ignored (the
//! pre-fix behavior) the assertion would fail — the source itself would be counted /
//! sacrificed / untapped. The decoy fails on *exactly* the `exclude_self` field.
//!
//! CR 109.1: "another [permanent]" excludes the object itself.
//!
//! Sites covered:
//!  1. `EffectAmount::PermanentCount` amount resolver (EF-W-PB2-1, éomer)
//!  2. `Effect::SacrificePermanents` effect + the `MayPayThenEffect` optional-cost path
//!     (EF-W-EMPTY-1, korvold; both funnel through `eligible_sacrifice_targets`)
//!  3. `Effect::UntapAll` executor (EF-W-MISS-2, Copperhorn Scout)
//!  4. `Condition::YouControlNOrMoreWithFilter` (marker EF-5)
//!  5. Activated-ability sacrifice cost (marker EF-4 / OOS-TS-2, Izoni / Yawgmoth /
//!     Commissar Severina Raine) via `ActivationCost.sacrifice_exclude_self`.

use std::collections::HashMap;

use mtg_engine::cards::card_definition::{
    CardDefinition, Condition, Cost, EffectAmount, PlayerTarget, TargetController, TargetFilter,
};
use mtg_engine::effects::check_static_condition;
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::replacement_effect::{
    ObjectFilter, ReplacementModification, ReplacementTrigger,
};
use mtg_engine::testing::replay_harness::enrich_spec_from_def;
use mtg_engine::{
    all_cards, card_name_to_id, process_command, AbilityDefinition, AttackTarget, CardId,
    CardRegistry, CardType, Command, CounterType, Effect, GameEvent, GameState, GameStateBuilder,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SubType, Target, TypeLine, ZoneId,
    HASH_SCHEMA_VERSION,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn life(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_default()
}

fn hand_size(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        events.extend(ev);
    }
    (current, events)
}

fn creature_type_line(subtypes: &[&str]) -> TypeLine {
    TypeLine {
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: subtypes.iter().map(|s| SubType(s.to_string())).collect(),
        ..Default::default()
    }
}

/// The three synthetic defs used to exercise the effect / optional-cost / PermanentCount
/// paths with fully controlled ObjectId ordering (so the pre-fix decoy bites).
fn synthetic_defs() -> Vec<CardDefinition> {
    vec![
        permanent_count_test_creature(),
        sacrifice_effect_source(),
        optional_cost_source(),
    ]
}

/// All real card defs plus the synthetic ones, keyed by name for `enrich_spec_from_def`.
fn all_defs() -> HashMap<String, CardDefinition> {
    let mut m: HashMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();
    for d in synthetic_defs() {
        m.insert(d.name.clone(), d);
    }
    m
}

/// A registry containing every real card plus the synthetic defs (needed for casting).
fn registry() -> std::sync::Arc<CardRegistry> {
    let mut cards = all_cards();
    cards.extend(synthetic_defs());
    CardRegistry::new(cards)
}

/// Card id for a name: real cards use `card_name_to_id`; the synthetic defs carry an
/// explicit id string.
fn id_for(name: &str) -> CardId {
    match name {
        "EF1 Count Test" => CardId("ef1-permanent-count".to_string()),
        "EF1 Sac Effect Source" => CardId("ef1-sac-effect-source".to_string()),
        "EF1 OptCost Source" => CardId("ef1-optcost-source".to_string()),
        _ => card_name_to_id(name),
    }
}

/// Build an enriched `ObjectSpec` for `name` in `zone` (abilities, types, etc. filled in
/// from the card def — a bare `ObjectSpec::card` has no abilities).
fn enrich(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(id_for(name)),
        defs,
    )
}

// ── Live HASH sentinel ────────────────────────────────────────────────────────

#[test]
fn test_ef1_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 47u8,
        "PB-EF1 added ActivationCost.sacrifice_exclude_self (HASH 43->44). Update this \
         sentinel and the state/hash.rs history block together; the authoritative check \
         is the SR-17 machine gate in tests/core/hash_schema.rs."
    );
}

// ── Site 1: EffectAmount::PermanentCount ─────────────────────────────────────

/// A creature that "enters with a +1/+1 counter for each OTHER creature you control".
fn permanent_count_test_creature() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ef1-permanent-count".to_string()),
        name: "EF1 Count Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: creature_type_line(&["Human"]),
        oracle_text: "Enters with a +1/+1 counter for each other creature you control.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield {
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::EntersWithCounters {
                counter: CounterType::PlusOnePlusOne,
                count: Box::new(EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                }),
            },
            is_self: true,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

fn cast_from_hand(
    mut state: GameState,
    caster: PlayerId,
    name: &str,
    mana: &[(ManaColor, u32)],
) -> GameState {
    let card = find_obj(&state, name);
    {
        let pool = &mut state.players_mut().get_mut(&caster).unwrap().mana_pool;
        for &(color, n) in mana {
            pool.add(color, n);
        }
    }
    state.turn_mut().priority_holder = Some(caster);
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
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
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell {} failed: {:?}", name, e));
    state
}

fn counters_on(state: &GameState, name: &str) -> u32 {
    let id = find_obj(state, name);
    state
        .objects()
        .get(&id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0)
}

#[test]
fn permanent_count_excludes_the_entering_source() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();

    // p1 controls NO other creatures. The entering creature is a creature it controls,
    // so if exclude_self were ignored it would count itself: 1 counter -> 2/2.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(p1, "EF1 Count Test", ZoneId::Hand(p1), &defs))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast_from_hand(state, p1, "EF1 Count Test", &[(ManaColor::Colorless, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        counters_on(&state, "EF1 Count Test"),
        0,
        "PermanentCount with exclude_self must NOT count the entering source itself \
         (CR 109.1). With 0 other creatures, 0 counters; a pre-fix engine counts itself -> 1."
    );
}

#[test]
fn permanent_count_counts_only_others() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();

    // Two OTHER creatures already on p1's battlefield -> 2 counters (not 3).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(ObjectSpec::creature(p1, "Other A", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "Other B", 1, 1).in_zone(ZoneId::Battlefield))
        .object(enrich(p1, "EF1 Count Test", ZoneId::Hand(p1), &defs))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast_from_hand(state, p1, "EF1 Count Test", &[(ManaColor::Colorless, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        counters_on(&state, "EF1 Count Test"),
        2,
        "PermanentCount(exclude_self) with 2 other creatures = 2 (pre-fix would count self -> 3)"
    );
}

// ── Site 2: Effect::SacrificePermanents (eligible_sacrifice_targets) ──────────

/// Creature with "{T}: Sacrifice another creature." (Effect::SacrificePermanents path.)
fn sacrifice_effect_source() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ef1-sac-effect-source".to_string()),
        name: "EF1 Sac Effect Source".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: creature_type_line(&["Wizard"]),
        oracle_text: "{T}: Sacrifice another creature.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::SacrificePermanents {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
                filter: Some(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::You,
                    exclude_self: true,
                    ..Default::default()
                }),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

#[test]
fn sacrifice_permanents_effect_excludes_source() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();

    // Source is inserted FIRST -> lowest ObjectId. `sacrifice_permanents_for_player`
    // picks in ascending ObjectId order, so a pre-fix engine (ignoring exclude_self)
    // would sacrifice the source itself. Post-fix it must sacrifice the fodder.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "EF1 Sac Effect Source",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Fodder", 1, 1).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "EF1 Sac Effect Source");
    assert!(
        source_id < find_obj(&state, "Fodder"),
        "test invariant: source must have the lower ObjectId for the decoy to bite"
    );

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
    .expect("activate sacrifice-effect ability");
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "EF1 Sac Effect Source"),
        "SacrificePermanents(exclude_self) must NOT sacrifice its own source (CR 109.1); \
         a pre-fix engine sacrifices the lowest-id permanent, which is the source"
    );
    assert!(
        in_graveyard(&state, "Fodder", p1),
        "the OTHER creature must be the one sacrificed"
    );
}

// ── Site 2: MayPayThenEffect optional-cost sacrifice path ────────────────────

fn optional_cost_source() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ef1-optcost-source".to_string()),
        name: "EF1 OptCost Source".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: creature_type_line(&["Cleric"]),
        oracle_text: "{T}: You may sacrifice another creature. If you do, you gain 5 life."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::MayPayThenEffect {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::You,
                    exclude_self: true,
                    ..Default::default()
                }),
                payer: PlayerTarget::Controller,
                then: Box::new(Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                }),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

#[test]
fn optional_cost_sacrifice_excludes_source() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(p1, "EF1 OptCost Source", ZoneId::Battlefield, &defs))
        .object(ObjectSpec::creature(p1, "OptFodder", 1, 1).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "EF1 OptCost Source");
    let life_before = life(&state, p1);

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
    .expect("activate optional-cost ability");
    let (state, _) = pass_all(state, &[p1, p2]);

    // The pay-when-able optional cost sacrifices the fodder (not the source), so the
    // "if you do" GainLife fires.
    assert!(
        on_battlefield(&state, "EF1 OptCost Source"),
        "optional Cost::Sacrifice(exclude_self) must not sacrifice its own source"
    );
    assert!(
        in_graveyard(&state, "OptFodder", p1),
        "the other creature must be sacrificed to pay the optional cost"
    );
    assert_eq!(
        life(&state, p1),
        life_before + 5,
        "the 'if you do' GainLife fires because the optional cost was paid with another creature"
    );
}

// ── Site 3: Effect::UntapAll (real card — Copperhorn Scout) ───────────────────

#[test]
fn copperhorn_untaps_others_but_not_itself() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();

    // Copperhorn Scout attacks (so it taps itself); another tapped creature untaps.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(p1, "Copperhorn Scout", ZoneId::Battlefield, &defs))
        .object(
            ObjectSpec::creature(p1, "Tapped Ally", 2, 2)
                .tapped()
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let scout_id = find_obj(&state, "Copperhorn Scout");
    let ally_id = find_obj(&state, "Tapped Ally");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(scout_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("declare Copperhorn Scout as attacker");
    // Resolve the attack trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !state.objects().get(&ally_id).unwrap().status.tapped,
        "Copperhorn Scout untaps each OTHER creature you control (CR 109.1)"
    );
    assert!(
        state.objects().get(&scout_id).unwrap().status.tapped,
        "Copperhorn Scout does NOT untap itself (it is attacking, hence tapped); a pre-fix \
         UntapAll ignoring exclude_self would untap it too"
    );
}

// ── Site 4: Condition::YouControlNOrMoreWithFilter (direct) ──────────────────

#[test]
fn you_control_n_or_more_condition_excludes_source() {
    let p1 = p(1);
    let p2 = p(2);

    let condition = Condition::YouControlNOrMoreWithFilter {
        count: 1,
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            controller: TargetController::You,
            exclude_self: true,
            ..Default::default()
        },
    };

    // Only the source is a creature: with exclude_self the count is 0 -> condition FALSE.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Cond Source", 2, 2).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let source_id = find_obj(&state, "Cond Source");
    assert!(
        !check_static_condition(&state, &condition, source_id, p1),
        "YouControlNOrMoreWithFilter(exclude_self) must not count the source; a pre-fix \
         engine counts it and returns true"
    );

    // Add a second creature: now the condition holds (the OTHER creature satisfies it).
    let state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Cond Source", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "Cond Other", 1, 1).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let src2 = find_obj(&state2, "Cond Source");
    assert!(
        check_static_condition(&state2, &condition, src2, p1),
        "with one OTHER creature, 'you control another creature' is satisfied"
    );
}

// ── Site 5: activated-ability sacrifice cost (real cards) ────────────────────

/// The activated ability whose cost includes a sacrifice-another filter.
fn sac_ability_index(state: &GameState, id: ObjectId) -> usize {
    state
        .objects()
        .get(&id)
        .unwrap()
        .characteristics
        .activated_abilities
        .iter()
        .position(|a| a.cost.sacrifice_filter.is_some())
        .expect("card has a sacrifice-cost activated ability")
}

fn add_mana(state: &mut GameState, player: PlayerId, mana: &[(ManaColor, u32)]) {
    let pool = &mut state.players_mut().get_mut(&player).unwrap().mana_pool;
    for &(c, n) in mana {
        pool.add(c, n);
    }
}

#[test]
fn izoni_cannot_sacrifice_itself_to_its_own_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Izoni, Thousand-Eyed",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Insect Fodder", 1, 1).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let izoni_id = find_obj(&state, "Izoni, Thousand-Eyed");
    let idx = sac_ability_index(&state, izoni_id);
    // Pay the mana so only the sacrifice restriction can reject the activation.
    add_mana(
        &mut state,
        p1,
        &[(ManaColor::Black, 1), (ManaColor::Green, 1)],
    );

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: izoni_id,
            ability_index: idx,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(izoni_id), // sacrifice ITSELF — illegal ("another")
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Izoni's 'Sacrifice ANOTHER creature' cost must reject sacrificing Izoni itself \
         (CR 109.1); a pre-fix engine (no ActivationCost.sacrifice_exclude_self) accepts it"
    );
}

#[test]
fn izoni_sacrifices_another_creature_and_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Izoni, Thousand-Eyed",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Insect Fodder", 1, 1).in_zone(ZoneId::Battlefield))
        // A card to draw (drawing from an empty library would not grow the hand).
        .object(ObjectSpec::card(p1, "Library Top").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let izoni_id = find_obj(&state, "Izoni, Thousand-Eyed");
    let fodder_id = find_obj(&state, "Insect Fodder");
    let idx = sac_ability_index(&state, izoni_id);
    add_mana(
        &mut state,
        p1,
        &[(ManaColor::Black, 1), (ManaColor::Green, 1)],
    );
    let life_before = life(&state, p1);
    let hand_before = hand_size(&state, p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: izoni_id,
            ability_index: idx,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(fodder_id),
            x_value: None,
        },
    )
    .expect("sacrificing ANOTHER creature is legal");
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Izoni, Thousand-Eyed"),
        "Izoni survives"
    );
    assert!(
        in_graveyard(&state, "Insect Fodder", p1),
        "the other creature is sacrificed"
    );
    assert_eq!(life(&state, p1), life_before + 1, "You gain 1 life");
    assert_eq!(hand_size(&state, p1), hand_before + 1, "and draw a card");
}

#[test]
fn yawgmoth_cannot_sacrifice_itself() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Yawgmoth, Thran Physician",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Sac Victim", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "Counter Target", 3, 3).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let yawg_id = find_obj(&state, "Yawgmoth, Thran Physician");
    let idx = sac_ability_index(&state, yawg_id);

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: yawg_id,
            ability_index: idx,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(yawg_id), // itself — illegal
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Yawgmoth's 'Pay 1 life, Sacrifice ANOTHER creature' cost must reject sacrificing itself"
    );
}

#[test]
fn yawgmoth_sacrifices_another_creature_and_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    // No mana needed — Yawgmoth's sacrifice ability costs Pay 1 life + Sacrifice.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Yawgmoth, Thran Physician",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Sac Victim", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "Counter Target", 3, 3).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p1, "Library Top").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let yawg_id = find_obj(&state, "Yawgmoth, Thran Physician");
    let victim_id = find_obj(&state, "Sac Victim");
    let target_id = find_obj(&state, "Counter Target");
    let idx = sac_ability_index(&state, yawg_id);
    let hand_before = hand_size(&state, p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: yawg_id,
            ability_index: idx,
            targets: vec![Target::Object(target_id)],
            discard_card: None,
            sacrifice_target: Some(victim_id), // ANOTHER creature — legal
            x_value: None,
        },
    )
    .expect("Yawgmoth sacrificing another creature is legal");
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Yawgmoth, Thran Physician"),
        "Yawgmoth survives"
    );
    assert!(
        in_graveyard(&state, "Sac Victim", p1),
        "the other creature is sacrificed"
    );
    assert_eq!(
        hand_size(&state, p1),
        hand_before + 1,
        "Yawgmoth draws a card on resolution"
    );
    // The -1/-1 counter landed on the target creature (up-to-one target).
    assert_eq!(
        state
            .objects()
            .get(&target_id)
            .unwrap()
            .counters
            .get(&CounterType::MinusOneMinusOne)
            .copied()
            .unwrap_or(0),
        1,
        "a -1/-1 counter is placed on the chosen target"
    );
}

#[test]
fn commissar_cannot_sacrifice_itself() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Commissar Severina Raine",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Guardsman", 1, 1).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let commissar_id = find_obj(&state, "Commissar Severina Raine");
    let idx = sac_ability_index(&state, commissar_id);
    add_mana(&mut state, p1, &[(ManaColor::Colorless, 2)]);

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: commissar_id,
            ability_index: idx,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(commissar_id),
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Commissar's '{{2}}, Sacrifice ANOTHER creature' cost must reject sacrificing herself"
    );
}

#[test]
fn commissar_sacrifices_another_creature_and_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Commissar Severina Raine",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Guardsman", 1, 1).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p1, "Library Top").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let commissar_id = find_obj(&state, "Commissar Severina Raine");
    let guardsman_id = find_obj(&state, "Guardsman");
    let idx = sac_ability_index(&state, commissar_id);
    add_mana(&mut state, p1, &[(ManaColor::Colorless, 2)]);
    let life_before = life(&state, p1);
    let hand_before = hand_size(&state, p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: commissar_id,
            ability_index: idx,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(guardsman_id), // ANOTHER creature — legal
            x_value: None,
        },
    )
    .expect("Commissar sacrificing another creature is legal");
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Commissar Severina Raine"),
        "Commissar survives"
    );
    assert!(
        in_graveyard(&state, "Guardsman", p1),
        "the other creature is sacrificed"
    );
    assert_eq!(life(&state, p1), life_before + 2, "You gain 2 life");
    assert_eq!(hand_size(&state, p1), hand_before + 1, "and draw a card");
}

// ── Card-level: Korvold ETB sacrifices another permanent ─────────────────────

#[test]
fn korvold_etb_sacrifices_another_permanent_not_itself() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();

    // Korvold in hand; p1 already controls a spare permanent to feed the ETB.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(
            p1,
            "Korvold, Fae-Cursed King",
            ZoneId::Hand(p1),
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Spare Goblin", 1, 1).in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    add_mana(
        &mut state,
        p1,
        &[
            (ManaColor::Colorless, 2),
            (ManaColor::Black, 1),
            (ManaColor::Red, 1),
            (ManaColor::Green, 1),
        ],
    );
    let state = cast_from_hand(state, p1, "Korvold, Fae-Cursed King", &[]);
    // Resolve Korvold, then its ETB "sacrifice another permanent" trigger.
    let (state, _) = pass_all(state, &[p1, p2, p1, p2]);

    assert!(
        on_battlefield(&state, "Korvold, Fae-Cursed King"),
        "Korvold's forced 'sacrifice another permanent' must not target Korvold itself"
    );
    assert!(
        in_graveyard(&state, "Spare Goblin", p1),
        "the spare permanent is the one sacrificed"
    );
}
