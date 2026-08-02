//! PB-DX6 stage 0 — pre-fix OBSERVATION probes only (OOS-RS2-1 + OOS-DP4-1).
//!
//! `memory/primitives/pb-plan-DX6.md` §0/§2/§2.0 is authoritative. This file makes
//! **no engine change**; every probe below runs against the unmodified tree and
//! records a number that was *observed by execution*, never reasoned to (plan §2's
//! standing discipline — the most-cited failure of this suite's last four batches).
//!
//! Two unflattened mana-cost payment sites remain in the engine:
//! - `rules/engine.rs::handle_turn_face_up` pays a raw `def.mana_cost` — CR 701.40b's
//!   "pay that cost" ignores CR 107.4e/107.4f hybrid/Phyrexian pips entirely.
//! - `rules/combat.rs::handle_declare_attackers`'s CR 508.1h attack-tax total has no
//!   payment-choice channel on `Command::DeclareAttackers`, so a pipped or X tax is
//!   rejected outright rather than made payable.
//!
//! ## The build-mode trap (plan §2.0) — read before touching Observation A or B
//!
//! `ManaPool::can_spend`'s first statement is `debug_assert_flattened(cost)`
//! (`crates/card-types/src/state/player.rs`). In a **debug** build (every `cargo
//! test` run, all of CI) an unflattened cost reaching this site **panics**. The
//! "flips for `{1}`, both hybrid pips free" figure is a **RELEASE-only** claim —
//! `debug_assert!` compiles to nothing in release, so the guard never fires and the
//! payment silently proceeds, undercharging the player. The two behaviours must be
//! observed separately and neither substitutes for the other:
//!
//! - Observation A (below) is the debug panic, run exactly as CI runs it. This is a
//!   real, always-green regression pin: it is the reason the bug survived — *every*
//!   test build the project has ever run would have caught it, and no test before
//!   this batch ever put a pipped cost through this site.
//! - Observation B (below) is the release figure. It cannot be produced by a normal
//!   `cargo test` run without disabling the guard, so it is `#[ignore]`d and is not
//!   part of the standing suite; see its doc comment for the exact manual procedure
//!   and the recorded result.
//!
//! The attack-tax path (Observations C/D) does **not** have this trap:
//! `rules/combat.rs` rejects a pipped or X tax with a real `Err` *before* reaching
//! `can_pay_cost`, in every build, so its pre-fix observation is just the
//! `InvalidCommand` message, quoted verbatim.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::stubs::ActiveRestriction;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardId, CardRegistry, CardType, Command, FaceDownKind, GameEvent,
    GameRestriction, GameState, GameStateBuilder, GameStateError, HybridMana, HybridManaPayment,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PhyrexianMana, PlayerId, Step, TurnFaceUpMethod,
    TypeLine, ZoneId,
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

fn add_restriction(
    state: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    restriction: GameRestriction,
) {
    state.restrictions_mut().push_back(ActiveRestriction {
        source,
        controller,
        restriction,
    });
}

