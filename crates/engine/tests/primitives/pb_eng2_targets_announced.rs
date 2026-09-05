//! ENG-2 (`scutemob-193`) — targets in the event log: an announcement-time target
//! event, `GameEvent::TargetsAnnounced`.
//!
//! CR 601.2c / 602.2b / 603.3d: the targets chosen for a spell or ability are
//! declared as part of putting it on the stack. Before this batch no `GameEvent`
//! carried them (`OOS-G7-1`), so a bot's targeted trigger (the Fell Specter class)
//! rendered as `"A triggered ability of Fell Specter goes on the stack"` with no
//! target named — the defect a human playtester actually reported.
//!
//! `memory/primitives/pb-plan-ENG2.md` is authoritative. Tests below are its §7
//! (a), (b), (c), (e), (f), (i) plus the §4.3 machine-checkable gate. (d) — the
//! object-target-vs-player-target redaction proof — lives in
//! `crates/view-model/src/tests.rs`, because `event_view_for` is a
//! `mtg-view-model` API and that crate depends on `mtg-engine`, not the reverse;
//! an engine test cannot call it. (g) — the gate's own reverts — and (h) — the
//! play-server HTTP probe — are executed/written outside this file (§7(g) is
//! executed by hand and reported in the batch's completion notes; §7(h) lives in
//! `tools/play-server/src/main.rs`).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::hash::HashInto;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition,
    CardRegistry, Command, Completeness, GameEvent, GameState, GameStateBuilder, ManaPool,
    ObjectId, ObjectSpec, PlayerId, SpellTarget, Target, ZoneId,
};

// ── Shared helpers ──────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn registry() -> Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

/// Build an `ObjectSpec` from the **real** committed `CardDefinition` (mirrors
/// PB-DX19's `real_card_spec`), so the object carries the def's real abilities
/// (including its `targets`) rather than a hand-built stand-in.
fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
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

fn cast(player: PlayerId, card: ObjectId, targets: Vec<Target>) -> Command {
    Command::CastSpell(Box::new(mtg_engine::CastSpellData {
        player,
        card,
        targets,
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
    }))
}

// ── (a) A spell cast targeting a player ──────────────────────────────────────

/// CR 601.2c — Lightning Bolt cast at an opposing player announces that target.
/// **Proves**: the class the reported defect belongs to (a *player* target,
/// which `PermanentTargeted` cannot express at all — it only carries an
/// `ObjectId`).
///
/// **Red by revert** (executed by hand, restored): remove
/// `push_target_announcement` from `handle_cast_spell` (S1) -- the assertion
/// on `announced` below fails because the event never appears.
#[test]
fn test_eng2_spell_cast_targeting_a_player_announces() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(real_card_spec(
            p1,
            "Lightning Bolt",
            ZoneId::Hand(p1),
            &defs,
        ))
        .player_mana(
            p1,
            ManaPool {
                red: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let bolt = find_obj(&state, "Lightning Bolt");
    let (state, events) = process_command(state, cast(p1, bolt, vec![Target::Player(p2)]))
        .expect("casting Lightning Bolt at a player must succeed");

    let spell_idx = events
        .iter()
        .position(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1))
        .expect("SpellCast must be emitted");
    let announced = events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsAnnounced {
                controller,
                targets,
                ..
            } => Some((*controller, targets.clone())),
            _ => None,
        })
        .expect("TargetsAnnounced must be emitted for a targeted cast (CR 601.2c)");
    let targets_idx = events
        .iter()
        .position(|e| matches!(e, GameEvent::TargetsAnnounced { .. }))
        .unwrap();

    assert!(
        spell_idx < targets_idx,
        "TargetsAnnounced must follow its sibling SpellCast, not precede it"
    );
    assert_eq!(announced.0, p1, "CR 601.2f: the controller is the caster");
    assert_eq!(
        announced.1,
        vec![SpellTarget {
            target: Target::Player(p2),
            zone_at_cast: None,
        }],
        "the announced target must be exactly the declared player target"
    );

    let _ = state; // silence unused warning if the compiler ever elides the drop
}

// ── (b) An activated ability targeting a battlefield object ─────────────────

