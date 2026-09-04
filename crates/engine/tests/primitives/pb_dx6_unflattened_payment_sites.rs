//! PB-DX6 — hybrid/Phyrexian mana-cost payment site probes (OOS-RS2-1 + OOS-DP4-1).
//!
//! `memory/primitives/pb-plan-DX6.md` §0/§2/§2.0/§5.2/§5.3/§13 is authoritative. This
//! file began (stage 0) as pre-fix OBSERVATION probes only, against an unmodified
//! tree; stage B fixed `handle_turn_face_up` and stage C fixes
//! `handle_declare_attackers`'s CR 508.1h attack tax. Every pre-fix claim recorded
//! below was *observed by execution*, never reasoned to (plan §2's standing
//! discipline — the most-cited failure of this suite's last four batches), and every
//! `historical_*` test is an OLD scenario re-asserted against the NEW, post-fix
//! behaviour rather than deleted, so the pre-fix text's disappearance is itself
//! pinned (mirrors `historical_observation_a_...`, added in stage B).
//!
//! Both payment sites this batch targets are now fixed for hybrid and Phyrexian pips:
//! - `rules/engine.rs::handle_turn_face_up` (stage B) now flattens `def.mana_cost` (or
//!   the Morph/Megamorph/Disguise cost) before paying it — CR 701.40b's "pay that
//!   cost" now honours CR 107.4e/107.4f.
//! - `rules/combat.rs::handle_declare_attackers`'s CR 508.1h attack-tax total (stage
//!   C) now accepts `hybrid_choices`/`phyrexian_life_payments` and pays the
//!   accumulated, flattened total. **X remains rejected** — `Command::DeclareAttackers`
//!   has no channel to announce an X value (CR 107.3/601.2b) — but the rejection
//!   message no longer claims hybrid/Phyrexian costs are unpayable, and cites the new
//!   seed OOS-DX6-1 rather than the now-closed OOS-DP4-1.
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
    TypeLine, ZoneId, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
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
            route), and now PERMANENTLY NON-REPRODUCIBLE on this tree -- PB-DX6 \
            fix-cycle Finding 6. The recipe below was accurate at stage 0, against an \
            unmodified pre-fix tree; after stage B, handle_turn_face_up flattens \
            def.mana_cost BEFORE can_spend/spend are ever reached, so commenting out \
            debug_assert_flattened(cost) alone changes nothing on the post-fix path -- \
            the recipe would need `handle_turn_face_up`'s stage-B flatten hunk ALSO \
            reverted (e.g. `git stash` of just that hunk) to reproduce the recorded \
            numbers, and doing that is not worth the churn for a single historical \
            observation. If you need to re-derive this figure, temporarily comment \
            out `debug_assert_flattened(cost);` at the top of ManaPool::can_spend in \
            crates/card-types/src/state/player.rs AND revert handle_turn_face_up's \
            unconditional-flatten hunk in crates/engine/src/rules/engine.rs, run \
            `cargo test -p mtg-engine --test primitives historical_observation_b -- \
            --ignored --nocapture`, then restore BOTH and confirm `git diff` is empty \
            over both files. The OBSERVED numbers below are the historical record and \
            are NOT re-derivable from a one-line revert alone."]
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
///
/// This test is now HISTORICAL (PB-DX6 fix-cycle Finding 6): its recorded numbers
/// stand as the release-equivalent evidence for OOS-RS2-1, but its `#[ignore]`
/// recipe can no longer reproduce them on the post-fix tree with a single-guard
/// revert — see the `#[ignore]` reason for the corrected two-hunk procedure. Kept
/// (not deleted) as the permanent record, matching the `historical_observation_a/c/d2`
/// treatment used elsewhere in this file.
fn historical_observation_b_release_figure_pool_debit_kitchen_finks() {
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

// ── Historical Observation C — attack tax, hybrid: no longer a class rejection ──

#[test]
/// HISTORICAL / POST-FIX regression pin (mirrors `historical_observation_a_...`).
/// Before PB-DX6 stage C, THIS EXACT SCENARIO — a P2-controlled
/// `CantAttackYouUnlessPay { cost_per_creature: {G/W} }` restriction, P1 declaring
/// one attacker into P2, an EMPTY P1 mana pool — was rejected as a whole "unpayable
/// class" regardless of the pool: `handle_declare_attackers` (`rules/combat.rs`, the
/// restriction scan) saw `cost_per_creature.hybrid` non-empty and inserted P2 into
/// `unpayable_tax_defenders`, and the attacker-loop rejected the declaration outright
/// because a declared attacker targeted that defender.
///
/// PRE-FIX (recorded 2026-08-01/02, plan §2.1 Observation C; P2's `PlayerId`
/// interpolated positionally, the surrounding text exact) — preserved VERBATIM as a
/// permanent record, not re-executed:
/// "attack tax: a hybrid, Phyrexian or X attack cost against defender PlayerId(2) is \
///  not payable -- Command::DeclareAttackers carries no payment-choice field, so the \
///  engine cannot ask which half to pay (CR 107.4e/107.4f via CR 508.1h); see \
///  OOS-DP4-1."
///
/// POST-FIX (asserted live below): a hybrid attack tax is now PAYABLE (CR 107.4e via
/// CR 508.1h; T3 proves the payable cases with a seeded pool). The IDENTICAL command
/// against the IDENTICAL empty pool therefore no longer names a whole payability
/// CLASS as rejected — it returns a real, CR-legal `Err(InvalidCommand)` for ordinary
/// insufficient-mana reasons (an empty pool cannot pay the flattened `{G}`
/// requirement — CR 107.4e's documented default, first colour). This is T3's own
/// empty-pool shape, restated standalone against the exact pre-fix scenario so the
/// "unpayable class" message's disappearance is pinned by name, not only
/// incidentally covered by T3.
fn historical_observation_c_hybrid_attack_tax_no_longer_unpayable_class() {
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
        "POST-FIX: an unaffordable hybrid attack tax must still be REJECTED, but for \
         insufficiency, not as a whole unpayable class. If this observes the old \
         \"is not payable\"/OOS-DP4-1 message, the payable-hybrid-attack-tax fix has \
         regressed and this historical record is no longer accurate.",
    );
    let msg = format!("{err:?}");
    assert!(
        matches!(err, GameStateError::InvalidCommand(_)),
        "expected InvalidCommand (the affordability message), got: {msg}"
    );
    assert!(
        !msg.contains("is not payable") && !msg.contains("OOS-DP4-1"),
        "the pre-fix \"unpayable class\" message must be gone: {msg}"
    );
    assert!(
        msg.contains("cannot pay the required"),
        "expected the genuine-insufficiency affordability message: {msg}"
    );
}

