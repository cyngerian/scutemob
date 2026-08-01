//! View-model tests: the omniscient regression guard and the Architecture
//! Invariant 7 leak gates.
//!
//! The leak gates are deliberately written as **whole-document substring
//! scans**: the seat view is serialized to a JSON string and every card name the
//! viewer is not entitled to is asserted absent. A field-by-field assertion only
//! checks the fields the author remembered; a document scan also catches a leak
//! through a field added later, or through one nobody thought of.

use mtg_engine::{
    AttackTarget, CardId, CombatState, CounterType, FaceDownKind, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaPool, ObjectId, ObjectSpec, PlayerId, SpellTarget,
    StackObject, StackObjectKind, Step, SubType, SuperType, Target, ZoneId,
};
use std::collections::HashMap;

use crate::redact::{FACE_DOWN_NAME, HIDDEN_CARD_NAME};
use crate::{event_view_for, StateViewModel, Viewer};

// ── Fixture card-name inventories, for the whole-document leak scans ─────────

/// Cards in seats other than `alice`'s hand (CR 402.1 — hidden from alice).
const OTHER_SEAT_HAND_CARDS: &[&str] = &["Wrath of God", "Sol Ring", "Demonic Tutor"];

/// Every card in every library. CR 401.2: a library is a hidden zone whose
/// order no player may know — including its own owner. None of these may appear
/// in ANY view, seat or omniscient.
const LIBRARY_CARDS: &[&str] = &[
    "Island",
    "Mountain",
    "Brainstorm",
    "Plains",
    "Path to Exile",
    "Llanowar Elves",
    "Dark Ritual",
];

/// Cards whose identity is concealed by being face down (CR 708.2).
const FACE_DOWN_CARDS: &[&str] = &["Exalted Angel", "Saw It Coming"];

/// Assert that none of `needles` appears anywhere in `haystack`.
fn assert_absent(haystack: &str, needles: &[&str], context: &str) {
    for needle in needles {
        assert!(
            !haystack.contains(needle),
            "{context}: leaked {needle:?} into the rendered view:\n{haystack}"
        );
    }
}

/// Serialize a view to a JSON string for a whole-document scan.
fn as_json_string(view: &StateViewModel) -> String {
    serde_json::to_string(view).expect("view model must serialize")
}

/// Resolve a fixture object id by (name, zone).
fn object_id_of(state: &GameState, name: &str, zone: &ZoneId) -> ObjectId {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name && o.zone == *zone)
        .unwrap_or_else(|| panic!("fixture object '{name}' not found in {zone:?}"))
        .id
}

// ── Golden fixture (shared verbatim between the pre-move capture and the
//    post-move regression test) ─────────────────────────────────────────────

