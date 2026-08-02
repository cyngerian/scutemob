//! SIM-1 test matrix (`memory/primitives/sim-1-plan.md` §4): commander casts from the
//! command zone reach `StubProvider`, `LocalGame`'s human and bot auto-tap paths, and
//! the engine itself agrees with every offer.
//!
//! CR 903.6 (commander to the command zone), CR 903.8 (cast from the command zone /
//! commander tax), CR 408.1 (the command zone is not commanders-only), CR 601.2a/601.2f
//! (casting proposal / total cost), CR 117.1a (timing), CR 101.2 (Drannith Magistrate).
//!
//! # The one rule every fixture in this file obeys
//!
//! **Every fixture MUST call `builder.player_commander(pid, cid)` in addition to
//! placing the object in `ZoneId::Command(pid)`.** Without the registration,
//! `PlayerState::commander_ids` is empty and SIM-1's whole mechanism --
//! `legal_actions::effective_cast_cost` and the command-zone loop in
//! `StubProvider::legal_actions` both key on `commander_ids`, never on the zone alone
//! (CR 408.1 makes the command zone a home for non-commander objects too, e.g.
//! emblems and CR 903.9a/b returns) -- silently treats the object as "just something
//! sitting in the zone" and every assertion below would pass or fail VACUOUSLY. See
//! plan §0.3 / §4. `commander_state` below bakes the pairing in so no test here can
//! forget it.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stubs::{ActiveRestriction, FlashGrant};
use mtg_engine::{
    apply_commander_tax, process_command, AttackTarget, CardId, CardType, Command, EffectDuration,
    FlashGrantFilter, GameRestriction, GameState, GameStateBuilder, HybridMana, ManaAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PhyrexianMana, PlayerId, Step, SuperType, ZoneId,
};
use mtg_simulator::{
    effective_cast_cost, ActionParams, AdvanceOutcome, Bot, HeuristicBot, HumanChoice, LegalAction,
    LegalActionProvider, LocalGame, LocalGameLimits, RandomBot, StubProvider,
};

// ── Small helpers ────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

/// Find a battlefield/zone object's id by its printed name. Panics loudly rather
/// than returning an `Option` -- a missing fixture object is a bug in the fixture,
/// not a case any of these tests want to handle gracefully.
fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?} in this fixture"))
}

/// Places `name`/`card_id` as an ACTUAL, registered commander for `pid`: an object in
/// `ZoneId::Command(pid)` AND a `player_commander(pid, card_id)` registration. See
/// the module doc -- this is the one rule every fixture here obeys.
fn commander_state(
    builder: GameStateBuilder,
    pid: PlayerId,
    name: &str,
    card_id: CardId,
    cost: ManaCost,
) -> GameStateBuilder {
    let spec = ObjectSpec::card(pid, name)
        .with_card_id(card_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(cost)
        .in_zone(ZoneId::Command(pid));
    builder.player_commander(pid, card_id).object(spec)
}

/// An untapped battlefield land producing 1 colorless mana. Deliberately colorless
/// (rather than the plan's illustrative "Plains"/white) so every fixture's
/// affordability arithmetic is pure mana-VALUE counting, with no color-pip
/// bookkeeping to get wrong.
fn colorless_land(pid: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::land(pid, name).with_mana_ability(ManaAbility::tap_for(ManaColor::Colorless))
}

/// `n` untapped colorless lands for `pid`, named `{prefix} 1`.."{prefix} n`.
fn n_colorless_lands(pid: PlayerId, prefix: &str, n: u32) -> Vec<ObjectSpec> {
    (1..=n)
        .map(|i| colorless_land(pid, &format!("{prefix} {i}")))
        .collect()
}

/// A pure-generic mana cost of value `mv` (no color pips) -- affordable by any `mv`
/// colorless (or otherwise) mana.
fn generic_cost(mv: u32) -> ManaCost {
    ManaCost {
        generic: mv,
        ..Default::default()
    }
}

/// Pre-set `player`'s commander tax counter for `card_id` to `tax` (CR 903.8) --
/// mirrors the identical pattern used throughout `crates/engine/tests/rules/commander.rs`
/// (e.g. `test_cast_commander_from_command_zone_second_time`).
fn set_tax(state: &mut GameState, player: PlayerId, card_id: &CardId, tax: u32) {
    state
        .players_mut()
        .get_mut(&player)
        .unwrap()
        .commander_tax
        .insert(card_id.clone(), tax);
}

/// A bare `Command::CastSpell` for `card`, announcing nothing (no targets, no X, no
/// modes, no alt cost) -- exactly what a vanilla commander needs.
fn cast_spell_cmd(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
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
    }))
}

fn find_action_index(actions: &[LegalAction], pred: impl Fn(&LegalAction) -> bool) -> usize {
    actions
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("no matching action found in {:?}", actions))
}

fn is_cast_spell_for(action: &LegalAction, card: ObjectId) -> bool {
    matches!(action, LegalAction::CastSpell { card: c, .. } if *c == card)
}

