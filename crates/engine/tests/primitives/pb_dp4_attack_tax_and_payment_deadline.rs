//! PB-DP4 — DP-10 (attack tax debit) + DP-11 (echo/cumulative-upkeep/recover payment
//! deadline) primitive tests.
//!
//! `memory/primitives/pb-plan-DP4.md` §7 is authoritative for what each probe pins.
//!
//! ## DP-10 (CR 508.1c/h/i/j)
//!
//! The Propaganda/Ghostly Prison attack tax is a real, colour-correct `ManaCost` debit
//! against the declaring player's pool, computed and paid in `rules/combat.rs`. CR 508.1d's
//! "you are never required to pay an attack cost" is honoured by the must-attack "able"
//! test (goad and `MustAttackEachCombat`), closing OOS-RS3-4.
//!
//! ## DP-11 (CR 702.30a/702.24a/702.59a, CR 118.12a)
//!
//! An unanswered echo / cumulative upkeep / recover payment is closed out with the
//! CR-mandated "otherwise" branch at the end of the priority round in which the ability
//! resolved (`rules/engine.rs::force_resolve_overdue_payments`, hooked into
//! `handle_all_passed`'s stack-EMPTY branch). Tests here use two setup styles:
//! - Full-chain (echo.rs / cumulative_upkeep.rs / recover.rs pattern): build the trigger
//!   from scratch and resolve it via `pass_all`, when the point of the test is to exercise
//!   the actual trigger-production path alongside the deadline.
//! - Direct seeding via the `pending_*_payments_mut()` escape hatches: when the point of
//!   the test is the SWEEP's own behavior (ordering, termination, multiplicity) and
//!   re-deriving a full trigger-production chain would only add unrelated surface area.
//!   Both styles are legitimate per `memory/conventions.md` (GameStateBuilder + accessors,
//!   not manual struct construction) since the escape hatches exist for exactly this.

use mtg_engine::cards::card_definition::ManaRestriction;
use mtg_engine::state::stubs::ActiveRestriction;
use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId, CardRegistry,
    CardType, Command, CounterType, CumulativeUpkeepCost, Designations, GameEvent, GameRestriction,
    GameState, GameStateBuilder, HybridMana, KeywordAbility, ManaAbility, ManaColor, ManaCost,
    ManaPool, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state.objects().iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn in_exile(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|obj| obj.characteristics.name == name && matches!(obj.zone, ZoneId::Exile))
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

/// Build a DeclareAttackers command.
fn declare_cmd(player: PlayerId, attackers: Vec<(ObjectId, AttackTarget)>) -> Command {
    Command::DeclareAttackers {
        player,
        attackers,
        enlist_choices: vec![],
        exert_choices: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

fn add_restriction(
    state: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    restriction: GameRestriction,
) {
    state.restrictions_mut().push_back(ActiveRestriction {
        source,
        controller,
        restriction,
    });
}

/// A dying 2/2 creature (lethal damage marked) for recover-trigger setups.
fn dying_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 2, 2).with_damage(2)
}

// ── DP-11 card definitions (Echo {1} / cumulative upkeep {1} / recover {1}) ────────
//
// Deliberately {1} generic (not the echo.rs {2}{R} / cumulative_upkeep.rs {1} blue
// patterns) so a single land tap funds every payment in this file's mana-based tests.

fn dp4_echo_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp4-test-echo".into()),
        name: "DP4 Echo Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Echo {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Echo(ManaCost {
                generic: 1,
                ..Default::default()
            })),
            AbilityDefinition::Echo {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

fn dp4_echo_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "DP4 Echo Test Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dp4-test-echo".into()))
        .with_keyword(KeywordAbility::Echo(ManaCost {
            generic: 1,
            ..Default::default()
        }))
        .with_types(vec![CardType::Creature])
}

fn dp4_cu_mana_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp4-test-cu-mana".into()),
        name: "DP4 CU Mana Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cumulative upkeep {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(
                CumulativeUpkeepCost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
            )),
            AbilityDefinition::CumulativeUpkeep {
                cost: CumulativeUpkeepCost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
            },
        ],
        ..Default::default()
    }
}

fn dp4_cu_mana_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "DP4 CU Mana Test")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dp4-test-cu-mana".into()))
        .with_keyword(KeywordAbility::CumulativeUpkeep(
            CumulativeUpkeepCost::Mana(ManaCost {
                generic: 1,
                ..Default::default()
            }),
        ))
        .with_types(vec![CardType::Enchantment])
}

fn dp4_recover_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp4-test-recover".into()),
        name: "DP4 Recover Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Recover {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Recover),
            AbilityDefinition::Recover {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

fn dp4_recover_in_graveyard(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "DP4 Recover Test")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("dp4-test-recover".into()))
        .with_keyword(KeywordAbility::Recover)
        .with_types(vec![CardType::Sorcery])
}

// ══════════════════════════════════════════════════════════════════════════════
// §7.1 DP-10 fail-before / pass-after probes
// ══════════════════════════════════════════════════════════════════════════════

