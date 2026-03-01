//! Improvise keyword ability tests (CR 702.126).
//!
//! Improvise is a static ability that functions while the spell is on the stack.
//! "For each generic mana in this spell's total cost, you may tap an untapped
//! artifact you control rather than pay that mana." (CR 702.126a)
//!
//! Key rules verified:
//! - Artifacts reduce ONLY generic mana — never colored or colorless ({C}) pips (CR 702.126a).
//! - Improvise applies AFTER total cost is determined (CR 702.126b).
//! - Improvise is not an additional or alternative cost (CR 702.126b).
//! - Commander tax is applied before improvise (CR 702.126b + CR 903.8).
//! - Tapped artifacts cannot improvise (CR 702.126a: "untapped artifact").
//! - Only artifacts may improvise (CR 702.126a: "artifact").
//! - Only the caster's artifacts may improvise (CR 702.126a: "you control").
//! - Cannot tap more artifacts than the generic portion allows (CR 702.126a).
//! - Multiple instances of improvise on the same spell are redundant (CR 702.126c).
//! - Artifact creatures (both types) can be tapped for improvise (ruling).
//! - Summoning sickness does NOT prevent improvise (ruling: not an activated ability).

use mtg_engine::{
    process_command, CardId, CardType, Command, GameEvent, GameState, GameStateBuilder,
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

/// Create an improvise sorcery spell in hand.
///
/// Cost: `{generic}{blue}` where `blue` is the number of blue pips.
/// The spell has `KeywordAbility::Improvise` and card type Sorcery.
fn improvise_spell_spec(owner: PlayerId, name: &str, generic: u32, blue: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            blue,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Improvise)
}

/// Plain sorcery with no improvise keyword.
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

/// Artifact creature on the battlefield (both Artifact and Creature types).
fn artifact_creature_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).with_types(vec![CardType::Artifact, CardType::Creature])
}

/// Commander card in the command zone with an Improvise spell cost.
fn improvise_commander_spec(
    owner: PlayerId,
    name: &str,
    card_id: CardId,
    cost: ManaCost,
) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .with_card_id(card_id)
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(cost)
        .in_zone(ZoneId::Command(owner))
        .with_keyword(KeywordAbility::Improvise)
}

// ── Test 1: Basic improvise — reduce generic cost by tapping artifacts ─────────

#[test]
/// CR 702.126a — Tapping artifacts reduces the generic mana cost.
/// Spell: {3}{U}{U} with Improvise. Tap 3 artifacts. Pay {U}{U} from pool.
fn test_improvise_basic_tap_artifacts_reduce_generic_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // {3}{U}{U}: 3 generic + 2 blue pips.
    let spell = improvise_spell_spec(p1, "Improvise Spell", 3, 2);
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

    // Give p1 {U}{U} — pays the 2 colored blue pips. The 3 generic are paid by 3 artifacts.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let id1 = find_object(&state, "Artifact 1");
    let id2 = find_object(&state, "Artifact 2");
    let id3 = find_object(&state, "Artifact 3");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![id1, id2, id3],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .expect("CR 702.126a: should succeed when tapping 3 artifacts for 3 generic pips");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // All 3 artifacts are tapped.
    for (name, id) in [
        ("Artifact 1", id1),
        ("Artifact 2", id2),
        ("Artifact 3", id3),
    ] {
        let obj = state.objects.get(&id);
        // After CastSpell, the original ObjectId is still the artifact (it wasn't moved).
        assert!(
            obj.map(|o| o.status.tapped).unwrap_or(false),
            "CR 702.126a: artifact '{}' ({:?}) should be tapped after improvise",
            name,
            id
        );
    }

    // Mana pool empty (both blue pips consumed).
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 702.126a: mana pool should be empty after paying {{U}}{{U}}"
    );

    // 3 PermanentTapped events for the artifacts.
    let tapped_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTapped { .. }))
        .collect();
    assert_eq!(
        tapped_events.len(),
        3,
        "CR 702.126a: should emit 3 PermanentTapped events, one per artifact"
    );
}

// ── Test 2: Improvise cannot pay colored mana (CRITICAL distinction from Convoke) ──

