//! PB-AC4: real-card end-to-end integration tests for
//! `ModeSelection.mode_targets` (CR 700.2c/700.2f per-mode targeting).
//!
//! `crates/engine/tests/pb_ac4_per_mode_targeting.rs` exercises the primitive only
//! against synthetic `CardDefinition`s built by hand. These tests drive the REAL
//! `CardDefinition`s that shipped in the PB-AC4 backfill (`casualties_of_war`,
//! `izzet_charm`, `cryptic_command`, `archmages_charm`) through the engine's normal
//! cast / resolution pipeline (`Command::CastSpell`, `process_command`), using
//! `enrich_spec_from_def` / the real `all_cards()` registry (never a hand-built
//! synthetic `Effect`).
//!
//! CR Rules covered:
//! - CR 601.2c / 700.2c: targets are announced/validated only for chosen modes.
//! - CR 700.2f: two chosen modes' targets are sliced independently (no cross-mode
//!   contamination).
//! - CR 701.5b: a countered spell is put into its owner's graveyard.
//! - CR 108.3: "owner's hand" bounce destination.
//! - CR 613.1b: gain control (no stated duration -> indefinite).
//!
//! Regression proven by `test_casualties_of_war_castable_choosing_creature_subset`:
//! before PB-AC4, Casualties of War's flat-union target validator required the
//! caster to declare a target for ALL FIVE modes (artifact/creature/enchantment/
//! land/planeswalker) regardless of which mode was actually chosen, making the card
//! uncastable in board states lacking one of those permanent types. This is exactly
//! that wrong-game-state bug, fixed.

use mtg_engine::rules::events::GameEvent;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardId, CardRegistry, CardType, Command,
    GameStateBuilder, ManaColor, ObjectId, ObjectSpec, PlayerId, StackObject, StackObjectKind,
    Step, Target, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, mtg_engine::CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn find_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_in_hand(state: &mtg_engine::GameState, player: PlayerId, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == ZoneId::Hand(player))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("card '{}' not found in {}'s hand", name, player.0))
}

fn obj_in_graveyard(state: &mtg_engine::GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn obj_in_hand(state: &mtg_engine::GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Hand(owner))
}

fn obj_controller(state: &mtg_engine::GameState, name: &str) -> PlayerId {
    state
        .objects
        .values()
        .find(|o| o.characteristics.name == name)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
        .controller
}

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

/// Cast a (possibly modal) real card by name from `player`'s hand.
fn cast_modal(
    state: mtg_engine::GameState,
    player: PlayerId,
    name: &str,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let card_id = find_in_hand(&state, player, name);
    let mut state = state;
    state.turn.priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_id,
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell({}) failed: {:?}", name, e))
}

/// Push a bare `StackObject::Spell` entry wrapping `source_object` onto
/// `state.stack_objects`. Mirrors the pattern in `pb_ac2_card_integration.rs`.
fn push_spell_stack_object(
    state: &mut mtg_engine::GameState,
    source_object: ObjectId,
    controller: PlayerId,
) -> ObjectId {
    let stack_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: stack_id,
        controller,
        kind: StackObjectKind::Spell { source_object },
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
        sacrificed_creature_powers: vec![],
        lki_counters: im::OrdMap::new(),
        lki_power: None,
    });
    stack_id
}

// ── 1. Casualties of War — castable when choosing only "destroy target creature" ──