fn contains_cast_spell_for(actions: &[LegalAction], card: ObjectId) -> bool {
    actions.iter().any(|a| is_cast_spell_for(a, card))
}

fn small_limits(max_turns: u32) -> LocalGameLimits {
    LocalGameLimits {
        max_turns,
        max_commands: max_turns * 200,
        max_consecutive_passes: 100,
        record_journal: true,
    }
}

/// Drive a human-seat `LocalGame` forward, passing priority at every decision that
/// does not yet offer `commander`'s cast, until one does. Needed because
/// `LocalGame::start` always resets the turn to `Step::Untap` regardless of what the
/// fixture's builder set (`local_game.rs`'s own doc, and confirmed here by
/// observation): the FIRST human decision is Upkeep, not the main phase, and a
/// commander is sorcery-speed only (T2), so it is never in THAT decision's action
/// list. Once this returns, the decision's action list holds every action legal in
/// that SAME priority window -- including any `TapForMana` actions a caller wants to
/// submit before the cast itself (T8b).
fn drive_to_commander_offer<P: LegalActionProvider>(
    game: &mut LocalGame<P>,
    commander: ObjectId,
) -> mtg_simulator::PendingDecision {
    loop {
        let decision = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!(
                "expected AwaitingHuman while driving to the commander offer, got {:?}",
                other
            ),
        };
        if contains_cast_spell_for(&decision.actions, commander) {
            return decision;
        }
        let idx = find_action_index(&decision.actions, |a| {
            matches!(a, LegalAction::PassPriority)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: idx,
                params: ActionParams::default(),
            },
        )
        .unwrap_or_else(|e| panic!("PassPriority submit failed while driving: {:?}", e));
    }
}

/// T9's purpose-built bot: always casts `commander` the moment the provider offers
/// it, otherwise passes. Deliberately NOT `HeuristicBot` -- T10 already pins that
/// bot's scoring; T9 isolates exactly one thing, whether `advance()`'s bot-seat
/// auto-tap (`local_game.rs` Step 7) taps for the TAXED cost rather than the
/// printed one.
struct CastsTheCommanderBot {
    commander: ObjectId,
}

impl Bot for CastsTheCommanderBot {
    fn choose_action(
        &mut self,
        _state: &GameState,
        player: PlayerId,
        legal: &[LegalAction],
    ) -> Command {
        if legal.iter().any(|a| is_cast_spell_for(a, self.commander)) {
            return cast_spell_cmd(player, self.commander);
        }
        Command::PassPriority { player }
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
        "CastsTheCommanderBot"
    }
}

// ── T1 ───────────────────────────────────────────────────────────────────────────

/// CR 903.8 / CR 117.1a: a registered, affordable, untaxed commander is offered as a
/// `CastSpell` action during the controller's main phase with an empty stack.
///
/// Discriminates on the command-zone ENUMERATION alone (SIM-1 plan §3 Step 5):
/// reverting that step (deleting the command-zone loop from
/// `StubProvider::legal_actions`) makes this the only test in the file that fails,
/// since it is the only one asserting an offer exists at all with no other moving
/// part (no tax, no restriction, no flash grant).
#[test]
fn test_sim1_commander_offered_at_sorcery_speed() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-a");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 Commander", card_id, generic_cost(2));
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        contains_cast_spell_for(&actions, commander_id),
        "an affordable, untaxed, registered commander must be offered at sorcery \
         speed during the controller's main phase: {:?}",
        actions
    );
}

// ── T2 ───────────────────────────────────────────────────────────────────────────

/// CR 117.1a: the SAME commander, same affordability, is withheld outside a main
/// phase with an empty stack (here: Upkeep) -- a commander has no inherent flash and
/// is sorcery-speed only unless something grants it (T2b).
///
/// Non-vacuity: the action list is non-empty and still carries `PassPriority`, so a
/// fixture that silently produced no actions at all could not pass this vacuously.
#[test]
fn test_sim1_commander_withheld_at_instant_speed() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-b");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 Commander", card_id, generic_cost(2));
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let state = builder
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .expect("fixture should build");

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        !contains_cast_spell_for(&actions, commander_id),
        "a plain commander must NOT be offered outside a main phase with an empty \
         stack: {:?}",
        actions
    );
    assert!(
        !actions.is_empty(),
        "non-vacuity: some action must be offered"
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, LegalAction::PassPriority)),
        "non-vacuity: PassPriority must be among the offered actions: {:?}",
        actions
    );
}

// ── T2b ──────────────────────────────────────────────────────────────────────────

