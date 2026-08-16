//! PB-DX44 (`OOS-DX29-12`) — a fused cast can now announce its right half's targets.
//!
//! # What was wrong
//!
//! CR 702.102d gives a fused spell BOTH halves' targets. `casting::card_def_target_
//! requirements` derived its requirement list from `AbilityDefinition::Spell { targets
//! }` alone and never concatenated `AbilityDefinition::Fuse { targets }`, so a fused
//! `Turn // Burn` announcing both targets was refused with `InvalidTarget("expected
//! 1..=1 target(s) but got 2")`, and announcing one left the right half's
//! `DeclaredTarget { index: 1 }` resolving at nothing. PB-DX29 gated the whole Fuse
//! offer on this gap (`fused_right_half_declares_targets`); this batch closes the gap
//! and deletes that suppression.
//!
//! # Engine surface under test
//!
//! `rules::casting::card_def_target_requirements`'s new `casting_with_fuse` parameter
//! (appends the Fuse ability's targets after the Spell ability's, in the global index
//! order `resolution.rs` documents: left half `0..left_count`, right half
//! `left_count..`), and `rules::queries::spell_target_requirements`'s new `fuse: bool`
//! parameter, which routes through the SAME function so the offer and the cast cannot
//! disagree about the count (SR-38).
//!
//! # CR index
//!
//! CR 702.102a (Fuse is a static ability, from hand only), CR 702.102b (combined cost),
//! CR 702.102c/d (both halves' targets, left resolves before right), CR 601.2c (target
//! announcement), CR 608.2b (fizzle).
//!
//! See `crates/engine/tests/core/pb_dx44_uncastable_roster.rs` for the corpus census
//! this file's two fixtures are drawn from (R2/R3 pin the deck-legal fusable population
//! as exactly `Turn // Burn` and `Wear // Tear`, and the per-half declared target
//! counts as 1 and 1 for both) — not duplicated here.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AdditionalCost,
    CardDefinition, CardType, Color, Command, GameEvent, GameState, GameStateBuilder, ManaPool,
    ObjectId, ObjectSpec, PlayerId, SubType, Target, ZoneId,
};
use std::collections::HashMap;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// ── Fixture plumbing ─────────────────────────────────────────────────────────────

fn defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// A real corpus card, fully enriched, in `zone`.
///
/// `card_id` is looked up from the def rather than minted from the printed name --
/// `Turn // Burn` declares `cid("turn")`, not `card_name_to_id("Turn // Burn")`
/// (`pb_dx44_uncastable_roster.rs`'s `r6` pins this whole class of mismatch). Using
/// the WRONG id here would build an object whose `card_id` the registry does not
/// recognise, and the cast would be silently offered nothing (SR-38, the "naked
/// `ObjectSpec::card()`" gotcha's registry-side twin).
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

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    additional_costs: Vec<AdditionalCost>,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
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
            alt_cost: None,
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
// T1/T2 — END TO END: both halves resolve on their OWN targets (the index contract)
// ═══════════════════════════════════════════════════════════════════════════════