fn declare_cmd(player: PlayerId, attackers: Vec<(ObjectId, AttackTarget)>) -> Command {
    Command::DeclareAttackers {
        player,
        attackers,
        enlist_choices: vec![],
        exert_choices: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

/// Build a manifested (`FaceDownKind::Manifest`), face-down battlefield object of the
/// given REAL corpus creature `name`, controlled by P1, P1 holding priority. `defs`/
/// `registry` let callers extend `all_cards()` with a synthetic def (T4).
fn manifest_state_with(
    defs: &HashMap<String, CardDefinition>,
    registry: Arc<CardRegistry>,
    name: &str,
    life_total: i32,
) -> (GameState, ObjectId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let spec = enrich(p1, name, ZoneId::Battlefield, defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let id = find_by_name(&state, name);
    if let Some(obj) = state.objects_mut().get_mut(&id) {
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Manifest);
    }
    state.turn_mut().priority_holder = Some(p1);
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.life_total = life_total;
    }
    (state, id, p1)
}

/// Convenience wrapper of [`manifest_state_with`] for a REAL corpus card (T1/T2):
/// builds its own `all_cards()`-derived registry (matching this file's pre-existing
/// Observation A/B style). T1/T2 never assert on life, so an arbitrary high life
/// total (well clear of any accidental CR 704.5a boundary) is used rather than
/// depending on the builder's own default.
fn manifest_state(name: &str) -> (GameState, ObjectId, PlayerId) {
    let (defs, registry) = build_defs_and_registry();
    manifest_state_with(&defs, registry, name, 40)
}

/// Set a player's unrestricted mana pool to exactly these six fields (all others left
/// at their existing values, which is 0 for a freshly-built state).
#[allow(clippy::too_many_arguments)]
fn set_pool(
    state: &mut GameState,
    player: PlayerId,
    white: u32,
    blue: u32,
    black: u32,
    red: u32,
    green: u32,
    colorless: u32,
) {
    if let Some(ps) = state.players_mut().get_mut(&player) {
        ps.mana_pool.white = white;
        ps.mana_pool.blue = blue;
        ps.mana_pool.black = black;
        ps.mana_pool.red = red;
        ps.mana_pool.green = green;
        ps.mana_pool.colorless = colorless;
    }
}

/// Synthetic creature def for T4: `{generic}{G/P}` -- a single Phyrexian pip (CR
/// 107.4f), plain or with a generic component. No shipped card on either PB-DX6
/// roster carries a Phyrexian pip in a printed `mana_cost` reachable by
/// Manifest/Cloak (plan §0 roster: all 5 hybrid-only), so this fixture is
/// necessary, not a substitute for a real corpus card.
fn phyrexian_manifest_def(card_id_str: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id_str.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        oracle_text: "PB-DX6 T4 test fixture only.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Observation A (HISTORICAL) — turn-face-up used to panic in DEBUG (plan §2.0) ─

#[test]
/// HISTORICAL / POST-FIX regression pin. Before PB-DX6 stage B, THIS EXACT SCENARIO
/// — Kitchen Finks manifested, empty pool, `TurnFaceUp { method: ManaCost,
/// hybrid_choices: vec![], .. }` — panicked inside `debug_assert_flattened` in every
/// debug build (`cargo test`, all of CI), because `handle_turn_face_up`
/// (`rules/engine.rs`) called `can_spend`/`spend` on the raw, unflattened
/// `{1}{G/W}{G/W}` cost. The PRE-FIX panic text is preserved VERBATIM below as a
/// permanent record (this batch's own plan, at the stage-B dispatch, requires this
/// text not be deleted, only re-expressed) — it is the reason this bug survived
/// every test build the project has ever run, and no test before this batch ever put
/// a pipped cost through this site:
///
/// PRE-FIX (recorded 2026-08-02, debug build, unmodified tree, `catch_unwind`
/// downcast to `String`):
/// "unflattened mana cost reached the payment path: 2 hybrid + 0 Phyrexian pip(s) \
///  would be paid for free (CR 107.4e/107.4f). Call ManaCost::flatten_hybrid_phyrexian \
///  first. cost = ManaCost { white: 0, blue: 0, black: 0, red: 0, green: 0, colorless: 0, \
///  generic: 1, hybrid: [ColorColor(Green, White), ColorColor(Green, White)], \
///  phyrexian: [], x_count: 0 }"
///
/// POST-FIX (asserted live below): `handle_turn_face_up` now flattens the cost
/// UNCONDITIONALLY (whenever `def.mana_cost` itself carries a hybrid/Phyrexian pip,
/// independent of whether `hybrid_choices` was supplied) before it ever reaches
/// `can_spend` (CR 107.4e). The identical command against the identical empty pool
/// therefore no longer panics at all — it returns a real, CR-legal
/// `Err(InvalidCommand)` for ordinary insufficient-mana reasons (the flattened cost's
/// `{1}` generic plus two now-priced `{G/W}` pips are unaffordable from an empty
/// pool). This is T1's "empty pool -> Err" case, restated standalone against the
/// exact pre-fix panic scenario so the panic's disappearance is pinned by name, not
/// only incidentally covered by T1.
fn historical_observation_a_no_longer_panics_post_fix() {
    let (state, finks_id, p1) = manifest_state("Kitchen Finks");

    let result = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: finks_id,
            method: TurnFaceUpMethod::ManaCost,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    let err = result.expect_err(
        "POST-FIX: an unaffordable pipped turn-face-up cost must be REJECTED, not \
         panic. If this observes a panic, the flatten-before-payment fix has \
         regressed and this historical record is no longer accurate.",
    );
    let msg = format!("{err:?}");
    assert!(
        matches!(err, GameStateError::InvalidCommand(_)),
        "expected InvalidCommand (the existing affordability message), got: {msg}"
    );
    assert!(
        msg.contains("TurnFaceUp: player cannot pay the turn-face-up cost"),
        "expected the existing, unchanged affordability string (plan §5.1: keep it \
         for the mana-insufficiency case): {msg}"
    );
}

// ── Observation B — turn-face-up, RELEASE figure (plan §2.0, manual procedure) ──

#[test]
#[ignore = "PB-DX6 stage-0 manual observation only (plan §2.0's preferred cheap \
            route). Reproduce by temporarily commenting out the \
            `debug_assert_flattened(cost);` line at the top of ManaPool::can_spend in \
            crates/card-types/src/state/player.rs, then run: \
            `cargo test -p mtg-engine --test primitives observation_b -- --ignored \
            --nocapture`. Restore the commented-out line immediately after and confirm \
            `git diff -- crates/card-types/src/state/player.rs` is empty. With the \
            guard PRESENT (the tree's normal state) this test panics rather than \
            failing an assertion, which is why it is #[ignore]d rather than left in \
            the standing suite."]
/// PRE-FIX / RELEASE-equivalent observation (plan §2.0). Commenting out the
/// `debug_assert!` call has the identical effect, for this call, to a release build
/// (`debug_assert!` compiles to nothing in release) without paying for a full
/// `--release` recompile — the "confirmatory, expensive" alternative the plan names
/// is `cargo test --release`, not used here.
///
/// Same fixture as Observation A, but the attacking player's pool is seeded
/// `{1}{G}{W}` (colorless: 1, green: 1, white: 1) before the flip so the debit can be
/// read back per field. Kitchen Finks' printed cost is `{1}{G/W}{G/W}` — `generic: 1`,
/// `hybrid: [ColorColor(Green, White), ColorColor(Green, White)]`, all six explicit
/// colour/colourless fields 0. `can_spend`'s per-colour loop therefore requires
/// nothing (all six declared fields are 0) and only the `generic: 1` requirement is
/// checked/paid; `spend`'s generic loop takes from colourless first
/// (`crates/card-types/src/state/player.rs::spend`'s documented order: colourless,
/// green, red, black, blue, white).
///
/// OBSERVED (recorded 2026-08-02, guard commented out, single manual run — see the
/// procedure in this test's `#[ignore]` reason; NOT reproduced by the standing
/// suite):
/// - `can_spend(&kitchen_finks_cost, None)` returned `true` (both hybrid pips are
///   invisible to the check — no colour requirement is ever raised for them).
/// - `process_command(..., TurnFaceUp { method: ManaCost, .. })` returned `Ok(_)`.
/// - Pool before: `{ colorless: 1, green: 1, white: 1 }` (all other fields 0).
/// - Pool after:  `{ colorless: 0, green: 1, white: 1 }` (all other fields 0) — only
///   the `{1}` generic component was charged, taken from colourless; **both `{G/W}`
///   pips were paid for free**, exactly as OOS-RS2-1 describes.
fn observation_b_release_figure_pool_debit_kitchen_finks() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();

    let spec = enrich(p1, "Kitchen Finks", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let finks_id = find_by_name(&state, "Kitchen Finks");
    if let Some(obj) = state.objects_mut().get_mut(&finks_id) {
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Manifest);
    }
    state.turn_mut().priority_holder = Some(p1);
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool.colorless = 1;
        ps.mana_pool.green = 1;
        ps.mana_pool.white = 1;
    }

    let pool_before = state.player(p1).expect("p1 exists").mana_pool.clone();

    let (state, _events) = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: finks_id,
            method: TurnFaceUpMethod::ManaCost,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "PRE-FIX release/guard-disabled expectation: with debug_assert_flattened \
         disabled, the raw unflattened cost's generic:1 component is affordable from \
         the seeded pool and the flip must succeed, undercharging both hybrid pips.",
    );

    let pool_after = &state.player(p1).expect("p1 exists").mana_pool;
    assert_eq!(
        pool_after.colorless, 0,
        "the {{1}} generic component must be charged from colourless first (spend's \
         documented order): before={:?} after.colorless={}",
        pool_before, pool_after.colorless
    );
    assert_eq!(
        pool_after.green, 1,
        "both {{G/W}} pips are paid for free pre-fix: green must be UNTOUCHED: {:?}",
        pool_after
    );
    assert_eq!(
        pool_after.white, 1,
        "both {{G/W}} pips are paid for free pre-fix: white must be UNTOUCHED: {:?}",
        pool_after
    );
}

