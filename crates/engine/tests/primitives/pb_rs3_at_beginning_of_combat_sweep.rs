//! PB-RS3 (OOS-OS9-1): card-def `AtBeginningOfCombat` sweep.
//!
//! `begin_combat` (`crates/engine/src/rules/turn_actions.rs:1684-1703`) only
//! collects EMBLEM triggers for `TriggerEvent::AtBeginningOfCombat` (CR 114.4) --
//! it never scans the battlefield for card-defined
//! `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::
//! AtBeginningOfCombat, .. }` abilities. `helm_of_the_host` is `Completeness::
//! Complete` (by `#[default]` -- it carries no explicit marker) and its only
//! non-Equip ability is exactly this trigger, so it is deck-legal today and
//! silently does nothing (Invariant #9).
//!
//! Step 0 of this PB: this file contains ONLY the probe (Test 1 of the plan's
//! §6 table, `memory/primitives/pb-plan-RS3.md`). It must FAIL against pre-fix
//! HEAD. No production code is touched in this commit.

use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry,
    Command, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Helpers (mirrors tests/primitives/pb_os9_lieutenant_commander_control.rs) ──

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

fn load_defs_from(defs: &[CardDefinition]) -> HashMap<String, CardDefinition> {
    defs.iter().map(|d| (d.name.clone(), d.clone())).collect()
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

// ── Test 1 (the probe) ──────────────────────────────────────────────────────

/// CR 506.1 (beginning of combat is a step of the combat phase), 603.2
/// (triggered abilities trigger on their event), 603.3 (a triggered ability
/// is put on the stack the next time a player would receive priority).
///
/// Helm of the Host, attached to a vanilla creature, on p1's (the active
/// player's) battlefield. Drive the game through the REAL step transition
/// from `PreCombatMain` into `Step::BeginningOfCombat` via `Command::
/// PassPriority` (NOT a manually-pushed `PendingTrigger` -- that isolation
/// strategy is what `pb_os9_lieutenant_commander_control.rs` already used and
/// it cannot detect this bug, since it bypasses `begin_combat` entirely).
///
/// Pre-fix: `begin_combat` has no card-def scan, so the trigger never queues,
/// never hits the stack, and no token is created. This assertion must FAIL
/// against pre-fix HEAD.
#[test]
fn test_helm_of_the_host_creates_token_copy_at_beginning_of_combat() {
    let p1 = p(1);
    let p2 = p(2);

    let all = all_cards();
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let helm = enrich_spec_from_def(
        ObjectSpec::card(p1, "Helm of the Host")
            .with_card_id(cid("helm-of-the-host"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let creature = ObjectSpec::creature(p1, "Test Equipped Creature", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(helm)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let helm_id = find_object(&state, "Helm of the Host");
    let creature_id = find_object(&state, "Test Equipped Creature");

    // CR 301.5c: manually attach the Helm to the creature (equivalent to
    // resolving its Equip ability -- the attach mechanism itself is not what
    // this probe is testing).
    state.objects_mut().get_mut(&helm_id).unwrap().attached_to = Some(creature_id);
    state
        .objects_mut()
        .get_mut(&creature_id)
        .unwrap()
        .attachments
        .push_back(helm_id);

    // Drive the real step transition: both players pass priority with an
    // empty stack at PreCombatMain. `handle_all_passed` -> `advance_step`
    // moves PreCombatMain -> BeginningOfCombat; `enter_step` then runs
    // `execute_turn_based_actions` (which dispatches to `begin_combat`) and,
    // since BeginningOfCombat has priority, flushes any pending triggers onto
    // the stack before granting priority back to the active player.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.turn().step,
        Step::BeginningOfCombat,
        "sanity check: the step transition itself should have happened \
         regardless of the trigger-sweep bug"
    );

    // Let anything now on the stack (post-fix: the Helm's trigger) resolve.
    let (state, _) = drain_stack(state, &[p1, p2]);

    let token_count = state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.is_token
                && o.characteristics.name == "Test Equipped Creature"
        })
        .count();

    assert_eq!(
        token_count, 1,
        "CR 506.1/603.2/603.3: Helm of the Host's AtBeginningOfCombat trigger \
         should fire when the game transitions into the beginning of combat \
         step (on the active player's turn) and create a token copy of the \
         equipped creature -- `begin_combat` has no card-def scan today, so \
         this is expected to observe 0 tokens pre-fix"
    );
}