/// CR 602.2b — Rogue's Passage's `{4},{T}: target creature can't be blocked`
/// announces its target, and the announcement agrees with the (separately
/// emitted) `PermanentTargeted` Ward-dispatch event -- i.e. the display channel
/// does not contradict the Ward channel.
///
/// **Red by revert**: remove the helper call from `handle_activate_ability` (A1).
#[test]
fn test_eng2_activated_ability_targeting_a_permanent_announces() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(real_card_spec(
            p1,
            "Rogue's Passage",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .player_mana(
            p1,
            ManaPool {
                colorless: 4,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let passage = find_obj(&state, "Rogue's Passage");
    let creature = find_obj(&state, "Target Creature");

    let (_state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: passage,
            // Ability index 0 into `activated_abilities` -- NOT the same as
            // Rogue's Passage's oracle-text ordering. `ability_index` is
            // 0-indexed into non-mana `activated_abilities` only; the `{T}:
            // Add {C}` mana ability at oracle-text index 0 is filtered into
            // `mana_abilities` by `enrich_spec_from_def` and does not occupy
            // a non-mana slot, so the `{4},{T}: target creature can't be
            // blocked` ability is `activated_abilities[0]`.
            ability_index: 0,
            targets: vec![Target::Object(creature)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("activating Rogue's Passage's second ability must succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "AbilityActivated must be emitted"
    );
    let announced_targets = events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsAnnounced { targets, .. } => Some(targets.clone()),
            _ => None,
        })
        .expect("TargetsAnnounced must be emitted (CR 602.2b)");
    assert_eq!(
        announced_targets,
        vec![SpellTarget {
            target: Target::Object(creature),
            zone_at_cast: Some(ZoneId::Battlefield),
        }]
    );
    let permanent_targeted = events
        .iter()
        .find_map(|e| match e {
            GameEvent::PermanentTargeted { target_id, .. } => Some(*target_id),
            _ => None,
        })
        .expect("PermanentTargeted must still be emitted (CR 702.21a Ward dispatch)");
    assert_eq!(
        permanent_targeted, creature,
        "the display event and the Ward-dispatch event must agree on the target"
    );
}

// ── (c) A triggered ability targeting a player — the Fell Specter class ─────

/// CR 603.3d + CR 601.2c — Fell Specter's ETB ("target opponent discards a
/// card") announces its target once the CR 603.3d choice is answered with the
/// engine's own default.
///
/// **PB-DX48 INVERSION NOTE, replacing the original "DEVIATION PIN
/// (`OOS-ENG2-1`)" framing.** That framing was WRONG about what a fix could ever
/// change here, and it is corrected rather than silently updated: Fell Specter's
/// ETB targets `TargetRequirement::TargetOpponent` -- a PLAYER, not an object --
/// and `GameEvent::PermanentTargeted` carries only an `ObjectId`; there is no
/// value it could ever hold for a player target. So the assertion below is not a
/// deviation PB-DX48 closes, it is CR 702.21a's own SCOPE: Ward (and every other
/// `PermanentTargeted` consumer) can only ever fire off an OBJECT becoming a
/// target, and this test's negative case could never have flipped, at PB-DX48 or
/// any other batch, no matter how the dispatch mechanism changed. The positive
/// sibling immediately below --
/// `test_eng2_hyrax_tower_scout_etb_object_target_emits_permanent_targeted` --
/// is the discriminator the original pin was actually asking a successor to
/// produce: the SAME `flush_sorted` T7 site, with an OBJECT target instead of a
/// player one, now emits exactly one `PermanentTargeted` (it emitted none before
/// PB-DX48).
///
/// **Red by revert**: remove the helper call from `flush_sorted`'s main arm
/// (T7). Reverting only the `event_view` prose arm (a *different* red, not
/// exercised by this engine-level file) produces the bare-kind string
/// `"TargetsAnnounced"` -- covered in `crates/view-model/src/tests.rs`.
#[test]
fn test_eng2_fell_specter_etb_announces_its_target() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = defs_map();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry())
        .object(real_card_spec(p1, "Fell Specter", ZoneId::Hand(p1), &defs))
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                black: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let specter_in_hand = find_obj(&state, "Fell Specter");
    let mut state = state;
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = process_command(state, cast(p1, specter_in_hand, vec![]))
        .expect("casting Fell Specter must succeed");
    // Fell Specter is the only spell on the stack; one full pass round resolves
    // it (creature ETB), which reaches `flush_sorted`'s T7 site.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // CR 603.3d: three legal opponents (p2, p3, p4) is a real announcement --
    // the flush must have suspended rather than auto-placed the trigger.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TriggerTargetChoiceRequired { .. })),
        "CR 603.3d: Fell Specter's ETB has three legal opponents, so the flush \
         must suspend on a real choice"
    );
    let specter = find_obj(&state, "Fell Specter");
    let entry = state
        .pending_trigger_targets()
        .expect("no CR 603.3d trigger-target choice is pending")
        .clone();
    let default_target = entry.slots[0]
        .default
        .clone()
        .expect("TargetOpponent has a deterministic default (first live opponent)");

    let (state, resumed) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: entry.player,
            choice_id: entry.choice_id,
            targets: vec![vec![default_target.target.clone()]],
        },
    )
    .expect("the engine must accept its own default answer (SR-38)");

    assert!(
        resumed
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "the trigger must reach the stack once its target is announced"
    );
    let announced = resumed
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsAnnounced {
                controller,
                source_object_id,
                targets,
                ..
            } => Some((*controller, *source_object_id, targets.clone())),
            _ => None,
        })
        .expect("TargetsAnnounced must be emitted on the RESUMED flush (risk #7 of the plan)");
    assert_eq!(announced.0, p1);
    assert_eq!(announced.1, specter);
    assert_eq!(announced.2, vec![default_target.clone()]);

    assert!(
        !resumed
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentTargeted { .. })),
        "CR 702.21a SCOPE (not a deviation -- see the doc comment above): a PLAYER \
         target can never raise PermanentTargeted, which carries only an ObjectId. \
         This boolean cannot flip; the discriminator PB-DX48 needed is the sibling \
         test below, on an OBJECT target through the same T7 site."
    );

    let _ = state;
}

/// CR 702.21a's positive sibling to the test above, and the actual discriminator
/// `OOS-ENG2-1`'s original pin was asking a successor to produce (that pin's own
/// negative assertion, on a PLAYER target, could never flip -- see the corrected
/// doc comment on `test_eng2_fell_specter_etb_announces_its_target`).
///
/// Hyrax Tower Scout ("When this creature enters, untap target creature.") is a
/// real, deck-legal, effectively-`Complete` corpus def whose ETB targets an
/// OBJECT through the exact same `flush_sorted` T7 main arm Fell Specter's ETB
/// uses. Before PB-DX48, `flush_sorted` never emitted `PermanentTargeted` at all
/// (`OOS-ENG2-1` / `OOS-ENG2-2`); after it, an object-targeting triggered ability
/// at this site emits EXACTLY one.
///
/// **Red by revert**: remove `events.extend(permanent_targeted_events(..))` from
/// `rules/events.rs::push_target_announcement` (Part A) -- the announced-targets
/// assertions below stay green (they read `TargetsAnnounced`, untouched by that
/// revert) while the `PermanentTargeted` count silently drops to 0, which is
/// exactly the pre-PB-DX48 defect this test exists to catch.
#[test]
fn test_eng2_hyrax_tower_scout_etb_object_target_emits_permanent_targeted() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(real_card_spec(
            p1,
            "Hyrax Tower Scout",
            ZoneId::Hand(p1),
            &defs,
        ))
        .object(ObjectSpec::creature(p2, "Opp Target Creature", 2, 2).tapped())
        .player_mana(
            p1,
            ManaPool {
                colorless: 2,
                green: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let scout_in_hand = find_obj(&state, "Hyrax Tower Scout");
    let opp_creature = find_obj(&state, "Opp Target Creature");
    let mut state = state;
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = process_command(state, cast(p1, scout_in_hand, vec![]))
        .expect("casting Hyrax Tower Scout must succeed");
    // Hyrax Tower Scout is the only spell on the stack; one full pass round
    // resolves it (creature ETB), reaching flush_sorted's T7 site.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 601.2c: "target creature" with no filter offers >= 2 candidates here
    // (Hyrax Tower Scout itself and the opponent's creature), so the flush must
    // suspend on a real choice rather than auto-placing the trigger.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TriggerTargetChoiceRequired { .. })),
        "CR 603.3d: at least two legal creature candidates must force a real \
         announcement, not an auto-placed default"
    );
    let scout = find_obj(&state, "Hyrax Tower Scout");
    let entry = state
        .pending_trigger_targets()
        .expect("no CR 603.3d trigger-target choice is pending")
        .clone();
    // Choose the OPPONENT's creature explicitly (not Hyrax Tower Scout's own
    // default), so this test discriminates "the announced target is an object on
    // the battlefield" from "whatever the engine happened to default to".
    let chosen = entry.slots[0]
        .candidates
        .iter()
        .find(|c| c.target == Target::Object(opp_creature))
        .cloned()
        .expect("the opponent's creature must be a legal candidate");

    let (state, resumed) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: entry.player,
            choice_id: entry.choice_id,
            targets: vec![vec![chosen.target.clone()]],
        },
    )
    .expect("choosing the opponent's creature must be accepted");

    let announced = resumed
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsAnnounced {
                controller,
                source_object_id,
                targets,
                ..
            } => Some((*controller, *source_object_id, targets.clone())),
            _ => None,
        })
        .expect("TargetsAnnounced must be emitted on the RESUMED flush");
    assert_eq!(announced.0, p1);
    assert_eq!(announced.1, scout);
    assert_eq!(announced.2, vec![chosen.clone()]);

    let permanent_targeted_count = resumed
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::PermanentTargeted { target_id, .. } if *target_id == opp_creature
            )
        })
        .count();
    assert_eq!(
        permanent_targeted_count, 1,
        "CR 702.21a: flush_sorted's T7 arm must emit EXACTLY ONE PermanentTargeted \
         for an object-targeting triggered ability, the count PB-DX48 exists to \
         make true (not merely >= 1 -- the batch's own headline finding is a \
         double-dispatch defect a bare presence check would miss)"
    );

    let _ = state;
}

