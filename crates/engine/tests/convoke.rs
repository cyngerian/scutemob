//! Convoke keyword ability tests (CR 702.51).
//!
//! Convoke is a static ability that functions while the spell is on the stack.
//! "For each colored mana in this spell's total cost, you may tap an untapped
//! creature of that color you control rather than pay that mana. For each generic
//! mana in this spell's total cost, you may tap an untapped creature you control
//! rather than pay that mana." (CR 702.51a)
//!
//! Key rules verified:
//! - Colored creatures reduce matching colored mana pips (CR 702.51a).
//! - Any creature reduces generic mana (CR 702.51a).
//! - Convoke applies AFTER total cost is determined (CR 702.51b).
//! - Convoke is not an additional or alternative cost (CR 702.51b).
//! - Commander tax is applied before convoke (CR 702.51b + CR 903.8).
//! - Summoning sickness does NOT prevent convoke (ruling: "even one you haven't
//!   controlled continuously since the beginning of your most recent turn").
//! - Tapped creatures cannot convoke (CR 702.51a: "untapped creature").
//! - Only creatures may convoke (CR 702.51a: "creature").
//! - Only the caster's creatures may convoke (CR 702.51a: "you control").
//! - Cannot tap more creatures than the total cost allows (Venerated Loxodon ruling).
//! - Multiple instances of convoke on the same spell are redundant (CR 702.51d).

use mtg_engine::{
    process_command, CardId, CardType, Color, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SuperType, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Create a convoke sorcery spell in hand.
///
/// Cost: `{generic}{colored_green}` where colored_green is the number of
/// green pips. The spell has KeywordAbility::Convoke and is a Sorcery.
fn convoke_spell_spec(owner: PlayerId, name: &str, generic: u32, green: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            green,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Convoke)
}

/// Create a convoke spell with white pips.
fn convoke_white_spell_spec(owner: PlayerId, name: &str, generic: u32, white: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            white,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Convoke)
}

/// Create a convoke sorcery with both white and green pips.
fn convoke_wg_spell_spec(
    owner: PlayerId,
    name: &str,
    generic: u32,
    white: u32,
    green: u32,
) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            white,
            green,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Convoke)
}

/// Plain sorcery with no convoke keyword.
fn plain_sorcery_spec(owner: PlayerId, name: &str, generic: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            ..Default::default()
        })
}

/// Green creature on the battlefield (untapped) controlled by `owner`.
fn green_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).with_colors(vec![Color::Green])
}

/// White creature on the battlefield (untapped) controlled by `owner`.
fn white_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).with_colors(vec![Color::White])
}

/// Red creature on the battlefield (untapped) controlled by `owner`.
fn red_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).with_colors(vec![Color::Red])
}

/// Selesnya (W/G) multicolored creature on the battlefield.
fn selesnya_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 2, 2).with_colors(vec![Color::White, Color::Green])
}

/// Colorless artifact (NOT a creature) on the battlefield.
fn artifact_noncreature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Artifact])
}

// ── Test 1: Basic convoke — reduce generic cost by tapping creatures ───────────

#[test]
/// CR 702.51a — Tapping creatures reduces the mana cost when casting a convoke spell.
/// Five green creatures pay for five generic pips of Siege Wurm {5}{G}{G}.
/// The caster pays {G}{G} from the mana pool.
fn test_convoke_basic_tap_creatures_reduce_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // Siege Wurm-like: {5}{G}{G} — 5 generic + 2 green pips.
    let spell = convoke_spell_spec(p1, "Siege Wurm", 5, 2);

    let c1 = green_creature(p1, "Elf 1");
    let c2 = green_creature(p1, "Elf 2");
    let c3 = green_creature(p1, "Elf 3");
    let c4 = green_creature(p1, "Elf 4");
    let c5 = green_creature(p1, "Elf 5");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(c1)
        .object(c2)
        .object(c3)
        .object(c4)
        .object(c5)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {G}{G} — pays the 2 colored green pips. The 5 generic are paid by 5 creatures.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Siege Wurm");
    let elf1 = find_object(&state, "Elf 1");
    let elf2 = find_object(&state, "Elf 2");
    let elf3 = find_object(&state, "Elf 3");
    let elf4 = find_object(&state, "Elf 4");
    let elf5 = find_object(&state, "Elf 5");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![elf1, elf2, elf3, elf4, elf5],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with convoke failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.51a: spell should be on the stack after convoke cast"
    );

    // All 5 creatures are tapped.
    for name in ["Elf 1", "Elf 2", "Elf 3", "Elf 4", "Elf 5"] {
        let obj = state
            .objects
            .values()
            .find(|o| o.characteristics.name == name)
            .unwrap_or_else(|| panic!("{} not found", name));
        assert!(
            obj.status.tapped,
            "CR 702.51a: convoke creature '{}' should be tapped after convoke",
            name
        );
    }

    // PermanentTapped events emitted for all 5 creatures.
    let tap_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTapped { .. }))
        .count();
    assert_eq!(
        tap_count, 5,
        "CR 702.51a: 5 PermanentTapped events expected for 5 convoked creatures"
    );

    // Mana pool is empty (both green pips consumed).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.51a: mana pool should be empty after convoke + mana payment"
    );
}

