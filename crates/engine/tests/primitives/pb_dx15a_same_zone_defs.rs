//! PB-DX15a (CR 400.7): per-def behavioural probes for the five named `Complete`, deck-legal
//! card definitions whose printed effect routes cards back into the library they are already in
//! (`pb_dx15a_same_zone_identity_roster.rs` family A), plus one probe for family D's blast
//! radius.
//!
//! Each probe asserts the three things a same-zone move must be true of:
//!
//! 1. **Identity** — every card that stayed in the library keeps its ORIGINAL `ObjectId`
//!    (CR 400.7: "if an object moves **from one zone to another** … it becomes a new object";
//!    a card routed to the zone it is already in never satisfies the antecedent).
//! 2. **Counter neutrality** — `state.current_timestamp()` moves by EXACTLY the number of values
//!    the effect legitimately consumes, and the derivation is spelled out in each test rather
//!    than pinned as a magic number. `timestamp_counter` is `next_object_id`'s counter AND the
//!    seed source for every `Zone::shuffle` and coin flip, so a renumbering reorder silently
//!    perturbed future randomness by however many cards a def happened to route.
//! 3. **Order** — the routed cards end up where the card prints (top or bottom), so nothing here
//!    passes because the helpers became no-ops.
//!
//! ## Fixture rule (inherited from `crates/simulator/tests/pb_dx43_intrinsic_mana_channel.rs`)
//!
//! The subject card is ALWAYS built from its shipped `CardDefinition` via
//! `enrich_spec_from_def`, and the effect executed is ALWAYS lifted out of that same shipped
//! def — never a hand-written `Effect` literal that approximates it. `ObjectSpec::card()`
//! creates a naked object, and an approximated effect literal is a fixture that can agree with
//! a test while disagreeing with the card a player actually casts. The library FILLER cards are
//! synthetic on purpose (they only need a type line and a name), and each probe says which is
//! which.
//!
//! Effects are driven through `execute_effect` on a real `EffectContext`, the idiom
//! `pb_os8_look_at_top_then_place.rs` established, rather than through a cast: three of the five
//! are ETB triggers whose cast path adds several unrelated `timestamp_counter` consumers
//! (stack-object ids, trigger ids, SBA passes) and would make claim (2) unfalsifiable.
//! The one probe that DOES need the real resolution path — family D's Partner With — drives it
//! end to end, because that arm lives in `rules/resolution.rs` and has no `Effect` node at all.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::targeting::{SpellTarget, Target};
use mtg_engine::state::test_util;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, Effect, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, StackObject, StackObjectKind, Step, SubType, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn def_of(name: &str) -> CardDefinition {
    defs_map()
        .get(name)
        .unwrap_or_else(|| panic!("no shipped CardDefinition named '{name}'"))
        .clone()
}

/// Build the SUBJECT card from its own shipped definition (types, P/T, abilities and all),
/// never from a hand-written approximation.
fn real_card(owner: PlayerId, name: &str, zone: ZoneId) -> ObjectSpec {
    let defs = defs_map();
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no shipped CardDefinition named '{name}'"));
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(def.card_id.clone()),
        &defs,
    )
}

/// A synthetic library filler. Only its name and type line matter to the probes.
fn filler(owner: PlayerId, name: &str, ty: CardType, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .with_card_id(CardId(format!(
            "dx15a-{}",
            name.to_lowercase().replace(' ', "-")
        )))
        .with_types(vec![ty])
        .in_zone(zone)
}

fn filler_with_subtype(
    owner: PlayerId,
    name: &str,
    ty: CardType,
    sub: &str,
    zone: ZoneId,
) -> ObjectSpec {
    filler(owner, name, ty, zone).with_subtypes(vec![SubType(sub.to_string())])
}

/// The library's contents, **bottom-to-top** (`Zone::object_ids()` on an ordered zone walks the
/// backing `Vector` in storage order, and the top is the LAST index — `Zone::top()` is
/// `v.last()`).
fn lib_ids(state: &GameState, owner: PlayerId) -> Vec<ObjectId> {
    state
        .zone(&ZoneId::Library(owner))
        .expect("library exists")
        .object_ids()
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| format!("<dead {id:?}>"))
}

fn lib_names(state: &GameState, owner: PlayerId) -> Vec<String> {
    lib_ids(state, owner)
        .into_iter()
        .map(|id| name_of(state, id))
        .collect()
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Hand(owner))
}

/// Every top-level `Effect` declared by `def`'s abilities (front face only — none of the five
/// subjects declares a member effect on a back face; `growing_rites_of_itlimoc`'s back face is
/// a land with two mana abilities).
fn ability_effects(def: &CardDefinition) -> Vec<Effect> {
    def.abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Triggered { effect, .. }
            | AbilityDefinition::Spell { effect, .. }
            | AbilityDefinition::Activated { effect, .. } => Some(effect.clone()),
            _ => None,
        })
        .collect()
}

