//! PB-DX29 (`OOS-UI2-4`) — the seven remaining cast-side additional-cost kinds, from the
//! provider's offer through the cost arithmetic to a resolved game state.
//!
//! # What this file is for
//!
//! `AdditionalCost` has 15 variants and, before PB-DX29, exactly **two** of them
//! (`Sacrifice`, `Squad`) were ever put in front of a client. Part B1 added seven more
//! to `legal_actions::build_additional_cost_plan` and — the load-bearing half — extended
//! `effective_cast_cost_with_additional` from Squad-only to every mana-bearing rider,
//! because `LocalGame::auto_tap_commands_for` asks *that* function how much mana to tap.
//! Without the extension the client would have offered a rider, accepted it, tapped for
//! the base cost, and watched the engine refuse the cast with `InsufficientMana` — the
//! SR-38 "clean offer, server rejection" shape UI-2 and SIM-6 exist to delete.
//!
//! The three groups below are named for the three claims:
//!
//! * **P** — the OFFER (`StubProvider::legal_actions` → `AdditionalCostPlan`). Includes
//!   the marker/cost pairing in both directions (P2), SR-38 suppression with its
//!   before/after pair (P3), CR 702.102a's zone clause (P4), and the affordability bound
//!   (P5).
//! * **C** — the COST arithmetic (`effective_cast_cost_with_additional`). C2 is the
//!   important one: every prediction is checked **against the engine by execution** —
//!   the spell is really cast with exactly the predicted mana and really refused with one
//!   mana less — rather than by reading `casting.rs` and agreeing with it.
//! * **E** — END TO END: the announced rider really changes the resolved game state.
//!
//! # Rosters are enumerated, never grepped (SR-36)
//!
//! Every corpus member used as a fixture below is one `crates/engine/tests/core/
//! pb_dx29_additional_cost_roster.rs` R1 pins by walking `all_cards()`. C2g adds one more
//! `all_cards()` walk of its own, for a divergence this file found (see its doc).
//!
//! # CR index
//!
//! CR 601.2b/f-h (announcing and paying the total cost), CR 118.8/118.8d (additional
//! costs), CR 702.42a (entwine), CR 702.47a/b (splice), CR 702.56a (replicate),
//! CR 702.102a/b (fuse), CR 702.120a (escalate), CR 702.157a (squad), CR 702.174a/d
//! (gift), CR 702.175a (offspring), CR 400.7 (a moved object is a new object).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, AdditionalCost,
    CardDefinition, CardId, CardRegistry, CardType, Command, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, Target,
    ZoneId,
};
use mtg_simulator::legal_actions::{CountCostKind, MarkerCostKind};
use mtg_simulator::{
    effective_cast_cost, effective_cast_cost_with_additional, ActionParams, AdditionalCostPlan,
    AdvanceOutcome, Bot, HumanChoice, LegalAction, LegalActionProvider, LocalGame, LocalGameLimits,
    StubProvider,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// ── Fixture plumbing ─────────────────────────────────────────────────────────────

/// Every corpus definition keyed by printed name — the shape `enrich_spec_from_def`
/// wants.
fn defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// The whole corpus as a registry. `CardRegistry::new` already returns an `Arc`; do not
/// wrap it again.
fn corpus_registry() -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

/// The `CardId` a corpus def actually declares, which is **not** always
/// `card_name_to_id(name)`.
///
/// `turn.rs` is `cid("turn")` while its printed name is `Turn // Burn`, so the harness
/// helper would mint `CardId("turn-burn")` — an id no registry entry answers to. The
/// provider's `state.card_registry().get(cid)` then returns `None` and
/// `build_additional_cost_plan` silently returns `AdditionalCostPlan::default()`: the
/// cast is still offered (the offer gate reads `characteristics.mana_cost`, which
/// enrichment did fill in) and every rider quietly disappears. Three tests in this file
/// failed exactly that way before this helper existed, which is the honest reason it is
/// here rather than a preference for tidiness.
fn corpus_card_id(defs: &HashMap<String, CardDefinition>, name: &str) -> CardId {
    defs.get(name)
        .unwrap_or_else(|| {
            panic!("{name:?} is not in `all_cards()` — this fixture's premise is gone")
        })
        .card_id
        .clone()
}

/// A real corpus card, fully enriched, in `zone`.
///
/// `ObjectSpec::card()` creates a NAKED object — no card types, no mana cost, no keyword
/// markers — and `StubProvider::legal_actions` reads `obj.characteristics.mana_cost`
/// directly, so an un-enriched fixture is silently offered nothing at all. Every fixture
/// in this file goes through here.
fn corpus_object(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .with_card_id(corpus_card_id(defs, name))
            .in_zone(zone),
        defs,
    )
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

/// The `AdditionalCostPlan` the provider attaches to the `CastSpell` offer for `card`.
///
/// Panics with the whole action list when no such offer exists, because "the plan is
/// empty" and "the cast was never offered" are different failures and a test that
/// conflated them would be lying about which one it caught.
fn cast_plan(state: &GameState, player: PlayerId, card: ObjectId) -> AdditionalCostPlan {
    let actions = StubProvider.legal_actions(state, player);
    let found = actions.iter().find_map(|a| match a {
        LegalAction::CastSpell {
            card: c,
            additional_costs,
            ..
        } if *c == card => Some(additional_costs.clone()),
        _ => None,
    });
    found.unwrap_or_else(|| {
        panic!("no `CastSpell` offer for {card:?}; the whole action list was {actions:?}")
    })
}

/// Whether the provider offered a `CastSpell` for `card` at all.
fn cast_is_offered(state: &GameState, player: PlayerId, card: ObjectId) -> bool {
    StubProvider
        .legal_actions(state, player)
        .iter()
        .any(|a| matches!(a, LegalAction::CastSpell { card: c, .. } if *c == card))
}

/// A pool that pays `cost` EXACTLY: each coloured pip from its own colour, every generic
/// pip from colourless (which is the head of `ManaPool::spend`'s generic order).
fn exact_pool(cost: &ManaCost) -> ManaPool {
    ManaPool {
        white: cost.white,
        blue: cost.blue,
        black: cost.black,
        red: cost.red,
        green: cost.green,
        colorless: cost.colorless + cost.generic,
        ..Default::default()
    }
}

/// `exact_pool(cost)` minus exactly one mana — colourless first, then the first colour
/// with anything in it. Used to prove the prediction is not merely an over-estimate.
fn one_mana_short(cost: &ManaCost) -> ManaPool {
    let mut pool = exact_pool(cost);
    for slot in 0..6 {
        let field = match slot {
            0 => &mut pool.colorless,
            1 => &mut pool.white,
            2 => &mut pool.blue,
            3 => &mut pool.black,
            4 => &mut pool.red,
            _ => &mut pool.green,
        };
        if *field > 0 {
            *field -= 1;
            return pool;
        }
    }
    panic!("cannot take a mana away from a free cost — the fixture is degenerate");
}

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
    additional_costs: Vec<AdditionalCost>,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen,
            x_value: 0,
            face_down_kind: None,
            additional_costs,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
}

/// Round-robin priority passes until the stack is empty (or a step boundary is reached
/// enough times to say the fixture is wrong).
fn resolve_stack(mut state: GameState) -> GameState {
    for _ in 0..40 {
        if state.stack_objects().is_empty() {
            return state;
        }
        for pl in [P1, P2] {
            let (next, _) = process_command(state.clone(), Command::PassPriority { player: pl })
                .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
            state = next;
        }
    }
    panic!("stack did not empty in 40 priority rounds");
}

fn tokens_named(state: &GameState, name: &str, controller: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| {
            o.characteristics.name == name
                && o.controller == controller
                && o.zone == ZoneId::Battlefield
        })
        .count()
}

// ═══════════════════════════════════════════════════════════════════════════════
// P — the OFFER
// ═══════════════════════════════════════════════════════════════════════════════