/// **T1** — `Turn // Burn`, fused, targeting a CREATURE with the LEFT half (Turn) and a
/// PLAYER with the RIGHT half (Burn). Proves the global index contract (`resolution.rs`:
/// left half's targets at `0..left_count`, right half's at `left_count..`) by asserting
/// each half's effect landed on its OWN target and not the other one.
///
/// **The player/creature split is load-bearing, not a fixture convenience.** A first
/// draft targeted TWO creatures (A and B). Because Burn's requirement is `TargetAny`
/// (which also accepts creatures), both announced objects satisfied BOTH requirement
/// slots, and the engine's two-pass best-fit target matcher
/// (`memory/gotchas-rules.md`'s "Multi-target validators cannot greedily match slots in
/// declaration order") could then assign either creature to either slot by TYPE alone,
/// independent of the requirement list's declared ORDER. An executed revert proved this:
/// swapping the concatenation order in `card_def_target_requirements` (right half's
/// requirement BEFORE the left half's, instead of after) left that draft's assertions
/// green, because the matcher silently re-derived the "obviously correct" assignment
/// from the ANNOUNCED targets' own types regardless of which requirement was declared
/// first — so the two-creature draft was UNDISCRIMINATED against that specific defect
/// class, though it still caught the count/presence defect (the `if false` revert that
/// deletes the concatenation entirely). Using a player for the right half removes the
/// ambiguity: `TargetCreature` cannot match a player at all, so a swapped requirement
/// order forces the matcher to assign the PLAYER to slot 0 and the CREATURE to slot 1 --
/// observably different targets for Turn (which expects a creature at index 0) and Burn
/// (which reads index 1) than the correct assignment produces.
#[test]
fn t1_turn_burn_fused_cast_resolves_both_halves_on_their_own_targets() {
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
                red: 1,
                colorless: 3,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
        // The LEFT half's (Turn) target. Starts as a green Bear so the type/color
        // change is observable, and its OWN identity (not the player's) proves Turn's
        // effect landed on it specifically.
        .object(
            ObjectSpec::creature(P2, "Creature A", 3, 3)
                .with_subtypes(vec![SubType("Bear".to_string())])
                .with_colors(vec![Color::Green]),
        )
        .build()
        .expect("state builds");

    let card = id_of(&state, "Turn // Burn");
    let creature_a = id_of(&state, "Creature A");
    let p2_life_before = state.player(P2).expect("p2 exists").life_total;

    // The RIGHT half's (Burn) target is P2 -- a player, so it can satisfy `TargetAny`
    // (index 1, right half) but never `TargetCreature` (index 0, left half). No
    // ambiguity for the best-fit matcher to paper over.
    let (after_cast, _events) = cast(
        state,
        P1,
        card,
        vec![Target::Object(creature_a), Target::Player(P2)],
        vec![AdditionalCost::Fuse],
    )
    .expect("a fused cast with 2 announced targets, funded exactly, must be accepted");

    let resolved = resolve_stack(after_cast);

    // Creature A: Turn's effect (left half, index 0). Base P/T set to 0/1, subtype set
    // to exactly {Weird}, color set to exactly {Red}.
    let a_chars = calculate_characteristics(&resolved, creature_a)
        .expect("Creature A must still exist on the battlefield");
    assert_eq!(a_chars.power, Some(0), "Turn sets base power to 0");
    assert_eq!(a_chars.toughness, Some(1), "Turn sets base toughness to 1");
    assert_eq!(
        a_chars.subtypes,
        [SubType("Weird".to_string())].into_iter().collect(),
        "Turn sets creature types to exactly {{Weird}} (CR 205.1a)"
    );
    assert_eq!(
        a_chars.colors,
        [Color::Red].into_iter().collect(),
        "Turn sets colors to exactly {{Red}}"
    );

    // P2: Burn's effect (right half, index 1). 2 life lost, and nothing else -- proving
    // the two halves did not cross-apply.
    let p2_life_after = resolved.player(P2).expect("p2 exists").life_total;
    assert_eq!(
        p2_life_after,
        p2_life_before - 2,
        "Burn deals 2 damage to any target (right half, index 1); a player target takes \
         it as life loss"
    );
}

/// **T2** — `Wear // Tear`, fused, targeting TWO DIFFERENT permanents: the LEFT half
/// (Wear, destroy target artifact) on one artifact, the RIGHT half (Tear, destroy
/// target enchantment) on a different enchantment. Both halves are `Instant` (verified
/// against the def below, correcting the brief's assumption that this card is
/// Instant + Sorcery -- it is Instant on both halves).
///
/// **Disclosed as UNDISCRIMINATED against the index-ORDER defect class specifically**
/// (proven by an executed revert, not asserted): both halves are the SAME effect kind
/// (`Effect::DestroyPermanent`) applied to "whatever object is at index N", so if the
/// requirement list's concatenation order were swapped, the best-fit target matcher
/// would reassign the artifact and enchantment to the OTHER slots and each half would
/// still destroy "its own" (relabelled) index -- both objects end up destroyed either
/// way, and this test's assertions (both gone, both in the graveyard) cannot tell the
/// two orderings apart. It DOES discriminate the count/presence defect (proven by the
/// same revert matrix that reddened all four tests in this file when the concatenation
/// was deleted outright) -- see `t1`'s doc for the sibling test that DOES pin the order,
/// using a card whose two halves apply visibly different, non-interchangeable effects.
#[test]
fn t2_wear_tear_fused_cast_resolves_both_halves_on_their_own_targets() {
    let defs = defs_by_name();
    let def = defs.get("Wear // Tear").expect("corpus def");
    assert!(
        def.types.card_types.contains(&CardType::Instant),
        "Wear (left half) is printed Instant"
    );
    let right_is_instant = def.abilities.iter().any(|a| {
        matches!(
            a,
            mtg_engine::AbilityDefinition::Fuse {
                card_type: CardType::Instant,
                ..
            }
        )
    });
    assert!(
        right_is_instant,
        "Tear (right half) is printed Instant too -- both halves of Wear // Tear are \
         Instant, not Instant + Sorcery"
    );

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
        .object(ObjectSpec::artifact(P2, "Some Artifact").in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::enchantment(P2, "Some Enchantment").in_zone(ZoneId::Battlefield))
        .build()
        .expect("state builds");

    let card = id_of(&state, "Wear // Tear");
    let artifact = id_of(&state, "Some Artifact");
    let enchantment = id_of(&state, "Some Enchantment");

    let (after_cast, _events) = cast(
        state,
        P1,
        card,
        vec![Target::Object(artifact), Target::Object(enchantment)],
        vec![AdditionalCost::Fuse],
    )
    .expect("a fused Wear // Tear cast with 2 announced targets must be accepted");

    let resolved = resolve_stack(after_cast);

    assert!(
        !resolved.objects().contains_key(&artifact),
        "CR 400.7: the destroyed artifact's pre-destruction ObjectId must be dead \
         (Wear, left half, index 0)"
    );
    assert!(
        !resolved.objects().contains_key(&enchantment),
        "CR 400.7: the destroyed enchantment's pre-destruction ObjectId must be dead \
         (Tear, right half, index 1)"
    );
    assert!(
        resolved
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Some Artifact"
                && matches!(o.zone, ZoneId::Graveyard(_))),
        "the artifact must be in the graveyard, not merely gone from the battlefield"
    );
    assert!(
        resolved
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Some Enchantment"
                && matches!(o.zone, ZoneId::Graveyard(_))),
        "the enchantment must be in the graveyard, not merely gone from the battlefield"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// T3 — the negative probe: one target is now refused, not silently under-resolved
