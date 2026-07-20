//! Tests for PB-RS2: activated-cost and mana-ability hybrid/Phyrexian pip payment
//! (OOS-RS-2 + OOS-OS8-1).
//!
//! CR 602.2b: "An activated ability's analog to a spell's mana cost (as referenced in
//! rule 601.2f) is its activation cost." Before this PB, `handle_activate_ability`
//! (`rules/abilities.rs`) and `handle_tap_for_mana` (`rules/mana.rs`) both paid the
//! **raw** `ManaCost` — never calling `ManaCost::flatten_hybrid_phyrexian` the way
//! `casting.rs` does for spells — so a pure hybrid/Phyrexian pip (whose `mana_value()`
//! is nonzero per CR 202.3f/202.3g, passing the `> 0` payment gate) was charged as an
//! all-zero cost. `can_spend`/`spend` in `card-types/src/state/player.rs` only read the
//! six standard colors + generic, never `cost.hybrid`/`cost.phyrexian`, so payment
//! silently deducted nothing. This affected 7 shipped filter lands (`{B/R},{T}: Add
//! {B}{B}, {B}{R}, or {R}{R}` — the mana-ability path, `mana.rs`) and any card using a
//! hybrid/Phyrexian pip in a stack-using activated-ability cost (`abilities.rs`).
//!
//! Probes A and B below were written and run BEFORE any engine edit in this PB,
//! asserting `Ok(_)` — the free-pip bug reproducing today. See
//! `memory/primitive-wip.md` for the recorded pre-fix `cargo test` output. Both are
//! now inverted to `Err(InsufficientMana)`-asserting, permanently-kept regression
//! tests (renamed per plan §9.1), and re-run post-fix to confirm both fail loudly:
//! `hybrid_pip_in_activated_cost_requires_mana` (was `probe_...`) and
//! `hybrid_pip_in_mana_ability_cost_requires_mana` (was `probe_...`).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, ActivatedAbility,
    ActivationCost, CardDefinition, CardRegistry, Command, Effect, EffectAmount, GameEvent,
    GameState, GameStateBuilder, GameStateError, HybridMana, HybridManaPayment, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PhyrexianMana, PlayerId, PlayerTarget, Step, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

fn enrich(
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

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

fn pool_amount(state: &GameState, player: PlayerId, color: ManaColor) -> u32 {
    let pool = &state.player(player).expect("player exists").mana_pool;
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

/// A synthetic artifact with a single stack-using activated ability costing exactly one
/// `{B/R}` hybrid pip (no tap, no other component) and a non-mana effect (GainLife —
/// `try_as_tap_mana_ability` in `testing/replay_harness.rs` does not recognize `GainLife`,
/// so this never lowers into a mana ability; it stays on `abilities.rs`'s stack path).
fn hybrid_pip_activated_source(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::artifact(owner, "Test Filter Rock").with_activated_ability(ActivatedAbility {
        cost: ActivationCost {
            mana_cost: Some(ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Black, ManaColor::Red)],
                ..Default::default()
            }),
            ..Default::default()
        },
        effect: Some(Effect::GainLife {
            amount: EffectAmount::Fixed(1),
            player: PlayerTarget::Controller,
        }),
        ..Default::default()
    })
}

// ── §9.1: Step-0 probes, kept as permanent regressions ─────────────────────────

/// CR 107.4e, 602.2b — a `{B/R}`-cost stack-using activated ability, activated with an
/// empty mana pool, must require mana. Was `probe_hybrid_pip_is_currently_free_activated_ability`,
/// which asserted `Ok(_)` pre-fix (see `memory/primitive-wip.md` for the recorded
/// pre-fix run) — inverted post-fix.
#[test]
fn hybrid_pip_in_activated_cost_requires_mana() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");

    let source = find_by_name(&state, "Test Filter Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "CR 107.4e/602.2b: a {{B/R}} activated-ability cost must require mana when the pool \
         is empty (was OOS-RS-2's free-pip bug: this used to be Ok(_) — see \
         memory/primitive-wip.md for the pre-fix probe output): {result:?}"
    );
}

