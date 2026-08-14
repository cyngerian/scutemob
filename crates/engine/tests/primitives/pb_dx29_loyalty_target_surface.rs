//! PB-DX29 (engine half) — a loyalty ability can be targeted and can name its X
//! (`OOS-M11-10(loyalty)`).
//!
//! Covers the two read-only queries shipped in `crates/engine/src/rules/queries.rs`
//! (`loyalty_ability_target_requirements`, `loyalty_ability_needs_x`) and the
//! end-to-end `Command::ActivateLoyaltyAbility` path that consumes what they
//! advertise.
//!
//! ## What was wrong
//!
//! `Command::ActivateLoyaltyAbility` has carried `targets` and `x_value` since
//! M11-local S5, and `handle_activate_loyalty_ability` validates the first
//! (`validate_targets_with_source`, CR 601.2c) and spends the second
//! (`x_value.unwrap_or(0)` against `LoyaltyCost::MinusX`, CR 606.4 / CR 107.3m).
//! Nothing above the engine could build a `Command` naming either: `params.rs` sat
//! outside its own parameterization allowlist and hard-coded
//! `targets: Vec::new(), x_value: None`, and no query existed that a client could ask
//! "what does loyalty ability *k* target?".
//!
//! ## The index space is the whole point
//!
//! `ability_target_requirements` indexes `Characteristics::activated_abilities`;
//! a loyalty `ability_index` indexes the REGISTRY def's `AbilityDefinition::
//! LoyaltyAbility` entries, filtered, which is what `handle_activate_loyalty_ability`
//! and `mtg_simulator::legal_actions`' offer loop both use. T3 proves the two spaces
//! are unrelated by execution, and T3B pins the corpus fact that makes the divergence
//! currently unobservable in a single card — so the day a planeswalker with an
//! ordinary activated ability is authored, that test reddens and this file's reasoning
//! has to be revisited rather than silently inherited.
//!
//! CR references used throughout:
//! * CR 606.3 — a loyalty ability is activated at sorcery speed, once per turn.
//! * CR 606.4 — the loyalty cost is paid on activation.
//! * CR 601.2c — targets are chosen (and validated) at announcement.
//! * CR 107.3m — the value of `{X}`/`-X` is chosen by the activating player.
//! * CR 400.7 — an object that changes zones is a new object, so every post-move
//!   verification in this file is BY NAME.

use std::collections::HashMap;

use mtg_engine::{
    ability_target_requirements, all_cards, card_name_to_id, enrich_spec_from_def,
    legal_targets_per_slot, loyalty_ability_needs_x, loyalty_ability_target_requirements,
    process_command, AbilityDefinition, CardDefinition, CardRegistry, CardType, Command,
    CounterType, GameState, GameStateBuilder, GameStateError, LoyaltyCost, ObjectId, ObjectSpec,
    PlayerId, Step, Target, TargetController, TargetFilter, TargetRequirement, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// The `(cost, targets)` pairs of a def's loyalty abilities, in the SAME filtered
/// order `handle_activate_loyalty_ability` and `queries::loyalty_ability_*` use.
/// Enumerated from `all_cards()` (SR-36), never grepped from source.
fn loyalty_abilities(def: &CardDefinition) -> Vec<(LoyaltyCost, Vec<TargetRequirement>)> {
    def.abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::LoyaltyAbility { cost, targets, .. } => {
                Some((cost.clone(), targets.clone()))
            }
            _ => None,
        })
        .collect()
}

fn def_by_name(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("corpus has no card definition named '{name}'"))
}

/// A battlefield ObjectSpec for a real corpus planeswalker, enriched from its own
/// `CardDefinition` (the standing `ObjectSpec::card()`-is-naked gotcha) and given
/// `loyalty` Loyalty counters — `enrich_spec_from_def` sets
/// `characteristics.loyalty`, which is NOT what CR 606.4's payment reads.
fn planeswalker_on_battlefield(
    owner: PlayerId,
    name: &str,
    loyalty: u32,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .with_card_id(card_name_to_id(name))
            .in_zone(ZoneId::Battlefield)
            .with_counter(CounterType::Loyalty, loyalty),
        defs,
    )
}

/// Two players, `p1` active and holding priority in a main phase with an empty stack
/// — the only window CR 606.3 permits a loyalty activation in.
fn main_phase_state(objects: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p(1))
        .at_step(Step::PreCombatMain);
    for spec in objects {
        builder = builder.object(spec);
    }
    builder.build().expect("PB-DX29 fixture must build")
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<mtg_engine::GameEvent>) {
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

fn object_by_name<'a>(state: &'a GameState, name: &str) -> &'a mtg_engine::GameObject {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .unwrap_or_else(|| panic!("no live object named '{name}'"))
}

