//! PB-SFT: Effect::SacrificePermanents `filter: Option<TargetFilter>` tests.
//!
//! Verifies that the `filter` field on `Effect::SacrificePermanents` correctly
//! restricts which permanents a player may sacrifice at resolution time.
//!
//! CR Rules covered:
//! - CR 701.21a: "Sacrifice" — the player chooses permanents they control to
//!   sacrifice. If the player controls fewer than N matching permanents, they
//!   sacrifice all matching permanents they control. Sacrifice bypasses indestructible.
//! - CR 109.1: Permanents are objects on the battlefield. Filter conditions
//!   constrain which subset of those permanents are eligible.
//! - CR 613.1d: Use layer-resolved characteristics (not raw base values) when
//!   evaluating filter conditions on permanents.

use mtg_engine::cards::card_definition::{
    CardDefinition, EffectAmount, PlayerTarget, TargetFilter,
};
use mtg_engine::{
    process_command, AbilityDefinition, CardId, CardRegistry, CardType, Command, Effect,
    GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
    HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn in_graveyard(state: &mtg_engine::GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn in_zone(state: &mtg_engine::GameState, name: &str, zone: ZoneId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == zone)
}

fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<mtg_engine::GameEvent>) {
    let mut events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        events.extend(ev);
    }
    (current, events)
}

/// Build a simple sorcery CardDefinition whose spell effect is `Effect::SacrificePermanents`.
fn sacrifice_sorcery_def(effect: Effect) -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-sac-sorcery".to_string()),
        name: "Test Sacrifice Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Test sacrifice sorcery.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Cast the test sorcery from p1's hand (pays {1} generic mana).
fn cast_test_sorcery(
    mut state: mtg_engine::GameState,
    p1: PlayerId,
) -> (mtg_engine::GameState, Vec<mtg_engine::GameEvent>) {
    let spell_id = find_obj(&state, "Test Sacrifice Sorcery");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);
    process_command(
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
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR N/A (hash infrastructure) — PB-SFT/PB-CC-B/PB-CC-C: HASH_SCHEMA_VERSION is 11.
/// Bumped from 10 (PB-CC-C) to account for LayerModification gaining
/// `ModifyPowerDynamic` and `ModifyToughnessDynamic` variants (CR 613.1c).
#[test]
fn test_sft_hash_schema_version_is_11() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 11u8,
        "PB-CC-C: HASH_SCHEMA_VERSION must be 11 (bump from PB-CC-B's 10 for \
         LayerModification::ModifyPowerDynamic/ModifyToughnessDynamic)"
    );
}