// ── T1 — Kitchen Finks, both hybrid pips independently chargeable (plan §2.1) ───

#[test]
/// CR 701.40b, 107.4e — `handle_turn_face_up` now flattens `{1}{G/W}{G/W}` before
/// payment (plan §5.1), so each `{G/W}` pip is independently chargeable: "A hybrid
/// symbol such as {W/U} can be paid with either white or blue mana... A hybrid mana
/// symbol is all of its component colors" (CR 107.4e).
fn manifested_kitchen_finks_flip_charges_both_hybrid_pips() {
    // Case 1: empty pool -> Err (unaffordable; the existing, unchanged message).
    {
        let (state, finks_id, p1) = manifest_state("Kitchen Finks");
        let err = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: finks_id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        )
        .expect_err("empty pool cannot pay {1}{G/W}{G/W}");
        let msg = format!("{err:?}");
        assert!(
            matches!(err, GameStateError::InvalidCommand(_))
                && msg.contains("TurnFaceUp: player cannot pay the turn-face-up cost"),
            "expected the existing affordability message, got: {msg}"
        );
    }

    // Case 2: pool {1}{G}{G}, explicit [Green, Green] -> Ok, pool empty after.
    {
        let (mut state, finks_id, p1) = manifest_state("Kitchen Finks");
        set_pool(&mut state, p1, 0, 0, 0, 0, 2, 1);
        let (state, events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: finks_id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![
                    HybridManaPayment::Color(ManaColor::Green),
                    HybridManaPayment::Color(ManaColor::Green),
                ],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("pool {1}{G}{G} pays {1}{G/W}{G/W} with both pips chosen Green");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be fully drained: {pool:?}");
        // Architecture Invariant 4 repair: a debit now emits ManaCostPaid with the
        // ORIGINAL, pipped cost (plan §5.1), not the flattened one.
        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::ManaCostPaid { player: pl, cost }
                    if *pl == p1 && !cost.hybrid.is_empty()
            )),
            "ManaCostPaid must be emitted carrying the unflattened (pipped) cost: {events:?}"
        );
    }

    // Case 3: pool {1}{G}{W}, [Green, White] -> Ok, pool empty (each pip is chosen
    // INDEPENDENTLY, CR 107.4e).
    {
        let (mut state, finks_id, p1) = manifest_state("Kitchen Finks");
        set_pool(&mut state, p1, 1, 0, 0, 0, 1, 1);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: finks_id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![
                    HybridManaPayment::Color(ManaColor::Green),
                    HybridManaPayment::Color(ManaColor::White),
                ],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("each {G/W} pip is chosen independently (CR 107.4e): Green + White");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be fully drained: {pool:?}");
    }

    // Case 4: pool {1}{G}{G}, [Blue, Green] -> Err naming CR 107.4e (Blue is not a
    // component color of either {G/W} pip).
    {
        let (mut state, finks_id, p1) = manifest_state("Kitchen Finks");
        set_pool(&mut state, p1, 0, 0, 0, 0, 2, 1);
        let err = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: finks_id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![
                    HybridManaPayment::Color(ManaColor::Blue),
                    HybridManaPayment::Color(ManaColor::Green),
                ],
                phyrexian_life_payments: vec![],
            },
        )
        .expect_err("Blue is not a valid component of {G/W}");
        let msg = format!("{err:?}");
        assert!(
            matches!(err, GameStateError::InvalidCommand(_)) && msg.contains("CR 107.4e"),
            "message must be InvalidCommand citing CR 107.4e: {msg}"
        );
    }

    // Case 5: pool {1}{G}{G}, hybrid_choices: [] -> Ok (documented default: first
    // color of each pip, i.e. Green — ManaCost::flatten_hybrid_phyrexian's doc).
    {
        let (mut state, finks_id, p1) = manifest_state("Kitchen Finks");
        set_pool(&mut state, p1, 0, 0, 0, 0, 2, 1);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: finks_id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("empty hybrid_choices defaults each pip to its first color (Green)");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(
            pool.total(),
            0,
            "default-Green payment must drain the {{G}}{{G}} pool: {pool:?}"
        );
    }
}

