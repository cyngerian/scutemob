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
//! classification, exhaustively, against every `StackObjectKind` variant. T7
//! (Stage 5) proves `resolution::counter_stack_object` — the engine's second,
//! non-production counter path — agrees with `Effect::CounterSpell` on the same
//! two newly-fixed shapes.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack_registry::card_in_stack_zone;
use mtg_engine::state::stubs::DelayedTriggerAction;
use mtg_engine::state::test_util;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AdditionalCost,
    AttackTarget, CardDefinition, CardEffectTarget, CardId, CardRegistry, CardType, Command,
    DungeonId, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, SpellTarget, StackObject,
    StackObjectKind, Step, SubType, Target, TriggerData,
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

/// Push a bare `StackObject::MutatingCreatureSpell` entry, mirroring the
/// manual-stack-push idiom used elsewhere in the suite (see
/// `crates/engine/tests/casting/optional_cost_and_counter_tax.rs`'s
/// `push_spell_stack_object` -- same full-literal shape, `kind` swapped).
fn push_mutating_creature_spell_stack_object(
    state: &mut GameState,
    source_object: ObjectId,
    target: ObjectId,
    controller: PlayerId,
) -> ObjectId {
    let stack_id = test_util::next_object_id(state);
    state.stack_objects_mut().push_back(StackObject {
        id: stack_id,
        controller,
        kind: StackObjectKind::MutatingCreatureSpell {
            source_object,
            target,
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
        x_value: 0,
        evidence_collected: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
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
    });
    stack_id
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

// ── T2 — shape (a), SYNTHETIC, the Ward-shaped so.id == id lookup ──────────────

/// CR 702.21a / CR 701.6a — a `MutatingCreatureSpell` countered through the
/// `so.id == id` clause (the Ward shape: a trigger names the STACK ENTRY's own
/// id, not the card's id) must move the card to the graveyard exactly like a
/// card-id counter does.
///
/// **SYNTHETIC, and here is why (plan §2.2).** No `Complete` mutate def declares
/// a spell-level target requirement (roster M3 = 0, `pb-DX25-stage0.md`), so no
/// real Ward creature can announce a `PermanentTargeted` event against a mutate
/// spell's target today (`OOS-DX25-1` -- the mutate target is carried in
/// `AdditionalCost::Mutate`, never in `spell_targets`). `ward.rs:136-260` was
/// read first, per the plan's instruction, before falling back to this route: a
/// real Ward creature cannot be made to fire against a mutate spell's target
/// without first fixing `OOS-DX25-1`, which is explicitly out of scope (plan §12
/// risk 6). This fixture reaches the `so.id == id` clause directly, the same
/// shape a real Ward trigger's `EffectContext` would carry
/// (`abilities.rs:4605-4633`/`:8400-8405` tag the pending trigger with the stack
/// entry's own id as its target) -- **route used: hand-built `EffectContext`,
/// not a hand-built trigger `StackObject`** (simpler, and `EffectContext.targets`
/// is exactly what `EffectTarget::DeclaredTarget` reads at resolution time,
/// so this is a faithful shortcut, not a different mechanism).
#[test]
fn test_dx25_ward_path_counter_on_a_mutate_spell_moves_the_card() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Mutating Beast")
                .in_zone(ZoneId::Stack)
                .with_card_id(CardId("mock-mutating-beast".to_string())),
        )
        .object(wolf_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let beast_card_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");

    let stack_entry_id =
        push_mutating_creature_spell_stack_object(&mut state, beast_card_id, wolf_id, p1);
    assert_eq!(state.stack_objects().len(), 1);

    let mut ctx = EffectContext::new(
        p2,
        wolf_id,
        vec![SpellTarget {
            target: Target::Object(stack_entry_id),
            zone_at_cast: None,
        }],
    );
    let events = execute_effect(
        &mut state,
        &Effect::CounterSpell {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            exile_instead: false,
        },
        &mut ctx,
    );

    assert!(
        state.stack_objects().is_empty(),
        "CR 701.6a: the Ward-shaped counter must remove the stack entry"
    );
    let beast_graveyard_id = find_in_zone(&state, "Mock Mutating Beast", ZoneId::Graveyard(p1))
        .unwrap_or_else(|| {
            panic!(
                "CR 701.6a / CR 702.140a: a MutatingCreatureSpell countered via the \
                 Ward-shaped so.id == id lookup must move its card to the graveyard, \
                 not strand it in ZoneId::Stack"
            )
        });
    assert!(
        find_in_zone(&state, "Mock Mutating Beast", ZoneId::Stack).is_none(),
        "CR 701.6a: the card must no longer be in ZoneId::Stack"
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::SpellCountered { .. }))
            .count(),
        1,
        "exactly one SpellCountered event expected, got {:?}",
        events
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::SpellCountered { source_object_id, .. }
            if *source_object_id == beast_graveyard_id
        )),
        "SpellCountered.source_object_id must be the post-move graveyard id"
    );
}

