//! PB-EF7 (scutemob-108): modal `AbilityDefinition::Activated { modes }` (EF-W-PB2-4).
//!
//! CR 601.2b / 602.2b / 700.2: a "Choose one —" **activated** ability announces its
//! mode(s) as part of activating it (602.2b makes 601.2b apply to activation), and
//! resolves ONLY the chosen mode. Approach (a): the chosen mode's effect is baked into
//! the stack object's `embedded_effect` **at activation time**, not resolution — required
//! because both eligible cards (Goblin Cratermaker, Cankerbloom) pay `Cost::SacrificeSelf`,
//! so by resolution the source `ObjectId` is dead (CR 400.7). `resolution.rs`'s
//! `ActivatedAbility` arm is UNCHANGED by this PB: it already resolves purely from
//! `embedded_effect` + `stack_obj.targets`, which is exactly what makes approach (a) work.
//!
//! Cards: Goblin Cratermaker (2 modes: deal 2 damage to target creature / destroy target
//! colorless nonland permanent) and Cankerbloom (3 modes: destroy target artifact /
//! destroy target enchantment / proliferate — the last with an EMPTY target slice per
//! CR 700.2c, the headline fix over the old `Effect::Choose` encoding).

use std::collections::HashMap;

use mtg_engine::state::test_util;
use mtg_engine::{
    enrich_spec_from_def, process_command, ActivatedAbility, ActivationCost, CardDefinition,
    CardRegistry, Color, Command, CounterType, Effect, EffectAmount, GameEvent, GameState,
    GameStateBuilder, GameStateError, ManaColor, ModeSelection, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, Target, TargetRequirement, ZoneId, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects()
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

fn counter_count(state: &GameState, name: &str, counter: CounterType) -> u32 {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .and_then(|(_, obj)| obj.counters.get(&counter).copied())
        .unwrap_or(0)
}

/// Pass priority for all listed players once (resolves the top of the stack).
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

fn activate(
    state: GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen,
        },
    )
}

/// Goblin Cratermaker's only non-mana activated ability lands at index 0.
const GC_ABILITY_INDEX: usize = 0;
/// Cankerbloom's only non-mana activated ability lands at index 0.
const CB_ABILITY_INDEX: usize = 0;

/// Build a 2-player state with Goblin Cratermaker on the battlefield under p1's
/// control, `mana` colorless mana pre-loaded into p1's pool, plus any `extra`
/// battlefield permanents (decoys/targets). Returns `(state, p1, p2)`.
fn build_goblin_state(mana: u32, extra: Vec<ObjectSpec>) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::goblin_cratermaker::card();
    let defs_map: HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def.clone()]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Goblin Cratermaker")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ));
    for spec in extra {
        builder = builder.object(spec);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    if mana > 0 {
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, mana);
    }
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Build a 2-player state with Cankerbloom on the battlefield under p1's control,
/// `mana` colorless mana pre-loaded, plus `extra` battlefield permanents.
fn build_cankerbloom_state(mana: u32, extra: Vec<ObjectSpec>) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::cankerbloom::card();
    let defs_map: HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def.clone()]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Cankerbloom")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ));
    for spec in extra {
        builder = builder.object(spec);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    if mana > 0 {
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, mana);
    }
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Build a 2-player state with a synthetic modal activated ability on an artifact
/// (`EF7 Synthetic Modal Artifact`, ability index 0). The cost is a bare tap, which
/// makes "no cost paid" provable via `status.tapped` on a rejected activation.
/// `effect: None` is fine for these tests -- all of them assert on a REJECTED
/// activation, so `handle_activate_ability` never reaches the point where it would
/// bake a chosen mode's effect into `embedded_effect`.
fn build_synthetic_modal_state(modes: ModeSelection) -> (GameState, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let ability = ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            ..Default::default()
        },
        description: "EF7 synthetic modal test ability".to_string(),
        effect: None,
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: Some(modes),
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::artifact(p1, "EF7 Synthetic Modal Artifact")
                .with_activated_ability(ability),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let source_id = find_object(&state, "EF7 Synthetic Modal Artifact");
    (state, p1, source_id)
}