/// The PB-DP8 human-answer path, with a **non-default** answer: a 4-player
/// state gives Fell Specter's `TargetOpponent` slot >= 2 candidates, so this
/// discriminates "the engine announced" from "the engine announced its own
/// default" -- without it, `test_eng2_fell_specter_etb_announces_its_target`
/// alone cannot tell the two apart.
#[test]
fn test_eng2_fell_specter_etb_announces_a_non_default_human_choice() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = defs_map();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry())
        .object(real_card_spec(p1, "Fell Specter", ZoneId::Hand(p1), &defs))
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                black: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let specter_in_hand = find_obj(&state, "Fell Specter");
    let mut state = state;
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = process_command(state, cast(p1, specter_in_hand, vec![]))
        .expect("casting Fell Specter must succeed");
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let entry = state
        .pending_trigger_targets()
        .expect("no CR 603.3d trigger-target choice is pending")
        .clone();
    let default_target = entry.slots[0]
        .default
        .clone()
        .expect("TargetOpponent has a deterministic default");
    let candidates: Vec<SpellTarget> = entry.slots[0].candidates.clone();
    assert!(
        candidates.len() >= 2,
        "a 4-player state must offer >= 2 opponent candidates, or this test \
         cannot pick a non-default one"
    );
    let non_default = candidates
        .iter()
        .find(|c| **c != default_target)
        .expect("at least one candidate must differ from the default")
        .clone();

    let (_state, resumed) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: entry.player,
            choice_id: entry.choice_id,
            targets: vec![vec![non_default.target.clone()]],
        },
    )
    .expect("the engine must accept a legal non-default answer");

    let announced_targets = resumed
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsAnnounced { targets, .. } => Some(targets.clone()),
            _ => None,
        })
        .expect("TargetsAnnounced must be emitted");
    assert_eq!(
        announced_targets,
        vec![non_default.clone()],
        "the announcement must name the seat the human picked, not the engine's default"
    );
    assert_ne!(
        announced_targets,
        vec![default_target],
        "this test is vacuous unless the picked answer differs from the default"
    );
}

// ── (e) A non-targeting announcement emits nothing ───────────────────────────

/// CR 115.1 makes targeting optional; a spell with no targets must announce
/// nothing. Without this test (and its activation twin below) the "emitted
/// only when non-empty" clause in `GameEvent::TargetsAnnounced`'s doc comment
/// is just a comment.
#[test]
fn test_eng2_a_nontargeting_cast_announces_nothing() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Vanilla Bear", 2, 2).in_zone(ZoneId::Hand(p1)))
        .build()
        .expect("GameStateBuilder::build must succeed");

    // A vanilla creature spec (naked, no CardDefinition/registry entry) has an
    // empty AbilityDefinition list -- no targets requested regardless.
    let bear = find_obj(&state, "Vanilla Bear");
    state.turn_mut().priority_holder = Some(p1);

    let (_state, events) = process_command(state, cast(p1, bear, vec![]))
        .expect("casting a non-targeting creature spell must succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "precondition: the cast itself must succeed"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsAnnounced { .. })),
        "CR 115.1: a non-targeting cast must announce nothing"
    );
}

/// The activation twin: Rogue's Passage's mana ability (index 0) is `{T}: Add
/// {C}` and never uses the stack (CR 605.3), so it emits neither
/// `AbilityActivated` nor `TargetsAnnounced`. Control on the "the ability
/// itself never uses the stack" half; the primary claim is the absence of the
/// announcement.
#[test]
fn test_eng2_a_nontargeting_activation_announces_nothing() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(real_card_spec(
            p1,
            "Rogue's Passage",
            ZoneId::Battlefield,
            &defs,
        ))
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let passage = find_obj(&state, "Rogue's Passage");
    let (_state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: passage,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("tapping Rogue's Passage for mana must succeed");

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { .. })),
        "CR 605.3: a mana ability does not use the stack -- no AbilityActivated"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsAnnounced { .. })),
        "a mana ability has no targets to announce"
    );
}

// ── (f) The hash arm's own bytes ─────────────────────────────────────────────

/// Direct `HashInto` unit test for `GameEvent::TargetsAnnounced`, following the
/// established precedent in `primitive_pb_oos_lki_power_3.rs`. There is no way
/// to route this arm's bytes through `stream_fingerprint`
/// (`GameEvent` reaches the hash stream only via
/// `PendingTrigger.triggering_event`, and a `TargetsAnnounced` is never a
/// trigger's triggering event -- exact precedent HASH v58 / PB-OS6), so this
/// direct test is the ONLY place the arm's bytes are proven, and the batch's
/// `- N:` HASH history line must say so.
#[test]
fn test_eng2_targets_announced_hashes_its_targets() {
    fn hash_event(ev: &GameEvent) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        ev.hash_into(&mut h);
        *h.finalize().as_bytes()
    }

    let controller = p(1);
    let source = ObjectId(100);
    let stack_id = ObjectId(101);

    let empty = GameEvent::TargetsAnnounced {
        controller,
        source_object_id: source,
        stack_object_id: stack_id,
        targets: vec![],
    };
    let player1 = GameEvent::TargetsAnnounced {
        controller,
        source_object_id: source,
        stack_object_id: stack_id,
        targets: vec![SpellTarget {
            target: Target::Player(p(1)),
            zone_at_cast: None,
        }],
    };
    let player2 = GameEvent::TargetsAnnounced {
        controller,
        source_object_id: source,
        stack_object_id: stack_id,
        targets: vec![SpellTarget {
            target: Target::Player(p(2)),
            zone_at_cast: None,
        }],
    };
    let object1 = GameEvent::TargetsAnnounced {
        controller,
        source_object_id: source,
        stack_object_id: stack_id,
        targets: vec![SpellTarget {
            target: Target::Object(ObjectId(200)),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    };

    let hashes = [
        hash_event(&empty),
        hash_event(&player1),
        hash_event(&player2),
        hash_event(&object1),
    ];
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i], hashes[j],
                "TargetsAnnounced values #{i} and #{j} differ only in `targets` but \
                 hashed identically"
            );
        }
    }
}

// ── (i) Roster / non-vacuity gate ────────────────────────────────────────────

