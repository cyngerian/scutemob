//! PB-DX15a — behavioural probes for the three CR 608.2e / CR 101.4 APNAP sites that
//! shipped with **no test of their own**.
//!
//! # The rule
//!
//! CR 101.4: *"If multiple players would make choices and/or take actions at the same
//! time, the active player (referred to as AP) chooses first, then each other player in
//! turn order does so (each of those players is referred to as NAP)."*
//! CR 608.2e: *"Some spells and abilities have multiple steps or actions [...] that
//! involve multiple players. [...] the affected players do so in APNAP order."*
//!
//! # What this file is for
//!
//! PB-DX15a routed **five** iteration sites through the new helper
//! `rules::abilities::apnap_order_all_players`. Only two of them were probed:
//! `resolve_player_target_list`'s `PlayerTarget::EachPlayer` (search / scry / surveil /
//! discard, in `pb_dp9_effect_choice.rs`) and the simulator channel. The `/review`
//! reverted each of the other three to the pre-batch `state.players.keys()` walk and the
//! whole 4,829-test workspace stayed green. This file is the missing coverage:
//!
//! | # | site (`crates/engine/src/effects/mod.rs`) | probe |
//! |---|------------------------------------------|-------|
//! | 1 | `resolve_effect_target_list`'s `EffectTarget::EachPlayer` / `EachOpponent` | [`t1_each_opponent_damage_is_dealt_in_apnap_order`] |
//! | 2 | `Effect::LivingDeath`'s three-step player walk | [`t2_living_death_exiles_and_returns_in_apnap_order`] |
//! | 3 | `Effect::ReturnAllFromGraveyardToBattlefield`'s graveyard-owner walk | [`t3_return_all_unique_names_keeps_the_apnap_first_copy`] |
//!
//! # The constraint every probe here obeys, and why it is the whole point
//!
//! `GameStateBuilder` seeds `turn_order` from the order `add_player` was called
//! (`state/builder.rs:325`), and every fixture in this repository calls it in ascending
//! `PlayerId` order. So when the ACTIVE player is the LOWEST id, "rotate turn order to
//! start at the active player" is the **identity**, and APNAP order is byte-identical to
//! ascending `PlayerId` order. A test built on such a fixture pins nothing about
//! CR 101.4 whatever its doc comment claims — which is exactly how `OOS-DP9-8` survived
//! from PB-DP9 to PB-DX15a behind a test that said it was pinning the deviation. See
//! `pb_dp9_effect_choice.rs::test_dx15a_active_lowest_id_makes_apnap_and_ascending_indistinguishable`
//! for that statement as a standalone gate.
//!
//! Every fixture below therefore has **three seats and `p(2)` active**, so APNAP is
//! `[p2, p3, p1]` and ascending is `[p1, p2, p3]` — different in every position — and
//! every probe calls [`assert_fixture_can_express_apnap`] before it asserts anything, so
//! it cannot go quietly vacuous if a future edit moves the fixture's active player.
//! Every assertion is a **full ordered list**, never a set, a count or a membership.

use std::collections::HashMap;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, CombatDamageTarget, Command, Effect, GameEvent, GameState,
    GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step,
    TargetFilter, TypeLine, ZoneId,
};

// ── Shared helpers ───────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// The REAL corpus `CardDefinition` of that name, from `all_cards()`.
///
/// SR-36's rule: enumerate `all_cards()`, never grep source. `ObjectSpec::card()` builds
/// a naked object with no types, no P/T and no abilities, so every card object below is
/// built through [`card_in`], which enriches from the def and pins the `card_id` the ETB
/// chain looks the def back up by.
fn corpus_def(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("no corpus CardDefinition is named {name:?}"))
}