/// **P1a** — CR 702.56a. Train of Thought (`{1}{U}`, Replicate `{1}{U}`) is offered with
/// a `CountCostOption` carrying the printed replicate cost and a `max_count` derived
/// from real mana rather than a constant.
#[test]
fn p1a_replicate_is_offered_with_its_printed_cost_and_a_real_max_count() {
    let defs = defs_by_name();
    // Base {1}{U} = 2, replicate {1}{U} = 2 per payment. Six mana ({U}{U}{U} + 3
    // colourless) pays the base plus exactly two replications, never three.
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 3,
                colorless: 3,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Train of Thought",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Train of Thought");

    let plan = cast_plan(&state, P1, card);
    assert_eq!(
        plan.counts.len(),
        1,
        "exactly one pay-N-times rider is printed on this card; got {:?}",
        plan.counts
    );
    let replicate = &plan.counts[0];
    assert_eq!(replicate.kind, CountCostKind::Replicate);
    assert_eq!(
        replicate.cost,
        ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        },
        "CR 702.56a: the descriptor carries the def's own printed replicate cost"
    );
    assert_eq!(
        replicate.max_count, 2,
        "six mana covers {{1}}{{U}} plus two {{1}}{{U}} replications and no more"
    );
    // The other five kinds must stay absent — a plan that answered "yes" to everything
    // would pass every positive assertion in this file.
    assert!(plan.markers.is_empty(), "{:?}", plan.markers);
    assert!(plan.gift.is_none());
    assert!(plan.splice.is_none());
    assert!(plan.squad.is_none());
    assert!(plan.sacrifice.is_none());
}

/// **P1b** — CR 702.120a. Escalate's `max_count` is the number of ADDITIONAL modes, not
/// an affordability number alone: `casting.rs` clamps a larger announcement to the mode
/// count, so offering more would be an offer that means nothing.
///
/// Collective Resistance is `{1}{G}` with Escalate `{G}` and **three** modes, so the
/// ceiling is 2. The pool here would pay for five escalations.
#[test]
fn p1b_escalate_is_offered_bounded_by_the_additional_mode_count() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                green: 9,
                colorless: 9,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Collective Resistance",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Collective Resistance");

    let plan = cast_plan(&state, P1, card);
    let escalate = plan
        .counts
        .iter()
        .find(|c| c.kind == CountCostKind::Escalate)
        .expect("Collective Resistance declares Escalate");
    assert_eq!(
        escalate.cost,
        ManaCost {
            green: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        escalate.max_count, 2,
        "CR 702.120a: three printed modes means two ADDITIONAL modes, whatever the pool \
         could afford"
    );
}

/// **P1c** — CR 702.42a. Entwine is a `MarkerCostOption` (paid or not), carrying the
/// printed cost. Goblin War Party is `{3}{R}` with Entwine `{2}{R}`.
#[test]
fn p1c_entwine_is_offered_with_its_printed_cost() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                red: 2,
                colorless: 5,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Goblin War Party",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Goblin War Party");

    let plan = cast_plan(&state, P1, card);
    assert_eq!(plan.markers.len(), 1, "{:?}", plan.markers);
    assert_eq!(plan.markers[0].kind, MarkerCostKind::Entwine);
    assert_eq!(
        plan.markers[0].cost,
        Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        "CR 702.42a: entwine has a printed cost of its own, unlike fuse"
    );
    assert!(plan.counts.is_empty(), "{:?}", plan.counts);
}

/// **P1d** — CR 702.175a. Offspring, same `MarkerCostOption` shape. Flowerfoot
/// Swordmaster is `{W}` with Offspring `{2}`.
///
/// The def is `partial` (its Valiant clause is engine-blocked), which is irrelevant here:
/// completeness gates `validate_deck`, and this fixture never starts a game. Recorded so
/// the choice of card reads as deliberate rather than careless.
#[test]
fn p1d_offspring_is_offered_with_its_printed_cost() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                white: 1,
                colorless: 2,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Flowerfoot Swordmaster",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Flowerfoot Swordmaster");

    let plan = cast_plan(&state, P1, card);
    assert_eq!(plan.markers.len(), 1, "{:?}", plan.markers);
    assert_eq!(plan.markers[0].kind, MarkerCostKind::Offspring);
    assert_eq!(
        plan.markers[0].cost,
        Some(ManaCost {
            generic: 2,
            ..Default::default()
        })
    );
}

/// **P1e** — CR 702.102a/b/d, **re-inverted a second time, by PB-DX44 closing
/// `OOS-DX29-12`.**
///
/// This test originally asserted that Fuse IS offered from hand (before PB-DX29 found
/// the target-announcement gap); PB-DX29 then inverted it to assert the SUPPRESSION,
/// with an explicit instruction in its own doc to re-point this at a real fused cast
/// once `casting.rs` learns CR 702.102d — not to delete it. PB-DX44 is that fix
/// (`card_def_target_requirements`'s new `casting_with_fuse` parameter, PB-DX44's
/// implementation notes), so this asserts the OFFER again, plus the property that makes
/// it safe now: the offer's target-requirement COUNT agrees with what the cast
/// validates against — the differential this whole file's C2 group exists for, applied
/// to targets rather than mana. Full end-to-end resolution (each half hitting its own
/// target) is `crates/engine/tests/rules/pb_dx44_fuse_targets.rs`, not duplicated here.
#[test]
fn p1e_fuse_is_offered_and_its_target_count_matches_what_the_cast_validates() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 1,
                red: 1,
                colorless: 3,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
        .object(ObjectSpec::creature(P2, "Some Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Turn // Burn");

    // (1) The offer: Fuse is a marker with no separate cost (CR 702.102b) and is
    // affordable from the pool above (base {2}{U} + right half {1}{R} = {3}{U}{R},
    // mana value 5; the pool holds exactly that).
    let plan = cast_plan(&state, P1, card);
    assert_eq!(
        plan.markers
            .iter()
            .find(|m| m.kind == MarkerCostKind::Fuse)
            .map(|m| (m.cost.clone(), m.affordable)),
        Some((None, true)),
        "CR 702.102a/b: Fuse must be offered from hand, with no separate cost and \
         affordable from this pool. Offered: {:?}",
        plan.markers
    );

    // (2) The offer's target-requirement count, read exactly the way the browser and
    // the bot both would (`fuse: true`), must equal what a real fused cast validates
    // against — 2, per CR 702.102d (left half + right half). This is the SR-38
    // differential: an offer whose count disagrees with the cast is a clean offer
    // followed by a guaranteed server rejection, which is exactly what PB-DX29 found
    // and PB-DX44 fixes.
    let offered_reqs = mtg_engine::spell_target_requirements(&state, card, &[], None, true);
    assert_eq!(
        offered_reqs.len(),
        2,
        "CR 702.102d: a fused cast announces both halves' targets; got {offered_reqs:?}"
    );

    // (3) The real cast, with exactly the requirement count the offer reported, is
    // accepted -- the offer and the cast are proven to be ONE arithmetic by execution,
    // not merely by reading the same source.
    let bear = id_of(&state, "Some Bear");
    let (after, _events) = cast(
        state,
        P1,
        card,
        vec![Target::Object(bear), Target::Object(bear)],
        vec![],
        vec![AdditionalCost::Fuse],
    )
    .expect("a fused cast with 2 announced targets, funded exactly, must be accepted");
    assert!(
        after
            .stack_objects()
            .iter()
            .any(|so| matches!(&so.kind, mtg_engine::StackObjectKind::Spell { .. })),
        "the fused spell must actually be on the stack after a successful cast"
    );
}

/// **P1f** — CR 702.174a/d. Gift's "cost" is naming an opponent, so its descriptor
/// carries a `GiftType` and a player set and no mana at all. Nocturnal Hunger is
/// `Complete` and deck-legal, and its `KeywordAbility::Gift` marker was **missing** until
/// PB-DX29 repaired the def — which made the printed gift unpayable by any client.
#[test]
fn p1f_gift_is_offered_with_its_type_and_every_other_active_player() {
    let defs = defs_by_name();
    let p3 = PlayerId(3);
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .add_player(p3)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                black: 1,
                colorless: 2,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Nocturnal Hunger",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Nocturnal Hunger");

    let plan = cast_plan(&state, P1, card);
    let gift = plan.gift.as_ref().expect("Nocturnal Hunger gifts a Food");
    assert!(
        matches!(
            gift.gift_type,
            mtg_engine::cards::card_definition::GiftType::Food
        ),
        "printed 'Gift a Food'; got {:?}",
        gift.gift_type
    );
    let eligible: BTreeSet<PlayerId> = gift.eligible.iter().copied().collect();
    assert_eq!(
        eligible,
        [P2, p3].into_iter().collect::<BTreeSet<_>>(),
        "CR 702.174a: every OTHER player in the game, and never the caster — \
         `casting.rs` refuses `opponent == player` outright"
    );
    assert!(plan.counts.is_empty() && plan.markers.is_empty());
}

