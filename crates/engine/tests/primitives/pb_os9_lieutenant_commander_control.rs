//! PB-OS9 (OOS-EF3b-1): `Condition::YouControlYourCommander`.
//!
//! CR 903.3d (controlling a commander -- a permanent on the battlefield), 903.3
//! (the commander designation is an attribute of the CARD, tracked per-owner in
//! `PlayerState::commander_ids`), 603.4 (intervening-if re-checked at resolution),
//! 604.2 (conditional static abilities), 611.2a (control). Serves BOTH consumption
//! shapes: intervening-if on a triggered ability (`loyal_apprentice`,
//! `siege_gang_lieutenant`) and a continuous-grant condition
//! (`skyhunter_strike_force`).
//!
//! **"Your commander" vs "a commander" (the central correctness trap):** this
//! Condition is strictly *owned-by-controller* -- distinct from the existing
//! CommanderFreeCast predicate (CR 118.9, `casting.rs`) which accepts ANY player's
//! commander. `test_control_opponents_commander_only_still_off` pins this directly.
//!
//! **Genuine engine gap discovered during this batch (verified by execution, not
//! just source-reading -- SR-34/36):** `TriggerCondition::AtBeginningOfCombat` has
//! NO card-def sweep anywhere in the engine's turn-based-action machinery.
//! `crates/engine/src/rules/turn_actions.rs` hardcodes a per-step sweep for
//! AtBeginningOfYourUpkeep / AtBeginningOfFirstMainPhase / AtBeginningOfPostcombatMain
//! / AtBeginningOfYourEndStep, but `begin_combat()` (the `Step::BeginningOfCombat`
//! handler) only queues EMBLEM triggers (`collect_emblem_triggers_for_event`) --
//! never card-defined `AbilityDefinition::Triggered { trigger_condition:
//! TriggerCondition::AtBeginningOfCombat, .. }` abilities. Confirmed empirically: a
//! battlefield object with this trigger condition produces ZERO pending triggers and
//! ZERO stack objects when the game transitions into `BeginningOfCombat` via the real
//! `Command::PassPriority` path. This is a PRE-EXISTING gap (also silently affects
//! `legion_warboss.rs`, `goblin_rabblemaster.rs`, `mirage_phalanx.rs`,
//! `helm_of_the_host.rs`), out of PB-OS9's scope to fix (see the "STILL BLOCKED"
//! notes on `loyal_apprentice.rs` / `siege_gang_lieutenant.rs` for the full account).
//! Because of this, `loyal_apprentice` and `siege_gang_lieutenant` stay
//! `Completeness::partial` -- their Lieutenant DSL is CR-correct but currently inert
//! in real gameplay. The tests below for those two cards deliberately isolate the
//! part that IS this primitive's job (the intervening-if condition re-check AT
//! RESOLUTION, CR 603.4, and the effect it gates) by queueing the exact
//! `PendingTrigger` the missing sweep would produce, directly -- proving
//! `Condition::YouControlYourCommander` and the CreateToken effect are wired
//! correctly and are ready to fire the moment that sweep is added. They do NOT claim
//! the trigger fires via a real `BeginningOfCombat` step transition (it does not).
//!
//! `skyhunter_strike_force`'s continuous-grant condition has no such dependency --
//! it goes through `check_static_condition`'s layer-application-time re-evaluation
//! (`calculate_characteristics`), unrelated to the broken sweep, and is fully
//! `Completeness::Complete`.

