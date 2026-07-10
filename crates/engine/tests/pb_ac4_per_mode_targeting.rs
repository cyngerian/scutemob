//! PB-AC4: Modal & optional targeting — per-mode `TargetRequirement` on `ModeSelection`
//! (CR 700.2c / 700.2f).
//!
//! Before this batch, modal spells validated targets against the FLAT UNION of every
//! mode's `TargetRequirement`s, even for modes the caster did not choose. This produced
//! wrong game state: a "Choose one — Destroy target creature / Destroy target artifact /
//! Destroy target land" spell (Casualties-of-War-style) demanded the caster declare a
//! target for every mode's type, even types absent from the battlefield, making the spell
//! effectively uncastable in most board states.
//!
//! `ModeSelection.mode_targets: Option<Vec<Vec<TargetRequirement>>>` fixes this: when set,
//! `mode_targets[i]` is mode `i`'s own (fixed-length) target-requirement list. Targets are
//! announced/validated ONLY for the chosen modes' requirements, concatenated in ascending
//! chosen-mode order (CR 700.2c: "its controller will need to choose those targets only if
//! they chose that mode"). At resolution, each chosen mode's effects see a LOCAL
//! `DeclaredTarget { index }` slice (index 0 = that mode's first target) — no cross-mode
//! contamination (CR 700.2f: "Modal spells and abilities may have different targeting
//! requirements for each mode").
//!
//! CR Rules covered:
//! - CR 601.2c / 700.2c: targets are announced/validated only for chosen modes.
//! - CR 700.2a: illegal-mode gating (pre-existing, unaffected by this batch).
//! - CR 700.2d: duplicate modes get independent target slices.
//! - CR 700.2f: two chosen modes' targets are sliced independently (no cross-contamination).
//! - CR 608.2b: partial illegal target skips only that mode; full illegal fizzles the spell.
//! - `mode_targets: None` (legacy modal spells) behave exactly as before AC4.

use std::sync::Arc;

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Effect, EffectTarget, TargetRequirement, TypeLine,
};
use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::{CardId, CardType, GameStateBuilder, ObjectSpec, PlayerId, Target, ZoneId};
use mtg_engine::{
    AdditionalCost, CardRegistry, EffectAmount, GameState, KeywordAbility, ManaCost, ModeSelection,
    ObjectId, PlayerTarget, HASH_SCHEMA_VERSION,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn obj_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn obj_in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut s = state;
    let mut all_events = Vec::new();
    for &pl in players {
        let (ns, evs) = process_command(s, Command::PassPriority { player: pl }).unwrap();
        all_events.extend(evs);
        s = ns;
    }
    (s, all_events)
}