/// **P1g** — CR 702.47a/b. Splice's descriptor is a **legality** set, not an
/// affordability one (each spliced card costs a different amount, so the monotone `1..N`
/// walk the count riders use does not apply). Reach Through Mists (`{U}`, Instant —
/// Arcane) is the spell; Glacial Ray (Splice onto Arcane `{1}{R}`) is the eligible card.
#[test]
fn p1g_splice_is_offered_when_an_eligible_arcane_splice_card_is_in_hand() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 1,
                red: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Reach Through Mists",
            ZoneId::Hand(P1),
        ))
        .object(corpus_object(&defs, P1, "Glacial Ray", ZoneId::Hand(P1)))
        .build()
        .expect("state builds");
    let arcane = id_of(&state, "Reach Through Mists");
    let ray = id_of(&state, "Glacial Ray");

    let plan = cast_plan(&state, P1, arcane);
    let splice = plan
        .splice
        .as_ref()
        .expect("a Glacial Ray in hand is spliceable onto an Arcane spell");
    assert_eq!(
        splice.eligible,
        vec![ray],
        "CR 702.47a: the eligible set is exactly the hand cards whose `onto_subtype` the \
         spell carries"
    );

    // The asymmetry a reader most often misses, pinned: the SPELL needs no splice
    // keyword of its own. Reach Through Mists carries none.
    let spell_def = defs.get("Reach Through Mists").unwrap();
    assert!(
        !spell_def
            .abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Splice))),
        "the fixture is only meaningful while the spell being spliced onto has no Splice \
         keyword of its own (CR 702.47a is one-sided)"
    );
}

// ── P2: the marker/cost pairing, in BOTH directions ─────────────────────────────

/// A synthetic instant carrying only what the caller asks for. Used for the
/// marker-without-cost direction, which **no corpus def has** — `pb_dx29_additional_cost_
/// roster::r2` is the gate that keeps it that way, so a fixture for it must be built.
fn synthetic_replicate_def(name: &str, marker: bool, cost: bool) -> CardDefinition {
    let mut abilities = Vec::new();
    if marker {
        abilities.push(AbilityDefinition::Keyword(KeywordAbility::Replicate));
    }
    if cost {
        abilities.push(AbilityDefinition::Replicate {
            cost: ManaCost {
                generic: 1,
                ..Default::default()
            },
        });
    }
    abilities.push(AbilityDefinition::Spell {
        effect: mtg_engine::Effect::Nothing,
        targets: vec![],
        modes: None,
        cant_be_countered: false,
    });
    CardDefinition {
        card_id: CardId(format!("pb-dx29-{}", name.to_lowercase().replace(' ', "-"))),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::cards::helpers::types(&[CardType::Instant]),
        oracle_text: "Do nothing.".to_string(),
        abilities,
        ..Default::default()
    }
}

/// Build a one-card state around a synthetic def, registered beside the whole corpus.
fn state_with_synthetic(def: CardDefinition) -> (GameState, ObjectId) {
    let name = def.name.clone();
    let card_id = def.card_id.clone();
    let mut all = all_cards();
    all.push(def.clone());
    let registry = CardRegistry::new(all);
    let mut defs = defs_by_name();
    defs.insert(name.clone(), def);
    let spec = enrich_spec_from_def(
        ObjectSpec::card(P1, &name)
            .with_card_id(card_id)
            .in_zone(ZoneId::Hand(P1)),
        &defs,
    );
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(registry)
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                colorless: 8,
                ..Default::default()
            },
        )
        .object(spec)
        .build()
        .expect("state builds");
    let id = id_of(&state, &name);
    (state, id)
}

/// **P2a** — CR 702.102a. The COST-without-MARKER direction, on a **real corpus def**.
///
/// `connive.rs` (Connive // Concoct) carries `AbilityDefinition::Fuse` as a pure data
/// carrier for the right half's name/cost/types and deliberately does **not** carry
/// `KeywordAbility::Fuse`, because neither printed half has fuse. `casting.rs:1279` gates
/// the fused cast on the marker, so offering Fuse here would be an offer the engine
/// refuses on sight. The provider must therefore read BOTH halves, not just the cost.
///
/// This is the exact shape `nocturnal_hunger` had (cost present, marker absent) before
/// PB-DX29 repaired it, surviving in the corpus on purpose.
#[test]
fn p2a_a_fuse_cost_without_its_marker_is_not_offered() {
    let defs = defs_by_name();
    let def = defs
        .get("Connive // Concoct")
        .expect("the corpus's declared fuse DATA CARRIER");
    // The premise, asserted rather than assumed — if the def ever gains the marker this
    // test stops testing what it says it does.
    assert!(
        def.abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Fuse { .. })),
        "premise: Connive // Concoct carries the Fuse COST variant"
    );
    assert!(
        !def.abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Fuse))),
        "premise: Connive // Concoct carries NO `KeywordAbility::Fuse` marker"
    );

    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 2,
                black: 2,
                colorless: 8,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Connive // Concoct",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Connive // Concoct");

    let plan = cast_plan(&state, P1, card);
    assert!(
        !plan.markers.iter().any(|m| m.kind == MarkerCostKind::Fuse),
        "CR 702.102a: `casting.rs` gates the fused cast on `KeywordAbility::Fuse` BEFORE \
         it reads `AbilityDefinition::Fuse`, so a data-carrier def must not be offered \
         Fuse; got {:?}",
        plan.markers
    );
}

/// **P2b** — CR 702.56a. The MARKER-without-COST direction, which the corpus has none of
/// (the R2 roster gate exists to keep it that way), so the fixture is a synthetic def
/// registered beside the corpus.
///
/// `casting.rs` gates on the marker and then calls `get_replicate_cost`; when that
/// returns `None` the cast is refused with *"spell has replicate keyword but no replicate
/// cost defined"*. Offering the rider anyway would be an offer whose every non-zero
/// answer is a 422.
#[test]
fn p2b_a_replicate_marker_without_its_cost_is_not_offered() {
    let (state, card) =
        state_with_synthetic(synthetic_replicate_def("PB-DX29 Marker Only", true, false));
    let plan = cast_plan(&state, P1, card);
    assert!(
        plan.counts.is_empty(),
        "a marker-only def has no payable replicate cost; got {:?}",
        plan.counts
    );
    // …and the cast itself is still offered. A missing OPTIONAL rider never suppresses
    // the spell (CR 702.56a, "any number of times", including zero).
    assert!(cast_is_offered(&state, P1, card));
}

/// **P2c** — the control for P2b, on the SAME synthetic shape with both halves present.
/// Without it, P2b passes for any reason at all — including a fixture the provider
/// cannot see.
#[test]
fn p2c_both_halves_present_is_offered_on_the_same_synthetic_shape() {
    let (state, card) =
        state_with_synthetic(synthetic_replicate_def("PB-DX29 Both Halves", true, true));
    let plan = cast_plan(&state, P1, card);
    assert_eq!(plan.counts.len(), 1, "{:?}", plan.counts);
    assert_eq!(plan.counts[0].kind, CountCostKind::Replicate);

    // And the third corner: COST without MARKER, same shape, so the pairing is proven
    // in both directions on one fixture rather than across two unrelated ones.
    let (state, card) =
        state_with_synthetic(synthetic_replicate_def("PB-DX29 Cost Only", false, true));
    let plan = cast_plan(&state, P1, card);
    assert!(
        plan.counts.is_empty(),
        "CR 702.56a: `casting.rs` checks `chars.keywords.contains(&Replicate)` first; \
         got {:?}",
        plan.counts
    );
}

// ── P3: SR-38 suppression, as before/after pairs on ONE board ──────────────────

/// **P3** — CR 702.174a / SR-38. Gift is `None` when no other player is still in the
/// game, and `Some` on the SAME board the moment one is. Two unrelated assertions would
/// prove nothing about the suppression itself.
#[test]
fn p3a_gift_is_suppressed_with_no_other_active_player_and_offered_once_one_exists() {
    let defs = defs_by_name();
    let build = |opponent_alive: bool| {
        let mut state = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(
                P1,
                ManaPool {
                    black: 1,
                    colorless: 2,
                    ..Default::default()
                },
            )
            .object(corpus_object(
                &defs,
                P1,
                "Nocturnal Hunger",
                ZoneId::Hand(P1),
            ))
            .build()
            .expect("state builds");
        if !opponent_alive {
            // CR 104.3a — the only way `active_players()` shrinks. `build_additional_
            // cost_plan` reads exactly this list, mirroring `casting.rs`'s own gift gate.
            state.players_mut().get_mut(&P2).unwrap().has_conceded = true;
        }
        state
    };

    let dead = build(false);
    let card = id_of(&dead, "Nocturnal Hunger");
    assert!(
        cast_plan(&dead, P1, card).gift.is_none(),
        "SR-38: with no eligible opponent the gift half of the offer must be absent, not \
         an empty picker"
    );

    let alive = build(true);
    let card = id_of(&alive, "Nocturnal Hunger");
    let gift = cast_plan(&alive, P1, card)
        .gift
        .expect("the same board with P2 still in the game must offer the gift");
    assert_eq!(gift.eligible, vec![P2]);
}

