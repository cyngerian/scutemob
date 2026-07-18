//! PB-EF4: `EffectFilter::TriggeringCreature` (continuous-effect subject) and
//! `Effect::DealDamage.source: Option<EffectTarget>` (damage-source override).
//!
//! Two capability additions, both building on PB-EF3's `EffectContext.triggering_creature_id`
//! threading (StackObject -> EffectContext):
//!
//! 1. **`EffectFilter::TriggeringCreature`** (CR 611.2a / 613.1f) — a continuous-effect
//!    filter resolved at `Effect::ApplyContinuousEffect` execution time to
//!    `SingleObject(ctx.triggering_creature_id)`, mirroring `EffectFilter::Source`. Lets
//!    "when a creature enters/attacks, IT gains <keyword> / gets +N/+N until end of turn"
//!    be expressed (Dragon Tempest's flying half, Ogre Battledriver, Atarka World Render,
//!    Fervent Charge, Dreadhorde Invasion's attack half).
//! 2. **`Effect::DealDamage.source: Option<EffectTarget>`** (CR 119.3 / 702.15a) — `None` =
//!    unchanged (`ctx.source`); `Some(t)` resolves `t` to a single ObjectId and uses THAT
//!    as the damage source for doubling, prevention, infect/lifelink/deathtouch keyword
//!    reads, lifelink-gain controller, and the `source:` field of `DamageDealt`/
//!    `PoisonCountersGiven`. `source: Some(EffectTarget::TriggeringCreature)` = "the
//!    entering/attacking creature deals it" (Dragon Tempest's Dragon half, Scourge of
//!    Valkas, Warstorm Surge).
//!
//! Test pattern for ETB triggers: builder-placed objects skip the normal ETB pipeline (no
//! `PermanentEnteredBattlefield` event is emitted just because an object was placed by
//! `GameStateBuilder`), so ETB scenarios synthesize the event directly and drive it through
//! the same `check_triggers` + `flush_pending_triggers` pair `check_and_flush_triggers`
//! wraps internally (mirrors `pb_l_landfall.rs`). Attack triggers use the real
//! `Command::DeclareAttackers` path (mirrors `pb_ef3b_granted_keyword_triggers.rs`), which
//! already dispatches `AnyCreatureYouControlAttacks` end to end.

