//! PB-DX25 — `Effect::CounterSpell`'s three stack-object shapes (`OOS-SIM3-5`).
//!
//! CR 701.6a: "To counter a spell or ability means to cancel it, removing it from
//! the stack. It doesn't resolve and none of its effects occur. A countered spell
//! is put into its owner's graveyard." That sentence is about a CARD. A stack
//! object with no card (a copy, CR 707.10) has nothing to put anywhere; a stack
//! object WITH a card (`Spell`, and — since CR 702.140a / CR 729.2 — also
//! `MutatingCreatureSpell`) must have that card moved. At HEAD,
//! `Effect::CounterSpell`'s zone-move decides which is which by matching the
//! `StackObjectKind` variant NAME rather than asking "does this kind own a card in
//! `ZoneId::Stack`" — and it only asks that question for the literal `Spell`
//! variant. `crates/engine/src/state/stack_registry.rs` (added this batch) answers
//! that question once, exhaustively, for the whole engine.
//!
//! Three shapes (plan §2):
//! - **(c) LIVE**: an ordinary counter targeting a mutate spell's card is a silent
//!   no-op — the target is legal (the card really is in `ZoneId::Stack`), the mana
//!   is paid, and nothing happens. **T1**, real corpus cards (`gemrazer` ×
//!   `counterspell`).
//! - **(a)**: the Ward-shaped `so.id == id` lookup already finds a
//!   `MutatingCreatureSpell` at HEAD (a stack entry's own id lives in a different
//!   id space than any card's `ObjectId` — see the `Simulator / play-client`
//!   gotcha in `memory/gotchas-infra.md` — so this clause was never kind-specific).
//!   What is missing is what happens AFTER the lookup: no zone-move arm for this
//!   kind, so the entry is removed but its card is stranded in `ZoneId::Stack`
//!   forever. **T2**, SYNTHETIC (see T2's own doc for why).
//! - **(b)**: countering a COPY of a spell must move no card (CR 707.10 — a copy
//!   IS a spell but has no card of its own; `copy.rs` clones the original's
//!   `kind`, so a naive fix would move the ORIGINAL's card). **T3**, SYNTHETIC.
//!
//! T4/T5 cover destination preservation (CR 702.34a/702.133a) and the
//! `cant_be_countered` controller-capture ordering (EF-W-MISS-1 / An Offer) on the
//! newly-reachable `MutatingCreatureSpell` path. T6 pins `stack_registry`'s own
//! classification, exhaustively, against every `StackObjectKind` variant.
//!
//! **This file holds T1-T6 only.** T7 (the `resolution::counter_stack_object`
//! second-path parity probe) is Stage 5 — a different runner's scope; see
//! `memory/primitives/pb-plan-DX25.md` §10.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack_registry::card_in_stack_zone;
use mtg_engine::state::stubs::DelayedTriggerAction;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AdditionalCost,
    AttackTarget, CardDefinition, CardId, CardRegistry, CardType, Command, DungeonId, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, SubType, Target,
    TriggerData,
};

// ── Shared helpers ──────────────────────────────────────────────────────────────

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

/// Every `Complete`-or-not card def, keyed by name — the shape `enrich_spec_from_def`
/// wants (mirrors `crates/engine/tests/mechanics_e_l/golgari_grave_troll.rs`'s
/// `build_defs_and_registry`).
fn all_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn full_registry() -> Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

fn enrich(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// Pass priority for each listed player once, in order. Two consecutive passes
/// with no intervening action resolves the top of the stack (CR 405.5 / CR 117.4).
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

/// Drain the whole stack, always passing as whoever CURRENTLY holds priority
/// (read from `state`, not a hardcoded rotation) — after a resolution, priority
/// resets to the active player (CR 117.3b), which is not necessarily the "next"
/// player in whatever fixed order a caller might have assumed.
fn drain_stack(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    while !state.stack_objects().is_empty() {
        let holder = state
            .turn()
            .priority_holder
            .expect("priority holder must be set while the stack is non-empty");
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// A non-Human creature target for a mutate cast — mirrors
/// `crates/engine/tests/mechanics_m_z/mutate.rs`'s "Mock Wolf" fixture exactly
/// (not a real card def; only the mutate TARGET needs to exist, not be castable).
fn wolf_spec(owner: PlayerId) -> ObjectSpec {
    let mut wolf = ObjectSpec::card(owner, "Mock Wolf")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-wolf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())]);
    wolf.power = Some(2);
    wolf.toughness = Some(3);
    wolf
}