fn loyalty_of(state: &GameState, name: &str) -> u32 {
    object_by_name(state, name)
        .counters
        .get(&CounterType::Loyalty)
        .copied()
        .unwrap_or(0)
}

// ── T1: the query returns the ability's own declared targets, per index ────────

/// CR 606.3 / CR 601.2c — T1. `loyalty_ability_target_requirements` reports the
/// requirement list of the loyalty ability at `ability_index`, for four real
/// `Complete` corpus planeswalkers, with the EXACT index of each targeted ability
/// pinned. The expected requirement values are written out literally rather than
/// read back out of the same `CardDefinition` the query reads — comparing the query
/// to its own input would be a tautology.
///
/// **Revert to watch red**: change `.nth(ability_index)` in
/// `queries::loyalty_ability_target_requirements` to `.nth(0)`, or drop the
/// `AbilityDefinition::LoyaltyAbility` filter so the raw `def.abilities` index is
/// used. Elspeth (whose loyalty list is offset by a `Replacement` ability at
/// `def.abilities[0]`) discriminates the second revert on its own.
#[test]
fn test_dx29_t1_loyalty_target_requirements_are_the_declared_targets_per_index() {
    let defs = load_defs();
    let p1 = p(1);

    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Elspeth, Storm Slayer", 5, &defs),
        planeswalker_on_battlefield(p1, "Freyalise, Llanowar's Fury", 3, &defs),
        planeswalker_on_battlefield(p1, "Sarkhan Vol", 4, &defs),
        planeswalker_on_battlefield(p1, "Teferi, Time Raveler", 4, &defs),
    ]);

    // ── Elspeth, Storm Slayer — the targeted ability is loyalty index 2 (-3),
    //    though it is `def.abilities[3]`: `abilities[0]` is the token-doubling
    //    `Replacement`. This card alone proves the filter is applied.
    let elspeth = find_object(&state, "Elspeth, Storm Slayer");
    assert_eq!(
        loyalty_ability_target_requirements(&state, elspeth, 2),
        vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
            controller: TargetController::Opponent,
            min_cmc: Some(3),
            ..Default::default()
        })],
        "CR 601.2c: Elspeth's -3 targets a creature an opponent controls with MV >= 3"
    );
    assert!(
        loyalty_ability_target_requirements(&state, elspeth, 0).is_empty(),
        "Elspeth's +1 (loyalty index 0) is untargeted"
    );
    assert!(
        loyalty_ability_target_requirements(&state, elspeth, 1).is_empty(),
        "Elspeth's 0 (loyalty index 1) is untargeted"
    );
    // Non-vacuity: the offset really exists, so index 2 is not accidentally the
    // same thing as `def.abilities[2]`.
    let elspeth_def = def_by_name("Elspeth, Storm Slayer");
    assert!(
        matches!(
            elspeth_def.abilities.first(),
            Some(AbilityDefinition::Replacement { .. })
        ),
        "precondition: Elspeth's def must still open with a non-loyalty ability, \
         otherwise T1 no longer discriminates the filter"
    );
    assert_eq!(
        loyalty_abilities(&elspeth_def).len(),
        3,
        "precondition: Elspeth has exactly three loyalty abilities"
    );

    // ── Freyalise, Llanowar's Fury — -2 at loyalty index 1.
    let freyalise = find_object(&state, "Freyalise, Llanowar's Fury");
    assert_eq!(
        loyalty_ability_target_requirements(&state, freyalise, 1),
        vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
            has_card_types: vec![CardType::Artifact, CardType::Enchantment],
            ..Default::default()
        })],
        "CR 601.2c: Freyalise's -2 destroys target artifact or enchantment"
    );
    assert!(
        loyalty_ability_target_requirements(&state, freyalise, 0).is_empty()
            && loyalty_ability_target_requirements(&state, freyalise, 2).is_empty(),
        "Freyalise's +2 and -6 are untargeted"
    );

    // ── Sarkhan Vol — -2 at loyalty index 1, a bare `TargetCreature`.
    let sarkhan = find_object(&state, "Sarkhan Vol");
    assert_eq!(
        loyalty_ability_target_requirements(&state, sarkhan, 1),
        vec![TargetRequirement::TargetCreature],
        "CR 601.2c: Sarkhan Vol's -2 gains control of target creature"
    );

    // ── Teferi, Time Raveler — -3 at loyalty index 1, an OPTIONAL slot.
    let teferi = find_object(&state, "Teferi, Time Raveler");
    assert_eq!(
        loyalty_ability_target_requirements(&state, teferi, 1),
        vec![TargetRequirement::UpToN {
            count: 1,
            inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_types: vec![
                    CardType::Artifact,
                    CardType::Creature,
                    CardType::Enchantment,
                ],
                ..Default::default()
            })),
        }],
        "CR 601.2c: Teferi's -3 returns UP TO ONE target artifact, creature or enchantment"
    );
    assert_eq!(
        mtg_engine::target_count_range(&loyalty_ability_target_requirements(&state, teferi, 1)),
        (0, 1),
        "CR 115.1b: an `UpToN` slot's minimum is 0 -- the offer layer must not treat \
         Teferi's -3 as mandatory"
    );
}

