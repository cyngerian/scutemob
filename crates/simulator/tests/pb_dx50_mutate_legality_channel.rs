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

use std::collections::HashMap;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition, CardId,
    CardRegistry, CardType, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, Step, SubType, SuperType,
    TypeLine, ZoneId,
};
use mtg_simulator::{
    action_to_command_with_params, build_registry, ActionParams, LegalAction, LegalActionProvider,
    StubProvider,
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
        1,
        "PB-DX50: the offer layer emits ONE action per legal host. It emitted two before \
         (one per `(host, on_top)` pair) until CR 702.140c moved the over/under choice to \
         resolution time -- that halving is the movement this batch budgeted in writing. \
         Offers: {offers:#?}"
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
    assert_eq!(
        offers.len(),
        1,
        "precondition: one offer to drive (PB-DX50 halved the mutate offer count)"
    );

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
        1,
        "CR 702.140a / CR 108.3: the host axis is OWNERSHIP, not control -- a host the \
         caster owns but an opponent controls is legal. One offer per host since PB-DX50. \
         Offers: {offers:#?}"
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

// ── c5 — the DECK-LEGAL proof, on two real corpus cards ─────────────────────────

/// The corpus's mutator: `Complete`, deck-legal, mutate `{1}{G}{G}`, a Beast (so not
/// itself a Human, and its name is unambiguous against the host's). Same card
/// `pb_dx29_mutate_on_top.rs` drives.
const REAL_MUTATOR: &str = "Gemrazer";
/// The corpus's Ward host: `Complete`, deck-legal,
/// `AbilityDefinition::Keyword(KeywordAbility::Ward(2))`, and *Legendary Creature —
/// Merfolk Wizard*, i.e. **non-Human**, so CR 702.140a admits it as a mutate host.
const REAL_WARD_HOST: &str = "Adrix and Nev, Twincasters";

/// Every card definition keyed by NAME — the shape `enrich_spec_from_def` wants, mirroring
/// `pb_dx29_mutate_on_top.rs`'s own helper rather than inventing a second idiom.
fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// CR 702.21a / CR 702.140a — **the deck-legal proof, and the only probe in this batch
/// that touches real corpus cards.**
///
/// Every other probe in PB-DX50 uses synthetic defs. That is the shape this project keeps
/// punishing: PB-DX43's *existence is never sufficiency*, and PB-DX47's finding that a
/// probe was green only because its fixture was a naked `ObjectSpec::card()` — a shape no
/// production path can produce. So this one builds through `build_registry()` +
/// `card_name_to_id` + `enrich_spec_from_def`, the production pregame path, and names two
/// cards a player can legally put in a deck today.
///
/// **Board**: P1 OWNS `Adrix and Nev, Twincasters` (CR 702.140a's axis is OWNERSHIP,
/// CR 108.3) and P2 CONTROLS it (CR 702.21a's axis is CONTROL, CR 109.4 — an ordinary
/// Mind Control position). P1 casts `Gemrazer` for its mutate cost targeting Adrix.
///
/// **Verdict, by COUNT and never `>= 1`** (PB-DX48's rule: a double-dispatch design
/// satisfies every `>= 1` assertion in the tree): exactly one `PermanentTargeted` naming
/// Adrix, and exactly one Ward `AbilityTriggered` whose `source_object_id` is Adrix and
/// whose controller is P2.
///
/// **CR 704.5j (the legend rule) was CHECKED, not assumed, and it does not perturb this
/// fixture.** The rule fires only when ONE player controls two or more legendary
/// permanents with the same name. P2 controls exactly one Adrix and P1 controls no
/// legendary permanent at all, so no SBA applies; the assertion below re-checks that
/// explicitly rather than trusting the reading. The merge itself is never reached — the
/// verdict is taken at ANNOUNCEMENT, with the ward trigger sitting on top of the still-
/// unresolved mutating creature spell — so the question of what the merged permanent's
/// name and supertypes would be (CR 729.2a) never arises here either.
#[test]
fn c5_deck_legal_gemrazer_onto_adrix_fires_ward_exactly_once() {
    let defs = card_defs_by_name();
    let p1 = p(1);
    let p2 = p(2);

    // Preconditions on the real defs, asserted rather than assumed. If a future card-def
    // edit demotes either card or removes the Ward marker, this probe must say WHY it
    // stopped being a deck-legal proof instead of quietly becoming a different test.
    let mutator_def = defs
        .get(REAL_MUTATOR)
        .unwrap_or_else(|| panic!("{REAL_MUTATOR} must exist in all_cards()"));
    let host_def = defs
        .get(REAL_WARD_HOST)
        .unwrap_or_else(|| panic!("{REAL_WARD_HOST} must exist in all_cards()"));
    assert_eq!(
        mutator_def.completeness,
        mtg_engine::cards::Completeness::Complete,
        "AC 7301 asks for a DECK-LEGAL fixture: {REAL_MUTATOR} must be Complete"
    );
    assert_eq!(
        host_def.completeness,
        mtg_engine::cards::Completeness::Complete,
        "AC 7301 asks for a DECK-LEGAL fixture: {REAL_WARD_HOST} must be Complete"
    );
    assert!(
        host_def.abilities.iter().any(|a| matches!(
            a,
            mtg_engine::AbilityDefinition::Keyword(KeywordAbility::Ward(_))
        )),
        "{REAL_WARD_HOST} must declare KeywordAbility::Ward -- without it this probe \
         measures nothing about CR 702.21a"
    );
    assert!(
        !host_def
            .types
            .subtypes
            .contains(&SubType("Human".to_string())),
        "CR 702.140a: the host must be non-Human. {REAL_WARD_HOST} is a Merfolk Wizard."
    );
    assert!(
        host_def.types.supertypes.contains(&SuperType::Legendary),
        "precondition for the CR 704.5j check below: {REAL_WARD_HOST} really is legendary, \
         so the legend-rule question is a real one and not a hypothetical"
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(build_registry())
        // Mana in the POOL, not on lands: `can_afford` answers "the pool alone covers
        // this" first, so the offer this probe reads is not entangled with the mana
        // solver (`pb_dx29_mutate_on_top.rs`'s own reasoning).
        .player_mana(
            p1,
            ManaPool {
                green: 3,
                ..Default::default()
            },
        )
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, REAL_MUTATOR)
                .with_card_id(card_name_to_id(REAL_MUTATOR))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        ))
        // Owned by p1 (CR 702.140a), controlled by p2 (CR 702.21a).
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, REAL_WARD_HOST)
                .with_card_id(card_name_to_id(REAL_WARD_HOST))
                .in_zone(ZoneId::Battlefield)
                .controlled_by(p2),
            &defs,
        ))
        .build()
        .expect("PB-DX50 deck-legal mutate fixture must build");
    state.turn_mut().priority_holder = Some(p1);

    let gemrazer_id = find_object(&state, REAL_MUTATOR);
    let adrix_id = find_object(&state, REAL_WARD_HOST);
    assert_eq!(
        state.objects().get(&adrix_id).unwrap().owner,
        p1,
        "precondition: CR 108.3 -- p1 OWNS Adrix"
    );
    assert_eq!(
        state.objects().get(&adrix_id).unwrap().controller,
        p2,
        "precondition: CR 109.4 -- p2 CONTROLS Adrix, which is what makes CR 702.21a's \
         'an opponent controls' clause reachable at all"
    );

    // CR 704.5j: the legend rule fires only when ONE player controls two or more legendary
    // permanents with the same name. Checked by counting, not by reading the rule.
    let legendary_per_controller: HashMap<PlayerId, usize> = state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.characteristics.supertypes.contains(&SuperType::Legendary)
        })
        .fold(HashMap::new(), |mut acc, o| {
            *acc.entry(o.controller).or_insert(0) += 1;
            acc
        });
    assert!(
        legendary_per_controller.values().all(|n| *n <= 1),
        "CR 704.5j: no player may control two legendary permanents here, or an SBA would \
         remove one and perturb every count below. Measured: {legendary_per_controller:?}"
    );

    // The offer layer must OFFER this host -- SR-38's half, on real cards.
    let offers = mutate_offers(&state);
    assert_eq!(
        offers.len(),
        1,
        "SR-38: `Gemrazer` onto `Adrix and Nev, Twincasters` is a legal mutate \
         (CR 702.140a: non-Human, owned by the caster), so the real offer layer must emit \
         one action for it -- one per host since PB-DX50 moved the over/under choice to \
         resolution (CR 702.140c). Ward does NOT make a target illegal -- it taxes it \
         (CR 702.21a). Offers: {offers:#?}"
    );
    let offer = offers
        .iter()
        .find(|a| matches!(a, LegalAction::CastWithMutate { mutate_target, .. } if *mutate_target == adrix_id))
        .expect("one of the offers must name Adrix as the host");

    // Offer -> production mapping -> engine. Never hand-assembled.
    let command = action_to_command_with_params(&state, p1, offer, &ActionParams::default())
        .expect("the production mapping must build a Command for its own offer");
    let (after, events) = process_command(state.clone(), command)
        .expect("SR-38: an action the offer layer emitted must be accepted by the engine");
    // CR 400.7: moving hand -> stack makes a NEW object, so the post-cast Gemrazer has a
    // DIFFERENT ObjectId. The first draft of this line asserted the ids were equal and was
    // refuted on its first run -- recorded rather than quietly corrected, because it is the
    // engine's #1 bug class and this file should not read as if it were unaware of it.
    let stack_gemrazer = find_object(&after, REAL_MUTATOR);
    assert_ne!(
        stack_gemrazer, gemrazer_id,
        "CR 400.7: the cast created a new object; the hand id is dead"
    );
    assert_eq!(
        after.objects().get(&stack_gemrazer).unwrap().zone,
        ZoneId::Stack,
        "the Gemrazer card is in the Stack zone while its spell is on the stack"
    );

    let targeted = events
        .iter()
        .filter(|e| {
            matches!(e, GameEvent::PermanentTargeted { target_id, .. } if *target_id == adrix_id)
        })
        .count();
    assert_eq!(
        targeted, 1,
        "CR 702.21a: EXACTLY one PermanentTargeted naming Adrix. Before PB-DX50 this was \
         ZERO -- the mutate host never entered the StackObject's `targets`, which is the \
         only thing `permanent_targeted_events` reads. Events: {events:#?}"
    );

    let ward_triggers = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { controller, source_object_id, .. }
                if *source_object_id == adrix_id && *controller == p2
            )
        })
        .count();
    assert_eq!(
        ward_triggers, 1,
        "CR 702.21a: EXACTLY one Ward AbilityTriggered, controlled by Adrix's CONTROLLER \
         p2. This is `OOS-DX25-1`'s headline, on two deck-legal `Complete` corpus cards \
         cast through the real offer layer and the real command mapping. Events: \
         {events:#?}"
    );
    assert_eq!(
        after.stack_objects().len(),
        2,
        "the mutating creature spell plus the ward trigger on top of it"
    );
    assert!(
        after
            .objects()
            .get(&adrix_id)
            .is_some_and(|o| o.zone == ZoneId::Battlefield),
        "CR 704.5j did not fire: Adrix is still on the battlefield, so nothing about the \
         counts above is an artefact of a legend-rule sacrifice"
    );
}