// ── T2 — Blade Historian + Boggart Ram-Gang, same fix on the whole live roster ──

#[test]
/// CR 701.40b, 107.4e — proves the fix on the OTHER two live-wrong `Complete`
/// roster members (plan §0: the roster is 3, not the dispatch brief's 1 — Blade
/// Historian is `Complete` only by the `#[default]` derive), table-driven so the
/// fix is proven on the whole live roster, not on Kitchen Finks alone.
fn manifested_blade_historian_and_boggart_ram_gang() {
    // Blade Historian: {R/W}{R/W}{R/W}{R/W}, generic 0 — all four pips Red.
    {
        let (mut state, id, p1) = manifest_state("Blade Historian");
        set_pool(&mut state, p1, 0, 0, 0, 4, 0, 0);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Red); 4],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("4 x {R/W} paid entirely with Red");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be drained: {pool:?}");
    }

    // Blade Historian mixed: 2 Red + 2 White (each pip independent, CR 107.4e).
    {
        let (mut state, id, p1) = manifest_state("Blade Historian");
        set_pool(&mut state, p1, 2, 0, 0, 2, 0, 0);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![
                    HybridManaPayment::Color(ManaColor::Red),
                    HybridManaPayment::Color(ManaColor::Red),
                    HybridManaPayment::Color(ManaColor::White),
                    HybridManaPayment::Color(ManaColor::White),
                ],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("2 x {R/W} paid Red, 2 x {R/W} paid White independently (CR 107.4e)");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be drained: {pool:?}");
    }

    // Boggart Ram-Gang: {R/G}{R/G}{R/G}, generic 0 — all three pips Red.
    {
        let (mut state, id, p1) = manifest_state("Boggart Ram-Gang");
        set_pool(&mut state, p1, 0, 0, 0, 3, 0, 0);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Red); 3],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("3 x {R/G} paid entirely with Red");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be drained: {pool:?}");
    }

    // Boggart Ram-Gang: empty pool -> Err (unaffordable, unchanged message).
    {
        let (state, id, p1) = manifest_state("Boggart Ram-Gang");
        let err = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        )
        .expect_err("empty pool cannot pay {R/G}{R/G}{R/G}");
        let msg = format!("{err:?}");
        assert!(
            matches!(err, GameStateError::InvalidCommand(_))
                && msg.contains("TurnFaceUp: player cannot pay the turn-face-up cost"),
            "expected the existing affordability message, got: {msg}"
        );
    }
}

