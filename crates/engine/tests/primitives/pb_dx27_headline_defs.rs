//! PB-DX27 /review fix cycle (scutemob-209): behavioural coverage for
//! `chord_of_calling`, `green_suns_zenith`, and `the_world_tree`.
//!
//! The reviewer's HIGH finding: both `chord_of_calling` and `green_suns_zenith` were
//! promoted to deck-legal completeness markers with their printed "then shuffle"
//! clause UNAUTHORED -- `Effect::SearchLibrary`'s only shuffle is the
//! `shuffle_before_placing` branch (`effects/mod.rs:3839-3844`, the Vampiric-Tutor
//! "shuffle THEN put on top" pattern), which never fires after placement. The
//! reviewer's own diagnosis of *why* it shipped: these three defs had ZERO
//! behavioural coverage anywhere in the tree -- only source-scanning gates ever
//! looked at them. This file closes that hole.
//!
//! By the time this file was written, the `Effect::Shuffle` fix had already been
//! applied to both defs, and `green_suns_zenith` had been demoted back to `partial`
//! (its second clause -- "Shuffle Green Sun's Zenith into its owner's library" -- is
//! a deterministic top-of-library placement per `resolution.rs:2023-2025`, not a
//! real shuffle; `nexus_of_fate` is `partial` for the identical reason).
//!
//! Every test below drives a real `Command` against the real `CardDefinition` in
//! `crates/card-defs/src/defs/`, not a synthetic fixture (SR-34/36).
//!
//! CR rules covered: 601.2c, 608.2d, 701.23a/b/d, 202.3, 614.1a, 613.1d, 613.1f,
//! 605.1a, 400.7.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, process_command, CardDefinition, CardId, CardRegistry,
    Color, Command, GameEvent, GameState, GameStateBuilder, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
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

fn find_object_controlled_by(state: &GameState, name: &str, controller: PlayerId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.controller == controller)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' controlled by {:?} not found", name, controller))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    mtg_engine::enrich_spec_from_def(base, defs)
}

/// Pass priority for all listed players once, then answer any resolution-time
/// blocking decision (search/scry/surveil/discard/trigger-target/cleanup-discard)
/// with the engine's own deterministic default and fold in the replay's events.
/// CR 601.2c/608.2d/701.23a: a search with a stated quality ALWAYS suspends the
/// resolution for a player announcement (even with exactly one candidate), so
/// every test below that resolves a search needs this, not the bare PassPriority
/// loop other X-cost tests use for non-searching spells.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    let (current, pump_events) =
        mtg_engine::testing::replay_harness::auto_answer_blocking_decisions(current);
    all_events.extend(pump_events);
    (current, all_events)
}

/// Cast an X-cost spell from hand, paying `generic` colorless (for X) plus
/// `green` green mana, with the given `x_value`.
fn cast_x_spell(
    state: GameState,
    caster: PlayerId,
    card_id: ObjectId,
    generic: u32,
    green: u32,
    x_value: u32,
) -> (GameState, Vec<GameEvent>) {
    let mut state = state;
    {
        let pool = &mut state.players_mut().get_mut(&caster).unwrap().mana_pool;
        if generic > 0 {
            pool.add(ManaColor::Colorless, generic);
        }
        if green > 0 {
            pool.add(ManaColor::Green, green);
        }
    }
    state.turn_mut().priority_holder = Some(caster);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell(x_value={}) failed: {:?}", x_value, e))
}

// ═══════════════════════════════════════════════════════════════════════════
// Chord of Calling -- {X}{G}{G}{G}, Convoke. Search your library for a creature
// card with mana value X or less, put it onto the battlefield, then shuffle.
// ═══════════════════════════════════════════════════════════════════════════