/// The one `Effect` in `def` (searching inside `Effect::Sequence`) for which `pred` holds.
/// Panics if there is not exactly one — a def growing a second member effect must be noticed,
/// not silently half-tested.
fn sole_effect_matching(def: &CardDefinition, pred: impl Fn(&Effect) -> bool + Copy) -> Effect {
    fn collect(e: &Effect, out: &mut Vec<Effect>) {
        out.push(e.clone());
        if let Effect::Sequence(inner) = e {
            for c in inner {
                collect(c, out);
            }
        }
    }
    let mut all = Vec::new();
    for e in ability_effects(def) {
        collect(&e, &mut all);
    }
    let mut hits: Vec<Effect> = all.into_iter().filter(|e| pred(e)).collect();
    assert_eq!(
        hits.len(),
        1,
        "'{}' should declare exactly one effect matching the probe's predicate, found {}",
        def.name,
        hits.len()
    );
    hits.remove(0)
}

fn is_reveal_and_route(e: &Effect) -> bool {
    matches!(e, Effect::RevealAndRoute { .. })
}

fn is_look_at_top_then_place(e: &Effect) -> bool {
    matches!(e, Effect::LookAtTopThenPlace { .. })
}

/// Asserts that every id in `before` that is still in the library is the SAME object it was —
/// i.e. that the routed cards were repositioned, not retired and re-minted (CR 400.7).
///
/// Deliberately checks by SET of ids rather than by name: a re-minted card has the same name and
/// the same zone, so a name-based check passes under the exact defect this batch removed. That
/// is the trap `pb_os8_look_at_top_then_place.rs`' out-of-window probe originally leaned on and
/// which this batch's fix has now taken away from it (see the note in that file).
fn assert_library_ids_survived(
    state: &GameState,
    owner: PlayerId,
    before: &[ObjectId],
    expected_survivors: &[&str],
) {
    let after: Vec<ObjectId> = lib_ids(state, owner);
    let expected: std::collections::BTreeSet<String> =
        expected_survivors.iter().map(|s| s.to_string()).collect();
    let got: std::collections::BTreeSet<String> =
        after.iter().map(|&id| name_of(state, id)).collect();
    assert_eq!(
        got, expected,
        "wrong cards survived in the library (by name): {got:?}"
    );
    for &id in &after {
        assert!(
            before.contains(&id),
            "CR 400.7: {id:?} ('{}') is in the library but was NOT one of the ids that started \
             there — a card that never left its zone was retired and re-minted as a new object. \
             before={before:?} after={after:?}",
            name_of(state, id)
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════
// Family A — Effect::RevealAndRoute, unmatched_dest = Library
// ═════════════════════════════════════════════════════════════════════════════════════════════

/// CR 400.7 / CR 701.20a — **Goblin Ringleader**: "reveal the top four cards of your library.
/// Put all Goblin cards revealed this way into your hand and the rest on the bottom of your
/// library in any order."
///
/// Four non-Goblins on top: nothing is put into hand, all four are routed to the bottom of the
/// library they are already in. Expected `timestamp_counter` delta: **0** — four same-zone
/// repositions and no other draw. Under the pre-PB-DX15a behaviour this consumed **4** and
/// renumbered all four cards.
/// **Reverts watched red**: V2 (disable the `from == to` guard in `move_object_to_bottom_of_zone`), V3 (swap the two `ZoneEnd` arms), V4 (make `reposition_within_own_zone` stop repositioning).
#[test]
fn t_goblin_ringleader_bottoming_four_non_goblins_mints_nothing() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        // Push order is bottom-to-top: Decoy is the library's bottom card and is never in the
        // examined top-4 window.
        .object(filler(p1, "Decoy Below", CardType::Land, lib))
        .object(filler(p1, "Plain A", CardType::Land, lib))
        .object(filler(p1, "Plain B", CardType::Land, lib))
        .object(filler(p1, "Plain C", CardType::Instant, lib))
        .object(filler(p1, "Plain D", CardType::Sorcery, lib))
        .object(real_card(p1, "Goblin Ringleader", ZoneId::Battlefield))
        .build()
        .unwrap();

    let source = id_of(&state, "Goblin Ringleader");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let effect = sole_effect_matching(&def_of("Goblin Ringleader"), is_reveal_and_route);
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _ = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.current_timestamp(),
        ts_before,
        "CR 400.7: four cards routed back into the library they were already in must consume \
         ZERO timestamp_counter values (that counter seeds every shuffle and coin flip). \
         library now: {:?}",
        lib_names(&state, p1)
    );
    assert_library_ids_survived(
        &state,
        p1,
        &before,
        &["Decoy Below", "Plain A", "Plain B", "Plain C", "Plain D"],
    );
    // Order: the four examined cards were bottomed in ObjectId-ascending order (the engine's
    // deterministic stand-in for "in any order"), each `push_front`ed to index 0, so the last
    // one bottomed sits at index 0. `Decoy Below` — never examined — floats to the top.
    assert_eq!(
        lib_names(&state, p1).last().map(String::as_str),
        Some("Decoy Below"),
        "CR 121.1: the never-examined bottom card must end up on TOP once the four examined \
         cards are correctly placed beneath it. library: {:?}",
        lib_names(&state, p1)
    );
}

/// CR 400.7 / CR 701.20a — **Goblin Ringleader**, the matching half: one Goblin among the top
/// four goes to hand (a real zone change, CR 400.7's antecedent satisfied: one id minted), the
/// other three are bottomed in place (zero minted). Expected delta: **exactly 1**.
///
/// This is the row that proves claim (2) is a real arithmetic and not "assert zero" — the fix
/// must leave genuine zone changes alone.
/// **Reverts watched red**: V2, V4.
#[test]
fn t_goblin_ringleader_mints_only_for_the_card_that_actually_changes_zones() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(filler(p1, "Plain A", CardType::Land, lib))
        .object(filler(p1, "Plain B", CardType::Land, lib))
        .object(filler(p1, "Plain C", CardType::Instant, lib))
        .object(filler_with_subtype(
            p1,
            "Goblin Recruit",
            CardType::Creature,
            "Goblin",
            lib,
        ))
        .object(real_card(p1, "Goblin Ringleader", ZoneId::Battlefield))
        .build()
        .unwrap();

    let source = id_of(&state, "Goblin Ringleader");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let effect = sole_effect_matching(&def_of("Goblin Ringleader"), is_reveal_and_route);
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _ = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "Goblin Recruit", p1),
        "the Goblin must be put into hand"
    );
    assert_eq!(
        state.current_timestamp(),
        ts_before + 1,
        "exactly ONE id is minted: the Goblin genuinely moved library→hand (CR 400.7). The \
         three bottomed cards never left their zone and must mint nothing. library: {:?}",
        lib_names(&state, p1)
    );
    assert_library_ids_survived(&state, p1, &before, &["Plain A", "Plain B", "Plain C"]);
}