// ── T2: total, never panics ───────────────────────────────────────────────────

/// CR 606.3 — T2. Every degenerate argument on a LIVE object yields `vec![]` /
/// `false` rather than a panic: an out-of-range `ability_index` (including
/// `usize::MAX`), a non-planeswalker card, and an object with no `card_id` at all.
///
/// The fourth degenerate case the queries' own docs promise — an `ObjectId` that
/// names nothing — is **not** covered here because it does not hold; see
/// `test_dx29_t2b_...`.
///
/// **Revert to watch red**: replace the `let Some(..) else { return vec![] }` chain
/// in either query with `.unwrap()`, or `.nth(ability_index).cloned().unwrap()`.
#[test]
fn test_dx29_t2_degenerate_arguments_on_a_live_object_return_empty_and_never_panic() {
    let defs = load_defs();
    let p1 = p(1);

    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Sarkhan Vol", 4, &defs),
        // A creature with a real card_id, so the "not a planeswalker" case is not
        // secretly the "no card_id" case.
        enrich_spec_from_def(
            ObjectSpec::card(p1, "Grizzly Bears")
                .with_card_id(card_name_to_id("Grizzly Bears"))
                .in_zone(ZoneId::Battlefield),
            &defs,
        ),
        // A naked object with NO card_id at all.
        ObjectSpec::creature(p1, "Nameless Fixture", 1, 1).in_zone(ZoneId::Battlefield),
    ]);

    let sarkhan = find_object(&state, "Sarkhan Vol");
    // Non-vacuity floor: index 1 IS populated, so the out-of-range assertions below
    // are about range and not about the card being empty.
    assert!(
        !loyalty_ability_target_requirements(&state, sarkhan, 1).is_empty(),
        "precondition: Sarkhan Vol's loyalty index 1 must be targeted"
    );
    assert!(
        loyalty_ability_target_requirements(&state, sarkhan, 3).is_empty(),
        "an out-of-range loyalty index yields no requirements"
    );
    assert!(
        loyalty_ability_target_requirements(&state, sarkhan, usize::MAX).is_empty(),
        "usize::MAX must not panic"
    );
    assert!(
        !loyalty_ability_needs_x(&state, sarkhan, 3),
        "an out-of-range loyalty index does not need X"
    );

    let bears = find_object(&state, "Grizzly Bears");
    assert!(
        loyalty_ability_target_requirements(&state, bears, 0).is_empty(),
        "a non-planeswalker card has no loyalty abilities"
    );
    assert!(!loyalty_ability_needs_x(&state, bears, 0));

    let nameless = find_object(&state, "Nameless Fixture");
    assert!(
        loyalty_ability_target_requirements(&state, nameless, 0).is_empty(),
        "an object with no card_id resolves to no requirements"
    );
    assert!(!loyalty_ability_needs_x(&state, nameless, 0));
}