fn defs_map(defs: &[CardDefinition]) -> HashMap<String, CardDefinition> {
    defs.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

fn card_in(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let card_id: CardId = defs
        .get(name)
        .map(|d| d.card_id.clone())
        .unwrap_or_else(|| panic!("{name:?} is not in this fixture's registry"));
    enrich_spec_from_def(ObjectSpec::card(owner, name), defs)
        .with_card_id(card_id)
        .in_zone(zone)
}

/// **The gate that stops every probe in this file from going vacuous.**
///
/// CR 101.4 defines APNAP as the active player followed by the remaining players in turn
/// order. Asserted here as three separate facts so the failure message names which one
/// broke: three or more seats; the active player is NOT the lowest `PlayerId`; and the
/// helper's answer genuinely differs from ascending `PlayerId` in every position.
fn assert_fixture_can_express_apnap(state: &GameState) {
    let mut ascending: Vec<PlayerId> = state.players().keys().copied().collect();
    ascending.sort();
    assert!(
        ascending.len() >= 3,
        "CR 101.4 needs 3+ seats to separate APNAP from ascending PlayerId in more \
         than one position; this fixture has {}",
        ascending.len()
    );
    let active = state.turn().active_player;
    assert_ne!(
        active, ascending[0],
        "the ACTIVE player is the lowest PlayerId, so rotating turn order to start at \
         it is the identity and this fixture cannot tell APNAP from ascending order"
    );
    let apnap = mtg_engine::rules::abilities::apnap_order_all_players(state);
    assert_eq!(
        apnap,
        vec![p(2), p(3), p(1)],
        "with p2 active over turn order [p1, p2, p3], CR 101.4 APNAP is [p2, p3, p1]"
    );
    assert_ne!(
        apnap, ascending,
        "APNAP and ascending PlayerId must be different lists, or nothing below \
         discriminates"
    );
}

/// A three-seat table with `p(2)` active — see the module doc for why `p(2)` and not
/// `p(1)`. Asserts [`assert_fixture_can_express_apnap`] before handing the state back.
fn table_active_p2(defs: &[CardDefinition], specs: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(defs.to_vec()))
        .active_player(p(2))
        .at_step(Step::PreCombatMain);
    for spec in specs {
        builder = builder.object(spec);
    }
    let state = builder.build().expect("fixture should build");
    assert_fixture_can_express_apnap(&state);
    state
}

fn give_mana(state: &mut GameState, player: PlayerId, mana: &[(ManaColor, u32)]) {
    let pool = &mut state
        .players_mut()
        .get_mut(&player)
        .expect("player exists")
        .mana_pool;
    for (color, amount) in mana {
        pool.add(*color, *amount);
    }
}

fn cast_cmd(player: PlayerId, card: ObjectId) -> Command {
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

fn object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == zone)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("no object named {name:?} in {zone:?}"))
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| "<gone>".to_string())
}

fn names_in_zone(state: &GameState, zone: ZoneId) -> Vec<String> {
    state
        .zones()
        .get(&zone)
        .map(|z| z.object_ids())
        .unwrap_or_default()
        .into_iter()
        .map(|id| name_of(state, id))
        .collect()
}

/// `p(2)` casts the named card out of their hand, then the table passes around
/// (`p2 → p3 → p1`, the CR 117.3c/CR 101.4 order for an active `p(2)`) until the stack
/// is empty. Every event from the cast and from every resolution is returned, in order.
fn cast_and_settle(state: GameState, name: &str) -> (GameState, Vec<GameEvent>) {
    let card = object_in_zone(&state, name, ZoneId::Hand(p(2)));
    let (mut state, mut events) =
        process_command(state, cast_cmd(p(2), card)).expect("the cast should be accepted");
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        for pl in [p(2), p(3), p(1)] {
            let (s, ev) = process_command(state, Command::PassPriority { player: pl })
                .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
            state = s;
            events.extend(ev);
            if state.stack_objects().is_empty() {
                break;
            }
        }
        guard += 1;
        assert!(guard < 20, "the stack never settled");
    }
    (state, events)
}

/// The ordered list of players who took non-combat damage, in the order the engine
/// emitted the events.
fn damaged_players_in_order(events: &[GameEvent]) -> Vec<PlayerId> {
    events
        .iter()
        .filter_map(|e| match e {
            GameEvent::DamageDealt {
                target: CombatDamageTarget::Player(pl),
                ..
            } => Some(*pl),
            _ => None,
        })
        .collect()
}

