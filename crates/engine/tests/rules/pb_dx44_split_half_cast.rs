//! PB-DX44 stage 2a (`OOS-DX29-9`) — casting ONLY the right half of a split card.
//!
//! # What was wrong
//!
//! CR 702.102a and CR 709.4: a player may cast either half of a split card, or (with
//! Fuse) both. `AbilityDefinition::Spell` (the LEFT half) is resolved for every
//! non-fused cast, and `AbilityDefinition::Fuse` (the DSL's storage for the RIGHT half)
//! was reachable ONLY under `AdditionalCost::Fuse`. There was no `Command` shape that
//! selected the right half alone: `Turn // Burn` could be cast as Turn, or as Turn+Burn
//! (fused), but never as Burn by itself.
//!
//! # Engine surface under test
//!
//! `AltCostKind::SplitRightHalf` (new, `crates/card-types/src/state/types.rs`),
//! `StackObject.cast_right_half` (new, hashed), `casting.rs`'s `cast_right_half`
//! derivation and its cost/target/validation arms, `card_def_target_requirements`'s
//! third mode (REPLACE, not append), and `resolution.rs`'s right-half effect dispatch
//! plus its `left_count` target-index padding (the silent-wrong-game-state hazard this
//! whole stage exists to close — see `t1`/`t2`'s doc for how each test discriminates it).
//!
//! # CR index
//!
//! CR 601.2a (announcing which half), CR 702.102a (Fuse is a static ability; a
//! right-half-only cast is NOT fusing), CR 709.4 (a split card has two halves, each with
//! its own name, mana cost and text; while on the stack as one half, the spell has only
//! that half's characteristics), CR 608.2b (fizzle / target legality).
//!
//! See `crates/engine/tests/core/pb_dx44_uncastable_roster.rs` for the corpus census
//! (`r2`: the deck-legal right-half population is `Turn // Burn`, `Wear // Tear`,
//! `Connive // Concoct` — THREE, not the two that are also fusable; `r3`: the per-half
//! declared target counts, 1/1/0 respectively; `r7`: every right half's `card_type`
//! matches its card's printed type, the CR 709.4 timing residual `casting.rs` documents
//! beside `is_instant_speed`) — not duplicated here.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AdditionalCost,
    AltCostKind, CardDefinition, Color, Command, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaPool, ObjectId, ObjectSpec, PlayerId, SubType, Target, ZoneId,
};
use std::collections::HashMap;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// ── Fixture plumbing (mirrors `pb_dx44_fuse_targets.rs`) ────────────────────────────

fn defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// A real corpus card, fully enriched, in `zone`. `card_id` is looked up from the def
/// rather than minted from the printed name — several split cards key on the front face
/// alone (`pb_dx44_uncastable_roster.rs`'s `r6`), and using the wrong id here builds an
/// object the registry does not recognise (SR-38).
fn corpus_object(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("{name:?} is not in `all_cards()`"));
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .with_card_id(def.card_id.clone())
            .in_zone(zone),
        defs,
    )
}

fn corpus_registry(
    defs: &HashMap<String, CardDefinition>,
) -> std::sync::Arc<mtg_engine::CardRegistry> {
    mtg_engine::CardRegistry::new(defs.values().cloned())
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

/// Like `pb_dx44_fuse_targets.rs`'s `cast` helper, but with `alt_cost` exposed — that
/// file hardcodes `alt_cost: None` because it only ever drives the FUSED path (via
/// `AdditionalCost::Fuse`), where this file's whole subject is the `alt_cost` channel.
#[allow(clippy::too_many_arguments)]
fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    alt_cost: Option<AltCostKind>,
    additional_costs: Vec<AdditionalCost>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
}