fn setup_chord_with_cheap_beast() -> (GameState, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let chord = real_card_spec(p1, "Chord of Calling", ZoneId::Hand(p1), &defs);
    // Mana value 2 -- a legal find at X=2. The only creature in the library, so
    // the engine's deterministic default answer (lowest ObjectId among
    // candidates) is unambiguous.
    let cheap_beast = ObjectSpec::creature(p1, "Cheap Beast", 2, 2)
        .with_card_id(CardId("cheap-beast".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));
    // Non-creature filler so the library isn't trivially a single card -- proves
    // the shuffle event is about "the library", not an artifact of a 1-card zone.
    let filler_a = ObjectSpec::card(p1, "Chord Filler A").in_zone(ZoneId::Library(p1));
    let filler_b = ObjectSpec::card(p1, "Chord Filler B").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(chord)
        .object(cheap_beast)
        .object(filler_a)
        .object(filler_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let chord_id = find_object(&state, "Chord of Calling");
    (state, chord_id, p1, p2)
}

/// CR 601.2c/701.23a (PB-DX27 headline probe 1): casting Chord of Calling with
/// X=2 finds and places the MV-2 "Cheap Beast" onto the battlefield.
#[test]
fn t1_chord_of_calling_finds_a_creature_within_the_x_cap() {
    let (state, chord_id, p1, p2) = setup_chord_with_cheap_beast();

    // X=2: {2}{G}{G}{G}.
    let (state, _) = cast_x_spell(state, p1, chord_id, 2, 3, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Cheap Beast"),
        "Chord of Calling X=2 should find and place the MV-2 'Cheap Beast'"
    );
}

/// PB-DX27 /review (HIGH): "then shuffle" is a SEPARATE printed clause and must
/// fire after the search resolves. This is the probe whose absence let the HIGH
/// ship -- it fails red if `Effect::Shuffle` is removed from the card def's
/// `Effect::Sequence` (see revert row R2 in the task report).
#[test]
fn t2_chord_of_calling_shuffles_the_library_after_resolving() {
    let (state, chord_id, p1, p2) = setup_chord_with_cheap_beast();

    let (state, _) = cast_x_spell(state, p1, chord_id, 2, 3, 2);
    let (_state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LibraryShuffled { player } if *player == p1)),
        "Chord of Calling's resolution must emit LibraryShuffled for the controller \
         after the search completes (the printed 'then shuffle' clause); got {:?}",
        events
    );
}

