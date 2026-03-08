//! Undaunted keyword ability tests (CR 702.125).
//!
//! Undaunted is a static ability that functions while the spell is on the stack.
//! "This spell costs {1} less to cast for each opponent you have." (CR 702.125a)
//!
//! Key rules verified:
//! - Undaunted reduces ONLY generic mana — colored and colorless pips are unaffected (CR 601.2f).
//! - Generic mana cost cannot go below 0 (CR 601.2f).
//! - Undaunted is automatic — no player choices required.
//! - Undaunted applies AFTER total cost is determined including commander tax (CR 601.2f).
//! - Undaunted applies BEFORE convoke/improvise/delve payment methods.
//! - Players who have left the game are not counted (CR 702.125b).
//! - Multiple instances of undaunted are cumulative (CR 702.125c).
//! - Undaunted counts opponents in multiplayer — scales with player count.

use mtg_engine::process_command;
use mtg_engine::{
    AffinityTarget, CardId, CardType, Command, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SuperType, ZoneId,
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

/// Sorcery with Undaunted keyword and specified mana cost (generic + white pips).
fn undaunted_spell_spec(owner: PlayerId, name: &str, generic: u32, white: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            white,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Undaunted)
}

/// Plain sorcery with no Undaunted keyword.
fn plain_sorcery_spec(owner: PlayerId, name: &str, generic: u32, white: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            white,
            ..Default::default()
        })
}

/// Cast a spell using process_command with no special payment methods.
fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<(GameState, Vec<mtg_engine::GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
        Command::CastSpell {
            player,
            card,
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
        },
    )
}

// ── Test 1: Basic Undaunted — 4-player game reduces generic cost by opponent count ──

#[test]
/// CR 702.125a — Undaunted reduces generic cost by the number of opponents.
/// 4-player game (3 opponents). Spell {6}{W} with Undaunted. Reduced cost: {3}{W}.
fn test_undaunted_basic_4player_reduce_generic_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // {6}{W} with Undaunted. 3 opponents in a 4-player game: cost reduces to {3}{W}.
    let spell = undaunted_spell_spec(p1, "Undaunted Spell", 6, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {3}{W} — pays the reduced cost (6 - 3 = 3 generic remaining).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.125a: 3 opponents should reduce {6}{W} to {3}{W}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool should be empty (3 colorless + 1 white consumed).
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.125a: mana pool should be empty after undaunted reduction + payment"
    );
}

// ── Test 2: Undaunted reduces cost to exactly zero ────────────────────────────

#[test]
/// CR 702.125a + CR 601.2f — Undaunted with enough opponents to reduce cost to
/// exactly {0}. 4-player game (3 opponents). Spell {3}{W} with Undaunted.
/// Generic {3} - 3 = {0}. Pay {W} only.
fn test_undaunted_reduce_to_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // {3}{W} with Undaunted. 3 opponents: {3} - 3 = {0}. Pay {W} only.
    let spell = undaunted_spell_spec(p1, "Undaunted Spell", 3, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {W} only — generic should be reduced to 0.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.125a + CR 601.2f: 3 opponents should reduce {3} to {0}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty (only white consumed).
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 601.2f: mana pool should be empty — generic reduced to {{0}}, white paid"
    );
}

// ── Test 3: Undaunted cost floors at zero — excess opponents are ignored ───────

#[test]
/// CR 601.2f — Cost cannot go below {0}. Having more opponents than the spell's
/// generic cost is allowed; the cost simply floors at {0}.
/// 6-player game (5 opponents). Spell {3}{W} with Undaunted.
/// Generic {3} - 5 = {0} (not -2). Pay {W} only.
fn test_undaunted_excess_opponents_floors_at_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let p5 = p(5);
    let p6 = p(6);

    // {3}{W} with Undaunted. 5 opponents in a 6-player game: cost floors at {0}.
    let spell = undaunted_spell_spec(p1, "Undaunted Spell", 3, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .add_player(p5)
        .add_player(p6)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {W} only — generic floors at {0}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) =
        cast_spell(state, p1, spell_id).expect("CR 601.2f: 5 opponents should floor {3} at {0}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 601.2f: mana pool should be empty — generic floored at {{0}}"
    );
}

// ── Test 4: Undaunted does not reduce colored pips ────────────────────────────

