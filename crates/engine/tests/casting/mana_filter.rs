//! Tests for PB-34: Filter land mana production (Effect::AddManaFilterChoice) and
//! AddManaScaled orphan bug fix.
//!
//! Filter lands pay a hybrid mana cost plus tap to produce 2 mana from a constrained
//! color pair. Example: "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}."
//!
//! Engine simplification: AddManaFilterChoice produces 1 of color_a + 1 of color_b
//! (the middle option). Interactive full-choice deferred to M10.
//!
//! CR 605.1a — activated mana abilities resolve immediately (no priority window), and
//! `handle_tap_for_mana` never puts anything on the stack (CR 605.3b).
//!
//! **SR-34 update (2026-07-17).** Before SR-34, `enrich_spec_from_def` only lowered a
//! bare `Cost::Tap` activated ability into a `ManaAbility` — the whole reason this
//! module's original note claimed "filter lands use `Cost::Sequence` and go through
//! `ActivateAbility`, which puts them on the stack. Stack resolution yields the same
//! final mana result." **That claim was always the wrong bar** (SF-1 in
//! `memory/card-authoring/sr33-engine-findings-2026-07-17.md`): CR 605.3b is not about
//! the *final mana*, it is about whether an opponent gets a priority window to respond
//! and whether the ability can be activated mid-cast (CR 605.3a) — a filter land on the
//! stack cannot fund a spell the way a Signet or a basic land can. SR-34 widened the
//! lowering gate to any cost payable through `Command::TapForMana` (see
//! `mana_ability_cost_components` in `testing/replay_harness.rs`), which includes a
//! `Cost::Sequence([Mana(hybrid), Tap])` filter ability. **Filter lands are now real
//! mana abilities** and this file's tests activate them via `Command::TapForMana`, not
//! `Command::ActivateAbility`.
//!
//! **What is still NOT fixed, and this file must not claim otherwise (SR-34 §8 item 6):**
//! `ManaPool::can_spend` / `ManaPool::spend` (`card-types/src/state/player.rs`) read only
//! the fixed-color and generic fields of a `ManaCost` — `hybrid` and `phyrexian` are
//! ignored entirely, before and after SR-34. A filter land's `ManaCost { hybrid: [{W/B}],
//! ..Default::default() }` has `mana_value() == 1` (CR 202.3f counts a `ColorColor` hybrid
//! symbol as 1), so `handle_tap_for_mana`'s cost-legality check DOES run `can_spend` —
//! but `can_spend` only ever reads `white`/`blue`/`black`/`red`/`green`/`colorless`/
//! `generic`, every one of which is 0 on a pure-hybrid cost, so it returns `true`
//! unconditionally regardless of pool contents, and `spend()` then deducts nothing.
//! Filter lands genuinely improved (off the stack, usable mid-cast, CR 605.3b) but their
//! printed `{W/B}` cost is paid for free. This is a pre-existing P4 item, unchanged by
//! SR-34.
//! CR 602.2 — activated abilities cost must be paid before the ability resolves.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor, ManaPool, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
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

fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// Build a state with a single filter land on the battlefield for p(1).
fn build_with_filter_land(name: &str) -> GameState {
    let (defs, registry) = build_defs_and_registry();
    let spec = make_spec(p(1), name, ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");

    state.turn_mut().priority_holder = Some(p(1));
    state
}

// ── CR 605.1a / SR-34: Filter land produces 2 mana (1 of each color) ─────────

#[test]
/// CR 605.1a / SR-34 — Fetid Heath: activating the filter ability via `TapForMana`
/// (not `ActivateAbility`) adds {W}{B} to the pool and resolves immediately, no stack
/// (CR 605.3b). Effect::AddManaFilterChoice produces 1 white + 1 black (middle option
/// of 3 choices). Starting with an empty mana pool, activation should yield white:1 +
/// black:1.
/// NOTE: Hybrid mana enforcement is a pre-existing limitation (SR-34 §8 item 6); the
/// hybrid activation cost is structurally correct in the ability definition but not
/// validated or deducted at activation time — see the module doc comment.
fn test_filter_land_produces_two_mana_fetid_heath() {
    let state = build_with_filter_land("Fetid Heath");
    let land_id = find_by_name(&state, "Fetid Heath");

    // Fetid Heath abilities (post-SR-34, both are ManaAbilities, neither is in
    // activated_abilities):
    //   mana_abilities[0]: {T}: Add {C}
    //   mana_abilities[1]: {W/B},{T}: Add {W}{B} (AddManaFilterChoice)
    let (state_resolved, resolve_events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 1,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("filter land activation should succeed (CR 605.1a)");

    // No stack (CR 605.3b): a mana ability resolves immediately.
    assert!(
        state_resolved.stack_objects().is_empty(),
        "a mana ability must not use the stack (CR 605.3b)"
    );

    // After activation: p(1) should have 1 white and 1 black mana added.
    let pool = &state_resolved.players()[&p(1)].mana_pool;
    assert_eq!(
        pool.white, 1,
        "AddManaFilterChoice should add 1 white mana to empty pool"
    );
    assert_eq!(
        pool.black, 1,
        "AddManaFilterChoice should add 1 black mana to empty pool"
    );
    assert_eq!(pool.blue, 0, "no blue mana should be added");
    assert_eq!(pool.red, 0, "no red mana should be added");
    assert_eq!(pool.green, 0, "no green mana should be added");
    assert_eq!(pool.colorless, 0, "no colorless mana should be added");

    // ManaAdded events should have fired for both white and black.
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::White,
                amount: 1,
                ..
            } if *player == p(1)
        )),
        "ManaAdded(White, 1) event should be emitted (CR 605.1a)"
    );
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::Black,
                amount: 1,
                ..
            } if *player == p(1)
        )),
        "ManaAdded(Black, 1) event should be emitted (CR 605.1a)"
    );
}