/// CR 400.7 / CR 606.3 — T2B. A stale `ObjectId` yields the documented empty answer in
/// **both** profiles, and this test exists because it did not when it was written.
///
/// # The history, kept because the shape recurs
///
/// PB-DX29's first draft looked the source object up with `GameState::expect_object`,
/// whose contract is *"the caller has already established this id is live; a `None` here
/// is an engine bug"* — it fires a `debug_assert!` and only degrades to `None` in
/// release. So on a CR 400.7-retired id (a planeswalker that died in response) both
/// queries **panicked in a debug build**, while:
///
/// * their own rustdoc promised *"Missing object, missing `card_id`, an unregistered
///   card, or an out-of-range `ability_index` all yield `vec![]` — this function never
///   panics and never unwraps"*;
/// * `queries.rs`'s module doc calls the whole file a read-only **advisory** surface for
///   UI/simulator callers, i.e. callers holding ids a human clicked;
/// * every other lookup in `queries.rs` avoided `expect_object` — the sibling
///   `ability_target_requirements` goes through `calculate_characteristics`, a quiet
///   `None`. `expect_object` appeared in that file at exactly two sites, both new.
///
/// It was never reachable through the three shipped call sites (`view.rs`,
/// `targeting.rs` and `legal_actions.rs` all pass an id enumerated from live state in
/// the same breath), so it was a documentation-vs-behaviour defect on a `pub`,
/// `lib.rs`-re-exported function rather than a live game bug. Fixed by reading
/// `state.objects().get(&source)` directly. **The generalisable half is why an
/// "impossible absence" helper is the wrong tool in a query module at all**: what is
/// impossible for an engine-internal caller is ordinary input for a UI one.
///
/// This test asserts the CONTRACT, in both profiles, so a future `expect_object`
/// creeping back in fails in debug and the release path stays covered too.
#[test]
fn test_dx29_t2b_an_unknown_object_id_yields_the_documented_empty_answer() {
    let defs = load_defs();
    let state = main_phase_state(vec![planeswalker_on_battlefield(
        p(1),
        "Sarkhan Vol",
        4,
        &defs,
    )]);

    // An id that names nothing. `next_object_id` is monotone, so a value above every
    // live id is guaranteed absent.
    let unknown = ObjectId(
        state
            .objects()
            .keys()
            .map(|ObjectId(n)| *n)
            .max()
            .unwrap_or(0)
            + 1_000,
    );
    assert!(
        state.objects().get(&unknown).is_none(),
        "precondition: the probe id must really be absent"
    );
    // Non-vacuity: a LIVE id on the same board returns something, so the empties below
    // are about the missing id and not about the queries having stopped working.
    let sarkhan = find_object(&state, "Sarkhan Vol");
    assert!(
        !loyalty_ability_target_requirements(&state, sarkhan, 1).is_empty(),
        "precondition: a live planeswalker's targeted loyalty index must be non-empty"
    );

    assert!(
        loyalty_ability_target_requirements(&state, unknown, 0).is_empty(),
        "CR 400.7: a retired ObjectId must yield the documented empty requirement list, \
         in every profile. If this PANICS in a debug build, an `expect_object`-style \
         impossible-absence lookup has come back into `queries.rs` -- that helper fires a \
         `debug_assert!` and is the wrong tool in an advisory query surface whose callers \
         hold ids a human clicked."
    );
    assert!(
        !loyalty_ability_needs_x(&state, unknown, 0),
        "CR 400.7: same for `loyalty_ability_needs_x` -- see the message above."
    );
}

// ── T3: the two index spaces are different ────────────────────────────────────

/// CR 606.3 / CR 602.2b — T3, the load-bearing one. `ability_target_requirements` and
/// `loyalty_ability_target_requirements` take the same `(source, ability_index)`
/// argument shape and index unrelated lists.
///
/// **The corpus cannot currently produce a single card that carries BOTH an
/// `AbilityDefinition::Activated` and an `AbilityDefinition::LoyaltyAbility`** — see
/// `test_dx29_t3b_...`, which pins that fact so it reddens the day such a card is
/// authored. So this test does not fake a mixed card; it proves the divergence on a
/// real corpus planeswalker instead, which is the honest form of the claim: at the
/// same index `i = 2` on Elspeth, Storm Slayer, the activated-ability query reports
/// NOTHING (her layer-resolved `activated_abilities` list is empty — she has no
/// activated abilities at all) while the loyalty query reports her -3's real
/// requirement. Same object, same index, different answers.
///
/// **Revert to watch red**: point `targeting.rs`'s / `view.rs`'s loyalty arm at
/// `mtg_engine::ability_target_requirements` — the "obvious reuse" this batch exists
/// to refuse. Directly: make `loyalty_ability_target_requirements` delegate to
/// `ability_target_requirements`; the inequality below then collapses.
#[test]
fn test_dx29_t3_loyalty_index_space_is_not_the_activated_ability_index_space() {
    let defs = load_defs();
    let p1 = p(1);
    let state = main_phase_state(vec![planeswalker_on_battlefield(
        p1,
        "Elspeth, Storm Slayer",
        5,
        &defs,
    )]);
    let elspeth = find_object(&state, "Elspeth, Storm Slayer");

    // The structural fact the divergence rests on: Elspeth's layer-resolved
    // `activated_abilities` list is empty, while her loyalty list has three entries.
    let chars = &state.objects().get(&elspeth).unwrap().characteristics;
    assert!(
        chars.activated_abilities.is_empty(),
        "precondition: Elspeth carries no activated abilities, so the two lists cannot \
         coincidentally agree -- got {:?}",
        chars.activated_abilities
    );
    assert_eq!(
        loyalty_abilities(&def_by_name("Elspeth, Storm Slayer")).len(),
        3,
        "precondition: Elspeth carries three loyalty abilities"
    );

    let activated = ability_target_requirements(&state, elspeth, 2);
    let loyalty = loyalty_ability_target_requirements(&state, elspeth, 2);
    assert_ne!(
        activated, loyalty,
        "CR 606.3: index 2 must mean different things to the two queries; \
         activated={activated:?} loyalty={loyalty:?}"
    );
    // Non-vacuity in both directions -- an inequality between two empty-ish values
    // would prove nothing.
    assert!(
        activated.is_empty(),
        "the activated-ability space is empty at index 2"
    );
    assert_eq!(
        loyalty.len(),
        1,
        "the loyalty space names exactly one requirement at index 2"
    );
}

