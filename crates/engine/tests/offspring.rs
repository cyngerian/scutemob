//! Offspring keyword ability tests (CR 702.175).
//!
//! Offspring represents two linked abilities:
//! 1. A static ability: "You may pay an additional [cost] as you cast this spell." (CR 702.175a)
//! 2. A triggered ability: "When this permanent enters, if its offspring cost was paid,
//!    create a token that's a copy of it, except it's 1/1." (CR 702.175a)
//!
//! Key rules verified:
//! - Offspring is binary -- paid once or not at all (unlike Squad which can be paid N times).
//! - Paying offspring adds the offspring cost once to the total mana cost.
//! - When offspring_paid=true, an OffspringTrigger appears on the stack after the spell resolves.
//! - Resolving OffspringTrigger creates 1 token copy on the battlefield.
//! - Token copies use copiable values (CR 707.2) -- same name, types as original.
//! - Token P/T is overridden to 1/1 (CR 707.9d) via a Layer 7b SetPowerToughness effect.
//! - Tokens are NOT cast; spells_cast_this_turn is NOT incremented (ruling 2024-07-26).
//! - Providing offspring_paid=true for a spell without Offspring keyword is rejected.
//! - If the source leaves the battlefield before the trigger resolves, the token IS still
//!   created (ruling 2024-07-26 LKI -- different from Squad's skip behavior).

use mtg_engine::AdditionalCost;
use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, GameEvent, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_objects_on_battlefield<'a>(state: &'a mtg_engine::GameState, name: &str) -> Vec<ObjectId> {
    state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .collect()
}

/// Count battlefield permanents matching the given name.
fn count_on_battlefield(state: &mtg_engine::GameState, name: &str) -> usize {
    find_objects_on_battlefield(state, name).len()
}

/// Pass priority for all listed players once (round-robin, one pass each).
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Synthetic Offspring {2} creature: 2/3 for {2}{W}. Offspring {2}.
/// Modeled after Flowerfoot Swordmaster -- a small creature with Offspring {2}.
fn offspring_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("offspring-test".to_string()),
        name: "Offspring Warrior".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Offspring {2} (You may pay an additional {2} as you cast this spell. \
            If you do, when this creature enters, create a token that's a copy of it, \
            except it's 1/1.)"
            .to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Offspring),
            AbilityDefinition::Offspring {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// A plain creature without Offspring keyword -- used to verify rejection of offspring_paid=true.
fn plain_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-creature-offspring-test".to_string()),
        name: "Plain Soldier Offspring".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

/// Build the ObjectSpec for the Offspring Warrior in the given player's hand.
fn offspring_spec(player: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(player, "Offspring Warrior")
        .in_zone(ZoneId::Hand(player))
        .with_card_id(CardId("offspring-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Offspring)
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        });
    spec.power = Some(2);
    spec.toughness = Some(3);
    spec
}

/// Build the ObjectSpec for the plain creature.
fn plain_spec_offspring(player: PlayerId) -> ObjectSpec {
    ObjectSpec::card(player, "Plain Soldier Offspring")
        .in_zone(ZoneId::Hand(player))
        .with_card_id(CardId("plain-creature-offspring-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        })
}

/// Helper: build a 2-player state with Offspring Warrior in p1's hand.
/// Adds enough mana for the base cost {2}{W} plus `extra_generic` additional colorless.
fn setup_offspring_state(
    extra_generic: u32,
) -> (mtg_engine::GameState, PlayerId, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![offspring_creature_def(), plain_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(offspring_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let ps = state.players.get_mut(&p1).unwrap();
    // Base cost: {2}{W}
    ps.mana_pool.add(ManaColor::Colorless, 2);
    ps.mana_pool.add(ManaColor::White, 1);
    // Extra colorless for offspring payment
    ps.mana_pool.add(ManaColor::Colorless, extra_generic);
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Offspring Warrior")
        .map(|(id, _)| *id)
        .expect("Offspring Warrior should be in hand");

    (state, p1, p2, card_id)
}

/// Helper: cast Offspring Warrior with the given offspring_paid flag.
fn cast_offspring(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    offspring_paid: bool,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: if offspring_paid {
                vec![AdditionalCost::Offspring]
            } else {
                vec![]
            },
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "CastSpell(offspring_paid={}) failed: {:?}",
            offspring_paid, e
        )
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.175a — Offspring with offspring_paid=false: cast without paying the offspring cost.
/// Verify no OffspringTrigger is created, the spell resolves, one creature on battlefield.
#[test]
fn test_offspring_not_paid() {
    // No extra mana needed -- just base cost {2}{W}.
    let (state, p1, p2, card_id) = setup_offspring_state(0);

    // Cast without paying offspring (offspring_paid = false). Sufficient mana: {2}{W}.
    let (state, _) = cast_offspring(state, p1, card_id, false);

    // Only the spell should be on the stack -- no trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.175a intervening-if: no OffspringTrigger when offspring cost was not paid"
    );

    // Resolve the spell (p1, p2 pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Offspring Warrior is on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        1,
        "CR 702.175a: creature should be on battlefield after resolving"
    );

    // No OffspringTrigger on the stack.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.175a: no OffspringTrigger should be on stack when offspring_paid=false"
    );
}

/// CR 702.175a — Offspring with offspring_paid=true: cast paying the offspring cost.
/// After resolving the spell AND the OffspringTrigger: 2 permanents on battlefield
/// (1 original 2/3 + 1 token 1/1 copy).
#[test]
fn test_offspring_basic_paid() {
    // Extra {2} for the offspring payment.
    let (state, p1, p2, card_id) = setup_offspring_state(2);

    let (state, _) = cast_offspring(state, p1, card_id, true);

    // Only the spell is on the stack immediately after casting (no on-cast trigger).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "only the spell should be on the stack immediately after casting"
    );

    // Resolve the spell (p1, p2 pass priority). The OffspringTrigger should be placed.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After spell resolves, Offspring Warrior is on battlefield and OffspringTrigger is on stack.
    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        1,
        "original creature should be on battlefield after spell resolves"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.175a: OffspringTrigger should be on stack after spell resolves with offspring_paid=true"
    );

    // Resolve the OffspringTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Now both original and 1 token copy are on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        2,
        "CR 702.175a: original + 1 token copy should be on battlefield (offspring_paid=true)"
    );
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after OffspringTrigger resolves"
    );
}