/// Round-robin priority passes until the stack is empty.
fn resolve_stack(mut state: GameState) -> GameState {
    for _ in 0..20 {
        if state.stack_objects().is_empty() {
            return state;
        }
        for pl in [P1, P2] {
            let (next, _) = process_command(state.clone(), Command::PassPriority { player: pl })
                .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
            state = next;
        }
    }
    panic!("stack did not empty in 20 priority rounds");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T1/T2/T3 — END TO END: right-half-only casts run ONLY the right half, on ITS OWN
// targets, at ITS OWN cost. Each is the index-hazard discriminator for its def: if
// `resolution.rs`'s `left_count` padding were absent or wrong, the right half's
// `DeclaredTarget { index: left_count + i }` reads out of range and resolves at
// NOTHING (silent wrong game state, not a refusal) -- proven by an executed revert
// (temporarily short-circuiting the padding branch to `legal_targets` unmodified
// reddens t1 and t2 on exactly this assertion, and leaves t3 green vacuously since
// Concoct declares no targets at all -- which is why t3 alone cannot discriminate the
// hazard and t1/t2 are load-bearing).
// ═══════════════════════════════════════════════════════════════════════════════

/// **T1** — `Turn // Burn`, right-half-only: casts Burn ALONE.
///
/// Proves, on one cast:
/// - (a) Burn's effect landed on the announced target (P2 loses 2 life) — the index
///   hazard discriminator: Burn's `DeclaredTarget` reads index 1 in the def's own
///   authoring convention (CR 702.102d, written assuming a FUSED cast's combined index
///   space), while a right-half-only cast announces exactly ONE target, which without
///   `left_count` padding lands at index 0 and Burn resolves at nothing.
/// - (b) Turn's effect (the LEFT half) did NOT run: an untargeted creature on the
///   battlefield keeps its printed power/toughness, subtypes and colors — Turn would
///   have set them to 0/1 Weird Red were it (wrongly) executed.
/// - (c) the mana charged is Burn's OWN cost (`{1}{R}`), not Turn's (`{2}{U}`) and not
///   the combined fused cost (`{2}{U}` + `{1}{R}`) — funded with EXACTLY `{1}{R}` and
///   the pool is empty afterward, so a wrong charge in either direction would either
///   refuse the cast (undercharge relative to what's funded) or leave mana stranded
///   (overcharge).
#[test]
fn t1_turn_burn_right_half_only_casts_burn_alone() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                red: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
        // Untargeted bystander: if Turn (left half) wrongly ran, this creature's base
        // P/T becomes 0/1, its subtypes become exactly {Weird}, and its colors become
        // exactly {Red}. It is never announced as a target.
        .object(
            ObjectSpec::creature(P2, "Bystander Bear", 3, 3)
                .with_subtypes(vec![SubType("Bear".to_string())])
                .with_colors(vec![Color::Green]),
        )
        .build()
        .expect("state builds");

    let card = id_of(&state, "Turn // Burn");
    let bystander = id_of(&state, "Bystander Bear");
    let p2_life_before = state.player(P2).expect("p2 exists").life_total;

    let (after_cast, _events) = cast(
        state,
        P1,
        card,
        vec![Target::Player(P2)],
        Some(AltCostKind::SplitRightHalf),
        vec![],
    )
    .expect(
        "a right-half-only cast announcing exactly Burn's one target, funded exactly, \
         must be accepted",
    );

    // (c) exact charge: the pool held precisely Burn's cost and nothing is left over.
    assert!(
        after_cast
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "Burn's own cost ({{1}}{{R}}) must be charged exactly -- a wrong charge (Turn's \
         {{2}}{{U}} or the fused {{3}}{{U}}{{R}}) could not have been paid from this pool \
         at all, so reaching this point already proves the amount; emptiness proves no \
         surplus was silently left unspent"
    );

    let resolved = resolve_stack(after_cast);

    // (a) the index hazard discriminator: Burn landed on the announced player target.
    let p2_life_after = resolved.player(P2).expect("p2 exists").life_total;
    assert_eq!(
        p2_life_after,
        p2_life_before - 2,
        "Burn deals 2 damage to any target; a right-half-only cast must resolve the \
         RIGHT half's own effect on the announced target. If the effect context's target \
         list is not padded to Burn's globally-offset `DeclaredTarget {{ index: 1 }}`, \
         this assertion is exactly what goes red: the effect resolves at nothing and life \
         stays unchanged"
    );

    // (b) Turn (left half) did not run.
    let bystander_chars = calculate_characteristics(&resolved, bystander)
        .expect("the untargeted bystander must still exist");
    assert_eq!(
        bystander_chars.power,
        Some(3),
        "Turn's base-power-0 effect must NOT apply -- Turn (left half) never ran"
    );
    assert_eq!(bystander_chars.toughness, Some(3), "same, for toughness");
    assert!(
        bystander_chars
            .subtypes
            .contains(&SubType("Bear".to_string())),
        "Turn's SetCreatureTypes(Weird) must NOT apply"
    );
    assert!(
        bystander_chars.colors.contains(&Color::Green),
        "Turn's SetColors(Red) must NOT apply"
    );
}