/// CR 107.4e, 605.1a, 602.2b — the mana-ability path (`Command::TapForMana`), the one
/// that covers the 7 shipped filter lands. Was
/// `probe_hybrid_pip_is_currently_free_mana_ability`, which asserted `Ok(_)` with a
/// 2-mana-from-nothing profit pre-fix — inverted post-fix.
#[test]
fn hybrid_pip_in_mana_ability_cost_requires_mana() {
    let (defs, registry) = build_defs_and_registry();
    let land = enrich(p(1), "Graven Cairns", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(land)
        .build()
        .expect("state builds");

    let source = find_by_name(&state, "Graven Cairns");
    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 1, // {B/R},{T}: Add {B}{B}/{B}{R}/{R}{R} — ability 0 is {T}: Add {C}
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "CR 107.4e/605.1a: Graven Cairns's {{B/R}} filter ability must require mana when the \
         pool is empty (was OOS-RS-2's free-pip bug: this used to be Ok(_) with a 2-mana-from- \
         nothing profit — see memory/primitive-wip.md for the pre-fix probe output): {result:?}"
    );
}

// ── §9.2: Hybrid — both halves payable ──────────────────────────────────────────

/// CR 107.4e — a `{B/R}` cost is payable with either color, and only that color.
#[test]
fn hybrid_activated_cost_payable_with_either_half() {
    // Only {B} in pool, choosing Black -> Ok, pool drained.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    let source = find_by_name(&state, "Test Filter Rock");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Black)],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("paying the black half of {B/R} with black mana must succeed");
    assert_eq!(pool_amount(&state, p(1), ManaColor::Black), 0);

    // Only {R} in pool, choosing Red -> Ok.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    let source = find_by_name(&state, "Test Filter Rock");
    process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Red)],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("paying the red half of {B/R} with red mana must succeed");

    // Only {B} in pool, choosing Red -> Err (wrong half).
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    let source = find_by_name(&state, "Test Filter Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Red)],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::InsufficientMana)));
}

/// CR 107.4e ("as represented by the two halves of the symbol") — a hybrid choice
/// naming a color that is neither half of the pip is rejected, not silently defaulted.
#[test]
fn hybrid_choice_must_name_a_component_of_the_pip() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    let source = find_by_name(&state, "Test Filter Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Green)],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "a {{B/R}} pip paid with Green (neither half) must be rejected (CR 107.4e): {result:?}"
    );
}

/// CR 107.4e + the flatten default — empty `hybrid_choices` defaults to the first color
/// of the pip (`{B/R}` -> Black), matching the ~200 migrated call sites' backward
/// compatibility contract.
#[test]
fn hybrid_empty_choices_defaults_to_first_color() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    let source = find_by_name(&state, "Test Filter Rock");
    process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("empty hybrid_choices must default to the first color (Black)");

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(hybrid_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    let source = find_by_name(&state, "Test Filter Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "empty hybrid_choices defaults to Black, so a pool with only Red must fail: {result:?}"
    );
}

/// CR 107.4e ("{2/B} ... either one black mana or two mana of any type"), CR 202.3f — a
/// monocolored hybrid pip is payable as two generic mana.
#[test]
fn monocolored_hybrid_payable_as_two_generic() {
    let source = ObjectSpec::artifact(p(1), "Test Monohybrid Rock").with_activated_ability(
        ActivatedAbility {
            cost: ActivationCost {
                mana_cost: Some(ManaCost {
                    hybrid: vec![HybridMana::GenericColor(ManaColor::Black)],
                    ..Default::default()
                }),
                ..Default::default()
            },
            effect: Some(Effect::GainLife {
                amount: EffectAmount::Fixed(1),
                player: PlayerTarget::Controller,
            }),
            ..Default::default()
        },
    );
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    let source = find_by_name(&state, "Test Monohybrid Rock");
    process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![HybridManaPayment::Generic],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("{2/B} must be payable as 2 generic mana (CR 107.4e/202.3f)");
}

// ── §9.3: Phyrexian — mana vs life ──────────────────────────────────────────────

fn phyrexian_pip_activated_source(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::artifact(owner, "Test Phyrexian Rock").with_activated_ability(ActivatedAbility {
        cost: ActivationCost {
            mana_cost: Some(ManaCost {
                phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
                ..Default::default()
            }),
            ..Default::default()
        },
        effect: Some(Effect::GainLife {
            amount: EffectAmount::Fixed(1),
            player: PlayerTarget::Controller,
        }),
        ..Default::default()
    })
}

/// CR 107.4f — a `{G/P}` cost is payable with green mana, no life lost.
#[test]
fn phyrexian_activated_cost_payable_with_mana() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(phyrexian_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    let life_before = state.player(p(1)).unwrap().life_total;
    let source = find_by_name(&state, "Test Phyrexian Rock");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![false],
        },
    )
    .expect("paying {G/P} with green mana must succeed");
    assert_eq!(pool_amount(&state, p(1), ManaColor::Green), 0);
    assert_eq!(state.player(p(1)).unwrap().life_total, life_before);
}