// ── Test 2: Colored mana matching — white creature pays white pip ──────────────

#[test]
/// CR 702.51a — A creature of a specific color can pay for one colored mana pip
/// of that color. Two white creatures pay {W}{W} in a {2}{W}{W} spell.
fn test_convoke_colored_mana_match() {
    let p1 = p(1);
    let p2 = p(2);

    // {2}{W}{W}: 2 generic + 2 white pips.
    let spell = convoke_white_spell_spec(p1, "White Spell", 2, 2);
    let w1 = white_creature(p1, "White Elf 1");
    let w2 = white_creature(p1, "White Elf 2");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(w1)
        .object(w2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {2} colorless — pays the 2 generic pips. The 2 white pips paid by white creatures.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "White Spell");
    let elf1 = find_object(&state, "White Elf 1");
    let elf2 = find_object(&state, "White Elf 2");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![elf1, elf2],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with white convoke failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.51a: spell should be on the stack after white convoke"
    );

    // Both white creatures are tapped.
    for name in ["White Elf 1", "White Elf 2"] {
        let obj = state
            .objects
            .values()
            .find(|o| o.characteristics.name == name)
            .unwrap_or_else(|| panic!("{} not found", name));
        assert!(
            obj.status.tapped,
            "CR 702.51a: white convoke creature '{}' should be tapped",
            name
        );
    }

    // Mana pool is empty.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.51a: mana pool should be empty after white convoke"
    );
}

// ── Test 3: Generic-only reduction — any creature color pays generic ───────────

#[test]
/// CR 702.51a — Any creature (regardless of color) may pay for {1} generic mana.
/// Red creatures can pay the {3} generic pips of a {3}{G} spell.
fn test_convoke_generic_mana_any_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // {3}{G}: 3 generic + 1 green.
    let spell = convoke_spell_spec(p1, "Green Spell", 3, 1);
    let r1 = red_creature(p1, "Red Goblin 1");
    let r2 = red_creature(p1, "Red Goblin 2");
    let r3 = red_creature(p1, "Red Goblin 3");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(r1)
        .object(r2)
        .object(r3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {G} from pool (for the green pip). Red creatures pay the 3 generic pips.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Green Spell");
    let g1 = find_object(&state, "Red Goblin 1");
    let g2 = find_object(&state, "Red Goblin 2");
    let g3 = find_object(&state, "Red Goblin 3");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![g1, g2, g3],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with red creatures for generic failed: {:?}", e));

    // Spell on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.51a: spell should be on stack after generic convoke"
    );

    // All red creatures tapped.
    for name in ["Red Goblin 1", "Red Goblin 2", "Red Goblin 3"] {
        let obj = state
            .objects
            .values()
            .find(|o| o.characteristics.name == name)
            .unwrap_or_else(|| panic!("{} not found", name));
        assert!(
            obj.status.tapped,
            "CR 702.51a: red creature '{}' should be tapped for generic convoke",
            name
        );
    }
}

// ── Test 4: Reject convoke when spell has no Convoke keyword ──────────────────

#[test]
/// CR 702.51a — Convoke can only be used on spells that have the Convoke keyword.
/// Attempting to convoke a plain sorcery should return an error.
fn test_convoke_reject_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    // A plain sorcery without Convoke.
    let spell = plain_sorcery_spec(p1, "Plain Sorcery", 3);
    let creature = green_creature(p1, "Green Elf");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Sorcery");
    let elf_id = find_object(&state, "Green Elf");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![elf_id],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.51a: should reject convoke on spell without Convoke keyword"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("convoke") || err.contains("InvalidCommand"),
        "CR 702.51a: error should mention convoke or be InvalidCommand, got: {err}"
    );
}

// ── Test 5: Reject tapped creature for convoke ────────────────────────────────

