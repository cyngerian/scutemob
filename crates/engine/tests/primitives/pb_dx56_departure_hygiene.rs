//! PB-DX56 (`scutemob-235`) F1/F2/F3 -- three CR 400.7 / CR 800.4 departure-hygiene
//! fixes. `memory/primitives/pb-DX56-mechanism-census.md` §1 is F1's evidence base;
//! §2 is F2/F3's.
//!
//! # F1 -- CR 400.7 attachment symmetry (`OOS-DX22-8`)
//!
//! `GameState::move_object_to_zone` / `move_object_to_bottom_of_zone` retire the
//! departing object's id and mint a fresh one, but performed no fix-up on the OTHER
//! side of an `attached_to` relationship -- only `paired_with` (CR 702.95e) and
//! `replacement_effects` (MR-M8-16) were cleaned up. When an attacher (an Aura,
//! Equipment, or Fortification) left the battlefield by a route other than the two
//! `sba.rs` cleanup arms (`check_aura_sbas` / `check_equipment_sbas`), its host kept
//! the dead `ObjectId` in `host.attachments` **permanently** -- at rest, not
//! transient. `t1`/`t2` pin the fix; `t3` pins the DELIBERATELY UN-FIXED reverse
//! direction (see the module doc on `GameState::detach_from_host_on_departure`).
//!
//! # F2 -- CR 800.4k: `advance_turn`'s extra-turn branch had no liveness filter
//!
//! CR 800.4k, verbatim: "If a player who has left the game would begin a turn,
//! that turn doesn't begin." The normal-turn-order branch of
//! `rules::turn_structure::advance_turn` honours this
//! (`next_player_in_turn_order` skips `has_lost || has_conceded`), but the
//! extra-turn branch (`turn.extra_turns.pop_back()`) applied no filter at all, and
//! nothing ever prunes the queue elsewhere. `t4` pins the fix (a departed player's
//! queued extra turn is discarded, not begun); `t5` pins the unaffected case (a
//! live player's queued extra turn is still honoured, LIFO,
//! `last_regular_active` untouched).
//!
//! # F3 -- CR 800.4a: `enter_step`'s cleanup-SBA-round grant was unconditional
//! (`OOS-DP9-19`)
//!
//! CR 800.4a's last sentence is unconditional: "If the player who left the game
//! had priority at the time they left, priority passes to the next player in turn
//! order who's still in the game." `enter_step`'s Cleanup-step SBA-round grant
//! wrote `priority_holder = Some(state.turn.active_player)` with no liveness test
//! -- the one hole `priority::grant_priority_to_active_player`'s own doc comment
//! named as "still unconditional (OOS-DP9-19)". Fixed by routing through that
//! helper instead of hand-rolling the grant a third time. `t6` pins it.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    process_command, rules, CardEffectTarget, Command, Effect, GameEvent, GameState,
    GameStateBuilder, ObjectId, PlayerId, SpellTarget, Step, SubType, Target, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

/// A `Command::DestroyPermanent`-shaped destroy, driven directly through
/// `execute_effect` (mirrors `crates/engine/tests/combat/tapped_and_attacking.rs`'s
/// `ec_attacker` idiom) rather than through a card's own activated ability, since
/// the subject here is the zone-move machinery itself, not any one card.
fn destroy(state: &mut GameState, controller: PlayerId, target_id: ObjectId) -> Vec<GameEvent> {
    let mut ctx = EffectContext::new(
        controller,
        target_id, // source is irrelevant to DestroyPermanent's own logic
        vec![SpellTarget {
            target: Target::Object(target_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let effect = Effect::DestroyPermanent {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        cant_be_regenerated: true, // keep the probe's subject to the zone move alone
    };
    execute_effect(state, &effect, &mut ctx)
}

/// Manually attach `attacher` to `host` (mirrors
/// `crates/engine/tests/mechanics_e_l/equip.rs`'s `test_equip_reequip_detaches_from_previous`
/// poke idiom -- the DSL has no `ObjectSpec` field for attachments, so a real attach
/// state is built by mutating the fields directly after `GameStateBuilder::build()`).
fn attach(state: &mut GameState, attacher: ObjectId, host: ObjectId) {
    state.objects_mut().get_mut(&attacher).unwrap().attached_to = Some(host);
    state
        .objects_mut()
        .get_mut(&host)
        .unwrap()
        .attachments
        .push_back(attacher);
}

// ── F1: CR 400.7 attachment symmetry ─────────────────────────────────────────

/// CR 400.7 / `OOS-DX22-8` -- when an attached Equipment leaves the battlefield by
/// a zone change that is NOT one of the two `sba.rs` cleanup arms, the host's
/// `attachments` must drop the departed id, not keep it forever.
#[test]
fn t1_destroying_the_attacher_removes_it_from_the_hosts_attachments() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())]);
    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");
    attach(&mut state, equip_id, creature_id);

    // Precondition: the attachment is really there before the departure.
    assert!(
        state
            .objects()
            .get(&creature_id)
            .unwrap()
            .attachments
            .contains(&equip_id),
        "precondition: creature should list the equipment as attached"
    );

    destroy(&mut state, p1, equip_id);

    // The equipment is gone from the battlefield (a NEW object exists in the
    // graveyard under a fresh id -- CR 400.7 -- so `equip_id` itself is retired).
    assert!(
        !state.objects().contains_key(&equip_id),
        "the equipment's old id should be retired by the zone move"
    );

    // F1: the creature's `attachments` must no longer carry the dead id.
    let creature_obj = state.objects().get(&creature_id).expect("creature exists");
    assert!(
        !creature_obj.attachments.contains(&equip_id),
        "creature.attachments should be cleaned of the departed equipment's id, \
         not left dangling forever (OOS-DX22-8)"
    );
}

