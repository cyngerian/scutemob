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
    assert!(
        msg.contains("attack tax") && msg.contains("106.6"),
        "message should cite both attack tax and CR 106.6: {msg}"
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
/// CR 107.4e/107.4f via CR 508.1h — a hybrid attack tax is REJECTED, not silently paid
/// for free. Pre-fix: the hybrid pip is invisible to the field-sum, so the declaration
/// succeeds for free.
fn test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free() {
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
    let err = result
        .expect_err("CR 107.4e/107.4f: a hybrid attack tax must be rejected, not silently free");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("attack tax") && msg.contains("OOS-DP4-1"),
        "message should mention attack tax and the seed: {msg}"
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
    assert!(
        attack_result.is_err(),
        "attacking with no mana to pay the tax should still be rejected"
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
    assert!(
        empty_result.is_err(),
        "CR 508.1d: with p3/p4 untaxed, the must-attack requirement is still obeyable \
         and must be forced"
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
    assert!(
        empty_result.is_err(),
        "CR 508.1c: attacking the untaxed planeswalker is a free target, so the \
         must-attack requirement is still obeyable and must be forced"
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
/// CR 101.4 — when multiple players simultaneously owe a pay-or-lose-it payment, the
/// sweep resolves them in APNAP order.
fn test_101_4_multiple_outstanding_payments_resolve_in_apnap_order() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ObjectSpec::creature(p1, "DP4 Echo Owed By P1", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p3, "DP4 Recover Owed By P3").in_zone(ZoneId::Graveyard(p3)))
        .build()
        .unwrap();

    let echo_perm = find_by_name(&state, "DP4 Echo Owed By P1");
    let recover_card = find_by_name(&state, "DP4 Recover Owed By P3");

    // Seed both pending payments directly -- both outstanding simultaneously, exactly
    // the scenario the sweep must resolve in one pass, APNAP order (CR 101.4).
    state.pending_echo_payments_mut().push_back((
        p1,
        echo_perm,
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    state.pending_recover_payments_mut().push_back((
        p3,
        recover_card,
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    state.turn_mut().priority_holder = Some(p1);

    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    assert!(
        !on_battlefield(&state, "DP4 Echo Owed By P1"),
        "p1's unanswered echo must be sacrificed"
    );
    assert!(
        in_exile(&state, "DP4 Recover Owed By P3"),
        "p3's unanswered recover must be exiled"
    );

    let creature_died_idx = events
        .iter()
        .position(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .expect("CreatureDied for p1's echo");
    let recover_declined_idx = events
        .iter()
        .position(|e| matches!(e, GameEvent::RecoverDeclined { .. }))
        .expect("RecoverDeclined for p3's recover");
    assert!(
        creature_died_idx < recover_declined_idx,
        "CR 101.4: APNAP order visits p1 before p3, so p1's decline event must precede \
         p3's in the returned event vector"
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

    // p2 (non-active, not the owing player) currently holds priority, mid-round.
    state.turn_mut().priority_holder = Some(p2);
    state.turn_mut().players_passed = imbl::OrdSet::new();

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
    assert!(
        state.turn().players_passed.is_empty(),
        "the pass set must be left exactly as it was"
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
/// CR 702.24b — when a permanent has two outstanding cumulative-upkeep entries, the
/// sweep's snapshot-then-mutate discipline drains BOTH: the first forced decline
/// sacrifices the permanent, and the second entry is consumed silently (CR 400.7 --
/// the permanent has already left).
fn test_702_24b_two_cumulative_upkeep_instances_both_reach_the_boundary() {
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