use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
use mtg_engine::state::test_util;
use mtg_engine::{
    all_cards, calculate_characteristics, process_command, AttackTarget, CardContinuousEffectDef,
    CardDefinition, CardEffectTarget, CardRegistry, Command, ETBTriggerFilter, Effect,
    EffectAmount, EffectDuration, EffectFilter, EffectLayer, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId, Step,
    SubType, TriggerEvent, TriggeredAbilityDef, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn life(state: &GameState, player: PlayerId) -> i32 {
    state.players().get(&player).unwrap().life_total
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

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

/// Drain the stack fully (repeated `pass_all` rounds) — needed when a single ETB
/// synthesizes multiple stack objects (e.g. two triggers from the same event).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    while !state.stack_objects().is_empty() {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Synthesize a `PermanentEnteredBattlefield` event for an already-builder-placed
/// object and drive it through `check_triggers` + `flush_pending_triggers` (the same
/// pair `check_and_flush_triggers` wraps internally in the real command handlers).
/// Builder-placed objects never go through `resolution.rs`'s ETB pipeline, so nothing
/// fires for them unless this is called explicitly.
fn enter_battlefield(state: &mut GameState, entering_id: ObjectId, controller: PlayerId) {
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: entering_id,
        player: controller,
    }];
    let triggers = check_triggers(state, &events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let _ = flush_pending_triggers(state);
}

// ── Decoy 1: EffectFilter::TriggeringCreature selects exactly the entering creature ──

/// CR 611.2a — An anthem-style source ("whenever another creature you control enters,
/// that creature gets +2/+0 and gains haste until end of turn", the Ogre Battledriver
/// shape) targets the grant at `EffectFilter::TriggeringCreature`. A same-type decoy
/// creature already on the battlefield before the trigger must NOT be affected — only
/// the entering creature is.
///
/// Non-vacuity (verified by temporary revert): swapping the filter to
/// `EffectFilter::CreaturesYouControl` makes the decoy ALSO get pumped/hasted, reddening
/// this test. Confirms the filter is doing real, selective work, not "all my creatures".
#[test]
fn test_ef4_triggering_creature_filter_selects_exactly_the_trigger_source() {
    let p1 = p(1);
    let p2 = p(2);

    let anthem =
        ObjectSpec::enchantment(p1, "Anthem Source").with_triggered_ability(TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
            intervening_if: None,
            description: "test: entering creature gets +2/+0 and gains haste".to_string(),
            etb_filter: Some(ETBTriggerFilter {
                creature_only: true,
                controller_you: true,
                exclude_self: true,
                color_filter: None,
                card_type_filter: None,
            }),
            effect: Some(Effect::Sequence(vec![
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(CardContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(2),
                        filter: EffectFilter::TriggeringCreature,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(CardContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                        filter: EffectFilter::TriggeringCreature,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
            ])),
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(anthem)
        .object(ObjectSpec::creature(p1, "Decoy Creature", 2, 2))
        .object(ObjectSpec::creature(p1, "Entering Creature", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let decoy_id = find_object(&state, "Decoy Creature");
    let entering_id = find_object(&state, "Entering Creature");

    let mut state = state;
    enter_battlefield(&mut state, entering_id, p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let entering_chars = calculate_characteristics(&state, entering_id).unwrap();
    assert_eq!(
        entering_chars.power,
        Some(4),
        "PB-EF4: the entering creature should be pumped +2 power (2+2=4)"
    );
    assert!(
        entering_chars.keywords.contains(&KeywordAbility::Haste),
        "PB-EF4: the entering creature should gain haste"
    );

    let decoy_chars = calculate_characteristics(&state, decoy_id).unwrap();
    assert_eq!(
        decoy_chars.power,
        Some(2),
        "PB-EF4: EffectFilter::TriggeringCreature must NOT affect a decoy already on the \
         battlefield -- only the entering creature"
    );
    assert!(
        !decoy_chars.keywords.contains(&KeywordAbility::Haste),
        "PB-EF4: the decoy must not gain haste"
    );
}

// ── Decoy 2: DealDamage source override attributes lifelink to the SOURCE's controller ──

/// Shared setup for the source-override decoy pair: P2 controls a "Reactive
/// Enchantment" that fires on ANY creature entering (not just P2's own) and deals 3
/// damage to each of P2's opponents. P1 controls the entering creature, which has
/// Lifelink. `dmg_source` selects `Effect::DealDamage.source`.
fn build_source_override_state(dmg_source: Option<CardEffectTarget>) -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let reactor = ObjectSpec::enchantment(p2, "Reactive Enchantment").with_triggered_ability(
        TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
            intervening_if: None,
            description: "test: whenever a creature enters, it deals 3 damage".to_string(),
            etb_filter: Some(ETBTriggerFilter {
                creature_only: true,
                // Fires for ANY creature entering, not only the ability controller's own
                // -- required so P1's lifelinker (not P2's own creature) fires this.
                controller_you: false,
                exclude_self: false,
                color_filter: None,
                card_type_filter: None,
            }),
            effect: Some(Effect::DealDamage {
                target: CardEffectTarget::EachOpponent,
                amount: EffectAmount::Fixed(3),
                source: dmg_source,
            }),
        },
    );

    let entering =
        ObjectSpec::creature(p1, "P1 Lifelinker", 2, 2).with_keyword(KeywordAbility::Lifelink);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(reactor)
        .object(entering)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let entering_id = find_object(&state, "P1 Lifelinker");
    (state, entering_id)
}

/// CR 702.15a — `source: Some(EffectTarget::TriggeringCreature)` on a `DealDamage`
/// whose ability-controller (P2) differs from the entering creature's controller
/// (P1): the entering creature (which has Lifelink) is the damage source, so ITS
/// controller (P1) gains the life, not the ability's controller (P2).
///
/// Non-vacuity (verified by temporary revert): reverting Change 6 (threading
/// `damage_source_id`) makes the lifelink check read `ctx.source` (the Reactive
/// Enchantment, which has no Lifelink keyword) -- no `LifeGained` event fires for
/// anyone, and this test reddens (the assertion that P1 gains life fails).
#[test]
fn test_ef4_dealdamage_source_override_attributes_lifelink_to_source_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let (mut state, entering_id) =
        build_source_override_state(Some(CardEffectTarget::TriggeringCreature));

    enter_battlefield(&mut state, entering_id, p1);
    let (state, events) = drain_stack(state, &[p1, p2]);

    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::LifeGained { player, amount } if *player == p1 && *amount == 3)
        ),
        "PB-EF4: the entering creature's controller (P1) should gain 3 life from its own \
         Lifelink, since it -- not the ability's controller -- is the damage source"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeGained { player, .. } if *player == p2)),
        "PB-EF4: the ability's controller (P2) must NOT gain life -- P2's enchantment has \
         no Lifelink of its own"
    );
    let _ = life(&state, p2);
}