/// Cast a (possibly modal) spell, choosing `modes_chosen` and declaring `targets`.
fn cast_modal(
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> Command {
    Command::CastSpell {
        player,
        card,
        targets,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: None,
        prototype: false,
        modes_chosen,
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

/// "Modal Strike" — Instant, no mana cost (mana payment is orthogonal to this batch).
///
/// Choose one or more —
/// • Destroy target creature.      (mode_targets[0] = [TargetCreature])
/// • Destroy target artifact.      (mode_targets[1] = [TargetArtifact])
/// • Destroy target land.          (mode_targets[2] = [TargetLand])
/// • Gain 1 life.                  (mode_targets[3] = [] — no target)
///
/// Mirrors the Casualties of War / Cryptic Command shape: multiple single-target modes of
/// different types plus a targetless mode, all under one modal spell.
fn modal_strike_def() -> CardDefinition {
    CardDefinition {
        name: "Modal Strike".to_string(),
        card_id: CardId("test-modal-strike".to_string()),
        mana_cost: None,
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Choose one or more —\n\
             • Destroy target creature.\n\
             • Destroy target artifact.\n\
             • Destroy target land.\n\
             • Gain 1 life."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 4,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: destroy target creature.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 1: destroy target artifact.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 2: destroy target land.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 3: gain 1 life (no target).
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetCreature],
                    vec![TargetRequirement::TargetArtifact],
                    vec![TargetRequirement::TargetLand],
                    vec![],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Duplicate Destroy" — Instant, no mana cost. CR 700.2d: allows choosing the same mode
/// twice (`allow_duplicate_modes: true`). Single mode: destroy target creature.
fn duplicate_destroy_def() -> CardDefinition {
    CardDefinition {
        name: "Duplicate Destroy".to_string(),
        card_id: CardId("test-duplicate-destroy".to_string()),
        mana_cost: None,
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Choose one or both. You may choose the same mode more than once —\n\
             • Destroy target creature."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 2,
                allow_duplicate_modes: true,
                mode_costs: None,
                modes: vec![Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                }],
                mode_targets: Some(vec![vec![TargetRequirement::TargetCreature]]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Legacy Modal Spell" — Instant, no mana cost, `mode_targets: None` (pre-AC4 shape).
/// Choose one — Gain 3 life; or draw a card. Neither mode targets.
fn legacy_modal_def() -> CardDefinition {
    CardDefinition {
        name: "Legacy Modal Spell".to_string(),
        card_id: CardId("test-legacy-modal".to_string()),
        mana_cost: None,
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Choose one —\n• Gain 3 life.\n• Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ],
                mode_targets: None,
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A mandatory single-target "Destroy target creature" spell used as a same-turn response
/// to make a Modal Strike target illegal before it resolves (CR 608.2b setup).
fn mandatory_destroy_creature_def() -> CardDefinition {
    CardDefinition {
        name: "Mandatory Destroy Creature".to_string(),
        card_id: CardId("test-mandatory-destroy-creature".to_string()),
        mana_cost: None,
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Escalate Modal Strike" — Instant (matches `build_state`'s hardcoded object type), no base
/// mana cost, Escalate {0} (no mana required so the test doesn't need to fund a mana pool — the
/// cast is expected to be REJECTED before mana payment is ever attempted). Modal with
/// `mode_targets: Some(...)` AND the Escalate keyword — the unsupported combination from PB-AC4
/// fix-phase Finding 1 (MEDIUM).
fn escalate_modal_strike_def() -> CardDefinition {
    CardDefinition {
        name: "Escalate Modal Strike".to_string(),
        card_id: CardId("test-escalate-modal-strike".to_string()),
        mana_cost: None,
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Choose one or more —\n\
             • Destroy target creature.\n\
             • Destroy target artifact.\n\
             Escalate — Pay no mana."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost::default(),
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 2,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            cant_be_regenerated: false,
                        },
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            cant_be_regenerated: false,
                        },
                    ],
                    mode_targets: Some(vec![
                        vec![TargetRequirement::TargetCreature],
                        vec![TargetRequirement::TargetArtifact],
                    ]),
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Build a game state with `spell_defs` registered, each `(player, card_name)` pair placed
/// in that player's hand, plus `extra_objects` (creatures/artifacts/lands, etc.) added to
/// the battlefield. Players are derived from `players` (in order, e.g. `[p1, p2]`).
fn build_state(
    players: &[PlayerId],
    spell_defs: Vec<CardDefinition>,
    hand_spells: Vec<(PlayerId, &str)>,
    extra_objects: Vec<ObjectSpec>,
) -> GameState {
    let registry: Arc<CardRegistry> = CardRegistry::new(spell_defs.clone());

    let mut builder = GameStateBuilder::new().with_registry(registry);
    for &pl in players {
        builder = builder.add_player(pl);
    }

    for (owner, name) in hand_spells {
        let def = spell_defs
            .iter()
            .find(|d| d.name == name)
            .unwrap_or_else(|| panic!("spell def '{}' not registered", name));
        let spell = ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Hand(owner))
            .with_card_id(def.card_id.clone())
            .with_types(vec![CardType::Instant]);
        builder = builder.object(spell);
    }
    for obj in extra_objects {
        builder = builder.object(obj);
    }
    let mut state = builder
        .build()
        .expect("GameStateBuilder::build must succeed");
    state.turn.priority_holder = Some(players[0]);
    state
}

fn build_2p_state(
    spell_defs: Vec<CardDefinition>,
    hand_spells: Vec<&str>,
    extra_objects: Vec<ObjectSpec>,
) -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let hand: Vec<(PlayerId, &str)> = hand_spells.into_iter().map(|n| (p1, n)).collect();
    build_state(&[p1, p2], spell_defs, hand, extra_objects)
}

// ── T1: CR 601.2c/700.2c — targets only required for the chosen mode ──────────────────

/// CR 601.2c / CR 700.2c — Casting Modal Strike choosing ONLY "destroy target creature"
/// (mode 0) succeeds even though NO artifact and NO land exist on the battlefield. Before
/// AC4, the flat-union validator would have demanded targets for modes 1 and 2 too, making
/// this cast illegal in this board state (the wrong-game-state bug AC4 fixes).
#[test]
fn test_601_2c_modal_targets_only_for_chosen_mode() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = vec![modal_strike_def()];
    let creature = ObjectSpec::creature(p2, "Lone Creature", 2, 2);
    let state = build_2p_state(defs, vec!["Modal Strike"], vec![creature]);

    let spell_id = find_obj(&state, "Modal Strike");
    let creature_id = find_obj(&state, "Lone Creature");

    // No artifact, no land exist anywhere in this state — the union-of-all-modes
    // validator would reject this cast; the per-mode validator must accept it.
    let (state, _) = process_command(
        state,
        cast_modal(p1, spell_id, vec![Target::Object(creature_id)], vec![0]),
    )
    .expect("CR 700.2c: choosing only mode 0 must not require artifact/land targets");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        obj_in_graveyard(&state, "Lone Creature", p2),
        "CR 700.2c: mode 0's declared creature target must be destroyed"
    );
}

// ── T2: CR 700.2c — unchosen mode targets are not required ────────────────────────────

/// CR 700.2c — Choosing mode 3 (targetless "gain 1 life") requires declaring ZERO targets,
/// even though modes 0-2 (unchosen) each require a target of a type that doesn't exist on
/// the battlefield in this state.
#[test]
fn test_700_2c_unchosen_mode_targets_not_required() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = vec![modal_strike_def()];
    let state = build_2p_state(defs, vec!["Modal Strike"], vec![]);
    let spell_id = find_obj(&state, "Modal Strike");
    let initial_life = state.players[&p1].life_total;

    let (state, _) = process_command(state, cast_modal(p1, spell_id, vec![], vec![3]))
        .expect("CR 700.2c: choosing the targetless mode requires 0 declared targets");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 1,
        "CR 700.2c: mode 3 (gain 1 life) must have fired"
    );
}

// ── T3: CR 601.2c — wrong-type target rejected per mode (positional check) ────────────

/// CR 601.2c — Choosing mode 0 (destroy target creature) but declaring a LAND as the
/// target must be rejected: per-mode validation is positional (position 0 must satisfy
/// `mode_targets[0]` = `TargetCreature`), not best-fit across the whole card.
#[test]
fn test_601_2c_wrong_type_target_rejected_per_mode() {
    let p1 = p(1);
    let defs = vec![modal_strike_def()];
    let land = ObjectSpec::land(p1, "Wrong Type Land");
    let state = build_2p_state(defs, vec!["Modal Strike"], vec![land]);
    let spell_id = find_obj(&state, "Modal Strike");
    let land_id = find_obj(&state, "Wrong Type Land");

    let result = process_command(
        state,
        cast_modal(p1, spell_id, vec![Target::Object(land_id)], vec![0]),
    );
    assert!(
        result.is_err(),
        "CR 601.2c: a land target must be rejected for mode 0 (TargetCreature)"
    );
}

// ── T4: CR 700.2f — two chosen modes' targets sliced independently ────────────────────

/// CR 700.2f — Choosing modes 0 (creature) and 2 (land) with two distinct targets resolves
/// each mode against ITS OWN target only: the creature is destroyed by mode 0's effect, the
/// land is destroyed by mode 2's effect. A slicing bug (wrong offsets) would either reject
/// the cast at validation (position 1 failing `TargetLand`) or cross-apply the effects; both
/// destroys succeeding independently proves the slices are correct.
#[test]
fn test_700_2f_two_modes_two_targets_sliced_independently() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = vec![modal_strike_def()];
    let creature = ObjectSpec::creature(p2, "Sliced Creature", 2, 2);
    let land = ObjectSpec::land(p2, "Sliced Land");
    let artifact = ObjectSpec::artifact(p2, "Untouched Artifact");
    let state = build_2p_state(defs, vec!["Modal Strike"], vec![creature, land, artifact]);

    let spell_id = find_obj(&state, "Modal Strike");
    let creature_id = find_obj(&state, "Sliced Creature");
    let land_id = find_obj(&state, "Sliced Land");

    let (state, _) = process_command(
        state,
        cast_modal(
            p1,
            spell_id,
            vec![Target::Object(creature_id), Target::Object(land_id)],
            vec![0, 2],
        ),
    )
    .expect("CR 700.2f: modes 0 and 2 with matching-type targets must be castable");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        obj_in_graveyard(&state, "Sliced Creature", p2),
        "CR 700.2f: mode 0's target (the creature) must be destroyed"
    );
    assert!(
        obj_in_graveyard(&state, "Sliced Land", p2),
        "CR 700.2f: mode 2's target (the land) must be destroyed"
    );
    assert!(
        obj_on_battlefield(&state, "Untouched Artifact"),
        "CR 700.2f: the unchosen artifact must be untouched"
    );
}