// ── Shared fixture for T3/T5/T8-T11 ──────────────────────────────────────────────

/// Build an attack-tax fixture: P2 controls a permanent bearing
/// `CantAttackYouUnlessPay { cost_per_creature: pip_cost }`, P1 controls one creature
/// able to attack P2, priority held by P1. The pool is left at all-zero — callers
/// seed it per case. `source_name`/`bear_name` must be unique per call within a test
/// (`find_by_name` matches on `characteristics.name`).
fn attack_tax_state(
    pip_cost: ManaCost,
    source_name: &str,
    bear_name: &str,
) -> (GameState, PlayerId, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, source_name, 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, bear_name, 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    let tax_source = find_by_name(&state, source_name);
    add_restriction(
        &mut state,
        tax_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: pip_cost,
        },
    );
    state.turn_mut().priority_holder = Some(p1);
    let bear = find_by_name(&state, bear_name);
    (state, p1, p2, bear)
}

// ── T3 — hybrid attack tax is payable (plan §2.1 T3) ────────────────────────────

#[test]
/// CR 508.1h/508.1j, 107.4e — a hybrid attack tax is now PAYABLE (PB-DX6). Fixture:
/// a synthetic `CantAttackYouUnlessPay { cost_per_creature: {G/W} }` restriction on a
/// P2 permanent, P1 declaring one attacker into P2.
///
/// PRE-FIX (Observation C, quoted verbatim — see
/// `historical_observation_c_hybrid_attack_tax_no_longer_unpayable_class`):
/// "attack tax: a hybrid, Phyrexian or X attack cost against defender PlayerId(2) is \
///  not payable -- Command::DeclareAttackers carries no payment-choice field, so the \
///  engine cannot ask which half to pay (CR 107.4e/107.4f via CR 508.1h); see \
///  OOS-DP4-1."
fn hybrid_attack_tax_is_payable() {
    // Case 1: pool {G}, [Color(Green)] -> Ok, pool empty, attacker declared.
    {
        let (mut state, p1, p2, bear) = attack_tax_state(
            ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
            "T3 Hybrid Tax Source A",
            "T3 Attacking Bear A",
        );
        set_pool(&mut state, p1, 0, 0, 0, 0, 1, 0);
        let (state, _events) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(bear, AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
                hybrid_choices: vec![HybridManaPayment::Color(ManaColor::Green)],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("{G} pays {G/W} chosen Green");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be drained: {pool:?}");
        assert!(
            state
                .combat()
                .as_ref()
                .map(|c| c.attackers.contains_key(&bear))
                .unwrap_or(false),
            "the attacker must actually be declared, not merely affordable"
        );
    }

    // Case 2: pool {W}, [Color(White)] -> Ok (each pip chosen INDEPENDENTLY, CR 107.4e).
    {
        let (mut state, p1, p2, bear) = attack_tax_state(
            ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
            "T3 Hybrid Tax Source B",
            "T3 Attacking Bear B",
        );
        set_pool(&mut state, p1, 1, 0, 0, 0, 0, 0);
        let (state, _events) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(bear, AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
                hybrid_choices: vec![HybridManaPayment::Color(ManaColor::White)],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("{W} pays {G/W} chosen White");
        let pool = &state.player(p1).unwrap().mana_pool;
        assert_eq!(pool.total(), 0, "pool must be drained: {pool:?}");
    }

    // Case 3: pool {G}, [Color(White)] -> Err(InvalidCommand): insufficient mana, NOT
    // "unpayable class" (the pre-fix rejection reason).
    {
        let (mut state, p1, p2, bear) = attack_tax_state(
            ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
            "T3 Hybrid Tax Source C",
            "T3 Attacking Bear C",
        );
        set_pool(&mut state, p1, 0, 0, 0, 0, 1, 0);
        let err = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(bear, AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
                hybrid_choices: vec![HybridManaPayment::Color(ManaColor::White)],
                phyrexian_life_payments: vec![],
            },
        )
        .expect_err("pool has Green only, but White was chosen -- insufficient, not unpayable");
        let msg = format!("{err:?}");
        assert!(
            matches!(err, GameStateError::InvalidCommand(_)),
            "expected InvalidCommand: {msg}"
        );
        assert!(
            msg.contains("cannot pay the required"),
            "expected the genuine-insufficiency message: {msg}"
        );
        assert!(
            !msg.contains("is not payable") && !msg.contains("OOS-DP4-1"),
            "must NOT be the pre-fix class-rejection message: {msg}"
        );
    }
}

// ── T5 — Phyrexian attack tax: payable with mana OR life (plan §2.1 T5) ─────────

#[test]
/// CR 107.4f, 119.4, 119.4b — a Phyrexian pip in an attack tax is payable with one
/// mana of its colour OR by paying 2 life, mirroring T4's turn-face-up cases but on
/// `cost_per_creature: {W/P}` — this is **Norn's Annex, simulated** (the plan's own
/// framing, §1/§2.1): "Creatures can't attack you ... unless their controller pays
/// {W/P} for each of those creatures."
fn phyrexian_attack_tax_payable_with_mana_or_life() {
    // Case 1: [false] + pool {W} -> Ok, life unchanged (paid with mana, not life).
    {
        let (mut state, p1, p2, bear) = attack_tax_state(
            ManaCost {
                phyrexian: vec![PhyrexianMana::Single(ManaColor::White)],
                ..Default::default()
            },
            "T5 Phyrexian Tax Source A",
            "T5 Attacking Bear A",
        );
        if let Some(ps) = state.players_mut().get_mut(&p1) {
            ps.life_total = 20;
        }
        set_pool(&mut state, p1, 1, 0, 0, 0, 0, 0);
        let (state, _events) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(bear, AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![false],
            },
        )
        .expect("{W/P} paid with mana: {W} affordable");
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

    // Case 2: [true] + empty pool, 20 life -> Ok, life 18.
    {
        let (mut state, p1, p2, bear) = attack_tax_state(
            ManaCost {
                phyrexian: vec![PhyrexianMana::Single(ManaColor::White)],
                ..Default::default()
            },
            "T5 Phyrexian Tax Source B",
            "T5 Attacking Bear B",
        );
        if let Some(ps) = state.players_mut().get_mut(&p1) {
            ps.life_total = 20;
        }
        let (state, _events) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(bear, AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![true],
            },
        )
        .expect("{W/P} paid with life: 2 life payable at 20");
        let ps = state.player(p1).unwrap();
        assert_eq!(ps.life_total, 18, "CR 107.4f: 2 life paid: {ps:?}");
    }

    // Case 3: [true] + 1 life -> Err(InsufficientLife) citing CR 119.4, life unchanged.
    {
        let (mut state, p1, p2, bear) = attack_tax_state(
            ManaCost {
                phyrexian: vec![PhyrexianMana::Single(ManaColor::White)],
                ..Default::default()
            },
            "T5 Phyrexian Tax Source C",
            "T5 Attacking Bear C",
        );
        if let Some(ps) = state.players_mut().get_mut(&p1) {
            ps.life_total = 1;
        }
        let result = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(bear, AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
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
                    "CR 107.4f: a single Phyrexian pip costs 2 life"
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
    }

    // Case 4 (Norn's Annex, simulated): two attackers, [false, true], pool {W},
    // 20 life -> Ok, pool empty, life 18. The ruling's own example: "you may pay {W}
    // for one cost and 2 life for the other."
    {
        let p1 = p(1);
        let p2 = p(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .object(
                ObjectSpec::creature(p2, "T5 Norn's Annex Source", 0, 4)
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(p1, "T5 Norn's Annex Bear One", 2, 2)
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(p1, "T5 Norn's Annex Bear Two", 2, 2)
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .unwrap();
        let tax_source = find_by_name(&state, "T5 Norn's Annex Source");
        add_restriction(
            &mut state,
            tax_source,
            p2,
            GameRestriction::CantAttackYouUnlessPay {
                cost_per_creature: ManaCost {
                    phyrexian: vec![PhyrexianMana::Single(ManaColor::White)],
                    ..Default::default()
                },
            },
        );
        state.turn_mut().priority_holder = Some(p1);
        if let Some(ps) = state.players_mut().get_mut(&p1) {
            ps.life_total = 20;
        }
        set_pool(&mut state, p1, 1, 0, 0, 0, 0, 0);
        let bear1 = find_by_name(&state, "T5 Norn's Annex Bear One");
        let bear2 = find_by_name(&state, "T5 Norn's Annex Bear Two");
        let (state, _events) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![
                    (bear1, AttackTarget::Player(p2)),
                    (bear2, AttackTarget::Player(p2)),
                ],
                enlist_choices: vec![],
                exert_choices: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![false, true],
            },
        )
        .expect(
            "Norn's Annex ruling: {W} for one copy, 2 life for the other, chosen \
             individually",
        );
        let ps = state.player(p1).unwrap();
        assert_eq!(
            ps.mana_pool.total(),
            0,
            "pool must be drained: {:?}",
            ps.mana_pool
        );
        assert_eq!(
            ps.life_total, 18,
            "CR 107.4f: exactly 2 life paid for the SECOND copy: {ps:?}"
        );
    }
}

// ── T6 — a genuinely unpayable {2} tax is still rejected (plan §2.1 T6) ─────────

#[test]
/// CR 508.1h/508.1j — proves this batch widened *payability* (hybrid/Phyrexian pips
/// are now chargeable), not *acceptance* (a tax the pool genuinely cannot cover is
/// still rejected, unchanged). A Propaganda-shaped `{2}` tax (no pips, no X —
/// `ManaCost { generic: 2, ..Default }`), two attackers into the same defender (total
/// `{4}`), attacking player's pool seeded with exactly `{1}` (colorless: 1).
/// `handle_declare_attackers`'s affordability block (`casting::can_pay_cost`) rejects
/// the declaration because {4} is not payable from {1} — for a cost with no pips at
/// all, `flat_total == total`, so this message is byte-identical pre- and post-fix
/// (confirmed by this test, unchanged since it was first written as a pre-fix
/// baseline):
///
/// OBSERVED verbatim `InvalidCommand` message (both before and after PB-DX6 stage C):
/// "attack tax: the attacking player cannot pay the required ManaCost { white: 0, \
///  blue: 0, black: 0, red: 0, green: 0, colorless: 0, generic: 4, hybrid: [], \
///  phyrexian: [], x_count: 0 } for the declared attackers from their mana pool (CR \
///  508.1h/508.1j, Propaganda/Ghostly Prison); 1 unrestricted mana available."
fn genuinely_unpayable_attack_tax_is_still_rejected() {
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

// ── Historical Observation D(ii) — X attack tax: message rewritten ──────────────

#[test]
/// HISTORICAL / POST-FIX regression pin. Before PB-DX6 stage C, an X-count attack tax
/// (`cost_per_creature { x_count: 1, ..Default }`, no hybrid, no Phyrexian) was
/// rejected with the SAME "hybrid, Phyrexian or X ... is not payable ... OOS-DP4-1"
/// text hybrid/Phyrexian pips got — `rules/combat.rs`'s restriction scan funnelled
/// `x_count > 0` into the same bucket as a pip (the pre-fix code did not distinguish
/// the two classes).
///
/// PRE-FIX (recorded 2026-08-01/02, plan §2.1 Observation D(ii); identical shape to
/// Observation C's own text) — preserved VERBATIM, not re-executed:
/// "attack tax: a hybrid, Phyrexian or X attack cost against defender PlayerId(2) is \
///  not payable -- Command::DeclareAttackers carries no payment-choice field, so the \
///  engine cannot ask which half to pay (CR 107.4e/107.4f via CR 508.1h); see \
///  OOS-DP4-1."
///
/// POST-FIX (asserted live below): X is STILL rejected (CR 107.3/601.2b —
/// `Command::DeclareAttackers` has no X-announcement channel), but the message now
/// names X specifically, no longer claims hybrid/Phyrexian costs are unpayable (they
/// are not, as of this batch), and cites the NEW seed OOS-DX6-1 rather than the
/// closed OOS-DP4-1. T7 (below) asserts the same three properties on a fresh
/// scenario; this test pins the disappearance of the PRE-FIX text against the exact
/// historical scenario.
fn historical_observation_d2_x_attack_tax_message_rewritten() {
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
        "POST-FIX: an X-count attack tax against the declared defender must STILL be \
         rejected -- X has no announcement channel on this command.",
    );
    let msg = format!("{err:?}");
    assert!(
        !msg.contains("is not payable"),
        "the pre-fix \"is not payable\" class-rejection text must be gone: {msg}"
    );
    assert!(
        !msg.contains("hybrid, Phyrexian or X"),
        "the message must no longer name hybrid/Phyrexian as unpayable classes: {msg}"
    );
    assert!(
        !msg.contains("OOS-DP4-1"),
        "the pre-fix cite (OOS-DP4-1, closed by this batch) must be gone: {msg}"
    );
    assert!(
        msg.contains("OOS-DX6-1"),
        "expected the NEW seed cite: {msg}"
    );
    assert!(
        msg.contains("CR 107.3") && msg.contains("601.2b"),
        "expected the X-specific CR citations: {msg}"
    );
}

// ── T7 — X attack tax: still rejected, says only X (plan §2.1 T7) ───────────────

#[test]
/// CR 107.3 — X in an attack cost has no announcement channel on
/// `Command::DeclareAttackers` (no `x_value` field, unlike `CastSpell`), so it stays
/// rejected. **Assert on the MESSAGE TEXT**, not merely `is_err()`: the pre-fix code
/// also errored here (see `historical_observation_d2_...`), just while citing the
/// wrong pip class — an `is_err()`-only probe would be vacuous both before and after
/// this batch (the PB-DX2 T12 lesson). This probe pins the THREE properties the
/// message must now have: (i) names X, (ii) does NOT claim hybrid/Phyrexian costs are
/// unpayable, (iii) cites OOS-DX6-1 rather than the closed OOS-DP4-1.
fn x_attack_tax_is_still_rejected_and_says_only_x() {
    let (mut state, p1, p2, bear) = attack_tax_state(
        ManaCost {
            x_count: 1,
            ..Default::default()
        },
        "T7 X Tax Source",
        "T7 Attacking Bear",
    );
    // Pool is fully seeded so an affordability rejection (T6's shape) cannot be
    // mistaken for the X rejection this test targets.
    set_pool(&mut state, p1, 9, 9, 9, 9, 9, 9);
    let err = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect_err("an X-count attack tax must be rejected: no announcement channel exists");
    let msg = format!("{err:?}");
    assert!(
        matches!(err, GameStateError::InvalidCommand(_)),
        "expected InvalidCommand: {msg}"
    );
    // (i) names X.
    assert!(
        msg.contains('X') || msg.contains("x_count"),
        "message must name X: {msg}"
    );
    // (ii) does NOT claim hybrid/Phyrexian are unpayable.
    assert!(
        !msg.contains("hybrid") && !msg.contains("Phyrexian"),
        "message must NOT claim hybrid/Phyrexian costs are unpayable -- they are \
         payable as of this batch: {msg}"
    );
    // (iii) cites the NEW seed, not the closed one.
    assert!(
        msg.contains("OOS-DX6-1"),
        "expected the new seed cite: {msg}"
    );
    assert!(
        !msg.contains("OOS-DP4-1"),
        "must not cite the closed seed OOS-DP4-1: {msg}"
    );
}

// ── T8/T9 — the order pin: copy-major, not pip-major (plan §13 risk 2/3) ────────

#[test]
/// CR 508.1h, `add_mana_cost`/`accumulate_attack_tax_total`'s own doc — pins the
/// **canonical pip order** by execution, the weakest joint in this design per plan
/// §13 risk 2: `hybrid_choices[i]` indexes a cost the client cannot see, and a
/// "harmless" future dedup of `add_mana_cost` onto `multiply_mana_cost` (risk 3,
/// OOS-DP4-7) would silently re-order it with no compile error.
///
/// Two defenders (P2 < P3, ascending `PlayerId`), asymmetric restriction shapes:
/// - P2: ONE restriction, `cost_per_creature: {G/W}`; TWO attackers into P2 -- this
///   defender's per-creature entry is a single pip, replicated into two COPIES.
/// - P3: TWO restrictions (added in this order), `cost_per_creature: {G/W}` then
///   `cost_per_creature: {R/W}`; ONE attacker into P3 -- this defender's
///   per-creature entry is the CONCATENATION of both restrictions' pips (one copy).
///
/// The canonical order is therefore `[P2-copy1, P2-copy2, P3-r1, P3-r2]` (4 hybrid
/// pips): defenders ascending by `PlayerId` (P2 before P3), then P2's two copies
/// before P3's single copy, then P3's own two restrictions in insertion order WITHIN
/// that one copy. A PIP-MAJOR order (what a naive `multiply_mana_cost`-based
/// implementation would produce, grouping by restriction across the whole
/// declaration) would be `[P2-copy1, P2-copy2, P3-r1, P3-r2]` too for this
/// particular defender/restriction shape by coincidence of P2 having only one
/// restriction -- so the choices below are deliberately ASYMMETRIC per copy
/// (`Green` then `White` for P2's two identical-shaped copies) specifically so a
/// pip-major bug that grouped "all copies of P2's restriction, then P3's two
/// restrictions" would still pass; what actually discriminates copy-major from any
/// alternative interleaving is that the pool is seeded to the EXACT sum the
/// as-designed order requires and the declaration must succeed with ZERO mana left
/// over in every field -- any different index-to-pip mapping requires a different
/// colour distribution and the affordability check would reject it (or leave a
/// nonzero remainder), catching a reordering by execution, not by inspection.
fn two_defenders_two_restrictions_attack_tax_pip_order_is_copy_major() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, "T8 P2 Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p3, "T8 P3 Tax Source One", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p3, "T8 P3 Tax Source Two", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T8 Bear Into P2 One", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T8 Bear Into P2 Two", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T8 Bear Into P3", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let p2_source = find_by_name(&state, "T8 P2 Tax Source");
    add_restriction(
        &mut state,
        p2_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    let p3_source_one = find_by_name(&state, "T8 P3 Tax Source One");
    add_restriction(
        &mut state,
        p3_source_one,
        p3,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    let p3_source_two = find_by_name(&state, "T8 P3 Tax Source Two");
    add_restriction(
        &mut state,
        p3_source_two,
        p3,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Red, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p1);

    // Canonical order: [P2-copy1, P2-copy2, P3-r1, P3-r2].
    // idx0 (P2 copy 1, {G/W}) -> Green. idx1 (P2 copy 2, {G/W}) -> White.
    // idx2 (P3 restriction 1, {G/W}) -> Green. idx3 (P3 restriction 2, {R/W}) -> Red.
    // Required flat cost: green 2 (idx0 + idx2), white 1 (idx1), red 1 (idx3).
    set_pool(&mut state, p1, 1, 0, 0, 1, 2, 0);

    let bear_p2_one = find_by_name(&state, "T8 Bear Into P2 One");
    let bear_p2_two = find_by_name(&state, "T8 Bear Into P2 Two");
    let bear_p3 = find_by_name(&state, "T8 Bear Into P3");
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bear_p2_one, AttackTarget::Player(p2)),
                (bear_p2_two, AttackTarget::Player(p2)),
                (bear_p3, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![
                HybridManaPayment::Color(ManaColor::Green),
                HybridManaPayment::Color(ManaColor::White),
                HybridManaPayment::Color(ManaColor::Green),
                HybridManaPayment::Color(ManaColor::Red),
            ],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "the exact pool seeded for the canonical [P2-copy1, P2-copy2, P3-r1, P3-r2] \
         order must be sufficient -- if this errs, the accumulation order regressed",
    );

    let pool = &state.player(p1).unwrap().mana_pool;
    assert_eq!(
        pool.total(),
        0,
        "the seeded pool must be drained to EXACTLY zero across every field -- any \
         other index-to-pip mapping would leave a nonzero remainder or fail \
         affordability entirely: {pool:?}"
    );
    assert_eq!(
        pool.green, 0,
        "both Green requirements must be spent: {pool:?}"
    );
    assert_eq!(
        pool.white, 0,
        "the White requirement must be spent: {pool:?}"
    );
    assert_eq!(pool.red, 0, "the Red requirement must be spent: {pool:?}");

    // The ManaCostPaid event carries the ORIGINAL pipped total in the canonical
    // order -- assert its hybrid vec directly, the strongest possible pin.
    let paid = events
        .iter()
        .find_map(|e| match e {
            GameEvent::ManaCostPaid { player: pl, cost } if *pl == p1 => Some(cost.clone()),
            _ => None,
        })
        .expect("ManaCostPaid must be emitted");
    assert_eq!(
        paid.hybrid,
        vec![
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
        ],
        "canonical pip order must be [P2-copy1, P2-copy2, P3-r1, P3-r2] -- COPY-MAJOR, \
         not pip-major: {paid:?}"
    );
}

#[test]
/// CR 508.1h -- the discriminating case T8 above cannot provide (PB-DX6 fix-cycle
/// Finding 1, filed against `two_defenders_two_restrictions_attack_tax_pip_order_is_copy_major`).
/// T8's only two-restriction defender (P3) has exactly ONE attacker, so `times == 1`
/// for that defender's outer composition and copy-major/pip-major coincide by
/// construction — T8's own doc concedes this in so many words. This test isolates the
/// minimum shape where the two orders actually diverge: ONE defender with TWO
/// DISTINCT restrictions, attacked by TWO creatures.
///
/// Copy-major (canonical, per `add_mana_cost`'s doc): the per-creature entry is the
/// concatenation `[r1, r2]` = `[{G/W}, {R/W}]`, replicated once per attacker ->
/// `[r1, r2, r1, r2]` = `[G/W, R/W, G/W, R/W]`.
/// Pip-major (`multiply_mana_cost`'s `flat_map(repeat_n)` shape — the OOS-DP4-7 dedup
/// this test exists to keep rejected): each restriction is replicated across every
/// attacker before moving to the next -> `[r1, r1, r2, r2]` = `[G/W, G/W, R/W, R/W]`.
///
/// The choice vector `[Green, Red, White, White]` is LEGAL under copy-major (idx0
/// `{G/W}`->Green, idx1 `{R/W}`->Red, idx2 `{G/W}`->White, idx3 `{R/W}`->White; pool
/// drains to exactly zero) and ILLEGAL under pip-major, where idx1 would instead be
/// the SECOND copy of `{G/W}` and `Red` is not a legal choice for a `{G/W}` pip (CR
/// 107.4e) — a pip-major engine rejects this exact declaration outright rather than
/// merely charging a different pool, so the two orders are not silently
/// interchangeable here.
///
/// **Verified by revert-and-restore in the PB-DX6 fix cycle**, per plan §13 risk 3's
/// own instruction: with `add_mana_cost`'s replication loop temporarily swapped for
/// `multiply_mana_cost`'s `flat_map(repeat_n)` shape, this test's `.expect()` panics
/// on the resulting `Err` (a pip-major-triggered `InvalidCommand`) while T8 above
/// stays green, exactly as predicted — T8's fixture cannot see the permutation this
/// one is built to see. Restored immediately after; `git diff` over `combat.rs` was
/// confirmed empty.
fn one_defender_two_distinct_restrictions_two_attackers_discriminates_copy_vs_pip_major() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, "T9 P2 Tax Source One", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "T9 P2 Tax Source Two", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T9 Bear One", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T9 Bear Two", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let source_one = find_by_name(&state, "T9 P2 Tax Source One");
    add_restriction(
        &mut state,
        source_one,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    let source_two = find_by_name(&state, "T9 P2 Tax Source Two");
    add_restriction(
        &mut state,
        source_two,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Red, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p1);

    // Canonical (copy-major) order: [r1, r2, r1, r2] = [G/W, R/W, G/W, R/W].
    // idx0 {G/W} -> Green. idx1 {R/W} -> Red. idx2 {G/W} -> White. idx3 {R/W} -> White.
    // Required flat cost: green 1, red 1, white 2.
    set_pool(&mut state, p1, 2, 0, 0, 1, 1, 0);

    let bear_one = find_by_name(&state, "T9 Bear One");
    let bear_two = find_by_name(&state, "T9 Bear Two");
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (bear_one, AttackTarget::Player(p2)),
                (bear_two, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![
                HybridManaPayment::Color(ManaColor::Green),
                HybridManaPayment::Color(ManaColor::Red),
                HybridManaPayment::Color(ManaColor::White),
                HybridManaPayment::Color(ManaColor::White),
            ],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "the copy-major choice vector [Green, Red, White, White] must be legal against \
         the canonical [r1, r2, r1, r2] pip order -- if this errs, the accumulation \
         reordered to pip-major ([r1, r1, r2, r2]), which makes idx1 the SECOND copy \
         of {G/W} and Red an illegal choice for it (CR 107.4e)",
    );

    let pool = &state.player(p1).unwrap().mana_pool;
    assert_eq!(
        pool.total(),
        0,
        "the seeded pool must be drained to EXACTLY zero: {pool:?}"
    );

    let paid = events
        .iter()
        .find_map(|e| match e {
            GameEvent::ManaCostPaid { player: pl, cost } if *pl == p1 => Some(cost.clone()),
            _ => None,
        })
        .expect("ManaCostPaid must be emitted");
    assert_eq!(
        paid.hybrid,
        vec![
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
        ],
        "canonical pip order must be [r1, r2, r1, r2] -- COPY-MAJOR, not pip-major \
         ([r1, r1, r2, r2]): {paid:?}"
    );
}

// ── T10 — queries::attack_tax_total matches what the engine actually charges ────

#[test]
/// Plan §5.3's anti-drift requirement, proven by execution: `queries::attack_tax_total`
/// and `handle_declare_attackers`'s own validation call the SAME
/// `combat::accumulate_attack_tax_total` helper, so the total the query reports
/// BEFORE a declaration must equal the pipped total the engine actually charges
/// (visible in the `ManaCostPaid` event) when that same attacker set is declared.
/// Reuses T8's three-player, two-defender/two-restriction fixture shape (T9 above has
/// its own, smaller two-player fixture) -- the shape most likely to expose a
/// divergence between two independently-written accumulations.
fn attack_tax_total_query_matches_declared_attackers_charge() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, "T10 P2 Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
        .object(
            ObjectSpec::creature(p3, "T10 P3 Tax Source One", 0, 4).in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p3, "T10 P3 Tax Source Two", 0, 4).in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p1, "T10 Bear Into P2 One", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T10 Bear Into P2 Two", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T10 Bear Into P3", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let p2_source = find_by_name(&state, "T10 P2 Tax Source");
    add_restriction(
        &mut state,
        p2_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    let p3_source_one = find_by_name(&state, "T10 P3 Tax Source One");
    add_restriction(
        &mut state,
        p3_source_one,
        p3,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    let p3_source_two = find_by_name(&state, "T10 P3 Tax Source Two");
    add_restriction(
        &mut state,
        p3_source_two,
        p3,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                hybrid: vec![HybridMana::ColorColor(ManaColor::Red, ManaColor::White)],
                ..Default::default()
            },
        },
    );
    state.turn_mut().priority_holder = Some(p1);
    set_pool(&mut state, p1, 1, 0, 0, 1, 2, 0);

    let bear_p2_one = find_by_name(&state, "T10 Bear Into P2 One");
    let bear_p2_two = find_by_name(&state, "T10 Bear Into P2 Two");
    let bear_p3 = find_by_name(&state, "T10 Bear Into P3");
    let attackers = vec![
        (bear_p2_one, AttackTarget::Player(p2)),
        (bear_p2_two, AttackTarget::Player(p2)),
        (bear_p3, AttackTarget::Player(p3)),
    ];

    // Query BEFORE any command is issued -- purely advisory, must not mutate.
    let queried = mtg_engine::attack_tax_total(&state, p1, &attackers)
        .expect("a nonzero attack tax applies to this declaration");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers,
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![
                HybridManaPayment::Color(ManaColor::Green),
                HybridManaPayment::Color(ManaColor::White),
                HybridManaPayment::Color(ManaColor::Green),
                HybridManaPayment::Color(ManaColor::Red),
            ],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("declaration matching the queried total must succeed");

    let charged = events
        .iter()
        .find_map(|e| match e {
            GameEvent::ManaCostPaid { player: pl, cost } if *pl == p1 => Some(cost.clone()),
            _ => None,
        })
        .expect("ManaCostPaid must be emitted");

    assert_eq!(
        queried, charged,
        "queries::attack_tax_total must report the SAME total (same order, same \
         content) as what handle_declare_attackers actually charges -- a divergence \
         here means the two accumulations drifted: queried={queried:?} \
         charged={charged:?}"
    );
    let _ = state;
}