#[test]
/// CR 508.1j — the attack tax is a real DEBIT, not just an affordability check.
/// Pre-fix: declaration succeeds and the {2} tax mana is never spent.
fn test_508_1j_attack_tax_is_debited_from_the_pool() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .player_mana(
            p(2),
            ManaPool {
                colorless: 2,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    let (state, events) = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Player(p(1)))]),
    )
    .expect("a funded attacker should be allowed to attack");

    assert_eq!(
        state.player(p(2)).unwrap().mana_pool.total(),
        0,
        "CR 508.1j: the {{2}} attack tax must be debited from the pool"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaCostPaid { player, cost } if *player == p(2) && cost.generic == 2
        )),
        "a ManaCostPaid event for the {{2}} attack tax should be emitted; got {:?}",
        events
    );
}

#[test]
/// CR 508.1h / CR 106.1 — a coloured attack tax must not be flattened to a generic total.
/// Pre-fix: the restriction's `{W}{W}` is summed into a colour-blind `u32`, so `{C}{C}`
/// wrongly satisfies it.
fn test_508_1h_attack_tax_colour_is_not_flattened_to_generic() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .player_mana(
            p(2),
            ManaPool {
                colorless: 2,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                white: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    let result = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Player(p(1)))]),
    );

    let err = result.expect_err("CR 508.1h/106.1: a {W}{W} tax must not be payable with {C}{C}");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("attack tax"),
        "error should mention attack tax: {msg}"
    );
}

#[test]
/// CR 508.1j — a coloured attack tax IS payable with the matching colours.
fn test_508_1j_coloured_attack_tax_paid_with_correct_colours() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .player_mana(
            p(2),
            ManaPool {
                white: 2,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                white: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    let (state, _) = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Player(p(1)))]),
    )
    .expect("a {W}{W} tax paid with {W}{W} should be accepted");

    assert_eq!(
        state.player(p(2)).unwrap().mana_pool.white,
        0,
        "the coloured tax should have been debited in the matching colour"
    );
}

#[test]
/// CR 106.6 — restricted mana cannot pay an attack tax (no `ManaRestriction` in this
/// engine matches a non-spell cost). Pre-fix: `total_with_restricted()` counted it as
/// affordable, and nothing was ever spent anyway.
fn test_106_6_restricted_mana_cannot_pay_an_attack_tax() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    // p2's pool holds ONLY restricted mana -- zero unrestricted mana.
    state
        .players_mut()
        .get_mut(&p(2))
        .unwrap()
        .mana_pool
        .add_restricted(ManaColor::Green, 2, ManaRestriction::CreatureSpellsOnly);

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    let result = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Player(p(1)))]),
    );

    let err = result.expect_err("CR 106.6: restricted mana cannot pay a non-spell attack tax");
    let msg = format!("{:?}", err);
    // Fix cycle (E5): the message states the observable shortfall (0 unrestricted mana
    // available, despite p2's restricted-mana pool) rather than asserting the
    // engine-internal CR 106.6 rationale ("no ManaRestriction matches a non-spell cost")
    // as a player-facing fact. That rationale now lives in a code comment at the
    // rejection site (combat.rs), not in this string.
    assert!(
        msg.contains("attack tax") && msg.contains("0 unrestricted mana"),
        "message should cite attack tax and the observed (zero) unrestricted mana \
         available: {msg}"
    );
}

#[test]
/// CR 508.1h — attack taxes from multiple sources are cumulative per defender, and the
/// total is per-defender-cost x attacker-count summed across all taxed defenders.
fn test_508_1h_attack_tax_sums_per_defender_and_per_attacker() {
    let mut state = GameStateBuilder::four_player()
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(3), "Ghostly Prison", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear A", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear B", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear C", 2, 2).in_zone(ZoneId::Battlefield))
        .player_mana(
            p(2),
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    let ghostly_prison = find_by_name(&state, "Ghostly Prison");
    add_restriction(
        &mut state,
        ghostly_prison,
        p(3),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear_a = find_by_name(&state, "Bear A");
    let bear_b = find_by_name(&state, "Bear B");
    let bear_c = find_by_name(&state, "Bear C");

    let (state, events) = process_command(
        state,
        declare_cmd(
            p(2),
            vec![
                (bear_a, AttackTarget::Player(p(1))),
                (bear_b, AttackTarget::Player(p(1))),
                (bear_c, AttackTarget::Player(p(3))),
            ],
        ),
    )
    .expect("2 attackers at {2} + 1 attacker at {2} = {6}, exactly funded");

    assert_eq!(state.player(p(2)).unwrap().mana_pool.total(), 0);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::ManaCostPaid { player, cost } if *player == p(2) && cost.generic == 6
    )));
}