// ── c6 — the offer set IS the engine's answer, on a four-class board ────────────

/// A board carrying one of every class the CR 702.140a decision has to separate, so the
/// equality below cannot be satisfied by accident:
///
/// | host | class | CR |
/// |---|---|---|
/// | `DX50C Wolf Host` | legal | — |
/// | `DX50C Human Host` | Human, p1's | CR 702.140a's "non-Human" |
/// | `DX50C Shroud Host` | shroud, p1's | CR 702.18a |
///
/// **The `/review` proposed "a hexproof host" here and it would NOT have discriminated.**
/// CR 702.11b is *"can't be the target of spells **your opponents control**"*, and this
/// mutate is cast by the host's own controller, so a hexproof host of p1's is a perfectly
/// legal mutate target — the first draft of this fixture asserted otherwise and the
/// engine's own answer refuted it (`{Wolf, Hexproof}`, not `{Wolf}`). Shroud (CR 702.18a,
/// *"can't be the target of spells or abilities"*, full stop) is the protection-family
/// class that actually separates here, which is why `c1` uses it too.
/// | `DX50C Foreign Host` | owned by p2 | CR 702.140a's "same owner as this spell" |
fn four_class_board() -> GameState {
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

    let mk = |owner: PlayerId, name: &str, sub: &str, kw: Option<KeywordAbility>| {
        let mut o = ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("dx50c-wolf-host".to_string()))
            .with_types(vec![CardType::Creature])
            .with_subtypes(vec![SubType(sub.to_string())])
            .controlled_by(owner);
        o.power = Some(2);
        o.toughness = Some(3);
        if let Some(k) = kw {
            o = o.with_keyword(k);
        }
        o
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![beast_def(), host_def()]))
        .object(beast)
        .object(mk(p1, HOST, "Wolf", None))
        .object(mk(p1, "DX50C Human Host", "Human", None))
        .object(mk(
            p1,
            "DX50C Shroud Host",
            "Wolf",
            Some(KeywordAbility::Shroud),
        ))
        .object(mk(p2, "DX50C Foreign Host", "Wolf", None))
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