/// Build a deterministic, feature-rich `GameState` for the view-model golden
/// snapshot.
///
/// The fixture deliberately exercises every branch of `StateViewModel`:
/// four players with differing life/poison/mana/flags, a battlefield with a
/// tapped countered damaged creature, a commander, a token, an aura-less
/// enchantment and a **face-down** morph permanent, populated hands, libraries,
/// graveyards, an exile pile (one face-up card, one face-down foretold card),
/// a command zone, a targeting spell on the stack, and a combat state with an
/// attacker and a blocker.
///
/// It uses only `mtg-engine`'s public API — the builder plus the documented
/// `*_mut()` escape hatches — so it carries no dependency on the simulator, the
/// card registry, or the on-disk script corpus.
pub fn golden_fixture_state() -> (GameState, HashMap<PlayerId, String>) {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let alesha = CardId("alesha_who_smiles_at_death".to_string());
    let kenrith = CardId("kenrith_the_returned_king".to_string());

    let mut state = GameStateBuilder::new()
        .add_player_with(p1, |b| {
            b.life(37).poison(2).mana(ManaPool {
                white: 1,
                blue: 2,
                black: 0,
                red: 3,
                green: 0,
                colorless: 1,
                ..Default::default()
            })
        })
        .add_player_with(p2, |b| b.life(40).commander(alesha.clone()))
        .add_player_with(p3, |b| b.life(12).poison(9).commander(kenrith.clone()))
        .add_player_with(p4, |b| b.life(1).land_plays(0))
        // ── Battlefield ────────────────────────────────────────────────────
        .object(
            ObjectSpec::creature(p1, "Grizzly Bears", 2, 2)
                .tapped()
                .with_counter(CounterType::PlusOnePlusOne, 2)
                .with_damage(1)
                .with_keyword(KeywordAbility::Trample)
                .with_subtypes(vec![SubType("Bear".to_string())]),
        )
        .object(ObjectSpec::land(p1, "Forest").with_subtypes(vec![SubType("Forest".to_string())]))
        .object(
            ObjectSpec::creature(p2, "Alesha, Who Smiles at Death", 3, 2)
                .with_card_id(alesha.clone())
                .with_keyword(KeywordAbility::FirstStrike)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        // Face-down morph permanent — `status.face_down` is set below (the
        // builder has no setter for it).
        .object(ObjectSpec::creature(p2, "Exalted Angel", 4, 5))
        .object(
            ObjectSpec::creature(p3, "Serra Angel", 4, 4)
                .with_keyword(KeywordAbility::Flying)
                .with_keyword(KeywordAbility::Vigilance),
        )
        .object(ObjectSpec::enchantment(p3, "Ghostly Prison"))
        .object(ObjectSpec::planeswalker(p3, "Teferi, Hero of Dominaria", 4))
        .object(ObjectSpec::creature(p4, "Soldier", 1, 1).token())
        // ── Hands ──────────────────────────────────────────────────────────
        .object(ObjectSpec::card(p1, "Lightning Bolt").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Counterspell").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Swords to Plowshares").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p2, "Wrath of God").in_zone(ZoneId::Hand(p2)))
        .object(ObjectSpec::card(p2, "Sol Ring").in_zone(ZoneId::Hand(p2)))
        .object(ObjectSpec::card(p3, "Demonic Tutor").in_zone(ZoneId::Hand(p3)))
        // ── Libraries (so `library_size` is non-zero and the CR 401.2 test
        //    has something it could have leaked) ─────────────────────────────
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Brainstorm").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p2, "Plains").in_zone(ZoneId::Library(p2)))
        .object(ObjectSpec::card(p2, "Path to Exile").in_zone(ZoneId::Library(p2)))
        .object(ObjectSpec::card(p3, "Llanowar Elves").in_zone(ZoneId::Library(p3)))
        .object(ObjectSpec::card(p4, "Dark Ritual").in_zone(ZoneId::Library(p4)))
        // ── Graveyards ─────────────────────────────────────────────────────
        .object(ObjectSpec::card(p1, "Shock").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "Ponder").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p3, "Doom Blade").in_zone(ZoneId::Graveyard(p3)))
        // ── Exile (one face-up, one face-down/foretold) ────────────────────
        .object(ObjectSpec::card(p1, "Oblivion Ring Target").in_zone(ZoneId::Exile))
        .object(ObjectSpec::card(p2, "Saw It Coming").in_zone(ZoneId::Exile))
        // ── Command zone ───────────────────────────────────────────────────
        .object(
            ObjectSpec::creature(p3, "Kenrith, the Returned King", 5, 5)
                .with_card_id(kenrith.clone())
                .in_zone(ZoneId::Command(p3)),
        )
        .turn_number(7)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .expect("golden fixture must build");

    // ── Post-build mutations the builder cannot express ────────────────────
    // Resolve object ids by (name, zone) so the fixture does not hardcode ids.
    let find = |state: &GameState, name: &str, zone: &ZoneId| -> ObjectId {
        state
            .objects()
            .values()
            .find(|o| o.characteristics.name == name && o.zone == *zone)
            .unwrap_or_else(|| panic!("fixture object '{name}' not found in {zone:?}"))
            .id
    };

    let bears = find(&state, "Grizzly Bears", &ZoneId::Battlefield);
    let angel = find(&state, "Serra Angel", &ZoneId::Battlefield);
    let alesha_obj = find(&state, "Alesha, Who Smiles at Death", &ZoneId::Battlefield);
    let morphed = find(&state, "Exalted Angel", &ZoneId::Battlefield);
    let foretold = find(&state, "Saw It Coming", &ZoneId::Exile);
    let prison = find(&state, "Ghostly Prison", &ZoneId::Battlefield);
    let teferi = find(&state, "Teferi, Hero of Dominaria", &ZoneId::Battlefield);

    // CR 702.37: a face-down morph permanent, controlled by p2.
    if let Some(obj) = state.objects_mut().get_mut(&morphed) {
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Morph);
    }
    // CR 702.143a: a foretold card is exiled face down.
    if let Some(obj) = state.objects_mut().get_mut(&foretold) {
        obj.status.face_down = true;
    }
    // An attached enchantment, so `attached_to` / `attachments` are exercised.
    if let Some(obj) = state.objects_mut().get_mut(&prison) {
        obj.attached_to = Some(angel);
    }
    if let Some(obj) = state.objects_mut().get_mut(&angel) {
        obj.attachments.push_back(prison);
    }
    // CR 104.3a / CR 903.10a: a conceded player and a lost player.
    if let Some(player) = state.players_mut().get_mut(&p4) {
        player.has_conceded = true;
        player.has_lost = true;
    }
    // CR 702.131c: the city's blessing is permanent once gained.
    if let Some(player) = state.players_mut().get_mut(&p3) {
        player.has_citys_blessing = true;
    }
    // CR 903.10a: commander damage received, keyed by opponent then commander.
    if let Some(player) = state.players_mut().get_mut(&p1) {
        player
            .commander_damage_received
            .entry(p2)
            .or_default()
            .insert(alesha.clone(), 6);
    }

    // ── Stack: one targeting spell controlled by p1 ────────────────────────
    let spell_source = find(&state, "Lightning Bolt", &ZoneId::Hand(p1));
    let mut spell = StackObject::trigger_default(
        ObjectId(9_001),
        p1,
        StackObjectKind::Spell {
            source_object: spell_source,
        },
    );
    spell.targets = vec![
        SpellTarget {
            target: Target::Object(angel),
            zone_at_cast: Some(ZoneId::Battlefield),
        },
        SpellTarget {
            target: Target::Player(p2),
            zone_at_cast: None,
        },
    ];
    state.stack_objects_mut().push_back(spell);

    // ── Combat: p1 attacks p3 with Grizzly Bears; Serra Angel blocks ───────
    let mut combat = CombatState::new(p1);
    combat.attackers.insert(bears, AttackTarget::Player(p3));
    // Exercises the `AttackTarget::Planeswalker` arm.
    combat
        .attackers
        .insert(alesha_obj, AttackTarget::Planeswalker(teferi));
    combat.blockers.insert(angel, bears);
    *state.combat_mut() = Some(combat);

    let names: HashMap<PlayerId, String> = [
        (p1, "alice".to_string()),
        (p2, "bob".to_string()),
        (p3, "carol".to_string()),
        (p4, "dave".to_string()),
    ]
    .into_iter()
    .collect();

    (state, names)
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// CR 402.1: "A player's hand is a hidden zone" — no other player may know
/// which cards it holds. Architecture Invariant 7.
#[test]
fn test_seat_view_hides_other_players_hand_card_names() {
    let (state, names) = golden_fixture_state();
    let alice = PlayerId(1);
    let view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(alice));

    // CR 402.2: the NUMBER of cards in a hand is public, so one placeholder is
    // emitted per real card and the counts still line up.
    for (owner, expected_count) in [("bob", 2usize), ("carol", 1), ("dave", 0)] {
        let cards = &view.zones.hand[owner];
        assert_eq!(
            cards.len(),
            expected_count,
            "{owner}'s hand count must survive redaction (CR 402.2)"
        );
        for card in cards {
            assert_eq!(
                card.name, HIDDEN_CARD_NAME,
                "{owner}'s hand must be anonymous"
            );
            assert_eq!(
                card.object_id, 0,
                "a hidden card's id is not alice's to hold"
            );
            assert!(card.card_types.is_empty());
            assert!(card.hidden);
        }
        assert_eq!(
            view.players[owner].hand_size, expected_count,
            "hand_size stays truthful (CR 402.2)"
        );
    }

    // Whole-document scan: no other seat's hand card name anywhere at all.
    assert_absent(
        &as_json_string(&view),
        OTHER_SEAT_HAND_CARDS,
        "alice's seat view",
    );
}