/// **P3b** — CR 702.47a / SR-38. Splice is `None` with an empty hand and `None` with a
/// hand holding an ineligible card (a splice card whose `onto_subtype` the spell does not
/// carry), and `Some` once a genuinely eligible card arrives. All three on one board.
#[test]
fn p3b_splice_is_suppressed_until_an_eligible_card_is_in_hand() {
    let defs = defs_by_name();
    // `extra` is a second card put in P1's hand beside the Arcane spell being cast.
    let build = |extra: Option<&str>| {
        let mut b = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(
                P1,
                ManaPool {
                    blue: 1,
                    red: 1,
                    colorless: 1,
                    ..Default::default()
                },
            )
            .object(corpus_object(
                &defs,
                P1,
                "Reach Through Mists",
                ZoneId::Hand(P1),
            ));
        if let Some(name) = extra {
            b = b.object(corpus_object(&defs, P1, name, ZoneId::Hand(P1)));
        }
        b.build().expect("state builds")
    };

    // (a) nothing else in hand.
    let empty = build(None);
    let card = id_of(&empty, "Reach Through Mists");
    assert!(
        cast_plan(&empty, P1, card).splice.is_none(),
        "SR-38: no eligible card means no splice offer at all"
    );

    // (b) a card in hand that carries no Splice ability whatsoever.
    let irrelevant = build(Some("Lightning Bolt"));
    let card = id_of(&irrelevant, "Reach Through Mists");
    assert!(
        cast_plan(&irrelevant, P1, card).splice.is_none(),
        "a hand card without `AbilityDefinition::Splice` is not eligible"
    );

    // (c) the real thing.
    let eligible = build(Some("Glacial Ray"));
    let card = id_of(&eligible, "Reach Through Mists");
    let ray = id_of(&eligible, "Glacial Ray");
    assert_eq!(
        cast_plan(&eligible, P1, card)
            .splice
            .expect("Glacial Ray splices onto Arcane")
            .eligible,
        vec![ray],
        "the same board, one card different, and the offer appears"
    );

    // (d) the SUBTYPE half of the two-sided gate, which (a)-(c) cannot reach: an
    // eligible-looking Glacial Ray in hand while the spell being cast is NOT Arcane.
    // CR 702.47a keys splice on the spell carrying the splice card's `onto_subtype`, and
    // a fixture whose subject is subtypeless (Lightning Bolt) would be caught by
    // `eligible_splice_cards`'s empty-subtypes early return instead, proving nothing
    // about the comparison itself. Llanowar Elves is Elf Druid — subtypes present, and
    // none of them Arcane.
    let wrong_subtype = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                green: 1,
                red: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Llanowar Elves", ZoneId::Hand(P1)))
        .object(corpus_object(&defs, P1, "Glacial Ray", ZoneId::Hand(P1)))
        .build()
        .expect("state builds");
    let elves = id_of(&wrong_subtype, "Llanowar Elves");
    assert!(
        !mtg_engine::calculate_characteristics(&wrong_subtype, elves)
            .expect("the card exists")
            .subtypes
            .is_empty(),
        "the fixture only tests the comparison while the subject HAS subtypes — a \
         subtypeless one is short-circuited before the comparison is reached"
    );
    assert!(
        cast_plan(&wrong_subtype, P1, elves).splice.is_none(),
        "CR 702.47a: Glacial Ray splices onto Arcane, and an Elf Druid is not Arcane"
    );
}

/// **P4** — CR 702.102a, "from your hand". Fuse is the one rider whose legality depends
/// on the zone the cast is from, and `build_additional_cost_plan` is shared by the hand
/// loop and the command-zone loop, so the clause has to live inside it.
///
/// # Why this uses a SYNTHETIC def, and why that is a finding rather than a convenience
///
/// The first version of this test used `Turn // Burn` from the corpus and asserted the
/// offer is present from hand and absent from the command zone. It stopped working the
/// moment PB-DX29 added its CR 702.102d suppression (see `p1e`): **every corpus fuse def
/// has a targeted right half**, so the target suppression covers all of them and the
/// zone clause is no longer independently observable on real cards.
///
/// Two properties in one predicate, one of which shadows the other, is exactly the shape
/// that leaves a clause untested while its test passes. So the zone clause is exercised
/// on a synthetic split card whose right half declares **no** targets — the target
/// suppression cannot fire, and only the zone clause can decide the outcome. Both halves
/// are built from ONE closure so the only difference between them is the zone.
///
/// A split instant cannot legally be a commander (CR 903.3 wants a legendary creature),
/// and this fixture does not pretend otherwise. It does not need to: the provider's
/// command-zone loop gates on `commander_ids` and `can_cast_at_this_time` alone — never
/// on commander legality, which `validate_deck` owns and which never runs here — so this
/// drives the real code path the zone clause has to survive.
#[test]
fn p4_fuse_is_offered_from_hand_and_never_from_the_command_zone() {
    let def = synthetic_untargeted_fuse_def("PB-DX29 Untargeted Fuse");
    let name = def.name.clone();
    let card_id = def.card_id.clone();

    let build = |zone: ZoneId| {
        let mut all = all_cards();
        all.push(def.clone());
        let registry = CardRegistry::new(all);
        let mut defs = defs_by_name();
        defs.insert(name.clone(), def.clone());
        let spec = enrich_spec_from_def(
            ObjectSpec::card(P1, &name)
                .with_card_id(card_id.clone())
                .in_zone(zone),
            &defs,
        );
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(registry)
            .active_player(P1)
            .player_commander(P1, card_id.clone())
            .player_mana(
                P1,
                ManaPool {
                    colorless: 6,
                    ..Default::default()
                },
            )
            .object(spec)
            .build()
            .expect("state builds")
    };

    let from_hand = build(ZoneId::Hand(P1));
    let card = id_of(&from_hand, &name);
    assert!(
        cast_plan(&from_hand, P1, card)
            .markers
            .iter()
            .any(|m| m.kind == MarkerCostKind::Fuse),
        "CR 702.102a: a cast from HAND may fuse. If this is empty, either the zone clause \
         inverted or the CR 702.102d target suppression is firing on a right half that \
         declares no targets."
    );

    let from_command = build(ZoneId::Command(P1));
    let card = id_of(&from_command, &name);
    let plan = cast_plan(&from_command, P1, card);
    assert!(
        !plan.markers.iter().any(|m| m.kind == MarkerCostKind::Fuse),
        "CR 702.102a: `casting.rs` refuses a fused cast from anywhere but hand, so the \
         command-zone offer must not carry it; got {:?}",
        plan.markers
    );
}