/// **T2** — `Wear // Tear`, right-half-only: casts Tear ALONE.
///
/// Same three-part proof as `t1`, with the roles of "index hazard discriminator" and
/// "left half did not run" split across two DIFFERENT permanents (an artifact and an
/// enchantment) so each can be asserted independently: Tear (right half, index 1)
/// destroys the announced enchantment; Wear (left half) never runs, so an UNTARGETED
/// artifact on the battlefield survives.
#[test]
fn t2_wear_tear_right_half_only_casts_tear_alone() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                white: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Wear // Tear", ZoneId::Hand(P1)))
        // Untargeted bystander: if Wear (left half) wrongly ran, this artifact would be
        // destroyed. It is never announced as a target.
        .object(ObjectSpec::artifact(P2, "Bystander Artifact").in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::enchantment(P2, "Doomed Enchantment").in_zone(ZoneId::Battlefield))
        .build()
        .expect("state builds");

    let card = id_of(&state, "Wear // Tear");
    let bystander_artifact = id_of(&state, "Bystander Artifact");
    let enchantment = id_of(&state, "Doomed Enchantment");

    let (after_cast, _events) = cast(
        state,
        P1,
        card,
        vec![Target::Object(enchantment)],
        Some(AltCostKind::SplitRightHalf),
        vec![],
    )
    .expect(
        "a right-half-only cast announcing exactly Tear's one target, funded exactly \
         with {W}, must be accepted",
    );

    assert!(
        after_cast
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "Tear's own cost ({{W}}) must be charged exactly, not Wear's ({{1}}{{R}}) and not \
         the fused combined cost"
    );

    let resolved = resolve_stack(after_cast);

    // The index hazard discriminator: Tear (right half, index 1) landed on the
    // announced enchantment.
    assert!(
        !resolved.objects().contains_key(&enchantment),
        "CR 400.7: the destroyed enchantment's pre-destruction ObjectId must be dead -- \
         Tear's `DeclaredTarget {{ index: 1 }}` must resolve against the announced \
         target, which requires the effect context's target list to be padded with \
         Wear's declared-requirement count (1) ahead of the announced target"
    );
    assert!(
        resolved
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Doomed Enchantment"
                && matches!(o.zone, ZoneId::Graveyard(_))),
        "the enchantment must be in the graveyard, not merely gone from the battlefield"
    );

    // Wear (left half) did not run: the untargeted artifact survives.
    assert!(
        resolved.objects().contains_key(&bystander_artifact),
        "Wear's DestroyPermanent effect must NOT apply -- Wear (left half) never ran, \
         so the untargeted artifact must still be on the battlefield"
    );
}