// ── Version sentinels ─────────────────────────────────────────────────────────

/// CR 700.2a: PB-EF7 bumped both wire versions. Authoritative machine gates are
/// `tests/core/hash_schema.rs` / `tests/core/protocol_schema.rs`; this sentinel just
/// forces a deliberate edit here (and to `state/hash.rs` / `rules/protocol.rs`) on
/// any future bump, mirroring the convention on every other PB test module.
#[test]
fn test_ef7_hash_and_protocol_versions() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 58u8,
        "PB-EF7 added AbilityDefinition::Activated::modes / ActivatedAbility::modes \
         (HASH 49->50). Update this sentinel and the state/hash.rs history block together."
    );
    assert_eq!(
        PROTOCOL_VERSION, 21,
        "PB-EF7 added Command::ActivateAbility.modes_chosen (PROTOCOL 11->12). Update this \
         sentinel and the rules/protocol.rs history block together."
    );
}

// ── Generic modal-activated mechanism (CR 602.2b/700.2a) ─────────────────────

/// CR 602.2b/700.2a: activating a modal ability choosing mode 0 resolves ONLY mode 0's
/// effect. Forward decoy: a colorless nonland permanent that ONLY mode 1 could legally
/// affect is placed on the board and must remain untouched after resolution — if the
/// engine executed mode 1 (or both modes), this decoy would be destroyed.
#[test]
fn test_602_2b_modal_activated_resolves_only_chosen_mode() {
    let (state, p1, p2) = build_goblin_state(
        1,
        vec![
            ObjectSpec::creature(p(2), "GC Target Creature", 1, 2),
            ObjectSpec::artifact(p(1), "GC Colorless Rock"),
        ],
    );
    let source_id = find_object(&state, "Goblin Cratermaker");
    let target_id = find_object(&state, "GC Target Creature");

    let (state, _events) = activate(
        state,
        p1,
        source_id,
        GC_ABILITY_INDEX,
        vec![Target::Object(target_id)],
        vec![0],
    )
    .unwrap_or_else(|e| panic!("mode-0 activation should succeed: {:?}", e));

    // Cost already paid at activation (CR 602.2c): source is sacrificed immediately.
    assert!(
        in_graveyard(&state, "Goblin Cratermaker", p1),
        "sacrifice-self cost is paid at activation, before resolution"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_graveyard(&state, "GC Target Creature", p2),
        "CR 700.2a: mode 0 (2 damage) should have killed the 1/2 target creature"
    );
    assert!(
        on_battlefield(&state, "GC Colorless Rock"),
        "CR 700.2a: mode 1 (destroy colorless permanent) must NOT have resolved — the \
         colorless rock is untouched proof that only the chosen mode ran"
    );
}

