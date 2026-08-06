//! PB-DX25c (`OOS-DX25b-3`): legality probes for `rules::retarget::plan_target_change`.
//!
//! **Hard constraint (plan §5, stated because the batch is judged on it —
//! PB-DX25b's durable lesson): every probe below reaches the code through a
//! real `Command::CastSpell` and real `PassPriority` resolution.** T9c is the
//! ONLY probe permitted a hand-built `StackObject`, and its doc says exactly
//! what it therefore does not prove.
//!
//! **T5 (CR 115.3 distinctness at retarget) is DROPPED, per the plan's own
//! permission** (§5.2 T5, §9 checklist): a `must_change: true` victim is only
//! ever reachable via `TargetSpellWithSingleTarget` /
//! `TargetSpellOrAbilityWithSingleTarget`, both of which require the VICTIM
//! to have declared exactly ONE target at cast time. A victim with two
//! mandatory distinct slots (`TargetPermanentDistinctFrom`) therefore can
//! never satisfy the single-target gate in the first place -- there is no
//! real cast that reaches `plan_target_change` with `so.targets.len() > 1`.
//! CR 115.3 distinctness-at-retarget is exercised for free by
//! `casting::validate_targets_inner`'s own `enforce_inter_target_
//! distinctness` call (shared with cast-time validation, PB-DX25c plan
//! §3.2), but no PROBE in this file can discriminate it without a hand-built
//! fixture the hard constraint above forbids.
//!
//! **T11 (CR 608.2b `zone_at_cast` rebuilt) is FOLDED into T6** rather than
//! given its own fixture: T6's cross-kind redirect (a `TargetAny` victim
//! moving from a player, `zone_at_cast: None`, to a creature, `zone_at_cast:
//! Some(Battlefield)`) is exactly the "old and new zones differ" case T11
//! asks for, and T9b (`pb_dx25b_announced_stack_target_space.rs`) already
//! covers the same-zone case. Building a THIRD fixture for the identical
//! assertion would not discriminate anything T6 doesn't already discriminate.

use std::sync::Arc;

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::hash::HashInto;
use mtg_engine::state::stack::{StackObject, StackObjectKind};
use mtg_engine::state::test_util;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CardType, Color, Command, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, SpellTarget, Step, Target,
    TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_stack_obj_on_stack(state: &GameState, name_substr: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Stack && obj.characteristics.name.contains(name_substr)
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no ZoneId::Stack object containing '{}' found", name_substr))
}

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
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
}

/// Passes priority as whoever the state says currently holds it, repeatedly,
/// until the top of the stack resolves (the stack shrinks) -- robust to
/// player count and to a player being skipped for elimination, unlike a
/// hard-coded pass ORDER (`pb_dx25b_announced_stack_target_space.rs`'s
/// `pass_n` fixes the player list, which only works when every listed
/// player is still eligible to pass; T4 needs a conceded player skipped
/// automatically).
fn resolve_top_of_stack(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let start_len = state.stack_objects().len();
    let mut all_events = Vec::new();
    for _ in 0..20 {
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder to resolve the stack"));
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
        if state.stack_objects().len() < start_len {
            return (state, all_events);
        }
    }
    panic!(
        "stack did not resolve after 20 passes (started at {} objects, still {}); events: {:?}",
        start_len,
        state.stack_objects().len(),
        all_events
    );
}

/// A "destroy target creature" instant, mirroring
/// `pb_dx25b_announced_stack_target_space.rs`'s T9 fixture. Used as the
/// victim spell for T1/T2 (object-branch legality checks).
fn destroy_creature_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: Destroy target creature."),
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

/// A "target opponent loses 3 life" instant with `TargetRequirement::
/// TargetOpponent` -- used by T3 (the requirement-check half of the player
/// branch) and T4 (the has_conceded half).
fn life_loss_player_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: Target opponent loses 3 life."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::LoseLife {
                player: mtg_engine::PlayerTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetOpponent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "target player loses 3 life" instant -- `TargetRequirement::TargetPlayer`,
/// UNCONDITIONAL (unlike `life_loss_player_def`'s `TargetOpponent`, every
/// player including the caster's own ally satisfies this). Used by T3b to
/// discriminate the chooser-first PREFERENCE (§3.3) from plain seat order:
/// with no requirement restricting candidacy, a chooser who is legal but NOT
/// first in `turn_order` still must be tried first if the preference is real.
fn any_player_life_loss_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: Target player loses 3 life."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::LoseLife {
                player: mtg_engine::PlayerTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "deal 3 damage to any target" instant -- CR 115.4's `TargetAny`. Used
/// by T6 for the cross-kind (player<->object) redirect probe.
fn any_target_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: deals 3 damage to any target."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
                source: None,
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "fizzle target BLUE spell" instant -- `TargetSpellWithFilter` restricted
/// to blue spells. Used by T7. **Not** `TargetSpellWithSingleTarget`: a
/// discovered structural fact (recorded on T7 itself) makes that requirement
/// unable to observe the ACTIVELY-RESOLVING spell as a candidate, because its
/// own `StackObject` entry has already been popped by the time its effect
/// runs -- `TargetSpellWithFilter` only ever consults `state.objects` +
/// layer-resolved characteristics, never `state.stack_objects`, so it has no
/// such blind spot.
fn target_spell_with_filter_def(name: &str, card_id: &str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: does nothing to target blue spell."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![TargetRequirement::TargetSpellWithFilter(
                mtg_engine::TargetFilter {
                    colors: Some(colors.into_iter().collect()),
                    ..Default::default()
                },
            )],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "fizzle target spell" instant -- PLAIN `TargetRequirement::TargetSpell`,
