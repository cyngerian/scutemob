//! PB-DX53 (`scutemob-231`) — CR 508.3d/508.4, ruling 2007-10-01 (`OOS-DX21-1`):
//! the extra-combat raid-count split, through the REAL `LocalGame`/`HumanChoice`
//! channel.
//!
//! `crates/engine/tests/primitives/pb_dx53_raid_count_split.rs` proves the
//! primitive at the engine level (t1-t7). This file exists because a corrected
//! `Condition` is not a repaired decision until the OFFER LAYER honours it
//! (`kaito_shizuki`'s lesson, PB-DX43): `windbrisk_heights`'s raid ability
//! carries an `activation_condition`, and `legal_actions.rs`'s
//! `activated_ability_is_activatable` gates the OFFER on the identical
//! `check_condition` call the engine uses at resolution (CR 602.5b) -- so a
//! failing condition is never even OFFERED (SR-38), and a passing one both
//! offers AND resolves.
//!
//! `aggravated_assault` (`Complete`, deck-legal) is the REAL CR 500.8 card that
//! creates the extra combat phase -- driven end to end rather than spliced into
//! `state.turn_mut().additional_phases` directly, so this probe exercises the
//! genuine `Effect::AdditionalCombatPhase` -> `EndOfCombat` redirect machinery
//! together with the PB-DX53 write site, not either in isolation.
//!
//! * **c1** -- 3 attackers in combat 1 + 1 attacker in combat 2 (4 distinct
//!   creatures this turn): Windbrisk's raid ability IS offered and ACCEPTED
//!   (resolves off the stack with no error).
//! * **c2** -- 1 attacker in combat 1 + 1 attacker in combat 2 (2 distinct
//!   creatures, never reaching 3 at any point in the turn): Windbrisk's raid
//!   ability is REFUSED, i.e. absent from the offered action list entirely
//!   (SR-38) -- non-vacuity for c1, proving the acceptance there is the
//!   condition's doing and not an unconditional offer. **Deviates from the
//!   plan's literal "2 + 1" for this row** -- 2 + 1 = 3, which the ruling
//!   itself says must be ACCEPTED, not refused; see `c2`'s own doc comment.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, AttackTarget, CardDefinition, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, LegalAction, LocalGame, LocalGameLimits, PendingDecision,
    StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 53_53_53;

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// A bot that always passes priority and never blocks/attacks/targets --
/// deterministic filler for `p2`, who owns nothing this drive needs to act on
/// (mirrors `crates/simulator/tests/local_game.rs`'s `AlwaysPassBot`).
struct AlwaysPassBot;

impl Bot for AlwaysPassBot {
    fn choose_action(
        &mut self,
        _state: &GameState,
        player: PlayerId,
        _legal: &[LegalAction],
    ) -> mtg_engine::Command {
        mtg_engine::Command::PassPriority { player }
    }
    fn choose_targets(
        &mut self,
        _state: &GameState,
        _valid: &[ObjectId],
        _count: usize,
    ) -> Vec<ObjectId> {
        Vec::new()
    }
    fn choose_attackers(
        &mut self,
        _state: &GameState,
        _eligible: &[ObjectId],
        _targets: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)> {
        Vec::new()
    }
    fn choose_blockers(
        &mut self,
        _state: &GameState,
        _eligible: &[ObjectId],
        _attackers: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)> {
        Vec::new()
    }
    fn choose_mulligan_bottom(&mut self, _hand: &[ObjectId], _count: usize) -> Vec<ObjectId> {
        Vec::new()
    }
    fn name(&self) -> &str {
        "AlwaysPassBot"
    }
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 2,
        max_commands: 2000,
        max_consecutive_passes: 500,
        record_journal: false,
    }
}

/// `p1` controls `aggravated_assault`, `windbrisk_heights`, five Mountains (pays
/// Aggravated Assault's `{3}{R}{R}`), one Plains (pays Windbrisk's `{W}`), and
/// four vanilla creatures. `p2` has nothing -- an `AlwaysPassBot` -- so every
/// combat and main-phase decision in this drive belongs to `p1`.
fn fixture() -> GameState {
    let defs = card_defs_by_name();
    let real = |name: &str, zone: ZoneId| {
        enrich_spec_from_def(
            ObjectSpec::card(p(1), name)
                .in_zone(zone)
                .with_card_id(card_name_to_id(name)),
            &defs,
        )
    };

    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(real("Aggravated Assault", ZoneId::Battlefield))
        .object(real("Windbrisk Heights", ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(1), "Bear A", 2, 2))
        .object(ObjectSpec::creature(p(1), "Bear B", 2, 2))
        .object(ObjectSpec::creature(p(1), "Bear C", 2, 2))
        .object(ObjectSpec::creature(p(1), "Bear D", 2, 2))
        .object(real("Plains", ZoneId::Battlefield));
    // Five Mountains -- {R} pays both the generic and the coloured pips of
    // Aggravated Assault's {3}{R}{R}. Duplicate names are fine: every consumer
    // below finds a Mountain generically through the offered `LegalAction`
    // list, never through `id_of`.
    for _ in 0..5 {
        builder = builder.object(real("Mountain", ZoneId::Battlefield));
    }
    // Library filler so neither player decks out inside `limits().max_turns`.
    for player in [p(1), p(2)] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Library Filler {i} P{}", player.0))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().expect("PB-DX53 channel fixture must build")
}