use mtg_engine::effects::{check_condition, check_static_condition, execute_effect, EffectContext};
use mtg_engine::rules::abilities::flush_pending_triggers;
use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::state::stubs::{PendingTrigger, PendingTriggerKind};
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardEffectTarget, CardId, CardRegistry, CardType, Command, Condition, Effect,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, SuperType, TypeLine, ZoneId, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn ec(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

fn load_defs_from(defs: &[CardDefinition]) -> HashMap<String, CardDefinition> {
    defs.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

/// Minimal legendary-creature commander definition (mirrors
/// `mechanics_a_d/domain_and_freecast.rs::test_commander_def`).
fn commander_def(name: &str, id: &str) -> CardDefinition {
    CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            supertypes: [SuperType::Legendary].into_iter().collect(),
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Legendary creature.".to_string(),
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
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

fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(
            guard < 50,
            "drain_stack: stack did not empty after 50 rounds"
        );
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// `GameStateBuilder` places battlefield objects directly (no real ETB event), so
/// `AbilityDefinition::Static` continuous effects (e.g. Skyhunter's Lieutenant
/// anthem) are never registered into `state.continuous_effects` automatically --
/// that registration normally happens at `resolution.rs`'s ETB site. Call it
/// explicitly to simulate "this permanent entered the battlefield."
fn register_skyhunter_static(state: &mut GameState, skyhunter_id: ObjectId) {
    let registry = state.card_registry().clone();
    register_static_continuous_effects(
        state,
        skyhunter_id,
        Some(&cid("skyhunter-strike-force")),
        &registry,
        false,
    );
}

fn declare_attackers(
    state: GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> GameState {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");
    state
}

// ── Tests 1-6: direct check_condition unit tests ────────────────────────────────

/// CR 903.3d: your own commander, on the battlefield, under your control -> true.
#[test]
fn test_you_control_your_commander_true_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-1");

    let commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_cid)
        .object(commander)
        .build()
        .unwrap();

    let commander_id = find_object(&state, "Test Commander");
    let ctx = ec(p1, commander_id);
    assert!(
        check_condition(&state, &Condition::YouControlYourCommander, &ctx),
        "CR 903.3d: p1 controls their own commander on the battlefield -- should be true"
    );
}

/// CR 903.3d: commander in the command zone (not on the battlefield) -> false.
/// Non-vacuous: an ordinary battlefield creature is present so the object scan is
/// non-empty.
#[test]
fn test_you_control_your_commander_false_in_command_zone() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-2");

    let commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Command(p1));
    let ordinary = ObjectSpec::creature(p1, "Ordinary Creature", 2, 2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_cid)
        .object(commander)
        .object(ordinary)
        .build()
        .unwrap();

    let ordinary_id = find_object(&state, "Ordinary Creature");
    let ctx = ec(p1, ordinary_id);
    assert!(
        !check_condition(&state, &Condition::YouControlYourCommander, &ctx),
        "CR 903.3d: the commander is in the command zone, not on the battlefield -- \
         should be false"
    );
}

/// CR 903.3d + SBA batch semantics: destroying the commander drops the condition
/// in the same recompute (CR 701.7 destroy is an immediate zone change).
#[test]
fn test_you_control_your_commander_drops_when_commander_dies() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-3");

    let commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_cid)
        .object(commander)
        .build()
        .unwrap();

    let commander_id = find_object(&state, "Test Commander");
    let ctx = ec(p1, commander_id);
    assert!(
        check_condition(&state, &Condition::YouControlYourCommander, &ctx),
        "precondition: commander on the battlefield -- should be true"
    );

    execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut ec(p1, commander_id),
    );

    let ctx_after = ec(p1, commander_id);
    assert!(
        !check_condition(&state, &Condition::YouControlYourCommander, &ctx_after),
        "CR 903.3d: after the commander is destroyed it is no longer on the \
         battlefield -- the condition should drop immediately"
    );
}