#[test]
/// CR 508.1c (regression guard) — attacking a planeswalker is not "attacking you", so a
/// CantAttackYouUnlessPay tax never applies to it. Must pass before AND after the fix.
fn test_508_1c_planeswalker_attack_is_not_taxed() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::planeswalker(p(1), "Test Walker", 4))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap(); // p2 has zero mana

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    let walker = find_by_name(&state, "Test Walker");
    let result = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Planeswalker(walker))]),
    );
    assert!(
        result.is_ok(),
        "CR 508.1c: attacking a planeswalker is not \"attacking you\" -- Propaganda's tax \
         does not apply: {:?}",
        result.err()
    );
}

#[test]
/// CR 107.4e/107.4f/508.1h/508.1j — **reconciled by PB-DX6 (fix cycle, Finding 2)**.
/// This test's ORIGINAL subject (a hybrid pip is invisible to the field-sum, so the
/// declaration succeeds for free) was closed by PB-DX6: hybrid pips are now payable
/// via `hybrid_choices`, and are rejected only when the declared attackers'
/// accumulated, FLATTENED cost cannot actually be paid from the pool. This test now
/// pins THAT: an insufficient pool cannot pay an otherwise-payable hybrid tax, and the
/// rejection message must be the CR 508.1h/508.1j "cannot pay the required" shape, NOT
/// the PB-DX6-superseded "is not payable" class-rejection shape (see
/// `historical_observation_c_hybrid_attack_tax_no_longer_unpayable_class` in
/// `pb_dx6_unflattened_payment_sites.rs` for that shape's own pin). Kept, not deleted
/// — the scenario (zero-mana player attacks into a taxed defender) is still worth
/// pinning, just against the new regime.
fn test_107_4e_insufficient_pool_cannot_pay_an_otherwise_payable_hybrid_attack_tax() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap(); // p2 has zero mana -- the hybrid pip must be REJECTED, not paid free

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::GenericColor(ManaColor::White)],
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    let result = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Player(p(1)))]),
    );
    let err = result.expect_err(
        "CR 508.1h/508.1j: a hybrid attack tax the player's empty pool cannot pay must \
         still be rejected",
    );
    let msg = format!("{:?}", err);
    // PB-DX6 fix cycle, Finding 2: pre-fix this asserted `msg.contains("hybrid")`,
    // which post-PB-DX6 is satisfied UNCONDITIONALLY by the `ManaCost` `Debug` impl's
    // own field name ("hybrid: []") regardless of whether the rejection reason has
    // anything to do with hybrid mana at all -- a vacuous assertion. Assert instead on
    // the CR 508.1h/508.1j "cannot pay" shape this test now actually exercises, and
    // explicitly rule out the superseded "is not payable" class-rejection message
    // (the shape `historical_observation_c_hybrid_attack_tax_no_longer_unpayable_class`
    // in `pb_dx6_unflattened_payment_sites.rs` pins the ABSENCE of).
    assert!(
        msg.contains("cannot pay the required"),
        "message should name the CR 508.1h/508.1j insufficient-pool rejection: {msg}"
    );
    assert!(
        !msg.contains("is not payable"),
        "message must NOT use the PB-DX6-superseded class-rejection shape -- hybrid is \
         payable now, this rejection is about the pool, not the pip: {msg}"
    );
}

#[test]
/// CR 508.1c/107.4e (fix cycle, E1) — an X attack tax on ONE defender must not block a
/// declaration that doesn't attack that defender at all. Pre-fix (E1): the rejection
/// fired unconditionally whenever such a restriction existed anywhere on the
/// battlefield, even for an attack against a different, untaxed defender.
///
/// **PB-DX6 fix cycle, Finding 3**: this test originally used a HYBRID restriction.
/// Post-PB-DX6, hybrid is payable and never enters `x_tax_defenders` at all -- the E1
/// scoping loop this test exists to pin is never reached for a hybrid restriction, so
/// the assertion held whether or not E1's scoping was present (verified by
/// revert-and-execute in the fix cycle: reverting the scoping loop to an unconditional
/// rejection left this test green). Switched to `x_count: 1`, the only restriction
/// shape that still reaches `x_tax_defenders` and therefore the only shape that can
/// still discriminate E1's scoping.
fn test_107_4e_x_tax_does_not_block_attacks_on_other_defenders() {
    let mut state = GameStateBuilder::four_player()
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap(); // p2 has zero mana -- irrelevant, since p2 never attacks p1

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                x_count: 1,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let bear = find_by_name(&state, "Bear");
    // Attack p3 (untaxed), not p1 (the X-tax defender).
    let result = process_command(
        state,
        declare_cmd(p(2), vec![(bear, AttackTarget::Player(p(3)))]),
    );
    assert!(
        result.is_ok(),
        "E1: an unrelated X-tax restriction on a different defender must not block \
         an attack against p3: {:?}",
        result.err()
    );
}