// ── T4 — Phyrexian pip: payable with mana OR life (plan §2.1) ───────────────────

#[test]
/// CR 107.4f, 119.4 — a Phyrexian pip in a turn-face-up cost is payable with one
/// mana of its color OR by paying 2 life, and the CR 119.4 life-total check happens
/// BEFORE any mutation. Two synthetic creature defs (no shipped card carries a
/// Phyrexian pip in a printed `mana_cost` reachable by Manifest/Cloak on this
/// roster — plan §0 roster is all-hybrid): `{1}{G/P}` for cases 1-4, and a PURE
/// `{G/P}` (no generic) for case 5, which needs a cost whose raw `mana_value()` is
/// nonzero but whose FLATTENED `mana_value()` is zero.
fn turn_face_up_phyrexian_pip_payable_with_mana_or_life() {
    let mut cards = all_cards();
    cards.push(phyrexian_manifest_def(
        "test-phyrexian-manifest",
        "Test Phyrexian Manifest",
        1,
    ));
    cards.push(phyrexian_manifest_def(
        "test-pure-phyrexian-manifest",
        "Test Pure Phyrexian Manifest",
        0,
    ));
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    // Case 1: [false] + {1}{G} -> Ok, life unchanged (paid with mana, not life).
    {
        let (mut state, id, p1) =
            manifest_state_with(&defs, registry.clone(), "Test Phyrexian Manifest", 20);
        set_pool(&mut state, p1, 0, 0, 0, 0, 1, 1);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![false],
            },
        )
        .expect("{G/P} paid with mana: {1}{G} affordable");
        let ps = state.player(p1).unwrap();
        assert_eq!(
            ps.life_total, 20,
            "mana payment must not touch life: {ps:?}"
        );
        assert_eq!(
            ps.mana_pool.total(),
            0,
            "pool must be drained: {:?}",
            ps.mana_pool
        );
    }

    // Case 2: [true] + {1} only, 20 life -> Ok, life 18.
    {
        let (mut state, id, p1) =
            manifest_state_with(&defs, registry.clone(), "Test Phyrexian Manifest", 20);
        set_pool(&mut state, p1, 0, 0, 0, 0, 0, 1);
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![true],
            },
        )
        .expect("{G/P} paid with life: {1} generic affordable, 2 life payable at 20");
        let ps = state.player(p1).unwrap();
        assert_eq!(ps.life_total, 18, "CR 107.4f: 2 life paid: {ps:?}");
    }

    // Case 3: [true] + {1} only, 1 life -> Err(InsufficientLife) citing CR 119.4,
    // life unchanged.
    {
        let (mut state, id, p1) =
            manifest_state_with(&defs, registry.clone(), "Test Phyrexian Manifest", 1);
        set_pool(&mut state, p1, 0, 0, 0, 0, 0, 1);
        let result = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![true],
            },
        );
        match result {
            Err(GameStateError::InsufficientLife {
                required, actual, ..
            }) => {
                assert_eq!(
                    required, 2,
                    "CR 107.4f: a single Phyrexian pip paid with life costs 2"
                );
                assert_eq!(
                    actual, 1,
                    "life reported in the error must be pre-mutation: 1"
                );
            }
            other => panic!(
                "CR 119.4: at 1 life, paying 2 life must be rejected with \
                 InsufficientLife: {other:?}"
            ),
        }
        // `Err` discards the whole `GameState` (Architecture Invariants 2/3, plan
        // §5.1's "no rollback needed" argument) -- there is no post-command state to
        // re-read life from here; the error's own `actual: 1` field above is the
        // proof that no mutation happened before the check fired.
    }

    // Case 4: [true] + {1} only, 2 life -> Ok, life 0 -- and the CR 704.5a SBA loss
    // fires SEPARATELY as the legal-but-losing boundary (mirrors PB-RS2's
    // "exactly enough" pattern).
    {
        let (mut state, id, p1) =
            manifest_state_with(&defs, registry.clone(), "Test Phyrexian Manifest", 2);
        set_pool(&mut state, p1, 0, 0, 0, 0, 0, 1);
        let (state, events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![true],
            },
        )
        .expect("CR 119.4: at exactly 2 life, paying 2 life is legal (2 >= 2)");
        let ps = state.player(p1).unwrap();
        assert_eq!(ps.life_total, 0, "life reaches exactly 0: {ps:?}");
        assert!(
            ps.has_lost,
            "CR 704.5a: 0 life is legal to REACH but the player loses as an SBA \
             immediately after -- handle_turn_face_up's existing end-of-function \
             sba::check_and_apply_sbas call (unrelated to this batch) catches it"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::PlayerLost { player: pl, .. } if *pl == p1)),
            "PlayerLost must be present in the returned event stream: {events:?}"
        );
    }

    // Case 5: [true] + EMPTY pool on a PURE {G/P} cost (no generic) -> Ok, life 18
    // (a delta of -2 from 20) -- pins that the flatten runs BEFORE the
    // `mana_value() > 0` gate: the raw cost's mana_value() is 1 (one Phyrexian
    // pip, CR 202.3g), the FLATTENED cost's is 0 (the pip is paid with life, not
    // mana), so the mana check is correctly SKIPPED entirely and an empty pool is
    // no obstacle at all.
    {
        let (state, id, p1) =
            manifest_state_with(&defs, registry.clone(), "Test Pure Phyrexian Manifest", 20);
        assert_eq!(
            state.player(p1).unwrap().mana_pool.total(),
            0,
            "pool deliberately left empty for this case"
        );
        let (state, _events) = process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: id,
                method: TurnFaceUpMethod::ManaCost,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![true],
            },
        )
        .expect(
            "a pure {G/P} cost paid with life flattens to mana_value() == 0, so an \
             EMPTY pool must not block it -- the flatten runs before the mana gate",
        );
        let ps = state.player(p1).unwrap();
        assert_eq!(
            ps.life_total, 18,
            "CR 107.4f: 2 life paid, a SIBLING of the (correctly skipped) mana gate: {ps:?}"
        );
    }
}