// ── T5: CR 608.2b — partial illegal target skips only that mode ───────────────────────

/// CR 608.2b — Choosing modes 0 (creature) and 2 (land); the creature target is destroyed
/// by a separate spell BEFORE Modal Strike resolves. At resolution: the creature target is
/// illegal (no longer on the battlefield) — mode 0 does nothing; the land target is still
/// legal — mode 2 destroys it. The spell resolves normally (not a full fizzle) because at
/// least one target (the land) remains legal.
#[test]
fn test_608_2b_modal_partial_illegal_target_skips_only_that_mode() {
    let p1 = p(1);
    let p2 = p(2);
    let players = [p1, p2];
    let defs = vec![modal_strike_def(), mandatory_destroy_creature_def()];
    let creature = ObjectSpec::creature(p2, "Doomed Creature", 2, 2);
    let land = ObjectSpec::land(p2, "Surviving Land");
    let state = build_state(
        &players,
        defs,
        vec![(p1, "Modal Strike"), (p2, "Mandatory Destroy Creature")],
        vec![creature, land],
    );

    let modal_id = find_obj(&state, "Modal Strike");
    let remover_id = find_obj(&state, "Mandatory Destroy Creature");
    let creature_id = find_obj(&state, "Doomed Creature");
    let land_id = find_obj(&state, "Surviving Land");

    // P1 casts Modal Strike targeting the creature (mode 0) and the land (mode 2).
    let (state, _) = process_command(
        state,
        cast_modal(
            p1,
            modal_id,
            vec![Target::Object(creature_id), Target::Object(land_id)],
            vec![0, 2],
        ),
    )
    .expect("P1 casts Modal Strike targeting creature + land");

    // P1 passes; P2 casts Mandatory Destroy Creature targeting the SAME creature in response.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 })
        .expect("P1 passes priority after casting");
    let (state, _) = process_command(
        state,
        cast_modal(p2, remover_id, vec![Target::Object(creature_id)], vec![]),
    )
    .expect("P2 casts Mandatory Destroy Creature targeting the creature");

    // All pass: Mandatory Destroy Creature resolves first (LIFO stack) — creature dies.
    let (state, _) = pass_all(state, &players);
    assert!(
        obj_in_graveyard(&state, "Doomed Creature", p2),
        "the response spell must have destroyed the creature first"
    );
    assert!(
        obj_on_battlefield(&state, "Surviving Land"),
        "the land must still be on the battlefield before Modal Strike resolves"
    );

    // All pass again: Modal Strike resolves. CR 608.2b: the creature target is now
    // illegal (zone changed) — mode 0 is a no-op. The land target is still legal — mode 2
    // destroys it. The spell must resolve normally (not fizzle).
    let (state, events) = pass_all(state, &players);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "CR 608.2b: with one legal target remaining, Modal Strike must NOT fizzle"
    );
    assert!(
        obj_in_graveyard(&state, "Surviving Land", p2),
        "CR 608.2b: mode 2's still-legal target (the land) must be destroyed"
    );
    assert!(
        obj_in_graveyard(&state, "Doomed Creature", p2),
        "the creature remains in the graveyard (mode 0 was a no-op, not a second destroy)"
    );
}

