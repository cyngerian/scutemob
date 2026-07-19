//! PB-EF3b: granted keyword-triggers fire (Melee / Battle Cry / Annihilator).
//!
//! CR 702.86 (Annihilator), 702.91 (Battle Cry), 702.121 (Melee), 613.1f (Layer 6
//! ability grants), 603.2 (attack triggers).
//!
//! Before this batch, `builder.rs` synthesized the derived `TriggeredAbilityDef`
//! for these three trigger-bearing keywords only for PRINTED keywords. A keyword
//! GRANTED by a continuous effect (`LayerModification::AddKeyword`) landed in
//! `Characteristics.keywords` with no derived trigger, so it was a silent no-op
//! (EF-W-MISS-3). `layers::calculate_characteristics` now reconciles this after
//! all layers + merge integration, via the shared helper
//! `state::builder::derived_attack_trigger_for_keyword`. The Melee/Myriad/Provoke
//! kind-tags in `abilities.rs::AttackersDeclared` were also switched from a raw
//! base-characteristics read (wrong index for a granted-only trigger) to the
//! layer-resolved read.
//!
//! Keyword model is a SET (`OrdSet`): printed + granted collapse to one entry, so
//! CR 702.x.b ("each instance triggers separately") is not representable when the
//! two sources overlap on the SAME keyword — the no-double-fire test below pins
//! this as a known, deliberate limitation.

use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, enrich_spec_from_def, process_command,
    AttackTarget, CardDefinition, CardRegistry, Command, ContinuousEffect, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    LayerModification, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn battlefield_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == player)
        .count()
}

/// Pass priority for all listed players once.
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

/// A continuous effect granting `kw` to "other creatures you control" of
/// `source_id`'s controller (CR 604.2 shape — mirrors the Adriana/Stromkirk
/// Captain anthem, `EffectFilter::OtherCreaturesYouControl`).
fn grant_keyword_to_others(
    id: u64,
    source_id: ObjectId,
    timestamp: u64,
    kw: KeywordAbility,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: Some(source_id),
        timestamp,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::OtherCreaturesYouControl,
        modification: LayerModification::AddKeyword(kw),
        is_cda: false,
        condition: None,
    }
}

/// A continuous effect granting `kw` to exactly ONE object (`EffectFilter::SingleObject`),
/// used where the test wants to grant a keyword to a specific creature without also
/// granting it to every other creature the controller happens to have on the
/// battlefield (which `OtherCreaturesYouControl` would do).
fn grant_keyword_to_single_object(
    id: u64,
    target_id: ObjectId,
    timestamp: u64,
    kw: KeywordAbility,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(target_id),
        modification: LayerModification::AddKeyword(kw),
        is_cda: false,
        condition: None,
    }
}

// ── Test 1: Granted Melee via anthem fires and pumps ─────────────────────────

#[test]
/// CR 702.121a — A vanilla 2/2 with NO printed Melee is granted Melee by an
/// anthem source (another permanent, `EffectFilter::OtherCreaturesYouControl`).
/// Attacking 1 opponent fires the granted Melee trigger and pumps +1/+1 (3/3).
///
/// Non-vacuity: without Change 3 (layer reconciliation) + Change 4 (Melee tag
/// fix), the granted creature has NO derived trigger, so no AbilityTriggered
/// event fires and the creature stays 2/2 (verified by temporary revert).
fn test_ef3b_granted_melee_via_anthem_fires_and_pumps() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::enchantment(p1, "Anthem Source"))
        .object(ObjectSpec::creature(p1, "Vanilla Attacker", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Anthem Source");
    let attacker_id = find_object(&state, "Vanilla Attacker");

    state
        .continuous_effects_mut()
        .push_back(grant_keyword_to_others(
            100,
            source_id,
            10,
            KeywordAbility::Melee,
        ));

    // Sanity: the anthem actually grants the keyword before we attack.
    let pre_chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert!(
        pre_chars.keywords.contains(&KeywordAbility::Melee),
        "anthem should grant Melee to the vanilla attacker before it attacks"
    );

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    assert!(
        declare_events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )),
        "PB-EF3b: granted Melee should fire an AbilityTriggered event"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "PB-EF3b: granted Melee trigger should be on the stack"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "PB-EF3b: granted Melee should pump +1 power for 1 opponent attacked (2+1=3)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "PB-EF3b: granted Melee should pump +1 toughness (2+1=3)"
    );
}