/// CR 606.3 — T3B, the roster pin behind T3's honesty clause. **No `CardDefinition` in
/// `all_cards()` carries both an `AbilityDefinition::Activated` and an
/// `AbilityDefinition::LoyaltyAbility`.** That is why T3 argues the divergence
/// structurally rather than exhibiting one card whose two queries disagree with both
/// answers non-empty. The day such a card is authored this test reddens, and T3's
/// reasoning must be upgraded to the direct demonstration rather than inherited.
///
/// Both populations carry a non-vacuity floor, so the assertion cannot pass by the
/// corpus having lost either kind of ability.
#[test]
fn test_dx29_t3b_no_corpus_def_carries_both_a_loyalty_and_an_activated_ability() {
    let cards = all_cards();

    let has_loyalty = |d: &CardDefinition| {
        d.abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::LoyaltyAbility { .. }))
    };
    let has_activated = |d: &CardDefinition| {
        d.abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Activated { .. }))
    };

    let loyalty_defs: Vec<&str> = cards
        .iter()
        .filter(|d| has_loyalty(d))
        .map(|d| d.name.as_str())
        .collect();
    let activated_defs = cards.iter().filter(|d| has_activated(d)).count();
    let both: Vec<&str> = cards
        .iter()
        .filter(|d| has_loyalty(d) && has_activated(d))
        .map(|d| d.name.as_str())
        .collect();

    // Non-vacuity floors on BOTH populations.
    assert!(
        loyalty_defs.len() >= 30,
        "non-vacuity: expected >= 30 defs with a loyalty ability, got {} ({:?})",
        loyalty_defs.len(),
        loyalty_defs
    );
    assert!(
        activated_defs >= 200,
        "non-vacuity: expected >= 200 defs with an activated ability, got {activated_defs}"
    );

    assert!(
        both.is_empty(),
        "PB-DX29 T3's structural argument assumed no card carries both kinds of \
         ability; {:?} now does. Upgrade T3 to a direct two-non-empty-answers \
         demonstration on that card and re-read \
         `queries::loyalty_ability_target_requirements`' doc.",
        both
    );
}

// ── T4: `needs_x` is exactly `LoyaltyCost::MinusX` ────────────────────────────