/// CR 400.7 / CR 701.20a — **Sylvan Messenger**: the Elf-tribal twin of Goblin Ringleader, same
/// `Effect::RevealAndRoute` shape, probed independently rather than assumed equivalent (the two
/// defs are separate files and only one of them is `Complete` by an explicit marker).
/// **Reverts watched red**: V2, V3, V4, and V8 (the card def leaving family A).
#[test]
fn t_sylvan_messenger_bottoming_four_non_elves_mints_nothing() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(filler(p1, "Decoy Below", CardType::Land, lib))
        .object(filler(p1, "Plain A", CardType::Land, lib))
        .object(filler(p1, "Plain B", CardType::Land, lib))
        .object(filler(p1, "Plain C", CardType::Instant, lib))
        .object(filler(p1, "Plain D", CardType::Sorcery, lib))
        .object(real_card(p1, "Sylvan Messenger", ZoneId::Battlefield))
        .build()
        .unwrap();

    let source = id_of(&state, "Sylvan Messenger");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let effect = sole_effect_matching(&def_of("Sylvan Messenger"), is_reveal_and_route);
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _ = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.current_timestamp(),
        ts_before,
        "CR 400.7: bottoming four cards already in the library must mint nothing"
    );
    assert_library_ids_survived(
        &state,
        p1,
        &before,
        &["Decoy Below", "Plain A", "Plain B", "Plain C", "Plain D"],
    );
    assert_eq!(
        lib_names(&state, p1).last().map(String::as_str),
        Some("Decoy Below"),
        "CR 121.1: the never-examined card must float to the top. library: {:?}",
        lib_names(&state, p1)
    );
}

// ═════════════════════════════════════════════════════════════════════════════════════════════
// Family A — Effect::LookAtTopThenPlace, rest_to = Library
// ═════════════════════════════════════════════════════════════════════════════════════════════

/// CR 400.7 / CR 120 — **Growing Rites of Itlimoc**: "look at the top four cards of your
/// library. You may reveal a creature card from among them and put it into your hand. Put the
/// rest on the bottom of your library in any order."
///
/// No creature in the window: nothing is placed and all four are bottomed in place. Expected
/// delta **0** (was 4).
/// **Reverts watched red**: V2, V3, V4.
#[test]
fn t_growing_rites_bottoming_the_whole_window_mints_nothing() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(filler(p1, "Decoy Below", CardType::Land, lib))
        .object(filler(p1, "Plain A", CardType::Land, lib))
        .object(filler(p1, "Plain B", CardType::Land, lib))
        .object(filler(p1, "Plain C", CardType::Instant, lib))
        .object(filler(p1, "Plain D", CardType::Sorcery, lib))
        .object(real_card(
            p1,
            "Growing Rites of Itlimoc",
            ZoneId::Battlefield,
        ))
        .build()
        .unwrap();

    let source = id_of(&state, "Growing Rites of Itlimoc");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let effect = sole_effect_matching(
        &def_of("Growing Rites of Itlimoc"),
        is_look_at_top_then_place,
    );
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _ = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.current_timestamp(),
        ts_before,
        "CR 400.7: the four looked-at cards return to the library they never left"
    );
    assert_library_ids_survived(
        &state,
        p1,
        &before,
        &["Decoy Below", "Plain A", "Plain B", "Plain C", "Plain D"],
    );
    assert_eq!(
        lib_names(&state, p1).last().map(String::as_str),
        Some("Decoy Below"),
        "CR 121.1: the never-examined card must float to the top. library: {:?}",
        lib_names(&state, p1)
    );
}