// ── T6: CR 608.2b — all targets illegal fizzles the whole spell ───────────────────────

/// CR 608.2b — Choosing ONLY mode 0 (single target, the creature); that creature is
/// destroyed by a separate spell before Modal Strike resolves. With its only target now
/// illegal, Modal Strike fizzles entirely (no effect at all — CR 608.2b "If all its targets
/// ... are now illegal, the spell ... doesn't resolve").
#[test]
fn test_608_2b_modal_all_targets_illegal_fizzles() {
    let p1 = p(1);
    let p2 = p(2);
    let players = [p1, p2];
    let defs = vec![modal_strike_def(), mandatory_destroy_creature_def()];
    let creature = ObjectSpec::creature(p2, "Sole Target Creature", 2, 2);
    let state = build_state(
        &players,
        defs,
        vec![(p1, "Modal Strike"), (p2, "Mandatory Destroy Creature")],
        vec![creature],
    );

    let modal_id = find_obj(&state, "Modal Strike");
    let remover_id = find_obj(&state, "Mandatory Destroy Creature");
    let creature_id = find_obj(&state, "Sole Target Creature");
    let initial_life = state.players[&p1].life_total;

    let (state, _) = process_command(
        state,
        cast_modal(p1, modal_id, vec![Target::Object(creature_id)], vec![0]),
    )
    .expect("P1 casts Modal Strike targeting the only creature");

    let (state, _) = process_command(state, Command::PassPriority { player: p1 })
        .expect("P1 passes priority after casting");
    let (state, _) = process_command(
        state,
        cast_modal(p2, remover_id, vec![Target::Object(creature_id)], vec![]),
    )
    .expect("P2 casts Mandatory Destroy Creature targeting the same creature");

    // Resolve the response: creature dies.
    let (state, _) = pass_all(state, &players);
    assert!(obj_in_graveyard(&state, "Sole Target Creature", p2));

    // Resolve Modal Strike: its only target is now illegal — full fizzle.
    let (state, events) = pass_all(state, &players);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "CR 608.2b: Modal Strike's only target is illegal — it must fizzle"
    );
    assert_eq!(
        state.players[&p1].life_total, initial_life,
        "a fizzled spell has no effect (mode 3's life-gain never ran; mode 0 was the only chosen mode)"
    );
}