/// Companion regression to the decoy above: the SAME setup, but `DealDamage.source:
/// None` (the pre-PB-EF4 default path). The damage is sourced from `ctx.source` (the
/// Reactive Enchantment), which has no Lifelink, so no `LifeGained` event fires even
/// though the entering creature itself has Lifelink. Proves Change 6 is
/// behavior-preserving when `source` is not overridden.
#[test]
fn test_ef4_dealdamage_source_none_default_path_unchanged() {
    let p1 = p(1);
    let p2 = p(2);
    let (mut state, entering_id) = build_source_override_state(None);

    enter_battlefield(&mut state, entering_id, p1);
    let (_state, events) = drain_stack(state, &[p1, p2]);

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeGained { .. })),
        "PB-EF4 regression: with source: None, damage is sourced from ctx.source (the \
         enchantment, no Lifelink) -- the entering creature's own Lifelink must NOT apply"
    );
}

// ── Regression: departed triggering creature still reads LKI (review LOW #1) ────

/// CR 113.7a / 608.2m (SR-13 pattern) -- `Effect::DealDamage.source: Some(TriggeringCreature)`
/// must fall back to the departed creature's LKI-readable id, not `ctx.source` (the ability's
/// host), when the triggering creature has already left the battlefield before its own
/// trigger resolves (e.g. destroyed in response). `resolve_effect_target_list` gates
/// `EffectTarget::TriggeringCreature` on `state.objects.contains_key`, so a live resolve
/// returns no object in that case -- the override must not silently mis-attribute the damage
/// to the Reactive Enchantment (which has no Lifelink of its own).
///
/// Non-vacuity (verified by temporary revert): reverting the `damage_source_id` fallback to
/// `unwrap_or(ctx.source)` directly (skipping the `ctx.triggering_creature_id` step) makes the
/// lifelink check read the Reactive Enchantment instead -- no `LifeGained` event fires, and
/// this test reddens.
#[test]
fn test_ef4_dealdamage_source_departed_triggering_creature_reads_lki() {
    let p1 = p(1);
    let p2 = p(2);
    let (mut state, entering_id) =
        build_source_override_state(Some(CardEffectTarget::TriggeringCreature));

    enter_battlefield(&mut state, entering_id, p1);

    // Simulate the entering creature being destroyed in response to its own trigger.
    // CR 400.7: this retires `entering_id` and assigns a new ObjectId in the graveyard --
    // exactly the "already left the battlefield" case the fix targets. The trigger's
    // `triggering_creature_id` was already captured onto the stack object before this move,
    // so the departed id is still what the DealDamage effect will try to resolve against.
    test_util::move_object_to_zone(&mut state, entering_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard should succeed");
    assert!(
        state.objects().get(&entering_id).is_none(),
        "precondition: entering_id must be retired (CR 400.7) before the trigger resolves"
    );

    let (_state, events) = drain_stack(state, &[p1, p2]);

    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::LifeGained { player, amount } if *player == p1 && *amount == 3)
        ),
        "PB-EF4 review fix: the departed triggering creature's Lifelink should still apply via \
         LKI (damage_source_id falls back to ctx.triggering_creature_id, not ctx.source, when \
         the creature has already left the battlefield)"
    );
}

// ── Card integration: Dragon Tempest ─────────────────────────────────────────────