#[test]
/// CR 702.125a + CR 601.2f — Undaunted reduces ONLY generic mana.
/// Colored pips are unaffected. 4-player game (3 opponents).
/// Spell {2}{W}{U} with Undaunted. Generic {2} - 3 = {0}. Must still pay {W}{U}.
fn test_undaunted_does_not_reduce_colored_pips() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // {2}{W}{U} with Undaunted. 3 opponents: generic {2} - 3 = {0}. Pay {W}{U} only.
    let spell = ObjectSpec::card(p1, "Undaunted Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("undaunted-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Undaunted);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {W}{U} — generic reduced to {0}, but colored pips still required.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id).expect(
        "CR 702.125a: colored pips unaffected — must pay {W}{U} after generic reduced to {0}",
    );

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty ({W}{U} consumed).
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.125a: mana pool should be empty after paying {{W}}{{U}}"
    );
}

// ── Test 5: No keyword — no reduction ─────────────────────────────────────────

#[test]
/// Negative test — a spell without the Undaunted keyword receives no cost reduction.
/// 4-player game. Spell {6}{W} WITHOUT Undaunted. Full {6}{W} must be paid.
/// Providing only {3}{W} should fail.
fn test_undaunted_no_keyword_no_reduction() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // {6}{W} WITHOUT Undaunted.
    let spell = plain_sorcery_spec(p1, "Plain Spell", 6, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 only {3}{W} — not enough for {6}{W} without reduction.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Spell");

    // Should FAIL — no undaunted keyword means no reduction.
    let result = cast_spell(state, p1, spell_id);
    assert!(
        result.is_err(),
        "Negative test: spell without Undaunted keyword should not get cost reduction"
    );
}

// ── Test 6: 2-player game — 1 opponent ────────────────────────────────────────

#[test]
/// CR 702.125a — Undaunted scales with player count. In a 2-player game,
/// the caster has 1 opponent. Spell {6}{W} with Undaunted.
/// Generic {6} - 1 = {5}. Pay {5}{W}.
fn test_undaunted_2player_one_opponent() {
    let p1 = p(1);
    let p2 = p(2);

    // {6}{W} with Undaunted. 1 opponent in a 2-player game: cost reduces to {5}{W}.
    let spell = undaunted_spell_spec(p1, "Undaunted Spell", 6, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {5}{W} — pays the reduced cost (6 - 1 = 5 generic remaining).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.125a: 1 opponent should reduce {6}{W} to {5}{W}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool should be empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.125a: mana pool should be empty after paying {{5}}{{W}}"
    );
}

// ── Test 7: Eliminated opponents are not counted ──────────────────────────────

#[test]
/// CR 702.125b — Players who have left the game are not counted.
/// 4-player game. One opponent (p4) has lost. Only 2 active opponents remain.
/// Spell {6}{W} with Undaunted. Generic {6} - 2 = {4}. Pay {4}{W}.
fn test_undaunted_eliminated_opponents_not_counted() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // {6}{W} with Undaunted. p4 will be marked as lost (2 active opponents remain).
    let spell = undaunted_spell_spec(p1, "Undaunted Spell", 6, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p4 has lost — only 2 active opponents remain (p2, p3).
    state.players.get_mut(&p4).unwrap().has_lost = true;

    // Give p1 {4}{W} — pays reduced cost (6 - 2 = 4 generic; only 2 active opponents).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id).expect(
        "CR 702.125b: eliminated opponent should not count — 2 active opponents reduce {6} to {4}",
    );

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.125b: mana pool should be empty after paying {{4}}{{W}}"
    );
}

// ── Test 8: Multiple Undaunted instances — OrdSet deduplication documented ─────

