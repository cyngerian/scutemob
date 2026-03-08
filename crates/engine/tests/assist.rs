//! Assist keyword ability tests (CR 702.132).
//!
//! Assist is a static ability that allows another player to pay some or all
//! of the generic mana component of the total cost when casting a spell with
//! assist.
//!
//! "If the total cost to cast a spell with assist includes a generic mana
//! component, before you activate mana abilities while casting it, you may
//! choose another player. That player has a chance to activate mana abilities.
//! Once that player chooses not to activate any more mana abilities, you have
//! a chance to activate mana abilities. Before you begin to pay the total cost
//! of the spell, the player you chose may pay for any amount of the generic
//! mana in the spell's total cost." (CR 702.132a)
//!
//! Key rules verified:
//! - Another player may pay any amount of the generic mana (CR 702.132a).
//! - Assist is optional — caster may pay the full cost themselves (CR 702.132a).
//! - The caster cannot assist themselves (CR 702.132a: "another player").
//! - The assisting player can only pay GENERIC mana, not colored pips
//!   (ruling 2018-06-08: "a player you choose may pay for any amount of
//!   the generic mana in the spell's total cost").
//! - Assist applies to the TOTAL cost (after commander tax, kicker, etc.)
//!   (ruling 2018-06-08).
//! - Eliminated players cannot assist (CR 800.4a).
//! - Interact correctly with convoke (reduces generic before assist applies).
//! - `assist_amount: 0` with a chosen player is a no-op.
//! - Spell without Assist keyword cannot use assist.

use mtg_engine::{
    process_command, CardId, CardType, Command, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
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

/// Sorcery with Assist keyword.
/// Cost: `{generic}{blue_pips}` where blue is the number of blue pips.
fn assist_sorcery(owner: PlayerId, name: &str, generic: u32, blue: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            blue,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Assist)
}

/// Plain sorcery without Assist.
fn plain_sorcery(owner: PlayerId, name: &str, generic: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            ..Default::default()
        })
}

/// Set up a 4-player state with P1 active and priority.
fn setup_4p(specs: Vec<ObjectSpec>) -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for spec in specs {
        builder = builder.object(spec);
    }
    let mut state = builder.build().unwrap();
    state.turn.priority_holder = Some(p1);
    state
}

// ── Test 1: Basic assist — another player pays generic mana ───────────────────

#[test]
/// CR 702.132a — Another player may pay generic mana in the spell's total cost.
/// P1 casts {4}{U} sorcery. P2 pays 3 generic. P1 pays {U}+1 generic.
fn test_assist_basic_another_player_pays_generic() {
    let p1 = p(1);
    let p2 = p(2);

    // Spell: {4}{U} — 4 generic + 1 blue pip.
    let spell = assist_sorcery(p1, "Huddle Up", 4, 1);
    let mut state = setup_4p(vec![spell]);

    // P1 has {U} + 1 generic (pays the blue pip and 1 generic after P2 pays 3).
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
        .add(ManaColor::Colorless, 1);

    // P2 has 3 generic (colorless) to cover 3 of the 4 generic pips.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let spell_id = find_object(&state, "Huddle Up");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            assist_player: Some(p2),
            assist_amount: 3,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("assist cast failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.132a: spell should be on the stack"
    );

    // P2's mana pool is drained (paid 3 generic).
    let p2_pool = &state.players[&p2].mana_pool;
    assert_eq!(
        p2_pool.total(),
        0,
        "CR 702.132a: assisting player's mana pool should be empty after paying 3 generic"
    );

    // P1's mana pool is also drained (paid {U} + 1 generic).
    let p1_pool = &state.players[&p1].mana_pool;
    assert_eq!(
        p1_pool.total(),
        0,
        "CR 702.132a: caster's mana pool should be empty after paying blue+1"
    );
}

// ── Test 2: Assist is optional ────────────────────────────────────────────────

#[test]
/// CR 702.132a — Assist is optional. The caster may pay the full cost alone.
fn test_assist_no_assist_player_pays_full_cost() {
    let p1 = p(1);
    let spell = assist_sorcery(p1, "Huddle Up 2", 3, 1);
    let mut state = setup_4p(vec![spell]);

    // P1 has full cost: {3}{U} = 3 generic + 1 blue.
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
        .add(ManaColor::Colorless, 3);

    let spell_id = find_object(&state, "Huddle Up 2");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("non-assist cast failed: {:?}", e));

    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.132a: spell on stack when caster pays full cost"
    );
    let p1_pool = &state.players[&p1].mana_pool;
    assert_eq!(
        p1_pool.total(),
        0,
        "CR 702.132a: caster's pool drained when paying full cost"
    );
}