#[test]
/// CR 508.1c/107.4e (fix cycle, E1) — an empty attack declaration must not be blocked by
/// an unrelated X attack-tax restriction that no declared attacker engages. Pre-fix
/// (E1): the rejection fired on the mere existence of the restriction.
///
/// **PB-DX6 fix cycle, Finding 3**: switched from a hybrid restriction (which, post-
/// PB-DX6, never reaches `x_tax_defenders` and so cannot discriminate the E1 scoping
/// this test exists to pin — see the sibling test above) to `x_count: 1`, the one
/// remaining rejection class.
fn test_107_4e_x_tax_does_not_block_an_empty_declaration() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap(); // p2 has zero mana

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                x_count: 1,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let result = process_command(state, declare_cmd(p(2), vec![]));
    assert!(
        result.is_ok(),
        "E1: an empty declaration must not be rejected by an X-tax restriction no \
         declared attacker engages: {:?}",
        result.err()
    );
}

#[test]
/// CR 508.1d — a must-attack requirement can never force a payment. Pre-fix: BOTH the
/// empty declaration and the paying declaration return Err, which IS the deadlock.
fn test_508_1d_must_attack_creature_is_not_forced_to_pay_an_attack_tax() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(p(2), "Forced Goblin", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .with_keyword(KeywordAbility::MustAttackEachCombat),
        )
        .build()
        .unwrap(); // p2 has zero mana

    let mut state = state;
    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    // Declining the forced attack must be legal now (CR 508.1d).
    let empty_result = process_command(state.clone(), declare_cmd(p(2), vec![]));
    assert!(
        empty_result.is_ok(),
        "CR 508.1d: the player is not required to pay an attack cost to satisfy a \
         must-attack requirement: {:?}",
        empty_result.err()
    );

    // Attacking anyway (attempting to pay a tax with no mana) is still rejected.
    let goblin = find_by_name(&state, "Forced Goblin");
    let attack_result = process_command(
        state,
        declare_cmd(p(2), vec![(goblin, AttackTarget::Player(p(1)))]),
    );
    let attack_err =
        attack_result.expect_err("attacking with no mana to pay the tax should still be rejected");
    // Fix cycle (T5): the pre-fix assertion only checked `is_err()`, which would pass
    // against ANY unrelated rejection reason. Pin the actual cause.
    let attack_msg = format!("{:?}", attack_err);
    assert!(
        attack_msg.contains("attack tax"),
        "rejection must be the attack-tax affordability check, not some other reason: \
         {attack_msg}"
    );
}

#[test]
/// CR 508.1d / CR 701.15b — the same carve-out applies to a goaded creature.
fn test_508_1d_goaded_creature_is_not_forced_to_pay_an_attack_tax() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(2), "Goaded Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap(); // p2 has zero mana

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    let bear = find_by_name(&state, "Goaded Bear");
    state
        .objects_mut()
        .get_mut(&bear)
        .unwrap()
        .goaded_by
        .push_back(p(1));
    state.turn_mut().priority_holder = Some(p(2));

    let empty_result = process_command(state, declare_cmd(p(2), vec![]));
    assert!(
        empty_result.is_ok(),
        "CR 701.15b/508.1d: a goaded creature is not required to pay an attack cost: {:?}",
        empty_result.err()
    );
}

#[test]
/// CR 508.1d (regression guard) — with an untaxed opponent (p3/p4) available, the
/// must-attack requirement is still obeyable and must be forced.
fn test_508_1d_must_attack_still_forced_when_an_untaxed_opponent_exists() {
    let mut state = GameStateBuilder::four_player()
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(p(2), "Forced Goblin", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .with_keyword(KeywordAbility::MustAttackEachCombat),
        )
        .build()
        .unwrap(); // p2 has zero mana; p3/p4 are untaxed

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let empty_result = process_command(state, declare_cmd(p(2), vec![]));
    let err = empty_result
        .expect_err("CR 508.1d: with p3/p4 untaxed, the must-attack requirement is obeyable");
    // Closing `/review` finding 2: a bare `is_err()` here would pass against ANY unrelated
    // rejection — and this is the direction where a false pass matters most, because a
    // `has_uncosted_attack_target` that returned a blanket `false` would silently disable
    // must-attack enforcement engine-wide and this guard is what is supposed to catch it.
    // Pin the actual cause, mirroring the T5 hardening applied to the sibling probe above.
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("must attack each combat if able"),
        "the rejection must be the must-attack requirement itself (CR 508.1d), not some \
         other reason: {msg}"
    );
}