/// CR 601.3b: the identical instant-speed fixture as T2, but under an active
/// `FlashGrantFilter::AllSpells` grant for this player -- now the commander IS
/// offered. Pins that the command-zone loop MIRRORS the hand loop's whole timing
/// predicate (`can_cast_at_this_time`), including the flash-grant scan, rather than
/// hard-coding "sorcery speed only" for command-zone casts.
#[test]
fn test_sim1_commander_offered_at_instant_speed_under_a_flash_grant() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-b2");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 Commander", card_id, generic_cost(2));
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .expect("fixture should build");

    state.flash_grants_mut().push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::AllSpells,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        contains_cast_spell_for(&actions, commander_id),
        "under an AllSpells flash grant the commander must be offered at instant \
         speed: {:?}",
        actions
    );
}

// ── T3 ───────────────────────────────────────────────────────────────────────────

/// CR 903.8 / CR 601.2f (criterion 5985): a commander taxed once (printed {generic
/// 2}, taxed {generic 4}) is withheld when only the PRINTED cost is affordable --
/// exactly 2 untapped lands, empty pool. Asserts BOTH halves of the SR-38 subset
/// property: the provider withholds it, AND a hand-built `Command::CastSpell` for it
/// really is rejected by `process_command`. That second half is what turns "the
/// provider is conservative" into "the provider is a correct subset of the engine".
#[test]
fn test_sim1_taxed_commander_is_withheld_when_only_the_printed_cost_is_affordable() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-c");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(
        builder,
        p1,
        "Sim1 Commander",
        card_id.clone(),
        generic_cost(2),
    );
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");
    set_tax(&mut state, p1, &card_id, 1);

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        !contains_cast_spell_for(&actions, commander_id),
        "a commander taxed to {{generic 4}} with only 2 mana available must NOT be \
         offered: {:?}",
        actions
    );
    assert!(
        !actions.is_empty(),
        "non-vacuity: some action must be offered"
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, LegalAction::PassPriority)),
        "non-vacuity: PassPriority must be among the offered actions: {:?}",
        actions
    );

    let result = process_command(state, cast_spell_cmd(p1, commander_id));
    assert!(
        result.is_err(),
        "the engine itself must also reject this cast -- 2 mana cannot pay a \
         {{generic 4}} taxed cost: {:?}",
        result
    );
}

// ── T4 ───────────────────────────────────────────────────────────────────────────

/// Non-vacuity partner for T3: the SAME fixture shape with tax absent (0) IS
/// offered with exactly 2 lands -- proves T3's withholding is the TAX, not some
/// unrelated defect in the fixture (e.g. an affordability check that always fails).
#[test]
fn test_sim1_commander_offered_at_zero_tax_with_exactly_the_printed_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-d");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 Commander", card_id, generic_cost(2));
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        contains_cast_spell_for(&actions, commander_id),
        "at zero tax, exactly the printed cost in mana must be enough: {:?}",
        actions
    );
}

// ── T4b ──────────────────────────────────────────────────────────────────────────

/// The other side of T3: once the TAXED cost itself becomes affordable (4 lands
/// instead of 2), the commander IS offered again -- proves the gate does not simply
/// suppress every taxed commander outright, only unaffordable ones.
#[test]
fn test_sim1_taxed_commander_offered_once_the_taxed_cost_is_affordable() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-e");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(
        builder,
        p1,
        "Sim1 Commander",
        card_id.clone(),
        generic_cost(2),
    );
    for land in n_colorless_lands(p1, "Land", 4) {
        builder = builder.object(land);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");
    set_tax(&mut state, p1, &card_id, 1);

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        contains_cast_spell_for(&actions, commander_id),
        "4 lands must be enough to pay the taxed {{generic 4}} cost: {:?}",
        actions
    );
}

// ── T5 ───────────────────────────────────────────────────────────────────────────

/// CR 903.8 / CR 408.1: a real, registered, trivially-affordable (mv 0) commander
/// sits in the command zone alongside a SECOND, fully-affordable card object that is
/// NOT registered as a commander (`commander_ids` never contains its `CardId`) --
/// exactly the shape CR 408.1 permits (the command zone is not commanders-only) and
/// exactly the shape `mtg-fuzzer`/`tests/local_game.rs` are in today (§0.3: neither
/// registers ANY commander, so this is also literally their fixture shape).
///
/// The real commander IS offered (non-vacuity: this also proves the fixture is not
/// simply broken); the non-commander object is NEVER offered, and a hand-built
/// `Command::CastSpell` for it is rejected by the engine's own CR 903.8 gate.
#[test]
fn test_sim1_a_non_commander_object_in_the_command_zone_is_never_offered() {
    let p1 = p(1);
    let p2 = p(2);
    let real_commander_id = cid("sim1-commander-f");
    let fake_id = cid("sim1-not-a-commander");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    // The real commander: mv 0, trivially affordable regardless of mana sources.
    builder = commander_state(
        builder,
        p1,
        "Sim1 Real Commander",
        real_commander_id,
        generic_cost(0),
    );
    // A second command-zone object, NOT registered via `player_commander` -- e.g. an
    // emblem-shaped object, or (§0.3) exactly what `mtg-fuzzer` builds today.
    let fake_spec = ObjectSpec::card(p1, "Sim1 Fake Commander")
        .with_card_id(fake_id.clone())
        .with_types(vec![CardType::Creature])
        .with_mana_cost(generic_cost(2))
        .in_zone(ZoneId::Command(p1));
    builder = builder.object(fake_spec);
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");

    let real_id = find_by_name(&state, "Sim1 Real Commander");
    let fake_obj_id = find_by_name(&state, "Sim1 Fake Commander");
    let actions = StubProvider.legal_actions(&state, p1);

    assert!(
        contains_cast_spell_for(&actions, real_id),
        "non-vacuity: the REAL registered commander must still be offered: {:?}",
        actions
    );
    assert!(
        !contains_cast_spell_for(&actions, fake_obj_id),
        "an object in the command zone that is NOT one of this player's registered \
         commanders must never be offered: {:?}",
        actions
    );
    assert!(!actions.is_empty());
    assert!(actions
        .iter()
        .any(|a| matches!(a, LegalAction::PassPriority)));

    let result = process_command(state, cast_spell_cmd(p1, fake_obj_id));
    assert!(
        result.is_err(),
        "the engine's own CR 903.8 gate must also reject casting a non-commander \
         object from the command zone: {:?}",
        result
    );
}