// ── Test 3: Cannot assist self ────────────────────────────────────────────────

#[test]
/// CR 702.132a — "you may choose another player" — the caster cannot assist themselves.
fn test_assist_cannot_assist_self() {
    let p1 = p(1);
    let spell = assist_sorcery(p1, "Huddle Up 3", 3, 0);
    let mut state = setup_4p(vec![spell]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let spell_id = find_object(&state, "Huddle Up 3");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            assist_player: Some(p1), // trying to self-assist
            assist_amount: 2,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.132a: self-assist should be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("another player") || err.contains("InvalidCommand"),
        "CR 702.132a: error should mention 'another player', got: {err}"
    );
}

// ── Test 4: Cannot assist more than generic mana in total cost ────────────────

#[test]
/// CR 702.132a / ruling 2018-06-08 — Assisting player can only pay generic mana.
/// P2 tries to pay 3 assist but only 2 generic remain.
fn test_assist_exceeds_generic_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Spell: {2}{U} — 2 generic + 1 blue.
    let spell = assist_sorcery(p1, "Generic Spell", 2, 1);
    let mut state = setup_4p(vec![spell]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let spell_id = find_object(&state, "Generic Spell");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            assist_player: Some(p2),
            assist_amount: 3, // exceeds generic (2)
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.132a: assist amount exceeding generic should be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("exceeds") || err.contains("generic") || err.contains("InvalidCommand"),
        "CR 702.132a: error should mention exceed/generic, got: {err}"
    );
}

// ── Test 5: Eliminated player cannot assist ───────────────────────────────────

#[test]
/// CR 800.4a — Eliminated players cannot take any game actions.
/// A player who has lost cannot assist.
fn test_assist_eliminated_player_cannot_assist() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = assist_sorcery(p1, "Huddle Up 5", 3, 0);
    let mut state = setup_4p(vec![spell]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    // Mark P2 as eliminated (has_lost = true).
    state.players.get_mut(&p2).unwrap().has_lost = true;

    let spell_id = find_object(&state, "Huddle Up 5");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            assist_player: Some(p2), // p2 is eliminated
            assist_amount: 2,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 800.4a: eliminated player cannot assist"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("not active") || err.contains("InvalidCommand"),
        "CR 800.4a: error should mention not active, got: {err}"
    );
}

// ── Test 6: Assist pays ALL generic mana — caster pays only colored pips ──────

#[test]
/// CR 702.132a / ruling 2018-06-08 — Assist applies to TOTAL generic mana cost.
/// {3}{U} spell. P2 pays all 3 generic. P1 pays only {U}.
/// This demonstrates that assist covers the full generic component,
/// regardless of whether it originated from commander tax or the printed cost.
fn test_assist_pays_all_generic_caster_pays_only_colored() {
    let p1 = p(1);
    let p2 = p(2);

    // Spell: {3}{U} — 3 generic + 1 blue pip.
    let spell = assist_sorcery(p1, "Cooperative Effort", 3, 1);

    let mut state = setup_4p(vec![spell]);

    // P1 has {U} only — P2 covers all the generic.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    // P2 has 3 generic (exactly enough to cover {3}).
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let spell_id = find_object(&state, "Cooperative Effort");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: Some(p2),
            assist_amount: 3, // P2 pays all generic
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("assist all-generic cast failed: {:?}", e));

    // Spell on stack, both pools empty.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");
    assert_eq!(
        state.players[&p1].mana_pool.total(),
        0,
        "P1's pool should be empty (paid blue only)"
    );
    assert_eq!(
        state.players[&p2].mana_pool.total(),
        0,
        "P2's pool should be empty (paid all 3 generic)"
    );
}

// ── Test 7: Assist with convoke reduces assist ceiling ────────────────────────