/// no filter at all. Used by T7b: the exact shape `OOS-DX25c-5`'s registry
/// row names as the concrete failure scenario (Counterspell, unfiltered), so
/// this is the requirement variant the new `self_id` guard must be proven
/// against directly, not through `TargetSpellWithFilter`'s colour side door.
fn target_spell_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: fizzles target spell."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "change the target of target spell with a single target" instant --
/// structurally identical to Misdirection's own Spell ability, but a
/// SEPARATE card def so T8 can build a triangle of three stack objects
/// (decoy, this "clone", and a real Misdirection) without reusing
/// Misdirection's card id twice.
fn misdirection_clone_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: change the target of target spell with a single target."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ChangeTargets {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                must_change: true,
            },
            targets: vec![TargetRequirement::TargetSpellWithSingleTarget],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn life_of(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
        .life_total
}

// ── T1: hexproof blocks the redirect (CR 702.11b) ──────────────────────────

/// CR 702.11b / CR 115.7a — a hexproof creature controlled by someone OTHER
/// than the victim spell's controller is not a legal redirect candidate.
/// Discriminates the CHARACTERISTIC half of `validate_object_satisfies_
/// requirement`, a different code path from the plain type check T9/T9b
/// exercise.
#[test]
fn t1_hexproof_blocks_the_redirect() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let destroy = destroy_creature_def("PB-DX25c T1 Destroy", "pb-dx25c-t1-destroy");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), destroy.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::creature(p1, "T1 Victim Creature", 2, 2))
        .object(
            ObjectSpec::creature(p3, "T1 Hexproof Creature", 3, 3)
                .with_keyword(mtg_engine::KeywordAbility::Hexproof),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T1 Destroy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_obj(&state, "T1 Victim Creature");
    let destroy_hand_id = find_obj(&state, "PB-DX25c T1 Destroy");

    // p2 casts "destroy target creature" at p1's creature.
    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(creature_id)],
    )
    .unwrap_or_else(|e| panic!("Destroy cast must succeed: {:?}", e));
    let destroy_card_id = find_stack_obj_on_stack(&state, "T1 Destroy");

    // p1 casts Misdirection targeting that spell.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(destroy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    // Resolve Misdirection: the ONLY other creature is p3's HEXPROOF creature,
    // and p3 != p2 (the victim spell's controller), so CR 702.11b blocks it --
    // no legal alternative exists, CR 115.7a's fallback applies.
    let (state, resolve_events) = resolve_top_of_stack(state);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "CR 702.11b: a hexproof creature controlled by someone other than the \
         victim's controller must not be offered as a redirect candidate -- \
         got: {:?}",
        resolve_events
    );

    let (state, _) = resolve_top_of_stack(state);
    assert!(
        !state.objects().values().any(
            |o| o.characteristics.name == "T1 Victim Creature" && o.zone == ZoneId::Battlefield
        ),
        "the original creature must still be destroyed"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "T1 Hexproof Creature"
                && o.zone == ZoneId::Battlefield),
        "the hexproof creature must survive -- it was never a legal candidate"
    );
}

// ── T2: protection from the victim's colour blocks the redirect ────────────

/// CR 702.16b — a creature with protection from the victim spell's colour is
/// not a legal redirect candidate. Discriminates the `source_chars` argument
/// of `plan_target_change` step 4 -- the ONLY probe in this file that does,
/// so if `source_chars` were passed as `None`, this is the test that catches
/// it (V4 of the plan's revert matrix).
#[test]
fn t2_protection_from_victim_colour_blocks_the_redirect() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let destroy = destroy_creature_def("PB-DX25c T2 Destroy", "pb-dx25c-t2-destroy");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), destroy.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::creature(p1, "T2 Victim Creature", 2, 2))
        .object(
            ObjectSpec::creature(p3, "T2 Protected Creature", 3, 3).with_keyword(
                mtg_engine::KeywordAbility::ProtectionFrom(
                    mtg_engine::state::types::ProtectionQuality::FromColor(Color::Black),
                ),
            ),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T2 Destroy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy.card_id.clone())
                .with_types(vec![CardType::Instant])
                // ObjectSpec::card() is naked (gotchas-infra.md): colors are
                // NOT re-derived from CardDefinition.mana_cost at cast time
                // (state/mod.rs only does that for the prototype-revert
                // path) -- set explicitly so source_chars carries Black for
                // the CR 702.16b check this test exists to exercise.
                .with_colors(vec![Color::Black]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_obj(&state, "T2 Victim Creature");
    let destroy_hand_id = find_obj(&state, "PB-DX25c T2 Destroy");

    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(creature_id)],
    )
    .unwrap_or_else(|e| panic!("Destroy cast must succeed: {:?}", e));
    let destroy_card_id = find_stack_obj_on_stack(&state, "T2 Destroy");

    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(destroy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let (state, resolve_events) = resolve_top_of_stack(state);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "CR 702.16b: a creature with protection from black must not be offered \
         as a redirect candidate for a BLACK victim spell -- got: {:?}",
        resolve_events
    );

    let (state, _) = resolve_top_of_stack(state);
    assert!(
        !state.objects().values().any(
            |o| o.characteristics.name == "T2 Victim Creature" && o.zone == ZoneId::Battlefield
        ),
        "the original creature must still be destroyed"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "T2 Protected Creature"
                && o.zone == ZoneId::Battlefield),
        "the protected creature must survive -- it was never a legal candidate"
    );
}

// ── T3: CR 115.7a on the PLAYER branch, TargetOpponent requirement ─────────