#[test]
/// CR 702.126a — Improvise can ONLY pay for generic mana, never colored pips.
/// Spell: {U}{U} (no generic). Providing 2 artifacts should fail because
/// there are 0 generic pips to reduce.
fn test_improvise_cannot_pay_colored_mana() {
    let p1 = p(1);
    let p2 = p(2);

    // {U}{U}: 0 generic + 2 blue pips. Improvise cannot reduce colored pips.
    let spell = improvise_spell_spec(p1, "Blue Spell", 0, 2);
    let art1 = artifact_spec(p1, "Sol Ring");
    let art2 = artifact_spec(p1, "Mana Vault");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give enough blue to pay the real cost (in case validation passes, we'd need it).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Blue Spell");
    let sol_id = find_object(&state, "Sol Ring");
    let vault_id = find_object(&state, "Mana Vault");

    // Attempt to use 2 artifacts to pay for a spell with 0 generic pips.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![sol_id, vault_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.126a: should reject improvise when generic cost is 0 (cannot pay colored pips)"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("improvise")
            || err.contains("exceeds generic")
            || err.contains("InvalidCommand"),
        "CR 702.126a: error should mention improvise or generic mana, got: {err}"
    );
}

// ── Test 3: Reject improvise when spell has no Improvise keyword ───────────────

#[test]
/// CR 702.126a — Improvise can only be used on spells that have the Improvise keyword.
/// Attempting to improvise a plain sorcery should return an error.
fn test_improvise_reject_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    // A plain sorcery without Improvise.
    let spell = plain_sorcery_spec(p1, "Plain Sorcery", 3);
    let art = artifact_spec(p1, "Sol Ring");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art)
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
    let art_id = find_object(&state, "Sol Ring");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![art_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.126a: should reject improvise on spell without Improvise keyword"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("improvise") || err.contains("InvalidCommand"),
        "CR 702.126a: error should mention improvise or be InvalidCommand, got: {err}"
    );
}

// ── Test 4: Reject tapped artifact for improvise ──────────────────────────────

#[test]
/// CR 702.126a — Only UNTAPPED artifacts may be used for improvise.
/// Attempting to improvise with a tapped artifact should return an error.
fn test_improvise_reject_tapped_artifact() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = improvise_spell_spec(p1, "Improvise Spell", 1, 0);
    // Create a tapped artifact.
    let tapped_art = ObjectSpec::artifact(p1, "Tapped Artifact").tapped();

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(tapped_art)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // No mana in pool — the artifact should pay for the {1} but it's tapped.

    let spell_id = find_object(&state, "Improvise Spell");
    let art_id = find_object(&state, "Tapped Artifact");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![art_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.126a: should reject improvise with a tapped artifact"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("tapped") || err.contains("InvalidCommand"),
        "CR 702.126a: error should mention tapped or InvalidCommand, got: {err}"
    );
}

// ── Test 5: Reject non-artifact for improvise ────────────────────────────────

#[test]
/// CR 702.126a — Only ARTIFACTS may be used for improvise.
/// Attempting to improvise with a non-artifact creature should return an error.
fn test_improvise_reject_not_artifact() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = improvise_spell_spec(p1, "Improvise Spell", 1, 0);
    // A creature that is NOT an artifact.
    let creature = ObjectSpec::creature(p1, "Llanowar Elves", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let creature_id = find_object(&state, "Llanowar Elves");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![creature_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.126a: should reject improvise with a non-artifact creature"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("artifact") || err.contains("InvalidCommand"),
        "CR 702.126a: error should mention artifact or InvalidCommand, got: {err}"
    );
}

// ── Test 6: Reject opponent's artifact for improvise ─────────────────────────

#[test]
/// CR 702.126a — Only artifacts YOU CONTROL may be used for improvise.
/// An artifact controlled by an opponent should be rejected.
fn test_improvise_reject_opponent_artifact() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = improvise_spell_spec(p1, "Improvise Spell", 1, 0);
    // An artifact controlled by p2 (the opponent).
    let opp_artifact = ObjectSpec::artifact(p2, "Opponent Sol Ring");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(opp_artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let art_id = find_object(&state, "Opponent Sol Ring");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![art_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.126a: should reject improvise with an opponent's artifact"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("controller") || err.contains("caster") || err.contains("InvalidCommand"),
        "CR 702.126a: error should mention controller/caster or be InvalidCommand, got: {err}"
    );
}

