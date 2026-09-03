//! PB-DX50 half 1 (`OOS-DX25-1`) — CR 702.140a mutate target legality through the REAL
//! channels.
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx50_mutate_target_legality.rs` and prove that all
//! three behavioural sites consult `casting::mutate_target_requirement()`. This file
//! exists because **existence is never sufficiency** (the `kaito_shizuki` lesson,
//! PB-DX43): a rule the engine applies but no client can reach is not a repaired
//! behaviour, and — more sharply here — a rule the engine now REFUSES that the offer layer
//! still OFFERS is a *new* defect, not a fixed one.
//!
//! **The SR-38 half is the reason this file exists.** Half 1 tightens the cast path with
//! the full CR 115 machinery (hexproof CR 702.11b, shroud CR 702.18a, protection
//! CR 702.16b, layer-resolved types). `StubProvider` kept a FOURTH hand-rolled copy of the
//! CR 702.140a predicate that read `o.characteristics` RAW. Leaving it would have shipped a
//! clean offer followed by a guaranteed refusal — the shape PB-DX29 gated Fuse to avoid,
//! PB-DX44 re-created while fixing it, and PB-DX45 shipped and had to fix. This batch would
//! have been the fourth. `c1`/`c2` are the pair that pins it.
//!
//! | probe | channel | what it discriminates |
//! |---|---|---|
//! | `c1` | `StubProvider::legal_actions` | a shroud host is NOT offered as a mutate host |
//! | `c2` | `StubProvider::legal_actions`, identical fixture minus the shroud | the SAME host IS offered — without this, `c1` is satisfied by an offer layer that offers nothing at all |
//! | `c3` | offer → `action_to_command_with_params` → `process_command` | every offered mutate is ACCEPTED (SR-38's own property), and the accepted cast announces the host: exactly one `PermanentTargeted` |
//! | `c4` | same, with a hexproof host under an opponent's control | the offer is withheld, so no client can reach the refusal at all |
//!
//! # Assert by COUNTS and by ACCEPTANCE, never by ">= 1"
//!
//! Following `pb_dx48_ward_channel.rs`' standard: `c3` asserts `PermanentTargeted` count
//! `== 1`, because a double-dispatch design satisfies every `>= 1` assertion in the tree.
//! And `c3`'s real verdict is that `process_command` **accepts** a command the offer layer
//! produced — the literal statement of SR-38 — not that a picker rendered.

use mtg_engine::{
    process_command, CardDefinition, CardId, CardRegistry, CardType, Command, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    SubType, TypeLine, ZoneId,
};
use mtg_simulator::{
    action_to_command_with_params, ActionParams, LegalAction, LegalActionProvider, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const BEAST: &str = "DX50C Mutating Beast";
const HOST: &str = "DX50C Wolf Host";

fn beast_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx50c-mutating-beast".to_string()),
        name: BEAST.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Beast".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Mutate {1}{G}{G}".to_string(),
        abilities: vec![
            mtg_engine::AbilityDefinition::Keyword(KeywordAbility::Mutate),
            mtg_engine::AbilityDefinition::MutateCost {
                cost: ManaCost {
                    generic: 1,
                    green: 2,
                    ..Default::default()
                },
            },
        ],
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    }
}

fn host_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx50c-wolf-host".to_string()),
        name: HOST.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(3),
        ..Default::default()
    }
}

/// A board with the beast in p1's hand, one host owned by p1, and enough mana for the
/// `{1}{G}{G}` mutate cost. `host_keyword` and `host_controller` are the only inputs that
/// vary between the paired probes below.
fn board(host_keyword: Option<KeywordAbility>, host_controller: PlayerId) -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let mut beast = ObjectSpec::card(p1, BEAST)
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dx50c-mutating-beast".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Beast".to_string())])
        .with_keyword(KeywordAbility::Mutate)
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });
    beast.power = Some(4);
    beast.toughness = Some(4);

    let mut host = ObjectSpec::card(p1, HOST)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dx50c-wolf-host".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Wolf".to_string())])
        .controlled_by(host_controller);
    host.power = Some(2);
    host.toughness = Some(3);
    if let Some(kw) = host_keyword {
        host = host.with_keyword(kw);
    }

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![beast_def(), host_def()]))
        .object(beast)
        .object(host)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Green, 4);
    pool.add(ManaColor::Colorless, 4);
    state.turn_mut().priority_holder = Some(p1);
    state
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

/// Every `CastWithMutate` the REAL offer layer emits for p1 on this board.
fn mutate_offers(state: &GameState) -> Vec<LegalAction> {
    StubProvider
        .legal_actions(state, p(1))
        .into_iter()
        .filter(|a| matches!(a, LegalAction::CastWithMutate { .. }))
        .collect()
}

// ── c1 / c2 — the SR-38 pair ────────────────────────────────────────────────────

/// CR 702.18a — the offer layer does not offer a shroud host.
///
/// This is the SR-38 half. Under the pre-PB-DX50 offer layer (a raw
/// `owner == player && Creature && !Human` filter) this board produced two
/// `CastWithMutate` actions — one per `on_top` value — and every one of them is refused by
/// the post-PB-DX50 cast path. `c2` is the mandatory partner: on its own this probe is
/// also satisfied by an offer layer that has stopped offering mutate entirely.
#[test]
fn c1_a_shroud_host_is_not_offered_as_a_mutate_host() {
    let state = board(Some(KeywordAbility::Shroud), p(1));
    let offers = mutate_offers(&state);
    assert!(
        offers.is_empty(),
        "SR-38 / CR 702.18a: a shroud host is refused by the cast path, so it must never \
         be offered. Offering it is a clean offer followed by a guaranteed refusal. \
         Offers: {offers:#?}"
    );
}

/// The non-vacuity partner for `c1`: the identical board minus the shroud DOES offer the
/// same host. Differs from `c1` in exactly one input.
#[test]
fn c2_the_same_host_without_shroud_is_offered() {
    let state = board(None, p(1));
    let host_id = find_object(&state, HOST);
    let offers = mutate_offers(&state);
    assert_eq!(
        offers.len(),
        2,
        "PB-DX29: the offer layer emits one action per (host, on_top) pair, so one legal \
         host yields exactly two actions. Offers: {offers:#?}"
    );
    for o in &offers {
        match o {
            LegalAction::CastWithMutate { mutate_target, .. } => assert_eq!(
                *mutate_target, host_id,
                "the only legal host on this board is the Wolf"
            ),
            other => panic!("filtered to CastWithMutate, got {other:?}"),
        }
    }
}

// ── c3 — offer -> command -> engine, end to end ─────────────────────────────────

/// SR-38 stated literally: every mutate the offer layer emits is ACCEPTED by
/// `process_command`, and the accepted cast announces the host exactly once.
///
/// The command is built by `action_to_command_with_params` — the same production mapping
/// `LocalGame::advance()` and `tools/play-server` route through — never hand-assembled, so
/// this probe measures the channel rather than a fixture's idea of it.
#[test]
fn c3_every_offered_mutate_is_accepted_and_announces_its_host() {
    let state = board(None, p(1));
    let host_id = find_object(&state, HOST);
    let offers = mutate_offers(&state);
    assert_eq!(offers.len(), 2, "precondition: two offers to drive");

    for offer in &offers {
        let command = action_to_command_with_params(&state, p(1), offer, &ActionParams::default())
            .expect("the production mapping must build a Command for its own offer");
        assert!(
            matches!(command, Command::CastSpell(_)),
            "a CastWithMutate maps to a CastSpell command"
        );
        let (after, events) = process_command(state.clone(), command).unwrap_or_else(|e| {
            panic!(
                "SR-38: an action the offer layer emitted must be accepted by the engine, \
                 got {e:?} for {offer:?}"
            )
        });
        let targeted = events
            .iter()
            .filter(|e| {
                matches!(e, GameEvent::PermanentTargeted { target_id, .. } if *target_id == host_id)
            })
            .count();
        assert_eq!(
            targeted, 1,
            "CR 702.21a: exactly one PermanentTargeted for the host, from a cast the offer \
             layer itself produced. A `>= 1` assertion here would pass on a \
             double-dispatch design (PB-DX48)."
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::TargetsAnnounced { .. })),
            "CR 601.2c: the host reaches the event log"
        );
        assert_eq!(
            after.stack_objects().len(),
            1,
            "the mutating creature spell is on the stack"
        );
        assert_eq!(
            after.stack_objects()[0].targets.len(),
            1,
            "the StackObject records the host as a real target"
        );
    }
}

// ── c4 — hexproof, the control-axis case ────────────────────────────────────────

/// CR 702.11b — a hexproof host an OPPONENT controls (and the caster owns, per
/// CR 702.140a) is withheld from the offer layer too.
///
/// Kept distinct from `c1` because hexproof and shroud take different branches of
/// `validate_target_protection` (hexproof is opponent-only, shroud is universal), and
/// because this is the board on which CR 702.140a's ownership axis and CR 702.11b's
/// control axis genuinely diverge. `c4b` is its non-vacuity partner.
#[test]
fn c4_a_hexproof_host_an_opponent_controls_is_not_offered() {
    let state = board(Some(KeywordAbility::Hexproof), p(2));
    let offers = mutate_offers(&state);
    assert!(
        offers.is_empty(),
        "SR-38 / CR 702.11b: a hexproof host controlled by an opponent is refused by the \
         cast path, so it must not be offered. Offers: {offers:#?}"
    );
}

/// The non-vacuity partner for `c4`: the same opponent-controlled, caster-owned host
/// WITHOUT hexproof is a legal mutate host (CR 702.140a is keyed on ownership, CR 108.3)
/// and is offered. Without this, `c4` would also pass against an offer layer that had
/// wrongly started keying on CONTROL.
#[test]
fn c4b_an_opponent_controlled_but_caster_owned_host_is_still_offered() {
    let state = board(None, p(2));
    let host_id = find_object(&state, HOST);
    let offers = mutate_offers(&state);
    assert_eq!(
        offers.len(),
        2,
        "CR 702.140a / CR 108.3: the host axis is OWNERSHIP, not control -- a host the \
         caster owns but an opponent controls is legal. Offers: {offers:#?}"
    );
    // And the engine agrees: the offer is accepted.
    let command = action_to_command_with_params(&state, p(1), &offers[0], &ActionParams::default())
        .expect("the production mapping must build a Command for its own offer");
    let (_after, events) = process_command(state.clone(), command)
        .expect("SR-38: the offer layer's own action must be accepted");
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::PermanentTargeted { target_id, .. } if *target_id == host_id)
        ),
        "CR 702.21a: the announcement reaches the event log on the ownership-axis board too"
    );
}