// ── T7: CR 700.2d — duplicate modes get independent target slices ─────────────────────

/// CR 700.2d — Choosing mode 0 twice (`allow_duplicate_modes: true`) with two DIFFERENT
/// creature targets destroys BOTH creatures — each instance of the duplicated mode gets its
/// own contiguous target slice, not a shared one.
#[test]
fn test_700_2d_duplicate_modes_independent_target_slices() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = vec![duplicate_destroy_def()];
    let creature_a = ObjectSpec::creature(p2, "Dupe Creature A", 2, 2);
    let creature_b = ObjectSpec::creature(p2, "Dupe Creature B", 3, 3);
    let state = build_2p_state(
        defs,
        vec!["Duplicate Destroy"],
        vec![creature_a, creature_b],
    );

    let spell_id = find_obj(&state, "Duplicate Destroy");
    let a_id = find_obj(&state, "Dupe Creature A");
    let b_id = find_obj(&state, "Dupe Creature B");

    let (state, _) = process_command(
        state,
        cast_modal(
            p1,
            spell_id,
            vec![Target::Object(a_id), Target::Object(b_id)],
            vec![0, 0],
        ),
    )
    .expect("CR 700.2d: choosing mode 0 twice with two distinct targets must be legal");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        obj_in_graveyard(&state, "Dupe Creature A", p2),
        "CR 700.2d: the first instance of mode 0 must destroy Creature A"
    );
    assert!(
        obj_in_graveyard(&state, "Dupe Creature B", p2),
        "CR 700.2d: the second instance of mode 0 must destroy Creature B"
    );
}