/// CR 700.2a: reverse of the above — activating mode 1 destroys the colorless permanent
/// and leaves mode 0's legal target (the creature) completely untouched.
#[test]
fn test_700_2a_modal_activated_reverse_decoy() {
    let (state, p1, p2) = build_goblin_state(
        1,
        vec![
            ObjectSpec::creature(p(2), "GC Target Creature", 1, 2),
            ObjectSpec::artifact(p(1), "GC Colorless Rock"),
        ],
    );
    let source_id = find_object(&state, "Goblin Cratermaker");
    let rock_id = find_object(&state, "GC Colorless Rock");

    let (state, _events) = activate(
        state,
        p1,
        source_id,
        GC_ABILITY_INDEX,
        vec![Target::Object(rock_id)],
        vec![1],
    )
    .unwrap_or_else(|e| panic!("mode-1 activation should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_graveyard(&state, "GC Colorless Rock", p1),
        "CR 700.2a: mode 1 (destroy colorless permanent) should have resolved"
    );
    assert!(
        on_battlefield(&state, "GC Target Creature"),
        "CR 700.2a: mode 0 (2 damage) must NOT have resolved — the creature is untouched \
         proof that only the chosen mode ran"
    );
}

/// CR 700.2a: an out-of-range mode index on a 2-mode ability is rejected, and no cost is
/// paid (CR 602.2 -- an illegal activation rewinds to before it started).
#[test]
fn test_700_2a_invalid_mode_index_rejected() {
    let (state, p1, _p2) = build_goblin_state(1, vec![]);
    let source_id = find_object(&state, "Goblin Cratermaker");

    let result = activate(state, p1, source_id, GC_ABILITY_INDEX, vec![], vec![5]);

    // The real, non-vacuous check: the activation is rejected. CR 602.2's "no cost paid /
    // rewind to before it started" is guaranteed structurally, not observably here — mode
    // validation runs BEFORE the cost-payment block (abilities.rs), and `process_command`
    // consumes `state` and drops the (unmutated) value on `Err`, so no partial mutation can
    // escape. There is no post-`Err` state to inspect, so asserting on a pre-call clone would
    // be vacuous.
    assert!(
        result.is_err(),
        "CR 700.2a: mode index 5 is out of range for a 2-mode ability"
    );
}

/// CR 700.2a: `modes_chosen` on a non-modal activated ability (no `ModeSelection`) is
/// rejected outright.
#[test]
fn test_700_2a_modes_chosen_on_nonmodal_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            ..Default::default()
        },
        description: "EF7 nonmodal test ability".to_string(),
        effect: Some(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(1),
        }),
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::artifact(p(1), "EF7 Nonmodal Artifact").with_activated_ability(ability))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let source_id = find_object(&state, "EF7 Nonmodal Artifact");

    let result = activate(state, p1, source_id, 0, vec![], vec![0]);

    assert!(
        result.is_err(),
        "CR 700.2a: modes_chosen on a non-modal ability must be rejected"
    );
}

/// CR 700.2d: choosing the same mode index twice is rejected when
/// `allow_duplicate_modes` is `false` (the default), and no cost is paid.
#[test]
fn test_700_2d_duplicate_mode_rejected_when_disallowed() {
    let ms = ModeSelection {
        min_modes: 1,
        max_modes: 2,
        allow_duplicate_modes: false,
        mode_costs: None,
        modes: vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(2),
            },
        ],
        mode_targets: None,
    };
    let (state, p1, source_id) = build_synthetic_modal_state(ms);

    let result = activate(state, p1, source_id, 0, vec![], vec![0, 0]);

    // Rejection is the non-vacuous check (see `test_700_2a_invalid_mode_index_rejected` for
    // why "no cost paid" is a structural, not observable, guarantee on the `Err` path).
    assert!(
        result.is_err(),
        "CR 700.2d: choosing mode 0 twice must be rejected when allow_duplicate_modes is false"
    );
}

/// CR 700.2a: choosing fewer modes than `min_modes` (Part A) or more modes than
/// `max_modes` (Part B) is rejected, and no cost is paid in either case.
#[test]
fn test_700_2a_mode_count_bounds_rejected() {
    // Part A: too few modes chosen (min_modes: 2, only 1 chosen).
    let ms_min = ModeSelection {
        min_modes: 2,
        max_modes: 2,
        allow_duplicate_modes: true,
        mode_costs: None,
        modes: vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(2),
            },
        ],
        mode_targets: None,
    };
    let (state, p1, source_id) = build_synthetic_modal_state(ms_min);

    let result = activate(state, p1, source_id, 0, vec![], vec![0]);

    // Rejection is the non-vacuous check (rewind is structural — see
    // `test_700_2a_invalid_mode_index_rejected`).
    assert!(
        result.is_err(),
        "CR 700.2a: choosing 1 mode when min_modes=2 must be rejected"
    );

    // Part B: too many modes chosen (max_modes: 1, 2 chosen).
    let ms_max = ModeSelection {
        min_modes: 1,
        max_modes: 1,
        allow_duplicate_modes: true,
        mode_costs: None,
        modes: vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(2),
            },
        ],
        mode_targets: None,
    };
    let (state, p1, source_id) = build_synthetic_modal_state(ms_max);

    let result = activate(state, p1, source_id, 0, vec![], vec![0, 1]);

    // Rejection is the non-vacuous check (rewind is structural — see
    // `test_700_2a_invalid_mode_index_rejected`).
    assert!(
        result.is_err(),
        "CR 700.2a: choosing 2 modes when max_modes=1 must be rejected"
    );
}