#[test]
/// CR 508.1d (regression guard) — even when every opponent PLAYER is taxed, an opponent
/// planeswalker is still an uncosted target (CR 508.1c), so the requirement is forced.
fn test_508_1d_must_attack_still_forced_when_only_an_opponent_planeswalker_is_untaxed() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Propaganda", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::planeswalker(p(1), "Test Walker", 4))
        .object(
            ObjectSpec::creature(p(2), "Forced Goblin", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .with_keyword(KeywordAbility::MustAttackEachCombat),
        )
        .build()
        .unwrap(); // p2 has zero mana; p1 (the only opponent) is taxed but controls a pw

    let propaganda = find_by_name(&state, "Propaganda");
    add_restriction(
        &mut state,
        propaganda,
        p(1),
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p(2));

    let empty_result = process_command(state, declare_cmd(p(2), vec![]));
    let err = empty_result.expect_err(
        "CR 508.1c: the untaxed planeswalker is a free target, so the requirement is obeyable",
    );
    // Closing `/review` finding 2 — see the sibling guard above for why a bare `is_err()`
    // is not good enough here.
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("must attack each combat if able"),
        "the rejection must be the must-attack requirement itself (CR 508.1d), not some \
         other reason: {msg}"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// §7.2 DP-11 fail-before / pass-after probes
// ══════════════════════════════════════════════════════════════════════════════

#[test]
/// CR 702.30a / CR 118.12a — an unanswered echo payment is sacrificed at the priority
/// round boundary. Pre-fix: the permanent survives forever and the step simply advances.
fn test_702_30a_unanswered_echo_is_sacrificed_at_the_round_boundary() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp4_echo_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(dp4_echo_on_battlefield(p1))
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, "DP4 Echo Test Creature");
    state
        .objects_mut()
        .get_mut(&obj_id)
        .unwrap()
        .designations
        .insert(Designations::ECHO_PENDING);
    state.turn_mut().priority_holder = Some(p1);

    // Untap -> Upkeep: EchoTrigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::Upkeep);

    // Resolve the trigger: EchoPaymentRequired emitted, entry queued. p1 has no mana.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    assert!(resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::EchoPaymentRequired { .. })));
    assert!(!state.pending_echo_payments().is_empty());

    // Pass again WITHOUT sending PayEcho -- crosses the priority-round boundary.
    let (state, boundary_events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "DP4 Echo Test Creature"),
        "CR 702.30a/118.12a: an unanswered echo payment must be sacrificed at the round \
         boundary"
    );
    assert!(in_graveyard(&state, "DP4 Echo Test Creature", p1));
    assert!(boundary_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. })));
    assert!(state.pending_echo_payments().is_empty());
    assert_eq!(
        state.turn().step,
        Step::Upkeep,
        "the sweep re-grants priority in the SAME step rather than advancing"
    );
}

#[test]
/// CR 702.24a / CR 118.12a — an unanswered cumulative upkeep payment is sacrificed at
/// the round boundary.
fn test_702_24a_unanswered_cumulative_upkeep_is_sacrificed_at_the_round_boundary() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp4_cu_mana_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(dp4_cu_mana_on_battlefield(p1))
        .build()
        .unwrap();

    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::Upkeep);

    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    assert!(resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::CumulativeUpkeepPaymentRequired { .. })));
    assert!(!state.pending_cumulative_upkeep_payments().is_empty());

    let (state, boundary_events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "DP4 CU Mana Test"),
        "CR 702.24a/118.12a: an unanswered cumulative upkeep payment must be sacrificed \
         at the round boundary"
    );
    assert!(in_graveyard(&state, "DP4 CU Mana Test", p1));
    assert!(boundary_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. })));
    assert!(state.pending_cumulative_upkeep_payments().is_empty());
}

#[test]
/// CR 702.59a / CR 118.12a — an unanswered recover payment is exiled at the round
/// boundary.
fn test_702_59a_unanswered_recover_card_is_exiled_at_the_round_boundary() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp4_recover_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dp4_recover_in_graveyard(p1))
        .object(dying_creature(p1, "DP4 Dying Bear"))
        .build()
        .unwrap();

    state.turn_mut().priority_holder = Some(p1);

    // Both pass -> SBA kills the Bear -> RecoverTrigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Both pass -> trigger resolves -> RecoverPaymentRequired.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    assert!(resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::RecoverPaymentRequired { .. })));
    assert!(!state.pending_recover_payments().is_empty());

    // Pass again WITHOUT sending PayRecover -- crosses the boundary.
    let (state, boundary_events) = pass_all(state, &[p1, p2]);

    assert!(!in_graveyard(&state, "DP4 Recover Test", p1));
    assert!(
        in_exile(&state, "DP4 Recover Test"),
        "CR 702.59a/118.12a: an unanswered recover payment must be exiled at the round \
         boundary"
    );
    assert!(boundary_events
        .iter()
        .any(|e| matches!(e, GameEvent::RecoverDeclined { .. })));
    assert!(state.pending_recover_payments().is_empty());
}