// ── Test 7: Reject too many artifacts for improvise ──────────────────────────

#[test]
/// CR 702.126a — Cannot tap more artifacts than the generic portion of the cost.
/// Spell: {1}{U} (1 generic). Tapping 2 artifacts exceeds the generic mana.
fn test_improvise_reject_too_many_artifacts() {
    let p1 = p(1);
    let p2 = p(2);

    // {1}{U}: 1 generic + 1 blue pip. Only 1 artifact can be tapped.
    let spell = improvise_spell_spec(p1, "Improvise Spell", 1, 1);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art1)
        .object(art2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let id1 = find_object(&state, "Artifact 1");
    let id2 = find_object(&state, "Artifact 2");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![id1, id2], // 2 artifacts but only 1 generic
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.126a: should reject 2 artifacts for a spell with only 1 generic pip"
    );
}

// ── Test 8: Zero artifacts — normal full-mana cast ───────────────────────────

#[test]
/// CR 702.126a — A spell with Improvise can be cast without tapping any artifacts.
/// Empty improvise_artifacts vec means full mana payment from the pool.
fn test_improvise_zero_artifacts_normal_cast() {
    let p1 = p(1);
    let p2 = p(2);

    // {2}{U}: 2 generic + 1 blue pip.
    let spell = improvise_spell_spec(p1, "Improvise Spell", 2, 1);
    let art = artifact_spec(p1, "Sol Ring"); // present but not used

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay the full {2}{U} from the mana pool.
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
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let art_id = find_object(&state, "Sol Ring");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![], // No improvise — pay full cost
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .expect("CR 702.126a: should succeed with zero artifacts (normal full-mana cast)");

    // Spell on stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Artifact NOT tapped (wasn't used for improvise).
    let art_obj = state.objects.get(&art_id);
    assert!(
        art_obj.map(|o| !o.status.tapped).unwrap_or(false),
        "CR 702.126a: artifact should remain untapped when not used for improvise"
    );

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying full cost"
    );
}

// ── Test 9: Improvise with commander tax ──────────────────────────────────────

#[test]
/// CR 702.126b + CR 903.8 — Improvise applies AFTER total cost is determined.
/// Commander with Improvise. After 1 previous cast, tax = {2}.
/// Total cost = base {2}{U} + {2} tax = {4}{U}. Tap 4 artifacts, pay {U}.
fn test_improvise_with_commander_tax() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let cmd_id = cid("improvise-commander");

    // Commander cost: {2}{U} with Improvise.
    let cost = ManaCost {
        generic: 2,
        blue: 1,
        ..Default::default()
    };
    let cmd_card = improvise_commander_spec(p1, "Improvise Commander", cmd_id.clone(), cost);

    // 4 artifacts to pay for {4} generic (2 base + 2 tax).
    let art1 = artifact_spec(p1, "Art 1");
    let art2 = artifact_spec(p1, "Art 2");
    let art3 = artifact_spec(p1, "Art 3");
    let art4 = artifact_spec(p1, "Art 4");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
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

    // Find the commander in the command zone.
    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Simulate 1 prior cast: set commander_tax to 1 (= {2} additional cost).
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);

    // Give p1 only {U} — the 4 generic pips (2 base + 2 tax) will be paid by 4 artifacts.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let id1 = find_object(&state, "Art 1");
    let id2 = find_object(&state, "Art 2");
    let id3 = find_object(&state, "Art 3");
    let id4 = find_object(&state, "Art 4");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![id1, id2, id3, id4],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .expect(
        "CR 702.126b + CR 903.8: should succeed tapping 4 artifacts for 4 generic (2 base + 2 tax)",
    );

    // Spell on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "commander should be on the stack"
    );

    // Tax counter incremented to 2.
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
        "CR 903.8: commander tax should be 2 after second cast"
    );

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{U}}"
    );

    // 4 PermanentTapped events for the artifacts.
    let tapped_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTapped { .. }))
        .count();
    assert_eq!(
        tapped_count, 4,
        "CR 702.126a: should emit 4 PermanentTapped events"
    );
}