/// CR 601.2c / SR-36 -- how many `Completeness::Complete` defs the CR 601.2c
/// announcement reaches, derived by enumerating `all_cards()` (never by
/// grepping source). A `>=` floor, not `==`: the authoring campaign grows
/// continuously and an exact pin would redden on unrelated work.
#[test]
fn test_eng2_announcement_roster() {
    fn has_targeted_ability(abilities: &[AbilityDefinition]) -> bool {
        abilities.iter().any(|a| match a {
            AbilityDefinition::Spell { targets, .. } => !targets.is_empty(),
            AbilityDefinition::Activated { targets, .. } => !targets.is_empty(),
            AbilityDefinition::Triggered { targets, .. } => !targets.is_empty(),
            _ => false,
        })
    }

    let mut roster: Vec<String> = Vec::new();
    let mut incomplete = 0usize;
    for def in all_cards() {
        let mut hit = has_targeted_ability(&def.abilities);
        if let Some(face) = def.back_face.as_ref() {
            hit |= has_targeted_ability(&face.abilities);
        }
        if let Some(face) = def.adventure_face.as_ref() {
            hit |= has_targeted_ability(&face.abilities);
        }
        if !hit {
            continue;
        }
        if def.completeness == Completeness::Complete {
            roster.push(def.name.clone());
        } else {
            incomplete += 1;
        }
    }
    roster.sort();
    println!(
        "ENG-2 announcement roster: {} effectively-Complete defs carry a targeted \
         Spell/Activated/Triggered ability ({} more carry a non-Complete marker)",
        roster.len(),
        incomplete
    );
    for name in &roster {
        println!("  {name}");
    }
    assert!(
        roster.len() >= 200,
        "ENG-2 announcement roster collapsed to {} defs (expected >= 200); measured \
         well above this floor at implementation time -- if it dropped, either the \
         corpus regressed or this scan stopped matching real defs",
        roster.len()
    );
}

// ── The stack-push announcement variant set (`OOS-DX28-1`) ───────────────────

/// The three `GameEvent` variants that mark a NEW object arriving on the stack, and whose
/// announcement sites `every_announcement_site_is_classified` therefore has to classify.
///
/// Lifted out of that test's body by PB-DX57 so it can be pinned. It used to be a fn-body
/// `const` that nothing compared to `pub enum GameEvent`'s declaration: a fourth stack-push
/// variant would never have been scanned, its announcement sites would never have been
/// classified, and the "closed set" claim in that test's own name would have become false in
/// silence. `stack_push_variants_are_classified_against_the_declaration` closes that.
const STACK_PUSH_ANNOUNCEMENT_VARIANTS: &[&str] =
    &["SpellCast", "AbilityActivated", "AbilityTriggered"];

/// `GameEvent` variants that carry a `stack_object_id` and are NOT a stack push, each with
/// the reason it is not one. Together with [`STACK_PUSH_ANNOUNCEMENT_VARIANTS`] this must be
/// the WHOLE `stack_object_id`-carrying population — see
/// `stack_push_variants_are_classified_against_the_declaration`.
const NON_PUSH_STACK_OBJECT_EVENTS: &[(&str, &str)] = &[
    (
        "SpellResolved",
        "CR 608.2n -- a DEPARTURE from the stack. Nothing is announced; the targets were \
         announced at the matching SpellCast.",
    ),
    (
        "SpellCountered",
        "CR 701.5a -- a departure. The countered spell's targets were announced at its cast.",
    ),
    (
        "SpellFizzled",
        "CR 608.2b -- a departure (every target illegal). Announcement already happened.",
    ),
    (
        "AbilityResolved",
        "CR 608.2n / 113.7a -- a departure, the ability ceasing to exist.",
    ),
    (
        "TargetsAnnounced",
        "ENG-2's own event. It is emitted ALONGSIDE a push, by \
         `rules::events::push_target_announcement`, and reports the targets that push \
         announced -- so it is the announcement's PAYLOAD, not a second push. Scanning for \
         it here would double-count every site this test classifies.",
    ),
    (
        "TargetsChanged",
        "CR 115.7 -- a retarget of an object ALREADY on the stack (`Effect::ChangeTargets`, \
         PB-DX25c). No new stack object arrives, so there is no announcement site.",
    ),
];