#[test]
/// CR 702.30a (regression guard) — an echo payment PAID before the boundary must not be
/// eaten by the deadline sweep.
fn test_702_30a_echo_paid_before_the_boundary_still_survives() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp4_echo_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(dp4_echo_on_battlefield(p1))
        // A library card so the Draw step (crossed below) doesn't lose the game.
        .object(ObjectSpec::card(p1, "DP4 Library Filler").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, "DP4 Echo Test Creature");
    state
        .objects_mut()
        .get_mut(&obj_id)
        .unwrap()
        .designations
        .insert(Designations::ECHO_PENDING);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::Upkeep);
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let perm_id = find_in_zone(&state, "DP4 Echo Test Creature", ZoneId::Battlefield).unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let (state, _) = process_command(
        state,
        Command::PayEcho {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("PayEcho should succeed");

    // Cross what would have been the boundary -- twice, per the plan's probe 14.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "DP4 Echo Test Creature"),
        "a PAID echo must not be eaten by the deadline sweep"
    );
}

#[test]
/// CR 117.4 — the anti-deadlock pin. From the post-sweep state (echo just declined at
/// the boundary), one more pass round must advance the step normally: the sweep drains
/// every entry every time it runs, so the extra round terminates.
fn test_dp11_boundary_sweep_does_not_deadlock_the_priority_round() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp4_echo_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(dp4_echo_on_battlefield(p1))
        .object(ObjectSpec::card(p1, "DP4 Library Filler").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, "DP4 Echo Test Creature");
    state
        .objects_mut()
        .get_mut(&obj_id)
        .unwrap()
        .designations
        .insert(Designations::ECHO_PENDING);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::Upkeep);
    let (state, _) = pass_all(state, &[p1, p2]); // resolve trigger
    let (state, _) = pass_all(state, &[p1, p2]); // cross boundary: sweep sacrifices

    assert_eq!(
        state.turn().step,
        Step::Upkeep,
        "the sweep should not have advanced the step yet"
    );
    assert!(state.pending_echo_payments().is_empty());

    // One more round: the pending vector is now empty, so this round must be a normal
    // advance, not another sweep round -- proving the extra round terminates.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn().step,
        Step::Draw,
        "the priority round must terminate and advance normally"
    );
    assert_eq!(state.turn().priority_holder, Some(p1));
}

#[test]
/// CR 400.7 (fix cycle, T3; plan risk 4) — an all-no-op sweep (every outstanding entry's
/// permanent/card has already left its zone by the time the sweep runs) must fall
/// through and advance the step normally, NOT re-grant priority for another round. The
/// guard in `handle_all_passed` is `!payment_events.is_empty()`, not "an entry was
/// consumed" -- a no-op decline consumes the entry from the pending vector but produces
/// zero events (`handle_pay_echo` / `handle_pay_recover` both short-circuit to
/// `Ok(vec![])` when the permanent/card is no longer where the trigger left it). Getting
/// this backwards produces an infinite priority round with no error (the plan's own
/// characterization of the highest-consequence failure this design must avoid). Every
/// probe elsewhere in this file exercises the CASE WHERE THE SWEEP PRODUCES EVENTS
/// (probe 19's second, no-op entry rides along with a first entry that DOES produce
/// events) -- this is the first probe where the sweep produces NONE.
fn test_dp11_all_no_op_sweep_falls_through_and_advances() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(ObjectSpec::card(p1, "DP4 Echo Already Gone").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "DP4 Library Filler").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    // The permanent already left the battlefield (CR 400.7) -- `handle_pay_echo`'s
    // `source_info` guard finds it absent from `ZoneId::Battlefield` and returns
    // `Ok(vec![])` without emitting anything. The pending entry is still consumed
    // (removed from the vector), but that consumption produces zero events.
    let gone = find_by_name(&state, "DP4 Echo Already Gone");
    state.pending_echo_payments_mut().push_back((
        p1,
        gone,
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state.pending_echo_payments().is_empty(),
        "the no-op entry must still be consumed (removed) even though it produced no events"
    );
    assert_eq!(
        state.turn().step,
        Step::Draw,
        "T3 / plan risk 4: an all-no-op sweep must fall through to the normal step \
         advance, not re-grant priority for another round in Upkeep -- doing the latter \
         with a guard that never empties again is an infinite priority round with no error"
    );
    assert_eq!(state.turn().priority_holder, Some(p1));
}