// ── Observation C — attack tax, hybrid: rejected in every build (plan §2.0) ─────

#[test]
/// PRE-FIX, every build (no debug/release split — `rules/combat.rs` rejects a pipped
/// attack tax with a real `Err` before ever reaching `can_pay_cost`, plan §2.0's last
/// paragraph).
///
/// A synthetic `GameRestriction::CantAttackYouUnlessPay { cost_per_creature: {G/W} }`
/// (`HybridMana::ColorColor(Green, White)`, no other fields) sits on a P2 permanent;
/// P1 declares one attacker into P2. `handle_declare_attackers`
/// (`rules/combat.rs`, the restriction scan) sees `cost_per_creature.hybrid` non-empty
/// and inserts P2 into `unpayable_tax_defenders`; the attacker-loop then rejects the
/// declaration outright because a declared attacker targets that defender.
///
/// OBSERVED verbatim `InvalidCommand` message this run (P2's `ObjectId` interpolated
/// via `{:?}` on `PlayerId`, so the exact numeral varies by test-local id assignment;
/// the surrounding text is exact and is asserted below):
/// "attack tax: a hybrid, Phyrexian or X attack cost against defender PlayerId(2) is \
///  not payable -- Command::DeclareAttackers carries no payment-choice field, so the \
///  engine cannot ask which half to pay (CR 107.4e/107.4f via CR 508.1h); see \
///  OOS-DP4-1."
///
/// Plan §2.1 T3 claims this message contains `"is not payable"` and `"OOS-DP4-1"` —
/// both confirmed true by this run, verbatim, not merely reasoned from the source
/// read.
fn observation_c_hybrid_attack_tax_rejected_pre_fix() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, "Hybrid Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "Attacking Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let tax_source = find_by_name(&state, "Hybrid Tax Source");
    add_restriction(
        &mut state,
        tax_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p1);

    let bear = find_by_name(&state, "Attacking Bear");
    let result = process_command(
        state,
        declare_cmd(p1, vec![(bear, AttackTarget::Player(p2))]),
    );

    let err = result.expect_err(
        "PRE-FIX expectation: a hybrid attack tax against the declared defender must \
         be rejected outright, not silently paid free (OOS-DP4-1).",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("is not payable"),
        "plan §2.1 T3 claim not observed -- message did not contain \"is not \
         payable\": {msg}"
    );
    assert!(
        msg.contains("OOS-DP4-1"),
        "plan §2.1 T3 claim not observed -- message did not contain \"OOS-DP4-1\": \
         {msg}"
    );
    assert!(
        msg.contains("hybrid"),
        "message should name the pip class it rejected: {msg}"
    );
}