fn start_human_game() -> LocalGame<StubProvider> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(AlwaysPassBot));
    let human: BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (game, _events) =
        LocalGame::start(fixture(), SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX53 channel game must start");
    game
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

fn advance_human(game: &mut LocalGame<StubProvider>) -> PendingDecision {
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman (p1's turn throughout this drive), got {other:?}"),
    }
}

fn submit(
    game: &mut LocalGame<StubProvider>,
    decision: &PendingDecision,
    index: usize,
    params: ActionParams,
) {
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: index,
            params,
        },
    )
    .unwrap_or_else(|e| panic!("submitting action index {index} failed: {e:?}"));
}

/// Pass priority until an action matching `want` is offered, submitting a plain
/// `PassPriority` at every window along the way. Panics rather than returning
/// `None` -- a probe that silently stops early asserts nothing.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    want: impl Fn(&LegalAction) -> bool,
) -> PendingDecision {
    for _ in 0..400 {
        let d = advance_human(game);
        if d.actions.iter().any(&want) {
            return d;
        }
        let pass = d
            .actions
            .iter()
            .position(|a| matches!(a, LegalAction::PassPriority))
            .unwrap_or_else(|| panic!("no matching action and no PassPriority: {:?}", d.actions));
        submit(game, &d, pass, ActionParams::default());
    }
    panic!("drive_until exceeded its iteration budget without finding a matching action");
}

fn is_named(state: &GameState, id: ObjectId, name: &str) -> bool {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name == name)
        .unwrap_or(false)
}

fn find_tap_source(decision: &PendingDecision, state: &GameState, name: &str) -> usize {
    decision
        .actions
        .iter()
        .position(
            |a| matches!(a, LegalAction::TapForMana { source, .. } if is_named(state, *source, name)),
        )
        .unwrap_or_else(|| panic!("no untapped {name:?} offered TapForMana: {:?}", decision.actions))
}

fn find_activate(decision: &PendingDecision, target: ObjectId) -> Option<usize> {
    decision
        .actions
        .iter()
        .position(|a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == target))
}

/// Drive from turn start through Aggravated Assault's activation (paying its
/// real `{3}{R}{R}` from five real Mountains) and into the resulting extra
/// combat, declaring `combat1` attackers in the first combat and `combat2`
/// (distinct, never-yet-tapped) attackers in the second. Returns the game
/// paused at the priority window immediately after the SECOND declaration,
/// plus the ids of every Bear in declaration order (A, B, C, D).
fn drive_through_two_combats(
    combat1: usize,
    combat2: usize,
) -> (
    LocalGame<StubProvider>,
    PendingDecision,
    ObjectId,
    Vec<ObjectId>,
) {
    let mut game = start_human_game();
    let state0 = game.state().clone();
    let aggravated_id = id_of(&state0, "Aggravated Assault");
    let windbrisk_id = id_of(&state0, "Windbrisk Heights");
    let bears: Vec<ObjectId> = ["Bear A", "Bear B", "Bear C", "Bear D"]
        .iter()
        .map(|n| id_of(&state0, n))
        .collect();
    let p2 = p(2);

    assert!(
        combat1 + combat2 <= bears.len(),
        "fixture only has {} distinct creatures, requested {}",
        bears.len(),
        combat1 + combat2
    );

    // 1. Reach Aggravated Assault's own activation window (CR 500.10a sorcery
    // speed: precombat main, empty stack, active player, priority). Mana
    // affordability is NOT checked at offer time (`activated_ability_is_activatable`
    // has no such gate), so this appears before any Mountain is tapped.
    let d = drive_until(
        &mut game,
        |a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == aggravated_id),
    );

    // 2. Tap five Mountains for {R}{R}{R}{R}{R}, which pays {3}{R}{R} in full.
    let mut current = d;
    for _ in 0..5 {
        let state = game.state().clone();
        let idx = find_tap_source(&current, &state, "Mountain");
        submit(&mut game, &current, idx, ActionParams::default());
        current = advance_human(&mut game);
    }

    // 3. Activate Aggravated Assault.
    let idx = find_activate(&current, aggravated_id)
        .expect("Aggravated Assault's own activation must still be offered after tapping mana");
    submit(&mut game, &current, idx, ActionParams::default());

    // 4. Drive to combat 1's DeclareAttackers (through the rest of precombat
    // main -- letting Aggravated Assault resolve -- and BeginningOfCombat).
    let d = drive_until(&mut game, |a| {
        matches!(a, LegalAction::DeclareAttackers { .. })
    });
    let idx = d
        .actions
        .iter()
        .position(|a| matches!(a, LegalAction::DeclareAttackers { .. }))
        .unwrap();
    let attackers1: Vec<(ObjectId, AttackTarget)> = bears[..combat1]
        .iter()
        .map(|&id| (id, AttackTarget::Player(p2)))
        .collect();
    submit(
        &mut game,
        &d,
        idx,
        ActionParams {
            attackers: attackers1,
            ..ActionParams::default()
        },
    );

    // 5. Drive through DeclareBlockers (p2's decision, handled internally by
    // the bot)/CombatDamage/EndOfCombat -- CR 500.8's redirect sends us to a
    // SECOND BeginningOfCombat/DeclareAttackers instead of PostCombatMain.
    let d = drive_until(&mut game, |a| {
        matches!(a, LegalAction::DeclareAttackers { .. })
    });
    let idx = d
        .actions
        .iter()
        .position(|a| matches!(a, LegalAction::DeclareAttackers { .. }))
        .unwrap();
    let attackers2: Vec<(ObjectId, AttackTarget)> = bears[combat1..combat1 + combat2]
        .iter()
        .map(|&id| (id, AttackTarget::Player(p2)))
        .collect();
    submit(
        &mut game,
        &d,
        idx,
        ActionParams {
            attackers: attackers2,
            ..ActionParams::default()
        },
    );

    // 6. The priority window immediately after combat 2's declaration -- CR
    // 117.3c (PB-DP1): the actor keeps priority, so this is the SAME window
    // Windbrisk's raid ability would be offered in, with no intervening pass.
    let post_declare = advance_human(&mut game);
    (game, post_declare, windbrisk_id, bears)
}