/// Paired negative: a permanent with nothing attached to it must be entirely
/// unaffected by a departure elsewhere -- `detach_from_host_on_departure` must not
/// touch any object other than the departing one's actual host.
#[test]
fn t2_destroying_an_unrelated_permanent_leaves_an_unattached_permanents_state_alone() {
    let p1 = p(1);
    let p2 = p(2);

    let bystander = mtg_engine::ObjectSpec::creature(p1, "Bystander Bear", 2, 2);
    let victim = mtg_engine::ObjectSpec::creature(p2, "Victim Bear", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(bystander)
        .object(victim)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let bystander_id = find_object(&state, "Bystander Bear");
    let victim_id = find_object(&state, "Victim Bear");

    assert!(
        state
            .objects()
            .get(&bystander_id)
            .unwrap()
            .attachments
            .is_empty(),
        "precondition: bystander has nothing attached"
    );
    let bystander_attached_to_before = state.objects().get(&bystander_id).unwrap().attached_to;

    destroy(&mut state, p2, victim_id);

    assert!(
        !state.objects().contains_key(&victim_id),
        "the victim's old id should be retired by the zone move"
    );
    let bystander_obj = state
        .objects()
        .get(&bystander_id)
        .expect("bystander exists");
    assert!(
        bystander_obj.attachments.is_empty(),
        "an unrelated permanent's attachments must not be touched by someone else's departure"
    );
    assert_eq!(
        bystander_obj.attached_to, bystander_attached_to_before,
        "an unrelated permanent's attached_to must not be touched by someone else's departure"
    );
}

/// Deliberately UN-fixed, wrong-way-round pin: when the HOST of an attachment
/// departs (rather than the attacher), the attacher's `attached_to` is left
/// dangling by the zone-move machinery on purpose. CR 704.5m and CR 704.5n
/// prescribe OPPOSITE, type-dependent dispositions for this case (Aura ->
/// owner's graveyard; Equipment/Fortification -> merely unattached, stays on the
/// battlefield), already implemented as a state-based action by
/// `rules::sba::check_aura_sbas` / `check_equipment_sbas`. Performing either
/// disposition inside the zone-move helper itself, outside an SBA sweep, would be
/// CR-wrong -- see `GameState::detach_from_host_on_departure`'s doc comment. This
/// test exists so a future "finish the job" edit that clears `attached_to`
/// symmetrically here goes red instead of silently shipping a CR violation.
#[test]
fn t3_destroying_the_host_deliberately_leaves_the_attachers_attached_to_dangling() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())]);
    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");
    attach(&mut state, equip_id, creature_id);

    // Destroy the HOST, not the attacher.
    destroy(&mut state, p1, creature_id);

    assert!(
        !state.objects().contains_key(&creature_id),
        "the creature's old id should be retired by the zone move"
    );

    // The equipment object itself never moved -- it is still the SAME id, still on
    // the battlefield, and its `attached_to` is left pointing at the now-dead
    // creature id. This is the dangle only an SBA sweep (`check_equipment_sbas`)
    // may resolve, and this probe calls `execute_effect` directly with no SBA
    // sweep in between -- so the dangling state must still be observable here.
    let equip_obj = state
        .objects()
        .get(&equip_id)
        .expect("equipment still exists, unmoved");
    assert_eq!(
        equip_obj.attached_to,
        Some(creature_id),
        "wrong-way-round pin: the zone-move helper deliberately does NOT clear \
         attached_to when the HOST departs -- that is `rules::sba`'s job, and its \
         disposition depends on the attacher's type (CR 704.5m vs CR 704.5n)"
    );
}

// ── F2: CR 800.4k extra-turn liveness filter ─────────────────────────────────