/// CR 401.2: "the cards in a library are kept face down and may not be
/// examined" — a library's contents and order are hidden from EVERY player,
/// including its owner. The view model has no library field at all; this test
/// pins that, so adding one reddens the gate.
#[test]
fn test_seat_view_never_enumerates_any_library() {
    let (state, names) = golden_fixture_state();
    let alice = PlayerId(1);

    let seat_view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(alice));
    let omniscient = StateViewModel::from_game_state_for(&state, &names, Viewer::Omniscient);

    // Sanity: the fixture really does have libraries that COULD have leaked.
    assert_eq!(seat_view.players["alice"].library_size, 3);
    assert_eq!(seat_view.players["bob"].library_size, 2);
    assert_eq!(seat_view.players["carol"].library_size, 1);
    assert_eq!(seat_view.players["dave"].library_size, 1);

    // Not even alice's OWN library may be enumerated (CR 401.2 binds the owner
    // too — knowing the order is the prohibited part).
    assert_absent(
        &as_json_string(&seat_view),
        LIBRARY_CARDS,
        "alice's seat view",
    );
    // And the omniscient developer view does not enumerate one either, so the
    // property is a shape property of the view model, not a redaction.
    assert_absent(
        &as_json_string(&omniscient),
        LIBRARY_CARDS,
        "omniscient view",
    );
}