// ── T6 ───────────────────────────────────────────────────────────────────────────

/// CR 903.8: two players each have their OWN registered, affordable commander.
/// Neither player is ever offered the OTHER's commander -- the command-zone loop
/// scans only `ZoneId::Command(player)` for `player`'s own call, never every seat's
/// zone. Checked in both directions (`legal_actions(&state, P1)` and
/// `legal_actions(&state, P2)`, the second built from a clone with active
/// player/priority flipped so P2's cast is not ALSO withheld by sorcery-speed
/// timing).
#[test]
fn test_sim1_another_players_commander_is_never_offered() {
    let p1 = p(1);
    let p2 = p(2);
    let p1_cid = cid("sim1-commander-p1");
    let p2_cid = cid("sim1-commander-p2");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 P1 Commander", p1_cid, generic_cost(2));
    builder = commander_state(builder, p2, "Sim1 P2 Commander", p2_cid, generic_cost(2));
    for land in n_colorless_lands(p1, "P1 Land", 2) {
        builder = builder.object(land);
    }
    for land in n_colorless_lands(p2, "P2 Land", 2) {
        builder = builder.object(land);
    }
    let state_p1_active = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");

    let p1_commander_id = find_by_name(&state_p1_active, "Sim1 P1 Commander");
    let p2_commander_id = find_by_name(&state_p1_active, "Sim1 P2 Commander");

    let p1_actions = StubProvider.legal_actions(&state_p1_active, p1);
    assert!(
        contains_cast_spell_for(&p1_actions, p1_commander_id),
        "non-vacuity: P1's own commander must be offered to P1: {:?}",
        p1_actions
    );
    assert!(
        !contains_cast_spell_for(&p1_actions, p2_commander_id),
        "P2's commander must never appear in P1's legal actions: {:?}",
        p1_actions
    );

    // Flip active player/priority to P2 so P2's OWN cast is not withheld by
    // sorcery-speed timing while checking the reverse direction.
    let mut state_p2_active = state_p1_active;
    state_p2_active.turn_mut().active_player = p2;
    state_p2_active.turn_mut().priority_holder = Some(p2);

    let p2_actions = StubProvider.legal_actions(&state_p2_active, p2);
    assert!(
        contains_cast_spell_for(&p2_actions, p2_commander_id),
        "non-vacuity: P2's own commander must be offered to P2: {:?}",
        p2_actions
    );
    assert!(
        !contains_cast_spell_for(&p2_actions, p1_commander_id),
        "P1's commander must never appear in P2's legal actions: {:?}",
        p2_actions
    );
}

// ── T7 ───────────────────────────────────────────────────────────────────────────

/// CR 903.8: the end-to-end offer -> submit round trip through `LocalGame`. A human
/// seat is offered the commander cast, submits it with `auto_tap: true`, the engine
/// accepts it, the object leaves the command zone (onto the stack), and the tax
/// counter increments from 0 to 1. Reverting EITHER the command-zone enumeration
/// (Step 5) or the human auto-tap fix (Step 6) breaks this: with Step 5 reverted
/// there is no action to submit at all; with Step 6 reverted (and a cost that needs
/// tapping) the submit is rejected.
#[test]
fn test_sim1_casting_the_commander_increments_the_tax() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-g");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(
        builder,
        p1,
        "Sim1 Commander",
        card_id.clone(),
        generic_cost(2),
    );
    for land in n_colorless_lands(p1, "Land", 2) {
        builder = builder.object(land);
    }
    let state = builder
        .active_player(p1)
        .build()
        .expect("fixture should build");

    let commander_id = find_by_name(&state, "Sim1 Commander");
    let human_seats: BTreeSet<PlayerId> = [p1].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        HashMap::new(),
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    let decision = drive_to_commander_offer(&mut game, commander_id);
    let idx = find_action_index(&decision.actions, |a| is_cast_spell_for(a, commander_id));

    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    );
    assert!(
        result.is_ok(),
        "the cast must be accepted: {:?}",
        result.err()
    );

    let state = game.state();
    assert!(
        state
            .zones()
            .get(&ZoneId::Command(p1))
            .unwrap()
            .object_ids()
            .is_empty(),
        "the commander must have left the command zone"
    );
    assert_eq!(
        state
            .players()
            .get(&p1)
            .unwrap()
            .commander_tax
            .get(&card_id)
            .copied(),
        Some(1),
        "CR 903.8: the tax counter must increment to 1 after the first cast"
    );
}

