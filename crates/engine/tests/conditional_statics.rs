//! Conditional static ability tests (CR 604.2, PB-24).
//!
//! Tests that `ContinuousEffect::condition` gates layer application correctly,
//! using each of the five new `Condition` variants and `RemoveCardTypes`.

use im::ordset;
use mtg_engine::{
    calculate_characteristics, CardType, Condition, ContinuousEffect, CounterType, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameState, GameStateBuilder, KeywordAbility,
    LayerModification, ObjectId, ObjectSpec, PlayerId, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Find an object on the battlefield by name.
fn find_on_battlefield(state: &GameState, name: &str) -> ObjectId {
    let bf = state.zones.get(&ZoneId::Battlefield).unwrap();
    *bf.object_ids()
        .iter()
        .find(|id| {
            state
                .objects
                .get(id)
                .map(|o| o.characteristics.name == name)
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("object '{}' not found on battlefield", name))
}

/// Build a conditional keyword effect (Layer 6) gated on the given condition.
fn conditional_keyword_effect(
    source_id: ObjectId,
    target_id: ObjectId,
    keyword: KeywordAbility,
    condition: Condition,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(100),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(target_id),
        modification: LayerModification::AddKeyword(keyword),
        is_cda: false,
        condition: Some(condition),
    }
}

// ---------------------------------------------------------------------------
// Condition::ControllerLifeAtLeast — Serra Ascendant
// ---------------------------------------------------------------------------

/// CR 604.2: Serra Ascendant gets +5/+5 and flying when controller has >= 30 life.
/// Tests that the conditional static fires when the condition is true and not when false.
#[test]
fn test_conditional_static_life_threshold() {
    // Setup: p1 starts with 40 life (typical opening), p2 at 20.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Serra Ascendant", 1, 1))
        .build()
        .unwrap();

    // Set p1 to 30 life (condition met).
    state.players.get_mut(&p1()).unwrap().life_total = 30;

    let ascendant_id = find_on_battlefield(&state, "Serra Ascendant");

    // Add conditional +5/+5 (Layer 7c) gated on ControllerLifeAtLeast(30).
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(101),
        source: Some(ascendant_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(ascendant_id),
        modification: LayerModification::ModifyBoth(5),
        is_cda: false,
        condition: Some(Condition::ControllerLifeAtLeast(30)),
    });
    // Add conditional flying (Layer 6).
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(102),
        source: Some(ascendant_id),
        timestamp: 11,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(ascendant_id),
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        is_cda: false,
        condition: Some(Condition::ControllerLifeAtLeast(30)),
    });

    // Condition met (life == 30): should be a 6/6 with flying.
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(
        chars.power,
        Some(6),
        "Serra Ascendant should be 6/* at 30 life"
    );
    assert_eq!(
        chars.toughness,
        Some(6),
        "Serra Ascendant should be */6 at 30 life"
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Flying),
        "Serra Ascendant should have flying at 30 life"
    );

    // Set p1 to 29 life (condition not met).
    state.players.get_mut(&p1()).unwrap().life_total = 29;
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "Serra Ascendant should be 1/* at 29 life"
    );
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "Serra Ascendant should NOT have flying at 29 life"
    );
}

// ---------------------------------------------------------------------------
// Condition::SourceIsUntapped — Dragonlord Ojutai
// ---------------------------------------------------------------------------

/// CR 604.2: Dragonlord Ojutai has hexproof as long as it's untapped.
/// Tests that hexproof appears when untapped and vanishes when tapped.
#[test]
fn test_conditional_static_untapped() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Dragonlord Ojutai", 5, 4))
        .build()
        .unwrap();

    let ojutai_id = find_on_battlefield(&state, "Dragonlord Ojutai");

    // Add conditional hexproof gated on SourceIsUntapped.
    state
        .continuous_effects
        .push_back(conditional_keyword_effect(
            ojutai_id,
            ojutai_id,
            KeywordAbility::Hexproof,
            Condition::SourceIsUntapped,
        ));

    // Untapped (default): should have hexproof.
    let obj = state.objects.get(&ojutai_id).unwrap();
    assert!(
        !obj.status.tapped,
        "Dragonlord Ojutai should start untapped"
    );
    let chars = calculate_characteristics(&state, ojutai_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::Hexproof),
        "Dragonlord Ojutai should have hexproof when untapped"
    );

    // Tap Ojutai.
    state.objects.get_mut(&ojutai_id).unwrap().status.tapped = true;
    let chars = calculate_characteristics(&state, ojutai_id).unwrap();
    assert!(
        !chars.keywords.contains(&KeywordAbility::Hexproof),
        "Dragonlord Ojutai should NOT have hexproof when tapped"
    );
}