/// CR 107.4f, 119.4 — a `{G/P}` cost is payable with 2 life and an empty pool. The
/// case this whole seed exists for.
#[test]
fn phyrexian_activated_cost_payable_with_two_life() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(phyrexian_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 20;
    let source = find_by_name(&state, "Test Phyrexian Rock");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect("paying {G/P} with 2 life (at 20 life) must succeed");
    assert_eq!(state.player(p(1)).unwrap().life_total, 18);
}

/// CR 119.4 — Phyrexian life payment requires life_total >= the amount of the
/// payment. At 1 life, paying 2 is illegal. At exactly 2, it is legal (documents the
/// legal-vs-suicidal boundary the simulator must respect separately, §7.1 of the plan).
#[test]
fn phyrexian_life_payment_requires_sufficient_life() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(phyrexian_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 1;
    let source = find_by_name(&state, "Test Phyrexian Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InsufficientLife { .. })),
        "CR 119.4: at 1 life, paying 2 life for {{G/P}} must be rejected: {result:?}"
    );

    // At exactly 2 life: legal (2 >= 2), drops to 0.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(phyrexian_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 2;
    let source = find_by_name(&state, "Test Phyrexian Rock");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect("CR 119.4: at exactly 2 life, paying 2 life is legal (2 >= 2)");
    assert_eq!(state.player(p(1)).unwrap().life_total, 0);
}