// ── T8 ───────────────────────────────────────────────────────────────────────────

/// **`local_game.rs`'s discriminator.** Tax preset to 1 (printed {generic 2}, taxed
/// {generic 4}), pool empty, exactly 4 untapped lands (exactly enough for the TAXED
/// cost, not merely the printed one). `submit(auto_tap: true)` must succeed.
///
/// Reverting ONLY Step 6 (`auto_tap_commands_for` reading the printed cost instead of
/// `effective_cast_cost`) makes the auto-tap plan cover only 2 lands' worth, leaving
/// the taxed {generic 4} cost underpaid -- `LocalGameError::Rejected` -- while T3
/// (which never reaches `LocalGame`/`submit` at all) is unaffected by that revert.
/// That is what makes T3 and T8 the two independent discriminators the plan asks
/// for: T3 fails only if Step 1 (the offer gate) regresses; T8 fails only if Step 6
/// (the human auto-tap) regresses.
#[test]
fn test_sim1_human_auto_tap_pays_the_taxed_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-h");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(
        builder,
        p1,
        "Sim1 Commander",
        card_id.clone(),
        generic_cost(2),
    );
    for land in n_colorless_lands(p1, "Land", 4) {
        builder = builder.object(land);
    }
    let mut state = builder
        .active_player(p1)
        .build()
        .expect("fixture should build");
    set_tax(&mut state, p1, &card_id, 1);
    let commander_id = find_by_name(&state, "Sim1 Commander");

    let human_seats: BTreeSet<PlayerId> = [p1].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        HashMap::new(),
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    let decision = drive_to_commander_offer(&mut game, commander_id);
    let idx = find_action_index(&decision.actions, |a| is_cast_spell_for(a, commander_id));

    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    );
    assert!(
        result.is_ok(),
        "auto-tap must cover the FULL taxed cost (4 mana), not just the printed \
         cost (2 mana): {:?}",
        result.err()
    );
}

// ── T8b ──────────────────────────────────────────────────────────────────────────

/// The exact sentence `local_game.rs` used to carry as a known limitation
/// ("Recasting a taxed commander with a pool that covers only the printed cost
/// therefore skips tapping and the cast is rejected"). Tax preset to 1 (printed
/// {generic 2}, taxed {generic 4}). Before the CastSpell submission, the human seat
/// manually taps 2 of 6 available lands into their pool -- exactly enough to cover
/// the PRINTED cost, not the taxed one. 4 lands remain untapped.
///
/// Pre-Step-6: `auto_tap_commands_for`'s early return checked
/// `can_pay_cost(pool, printed_cost)`, which is now TRUE, so it returned `None`
/// (prepend no taps) and the cast was submitted paying only the 2-mana pool against
/// a 4-mana real cost -- rejected. Post-Step-6 the early return checks the TAXED
/// cost, which the 2-mana pool does NOT cover, so it falls through to a fresh tap
/// plan for the full taxed cost from the 4 remaining untapped lands -- accepted.
#[test]
fn test_sim1_a_pool_covering_only_the_printed_cost_does_not_skip_tapping() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-i");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(
        builder,
        p1,
        "Sim1 Commander",
        card_id.clone(),
        generic_cost(2),
    );
    // 2 lands to be manually pre-tapped into the pool (matching the printed cost),
    // plus 4 more left untapped for the post-fix auto-tap to draw on for the full
    // taxed cost.
    for land in n_colorless_lands(p1, "Pool Land", 2) {
        builder = builder.object(land);
    }
    for land in n_colorless_lands(p1, "Spare Land", 4) {
        builder = builder.object(land);
    }
    let mut state = builder
        .active_player(p1)
        .build()
        .expect("fixture should build");
    set_tax(&mut state, p1, &card_id, 1);
    let commander_id = find_by_name(&state, "Sim1 Commander");

    let human_seats: BTreeSet<PlayerId> = [p1].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        HashMap::new(),
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    // Reach the main-phase priority window first (the commander is already
    // affordable outright with 6 untapped lands, so its appearance in the action
    // list is also the "we have arrived" marker) -- then, from THAT SAME priority
    // window, tap the 2 "Pool Land" sources manually instead of casting yet.
    let _arrived = drive_to_commander_offer(&mut game, commander_id);

    for i in 1..=2 {
        let decision = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!(
                "expected AwaitingHuman before tapping land {i}, got {:?}",
                other
            ),
        };
        let land_id = find_by_name(game.state(), &format!("Pool Land {i}"));
        let idx = find_action_index(
            &decision.actions,
            |a| matches!(a, LegalAction::TapForMana { source, .. } if *source == land_id),
        );
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: idx,
                params: ActionParams::default(),
            },
        )
        .unwrap_or_else(|e| panic!("tapping Pool Land {i} into the pool failed: {:?}", e));
    }

    // Sanity check: the pool now covers the printed cost (mv 2) but not the taxed
    // cost (mv 4) -- otherwise this fixture would not be testing what it claims to.
    let pool_total = game.state().players().get(&p1).unwrap().mana_pool.total();
    assert_eq!(
        pool_total, 2,
        "the pool must hold exactly the printed cost's mana value"
    );

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman before casting, got {:?}", other),
    };
    let idx = find_action_index(&decision.actions, |a| is_cast_spell_for(a, commander_id));

    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    );
    assert!(
        result.is_ok(),
        "a pool covering only the printed cost must NOT skip tapping the remaining \
         lands needed for the taxed cost: {:?}",
        result.err()
    );
}