/// CR 109.5 / 115.7a — a "target opponent" victim redirected by its OWN
/// caster must not land on that caster: p1 is not an opponent of p1 (CR
/// 109.5 makes "you" on the victim spell mean the victim's OWN controller,
/// not the player doing the redirecting -- here they happen to be the same
/// player, which is exactly the configuration that catches a `ctx.
/// controller`-preferring bug). Before PB-DX25c the player branch preferred
/// `ctx.controller` unconditionally, with no requirement check at all --
/// p1 would have been offered here despite CR 102.3/601.2c forbidding a
/// TargetOpponent spell from targeting its own caster.
#[test]
fn t3_target_opponent_requirement_on_the_player_branch() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let life_loss = life_loss_player_def("PB-DX25c T3 Life Loss", "pb-dx25c-t3-lifeloss");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), life_loss.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "PB-DX25c T3 Life Loss")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(life_loss.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let life_loss_hand_id = find_obj(&state, "PB-DX25c T3 Life Loss");

    // p1 casts "target opponent loses 3 life" at p2.
    let (state, _) = cast(state, p1, life_loss_hand_id, vec![Target::Player(p2)])
        .unwrap_or_else(|e| panic!("Life Loss cast must succeed: {:?}", e));
    let life_loss_card_id = find_stack_obj_on_stack(&state, "T3 Life Loss");

    // p1 ALSO casts Misdirection (in response to their own spell), redirecting it.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(life_loss_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let p1_life_before = life_of(&state, p1);
    let p3_life_before = life_of(&state, p3);
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Misdirection must redirect: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Player(p3),
        "AT HEAD (pre-fix) the player branch preferred ctx.controller (p1) \
         unconditionally with no requirement check -- p1 IS the caster \
         (so.controller), so CR 102.3/601.2c make p1 illegal as their OWN \
         TargetOpponent. p2 is the CURRENT target (excluded by construction). \
         p3 is the only remaining, legal opponent, so the redirect must land \
         there."
    );

    let (state, _) = resolve_top_of_stack(state);
    assert_eq!(
        life_of(&state, p3),
        p3_life_before - 3,
        "p3 (the redirected target) must lose 3 life"
    );
    assert_eq!(
        life_of(&state, p1),
        p1_life_before,
        "p1 (the illegal wrong-way-round HEAD target) must be untouched"
    );
}

// ── T3b: chooser-first PREFERENCE discriminated from plain seat order ──────

/// §3.3 -- the chooser-first preference is a deliberately preserved
/// observable, kept so this batch changes what is LEGAL, not what is
/// PREFERRED among legal candidates. `/review` (Finding T3) found it had
/// ZERO discriminating coverage: T1/T2/T3/T4's chooser always happened to
/// double as either the first legal candidate in seat order, or a candidate
/// excluded from mattering, so a build that dropped the chooser-first special
/// case (V9 in the revert matrix) left every existing test green.
///
/// This fixture is built specifically to fail V9: turn_order = [p1, p2, p3,
/// p4] (p3 is NOT first), the chooser (Misdirection's caster) is p3, and the
/// victim uses `TargetRequirement::TargetPlayer` -- UNCONDITIONAL, so p1
/// (first in seat order) is just as legal a candidate as p3 (the chooser).
/// If the redirect lands on p3, the chooser-first preference is real and
/// discriminated; if it lands on p1, the code has silently fallen back to
/// plain seat order.
#[test]
fn t3b_chooser_first_preference_beats_seat_order() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let life_loss = any_player_life_loss_def("PB-DX25c T3b Life Loss", "pb-dx25c-t3b-lifeloss");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), life_loss.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p3,
            ManaPool {
                colorless: 3,
                blue: 2,
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p3, "Misdirection")
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "PB-DX25c T3b Life Loss")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(life_loss.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let life_loss_hand_id = find_obj(&state, "PB-DX25c T3b Life Loss");

    // p1 casts "target player loses 3 life" at p4 -- p4 is the CURRENT
    // target, unrelated to both the chooser (p3) and the seat-order-first
    // legal candidate (p1 itself, still legal since TargetPlayer has no
    // self-exclusion).
    let (state, _) = cast(state, p1, life_loss_hand_id, vec![Target::Player(p4)])
        .unwrap_or_else(|e| panic!("Life Loss cast must succeed: {:?}", e));
    let life_loss_card_id = find_stack_obj_on_stack(&state, "T3b Life Loss");

    // p3 (NOT the victim's caster, NOT first in turn_order, NOT the current
    // target) Misdirects it.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p3,
        misdirection_hand_id,
        vec![Target::Object(life_loss_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let p1_life_before = life_of(&state, p1);
    let p3_life_before = life_of(&state, p3);
    let p4_life_before = life_of(&state, p4);
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Misdirection must redirect: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Player(p3),
        "the chooser (p3) is offered FIRST regardless of seat order -- \
         turn_order is [p1, p2, p3, p4], so a build that fell back to plain \
         seat order would land on p1 instead (p1 is legal: TargetPlayer has \
         no self-exclusion, and p1 is not the current target p4). This is \
         the fixture V9 (revert matrix) was missing: dropping the \
         chooser-first special case now reddens THIS assertion, where it \
         left every pre-existing probe green."
    );

    let (state, _) = resolve_top_of_stack(state);
    assert_eq!(
        life_of(&state, p3),
        p3_life_before - 3,
        "p3 (the chooser, and the redirected target) must lose 3 life"
    );
    assert_eq!(
        life_of(&state, p1),
        p1_life_before,
        "p1 (the seat-order-first candidate, wrongly picked by a \
         preference-less implementation) must be untouched"
    );
    assert_eq!(
        life_of(&state, p4),
        p4_life_before,
        "p4 (the original target) must be untouched"
    );
}

// ── T4: CR 104.3a / CR 115.7a on the PLAYER branch, has_conceded ───────────

/// CR 104.3a / CR 115.7a — a conceded player is not a legal redirect
/// candidate. Before PB-DX25c the player branch checked `has_lost` only;
/// `handle_concede` sets `has_conceded`, not `has_lost`, so a conceded
/// player was still offered.
///
/// **Design note**: to discriminate this from ordinary chooser-preference
/// (T3's subject), the CHOOSER must be excluded from candidacy so the scan
/// actually reaches the conceded player's SLOT in the order. Misdirection is
/// cast by p3, the victim spell's CURRENT target (chooser == current, so
/// candidates.find's `!= current` filter excludes it regardless of legality)
/// -- p1 (conceded) is pushed FIRST in the remaining turn-order scan
/// (`turn_order = [p1, p2, p3, p4]`), so a version of the code that checked
/// only `has_lost` would offer p1 before ever reaching p4.
#[test]
fn t4_conceded_player_is_not_a_legal_redirect_candidate() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let life_loss = life_loss_player_def("PB-DX25c T4 Life Loss", "pb-dx25c-t4-lifeloss");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), life_loss.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p3,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p3, "Misdirection")
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T4 Life Loss")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(life_loss.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // p1 concedes BEFORE anything else happens -- p1 never needs to act.
    // Premise check: this test documents its own premise, per the plan's
    // instruction -- `Command::Concede` sets has_conceded, NOT has_lost.
    let (state, _) = process_command(state, Command::Concede { player: p1 })
        .unwrap_or_else(|e| panic!("Concede must succeed: {:?}", e));
    let p1_state = state.players().get(&p1).unwrap();
    assert!(
        p1_state.has_conceded,
        "premise: Concede must set has_conceded"
    );
    assert!(!p1_state.has_lost, "premise: Concede must NOT set has_lost");

    let life_loss_hand_id = find_obj(&state, "PB-DX25c T4 Life Loss");
    // p2 casts "target opponent loses 3 life" at p3 (the eventual chooser).
    let (state, _) = cast(state, p2, life_loss_hand_id, vec![Target::Player(p3)])
        .unwrap_or_else(|e| panic!("Life Loss cast must succeed: {:?}", e));
    let life_loss_card_id = find_stack_obj_on_stack(&state, "T4 Life Loss");

    // p3 (the CURRENT target) casts Misdirection in response, redirecting
    // their own targeting.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p3,
        misdirection_hand_id,
        vec![Target::Object(life_loss_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let p4_life_before = life_of(&state, p4);
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
        _ => None,
    });
    let new_target =
        new_target.unwrap_or_else(|| panic!("Misdirection must redirect: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Player(p4),
        "candidate order is [p3(chooser, ==current, excluded), p1(conceded, \
         CR 104.3a), p2(fails TargetOpponent -- p2 IS the caster, CR 102.3 \
         self-exclusion), p4(legal)]. This pins the WHOLE legality \
         delegation (`validate_targets_inner` via `plan_target_change`) \
         landing on p4, not specifically the has_conceded filter in \
         isolation -- `validate_mapped_targets:6265` independently rejects a \
         conceded player downstream of `retarget_candidates`, so \
         `retarget_candidates`'s OWN has_conceded check is defense-in-depth \
         with no test that discriminates it alone (confirmed by execution: \
         V7 in the revert matrix drops that candidate-building filter and \
         this assertion stays GREEN, because the trial for p1 still fails \
         validation one layer downstream). A regression that dropped \
         has_conceded from BOTH layers would still redden this test, just \
         not by isolating which layer caught it."
    );

    let (state, _) = resolve_top_of_stack(state);
    assert_eq!(
        life_of(&state, p4),
        p4_life_before - 3,
        "p4 (the redirected target) must lose 3 life"
    );
}