/// A split card whose fused right half declares **no** targets — the one shape that
/// isolates CR 702.102a's zone clause from CR 702.102d's target suppression.
///
/// No corpus def has this shape (both deck-legal fuse defs target on the right), which is
/// why it is synthesised rather than found. `p1e`'s non-vacuity check is what pins that
/// corpus fact from the other side.
fn synthetic_untargeted_fuse_def(name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(format!("pb-dx29-{}", name.to_lowercase().replace(' ', "-"))),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::cards::helpers::types(&[CardType::Instant]),
        oracle_text: "Do nothing. // Do nothing.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fuse),
            AbilityDefinition::Fuse {
                name: "Right Half".to_string(),
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                card_type: CardType::Instant,
                effect: mtg_engine::Effect::Nothing,
                // The whole point of this fixture.
                targets: vec![],
            },
            AbilityDefinition::Spell {
                effect: mtg_engine::Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// **P5** — CR 702.56a. `repeated_cost_max_count` is a genuine affordability bound for
/// Replicate, not a cap, mirroring `squad_max_count_is_a_real_bound_not_an_arbitrary_cap`
/// one function over: three pools on one card, each producing the arithmetically exact N.
#[test]
fn p5_replicate_max_count_is_a_real_bound_not_an_arbitrary_cap() {
    let defs = defs_by_name();
    let make_state = |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Train of Thought",
                ZoneId::Hand(P1),
            ))
            .build()
            .expect("state builds")
    };
    let max_for = |pool: ManaPool| {
        let state = make_state(pool);
        let card = id_of(&state, "Train of Thought");
        let plan = cast_plan(&state, P1, card);
        let n = plan
            .counts
            .iter()
            .find(|c| c.kind == CountCostKind::Replicate)
            .expect("Replicate must be detected")
            .max_count;
        // Every pool below must still be able to cast the spell — a suppressed offer
        // would make `cast_plan` panic, so reaching here is itself the assertion.
        (n, cast_is_offered(&state, P1, card))
    };

    // Base {1}{U} = 2. Replicate {1}{U} = 2 per payment, and it needs a BLUE pip each
    // time, which is what makes this a colour-aware bound rather than a total.
    assert_eq!(
        max_for(ManaPool {
            blue: 1,
            colorless: 1,
            ..Default::default()
        }),
        (0, true),
        "exactly the base cost: zero replications, and the cast is still offered"
    );
    assert_eq!(
        max_for(ManaPool {
            blue: 2,
            colorless: 2,
            ..Default::default()
        }),
        (1, true),
        "one extra {{1}}{{U}}"
    );
    assert_eq!(
        max_for(ManaPool {
            blue: 3,
            colorless: 3,
            ..Default::default()
        }),
        (2, true),
        "two extra {{1}}{{U}}"
    );
    // Colour-blindness would report 2 here (six mana total) instead of 1.
    assert_eq!(
        max_for(ManaPool {
            blue: 2,
            colorless: 4,
            ..Default::default()
        }),
        (1, true),
        "six mana but only two blue pips: the second replication has no {{U}} to pay with"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// C — the COST arithmetic
// ═══════════════════════════════════════════════════════════════════════════════

/// **C1** — CR 118.8d. With no rider announced, `effective_cast_cost_with_additional` is
/// byte-identical to `effective_cast_cost`, on every card this file uses. This is the
/// property every existing caller (every bot cast, every auto-tap) depends on, and the
/// extension in part B1 had to preserve it.
#[test]
fn c1_no_announced_rider_is_byte_identical_to_the_plain_effective_cost() {
    let defs = defs_by_name();
    for name in [
        "Train of Thought",
        "Goblin War Party",
        "Collective Resistance",
        "Flowerfoot Swordmaster",
        "Turn // Burn",
        "Nocturnal Hunger",
        "Reach Through Mists",
        "Lightning Bolt",
    ] {
        let state = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .object(corpus_object(&defs, P1, name, ZoneId::Hand(P1)))
            .build()
            .expect("state builds");
        let card = id_of(&state, name);
        let plain = effective_cast_cost(&state, P1, card).expect("every fixture has a mana cost");
        let with_none = effective_cast_cost_with_additional(&state, P1, card, &[], &[], None)
            .expect("identity must not lose the cost");
        assert_eq!(
            plain, with_none,
            "{name}: identity broken for an empty rider list"
        );
        // And an announced rider that carries no mana must be identity too (CR 702.174a
        // — naming an opponent IS the gift's cost). Checked here rather than only in C4
        // so the identity claim covers the non-empty-list case as well.
        let with_gift = effective_cast_cost_with_additional(
            &state,
            P1,
            card,
            &[AdditionalCost::Gift { opponent: P2 }],
            &[],
            None,
        )
        .expect("gift must not make the cost unknowable");
        assert_eq!(plain, with_gift, "{name}: Gift must add nothing");
    }
}

/// The C2 engine cross-check, run once per rider.
///
/// The prediction is read from `effective_cast_cost_with_additional`; the SAME cast is
/// then driven through the real engine twice — once from a pool holding exactly the
/// predicted mana (must be ACCEPTED) and once from a pool one mana short (must be
/// REFUSED). Two-sided by construction, so an over-estimating prediction fails the first
/// leg and an under-estimating one fails the second. No `casting.rs` arithmetic is
/// re-derived here; the engine is the oracle.
fn assert_prediction_is_exactly_what_the_engine_charges(
    label: &str,
    build: &dyn Fn(ManaPool) -> GameState,
    card_name: &str,
    riders: &[AdditionalCost],
    targets: &dyn Fn(&GameState) -> Vec<Target>,
    modes: Vec<usize>,
) {
    // The prediction is pool-independent, so any pool will do to read it.
    let probe = build(ManaPool::default());
    let card = id_of(&probe, card_name);
    let base = effective_cast_cost(&probe, P1, card).expect("fixture has a mana cost");
    let predicted = effective_cast_cost_with_additional(&probe, P1, card, riders, &modes, None)
        .unwrap_or_else(|| panic!("{label}: no prediction at all"));
    assert!(
        predicted.mana_value() > base.mana_value(),
        "{label}: the rider must actually cost mana, else this test is vacuous \
         (base {}, predicted {})",
        base.mana_value(),
        predicted.mana_value()
    );

    let exact = build(exact_pool(&predicted));
    let card = id_of(&exact, card_name);
    let t = targets(&exact);
    let accepted = cast(exact, P1, card, t, modes.clone(), riders.to_vec());
    assert!(
        accepted.is_ok(),
        "{label}: the engine refused a cast funded with exactly the predicted \
         {predicted:?} — the prediction is an UNDER-estimate: {:?}",
        accepted.err()
    );

    let short = build(one_mana_short(&predicted));
    let card = id_of(&short, card_name);
    let t = targets(&short);
    let refused = cast(short, P1, card, t, modes, riders.to_vec());
    // `is_err()` alone would let this leg pass for the wrong reason (a bad target, a
    // timing refusal). The variant is pinned so the failure really is about the mana.
    assert!(
        matches!(refused, Err(mtg_engine::GameStateError::InsufficientMana)),
        "{label}: one mana less than the predicted {predicted:?} must be refused for \
         INSUFFICIENT MANA — an over-estimating prediction makes the auto-tap reach for \
         mana the engine never charges. Got: {:?}",
        refused.map(|_| "ACCEPTED")
    );
}

fn no_targets(_: &GameState) -> Vec<Target> {
    vec![]
}

/// **C2a** — CR 702.56a. Replicate ×2 on Train of Thought: `{1}{U}` + 2×`{1}{U}`.
#[test]
fn c2a_replicate_is_charged_the_way_the_engine_charges_it() {
    let defs = defs_by_name();
    let build = move |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Train of Thought",
                ZoneId::Hand(P1),
            ))
            .object(ObjectSpec::card(P1, "Filler").in_zone(ZoneId::Library(P1)))
            .build()
            .expect("state builds")
    };
    assert_prediction_is_exactly_what_the_engine_charges(
        "replicate x2",
        &build,
        "Train of Thought",
        &[AdditionalCost::Replicate { count: 2 }],
        &no_targets,
        vec![],
    );
}

/// **C2b** — CR 702.120a. Escalate ×1 on Collective Resistance: `{1}{G}` + 1×`{G}`.
/// `casting.rs` derives the chosen modes from the escalate count (`0..=escalate_modes`),
/// so `modes_chosen` is deliberately left empty.
#[test]
fn c2b_escalate_is_charged_the_way_the_engine_charges_it() {
    let defs = defs_by_name();
    let build = move |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Collective Resistance",
                ZoneId::Hand(P1),
            ))
            .object(ObjectSpec::artifact(P2, "Some Artifact").in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::enchantment(P2, "Some Enchantment").in_zone(ZoneId::Battlefield))
            .object(ObjectSpec::creature(P2, "Some Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds")
    };
    let targets = |state: &GameState| {
        vec![
            Target::Object(id_of(state, "Some Artifact")),
            Target::Object(id_of(state, "Some Enchantment")),
            Target::Object(id_of(state, "Some Bear")),
        ]
    };
    assert_prediction_is_exactly_what_the_engine_charges(
        "escalate x1",
        &build,
        "Collective Resistance",
        &[AdditionalCost::EscalateModes { count: 1 }],
        &targets,
        vec![],
    );
}

/// **C2c** — CR 702.42a. Entwine on Goblin War Party: `{3}{R}` + `{2}{R}`.
#[test]
fn c2c_entwine_is_charged_the_way_the_engine_charges_it() {
    let defs = defs_by_name();
    let build = move |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Goblin War Party",
                ZoneId::Hand(P1),
            ))
            .build()
            .expect("state builds")
    };
    assert_prediction_is_exactly_what_the_engine_charges(
        "entwine",
        &build,
        "Goblin War Party",
        &[AdditionalCost::Entwine],
        &no_targets,
        vec![],
    );
}