// ── T9 ───────────────────────────────────────────────────────────────────────────

/// **The bot-seat discriminator, `advance()`'s Step 7.** Tax preset to 1 (so the
/// FIRST cast this test drives already needs the taxed {generic 4} cost, not the
/// printed {generic 2} one -- a preset of 0 would make printed and taxed identical
/// and could not discriminate). Exactly 4 untapped lands. A purpose-built bot always
/// casts the commander when it is offered.
///
/// Reverting ONLY Step 7 (the bot-seat auto-tap in `advance()` reading the printed
/// cost) makes the bot's auto-tap plan cover only 2 lands' worth; the cast is
/// rejected, `advance()`'s `PassPriority` fallback fires, and the tax counter stays
/// at 1 forever (`HeuristicBot`/`RandomBot` would keep re-offering and re-failing the
/// identical action -- R5 of the plan -- which this purpose-built bot sidesteps by
/// design so the test measures exactly one thing).
#[test]
fn test_sim1_bot_auto_tap_pays_the_taxed_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-j");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(
        builder,
        p1,
        "Sim1 Commander",
        card_id.clone(),
        generic_cost(2),
    );
    for land in n_colorless_lands(p1, "Land", 4) {
        builder = builder.object(land);
    }
    let mut state = builder
        .active_player(p1)
        .build()
        .expect("fixture should build");
    set_tax(&mut state, p1, &card_id, 1);
    let commander_id = find_by_name(&state, "Sim1 Commander");

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        p1,
        Box::new(CastsTheCommanderBot {
            commander: commander_id,
        }),
    );
    // P2 gets no bot at all -- `advance()` auto-passes for an unassigned seat.

    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        BTreeSet::new(),
        small_limits(5),
        true,
    )
    .expect("game should start");

    let outcome = game.advance();
    assert!(
        !matches!(
            outcome,
            AdvanceOutcome::Halted(mtg_simulator::HaltReason::EngineError(_))
                | AdvanceOutcome::Halted(mtg_simulator::HaltReason::NoLegalActions { .. })
        ),
        "the bot's cast must not degrade into an engine error or a dead game: {:?}",
        outcome
    );

    let final_state = game.state();
    assert!(
        final_state
            .zones()
            .get(&ZoneId::Command(p1))
            .unwrap()
            .object_ids()
            .is_empty(),
        "the commander must have left the command zone -- the bot's cast succeeded"
    );
    assert_eq!(
        final_state
            .players()
            .get(&p1)
            .unwrap()
            .commander_tax
            .get(&card_id)
            .copied(),
        Some(2),
        "CR 903.8: preset tax 1 -> 2 after the bot's successful (taxed) cast"
    );
}

// ── T10 ──────────────────────────────────────────────────────────────────────────