// ── T1 — shape (c), REAL corpus cards, gemrazer x counterspell ─────────────────

/// CR 701.6a / CR 702.140a / CR 400.7 — countering a mutate spell's card (the
/// ordinary "counter target spell" path) removes it from the stack, puts the
/// countered card in its owner's graveyard under a fresh `ObjectId` (CR 400.7),
/// and the merge (CR 729.2) never happens.
///
/// REAL corpus cards, exactly the plan's §0.3 probe pair: `gemrazer` (explicit
/// `Completeness::Complete`, no spell-level target requirement) x `counterspell`
/// (`Complete` by derive, `TargetRequirement::TargetSpell`). At HEAD this test
/// FAILS: `position()`'s second clause matches only `StackObjectKind::Spell`, so
/// the counter finds nothing, Gemrazer resolves and merges with the Wolf, and the
/// stack is left non-empty by the Counterspell's own leftover no-op resolution
/// path being the only thing that runs. See `memory/primitives/pb-DX25-execution-
/// notes.md` for the verbatim pre-fix failure text.
#[test]
fn test_dx25_counterspell_counters_a_mutate_spell() {
    let p1 = p(1);
    let p2 = p(2);

    let defs = all_defs_by_name();
    let registry = full_registry();

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich(p1, "Gemrazer", ZoneId::Hand(p1), &defs))
        .object(wolf_spec(p1))
        .object(enrich(p2, "Counterspell", ZoneId::Hand(p2), &defs))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // CR 702.140a: Gemrazer's mutate cost is {1}{G}{G}.
    {
        let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Green, 3);
        pool.add(ManaColor::Colorless, 2);
    }
    // Counterspell is {U}{U}.
    {
        let pool = &mut state.players_mut().get_mut(&p2).unwrap().mana_pool;
        pool.add(ManaColor::Blue, 2);
    }
    state.turn_mut().priority_holder = Some(p1);

    let gemrazer_hand_id = find_object(&state, "Gemrazer");
    let wolf_id = find_object(&state, "Mock Wolf");
    let counterspell_hand_id = find_object(&state, "Counterspell");

    // CR 702.140a: p1 casts Gemrazer for its mutate cost, targeting the Wolf.
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: gemrazer_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Mutate),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Mutate {
                target: wolf_id,
                on_top: true,
            }],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell (mutate) failed: {:?}", e));

    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 702.140a: the mutating spell should be the only thing on the stack"
    );
    let gemrazer_stack_card_id = match &state.stack_objects()[0].kind {
        StackObjectKind::MutatingCreatureSpell {
            source_object,
            target,
        } => {
            assert_eq!(
                *target, wolf_id,
                "CR 702.140a: mutate target should be the Wolf"
            );
            *source_object
        }
        other => panic!(
            "CR 702.140a: expected MutatingCreatureSpell on the stack, got {:?}",
            other
        ),
    };

    // CR 117.3c (PB-DP1): priority after a cast goes to the actor. p1 passes so
    // p2 can act.
    let (state, _) = pass_all(state, &[p1]);

    // p2 casts Counterspell, targeting the CARD in ZoneId::Stack (CR 601.2c
    // TargetSpell validation: the id must be a `state.objects` key with
    // zone == Stack -- that is the card, never the stack-entry id).
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p2,
            card: counterspell_hand_id,
            targets: vec![Target::Object(gemrazer_stack_card_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![],
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell (Counterspell) failed: {:?}", e));

    assert_eq!(
        state.stack_objects().len(),
        2,
        "Counterspell should be on the stack above the mutating spell"
    );

    // Drain the stack: Counterspell resolves first (LIFO), countering Gemrazer.
    let (state, all_events) = drain_stack(state);

    assert!(
        state.stack_objects().is_empty(),
        "CR 701.6a: the stack must be empty once both spells have resolved/been countered"
    );

    // CR 701.6a: the countered card is in its OWNER's graveyard, under a NEW
    // ObjectId (CR 400.7 -- the pre-counter id is dead).
    let gemrazer_graveyard_id = find_in_zone(&state, "Gemrazer", ZoneId::Graveyard(p1))
        .unwrap_or_else(|| {
            panic!(
                "CR 701.6a: countered Gemrazer should be in p1's graveyard under a fresh ObjectId"
            )
        });
    assert_ne!(
        gemrazer_graveyard_id, gemrazer_stack_card_id,
        "CR 400.7: the graveyard object must be a NEW ObjectId, not the pre-counter Stack id"
    );

    // CR 729.2 must NOT have happened: the Wolf is unmerged.
    let wolf_obj = state
        .objects()
        .get(&wolf_id)
        .expect("Wolf should still be on the battlefield, unmerged");
    assert!(
        wolf_obj.merged_components.is_empty(),
        "CR 701.6a / CR 729.2: a properly countered mutate spell must NOT merge -- \
         merged_components should be empty, got {:?}",
        wolf_obj.merged_components
    );

    // Exactly one SpellCountered event, naming the post-counter graveyard id.
    let countered: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCountered { .. }))
        .collect();
    assert_eq!(
        countered.len(),
        1,
        "CR 701.6a: exactly one SpellCountered event expected, got {:?}",
        countered
    );
    assert!(
        matches!(
            countered[0],
            GameEvent::SpellCountered { source_object_id, .. }
            if *source_object_id == gemrazer_graveyard_id
        ),
        "CR 701.6a / CR 400.7: SpellCountered.source_object_id must be the POST-move \
         graveyard id, got {:?}",
        countered[0]
    );
}