#[test]
/// CR 101.4 — when multiple players simultaneously owe a pay-or-lose-it payment, the
/// sweep resolves them in APNAP order.
///
/// Fix cycle (T1): the ownership is INVERTED relative to the pre-fix version of this
/// test (which had p1 -- first in APNAP -- owe the ECHO, the first kind the sweep's
/// per-player loop visits, and p3 -- later in APNAP -- owe the RECOVER, the last kind
/// visited). That assignment could not discriminate a true per-player-APNAP-outer-loop
/// implementation from a hypothetical kind-grouped-globally one (process every player's
/// echo first in APNAP order, then every cumulative upkeep, then every recover): both
/// would have produced CreatureDied before RecoverDeclined. Here p3 owes the ECHO and
/// p1 owes the RECOVER, so the two hypotheses predict OPPOSITE orders --
/// per-player-APNAP visits p1 (recover) before p3 (echo), so RecoverDeclined must
/// precede CreatureDied; kind-grouped-globally would visit all echoes (p3's) before any
/// recovers (p1's), producing CreatureDied first. The assertion below fails under the
/// kind-grouped hypothesis and passes under the real (per-player-outer) implementation.
fn test_101_4_multiple_outstanding_payments_resolve_in_apnap_order() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ObjectSpec::creature(p3, "DP4 Echo Owed By P3", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p1, "DP4 Recover Owed By P1").in_zone(ZoneId::Graveyard(p1)))
        .build()
        .unwrap();

    let echo_perm = find_by_name(&state, "DP4 Echo Owed By P3");
    let recover_card = find_by_name(&state, "DP4 Recover Owed By P1");

    // Seed both pending payments directly -- both outstanding simultaneously, exactly
    // the scenario the sweep must resolve in one pass, APNAP order (CR 101.4).
    state.pending_echo_payments_mut().push_back((
        p3,
        echo_perm,
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    state.pending_recover_payments_mut().push_back((
        p1,
        recover_card,
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    state.turn_mut().priority_holder = Some(p1);

    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    assert!(
        !on_battlefield(&state, "DP4 Echo Owed By P3"),
        "p3's unanswered echo must be sacrificed"
    );
    assert!(
        in_exile(&state, "DP4 Recover Owed By P1"),
        "p1's unanswered recover must be exiled"
    );

    let creature_died_idx = events
        .iter()
        .position(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .expect("CreatureDied for p3's echo");
    let recover_declined_idx = events
        .iter()
        .position(|e| matches!(e, GameEvent::RecoverDeclined { .. }))
        .expect("RecoverDeclined for p1's recover");
    assert!(
        recover_declined_idx < creature_died_idx,
        "CR 101.4: APNAP order visits p1 before p3, so p1's RecoverDeclined must precede \
         p3's CreatureDied in the returned event vector. (This is the discriminating \
         direction -- see the test doc comment for why the pre-fix assignment could not \
         tell APNAP-outer-loop apart from kind-grouped-globally.)"
    );

    assert!(state.pending_echo_payments().is_empty());
    assert!(state.pending_recover_payments().is_empty());
}

#[test]
/// CR 117.3c / OOS-DP1-1 — answering a resolution-time payment does not reassign
/// priority, even when the owing player is NOT the one currently holding it.
fn test_dp11_answering_a_payment_does_not_reassign_priority() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ObjectSpec::card(p3, "DP4 Recover Owed By P3 Again").in_zone(ZoneId::Graveyard(p3)))
        .build()
        .unwrap();

    let recover_card = find_by_name(&state, "DP4 Recover Owed By P3 Again");
    state.pending_recover_payments_mut().push_back((
        p3,
        recover_card,
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));

    // p2 (non-active, not the owing player) currently holds priority, mid-round. p1
    // has already passed to reach p2 -- players_passed is seeded NON-EMPTY (fix
    // cycle, T2: the pre-fix version seeded `OrdSet::new()` and then asserted
    // `is_empty()`, which the deleted bodge (`players_passed = OrdSet::new()`) would
    // ALSO have produced -- that half of the assertion was vacuous. Seeding {p1}
    // and asserting it survives UNCHANGED is the discriminating form: the bodge
    // would have wiped it to empty).
    state.turn_mut().priority_holder = Some(p2);
    state.turn_mut().players_passed = imbl::OrdSet::unit(p1);

    let (state, _) = process_command(
        state,
        Command::PayRecover {
            player: p3,
            recover_card,
            pay: false,
        },
    )
    .expect("PayRecover should succeed even though p3 does not hold priority");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c/OOS-DP1-1: answering an out-of-band resolution-time payment must not \
         reassign priority"
    );
    assert_eq!(
        state.turn().players_passed,
        imbl::OrdSet::unit(p1),
        "the pass set must survive UNCHANGED -- the deleted bodge wrote \
         players_passed = OrdSet::new(), which would have wiped p1's pass out of the set"
    );
}

#[test]
/// CR 119.4 — a cumulative-upkeep life-cost payment beyond the player's life total is
/// rejected. Pre-fix: the arm has no affordability check and would drive the player
/// below 0.
fn test_119_4_cumulative_upkeep_life_cost_beyond_life_total_is_rejected() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::creature(p1, "DP4 CU Life Permanent", 0, 4)
                .in_zone(ZoneId::Battlefield)
                .with_counter(CounterType::Age, 2),
        )
        .build()
        .unwrap();

    let perm = find_by_name(&state, "DP4 CU Life Permanent");
    state.pending_cumulative_upkeep_payments_mut().push_back((
        p1,
        perm,
        CumulativeUpkeepCost::Life(3),
    ));
    state.players_mut().get_mut(&p1).unwrap().life_total = 5;
    state.turn_mut().priority_holder = Some(p1);

    // 2 age counters x 3 life = 6 life owed; life_total is 5.
    let result = process_command(
        state,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm,
            pay: true,
        },
    );
    let err = result.expect_err("CR 119.4: paying more life than the player has must be rejected");
    match err {
        mtg_engine::GameStateError::InsufficientLife {
            required, actual, ..
        } => {
            assert_eq!(required, 6);
            assert_eq!(actual, 5);
        }
        other => panic!("expected InsufficientLife, got {other:?}"),
    }
}