/// CR 702.175a / CR 707.9d -- Token is a 1/1 copy.
/// Verify the token's layer-resolved P/T is (1, 1) even though the original is a 2/3.
/// Also verify the token has the same name and types as the source.
#[test]
fn test_offspring_token_is_1_1() {
    // Extra {2} for the offspring payment.
    let (state, p1, p2, card_id) = setup_offspring_state(2);

    let (state, _) = cast_offspring(state, p1, card_id, true);
    // Resolve spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve OffspringTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let all_warriors = find_objects_on_battlefield(&state, "Offspring Warrior");
    assert_eq!(
        all_warriors.len(),
        2,
        "CR 702.175a: should have 2 Offspring Warriors on battlefield (1 original + 1 token)"
    );

    // Find the token (is_token == true).
    let token_id = all_warriors
        .iter()
        .find(|id| state.objects.get(id).map(|o| o.is_token).unwrap_or(false))
        .copied()
        .expect("CR 702.175a: exactly one token should exist");

    // Find the original (is_token == false).
    let original_id = all_warriors
        .iter()
        .find(|id| state.objects.get(id).map(|o| !o.is_token).unwrap_or(false))
        .copied()
        .expect("CR 702.175a: exactly one non-token should exist");

    // Original is 2/3.
    let orig_chars = calculate_characteristics(&state, original_id)
        .expect("original should have layer-resolved characteristics");
    assert_eq!(
        orig_chars.power.unwrap_or(-1),
        2,
        "original Offspring Warrior should be 2/3 (power)"
    );
    assert_eq!(
        orig_chars.toughness.unwrap_or(-1),
        3,
        "original Offspring Warrior should be 2/3 (toughness)"
    );

    // Token is 1/1 (P/T override from Layer 7b SetPowerToughness).
    let token_chars = calculate_characteristics(&state, token_id)
        .expect("CR 702.175a: token should have layer-resolved characteristics");
    assert_eq!(
        token_chars.name, "Offspring Warrior",
        "CR 707.2: token copy should have same name as source"
    );
    assert_eq!(
        token_chars.power.unwrap_or(-1),
        1,
        "CR 707.9d: token created by Offspring should have power 1/1"
    );
    assert_eq!(
        token_chars.toughness.unwrap_or(-1),
        1,
        "CR 707.9d: token created by Offspring should have toughness 1/1"
    );
    assert!(
        token_chars.card_types.contains(&CardType::Creature),
        "CR 707.2: token copy should have same card types as source (Creature)"
    );
}