// ── Test 2: Granted Melee, multiplayer per-opponent count ────────────────────

#[test]
/// CR 702.121a — 4-player Commander. The granted-Melee creature attacks P2,
/// while two vanilla helpers attack P3 and P4. Three distinct opponents were
/// attacked with a creature this combat, so the granted-Melee creature gets
/// +3/+3 (2/2 -> 5/5), matching ruling 2016-08-23.
fn test_ef3b_granted_melee_multiplayer_per_opponent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Vanilla Attacker", 2, 2))
        .object(ObjectSpec::creature(p1, "Helper A", 2, 2))
        .object(ObjectSpec::creature(p1, "Helper B", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Vanilla Attacker");
    let helper_a = find_object(&state, "Helper A");
    let helper_b = find_object(&state, "Helper B");

    // Grant Melee to the Vanilla Attacker specifically (SingleObject) so the two
    // helpers are NOT granted Melee — isolates the "3 distinct opponents" count
    // to the one granted-Melee creature under test.
    state
        .continuous_effects_mut()
        .push_back(grant_keyword_to_single_object(
            100,
            attacker_id,
            10,
            KeywordAbility::Melee,
        ));

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker_id, AttackTarget::Player(p2)),
                (helper_a, AttackTarget::Player(p3)),
                (helper_b, AttackTarget::Player(p4)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(5),
        "PB-EF3b: granted Melee attacked 3 distinct opponents -> +3 power (2+3=5)"
    );
    assert_eq!(
        chars.toughness,
        Some(5),
        "PB-EF3b: granted Melee attacked 3 distinct opponents -> +3 toughness (2+3=5)"
    );
}

// ── Test 3: Granted Battle Cry via anthem ────────────────────────────────────

#[test]
/// CR 702.91a — A creature is granted Battle Cry by an anthem (no printed Battle
/// Cry). It attacks alongside another attacker. The OTHER attacker gets +1/+0;
/// the granted-Battle-Cry creature itself does not (ForEachOtherAttackingCreature
/// excludes it). Resolves purely via the embedded `ForEach` effect (Change 3
/// only; no kind-tagging needed for Battle Cry).
fn test_ef3b_granted_battle_cry_via_anthem() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Granted BC Attacker", 2, 2))
        .object(ObjectSpec::creature(p1, "Other Attacker", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let bc_id = find_object(&state, "Granted BC Attacker");
    let other_id = find_object(&state, "Other Attacker");

    // Grant Battle Cry to the "Granted BC Attacker" specifically (SingleObject)
    // so "Other Attacker" is NOT itself granted Battle Cry — isolates the
    // ForEachOtherAttackingCreature effect to a single granted source.
    state
        .continuous_effects_mut()
        .push_back(grant_keyword_to_single_object(
            100,
            bc_id,
            10,
            KeywordAbility::BattleCry,
        ));

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bc_id, AttackTarget::Player(p2)),
                (other_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    assert!(
        declare_events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == bc_id
        )),
        "PB-EF3b: granted Battle Cry should fire an AbilityTriggered event"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "PB-EF3b: exactly one granted Battle Cry trigger on the stack"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    let other_chars = calculate_characteristics(&state, other_id).unwrap();
    assert_eq!(
        other_chars.power,
        Some(3),
        "PB-EF3b: granted Battle Cry pumps the OTHER attacker +1/+0 (2+1=3)"
    );
    assert_eq!(
        other_chars.toughness,
        Some(2),
        "CR 702.91a: Battle Cry is +1/+0 (toughness unchanged)"
    );

    let bc_chars = calculate_characteristics(&state, bc_id).unwrap();
    assert_eq!(
        bc_chars.power,
        Some(2),
        "CR 702.91a: Battle Cry does not pump itself"
    );
}

// ── Test 4: Granted Annihilator via anthem ───────────────────────────────────