/// **T3** — `Connive // Concoct`, right-half-only: casts Concoct ALONE, with ZERO
/// announced targets (`pb_dx44_uncastable_roster::r3`: Concoct declares no target at
/// all — CR 115.10). Exercises the padding path with an EMPTY announced list: `r3`
/// pins the left half (Connive) at 1 declared requirement, so the padded context still
/// gets one `unchosen_slot` filler even though nothing is ever announced.
///
/// The library is left EMPTY so Concoct's `Surveil 3` clause raises no
/// `EffectChoiceQuestion` (CR 701.25c / `Effect::Surveil`'s own empty-library no-op),
/// and exactly ONE eligible creature card sits in the graveyard so the "return a
/// creature card" clause is CR 601.2c-DETERMINED (no question either) — the test proves
/// Concoct's effect landed without needing to drive a suspended-choice round trip,
/// which is out of scope for this stage.
///
/// Proves:
/// - Concoct's effect ran: the sole graveyard creature moved to the battlefield.
/// - Connive's effect (the LEFT half, "gain control of target creature") did NOT run:
///   an eligible bystander creature (power <= 2, matching Connive's own filter) that
///   Connive COULD have targeted stays under its original controller's control. Since a
///   right-half-only cast's requirement list is Concoct's alone (empty), Connive's
///   target is never even OFFERED — this is a structural guarantee as much as a
///   behavioural one, kept as a regression guard against `card_def_target_requirements`
///   ever being changed from REPLACE to APPEND for this mode.
/// - the mana charged is Concoct's own cost (`{3}{U}{B}`), not Connive's hybrid
///   (`{2}{U/B}{U/B}`).
#[test]
fn t3_connive_concoct_right_half_only_casts_concoct_alone_untargeted() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 1,
                black: 1,
                colorless: 3,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Connive // Concoct",
            ZoneId::Hand(P1),
        ))
        // The sole eligible creature card for Concoct's "return a creature card from
        // your graveyard" clause -- exactly one, so the choice is CR 601.2c-determined.
        .object(ObjectSpec::creature(P1, "Returning Creature", 2, 2).in_zone(ZoneId::Graveyard(P1)))
        // Bystander: a legal Connive target (power 1 <= 2) Connive would gain control
        // of if the LEFT half wrongly ran. Never announced as a target.
        .object(ObjectSpec::creature(P2, "Bystander Imp", 1, 1))
        .build()
        .expect("state builds");

    let card = id_of(&state, "Connive // Concoct");
    let bystander = id_of(&state, "Bystander Imp");

    let (after_cast, _events) = cast(
        state,
        P1,
        card,
        vec![],
        Some(AltCostKind::SplitRightHalf),
        vec![],
    )
    .expect(
        "a right-half-only Concoct cast announcing ZERO targets (CR 115.10: Concoct \
         prints no \"target\"), funded exactly with {3}{U}{B}, must be accepted",
    );

    assert!(
        after_cast
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "Concoct's own cost ({{3}}{{U}}{{B}}) must be charged exactly, not Connive's \
         hybrid cost ({{2}}{{U/B}}{{U/B}})"
    );

    let resolved = resolve_stack(after_cast);

    // Concoct's effect ran: the sole graveyard creature is now on the battlefield.
    let returned = resolved
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Returning Creature")
        .expect("the returned creature must still exist under SOME ObjectId");
    assert_eq!(
        returned.zone,
        ZoneId::Battlefield,
        "Concoct's MoveZone{{..to: Battlefield}} must have run -- the only eligible \
         graveyard creature must now be on the battlefield"
    );

    // Connive's effect (left half) did not run.
    let bystander_owner = resolved
        .objects()
        .get(&bystander)
        .expect("bystander still exists (never targeted, never destroyed)");
    assert_eq!(
        bystander_owner.controller, P2,
        "Connive's GainControl effect must NOT apply -- the bystander creature Connive \
         could have targeted stays under its original controller"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// T4/T5 — REGRESSION FLOOR: a plain left-half cast and a FUSED cast are unaffected by
// this stage's changes.
// ═══════════════════════════════════════════════════════════════════════════════

/// **T4** — `Turn // Burn` cast NORMALLY (`alt_cost: None`, no `AdditionalCost::Fuse`):
/// the left half (Turn) still runs, at Turn's own cost, unaffected by the new
/// `cast_right_half` branch existing alongside it.
#[test]
fn t4_left_half_only_cast_still_works_unchanged() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                blue: 1,
                colorless: 2,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
        .object(
            ObjectSpec::creature(P2, "Target Bear", 3, 3)
                .with_subtypes(vec![SubType("Bear".to_string())])
                .with_colors(vec![Color::Green]),
        )
        .build()
        .expect("state builds");

    let card = id_of(&state, "Turn // Burn");
    let target = id_of(&state, "Target Bear");

    let (after_cast, _events) = cast(state, P1, card, vec![Target::Object(target)], None, vec![])
        .expect("a plain, unfused, non-right-half cast of Turn must still be accepted");
    assert!(
        after_cast
            .player(P1)
            .expect("p1 exists")
            .mana_pool
            .is_empty(),
        "Turn's own cost ({{2}}{{U}}) must be charged, unaffected by this stage"
    );

    let resolved = resolve_stack(after_cast);
    let chars = calculate_characteristics(&resolved, target).expect("target still exists");
    assert_eq!(chars.power, Some(0), "Turn still sets base power to 0");
    assert_eq!(
        chars.toughness,
        Some(1),
        "Turn still sets base toughness to 1"
    );
}