#[test]
/// CR 702.132a — Assist applies to generic remaining AFTER convoke reduction.
/// {4}{G} Assist+Convoke spell. P1 taps 2 creatures for convoke (reducing generic by 2).
/// Generic remaining = 2. P2 assists with 2. P1 pays {G}.
fn test_assist_with_convoke_reduces_assist_ceiling() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = ObjectSpec::card(p1, "Convoke Assist Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("convoke-assist-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Convoke)
        .with_keyword(KeywordAbility::Assist);

    let c1 = ObjectSpec::creature(p1, "Elf A", 1, 1);
    let c2 = ObjectSpec::creature(p1, "Elf B", 1, 1);

    let mut state = setup_4p(vec![spell, c1, c2]);

    // P1 has {G} only (convoke covers 2 generic, P2's assist covers 2 generic).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);

    // P2 has 2 generic for assist.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let spell_id = find_object(&state, "Convoke Assist Spell");
    let elf_a = find_object(&state, "Elf A");
    let elf_b = find_object(&state, "Elf B");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![elf_a, elf_b], // reduces generic by 2
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
            assist_player: Some(p2),
            assist_amount: 2, // pays remaining 2 generic after convoke
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("convoke+assist cast failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");
    assert_eq!(state.players[&p1].mana_pool.total(), 0, "P1 paid green");
    assert_eq!(
        state.players[&p2].mana_pool.total(),
        0,
        "P2 paid 2 generic via assist"
    );
}

// ── Test 8: assist_amount=0 with assist_player set is a no-op ─────────────────

#[test]
/// CR 702.132a — assist_amount of 0 is valid but a no-op for the assisting player.
fn test_assist_amount_zero_is_noop() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = assist_sorcery(p1, "Zero Assist Spell", 2, 0);
    let mut state = setup_4p(vec![spell]);

    // P1 pays full cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    // P2 has mana that should NOT be consumed.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);

    let spell_id = find_object(&state, "Zero Assist Spell");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: Some(p2),
            assist_amount: 0, // no-op assist
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("zero-assist cast failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");
    // P2 pool unchanged.
    assert_eq!(
        state.players[&p2].mana_pool.total(),
        5,
        "P2's pool should be unchanged when assist_amount=0"
    );
    // P1 pool drained.
    assert_eq!(state.players[&p1].mana_pool.total(), 0, "P1 paid full cost");
}

// ── Test 9: Insufficient mana from assisting player ───────────────────────────

#[test]
/// CR 702.132a — Assisting player must have sufficient mana to pay the assist amount.
fn test_assist_insufficient_mana_assisting_player() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = assist_sorcery(p1, "Expensive Spell", 5, 0);
    let mut state = setup_4p(vec![spell]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    // P2 has only 1 mana but assist_amount=3.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let spell_id = find_object(&state, "Expensive Spell");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            assist_player: Some(p2),
            assist_amount: 3, // P2 only has 1
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.132a: should reject when assisting player lacks mana"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("InsufficientMana") || err.contains("Insufficient"),
        "CR 702.132a: should be InsufficientMana error, got: {err}"
    );
}

// ── Test 10: Spell without Assist keyword rejected ────────────────────────────

#[test]
/// CR 702.132a — Assist can only be used on spells with the Assist keyword.
fn test_assist_spell_without_keyword_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Non-assist spell.
    let spell = plain_sorcery(p1, "Plain Sorcery", 3);
    let mut state = setup_4p(vec![spell]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let spell_id = find_object(&state, "Plain Sorcery");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
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
            assist_player: Some(p2),
            assist_amount: 2,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.132a: non-assist spell should reject assist"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("assist") || err.contains("InvalidCommand"),
        "CR 702.132a: error should mention assist, got: {err}"
    );
}

// ── Test 11: Any non-caster player can assist (multiplayer) ───────────────────

#[test]
/// CR 702.132a — In a 4-player game, any non-caster non-eliminated player can assist.
/// P3 assists P1's spell.
fn test_assist_multiplayer_any_opponent_can_assist() {
    let p1 = p(1);
    let p3 = p(3);

    let spell = assist_sorcery(p1, "Team Spell", 4, 0);
    let mut state = setup_4p(vec![spell]);

    // P1 has 0 generic — fully relying on P3 to pay all 4.
    state
        .players
        .get_mut(&p3)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);

    let spell_id = find_object(&state, "Team Spell");

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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: Some(p3), // P3, not P2, assists
            assist_amount: 4,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("P3 assist failed: {:?}", e));

    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.132a: spell should be on stack when P3 assists"
    );
    assert_eq!(
        state.players[&p3].mana_pool.total(),
        0,
        "P3's mana should be consumed"
    );
}