#[test]
/// CR 702.86a — A creature is granted Annihilator 1 by an anthem (no printed
/// Annihilator). Attacking P2 (who controls 2 permanents) fires the trigger and
/// P2 sacrifices exactly 1 permanent. Verifies `defending_player_id` is stamped
/// correctly for a granted trigger and the embedded `SacrificePermanents`
/// resolves.
fn test_ef3b_granted_annihilator_via_anthem() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::enchantment(p1, "Anthem Source"))
        .object(ObjectSpec::creature(
            p1,
            "Granted Annihilator Attacker",
            2,
            2,
        ))
        .object(ObjectSpec::creature(p2, "Defender A", 1, 1))
        .object(ObjectSpec::creature(p2, "Defender B", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Anthem Source");
    let attacker_id = find_object(&state, "Granted Annihilator Attacker");

    state
        .continuous_effects_mut()
        .push_back(grant_keyword_to_others(
            100,
            source_id,
            10,
            KeywordAbility::Annihilator(1),
        ));

    assert_eq!(
        battlefield_count(&state, p2),
        2,
        "P2 should start with 2 permanents"
    );

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    assert!(
        declare_events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )),
        "PB-EF3b: granted Annihilator should fire an AbilityTriggered event"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        battlefield_count(&state, p2),
        1,
        "PB-EF3b: granted Annihilator 1 should force P2 to sacrifice exactly 1 permanent"
    );
}

// ── Test 5: No double-fire — printed + granted Melee (same keyword) ─────────

#[test]
/// CR 702.121b / set-model limitation — A creature has PRINTED Melee AND is also
/// granted Melee by an anthem (it is "other" relative to the anthem source).
/// `Characteristics.keywords` is a SET (`OrdSet`), so the two collapse to a
/// single entry; the layer reconciliation dedups by description equality
/// against the builder-synthesized def, so only ONE derived trigger exists.
/// Attacking 1 opponent should pump exactly +1/+1 (3/3), NOT +2/+2 (4/4).
///
/// Non-vacuity: removing the `already` dedup check in the reconciliation loop
/// makes this fail (creature ends 4/4) — verified by temporary revert.
fn test_ef3b_no_double_fire_printed_plus_granted_melee() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::enchantment(p1, "Anthem Source"))
        .object(
            ObjectSpec::creature(p1, "Double Melee Creature", 2, 2)
                .with_keyword(KeywordAbility::Melee),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Anthem Source");
    let attacker_id = find_object(&state, "Double Melee Creature");

    // Granting Melee AGAIN via the anthem: same keyword, different source.
    state
        .continuous_effects_mut()
        .push_back(grant_keyword_to_others(
            100,
            source_id,
            10,
            KeywordAbility::Melee,
        ));

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Exactly ONE trigger, not two.
    let triggered_count = declare_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        })
        .count();
    assert_eq!(
        triggered_count, 1,
        "PB-EF3b: printed + granted Melee on the same keyword must fire exactly once (set model)"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "PB-EF3b: exactly one Melee trigger on the stack, not two"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "PB-EF3b: printed+granted Melee fires exactly ONCE -> +1 power (2+1=3, NOT 4)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "PB-EF3b: printed+granted Melee fires exactly ONCE -> +1 toughness (2+1=3, NOT 4)"
    );
}

// ── Test 6: Printed-only Melee unchanged (regression) ────────────────────────

#[test]
/// CR 702.121a — A builder-built PRINTED-Melee creature with no anthem still
/// fires exactly once. Guards against the layer reconciliation double-appending
/// onto a builder-synthesized def (dedup must skip when the def is already
/// present in base characteristics).
fn test_ef3b_printed_melee_unchanged_regression() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Printed Melee Only", 2, 2)
                .with_keyword(KeywordAbility::Melee),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Printed Melee Only");

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let triggered_count = declare_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        })
        .count();
    assert_eq!(
        triggered_count, 1,
        "PB-EF3b regression: printed-only Melee must still fire exactly once"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "PB-EF3b regression: printed-only Melee -> exactly one trigger on the stack"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "PB-EF3b regression: printed-only Melee -> +1 power (2+1=3)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "PB-EF3b regression: printed-only Melee -> +1 toughness (2+1=3)"
    );
}

// ── Test 7: Card integration — Adriana, Captain of the Guard ────────────────

fn build_defs_registry() -> (Vec<CardDefinition>, std::sync::Arc<CardRegistry>) {
    let cards = all_cards();
    let registry = CardRegistry::new(cards.clone());
    (cards, registry)
}