/// Both bots choose the offered commander cast when it is legal.
///
/// `HeuristicBot` scores `CastSpell` at `50 + 10 * mana_value` versus `PassPriority`'s
/// 1, so with only those two actions offered it is DETERMINISTIC across seeds.
///
/// `RandomBot` picks uniformly at random between the two actions; over >= 32 seeds
/// this is a NON-VACUITY FLOOR (it returns the commander cast at least once), not an
/// equality -- a future reader must not tighten this into "every seed" or "exactly
/// half", which would make the test flaky. The second assertion (every result is
/// either the commander cast or `PassPriority`, never anything else) IS an equality,
/// and is the real point: a bot must never suggest a malformed command.
#[test]
fn test_sim1_both_bots_choose_the_offered_commander_cast() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = cid("sim1-commander-k");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 Commander", card_id, generic_cost(3));
    let state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");
    let commander_id = find_by_name(&state, "Sim1 Commander");

    let legal = vec![
        LegalAction::CastSpell {
            card: commander_id,
            from_zone: ZoneId::Command(p1),
            // UI-2: no additional-cost machinery is under test in this fixture.
            additional_costs: Default::default(),
        },
        LegalAction::PassPriority,
    ];

    // HeuristicBot: deterministic across seeds.
    for seed in [1u64, 2, 3, 42] {
        let mut bot = HeuristicBot::new(seed, "Heuristic".to_string());
        let cmd = bot.choose_action(&state, p1, &legal);
        assert!(
            matches!(cmd, Command::CastSpell(ref c) if c.card == commander_id),
            "HeuristicBot must deterministically prefer the commander cast over \
             PassPriority (seed {seed}): {:?}",
            cmd
        );
    }

    // RandomBot: a non-vacuity floor over >= 32 seeds, not an equality.
    let mut cast_count = 0u32;
    for seed in 0..32u64 {
        let mut bot = RandomBot::new(seed, "Random".to_string());
        let cmd = bot.choose_action(&state, p1, &legal);
        let is_cast = matches!(cmd, Command::CastSpell(ref c) if c.card == commander_id);
        let is_pass = matches!(cmd, Command::PassPriority { .. });
        assert!(
            is_cast || is_pass,
            "RandomBot must never return a malformed command from this two-action \
             list (seed {seed}): {:?}",
            cmd
        );
        if is_cast {
            cast_count += 1;
        }
    }
    assert!(
        cast_count >= 1,
        "over 32 seeds, RandomBot's uniform choice must pick the commander cast at \
         least once (non-vacuity floor, not an equality)"
    );
}

// ── T11 ──────────────────────────────────────────────────────────────────────────

/// CR 903.8 / CR 601.2f commutation property (plan §2.2): `apply_commander_tax`
/// writes ONLY `generic`, so flattening a hybrid/Phyrexian cost and applying the tax
/// commute, and the tax adds exactly `2 * tax` to the mana value regardless of X.
///
/// A pure unit test against `mtg_engine::apply_commander_tax` / `ManaCost` -- no
/// `GameState` needed. Re-implementing the tax locally, or flattening before taxing,
/// would break this.
#[test]
fn test_sim1_commander_tax_commutes_with_flattening_and_x() {
    let cost = ManaCost {
        generic: 1,
        hybrid: vec![HybridMana::ColorColor(ManaColor::White, ManaColor::Blue)],
        phyrexian: vec![PhyrexianMana::Single(ManaColor::Black)],
        x_count: 2,
        ..Default::default()
    };
    let tax = 3;

    let taxed = apply_commander_tax(&cost, tax);
    // The tax must be pure generic addition -- everything else field-for-field
    // identical.
    assert_eq!(
        taxed.hybrid, cost.hybrid,
        "hybrid pips must be preserved verbatim"
    );
    assert_eq!(
        taxed.phyrexian, cost.phyrexian,
        "Phyrexian pips must be preserved verbatim"
    );
    assert_eq!(
        taxed.x_count, cost.x_count,
        "x_count must be preserved verbatim"
    );
    assert_eq!(
        taxed.generic,
        cost.generic + tax * 2,
        "the tax adds exactly 2 * tax to the generic component"
    );
    assert_eq!(
        taxed.mana_value(),
        cost.mana_value() + tax * 2,
        "CR 202.3e: X is not counted off the stack, so the mana-value delta is \
         exactly 2 * tax regardless of x_count"
    );

    // flatten(tax(c)) == tax(flatten(c)) -- commutation.
    let (flat_after_tax, life_after_tax) = taxed
        .flatten_hybrid_phyrexian(&[], &[])
        .expect("flatten must succeed with default (empty) choice vectors");
    let (flat_before_tax, life_before_tax) = cost
        .flatten_hybrid_phyrexian(&[], &[])
        .expect("flatten must succeed with default (empty) choice vectors");
    let tax_after_flatten = apply_commander_tax(&flat_before_tax, tax);

    assert_eq!(
        flat_after_tax, tax_after_flatten,
        "flattening then taxing must equal taxing then flattening"
    );
    assert_eq!(
        life_after_tax, life_before_tax,
        "the tax must never change how much life a Phyrexian pip costs"
    );
}

// ── T12 ──────────────────────────────────────────────────────────────────────────