/// CR 800.4k -- an extra turn queued for a player who has since left the game must
/// be dropped ("that turn doesn't begin"), not begun.
#[test]
fn t4_a_departed_players_queued_extra_turn_does_not_begin() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Queue an extra turn for p1, then have p1 leave the game.
    state.turn_mut().extra_turns.push_back(p1);
    state.turn_mut().last_regular_active = p1;
    state.players_mut().get_mut(&p1).unwrap().has_lost = true;

    let (new_turn, _events) = rules::turn_structure::advance_turn(&state).unwrap();

    assert_ne!(
        new_turn.active_player, p1,
        "CR 800.4k: a queued extra turn for a departed player must not begin"
    );
    assert_eq!(
        new_turn.active_player, p2,
        "with p1's queued extra turn discarded, normal turn order should hand the \
         turn to p2"
    );
    assert!(
        new_turn.extra_turns.is_empty(),
        "the departed player's dead queue entry must be consumed (dropped), not \
         left to be re-tried on a later advance_turn call"
    );
}

/// Paired negative: an extra turn queued for a player who is still alive is
/// unaffected -- still LIFO, and `last_regular_active` is still untouched by the
/// extra-turn branch.
#[test]
fn t5_a_live_players_queued_extra_turn_is_still_honoured() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 queued an extra turn (e.g. Time Warp); LIFO order, most recent first.
    state.turn_mut().extra_turns.push_back(p2);
    state.turn_mut().last_regular_active = p1;

    let last_regular_active_before = state.turn().last_regular_active;

    let (new_turn, _events) = rules::turn_structure::advance_turn(&state).unwrap();

    assert_eq!(
        new_turn.active_player, p2,
        "a live player's queued extra turn must still be honoured"
    );
    assert!(
        new_turn.extra_turns.is_empty(),
        "the (only) queued extra turn should be consumed once taken"
    );
    assert_eq!(
        new_turn.last_regular_active, last_regular_active_before,
        "extra turns must not advance last_regular_active (normal turn order is \
         untouched by them)"
    );
}

// ── F3: CR 800.4a cleanup-SBA-round priority grant ───────────────────────────

/// CR 800.4a / `OOS-DP9-19` -- `enter_step`'s cleanup-SBA-round priority grant must
/// not name a departed active player. Drives the branch by forcing a repeated
/// cleanup SBA round (a creature with lethal damage marked dies during the
/// cleanup-step SBA check, which is exactly the "had_events" condition that
/// branch exists to serve), with the active player having already left the game.
#[test]
fn t6_cleanup_sba_round_grant_skips_a_departed_active_player() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    // Three players, not two: `process_command` refuses ANY command once
    // `active_players().len() <= 1` (`GameAlreadyOver`), so a 2-player fixture
    // with p1 departed would already be over the instant p1.has_lost is set --
    // there would be no live command surface left to drive `enter_step` through
    // at all. With p2 and p3 both still active the game is not over, and both
    // must pass in turn (APNAP) to reach `handle_all_passed`.
    //
    // A creature with 0 toughness (CR 704.5f), NOT lethal damage: `cleanup_actions`
    // (CR 514.2) unconditionally clears `damage_marked` on every permanent as the
    // FIRST turn-based action `enter_step`'s loop performs when (re-)entering
    // Cleanup, before any SBA sweep -- a damage-based setup would be erased before
    // it could be observed. A defined-but-zero toughness is untouched by that
    // clear and still dies to CR 704.5f the moment the SBA sweep in the branch
    // under test runs, which is exactly what this probe needs to make `had_events`
    // true without relying on damage.
    let dying_creature = mtg_engine::ObjectSpec::creature(p1, "Doomed Bear", 2, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(dying_creature)
        .active_player(p1)
        .at_step(Step::Cleanup)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Doomed Bear");

    // The active player has already left the game by the time this Cleanup-step
    // SBA round runs, and priority has already been granted to a live player for
    // this round (mirroring the state `enter_step`'s own prior grant would have
    // produced) so that both live players passing drives
    // `handle_all_passed` -> `enter_step` for real, through the same public
    // command surface every other channel uses (there is no direct public hook
    // for `enter_step` itself).
    state.players_mut().get_mut(&p1).unwrap().has_lost = true;
    state.turn_mut().priority_holder = Some(p2);

    let (state, _events) = process_command(state, Command::PassPriority { player: p2 })
        .unwrap_or_else(|e| panic!("PassPriority by p2 failed: {e:?}"));
    assert_eq!(
        state.turn().priority_holder,
        Some(p3),
        "t6 setup error: p2's pass should hand priority to the next LIVE player \
         (p1 is departed) in APNAP order"
    );
    let (state, _events) = process_command(state, Command::PassPriority { player: p3 })
        .unwrap_or_else(|e| panic!("PassPriority by p3 failed: {e:?}"));

    // The doomed creature really did die (confirms the SBA round this test relies
    // on to reach the branch under test actually ran).
    assert!(
        !state.objects().contains_key(&creature_id),
        "t6 setup error: the zero-toughness creature should have died to CR \
         704.5f during the cleanup SBA round -- if it didn't, this probe never \
         reaches the branch under test"
    );

    // F3: priority_holder must never name the departed p1.
    assert_ne!(
        state.turn().priority_holder,
        Some(p1),
        "CR 800.4a: priority must not be granted to a player who has left the game \
         (OOS-DP9-19)"
    );
}
