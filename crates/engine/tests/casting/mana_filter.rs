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
/// **SR-34 note (SF-8, HIGH — deliberately NOT fixed here; see
/// `memory/card-authoring/sr34-engine-findings-2026-07-17.md`).** This test — and
/// `test_add_mana_scaled_orphan_fix_all_cards` below — only ever check the *shape* of the
/// registered `ManaAbility` (non-empty, marked with the right colour key). Neither
/// activates the ability and asserts the mana that actually comes out. If they did, they
/// would fail: `handle_tap_for_mana` has no `AddManaScaled` branch and reads
/// `produces: {Green: 1}` literally, so Gaea's Cradle taps for exactly 1 green regardless
/// of creature count — a live HIGH bug this test's shape-only assertion cannot see
/// (SF-8's own report, verbatim: "a data-model test can pin a defect as a requirement",
/// SF-5). Kept as shape-only deliberately (not `#[ignore]`d) because the registration
/// property they check is still real and still worth pinning — `AddManaScaled` remains
/// gated out of the SR-34-widened lowering path for every cost shape except bare
/// `Cost::Tap` (Finding A, `mana_ability_lowering` in `testing/replay_harness.rs`), and a
/// regression there (Cabal Coffers losing its stack-based "right mana, wrong mechanism"
/// fallback) is a different, separately-pinned failure (`primitive_sr34_composite_mana_costs
/// ::composite_cost_add_mana_scaled_stays_on_the_stack`). Fixing SF-8 needs an
/// `EffectAmount` resolution context inside the stackless `TapForMana` path, which
/// `handle_tap_for_mana` does not have — a separate primitive.
fn test_add_mana_scaled_registered_as_mana_ability() {
    let (defs, registry) = build_defs_and_registry();
    let spec = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");

    let land_id = find_by_name(&state, "Gaea's Cradle");
    let obj = state.objects().get(&land_id).unwrap();

    // After PB-34 fix, Gaea's Cradle should have at least one registered ManaAbility.
    // (Pre-fix: AddManaScaled with Cost::Tap was not recognized by try_as_tap_mana_ability
    // and was silently excluded from both mana_abilities AND activated_abilities — never fired.)
    assert!(
        !obj.characteristics.mana_abilities.is_empty(),
        "Gaea's Cradle should have at least one registered ManaAbility after PB-34 fix (AddManaScaled orphan bug)"
    );

    // The registered ManaAbility should be marked as producing green mana (marker; actual count is dynamic).
    let has_green_ability = obj
        .characteristics
        .mana_abilities
        .iter()
        .any(|ma| ma.produces.contains_key(&ManaColor::Green));
    assert!(
        has_green_ability,
        "Gaea's Cradle ManaAbility should be marked as producing green mana"
    );
}

#[test]
/// PB-34: AddManaScaled orphan bug fix covers cards with Cost::Tap + AddManaScaled.
/// These were previously orphaned: not recognized by try_as_tap_mana_ability AND
/// excluded from activated_abilities — the ability was completely silent.
/// After the fix, each should have a registered ManaAbility.
///
/// Shape-only; see the SF-8 note on `test_add_mana_scaled_registered_as_mana_ability`
/// above — it applies here too.
///
/// Note: Cards with Cost::Sequence([Mana, Tap]) + AddManaScaled (Cabal Coffers,
/// Cabal Stronghold, Crypt of Agadeem) stay registered as activated abilities (not mana
/// abilities), but as of SR-34 that is NOT because they have an additional mana cost —
/// Signets have one too and now ARE mana abilities. It is `mana_ability_lowering`'s
/// explicit Finding-A exclusion: `AddManaScaled` is only accepted through bare
/// `Cost::Tap`, any other cost shape is refused so the ability stays on the stack rather
/// than being captured and producing the wrong (fixed) amount. Those three are NOT in
/// this list.
fn test_add_mana_scaled_orphan_fix_all_cards() {
    let scaled_mana_cards = [
        "Elvish Archdruid",
        "Priest of Titania",
        "Marwyn, the Nurturer",
        "Circle of Dreams Druid",
        "Gaea's Cradle",
        "Howlsquad Heavy",
    ];

    let (defs, registry) = build_defs_and_registry();

    for name in &scaled_mana_cards {
        let spec = make_spec(p(1), name, ZoneId::Battlefield, &defs);

        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap_or_else(|e| panic!("state build failed for {}: {:?}", name, e));

        let obj_id = find_by_name(&state, name);
        let obj = state.objects().get(&obj_id).unwrap();

        assert!(
            !obj.characteristics.mana_abilities.is_empty(),
            "{} should have a registered ManaAbility after PB-34 AddManaScaled orphan fix",
            name
        );
    }
}