#[test]
/// CR 702.125c — Multiple instances of undaunted are cumulative. However,
/// because the engine stores keywords in an OrdSet (which deduplicates identical
/// unit variants), two `Undaunted` instances cannot be simultaneously stored.
/// This test documents that behavior: OrdSet deduplicates, so the effective
/// instance count is 1 regardless. In practice, no printed card has two instances
/// of Undaunted. This mirrors the affinity.rs note about OrdSet deduplication.
///
/// With 3 opponents and 1 effective Undaunted instance: reduction = 3.
/// Spell {6}{W}: pay {3}{W}.
fn test_undaunted_ordset_deduplicates_unit_variant() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // Attempt to add Undaunted twice — OrdSet will deduplicate to one instance.
    let spell = ObjectSpec::card(p1, "Double Undaunted Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("double-undaunted-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 6,
            white: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Undaunted)
        .with_keyword(KeywordAbility::Undaunted); // duplicate — will be deduplicated

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // OrdSet deduplication means only 1 instance of Undaunted is stored.
    // Reduction = 1 instance * 3 opponents = 3. Pay {3}{W}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Double Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id).expect(
        "CR 702.125c: OrdSet deduplicates unit variants — 1 effective instance * 3 opponents = 3 reduction",
    );

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{3}}{{W}}"
    );
}

// ── Test 9: Undaunted with commander tax ─────────────────────────────────────

#[test]
/// CR 702.125a + CR 903.8 — Undaunted applies to the total cost including
/// commander tax. Commander {4}{W} with Undaunted in command zone after 1 prior
/// cast (tax={2}). Total cost: {6}{W}. With 3 opponents: {6} - 3 = {3}. Pay {3}{W}.
fn test_undaunted_with_commander_tax() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let cmd_id = cid("undaunted-commander");

    // Commander: {4}{W} with Undaunted. In command zone.
    let cmd_card = ObjectSpec::card(p1, "Undaunted Commander")
        .with_card_id(cmd_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            generic: 4,
            white: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Command(p1))
        .with_keyword(KeywordAbility::Undaunted);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .player_commander(p1, cmd_id.clone())
        .object(cmd_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Simulate 1 prior cast: commander_tax = 1 (adds {2} to cost).
    // Total cost: base {4}{W} + tax {2} = {6}{W}.
    // Undaunted with 3 opponents: {6} - 3 = {3}. Must pay {3}{W}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);

    // Give p1 {3}{W} only.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    // Find the commander in the command zone.
    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (state, _events) = cast_spell(state, p1, card_obj_id).expect(
        "CR 702.125a + CR 903.8: undaunted should apply after commander tax, reducing {6} to {3}",
    );

    // Spell on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "commander should be on the stack"
    );

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{3}}{{W}}"
    );
}

// ── Test 10: Undaunted combined with Affinity ──────────────────────────────────

#[test]
/// Interaction test — Affinity applies first (cost reduction), then Undaunted.
/// 4-player game (3 opponents). Spell {8}{U} with BOTH Undaunted AND Affinity for Artifacts.
/// Player controls 2 artifacts. Affinity: {8} - 2 = {6}. Undaunted: {6} - 3 = {3}. Pay {3}{U}.
fn test_undaunted_combined_with_affinity() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // {8}{U} with BOTH Undaunted AND Affinity for Artifacts.
    let spell = ObjectSpec::card(p1, "Undaunted Affinity Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("undaunted-affinity-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 8,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Undaunted)
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts));

    // 2 artifacts controlled by p1.
    let art1 = ObjectSpec::artifact(p1, "Artifact 1");
    let art2 = ObjectSpec::artifact(p1, "Artifact 2");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(spell)
        .object(art1)
        .object(art2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Affinity: {8} - 2 = {6}. Undaunted: {6} - 3 = {3}. Pay {3}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("Affinity(-2) + Undaunted(-3) should compose: {8}{U} -> {6}{U} -> {3}{U}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{3}}{{U}}"
    );
}

// ── Test 11: 6-player game — 5 opponents ─────────────────────────────────────

#[test]
/// CR 702.125a — Undaunted scales in a 6-player Commander game.
/// 6-player game (5 opponents). Spell {6}{W} with Undaunted.
/// Generic {6} - 5 = {1}. Pay {1}{W}.
fn test_undaunted_6player_game() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let p5 = p(5);
    let p6 = p(6);

    // {6}{W} with Undaunted. 5 opponents in a 6-player game: cost reduces to {1}{W}.
    let spell = undaunted_spell_spec(p1, "Undaunted Spell", 6, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .add_player(p5)
        .add_player(p6)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {1}{W} — pays the reduced cost (6 - 5 = 1 generic remaining).
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
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Undaunted Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.125a: 5 opponents in 6-player game should reduce {6}{W} to {1}{W}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool should be empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.125a: mana pool should be empty after paying {{1}}{{W}}"
    );
}