/// **C2d** — CR 702.175a. Offspring on Flowerfoot Swordmaster: `{W}` + `{2}`.
#[test]
fn c2d_offspring_is_charged_the_way_the_engine_charges_it() {
    let defs = defs_by_name();
    let build = move |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Flowerfoot Swordmaster",
                ZoneId::Hand(P1),
            ))
            .build()
            .expect("state builds")
    };
    assert_prediction_is_exactly_what_the_engine_charges(
        "offspring",
        &build,
        "Flowerfoot Swordmaster",
        &[AdditionalCost::Offspring],
        &no_targets,
        vec![],
    );
}

/// **C2e** — CR 702.102b. Fuse on Turn // Burn: `{2}{U}` + the right half's `{1}{R}`.
///
/// The two implementations add the right half at DIFFERENT points — `casting.rs` before
/// commander tax, the provider after — and the totals agree only because CR 903.8's tax
/// is an additive generic term rather than a multiplier. There is no tax in this fixture,
/// so this test does not prove that; it proves the summation itself.
#[test]
fn c2e_fuse_is_charged_the_way_the_engine_charges_it() {
    let defs = defs_by_name();
    let build = move |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
            .object(ObjectSpec::creature(P2, "Some Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds")
    };
    // **TWO targets, not one (PB-DX44, `OOS-DX29-12` CLOSED).** CR 702.102d gives a
    // fused spell both halves' targets — Turn // Burn declares `TargetCreature` on its
    // `Spell` ability (index 0, left/Turn half) and `TargetAny` on its `Fuse` ability
    // (index 1, right/Burn half). Both slots point at the same object here because this
    // test is about the COST arithmetic, not the index contract — see
    // `crates/engine/tests/rules/pb_dx44_fuse_targets.rs` for the test that targets two
    // DIFFERENT objects and proves each half resolves against its own target.
    let targets = |state: &GameState| {
        let bear = Target::Object(id_of(state, "Some Bear"));
        vec![bear.clone(), bear]
    };
    assert_prediction_is_exactly_what_the_engine_charges(
        "fuse",
        &build,
        "Turn // Burn",
        &[AdditionalCost::Fuse],
        &targets,
        vec![],
    );
}

/// **C2f** — CR 702.47b. Splice really is a SUM rather than a last-wins: each spliced
/// card's own cost is added. Reach Through Mists `{U}` + Glacial Ray `{1}{R}`.
#[test]
fn c2f_splice_is_charged_the_way_the_engine_charges_it() {
    let defs = defs_by_name();
    let build = move |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Reach Through Mists",
                ZoneId::Hand(P1),
            ))
            .object(corpus_object(&defs, P1, "Glacial Ray", ZoneId::Hand(P1)))
            .object(ObjectSpec::card(P1, "Filler").in_zone(ZoneId::Library(P1)))
            .build()
            .expect("state builds")
    };
    // The splice card's `ObjectId` is minted by the builder, so the rider list has to be
    // built per state. `assert_prediction_...` takes a fixed rider list, so this one is
    // spelled out rather than routed through it — and it keeps both legs.
    let probe = build(ManaPool::default());
    let card = id_of(&probe, "Reach Through Mists");
    let ray = id_of(&probe, "Glacial Ray");
    let riders = vec![AdditionalCost::Splice { cards: vec![ray] }];
    let base = effective_cast_cost(&probe, P1, card).expect("has a cost");
    let predicted = effective_cast_cost_with_additional(&probe, P1, card, &riders, &[], None)
        .expect("splice prediction");
    assert_eq!(
        predicted.mana_value(),
        base.mana_value() + 2,
        "CR 702.47b: Glacial Ray's splice cost {{1}}{{R}} is added in full"
    );

    let exact = build(exact_pool(&predicted));
    let card = id_of(&exact, "Reach Through Mists");
    let ray = id_of(&exact, "Glacial Ray");
    let accepted = cast(
        exact,
        P1,
        card,
        vec![],
        vec![],
        vec![AdditionalCost::Splice { cards: vec![ray] }],
    );
    assert!(
        accepted.is_ok(),
        "the engine refused a spliced cast funded with exactly {predicted:?}: {:?}",
        accepted.err()
    );

    let short = build(one_mana_short(&predicted));
    let card = id_of(&short, "Reach Through Mists");
    let ray = id_of(&short, "Glacial Ray");
    let refused = cast(
        short,
        P1,
        card,
        vec![],
        vec![],
        vec![AdditionalCost::Splice { cards: vec![ray] }],
    );
    assert!(
        matches!(refused, Err(mtg_engine::GameStateError::InsufficientMana)),
        "one mana short of {predicted:?} must be refused for INSUFFICIENT MANA; got {:?}",
        refused.map(|_| "ACCEPTED")
    );
}

/// **C2g** — the one place the provider does NOT mirror `casting.rs`, pinned as
/// unreachable rather than glossed.
///
/// `effective_cast_cost_with_additional`'s `add` closure copies the **seven numeric**
/// components of a rider cost and nothing else. That mirrors `casting.rs` exactly for
/// Squad / Replicate / Escalate / Entwine / Offspring / Splice, whose own loops also add
/// only those seven. It does **not** mirror it for **Fuse**: `casting.rs`'s
/// `base_cost_before_tax` arm additionally `extend`s `hybrid` and `phyrexian` and sums
/// `x_count` from the right half (`casting.rs:2625-2652`). A Fuse def whose right half
/// carried a hybrid pip, a Phyrexian pip or `{X}` would therefore be UNDER-predicted by
/// the auto-tap and refused by the engine.
///
/// **That is measured, not argued.** Adding `HybridMana::ColorColor(White, Blue)` to
/// `wear_tear.rs`'s Fuse cost in a scratch worktree made
/// `effective_cast_cost_with_additional` predict `{1}{R}{W}` (mana value **3**) while the
/// engine charged **4** and refused the fused cast with `InsufficientMana` from a pool
/// holding exactly the prediction — the same "clean offer, server rejection" the rest of
/// this batch exists to delete, one pip away. The card def was restored; nothing in the
/// corpus carries such a cost today, and this walk is what says so.
///
/// If it ever reddens, the fix is in `legal_actions.rs`'s `add` closure, not here.
#[test]
fn c2g_no_corpus_fuse_cost_carries_a_hybrid_phyrexian_or_x_component() {
    let mut seen = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    for def in all_cards() {
        for ability in &def.abilities {
            if let AbilityDefinition::Fuse { cost, .. } = ability {
                seen += 1;
                if !cost.hybrid.is_empty() || !cost.phyrexian.is_empty() || cost.x_count > 0 {
                    offenders.push(def.name.clone());
                }
            }
        }
    }
    assert!(
        seen >= 3,
        "non-vacuity floor: `all_cards()` yielded only {seen} `AbilityDefinition::Fuse` \
         costs, so this walk proves nothing"
    );
    assert!(
        offenders.is_empty(),
        "`legal_actions::effective_cast_cost_with_additional` adds only the seven numeric \
         components of a rider cost, while `casting.rs`'s fuse arm ALSO extends \
         hybrid/phyrexian and sums x_count. These defs make that divergence live, so the \
         auto-tap will under-fund their fused cast and the engine will refuse it: \
         {offenders:?}"
    );
}