// ── T6 — stack_registry classifies every StackObjectKind variant ───────────────

/// One instance of every `StackObjectKind` variant, by name. The names must be
/// checked against the enum's actual variant set independently (`grep -c "^
/// [A-Za-z]* {" ... | rg -v TriggerData` at plan-verification time measured 27) —
/// this function's own non-vacuity is what T6 checks below, not assumed here.
fn one_of_each_variant() -> Vec<(&'static str, StackObjectKind)> {
    let oid = |n: u64| ObjectId(n);
    let pid = p(1);
    let simple_effect = || {
        Box::new(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        })
    };
    vec![
        (
            "Spell",
            StackObjectKind::Spell {
                source_object: oid(1),
            },
        ),
        (
            "ActivatedAbility",
            StackObjectKind::ActivatedAbility {
                source_object: oid(1),
                ability_index: 0,
                embedded_effect: None,
            },
        ),
        (
            "LoyaltyAbility",
            StackObjectKind::LoyaltyAbility {
                source_object: oid(1),
                ability_index: 0,
                effect: simple_effect(),
            },
        ),
        (
            "TriggeredAbility",
            StackObjectKind::TriggeredAbility {
                source_object: oid(1),
                ability_index: 0,
                is_carddef_etb: false,
                embedded_effect: None,
            },
        ),
        (
            "MadnessTrigger",
            StackObjectKind::MadnessTrigger {
                source_object: oid(1),
                exiled_card: oid(2),
                madness_cost: ManaCost::default(),
                owner: pid,
            },
        ),
        (
            "MiracleTrigger",
            StackObjectKind::MiracleTrigger {
                source_object: oid(1),
                revealed_card: oid(2),
                miracle_cost: ManaCost::default(),
                owner: pid,
            },
        ),
        (
            "UnearthAbility",
            StackObjectKind::UnearthAbility {
                source_object: oid(1),
            },
        ),
        (
            "SuspendCounterTrigger",
            StackObjectKind::SuspendCounterTrigger {
                source_object: oid(1),
                suspended_card: oid(2),
            },
        ),
        (
            "SuspendCastTrigger",
            StackObjectKind::SuspendCastTrigger {
                source_object: oid(1),
                suspended_card: oid(2),
                owner: pid,
            },
        ),
        (
            "NinjutsuAbility",
            StackObjectKind::NinjutsuAbility {
                source_object: oid(1),
                ninja_card: oid(1),
                attack_target: AttackTarget::Player(pid),
                from_command_zone: false,
            },
        ),
        (
            "EmbalmAbility",
            StackObjectKind::EmbalmAbility {
                source_card_id: Some(CardId("x".to_string())),
            },
        ),
        (
            "EternalizeAbility",
            StackObjectKind::EternalizeAbility {
                source_card_id: Some(CardId("x".to_string())),
                source_name: "X".to_string(),
            },
        ),
        (
            "EncoreAbility",
            StackObjectKind::EncoreAbility {
                source_card_id: Some(CardId("x".to_string())),
                activator: pid,
            },
        ),
        (
            "ForecastAbility",
            StackObjectKind::ForecastAbility {
                source_object: oid(1),
                embedded_effect: simple_effect(),
            },
        ),
        (
            "ScavengeAbility",
            StackObjectKind::ScavengeAbility {
                source_card_id: Some(CardId("x".to_string())),
                power_snapshot: 0,
            },
        ),
        (
            "BloodrushAbility",
            StackObjectKind::BloodrushAbility {
                source_object: oid(1),
                target_creature: oid(2),
                power_boost: 0,
                toughness_boost: 0,
                grants_keyword: None,
            },
        ),
        (
            "SaddleAbility",
            StackObjectKind::SaddleAbility {
                source_object: oid(1),
            },
        ),
        (
            "MutatingCreatureSpell",
            StackObjectKind::MutatingCreatureSpell {
                source_object: oid(1),
                target: oid(2),
            },
        ),
        (
            "TransformTrigger",
            StackObjectKind::TransformTrigger {
                permanent: oid(1),
                ability_timestamp: 0,
            },
        ),
        (
            "CraftAbility",
            StackObjectKind::CraftAbility {
                source_card_id: Some(CardId("x".to_string())),
                exiled_source: oid(1),
                material_ids: vec![],
                activator: pid,
            },
        ),
        (
            "DayboundTransformTrigger",
            StackObjectKind::DayboundTransformTrigger { permanent: oid(1) },
        ),
        (
            "TurnFaceUpTrigger",
            StackObjectKind::TurnFaceUpTrigger {
                permanent: oid(1),
                source_card_id: Some(CardId("x".to_string())),
                ability_index: 0,
            },
        ),
        (
            "KeywordTrigger",
            StackObjectKind::KeywordTrigger {
                source_object: oid(1),
                keyword: KeywordAbility::Deathtouch,
                data: TriggerData::Simple,
            },
        ),
        (
            "RoomAbility",
            StackObjectKind::RoomAbility {
                owner: pid,
                dungeon: DungeonId::LostMineOfPhandelver,
                room: 0,
            },
        ),
        (
            "RingAbility",
            StackObjectKind::RingAbility {
                source_object: oid(1),
                effect: simple_effect(),
                controller: pid,
            },
        ),
        (
            "ClassLevelAbility",
            StackObjectKind::ClassLevelAbility {
                source_object: oid(1),
                target_level: 1,
            },
        ),
        (
            "DelayedActionTrigger",
            StackObjectKind::DelayedActionTrigger {
                source_object: oid(1),
                target: oid(2),
                action: DelayedTriggerAction::ExileObject,
            },
        ),
    ]
}