// ── T3 — shape (b), SYNTHETIC, countering a copy moves no card ─────────────────

/// CR 707.10 / CR 707.10a / CR 707.10b — countering a COPY of a spell removes
/// its stack entry but moves NO card (a copy has no card of its own); countering
/// the ORIGINAL (a sibling, non-vacuity fixture) DOES move the card, proving the
/// fixture is capable of moving one at all.
///
/// **SYNTHETIC, and here is why (plan §2.3).** Three independent mechanisms make
/// countering a copy unreachable at HEAD through the ordinary cast path: (1)
/// order -- `copy.rs` pushes the copy ABOVE the original, and `position()`
/// returns the FIRST (lowest-index) match, so a card-id lookup always lands on
/// the original while it's still present; (2) the dead-id filter --
/// `resolve_effect_target_list_indexed` only resolves a `DeclaredTarget` if the
/// id names a live object or stack entry, and once the original leaves the
/// stack its card gets a new id under CR 400.7, so the window in which the copy
/// could be found by the original's card id is empty, not merely narrow; (3)
/// nothing aims a counter at a copy in the first place -- `TargetSpell` cannot
/// name a copy's stack-entry id (it isn't a `state.objects` key), and Ward never
/// fires on a copy (`OOS-DX25-2`, no `PermanentTargeted` event from
/// `copy_spell_on_stack`). This fixture reaches the counter-on-a-copy state
/// directly via `rules::copy::copy_spell_on_stack` (a real, `pub` engine
/// function) + a hand-built `EffectContext` targeting the copy's own
/// stack-entry id (the ONLY id that could ever legitimately name it).
#[test]
fn test_dx25_countering_a_copy_moves_no_card() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Original Spell")
                .in_zone(ZoneId::Stack)
                .with_card_id(CardId("mock-original-spell".to_string())),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let original_card_id = find_object(&state, "Mock Original Spell");
    let original_stack_id = {
        let stack_id = test_util::next_object_id(&mut state);
        state.stack_objects_mut().push_back(StackObject {
            id: stack_id,
            controller: p1,
            kind: StackObjectKind::Spell {
                source_object: original_card_id,
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
            x_value: 0,
            evidence_collected: false,
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            modes_chosen: vec![],
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
        });
        stack_id
    };

    // CR 707.10: put a copy of the spell on the stack, above the original.
    let (copy_stack_id, _copy_event) =
        mtg_engine::rules::copy::copy_spell_on_stack(&mut state, original_stack_id, p2, false)
            .unwrap_or_else(|e| panic!("copy_spell_on_stack failed: {:?}", e));
    assert_eq!(
        state.stack_objects().len(),
        2,
        "original + copy on the stack"
    );

    // Counter the COPY, by its own stack-entry id (the only legitimate way to
    // name it -- see the doc comment above).
    let mut ctx = EffectContext::new(
        p1,
        original_card_id,
        vec![SpellTarget {
            target: Target::Object(copy_stack_id),
            zone_at_cast: None,
        }],
    );
    let events = execute_effect(
        &mut state,
        &Effect::CounterSpell {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            exile_instead: false,
        },
        &mut ctx,
    );

    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 707.10a: only the copy's entry should be removed"
    );
    assert!(
        state
            .stack_objects()
            .iter()
            .any(|so| so.id == original_stack_id),
        "the ORIGINAL's stack entry must be untouched"
    );
    assert_eq!(
        state.objects().get(&original_card_id).map(|o| o.zone),
        Some(ZoneId::Stack),
        "CR 707.10: the ORIGINAL's card must still be in ZoneId::Stack -- the copy \
         has no card of its own to move"
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::SpellCountered { .. }))
            .count(),
        1,
        "exactly one SpellCountered event expected, got {:?}",
        events
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::SpellCountered { stack_object_id, source_object_id, .. }
            if *stack_object_id == copy_stack_id && *source_object_id == copy_stack_id
        )),
        "CR 707.10: SpellCountered for a copy must name the copy's OWN stack-entry \
         id for both stack_object_id and source_object_id (no card id may be named) \
         -- got {:?}",
        events
    );

    // Non-vacuity, same test: countering the ORIGINAL -- the SAME `state`,
    // continued right after the copy-counter above, not a separate/sibling
    // fixture (review Finding, additional LOW notes: the prior wording called
    // this a "sibling fixture", which wrongly implied a second, independent
    // setup) -- DOES move its card, proving this fixture/effect path is
    // capable of moving a card at all, so the "no card moved" assertion above
    // is a real negative.
    let mut ctx2 = EffectContext::new(
        p1,
        original_card_id,
        vec![SpellTarget {
            target: Target::Object(original_card_id),
            zone_at_cast: None,
        }],
    );
    let _ = execute_effect(
        &mut state,
        &Effect::CounterSpell {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            exile_instead: false,
        },
        &mut ctx2,
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 701.6a: countering the original should remove its stack entry too"
    );
    assert!(
        find_in_zone(&state, "Mock Original Spell", ZoneId::Graveyard(p1)).is_some(),
        "non-vacuity: countering the ORIGINAL (not a copy) DOES move its card -- \
         this fixture and Effect::CounterSpell are capable of moving a card"
    );
}