/// CR 400.7 / CR 118.12 — **Birthing Ritual**: "look at the top seven cards of your library.
/// Then you may sacrifice a creature. If you do, you may put a creature card … Put the rest on
/// the bottom of your library in a random order."
///
/// With **no creature on the battlefield to sacrifice**, the interposed `Cost::Sacrifice` cannot
/// be paid, so nothing is placed and all seven go to the bottom of the library they are already
/// in. Expected delta **0** (was 7) — the largest single-effect blast radius in family A, and
/// the reason the seed calls this class a randomness perturbation rather than a cosmetic one.
///
/// The unpayable-cost setup is deliberate: paying the sacrifice would move a permanent
/// battlefield→graveyard, which IS a zone change and legitimately mints an id, muddying the
/// arithmetic this probe exists to pin.
/// **Reverts watched red**: V2, V3, V4.
#[test]
fn t_birthing_ritual_bottoming_seven_mints_nothing() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(filler(p1, "Decoy Below", CardType::Land, lib));
    for i in 0..7 {
        builder = builder.object(filler(
            p1,
            &format!("Window Card {i}"),
            CardType::Instant,
            lib,
        ));
    }
    // The enchantment itself is not a creature, so there is nothing to sacrifice.
    let mut state = builder
        .object(real_card(p1, "Birthing Ritual", ZoneId::Battlefield))
        .build()
        .unwrap();

    let source = id_of(&state, "Birthing Ritual");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let effect = sole_effect_matching(&def_of("Birthing Ritual"), is_look_at_top_then_place);
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _ = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.current_timestamp(),
        ts_before,
        "CR 400.7: seven cards routed to the bottom of their own library must mint nothing. \
         Before PB-DX15a this one trigger advanced the shuffle/coin-flip seed source by 7. \
         library: {:?}",
        lib_names(&state, p1)
    );
    let mut expected: Vec<String> = (0..7).map(|i| format!("Window Card {i}")).collect();
    expected.push("Decoy Below".to_string());
    let expected_refs: Vec<&str> = expected.iter().map(String::as_str).collect();
    assert_library_ids_survived(&state, p1, &before, &expected_refs);
    assert_eq!(
        lib_names(&state, p1).last().map(String::as_str),
        Some("Decoy Below"),
        "CR 121.1: the never-examined card must float to the top. library: {:?}",
        lib_names(&state, p1)
    );
}

// ═════════════════════════════════════════════════════════════════════════════════════════════
// Family A — Chaos Warp: the `LibraryPosition::Top` member
// ═════════════════════════════════════════════════════════════════════════════════════════════

/// CR 400.7 / CR 701.20a — **Chaos Warp**'s reveal arm in isolation: "…then reveals the top card
/// of their library. If it's a permanent card, they put it onto the battlefield."
///
/// A NON-permanent on top is routed to `unmatched_dest: Library { position: Top }` — i.e. put
/// back exactly where it already was. This is the member that makes family A's roster row refuse
/// to scope itself to `LibraryPosition::Bottom`. Expected delta **0**; the card must keep its id
/// and must still be the top card.
///
/// The arm is lifted out of the shipped def's `Effect::Sequence` rather than hand-written; the
/// full sequence is probed separately below, because its `Effect::Shuffle` and its
/// battlefield-bound `Effect::MoveZone` each legitimately consume a counter value and would hide
/// this arm's zero.
/// **Reverts watched red**: V1 (disable the `from == to` guard in `move_object_to_zone`), V3.
#[test]
fn t_chaos_warp_non_permanent_stays_on_top_under_its_own_id() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(filler(p1, "Below One", CardType::Land, lib))
        .object(filler(p1, "Below Two", CardType::Land, lib))
        // Top card: an instant, which is not a permanent card (CR 110.4a).
        .object(filler(p1, "Top Instant", CardType::Instant, lib))
        .object(real_card(p1, "Chaos Warp", ZoneId::Hand(p1)))
        .build()
        .unwrap();

    // The reveal arm resolves its player through `PlayerTarget::OwnerOf(DeclaredTarget{0})`, so
    // the context needs the warped permanent's target slot filled. Any object p1 owns will do
    // for owner resolution; use the Chaos Warp card itself.
    let warp = id_of(&state, "Chaos Warp");
    let top_before = id_of(&state, "Top Instant");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let arm = sole_effect_matching(&def_of("Chaos Warp"), is_reveal_and_route);
    let mut ctx = EffectContext::new(
        p1,
        warp,
        vec![SpellTarget {
            target: Target::Object(warp),
            zone_at_cast: Some(ZoneId::Hand(p1)),
        }],
    );
    let _ = execute_effect(&mut state, &arm, &mut ctx);

    assert_eq!(
        state.current_timestamp(),
        ts_before,
        "CR 400.7: a revealed non-permanent put back on TOP of the library it is already in \
         must mint nothing. library: {:?}",
        lib_names(&state, p1)
    );
    assert_library_ids_survived(
        &state,
        p1,
        &before,
        &["Below One", "Below Two", "Top Instant"],
    );
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).expect("library").top(),
        Some(top_before),
        "CR 401: the non-permanent must still be the TOP card, under its ORIGINAL id — \
         `unmatched_dest` is `Library {{ position: Top }}`. library: {:?}",
        lib_names(&state, p1)
    );
}

// ═════════════════════════════════════════════════════════════════════════════════════════════
// Family D — Partner With's full-library reorder, driven through resolution
// ═════════════════════════════════════════════════════════════════════════════════════════════