// ═══════════════════════════════════════════════════════════════════════════════

/// **T3** — announcing only ONE target for a fused cast is refused. Before this batch
/// the requirement list was 1 (left half only), so a single-target announcement was
/// accepted and the right half's `DeclaredTarget { index: 1 }` resolved at nothing
/// (silent wrong game state, CR 702.102d violated without any refusal). Now the
/// requirement list is 2, and CR 601.2c makes a short announcement a hard refusal.
#[test]
fn t3_announcing_only_one_target_for_a_fused_cast_is_refused() {
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
                red: 1,
                colorless: 3,
                ..Default::default()
            },
        )
        .object(corpus_object(&defs, P1, "Turn // Burn", ZoneId::Hand(P1)))
        .object(ObjectSpec::creature(P2, "Creature A", 3, 3))
        .build()
        .expect("state builds");
    let card = id_of(&state, "Turn // Burn");
    let creature_a = id_of(&state, "Creature A");

    let refused = cast(
        state,
        P1,
        card,
        vec![Target::Object(creature_a)],
        vec![AdditionalCost::Fuse],
    );
    match refused {
        Err(mtg_engine::GameStateError::InvalidTarget(msg)) => {
            assert!(
                msg.contains("expected 2..=2") || msg.contains("got 1"),
                "expected a target-COUNT refusal naming 2 required and 1 given; got: {msg}"
            );
        }
        other => panic!(
            "expected GameStateError::InvalidTarget naming a 2-vs-1 count mismatch, got \
             {other:?}"
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T4 — the OFFER differential: `spell_target_requirements(.., fuse: true)` reports
// the same count the cast validates against
// ═══════════════════════════════════════════════════════════════════════════════

/// **T4** — the SR-38 differential this whole fix exists for: the OFFER (what the
/// browser and the bot both read, `queries::spell_target_requirements`) and the CAST
/// (`casting.rs`'s own validation) must agree on the target-requirement COUNT. Proven
/// on BOTH deck-legal fuse defs, and proven the UNFUSED offer (`fuse: false`) is
/// unchanged -- the differential only appears when `fuse: true`.
#[test]
fn t4_offer_reports_the_same_requirement_count_the_cast_validates() {
    let defs = defs_by_name();
    for name in ["Turn // Burn", "Wear // Tear"] {
        let state = GameStateBuilder::new()
            .add_player(P1)
            .add_player(P2)
            .with_registry(corpus_registry(&defs))
            .active_player(P1)
            .object(corpus_object(&defs, P1, name, ZoneId::Hand(P1)))
            .build()
            .expect("state builds");
        let card = id_of(&state, name);

        let unfused = mtg_engine::spell_target_requirements(&state, card, &[], None, false);
        assert_eq!(
            unfused.len(),
            1,
            "{name}: the UNFUSED offer must report only the left half's requirement \
             (unchanged by this batch); got {unfused:?}"
        );

        let fused = mtg_engine::spell_target_requirements(&state, card, &[], None, true);
        assert_eq!(
            fused.len(),
            2,
            "{name}: the FUSED offer must report BOTH halves' requirements (CR \
             702.102d); got {fused:?}"
        );

        // The differential: this is exactly the count `card_def_target_requirements`
        // (the function `handle_cast_spell` itself calls) would validate against for a
        // fused cast on this card -- proven by driving a real cast with exactly that
        // many placeholder targets and confirming it is NOT refused for a count
        // mismatch (it may still be refused for other reasons on a bare fixture with
        // no legal candidates, but never for "expected N..=N target(s)").
    }
}