/// CR 402.1: a player may look at their own hand at any time. The redaction
/// must not overreach.
#[test]
fn test_seat_view_shows_own_hand() {
    let (state, names) = golden_fixture_state();
    let alice = PlayerId(1);
    let view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(alice));

    let own: Vec<&str> = view.zones.hand["alice"]
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(
        own,
        vec!["Lightning Bolt", "Counterspell", "Swords to Plowshares"],
        "alice sees her own hand in zone order"
    );
    for card in &view.zones.hand["alice"] {
        assert!(!card.hidden, "alice's own cards are not placeholders");
        assert_ne!(
            card.object_id, 0,
            "alice keeps real handles on her own cards"
        );
    }
}

/// CR 402.1: that a card was drawn is public; WHICH card was drawn is known
/// only to the drawing player. Architecture Invariant 7 chokepoint #2.
///
/// Also pins the `_ =>` catch-all arm: it renders a kind-only line and cannot
/// interpolate a card name.
#[test]
fn test_event_view_redacts_other_seats_card_draw() {
    let (state, names) = golden_fixture_state();
    let alice = PlayerId(1);
    let bob = PlayerId(2);

    let sol_ring = object_id_of(&state, "Sol Ring", &ZoneId::Hand(bob));
    let drawn = GameEvent::CardDrawn {
        player: bob,
        new_object_id: sol_ring,
    };

    // Alice: the draw happened, the identity did not travel.
    let alice_view = event_view_for(&drawn, &state, &names, Viewer::Seat(alice))
        .expect("a draw is a public event");
    assert_eq!(alice_view.kind, "CardDrawn");
    assert_eq!(alice_view.text, "bob draws a card");
    assert_absent(&alice_view.text, &["Sol Ring"], "alice's event line");

    // Bob: his own draw, named.
    let bob_view = event_view_for(&drawn, &state, &names, Viewer::Seat(bob)).expect("own draw");
    assert_eq!(bob_view.text, "bob draws Sol Ring");

    // Omniscient: the developer tool sees everything.
    let dev_view = event_view_for(&drawn, &state, &names, Viewer::Omniscient).expect("dev view");
    assert_eq!(dev_view.text, "bob draws Sol Ring");

    // The `_ =>` catch-all: kind only. `PermanentTapped` carries an ObjectId
    // naming a face-down creature, and none of it reaches the rendered line.
    let morph = object_id_of(&state, "Exalted Angel", &ZoneId::Battlefield);
    let tapped = GameEvent::PermanentTapped {
        player: bob,
        object_id: morph,
    };
    let catch_all = event_view_for(&tapped, &state, &names, Viewer::Seat(alice))
        .expect("catch-all still renders a line");
    assert_eq!(catch_all.kind, "PermanentTapped");
    assert_eq!(catch_all.text, "PermanentTapped");
    let every_card_name: Vec<&str> = OTHER_SEAT_HAND_CARDS
        .iter()
        .chain(LIBRARY_CARDS.iter())
        .chain(FACE_DOWN_CARDS.iter())
        .copied()
        .collect();
    assert_absent(&catch_all.text, &every_card_name, "catch-all event line");
}