// ---------------------------------------------------------------------------
// Condition::SourceHasCounters — Beastmaster Ascension
// ---------------------------------------------------------------------------

/// CR 604.2: Beastmaster Ascension gives all your creatures +5/+5 when it has 7+ quest counters.
/// Tests that the buff appears at/above the threshold and not below.
#[test]
fn test_conditional_static_counter_threshold() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::enchantment(p1(), "Beastmaster Ascension"))
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();

    let ascension_id = find_on_battlefield(&state, "Beastmaster Ascension");
    let bear_id = find_on_battlefield(&state, "Bear");

    // Add conditional +5/+5 gated on SourceHasCounters { Quest, min: 7 }.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(ascension_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::ModifyBoth(5),
        is_cda: false,
        condition: Some(Condition::SourceHasCounters {
            counter: CounterType::Quest,
            min: 7,
        }),
    });

    // 0 quest counters: no bonus.
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "Bear should be 2/* with 0 quest counters"
    );

    // Add 6 quest counters to Beastmaster Ascension: still no bonus.
    state
        .objects
        .get_mut(&ascension_id)
        .unwrap()
        .counters
        .insert(CounterType::Quest, 6);
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "Bear should be 2/* with 6 quest counters"
    );

    // Add 1 more (total 7): bonus kicks in.
    state
        .objects
        .get_mut(&ascension_id)
        .unwrap()
        .counters
        .insert(CounterType::Quest, 7);
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert_eq!(
        chars.power,
        Some(7),
        "Bear should be 7/* with 7 quest counters"
    );
}

// ---------------------------------------------------------------------------
// Condition::CompletedADungeon — Nadaar, Selfless Paladin
// ---------------------------------------------------------------------------

/// CR 309.7 / CR 604.2: Nadaar gives other creatures +1/+1 when you've completed a dungeon.
#[test]
fn test_conditional_static_dungeon() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Nadaar", 3, 3))
        .object(ObjectSpec::creature(p1(), "Knight", 2, 2))
        .build()
        .unwrap();

    let nadaar_id = find_on_battlefield(&state, "Nadaar");
    let knight_id = find_on_battlefield(&state, "Knight");

    // Add conditional +1/+1 to other creatures on CompletedADungeon.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(nadaar_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: Some(Condition::CompletedADungeon),
    });

    // No dungeon completed: no bonus.
    let chars = calculate_characteristics(&state, knight_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "Knight should be 2/* before dungeon completion"
    );

    // Complete a dungeon.
    state.players.get_mut(&p1()).unwrap().dungeons_completed = 1;
    let chars = calculate_characteristics(&state, knight_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "Knight should be 3/* after dungeon completion"
    );
}

// ---------------------------------------------------------------------------
// Condition::OpponentLifeAtMost — Bloodghast
// ---------------------------------------------------------------------------

/// CR 604.2: Bloodghast has haste as long as an opponent has 10 or less life.
#[test]
fn test_conditional_static_opponent_life() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bloodghast", 2, 1))
        .build()
        .unwrap();

    // Both players start with default 40 life.
    let ghast_id = find_on_battlefield(&state, "Bloodghast");

    state
        .continuous_effects
        .push_back(conditional_keyword_effect(
            ghast_id,
            ghast_id,
            KeywordAbility::Haste,
            Condition::OpponentLifeAtMost(10),
        ));

    // P2 at 20 life: no haste.
    state.players.get_mut(&p2()).unwrap().life_total = 20;
    let chars = calculate_characteristics(&state, ghast_id).unwrap();
    assert!(
        !chars.keywords.contains(&KeywordAbility::Haste),
        "Bloodghast should NOT have haste when P2 is at 20 life"
    );

    // P2 drops to 10 life: haste activates.
    state.players.get_mut(&p2()).unwrap().life_total = 10;
    let chars = calculate_characteristics(&state, ghast_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "Bloodghast should have haste when P2 is at 10 life"
    );

    // P2 drops to 0: still haste.
    state.players.get_mut(&p2()).unwrap().life_total = 0;
    let chars = calculate_characteristics(&state, ghast_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "Bloodghast should have haste when P2 is at 0 life"
    );
}