#[test]
/// CR 118.3 / SR-34 — filter land tap cost: land must be untapped to activate.
/// Tapping an already-tapped filter land (via `TapForMana`, post-SR-34 both of Fetid
/// Heath's abilities are ManaAbilities — see `test_filter_land_produces_two_mana_fetid_heath`)
/// returns `PermanentAlreadyTapped`.
fn test_filter_land_tap_required() {
    let mut state = build_with_filter_land("Fetid Heath");

    // Tap the land manually before trying to activate.
    let land_id = find_by_name(&state, "Fetid Heath");
    state.objects_mut().get_mut(&land_id).unwrap().status.tapped = true;

    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 1, // the filter ability

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    );

    assert!(
        result.is_err(),
        "activating tapped filter land should return an error (CR 118.3)"
    );
}

#[test]
/// CR 605.1a / SR-34: Effect::AddManaFilterChoice is correctly used in filter land card
/// definitions. Verify all 7 filter lands produce exactly 2 mana (1 of each constrained
/// color) by checking the mana pool delta from an empty starting state, activating via
/// `TapForMana` (each filter land's filter ability is `mana_abilities[1]`; index 0 is its
/// plain `{T}: Add {C}` ability — see `test_filter_land_produces_two_mana_fetid_heath`).
fn test_all_filter_lands_produce_correct_colors() {
    // (name, expected_color_a, expected_color_b)
    let filter_lands: &[(&str, ManaColor, ManaColor)] = &[
        ("Fetid Heath", ManaColor::White, ManaColor::Black),
        ("Rugged Prairie", ManaColor::Red, ManaColor::White),
        ("Twilight Mire", ManaColor::Black, ManaColor::Green),
        ("Flooded Grove", ManaColor::Green, ManaColor::Blue),
        ("Cascade Bluffs", ManaColor::Blue, ManaColor::Red),
        ("Sunken Ruins", ManaColor::Blue, ManaColor::Black),
        ("Graven Cairns", ManaColor::Black, ManaColor::Red),
    ];

    for (name, color_a, color_b) in filter_lands {
        let state = build_with_filter_land(name);
        let land_id = find_by_name(&state, name);

        // Capture pool before activation.
        let pool_before = state.players()[&p(1)].mana_pool.clone();

        let (state_resolved, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: land_id,
                ability_index: 1, // the filter ability

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .unwrap_or_else(|e| panic!("activating {} filter ability should succeed: {:?}", name, e));

        let pool_after = &state_resolved.players()[&p(1)].mana_pool;

        // Compute delta (mana added - mana spent; pre-existing hybrid enforcement gap means
        // the hybrid cost is NOT deducted from the pool, so delta is purely the AddManaFilterChoice).
        let delta_a = get_color(pool_after, *color_a) - get_color(&pool_before, *color_a);
        let delta_b = get_color(pool_after, *color_b) - get_color(&pool_before, *color_b);
        let total_added: i32 = [
            ManaColor::White,
            ManaColor::Blue,
            ManaColor::Black,
            ManaColor::Red,
            ManaColor::Green,
            ManaColor::Colorless,
        ]
        .iter()
        .map(|c| get_color(pool_after, *c) as i32 - get_color(&pool_before, *c) as i32)
        .sum();

        assert_eq!(
            delta_a, 1,
            "{}: AddManaFilterChoice should add exactly 1 {:?} mana",
            name, color_a
        );
        assert_eq!(
            delta_b, 1,
            "{}: AddManaFilterChoice should add exactly 1 {:?} mana",
            name, color_b
        );
        assert_eq!(
            total_added, 2,
            "{}: total mana delta should be exactly +2 (AddManaFilterChoice produces 2 mana)",
            name
        );
    }
}

fn get_color(pool: &ManaPool, color: ManaColor) -> u32 {
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

#[test]
/// PB-34: AddManaScaled abilities are now registered as ManaAbilities on objects.
/// Previously, AddManaScaled with Cost::Tap was orphaned — not recognized by
/// try_as_tap_mana_ability and skipped from activated_abilities. After PB-34 fix,
/// Gaea's Cradle should have a registered ManaAbility.
///
/// **SR-36 update (SF-8 fixed, `scutemob-92`).** This test used to check only the
/// *shape* of the registered `ManaAbility` (non-empty, marked with the right colour
/// key) — SF-8's own report named that exact pattern as the defect's cause ("a
/// data-model test can pin a defect as a requirement"). `handle_tap_for_mana` now has a
/// step (6c) that resolves `ManaAbility::scaled_amount` via `resolve_amount`, so this
/// test activates the ability and asserts the real amount: 2 creatures (Gaea's Cradle
/// itself is a land, so both fillers count) must produce 2 green, not the pre-fix
/// marker's constant 1.
fn test_add_mana_scaled_registered_as_mana_ability() {
    let (defs, registry) = build_defs_and_registry();
    let spec = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .object(mtg_engine::ObjectSpec::creature(
            p(1),
            "Filler Bear One",
            2,
            2,
        ))
        .object(mtg_engine::ObjectSpec::creature(
            p(1),
            "Filler Bear Two",
            2,
            2,
        ))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let land_id = find_by_name(&state, "Gaea's Cradle");
    assert_eq!(
        state.objects()[&land_id]
            .characteristics
            .mana_abilities
            .len(),
        1,
        "Gaea's Cradle should have exactly one registered ManaAbility"
    );

    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 0,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("Gaea's Cradle activation should succeed");

    assert_eq!(
        get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Green),
        2,
        "2 creatures controlled must produce 2 green — the pre-fix bug produced exactly 1 \
         regardless of board state (SF-8)"
    );
}

#[test]
/// PB-34: AddManaScaled orphan bug fix covers cards with Cost::Tap + AddManaScaled.
/// These were previously orphaned: not recognized by try_as_tap_mana_ability AND
/// excluded from activated_abilities — the ability was completely silent.
///
/// **SR-36 update (SF-8 fixed, `scutemob-92`).** Activates each card and asserts the
/// real mana it produces — never `!mana_abilities.is_empty()` alone (SF-8's own report:
/// "a data-model test can pin a defect as a requirement"). Each board is built so the
/// expected amount is NOT 1 (except where the printed effect genuinely produces 1 from a
/// single controlled permanent counting itself with no filler) — an "alone on the
/// battlefield" board for any of these cards happens to count exactly 1 of the relevant
/// type (itself), which is indistinguishable from the pre-fix `{colour: 1}` marker; a
/// filler creature per card is the assertion that tells the fix from the bug.
///
/// Note: Cards with Cost::Sequence([Mana, Tap]) + AddManaScaled (Cabal Coffers, Cabal
/// Stronghold, Crypt of Agadeem) are NOT bare `Cost::Tap`, so they are not in this list,
/// which stays bare-`Cost::Tap` cards only. They were excluded from mana-ability lowering
/// by SR-34's Finding-A guard until SF-8 deleted it. All three are activated-and-amount-
/// tested in `primitives/primitive_sr36_scaled_mana_and_life_costs.rs`
/// (`cabal_coffers_is_a_real_mana_ability`, `cabal_stronghold_counts_only_basic_swamps`,
/// `crypt_of_agadeem_counts_only_black_creature_cards_in_graveyard`), which is what their
/// `Partial` -> `Complete` upgrade rests on.
fn test_add_mana_scaled_orphan_fix_all_cards() {
    let (defs, registry) = build_defs_and_registry();

    // (card name, extra battlefield permanents, expected colour, expected amount)
    // Elvish Archdruid: {T}: Add {G} for each Elf you control. Itself (an Elf) + one more
    // Elf filler it controls = 2; a non-Elf filler would not move the count (not tested
    // here — see primitive_sr36_scaled_mana_and_life_costs.rs::elvish_archdruid_counts_only_elves
    // for the filter-liveness proof).
    {
        let spec = make_spec(p(1), "Elvish Archdruid", ZoneId::Battlefield, &defs);
        let filler = mtg_engine::ObjectSpec::creature(p(1), "Extra Elf", 1, 1)
            .with_subtypes(vec![mtg_engine::SubType("Elf".to_string())]);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .object(filler)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build for Elvish Archdruid");
        state.turn_mut().priority_holder = Some(p(1));
        let id = find_by_name(&state, "Elvish Archdruid");
        let (state, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("Elvish Archdruid activation should succeed");
        assert_eq!(
            get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Green),
            2,
            "Elvish Archdruid: itself + one more Elf = 2 green"
        );
    }

    // Priest of Titania: {T}: Add {G} for each Elf ON THE BATTLEFIELD (PlayerTarget::EachPlayer
    // scope — not "you control"). Itself + one Elf controlled by the OPPONENT must still
    // count, proving both the amount resolution and the EachPlayer scope are live.
    {
        let spec = make_spec(p(1), "Priest of Titania", ZoneId::Battlefield, &defs);
        let enemy_elf = mtg_engine::ObjectSpec::creature(p(2), "Enemy Elf", 1, 1)
            .with_subtypes(vec![mtg_engine::SubType("Elf".to_string())]);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .object(enemy_elf)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build for Priest of Titania");
        state.turn_mut().priority_holder = Some(p(1));
        let id = find_by_name(&state, "Priest of Titania");
        let (state, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("Priest of Titania activation should succeed");
        assert_eq!(
            get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Green),
            2,
            "Priest of Titania: itself + an opponent's Elf = 2 green (counts the whole \
             battlefield, not just permanents you control)"
        );
    }

    // Marwyn, the Nurturer: {T}: Add {G} equal to Marwyn's power. Give it a +1/+1 counter
    // so its power (2) differs from its base power (1) and from the pre-fix marker (1).
    {
        let spec = make_spec(p(1), "Marwyn, the Nurturer", ZoneId::Battlefield, &defs)
            .with_counter(mtg_engine::CounterType::PlusOnePlusOne, 1);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build for Marwyn");
        state.turn_mut().priority_holder = Some(p(1));
        let id = find_by_name(&state, "Marwyn, the Nurturer");
        let (state, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("Marwyn activation should succeed");
        assert_eq!(
            get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Green),
            2,
            "Marwyn: power 2 (base 1 + a +1/+1 counter) must produce 2 green"
        );
    }

    // Circle of Dreams Druid: {T}: Add {G} for each creature you control. Itself + one
    // filler creature (any type) = 2.
    {
        let spec = make_spec(p(1), "Circle of Dreams Druid", ZoneId::Battlefield, &defs);
        let filler = mtg_engine::ObjectSpec::creature(p(1), "Filler Bear", 2, 2);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .object(filler)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build for Circle of Dreams Druid");
        state.turn_mut().priority_holder = Some(p(1));
        let id = find_by_name(&state, "Circle of Dreams Druid");
        let (state, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("Circle of Dreams Druid activation should succeed");
        assert_eq!(
            get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Green),
            2,
            "Circle of Dreams Druid: itself + one filler creature = 2 green"
        );
    }

    // Gaea's Cradle: {T}: Add {G} for each creature you control. A land counts nothing
    // of itself, so ZERO creatures on an otherwise-populated (but creature-less) board
    // is the load-bearing case — pre-fix this produced exactly 1 regardless.
    {
        let spec = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build for Gaea's Cradle");
        state.turn_mut().priority_holder = Some(p(1));
        let id = find_by_name(&state, "Gaea's Cradle");
        let (state, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("Gaea's Cradle activation should succeed");
        assert_eq!(
            get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Green),
            0,
            "Gaea's Cradle: 0 creatures controlled must produce 0 green, not the pre-fix \
             marker's constant 1"
        );
    }

    // Howlsquad Heavy: "Max speed — {T}: Add {R} for each Goblin you control" (the Max
    // speed gate itself is a separate, un-actioned KnownWrong defect — see the card's
    // Completeness note; SF-8 only concerns the AMOUNT the ability computes once
    // activated). Itself + one Goblin filler = 2.
    {
        let spec = make_spec(p(1), "Howlsquad Heavy", ZoneId::Battlefield, &defs);
        let filler = mtg_engine::ObjectSpec::creature(p(1), "Filler Goblin", 1, 1)
            .with_subtypes(vec![mtg_engine::SubType("Goblin".to_string())]);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .object(filler)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("state should build for Howlsquad Heavy");
        state.turn_mut().priority_holder = Some(p(1));
        let id = find_by_name(&state, "Howlsquad Heavy");
        let (state, _) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: id,
                ability_index: 0,

                chosen_color: None,
                        hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
},
        )
        .expect("Howlsquad Heavy activation should succeed");
        assert_eq!(
            get_color(&state.player(p(1)).unwrap().mana_pool, ManaColor::Red),
            2,
            "Howlsquad Heavy: itself + one filler Goblin = 2 red"
        );
    }
}