/// Validation -- cast with offspring_paid=true on a creature without Offspring keyword is rejected.
/// (CR 702.175a: offspring cost can only be paid for an offspring spell.)
#[test]
fn test_offspring_rejected_without_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(plain_spec_offspring(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let ps = state.players.get_mut(&p1).unwrap();
    ps.mana_pool.add(ManaColor::Colorless, 1);
    ps.mana_pool.add(ManaColor::White, 1);
    ps.mana_pool.add(ManaColor::Colorless, 2); // extra for bogus offspring payment
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Plain Soldier Offspring")
        .map(|(id, _)| *id)
        .expect("Plain Soldier Offspring should be in hand");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            additional_costs: vec![AdditionalCost::Offspring], // trying to pay offspring cost on non-offspring creature
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.175a: offspring_paid=true on a non-offspring creature should be rejected"
    );
}

/// Ruling 2024-07-26 -- Tokens are NOT cast: spells_cast_this_turn is NOT
/// incremented for tokens created by Offspring.
#[test]
fn test_offspring_tokens_not_cast() {
    // Extra {2} for the offspring payment.
    let (state, p1, p2, card_id) = setup_offspring_state(2);

    // Record spells_cast_this_turn before casting.
    let spells_before = state.players[&p1].spells_cast_this_turn;

    // Cast the Offspring creature.
    let (state, _) = cast_offspring(state, p1, card_id, true);

    // spells_cast_this_turn should have increased by 1 (the spell itself).
    let spells_after_cast = state.players[&p1].spells_cast_this_turn;
    assert_eq!(
        spells_after_cast,
        spells_before + 1,
        "casting the offspring creature should increment spells_cast_this_turn by 1"
    );

    // Resolve spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve OffspringTrigger (creates 1 token).
    let (state, _) = pass_all(state, &[p1, p2]);

    // spells_cast_this_turn should STILL be spells_before + 1 -- token not cast.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn,
        spells_before + 1,
        "ruling 2024-07-26: token copies created by Offspring are not cast; \
         spells_cast_this_turn should not increase"
    );

    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        2,
        "1 original + 1 token should be on battlefield"
    );
}

/// Ruling 2024-07-26 -- If source leaves battlefield before trigger resolves, token IS still
/// created (LKI). This is the KEY difference from Squad (which skips when source is gone).
#[test]
fn test_offspring_source_leaves_still_creates_token() {
    // Extra {2} for the offspring payment.
    let (state, p1, p2, card_id) = setup_offspring_state(2);

    // Cast with offspring paid.
    let (state, _) = cast_offspring(state, p1, card_id, true);

    // Resolve the spell -- Offspring Warrior enters, OffspringTrigger goes on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        1,
        "original should be on battlefield after spell resolves"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "OffspringTrigger should be on stack"
    );

    // Before resolving the trigger, find and exile the original Offspring Warrior.
    let original_id = find_objects_on_battlefield(&state, "Offspring Warrior")
        .into_iter()
        .next()
        .expect("original should still be on battlefield");

    // Manually move the original to exile (simulating removal before trigger resolves).
    let mut state = state;
    let _ = state.move_object_to_zone(original_id, ZoneId::Exile);

    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        0,
        "original should be in exile after manual removal"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "OffspringTrigger should still be on stack"
    );

    // Resolve the OffspringTrigger -- token should STILL be created.
    // Ruling 2024-07-26: "If the spell resolves but the creature with offspring leaves the
    // battlefield before the offspring ability resolves, you'll still create a token copy of it."
    let (state, _) = pass_all(state, &[p1, p2]);

    // The token should have been created even though the source left.
    assert_eq!(
        count_on_battlefield(&state, "Offspring Warrior"),
        1,
        "ruling 2024-07-26: token should be created even if source left battlefield before trigger resolved"
    );

    // Verify the created object is a token.
    let token_ids = find_objects_on_battlefield(&state, "Offspring Warrior");
    assert!(
        token_ids
            .iter()
            .all(|id| state.objects.get(id).map(|o| o.is_token).unwrap_or(false)),
        "the remaining Offspring Warrior should be a token"
    );
}