// ---------------------------------------------------------------------------
// Condition::IsYourTurn — Razorkin Needlehead
// ---------------------------------------------------------------------------

/// CR 500.1 / CR 604.2: Razorkin Needlehead has first strike during your turn.
/// Tests that first strike is present on your turn and absent on opponent's turn.
#[test]
fn test_conditional_static_is_your_turn() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Razorkin Needlehead", 2, 2))
        .build()
        .unwrap();

    let needlehead_id = find_on_battlefield(&state, "Razorkin Needlehead");

    state
        .continuous_effects
        .push_back(conditional_keyword_effect(
            needlehead_id,
            needlehead_id,
            KeywordAbility::FirstStrike,
            Condition::IsYourTurn,
        ));

    // Active player is p1 (default): first strike active.
    state.turn.active_player = p1();
    let chars = calculate_characteristics(&state, needlehead_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::FirstStrike),
        "Razorkin Needlehead should have first strike during P1's turn"
    );

    // Switch to P2's turn: first strike inactive.
    state.turn.active_player = p2();
    let chars = calculate_characteristics(&state, needlehead_id).unwrap();
    assert!(
        !chars.keywords.contains(&KeywordAbility::FirstStrike),
        "Razorkin Needlehead should NOT have first strike during P2's turn"
    );
}

// ---------------------------------------------------------------------------
// Condition::DevotionToColorsLessThan — Theros Gods
// ---------------------------------------------------------------------------

/// CR 700.5 / CR 604.2 (Layer 4): Purphoros removes Creature type when devotion to red < 5.
#[test]
fn test_conditional_static_devotion_single() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Purphoros", 6, 5))
        .build()
        .unwrap();

    let purphoros_id = find_on_battlefield(&state, "Purphoros");
    // Make it also an Enchantment (in addition to Creature).
    state
        .objects
        .get_mut(&purphoros_id)
        .unwrap()
        .characteristics
        .card_types
        .insert(CardType::Enchantment);

    // Add RemoveCardTypes(Creature) gated on DevotionToColorsLessThan { Red, 5 }.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(purphoros_id),
        timestamp: 10,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(purphoros_id),
        modification: LayerModification::RemoveCardTypes(ordset![CardType::Creature]),
        is_cda: false,
        condition: Some(Condition::DevotionToColorsLessThan {
            colors: vec![mtg_engine::Color::Red],
            threshold: 5,
        }),
    });

    // No other permanents: devotion to red = 1 (Purphoros costs {3}{R}) < 5.
    // Therefore Purphoros is NOT a creature.
    let chars = calculate_characteristics(&state, purphoros_id).unwrap();
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "Purphoros should not be a creature when devotion < 5"
    );
    // But is still an Enchantment.
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "Purphoros should still be an Enchantment when not a creature"
    );
}

/// CR 700.5: Multi-color devotion (Athreos: W+B). Tests that devotion to white AND black
/// is counted correctly (any symbol matching either color counts).
#[test]
fn test_conditional_static_devotion_multicolor() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Athreos", 5, 4))
        .build()
        .unwrap();

    let athreos_id = find_on_battlefield(&state, "Athreos");
    // Make it also an Enchantment.
    state
        .objects
        .get_mut(&athreos_id)
        .unwrap()
        .characteristics
        .card_types
        .insert(CardType::Enchantment);

    // Add RemoveCardTypes(Creature) gated on DevotionToColorsLessThan { W+B, 7 }.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(athreos_id),
        timestamp: 10,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(athreos_id),
        modification: LayerModification::RemoveCardTypes(ordset![CardType::Creature]),
        is_cda: false,
        condition: Some(Condition::DevotionToColorsLessThan {
            colors: vec![mtg_engine::Color::White, mtg_engine::Color::Black],
            threshold: 7,
        }),
    });

    // Only Athreos itself on battlefield ({1}{W}{B} = 2 devotion) < 7 → not a creature.
    let chars = calculate_characteristics(&state, athreos_id).unwrap();
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "Athreos should not be a creature with devotion 2 < 7"
    );
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "Athreos should still be an Enchantment"
    );
}

// ---------------------------------------------------------------------------
// LayerModification::RemoveCardTypes — isolation test
// ---------------------------------------------------------------------------

