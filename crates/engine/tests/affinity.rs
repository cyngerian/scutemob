//! Affinity keyword ability tests (CR 702.41).
//!
//! Affinity is a static ability that functions while the spell is on the stack.
//! "Affinity for [text]" means "This spell costs {1} less to cast for each
//! [text] you control." (CR 702.41a)
//!
//! Key rules verified:
//! - Affinity reduces ONLY generic mana — colored pips are unaffected (CR 601.2f).
//! - Generic mana cost cannot go below 0 (CR 601.2f).
//! - Affinity is automatic — no player choices required.
//! - Affinity applies AFTER total cost is determined including commander tax and kicker (CR 601.2f).
//! - Affinity applies BEFORE convoke/improvise/delve payment methods.
//! - Affinity counts ALL matching permanents — tapped or untapped (CR 702.41a).
//! - Affinity counts only permanents the caster controls (CR 702.41a: "you control").
//! - Multiple instances of affinity are cumulative (CR 702.41b).
//! - Artifact creatures count as artifacts for affinity (ruling).

use mtg_engine::{
    process_command, AffinityTarget, CardId, CardType, Command, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SubType, SuperType,
    ZoneId,
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

/// Affinity for artifacts sorcery spell in hand.
///
/// Cost: {generic} (no colored pips by default).
fn affinity_spell_spec(owner: PlayerId, name: &str, generic: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts))
}

/// Affinity for artifacts sorcery with blue pips.
fn affinity_spell_with_blue_spec(
    owner: PlayerId,
    name: &str,
    generic: u32,
    blue: u32,
) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            blue,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts))
}

/// Plain sorcery with no affinity keyword.
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

/// Untapped artifact (non-creature) on the battlefield controlled by `owner`.
fn artifact_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
}

/// Tapped artifact on the battlefield controlled by `owner`.
fn tapped_artifact_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name).tapped()
}

/// Artifact creature on the battlefield (both Artifact and Creature types).
fn artifact_creature_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 2, 2).with_types(vec![CardType::Artifact, CardType::Creature])
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
        },
    )
}

// ── Test 1: Basic affinity — reduce generic cost by artifact count ─────────

#[test]
/// CR 702.41a — Affinity for artifacts reduces generic cost by the number of
/// artifacts the caster controls. Spell {4} with 3 artifacts controlled: pay {1}.
fn test_affinity_basic_reduce_generic_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} with Affinity for artifacts. Player controls 3 artifacts. Reduced cost: {4-3} = {1}.
    let spell = affinity_spell_spec(p1, "Affinity Spell", 4);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {1} — pays the reduced cost (4 - 3 = 1 generic remaining).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41a: should succeed when 3 artifacts reduce {4} to {1}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool should be empty (1 colorless consumed).
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.41a: mana pool should be empty after affinity reduction + payment"
    );
}

// ── Test 2: Affinity reduces cost to zero ─────────────────────────────────────

#[test]
/// CR 702.41a + CR 601.2f — Affinity for artifacts with enough artifacts
/// to reduce cost to exactly {0}. Cast for free.
fn test_affinity_reduce_to_zero() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} with Affinity for artifacts. Player controls 4 artifacts. Reduced cost: {0}.
    let spell = affinity_spell_spec(p1, "Affinity Spell", 4);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");
    let art4 = artifact_spec(p1, "Artifact 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 nothing — spell should be free.
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41a + CR 601.2f: should succeed when 4 artifacts reduce {4} to {0}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool still empty (nothing to pay).
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 601.2f: mana pool should be empty — spell cost reduced to {{0}}"
    );
}

// ── Test 3: Affinity cost floors at 0 — excess artifacts are ignored ──────────

#[test]
/// CR 601.2f — Cost cannot go below {0}. Having more artifacts than the spell's
/// generic cost is allowed; the cost simply floors at {0}.
fn test_affinity_excess_artifacts_no_negative_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} with Affinity for artifacts. Player controls 6 artifacts. Cost floors at {0}.
    let spell = affinity_spell_spec(p1, "Affinity Spell", 4);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");
    let art4 = artifact_spec(p1, "Artifact 4");
    let art5 = artifact_spec(p1, "Artifact 5");
    let art6 = artifact_spec(p1, "Artifact 6");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .object(art5)
        .object(art6)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 nothing — 6 artifacts > 4 generic, cost floors at {0}.
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: excess artifacts should floor cost at {0}, not go negative");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
}

// ── Test 4: Affinity does not reduce colored pips ────────────────────────────

#[test]
/// CR 702.41a + CR 601.2f — Affinity reduces generic mana only.
/// Colored pips must still be paid from the pool. Spell {4}{U} with 4 artifacts:
/// generic reduced to {0}, must still pay {U}.
fn test_affinity_does_not_reduce_colored_pips() {
    let p1 = p(1);
    let p2 = p(2);

    // {4}{U} with Affinity for artifacts. 4 artifacts → generic {0}. Must still pay {U}.
    let spell = affinity_spell_with_blue_spec(p1, "Affinity Spell", 4, 1);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");
    let art4 = artifact_spec(p1, "Artifact 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {U} — colored pip not reduced by affinity.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41a: should succeed when paying {U} after affinity reduces {4} to {0}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool should be empty — {U} was consumed.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.41a: the {{U}} pip should have been consumed; pool should be empty"
    );
}