/// A `StackObject` skeleton — every field at its inert default, `kind` set to a placeholder the
/// callers immediately overwrite. `StackObject` has no `Default` and 60-odd fields; this is the
/// same full literal `crates/engine/tests/rules/partner_with.rs` carries, hoisted once so the
/// three stack objects this file needs (a PartnerWith trigger, a Hideaway trigger, a Worldly
/// Tutor spell) do not each repeat it.
fn keyword_trigger_skeleton(
    id: ObjectId,
    source_object: ObjectId,
    controller: PlayerId,
) -> StackObject {
    StackObject {
        id,
        controller,
        kind: StackObjectKind::Spell { source_object },
        targets: vec![],
        target_requirements: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: test objects are not cleave casts.
        was_cleaved: false,
        // CR 715.3d: test objects are not adventure casts.
        was_cast_as_adventure: false,
        cast_right_half: false,
        // CR 702.47a: test objects have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    }
}

/// A `KeywordTrigger` stack object for `PartnerWith` (CR 702.124j). The arm is only reachable
/// from a real `StackObjectKind::KeywordTrigger`, which is why this file builds one rather than
/// calling an `Effect`: family D has no `Effect` node anywhere in its defs.
fn partner_with_trigger(
    id: ObjectId,
    source_object: ObjectId,
    partner_name: &str,
    target_player: PlayerId,
    controller: PlayerId,
) -> StackObject {
    let mut so = keyword_trigger_skeleton(id, source_object, controller);
    so.kind = StackObjectKind::KeywordTrigger {
        source_object,
        keyword: KeywordAbility::PartnerWith(partner_name.to_string()),
        data: mtg_engine::state::stack::TriggerData::ETBPartnerWith {
            partner_name: partner_name.to_string(),
            target_player,
        },
    };
    so
}

/// CR 400.7 / CR 702.124j — **Pir, Imaginative Rascal**: "…target player may search their
/// library for a card named Toothy, Imaginary Friend, reveal it, put it into their hand, **then
/// shuffle**."
///
/// `rules/resolution.rs` implements that shuffle as a full-library permutation by repeated
/// `expect_move_object_to_bottom_of_zone`. Before PB-DX15a that renumbered **every card in the
/// target player's library** and advanced `timestamp_counter` once per card — in a real
/// Commander game, ~99 draws off the shuffle/coin-flip seed source from a single ETB trigger.
///
/// Driven END TO END through the real resolution path (push the trigger, pass priority), not at
/// the primitive level: this family carries no `Effect` node at all, so a primitive-level claim
/// would not touch the code that actually reorders. The named partner is deliberately NOT in the
/// library, so the only zone change in the arm is absent and the whole counter delta belongs to
/// the shuffle seed.
/// **Reverts watched red**: V2, V4.
#[test]
fn t_partner_with_shuffle_reorders_the_library_without_renumbering_it() {
    let p1 = p(1);
    let p2 = p(2);
    let lib = ZoneId::Library(p1);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(real_card(
            p1,
            "Pir, Imaginative Rascal",
            ZoneId::Battlefield,
        ));
    for i in 0..12 {
        builder = builder.object(filler(p1, &format!("Deck Card {i}"), CardType::Land, lib));
    }
    let mut state = builder.build().unwrap();

    let pir = id_of(&state, "Pir, Imaginative Rascal");
    let before = lib_ids(&state, p1);
    assert_eq!(before.len(), 12, "sanity: twelve cards in the library");
    assert!(
        state
            .objects()
            .values()
            .all(|o| o.characteristics.name != "Toothy, Imaginary Friend"),
        "the named partner is deliberately absent, so the arm's only zone change does not occur"
    );

    let trigger_id = test_util::next_object_id(&mut state);
    state.stack_objects_mut().push_back(partner_with_trigger(
        trigger_id,
        pir,
        "Toothy, Imaginary Friend",
        p1,
        p1,
    ));
    state.turn_mut().priority_holder = Some(p1);

    let ts_before = state.current_timestamp();

    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    assert!(
        state.stack_objects().is_empty(),
        "CR 702.124j: the Partner With trigger should have resolved"
    );

    let after = lib_ids(&state, p1);
    assert_eq!(
        after.len(),
        12,
        "the shuffle must not change the library's size"
    );
    let before_set: std::collections::BTreeSet<ObjectId> = before.iter().copied().collect();
    let after_set: std::collections::BTreeSet<ObjectId> = after.iter().copied().collect();
    assert_eq!(
        before_set, after_set,
        "CR 400.7: a shuffle is a reorder WITHIN one zone, so every card must keep the \
         ObjectId it had. Before PB-DX15a each of the 12 was retired and re-minted. \
         before={before:?} after={after:?}"
    );
    assert_eq!(
        state.current_timestamp(),
        ts_before + 1,
        "the arm must consume exactly ONE counter value — the seeded-LCG shuffle seed \
         (`rules/resolution.rs`, the PartnerWith arm) — and NOT one per card. Before \
         PB-DX15a this was 1 + 12."
    );
}

/// CR 400.7 / CR 701.20 — **Chaos Warp**'s FULL printed sequence, so the same-zone arm is
/// measured in the company of the two genuine counter consumers it ships beside rather than in
/// isolation. "The owner of target permanent shuffles it into their library, then reveals the
/// top card of their library. If it's a permanent card, they put it onto the battlefield."
///
/// The expected `timestamp_counter` delta is DERIVED, not pinned as a magic number:
///
/// * `Effect::MoveZone` battlefield → library — a real zone change (CR 400.7): **1**;
/// * `Effect::Shuffle` — `effects/mod.rs` seeds `Zone::shuffle` from `timestamp_counter` and
///   advances it: **1**;
/// * `Effect::RevealAndRoute` — the revealed top card is routed to `unmatched_dest`
///   (`Library { Top }`, same zone: **0**) or to the battlefield (a zone change: **1**).
///
/// The probe asserts the branch it took rather than accepting either, and asserts that every
/// card which stayed in the library kept its id across the shuffle.
/// **Revert watched red**: V1.
#[test]
fn t_chaos_warp_full_sequence_consumes_only_what_it_owes() {
    let p1 = p(1);
    let lib = ZoneId::Library(p1);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(real_card(p1, "Chaos Warp", ZoneId::Hand(p1)))
        .object(filler(
            p1,
            "Warped Permanent",
            CardType::Artifact,
            ZoneId::Battlefield,
        ));
    for i in 0..6 {
        builder = builder.object(filler(
            p1,
            &format!("Deck Card {i}"),
            CardType::Instant,
            lib,
        ));
    }
    let mut state = builder.build().unwrap();

    let warp = id_of(&state, "Chaos Warp");
    let victim = id_of(&state, "Warped Permanent");
    // `victim`'s id is used only to fill the target slot; it is retired by the MoveZone below
    // (a genuine zone change) and is deliberately not expected to survive.
    let lib_before = lib_ids(&state, p1);
    assert_eq!(lib_before.len(), 6, "sanity: six cards in the library");
    let ts_before = state.current_timestamp();

    let seq = sole_effect_matching(&def_of("Chaos Warp"), |e| matches!(e, Effect::Sequence(_)));
    let mut ctx = EffectContext::new(
        p1,
        warp,
        vec![SpellTarget {
            target: Target::Object(victim),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let _ = execute_effect(&mut state, &seq, &mut ctx);

    // Which branch did the reveal take?
    let revealed_hit_battlefield = state
        .objects()
        .values()
        .any(|o| o.characteristics.name == "Warped Permanent" && o.zone == ZoneId::Battlefield);
    let expected_delta = 1 /* MoveZone battlefield→library */
        + 1 /* Effect::Shuffle seed */
        + u64::from(revealed_hit_battlefield); /* the reveal's own zone change, if any */

    eprintln!(
        "PB-DX15a Chaos Warp: revealed_hit_battlefield={revealed_hit_battlefield}, \
         ts delta={}, library={:?}",
        state.current_timestamp() - ts_before,
        lib_names(&state, p1)
    );
    assert_eq!(
        state.current_timestamp() - ts_before,
        expected_delta,
        "Chaos Warp must consume exactly 1 (MoveZone) + 1 (Shuffle) + {} (reveal's zone \
         change) counter values. Anything more means the shuffle or the same-zone reveal arm \
         is renumbering cards that never changed zones (CR 400.7).",
        u64::from(revealed_hit_battlefield)
    );

    // Every one of the six original library cards that is STILL in the library must be the same
    // object it was — the shuffle is a within-zone permutation (CR 701.20a), not 6 zone changes.
    for id in lib_ids(&state, p1) {
        // The warped permanent is the one card in this library that SHOULD carry a new id: it
        // genuinely moved battlefield → library, which is CR 400.7's antecedent. It is matched
        // by NAME because its pre-move id is dead by construction.
        assert!(
            lib_before.contains(&id) || name_of(&state, id) == "Warped Permanent",
            "CR 400.7: {id:?} ('{}') is in the library but is neither one of the six that \
             started there nor the warped permanent — a shuffled card was re-minted",
            name_of(&state, id)
        );
    }
    for &id in &lib_before {
        let still_there = state
            .objects()
            .get(&id)
            .map(|o| o.zone == ZoneId::Library(p1))
            .unwrap_or(false);
        let left_for_battlefield = state
            .objects()
            .get(&id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        assert!(
            still_there || left_for_battlefield,
            "CR 400.7: {id:?} was in the library before the shuffle and its id is now dead — a \
             shuffle must preserve identity"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════
// Family B — Effect::SearchLibrary, destination = Library
// ═════════════════════════════════════════════════════════════════════════════════════════════

/// Builds a 2-player state with **Worldly Tutor** on the stack as a real `StackObjectKind::
/// Spell` and `library_names` (bottom-to-top) in p1's library, ready for two `PassPriority`s.
///
/// The tutor is put on the stack through a genuine `move_object_to_zone` (hand → stack, a real
/// CR 400.7 zone change) rather than being conjured there, and the caller snapshots
/// `current_timestamp()` AFTER this returns, so the setup's own consumption is outside the
/// measurement window.
fn tutor_on_the_stack(library: &[(&str, CardType)]) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let lib = ZoneId::Library(p1);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(real_card(p1, "Worldly Tutor", ZoneId::Hand(p1)));
    for (name, ty) in library {
        builder = builder.object(filler(p1, name, *ty, lib));
    }
    let mut state = builder.build().unwrap();

    let tutor_in_hand = id_of(&state, "Worldly Tutor");
    let (tutor_on_stack, _) =
        test_util::move_object_to_zone(&mut state, tutor_in_hand, ZoneId::Stack).unwrap();
    let stack_id = test_util::next_object_id(&mut state);
    let mut so = keyword_trigger_skeleton(stack_id, tutor_on_stack, p1);
    so.kind = StackObjectKind::Spell {
        source_object: tutor_on_stack,
    };
    state.stack_objects_mut().push_back(so);
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

fn resolve_top_of_stack(state: GameState, p1: PlayerId, p2: PlayerId) -> GameState {
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    // CR 701.23a / CR 608.2d (PB-DP9): a library search blocks on the searching player's
    // announcement and the resolution is rolled back until it is answered. The harness answers
    // with the engine's own deterministic default, which is `candidates[0]`; every fixture here
    // is built so the filter leaves exactly one legal answer, so this probe asserts what the
    // FILTER admits and not which of several cards a player would pick.
    let (state, _) = mtg_engine::testing::replay_harness::auto_answer_blocking_decisions(state);
    assert!(
        state.stack_objects().is_empty(),
        "the spell should have resolved"
    );
    state
}

/// CR 400.7 / CR 701.23a / CR 701.20a — **Worldly Tutor**: "Search your library for a creature
/// card, reveal it, then shuffle and put the card on top."
///
/// The found card is placed into the library it was found in, so its `ObjectId` must survive —
/// before PB-DX15a the card a player tutored for was retired and the card that ended on top of
/// their library was a **different object** with a fresh id, which is what CR 400.7 explicitly
/// does not license for a card that never left its zone.
///
/// Driven through the real resolution path (stack object + `PassPriority` + PB-DP9's blocking
/// announcement), not through a bare `execute_effect`: `Effect::SearchLibrary` **suspends** and
/// rolls the whole resolution back until the search is answered, so a direct `execute_effect`
/// call applies nothing at all and would have passed this probe vacuously. (It did — the first
/// draft of this test measured a delta of 0 and an untouched library, and the reason was that
/// the effect had never run.)
///
/// The `timestamp_counter` delta is **2**, and the decomposition is not asserted by claim but
/// **proved by the control below**: `t_worldly_tutor_with_nothing_to_find_consumes_only_the_
/// spell_move` runs the identical fixture with no creature in the library, takes the
/// `candidates.is_empty()` branch (no search question, no shuffle), and measures **1**. The
/// difference between the two runs is exactly the `shuffle_before_placing` seed
/// (`effects/mod.rs`, one `timestamp_counter += 1` before `Zone::shuffle`), so the placement
/// itself is proven to consume **zero**.
///
/// One member of family B is probed rather than all eight: the eight share a single executor
/// arm, and `r2_search_library_into_library_population_is_pinned` is what guarantees the
/// membership. This probe exists because `shuffle_before_placing`'s own counter draw is the
/// arithmetic most easily mistaken for the defect.
/// **Reverts watched red**: V1, V3, V9 (the card def leaving family B).
#[test]
fn t_worldly_tutor_puts_the_found_card_back_on_top_under_its_own_id() {
    let (state, p1, p2) = tutor_on_the_stack(&[
        ("Deck Card A", CardType::Land),
        ("Deck Card B", CardType::Instant),
        // The library's ONLY creature, so the filter leaves exactly one legal answer.
        ("The Only Creature", CardType::Creature),
        ("Deck Card C", CardType::Sorcery),
    ]);
    let creature_before = id_of(&state, "The Only Creature");
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let state = resolve_top_of_stack(state, p1, p2);

    assert_library_ids_survived(
        &state,
        p1,
        &before,
        &[
            "Deck Card A",
            "Deck Card B",
            "Deck Card C",
            "The Only Creature",
        ],
    );
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).expect("library").top(),
        Some(creature_before),
        "CR 701.23a: the found creature must end on TOP of the library under its ORIGINAL id — \
         it never left the library, so CR 400.7 does not make it a new object. library: {:?}",
        lib_names(&state, p1)
    );
    assert_eq!(
        state.current_timestamp() - ts_before,
        2,
        "the resolution owes exactly two counter values: the spell's own stack→graveyard move \
         (a real zone change) and the `shuffle_before_placing` seed. A third would mean the \
         library→library placement is minting an id again (OOS-DP9-11). library: {:?}",
        lib_names(&state, p1)
    );
}

/// The control that turns the sibling probe's "2" from a pinned number into a **decomposition**.
///
/// CR 701.23d — with no creature in the library at all, `Effect::SearchLibrary` takes its
/// `candidates.is_empty()` branch: no announcement is asked for, nothing is found, and
/// `shuffle_before_placing` never runs (it is inside the `if let Some(card_id) = found` block).
/// The only counter value the resolution owes is the spell's own stack→graveyard move, so the
/// delta is **1**.
///
/// Read against its sibling: 2 − 1 = 1 = the shuffle seed, which leaves **0** for the placement.
/// That subtraction is the evidence for the sibling's claim; without this row, "2" would be a
/// number whose parts a reader has to take on trust.
///
/// **UNDISCRIMINATED, deliberately and disclosed here rather than only in the close notes.**
/// This is the one row in PB-DX15a's test surface that no revert of the batch's engine change
/// turns red, and that is structural rather than an oversight: by construction the fixture
/// contains **no same-zone move at all** (nothing is found, so nothing is placed), which is
/// exactly what makes it a valid control. All five engine reverts executed for this batch
/// (disable either `from == to` guard, swap the two `ZoneEnd` arms, make
/// `reposition_within_own_zone` stop repositioning, delete the Hideaway seed advance) leave it
/// green. Its value is the subtraction, not detection; do not "strengthen" it by giving it a
/// same-zone move, because that would destroy the property it exists to establish.
#[test]
fn t_worldly_tutor_with_nothing_to_find_consumes_only_the_spell_move() {
    let (state, p1, p2) = tutor_on_the_stack(&[
        ("Deck Card A", CardType::Land),
        ("Deck Card B", CardType::Instant),
        ("Deck Card C", CardType::Sorcery),
    ]);
    let before = lib_ids(&state, p1);
    let ts_before = state.current_timestamp();

    let state = resolve_top_of_stack(state, p1, p2);

    assert_library_ids_survived(
        &state,
        p1,
        &before,
        &["Deck Card A", "Deck Card B", "Deck Card C"],
    );
    assert_eq!(
        state.current_timestamp() - ts_before,
        1,
        "CR 701.23d: a search that finds nothing shuffles nothing and places nothing, so the \
         only counter value owed is the spell's own stack→graveyard move"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════════════════
// Family C — Hideaway's "rest on the bottom in a random order"
// ═════════════════════════════════════════════════════════════════════════════════════════════

fn hideaway_trigger(
    id: ObjectId,
    source_object: ObjectId,
    count: u32,
    controller: PlayerId,
) -> StackObject {
    let mut so = keyword_trigger_skeleton(id, source_object, controller);
    so.kind = StackObjectKind::KeywordTrigger {
        source_object,
        keyword: KeywordAbility::Hideaway(count),
        data: mtg_engine::state::stack::TriggerData::ETBHideaway { count },
    };
    so
}

/// CR 400.7 / CR 702.75a — **Windbrisk Heights**: "look at the top four cards of your library,
/// exile one face down, then put the rest on the bottom **in a random order**."
///
/// The three "rest" cards never leave the library, so their ids must survive; only the exiled
/// card genuinely changes zones. Expected `timestamp_counter` delta: **exactly 2** —
///
/// * 1 for the exile (library → exile, a real CR 400.7 zone change), and
/// * 1 for the seeded-LCG "random order" seed.
///
/// **That second value is itself part of this batch's engine change**, and this row is why it is
/// tested here: the Hideaway site used to READ `timestamp_counter` without advancing it, getting
/// away with it only because the bottom-moves that followed advanced the counter as a side
/// effect of minting ids. With those moves made identity-preserving, two Hideaway triggers with
/// no intervening counter movement would have seeded the LCG **identically**. The `+= 1` at
/// `rules/resolution.rs` is what this delta of 2 pins.
///
/// **Revert to watch red**: delete that `state.timestamp_counter += 1;` in the Hideaway arm
/// (delta becomes 1).
/// **Reverts watched red**: V2, V4, and V5 (delete the Hideaway LCG `timestamp_counter += 1`).
#[test]
fn t_hideaway_bottoms_the_rest_in_place_and_still_advances_its_own_seed() {
    let p1 = p(1);
    let p2 = p(2);
    let lib = ZoneId::Library(p1);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(real_card(p1, "Windbrisk Heights", ZoneId::Battlefield));
    for i in 0..6 {
        builder = builder.object(filler(p1, &format!("Deck Card {i}"), CardType::Land, lib));
    }
    let mut state = builder.build().unwrap();

    let land = id_of(&state, "Windbrisk Heights");
    let before = lib_ids(&state, p1);
    assert_eq!(before.len(), 6, "sanity: six cards in the library");

    let trigger_id = test_util::next_object_id(&mut state);
    state
        .stack_objects_mut()
        .push_back(hideaway_trigger(trigger_id, land, 4, p1));
    state.turn_mut().priority_holder = Some(p1);

    let ts_before = state.current_timestamp();

    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    assert!(
        state.stack_objects().is_empty(),
        "CR 702.75a: the Hideaway trigger should have resolved"
    );

    let after = lib_ids(&state, p1);
    assert_eq!(
        after.len(),
        5,
        "one of the six cards was exiled face down (CR 702.75a); the other five stay. \
         library: {:?}",
        lib_names(&state, p1)
    );
    for id in &after {
        assert!(
            before.contains(id),
            "CR 400.7: {id:?} ('{}') is in the library but was not one of the six that started \
             there — Hideaway's 'put the rest on the bottom' re-minted a card that never left \
             its zone",
            name_of(&state, *id)
        );
    }
    assert_eq!(
        state.current_timestamp() - ts_before,
        2,
        "CR 702.75a: exactly TWO counter values are owed — one for the exile (a genuine zone \
         change) and one for the 'random order' LCG seed. A delta of 1 means the Hideaway arm \
         reads the counter without advancing it, so two Hideaway triggers in one game with no \
         intervening counter movement would produce the SAME 'random' order. A delta of 4 means \
         the three bottomed cards are being renumbered again (OOS-DP9-11)."
    );
}