/// CR 707.9b -- Offspring token P/T override is Layer 7b rather than copiable values.
///
/// KNOWN DEVIATION: Per CR 707.9b, the "except it's 1/1" clause in CR 702.175a is a
/// copy-with-exception whose result (P/T = 1/1) should become part of the token's
/// *copiable values*. The current implementation uses a Layer 7b SetPowerToughness
/// continuous effect instead, which is NOT visible to get_copiable_values() in copy.rs.
///
/// Consequence: if a subsequent copy effect (e.g., Clone) targets an Offspring token, the
/// Clone would inherit the *source creature's* original P/T instead of 1/1, because
/// get_copiable_values() resolves CopyOf (Layer 1) but not the Layer 7b SetPowerToughness.
///
/// This test verifies the CURRENT (incorrect per 707.9b) behavior: the token is 1/1 on the
/// battlefield (correct for common play), but the gap with copiable values is documented
/// here for future fixers. Full fix requires a new LayerModification variant at Layer 1
/// (e.g., SetCopiablePT) -- see the TODO comment in resolution.rs at the Layer 7b site.
#[test]
fn test_offspring_token_pt_is_layer7b_known_deviation() {
    // CR 707.9b: "Some copy effects modify a characteristic as part of the copying process.
    // The final set of values for that characteristic becomes part of the copiable values
    // of the copy." The "except it's 1/1" in 702.175a is such a copy-with-exception, so
    // the 1/1 should be in the copiable values. Current engine uses Layer 7b instead.
    //
    // Build state with Offspring Warrior (2/3 base, Offspring {2}) using the shared helper.
    // Extra {2} is provided so the offspring cost can be paid.
    let (state, p1, p2, card_id) = setup_offspring_state(2);

    // Cast with offspring paid.
    let (state, _) = cast_offspring(state, p1, card_id, true);

    // Both players pass priority to resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // OffspringTrigger is now on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "OffspringTrigger should be on stack before token creation"
    );

    // Both players pass to resolve the trigger and create the token.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Locate the token.
    let tokens: Vec<ObjectId> = state
        .objects
        .iter()
        .filter(|(_, obj)| {
            obj.characteristics.name == "Offspring Warrior"
                && obj.zone == ZoneId::Battlefield
                && obj.is_token
        })
        .map(|(id, _)| *id)
        .collect();

    assert_eq!(
        tokens.len(),
        1,
        "should have exactly 1 Offspring token on battlefield"
    );

    let token_id = tokens[0];
    let chars = calculate_characteristics(&state, token_id)
        .expect("token should have layer-resolved characteristics");

    // The token IS 1/1 on the battlefield -- the Layer 7b override achieves the correct
    // visible result in normal play.
    assert_eq!(
        chars.power.unwrap_or(-1),
        1,
        "token power should be 1 on battlefield (Layer 7b SetPowerToughness applied)"
    );
    assert_eq!(
        chars.toughness.unwrap_or(-1),
        1,
        "token toughness should be 1 on battlefield (Layer 7b SetPowerToughness applied)"
    );

    // KNOWN DEVIATION (CR 707.9b): The token's *copiable* P/T is the source creature's P/T
    // (2/3) rather than 1/1, because the override lives at Layer 7b and is not part of
    // copiable values. A Clone targeting this token would see P/T = 2/3, not 1/1.
    //
    // Full fix: add LayerModification::SetCopiablePT applied at Layer 1 with a later
    // timestamp than the CopyOf effect, and update apply_layer_modification and
    // get_copiable_values in copy.rs. Tracked in resolution.rs TODO comment.
}