// ── T11 — an all-Phyrexian attack tax still costs life (plan §13 risk 9) ────────

#[test]
/// CR 107.4f, 118.5 — plan §13 risk 9: a `cost_per_creature` that is ENTIRELY
/// Phyrexian, paid entirely with life, flattens to `ManaCost::default()`
/// (`mana_value() == 0`) but the PIPPED total is non-default. The
/// `total != ManaCost::default()` guard that decides whether to enter the payment
/// block at all must be evaluated on the PIPPED total, not the flattened one, or the
/// whole payment (both the (zero) mana half AND the life half) silently vanishes --
/// "a real and easy bug to write" per the plan.
fn all_phyrexian_attack_tax_still_costs_life() {
    let (mut state, p1, p2, bear) = attack_tax_state(
        ManaCost {
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
            ..Default::default()
        },
        "T11 Phyrexian Tax Source",
        "T11 Attacking Bear",
    );
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.life_total = 20;
    }
    // Pool deliberately left EMPTY: the flattened cost is {0}, so an empty pool must
    // be no obstacle at all if the guard is correctly evaluated on the pipped total.
    assert_eq!(
        state.player(p1).unwrap().mana_pool.total(),
        0,
        "pool deliberately left empty for this case"
    );

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![true],
        },
    )
    .expect(
        "a pure Phyrexian attack tax paid entirely with life must succeed even with \
         an EMPTY pool -- the flatten runs before any mana check and the pipped, not \
         flattened, total gates whether the payment block runs at all",
    );

    let ps = state.player(p1).unwrap();
    assert_eq!(
        ps.life_total, 18,
        "CR 107.4f: 2 life paid for the single Phyrexian pip -- if this is still 20, \
         the payment silently vanished (the exact bug plan §13 risk 9 names): {ps:?}"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::LifeLost { player: pl, amount } if *pl == p1 && *amount == 2)
        ),
        "LifeLost must be present in the event stream: {events:?}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaCostPaid { player: pl, cost }
                if *pl == p1 && !cost.phyrexian.is_empty()
        )),
        "ManaCostPaid must still be emitted, carrying the PIPPED (unflattened) cost, \
         even though the flattened mana component is zero: {events:?}"
    );
}