// ── T6: cross-kind redirect (CR 115.4 TargetAny) + zone_at_cast rebuild ────

/// CR 115.4 / 115.7a / 608.2b — a `TargetAny` victim originally targeting a
/// CREATURE (an object) can be redirected onto a PLAYER: the unified
/// candidate universe (§3.3) is genuinely cross-kind, and the "players tried
/// before objects" ordering rule (§3.2) means the chooser -- a trivially
/// legal `TargetAny` candidate, since `TargetAny` places no restriction on
/// which player -- is offered ahead of any other object. Also discriminates
/// CR 608.2b (§3.2 step 8, folds T11): the ORIGINAL target's `zone_at_cast`
/// was `Some(Battlefield)` (a creature); the NEW target's must be rebuilt as
/// `None` (a player has no zone).
#[test]
fn t6_cross_kind_redirect_lands_on_a_player_and_rebuilds_zone_at_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let any_target = any_target_def("PB-DX25c T6 Any Target", "pb-dx25c-t6-anytarget");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), any_target.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::creature(p2, "T6 Victim Creature", 2, 2))
        .object(
            ObjectSpec::card(p2, "PB-DX25c T6 Any Target")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(any_target.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_obj(&state, "T6 Victim Creature");
    let any_target_hand_id = find_obj(&state, "PB-DX25c T6 Any Target");
    // p2 casts "deal 3 damage to any target" at their OWN creature (an
    // object -- the current target this test starts from).
    let (state, _) = cast(
        state,
        p2,
        any_target_hand_id,
        vec![Target::Object(creature_id)],
    )
    .unwrap_or_else(|e| panic!("Any Target cast must succeed: {:?}", e));
    let any_target_card_id = find_stack_obj_on_stack(&state, "T6 Any Target");

    // p1 casts Misdirection targeting it.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(any_target_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let p1_life_before = life_of(&state, p1);
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Misdirection must redirect: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Player(p1),
        "CR 115.4: the redirect must be able to cross from an OBJECT current \
         target to a PLAYER new target -- p1 (the chooser) is a trivially \
         legal TargetAny candidate and is tried before any object"
    );
    assert_eq!(
        new_target[0].zone_at_cast, None,
        "CR 608.2b: zone_at_cast must be rebuilt from the NEW target's own \
         zone (None, a player has no zone), not copied from the OLD \
         target's zone (Some(Battlefield)) -- this is the cross-zone case \
         T11 asks for"
    );

    let (state, _) = resolve_top_of_stack(state);
    assert_eq!(
        life_of(&state, p1),
        p1_life_before - 3,
        "p1 (the redirected-onto player) must take the 3 damage"
    );
    assert!(
        state.objects().values().any(
            |o| o.characteristics.name == "T6 Victim Creature" && o.zone == ZoneId::Battlefield
        ),
        "the original creature target must survive -- the redirect moved off it"
    );
}