/// CR 700.2c/700.2a: choosing MULTIPLE modes on an ability whose
/// `ModeSelection.mode_targets` is `Some` is a hard reject -- the per-mode LOCAL
/// target-slice indexing that `mode_targets` implies is not supported when more than
/// one mode's effects would need to be combined. No card in the corpus hits this
/// branch (both flipped cards are choose-exactly-one); this is a synthetic regression
/// test for the guard itself. Two LEGAL creature targets are supplied (one per chosen
/// mode's requirement) so the guard is the ONLY thing standing between this activation
/// and success -- without it, `validate_targets_positional` would accept both targets
/// and the activation would go through.
#[test]
fn test_700_2c_multi_mode_with_mode_targets_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let ms = ModeSelection {
        min_modes: 1,
        max_modes: 2,
        allow_duplicate_modes: false,
        mode_costs: None,
        modes: vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(2),
            },
        ],
        mode_targets: Some(vec![
            vec![TargetRequirement::TargetCreature],
            vec![TargetRequirement::TargetCreature],
        ]),
    };
    let ability = ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            ..Default::default()
        },
        description: "EF7 synthetic multi-mode+targets test ability".to_string(),
        effect: None,
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: Some(ms),
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::artifact(p1, "EF7 Synthetic Multi-Mode Artifact")
                .with_activated_ability(ability),
        )
        .object(ObjectSpec::creature(p2, "EF7 Mode Target A", 1, 1))
        .object(ObjectSpec::creature(p2, "EF7 Mode Target B", 1, 1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let source_id = find_object(&state, "EF7 Synthetic Multi-Mode Artifact");
    let target_a = find_object(&state, "EF7 Mode Target A");
    let target_b = find_object(&state, "EF7 Mode Target B");

    let result = activate(
        state,
        p1,
        source_id,
        0,
        vec![Target::Object(target_a), Target::Object(target_b)],
        vec![0, 1],
    );

    // Rejection is the non-vacuous check (rewind is structural — see
    // `test_700_2a_invalid_mode_index_rejected`).
    assert!(
        result.is_err(),
        "CR 700.2c: multiple modes chosen combined with ModeSelection.mode_targets must be \
         rejected, even with a legal target supplied for each chosen mode"
    );
}

/// CR 700.2a/601.2c: activating mode 1 (destroy target colorless nonland permanent)
/// with no target supplied, when no legal target exists anywhere on the board, is
/// rejected -- and no cost is paid.
#[test]
fn test_700_2a_mode_with_no_legal_target_rejected() {
    let (state, p1, _p2) = build_goblin_state(1, vec![]);
    let source_id = find_object(&state, "Goblin Cratermaker");

    let result = activate(state, p1, source_id, GC_ABILITY_INDEX, vec![], vec![1]);

    // Rejection is the non-vacuous check (rewind is structural — see
    // `test_700_2a_invalid_mode_index_rejected`).
    assert!(
        result.is_err(),
        "CR 700.2a/601.2c: mode 1 requires a target, and none was supplied while none is legal"
    );
}

/// CR 400.7/601.2b (LKI): approach (a) freezes BOTH the chosen mode AND its target into
/// `embedded_effect` / `stack_obj.targets` at activation. This test discriminates that
/// from a hypothetical re-resolution that re-derives the target from the board at
/// resolution time: a SECOND colorless nonland permanent (a clone of the original
/// target's characteristics, so it is legal for mode 1 in every respect except
/// identity) is added to the battlefield AFTER activation but BEFORE resolution. If
/// the engine re-picked a legal target for mode 1 at resolution instead of using the
/// frozen ObjectId, it could destroy the new permanent instead of (or in addition to)
/// the originally-chosen one -- this test fails in that case.
#[test]
fn test_601_2b_modal_choice_survives_intervening_change() {
    let (state, p1, p2) = build_goblin_state(
        1,
        vec![
            ObjectSpec::creature(p(2), "GC Target Creature", 1, 2),
            ObjectSpec::artifact(p(1), "GC Colorless Rock"),
        ],
    );
    let source_id = find_object(&state, "Goblin Cratermaker");
    let rock_id = find_object(&state, "GC Colorless Rock");

    let (mut state, _events) = activate(
        state,
        p1,
        source_id,
        GC_ABILITY_INDEX,
        vec![Target::Object(rock_id)],
        vec![1],
    )
    .unwrap_or_else(|e| panic!("mode-1 activation should succeed: {:?}", e));

    // Intervening board change between activation and resolution (CR 400.7): a SECOND
    // colorless nonland permanent enters the battlefield, legal for mode 1 in every
    // respect the original target is.
    let mut second_rock = state.object(rock_id).unwrap().clone();
    second_rock.characteristics.name = "GC Second Colorless Rock".to_string();
    test_util::add_object(&mut state, second_rock, ZoneId::Battlefield)
        .expect("adding the intervening decoy permanent must succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_graveyard(&state, "GC Colorless Rock", p1),
        "the originally-chosen mode (destroy colorless permanent) must still resolve \
         against its frozen target despite the intervening board change"
    );
    assert!(
        on_battlefield(&state, "GC Second Colorless Rock"),
        "the mode/target choice is frozen by ObjectId at activation, not re-derived at \
         resolution -- the intervening decoy permanent (added AFTER activation) must survive; \
         the test would fail here if the engine re-picked a legal target at resolution"
    );
    assert!(
        on_battlefield(&state, "GC Target Creature"),
        "mode 0 must still not have resolved after the intervening change"
    );
}

// ── Goblin Cratermaker card-integration tests ─────────────────────────────────

/// CR 700.2a: full activation of Goblin Cratermaker's mode 0 (2 damage to target
/// creature), decoy: a second creature not targeted remains undamaged.
#[test]
fn test_goblin_cratermaker_mode0_deals_damage() {
    let (state, p1, p2) = build_goblin_state(
        1,
        vec![
            ObjectSpec::creature(p(2), "GC Target Creature", 1, 2),
            ObjectSpec::creature(p(2), "GC Decoy Creature 2", 1, 2),
        ],
    );
    let source_id = find_object(&state, "Goblin Cratermaker");
    let target_id = find_object(&state, "GC Target Creature");

    let (state, _) = activate(
        state,
        p1,
        source_id,
        GC_ABILITY_INDEX,
        vec![Target::Object(target_id)],
        vec![0],
    )
    .unwrap_or_else(|e| panic!("mode-0 activation should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_graveyard(&state, "GC Target Creature", p2),
        "2 damage should kill the 1/2 target creature"
    );
    assert!(
        on_battlefield(&state, "GC Decoy Creature 2"),
        "the second (non-targeted) creature must be undamaged"
    );
}

/// CR 700.2a: Goblin Cratermaker's mode 1 (destroy target colorless nonland permanent).
/// `exclude_colors` decoy: a COLORED nonland permanent is not a legal target for this
/// mode (activation with it as target fails), while a colorless one is legal.
#[test]
fn test_goblin_cratermaker_mode1_destroys_colorless_only() {
    // Part A: a colored nonland permanent is NOT a legal target for mode 1.
    let (state, p1, _p2) = build_goblin_state(
        1,
        vec![ObjectSpec::artifact(p(1), "GC Red Rock").with_colors(vec![Color::Red])],
    );
    let source_id = find_object(&state, "Goblin Cratermaker");
    let red_rock_id = find_object(&state, "GC Red Rock");

    let result = activate(
        state,
        p1,
        source_id,
        GC_ABILITY_INDEX,
        vec![Target::Object(red_rock_id)],
        vec![1],
    );
    assert!(
        result.is_err(),
        "CR 700.2a: a colored nonland permanent is not a legal target for the \
         'destroy target colorless nonland permanent' mode (exclude_colors)"
    );

    // Part B: a colorless nonland permanent IS a legal target for mode 1.
    let (state, p1, p2) =
        build_goblin_state(1, vec![ObjectSpec::artifact(p(1), "GC Colorless Rock")]);
    let source_id = find_object(&state, "Goblin Cratermaker");
    let rock_id = find_object(&state, "GC Colorless Rock");

    let (state, _) = activate(
        state,
        p1,
        source_id,
        GC_ABILITY_INDEX,
        vec![Target::Object(rock_id)],
        vec![1],
    )
    .unwrap_or_else(|e| {
        panic!(
            "mode-1 activation on a colorless permanent should succeed: {:?}",
            e
        )
    });

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        in_graveyard(&state, "GC Colorless Rock", p1),
        "the colorless nonland permanent should have been destroyed"
    );
}

// ── Cankerbloom card-integration tests ────────────────────────────────────────

/// CR 700.2a: Cankerbloom mode 0 (destroy target artifact); decoy: an enchantment on the
/// battlefield is untouched.
#[test]
fn test_cankerbloom_mode0_destroys_artifact() {
    let (state, p1, p2) = build_cankerbloom_state(
        1,
        vec![
            ObjectSpec::artifact(p(2), "CB Test Artifact"),
            ObjectSpec::enchantment(p(2), "CB Test Enchantment"),
        ],
    );
    let source_id = find_object(&state, "Cankerbloom");
    let artifact_id = find_object(&state, "CB Test Artifact");

    let (state, _) = activate(
        state,
        p1,
        source_id,
        CB_ABILITY_INDEX,
        vec![Target::Object(artifact_id)],
        vec![0],
    )
    .unwrap_or_else(|e| panic!("mode-0 activation should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_graveyard(&state, "CB Test Artifact", p2),
        "mode 0 should have destroyed the artifact"
    );
    assert!(
        on_battlefield(&state, "CB Test Enchantment"),
        "the enchantment (mode 1's target type) must be untouched"
    );
}

/// CR 700.2a: Cankerbloom mode 1 (destroy target enchantment); decoy: an artifact on the
/// battlefield is untouched.
#[test]
fn test_cankerbloom_mode1_destroys_enchantment() {
    let (state, p1, p2) = build_cankerbloom_state(
        1,
        vec![
            ObjectSpec::artifact(p(2), "CB Test Artifact"),
            ObjectSpec::enchantment(p(2), "CB Test Enchantment"),
        ],
    );
    let source_id = find_object(&state, "Cankerbloom");
    let enchantment_id = find_object(&state, "CB Test Enchantment");

    let (state, _) = activate(
        state,
        p1,
        source_id,
        CB_ABILITY_INDEX,
        vec![Target::Object(enchantment_id)],
        vec![1],
    )
    .unwrap_or_else(|e| panic!("mode-1 activation should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        in_graveyard(&state, "CB Test Enchantment", p2),
        "mode 1 should have destroyed the enchantment"
    );
    assert!(
        on_battlefield(&state, "CB Test Artifact"),
        "the artifact (mode 0's target type) must be untouched"
    );
}

/// CR 700.2c: Cankerbloom mode 2 (Proliferate) has an EMPTY target slice -- activating it
/// with NO artifact or enchantment anywhere on the battlefield must still succeed (the old
/// `Effect::Choose` encoding wrongly demanded a legal artifact AND enchantment target up
/// front to activate at all).
#[test]
fn test_cankerbloom_mode2_proliferate_needs_no_target() {
    let (state, p1, p2) = build_cankerbloom_state(
        1,
        vec![{
            let mut spec = ObjectSpec::creature(p(1), "CB Counter Bearer", 2, 2);
            spec = spec.with_counter(CounterType::PlusOnePlusOne, 1);
            spec
        }],
    );
    let source_id = find_object(&state, "Cankerbloom");
    assert_eq!(
        counter_count(&state, "CB Counter Bearer", CounterType::PlusOnePlusOne),
        1
    );

    // No artifact, no enchantment anywhere on the board -- activation must still succeed.
    let (state, _) = activate(state, p1, source_id, CB_ABILITY_INDEX, vec![], vec![2])
        .unwrap_or_else(|e| {
            panic!(
                "CR 700.2c: proliferate mode must not require an artifact/enchantment target: {:?}",
                e
            )
        });

    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        counter_count(&state, "CB Counter Bearer", CounterType::PlusOnePlusOne),
        2,
        "proliferate should have added a second +1/+1 counter"
    );
}