/// CR 601.2c / CR 702.140a / CR 729.2 — `stack_registry::card_in_stack_zone`
/// classifies every `StackObjectKind` variant, exhaustively: `Some` for exactly
/// `Spell` and `MutatingCreatureSpell`, `None` for everything else. Non-vacuity:
/// the fixture's own variant count is asserted equal to the measured count (27 at
/// HEAD, `memory/primitives/pb-DX25-stage0.md`), so a 28th variant that compiles
/// (because it was classified in the registry) but is never added to this
/// fixture cannot silently escape T6's coverage.
#[test]
fn test_dx25_stack_registry_classifies_every_kind() {
    let variants = one_of_each_variant();
    assert_eq!(
        variants.len(),
        27,
        "CR 601.2c: this fixture must cover exactly the measured StackObjectKind \
         variant count (27) -- a mismatch means either a variant was added to the \
         enum without a fixture entry here, or this list has drifted"
    );

    let card_owning: Vec<&str> = variants
        .iter()
        .filter(|(_, kind)| card_in_stack_zone(kind).is_some())
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        card_owning,
        vec!["Spell", "MutatingCreatureSpell"],
        "CR 601.2c / CR 702.140a / CR 729.2: exactly Spell and MutatingCreatureSpell \
         own a card in ZoneId::Stack -- got {:?}",
        card_owning
    );
}