/// CR 606.4 / CR 107.3m — T4. `loyalty_ability_needs_x` is `true` for
/// `LoyaltyCost::MinusX` and `false` for `Plus`, `Minus` and `Zero`, checked per index
/// on two real corpus cards (Chandra, Flamecaller, `Complete`, and Ugin, the Spirit
/// Dragon, `partial`) — Chandra alone exhibits all four cost shapes across her three
/// abilities except `Minus`, which Ugin supplies at index 2.
///
/// **Revert to watch red**: change the `matches!(cost, LoyaltyCost::MinusX)` in
/// `queries::loyalty_ability_needs_x` to `!matches!(cost, LoyaltyCost::Zero)`, or make
/// it `is_some()` (i.e. "any loyalty ability needs X").
#[test]
fn test_dx29_t4_needs_x_is_true_exactly_for_minus_x() {
    let defs = load_defs();
    let p1 = p(1);
    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Chandra, Flamecaller", 4, &defs),
        planeswalker_on_battlefield(p1, "Ugin, the Spirit Dragon", 7, &defs),
    ]);

    let chandra = find_object(&state, "Chandra, Flamecaller");
    let chandra_costs: Vec<LoyaltyCost> = loyalty_abilities(&def_by_name("Chandra, Flamecaller"))
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    assert_eq!(
        chandra_costs,
        vec![LoyaltyCost::Plus(1), LoyaltyCost::Zero, LoyaltyCost::MinusX],
        "precondition: Chandra's loyalty costs, in filtered order"
    );
    assert!(
        !loyalty_ability_needs_x(&state, chandra, 0),
        "CR 606.4: `+1` names no X"
    );
    assert!(
        !loyalty_ability_needs_x(&state, chandra, 1),
        "CR 606.4: `0` names no X"
    );
    assert!(
        loyalty_ability_needs_x(&state, chandra, 2),
        "CR 107.3m: `-X` is the activating player's choice"
    );

    let ugin = find_object(&state, "Ugin, the Spirit Dragon");
    let ugin_costs: Vec<LoyaltyCost> = loyalty_abilities(&def_by_name("Ugin, the Spirit Dragon"))
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    assert_eq!(
        ugin_costs,
        vec![
            LoyaltyCost::Plus(2),
            LoyaltyCost::MinusX,
            LoyaltyCost::Minus(10)
        ],
        "precondition: Ugin's loyalty costs, in filtered order"
    );
    assert!(!loyalty_ability_needs_x(&state, ugin, 0));
    assert!(loyalty_ability_needs_x(&state, ugin, 1));
    assert!(
        !loyalty_ability_needs_x(&state, ugin, 2),
        "CR 606.4: a FIXED `-10` is not an X cost -- this is the arm a \
         `!matches!(cost, Zero)` mis-implementation gets wrong"
    );
}

/// CR 606.4 / CR 107.3m — T4B. The exact corpus roster of `LoyaltyCost::MinusX`
/// carriers, enumerated from `all_cards()` (SR-36), with a non-vacuity floor on the
/// enclosing loyalty population so the assertion cannot pass by the corpus losing
/// planeswalkers wholesale.
#[test]
fn test_dx29_t4b_minus_x_roster_over_all_cards() {
    let cards = all_cards();

    let mut minus_x: Vec<&str> = cards
        .iter()
        .filter(|d| {
            loyalty_abilities(d)
                .iter()
                .any(|(cost, _)| matches!(cost, LoyaltyCost::MinusX))
        })
        .map(|d| d.name.as_str())
        .collect();
    minus_x.sort_unstable();

    let loyalty_population = cards
        .iter()
        .filter(|d| !loyalty_abilities(d).is_empty())
        .count();
    assert!(
        loyalty_population >= 30,
        "non-vacuity: expected >= 30 defs with any loyalty ability, got {loyalty_population}"
    );

    assert_eq!(
        minus_x,
        vec!["Chandra, Flamecaller", "Ugin, the Spirit Dragon"],
        "the `LoyaltyCost::MinusX` roster moved. A new member is a card whose X now \
         reaches the engine through `params.rs`; re-read PB-DX29 T7 before re-pinning."
    );
}

// ── T5: end to end, with a NON-DEFAULT target ─────────────────────────────────