/// STOLEN commander decoy: p2 controls p1's commander. For p1 -- Lieutenant OFF
/// (they do not control it). For p2 -- ALSO off (the card is in p1's
/// `commander_ids`, not p2's -- "your commander" is owned, not just controlled).
/// Non-vacuous: a second fixture where control matches ownership proves the same
/// setup would read true if control had not been stolen.
#[test]
fn test_stolen_commander_decoy_lieutenant_off() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-4");

    // Stolen: p1 owns (player_commander), p2 controls (controlled_by).
    let stolen_commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .controlled_by(p2)
        .in_zone(ZoneId::Battlefield);

    let stolen_state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_cid.clone())
        .object(stolen_commander)
        .build()
        .unwrap();

    let stolen_id = find_object(&stolen_state, "Test Commander");
    assert!(
        !check_condition(
            &stolen_state,
            &Condition::YouControlYourCommander,
            &ec(p1, stolen_id)
        ),
        "CR 903.3d: p1 does not CONTROL their stolen commander -- Lieutenant should \
         be OFF for p1"
    );
    assert!(
        !check_condition(
            &stolen_state,
            &Condition::YouControlYourCommander,
            &ec(p2, stolen_id)
        ),
        "'your commander' is OWNED, not just controlled -- p2 controls the card but \
         it is in p1's commander_ids, not p2's -- Lieutenant should ALSO be OFF for p2"
    );

    // Stole back (control returned to the owner): the same setup with control
    // matching ownership -- proves the false results above are not vacuous.
    let returned_commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);
    let returned_state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_cid)
        .object(returned_commander)
        .build()
        .unwrap();
    let returned_id = find_object(&returned_state, "Test Commander");
    assert!(
        check_condition(
            &returned_state,
            &Condition::YouControlYourCommander,
            &ec(p1, returned_id)
        ),
        "non-vacuous check: with control matching ownership, the SAME commander \
         card should read true for p1"
    );
}

/// CR 903.3d: p1 controls p2's commander but not their OWN (which sits in the
/// command zone) -- Lieutenant should still be OFF. This is the exact scenario
/// that WOULD satisfy CR 118.9's CommanderFreeCast predicate ("a commander", any
/// owner) -- proving Lieutenant does NOT reuse that looser check.
#[test]
fn test_control_opponents_commander_only_still_off() {
    let p1 = p(1);
    let p2 = p(2);
    let p1_own_commander_cid = cid("test-commander-p1-own");
    let p2_commander_cid = cid("test-commander-p2");

    // p1's OWN commander stays in the command zone (not controlled/on battlefield).
    let p1_commander = ObjectSpec::creature(p1, "P1 Own Commander", 3, 3)
        .with_card_id(p1_own_commander_cid.clone())
        .in_zone(ZoneId::Command(p1));
    // p2's commander, on the battlefield, controlled by p1.
    let p2_commander = ObjectSpec::creature(p2, "P2 Commander", 2, 2)
        .with_card_id(p2_commander_cid.clone())
        .controlled_by(p1)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, p1_own_commander_cid)
        .player_commander(p2, p2_commander_cid)
        .object(p1_commander)
        .object(p2_commander)
        .build()
        .unwrap();

    let p2_commander_id = find_object(&state, "P2 Commander");
    let ctx = ec(p1, p2_commander_id);
    assert!(
        !check_condition(&state, &Condition::YouControlYourCommander, &ctx),
        "CR 903.3d: p1 controls an OPPONENT's commander but not their own -- \
         Lieutenant should be OFF ('your commander' is owned, distinct from the \
         CR 118.9 CommanderFreeCast 'a commander' predicate)"
    );
}

/// Partner commanders ruling: controlling just ONE of two owned commanders suffices.
#[test]
fn test_multiple_commanders_control_one_suffices() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_a_cid = cid("test-commander-a");
    let commander_b_cid = cid("test-commander-b");

    // Commander A on the battlefield; Commander B stays in the command zone.
    let commander_a = ObjectSpec::creature(p1, "Test Commander A", 3, 3)
        .with_card_id(commander_a_cid.clone())
        .in_zone(ZoneId::Battlefield);
    let commander_b = ObjectSpec::creature(p1, "Test Commander B", 2, 2)
        .with_card_id(commander_b_cid.clone())
        .in_zone(ZoneId::Command(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_a_cid)
        .player_commander(p1, commander_b_cid)
        .object(commander_a)
        .object(commander_b)
        .build()
        .unwrap();

    let commander_a_id = find_object(&state, "Test Commander A");
    let ctx = ec(p1, commander_a_id);
    assert!(
        check_condition(&state, &Condition::YouControlYourCommander, &ctx),
        "partner ruling: controlling just one of two owned commanders suffices -- \
         should be true"
    );
}