// ── T4 — destination preservation (CR 702.34a / CR 702.133a) ───────────────────

/// CR 701.6a / CR 702.34a / CR 702.133a — a countered spell's destination is
/// Exile when `exile_instead`, `cast_with_flashback`, OR `cast_with_jump_start`
/// was set, and `Graveyard(owner)` otherwise -- with `owner != controller` so
/// the OWNER lookup (not the controller) is what is actually being exercised.
/// A fifth sub-case pins that a `MutatingCreatureSpell` structurally cannot
/// carry `cast_with_flashback` (mutually exclusive alternative costs, CR
/// 118.9) -- asserted, not exercised, since there is no way to construct that
/// state.
///
/// **`cast_with_jump_start` (review Finding, additional LOW notes) was named
/// by plan §3.5 as "individually probed" and was not — sub-case 4 below fixes
/// that.** The plan's OTHER named claim, that the `owner =
/// state.objects.get(&source_object).map(|o| o.owner).unwrap_or(controller)`
/// fallback (`effects/mod.rs:2780-2784`) is "individually probed", does NOT
/// hold and is narrowed here rather than faked with a misleading sub-case:
/// `move_object_to_zone` (`state/mod.rs:1270-1273`) performs the SAME
/// `self.objects.get(&object_id)` lookup, on the SAME id, moments after this
/// line, with no mutation in between. If the owner lookup's `.get()` returns
/// `None` (triggering the fallback), the immediately-following
/// `fizzle_move_object_to_zone` call's own lookup ALSO returns `None`
/// (CR 400.7 fizzle), so the move never happens, no `SpellCountered` fires,
/// and the `unwrap_or(controller)` value is never actually used to place a
/// card anywhere observable. This fallback is dead code at THIS call site by
/// construction, not merely untested -- there is no legal or illegal
/// `GameState` that exercises it observably, so no sub-case is added for it.
#[test]
fn test_dx25_countered_spell_destination_is_preserved() {
    let p1 = p(1); // owner
    let p2 = p(2); // controller (e.g. after a control-change effect)
    let registry = CardRegistry::new(vec![]);

    let push_spell = |state: &mut GameState,
                      card_id: ObjectId,
                      controller: PlayerId,
                      cast_with_flashback: bool,
                      cast_with_jump_start: bool| {
        let stack_id = test_util::next_object_id(state);
        state.stack_objects_mut().push_back(StackObject {
            id: stack_id,
            controller,
            kind: StackObjectKind::Spell {
                source_object: card_id,
            },
            targets: vec![],
            target_requirements: vec![],
            cant_be_countered: false,
            is_copy: false,
            cast_with_flashback,
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
            cast_with_jump_start,
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
            x_value: 0,
            evidence_collected: false,
            spliced_effects: vec![],
            spliced_card_ids: vec![],
            modes_chosen: vec![],
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
        });
        stack_id
    };

    let counter = |state: &mut GameState, card_id: ObjectId, exile_instead: bool| {
        let mut ctx = EffectContext::new(
            p2,
            card_id,
            vec![SpellTarget {
                target: Target::Object(card_id),
                zone_at_cast: None,
            }],
        );
        execute_effect(
            state,
            &Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                exile_instead,
            },
            &mut ctx,
        )
    };

    // Sub-case 1: exile_instead: true -> Exile.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Card A")
                    .in_zone(ZoneId::Stack)
                    .with_card_id(CardId("card-a".to_string())),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let card_id = find_object(&state, "Card A");
        push_spell(&mut state, card_id, p2, false, false);
        counter(&mut state, card_id, true);
        assert!(
            find_in_zone(&state, "Card A", ZoneId::Exile).is_some(),
            "CR 701.6a: exile_instead should send the card to Exile"
        );
    }

    // Sub-case 2: cast_with_flashback: true -> Exile.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Card B")
                    .in_zone(ZoneId::Stack)
                    .with_card_id(CardId("card-b".to_string())),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let card_id = find_object(&state, "Card B");
        push_spell(&mut state, card_id, p2, true, false);
        counter(&mut state, card_id, false);
        assert!(
            find_in_zone(&state, "Card B", ZoneId::Exile).is_some(),
            "CR 702.34a: a flashback-cast spell should be exiled when countered"
        );
    }

    // Sub-case 3: neither -> Graveyard(owner), owner != controller.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Card C")
                    .in_zone(ZoneId::Stack)
                    .with_card_id(CardId("card-c".to_string())),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let card_id = find_object(&state, "Card C");
        // Controller p2, but the CARD's owner (set by ObjectSpec::card(p1, ..))
        // is p1 -- discriminates the owner lookup from a controller fallback.
        push_spell(&mut state, card_id, p2, false, false);
        counter(&mut state, card_id, false);
        assert!(
            find_in_zone(&state, "Card C", ZoneId::Graveyard(p1)).is_some(),
            "CR 701.6a: with neither exile_instead nor flashback, the card goes to \
             its OWNER's (p1's) graveyard, not the controller's (p2's)"
        );
        assert!(
            find_in_zone(&state, "Card C", ZoneId::Graveyard(p2)).is_none(),
            "the card must NOT be in the controller's graveyard"
        );
    }

    // Sub-case 4 (review Finding, additional LOW notes): cast_with_jump_start:
    // true -> Exile (CR 702.133a). Named by plan §3.5 as "individually probed"
    // and was not, until this fix cycle.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Card D")
                    .in_zone(ZoneId::Stack)
                    .with_card_id(CardId("card-d".to_string())),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let card_id = find_object(&state, "Card D");
        push_spell(&mut state, card_id, p2, false, true);
        counter(&mut state, card_id, false);
        assert!(
            find_in_zone(&state, "Card D", ZoneId::Exile).is_some(),
            "CR 702.133a: a jump-start-cast spell should be exiled when countered"
        );
    }

    // Sub-case 5: a MutatingCreatureSpell structurally cannot carry
    // cast_with_flashback -- mutually exclusive alternative costs (CR 118.9: a
    // spell is cast via at most one alternative cost). Asserted, not exercised
    // via a fixture: `CastSpellData::alt_cost` (`rules/command.rs:792`) is a
    // single `Option<AltCostKind>`, and `casting.rs`'s alt-cost dispatch
    // (`:2527`, cited by the plan) picks the `MutatingCreatureSpell` kind and
    // the `cast_with_flashback` flag from two DISJOINT arms of that same
    // `Option` match -- one CastSpell command can select at most one, so no
    // legal command, and therefore no `StackObject`, is ever constructed with
    // both `kind: StackObjectKind::MutatingCreatureSpell` and
    // `cast_with_flashback: true` set together. This is a fact about the
    // command's shape, not a runtime behaviour a fixture can exercise.
}