/// CR 606.3 / CR 606.4 / CR 601.2c / CR 400.7 — T5, the headline. A real Sarkhan Vol
/// activates its `-2` ("Gain control of target creature until end of turn. Untap that
/// creature.") naming the **second** candidate `legal_targets_per_slot` enumerates,
/// not the first.
///
/// "Second" is provably not a default: `legal_targets_per_slot` enumerates
/// deterministically (ascending `ObjectId`) and `targeting::plan_targets` — the only
/// automatic chooser in the tree — takes `candidates.first()`. The test asserts the
/// chosen candidate is NOT `candidates[0]` before submitting it, so an engine that
/// ignored the announced list and picked its own would fail.
///
/// Verification is BY NAME (CR 400.7), never by the pre-activation `ObjectId`.
///
/// **Revert to watch red**: in `handle_activate_loyalty_ability`, replace the
/// `spell_targets` construction with `Vec::new()`, or push
/// `StackObject { targets: vec![], .. }`. `Effect::GainControl`'s
/// `DeclaredTarget { index: 0 }` then resolves to nothing and neither creature
/// changes hands.
#[test]
fn test_dx29_t5_activation_honours_a_non_default_declared_target() {
    let defs = load_defs();
    let (p1, p2) = (p(1), p(2));

    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Sarkhan Vol", 4, &defs),
        ObjectSpec::creature(p2, "Opp Alpha", 2, 2)
            .in_zone(ZoneId::Battlefield)
            .tapped(),
        ObjectSpec::creature(p2, "Opp Beta", 3, 3)
            .in_zone(ZoneId::Battlefield)
            .tapped(),
    ]);

    let sarkhan = find_object(&state, "Sarkhan Vol");
    let reqs = loyalty_ability_target_requirements(&state, sarkhan, 1);
    assert_eq!(
        reqs,
        vec![TargetRequirement::TargetCreature],
        "precondition: Sarkhan Vol's -2 must still be a single mandatory TargetCreature"
    );

    let per_slot = legal_targets_per_slot(&state, p1, sarkhan, &reqs);
    assert_eq!(per_slot.len(), 1, "one slot, parallel to one requirement");
    let candidates = &per_slot[0];
    assert!(
        candidates.len() >= 2,
        "non-vacuity: the board must offer at least two candidates so that 'the \
         second' is meaningfully non-default; got {candidates:?}"
    );
    let chosen = candidates[1].clone();
    assert_ne!(
        chosen, candidates[0],
        "the chosen candidate must differ from the one an automatic chooser takes"
    );

    // Which NAME did we choose? Recorded now, while the id is still live.
    let chosen_id = match chosen {
        Target::Object(id) => id,
        other => panic!("expected an object candidate, got {other:?}"),
    };
    let chosen_name = state
        .objects()
        .get(&chosen_id)
        .expect("chosen candidate is live")
        .characteristics
        .name
        .clone();
    let other_name = match candidates[0] {
        Target::Object(id) => state
            .objects()
            .get(&id)
            .expect("first candidate is live")
            .characteristics
            .name
            .clone(),
        ref other => panic!("expected an object candidate, got {other:?}"),
    };
    assert_ne!(chosen_name, other_name, "the two names must be distinct");

    let (state, _events) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sarkhan,
            ability_index: 1,
            targets: vec![chosen.clone()],
            x_value: None,
        },
    )
    .expect("CR 601.2c: a legal declared target must be accepted");

    // CR 606.4: the -2 was paid at activation, before resolution.
    assert_eq!(
        loyalty_of(&state, "Sarkhan Vol"),
        2,
        "CR 606.4: 4 - 2 = 2 loyalty counters remain"
    );

    let (state, _events) = pass_all(state, &[p1, p2]);
    assert!(
        state.stack_objects().is_empty(),
        "the loyalty ability should have resolved"
    );

    // CR 400.7: verify BY NAME.
    let chosen_after = object_by_name(&state, &chosen_name);
    let other_after = object_by_name(&state, &other_name);
    assert_eq!(
        chosen_after.controller, p1,
        "CR 613.1b: '{chosen_name}' -- the ANNOUNCED target -- must have changed \
         controller to the activating player"
    );
    assert!(
        !chosen_after.status.tapped,
        "'{chosen_name}' must have been untapped by the ability"
    );
    assert_eq!(
        other_after.controller, p2,
        "'{other_name}' was not targeted and must NOT have changed controller"
    );
    assert!(
        other_after.status.tapped,
        "'{other_name}' was not targeted and must remain tapped"
    );
}

// ── T6: the pre-batch refusal is still a refusal ──────────────────────────────

/// CR 601.2c — T6. The same activation with `targets: vec![]` on a MANDATORY slot is
/// still refused with `GameStateError::InvalidTarget`, and the refusal names the
/// count range. Pinned so this batch's opening of the channel is not mistaken for a
/// loosening of CR 601.2c, and so the SR-38 offer suppression in
/// `mtg_simulator::legal_actions` is provably suppressing something real.
///
/// **Revert to watch red**: delete the `if !ability_targets.is_empty()` /
/// `validate_targets_with_source` block from `handle_activate_loyalty_ability`.
#[test]
fn test_dx29_t6_mandatory_target_ability_with_no_targets_is_refused() {
    let defs = load_defs();
    let (p1, p2) = (p(1), p(2));
    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Sarkhan Vol", 4, &defs),
        ObjectSpec::creature(p2, "Opp Alpha", 2, 2).in_zone(ZoneId::Battlefield),
    ]);
    let sarkhan = find_object(&state, "Sarkhan Vol");
    let before = loyalty_of(&state, "Sarkhan Vol");

    let result = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sarkhan,
            ability_index: 1,
            targets: vec![],
            x_value: None,
        },
    );

    match result {
        Err(GameStateError::InvalidTarget(msg)) => {
            assert!(
                msg.contains("expected 1..=1 target(s) but got 0"),
                "CR 601.2c: the refusal should name the count range, got {msg:?}"
            );
        }
        other => panic!(
            "CR 601.2c: a mandatory target slot with no announced target must be \
             Err(InvalidTarget), got {:?}",
            other.map(|(_, ev)| ev.len())
        ),
    }
    assert_eq!(
        before, 4,
        "precondition: the fixture really did start at 4 loyalty"
    );
}