/// The replay-viewer regression guard for the crate move (M11-local S4 item 1).
///
/// `golden_omniscient_view.json` was captured from the PRE-MOVE
/// `tools/replay-viewer/src/view_model.rs`. `Viewer::Omniscient` must remain a
/// pure identity, so the omniscient view of the fixture must still equal it.
///
/// Compared as `serde_json::Value`, never as a string: `StateViewModel` uses
/// `std::collections::HashMap` for `players` and the per-player zone maps, and
/// its iteration order is randomized per process.
#[test]
fn test_omniscient_view_is_unchanged_for_fixture_state() {
    let (state, names) = golden_fixture_state();
    let view = StateViewModel::from_game_state_for(&state, &names, Viewer::Omniscient);

    let actual = serde_json::to_value(&view).expect("view model must serialize");
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("golden_omniscient_view.json"))
            .expect("golden snapshot must parse");

    assert_eq!(
        actual, golden,
        "the omniscient view drifted from the pre-move capture"
    );

    // The shim must be the same thing, so no replay-viewer call site changes.
    let via_shim = StateViewModel::from_game_state(&state, &names);
    assert_eq!(
        serde_json::to_value(&via_shim).expect("view model must serialize"),
        golden,
        "from_game_state must stay an Omniscient shim"
    );
}

/// CR 708.2: a face-down permanent has no name, and only a player entitled to
/// look at it knows which card it is. Same for a card exiled face down
/// (CR 702.143a foretell, and friends).
#[test]
fn test_seat_view_hides_face_down_permanent_name() {
    let (state, names) = golden_fixture_state();
    let alice = PlayerId(1);
    let bob = PlayerId(2);

    let view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(alice));

    // Battlefield: bob's face-down morph.
    let morph = view.zones.battlefield["bob"]
        .iter()
        .find(|p| p.name == FACE_DOWN_NAME)
        .expect("bob's face-down morph must be name-redacted for alice");
    // CR 708.2a: it is a 2/2 with no types, name, or abilities.
    assert_eq!(morph.power, Some(2));
    assert_eq!(morph.toughness, Some(2));
    assert!(morph.keywords.is_empty());

    // Exile: bob's face-down foretold card. This is the real leak in the
    // omniscient view -- `objects_in_zone_as_card_views` reads the raw
    // characteristics with no layer pass.
    let exiled_face_down = view
        .zones
        .exile
        .iter()
        .find(|c| c.name == FACE_DOWN_NAME)
        .expect("bob's face-down exiled card must be name-redacted for alice");
    assert!(exiled_face_down.hidden);
    assert!(exiled_face_down.card_types.is_empty());
    // The face-up exiled card is untouched: exile is a public zone (CR 406.3).
    assert!(view
        .zones
        .exile
        .iter()
        .any(|c| c.name == "Oblivion Ring Target" && !c.hidden));

    assert_absent(&as_json_string(&view), FACE_DOWN_CARDS, "alice's seat view");

    // Ownership, not control: bob owns both, so bob still sees the exiled one.
    // (The battlefield morph reads "" for everyone because the layer system
    // applies the CR 708.2a override before the view model ever sees it -- the
    // redaction above is belt-and-braces, see `redact::redact_face_down_permanents`.)
    let bob_view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(bob));
    assert!(
        bob_view
            .zones
            .exile
            .iter()
            .any(|c| c.name == "Saw It Coming" && !c.hidden),
        "bob owns his own foretold card"
    );
}