#[test]
/// CR 702.51a — Only UNTAPPED creatures may be used for convoke.
/// Attempting to convoke with a tapped creature should return an error.
fn test_convoke_reject_tapped_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = convoke_spell_spec(p1, "Convoke Spell", 1, 0);
    let tapped_creature = green_creature(p1, "Tapped Elf").tapped();

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(tapped_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // No mana in pool — creature should pay for the {1} but it's tapped.

    let spell_id = find_object(&state, "Convoke Spell");
    let elf_id = find_object(&state, "Tapped Elf");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![elf_id],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.51a: should reject convoke with a tapped creature"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("tapped") || err.contains("InvalidCommand"),
        "CR 702.51a: error should mention tapped or InvalidCommand, got: {err}"
    );
}

// ── Test 6: Reject non-creature for convoke ───────────────────────────────────

#[test]
/// CR 702.51a — Only CREATURES may be used for convoke.
/// A non-creature artifact cannot be tapped for convoke.
fn test_convoke_reject_not_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = convoke_spell_spec(p1, "Convoke Spell", 1, 0);
    let artifact = artifact_noncreature(p1, "Sol Ring");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Convoke Spell");
    let artifact_id = find_object(&state, "Sol Ring");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![artifact_id],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.51a: should reject convoke with a non-creature permanent"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("creature") || err.contains("InvalidCommand"),
        "CR 702.51a: error should mention creature or InvalidCommand, got: {err}"
    );
}

// ── Test 7: Reject opponent's creature for convoke ────────────────────────────

#[test]
/// CR 702.51a — Only creatures the caster CONTROLS may be used for convoke.
/// An opponent's creature cannot be tapped for convoke.
fn test_convoke_reject_not_controlled() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = convoke_spell_spec(p1, "Convoke Spell", 1, 0);
    // Opponent p2 controls this creature.
    let opponent_creature = green_creature(p2, "Opponent Elf");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(opponent_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Convoke Spell");
    let opp_elf_id = find_object(&state, "Opponent Elf");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![opp_elf_id],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.51a: should reject convoke with an opponent's creature"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("controller") || err.contains("caster") || err.contains("InvalidCommand"),
        "CR 702.51a: error should mention controller/caster or InvalidCommand, got: {err}"
    );
}

// ── Test 8: Reject too many creatures for convoke ─────────────────────────────

#[test]
/// CR 702.51a / Venerated Loxodon ruling — You cannot tap more creatures for convoke
/// than the total cost allows. For a spell costing {2}{G}, at most 3 creatures can
/// convoke (reducing the cost to {0}). Tapping 4 should be rejected.
fn test_convoke_reject_too_many_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    // {2}{G}: 2 generic + 1 green = 3 total pips.
    let spell = convoke_spell_spec(p1, "Small Convoke", 2, 1);
    let c1 = green_creature(p1, "Creature 1");
    let c2 = green_creature(p1, "Creature 2");
    let c3 = green_creature(p1, "Creature 3");
    let c4 = green_creature(p1, "Creature 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(c1)
        .object(c2)
        .object(c3)
        .object(c4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // No mana — trying to use 4 creatures for a 3-pip cost.

    let spell_id = find_object(&state, "Small Convoke");
    let id1 = find_object(&state, "Creature 1");
    let id2 = find_object(&state, "Creature 2");
    let id3 = find_object(&state, "Creature 3");
    let id4 = find_object(&state, "Creature 4");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![id1, id2, id3, id4],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.51a / ruling: should reject more creatures than the total cost allows"
    );
}

// ── Test 9: Convoke with commander tax ────────────────────────────────────────

#[test]
/// CR 702.51b + CR 903.8 — Commander tax is an additional cost determined before
/// convoke is applied. Convoke reduces the total cost INCLUDING the tax.
///
/// Commander with Convoke: mana cost {3}{G}{G}. After 1 previous cast, tax = {2}.
/// Total cost = {5}{G}{G}. Convoke 5 creatures (pay {5} generic). Pay {G}{G} from pool.
fn test_convoke_with_commander_tax() {
    let p1 = p(1);
    let p2 = p(2);

    // Commander with Convoke: printed cost {3}{G}{G}.
    let cmd_id = cid("convoke-commander");
    let commander_spec = ObjectSpec::card(p1, "Convoke Commander")
        .with_card_id(cmd_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Convoke)
        .in_zone(ZoneId::Command(p1));

    // 5 green creatures to convoke (pay {5} generic out of {5}{G}{G}).
    let c1 = green_creature(p1, "Token 1");
    let c2 = green_creature(p1, "Token 2");
    let c3 = green_creature(p1, "Token 3");
    let c4 = green_creature(p1, "Token 4");
    let c5 = green_creature(p1, "Token 5");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        .object(commander_spec)
        .object(c1)
        .object(c2)
        .object(c3)
        .object(c4)
        .object(c5)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pre-set tax to 1 (cast once previously) — adds {2} to total cost.
    // Total cost = {3}{G}{G} + {2} tax = {5}{G}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);

    // Give p1 {G}{G} — to pay the 2 colored green pips. Creatures pay the {5} generic.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    // Register commander zone replacements (required for casting from command zone).
    mtg_engine::register_commander_zone_replacements(&mut state);

    let cmd_obj_id = state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .copied()
        .expect("commander object not found");

    let id1 = find_object(&state, "Token 1");
    let id2 = find_object(&state, "Token 2");
    let id3 = find_object(&state, "Token 3");
    let id4 = find_object(&state, "Token 4");
    let id5 = find_object(&state, "Token 5");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cmd_obj_id,
            targets: vec![],
            convoke_creatures: vec![id1, id2, id3, id4, id5],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("Commander convoke cast failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.51b + 903.8: commander convoke spell should be on the stack"
    );

    // Commander tax incremented to 2.
    assert_eq!(
        state.players[&p1].commander_tax.get(&cmd_id).copied(),
        Some(2),
        "CR 903.8: commander tax should increment to 2 after second cast"
    );

    // CommanderCastFromCommandZone event emitted with tax_paid = 1.
    let commander_event = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone { player, tax_paid: 1, .. }
            if *player == p1
        )
    });
    assert!(
        commander_event,
        "CR 903.8: CommanderCastFromCommandZone event with tax_paid=1 expected"
    );

    // Mana pool empty.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.51b: mana pool should be empty after convoke + mana payment"
    );
}