/// CR 603.6a / CR 611.2a / CR 119.3 — Dragon Tempest: a flyer entering gains haste
/// until end of turn (a non-flyer does not); a Dragon entering deals X damage (X =
/// Dragons controlled) sourced from the entering Dragon itself, not Dragon Tempest.
#[test]
fn test_ef4_dragon_tempest_flying_grants_haste_dragon_deals_damage() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let tempest = mtg_engine::enrich_spec_from_def(
        ObjectSpec::card(p1, "Dragon Tempest").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(tempest)
        .object(ObjectSpec::creature(p1, "Test Flyer", 2, 2).with_keyword(KeywordAbility::Flying))
        .object(
            ObjectSpec::creature(p1, "Test Dragon", 2, 2)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let flyer_id = find_object(&state, "Test Flyer");
    let dragon_id = find_object(&state, "Test Dragon");
    let p2_life_before = life(&state, p2);

    // The flyer enters: gains haste. The (non-flying) Dragon does not.
    let mut state = state;
    enter_battlefield(&mut state, flyer_id, p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let flyer_chars = calculate_characteristics(&state, flyer_id).unwrap();
    assert!(
        flyer_chars.keywords.contains(&KeywordAbility::Haste),
        "CR 611.2a: a flyer entering under Dragon Tempest should gain haste"
    );
    let dragon_chars_pre = calculate_characteristics(&state, dragon_id).unwrap();
    assert!(
        !dragon_chars_pre.keywords.contains(&KeywordAbility::Haste),
        "the non-flying Dragon must not gain haste from the flying-half trigger"
    );

    // The Dragon enters: deals X damage (X = 1 Dragon controlled) sourced from itself.
    let mut state = state;
    enter_battlefield(&mut state, dragon_id, p1);
    let (state, events) = drain_stack(state, &[p1, p2]);

    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::DamageDealt {
                source,
                amount: 1,
                ..
            } if *source == dragon_id
        )),
        "CR 119.3: Dragon Tempest's damage must be sourced from the entering Dragon \
         itself, not Dragon Tempest, amount = 1 (one Dragon controlled)"
    );
    assert_eq!(
        life(&state, p2),
        p2_life_before - 1,
        "CR 119: the auto-picked opponent should take 1 damage"
    );
}

// ── Card integration: Scourge of Valkas ──────────────────────────────────────────

/// CR 508 / CR 119.3 — Scourge of Valkas: its own ETB deals damage sourced from
/// itself; a SECOND Dragon entering also deals damage, sourced from THAT Dragon. One
/// trigger condition covers both halves ("this creature or another Dragon you
/// control enters").
#[test]
fn test_ef4_scourge_of_valkas_self_and_another_dragon() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // Scourge alone on the battlefield: its own "entry" deals X=1 damage (itself is
    // the only Dragon), sourced from Scourge itself.
    {
        let scourge = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Scourge of Valkas").in_zone(ZoneId::Battlefield),
            &defs,
        );

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(scourge)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let scourge_id = find_object(&state, "Scourge of Valkas");

        let mut state = state;
        enter_battlefield(&mut state, scourge_id, p1);
        let (_state, events) = drain_stack(state, &[p1, p2]);

        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::DamageDealt { source, amount: 1, .. } if *source == scourge_id
            )),
            "CR 119.3: Scourge's self-ETB should deal damage sourced from Scourge itself \
             (X = 1, the only Dragon controlled)"
        );
    }

    // Scourge already on the battlefield; a SECOND Dragon then enters. X = 2 Dragons
    // controlled (Scourge + the second Dragon); the damage is sourced from the
    // SECOND Dragon, not Scourge.
    {
        let scourge = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Scourge of Valkas").in_zone(ZoneId::Battlefield),
            &defs,
        );

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(scourge)
            .object(
                ObjectSpec::creature(p1, "Second Dragon", 2, 2)
                    .with_subtypes(vec![SubType("Dragon".to_string())]),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let scourge_id = find_object(&state, "Scourge of Valkas");
        let second_dragon_id = find_object(&state, "Second Dragon");

        let mut state = state;
        enter_battlefield(&mut state, second_dragon_id, p1);
        let (state, events) = drain_stack(state, &[p1, p2]);

        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::DamageDealt { source, amount: 2, .. } if *source == second_dragon_id
            )),
            "CR 119.3: another Dragon entering should deal damage sourced from THAT Dragon \
             (X = 2 Dragons controlled), not Scourge (id {:?})",
            scourge_id
        );
        let _ = life(&state, p2);
    }
}

// ── Card integration: Ogre Battledriver ──────────────────────────────────────────