#[test]
/// CR 119.4b (regression guard) — a life cost of 0 is always payable, no matter the
/// life total.
fn test_119_4b_cumulative_upkeep_zero_life_cost_is_always_payable() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::creature(p1, "DP4 CU Zero Life Permanent", 0, 4)
                .in_zone(ZoneId::Battlefield)
                .with_counter(CounterType::Age, 5),
        )
        .build()
        .unwrap();

    let perm = find_by_name(&state, "DP4 CU Zero Life Permanent");
    state.pending_cumulative_upkeep_payments_mut().push_back((
        p1,
        perm,
        CumulativeUpkeepCost::Life(0),
    ));
    state.players_mut().get_mut(&p1).unwrap().life_total = 1;
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm,
            pay: true,
        },
    )
    .expect("CR 119.4b: a life cost of 0 is always payable, regardless of life total");
    assert!(on_battlefield(&state, "DP4 CU Zero Life Permanent"));
}

#[test]
/// CR 400.7 (+ CR 702.24a's "otherwise, sacrifice it") — when a permanent has two
/// outstanding cumulative-upkeep entries, the sweep's snapshot-then-mutate discipline
/// drains BOTH: the first forced decline sacrifices the permanent, and the second entry
/// is consumed silently because the permanent has already left (CR 400.7).
///
/// Closing `/review` finding 6: this doc comment used to cite **CR 702.24b** (multiple
/// instances of cumulative upkeep are separate abilities, each counting all age counters
/// on the permanent). That rule is *not* what this test exercises — it seeds two entries
/// into the pending vector directly and pins the sweep's drain discipline, never the
/// age-counter arithmetic. The cite is corrected rather than the test rewritten, because
/// the drain discipline is the property worth pinning here (it is plan risk 13: a sweep
/// that iterated the live vector while the handlers mutate it would skip entries).
/// CR 702.24b's own arithmetic remains covered by the cumulative-upkeep mechanics suite.
fn test_400_7_two_cumulative_upkeep_entries_both_reach_the_boundary() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::creature(p1, "DP4 Two CU Permanent", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .with_counter(CounterType::Age, 2),
        )
        .build()
        .unwrap();

    let perm = find_by_name(&state, "DP4 Two CU Permanent");
    state.pending_cumulative_upkeep_payments_mut().push_back((
        p1,
        perm,
        CumulativeUpkeepCost::Mana(ManaCost {
            generic: 1,
            ..Default::default()
        }),
    ));
    state.pending_cumulative_upkeep_payments_mut().push_back((
        p1,
        perm,
        CumulativeUpkeepCost::Mana(ManaCost {
            generic: 2,
            ..Default::default()
        }),
    ));
    state.turn_mut().priority_holder = Some(p1);
    assert_eq!(state.pending_cumulative_upkeep_payments().len(), 2);

    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "DP4 Two CU Permanent"),
        "the first forced decline sacrifices the permanent"
    );
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 1,
        "CR 400.7: the second entry finds the permanent already gone and produces no \
         events"
    );
    assert!(
        state.pending_cumulative_upkeep_payments().is_empty(),
        "the sweep's snapshot-then-mutate discipline must drain BOTH entries"
    );
}

#[test]
/// CR 608.2g (regression guard) — the player may activate a mana ability during the
/// payment window (before the boundary) to fund the payment.
fn test_608_2g_mana_ability_during_the_payment_window_still_funds_the_payment() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp4_echo_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(dp4_echo_on_battlefield(p1))
        .object(
            ObjectSpec::land(p1, "DP4 Untapped Land")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Colorless)),
        )
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, "DP4 Echo Test Creature");
    state
        .objects_mut()
        .get_mut(&obj_id)
        .unwrap()
        .designations
        .insert(Designations::ECHO_PENDING);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn().step, Step::Upkeep);
    let (state, _) = pass_all(state, &[p1, p2]);

    let perm_id = find_in_zone(&state, "DP4 Echo Test Creature", ZoneId::Battlefield).unwrap();
    let land_id = find_by_name(&state, "DP4 Untapped Land");

    // Fund the payment via a mana ability activated DURING the payment window (CR 608.2g).
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: land_id,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("TapForMana should succeed during the payment window");
    assert!(state.player(p1).unwrap().mana_pool.total() >= 1);

    let (state, pay_events) = process_command(
        state,
        Command::PayEcho {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("PayEcho should succeed after funding via a mana ability in the payment window");

    assert!(on_battlefield(&state, "DP4 Echo Test Creature"));
    assert!(pay_events
        .iter()
        .any(|e| matches!(e, GameEvent::EchoPaid { .. })));
}
