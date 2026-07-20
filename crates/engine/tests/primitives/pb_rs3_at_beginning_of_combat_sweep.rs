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

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AttackTarget, CardDefinition,
    CardEffectTarget, CardId, CardRegistry, CardType, Color, Command, Effect, EffectAmount,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec, Phase,
    PlayerId, Step, SubType, SuperType, TokenSpec, TriggerEvent, TriggeredAbilityDef, TypeLine,
    ZoneId,
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

// ── Additional helpers for Tests 2-8 ─────────────────────────────────────────

/// Minimal legendary-creature commander definition (mirrors
/// `pb_os9_lieutenant_commander_control.rs::commander_def`).
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

/// Drive `state` forward, passing priority with whoever currently holds it,
/// until `state.turn().step == target`. Never issues `Command::DeclareAttackers`
/// -- with `combat.attackers` empty, `advance_step` (CR 508.8) auto-skips
/// DeclareBlockers/CombatDamage straight to EndOfCombat, so a pure
/// pass-priority walk is sufficient to traverse an entire no-attacks combat
/// phase (mirrors `tests/combat/additional_combat.rs`'s `pass_until_step_advance`,
/// generalized to a target step rather than "the next step").
fn drive_to_step(mut state: GameState, target: Step) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut guard = 0;
    while state.turn().step != target {
        guard += 1;
        assert!(
            guard < 300,
            "drive_to_step: did not reach {:?} within 300 rounds (stuck at {:?})",
            target,
            state.turn().step
        );
        let holder = state.turn().priority_holder.unwrap_or_else(|| {
            panic!(
                "no priority holder while driving toward {:?} (currently at {:?})",
                target,
                state.turn().step
            )
        });
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

fn count_battlefield_tokens_named(state: &GameState, name: &str) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.is_token && o.characteristics.name == name)
        .count()
}

// ── Test 2: the index-space discriminator (plan §3c) ────────────────────────

