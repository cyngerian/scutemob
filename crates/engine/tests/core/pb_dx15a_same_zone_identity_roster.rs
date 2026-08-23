//! PB-DX15a (CR 400.7): the corpus roster gate for the **same-zone move** class, plus the
//! behavioural pin that a same-zone move preserves object identity in every zone kind.
//!
//! CR 400.7's antecedent is *"if an object moves **from one zone to another**"*. A move whose
//! destination **is** the zone the object is already in is not that, so the object does **not**
//! become a new object: it keeps its `ObjectId`, keeps every field, captures no LKI, and — the
//! half nothing else in this suite watches — consumes **no** `timestamp_counter` value.
//! `timestamp_counter` is the seed source for every `Zone::shuffle` and every coin flip
//! (`effects/mod.rs:4133`, `:4312`, `:5243`, `:5267`, `rules/resolution.rs`' two Hideaway /
//! PartnerWith LCGs), so a renumbering same-zone move was not merely cosmetic — it perturbed
//! future randomness by an amount proportional to how many cards a card def happened to reorder.
//! Closes `OOS-DP9-11`.
//!
//! ## SR-36: every population here is enumerated from `all_cards()`, never grepped from source
//!
//! The one exception is `r5_the_two_move_helpers_guard_before_they_mint`, which scans
//! `crates/engine/src/state/mod.rs` — that row's subject is *engine call sites*, not cards, and
//! it exists because `GameState::reposition_within_own_zone`'s own doc comment promises it does
//! ("The roster gate in `crates/engine/tests/core/pb_dx15a_same_zone_identity_roster.rs` pins
//! the remaining `next_object_id()` call sites so a third minting path cannot appear silently").
//! A doc comment asserting a test that does not exist is a claim like any other; this is the
//! test.
//!
//! ## FOUR independent families, and that is the point
//!
//! PB-DX26 and PB-DX43 both learned the same lesson the hard way: **a roster derived from one
//! declaration construct measures that construct.** The same-zone class is reachable from four
//! structurally unrelated declaration sites, and no single walk sees more than one of them:
//!
//! * **A** — `Effect::RevealAndRoute { unmatched_dest: ZoneTarget::Library { .. } }` and
//!   `Effect::LookAtTopThenPlace { rest_to: ZoneTarget::Library { .. } }`. Both route the
//!   *unplaced remainder* — cards that never left the library — back into that library.
//!   **Deliberately not scoped to `LibraryPosition::Bottom`**: `chaos_warp` routes to
//!   `Top`, and a gate scoped to `Bottom` (the position the seed names, and the position four
//!   of the five members use) would silently omit it. The predicate keys on the ZONE, which is
//!   what decides same-zone-ness, not on the position, which does not.
//! * **B** — `Effect::SearchLibrary { destination: ZoneTarget::Library { .. } }`. The tutors
//!   that put the found card back on top of the library it was found in.
//! * **C** — `KeywordAbility::Hideaway(_)` (CR 702.75a). Reaches
//!   `expect_move_object_to_bottom_of_zone` from `rules/resolution.rs`, from a *keyword*, with
//!   no `Effect` node anywhere in the def — invisible to A and B by construction.
//! * **D** — `KeywordAbility::PartnerWith(_)` (CR 702.124j). Its "then shuffle" is implemented
//!   as a full-library reorder by repeated `expect_move_object_to_bottom_of_zone`, so it
//!   renumbered the **entire** library of the target player. Also invisible to A and B.
//!
//! ## Structural JSON walk (mirrors `pb_dx43_land_type_roster.rs` / `pb_dx42a_*`)
//!
//! Nodes are matched by single-key serde variant shape, so the walk reaches nodes nested at
//! arbitrary depth inside `Effect::Sequence`, `Effect::Repeat`, `Effect::ForEach`,
//! `Effect::Conditional`, a `back_face`'s abilities, a `TokenSpec`'s keywords, and so on.
//! `chaos_warp`'s member node is the third element of an `Effect::Sequence`; `growing_rites_
//! of_itlimoc`'s is inside the second of two `AbilityDefinition::Triggered` entries.
//!
//! ## Why there are no `!is_empty()` floors under the four exact-set pins
//!
//! Every family row ends in an exact-set `assert_eq!` over a `BTreeSet<String>`. A trailing
//! non-vacuity floor beneath one of those is **dead code** — the equality fires first on any
//! walk that went vacuous — and PB-DX28's review deleted exactly that construct from
//! `pb_dx43_land_type_roster.rs` for exactly that reason. The floors in this file are placed
//! only where they can actually fire: `t_total_population_report`'s disjointness identity
//! (an independent property the four pins do not imply) and
//! `t_same_zone_move_preserves_identity_in_every_zone_kind`'s zone-kind coverage check.