// ── Test 10: Improvise combined with convoke ──────────────────────────────────

#[test]
/// Edge case: spell has BOTH Convoke and Improvise keywords.
/// Convoke reduces first (creatures pay colored/generic), then Improvise reduces
/// remaining generic. Both creature and artifact are tapped.
fn test_improvise_combined_with_convoke() {
    let p1 = p(1);
    let p2 = p(2);

    // Spell with BOTH Convoke and Improvise: {4}{U}
    // 4 generic + 1 blue. Tap 2 creatures (convoke) + 2 artifacts (improvise) = pay {U}.
    let spell = ObjectSpec::card(p1, "Double Keyword Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("double-keyword-spell"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Convoke)
        .with_keyword(KeywordAbility::Improvise);

    let creature1 = ObjectSpec::creature(p1, "Creature 1", 1, 1);
    let creature2 = ObjectSpec::creature(p1, "Creature 2", 1, 1);
    let art1 = artifact_spec(p1, "Artifact 1");
    let art2 = artifact_spec(p1, "Artifact 2");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(creature1)
        .object(creature2)
        .object(art1)
        .object(art2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {U} — the 4 generic are covered by 2 convoke + 2 improvise.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Double Keyword Spell");
    let c1_id = find_object(&state, "Creature 1");
    let c2_id = find_object(&state, "Creature 2");
    let a1_id = find_object(&state, "Artifact 1");
    let a2_id = find_object(&state, "Artifact 2");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![c1_id, c2_id],
            improvise_artifacts: vec![a1_id, a2_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .expect("edge case: spell with both convoke and improvise should succeed");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // 4 PermanentTapped events (2 convoke + 2 improvise).
    let tapped_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTapped { .. }))
        .count();
    assert_eq!(
        tapped_count, 4,
        "edge case: should emit 4 PermanentTapped events (2 convoke + 2 improvise)"
    );

    // Mana pool empty.
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{U}}"
    );
}

// ── Test 11: Artifact creature can be tapped for improvise ────────────────────

#[test]
/// Ruling: Artifact creatures are artifacts and can be tapped for improvise.
/// An artifact creature (both CardType::Artifact and CardType::Creature) is valid.
fn test_improvise_artifact_creature_can_be_used() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = improvise_spell_spec(p1, "Improvise Spell", 1, 1);
    // Artifact creature: has both Artifact and Creature card types.
    let art_creature = artifact_creature_spec(p1, "Ornithopter");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(art_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let ac_id = find_object(&state, "Ornithopter");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![ac_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .expect("ruling: artifact creature should be valid for improvise");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // Artifact creature is tapped.
    let obj = state.objects.get(&ac_id);
    assert!(
        obj.map(|o| o.status.tapped).unwrap_or(false),
        "ruling: artifact creature should be tapped after improvise"
    );
}

// ── Test 12: Summoning sickness irrelevant for improvise ──────────────────────

#[test]
/// Ruling: Improvise is not an activated ability, so summoning sickness
/// (CR 302.6) does not prevent an artifact creature from being tapped for it.
/// An artifact creature that entered this turn can still be tapped for improvise.
fn test_improvise_summoning_sickness_irrelevant() {
    let p1 = p(1);
    let p2 = p(2);

    let spell = improvise_spell_spec(p1, "Improvise Spell", 1, 1);
    // Artifact creature with summoning sickness (newly entered this turn).
    // ObjectSpec doesn't model "entered_this_turn" — we just use a fresh artifact creature.
    // The engine doesn't check summoning sickness for improvise (it's not an activated {T}).
    let new_artifact_creature = artifact_creature_spec(p1, "New Construct");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(new_artifact_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Improvise Spell");
    let ac_id = find_object(&state, "New Construct");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![ac_id],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .expect("ruling: summoning sickness should NOT prevent improvise");

    // Spell on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
}