/// CR 603.3a: `loyal_apprentice`'s `Triggered` ability sits at
/// `abilities[1]`, BEHIND `abilities[0] == Keyword(Haste)`. The sweep must
/// compute `ability_index` by enumerating `effective_abilities()` FIRST and
/// filtering second (preserving true position), never by filtering to
/// `Triggered` abilities first and then enumerating the filtered list (which
/// would number Loyal Apprentice's only Triggered ability "0", its position
/// among Triggered abilities, not "1", its true position in
/// `effective_abilities()`). A regression to filter-then-enumerate resolves
/// `effective_abilities()[0]` == `Keyword(Haste)` at the CardDefETB branch,
/// which hits the `_ => None` arm (`resolution.rs:2041`) and silently no-ops:
/// the ability still fires (an `AbilityTriggered` event, a stack object,
/// `AbilityResolved`) but creates nothing. `siege_gang_lieutenant` and
/// `helm_of_the_host` both have their trigger at TRUE index 0 (nothing
/// precedes it), so neither can distinguish the two index-computation
/// strategies -- only a card with a non-Triggered ability before its
/// Triggered ability can. This test exists solely for that reason; do not
/// "simplify" the sweep's enumerate-then-filter shape to look more like a
/// plain filter -- doing so passes `cargo check` and breaks this test.
///
/// This test also catches the OTHER shape of plan §12's R1 hazard: an
/// implementer "correcting" the sweep to index into the dense
/// `characteristics.triggered_abilities` namespace (the one
/// `collect_triggers_for_event` uses for the `Normal` trigger path) instead of
/// `def.effective_abilities(is_transformed)` (the CardDefETB namespace
/// `resolution.rs:2019-2020` always requires -- "Always use the card
/// registry — never runtime triggered_abilities"). That dense namespace has
/// no lowering arm for step-based conditions like `AtBeginningOfCombat` at
/// all, so it would resolve to nothing and this test would observe 0
/// Thopters, not just the wrong one. See also
/// `tests/primitives/pb_ac7_ability_index_desync.rs`, which pins the same
/// dense-vs-CardDefETB distinction for a different trigger family.
#[test]
fn test_loyal_apprentice_trigger_uses_carddef_ability_index_namespace() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-idx");

    let mut all = all_cards();
    all.push(commander_def("Test Commander IDX", "test-commander-idx"));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let apprentice = enrich_spec_from_def(
        ObjectSpec::card(p1, "Loyal Apprentice")
            .with_card_id(cid("loyal-apprentice"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander IDX", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(apprentice)
        .object(commander)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Real step transition (PreCombatMain -> BeginningOfCombat), exactly like
    // Test 1 -- this is the only path that exercises the sweep's index
    // computation. A manually-pushed PendingTrigger (as
    // `pb_os9_lieutenant_commander_control.rs` used before this PB) bypasses
    // `begin_combat` entirely and cannot catch an index-space regression.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);

    let (state, _) = drain_stack(state, &[p1, p2]);

    let thopter_count = count_battlefield_tokens_named(&state, "Thopter");
    assert_eq!(
        thopter_count, 1,
        "CR 603.3a: exactly one Thopter should be created -- if this is 0, the \
         sweep computed ability_index=0 (Keyword(Haste)'s position) instead of \
         the true index 1 (the Triggered ability's actual position in \
         effective_abilities()), and the CardDefETB resolution branch silently \
         no-opped on the wrong ability"
    );
}

// ── Tests 3a/3b: siege_gang_lieutenant intervening-if, both directions ──────
//
// CR 603.4: "if [condition]" is checked BOTH when the ability triggers AND
// again when it resolves. Both checks are driven here through the REAL
// `begin_combat` step transition (not a manually-queued PendingTrigger), so
// these are genuinely end-to-end in a way `pb_os9_lieutenant_commander_control.rs`'s
// same-named tests are not (that file's tests predate the sweep and push the
// PendingTrigger by hand -- see that file's corrected header comment).
//
// Accepted engine-wide limitation (F3, `memory/card-authoring/review-pb-rs3-roster.md`):
// the trigger-time half of CR 603.4 is not separately re-checked here (only the
// resolution-time half is) -- `begin_combat`'s sweep queues the trigger
// unconditionally whenever the card-def carries the trigger condition, with no
// intervening-if check at queue time. That is a pre-existing, engine-wide
// convention (documented at `turn_actions.rs:265-266` for the analogous upkeep
// sweep), not something introduced or fixed by this PB. Filed as a seed
// (`rider-seed-triage-2026-07-19.md`) rather than fixed here. Test 3b below
// exercises the ONE direction the engine actually implements (condition true
// at trigger time, false by resolution time); the reverse divergent case
// (condition false at trigger time, true by resolution time -- which real MTG
// never triggers but this engine currently would, because there is no
// trigger-time check at all) is exactly the gap F3 describes and is NOT
// something a passing test here should imply is handled.

/// CR 603.4 (holds direction) / CR 903.3d: commander controlled both when the
/// trigger queues and when it resolves -- both Goblin tokens are created.
#[test]
fn test_siege_gang_lieutenant_intervening_if_holds_creates_two_goblins() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sg-holds");

    let mut all = all_cards();
    all.push(commander_def(
        "Test Commander SG Holds",
        "test-commander-sg-holds",
    ));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let siege_gang = enrich_spec_from_def(
        ObjectSpec::card(p1, "Siege-Gang Lieutenant")
            .with_card_id(cid("siege-gang-lieutenant"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SG Holds", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(siege_gang)
        .object(commander)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Real transition queues + immediately flushes the trigger onto the stack
    // (CR 603.3), but does not yet resolve it.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);

    // Commander stays on the battlefield throughout -- resolve the stack.
    let (state, _) = drain_stack(state, &[p1, p2]);

    let goblin_count = count_battlefield_tokens_named(&state, "Goblin");
    assert_eq!(
        goblin_count, 2,
        "CR 903.3d/603.4: with the commander controlled at both trigger and \
         resolution time, the Lieutenant trigger should create two Goblin tokens"
    );
}

/// CR 603.4 (fails direction): the commander is removed AFTER the trigger is
/// on the stack but BEFORE it resolves -- the resolution-time re-check should
/// fail and no tokens are created.
#[test]
fn test_siege_gang_lieutenant_intervening_if_fails_when_commander_removed() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("test-commander-sg-fails");

    let mut all = all_cards();
    all.push(commander_def(
        "Test Commander SG Fails",
        "test-commander-sg-fails",
    ));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let siege_gang = enrich_spec_from_def(
        ObjectSpec::card(p1, "Siege-Gang Lieutenant")
            .with_card_id(cid("siege-gang-lieutenant"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander = ObjectSpec::creature(p1, "Test Commander SG Fails", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(siege_gang)
        .object(commander)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let (mut state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);

    let commander_id = find_object(&state, "Test Commander SG Fails");
    // "Removed in response" -- destroy the commander while the Lieutenant
    // trigger sits on the stack, before it resolves.
    execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut EffectContext::new(p1, commander_id, vec![]),
    );

    let (state, _) = drain_stack(state, &[p1, p2]);

    let goblin_count = count_battlefield_tokens_named(&state, "Goblin");
    assert_eq!(
        goblin_count, 0,
        "CR 603.4: the commander was removed before the trigger resolved -- the \
         intervening-if re-check at resolution should fail and no Goblin tokens \
         should be created"
    );
}

// ── Test 5: APNAP / controller scoping (CR 101.4, 603.3b) ──────────────────

/// CR 101.4 (APNAP order) + CR 603.2 (controller-scoped trigger condition):
/// 4-player game. p1 (active) AND p2 (non-active) each control a Loyal
/// Apprentice and their own commander. Only p1's trigger should queue and
/// resolve.
///
/// Honest framing (matches the plan): because `AtBeginningOfCombat` is
/// active-player-only ("at the beginning of combat on YOUR turn" -- there is
/// no each-combat form in the corpus), the APNAP sort at
/// `abilities.rs:6970-6975` is exercised on a batch that is single-controller
/// by construction here. This test's real content is the controller filter in
/// `begin_combat` (§4): p2's card-def trigger must never even be QUEUED, not
/// merely "queued but sorted second." The `AbilityTriggered` event list is
/// asserted directly against that stronger claim.
#[test]
fn test_at_beginning_of_combat_multiplayer_only_active_player_triggers() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let p1_commander_cid = cid("test-commander-apnap-p1");
    let p2_commander_cid = cid("test-commander-apnap-p2");

    let mut all = all_cards();
    all.push(commander_def(
        "Test Commander APNAP P1",
        "test-commander-apnap-p1",
    ));
    all.push(commander_def(
        "Test Commander APNAP P2",
        "test-commander-apnap-p2",
    ));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    // NOTE: card_id must be the REAL registry id ("loyal-apprentice") for both
    // copies -- the runtime dispatch path looks up `state.card_registry.get
    // (obj.card_id)`, which is keyed by the id `all_cards()` assigns the card,
    // not by whatever string is passed to `with_card_id`. Two battlefield
    // objects may safely share one card_id; each still gets its own ObjectId.
    let apprentice_p1 = enrich_spec_from_def(
        ObjectSpec::card(p1, "Loyal Apprentice")
            .with_card_id(cid("loyal-apprentice"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let apprentice_p2 = enrich_spec_from_def(
        ObjectSpec::card(p2, "Loyal Apprentice")
            .with_card_id(cid("loyal-apprentice"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let commander_p1 = ObjectSpec::creature(p1, "Test Commander APNAP P1", 3, 3)
        .with_card_id(p1_commander_cid.clone())
        .in_zone(ZoneId::Battlefield);
    let commander_p2 = ObjectSpec::creature(p2, "Test Commander APNAP P2", 3, 3)
        .with_card_id(p2_commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_commander(p1, p1_commander_cid)
        .player_commander(p2, p2_commander_cid)
        .object(apprentice_p1)
        .object(apprentice_p2)
        .object(commander_p1)
        .object(commander_p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);

    let triggered: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::AbilityTriggered {
                controller,
                source_object_id,
                ..
            } => Some((*controller, *source_object_id)),
            _ => None,
        })
        .collect();

    assert_eq!(
        triggered.len(),
        1,
        "CR 603.2: exactly ONE AtBeginningOfCombat trigger should have queued \
         (the active player p1's) -- p2's Loyal Apprentice is non-active and \
         `begin_combat`'s controller filter must reject it before it ever \
         reaches `state.pending_triggers`, not merely sort it after p1's in \
         APNAP order. Observed: {:?}",
        triggered
    );
    assert_eq!(
        triggered[0].0, p1,
        "the one queued trigger should be controlled by the active player p1"
    );

    let (state, _) = drain_stack(state, &[p1, p2, p3, p4]);
    let thopter_count = count_battlefield_tokens_named(&state, "Thopter");
    assert_eq!(
        thopter_count, 1,
        "only p1's Loyal Apprentice trigger should resolve, creating one Thopter"
    );
    let p1_thopter = state
        .objects()
        .values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Thopter")
        .expect("the one Thopter should exist");
    assert_eq!(
        p1_thopter.controller, p1,
        "the Thopter should be controlled by p1, not p2"
    );
}

// ── Test 6: emblem + card-def coexistence (CR 114.4, 603.3b) ───────────────

/// CR 114.4 (emblem abilities function from the command zone) + CR 603.3b
/// (multiple simultaneous triggers go on the stack as one APNAP batch): a
/// Basri Ket "-6" emblem (`basri_ket.rs:71-85`, `TriggerEvent::
/// AtBeginningOfCombat`) in p1's command zone, PLUS a battlefield
/// `helm_of_the_host` (`TriggerCondition::AtBeginningOfCombat`) -- two
/// DIFFERENT dispatch mechanisms matching the SAME step. Both must fire
/// exactly once each: no doubling, no drops. The two collections are disjoint
/// by zone (command zone vs. battlefield) and by enum
/// (`TriggerEvent` vs. `TriggerCondition`), so there is no code path that could
/// pick up one object's trigger under the other mechanism.
///
/// Also pins queue ordering (plan §4): the card-def sweep runs BEFORE the
/// emblem scan inside `begin_combat`, so the Helm's `AbilityTriggered` event
/// must appear before the emblem's in the event stream (both share controller
/// p1, so the APNAP stable-sort in `flush_pending_triggers` does not reorder
/// them -- this is a genuine queue-order assertion, not an APNAP-order one).
#[test]
fn test_emblem_and_carddef_combat_triggers_coexist() {
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
    let creature = ObjectSpec::creature(p1, "Coexist Equipped Creature", 2, 2);

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
    let creature_id = find_object(&state, "Coexist Equipped Creature");
    state.objects_mut().get_mut(&helm_id).unwrap().attached_to = Some(creature_id);
    state
        .objects_mut()
        .get_mut(&creature_id)
        .unwrap()
        .attachments
        .push_back(helm_id);

    // Basri Ket's -6 emblem (basri_ket.rs:73-106), created directly via the
    // real CreateEmblem effect execution rather than a hand-built GameObject,
    // so this test exercises the actual emblem-creation code path.
    execute_effect(
        &mut state,
        &Effect::CreateEmblem {
            triggered_abilities: vec![TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::AtBeginningOfCombat,
                intervening_if: None,
                description: "At the beginning of combat on your turn, create a 1/1 white \
                              Soldier creature token, then put a +1/+1 counter on each \
                              creature you control."
                    .to_string(),
                effect: Some(Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::White].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        count: EffectAmount::Fixed(1),
                        ..Default::default()
                    },
                }),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: vec![],
            }],
            static_effects: vec![],
            play_from_graveyard: None,
        },
        &mut EffectContext::new(p1, helm_id, vec![]),
    );
    let emblem_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.is_emblem)
        .map(|(id, _)| *id)
        .expect("emblem should have been created in p1's command zone");

    let (state, events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);

    let helm_triggered_idx = events.iter().position(|e| {
        matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == helm_id)
    });
    let emblem_triggered_idx = events.iter().position(|e| {
        matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == emblem_id)
    });
    assert!(
        helm_triggered_idx.is_some(),
        "the Helm's card-def AtBeginningOfCombat trigger should have queued -- \
         events: {:?}",
        events
    );
    assert!(
        emblem_triggered_idx.is_some(),
        "the Basri Ket emblem's AtBeginningOfCombat trigger should have queued \
         -- events: {:?}",
        events
    );
    assert!(
        helm_triggered_idx.unwrap() < emblem_triggered_idx.unwrap(),
        "CR 603.3b determinism (plan §4): the card-def sweep is ordered before \
         the emblem scan inside begin_combat, so the Helm's AbilityTriggered \
         event should precede the emblem's"
    );

    let (state, _) = drain_stack(state, &[p1, p2]);

    let helm_copy_count = count_battlefield_tokens_named(&state, "Coexist Equipped Creature");
    let soldier_count = count_battlefield_tokens_named(&state, "Soldier");
    assert_eq!(
        helm_copy_count, 1,
        "the Helm's trigger should fire exactly once -- no doubling, no drop"
    );
    assert_eq!(
        soldier_count, 1,
        "the emblem's trigger should fire exactly once -- no doubling, no drop"
    );
}

// ── Test 7: extra-combat behavior (CR 506.1, 603.2) ─────────────────────────

/// CR 506.1 (the phase structure defines "beginning of combat" as a step that
/// occurs once per combat PHASE, not once per turn) + CR 603.2 (a triggered
/// ability triggers on EVERY occurrence of its event -- nothing in the
/// Helm's `once_per_turn: false` def makes it once-per-turn). Extra combat
/// phases (CR 500.8) route back through `Step::BeginningOfCombat`
/// (`turn_structure.rs:55-63`), so the Helm's trigger must fire again in a
/// second combat phase within the same turn -- 2 tokens total.
///
/// This guards the general "once per turn" class of regression named in the
/// plan (§4/R4/R2): any change that makes the sweep skip on a repeat entry
/// into `BeginningOfCombat` -- whether by gating on `state.turn.in_extra_combat`,
/// by nesting inside the `state.combat.is_none()` guard, or by any other
/// per-turn dedup bookkeeping copied from a once-per-turn sibling step -- drops
/// to 1 token instead of 2. Verified directly: gating the sweep on
/// `!state.turn.in_extra_combat` (the R4 shape) makes this assertion fail with
/// `left: 1, right: 2`, exactly as expected; nesting the sweep inside
/// `state.combat.is_none()` (the R2 shape) does NOT independently reproduce a
/// failure in THIS harness, because `end_combat` (`turn_actions.rs:2285`)
/// unconditionally resets `state.combat = None` before the redirect into the
/// next `BeginningOfCombat`, so the guard is always true again at the second
/// entry regardless of nesting -- the guard-nesting risk is real in the
/// abstract (per the plan's own reasoning about `combat.rs:59-61`) but this
/// specific harness cannot force `state.combat` to be `Some` at that point, so
/// this test's actual falsifying power is over the "fires again" property in
/// general, verified via the R4 shape above.
#[test]
fn test_at_beginning_of_combat_fires_in_extra_combat_phase() {
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
    let creature = ObjectSpec::creature(p1, "Extra Combat Equipped Creature", 2, 2);

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
    let creature_id = find_object(&state, "Extra Combat Equipped Creature");
    state.objects_mut().get_mut(&helm_id).unwrap().attached_to = Some(creature_id);
    state
        .objects_mut()
        .get_mut(&creature_id)
        .unwrap()
        .attachments
        .push_back(helm_id);

    // First combat: real transition into BeginningOfCombat, resolve.
    let (state, _) = drive_to_step(state, Step::BeginningOfCombat);
    let (state, _) = drain_stack(state, &[p1, p2]);
    assert_eq!(
        count_battlefield_tokens_named(&state, "Extra Combat Equipped Creature"),
        1,
        "sanity check: the first combat's trigger should have created one token"
    );

    // No attackers declared -- CR 508.8 auto-skips DeclareBlockers/CombatDamage,
    // landing at EndOfCombat.
    let (mut state, _) = drive_to_step(state, Step::EndOfCombat);

    // Inject a second combat phase (CR 500.8), exactly as
    // `tests/combat/additional_combat.rs::test_additional_combat_phase_basic` does.
    state.turn_mut().additional_phases.push_back(Phase::Combat);

    // Second (extra) combat: EndOfCombat -> BeginningOfCombat again.
    let (state, _) = drive_to_step(state, Step::BeginningOfCombat);
    assert!(
        state.turn().in_extra_combat,
        "sanity check: this should be the extra combat phase"
    );
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        count_battlefield_tokens_named(&state, "Extra Combat Equipped Creature"),
        2,
        "CR 506.1/603.2: the Helm's AtBeginningOfCombat trigger should fire AGAIN \
         in the extra combat phase, creating a second token copy (2 total) -- if \
         only 1 is observed, the sweep is likely nested inside the \
         `state.combat.is_none()` guard and silently dropped the second combat's \
         trigger"
    );
}

// ── Test 8: negative edge -- Helm unattached (CR 603.2, 702.6) ─────────────

/// CR 603.2 (the trigger condition has no dependency on Equip attachment, so
/// it fires regardless) + CR 702.6 (an Equipment not attached to anything has
/// no "equipped creature"): Helm of the Host on the battlefield, UNATTACHED.
/// The trigger still fires (there is no intervening-if gating it on being
/// attached), but `EffectTarget::EquippedCreature` resolves to an empty
/// target list (`effects/mod.rs:6853-6863`), so `CreateTokenCopy` finds no
/// `source_id` and returns early (`effects/mod.rs:5275-5277`) -- zero tokens,
/// no panic, no diagnostic.
#[test]
fn test_helm_of_the_host_unattached_creates_no_token() {
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

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(helm)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Sanity: the Helm is on the battlefield but has nothing attached.
    let helm_id = find_object(&state, "Helm of the Host");
    assert_eq!(
        state.objects().get(&helm_id).unwrap().attached_to,
        None,
        "sanity check: the Helm should be unattached"
    );

    let (state, events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == helm_id)),
        "the trigger should still fire even though the Helm is unattached -- \
         there is no intervening-if gating it on attachment"
    );

    let (state, _) = drain_stack(state, &[p1, p2]);

    let token_count = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.is_token)
        .count();
    assert_eq!(
        token_count, 0,
        "CR 702.6: an unattached Equipment has no equipped creature -- \
         CreateTokenCopy should resolve to zero tokens, not panic, and not \
         create a token from some fallback source"
    );
}