/// CR 119.4, 601.2h/602.2b — an ability with an explicit `life_cost` AND a Phyrexian
/// pip paid with life must check the COMBINED total against life_total, not each
/// independently.
#[test]
fn phyrexian_and_explicit_life_cost_check_combined_total() {
    fn combined_life_source(owner: PlayerId) -> ObjectSpec {
        ObjectSpec::artifact(owner, "Test Combined Life Rock").with_activated_ability(
            ActivatedAbility {
                cost: ActivationCost {
                    mana_cost: Some(ManaCost {
                        phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
                        ..Default::default()
                    }),
                    life_cost: 2,
                    ..Default::default()
                },
                effect: Some(Effect::GainLife {
                    amount: EffectAmount::Fixed(1),
                    player: PlayerTarget::Controller,
                }),
                ..Default::default()
            },
        )
    }

    // At 3 life: 2 (life_cost) + 2 (Phyrexian life) = 4 > 3 -> Err.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(combined_life_source(p(1)))
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 3;
    let source_id = find_by_name(&state, "Test Combined Life Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    );
    assert!(
        matches!(result, Err(_)),
        "CR 119.4: 3 life cannot pay a combined 4-life cost: {result:?}"
    );

    // At 4 life: exactly enough -> Ok, life 0.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(combined_life_source(p(1)))
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 4;
    let source_id = find_by_name(&state, "Test Combined Life Rock");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect("4 life can pay a combined 4-life cost");
    assert_eq!(state.player(p(1)).unwrap().life_total, 0);
}

/// CR 107.4f, 202.3g — a PURE `{B/P}` cost (raw `mana_value() == 1`) paid entirely with
/// life, empty pool, must succeed. Proves the flatten sits BEFORE the `mana_value() > 0`
/// gate and the life deduction is a sibling of that gate, not a child of it.
#[test]
fn phyrexian_paid_with_life_skips_the_mana_gate() {
    let source = ObjectSpec::artifact(p(1), "Test Pure Phyrexian Rock").with_activated_ability(
        ActivatedAbility {
            cost: ActivationCost {
                mana_cost: Some(ManaCost {
                    phyrexian: vec![PhyrexianMana::Single(ManaColor::Black)],
                    ..Default::default()
                }),
                ..Default::default()
            },
            effect: Some(Effect::GainLife {
                amount: EffectAmount::Fixed(1),
                player: PlayerTarget::Controller,
            }),
            ..Default::default()
        },
    );
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 20;
    let source_id = find_by_name(&state, "Test Pure Phyrexian Rock");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect("a pure {B/P} paid with life, empty pool, must succeed");
    assert_eq!(state.player(p(1)).unwrap().life_total, 18);
}

// ── §9.4: Filter-land cost regression (the live-wrong roster) ──────────────────

/// CR 107.4e, 605.1a — table-driven over all 7 filter lands: empty pool -> Err;
/// correct half in pool -> Ok, half consumed, filtered mana produced (net delta +1,
/// never +2); wrong half only -> Err.
#[test]
fn filter_land_charges_its_hybrid_pip() {
    let cases: &[(&str, ManaColor, ManaColor)] = &[
        ("Twilight Mire", ManaColor::Black, ManaColor::Green),
        ("Graven Cairns", ManaColor::Black, ManaColor::Red),
        ("Sunken Ruins", ManaColor::Blue, ManaColor::Black),
        ("Flooded Grove", ManaColor::Green, ManaColor::Blue),
        ("Rugged Prairie", ManaColor::Red, ManaColor::White),
        ("Fetid Heath", ManaColor::White, ManaColor::Black),
        ("Cascade Bluffs", ManaColor::Blue, ManaColor::Red),
    ];
    let (defs, registry) = build_defs_and_registry();

    for &(name, half_a, half_b) in cases {
        // Empty pool -> Err.
        let land = enrich(p(1), name, ZoneId::Battlefield, &defs);
        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .object(land)
            .build()
            .unwrap_or_else(|e| panic!("{name}: state builds: {e:?}"));
        let source = find_by_name(&state, name);
        let result = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source,
                ability_index: 1,
                chosen_color: None,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        );
        assert!(
            matches!(result, Err(GameStateError::InsufficientMana)),
            "{name}: empty pool must reject the filter ability: {result:?}"
        );

        // Correct half (half_a) in pool -> Ok, half consumed, +1 net mana.
        let land = enrich(p(1), name, ZoneId::Battlefield, &defs);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .object(land)
            .build()
            .unwrap_or_else(|e| panic!("{name}: state builds: {e:?}"));
        state
            .players_mut()
            .get_mut(&p(1))
            .unwrap()
            .mana_pool
            .add(half_a, 1);
        let pool_before = state.player(p(1)).unwrap().mana_pool.total();
        let source = find_by_name(&state, name);
        let (state, _events) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source,
                ability_index: 1,
                chosen_color: None,
                hybrid_choices: vec![HybridManaPayment::Color(half_a)],
                phyrexian_life_payments: vec![],
            },
        )
        .unwrap_or_else(|e| panic!("{name}: paying the correct half must succeed: {e:?}"));
        let pool_after = state.player(p(1)).unwrap().mana_pool.total();
        // The filter effect produces 1 of each half (color_a + color_b, CR 605.1a's
        // simplified middle option), and half_a is both the paid pip AND one of the two
        // produced colors — so the paid pip's color nets back to 1 (spent, then
        // reproduced), not 0. The real invariant is the net TOTAL delta below: +1
        // (2 produced - 1 paid), never +2 (the free-pip bug's signature).
        assert_eq!(
            pool_amount(&state, p(1), half_a),
            1,
            "{name}: half_a nets to 1 (1 spent, then 1 reproduced by the filter effect)"
        );
        assert_eq!(
            pool_amount(&state, p(1), half_b),
            1,
            "{name}: half_b (never paid) must be produced once by the filter effect"
        );
        assert_eq!(
            pool_after as i64 - pool_before as i64,
            1,
            "{name}: net mana delta must be exactly +1 (2 produced - 1 paid), never +2"
        );

        // Wrong half only (half_b) -> Err.
        let land = enrich(p(1), name, ZoneId::Battlefield, &defs);
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .object(land)
            .build()
            .unwrap_or_else(|e| panic!("{name}: state builds: {e:?}"));
        state
            .players_mut()
            .get_mut(&p(1))
            .unwrap()
            .mana_pool
            .add(half_b, 1);
        let source = find_by_name(&state, name);
        let result = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source,
                ability_index: 1,
                chosen_color: None,
                hybrid_choices: vec![HybridManaPayment::Color(half_a)],
                phyrexian_life_payments: vec![],
            },
        );
        assert!(
            matches!(result, Err(GameStateError::InsufficientMana)),
            "{name}: only the wrong half in pool must reject: {result:?}"
        );
    }
}