// ── T7: `x_value` end to end ──────────────────────────────────────────────────

/// CR 606.4 / CR 107.3m — T7, the live defect on a deck-legal `Complete` def.
/// Chandra, Flamecaller's `-X` ("Chandra deals X damage to each creature") with
/// `x_value: Some(3)` spends exactly three loyalty counters and deals exactly three
/// damage to each creature.
///
/// **Revert to watch red**: in `handle_activate_loyalty_ability`, hard-code
/// `LoyaltyCost::MinusX => 0` for the effective cost (the loyalty half), or drop
/// `ctx.x_value = stack_obj.x_value` from `resolution.rs`'s `LoyaltyAbility` arm (the
/// damage half). The `params.rs` half is covered in
/// `crates/simulator/tests/pb_dx29_loyalty_channel.rs` S2.
#[test]
fn test_dx29_t7_x_value_is_spent_and_dealt_end_to_end() {
    let defs = load_defs();
    let (p1, p2) = (p(1), p(2));
    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Chandra, Flamecaller", 4, &defs),
        ObjectSpec::creature(p1, "Own Wall", 0, 6).in_zone(ZoneId::Battlefield),
        ObjectSpec::creature(p2, "Opp Wall", 0, 6).in_zone(ZoneId::Battlefield),
    ]);
    let chandra = find_object(&state, "Chandra, Flamecaller");
    assert!(
        loyalty_ability_needs_x(&state, chandra, 2),
        "precondition: loyalty index 2 is Chandra's -X"
    );

    let (state, _events) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: chandra,
            ability_index: 2,
            targets: vec![],
            x_value: Some(3),
        },
    )
    .expect("CR 107.3m: X = 3 with 4 loyalty available must be accepted");

    assert_eq!(
        loyalty_of(&state, "Chandra, Flamecaller"),
        1,
        "CR 606.4: -X with X = 3 spends three loyalty counters (4 - 3 = 1)"
    );

    let (state, _events) = pass_all(state, &[p1, p2]);
    assert!(state.stack_objects().is_empty(), "the -X resolved");

    // CR 400.7: by name. Both walls are toughness 6, so they survive the 3 damage
    // and can be inspected rather than merely missed.
    assert_eq!(
        object_by_name(&state, "Own Wall").damage_marked,
        3,
        "CR 107.3m: X = 3 must deal 3 damage to each creature"
    );
    assert_eq!(
        object_by_name(&state, "Opp Wall").damage_marked,
        3,
        "CR 107.3m: 'each creature' includes the opponent's"
    );
}

/// CR 606.4 / CR 107.3m — T7B, the other half of the same claim. `x_value: None` is
/// X = 0: nothing is spent and nothing is dealt. This is the pre-batch behaviour of
/// EVERY `-X` activation (`params.rs` hard-coded `x_value: None`), pinned so T7's
/// `Some(3)` is provably about the announced value rather than about the ability
/// having any effect at all.
#[test]
fn test_dx29_t7b_absent_x_value_spends_nothing_and_deals_nothing() {
    let defs = load_defs();
    let (p1, p2) = (p(1), p(2));
    let state = main_phase_state(vec![
        planeswalker_on_battlefield(p1, "Chandra, Flamecaller", 4, &defs),
        ObjectSpec::creature(p1, "Own Wall", 0, 6).in_zone(ZoneId::Battlefield),
        ObjectSpec::creature(p2, "Opp Wall", 0, 6).in_zone(ZoneId::Battlefield),
    ]);
    let chandra = find_object(&state, "Chandra, Flamecaller");

    let (state, _events) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: chandra,
            ability_index: 2,
            targets: vec![],
            x_value: None,
        },
    )
    .expect("CR 107.3m: X = 0 is a legal choice");

    assert_eq!(
        loyalty_of(&state, "Chandra, Flamecaller"),
        4,
        "CR 606.4: an absent X is X = 0 and spends no loyalty"
    );

    let (state, _events) = pass_all(state, &[p1, p2]);
    assert!(state.stack_objects().is_empty(), "the -X resolved");
    assert_eq!(
        object_by_name(&state, "Own Wall").damage_marked,
        0,
        "CR 107.3m: X = 0 deals no damage"
    );
    assert_eq!(object_by_name(&state, "Opp Wall").damage_marked, 0);
}