/// **The consumer gate, behaviourally.** The set of hosts the offer layer emits is
/// EXACTLY `queries::legal_mutate_hosts`' answer — not a set that agrees with it on this
/// board, but the same set, asserted against the engine's own live return value.
///
/// # Why this exists on top of `core::pb_dx50_mutate_site_roster::r3`
///
/// `r3`'s first draft polices the DEFINITION (`mutate_target_requirement`'s uniqueness and
/// its call sites) and is structurally blind to the CONSUMER — and the consumer is where
/// **all four** historical hand-rolled copies lived. `crates/simulator/src/legal_actions.rs`
/// never names `mutate_target_requirement` at all; it calls `queries::legal_mutate_hosts`.
/// The batch's own `/review` defeated `r3` twice by planting a second host predicate there,
/// with **all four roster tests green** both times:
///
/// 1. one omitting CR 702.140a's non-Human conjunct — the literal SR-38 defect, a Human
///    host offered and then refused by the cast path;
/// 2. one spelling the subtype `SubType(String::from("Hum") + "an")`, which no `"Human"`
///    string-literal census can see.
///
/// **A source scan for a string literal cannot be made evasion-proof**, and pretending
/// otherwise is how a gate becomes a comment. So the load-bearing assertion moves here,
/// to a place where the offer layer's answer has to equal the engine's, whatever either
/// one is spelled like.
///
/// **Revert to watch red**: replace `legal_mutate_hosts(state, player, obj.id)` in
/// `legal_actions.rs` with any hand-rolled filter — both of the defeats above redden this.
#[test]
fn c6_the_offered_host_set_is_exactly_the_engines_own_answer() {
    let state = four_class_board();
    let beast_id = find_object(&state, BEAST);

    let engine_answer: std::collections::BTreeSet<ObjectId> =
        mtg_engine::rules::queries::legal_mutate_hosts(&state, p(1), beast_id)
            .into_iter()
            .collect();
    let offered: std::collections::BTreeSet<ObjectId> = mutate_offers(&state)
        .iter()
        .map(|a| match a {
            LegalAction::CastWithMutate { mutate_target, .. } => *mutate_target,
            other => panic!("filtered to CastWithMutate, got {other:?}"),
        })
        .collect();

    // Non-vacuity, stated in both directions, because set equality between two empty sets
    // is the failure mode this probe is most exposed to.
    let legal_host = find_object(&state, HOST);
    assert_eq!(
        engine_answer,
        [legal_host]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>(),
        "precondition: on this four-class board the engine's own CR 702.140a answer is \
         exactly the Wolf. If this moved, the FIXTURE changed, not the offer layer -- \
         re-derive before touching `legal_actions.rs`. Answer: {engine_answer:?}"
    );
    let creatures_on_battlefield = state
        .objects_in_zone(&ZoneId::Battlefield)
        .iter()
        .filter(|o| o.characteristics.card_types.contains(&CardType::Creature))
        .count();
    assert_eq!(
        creatures_on_battlefield, 4,
        "precondition: four candidate hosts are present, so the answer being ONE is a \
         real filter rather than an empty board"
    );

    assert_eq!(
        offered, engine_answer,
        "SR-38 / CR 702.140a: the mutate offer set must BE \
         `queries::legal_mutate_hosts`' answer, not a second predicate that agrees with \
         it. A host in `offered` but not in the engine's answer is a clean offer followed \
         by a guaranteed refusal; a host in the engine's answer but not `offered` is a \
         legal play no client can make. offered={offered:?} engine={engine_answer:?}"
    );
}