/// CR 701.21a + CR 109.1 — PB-SFT test 1: each-player-sacrifices-creature filter.
///
/// Fleshbag Marauder pattern: each player sacrifices a creature.
/// P1 controls a creature and an enchantment; P2 controls a creature and a land.
/// With `has_card_type: Some(CardType::Creature)` filter:
/// - P1 must sacrifice their creature, NOT the enchantment.
/// - P2 must sacrifice their creature, NOT the land.
#[test]
fn each_player_sacrifices_creature_filter() {
    let p1 = p(1);
    let p2 = p(2);

    let effect = Effect::SacrificePermanents {
        player: PlayerTarget::EachPlayer,
        count: EffectAmount::Fixed(1),
        filter: Some(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }),
    };

    let sorcery = ObjectSpec::card(p1, "Test Sacrifice Sorcery")
        .with_card_id(CardId("test-sac-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    // P1: creature + enchantment
    let p1_creature = ObjectSpec::creature(p1, "P1 Creature", 2, 2).in_zone(ZoneId::Battlefield);
    let p1_enchant = ObjectSpec::card(p1, "P1 Enchantment")
        .with_types(vec![CardType::Enchantment])
        .in_zone(ZoneId::Battlefield);

    // P2: creature + land
    let p2_creature = ObjectSpec::creature(p2, "P2 Creature", 1, 1).in_zone(ZoneId::Battlefield);
    let p2_land = ObjectSpec::card(p2, "P2 Land")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![sacrifice_sorcery_def(effect)]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery)
        .object(p1_creature)
        .object(p1_enchant)
        .object(p2_creature)
        .object(p2_land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cast the sorcery; it goes on the stack.
    let (state, _) = cast_test_sorcery(state, p1);
    // Both players pass — sorcery resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.21a: creatures were sacrificed.
    assert!(
        in_graveyard(&state, "P1 Creature", p1),
        "P1's creature must be in their graveyard after creature-filter sacrifice"
    );
    assert!(
        in_graveyard(&state, "P2 Creature", p2),
        "P2's creature must be in their graveyard after creature-filter sacrifice"
    );
    // Non-creatures were NOT sacrificed.
    assert!(
        in_zone(&state, "P1 Enchantment", ZoneId::Battlefield),
        "P1's enchantment must remain on battlefield — filter excludes non-creatures"
    );
    assert!(
        in_zone(&state, "P2 Land", ZoneId::Battlefield),
        "P2's land must remain on battlefield — filter excludes non-creatures"
    );
}

/// CR 701.21a + CR 109.1 — PB-SFT test 2: land-filter sacrifice.
///
/// Roiling Regrowth pattern: the caster sacrifices a land at resolution.
/// P1 controls a land and a creature. With `has_card_type: Some(CardType::Land)` filter:
/// - P1 must sacrifice their land, NOT the creature.
#[test]
fn each_player_sacrifices_land_filter() {
    let p1 = p(1);
    let p2 = p(2);

    let effect = Effect::SacrificePermanents {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
        filter: Some(TargetFilter {
            has_card_type: Some(CardType::Land),
            ..Default::default()
        }),
    };

    let sorcery = ObjectSpec::card(p1, "Test Sacrifice Sorcery")
        .with_card_id(CardId("test-sac-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    // P1: land + creature
    let p1_land = ObjectSpec::card(p1, "Forest")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);
    let p1_creature = ObjectSpec::creature(p1, "P1 Creature", 3, 3).in_zone(ZoneId::Battlefield);

    // P2: just a creature (not affected — Controller target)
    let p2_creature = ObjectSpec::creature(p2, "P2 Creature", 1, 1).in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![sacrifice_sorcery_def(effect)]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery)
        .object(p1_land)
        .object(p1_creature)
        .object(p2_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let (state, _) = cast_test_sorcery(state, p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.21a: P1's land was sacrificed.
    assert!(
        in_graveyard(&state, "Forest", p1),
        "P1's Forest must be in graveyard after land-filter sacrifice"
    );
    // P1's creature was NOT sacrificed.
    assert!(
        in_zone(&state, "P1 Creature", ZoneId::Battlefield),
        "P1's creature must remain — filter restricts sacrifice to lands only"
    );
    // P2 was not affected (Controller target).
    assert!(
        in_zone(&state, "P2 Creature", ZoneId::Battlefield),
        "P2's creature is unaffected — Controller target only applies to P1"
    );
}

/// CR 701.21a + CR 109.1 — PB-SFT test 3: count-2 creature filter.
///
/// Liliana Dreadhorde General -4 pattern: each player sacrifices 2 creatures.
/// P1 controls 3 creatures and 1 artifact. With creature filter and count = 2:
/// - P1 sacrifices the 2 lowest-ObjectId creatures.
/// - P1's artifact is untouched.
#[test]
fn multi_count_sacrifice_with_filter() {
    let p1 = p(1);
    let p2 = p(2);

    let effect = Effect::SacrificePermanents {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(2),
        filter: Some(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }),
    };

    let sorcery = ObjectSpec::card(p1, "Test Sacrifice Sorcery")
        .with_card_id(CardId("test-sac-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    // P1: 3 creatures + 1 artifact
    let p1_ca = ObjectSpec::creature(p1, "Creature A", 1, 1).in_zone(ZoneId::Battlefield);
    let p1_cb = ObjectSpec::creature(p1, "Creature B", 1, 1).in_zone(ZoneId::Battlefield);
    let p1_cc = ObjectSpec::creature(p1, "Creature C", 1, 1).in_zone(ZoneId::Battlefield);
    let p1_art = ObjectSpec::card(p1, "P1 Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);

    // P2: observer
    let p2_creature = ObjectSpec::creature(p2, "P2 Creature", 1, 1).in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![sacrifice_sorcery_def(effect)]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery)
        .object(p1_ca)
        .object(p1_cb)
        .object(p1_cc)
        .object(p1_art)
        .object(p2_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let (state, _) = cast_test_sorcery(state, p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Exactly 2 of P1's 3 creatures are sacrificed; artifact is untouched.
    let p1_creatures_on_bf: usize = state
        .objects
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.controller == p1
                && o.characteristics.card_types.contains(&CardType::Creature)
        })
        .count();
    assert_eq!(
        p1_creatures_on_bf, 1,
        "CR 701.21a: P1 should have exactly 1 creature remaining after count-2 sacrifice"
    );

    // Artifact was not sacrificed.
    assert!(
        in_zone(&state, "P1 Artifact", ZoneId::Battlefield),
        "P1's artifact must remain — filter restricts to creatures only"
    );
    // P2 unaffected.
    assert!(
        in_zone(&state, "P2 Creature", ZoneId::Battlefield),
        "P2's creature is unaffected — Controller target"
    );
}

/// CR 701.21a — PB-SFT test 4: filter excludes all permanents (zero match).
///
/// When a player controls no permanents matching the filter, no sacrifice occurs and
/// no error is raised. CR 701.21a: "If the player controls fewer ... they sacrifice
/// all [matching] permanents they control." Zero matching permanents → zero sacrificed.
#[test]
fn filter_excludes_all_player_has_nothing_to_sacrifice() {
    let p1 = p(1);
    let p2 = p(2);

    // Filter: creature — but P1 controls only lands.
    let effect = Effect::SacrificePermanents {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
        filter: Some(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }),
    };

    let sorcery = ObjectSpec::card(p1, "Test Sacrifice Sorcery")
        .with_card_id(CardId("test-sac-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    // P1: only lands (no creatures)
    let p1_land1 = ObjectSpec::card(p1, "Forest 1")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);
    let p1_land2 = ObjectSpec::card(p1, "Mountain")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![sacrifice_sorcery_def(effect)]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery)
        .object(p1_land1)
        .object(p1_land2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Must not panic or return an error.
    let (state, _) = cast_test_sorcery(state, p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both lands must remain — zero-match means zero sacrifice.
    assert!(
        in_zone(&state, "Forest 1", ZoneId::Battlefield),
        "Forest 1 must remain — no creatures to sacrifice"
    );
    assert!(
        in_zone(&state, "Mountain", ZoneId::Battlefield),
        "Mountain must remain — no creatures to sacrifice"
    );
}

/// CR 701.21a + CR 109.1 — PB-SFT test 5: multi-type OR filter (creature or planeswalker).
///
/// Vraska's Fall / Demon's Disciple pattern: player sacrifices a creature or planeswalker.
/// P2 controls an artifact + a creature + a planeswalker. With
/// `has_card_types: [Creature, Planeswalker]` (OR semantics, EachOpponent target):
/// - Deterministic fallback picks the lowest-ObjectId permanent that is EITHER a
///   creature or a planeswalker.
/// - The artifact must NOT be sacrificed.
/// - Exactly one of creature/planeswalker is sacrificed (count=1).
#[test]
fn multi_type_filter_creature_or_planeswalker() {
    let p1 = p(1);
    let p2 = p(2);

    // Use EachOpponent directly (avoids ForEach/DeclaredTarget complexity).
    let effect = Effect::SacrificePermanents {
        player: PlayerTarget::EachOpponent,
        count: EffectAmount::Fixed(1),
        filter: Some(TargetFilter {
            has_card_types: vec![CardType::Creature, CardType::Planeswalker],
            ..Default::default()
        }),
    };

    let sorcery = ObjectSpec::card(p1, "Test Sacrifice Sorcery")
        .with_card_id(CardId("test-sac-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    // P2: artifact + creature + planeswalker.
    // NOTE: use ObjectSpec::planeswalker(..., 3) not ObjectSpec::card.with_types([Planeswalker])
    // — a planeswalker with loyalty=None resolves to 0 and dies immediately to SBA 704.5i.
    let p2_artifact = ObjectSpec::card(p2, "P2 Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);
    let p2_creature = ObjectSpec::creature(p2, "P2 Creature", 2, 2).in_zone(ZoneId::Battlefield);
    let p2_pw = ObjectSpec::planeswalker(p2, "P2 Planeswalker", 3);

    let registry = CardRegistry::new(vec![sacrifice_sorcery_def(effect)]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery)
        .object(p2_artifact)
        .object(p2_creature)
        .object(p2_pw)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let (state, _) = cast_test_sorcery(state, p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Artifact must NOT have been sacrificed.
    assert!(
        in_zone(&state, "P2 Artifact", ZoneId::Battlefield),
        "P2's artifact must remain — filter excludes non-creature/non-planeswalker permanents"
    );

    // Exactly one of creature or planeswalker was sacrificed (deterministic: lowest ObjectId).
    let creature_gone = in_graveyard(&state, "P2 Creature", p2);
    let pw_gone = in_graveyard(&state, "P2 Planeswalker", p2);
    assert!(
        creature_gone || pw_gone,
        "Either P2's creature or planeswalker must be sacrificed (OR-type filter)"
    );
    // Exactly one sacrificed — not both.
    assert!(
        !(creature_gone && pw_gone),
        "Only one of creature/planeswalker should be sacrificed (count=1)"
    );
}