/// **T5** — `Wear // Tear`, FUSED (`AdditionalCost::Fuse`): both halves still resolve on
/// their own targets, unaffected by the new `cast_right_half` branch. Full coverage of
/// the fused path lives in `pb_dx44_fuse_targets.rs` (Stage 1) — this is a floor, not a
/// re-derivation.
#[test]
fn t5_fused_cast_still_works_unchanged() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                white: 1,
                red: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Wear // Tear", ZoneId::Hand(P1)))
        .object(ObjectSpec::artifact(P2, "Doomed Artifact").in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::enchantment(P2, "Doomed Enchantment 2").in_zone(ZoneId::Battlefield))
        .build()
        .expect("state builds");

    let card = id_of(&state, "Wear // Tear");
    let artifact = id_of(&state, "Doomed Artifact");
    let enchantment = id_of(&state, "Doomed Enchantment 2");

    let (after_cast, _events) = cast(
        state,
        P1,
        card,
        vec![Target::Object(artifact), Target::Object(enchantment)],
        None,
        vec![AdditionalCost::Fuse],
    )
    .expect("a fused Wear // Tear cast must still be accepted");

    let resolved = resolve_stack(after_cast);
    assert!(
        !resolved.objects().contains_key(&artifact),
        "Wear (left half, fused) must still destroy the artifact"
    );
    assert!(
        !resolved.objects().contains_key(&enchantment),
        "Tear (right half, fused) must still destroy the enchantment"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// T6/T7/T8 — GUARDS: each negative case, asserted on the error message.
// ═══════════════════════════════════════════════════════════════════════════════

/// **T6** — `SplitRightHalf` combined with `AdditionalCost::Fuse` is refused. This is
/// the pre-existing `casting_with_fuse` block's own guard (`alt_cost.is_some()` rejects
/// ANY alt cost when fusing) — `casting.rs`'s new `cast_right_half` validation block
/// deliberately does NOT duplicate this check (see its own doc comment: a second check
/// there would be dead code, since this guard already returns first).
#[test]
fn t6_split_right_half_combined_with_fuse_is_refused() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                white: 1,
                red: 1,
                colorless: 5,
                blue: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Wear // Tear", ZoneId::Hand(P1)))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Wear // Tear");

    let refused = cast(
        state,
        P1,
        card,
        vec![],
        Some(AltCostKind::SplitRightHalf),
        vec![AdditionalCost::Fuse],
    );
    match refused {
        Err(GameStateError::InvalidCommand(msg)) => {
            assert!(
                msg.contains("cannot combine fuse with an alternative cost"),
                "expected the fuse block's own mutual-exclusion message; got: {msg}"
            );
        }
        other => panic!("expected GameStateError::InvalidCommand, got {other:?}"),
    }
}