/// Split on top-level `,`, counting `{}`, `()`, `[]` and generic `<>` so a payload's own
/// commas are not boundaries. `pb_dx42a`'s `t7` recorded its own first draft anchoring on the
/// nearest `}` and "landing INSIDE the pattern list and silently returning three of the eight".
fn top_level_chunks(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut prev = ' ';
    for ch in body.chars() {
        match ch {
            '{' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            '<' if prev.is_ascii_alphanumeric() || prev == '_' => {
                depth += 1;
                cur.push(ch);
            }
            '>' if depth > 0 && prev != '-' && prev != '=' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
        prev = ch;
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

/// Remove leading `#[...]` attributes. Load-bearing: a `#[serde(default)]` sitting between a
/// field's doc comment and the field itself makes a naive parser read `#` as the field's
/// first character and drop the field entirely.
fn strip_leading_attributes(mut s: &str) -> &str {
    loop {
        s = s.trim_start();
        if !s.starts_with("#[") {
            return s;
        }
        let mut d = 0usize;
        let mut end = None;
        for (i, ch) in s.char_indices() {
            match ch {
                '[' => d += 1,
                ']' => {
                    d -= 1;
                    if d == 0 {
                        end = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => s = &s[e..],
            None => return s,
        }
    }
}

fn leading_identifier(chunk: &str) -> String {
    strip_leading_attributes(chunk)
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

/// Every variant of `pub enum <enum_name>` in a workspace-relative file, paired with the set
/// of field names its struct-like payload declares (empty for unit and tuple variants).
///
/// # This is a COPY, and the canonical version is named
///
/// The canonical implementation is
/// `crates/engine/tests/core/pb_dx57_declared_source.rs::declared_enum_variant_fields`. It
/// cannot be shared: `core` and `primitives` are separate test BINARIES, and
/// `tests/no_stray_test_binaries.rs::group_main_rs_declares_modules_and_nothing_else` allows
/// a group's `main.rs` to contain bare `mod x;` lines and nothing else, so neither a `use`
/// nor a `#[path]` re-export is available. `primitives/pb_dp9_effect_choice.rs:2641` settled
/// what to do about exactly this situation: keep the copy, say it is a copy, name the
/// canonical version, and cross-check BY VALUE rather than by text. The cross-check is in
/// the test below.
///
/// # Bounds, stated rather than left to be discovered
///
/// Strips `//` line comments only, then ASSERTS that the file carries no `/* */` block
/// comment (PB-DX8's `OOS-DX32-6` defeat: the byte-identical sentence reddened as a line
/// comment and left every test green as a block comment). Panics on an empty parse -- a
/// parser that returns `{}` makes every `assert_eq!` against it trivially true, which is
/// `OOS-DX28-1`'s own failure mode re-entering through its fix.
fn declared_enum_variant_fields(
    rel: &str,
    enum_name: &str,
) -> std::collections::BTreeMap<String, std::collections::BTreeSet<String>> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf();
    let path = root.join(rel);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", path.display()));
    assert!(
        !raw.contains("/*"),
        "{} grew a `/* */` block comment; this parser strips `//` only, so a block comment \
         can hide or fake a variant (`OOS-DX32-6`). Widen it, or use the canonical \
         `core::pb_dx57_declared_source::declared_enum_variant_fields`, which handles both.",
        path.display()
    );
    let clean: String = raw
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => format!("{}{}", &l[..i], " ".repeat(l.len() - i)),
            None => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n");

    let header = format!("pub enum {enum_name} {{");
    let at = clean.find(&header).unwrap_or_else(|| {
        panic!(
            "`{header}` not found in {}. The declaration was renamed or moved -- re-point \
             this pin rather than deleting it and keeping the hand-written list, which is \
             the defect `OOS-DX28-1` names.",
            path.display()
        )
    });
    let body_start = clean[at..].find('{').expect("enum has a body") + at + 1;
    let mut depth = 1usize;
    let mut end = None;
    for (i, ch) in clean[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(body_start + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &clean[body_start..end.expect("the enum body is never closed")];

    let mut out: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for chunk in top_level_chunks(body) {
        let name = leading_identifier(&chunk);
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            continue;
        }
        let fields: std::collections::BTreeSet<String> = match (chunk.find('{'), chunk.rfind('}')) {
            (Some(a), Some(b)) if b > a => top_level_chunks(&chunk[a + 1..b])
                .into_iter()
                .filter_map(|f| {
                    let f = strip_leading_attributes(f.trim());
                    let f = f.strip_prefix("pub ").unwrap_or(f).trim_start();
                    let ident: String = f
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    (!ident.is_empty()
                        && f[ident.len()..].trim_start().starts_with(':')
                        && ident.starts_with(|c: char| c.is_ascii_lowercase() || c == '_'))
                    .then_some(ident)
                })
                .collect(),
            _ => std::collections::BTreeSet::new(),
        };
        out.insert(name, fields);
    }
    assert!(
        !out.is_empty(),
        "parsed ZERO variants out of `{header}` in {}",
        path.display()
    );
    out
}

#[test]
/// `OOS-DX28-1` -- **the stack-push announcement variant set is classified against
/// `pub enum GameEvent`'s own declaration, not hand-listed.**
///
/// `STACK_PUSH_ANNOUNCEMENT_VARIANTS` is what `every_announcement_site_is_classified` scans
/// for. Until PB-DX57 nothing compared it to anything: a fourth stack-push event variant
/// would simply not have been scanned, its announcement sites would never have been
/// classified, and the closed-set claim in that test's own name would have gone quietly
/// false. That is `TARGET_FILTER_FIELDS`' failure one enum over -- a hand-maintained
/// fingerprint that goes blind on declaration growth with no compile error and no failure
/// message pointing anywhere near the cause.
///
/// ## Why this is a PARTITION and not a subset check
///
/// A bare `push ⊆ declared` catches a RENAME -- worth having, because a renamed variant
/// makes the scan match nothing while staying green -- and says nothing about GROWTH, which
/// is the direction the seed is about.
///
/// The semantic set (*"the events that mark a new object arriving on the stack"*) is not
/// derivable from a name, but it has a **structural necessary condition**: such an event has
/// to identify the stack object it created, i.e. carry a `stack_object_id` field. So the
/// candidate POOL is derivable, and this row requires every member of it to be classified
/// either as a push or as a non-push **with a stated reason** --
/// `core::pb_dx50_copy_additional_cost_roster::r1`'s partition shape, which the stage-0
/// census calls the cleanest instance of this repair in the tree. A tenth
/// `stack_object_id`-carrying variant is then a red test whose message names the choice its
/// author has to make, rather than a silent omission.
///
/// **"In silence" is precise, not rhetorical.** Planting a new `stack_object_id`-carrying
/// `GameEvent` variant was EXECUTED: it reddens the two WIRE gates
/// (`core::hash_schema::declaration_fingerprint_is_pinned`,
/// `core::protocol_schema::protocol_schema_fingerprint_is_pinned`) and, in the `primitives`
/// binary, **nothing but this row**. The wire gates say *"the wire moved"*, which is answered
/// by bumping a version number; they say nothing about whether the announcement census still
/// covers the enum.
///
/// **Stated bound.** The pool is a NECESSARY condition, not a sufficient one. A stack push
/// that somehow did not report its stack object id would sit outside it -- but such an event
/// could not be classified by `every_announcement_site_is_classified` either (that test keys
/// its sites on the `GameEvent::<Variant> {` construction and then asks which stack object
/// was announced), and nothing downstream could consume it.
///
/// ## The by-value cross-check with the canonical parser
///
/// `declared_enum_variant_fields` above is a copy; see its doc for why it cannot be shared.
/// The canonical parser lives in the `core` binary and is exercised there on this same file
/// by `core::pb_dx48_announcement_site_roster::r2d_permanent_targeted_fields_match_the_declaration`,
/// which asserts that `GameEvent::PermanentTargeted` declares exactly
/// `{target_id, targeting_stack_id, targeting_controller}`. This row asserts the same three
/// names out of its own parse. Two derivations that agree only with themselves are worth
/// nothing; if these two ever disagree about `events.rs`, one moves and the other does not.
fn stack_push_variants_are_classified_against_the_declaration() {
    let declared = declared_enum_variant_fields("crates/engine/src/rules/events.rs", "GameEvent");

    // Non-vacuity: a FLOOR, not a pin. `GameEvent`'s exact size is not this row's business;
    // a parse that collapsed to a handful would make everything below trivially satisfiable.
    assert!(
        declared.len() >= 100,
        "the GameEvent parse returned only {} variants -- the parser is broken, and every \
         assertion below would be vacuous",
        declared.len()
    );

    // ── The by-value cross-check with the canonical parser. See the doc.
    let permanent_targeted: Vec<&str> = declared
        .get("PermanentTargeted")
        .map(|f| f.iter().map(String::as_str).collect())
        .unwrap_or_default();
    assert_eq!(
        permanent_targeted,
        vec!["target_id", "targeting_controller", "targeting_stack_id"],
        "this file's COPY of the declaration parser reads GameEvent::PermanentTargeted's \
         payload as {permanent_targeted:?}; the canonical parser, exercised on the same file \
         by core::pb_dx48_announcement_site_roster::\
         r2d_permanent_targeted_fields_match_the_declaration, reads target_id / \
         targeting_stack_id / targeting_controller. One of the two parsers is wrong -- do \
         not reconcile by editing whichever is easier to change."
    );

    // ── The partition.
    let pool: std::collections::BTreeSet<&str> = declared
        .iter()
        .filter(|(_, fields)| fields.contains("stack_object_id"))
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        pool.len() >= 5,
        "only {} GameEvent variant(s) carry a `stack_object_id` -- the field was renamed, \
         and this row's candidate pool has silently collapsed to something every hand-written \
         list trivially covers",
        pool.len()
    );

    let push: std::collections::BTreeSet<&str> =
        STACK_PUSH_ANNOUNCEMENT_VARIANTS.iter().copied().collect();
    let non_push: std::collections::BTreeSet<&str> = NON_PUSH_STACK_OBJECT_EVENTS
        .iter()
        .map(|(n, _)| *n)
        .collect();

    let overlap: Vec<&&str> = push.intersection(&non_push).collect();
    assert!(
        overlap.is_empty(),
        "a GameEvent variant is classified BOTH as a stack push and as a non-push: {overlap:?}"
    );

    let classified: std::collections::BTreeSet<&str> = push.union(&non_push).copied().collect();
    assert_eq!(
        classified,
        pool,
        "`OOS-DX28-1`: the `stack_object_id`-carrying GameEvent variants are no longer \
         exactly the classified set.\n  UNCLASSIFIED (declared in `pub enum GameEvent`, in \
         neither list): {:?} -- decide whether each marks a NEW object arriving on the \
         stack. If it does, add it to STACK_PUSH_ANNOUNCEMENT_VARIANTS and classify its \
         emission sites in `every_announcement_site_is_classified`; if it does not, add it \
         to NON_PUSH_STACK_OBJECT_EVENTS with the reason.\n  DEAD (classified here, absent \
         from the declaration -- i.e. a rename, which would make the scan match nothing \
         while staying green): {:?}",
        pool.difference(&classified).collect::<Vec<_>>(),
        classified.difference(&pool).collect::<Vec<_>>()
    );

    // Every non-push entry carries a real reason, so the exclusion list cannot rot into a
    // bare name list: an allowlist whose reason is not checked is a comment.
    for (name, why) in NON_PUSH_STACK_OBJECT_EVENTS {
        assert!(
            why.len() > 30,
            "NON_PUSH_STACK_OBJECT_EVENTS entry `{name}` carries no real reason: {why:?}"
        );
    }
}
// ── The class gate ────────────────────────────────────────────────────────────

/// CR 601.2c / 602.2b / 603.3d -- a source gate over the batch's whole
/// correctness surface, because a convention ("call the helper at every
/// targeted announcement site") is not a guard. Precedent:
/// `pb_dx19_characteristics_recursion.rs::no_condition_evaluator_resolves_characteristics_directly`
/// (a brace-matched source walk).
///
/// Four parts, matching plan §4.3:
/// 1. the site census (a new/moved/deleted emission site in a known file);
/// 2. every `ANNOUNCES` site calls the helper;
/// 3. every `NEVER_TARGETS` site has not quietly grown targets;
/// 4. the five-file set is closed (a new emission site in a SIXTH file).
///
/// **Deviation from the plan's literal Part 3 wording, recorded here rather
/// than silently taken.** The plan says "assert its enclosing function body
/// contains none of [the target-setting patterns]". `casting.rs::handle_cast_spell`
/// is ONE giant function containing both S1 (`ANNOUNCES`, which legitimately
/// sets `targets: spell_targets` on ITS OWN `StackObject`) and T1-T5
/// (`NEVER_TARGETS`, the Storm/Gravestorm/Cascade/Casualty/Replicate trigger
/// pushes). A whole-function-body scan would find S1's own `targets:
/// spell_targets` literal and incorrectly flag every `NEVER_TARGETS` site in
/// that function. Part 3 below instead scopes each occurrence to the LOCAL
/// WINDOW between the previous occurrence's `events.push(` line (exclusive,
/// or the function's own opening line for the first occurrence) and this
/// occurrence's `events.push(` line (inclusive) -- the code uniquely
/// associated with that one push, which is exactly what "has this site
/// quietly grown targets" needs to mean once two occurrences share a function.
#[test]
fn every_announcement_site_is_classified() {
    // The five files §3.4 pins as the closed set, read once.
    const FILES: &[&str] = &[
        "rules/casting.rs",
        "rules/abilities.rs",
        "rules/copy.rs",
        "rules/resolution.rs",
        "rules/engine.rs",
    ];

    struct Hit {
        line_no: usize,
        variant: &'static str,
        func: String,
    }

    fn read_src(rel: &str) -> String {
        let path = format!("{}/src/{}", env!("CARGO_MANIFEST_DIR"), rel);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("must be able to read {path}: {e}"))
    }

    /// Scan comment-stripped lines of `src` for `events.push(` co-occurring with
    /// one of `GameEvent::<Variant> {` (with or without the `crate::rules::events::`
    /// prefix) on the SAME line, and for each hit, walk backward to the nearest
    /// column-0 function header to name its enclosing function.
    fn scan(src: &str) -> Vec<Hit> {
        let lines: Vec<&str> = src.lines().collect();
        // Pre-index every column-0 `fn` header's line number and name, so the
        // backward walk is a simple scan rather than re-parsing repeatedly.
        let mut fn_headers: Vec<(usize, String)> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed_start = line.trim_start();
            if line.len() == trimmed_start.len() // column 0, no leading whitespace
                && (trimmed_start.starts_with("fn ")
                    || trimmed_start.starts_with("pub fn ")
                    || trimmed_start.starts_with("pub(crate) fn ")
                    || trimmed_start.starts_with("async fn "))
            {
                let after_fn = trimmed_start
                    .split_once("fn ")
                    .map(|(_, rest)| rest)
                    .unwrap_or(trimmed_start);
                let name: String = after_fn
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                fn_headers.push((i, name));
            }
        }

        let mut hits = Vec::new();
        for (i, raw_line) in lines.iter().enumerate() {
            let code = raw_line.split("//").next().unwrap_or("");
            if !code.contains("events.push(") {
                continue;
            }
            for variant in STACK_PUSH_ANNOUNCEMENT_VARIANTS {
                let needle = format!("GameEvent::{variant} {{");
                if code.contains(&needle) {
                    // Walk backward for the nearest preceding fn header.
                    let func = fn_headers
                        .iter()
                        .rev()
                        .find(|(ln, _)| *ln <= i)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_else(|| {
                            panic!(
                                "no enclosing fn found for line {} (variant {variant})",
                                i + 1
                            )
                        });
                    hits.push(Hit {
                        line_no: i + 1, // 1-indexed, for human-readable failure text
                        variant,
                        func,
                    });
                }
            }
        }
        hits
    }

    // ── Part 1: the site census ──────────────────────────────────────────────

    let mut all_keys: Vec<String> = Vec::new();
    // Per (file, hit) classification and window bounds, keyed the same way, for
    // Parts 2 and 3.
    struct Classified {
        key: String,
        file: &'static str,
        func: String,
        line_no: usize,
    }
    let mut classified: Vec<Classified> = Vec::new();

    for &file in FILES {
        let src = read_src(file);
        let mut hits = scan(&src);
        hits.sort_by_key(|h| h.line_no);
        // Assign occurrence index PER (file, func), in ascending line order.
        let mut per_func_counter: HashMap<String, usize> = HashMap::new();
        for h in hits {
            let n = per_func_counter.entry(h.func.clone()).or_insert(0);
            let key = format!("{file}::{}#{n} -> {}", h.func, h.variant);
            *n += 1;
            all_keys.push(key.clone());
            classified.push(Classified {
                key,
                file,
                func: h.func,
                line_no: h.line_no,
            });
        }
    }

    // Non-vacuity: the raw scan found the expected minimum before any
    // classification is applied.
    assert!(
        all_keys.len() >= 26,
        "raw scan found only {} announcement-shaped sites; expected >= 26 (§3.4's \
         corrected roll-up). If this dropped, the scan pattern stopped matching \
         real emission sites.",
        all_keys.len()
    );

    all_keys.sort();

    // The pinned census. `true` = ANNOUNCES (calls push_target_announcement),
    // `false` = NEVER_TARGETS (structurally target-free today). Every entry
    // carries the site's short name and the reason, matching plan §4.3's
    // "one line each, each carrying an inline reason" requirement.
    const EXPECTED_SITES: &[(&str, bool, &str)] = &[
        ("rules/casting.rs::handle_cast_spell#0 -> SpellCast", true,
            "S1: spell_targets from validate_targets_* (CR 601.2c)"),
        ("rules/casting.rs::handle_cast_spell#1 -> AbilityTriggered", false,
            "T1: Storm copy trigger, trigger_default targets:vec![]"),
        ("rules/casting.rs::handle_cast_spell#2 -> AbilityTriggered", false,
            "T2: Gravestorm copy trigger, trigger_default targets:vec![]"),
        ("rules/casting.rs::handle_cast_spell#3 -> AbilityTriggered", false,
            "T3: Cascade trigger, trigger_default targets:vec![]"),
        ("rules/casting.rs::handle_cast_spell#4 -> AbilityTriggered", false,
            "T4: Casualty trigger, trigger_default targets:vec![]"),
        ("rules/casting.rs::handle_cast_spell#5 -> AbilityTriggered", false,
            "T5: Replicate trigger, trigger_default targets:vec![]"),
        ("rules/copy.rs::resolve_cascade#0 -> SpellCast", true,
            "S2: cascade free-cast, targets:vec![] today (OOS-ENG2-3), wired for correctness-under-change"),
        ("rules/copy.rs::resolve_discover#0 -> SpellCast", true,
            "S3: discover free-cast, targets:vec![] today (OOS-ENG2-3), wired for correctness-under-change"),
        ("rules/resolution.rs::resolve_top_of_stack_inner#0 -> SpellCast", true,
            "S4: cipher-copy, targets:vec![] today (OOS-ENG2-3), wired for correctness-under-change"),
        ("rules/resolution.rs::resolve_top_of_stack_inner#1 -> SpellCast", true,
            "S5: suspend free-cast, targets:vec![] today (OOS-ENG2-3), wired for correctness-under-change"),
        ("rules/abilities.rs::handle_activate_ability#0 -> AbilityActivated", true,
            "A1: spell_targets moved into stack_obj.targets (CR 602.2b)"),
        ("rules/abilities.rs::handle_cycle_card#0 -> AbilityActivated", false,
            "A2: cycling has no target (CR 702.29a)"),
        ("rules/abilities.rs::handle_activate_forecast#0 -> AbilityActivated", true,
            "A3: stack_obj.targets = spell_targets"),
        ("rules/abilities.rs::handle_activate_bloodrush#0 -> AbilityActivated", true,
            "A4: single unfiltered target"),
        ("rules/abilities.rs::handle_unearth_card#0 -> AbilityActivated", false,
            "A5: unearth has no target"),
        ("rules/abilities.rs::handle_ninjutsu#0 -> AbilityActivated", false,
            "A6: ninjutsu has no target (inherits attack target, not a declared one)"),
        ("rules/abilities.rs::handle_embalm_card#0 -> AbilityActivated", false,
            "A7: embalm has no target"),
        ("rules/abilities.rs::handle_eternalize_card#0 -> AbilityActivated", false,
            "A8: eternalize has no target"),
        ("rules/abilities.rs::handle_encore_card#0 -> AbilityActivated", false,
            "A9: encore has no target"),
        ("rules/abilities.rs::handle_crew_vehicle#0 -> AbilityActivated", false,
            "A10: crew targets nothing (CR 702.122a)"),
        ("rules/abilities.rs::handle_saddle_mount#0 -> AbilityActivated", false,
            "A11: saddle targets nothing (CR 702.171a)"),
        ("rules/abilities.rs::handle_scavenge_card#0 -> AbilityActivated", true,
            "A12: single target creature"),
        ("rules/abilities.rs::flush_sorted#0 -> AbilityTriggered", true,
            "T6: modular arm, single artifact-creature target"),
        ("rules/abilities.rs::flush_sorted#1 -> AbilityTriggered", true,
            "T7: main flush -- the reported defect's site"),
        ("rules/engine.rs::handle_activate_loyalty_ability#0 -> AbilityActivated", true,
            "A13: loyalty ability targets, built from the command's declared targets"),
        ("rules/engine.rs::handle_level_up_class#0 -> AbilityActivated", false,
            "A14: level-up has no target (CR 716.2a)"),
    ];

    let mut expected_keys: Vec<String> = EXPECTED_SITES
        .iter()
        .map(|(k, _, _)| k.to_string())
        .collect();
    expected_keys.sort();

    assert_eq!(
        all_keys, expected_keys,
        "the announcement-site census changed. A new site, a deleted site, or a \
         site that moved function must be classified ANNOUNCES or NEVER_TARGETS \
         here, with a reason, and this const updated."
    );

    // Non-vacuity: ANNOUNCES and NEVER_TARGETS are both non-empty.
    let announces_count = EXPECTED_SITES.iter().filter(|(_, a, _)| *a).count();
    let never_count = EXPECTED_SITES.len() - announces_count;
    assert!(announces_count > 0, "ANNOUNCES must be non-empty");
    assert!(never_count > 0, "NEVER_TARGETS must be non-empty");

    let classification: HashMap<&str, bool> =
        EXPECTED_SITES.iter().map(|(k, a, _)| (*k, *a)).collect();

    // ── Part 2: every ANNOUNCES site calls the helper, COUNTED ──────────────
    //
    // ENG-2 fix cycle (review finding 3): this used to assert only that the
    // function body CONTAINS `push_target_announcement(`. Two functions carry
    // TWO ANNOUNCES sites each — `flush_sorted` (T6 modular, T7 main flush) and
    // `resolve_top_of_stack_inner` (S4 cipher, S5 suspend) — so "contains at
    // least one" was satisfied by either call alone, and deleting the other was
    // invisible to the whole suite. That mattered most for **T6**, the modular
    // trigger's announcement, which has no behavioural probe anywhere: nothing
    // else in 4,341 tests exercises a modular trigger's target announcement, so
    // this count is its ONLY protection.
    //
    // Counting is still a body-level check, not a per-site one — an unrelated
    // `push_target_announcement` in the same function would mask a deleted real
    // one — but Part 1's census pins exactly which push sites exist per function,
    // so an unrelated call would have to be a new census entry and would fail
    // there first.
    for file in FILES {
        let src = read_src(file);
        let mut needed: HashMap<&str, usize> = HashMap::new();
        for c in classified
            .iter()
            .filter(|c| c.file == *file && classification[c.key.as_str()])
        {
            *needed.entry(c.func.as_str()).or_default() += 1;
        }
        for (func, want) in needed {
            let body = function_body(&src, func);
            let got = body.matches("push_target_announcement(").count();
            assert_eq!(
                got, want,
                "{file}::{func} is classified ANNOUNCES at {want} site(s) but its \
                 body calls push_target_announcement( {got} time(s). Every \
                 announcement site named in EXPECTED_SITES must have its own call: \
                 a function with two announcement sites needs two calls, and \
                 deleting one of them must fail here."
            );
        }
    }

    // ── Part 3: NEVER_TARGETS sites have not quietly grown targets ──────────
    //
    // Scoped to the LOCAL WINDOW between the previous occurrence's push line
    // (exclusive) and this occurrence's push line (inclusive), within the same
    // function -- see the deviation note in this test's doc comment.
    // ENG-2 fix cycle (review finding 7): these are literal SPELLINGS, not a
    // semantic check — a site that grows targets via some other syntax slips
    // past. Widened from three to cover the assignment and mutation forms that
    // actually exist in this codebase; `targets:` alone would false-positive on
    // the `targets: vec![]` that every NEVER_TARGETS site legitimately carries,
    // so the empty-literal form is deliberately NOT forbidden.
    let forbidden = [
        "stack_obj.targets =",
        "stack_obj.targets.push",
        "stack_obj.targets.extend",
        "targets: spell_targets",
        "targets: vec![SpellTarget",
        "targets: targets",
        "targets: declared_targets",
        "targets: resolved_targets",
    ];
    for file in FILES {
        let src = read_src(file);
        let lines: Vec<&str> = src.lines().collect();
        let mut by_func: HashMap<&str, Vec<&Classified>> = HashMap::new();
        for c in classified.iter().filter(|c| c.file == *file) {
            by_func.entry(c.func.as_str()).or_default().push(c);
        }
        for (_func, mut occs) in by_func {
            occs.sort_by_key(|c| c.line_no);
            let mut window_start = 0usize; // 0-indexed; will be set to fn header on first pass
            for (idx, c) in occs.iter().enumerate() {
                if idx == 0 {
                    // Find this function's own header line as the window floor.
                    window_start = lines
                        .iter()
                        .enumerate()
                        .take(c.line_no) // header must be at or before the site
                        .rev()
                        .find(|(_, l)| {
                            let t = l.trim_start();
                            l.len() == t.len()
                                && (t.starts_with("fn ")
                                    || t.starts_with("pub fn ")
                                    || t.starts_with("pub(crate) fn ")
                                    || t.starts_with("async fn "))
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                }
                if !classification[c.key.as_str()] {
                    let window = &lines[window_start..c.line_no.min(lines.len())];
                    for (offset, line) in window.iter().enumerate() {
                        let code = line.split("//").next().unwrap_or("");
                        for pat in forbidden {
                            assert!(
                                !code.contains(pat),
                                "{}: NEVER_TARGETS site '{}' has grown a target-setting \
                                 pattern ({pat:?}) at line {} -- reclassify into ANNOUNCES \
                                 and wire in push_target_announcement",
                                file,
                                c.key,
                                window_start + offset + 1
                            );
                        }
                    }
                }
                window_start = c.line_no; // next occurrence's window starts here (exclusive of this push's own struct-literal tail is fine; forbidden patterns precede the push, not follow it)
            }
        }
    }

    // ── Part 4: the file set is closed ───────────────────────────────────────

    fn walk_dir(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("must be able to read dir {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }

    let src_root = std::path::PathBuf::from(format!("{}/src", env!("CARGO_MANIFEST_DIR")));
    let mut all_rs_files = Vec::new();
    walk_dir(&src_root, &mut all_rs_files);

    let mut files_with_sites: Vec<String> = Vec::new();
    for path in &all_rs_files {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("must be able to read {}: {e}", path.display()));
        let hits = scan(&content);
        if !hits.is_empty() {
            let rel = path
                .strip_prefix(&src_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            files_with_sites.push(rel);
        }
    }
    files_with_sites.sort();
    let mut expected_files: Vec<String> = FILES.iter().map(|f| f.to_string()).collect();
    expected_files.sort();
    assert_eq!(
        files_with_sites, expected_files,
        "an emission site appeared in a file outside the pinned five -- Parts 1-3 \
         are blind to a brand-new module. Classify the new site's function and add \
         the file to the closed set."
    );

    // Non-vacuity: every brace-matched body used above was non-empty (a sanity
    // check that `function_body` really found real code, not an empty string).
    for c in &classified {
        let src = read_src(c.file);
        let body = function_body(&src, &c.func);
        assert!(
            !body.trim().is_empty(),
            "{}::{} resolved to an EMPTY function body -- brace matching broke",
            c.file,
            c.func
        );
    }

    /// Extract a top-level (column-0) function's body from its `fn <name>(`
    /// header to its closing brace.
    ///
    /// **Deviation from the PB-DX19 precedent's naive brace-counting, recorded
    /// here rather than silently taken.** Naive `{`/`}` counting over raw source
    /// text breaks the moment a string or char literal inside the function body
    /// contains an unpaired brace character -- and `resolve_top_of_stack_inner`
    /// (one of this batch's own functions) hits exactly that: counting from its
    /// header never returns to depth 0 before EOF, off by one. This codebase is
    /// `cargo fmt`'d (SR-35 / the Milestone Completion Checklist), which
    /// guarantees every nested block's closing brace is indented -- so the
    /// function's OWN closing brace is the first subsequent line whose entire
    /// trimmed content is `"}"` at column 0. That is what this helper looks for,
    /// and it is immune to brace characters embedded in literals.
    fn function_body(src: &str, name: &str) -> String {
        let lines: Vec<&str> = src.lines().collect();
        for prefix in ["pub fn ", "pub(crate) fn ", "async fn ", "fn "] {
            let needle = format!("{prefix}{name}(");
            if let Some(header_idx) = lines.iter().position(|l| l.starts_with(&needle)) {
                for (offset, line) in lines[(header_idx + 1)..].iter().enumerate() {
                    if *line == "}" {
                        let end_idx = header_idx + 1 + offset;
                        return lines[header_idx..=end_idx].join("\n");
                    }
                }
                panic!(
                    "no column-0 closing brace found after {name}'s header (line {})",
                    header_idx + 1
                );
            }
        }
        panic!("function '{name}' not found by any recognized fn-header spelling");
    }
}