/// CR 603.6a — Ogre Battledriver: another creature entering gets +2/+0 and haste;
/// Ogre's own ETB does NOT fire (exclude_self: true, "another").
#[test]
fn test_ef4_ogre_battledriver_pumps_and_hastes_only_the_enterer() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let ogre = mtg_engine::enrich_spec_from_def(
        ObjectSpec::card(p1, "Ogre Battledriver").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(ogre)
        .object(ObjectSpec::creature(p1, "Entering Creature", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let ogre_id = find_object(&state, "Ogre Battledriver");
    let entering_id = find_object(&state, "Entering Creature");

    let mut state = state;
    enter_battlefield(&mut state, entering_id, p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let entering_chars = calculate_characteristics(&state, entering_id).unwrap();
    assert_eq!(
        entering_chars.power,
        Some(4),
        "the entering creature should get +2/+0 (2+2=4)"
    );
    assert!(
        entering_chars.keywords.contains(&KeywordAbility::Haste),
        "the entering creature should gain haste"
    );

    // Ogre's own ETB must NOT fire.
    let mut state = state;
    enter_battlefield(&mut state, ogre_id, p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let ogre_chars = calculate_characteristics(&state, ogre_id).unwrap();
    assert_eq!(
        ogre_chars.power,
        Some(3),
        "CR 603.6a: 'another creature' -- Ogre's own ETB must not pump itself"
    );
    assert!(
        !ogre_chars.keywords.contains(&KeywordAbility::Haste),
        "Ogre's own ETB must not grant itself haste"
    );
}

// ── Card integration: Atarka, World Render ───────────────────────────────────────

/// CR 508.1m / CR 611.2a — Atarka, World Render: an attacking Dragon gains double
/// strike until end of turn; a non-Dragon attacker does not.
#[test]
fn test_ef4_atarka_grants_double_strike_to_attacking_dragon() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let atarka = mtg_engine::enrich_spec_from_def(
        ObjectSpec::card(p1, "Atarka, World Render").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(atarka)
        .object(
            ObjectSpec::creature(p1, "Dragon Attacker", 3, 3)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(ObjectSpec::creature(p1, "Non-Dragon Attacker", 3, 3))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let dragon_attacker_id = find_object(&state, "Dragon Attacker");
    let non_dragon_attacker_id = find_object(&state, "Non-Dragon Attacker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (dragon_attacker_id, AttackTarget::Player(p2)),
                (non_dragon_attacker_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = drain_stack(state, &[p1, p2]);

    let dragon_chars = calculate_characteristics(&state, dragon_attacker_id).unwrap();
    assert!(
        dragon_chars
            .keywords
            .contains(&KeywordAbility::DoubleStrike),
        "CR 508.1m: the attacking Dragon should gain double strike"
    );
    let non_dragon_chars = calculate_characteristics(&state, non_dragon_attacker_id).unwrap();
    assert!(
        !non_dragon_chars
            .keywords
            .contains(&KeywordAbility::DoubleStrike),
        "the non-Dragon attacker must not gain double strike"
    );
}

// ── Card integration: Fervent Charge ─────────────────────────────────────────────

/// CR 508.1m / CR 611.2a — Fervent Charge: any attacking creature you control gets
/// +2/+2 until end of turn; a non-attacking creature you control is unaffected.
#[test]
fn test_ef4_fervent_charge_pumps_attacking_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let charge = mtg_engine::enrich_spec_from_def(
        ObjectSpec::card(p1, "Fervent Charge").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(charge)
        .object(ObjectSpec::creature(p1, "Attacking Creature", 2, 2))
        .object(ObjectSpec::creature(p1, "Non-Attacking Creature", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacking Creature");
    let non_attacker_id = find_object(&state, "Non-Attacking Creature");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = drain_stack(state, &[p1, p2]);

    let attacker_chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        attacker_chars.power,
        Some(4),
        "the attacking creature should get +2/+2 (2+2=4)"
    );
    assert_eq!(attacker_chars.toughness, Some(4));

    let non_attacker_chars = calculate_characteristics(&state, non_attacker_id).unwrap();
    assert_eq!(
        non_attacker_chars.power,
        Some(2),
        "a non-attacking creature you control must be unaffected"
    );
    assert_eq!(non_attacker_chars.toughness, Some(2));
}

// ── Card integration: Dreadhorde Invasion ────────────────────────────────────────

/// CR 508.1m / CR 702.15a — Dreadhorde Invasion: a Zombie TOKEN with power >= 6
/// attacking gains lifelink until end of turn; a power-5 Zombie token and a
/// non-token Zombie with power 6 do NOT (filter gated on both `min_power` and
/// `is_token`).
#[test]
fn test_ef4_dreadhorde_invasion_lifelink_gated_by_token_and_power() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let invasion = mtg_engine::enrich_spec_from_def(
        ObjectSpec::card(p1, "Dreadhorde Invasion").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(invasion)
        .object(
            ObjectSpec::creature(p1, "Big Zombie Token", 6, 6)
                .with_subtypes(vec![SubType("Zombie".to_string())])
                .token(),
        )
        .object(
            ObjectSpec::creature(p1, "Small Zombie Token", 5, 5)
                .with_subtypes(vec![SubType("Zombie".to_string())])
                .token(),
        )
        .object(
            ObjectSpec::creature(p1, "Nontoken Big Zombie", 6, 6)
                .with_subtypes(vec![SubType("Zombie".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let big_token_id = find_object(&state, "Big Zombie Token");
    let small_token_id = find_object(&state, "Small Zombie Token");
    let nontoken_id = find_object(&state, "Nontoken Big Zombie");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (big_token_id, AttackTarget::Player(p2)),
                (small_token_id, AttackTarget::Player(p2)),
                (nontoken_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = drain_stack(state, &[p1, p2]);

    let big_chars = calculate_characteristics(&state, big_token_id).unwrap();
    assert!(
        big_chars.keywords.contains(&KeywordAbility::Lifelink),
        "a power-6 Zombie TOKEN attacking should gain lifelink"
    );
    let small_chars = calculate_characteristics(&state, small_token_id).unwrap();
    assert!(
        !small_chars.keywords.contains(&KeywordAbility::Lifelink),
        "a power-5 Zombie token must not gain lifelink (min_power gate)"
    );
    let nontoken_chars = calculate_characteristics(&state, nontoken_id).unwrap();
    assert!(
        !nontoken_chars.keywords.contains(&KeywordAbility::Lifelink),
        "a non-token power-6 Zombie must not gain lifelink (is_token gate)"
    );
}

// ── Card integration: Warstorm Surge ─────────────────────────────────────────────

/// CR 119.3 / CR 702.15a — Warstorm Surge: an entering creature deals damage equal
/// to its own power, sourced from ITSELF (not Warstorm Surge). A Lifelink-enterer
/// variant proves the source override drives the lifelink gain (not just the
/// event's `source:` field).
#[test]
fn test_ef4_warstorm_surge_entering_creature_deals_its_power_from_itself() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // Plain entering creature (no lifelink): assert DamageDealt.source == the
    // entering creature, not Warstorm Surge.
    {
        let surge = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Warstorm Surge").in_zone(ZoneId::Battlefield),
            &defs,
        );

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(surge)
            .object(ObjectSpec::creature(p1, "Entering Creature", 3, 3))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let surge_id = find_object(&state, "Warstorm Surge");
        let entering_id = find_object(&state, "Entering Creature");
        let p2_life_before = life(&state, p2);

        let mut state = state;
        enter_battlefield(&mut state, entering_id, p1);
        let (state, events) = drain_stack(state, &[p1, p2]);

        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::DamageDealt { source, amount: 3, .. }
                if *source == entering_id && *source != surge_id
            )),
            "CR 119.3: damage must be sourced from the entering creature itself (power \
             3), not Warstorm Surge"
        );
        assert_eq!(
            life(&state, p2),
            p2_life_before - 3,
            "the auto-picked opponent should take 3 damage (entering creature's power)"
        );
    }

    // Lifelink-enterer variant: the entering creature's OWN Lifelink applies, proving
    // the source override drives lifelink, not just the event's source field.
    {
        let surge = mtg_engine::enrich_spec_from_def(
            ObjectSpec::card(p1, "Warstorm Surge").in_zone(ZoneId::Battlefield),
            &defs,
        );

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(all_cards()))
            .object(surge)
            .object(
                ObjectSpec::creature(p1, "Lifelink Enterer", 2, 2)
                    .with_keyword(KeywordAbility::Lifelink),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let entering_id = find_object(&state, "Lifelink Enterer");

        let mut state = state;
        enter_battlefield(&mut state, entering_id, p1);
        let (_state, events) = drain_stack(state, &[p1, p2]);

        assert!(
            events.iter().any(
                |e| matches!(e, GameEvent::LifeGained { player, amount } if *player == p1 && *amount == 2)
            ),
            "CR 702.15a: the entering creature's OWN Lifelink should cause ITS controller \
             (P1) to gain life equal to the damage dealt (2), proving the source override \
             drives lifelink reads"
        );
    }
}