// ── Observation D(i) — genuinely unpayable {2} tax stays rejected (plan §2.1 T6) ──

#[test]
/// PRE-FIX baseline that T6 (implement phase) will assert stays UNCHANGED. A
/// Propaganda-shaped `{2}` tax (no pips, no X — `ManaCost { generic: 2, ..Default }`),
/// two attackers into the same defender (total `{4}`), attacking player's pool seeded
/// with exactly `{1}` (colorless: 1). `handle_declare_attackers`'s affordability block
/// (`casting::can_pay_cost`) rejects the declaration because {4} is not payable from
/// {1}.
///
/// OBSERVED verbatim `InvalidCommand` message this run:
/// "attack tax: the attacking player cannot pay the required ManaCost { white: 0, \
///  blue: 0, black: 0, red: 0, green: 0, colorless: 0, generic: 4, hybrid: [], \
///  phyrexian: [], x_count: 0 } for the declared attackers from their mana pool (CR \
///  508.1h/508.1j, Propaganda/Ghostly Prison); 1 unrestricted mana available."
fn observation_d1_propaganda_shaped_tax_still_rejected_pre_fix() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(
            ObjectSpec::creature(p2, "Propaganda Shaped Source", 0, 4).in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p1, "Attacking Bear One", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "Attacking Bear Two", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let tax_source = find_by_name(&state, "Propaganda Shaped Source");
    add_restriction(
        &mut state,
        tax_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                generic: 2,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p1);
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool.colorless = 1;
    }

    let bear1 = find_by_name(&state, "Attacking Bear One");
    let bear2 = find_by_name(&state, "Attacking Bear Two");
    let result = process_command(
        state,
        declare_cmd(
            p1,
            vec![
                (bear1, AttackTarget::Player(p2)),
                (bear2, AttackTarget::Player(p2)),
            ],
        ),
    );

    let err = result.expect_err(
        "PRE-FIX baseline: a genuinely unpayable {2}-per-creature tax (total {4} \
         against a {1} pool) must stay rejected.",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("cannot pay the required"),
        "message did not contain the expected \"cannot pay the required\" text: {msg}"
    );
    assert!(
        msg.contains("generic: 4"),
        "message did not show the summed {{4}} total (2 attackers x {{2}}): {msg}"
    );
}