/// The ordered list of `(controller, name)` pairs for every permanent that entered the
/// battlefield, in emission order. The name is read from the FINAL state, because
/// `PermanentEnteredBattlefield` carries the post-move (CR 400.7 new-object) id.
fn entered_battlefield_in_order(
    state: &GameState,
    events: &[GameEvent],
) -> Vec<(PlayerId, String)> {
    events
        .iter()
        .filter_map(|e| match e {
            GameEvent::PermanentEnteredBattlefield { player, object_id } => {
                Some((*player, name_of(state, *object_id)))
            }
            _ => None,
        })
        .collect()
}

// ── Site 1 — `resolve_effect_target_list`'s EachPlayer / EachOpponent ────────

#[test]
/// **Site 1.** CR 608.2e / CR 101.4 — `EffectTarget::EachOpponent` resolves to its
/// players in APNAP order, so "deals 1 damage to each opponent" damages them in that
/// order.
///
/// # Why this site needed its own probe
///
/// `OOS-DP9-8` names ONE function, `resolve_player_target_list`. `resolve_effect_target_list`
/// is a **separate** function with its own `EachPlayer`/`EachOpponent` arms, reached by
/// every effect whose target is an `EffectTarget` rather than a `PlayerTarget` — and it
/// walked the same `imbl::OrdMap` in the same ascending-`PlayerId` order. Fixing only the
/// function the seed named would have left the two disagreeing about the order of the
/// same set of players.
///
/// # The card is real
///
/// `voldaren_epicure` is a deck-legal `Complete` corpus def whose printed ETB trigger is
/// *"it deals 1 damage to each opponent"* — the exact construct. `p(2)` (active) casts
/// it, so the controller is the active player and the opponent list is `[p3, p1]` under
/// CR 101.4 against `[p1, p3]` under ascending `PlayerId`: **reversed**, not merely
/// permuted, so the assertion below discriminates in both positions.
fn t1_each_opponent_damage_is_dealt_in_apnap_order() {
    let epicure = corpus_def("Voldaren Epicure");
    let defs = defs_map(std::slice::from_ref(&epicure));
    let mut state = table_active_p2(
        std::slice::from_ref(&epicure),
        vec![card_in(p(2), "Voldaren Epicure", ZoneId::Hand(p(2)), &defs)],
    );
    give_mana(&mut state, p(2), &[(ManaColor::Red, 1)]);
    let life_before: Vec<(PlayerId, i32)> = [p(1), p(2), p(3)]
        .into_iter()
        .map(|pl| (pl, state.players()[&pl].life_total))
        .collect();

    let (state, events) = cast_and_settle(state, "Voldaren Epicure");

    // CR 101.4 — the ORDERED consequence. Ascending PlayerId would give [p1, p3].
    assert_eq!(
        damaged_players_in_order(&events),
        vec![p(3), p(1)],
        "CR 608.2e / CR 101.4: with p2 active AND controlling the source, the opponents \
         are damaged in APNAP order [p3, p1] -- ascending PlayerId would be [p1, p3]"
    );

    // Non-vacuity floor: the damage really happened, and only to the opponents. Without
    // this the ordered assertion above could be satisfied by a list built from an effect
    // that changed nothing.
    let life_after: Vec<(PlayerId, i32)> = [p(1), p(2), p(3)]
        .into_iter()
        .map(|pl| (pl, state.players()[&pl].life_total))
        .collect();
    assert_eq!(
        life_after,
        vec![
            (p(1), life_before[0].1 - 1),
            (p(2), life_before[1].1),
            (p(3), life_before[2].1 - 1),
        ],
        "CR 119.3: each opponent loses exactly 1 life and the controller loses none"
    );
}

// ── Site 2 — `Effect::LivingDeath` ───────────────────────────────────────────