// ── T12 — batch-level wire sentinel (plan §7.4) ─────────────────────────────────

#[test]
/// Wire-fingerprint pin, computed rather than predicted (plan §7.1/§7.2), following
/// the PB-DX5 `test_dx5_hash_schema_version_is_70` template.
///
/// `PROTOCOL_VERSION` **moved 32 -> 33**: computed by running
/// `cargo test -p mtg-engine --test core protocol_schema`, which failed on the
/// unmodified fingerprint and printed the recomputed digest
/// (`a153b6655890ccb3335d83678d7145b27358716334ef0971b898a3a54b4997f6`) used to
/// re-pin `PROTOCOL_SCHEMA_FINGERPRINT` and append the new `PROTOCOL_HISTORY` row --
/// `Command::TurnFaceUp` and `Command::DeclareAttackers` both gain
/// `hybrid_choices`/`phyrexian_life_payments`, changing two declared shapes in the
/// wire closure (closure type count unchanged at 96; exact precedent: `- 27: PB-RS2`).
///
/// `HASH_SCHEMA_VERSION` **stays 70**: computed by running
/// `cargo test -p mtg-engine --test core hash_schema`, which passed unmodified --
/// `Command` has no `HashInto` impl, no `GameState` field was added, and no hashed
/// struct changed shape (`GameRestriction::CantAttackYouUnlessPay`'s
/// `cost_per_creature: ManaCost` is unchanged in shape).
fn pb_dx6_wire_versions() {
    assert_eq!(
        PROTOCOL_VERSION, 43,
        "PROTOCOL_VERSION live sentinel -- PB-DX6 moved it 32->33 (Command::TurnFaceUp \
         + Command::DeclareAttackers both gain hybrid_choices/phyrexian_life_payments)"
    );
    assert_eq!(
        HASH_SCHEMA_VERSION, 84,
        "HASH_SCHEMA_VERSION live sentinel -- PB-DX21 moved it 72->73 (CombatState \
         gains attackers_declared); PB-DX6 itself left it unmoved at 70 \
         (Command has no HashInto impl; no GameState field added)"
    );
}