#[test]
/// Card integration, CR 702.121a — Adriana, Captain of the Guard (printed Melee,
/// and "Other creatures you control have melee.") is placed on the battlefield
/// alongside a vanilla 2/2. Both attack 1 opponent: the vanilla creature's
/// GRANTED Melee fires (2/2 -> 3/3) and Adriana's own PRINTED Melee fires
/// (4/4 -> 5/5). Adriana's anthem uses `EffectFilter::OtherCreaturesYouControl`,
/// which excludes Adriana herself from a second (granted) Melee — pinned by
/// asserting she gets exactly +1/+1, not +2/+2.
fn test_ef3b_adriana_grants_melee_to_other_creatures() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let (cards, registry) = build_defs_registry();
    let _ = &cards;

    let adriana_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Adriana, Captain of the Guard")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Adriana, Captain of the Guard")),
        &cards.iter().map(|d| (d.name.clone(), d.clone())).collect(),
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(adriana_spec)
        .object(ObjectSpec::creature(p1, "Vanilla Squire", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let adriana_id = find_object(&state, "Adriana, Captain of the Guard");
    let squire_id = find_object(&state, "Vanilla Squire");

    // Register Adriana's static anthem (builder-placed objects skip ETB, so the
    // continuous effect must be registered explicitly — mirrors what
    // resolution.rs does when Adriana actually resolves from the stack).
    let adriana_card_id = card_name_to_id("Adriana, Captain of the Guard");
    register_static_continuous_effects(
        &mut state,
        adriana_id,
        Some(&adriana_card_id),
        &registry,
        false,
    );

    // Sanity: the anthem grants Melee to the vanilla squire, NOT to Adriana herself.
    let squire_pre = calculate_characteristics(&state, squire_id).unwrap();
    assert!(
        squire_pre.keywords.contains(&KeywordAbility::Melee),
        "Adriana's anthem should grant Melee to other creatures you control"
    );

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (adriana_id, AttackTarget::Player(p2)),
                (squire_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two distinct Melee triggers: Adriana's printed one and the squire's granted one.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let squire_chars = calculate_characteristics(&state, squire_id).unwrap();
    assert_eq!(
        squire_chars.power,
        Some(3),
        "PB-EF3b: vanilla squire's GRANTED Melee fires -> +1/+1 (2+1=3)"
    );
    assert_eq!(squire_chars.toughness, Some(3));

    let adriana_chars = calculate_characteristics(&state, adriana_id).unwrap();
    assert_eq!(
        adriana_chars.power,
        Some(5),
        "PB-EF3b: Adriana's own PRINTED Melee fires exactly once -> +1/+1 (4+1=5), \
         'other creatures you control' excludes her from a second Melee"
    );
    assert_eq!(adriana_chars.toughness, Some(5));
}

// ── Test 8: Humility / RemoveAllAbilities strips granted Melee ───────────────

#[test]
/// CR 613.1f / Layer 6 removal — A creature is granted Melee by an anthem, but a
/// LATER-timestamped `RemoveAllAbilities` effect (Humility-style) clears
/// `chars.keywords` before the reconciliation loop runs. No derived trigger is
/// appended (the loop iterates the FINAL resolved keyword set, which is empty),
/// so attacking does not fire Melee and the creature's P/T is unaffected by it.
fn test_ef3b_humility_strips_granted_melee() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::enchantment(p1, "Anthem Source"))
        .object(ObjectSpec::creature(p1, "Humbled Attacker", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Anthem Source");
    let attacker_id = find_object(&state, "Humbled Attacker");

    // Grant Melee at timestamp 10.
    state
        .continuous_effects_mut()
        .push_back(grant_keyword_to_others(
            100,
            source_id,
            10,
            KeywordAbility::Melee,
        ));
    // RemoveAllAbilities at a LATER timestamp (20) — Layer 6 processes in
    // timestamp order, so this wins and clears the granted keyword.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(101),
        source: None,
        timestamp: 20,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::RemoveAllAbilities,
        is_cda: false,
        condition: None,
    });

    // Sanity: Melee is NOT in the resolved keyword set (Humility stripped it).
    let pre_chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert!(
        !pre_chars.keywords.contains(&KeywordAbility::Melee),
        "PB-EF3b: RemoveAllAbilities should strip the granted Melee keyword"
    );
    assert!(
        pre_chars.triggered_abilities.is_empty(),
        "PB-EF3b: no derived Melee trigger should be reconciled when the keyword is absent"
    );

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    assert!(
        !declare_events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )),
        "PB-EF3b: no Melee trigger should fire when RemoveAllAbilities stripped the keyword"
    );
    assert!(
        state.stack_objects().is_empty(),
        "PB-EF3b: no trigger on the stack under RemoveAllAbilities"
    );

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "PB-EF3b: no Melee bonus applied — power stays 2 (base)"
    );
}
