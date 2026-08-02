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
    CardDefinition, CardRegistry, Command, FaceDownKind, GameRestriction, GameState,
    GameStateBuilder, HybridMana, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    TurnFaceUpMethod, ZoneId,
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
    }
}

// ── Observation A — turn-face-up, DEBUG build: panics (plan §2.0) ──────────────

#[test]
/// PRE-FIX / DEBUG build. No engine code touched — this is `cargo test` run exactly
/// as CI runs it (`crates/engine/tests/primitives`, default profile, `debug_assertions
/// = true`).
///
/// Builds a face-down battlefield object over the REAL, corpus `kitchen_finks` def
/// (`Completeness::Complete`, `{1}{G/W}{G/W}`), sets `face_down_as:
/// Some(FaceDownKind::Manifest)`, controller P1 holding priority, and sends
/// `Command::TurnFaceUp { method: ManaCost, .. }`. `handle_turn_face_up`
/// (`rules/engine.rs`) calls `player_state.mana_pool.can_spend(&mana_cost, None)` on
/// the raw, unflattened cost; `can_spend`'s first statement is
/// `debug_assert_flattened(cost)` (`crates/card-types/src/state/player.rs`), which
/// panics because the cost still carries 2 hybrid pips.
///
/// The panic is captured with `std::panic::catch_unwind` (not `#[should_panic]`) so
/// the literal downcast payload can be asserted and recorded verbatim, rather than
/// merely checked for a substring.
///
/// OBSERVED (this run, debug build, no source touched):
/// "unflattened mana cost reached the payment path: 2 hybrid + 0 Phyrexian pip(s) \
///  would be paid for free (CR 107.4e/107.4f). Call ManaCost::flatten_hybrid_phyrexian \
///  first. cost = ManaCost { white: 0, blue: 0, black: 0, red: 0, green: 0, colorless: 0, \
///  generic: 1, hybrid: [ColorColor(Green, White), ColorColor(Green, White)], \
///  phyrexian: [], x_count: 0 }"
/// — the pool has zero mana; note the panic fires before any affordability check, so
/// the pool's contents are irrelevant to whether this panics.
fn observation_a_turn_face_up_panics_in_debug_build() {
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

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        process_command(
            state,
            Command::TurnFaceUp {
                player: p1,
                permanent: finks_id,
                method: TurnFaceUpMethod::ManaCost,
            },
        )
    }));

    let payload = result.expect_err(
        "PRE-FIX debug-build expectation (plan §2.0): handle_turn_face_up must panic \
         inside debug_assert_flattened when the raw hybrid cost reaches can_spend. If \
         this assertion fails, the engine has already been fixed (or the guard was \
         removed) and this stage-0 observation is stale.",
    );
    let msg = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "<non-string panic payload>".to_string());

    assert!(
        msg.contains("unflattened mana cost reached the payment path"),
        "panic message did not match the expected debug_assert_flattened text: {msg}"
    );
    assert!(
        msg.contains("2 hybrid + 0 Phyrexian pip(s)"),
        "panic message did not name 2 hybrid / 0 Phyrexian pips (Kitchen Finks' \
         {{1}}{{G/W}}{{G/W}}): {msg}"
    );
    assert!(
        msg.contains("CR 107.4e/107.4f"),
        "panic message did not cite CR 107.4e/107.4f: {msg}"
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
