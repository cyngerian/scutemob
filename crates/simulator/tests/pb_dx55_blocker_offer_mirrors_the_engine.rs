//! PB-DX55 Half 2 (`OOS-SIM5-3`, and `OOS-DX51-3` inside it) — the simulator side:
//! `StubProvider`'s `DeclareBlockers` offer mirrors the engine's real per-pair
//! legality, and `RandomBot`'s blocker picker never submits a declaration the engine
//! will refuse for a reason a per-pair predicate cannot express (CR 702.110a menace).
//!
//! `crates/engine/tests/primitives/pb_dx55_block_legality_query.rs` proves the
//! ENGINE side (`check_block_pair` / `validate_block_declaration` /
//! `queries::legal_blocks`) against `handle_declare_blockers` directly; this file
//! proves the SIMULATOR consumes that query faithfully, at the offer layer AND at
//! the bot's picking layer.

use mtg_engine::{
    process_command, AttackTarget, CombatState, Command, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use mtg_simulator::{Bot, LegalAction, LegalActionProvider, RandomBot, StubProvider};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

fn declare_blockers_from(actions: &[LegalAction]) -> Option<&LegalAction> {
    actions
        .iter()
        .find(|a| matches!(a, LegalAction::DeclareBlockers { .. }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// The offer mirrors `queries::legal_blocks` exactly (regression guard against a
// future raw battlefield re-scan reappearing at the offer layer)
// ═══════════════════════════════════════════════════════════════════════════════

/// `StubProvider`'s `DeclareBlockers.legal_blocks` for a real, mixed legal/illegal
/// fixture equals `queries::legal_blocks` computed independently against the same
/// state — the offer is not merely SHAPED like the query's output, it IS the query's
/// output, unaltered.
#[test]
fn t1_offer_legal_blocks_equals_the_engine_query() {
    let p1 = p(1);
    let p2 = p(2);

    let flyer = ObjectSpec::creature(p1, "O Flyer", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Flying);
    let grounder = ObjectSpec::creature(p1, "O Grounder", 2, 2).in_zone(ZoneId::Battlefield);
    let ground_blocker =
        ObjectSpec::creature(p2, "O Ground Blocker", 2, 2).in_zone(ZoneId::Battlefield);
    let flying_blocker = ObjectSpec::creature(p2, "O Flying Blocker", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Flying);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(flyer)
        .object(grounder)
        .object(ground_blocker)
        .object(flying_blocker)
        .build()
        .unwrap();

    let flyer_id = find_by_name(&state, "O Flyer");
    let grounder_id = find_by_name(&state, "O Grounder");

    let mut combat = CombatState::new(p1);
    combat.add_attacker(flyer_id, AttackTarget::Player(p2));
    combat.add_attacker(grounder_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);
    state.turn_mut().priority_holder = Some(p2);

    let actions = StubProvider.legal_actions(&state, p2);
    let LegalAction::DeclareBlockers { legal_blocks, .. } =
        declare_blockers_from(&actions).expect("DeclareBlockers must be offered")
    else {
        unreachable!()
    };

    let mut offer_sorted: Vec<(ObjectId, Vec<ObjectId>)> = legal_blocks.clone();
    offer_sorted.sort_by_key(|(id, _)| *id);
    let mut offer_sorted_inner = offer_sorted.clone();
    for (_, atks) in &mut offer_sorted_inner {
        atks.sort();
    }

    let mut expected = mtg_engine::rules::queries::legal_blocks(&state, p2);
    expected.sort_by_key(|(id, _)| *id);
    for (_, atks) in &mut expected {
        atks.sort();
    }

    assert_eq!(
        offer_sorted_inner, expected,
        "StubProvider's DeclareBlockers.legal_blocks must equal queries::legal_blocks \
         computed independently against the same state"
    );
    // Non-vacuity: the fixture must actually contain a nonempty, non-trivial offer
    // (both a legal and an excluded pair), or the equality above would hold for the
    // uninteresting reason that both sides are empty.
    assert!(
        !expected.is_empty(),
        "non-vacuity: this fixture must produce a non-empty offer"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 509.1a (`OOS-DX51-3`) — the attacking player is never offered DeclareBlockers,
// even with an UNTAPPED creature (PB-DX51's own probe used a tapped attacker and so
// never actually exercised this)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn t2_attacking_player_never_offered_declare_blockers_with_an_untapped_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // UNTAPPED, unlike PB-DX51's `pb_dx51_blocker_offer.rs::b1` fixture, which
    // deliberately tapped its attacker to sidestep this exact gap (see that file's
    // own comment) -- this is the fixture that closes it.
    let attacker = ObjectSpec::creature(p1, "T2 Attacker", 2, 2).in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p2, "T2 Blocker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(attacker)
        .object(blocker)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "T2 Attacker");

    let mut combat = CombatState::new(p1);
    combat.add_attacker(attacker_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);

    // Non-vacuity: p2 (the real defender) DOES get the offer against this board.
    state.turn_mut().priority_holder = Some(p2);
    let p2_actions = StubProvider.legal_actions(&state, p2);
    assert!(
        declare_blockers_from(&p2_actions).is_some(),
        "non-vacuity: the real defending player must be offered DeclareBlockers here"
    );

    // p1, the attacking player, controls an untapped "T2 Attacker" itself, which
    // would otherwise pass every other per-pair guard. Must still be offered NOTHING.
    state.turn_mut().priority_holder = Some(p1);
    let p1_actions = StubProvider.legal_actions(&state, p1);
    assert!(
        declare_blockers_from(&p1_actions).is_none(),
        "CR 509.1a (`OOS-DX51-3`): the attacking player must never be offered \
         DeclareBlockers, even with an untapped creature of its own: {p1_actions:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 702.110a — the bot's prune drops a lone blocker on a menace attacker rather
// than submitting a declaration the engine refuses
// ═══════════════════════════════════════════════════════════════════════════════

/// With exactly ONE eligible blocker available against a menace attacker, no
/// declaration can legally assign it alone (CR 702.110a needs two or more). Across
/// many RNG seeds, `RandomBot` must never produce a `Command::DeclareBlockers`
/// containing that lone assignment -- and every command it DOES produce must be
/// accepted by the real engine (asserted by RESOLUTION: `process_command`, not
/// merely by the shape of the returned `Command`).
#[test]
fn t3_bot_prune_drops_a_lone_blocker_on_a_menace_attacker() {
    let p1 = p(1);
    let p2 = p(2);

    let mut saw_a_zero_block_outcome = false;
    let mut saw_the_bot_actually_run_the_arm = false;

    for seed in 0u64..80 {
        let attacker = ObjectSpec::creature(p1, "Menace Attacker", 2, 2)
            .in_zone(ZoneId::Battlefield)
            .with_keyword(KeywordAbility::Menace);
        let blocker = ObjectSpec::creature(p2, "Lone Candidate", 5, 5).in_zone(ZoneId::Battlefield);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .at_step(Step::DeclareBlockers)
            .object(attacker)
            .object(blocker)
            .build()
            .unwrap();

        let attacker_id = find_by_name(&state, "Menace Attacker");
        let blocker_id = find_by_name(&state, "Lone Candidate");

        let mut combat = CombatState::new(p1);
        combat.add_attacker(attacker_id, AttackTarget::Player(p2));
        *state.combat_mut() = Some(combat);
        state.turn_mut().priority_holder = Some(p2);

        let actions = StubProvider.legal_actions(&state, p2);
        let action = declare_blockers_from(&actions)
            .unwrap_or_else(|| panic!("seed {seed}: DeclareBlockers must be offered"));
        // Non-vacuity precondition: the per-pair offer DOES include this pairing
        // (menace is a batch guard, invisible to `check_block_pair`/`legal_blocks`),
        // so a bot with no menace awareness at all would sometimes pick it.
        if let LegalAction::DeclareBlockers { legal_blocks, .. } = action {
            assert!(
                legal_blocks
                    .iter()
                    .any(|(b, atks)| *b == blocker_id && atks.contains(&attacker_id)),
                "seed {seed}: the per-pair offer must include the (illegal-alone) \
                 pairing so this probe is non-vacuous: {legal_blocks:?}"
            );
        } else {
            unreachable!()
        }

        let mut bot = RandomBot::new(seed, "Bot-2".to_string());
        let cmd = bot.choose_action(&state, p2, std::slice::from_ref(action));
        saw_the_bot_actually_run_the_arm = true;

        let blockers = match &cmd {
            Command::DeclareBlockers { blockers, .. } => blockers.clone(),
            other => panic!("seed {seed}: expected Command::DeclareBlockers, got {other:?}"),
        };
        let count_on_menace_attacker = blockers.iter().filter(|(_, a)| *a == attacker_id).count();
        assert_ne!(
            count_on_menace_attacker, 1,
            "seed {seed}: CR 702.110a: the bot must never submit exactly ONE blocker \
             against a menace attacker (the prune must drop a lone candidate rather \
             than let the declaration reach the engine and be refused): {blockers:?}"
        );
        if count_on_menace_attacker == 0 {
            saw_a_zero_block_outcome = true;
        }

        // Whatever the bot produced, the REAL engine must accept it -- asserted by
        // RESOLUTION (a real `process_command`), not by re-deriving legality here.
        let (after, _events) = process_command(state, cmd).unwrap_or_else(|e| {
            panic!(
                "seed {seed}: the engine refused the bot's own \
                 declaration, which the offer+prune pipeline must never produce: {e:?}"
            )
        });
        // And the resolved state genuinely reflects what was declared: either the
        // attacker has zero recorded blockers, or the SET-level guard was
        // satisfied by construction (impossible here with only one candidate, so
        // this branch is unreachable in THIS fixture and is asserted as such).
        let recorded = after
            .combat()
            .as_ref()
            .map(|c| c.blockers.values().filter(|&&a| a == attacker_id).count())
            .unwrap_or(0);
        assert_eq!(
            recorded, count_on_menace_attacker,
            "seed {seed}: the engine's own recorded blocker count must match the \
             submitted command"
        );
    }

    assert!(
        saw_the_bot_actually_run_the_arm,
        "non-vacuity floor: the bot must have actually run the DeclareBlockers arm at \
         least once across 80 seeds"
    );
    assert!(
        saw_a_zero_block_outcome,
        "non-vacuity floor: with only one candidate and no way to satisfy menace, at \
         least one of 80 seeds must have pruned it down to zero blockers on the \
         menace attacker (otherwise this probe could pass by the bot simply never \
         picking the lone candidate at all, which asserts nothing about the prune)"
    );
}

/// Companion positive case: with TWO eligible blockers, the bot CAN legally assign
/// both to the menace attacker, and when it happens to pick both, the prune must
/// NOT remove them (the prune is "drop a lone blocker", not "never block menace").
#[test]
fn t3b_two_candidates_can_both_legally_block_the_menace_attacker() {
    let p1 = p(1);
    let p2 = p(2);
    let mut saw_two = false;

    for seed in 0u64..200 {
        let attacker = ObjectSpec::creature(p1, "Menace Attacker 2", 2, 2)
            .in_zone(ZoneId::Battlefield)
            .with_keyword(KeywordAbility::Menace);
        let blocker_a = ObjectSpec::creature(p2, "Candidate A", 1, 1).in_zone(ZoneId::Battlefield);
        let blocker_b = ObjectSpec::creature(p2, "Candidate B", 1, 1).in_zone(ZoneId::Battlefield);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .at_step(Step::DeclareBlockers)
            .object(attacker)
            .object(blocker_a)
            .object(blocker_b)
            .build()
            .unwrap();

        let attacker_id = find_by_name(&state, "Menace Attacker 2");

        let mut combat = CombatState::new(p1);
        combat.add_attacker(attacker_id, AttackTarget::Player(p2));
        *state.combat_mut() = Some(combat);
        state.turn_mut().priority_holder = Some(p2);

        let actions = StubProvider.legal_actions(&state, p2);
        let action = declare_blockers_from(&actions)
            .unwrap_or_else(|| panic!("seed {seed}: DeclareBlockers must be offered"));

        let mut bot = RandomBot::new(seed, "Bot-2".to_string());
        let cmd = bot.choose_action(&state, p2, std::slice::from_ref(action));
        let blockers = match &cmd {
            Command::DeclareBlockers { blockers, .. } => blockers.clone(),
            other => panic!("seed {seed}: expected Command::DeclareBlockers, got {other:?}"),
        };
        let count_on_menace_attacker = blockers.iter().filter(|(_, a)| *a == attacker_id).count();
        assert_ne!(
            count_on_menace_attacker, 1,
            "seed {seed}: never exactly one, even with two candidates available: \
             {blockers:?}"
        );
        if count_on_menace_attacker == 2 {
            saw_two = true;
        }

        process_command(state, cmd).unwrap_or_else(|e| {
            panic!("seed {seed}: the engine refused the bot's own declaration: {e:?}")
        });
    }

    assert!(
        saw_two,
        "non-vacuity floor: across 200 seeds, at least one must have both candidates \
         chosen to block the menace attacker, proving the prune does not remove a \
         legally-satisfied menace assignment"
    );
}