use mtg_engine::state::test_util;
use mtg_engine::{
    all_cards, CardDefinition, CardRegistry, Completeness, GameState, GameStateBuilder, ObjectId,
    ObjectSpec, PlayerId, ZoneId,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn complete_defs() -> Vec<CardDefinition> {
    all_cards()
        .into_iter()
        .filter(|d| d.completeness == Completeness::Complete)
        .collect()
}

// ── Generic single-key-variant JSON helpers ─────────────────────────────────────────────────

/// If `v` is a single-key JSON object (the shape serde produces for a struct/tuple enum
/// variant), returns `(key, payload)`.
fn variant_key(v: &Value) -> Option<(&str, &Value)> {
    match v {
        Value::Object(m) if m.len() == 1 => {
            let (k, val) = m.iter().next().unwrap();
            Some((k.as_str(), val))
        }
        _ => None,
    }
}

/// Is `v` a `ZoneTarget::Library { owner, position }` node?
///
/// The `position` probe is what distinguishes this from any other hypothetical single-key
/// `"Library"` node. It also makes the predicate **fail closed**: rename or delete
/// `LibraryPosition` and this returns `false` everywhere, which reddens the exact-set pins in
/// R1 and R2 rather than quietly shrinking them to the empty set behind a floor.
fn is_library_zone_target(v: &Value) -> bool {
    matches!(variant_key(v), Some(("Library", payload)) if payload.get("position").is_some())
}

fn walk<F: FnMut(&Value)>(v: &Value, f: &mut F) {
    f(v);
    match v {
        Value::Object(m) => {
            for c in m.values() {
                walk(c, f);
            }
        }
        Value::Array(items) => {
            for c in items {
                walk(c, f);
            }
        }
        _ => {}
    }
}

/// Every `Complete` def somewhere inside whose serialized form `pred` matches a node.
fn population(pred: impl Fn(&Value) -> bool) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in complete_defs() {
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        let mut hit = false;
        walk(&json, &mut |node| {
            if pred(node) {
                hit = true;
            }
        });
        if hit {
            out.insert(def.name.clone());
        }
    }
    out
}

fn family_a() -> BTreeSet<String> {
    population(|v| match variant_key(v) {
        Some(("RevealAndRoute", pl)) => {
            pl.get("unmatched_dest").map(is_library_zone_target) == Some(true)
        }
        Some(("LookAtTopThenPlace", pl)) => {
            pl.get("rest_to").map(is_library_zone_target) == Some(true)
        }
        _ => false,
    })
}

fn family_b() -> BTreeSet<String> {
    population(|v| match variant_key(v) {
        Some(("SearchLibrary", pl)) => {
            pl.get("destination").map(is_library_zone_target) == Some(true)
        }
        _ => false,
    })
}

fn family_c() -> BTreeSet<String> {
    population(|v| matches!(variant_key(v), Some(("Hideaway", _))))
}

fn family_d() -> BTreeSet<String> {
    population(|v| matches!(variant_key(v), Some(("PartnerWith", _))))
}