// ── Test 5: No keyword, no reduction ─────────────────────────────────────────

#[test]
/// Negative test — a spell WITHOUT the affinity keyword gets no reduction.
/// The full mana cost must be paid. Spell {4} with 4 artifacts but no affinity.
fn test_affinity_no_keyword_no_reduction() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} WITHOUT affinity. Player controls 4 artifacts. Full {4} must be paid.
    let spell = plain_sorcery_spec(p1, "Plain Spell", 4);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");
    let art4 = artifact_spec(p1, "Artifact 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 only {1} — not enough without affinity reduction.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Spell");

    // Should FAIL — no affinity keyword means no reduction, need {4} not {1}.
    let result = cast_spell(state, p1, spell_id);
    assert!(
        result.is_err(),
        "Negative test: spell without affinity keyword should not get cost reduction"
    );
}

// ── Test 6: Affinity counts tapped artifacts ─────────────────────────────────

#[test]
/// CR 702.41a — Affinity counts ALL artifacts the caster controls, including
/// tapped ones. Unlike Improvise (which requires untapped), Affinity just counts.
fn test_affinity_counts_tapped_artifacts() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} with Affinity. 2 untapped + 2 tapped = 4 artifacts total. Cost reduces to {0}.
    let spell = affinity_spell_spec(p1, "Affinity Spell", 4);
    let art1 = artifact_spec(p1, "Untapped Artifact 1");
    let art2 = artifact_spec(p1, "Untapped Artifact 2");
    let art3 = tapped_artifact_spec(p1, "Tapped Artifact 1");
    let art4 = tapped_artifact_spec(p1, "Tapped Artifact 2");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 nothing — 4 artifacts (incl. tapped) reduce {4} to {0}.
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41a: tapped artifacts should count for affinity, reducing {4} to {0}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
}

// ── Test 7: Affinity only counts controlled permanents ────────────────────────

#[test]
/// CR 702.41a — "you control" — affinity counts only the caster's artifacts.
/// Opponent's artifacts do NOT reduce cost.
fn test_affinity_only_counts_controlled_permanents() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} with Affinity. p1 controls 1 artifact (cost reduces to {3}).
    // p2 controls 3 artifacts (do NOT count for p1's affinity).
    let spell = affinity_spell_spec(p1, "Affinity Spell", 4);
    let p1_art = artifact_spec(p1, "P1 Artifact");
    let p2_art1 = artifact_spec(p2, "P2 Artifact 1");
    let p2_art2 = artifact_spec(p2, "P2 Artifact 2");
    let p2_art3 = artifact_spec(p2, "P2 Artifact 3");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(p1_art)
        .object(p2_art1)
        .object(p2_art2)
        .object(p2_art3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {3} — only own 1 artifact counts, so {4-1} = {3} must be paid.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41a: should succeed with {3} (only own 1 artifact counts, 4-1=3)");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.41a: mana pool should be empty after paying {{3}}"
    );
}

// ── Test 8: Affinity combined with Improvise ──────────────────────────────────

#[test]
/// Interaction test — Affinity applies first (cost reduction), then Improvise
/// taps artifacts for remaining generic. Both can reference the same artifacts
/// because affinity only counts (does not tap) while improvise taps.
fn test_affinity_combined_with_improvise() {
    let p1 = p(1);
    let p2 = p(2);

    // {6}{U} with BOTH Affinity for artifacts AND Improvise.
    // Player controls 4 artifacts. Affinity: {6} - 4 = {2}. Improvise: tap 2 artifacts.
    // Remaining to pay: {U} from pool.
    let spell = ObjectSpec::card(p1, "Affinity Improvise Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("affinity-improvise-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 6,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts))
        .with_keyword(KeywordAbility::Improvise);

    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");
    let art4 = artifact_spec(p1, "Artifact 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {U} only — affinity reduces {6} to {2}, improvise taps 2 artifacts for {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Improvise Spell");
    let id3 = find_object(&state, "Artifact 3");
    let id4 = find_object(&state, "Artifact 4");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![id3, id4],
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
    .expect(
        "Affinity + Improvise: should succeed — affinity reduces {6} to {2}, improvise taps 2 more",
    );

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{U}}"
    );

    // 2 PermanentTapped events (from improvise, not affinity).
    let tapped_count = events
        .iter()
        .filter(|e| matches!(e, mtg_engine::GameEvent::PermanentTapped { .. }))
        .count();
    assert_eq!(
        tapped_count, 2,
        "Improvise should tap 2 artifacts; affinity does not tap"
    );
}

// ── Test 9: Affinity with commander tax ───────────────────────────────────────