/// CR 107.4f, 602.2b — Birthing Pod's `{1}{G/P}` activation cost: mana-only case must
/// reject an empty pool. Only runs if Birthing Pod is `Complete` (skips honestly rather
/// than failing if the card def wasn't flipped this PB — see `memory/primitive-wip.md`
/// for the disposition).
#[test]
fn birthing_pod_activation_charges_the_phyrexian_pip() {
    let (defs, _registry) = build_defs_and_registry();
    let Some(def) = defs.get("Birthing Pod") else {
        panic!("Birthing Pod card def not found");
    };
    if !def.completeness.is_complete() {
        eprintln!(
            "birthing_pod_activation_charges_the_phyrexian_pip: Birthing Pod is not Complete \
             (still {:?}) — see memory/primitive-wip.md's honest remaining-blocker note; \
             skipping rather than failing.",
            def.completeness
        );
        return;
    }
    // If Birthing Pod IS Complete, its activated ability must be at index 0 and must
    // charge its {1}{G/P} cost correctly. This assertion is intentionally strict: a
    // Complete Birthing Pod that doesn't pay real mana would be exactly the
    // "legal-but-wrong" hazard `project_legal_but_wrong_gap` warns about.
    let (defs, registry) = build_defs_and_registry();
    let pod = enrich(p(1), "Birthing Pod", ZoneId::Battlefield, &defs);
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry.clone())
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(pod)
        .object(ObjectSpec::creature(p(1), "Fodder Bear", 2, 2))
        .build()
        .expect("state builds");
    let source = find_by_name(&state, "Birthing Pod");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![false],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "Birthing Pod's {{1}}{{G/P}} cost with an empty pool and mana payment chosen must \
         reject: {result:?}"
    );

    // Positive path — {1}{G} in pool, [false] (pay mana), a real creature to sacrifice:
    // activation (cost payment + stack push) must succeed.
    let pod = enrich(p(1), "Birthing Pod", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry.clone())
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(pod)
        .object(ObjectSpec::creature(p(1), "Fodder Bear", 2, 2))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let source = find_by_name(&state, "Birthing Pod");
    let bear = find_by_name(&state, "Fodder Bear");
    process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(bear),
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![false],
        },
    )
    .expect("{1}{G} paid with mana, sacrificing a real creature, must activate");

    // Positive path — {1} only in pool, [true] (pay the Phyrexian pip with life): must
    // succeed and deduct exactly 2 life (CR 107.4f).
    let pod = enrich(p(1), "Birthing Pod", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry.clone())
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(pod)
        .object(ObjectSpec::creature(p(1), "Fodder Bear", 2, 2))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let life_before = state.player(p(1)).unwrap().life_total;
    let source = find_by_name(&state, "Birthing Pod");
    let bear = find_by_name(&state, "Fodder Bear");
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(bear),
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect("{1} paid with mana + {G/P} paid with 2 life must activate");
    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        life_before - 2,
        "paying the Phyrexian pip with life must deduct exactly 2 life"
    );

    // Negative path — {1} only in pool, [false] (pay mana): must reject (no green
    // available for the Phyrexian pip's mana option).
    let pod = enrich(p(1), "Birthing Pod", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(pod)
        .object(ObjectSpec::creature(p(1), "Fodder Bear", 2, 2))
        .build()
        .expect("state builds");
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let source = find_by_name(&state, "Birthing Pod");
    let bear = find_by_name(&state, "Fodder Bear");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(bear),
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![false],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "{{1}} only, paying the Phyrexian pip with mana, must reject (no green available): \
         {result:?}"
    );
}

/// Sentinel: `GameEvent::LifeLost` must be emitted for a Phyrexian pip paid with life,
/// mirroring `casting.rs`'s cast-side event shape.
#[test]
fn phyrexian_life_payment_emits_life_lost_event() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(phyrexian_pip_activated_source(p(1)))
        .build()
        .expect("state builds");
    state.players_mut().get_mut(&p(1)).unwrap().life_total = 20;
    let source = find_by_name(&state, "Test Phyrexian Rock");
    let (_state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect("paying with life must succeed");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 2 } if *player == p(1))),
        "a Phyrexian pip paid with 2 life must emit LifeLost {{ amount: 2 }}"
    );
}

// ── §9.5: Residue-guard integration sentinel ────────────────────────────────────

/// The residue guard (`ManaPool::can_spend`/`spend`'s `debug_assert_flattened`) has
/// its own dedicated test in `crates/card-types/src/state/player.rs`'s `#[cfg(test)]`
/// module (§6.4 of the plan: an engine integration test cannot reach an unflattened
/// cost once this PB's payment-path fix lands, since every real call site flattens
/// first). This is a placeholder documenting that fact so a reader of this file finds
/// the pointer rather than a missing test.
#[test]
fn residue_guard_test_lives_in_card_types_player_rs() {
    // See crates/card-types/src/state/player.rs: unflattened_hybrid_cost_panics_in_debug,
    // unflattened_phyrexian_cost_panics_in_debug, flattened_cost_does_not_panic.
}