// ── T8: Hash — mode_targets contributes to the hash; schema version sentinel ──────────

/// CR N/A (hash infrastructure) — PB-AC4: `HASH_SCHEMA_VERSION` is 31 (this batch's bump).
/// Two `ModeSelection` values differing ONLY in `mode_targets` (`None` vs `Some`) must hash
/// to distinct values, confirming the new field contributes to `HashInto`.
#[test]
fn test_ac4_hash_distinguishes_mode_targets() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    assert_eq!(
        HASH_SCHEMA_VERSION, 35u8,
        "PB-AC4 bumped HASH_SCHEMA_VERSION 30->31 (ModeSelection.mode_targets, CR 700.2c/700.2f). \
         If you bumped again, update this test and state/hash.rs history."
    );

    let base_modes = ModeSelection {
        min_modes: 1,
        max_modes: 1,
        allow_duplicate_modes: false,
        mode_costs: None,
        modes: vec![Effect::DestroyPermanent {
            target: EffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        }],
        mode_targets: None,
    };
    let with_mode_targets = ModeSelection {
        mode_targets: Some(vec![vec![TargetRequirement::TargetCreature]]),
        ..base_modes.clone()
    };
    let with_different_mode_targets = ModeSelection {
        mode_targets: Some(vec![vec![TargetRequirement::TargetArtifact]]),
        ..base_modes.clone()
    };

    let hash_of = |ms: &ModeSelection| -> [u8; 32] {
        let mut hasher = Hasher::new();
        ms.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let h_none = hash_of(&base_modes);
    let h_some_creature = hash_of(&with_mode_targets);
    let h_some_artifact = hash_of(&with_different_mode_targets);

    assert_ne!(
        h_none, h_some_creature,
        "mode_targets: None vs Some must hash differently"
    );
    assert_ne!(
        h_some_creature, h_some_artifact,
        "different mode_targets contents must hash differently"
    );
}

// ── T9: Backward compat — mode_targets: None spells are unaffected ────────────────────

/// CR 700.2a — A legacy modal spell (`mode_targets: None`) behaves exactly as it did before
/// PB-AC4: choosing mode 0 (gain 3 life) fires only that mode, with the flat (legacy) target
/// path taken (here: no targets required, since neither mode targets).
#[test]
fn test_ac4_backward_compat_mode_targets_none_unaffected() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = vec![legacy_modal_def()];
    let state = build_2p_state(defs, vec!["Legacy Modal Spell"], vec![]);
    let spell_id = find_obj(&state, "Legacy Modal Spell");
    let initial_life = state.players[&p1].life_total;

    let (state, _) = process_command(state, cast_modal(p1, spell_id, vec![], vec![0]))
        .expect("legacy modal spell (mode_targets: None) must cast exactly as before AC4");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "mode 0 (GainLife 3) must have fired; mode_targets: None is unaffected by AC4"
    );
}

// ── T10: Multiplayer — choosing a subset of modes across different opponents ──────────