fn setup_chord_with_pricey_beast_only() -> (GameState, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let chord = real_card_spec(p1, "Chord of Calling", ZoneId::Hand(p1), &defs);
    // Mana value 5 -- the ONLY creature in the library, and too expensive for
    // X=2. If the max_cmc_amount cap is real, candidates is empty and nothing
    // is ever offered as a search choice.
    let pricey_beast = ObjectSpec::creature(p1, "Pricey Beast", 5, 5)
        .with_card_id(CardId("pricey-beast".to_string()))
        .with_mana_cost(ManaCost {
            generic: 5,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(chord)
        .object(pricey_beast)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let chord_id = find_object(&state, "Chord of Calling");
    (state, chord_id, p1, p2)
}

/// CR 202.3/608.2h (PB-DX27 headline probe 3): the `max_cmc_amount =
/// EffectAmount::XValue` cap is real. With X=2 and the only creature in the
/// library at mana value 5, the creature is never a legal candidate -- so no
/// `PermanentEnteredBattlefield` ever happens for it and it stays in the
/// library. (The engine reports "no legal card" by never asking a question at
/// all: `candidates.is_empty()` short-circuits straight to `found = None`, so
/// there is no `EffectChoiceQuestion::SearchLibrary` to inspect from a test --
/// the only observable consequence is the absence of the placement.)
#[test]
fn t3_chord_of_calling_respects_the_max_cmc_cap() {
    let (state, chord_id, p1, p2) = setup_chord_with_pricey_beast_only();

    // X=2: {2}{G}{G}{G}. Pricey Beast (MV 5) exceeds the cap.
    let (state, _) = cast_x_spell(state, p1, chord_id, 2, 3, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Pricey Beast"),
        "Chord of Calling X=2 must not find a MV-5 creature -- the 'mana value X \
         or less' cap must exclude it from the candidate set entirely"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Pricey Beast" && o.zone == ZoneId::Library(p1)),
        "the too-expensive creature must remain in the library, never having been \
         a legal search candidate"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Green Sun's Zenith -- {X}{G}, Sorcery. Search your library for a green
// creature card with mana value X or less, put it onto the battlefield, then
// shuffle. Shuffle Green Sun's Zenith into its owner's library.
// ═══════════════════════════════════════════════════════════════════════════

fn setup_gsz_with_green_beastie() -> (GameState, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let gsz = real_card_spec(p1, "Green Sun's Zenith", ZoneId::Hand(p1), &defs);
    let green_beastie = ObjectSpec::creature(p1, "Green Beastie", 2, 2)
        .with_card_id(CardId("green-beastie".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .with_colors(vec![Color::Green])
        .in_zone(ZoneId::Library(p1));
    let filler = ObjectSpec::card(p1, "GSZ Filler").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(gsz)
        .object(green_beastie)
        .object(filler)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let gsz_id = find_object(&state, "Green Sun's Zenith");
    (state, gsz_id, p1, p2)
}

/// CR 202.3/608.2h + CR 601.2c (PB-DX27 headline probe 4a): casting Green Sun's
/// Zenith with X=2 finds the MV-2 green "Green Beastie" and places it, and the
/// library is shuffled after the search resolves (clause 1's "then shuffle").
#[test]
fn t4_green_suns_zenith_finds_green_creature_within_x_and_shuffles() {
    let (state, gsz_id, p1, p2) = setup_gsz_with_green_beastie();

    // X=2: {2}{G}.
    let (state, _) = cast_x_spell(state, p1, gsz_id, 2, 1, 2);
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Green Beastie"),
        "Green Sun's Zenith X=2 should find and place the MV-2 green 'Green Beastie'"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LibraryShuffled { player } if *player == p1)),
        "Green Sun's Zenith's resolution must emit LibraryShuffled for the \
         controller after the search completes (clause 1's 'then shuffle'); got {:?}",
        events
    );
}

/// CR 202.3 (PB-DX27 headline probe 4b): the filter is green-restricted. With
/// X=2 and the only creature in the library being a non-green MV-2 "Red
/// Beastie", it must not be a legal find (candidates is empty because the
/// filter's `colors: Some({Green})` excludes it before the cap is even
/// consulted).
#[test]
fn t4b_green_suns_zenith_excludes_a_non_green_creature_within_the_cap() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let gsz = real_card_spec(p1, "Green Sun's Zenith", ZoneId::Hand(p1), &defs);
    let red_beastie = ObjectSpec::creature(p1, "Red Beastie", 2, 2)
        .with_card_id(CardId("red-beastie".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(gsz)
        .object(red_beastie)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let gsz_id = find_object(&state, "Green Sun's Zenith");

    // X=2: {2}{G}. Red Beastie is within the mana-value cap but is not green.
    let (state, _) = cast_x_spell(state, p1, gsz_id, 2, 1, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Red Beastie"),
        "CR 202.3: 'green creature card' -- a non-green creature within the \
         mana-value cap must not be a legal find for Green Sun's Zenith"
    );
}

/// KNOWN DEVIATION (PB-DX27 /review, CR 614.1a, `resolution.rs:2023-2025`):
/// clause 2 ("Shuffle Green Sun's Zenith into its owner's library") is
/// implemented via `self_shuffle_on_resolution`, which the engine documents
/// in-source as a DETERMINISTIC top-of-library placement, not a real shuffle.
/// Follows the `deviation_animated_nexus_does_not_count_toward_metalcraft`
/// precedent from PB-DX19: this test PINS the wrong-game-state behaviour so a
/// future engine-side fix (real shuffle-in placement) must INVERT this
/// assertion, not silently coexist with it. `nexus_of_fate` is the corpus's
/// only other user of the flag and carries the identical `partial` marker for
/// the same reason.
#[test]
fn deviation_green_suns_zenith_lands_on_top_of_library_instead_of_shuffled_in() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // No creature in the library at all, so clause 1 ("search for a green
    // creature...") is a legal no-op decline (candidates empty, no question
    // asked) and this probe isolates clause 2 in isolation.
    let gsz = real_card_spec(p1, "Green Sun's Zenith", ZoneId::Hand(p1), &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(gsz)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let gsz_id = find_object(&state, "Green Sun's Zenith");

    // X=0: {0}{G}.
    let (state, _) = cast_x_spell(state, p1, gsz_id, 0, 1, 0);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Green Sun's Zenith"
                && o.zone == ZoneId::Graveyard(p1)),
        "Green Sun's Zenith must not end up in the graveyard: \
         self_shuffle_on_resolution replaces that destination (CR 614.1a)"
    );

    let gsz_new_id = state
        .objects()
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "Green Sun's Zenith" && o.zone == ZoneId::Library(p1)
        })
        .map(|(id, _)| *id)
        .expect("Green Sun's Zenith must be in its owner's library after resolving");

    // PINNED DEVIATION: real Magic shuffles it in; this engine puts it on top.
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().top(),
        Some(gsz_new_id),
        "deviation_green_suns_zenith_lands_on_top_of_library_instead_of_shuffled_in \
         -- CR 614.1a says 'shuffle into', but resolution.rs:2023-2025 documents a \
         deterministic top-of-library placement instead; a real fix must invert \
         this assertion"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// The World Tree -- Land (partial). CR 613.1f Layer 6 static: "As long as you
// control six or more lands, lands you control have '{T}: Add one mana of any
// color.'"
// ═══════════════════════════════════════════════════════════════════════════

/// Build p1 a hand with The World Tree plus `filler_lands` plain lands already
/// on the battlefield (no native mana ability of their own, so any mana
/// ability they gain later is unambiguously the grant). The World Tree itself
/// is PLAYED via `Command::PlayLand` (a real Command through the real ETB
/// path, `rules::replacement::register_static_continuous_effects`) rather than
/// placed pre-built, because `GameStateBuilder` never registers
/// `AbilityDefinition::Static` continuous effects for objects placed directly
/// -- only real zone-entry does.
fn setup_world_tree_battlefield(filler_lands: usize) -> (GameState, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let world_tree = real_card_spec(p1, "The World Tree", ZoneId::Hand(p1), &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(world_tree)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    for i in 0..filler_lands {
        builder = builder.object(
            ObjectSpec::land(p1, &format!("Plain Land {}", i)).in_zone(ZoneId::Battlefield),
        );
    }

    let mut state = builder.build().unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let tree_hand_id = find_object(&state, "The World Tree");
    (state, tree_hand_id, p1, p2)
}

/// CR 613.1f/605.1a (PB-DX27 headline probe 6): with 5 filler lands + The World
/// Tree itself = 6 lands controlled, a plain land you control gains "{T}: Add
/// one mana of any color", and that ability actually functions (taps and
/// produces mana), not merely appears in `calculate_characteristics`.
#[test]
fn t6_world_tree_grants_mana_ability_to_lands_with_six_or_more() {
    let (state, tree_hand_id, p1, _p2) = setup_world_tree_battlefield(5);

    let (state, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: tree_hand_id,
        },
    )
    .expect("playing The World Tree should succeed");

    let filler_id = find_object_controlled_by(&state, "Plain Land 0", p1);
    let chars = calculate_characteristics(&state, filler_id).expect("calculate_characteristics");
    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|a| a.any_color && a.requires_tap),
        "CR 613.1f: with 6+ lands controlled, a plain land you control must gain \
         '{{T}}: Add one mana of any color' from The World Tree's Layer 6 grant"
    );

    // Prove it FUNCTIONS, not just that it appears: tap the filler land for mana.
    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: filler_id,
            ability_index: 0,
            chosen_color: Some(ManaColor::Blue),
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("TapForMana via the granted ability should succeed");
    assert!(
        state.objects()[&filler_id].status.tapped,
        "the land should be tapped after activating the granted ability"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaAdded { player, .. } if *player == p1)),
        "ManaAdded event expected from the granted mana ability; got {:?}",
        events
    );
}