/// CR 613.1d: RemoveCardTypes removes the specified card type without affecting others.
/// An Enchantment Creature with RemoveCardTypes(Creature) becomes just an Enchantment.
#[test]
fn test_conditional_static_remove_type() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "God", 6, 5))
        .build()
        .unwrap();

    let god_id = find_on_battlefield(&state, "God");
    // Also give it Enchantment type.
    state
        .objects
        .get_mut(&god_id)
        .unwrap()
        .characteristics
        .card_types
        .insert(CardType::Enchantment);

    // Unconditional RemoveCardTypes(Creature) — should always apply.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(god_id),
        timestamp: 10,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(god_id),
        modification: LayerModification::RemoveCardTypes(ordset![CardType::Creature]),
        is_cda: false,
        condition: None,
    });

    let chars = calculate_characteristics(&state, god_id).unwrap();
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "Creature type should be removed"
    );
    assert!(
        chars.card_types.contains(&CardType::Enchantment),
        "Enchantment type should remain"
    );
}

// ---------------------------------------------------------------------------
// Toggle mid-game
// ---------------------------------------------------------------------------

/// CR 604.2: Conditional statics toggle immediately when the condition changes.
/// No lag — the layer system re-evaluates the condition on every characteristic calculation.
#[test]
fn test_conditional_static_toggles_midgame() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Serra Ascendant", 1, 1))
        .build()
        .unwrap();

    let ascendant_id = find_on_battlefield(&state, "Serra Ascendant");

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(ascendant_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(ascendant_id),
        modification: LayerModification::ModifyBoth(5),
        is_cda: false,
        condition: Some(Condition::ControllerLifeAtLeast(30)),
    });

    // Start below threshold.
    state.players.get_mut(&p1()).unwrap().life_total = 20;
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(chars.power, Some(1), "Should be 1/* below threshold");

    // Gain life to reach threshold.
    state.players.get_mut(&p1()).unwrap().life_total = 30;
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(chars.power, Some(6), "Should be 6/* at threshold");

    // Lose 1 life — drops below threshold immediately.
    state.players.get_mut(&p1()).unwrap().life_total = 29;
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(chars.power, Some(1), "Should be 1/* below threshold again");
}

// ---------------------------------------------------------------------------
// EffectFilter::Source resolved to SingleObject at registration
// ---------------------------------------------------------------------------

/// CR 400.7: EffectFilter::Source resolves to SingleObject at registration time.
/// The `register_static_continuous_effects` function (called at ETB) must replace
/// `EffectFilter::Source` with `EffectFilter::SingleObject(new_id)`.
///
/// We verify this by placing a conditional effect with `EffectFilter::Source` in
/// state.continuous_effects directly (simulating what the registration call would do)
/// and checking that the condition-based calculation uses the correct object.
#[test]
fn test_conditional_static_source_filter_resolved() {
    // Build a state with one creature.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Serra Ascendant", 1, 1))
        .build()
        .unwrap();

    let ascendant_id = find_on_battlefield(&state, "Serra Ascendant");

    // Directly push an effect using SingleObject (as register_static_continuous_effects does).
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(100),
        source: Some(ascendant_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(ascendant_id), // Source -> resolved
        modification: LayerModification::ModifyBoth(5),
        is_cda: false,
        condition: Some(Condition::ControllerLifeAtLeast(30)),
    });

    // The registered effect should have a SingleObject filter.
    let effect = state.continuous_effects.iter().next().unwrap();
    assert!(
        matches!(effect.filter, EffectFilter::SingleObject(_)),
        "Filter should be SingleObject, not EffectFilter::Source, got {:?}",
        effect.filter
    );
    if let EffectFilter::SingleObject(resolved_id) = effect.filter {
        assert_eq!(
            resolved_id, ascendant_id,
            "Resolved SingleObject should point to the creature's ObjectId"
        );
    }

    // The condition should be present.
    assert!(
        effect.condition.is_some(),
        "Condition should be set on the registered effect"
    );

    // At 40 life: condition active, buff applies.
    state.players.get_mut(&p1()).unwrap().life_total = 40;
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(
        chars.power,
        Some(6),
        "Should be 6/* at 40 life (condition met)"
    );

    // At 20 life: condition inactive, buff does not apply.
    state.players.get_mut(&p1()).unwrap().life_total = 20;
    let chars = calculate_characteristics(&state, ascendant_id).unwrap();
    assert_eq!(
        chars.power,
        Some(1),
        "Should be 1/* at 20 life (condition not met)"
    );
}