// ── Observation D(ii) — X attack tax stays rejected (plan §2.1 T7 baseline) ─────

#[test]
/// PRE-FIX baseline that T7 (implement phase) will assert changes its REASONING but
/// not its OUTCOME. `cost_per_creature` here has `x_count: 1` and is otherwise
/// `ManaCost::default()` — no hybrid, no Phyrexian. `rules/combat.rs`'s restriction
/// scan funnels `x_count > 0` into the SAME `unpayable_tax_defenders` bucket as a
/// hybrid/Phyrexian pip (today's code does not distinguish the two classes), so the
/// pre-fix message is the identical hybrid/Phyrexian-shaped text Observation C
/// recorded — which is exactly the "errors for a different and now-wrong reason"
/// hazard plan §2.1 T7 names (the PB-DX2 T12 lesson): an `is_err()`-only assertion
/// here would be vacuous both before and after the coming fix, since the pre-fix code
/// also errors, just while citing the wrong pip class.
///
/// OBSERVED verbatim `InvalidCommand` message this run (identical shape to
/// Observation C's, confirming the shared-bucket claim above by execution rather than
/// by re-reading `combat.rs`):
/// "attack tax: a hybrid, Phyrexian or X attack cost against defender PlayerId(2) is \
///  not payable -- Command::DeclareAttackers carries no payment-choice field, so the \
///  engine cannot ask which half to pay (CR 107.4e/107.4f via CR 508.1h); see \
///  OOS-DP4-1."
///
/// Plan §5.2.3 states that post-fix this message must lose its hybrid/Phyrexian
/// clause, cite CR 107.3 + CR 601.2b, and cite the NEW seed rather than OOS-DP4-1 —
/// none of that has happened yet; this probe pins what "yet" looks like.
fn observation_d2_x_attack_tax_rejected_pre_fix() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, "X Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "Attacking Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let tax_source = find_by_name(&state, "X Tax Source");
    add_restriction(
        &mut state,
        tax_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                x_count: 1,
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p1);

    let bear = find_by_name(&state, "Attacking Bear");
    let result = process_command(
        state,
        declare_cmd(p1, vec![(bear, AttackTarget::Player(p2))]),
    );

    let err = result.expect_err(
        "PRE-FIX baseline: an X-count attack tax against the declared defender must \
         stay rejected.",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("is not payable"),
        "message did not contain \"is not payable\": {msg}"
    );
    assert!(
        msg.contains("OOS-DP4-1"),
        "message did not contain \"OOS-DP4-1\" (pre-fix cite; the coming fix must \
         replace this with a new seed per plan §5.2.3): {msg}"
    );
    assert!(
        msg.contains("hybrid, Phyrexian or X"),
        "PRE-FIX baseline: the message names all three pip classes generically, \
         confirming X shares the hybrid/Phyrexian rejection bucket rather than \
         getting its own CR 107.3/601.2b-cited message: {msg}"
    );
}