/// CR 613.1d (PB-DX27 headline probe 7): with only 5 lands controlled (4
/// filler + The World Tree), the intervening-if condition is FALSE, so no
/// mana ability is granted -- evaluated continuously, not "once it's ever been
/// true".
#[test]
fn t7_world_tree_grants_nothing_with_only_five_lands() {
    let (state, tree_hand_id, p1, _p2) = setup_world_tree_battlefield(4);

    let (state, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: tree_hand_id,
        },
    )
    .expect("playing The World Tree should succeed");

    let filler_id = find_object_controlled_by(&state, "Plain Land 0", p1);
    let chars = calculate_characteristics(&state, filler_id).expect("calculate_characteristics");
    assert!(
        chars.mana_abilities.is_empty(),
        "CR 613.1d: with only 5 lands the 'six or more' intervening-if condition \
         is false, so the plain land must NOT have any mana ability granted by \
         The World Tree"
    );

    let result = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: filler_id,
            ability_index: 0,
            chosen_color: Some(ManaColor::Blue),
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "there is no mana ability at index 0 to activate with only 5 lands \
         controlled; got {:?}",
        result
    );
}

/// CR 605.1a (PB-DX27 headline probe 8): The World Tree's OWN printed "{T}:
/// Add {G}" ability still works and is not displaced by the Layer 6 grant
/// (which is additive, and in any case does not fire below the six-land
/// threshold this single-permanent setup never reaches).
#[test]
fn t8_world_tree_own_tap_ability_still_produces_green() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // Placed directly on the battlefield: the "enters tapped" self-replacement
    // never runs (GameStateBuilder does not replay zone-entry replacements), so
    // it starts untapped -- ObjectSpec's own default.
    let tree = real_card_spec(p1, "The World Tree", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(tree)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let tree_id = find_object(&state, "The World Tree");
    assert!(
        !state.objects()[&tree_id].status.tapped,
        "precondition: The World Tree starts untapped in this setup"
    );

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: tree_id,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("The World Tree's own {T}: Add {G} ability should still activate");

    assert!(
        state.objects()[&tree_id].status.tapped,
        "The World Tree should be tapped after activating its own mana ability"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::Green,
                ..
            } if *player == p1
        )),
        "The World Tree's native tap ability must still add {{G}}, undisplaced by \
         the Layer 6 grant; got {:?}",
        events
    );
}