/// CR 903.8: `effective_cast_cost` is the IDENTITY for every non-command-zone cast --
/// the no-regression proof that no existing hand offer, tap plan, or recorded seed
/// can move. Two sub-cases, both in `Hand(p1)`:
///   (a) a plain hand card, never registered as a commander at all;
///   (b) a card that IS registered as a commander (CR 903.9b lets a commander end up
///       in a player's hand) with `commander_tax` preset to 5 -- proving the ZONE
///       guard, not the `commander_ids` membership, is what decides identity-vs-tax.
#[test]
fn test_sim1_effective_cast_cost_is_the_identity_for_a_hand_card() {
    let p1 = p(1);
    let p2 = p(2);
    let plain_id = cid("sim1-plain-hand-card");
    let commander_in_hand_id = cid("sim1-commander-in-hand");

    let plain_cost = generic_cost(3);
    let commander_cost = generic_cost(4);

    let plain_spec = ObjectSpec::card(p1, "Sim1 Plain Hand Card")
        .with_card_id(plain_id)
        .with_mana_cost(plain_cost.clone())
        .in_zone(ZoneId::Hand(p1));
    let commander_spec = ObjectSpec::card(p1, "Sim1 Commander In Hand")
        .with_card_id(commander_in_hand_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(commander_cost.clone())
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, commander_in_hand_id.clone())
        .object(plain_spec)
        .object(commander_spec)
        .active_player(p1)
        .build()
        .expect("fixture should build");
    set_tax(&mut state, p1, &commander_in_hand_id, 5);

    let plain_obj = find_by_name(&state, "Sim1 Plain Hand Card");
    let commander_obj = find_by_name(&state, "Sim1 Commander In Hand");

    assert_eq!(
        effective_cast_cost(&state, p1, plain_obj),
        Some(plain_cost),
        "a plain hand card's cost must be returned unchanged"
    );
    assert_eq!(
        effective_cast_cost(&state, p1, commander_obj),
        Some(commander_cost),
        "a REGISTERED commander sitting in HAND (not the command zone) must still \
         return its printed cost unchanged -- the zone guard, not commander_ids \
         membership, decides this, even with commander_tax preset to 5"
    );
}

// ── T13 ──────────────────────────────────────────────────────────────────────────

/// CR 101.2 (Drannith Magistrate): an opponent's `OpponentsCantCastFromNonHand`
/// restriction suppresses P1's command-zone offer entirely, and a hand-built
/// `Command::CastSpell` for it is rejected by the engine citing CR 101.2.
///
/// Companion assertion (stops the fix from over-suppressing): the RESTRICTION'S OWN
/// CONTROLLER (P2) casting THEIR OWN commander from THEIR OWN command zone is
/// unaffected -- CR 101.2's "opponents" language is controller-relative, not global.
#[test]
fn test_sim1_drannith_magistrate_suppresses_the_command_zone_offer() {
    let p1 = p(1);
    let p2 = p(2);
    let p1_cid = cid("sim1-commander-l1");
    let p2_cid = cid("sim1-commander-l2");

    let mut builder = GameStateBuilder::new().add_player(p1).add_player(p2);
    builder = commander_state(builder, p1, "Sim1 P1 Commander", p1_cid, generic_cost(0));
    builder = commander_state(builder, p2, "Sim1 P2 Commander", p2_cid, generic_cost(0));
    builder = builder
        .object(ObjectSpec::creature(p2, "Drannith Magistrate", 2, 2).in_zone(ZoneId::Battlefield));
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("fixture should build");

    let magistrate_id = find_by_name(&state, "Drannith Magistrate");
    state.restrictions_mut().push_back(ActiveRestriction {
        source: magistrate_id,
        controller: p2,
        restriction: GameRestriction::OpponentsCantCastFromNonHand,
    });

    let p1_commander_id = find_by_name(&state, "Sim1 P1 Commander");
    let p2_commander_id = find_by_name(&state, "Sim1 P2 Commander");

    let p1_actions = StubProvider.legal_actions(&state, p1);
    assert!(
        !contains_cast_spell_for(&p1_actions, p1_commander_id),
        "Drannith Magistrate must suppress P1's (an opponent's) command-zone offer: \
         {:?}",
        p1_actions
    );
    assert!(
        !p1_actions.is_empty(),
        "non-vacuity: some action must be offered"
    );
    assert!(
        p1_actions
            .iter()
            .any(|a| matches!(a, LegalAction::PassPriority)),
        "non-vacuity: PassPriority must be among the offered actions: {:?}",
        p1_actions
    );

    let result = process_command(state.clone(), cast_spell_cmd(p1, p1_commander_id));
    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("CR 101.2"),
                "the rejection must cite CR 101.2: {msg}"
            );
        }
        Ok(_) => panic!("the engine must reject P1's cast under Drannith Magistrate"),
    }

    // Companion: P2 (the restriction's own controller) casting P2's OWN commander is
    // unaffected -- flip active player/priority to P2, same technique as T6.
    let mut state_p2_active = state;
    state_p2_active.turn_mut().active_player = p2;
    state_p2_active.turn_mut().priority_holder = Some(p2);

    let p2_actions = StubProvider.legal_actions(&state_p2_active, p2);
    assert!(
        contains_cast_spell_for(&p2_actions, p2_commander_id),
        "the restriction's own controller casting their own commander must be \
         unaffected -- CR 101.2's 'opponents' language is controller-relative: {:?}",
        p2_actions
    );
}