/// CR 700.2c (multiplayer coverage) — In a 4-player game, P1 casts Modal Strike choosing
/// modes 0 and 2, destroying P2's creature and P3's land. P4 is untouched, and no target for
/// the unchosen artifact mode is required from any player's permanents.
#[test]
fn test_700_2c_multiplayer_choose_subset_across_opponents() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];
    let defs = vec![modal_strike_def()];
    let creature = ObjectSpec::creature(p2, "P2 Creature", 2, 2);
    let land = ObjectSpec::land(p3, "P3 Land");
    let p4_artifact_free_marker = ObjectSpec::creature(p4, "P4 Untouched Creature", 1, 1);
    let state = build_state(
        &players,
        defs,
        vec![(p1, "Modal Strike")],
        vec![creature, land, p4_artifact_free_marker],
    );

    let spell_id = find_obj(&state, "Modal Strike");
    let creature_id = find_obj(&state, "P2 Creature");
    let land_id = find_obj(&state, "P3 Land");

    let (state, _) = process_command(
        state,
        cast_modal(
            p1,
            spell_id,
            vec![Target::Object(creature_id), Target::Object(land_id)],
            vec![0, 2],
        ),
    )
    .expect("CR 700.2c: choosing modes 0+2 across two opponents' permanents must be legal");

    let (state, _) = pass_all(state, &players);
    assert!(
        obj_in_graveyard(&state, "P2 Creature", p2),
        "CR 700.2c: P2's creature (mode 0's target) must be destroyed"
    );
    assert!(
        obj_in_graveyard(&state, "P3 Land", p3),
        "CR 700.2c: P3's land (mode 2's target) must be destroyed"
    );
    assert!(
        obj_on_battlefield(&state, "P4 Untouched Creature"),
        "P4's creature must be untouched — it was never targeted"
    );
}

// ── T11: Fix-phase Finding 1 (MEDIUM) — Escalate + mode_targets is fail-safe ───────────

/// CR 700.2c / 702.120a — PB-AC4 fix-phase Finding 1 (MEDIUM). Cast-time
/// `mode_targets_active` (casting.rs) has no Escalate branch, while resolution's
/// `chosen_mode_indices` (resolution.rs) does. Casting a spell that combines Escalate's
/// backward-compat path (Escalate paid via `AdditionalCost::EscalateModes`, `modes_chosen`
/// left empty) with `ModeSelection.mode_targets: Some(...)` must be REJECTED at cast time
/// with a typed error — not accepted and left to silently under-resolve mode 1 with an
/// empty target slice at resolution.
#[test]
fn test_700_2c_702_120a_escalate_with_mode_targets_rejected_at_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let def = escalate_modal_strike_def();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![def.clone()]);
    let creature = ObjectSpec::creature(p2, "Escalate Target Creature", 2, 2);
    let spell = ObjectSpec::card(p1, "Escalate Modal Strike")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(def.card_id.clone())
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Escalate);
    let mut state = GameStateBuilder::new()
        .with_registry(registry)
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(creature)
        .build()
        .expect("GameStateBuilder::build must succeed");
    state.turn.priority_holder = Some(p1);

    let spell_id = find_obj(&state, "Escalate Modal Strike");
    let creature_id = find_obj(&state, "Escalate Target Creature");

    // Pay Escalate for 1 extra mode (count: 1) via the backward-compat path (modes_chosen
    // left empty) — this is exactly the ambiguous combination Finding 1 identified.
    let result = process_command(
        state.clone(),
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
            additional_costs: vec![AdditionalCost::EscalateModes { count: 1 }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    let err = result.expect_err(
        "Finding 1 (MEDIUM): Escalate + ModeSelection.mode_targets must be rejected at cast \
         time (fail-safe), not accepted and silently under-resolved at resolution",
    );
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("Escalate") && err_msg.contains("mode_targets"),
        "the rejection must be the Finding-1 typed error (Escalate combined with \
         ModeSelection.mode_targets is not supported), not an unrelated validation failure: got {}",
        err_msg
    );

    // Sanity check: WITHOUT Escalate paid, the same card (which also has ordinary
    // `mode_targets`) casts normally — proving the reject is specific to the Escalate
    // combination, not a defect in the base `mode_targets` path.
    let (state, _) = process_command(
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
            modes_chosen: vec![0],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("without Escalate paid, the same mode_targets spell must cast normally");
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        obj_in_graveyard(&state, "Escalate Target Creature", p2),
        "mode 0's target must have been destroyed on the non-Escalate cast"
    );
}