/// CR 601.2c / CR 700.2c — Casualties of War: "Choose one or more — Destroy target
/// artifact / creature / enchantment / land / planeswalker." Casting choosing ONLY
/// the creature mode succeeds even though NO artifact, enchantment, land, or
/// planeswalker exists anywhere in the game. Before PB-AC4, the flat-union target
/// validator demanded a target be declared for all 5 modes regardless of which
/// mode(s) were chosen -- making this card uncastable in this (very common) board
/// state. This is the flagship wrong-game-state bug PB-AC4 fixes.
#[test]
fn test_casualties_of_war_castable_choosing_creature_subset() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let casualties = enrich_spec_from_def(
        ObjectSpec::card(p1, "Casualties of War")
            .with_card_id(CardId("casualties-of-war".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let victim = ObjectSpec::creature(p2, "Doomed Creature", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(casualties)
        .object(victim)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // {2}{B}{B}{G}{G} — pay generic with colorless.
    {
        let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Black, 2);
        pool.add(ManaColor::Green, 2);
        pool.add(ManaColor::Colorless, 2);
    }

    let creature_id = find_by_name(&state, "Doomed Creature");

    // No artifact, enchantment, land, or planeswalker exists ANYWHERE in this game
    // state. Choosing only mode 1 (destroy target creature, index 1 in the card's
    // `modes` list) must be castable with a single declared target.
    let (state, _) = cast_modal(
        state,
        p1,
        "Casualties of War",
        vec![Target::Object(creature_id)],
        vec![1],
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        obj_in_graveyard(&state, "Doomed Creature", p2),
        "CR 700.2c: mode 1's declared creature target must be destroyed"
    );
}

// ── 2. Izzet Charm — unchosen "counter noncreature spell" mode needs no target ────

/// CR 700.2c — Izzet Charm: "Choose one — Counter target noncreature spell unless
/// its controller pays {2}. / Izzet Charm deals 2 damage to target creature. / Draw
/// two cards, then discard two cards." Casting choosing ONLY the damage mode (index
/// 1) succeeds with NO spell on the stack to counter -- proving the unchosen
/// counter-mode's `TargetSpellWithFilter` requirement is not enforced.
#[test]
fn test_izzet_charm_damage_mode_needs_no_spell_target() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let izzet_charm = enrich_spec_from_def(
        ObjectSpec::card(p1, "Izzet Charm")
            .with_card_id(CardId("izzet-charm".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let victim = ObjectSpec::creature(p2, "Fragile Creature", 1, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(izzet_charm)
        .object(victim)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    {
        let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Blue, 1);
        pool.add(ManaColor::Red, 1);
    }

    let creature_id = find_by_name(&state, "Fragile Creature");

    // Mode 1 = "deals 2 damage to target creature". Nothing is on the stack, so a
    // flat-union validator (which would also demand a noncreature-spell target for
    // unchosen mode 0) would reject this cast.
    let (state, _) = cast_modal(
        state,
        p1,
        "Izzet Charm",
        vec![Target::Object(creature_id)],
        vec![1],
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        obj_in_graveyard(&state, "Fragile Creature", p2),
        "CR 700.2c: mode 1's 2 damage must have killed the 1/2 creature"
    );
}

// ── 3. Cryptic Command — two chosen modes sliced independently ────────────────────

/// CR 700.2f — Cryptic Command: "Choose two — Counter target spell. / Return target
/// permanent to its owner's hand. / Tap all creatures your opponents control. / Draw
/// a card." Choosing modes 0 (counter) and 1 (bounce) with two DIFFERENT declared
/// targets resolves each mode against its OWN target only: the countered spell goes
/// to its owner's graveyard (CR 701.5b); the bounced permanent goes to its owner's
/// hand (CR 108.3). A slicing bug would either reject the cast (wrong target type at
/// the wrong position) or cross-apply the effects.
#[test]
fn test_cryptic_command_counter_and_bounce_sliced_independently() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let target_spell = ObjectSpec::card(p2, "Countable Spell")
        .in_zone(ZoneId::Stack)
        .with_types(vec![CardType::Instant]);
    let bounce_target = ObjectSpec::creature(p2, "Bounce Target Creature", 3, 3);

    let cryptic = enrich_spec_from_def(
        ObjectSpec::card(p1, "Cryptic Command")
            .with_card_id(CardId("cryptic-command".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(target_spell)
        .object(bounce_target)
        .object(cryptic)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let spell_id = find_by_name(&state, "Countable Spell");
    push_spell_stack_object(&mut state, spell_id, p2);
    let bounce_id = find_by_name(&state, "Bounce Target Creature");

    // {1}{U}{U}{U} — all blue covers both the colored pips and the generic.
    {
        let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Blue, 4);
    }

    // modes_chosen ascending [0, 1] -> targets declared in the same order:
    // index 0 = mode 0's target (the spell), index 1 = mode 1's target (the creature).
    let (state, _) = cast_modal(
        state,
        p1,
        "Cryptic Command",
        vec![Target::Object(spell_id), Target::Object(bounce_id)],
        vec![0, 1],
    );

    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(so.kind, StackObjectKind::Spell { source_object } if source_object != spell_id)),
        "Cryptic Command should be on the stack above the target spell"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        obj_in_graveyard(&state, "Countable Spell", p2),
        "CR 701.5b: mode 0's countered spell must go to its owner's graveyard"
    );
    assert!(
        obj_in_hand(&state, "Bounce Target Creature", p2),
        "CR 108.3: mode 1's bounced permanent must go to its owner's hand"
    );
}

// ── 4. Archmage's Charm — new full enablement: gain-control mode ──────────────────

/// CR 613.1b / CR 700.2c — Archmage's Charm: "Choose one — Counter target spell. /
/// Target player draws two cards. / Gain control of target nonland permanent with
/// mana value 1 or less." Before PB-AC4 this entire card was a stubbed
/// `Effect::Nothing` (the plan predicted mode 2's gain-control + MV filter might
/// still be blocked; verification against the engine found `Effect::GainControl`
/// and `TargetFilter.max_cmc` both already exist). Choosing mode 2 targeting a
/// mana-value-1 permanent gains control of it for its controller.
#[test]
fn test_archmages_charm_gain_control_of_low_mv_permanent() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let archmages = enrich_spec_from_def(
        ObjectSpec::card(p1, "Archmage's Charm")
            .with_card_id(CardId("archmages-charm".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    // A mana value 1 artifact controlled by p2.
    let trinket = ObjectSpec::artifact(p2, "Cheap Trinket").with_mana_cost(mtg_engine::ManaCost {
        generic: 1,
        ..Default::default()
    });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(archmages)
        .object(trinket)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    {
        let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Blue, 3);
    }

    let trinket_id = find_by_name(&state, "Cheap Trinket");
    assert_eq!(obj_controller(&state, "Cheap Trinket"), p2);

    let (state, _) = cast_modal(
        state,
        p1,
        "Archmage's Charm",
        vec![Target::Object(trinket_id)],
        vec![2],
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        obj_controller(&state, "Cheap Trinket"),
        p1,
        "CR 613.1b: mode 2 must have given p1 control of the mana-value-1 permanent"
    );
}