#[test]
/// CR 702.41a + CR 903.8 — Affinity applies to the total cost including commander tax.
/// Commander {2}{U} with Affinity, cast from command zone after 1 prior cast (tax={2}).
/// Total cost: {4}{U}. With 4 artifacts: generic reduces to {0}. Pay {U} from pool.
fn test_affinity_with_commander_tax() {
    let p1 = p(1);
    let p2 = p(2);

    let cmd_id = cid("affinity-commander");

    // Commander: {2}{U} with Affinity for artifacts. In command zone.
    let cmd_card = ObjectSpec::card(p1, "Affinity Commander")
        .with_card_id(cmd_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Command(p1))
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts));

    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");
    let art4 = artifact_spec(p1, "Artifact 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        .object(cmd_card)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(art4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Simulate 1 prior cast: commander_tax = 1 (adds {2} to cost).
    // Total cost: base {2}{U} + tax {2} = {4}{U}.
    // Affinity with 4 artifacts: {4} - 4 = {0}. Must pay {U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);

    // Give p1 {U} only.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
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
        "CR 702.41a + CR 903.8: affinity should apply after commander tax, reducing {4} to {0}",
    );

    // Spell on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "commander should be on the stack"
    );

    // Tax incremented to 2.
    let tax = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_id)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        tax, 2,
        "CR 903.8: commander tax should increment to 2 after second cast"
    );

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{U}}"
    );
}

// ── Test 10: Multiple affinity instances with different targets are cumulative ──

#[test]
/// CR 702.41b — Multiple instances of affinity are cumulative.
/// Spell {8} with "affinity for artifacts" AND "affinity for Plains".
/// Player controls 3 artifacts and 2 Plains. Each instance reduces independently:
/// 8 - 3 (artifacts) - 2 (Plains) = 3. Must pay {3}.
///
/// Note: The engine stores keywords in an OrdSet which deduplicates identical values.
/// Two identical "affinity for artifacts" instances cannot be simultaneously stored.
/// CR 702.41b is exercised here with two *distinct* affinity targets to verify that
/// both instances apply independently to the same spell.
fn test_affinity_multiple_instances_cumulative() {
    let p1 = p(1);
    let p2 = p(2);

    // {8} with BOTH Affinity for artifacts AND Affinity for Plains.
    let spell = ObjectSpec::card(p1, "Multi Affinity Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("multi-affinity-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 8,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::Artifacts))
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::BasicLandType(
            SubType("Plains".to_string()),
        )));

    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");
    let art3 = artifact_spec(p1, "Artifact 3");

    let plains1 = ObjectSpec::card(p1, "Plains 1")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Plains".to_string())]);
    let plains2 = ObjectSpec::card(p1, "Plains 2")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .object(plains1)
        .object(plains2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {3} — affinity for artifacts (-3) + affinity for Plains (-2) = 8-3-2=3.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Multi Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41b: affinity for artifacts (-3) + affinity for Plains (-2) should reduce {8} to {3}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.41b: mana pool should be empty after paying {{3}}"
    );
}

// ── Test 11: Artifact creatures count for affinity ────────────────────────────

#[test]
/// Ruling — Artifact creatures ARE artifacts. They count for "affinity for artifacts."
/// Spell {4} with Affinity. Player controls 4 artifact creatures. Cost reduces to {0}.
fn test_affinity_artifact_creature_counts() {
    let p1 = p(1);
    let p2 = p(2);

    // {4} with Affinity for artifacts. 4 artifact creatures on battlefield.
    let spell = affinity_spell_spec(p1, "Affinity Spell", 4);
    let ac1 = artifact_creature_spec(p1, "Artifact Creature 1");
    let ac2 = artifact_creature_spec(p1, "Artifact Creature 2");
    let ac3 = artifact_creature_spec(p1, "Artifact Creature 3");
    let ac4 = artifact_creature_spec(p1, "Artifact Creature 4");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(ac1)
        .object(ac2)
        .object(ac3)
        .object(ac4)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 nothing — 4 artifact creatures reduce {4} to {0}.
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("Ruling: artifact creatures are artifacts, should count for affinity — {4} to {0}");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
}

// ── Test 12: Affinity for basic land type ────────────────────────────────────

#[test]
/// CR 702.41a — "Affinity for [quality]" can match any permanent type.
/// Test "affinity for plains" — count Plains lands the caster controls.
/// Spell {3} with Affinity for Plains. Player controls 2 Plains. Cost reduces to {1}.
fn test_affinity_for_basic_land_type() {
    let p1 = p(1);
    let p2 = p(2);

    // {3} with Affinity for Plains (BasicLandType).
    let spell = ObjectSpec::card(p1, "Plains Affinity Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("plains-affinity-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Affinity(AffinityTarget::BasicLandType(
            SubType("Plains".to_string()),
        )));

    // Two Plains lands.
    let plains1 = ObjectSpec::card(p1, "Plains 1")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Plains".to_string())]);
    let plains2 = ObjectSpec::card(p1, "Plains 2")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(plains1)
        .object(plains2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {1} — 2 Plains reduce {3} to {1}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plains Affinity Spell");

    let (state, _events) = cast_spell(state, p1, spell_id)
        .expect("CR 702.41a: affinity for Plains should reduce {3} to {1} with 2 Plains");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{1}}"
    );
}