fn named(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

// ── R1: the remainder-routing family ────────────────────────────────────────────────────────

/// **Family A**, pinned BY NAME (measured 2026-08-23, and the count reproduced the plan's
/// census of 5 exactly).
///
/// CR 400.7 / CR 701.20a (reveal) / CR 120 (look). Every `Complete` def whose
/// `Effect::RevealAndRoute.unmatched_dest` or `Effect::LookAtTopThenPlace.rest_to` is a library:
/// the remainder cards were looked at *in* the library and are routed back *to* that library, so
/// before PB-DX15a every one of them was retired and re-minted with a fresh `ObjectId`.
///
/// `chaos_warp` is the member that justifies not scoping this row to `LibraryPosition::Bottom`
/// — its `unmatched_dest` is `Top` ("reveals the top card … if it's a permanent card they put
/// it onto the battlefield", so a non-permanent stays exactly where it is).
///
/// **If this GREW**: the new member needs a probe in
/// `crates/engine/tests/primitives/pb_dx15a_same_zone_defs.rs`. **If it SHRANK**: a def lost its
/// library-routing effect — re-check the def, not this gate.
/// **Revert watched red**: V8 — `sylvan_messenger`'s `unmatched_dest` changed from `Library` to `Graveyard`.
#[test]
fn r1_remainder_routed_back_to_library_population_is_pinned() {
    let actual = family_a();
    eprintln!("PB-DX15a family A ({}): {:?}", actual.len(), actual);
    assert_eq!(
        actual,
        named(&[
            "Birthing Ritual",
            "Chaos Warp",
            "Goblin Ringleader",
            "Growing Rites of Itlimoc",
            "Sylvan Messenger",
        ]),
        "family A (RevealAndRoute.unmatched_dest / LookAtTopThenPlace.rest_to → Library) has \
         changed"
    );
}

// ── R2: the tutor family ────────────────────────────────────────────────────────────────────

/// **Family B**, pinned BY NAME (measured 2026-08-23; reproduced the plan's census of 8).
///
/// CR 400.7 / CR 701.23a (search). Every `Complete` def whose `Effect::SearchLibrary
/// .destination` is a library — the "search your library for a card, … then shuffle, then put
/// that card on top" tutors. The found card never leaves the library, so its id must survive.
///
/// Note that this family's members ALSO consume one counter value legitimately, for the
/// `shuffle_before_placing` seed; that is a real randomness draw and PB-DX15a does not touch it.
/// See `pb_dx15a_same_zone_defs.rs` for the arithmetic.
/// **Revert watched red**: V9 — `worldly_tutor`'s `destination` changed from `Library` to `Hand`.
#[test]
fn r2_search_library_into_library_population_is_pinned() {
    let actual = family_b();
    eprintln!("PB-DX15a family B ({}): {:?}", actual.len(), actual);
    assert_eq!(
        actual,
        named(&[
            "Elvish Harbinger",
            "Enlightened Tutor",
            "Forerunner of the Legion",
            "Imperial Seal",
            "Insatiable Avarice",
            "Mystical Tutor",
            "Vampiric Tutor",
            "Worldly Tutor",
        ]),
        "family B (SearchLibrary.destination → Library) has changed"
    );
}

// ── R3: Hideaway ────────────────────────────────────────────────────────────────────────────

/// **Family C**, pinned BY NAME (measured 2026-08-23; reproduced the plan's census of 1).
///
/// CR 702.75a: "look at the top four cards of your library, exile one face down, then put the
/// rest on the bottom **in a random order**." That reorder is implemented in
/// `rules/resolution.rs` as repeated `expect_move_object_to_bottom_of_zone` over cards that are
/// already in that library — three same-zone moves per Hideaway trigger.
///
/// This family carries **no `Effect` node at all**: the whole behaviour hangs off
/// `AbilityDefinition::Keyword(KeywordAbility::Hideaway(4))`. R1's and R2's walks are
/// structurally incapable of seeing it, which is why it is its own row rather than folded in.
/// **Revert watched red**: V10 — `windbrisk_heights` drops its `Hideaway(4)` keyword.
#[test]
fn r3_hideaway_population_is_pinned() {
    let actual = family_c();
    eprintln!("PB-DX15a family C ({}): {:?}", actual.len(), actual);
    assert_eq!(
        actual,
        named(&["Windbrisk Heights"]),
        "family C (KeywordAbility::Hideaway) has changed"
    );
}

// ── R4: Partner With ────────────────────────────────────────────────────────────────────────

/// **Family D**, pinned BY NAME (measured 2026-08-23; reproduced the plan's census of 3).
///
/// CR 702.124j: "… reveal it, put it into their hand, **then shuffle**." `rules/resolution.rs`
/// implements that shuffle as a full-library permutation by repeated
/// `expect_move_object_to_bottom_of_zone`, so before PB-DX15a a single Partner With trigger
/// renumbered **every card in the target player's library** and advanced `timestamp_counter`
/// once per card. This is the largest per-event blast radius in the class — a 99-card Commander
/// library moves the shuffle/coin-flip seed source by 99 in one trigger.
/// **Revert watched red**: V11 — `pir_imaginative_rascal` drops its `PartnerWith` keyword.
#[test]
fn r4_partner_with_population_is_pinned() {
    let actual = family_d();
    eprintln!("PB-DX15a family D ({}): {:?}", actual.len(), actual);
    assert_eq!(
        actual,
        named(&[
            "Brallin, Skyshark Rider",
            "Pir, Imaginative Rascal",
            "Toothy, Imaginary Friend",
        ]),
        "family D (KeywordAbility::PartnerWith) has changed"
    );
}

// ── The union report ────────────────────────────────────────────────────────────────────────

/// Prints the union and pins its size, following PB-DX27's rule: **publish the figure, do not
/// transcribe it**. The prose above quotes 5 / 8 / 1 / 3 / 17; this test is where those numbers
/// are decided, and its `eprintln!` is where a reader gets them without trusting a comment.
///
/// # What the remaining assertion does and does NOT catch (`/review` Issue 4)
///
/// The first draft of this doc claimed the disjointness identity
/// (`|A| + |B| + |C| + |D| == |A ∪ B ∪ C ∪ D|`) was "a **live** check and not a dead floor …
/// a property none of the four exact-set pins implies". **That was false, and it is corrected
/// here rather than quietly deleted.** R1-R4 pin A, B, C and D each with an exact-set
/// `assert_eq!` against a *literal* name list, and those four literals are pairwise disjoint —
/// so while R1-R4 pass, the identity holds by construction and cannot fail. It is dead in the
/// PB-DX28 sense, and **no revert row in the matrix discriminates it**: V8-V11 each shrink one
/// family, which reddens that family's own R row first.
///
/// It is kept, narrowly, for the one thing it still does: it guards a **future re-pin**. If
/// someone later widens R1's or R3's literal list so the families overlap, this fires and says
/// the per-def file now needs one probe where it had two. That is a real if modest function, and
/// stating it is the difference between a floor and a floor that lies about itself.
///
/// A companion `assert_eq!(union.len(), 17)` was **deleted**: it was determined by R1-R4 in
/// exactly the same way and guarded nothing at all, since any change to a family's literal
/// reddens that family's row. The union size is still *published* by the `eprintln!` below,
/// which is where PB-DX27's "publish the figure, do not transcribe it" rule is actually
/// discharged.
#[test]
fn t_total_population_report() {
    let (a, b, c, d) = (family_a(), family_b(), family_c(), family_d());
    let mut union = BTreeSet::new();
    for fam in [&a, &b, &c, &d] {
        union.extend(fam.iter().cloned());
    }

    eprintln!("── PB-DX15a same-zone-move corpus population ──────────────────────────────");
    eprintln!("  A remainder→library ({}): {:?}", a.len(), a);
    eprintln!("  B search→library     ({}): {:?}", b.len(), b);
    eprintln!("  C Hideaway           ({}): {:?}", c.len(), c);
    eprintln!("  D PartnerWith        ({}): {:?}", d.len(), d);
    eprintln!("  UNION                ({}): {:?}", union.len(), union);

    assert_eq!(
        a.len() + b.len() + c.len() + d.len(),
        union.len(),
        "the four families are no longer pairwise disjoint — some def now reaches the same-zone \
         class through two different declaration constructs. That is not a failure of the fix, \
         but it changes how the per-def probe file should be read; go name the overlap. \
         A={a:?} B={b:?} C={c:?} D={d:?}"
    );
}

// ── The behavioural row: identity is preserved in EVERY zone kind ────────────────────────────

/// Every `ZoneId` variant, listed so that adding an eighth is a visible omission here rather
/// than an untested gap. Three of the seven are ordered (`Library`, `Graveyard`, `Stack`) and
/// four are unordered (`Hand`, `Battlefield`, `Exile`, `Command`) — a same-zone move into an
/// unordered zone has no order to permute and must be a true no-op, which is the correct
/// reading of "put this battlefield permanent onto the battlefield".
fn all_zone_kinds(owner: PlayerId) -> Vec<ZoneId> {
    vec![
        ZoneId::Library(owner),
        ZoneId::Hand(owner),
        ZoneId::Battlefield,
        ZoneId::Graveyard(owner),
        ZoneId::Stack,
        ZoneId::Exile,
        ZoneId::Command(owner),
    ]
}

fn build_with_three_in(zone: ZoneId, owner: PlayerId) -> GameState {
    GameStateBuilder::four_player()
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(owner, "Same Zone Alpha").in_zone(zone))
        .object(ObjectSpec::card(owner, "Same Zone Beta").in_zone(zone))
        .object(ObjectSpec::card(owner, "Same Zone Gamma").in_zone(zone))
        .build()
        .unwrap_or_else(|e| panic!("state failed to build for {zone:?}: {e:?}"))
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

/// CR 400.7 — the load-bearing behavioural pin, exercised in **all seven** zone kinds through
/// **both** move helpers.
///
/// For each zone kind: three objects live in that zone; move the middle one *into the zone it is
/// already in*, once via `move_object_to_zone` (top end) and once via
/// `move_object_to_bottom_of_zone` (bottom end), and assert all four of the properties CR 400.7
/// denies a same-zone move the right to change:
///
/// * (a) the returned `ObjectId` **is** the original — no new object (CR 400.7);
/// * (b) `state.current_timestamp()` is **unchanged** — no id was minted, so the shuffle /
///   coin-flip seed source did not move;
/// * (c) the object is still resolvable at the original id, and still reports that zone;
/// * (d) the zone's length is unchanged — a reposition removes and re-adds exactly one entry.
///
/// **Revert to watch red**: delete either `if from == to { … }` early return in
/// `crates/engine/src/state/mod.rs`.
/// **Reverts watched red**: V1, V2, V4.
#[test]
fn t_same_zone_move_preserves_identity_in_every_zone_kind() {
    let owner = p(1);
    let kinds = all_zone_kinds(owner);

    // Live non-vacuity floor: this one CAN fire, because nothing else in this test pins the
    // list's contents. `ZoneId` has seven variants (CR 400.1); if it grows an eighth and this
    // list is not extended, the new zone kind would go untested in silence.
    assert_eq!(
        kinds.len(),
        7,
        "CR 400.1: seven zone kinds are expected; the list must name every `ZoneId` variant"
    );
    let distinct_types: BTreeSet<_> = kinds.iter().map(|z| z.zone_type()).collect();
    assert_eq!(
        distinct_types.len(),
        7,
        "the seven entries must be seven DIFFERENT zone types, not a repeat: {distinct_types:?}"
    );

    for zone in kinds {
        for bottom in [false, true] {
            let mut state = build_with_three_in(zone, owner);
            let subject = id_of(&state, "Same Zone Beta");
            let ts_before = state.current_timestamp();
            let len_before = state.zone(&zone).expect("zone exists").len();
            assert_eq!(
                len_before, 3,
                "sanity: {zone:?} should hold the three fixtures"
            );

            let (returned, _old) = if bottom {
                test_util::move_object_to_bottom_of_zone(&mut state, subject, zone)
            } else {
                test_util::move_object_to_zone(&mut state, subject, zone)
            }
            .unwrap_or_else(|e| {
                panic!("same-zone move into {zone:?} (bottom={bottom}) failed: {e:?}")
            });

            // (a) CR 400.7: the object did not move from one zone to ANOTHER, so it is not a
            //     new object and its id must be the id it already had.
            assert_eq!(
                returned, subject,
                "CR 400.7: a same-zone move into {zone:?} (bottom={bottom}) minted a new \
                 ObjectId {returned:?} for an object that never left its zone"
            );
            // (b) `timestamp_counter` is the object-id counter AND the seed source for every
            //     `Zone::shuffle` and coin flip. A same-zone move must not consume one.
            assert_eq!(
                state.current_timestamp(),
                ts_before,
                "a same-zone move into {zone:?} (bottom={bottom}) consumed a \
                 timestamp_counter value — that counter seeds every shuffle and coin flip, so \
                 this silently perturbs future randomness"
            );
            // (c) The original id is still live and still reports the same zone.
            let obj = state.objects().get(&subject).unwrap_or_else(|| {
                panic!(
                    "{subject:?} is gone from state.objects() after a \
                     same-zone move into {zone:?} (bottom={bottom})"
                )
            });
            assert_eq!(
                obj.zone, zone,
                "the repositioned object should still report {zone:?}"
            );
            assert_eq!(
                obj.characteristics.name, "Same Zone Beta",
                "the id must still resolve to the SAME object, not a re-minted stand-in"
            );
            // (d) Exactly one entry was removed and re-added.
            assert_eq!(
                state.zone(&zone).expect("zone exists").len(),
                len_before,
                "a reposition must not change {zone:?}'s size (bottom={bottom})"
            );
        }
    }
}

/// CR 401 — the ordered-zone halves actually reposition, so (a)-(d) above are not passing
/// because the helpers became no-ops.
///
/// In an ordered zone (`Zone::Ordered(Vector)`, top == last index) a `move_object_to_zone`
/// same-zone move must land the subject on **top** and a `move_object_to_bottom_of_zone` must
/// land it at **index 0**. `Zone::object_ids()` returns unordered zones' contents in `OrdSet`
/// order (ascending `ObjectId`), which is why this row is scoped to the three ordered kinds:
/// asserting a position in an unordered zone would be asserting a fact about `OrdSet`, not
/// about the engine.
///
/// **Revert to watch red**: swap `ZoneEnd::Top`/`ZoneEnd::Bottom` in
/// `reposition_within_own_zone`, or make it `return object_id` without repositioning.
/// **Reverts watched red**: V1, V2, V3.
#[test]
fn t_same_zone_move_actually_repositions_in_ordered_zones() {
    let owner = p(1);
    for zone in [
        ZoneId::Library(owner),
        ZoneId::Graveyard(owner),
        ZoneId::Stack,
    ] {
        // `Same Zone Alpha` is added first, so in an ordered zone it starts at index 0 (the
        // bottom) and is NOT on top. Moving it "to" its own zone must put it on top.
        let mut state = build_with_three_in(zone, owner);
        let subject = id_of(&state, "Same Zone Alpha");
        assert_eq!(
            state.zone(&zone).unwrap().object_ids()[0],
            subject,
            "sanity: {zone:?} should start with Alpha at index 0"
        );
        let (returned, _) = test_util::move_object_to_zone(&mut state, subject, zone).unwrap();
        assert_eq!(returned, subject);
        assert_eq!(
            state.zone(&zone).unwrap().top(),
            Some(subject),
            "CR 401: a same-zone move through `move_object_to_zone` must reposition the object \
             to the TOP of {zone:?}, not leave it where it was"
        );

        // And the bottom helper must put a top card at index 0.
        let mut state = build_with_three_in(zone, owner);
        let subject = id_of(&state, "Same Zone Gamma");
        assert_eq!(
            state.zone(&zone).unwrap().top(),
            Some(subject),
            "sanity: {zone:?} should start with Gamma on top"
        );
        let (returned, _) =
            test_util::move_object_to_bottom_of_zone(&mut state, subject, zone).unwrap();
        assert_eq!(returned, subject);
        assert_eq!(
            state.zone(&zone).unwrap().object_ids()[0],
            subject,
            "CR 401: a same-zone move through `move_object_to_bottom_of_zone` must reposition \
             the object to the BOTTOM (index 0) of {zone:?}"
        );
    }
}

// ── R5: the source-scan pin `reposition_within_own_zone`'s doc promises ─────────────────────

/// The two move helpers must each **guard before they mint**.
///
/// `GameState::reposition_within_own_zone`'s doc comment states that this file "pins the
/// remaining `next_object_id()` call sites so a third minting path cannot appear silently".
/// This is that pin. It asserts three things about `crates/engine/src/state/mod.rs`:
///
/// 1. the file contains exactly **5** `next_object_id()` call sites (`add_object`, the two move
///    helpers' `new_id`, and the two merged-component paths) — a sixth is a new minting path and
///    must be classified deliberately;
/// 2. each of `move_object_to_zone` and `move_object_to_bottom_of_zone` contains exactly one
///    `next_object_id()`; and
/// 3. in each, the `reposition_within_own_zone` guard appears **before** that call — deleting
///    the guard, or moving it below the mint, reddens this row even if every card-level probe
///    were somehow satisfied.
///
/// Comment-stripping is deliberate and load-bearing (`OOS-DX32-6`, and PB-DX24's proof that a
/// gate which counts a token inside a comment measures the comment): `reposition_within_own_zone`
/// is *named* in a doc comment inside `move_object_to_zone`'s neighbourhood, so an unstripped
/// scan could find the guard "before" the mint without any guard existing.
///
/// **Revert to watch red**: delete either `if from == to { … }` early return.
/// **Reverts watched red**: V6 (a sixth `next_object_id()` call site added inside `move_object_to_bottom_of_zone`) and V7 (`reposition_within_own_zone` renamed with behaviour unchanged — the needle disappears and this row is the only one that notices, which is both its purpose and the honest statement of its limit: it measures a NAME, so it cannot police a guard that is present but wrong).
/// # Reach, stated in the criterion's own terms (`/review` Issue 9)
///
/// This gate scans **`crates/engine/src/state/mod.rs` only**. `GameState::next_object_id` is
/// `pub(crate)` with ~30 callers outside that file (`rules/abilities.rs`, `rules/copy.rs`, …),
/// so a hand-rolled same-zone renumber written elsewhere — `zone.remove(id)` +
/// `next_object_id()` + `objects.insert(..)` — is **invisible** to it.
///
/// That is a smaller claim than "a new same-zone caller cannot renumber", and the difference
/// matters. What actually closes the class is not this gate but the guard itself: every
/// same-zone move *through the two helpers* is identity-preserving **by construction**, so
/// there is no caller of them left to police. This row's job is narrower — it stops a third
/// minting path from being added *inside* the helpers without anyone noticing. A same-zone
/// reorder open-coded in `effects/` or `rules/` would evade both, and no gate in this batch
/// covers that; it is stated here rather than left to be discovered.
///
#[test]
fn r5_the_two_move_helpers_guard_before_they_mint() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/state/mod.rs");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", path.display()));

    // Strip `//`-style comments (including `///` docs) line by line. `state/mod.rs` carries no
    // `/* */` blocks; if one ever appears carrying either needle, assertion (3) is the one that
    // would go wrong, and it fails CLOSED (a commented-out guard reads as absent).
    let src: String = raw
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");

    let mint_sites = src.matches("next_object_id()").count();
    eprintln!("PB-DX15a R5: {mint_sites} `next_object_id()` call sites in state/mod.rs");
    assert_eq!(
        mint_sites, 5,
        "state/mod.rs's ObjectId-minting call sites changed from the 5 measured 2026-08-23 \
         (`add_object`; `move_object_to_zone`'s `new_id`; `move_object_to_bottom_of_zone`'s \
         `new_id`; the two merged-component re-mint paths). A new one is a new CR 400.7 \
         'becomes a new object' path and must be checked for the same-zone guard before this \
         count is updated."
    );

    // Per-function mint counts, measured 2026-08-23. `move_object_to_zone` legitimately
    // carries THREE: its own cross-zone `new_id`, plus two `component_id` re-mints for the
    // merged components of a mutate/meld pile coming apart (CR 702.140f / CR 728.3b), each of
    // which really is a separate object arriving in a separate zone.
    // `move_object_to_bottom_of_zone` carries one. The first draft of this row asserted "one
    // each" and the test reddened on its first run — the count is measured here rather than
    // assumed, which is the whole reason it is a gate.
    for (func, expected_mints) in [
        ("fn move_object_to_zone(", 3usize),
        ("fn move_object_to_bottom_of_zone(", 1usize),
    ] {
        let start = src.find(func).unwrap_or_else(|| {
            panic!(
                "`{func}` not found in state/mod.rs — has it been renamed? \
                 `reposition_within_own_zone`'s doc comment names both helpers."
            )
        });
        // The body runs to the next `\n    fn ` / `\n    pub` sibling at impl-item indentation.
        let rest = &src[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .into_iter()
            .chain(rest[1..].find("\n    pub "))
            .min()
            .map(|i| i + 1)
            .unwrap_or(rest.len());
        let body = &rest[..end];

        assert_eq!(
            body.matches("next_object_id()").count(),
            expected_mints,
            "`{func}` should mint exactly {expected_mints} ObjectId(s); a change here is a new \
             CR 400.7 'becomes a new object' path inside a move helper"
        );
        let guard_at = body.find("reposition_within_own_zone").unwrap_or_else(|| {
            panic!(
                "`{func}` no longer calls `reposition_within_own_zone` — the CR 400.7 same-zone \
                 guard is GONE, and every same-zone move in the corpus is renumbering objects \
                 again (OOS-DP9-11)"
            )
        });
        // The FIRST mint in the body — the guard must precede every one of them, and
        // preceding the first is sufficient because the guard `return`s.
        let mint_at = body.find("next_object_id()").expect("counted above");
        assert!(
            guard_at < mint_at,
            "`{func}` reaches `next_object_id()` before its same-zone guard — the guard must \
             short-circuit BEFORE an id is minted, or CR 400.7's 'new object' rule is applied \
             to an object that never changed zones"
        );
    }
}