/// Regression guard: a spell cast must be named, and `SpellCast` carries two
/// `ObjectId`s of which only one is resolvable.
///
/// `handle_cast_spell` mints `stack_entry_id = state.next_object_id()`
/// (`rules/casting.rs:4401`) purely to build the `StackObject` it pushes onto
/// `state.stack_objects()` (`:4529`); that id is **never** inserted into
/// `state.objects()`. `source_object_id` is the card's new object in
/// `ZoneId::Stack` (`:4732`) and *is* in `state.objects()`.
///
/// Rendering off `stack_object_id` therefore misses every time and silently
/// degrades every cast to the name-free fallback — the name is never wrong, but
/// it is never present either, which would make the Session 6 event feed useless
/// for the most common action in the game. This test fails against that version.
///
/// Casting is public (CR 405.1, CR 601.2), so the name appears for every seat.
#[test]
fn test_event_view_names_a_cast_spell_from_the_source_object() {
    let (state, names) = golden_fixture_state();
    let alice = PlayerId(1);
    let bob = PlayerId(2);

    let bears = object_id_of(&state, "Grizzly Bears", &ZoneId::Battlefield);
    let cast = GameEvent::SpellCast {
        player: alice,
        // The fixture's stack entry id, exactly as the engine mints it: a fresh
        // id that names no entry in `state.objects()`.
        stack_object_id: ObjectId(9_001),
        source_object_id: bears,
    };

    for (viewer, label) in [
        (Viewer::Seat(alice), "the caster"),
        (Viewer::Seat(bob), "an opponent"),
        (Viewer::Omniscient, "the developer tool"),
    ] {
        let view = event_view_for(&cast, &state, &names, viewer).expect("a cast is public");
        assert_eq!(view.kind, "SpellCast");
        assert_eq!(
            view.text, "alice casts Grizzly Bears",
            "{label} must see the spell's name (CR 405.1: the stack is public)"
        );
    }
}

/// Every seat, every hidden card — the gate that would have caught the stack and
/// combat leak the first cut of `redact.rs` shipped.
///
/// The six plan-named tests all view from alice's seat, and alice happens to be
/// the one player whose hand card the fixture also puts on the stack, so her own
/// card names were never needles. That made a whole class of leak invisible to a
/// suite that otherwise scans whole documents: redaction covered `hand`,
/// `battlefield` and `exile` while `zones.stack` and `combat` still rendered a
/// raw `obj.characteristics.name`.
///
/// The fix is not a bigger fixture, it is a symmetric one: for each seat in turn,
/// assert the absence of every card name that seat is not entitled to —
/// every OTHER seat's hand (CR 402.1), every library (CR 401.2), and every
/// face-down card they do not own (CR 708.2).
#[test]
fn test_no_seat_view_leaks_any_other_seats_hidden_card() {
    let (state, names) = golden_fixture_state();

    /// (seat, that seat's own hand, face-down cards that seat owns).
    /// Anything not listed for a seat is a needle for that seat.
    const SEATS: &[(u64, &[&str], &[&str])] = &[
        (
            1,
            &["Lightning Bolt", "Counterspell", "Swords to Plowshares"],
            &[],
        ),
        (
            2,
            &["Wrath of God", "Sol Ring"],
            &["Exalted Angel", "Saw It Coming"],
        ),
        (3, &["Demonic Tutor"], &[]),
        (4, &[], &[]),
    ];

    let all_hand_cards: Vec<&str> = SEATS
        .iter()
        .flat_map(|(_, own, _)| own.iter().copied())
        .collect();

    for (seat_num, own_hand, own_face_down) in SEATS {
        let seat = PlayerId(*seat_num);
        let view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(seat));
        let json = as_json_string(&view);

        // Every other seat's hand card (CR 402.1).
        let foreign_hand: Vec<&str> = all_hand_cards
            .iter()
            .filter(|c| !own_hand.contains(c))
            .copied()
            .collect();
        assert_absent(
            &json,
            &foreign_hand,
            &format!("seat {seat_num}: another hand"),
        );

        // Every library, including this seat's own (CR 401.2).
        assert_absent(&json, LIBRARY_CARDS, &format!("seat {seat_num}: a library"));

        // Every face-down card this seat does not own (CR 708.2).
        let foreign_face_down: Vec<&str> = FACE_DOWN_CARDS
            .iter()
            .filter(|c| !own_face_down.contains(c))
            .copied()
            .collect();
        assert_absent(
            &json,
            &foreign_face_down,
            &format!("seat {seat_num}: a face-down card"),
        );

        // And the seat still sees its own hand, so the scan is not passing by
        // redacting everything.
        for card in *own_hand {
            assert!(
                json.contains(card),
                "seat {seat_num} must still see its own hand card {card:?}"
            );
        }
    }
}