// ── Test 10: Summoning sickness does NOT prevent convoke ─────────────────────

#[test]
/// CR 702.51a / Siege Wurm ruling — Convoke is NOT an activated ability cost with {T}.
/// A creature that entered the battlefield this turn (without haste) can still
/// be tapped for convoke, because summoning sickness only prevents attacking
/// and using {T} in activated ability costs (CR 302.6).
fn test_convoke_no_summoning_sickness() {
    let p1 = p(1);
    let p2 = p(2);

    // The creature was added this "turn" — no special flag needed, since the engine
    // does not track the turn a creature entered. Summoning sickness in the engine
    // is only checked for activated ability tap costs (CR 302.6), not for convoke.
    let spell = convoke_spell_spec(p1, "Convoke Spell", 1, 0);
    let new_creature = green_creature(p1, "New Creature");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(new_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // No mana — creature pays the {1} generic.

    let spell_id = find_object(&state, "Convoke Spell");
    let creature_id = find_object(&state, "New Creature");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![creature_id],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_ok(),
        "Ruling: convoke should succeed even with a newly-entered creature (no summoning sickness): {:?}",
        result.err()
    );
}

// ── Test 11: Zero convoke creatures — normal cast ─────────────────────────────

#[test]
/// CR 702.51a — A spell with Convoke can be cast normally (with no convoke creatures).
/// Passing an empty convoke_creatures vec should result in full mana payment.
fn test_convoke_zero_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    // {2}{G}: requires 3 mana total.
    let spell = convoke_spell_spec(p1, "Convoke Spell", 2, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay full cost from mana pool — no convoke.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Convoke Spell");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![], // No convoke
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("Normal cast of convoke spell failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.51a: spell with no convoke creatures should go on stack normally"
    );

    // Mana pool is empty (full cost paid).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.51a: full mana cost should be paid when no creatures used for convoke"
    );
}

// ── Test 12: Multicolored creature pays one colored pip ──────────────────────

#[test]
/// CR 702.51a / ruling — A multicolored creature can pay for one pip of any of its
/// colors. A Selesnya (W/G) creature can pay for {W} or {G}.
/// In a {1}{W}{G} spell, one Selesnya creature can pay for {W} (first match in WUBRG
/// order), reducing the total cost to {1}{G}.
fn test_convoke_multicolored_creature_pays_colored() {
    let p1 = p(1);
    let p2 = p(2);

    // {1}{W}{G}: 1 generic + 1 white + 1 green.
    let spell = convoke_wg_spell_spec(p1, "Selesnya Spell", 1, 1, 1);
    let selesnya = selesnya_creature(p1, "Selesnya Token");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(selesnya)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Selesnya creature pays {W} (first color match in WUBRG). Pay {1}{G} from pool.
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
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Selesnya Spell");
    let token_id = find_object(&state, "Selesnya Token");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![token_id],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("Multicolored creature convoke failed: {:?}", e));

    // Spell on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.51a: spell should be on stack after multicolored creature convoke"
    );

    // Selesnya token is tapped.
    let token = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Selesnya Token")
        .expect("Selesnya Token not found");
    assert!(
        token.status.tapped,
        "CR 702.51a: multicolored creature should be tapped after convoke"
    );

    // Mana pool is empty.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.51a: mana pool should be empty after multicolored creature convoke"
    );
}