// ── T5 — cant_be_countered still sets countered_spell_controller ───────────────

/// EF-W-MISS-1 / An Offer You Can't Refuse ruling (2022-04-29), CR 701.6a — a
/// `cant_be_countered` mutate spell: the entry stays on the stack, the card
/// stays in `ZoneId::Stack`, and `ctx.countered_spell_controller` is STILL set
/// (captured before the `cant_be_countered` check, unconditionally). Newly
/// reachable by this batch: before PB-DX25, `position()` never found a
/// `MutatingCreatureSpell` at all, so this line of `Effect::CounterSpell` never
/// ran for one.
#[test]
fn test_dx25_uncounterable_mutate_spell_still_sets_the_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Mutating Beast")
                .in_zone(ZoneId::Stack)
                .with_card_id(CardId("mock-mutating-beast".to_string())),
        )
        .object(wolf_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let beast_card_id = find_object(&state, "Mock Mutating Beast");
    let wolf_id = find_object(&state, "Mock Wolf");
    push_mutating_creature_spell_stack_object(&mut state, beast_card_id, wolf_id, p1);
    // CR 101.6: mark it uncounterable.
    state.stack_objects_mut()[0].cant_be_countered = true;

    let mut ctx = EffectContext::new(
        p2,
        wolf_id,
        vec![SpellTarget {
            target: Target::Object(beast_card_id),
            zone_at_cast: None,
        }],
    );
    let events = execute_effect(
        &mut state,
        &Effect::CounterSpell {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            exile_instead: false,
        },
        &mut ctx,
    );

    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 101.6: an uncounterable spell's entry must stay on the stack"
    );
    assert_eq!(
        state.objects().get(&beast_card_id).map(|o| o.zone),
        Some(ZoneId::Stack),
        "the card must stay in ZoneId::Stack -- nothing was countered"
    );
    assert!(
        events.is_empty(),
        "no SpellCountered event should be emitted, got {:?}",
        events
    );
    assert_eq!(
        ctx.countered_spell_controller,
        Some(p1),
        "EF-W-MISS-1 / An Offer: countered_spell_controller must be set to the \
         (uncounterable) target's controller even though nothing was countered"
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
/// `Spell` and `MutatingCreatureSpell`, `None` for everything else.
///
/// **What the `variants.len() == 27` assertion below actually proves, and what
/// it does NOT (review Finding 4, PB-DX25 fix cycle):** `one_of_each_variant()`
/// is a hand-written `Vec` and the literal `27` is compared against that SAME
/// vec's own length — a self-comparison. It catches this fixture drifting from
/// its own author's intent (e.g. an accidental duplicate or a dropped entry),
/// but it CANNOT detect a 28th `StackObjectKind` variant added to the enum and
/// classified in the registry: `variants.len()` would still read 27 from this
/// hand-written list, and this assertion would still pass. The property this
/// comment previously (and wrongly) claimed the assertion held — "a 28th
/// variant that compiles but is never added to this fixture cannot silently
/// escape coverage" — is held by `g1_scan_is_not_vacuous`
/// (`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`), which counts
/// classification arms in the SOURCE file itself, not in a hand-written test
/// fixture in a different crate target. That is the different, correct
/// subject; this assertion's real job is narrower, and is stated as such.
#[test]
fn test_dx25_stack_registry_classifies_every_kind() {
    let variants = one_of_each_variant();
    assert_eq!(
        variants.len(),
        27,
        "this fixture's own variant count moved from 27 -- this is a \
         self-comparison (fixture vs its own literal) that catches drift in \
         THIS list only; it does NOT detect a new StackObjectKind variant \
         added to the enum -- that property is g1_scan_is_not_vacuous's job, \
         in crates/engine/tests/core/pb_dx25_stack_registry_roster.rs"
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

// ── T7 — resolution::counter_stack_object agrees with Effect::CounterSpell ─────

/// CR 701.6a / CR 707.10 / CR 707.10b -- `resolution::counter_stack_object`,
/// the engine's second (non-production) counter path, agrees with
/// `Effect::CounterSpell` on all three of PB-DX25's shapes: a
/// `MutatingCreatureSpell` moves its card (mirrors T1's end state), a COPY
/// moves no card and is named by its OWN stack-entry id (mirrors T3's end
/// state), and (review Finding 7, fix cycle) a countered `ActivatedAbility`
/// names its UNMOVED source object and moves no card at all (CR 707.10b: a
/// copy of an ability has the same source as the original). This third half is
/// this function's own genuinely new emission branch (plan §3.6 collapsed the
/// function's 20-variant ability OR-list onto the same `named` shape
/// `effects/mod.rs`'s arm uses, which is new here) and had no probe on either
/// counter path before this fix cycle. This is the only pin on
/// `resolution::counter_stack_object` -- a `pub` function with zero production
/// callers (plan §3.6 / `OOS-DX25-5`): both counter effects in the corpus
/// resolve through `Effect::CounterSpell` alone, but the function is `pub` API
/// and leaving one of two counter paths carrying PB-DX25's pre-fix defect is
/// precisely how a future caller would inherit it.
#[test]
fn test_dx25_both_engine_counter_paths_agree() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![]);

    // ── Half 1: a MutatingCreatureSpell moves its card (mirrors T1's shape) ──
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Mock Mutating Beast")
                    .in_zone(ZoneId::Stack)
                    .with_card_id(CardId("mock-mutating-beast".to_string())),
            )
            .object(wolf_spec(p1))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let beast_card_id = find_object(&state, "Mock Mutating Beast");
        let wolf_id = find_object(&state, "Mock Wolf");
        let stack_entry_id =
            push_mutating_creature_spell_stack_object(&mut state, beast_card_id, wolf_id, p1);

        let events =
            mtg_engine::rules::resolution::counter_stack_object(&mut state, stack_entry_id)
                .unwrap_or_else(|e| {
                    panic!(
                        "counter_stack_object (MutatingCreatureSpell) failed: {:?}",
                        e
                    )
                });

        assert!(
            state.stack_objects().is_empty(),
            "CR 701.6a: the stack entry must be removed"
        );
        let beast_graveyard_id = find_in_zone(&state, "Mock Mutating Beast", ZoneId::Graveyard(p1))
            .unwrap_or_else(|| {
                panic!(
                    "CR 701.6a / CR 702.140a: counter_stack_object must move a \
                         MutatingCreatureSpell's card to the graveyard, exactly like \
                         Effect::CounterSpell does (T1) -- the second counter path must \
                         not carry PB-DX25's pre-fix defect"
                )
            });
        assert!(
            find_in_zone(&state, "Mock Mutating Beast", ZoneId::Stack).is_none(),
            "the card must no longer be in ZoneId::Stack"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, GameEvent::SpellCountered { .. }))
                .count(),
            1,
            "exactly one SpellCountered event expected, got {:?}",
            events
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::SpellCountered { source_object_id, .. }
                if *source_object_id == beast_graveyard_id
            )),
            "SpellCountered.source_object_id must be the post-move graveyard id"
        );
    }

    // ── Half 2: a copy moves no card (mirrors T3's shape) ──
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Mock Original Spell")
                    .in_zone(ZoneId::Stack)
                    .with_card_id(CardId("mock-original-spell".to_string())),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let original_card_id = find_object(&state, "Mock Original Spell");
        let original_stack_id = {
            let stack_id = test_util::next_object_id(&mut state);
            state.stack_objects_mut().push_back(StackObject {
                id: stack_id,
                controller: p1,
                kind: StackObjectKind::Spell {
                    source_object: original_card_id,
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
                x_value: 0,
                evidence_collected: false,
                spliced_effects: vec![],
                spliced_card_ids: vec![],
                modes_chosen: vec![],
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
            });
            stack_id
        };

        let (copy_stack_id, _copy_event) =
            mtg_engine::rules::copy::copy_spell_on_stack(&mut state, original_stack_id, p2, false)
                .unwrap_or_else(|e| panic!("copy_spell_on_stack failed: {:?}", e));
        assert_eq!(
            state.stack_objects().len(),
            2,
            "original + copy on the stack"
        );

        let events = mtg_engine::rules::resolution::counter_stack_object(&mut state, copy_stack_id)
            .unwrap_or_else(|e| panic!("counter_stack_object (copy) failed: {:?}", e));

        assert_eq!(
            state.stack_objects().len(),
            1,
            "CR 707.10a: only the copy's entry should be removed"
        );
        assert!(
            state
                .stack_objects()
                .iter()
                .any(|so| so.id == original_stack_id),
            "the ORIGINAL's stack entry must be untouched"
        );
        assert_eq!(
            state.objects().get(&original_card_id).map(|o| o.zone),
            Some(ZoneId::Stack),
            "CR 707.10: the ORIGINAL's card must still be in ZoneId::Stack -- \
             counter_stack_object must not move it when countering a copy"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, GameEvent::SpellCountered { .. }))
                .count(),
            1,
            "exactly one SpellCountered event expected, got {:?}",
            events
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::SpellCountered { stack_object_id, source_object_id, .. }
                if *stack_object_id == copy_stack_id && *source_object_id == copy_stack_id
            )),
            "counter_stack_object must name a countered copy by its OWN stack-entry \
             id for both stack_object_id and source_object_id, exactly like \
             Effect::CounterSpell does (T3) -- got {:?}",
            events
        );
    }

    // ── Half 3 (review Finding 7): a countered ActivatedAbility names its
    //    UNMOVED source and moves no card (CR 707.10b) -- this function's own
    //    genuinely new emission branch, previously untested on either counter
    //    path. ──
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Mock Ability Source")
                    .in_zone(ZoneId::Battlefield)
                    .with_card_id(CardId("mock-ability-source".to_string())),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let source_object = find_object(&state, "Mock Ability Source");
        let stack_entry_id = {
            let stack_id = test_util::next_object_id(&mut state);
            state.stack_objects_mut().push_back(StackObject {
                id: stack_id,
                controller: p1,
                kind: StackObjectKind::ActivatedAbility {
                    source_object,
                    ability_index: 0,
                    embedded_effect: None,
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
                x_value: 0,
                evidence_collected: false,
                spliced_effects: vec![],
                spliced_card_ids: vec![],
                modes_chosen: vec![],
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
            });
            stack_id
        };

        let events =
            mtg_engine::rules::resolution::counter_stack_object(&mut state, stack_entry_id)
                .unwrap_or_else(|e| {
                    panic!("counter_stack_object (ActivatedAbility) failed: {:?}", e)
                });

        assert!(
            state.stack_objects().is_empty(),
            "CR 701.6a: the ability's stack entry must be removed"
        );
        assert_eq!(
            state.objects().get(&source_object).map(|o| o.zone),
            Some(ZoneId::Battlefield),
            "CR 701.6a / CR 707.10b: countering an ACTIVATED ABILITY moves no \
             card at all -- the source stays exactly where it was"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, GameEvent::SpellCountered { .. }))
                .count(),
            1,
            "exactly one SpellCountered event expected, got {:?}",
            events
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::SpellCountered { source_object_id, .. }
                if *source_object_id == source_object
            )),
            "CR 707.10b: SpellCountered.source_object_id must name the \
             ability's UNMOVED source -- got {:?}",
            events
        );
    }
}