/// **C3** — CR 601.2b. The scalar riders take the LAST announced entry, not the sum,
/// because `casting.rs`'s destructuring loop is a plain assignment
/// (`replicate_count = *count;`). Summing would make the auto-tap reach for strictly more
/// mana than the engine charges, find no plan, tap nothing, and let the cast be refused —
/// a 422 after a clean offer.
///
/// The same shape as `squad_cost_takes_the_last_announced_entry_exactly_as_the_engine_
/// does` in `legal_actions.rs`, extended to Replicate and Escalate, and asserted in BOTH
/// orders so it cannot pass by coincidence of which number is smaller.
#[test]
fn c3_scalar_riders_take_the_last_announced_entry_not_the_sum() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .object(corpus_object(
            &defs,
            P1,
            "Train of Thought",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Train of Thought");
    let mv = |riders: &[AdditionalCost]| {
        effective_cast_cost_with_additional(&state, P1, card, riders, &[], None)
            .expect("has a cost")
            .mana_value()
    };

    // Base {1}{U} = 2; replicate {1}{U} = 2 each.
    assert_eq!(mv(&[AdditionalCost::Replicate { count: 1 }]), 4);
    assert_eq!(
        mv(&[
            AdditionalCost::Replicate { count: 2 },
            AdditionalCost::Replicate { count: 1 },
        ]),
        4,
        "the LAST entry (count 1) is charged; summing would give 8"
    );
    assert_eq!(
        mv(&[
            AdditionalCost::Replicate { count: 1 },
            AdditionalCost::Replicate { count: 2 },
        ]),
        6,
        "and the other order, so this cannot pass by picking the smaller number"
    );

    // Escalate, on its own card, same property.
    let esc_state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .object(corpus_object(
            &defs,
            P1,
            "Collective Resistance",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let esc_card = id_of(&esc_state, "Collective Resistance");
    let esc_mv = |riders: &[AdditionalCost]| {
        effective_cast_cost_with_additional(&esc_state, P1, esc_card, riders, &[], None)
            .expect("has a cost")
            .mana_value()
    };
    // Base {1}{G} = 2; escalate {G} = 1 each.
    assert_eq!(esc_mv(&[AdditionalCost::EscalateModes { count: 2 }]), 4);
    assert_eq!(
        esc_mv(&[
            AdditionalCost::EscalateModes { count: 2 },
            AdditionalCost::EscalateModes { count: 1 },
        ]),
        3,
        "last-wins for escalate too; summing would give 5"
    );
}

/// **C4** — CR 702.174a. Gift adds nothing, and the two-sided form matters: the naming of
/// an opponent must not be silently free *because the rider was dropped*, so this also
/// asserts the engine really accepts the cast at the unchanged cost.
#[test]
fn c4_gift_adds_no_mana_and_the_engine_agrees() {
    let defs = defs_by_name();
    let build = |pool: ManaPool| {
        GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Nocturnal Hunger",
                ZoneId::Hand(P1),
            ))
            .object(ObjectSpec::creature(P2, "Some Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds")
    };
    let probe = build(ManaPool::default());
    let card = id_of(&probe, "Nocturnal Hunger");
    let base = effective_cast_cost(&probe, P1, card).expect("has a cost");
    let with_gift = effective_cast_cost_with_additional(
        &probe,
        P1,
        card,
        &[AdditionalCost::Gift { opponent: P2 }],
        &[],
        None,
    )
    .expect("has a cost");
    assert_eq!(
        base, with_gift,
        "CR 702.174a: naming an opponent IS the cost"
    );

    let exact = build(exact_pool(&base));
    let card = id_of(&exact, "Nocturnal Hunger");
    let bear = id_of(&exact, "Some Bear");
    let accepted = cast(
        exact,
        P1,
        card,
        vec![Target::Object(bear)],
        vec![],
        vec![AdditionalCost::Gift { opponent: P2 }],
    );
    assert!(
        accepted.is_ok(),
        "the engine must accept a gifted cast funded with the base cost alone: {:?}",
        accepted.err()
    );
}

/// **C5** — the defect this whole half exists to prevent, driven end to end.
///
/// A real `LocalGame` with `P1` human, no floating mana, four untapped Islands and a
/// Train of Thought in hand. The human announces `Replicate { count: 1 }` with
/// `auto_tap: true`. `LocalGame::auto_tap_commands_for` asks
/// `effective_cast_cost_with_additional` how much to tap, so if that function were still
/// Squad-only it would tap `{1}{U}` — two mana against a `{2}{U}{U}` charge — and the
/// engine would refuse the whole atomic sequence with `InsufficientMana`.
///
/// **Revert to watch red**: restore `effective_cast_cost_with_additional`'s early return
/// to `if squad_count == 0 { return Some(cost); }`. This test must fail with a
/// `LocalGameError::Rejected` naming the mana shortfall.
#[test]
fn c5_auto_tap_funds_an_announced_replicate_through_a_real_localgame() {
    let defs = defs_by_name();
    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .object(corpus_object(
            &defs,
            P1,
            "Train of Thought",
            ZoneId::Hand(P1),
        ));
    // Four Islands: exactly {2}{U}{U}, and not one mana more, so the test also fails if
    // the auto-tap over-reaches.
    for i in 0..4 {
        builder = builder.object(
            ObjectSpec::land(P1, &format!("Island {i}")).with_mana_ability(
                mtg_engine::ManaAbility::tap_for(mtg_engine::ManaColor::Blue),
            ),
        );
    }
    // Both seats need a library: the spell draws, and `LocalGame` runs real turns.
    for player in [P1, P2] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Filler {player:?} {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    let state = builder.build().expect("state builds");

    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    let (mut game, _events) = LocalGame::start(
        state,
        29_29,
        StubProvider,
        bots,
        human_seats,
        LocalGameLimits {
            max_turns: 4,
            max_commands: 2000,
            max_consecutive_passes: 500,
            record_journal: true,
        },
        true,
    )
    .expect("game starts");

    // `start_game` resets to `Step::Untap`, so walk to a main phase through real
    // priority passes until the cast is offered.
    let (seq, index) = {
        let mut found = None;
        for _ in 0..200 {
            let decision = match game.advance() {
                AdvanceOutcome::AwaitingHuman(d) => d,
                other => panic!("expected a human decision, got {other:?}"),
            };
            if let Some(i) = decision.actions.iter().position(|a| {
                matches!(a, LegalAction::CastSpell { additional_costs, .. }
                    if additional_costs
                        .counts
                        .iter()
                        .any(|c| c.kind == CountCostKind::Replicate))
            }) {
                found = Some((decision.seq, i));
                break;
            }
            let pass = decision
                .actions
                .iter()
                .position(|a| matches!(a, LegalAction::PassPriority))
                .expect("PassPriority is always offered at a priority window");
            game.submit(
                decision.seq,
                HumanChoice {
                    action_index: pass,
                    params: ActionParams::default(),
                },
            )
            .expect("passing priority is legal");
        }
        found.expect("Train of Thought was never offered with a Replicate rider")
    };

    let events = game
        .submit(
            seq,
            HumanChoice {
                action_index: index,
                params: ActionParams {
                    additional_costs: vec![AdditionalCost::Replicate { count: 1 }],
                    auto_tap: true,
                    ..Default::default()
                },
            },
        )
        .expect(
            "SR-38: the offer promised a payable Replicate, so the auto-tap must fund it \
             and the engine must accept the cast",
        );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "the cast really happened; events were {events:?}"
    );
    // Every Island really was tapped — the auto-tap paid {2}{U}{U}, not {1}{U}.
    let tapped = game
        .state()
        .objects()
        .values()
        .filter(|o| o.controller == P1 && o.characteristics.name.starts_with("Island"))
        .filter(|o| o.status.tapped)
        .count();
    assert_eq!(
        tapped, 4,
        "the replicate payment needs all four Islands; a Squad-only cost helper would \
         have tapped two"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// E — END TO END
// ═══════════════════════════════════════════════════════════════════════════════

/// **E1** — CR 702.56a. An announced replicate really copies the spell: a
/// `KeywordTrigger { keyword: Replicate, data: SpellCopy { copy_count } }` goes on the
/// stack above the spell, and once everything resolves Train of Thought has drawn TWO
/// cards rather than one.
///
/// Verified by counting the library rather than by an `ObjectId`, because every drawn
/// card is a new object as far as its zone is concerned (CR 400.7).
#[test]
fn e1_an_announced_replicate_really_produces_the_extra_copy() {
    let defs = defs_by_name();
    let build = |riders: Vec<AdditionalCost>, pool: ManaPool| {
        let mut b = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Train of Thought",
                ZoneId::Hand(P1),
            ));
        for i in 0..10 {
            b = b.object(ObjectSpec::card(P1, &format!("Filler {i}")).in_zone(ZoneId::Library(P1)));
        }
        let state = b.build().expect("state builds");
        let card = id_of(&state, "Train of Thought");
        let before = state.objects_in_zone(&ZoneId::Library(P1)).len();
        let (state, _) =
            cast(state, P1, card, vec![], vec![], riders).expect("the cast must be accepted");
        (state, before)
    };

    // Declining: one draw.
    let (plain, before) = build(
        vec![],
        ManaPool {
            blue: 1,
            colorless: 1,
            ..Default::default()
        },
    );
    assert_eq!(
        plain.stack_objects().len(),
        1,
        "CR 702.56a: no replicate trigger when the cost was not paid"
    );
    let plain = resolve_stack(plain);
    assert_eq!(
        before - plain.objects_in_zone(&ZoneId::Library(P1)).len(),
        1,
        "one draw with no replicate"
    );

    // Paying once: the trigger, then two draws.
    let (replicated, before) = build(
        vec![AdditionalCost::Replicate { count: 1 }],
        ManaPool {
            blue: 2,
            colorless: 2,
            ..Default::default()
        },
    );
    let top = replicated
        .stack_objects()
        .back()
        .expect("stack is not empty");
    assert!(
        matches!(
            top.kind,
            mtg_engine::StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Replicate,
                data: mtg_engine::state::stack::TriggerData::SpellCopy { copy_count: 1, .. },
                ..
            }
        ),
        "CR 702.56a: the copy trigger goes on top of the spell; got {:?}",
        top.kind
    );
    let replicated = resolve_stack(replicated);
    assert_eq!(
        before - replicated.objects_in_zone(&ZoneId::Library(P1)).len(),
        2,
        "CR 702.56a: the copy draws too, so the replicated cast draws twice"
    );
}

/// **E2** — CR 702.42a. An announced entwine really resolves ALL modes where declining
/// resolves only the chosen one.
///
/// Goblin War Party's two modes are "create three 1/1 Goblins" (mode 0) and "creatures
/// you control get +1/+1 and gain haste" (mode 1). A pre-placed 2/2 bear is the
/// discriminator: entwined it is layer-resolved to 3/3, declined it stays 2/2, and the
/// Goblins arrive either way, so a test that only counted tokens would pass for both.
#[test]
fn e2_an_announced_entwine_really_resolves_every_mode() {
    let defs = defs_by_name();
    let build = |riders: Vec<AdditionalCost>, modes: Vec<usize>, pool: ManaPool| {
        let state = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(P1, pool)
            .object(corpus_object(
                &defs,
                P1,
                "Goblin War Party",
                ZoneId::Hand(P1),
            ))
            .object(ObjectSpec::creature(P1, "Witness Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");
        let card = id_of(&state, "Goblin War Party");
        let (state, _) =
            cast(state, P1, card, vec![], modes, riders).expect("the cast must be accepted");
        resolve_stack(state)
    };
    let bear_power = |state: &GameState| {
        let bear = id_of(state, "Witness Bear");
        mtg_engine::calculate_characteristics(state, bear)
            .expect("the bear is on the battlefield")
            .power
            .expect("a 2/2 has a printed power")
    };

    // Declined: mode 0 only.
    let declined = build(
        vec![],
        vec![0],
        ManaPool {
            red: 1,
            colorless: 3,
            ..Default::default()
        },
    );
    assert_eq!(tokens_named(&declined, "Goblin", P1), 3, "mode 0 resolved");
    assert_eq!(
        bear_power(&declined),
        2,
        "CR 700.2: declining entwine chooses ONE mode, so the pump never happens"
    );

    // Entwined: both modes. `modes_chosen` is left empty — `casting.rs` charges and
    // resolves all modes itself when entwine is paid.
    let entwined = build(
        vec![AdditionalCost::Entwine],
        vec![],
        ManaPool {
            red: 2,
            colorless: 5,
            ..Default::default()
        },
    );
    assert_eq!(tokens_named(&entwined, "Goblin", P1), 3, "mode 0 resolved");
    assert_eq!(
        bear_power(&entwined),
        3,
        "CR 702.42a: paying entwine chooses BOTH modes, so the +1/+1 applies too"
    );
}

/// **E3** — CR 702.174a/d. An announced gift really gives the NAMED opponent the Food,
/// and never the caster or the other seat. Three players, so "the named opponent" is a
/// real choice rather than the only possibility.
#[test]
fn e3_an_announced_gift_really_gives_the_named_opponent_the_food() {
    let defs = defs_by_name();
    let p3 = PlayerId(3);
    let build = |riders: Vec<AdditionalCost>| {
        let state = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .add_player(p3)
            .with_registry(corpus_registry())
            .active_player(P1)
            .player_mana(
                P1,
                ManaPool {
                    black: 1,
                    colorless: 2,
                    ..Default::default()
                },
            )
            .object(corpus_object(
                &defs,
                P1,
                "Nocturnal Hunger",
                ZoneId::Hand(P1),
            ))
            .object(ObjectSpec::creature(P2, "Doomed Bear", 2, 2).in_zone(ZoneId::Battlefield))
            .build()
            .expect("state builds");
        let card = id_of(&state, "Nocturnal Hunger");
        let bear = id_of(&state, "Doomed Bear");
        let (state, _) = cast(state, P1, card, vec![Target::Object(bear)], vec![], riders)
            .expect("the cast must be accepted");
        // Three seats now, so `resolve_stack`'s two-seat loop is not enough.
        let mut state = state;
        for _ in 0..40 {
            if state.stack_objects().is_empty() {
                break;
            }
            for pl in [P1, P2, p3] {
                let (next, _) =
                    process_command(state.clone(), Command::PassPriority { player: pl })
                        .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
                state = next;
            }
        }
        state
    };

    let gifted = build(vec![AdditionalCost::Gift { opponent: p3 }]);
    assert_eq!(
        tokens_named(&gifted, "Food", p3),
        1,
        "CR 702.174d: the NAMED opponent creates the Food"
    );
    assert_eq!(tokens_named(&gifted, "Food", P1), 0, "never the caster");
    assert_eq!(
        tokens_named(&gifted, "Food", P2),
        0,
        "never the unnamed seat"
    );
    // CR 702.174b — Nocturnal Hunger's own "if the gift wasn't promised, you lose 2
    // life" clause is the second observable, and it discriminates the two branches from
    // the caster's side as well as the recipient's.
    assert_eq!(
        gifted.player(P1).expect("P1 exists").life_total,
        40,
        "the gift WAS promised, so the 2-life clause does not fire"
    );

    let declined = build(vec![]);
    assert_eq!(
        tokens_named(&declined, "Food", p3),
        0,
        "no gift announced, no Food anywhere"
    );
    assert_eq!(
        declined.player(P1).expect("P1 exists").life_total,
        38,
        "CR 702.174b: the gift was not promised, so the caster loses 2 life"
    );
}

/// **E4** — CR 118.12a. Declining every optional rider produces a `Command` byte-identical
/// to a plain cast, so a client that renders six new pickers and a human who touches none
/// of them changes nothing at all.
///
/// Built through `action_to_command_with_params`, which is the real mapping the browser
/// and the TUI both go through, on a card that offers a rider of every shape reachable at
/// once: Train of Thought (a `counts` entry) plus a Glacial Ray in hand and — because
/// splice needs an Arcane spell — the Arcane Reach Through Mists as the subject.
#[test]
fn e4_declining_every_optional_rider_builds_the_same_command_as_a_plain_cast() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 2,
                red: 2,
                colorless: 4,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Reach Through Mists",
            ZoneId::Hand(P1),
        ))
        .object(corpus_object(&defs, P1, "Glacial Ray", ZoneId::Hand(P1)))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Reach Through Mists");

    let offer = StubProvider
        .legal_actions(&state, P1)
        .into_iter()
        .find(|a| matches!(a, LegalAction::CastSpell { card: c, .. } if *c == card))
        .expect("the Arcane spell is offered");
    // Non-vacuity: the offer really does carry an optional rider to decline.
    let LegalAction::CastSpell {
        additional_costs, ..
    } = &offer
    else {
        unreachable!("matched by discriminant above");
    };
    assert!(
        additional_costs.splice.is_some(),
        "this test is only meaningful while the offer carries something declinable"
    );

    let declined =
        mtg_simulator::action_to_command_with_params(&state, P1, &offer, &ActionParams::default())
            .expect("declining every rider must map cleanly");

    // A plain cast built the same way, from an offer with no rider at all.
    let plain_state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry())
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 2,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Reach Through Mists",
            ZoneId::Hand(P1),
        ))
        .build()
        .expect("state builds");
    let plain_card = id_of(&plain_state, "Reach Through Mists");
    let plain_offer = StubProvider
        .legal_actions(&plain_state, P1)
        .into_iter()
        .find(|a| matches!(a, LegalAction::CastSpell { card: c, .. } if *c == plain_card))
        .expect("offered");
    let plain = mtg_simulator::action_to_command_with_params(
        &plain_state,
        P1,
        &plain_offer,
        &ActionParams::default(),
    )
    .expect("plain cast maps");

    let (Command::CastSpell(declined), Command::CastSpell(plain)) = (&declined, &plain) else {
        panic!("both must be CastSpell");
    };
    assert!(
        declined.additional_costs.is_empty(),
        "CR 118.12a: an unanswered optional rider must not become an announced one; got \
         {:?}",
        declined.additional_costs
    );
    assert_eq!(
        declined.additional_costs, plain.additional_costs,
        "declining every rider must produce the same additional-cost list as a spell that \
         had none to decline"
    );
    assert_eq!(declined.x_value, plain.x_value);
    assert_eq!(declined.modes_chosen, plain.modes_chosen);
    assert_eq!(declined.targets, plain.targets);
    assert_eq!(declined.alt_cost, plain.alt_cost);
}