#[test]
/// **c1** -- 3 attackers in combat 1 + 1 MORE, distinct, attacker in combat 2
/// (4 distinct creatures declared this turn): Windbrisk Heights' raid ability
/// is OFFERED and, submitted, ACCEPTED (resolves off the stack with no error).
fn c1_windbrisk_accepted_after_three_plus_one_across_an_extra_combat() {
    let (mut game, decision, windbrisk_id, _bears) = drive_through_two_combats(3, 1);

    let set_len = game
        .state()
        .player(p(1))
        .unwrap()
        .creatures_declared_as_attackers_this_turn
        .len();
    assert_eq!(
        set_len, 4,
        "precondition: 3 + 1 distinct creatures must have accumulated to 4 this turn"
    );

    // Pay Windbrisk's own {W} first -- a real Plains, tapped through the same
    // TapForMana channel used for Aggravated Assault's Mountains.
    let state = game.state().clone();
    let plains_idx = find_tap_source(&decision, &state, "Plains");
    submit(&mut game, &decision, plains_idx, ActionParams::default());
    let decision = advance_human(&mut game);

    let idx = find_activate(&decision, windbrisk_id).unwrap_or_else(|| {
        panic!(
            "Windbrisk Heights' raid ability must be OFFERED with 4 distinct creatures \
             declared this turn: {:?}",
            decision.actions
        )
    });
    submit(&mut game, &decision, idx, ActionParams::default());

    // Resolve the stack: pass until it is empty, proving genuine ACCEPTANCE
    // rather than a submission that merely didn't panic.
    for _ in 0..20 {
        if game.state().stack_objects().is_empty() {
            break;
        }
        let d = advance_human(&mut game);
        let pass = d
            .actions
            .iter()
            .position(|a| matches!(a, LegalAction::PassPriority))
            .expect("PassPriority must be legal while the ability resolves");
        submit(&mut game, &d, pass, ActionParams::default());
    }
    assert!(
        game.state().stack_objects().is_empty(),
        "Windbrisk Heights' raid ability must resolve off the stack"
    );
}

#[test]
/// **c2** -- non-vacuity for c1: 1 attacker in combat 1 + 1 MORE, distinct,
/// attacker in combat 2 (2 distinct creatures declared this turn, never
/// reaching 3 at any point) -- Windbrisk's raid ability must be REFUSED, i.e.
/// entirely ABSENT from the offered action list (SR-38), proving c1's
/// acceptance is the `Condition`'s doing and not an unconditional offer.
///
/// **Deviation from `memory/primitives/pb-DX53-plan.md` §8's literal numbers**,
/// reported rather than silently followed: the plan specifies "2 in combat 1 +
/// 1 in combat 2" for this row, but 2 + 1 = **3** distinct creatures, which
/// DOES satisfy ruling 2007-10-01's "three or more ... at any point in the
/// turn" and must be ACCEPTED, not refused -- the plan's own numbers describe
/// c1's outcome a second time, not a negative control for it. This test uses
/// 1 + 1 = 2 (genuinely under the threshold) instead.
fn c2_windbrisk_refused_when_the_turn_total_never_reaches_three() {
    let (game, decision, windbrisk_id, _bears) = drive_through_two_combats(1, 1);

    let set_len = game
        .state()
        .player(p(1))
        .unwrap()
        .creatures_declared_as_attackers_this_turn
        .len();
    assert_eq!(
        set_len, 2,
        "precondition: 1 + 1 distinct creatures must accumulate to 2 (never 3) this turn"
    );

    assert!(
        find_activate(&decision, windbrisk_id).is_none(),
        "Windbrisk Heights' raid ability must NOT be offered with only 2 distinct \
         creatures declared this turn: {:?}",
        decision.actions
    );
}