// ── Test 9: goblin_rabblemaster end-to-end (PB-RS3 review Finding 3) ───────
//
// Nothing prior to this test drives the REAL `goblin_rabblemaster` card def
// through both of its load-bearing abilities at once.
// `pb_rs3_rabblemaster_mustattack_probe.rs` proves the AddKeyword/
// OtherCreaturesYouControlWithSubtype composition mechanism using a MOCK def,
// and the roster sweep (`tests/core/pb_rs3_combat_trigger_roster.rs`) only
// pins the `Completeness` MARKER, not behavior. Neither exercises the actual
// interaction the oracle text describes (and the 2014-07-18 ruling confirms):
// the token Rabblemaster's own `AtBeginningOfCombat` ability [1] creates is a
// Goblin, so Rabblemaster's own Static ability [0] ("Other Goblin creatures
// you control attack each combat if able") then forces THAT token to attack.

/// CR 506.1/603.2 (the token-creation trigger fires at beginning of combat) +
/// CR 508.1d (a creature with a granted "attacks each combat if able" keyword
/// must attack if able) + the 2014-07-18 Goblin Rabblemaster ruling ("the
/// token isn't affected by summoning sickness... it must attack because of
/// Rabblemaster's other ability"). Drives the real `goblin_rabblemaster`
/// card def (from `all_cards()`, not a mock) through the real `begin_combat`
/// sweep AND the real `combat.rs` must-attack enforcement, so both this PB's
/// mandatory-test class (real-def combat trigger, this file's Test 1-8
/// pattern) and the probe's mechanism (real must-attack enforcement, that
/// file's pattern) are proven together on the actual card rather than
/// separately on a mock.
#[test]
fn test_goblin_rabblemaster_end_to_end() {
    let p1 = p(1);
    let p2 = p(2);

    let all = all_cards();
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let rabblemaster = enrich_spec_from_def(
        ObjectSpec::card(p1, "Goblin Rabblemaster")
            .with_card_id(cid("goblin-rabblemaster"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(rabblemaster)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let rabblemaster_id = find_object(&state, "Goblin Rabblemaster");

    // GameStateBuilder does not replay ETB (same caveat as
    // pb_rs3_rabblemaster_mustattack_probe.rs and
    // pb_os4b_face_aware_abilities.rs) -- register the REAL "Goblin
    // Rabblemaster" registry entry's Static grant manually. Unlike the probe,
    // this looks up the actual card def by card_id rather than constructing a
    // mock, so this is the first test to confirm the real def's
    // `AbilityDefinition::Static` is picked up by
    // `register_static_continuous_effects`.
    let card_id = state.objects()[&rabblemaster_id].card_id.clone();
    register_static_continuous_effects(
        &mut state,
        rabblemaster_id,
        card_id.as_ref(),
        &registry,
        false,
    );

    // Real step transition: PreCombatMain -> BeginningOfCombat. This is the
    // sweep this whole PB added -- it must queue and flush Rabblemaster's
    // ability [1] (AtBeginningOfCombat -> CreateToken) onto the stack.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::BeginningOfCombat);

    let (state, _) = drain_stack(state, &[p1, p2]);

    let goblin_count = count_battlefield_tokens_named(&state, "Goblin");
    assert_eq!(
        goblin_count, 1,
        "Rabblemaster's own AtBeginningOfCombat trigger should create exactly \
         one 1/1 red Goblin token"
    );

    let goblin_id = find_object(&state, "Goblin");
    let goblin_obj = state
        .objects()
        .get(&goblin_id)
        .expect("the created Goblin token should exist");
    assert_eq!(
        goblin_obj.characteristics.power,
        Some(1),
        "oracle text: the created token is a 1/1"
    );
    assert_eq!(
        goblin_obj.characteristics.toughness,
        Some(1),
        "oracle text: the created token is a 1/1"
    );
    assert!(
        goblin_obj.characteristics.colors.contains(&Color::Red),
        "oracle text: the created token is red"
    );
    assert!(
        goblin_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "oracle text: the created token has haste"
    );

    // Advance to DeclareAttackers -- no attacks have been declared yet.
    let (state, _) = drive_to_step(state, Step::DeclareAttackers);

    // CR 508.1d: the token just created is an "other Goblin creature" that
    // p1 controls -- Rabblemaster's own Static ability [0] must force it to
    // attack (it has haste, so summoning sickness does not shield it, per
    // the 2014-07-18 ruling). Declaring NO attackers must be rejected.
    let empty_result = process_command(
        state.clone(),
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    );
    assert!(
        empty_result.is_err(),
        "CR 508.1d: Rabblemaster's own Goblin token must be forced to attack \
         by Rabblemaster's own MustAttackEachCombat static grant -- declaring \
         no attackers should be rejected: {:?}",
        empty_result.ok().map(|_| ())
    );

    // Positive control: declaring the forced Goblin as an attacker must
    // succeed.
    let ok_result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(goblin_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    );
    assert!(
        ok_result.is_ok(),
        "declaring the forced Goblin as an attacker must be legal: {:?}",
        ok_result.err()
    );
}