#[test]
/// **Site 2.** CR 608.2e / CR 101.4 — Living Death's three-step mass zone change walks
/// the players in APNAP order, and the walk is observable **twice**: in the order step 1
/// exiles each player's graveyard creatures, and in the order step 3 returns them.
///
/// # Why the order is observable at all
///
/// Step 1 emits one `GameEvent::ObjectExiled { player, .. }` per card in player-walk
/// order, and step 3 emits one `GameEvent::PermanentEnteredBattlefield { player, .. }`
/// per card in the same order — so the ETB triggers of the returning permanents are
/// QUEUED in that order too (CR 603.3). The event log is the game's record of what
/// happened, so "unobservable" is not available as a defence here.
///
/// The card is the real corpus `living_death`, cast for real by the active player from
/// their hand, with one distinctly-named real creature card in each of the three
/// graveyards so the ordered list below discriminates in every position.
fn t2_living_death_exiles_and_returns_in_apnap_order() {
    let living_death = corpus_def("Living Death");
    let elves = corpus_def("Llanowar Elves");
    let birds = corpus_def("Birds of Paradise");
    let mystic = corpus_def("Elvish Mystic");
    let all = vec![
        living_death.clone(),
        elves.clone(),
        birds.clone(),
        mystic.clone(),
    ];
    let defs = defs_map(&all);

    let mut state = table_active_p2(
        &all,
        vec![
            card_in(p(2), "Living Death", ZoneId::Hand(p(2)), &defs),
            // One creature card per graveyard, distinct names so the ordered pairs below
            // name the seat as well as the position.
            card_in(p(1), "Llanowar Elves", ZoneId::Graveyard(p(1)), &defs),
            card_in(p(2), "Birds of Paradise", ZoneId::Graveyard(p(2)), &defs),
            card_in(p(3), "Elvish Mystic", ZoneId::Graveyard(p(3)), &defs),
        ],
    );
    give_mana(
        &mut state,
        p(2),
        &[(ManaColor::Black, 2), (ManaColor::Colorless, 3)],
    );

    let (state, events) = cast_and_settle(state, "Living Death");

    // Step 1's ordered consequence: whose graveyard is emptied first.
    let exiled_order: Vec<PlayerId> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::ObjectExiled { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(
        exiled_order,
        vec![p(2), p(3), p(1)],
        "CR 608.2e / CR 101.4: step 1 exiles each player's graveyard creatures in APNAP \
         order -- ascending PlayerId would be [p1, p2, p3]"
    );

    // Step 3's ordered consequence: the order the permanents enter, which is the order
    // their CR 603.3 ETB triggers are queued.
    assert_eq!(
        entered_battlefield_in_order(&state, &events),
        vec![
            (p(2), "Birds of Paradise".to_string()),
            (p(3), "Elvish Mystic".to_string()),
            (p(1), "Llanowar Elves".to_string()),
        ],
        "CR 608.2e / CR 101.4: step 3 returns each player's cards in APNAP order, under \
         their owners' control -- ascending PlayerId would put Llanowar Elves first"
    );

    // Non-vacuity floor: all three graveyards really are empty of creatures and all
    // three creatures really are on the battlefield. Without this the ordered lists
    // could both be built from a resolution that moved nothing.
    for pl in [p(1), p(2), p(3)] {
        assert_eq!(
            names_in_zone(&state, ZoneId::Graveyard(pl))
                .into_iter()
                .filter(|n| n != "Living Death")
                .collect::<Vec<_>>(),
            Vec::<String>::new(),
            "{pl:?}'s graveyard must hold no creature card after Living Death resolves"
        );
    }
    let mut battlefield = names_in_zone(&state, ZoneId::Battlefield);
    battlefield.sort();
    assert_eq!(
        battlefield,
        vec![
            "Birds of Paradise".to_string(),
            "Elvish Mystic".to_string(),
            "Llanowar Elves".to_string(),
        ],
        "all three creatures are on the battlefield"
    );
}

// ── Site 3 — `Effect::ReturnAllFromGraveyardToBattlefield` ───────────────────

/// A synthetic *"return all permanent cards with different names from ALL graveyards"*
/// sorcery.
///
/// # Why this is synthetic, stated plainly
///
/// The graveyard-owner walk's order survives into observable behaviour **only** when
/// `unique_names` is set, because the arm ends with a global
/// `candidates.sort_by_key(|(id, _)| *id)` that washes the walk order out of the move
/// order entirely (see [`t3b_return_all_without_unique_names_is_ordered_by_object_id`]).
/// With `unique_names` the walk order decides *which* copy of a duplicated name survives
/// the `retain`, which no later sort can undo.
///
/// **No corpus def combines the two**: `eerie_ultimatum` sets `unique_names: true` but
/// reads `PlayerTarget::Controller` (so the walk is `vec![ctx.controller]` and never
/// touches the helper), while `balthor_the_defiled` and `open_the_vaults` walk
/// `PlayerTarget::EachPlayer` with `unique_names: false`. So this probe uses a synthetic
/// `CardDefinition` around the real `Effect`, and the graveyard cards it operates on are
/// real corpus defs.
fn mass_reanimate_unique_names() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx15a-mass-reanimate-unique".into()),
        name: "Mass Reanimate Unique".into(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Return all permanent cards with different names from all graveyards to \
                      the battlefield under their owners' control."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ReturnAllFromGraveyardToBattlefield {
                graveyards: PlayerTarget::EachPlayer,
                filter: TargetFilter::default(),
                tapped: false,
                controller_override: None,
                unique_names: true,
                permanent_cards_only: true,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

#[test]
/// **Site 3.** CR 608.2e / CR 101.4 — the graveyard-owner walk in
/// `Effect::ReturnAllFromGraveyardToBattlefield` is in APNAP order, so when two players'
/// graveyards hold cards with the SAME name and the effect keeps one per name, the
/// survivor is the **active** player's copy, not the lowest-`PlayerId` player's.
///
/// Every seat has a Sol Ring and an Arcane Signet in their graveyard; the objects are
/// created in seat order, so ascending `ObjectId` is ascending `PlayerId`. Under
/// CR 101.4's `[p2, p3, p1]` walk the `retain` keeps p2's pair; under ascending
/// `PlayerId` it would keep p1's. The assertion is the full ordered list of
/// `(controller, name)` pairs for every permanent that entered, plus the full ordered
/// contents of all three graveyards afterwards.
fn t3_return_all_unique_names_keeps_the_apnap_first_copy() {
    let spell = mass_reanimate_unique_names();
    let sol_ring = corpus_def("Sol Ring");
    let signet = corpus_def("Arcane Signet");
    let all = vec![spell.clone(), sol_ring.clone(), signet.clone()];
    let defs = defs_map(&all);

    let mut specs = vec![card_in(
        p(2),
        "Mass Reanimate Unique",
        ZoneId::Hand(p(2)),
        &defs,
    )];
    // Seat order, Sol Ring before Arcane Signet within each seat, so ObjectIds ascend
    // p1 → p2 → p3 and the global `sort_by_key(id)` at the end of the arm cannot itself
    // manufacture the APNAP answer.
    for seat in [p(1), p(2), p(3)] {
        specs.push(card_in(seat, "Sol Ring", ZoneId::Graveyard(seat), &defs));
        specs.push(card_in(
            seat,
            "Arcane Signet",
            ZoneId::Graveyard(seat),
            &defs,
        ));
    }
    let mut state = table_active_p2(&all, specs);
    give_mana(&mut state, p(2), &[(ManaColor::Colorless, 1)]);

    let (state, events) = cast_and_settle(state, "Mass Reanimate Unique");

    assert_eq!(
        entered_battlefield_in_order(&state, &events),
        vec![
            (p(2), "Sol Ring".to_string()),
            (p(2), "Arcane Signet".to_string()),
        ],
        "CR 608.2e / CR 101.4: the graveyard walk is APNAP, so the ACTIVE player's copy \
         of each duplicated name is the one kept -- ascending PlayerId would return \
         p1's pair instead"
    );

    // The other half of the same fact, read off the zones rather than the events: the
    // losing copies are still in their owners' graveyards, and p2's graveyard holds only
    // the spell that just resolved.
    assert_eq!(
        names_in_zone(&state, ZoneId::Graveyard(p(1))),
        vec!["Sol Ring".to_string(), "Arcane Signet".to_string()],
        "p1's copies stay in p1's graveyard under APNAP"
    );
    assert_eq!(
        names_in_zone(&state, ZoneId::Graveyard(p(3))),
        vec!["Sol Ring".to_string(), "Arcane Signet".to_string()],
        "p3's copies stay in p3's graveyard"
    );
    assert_eq!(
        names_in_zone(&state, ZoneId::Graveyard(p(2))),
        vec!["Mass Reanimate Unique".to_string()],
        "CR 608.2m: p2's graveyard holds only the sorcery that just finished resolving"
    );
}

#[test]
/// **UNDISCRIMINATED for the CR 608.2e revert, and that is the finding.**
///
/// This probe drives the REAL corpus `open_the_vaults` — `PlayerTarget::EachPlayer`
/// graveyards, `unique_names: false` — and it is **green under both** the shipped APNAP
/// walk and the reverted ascending-`PlayerId` walk. It is stated here rather than only
/// in a report because a silently-green test is exactly the shape this file exists to
/// remove.
///
/// The reason is structural: the arm collects `(id, owner)` candidates in walk order and
/// then, before it moves anything, runs a global
/// `candidates.sort_by_key(|(id, _)| *id)`. With `unique_names: false` nothing between
/// the walk and that sort depends on order, so the walk order is erased. The walk is only
/// observable through the `retain` that `unique_names` enables — which is what
/// [`t3_return_all_unique_names_keeps_the_apnap_first_copy`] drives.
///
/// So this test pins the *erasure*, wrong-way-round: it asserts the entry order is
/// ascending `ObjectId` irrespective of the seat walk. The day the global sort is removed
/// or moved, this goes red and says that every `EachPlayer` corpus user of this effect
/// (`open_the_vaults`, `balthor_the_defiled`) has just become order-sensitive and needs
/// the CR 101.4 assertion that
/// [`t3_return_all_unique_names_keeps_the_apnap_first_copy`] makes.
fn t3b_return_all_without_unique_names_is_ordered_by_object_id() {
    let vaults = corpus_def("Open the Vaults");
    let sol_ring = corpus_def("Sol Ring");
    let signet = corpus_def("Arcane Signet");
    let all = vec![vaults.clone(), sol_ring.clone(), signet.clone()];
    let defs = defs_map(&all);

    let mut specs = vec![card_in(p(2), "Open the Vaults", ZoneId::Hand(p(2)), &defs)];
    for seat in [p(1), p(2), p(3)] {
        specs.push(card_in(seat, "Sol Ring", ZoneId::Graveyard(seat), &defs));
        specs.push(card_in(
            seat,
            "Arcane Signet",
            ZoneId::Graveyard(seat),
            &defs,
        ));
    }
    let mut state = table_active_p2(&all, specs);
    give_mana(
        &mut state,
        p(2),
        &[(ManaColor::White, 2), (ManaColor::Colorless, 4)],
    );

    let (state, events) = cast_and_settle(state, "Open the Vaults");

    assert_eq!(
        entered_battlefield_in_order(&state, &events),
        vec![
            (p(1), "Sol Ring".to_string()),
            (p(1), "Arcane Signet".to_string()),
            (p(2), "Sol Ring".to_string()),
            (p(2), "Arcane Signet".to_string()),
            (p(3), "Sol Ring".to_string()),
            (p(3), "Arcane Signet".to_string()),
        ],
        "with `unique_names: false` the arm's global `candidates.sort_by_key(id)` erases \
         the graveyard-owner walk order, so the entry order is ascending ObjectId (i.e. \
         creation order) whatever CR 101.4 says. This is the DISCLOSURE, not a claim \
         about APNAP: reverting the walk to ascending PlayerId leaves this test green."
    );
}