// ── Tests 7-9: Skyhunter Strike Force continuous-grant condition ───────────────

/// CR 903.3d/604.2: static grant active while the commander is controlled --
/// `calculate_characteristics` (the `check_static_condition` fallback path) grants
/// Melee to another creature you control; the Melee attack trigger (synthesized
/// per PB-EF3b for granted keywords) actually fires and pumps it.
#[test]
fn test_skyhunter_grant_active_and_melee_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sh1");

    let mut all = all_cards();
    all.push(commander_def("Test Commander SH1", "test-commander-sh1"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let skyhunter = enrich_spec_from_def(
        ObjectSpec::card(p1, "Skyhunter Strike Force")
            .with_card_id(cid("skyhunter-strike-force"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SH1", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);
    let buddy = ObjectSpec::creature(p1, "Buddy Creature", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(skyhunter)
        .object(commander)
        .object(buddy)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let skyhunter_id = find_object(&state, "Skyhunter Strike Force");
    register_skyhunter_static(&mut state, skyhunter_id);

    let buddy_id = find_object(&state, "Buddy Creature");
    let chars = calculate_characteristics(&state, buddy_id)
        .expect("Buddy Creature should have resolved characteristics");
    assert!(
        chars.keywords.contains(&KeywordAbility::Melee),
        "CR 903.3d/604.2: with the commander controlled, Skyhunter's Lieutenant \
         anthem should grant Melee to other creatures you control"
    );

    let state = declare_attackers(state, p1, vec![(buddy_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let power_after = calculate_characteristics(&state, buddy_id).and_then(|c| c.power);
    assert_eq!(
        power_after,
        Some(3),
        "post-PB-EF3b: the granted Melee's synthesized attack trigger should fire and \
         pump Buddy Creature +1/+1 for the one opponent it attacked (2 base -> 3)"
    );
}

/// The grant drops the moment the commander leaves the battlefield.
#[test]
fn test_skyhunter_grant_drops_when_commander_leaves() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sh2");

    let mut all = all_cards();
    all.push(commander_def("Test Commander SH2", "test-commander-sh2"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let skyhunter = enrich_spec_from_def(
        ObjectSpec::card(p1, "Skyhunter Strike Force")
            .with_card_id(cid("skyhunter-strike-force"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SH2", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);
    let buddy = ObjectSpec::creature(p1, "Buddy Creature", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(skyhunter)
        .object(commander)
        .object(buddy)
        .build()
        .unwrap();

    let skyhunter_id = find_object(&state, "Skyhunter Strike Force");
    register_skyhunter_static(&mut state, skyhunter_id);

    let buddy_id = find_object(&state, "Buddy Creature");
    let commander_id = find_object(&state, "Test Commander SH2");
    assert!(
        calculate_characteristics(&state, buddy_id)
            .unwrap()
            .keywords
            .contains(&KeywordAbility::Melee),
        "precondition: grant active while commander is controlled"
    );

    execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut ec(p1, commander_id),
    );

    assert!(
        !calculate_characteristics(&state, buddy_id)
            .unwrap()
            .keywords
            .contains(&KeywordAbility::Melee),
        "the commander left the battlefield -- the Lieutenant anthem's condition \
         should drop and Buddy Creature should lose Melee"
    );
}

/// STOLEN-commander sub-case: opponent gains control of your commander -> the
/// grant is off (same fixture as the active test, but controlled by p2).
#[test]
fn test_skyhunter_grant_off_when_commander_stolen() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sh3");

    let mut all = all_cards();
    all.push(commander_def("Test Commander SH3", "test-commander-sh3"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let skyhunter = enrich_spec_from_def(
        ObjectSpec::card(p1, "Skyhunter Strike Force")
            .with_card_id(cid("skyhunter-strike-force"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SH3", 3, 3)
        .with_card_id(commander_cid.clone())
        .controlled_by(p2)
        .in_zone(ZoneId::Battlefield);
    let buddy = ObjectSpec::creature(p1, "Buddy Creature", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(skyhunter)
        .object(commander)
        .object(buddy)
        .build()
        .unwrap();

    let skyhunter_id = find_object(&state, "Skyhunter Strike Force");
    register_skyhunter_static(&mut state, skyhunter_id);

    let buddy_id = find_object(&state, "Buddy Creature");
    assert!(
        !calculate_characteristics(&state, buddy_id)
            .unwrap()
            .keywords
            .contains(&KeywordAbility::Melee),
        "the commander is on the battlefield but controlled by an OPPONENT -- p1 \
         does not control it, so the Lieutenant anthem should NOT grant Melee"
    );
}

// ── Tests 10-11: Siege-Gang Lieutenant intervening-if at resolution ────────────

/// CR 903.3d/603.4: queues the exact `PendingTrigger` the (currently-missing)
/// `AtBeginningOfCombat` card-def sweep would produce, directly -- proving
/// `Condition::YouControlYourCommander`'s intervening-if re-check at resolution and
/// the CreateToken effect are wired correctly, isolated from the separately-tracked
/// engine gap (see the file-level doc comment).
#[test]
fn test_siege_gang_lieutenant_intervening_if_creates_tokens() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sg1");

    let mut all = all_cards();
    all.push(commander_def("Test Commander SG1", "test-commander-sg1"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let siege_gang = enrich_spec_from_def(
        ObjectSpec::card(p1, "Siege-Gang Lieutenant")
            .with_card_id(cid("siege-gang-lieutenant"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SG1", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(siege_gang)
        .object(commander)
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .build()
        .unwrap();

    let sg_id = find_object(&state, "Siege-Gang Lieutenant");

    // ability_index 0 is the Lieutenant Triggered ability (see siege_gang_lieutenant.rs).
    state.pending_triggers_mut().push_back(PendingTrigger {
        ability_index: 0,
        ..PendingTrigger::blank(sg_id, p1, PendingTriggerKind::CardDefETB)
    });
    flush_pending_triggers(&mut state);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);

    let goblin_count = state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == "Goblin" && o.zone == ZoneId::Battlefield)
        .count();
    assert_eq!(
        goblin_count, 2,
        "CR 903.3d/603.4: with the commander controlled at resolution, the \
         Lieutenant trigger should resolve and create two Goblin tokens"
    );
}

/// CR 603.4: the commander is removed BEFORE the trigger resolves -- the
/// intervening-if re-check at resolution should fail and no tokens are created.
#[test]
fn test_siege_gang_lieutenant_intervening_if_fails_when_commander_removed() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sg2");

    let mut all = all_cards();
    all.push(commander_def("Test Commander SG2", "test-commander-sg2"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let siege_gang = enrich_spec_from_def(
        ObjectSpec::card(p1, "Siege-Gang Lieutenant")
            .with_card_id(cid("siege-gang-lieutenant"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SG2", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(siege_gang)
        .object(commander)
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .build()
        .unwrap();

    let sg_id = find_object(&state, "Siege-Gang Lieutenant");
    let commander_id = find_object(&state, "Test Commander SG2");

    state.pending_triggers_mut().push_back(PendingTrigger {
        ability_index: 0,
        ..PendingTrigger::blank(sg_id, p1, PendingTriggerKind::CardDefETB)
    });
    flush_pending_triggers(&mut state);

    // "Removed in response" -- destroy the commander while the trigger sits on the
    // stack, before it resolves.
    execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut ec(p1, commander_id),
    );

    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let goblin_count = state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == "Goblin" && o.zone == ZoneId::Battlefield)
        .count();
    assert_eq!(
        goblin_count, 0,
        "CR 603.4: the commander was removed before the trigger resolved -- the \
         intervening-if re-check should fail and no Goblin tokens should be created"
    );
}

// ── Test 12: Loyal Apprentice token shape ───────────────────────────────────────

/// CR 903.3d/603.4: same isolation strategy as Siege-Gang -- proves the Thopter
/// token shape (1/1 colorless flying artifact creature, haste) and the
/// intervening-if gate are correct.
#[test]
fn test_loyal_apprentice_thopter_token_with_haste() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-la1");

    let mut all = all_cards();
    all.push(commander_def("Test Commander LA1", "test-commander-la1"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let apprentice = enrich_spec_from_def(
        ObjectSpec::card(p1, "Loyal Apprentice")
            .with_card_id(cid("loyal-apprentice"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander LA1", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(apprentice)
        .object(commander)
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .build()
        .unwrap();

    let apprentice_id = find_object(&state, "Loyal Apprentice");

    // ability_index 1: abilities[0] is Keyword(Haste), abilities[1] is the
    // Lieutenant Triggered ability (see loyal_apprentice.rs).
    state.pending_triggers_mut().push_back(PendingTrigger {
        ability_index: 1,
        ..PendingTrigger::blank(apprentice_id, p1, PendingTriggerKind::CardDefETB)
    });
    flush_pending_triggers(&mut state);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);

    let thopter = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Thopter" && o.zone == ZoneId::Battlefield)
        .unwrap_or_else(|| panic!("Thopter token should have been created"));
    assert!(
        thopter
            .characteristics
            .card_types
            .contains(&CardType::Artifact)
            && thopter
                .characteristics
                .card_types
                .contains(&CardType::Creature),
        "the Thopter token should be an artifact creature"
    );
    assert_eq!(thopter.characteristics.power, Some(1));
    assert_eq!(thopter.characteristics.toughness, Some(1));
    assert!(
        thopter
            .characteristics
            .keywords
            .contains(&KeywordAbility::Flying),
        "the Thopter token should have flying"
    );
    assert!(
        thopter
            .characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "the Thopter token should have haste (permanent-haste fallback, see \
         loyal_apprentice.rs's top-of-file comment)"
    );
}

// ── Test 13: registration smoke test ────────────────────────────────────────────

#[test]
fn test_pb_os9_cards_registered() {
    let all = all_cards();
    for name in [
        "Skyhunter Strike Force",
        "Loyal Apprentice",
        "Siege-Gang Lieutenant",
    ] {
        assert!(
            all.iter().any(|d| d.name == name),
            "'{}' should be present in all_cards()",
            name
        );
    }
}

// ── Test 14: check_static_condition fallback route (proves Change 3) ───────────

/// Direct proof that `check_static_condition`'s `_ =>` fallback correctly routes
/// `Condition::YouControlYourCommander` (no dedicated arm was added -- see
/// `crates/engine/src/effects/mod.rs`'s Change 3 comment).
#[test]
fn test_check_static_condition_fallback_routes_you_control_your_commander() {
    let p1 = p(1);
    let commander_cid = cid("test-commander-static");

    let commander = ObjectSpec::creature(p1, "Test Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);
    let source = ObjectSpec::creature(p1, "Some Source", 1, 1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(all_cards()))
        .player_commander(p1, commander_cid)
        .object(commander)
        .object(source)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Some Source");
    assert!(
        check_static_condition(&state, &Condition::YouControlYourCommander, source_id, p1),
        "check_static_condition's fallback should route to check_condition and \
         correctly evaluate true when the commander is controlled"
    );
}

// ── Test 15: wire sentinels ──────────────────────────────────────────────────────

/// PB-OS9 bumped PROTOCOL_VERSION 23 -> 24 and HASH_SCHEMA_VERSION 60 -> 61 (a
/// single new `Condition` unit variant, `YouControlYourCommander`, discriminant 51).
#[test]
fn test_pb_os9_version_sentinels() {
    assert_eq!(
        PROTOCOL_VERSION, 27,
        "PROTOCOL_VERSION should be 24 after PB-OS9 (Condition::YouControlYourCommander)"
    );
    assert_eq!(
        HASH_SCHEMA_VERSION, 63u8,
        "HASH_SCHEMA_VERSION should be 61 after PB-OS9 (Condition::YouControlYourCommander, \
         discriminant 51)"
    );
}