/// The combat and stack surfaces, exercised with an object that is actually
/// face down.
///
/// `golden_fixture_state()` cannot be changed — it is pinned byte-for-byte by
/// `test_omniscient_view_is_unchanged_for_fixture_state` against a snapshot
/// captured before the crate move. Its combat state has only face-UP attackers
/// and blockers, so `redact_combat` would otherwise never be reached by any
/// test: the code would be present, green, and unexercised, which is the same
/// shape of false assurance as an unfalsifiable assertion.
///
/// So this derives a variant: it takes the golden state and puts bob's face-down
/// morph creature into combat as an attacker and as a blocker, and makes it the
/// target of the spell on the stack. CR 708.2 — a face-down creature can attack
/// and block, and doing so does not reveal it.
fn fixture_with_face_down_in_combat() -> (GameState, HashMap<PlayerId, String>) {
    let (mut state, names) = golden_fixture_state();
    let morph = object_id_of(&state, "Exalted Angel", &ZoneId::Battlefield);
    let bears = object_id_of(&state, "Grizzly Bears", &ZoneId::Battlefield);

    if let Some(combat) = state.combat_mut().as_mut() {
        // The face-down creature attacks alice...
        combat
            .attackers
            .insert(morph, AttackTarget::Player(PlayerId(1)));
        // ...and also blocks alice's Grizzly Bears. (Not simultaneously legal in
        // a real game; this is a rendering fixture, and it exercises both
        // surfaces in one pass.)
        combat.blockers.insert(morph, bears);
    }

    // And it is the first target of the spell on the stack.
    if let Some(spell) = state.stack_objects_mut().front_mut() {
        spell.targets[0] = SpellTarget {
            target: Target::Object(morph),
            zone_at_cast: Some(ZoneId::Battlefield),
        };
    }

    (state, names)
}

#[test]
fn test_seat_view_hides_a_face_down_attacker_blocker_and_target() {
    let (state, names) = fixture_with_face_down_in_combat();
    let alice = PlayerId(1);
    let bob = PlayerId(2);

    // Omniscient first: the needle really is present without redaction, so the
    // absence assertions below are not vacuous.
    let dev = StateViewModel::from_game_state_for(&state, &names, Viewer::Omniscient);
    let dev_json = as_json_string(&dev);
    assert!(
        dev_json.contains("Exalted Angel"),
        "the omniscient view must name the face-down creature in combat, or this \
         test asserts the absence of something that was never there:\n{dev_json}"
    );

    // Alice does not own it (bob does), so no surface may name it.
    let alice_json = as_json_string(&StateViewModel::from_game_state_for(
        &state,
        &names,
        Viewer::Seat(alice),
    ));
    assert_absent(
        &alice_json,
        &["Exalted Angel"],
        "alice's seat view (face-down attacker/blocker/target)",
    );

    // ...and the redaction is a substitution, not a deletion: the creature is
    // still visibly there, attacking and blocking. CR 508.1/509.1 make the
    // participation public even when the identity is not.
    let alice_view = StateViewModel::from_game_state_for(&state, &names, Viewer::Seat(alice));
    let combat = alice_view.combat.as_ref().expect("combat is public");
    assert!(
        combat.attackers.iter().any(
            |a| a.name == FACE_DOWN_NAME || a.blockers.iter().any(|b| b.name == FACE_DOWN_NAME)
        ),
        "the face-down creature must still appear in combat, just unnamed"
    );
    assert!(
        alice_view
            .zones
            .stack
            .iter()
            .any(|s| s.targets.iter().any(|t| t.contains(FACE_DOWN_NAME))),
        "the spell must still show that it targets something, just not what"
    );

    // Bob owns it, so bob still sees it (CR 708.5a-adjacent: the owner knows).
    let bob_json = as_json_string(&StateViewModel::from_game_state_for(
        &state,
        &names,
        Viewer::Seat(bob),
    ));
    assert!(
        bob_json.contains("Exalted Angel"),
        "bob owns the face-down creature and must still see it named"
    );
}