// ── T7: Misdirection is itself a legal candidate (2004-10-04 ruling) ───────

/// "You can choose to make a spell on the stack target this spell (if such a
/// target choice would be legal had the spell been cast while this spell was
/// on the stack)." Misdirection's own card, still resident in `ZoneId::Stack`
/// at the moment its OWN effect executes (`resolution.rs`: "Execute the
/// card's effect before it moves to its final zone"), must be a legal
/// candidate for a `TargetSpellWithFilter` victim whose filter it satisfies.
///
/// **Discovered structural fact, worth recording because it decided this
/// test's shape**: `TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWith
/// SingleTarget` CANNOT observe the actively-resolving spell as a candidate.
/// Both requirements resolve the candidate through `stack_index_for_
/// announced_target(&state.stack_objects, id)` -- but `resolution.rs` pops a
/// `StackObject` off `state.stack_objects` BEFORE running its effect (the
/// popped entry is kept in a local `stack_obj` variable, not the vector), so
/// while Misdirection's own CARD is still in `state.objects` with
/// `zone == Stack` (confirmed empirically), its STACK-OBJECT ENTRY is
/// already gone by the time `plan_target_change` runs during its own
/// resolution -- `stack_index_for_announced_target` returns `None` for it,
/// and both single-target requirements report "not a spell". Plain
/// `TargetSpell` / `TargetSpellWithFilter` have no such blind spot (they
/// only ever consult `state.objects` + layer-resolved characteristics), so
/// this probe uses `TargetSpellWithFilter` instead.
///
/// **Historical note, corrected by fix cycle 2 (`OOS-DX25c-5` CLOSED)**: this
/// fixture's colour filter was ORIGINALLY engineered to double as a
/// self-exclusion workaround -- at the time this test was written,
/// `validate_object_satisfies_requirement`'s `TargetSpell`/`TargetSpellWith
/// Filter` arm took no `self_id` check at all, so with no filter a victim's
/// own card (a smaller `ObjectId` than Misdirection's, since Misdirection is
/// always cast AFTER the victim already exists on the stack) would have been
/// legally redirected onto ITSELF before ever reaching Misdirection's card.
/// That gap is now closed at the source (`casting.rs`'s `TargetSpell`/
/// `TargetSpellWithFilter` arm gained a `self_id` guard mirroring the two
/// single-target arms) -- self-exclusion is enforced STRUCTURALLY now, not
/// as an artefact of this fixture's colour engineering. See
/// `t7b_plain_target_spell_victim_cannot_redirect_onto_its_own_card` for the
/// probe that discriminates the guard directly, on the PLAIN `TargetSpell`
/// variant (no filter at all) the guard's own commit named as the concrete
/// failure scenario. This fixture's filter is kept as-is -- it still proves
/// the 2004-10-04 ruling under a real (if now redundant) colour constraint,
/// and rebuilding it around the guard would not add coverage T7b doesn't
/// already provide.
#[test]
fn t7_misdirection_is_itself_a_legal_candidate() {
    let p1 = p(1);
    let p2 = p(2);

    let decoy = any_target_def("PB-DX25c T7 Decoy", "pb-dx25c-t7-decoy");
    let victim = target_spell_with_filter_def(
        "PB-DX25c T7 Victim",
        "pb-dx25c-t7-victim",
        vec![Color::Blue],
    );
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), decoy.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T7 Decoy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(decoy.card_id.clone())
                .with_types(vec![CardType::Instant])
                // The victim's TargetSpellWithFilter(blue) requirement must
                // be satisfiable at CAST time too (decoy is its initial
                // target) -- give it explicit blue, same reasoning as
                // Misdirection's fixture above.
                .with_colors(vec![Color::Blue]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T7 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant])
                // Real Misdirection is {3}{U}{U} -- give the hand-built
                // fixture object the derived colour explicitly, since
                // ObjectSpec::card() is naked and never calls
                // enrich_spec_from_def (gotchas-infra.md).
                .with_colors(vec![Color::Blue]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // decoy: "deal 3 damage to any target" at p1 (explicitly blue -- see the
    // fixture comment above -- so casting the victim at it satisfies its
    // TargetSpellWithFilter(blue) requirement).
    let decoy_hand_id = find_obj(&state, "PB-DX25c T7 Decoy");
    let (state, _) = cast(state, p2, decoy_hand_id, vec![Target::Player(p1)])
        .unwrap_or_else(|e| panic!("Decoy cast must succeed: {:?}", e));
    let decoy_card_id = find_stack_obj_on_stack(&state, "T7 Decoy");

    let victim_hand_id = find_obj(&state, "PB-DX25c T7 Victim");
    let (state, _) = cast(
        state,
        p2,
        victim_hand_id,
        vec![Target::Object(decoy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Victim cast must succeed: {:?}", e));
    let victim_card_id = find_stack_obj_on_stack(&state, "T7 Victim");

    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let misdirection_card_id = find_stack_obj_on_stack(&state, "Misdirection");
    let (_, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Victim must redirect: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Object(misdirection_card_id),
        "2004-10-04 ruling: Misdirection's own card, still in ZoneId::Stack \
         at the moment its effect executes, must be a legal candidate for \
         the redirect -- with decoy(current, excluded) and the victim's own \
         (colourless) card failing the blue filter, Misdirection's (blue) \
         card is the only remaining candidate that satisfies \
         TargetSpellWithFilter"
    );
}

// ── T7b: OOS-DX25c-5 -- the plain-TargetSpell self-redirect guard ──────────

/// **Closes `OOS-DX25c-5`** (`/review` Finding E2, fix cycle 2): a victim
/// with a PLAIN `TargetRequirement::TargetSpell` (no filter, so no colour
/// side door like T7's) must never be redirected onto its own card. This is
/// the exact shape the registry row's concrete failure scenario names
/// (Counterspell, unfiltered) -- p2's victim targets p3's decoy; p1
/// Misdirects the victim. Candidates in ascending `ObjectId` order, current
/// target excluded: the victim's OWN card (would have been picked first,
/// pre-fix, since it is always minted before Misdirection's), then
/// Misdirection's own card. With the guard, the victim's own card is
/// excluded and Misdirection's card -- itself a legal `TargetSpell`
/// candidate, no filter to fail -- is the only one left.
#[test]
fn t7b_plain_target_spell_victim_cannot_redirect_onto_its_own_card() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let decoy = any_target_def("PB-DX25c T7b Decoy", "pb-dx25c-t7b-decoy");
    let victim = target_spell_def("PB-DX25c T7b Victim", "pb-dx25c-t7b-victim");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), decoy.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                blue: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p3,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p3, "PB-DX25c T7b Decoy")
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(decoy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T7b Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_colors(vec![Color::Blue]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let decoy_hand_id = find_obj(&state, "PB-DX25c T7b Decoy");
    let (state, _) = cast(state, p3, decoy_hand_id, vec![Target::Player(p1)])
        .unwrap_or_else(|e| panic!("Decoy cast must succeed: {:?}", e));
    let decoy_card_id = find_stack_obj_on_stack(&state, "T7b Decoy");

    let victim_hand_id = find_obj(&state, "PB-DX25c T7b Victim");
    let (state, _) = cast(
        state,
        p2,
        victim_hand_id,
        vec![Target::Object(decoy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Victim cast must succeed: {:?}", e));
    let victim_card_id = find_stack_obj_on_stack(&state, "T7b Victim");

    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));
    let misdirection_card_id = find_stack_obj_on_stack(&state, "Misdirection");

    let (_, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Victim must redirect: {:?}", resolve_events));
    assert_ne!(
        new_target[0].target,
        Target::Object(victim_card_id),
        "OOS-DX25c-5: a plain TargetSpell victim must never be redirected \
         onto its OWN card -- self_id exclusion must fire here even with no \
         filter to fall back on (Misdirection 2004-10-04: \"You can't make a \
         spell which is on the stack target itself\")"
    );
    assert_eq!(
        new_target[0].target,
        Target::Object(misdirection_card_id),
        "with decoy(current, excluded) and the victim's own card excluded \
         via self_id, Misdirection's own card -- a legal TargetSpell \
         candidate with no filter to fail -- is the only remaining \
         candidate"
    );
}

// ── T8: self-targeting is still refused (CR 601.2c) ────────────────────────

/// The victim spell cannot be retargeted onto its own card (`self_id`) --
/// discriminates `plan_target_change` step 4's `victim_card` argument.
///
/// **FOUR stack objects, not three -- measured, not assumed.** An earlier
/// draft used three (decoy/clone/Misdirection) and asserted `new_target !=
/// self` only inside an `if let Some(...)`, reasoning that CR 115.7a's
/// no-change fallback would ALSO prove self-exclusion if Misdirection's own
/// card were the only other candidate. Executed, that draft produced **zero**
/// `TargetsChanged` events -- vacuously "passing" without exercising
/// anything, for the SAME structural reason T7's doc records: Misdirection's
/// own `StackObject` entry has already been popped by the time its effect
/// runs, so it ALSO fails `TargetSpellWithSingleTarget`'s "is this a spell"
/// check (for "entry not found", not for self-exclusion) -- with clone(self,
/// excluded) and Misdirection(not-found) both gone, nothing remained and the
/// whole plan returned `None`. A fourth object -- "T8 Alternative", cast
/// AFTER the clone and BEFORE Misdirection so its `ObjectId` sits between
/// them, structurally untouched by any of this -- gives self-exclusion a
/// REAL alternative to be measured against, so the redirect fires and both
/// halves of the CR 601.2c claim (never-self, correctly-elsewhere) are
/// positive assertions rather than one conditional and a comment.
#[test]
fn t8_self_targeting_is_still_refused() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let decoy = any_target_def("PB-DX25c T8 Decoy", "pb-dx25c-t8-decoy");
    let clone_ = misdirection_clone_def("PB-DX25c T8 Clone", "pb-dx25c-t8-clone");
    let alternative = any_target_def("PB-DX25c T8 Alternative", "pb-dx25c-t8-alternative");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![
        misdirection.clone(),
        decoy.clone(),
        clone_.clone(),
        alternative.clone(),
    ]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                colorless: 2,
                blue: 1,
                red: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p3,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T8 Decoy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(decoy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T8 Clone")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(clone_.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p3, "PB-DX25c T8 Alternative")
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(alternative.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let decoy_hand_id = find_obj(&state, "PB-DX25c T8 Decoy");
    let (state, _) = cast(state, p2, decoy_hand_id, vec![Target::Player(p3)])
        .unwrap_or_else(|e| panic!("Decoy cast must succeed: {:?}", e));
    let decoy_card_id = find_stack_obj_on_stack(&state, "T8 Decoy");

    // p2's "clone" (structurally identical to Misdirection) targets the decoy.
    let clone_hand_id = find_obj(&state, "PB-DX25c T8 Clone");
    let (state, _) = cast(
        state,
        p2,
        clone_hand_id,
        vec![Target::Object(decoy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Clone cast must succeed: {:?}", e));
    let clone_card_id = find_stack_obj_on_stack(&state, "T8 Clone");

    // p3's "alternative" -- a genuine, untouched single-target spell that
    // will still be on the stack when Misdirection resolves.
    let alternative_hand_id = find_obj(&state, "PB-DX25c T8 Alternative");
    let (state, _) = cast(state, p3, alternative_hand_id, vec![Target::Player(p1)])
        .unwrap_or_else(|e| panic!("Alternative cast must succeed: {:?}", e));
    let alternative_card_id = find_stack_obj_on_stack(&state, "T8 Alternative");

    // p1's real Misdirection targets the clone.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(clone_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    let (_, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("clone must redirect: {:?}", resolve_events));
    assert_ne!(
        new_target[0].target,
        Target::Object(clone_card_id),
        "CR 601.2c: the clone's own card must never be a legal redirect \
         target for its own retarget (self_id exclusion), even though it \
         satisfies TargetSpellWithSingleTarget's type check trivially"
    );
    assert_eq!(
        new_target[0].target,
        Target::Object(alternative_card_id),
        "with decoy(current, excluded) and clone(self, excluded via \
         self_id), the alternative is the only remaining legal candidate"
    );
}

// ── BB1/BB2: Bolt Bend, OBJECT branch (AC 6303's `bolt_bend` half) ─────────

/// **Closes AC 6303's `bolt_bend` half** (`/review` fix cycle 2, Issue 2): no
/// prior PB-DX25c test cast the real `bolt_bend` def on the OBJECT branch --
/// `pb_dx25b_announced_stack_target_space.rs::t2_bolt_bend_announces_and_
/// resolves` casts it, but exercises the PLAYER branch (its victim targets a
/// player). This test drives a REAL Bolt Bend redirecting a "destroy target
/// creature" victim: a land is present specifically to prove the redirect
/// can never land on it (CR 601.2c: `TargetCreature` is a type check, not a
/// zone check, and only a legal creature satisfies it).
///
/// **Correction (`/review` fix cycle 3, Issue 1)**: the land is declared
/// BEFORE the legal creature, giving it the lower `ObjectId`. Pre-batch
/// HEAD picked "the smallest `ObjectId` in the same zone that isn't the
/// current target" with no legality check at all -- with the land declared
/// AFTER the legal creature (this fixture's original order), the land was
/// never the smallest id, so HEAD's own blind heuristic already returned
/// the legal creature by accident and this test would have PASSED
/// unmodified at pre-batch HEAD. See
/// `memory/primitives/pb-DX25c-execution-notes.md`'s revert matrix
/// addendum for the HEAD-heuristic mutation this correction adds.
#[test]
fn bb1_bolt_bend_object_branch_lands_only_on_a_legal_creature_never_a_land() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let destroy = destroy_creature_def("PB-DX25c BB1 Destroy", "pb-dx25c-bb1-destroy");
    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone(), destroy.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                red: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::creature(p3, "BB1 Original Creature", 2, 2))
        // The land is declared BEFORE the legal creature so it receives the
        // LOWER ObjectId (`/review` fix cycle 3, Issue 1) -- pre-batch HEAD's
        // object branch picked "the smallest ObjectId in the same zone that
        // isn't the current target" with no legality check at all
        // (`retarget.rs`'s own module doc), so a land declared AFTER the
        // legal creature would already have satisfied HEAD's blind
        // heuristic by accident, leaving this fixture unable to discriminate
        // the shipped defect it exists to catch.
        .object(ObjectSpec::land(p1, "BB1 Land"))
        .object(ObjectSpec::creature(p1, "BB1 Legal Creature", 2, 2))
        .object(
            ObjectSpec::card(p1, "Bolt Bend")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(bolt_bend.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c BB1 Destroy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let original_creature_id = find_obj(&state, "BB1 Original Creature");
    let legal_creature_id = find_obj(&state, "BB1 Legal Creature");
    let land_id = find_obj(&state, "BB1 Land");

    let destroy_hand_id = find_obj(&state, "PB-DX25c BB1 Destroy");
    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(original_creature_id)],
    )
    .unwrap_or_else(|e| panic!("Destroy cast must succeed: {:?}", e));
    let destroy_card_id = find_stack_obj_on_stack(&state, "BB1 Destroy");

    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");
    let (state, _) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::Object(destroy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Bolt Bend cast must succeed: {:?}", e));

    // First resolution: Bolt Bend itself.
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Destroy must redirect: {:?}", resolve_events));
    assert_ne!(
        new_target[0].target,
        Target::Object(land_id),
        "CR 601.2c/115.7a: TargetCreature is a TYPE check, not a zone check \
         -- the redirect must never land on the land even though it is a \
         legal battlefield object in retarget_candidates's universe"
    );
    assert_eq!(
        new_target[0].target,
        Target::Object(legal_creature_id),
        "with the original creature (current target, excluded) and the \
         land (fails TargetCreature) both unavailable, BB1 Legal Creature \
         is the only remaining CR 115.7a-legal candidate"
    );

    // Second resolution: the (now-redirected) Destroy effect.
    let (state, _) = resolve_top_of_stack(state);
    assert!(
        !state.objects().values().any(
            |o| o.characteristics.name == "BB1 Legal Creature" && o.zone == ZoneId::Battlefield
        ),
        "the redirected Destroy must actually destroy the NEW (legal) target"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "BB1 Original Creature"
                && o.zone == ZoneId::Battlefield),
        "the original creature target must survive -- the redirect moved off it"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "BB1 Land" && o.zone == ZoneId::Battlefield),
        "the land must never have been a legal candidate, so it must survive untouched"
    );
}

/// **The no-legal-target half of AC 6303's `bolt_bend` object-branch
/// coverage.** CR 115.7a's own fallback ("If a target can't be changed to
/// another legal target, the original target is unchanged") on a REAL Bolt
/// Bend cast against a REAL "destroy target creature" victim, with no other
/// creature on the battlefield at all -- not even a land to be tempted by,
/// since `TargetCreature` would reject one anyway; this isolates the
/// "there is genuinely nothing else" case from BB1's "there is something,
/// but it's the wrong type" case.
///
/// **This is a CR 115.7a fallback CONFORMANCE PIN, not a discriminator of
/// the PB-DX25c fix** (`/review` fix cycle 3, Issue 2, stated rather than
/// contrived): with only the current target present on the battlefield,
/// `retarget_candidates`' object universe contains nothing else to pick, so
/// pre-batch HEAD's own blind "smallest ObjectId in the same zone that
/// isn't the current target" heuristic finds no candidate either and the
/// redirect is a no-op there too -- this test is green at both pre-batch
/// HEAD and at HEAD-after-the-fix. `bb1_bolt_bend_object_branch_lands_
/// only_on_a_legal_creature_never_a_land` is the sibling that actually
/// discriminates the shipped defect (a legal-battlefield-object-but-wrong-
/// type decoy IS present there).
#[test]
fn bb2_bolt_bend_object_branch_no_legal_target_leaves_targets_unchanged() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let destroy = destroy_creature_def("PB-DX25c BB2 Destroy", "pb-dx25c-bb2-destroy");
    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone(), destroy.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                red: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::creature(p3, "BB2 Original Creature", 2, 2))
        .object(
            ObjectSpec::card(p1, "Bolt Bend")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(bolt_bend.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c BB2 Destroy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let original_creature_id = find_obj(&state, "BB2 Original Creature");

    let destroy_hand_id = find_obj(&state, "PB-DX25c BB2 Destroy");
    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(original_creature_id)],
    )
    .unwrap_or_else(|e| panic!("Destroy cast must succeed: {:?}", e));
    let destroy_card_id = find_stack_obj_on_stack(&state, "BB2 Destroy");

    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");
    let (state, _) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::Object(destroy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Bolt Bend cast must succeed: {:?}", e));

    // First resolution: Bolt Bend itself -- CR 115.7a fallback, no legal
    // alternative creature exists.
    let (state, resolve_events) = resolve_top_of_stack(state);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "CR 115.7a: with no legal alternative creature, NO TargetsChanged \
         event may be emitted -- the original target is unchanged, events: \
         {:?}",
        resolve_events
    );

    // Second resolution: Destroy resolves against its UNCHANGED original target.
    let (state, _) = resolve_top_of_stack(state);
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "BB2 Original Creature"
                && o.zone == ZoneId::Battlefield),
        "the fallback must leave the original target intact, not fizzle it \
         -- Destroy must still destroy it"
    );
}

// ── T9c: the fail-closed guard (the ONLY hand-built StackObject in this file) ──

/// §3.4's fail-closed decision, pinned directly. This configuration --
/// non-empty `targets` with an EMPTY `target_requirements` -- is unreachable
/// on any path `plan_target_change` can actually reach, because a
/// `ChangeTargets` victim is always a `Spell`/`MutatingCreatureSpell`
/// (`OOS-DX25b-1`: only a CARD id can ever be "announced" by a player, and
/// `stack_index_for_announced_target` resolves `pos` from that announced id
/// alone -- an ability's stack entry owns no card and can never be named).
///
/// **This is NOT a claim that every production `.targets`-writing site
/// records a real list -- it is false as stated, and `/review` caught it**:
/// `abilities.rs:1799` (Forecast), `:2017` (Bloodrush), `:8837` (Modular),
/// `:10975` (Scavenge) each write a NON-EMPTY `targets` with a deliberately
/// EMPTY `target_requirements` (there is no `TargetRequirement` shape for
/// "the attacking creature" or "the deterministically-scanned artifact
/// creature" those abilities target -- see `stack::StackObject`'s own doc:
/// "Empty means ... no list was recorded at this push site"). Those four
/// sites are exactly this configuration, and they are why the fail-closed
/// guard is load-bearing rather than decorative: the day `OOS-DX25b-1`
/// closes and abilities become reachable `ChangeTargets` victims, this guard
/// is what stops one of those four from silently reintroducing an unfiltered
/// redirect instead of correctly refusing to change a target it was never
/// told the legality rule for.
///
/// This fixture exists solely to prove the guard on the ONE path that CAN
/// reach it today (a hand-built `StackObject`, deliberately the only one in
/// this file), and proves NOTHING about the production path.
#[test]
fn t9c_missing_requirement_list_fails_closed() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_card = ObjectId(9001);
    let entry_id = test_util::next_object_id(&mut state);
    let victim = StackObject {
        id: entry_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: victim_card,
        },
        targets: vec![SpellTarget {
            target: Target::Player(p1),
            zone_at_cast: None,
        }],
        target_requirements: vec![], // deliberately empty -- the fail-closed case
        ..blank_stack_object()
    };
    state.stack_objects_mut().push_back(victim);

    let source = ObjectId(0);
    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(entry_id),
            zone_at_cast: Some(ZoneId::Stack),
        }],
    );
    let effect = Effect::ChangeTargets {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        must_change: true,
    };
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "an empty target_requirements list must fail closed -- no change, no \
         event -- even though a legal alternative player (p2) exists"
    );
    let victim = state
        .stack_objects()
        .iter()
        .find(|s| s.id == entry_id)
        .unwrap();
    assert_eq!(
        victim.targets[0].target,
        Target::Player(p1),
        "target must remain unchanged"
    );
}

fn blank_stack_object() -> StackObject {
    StackObject {
        id: ObjectId(0),
        controller: p(1),
        kind: StackObjectKind::Spell {
            source_object: ObjectId(0),
        },
        targets: vec![],
        target_requirements: vec![],
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
        was_warped: false,
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
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    }
}

// ── T10: HashInto field coverage ────────────────────────────────────────────

/// `StackObject::target_requirements` must be hashed. Required because
/// `canonical_fixture()` cannot populate `stack_objects`
/// (`hash_schema.rs:713-726`), so this field's own bytes are otherwise
/// inside NO gate -- the exact situation the v73 row records for a sibling
/// field.
#[test]
fn t10_target_requirements_field_is_hashed() {
    let base = blank_stack_object();
    let mut with_req = base.clone();
    with_req.target_requirements = vec![TargetRequirement::TargetCreature];

    let hash_of = |so: &StackObject| -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        so.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&base),
        hash_of(&with_req),
        "two StackObjects differing ONLY in target_requirements must hash \
         differently -- StackObject::hash_into must feed the new field"
    );
    assert_eq!(
        hash_of(&base),
        hash_of(&base),
        "identical StackObjects must hash identically (sanity)"
    );
}