/// **T7** — `SplitRightHalf` on a card with no `AbilityDefinition::Fuse` (an ordinary,
/// non-split card) is refused with a CR 709.4 message naming the missing right half.
///
/// **The assertion is an EXACT string match, not a substring check, and that precision
/// is load-bearing.** `casting.rs` checks `get_fuse_data(..).is_none()` at TWO sites --
/// the `cast_right_half` validation block this test targets, and (independently) the
/// cost-derivation `else if cast_right_half` arm further down, which re-checks the same
/// helper as defense-in-depth and returns a SIMILAR but not identical message ("...no
/// AbilityDefinition::Fuse (right half) COST defined..."). A substring check like
/// `msg.contains("no AbilityDefinition::Fuse")` would pass against EITHER site and could
/// not tell "the validation block fired" from "the validation block was skipped and the
/// cost block caught it instead" -- proven by an executed revert (temporarily
/// short-circuiting the validation block's own check left this test GREEN, because the
/// cost block's redundant check produced a message the old substring assertion still
/// matched). The exact-string assertion below is what makes that revert go RED.
#[test]
fn t7_split_right_half_on_a_non_split_card_is_refused() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Lightning Bolt", ZoneId::Hand(P1)))
        .object(ObjectSpec::creature(P2, "Some Creature", 2, 2))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Lightning Bolt");
    let target = id_of(&state, "Some Creature");

    let refused = cast(
        state,
        P1,
        card,
        vec![Target::Object(target)],
        Some(AltCostKind::SplitRightHalf),
        vec![],
    );
    match refused {
        Err(GameStateError::InvalidCommand(msg)) => {
            assert_eq!(
                msg,
                "split right half: card has no AbilityDefinition::Fuse (right half) \
                 defined (CR 709.4)",
                "expected the VALIDATION block's own message (not the cost block's \
                 similar-but-different redundant one); got: {msg}"
            );
        }
        other => panic!("expected GameStateError::InvalidCommand, got {other:?}"),
    }
}

/// **T8** — `SplitRightHalf` cast from the graveyard (not hand) is refused. CR 709.4's
/// single-half cast has no independent alternative zone of its own (unlike Aftermath,
/// which is specifically a graveyard-only channel for the SECOND half).
///
/// **The refusal comes from `casting.rs`'s pre-existing GENERAL "card is not in your
/// hand" zone-legality gate, not from a `cast_right_half`-specific message.** That gate
/// runs before `cast_right_half`'s own validation block even starts, and
/// `SplitRightHalf` is deliberately NOT one of its exemptions (unlike, e.g.,
/// `casting_with_aftermath`, which the gate DOES exempt because Aftermath legitimately
/// casts from the graveyard) — so a second, `cast_right_half`-local "must be in hand"
/// check would be dead code, and `casting.rs`'s own comment says so rather than shipping
/// an unreachable branch.
#[test]
fn t8_split_right_half_from_graveyard_is_refused() {
    let defs = defs_by_name();
    let state = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(corpus_registry(&defs))
        .active_player(P1)
        .player_mana(
            P1,
            ManaPool {
                red: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(corpus_object(
            &defs,
            P1,
            "Turn // Burn",
            ZoneId::Graveyard(P1),
        ))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Turn // Burn");

    let refused = cast(
        state,
        P1,
        card,
        vec![Target::Player(P2)],
        Some(AltCostKind::SplitRightHalf),
        vec![],
    );
    match refused {
        Err(GameStateError::InvalidCommand(msg)) => {
            assert_eq!(
                msg, "card is not in your hand",
                "expected the pre-existing general zone-legality gate's message -- \
                 `cast_right_half` is not one of that gate's zone exemptions, so this is \
                 where the refusal comes from; got: {msg}"
            );
        }
        other => panic!("expected GameStateError::InvalidCommand, got {other:?}"),
    }
}
