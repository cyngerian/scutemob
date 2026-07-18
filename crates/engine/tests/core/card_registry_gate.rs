//! Registry-level enforcement of Architecture Invariant 9.
//!
//! Two machine gates, previously maintained only by worker discipline:
//!
//! 1. `CardRegistry::try_new` rejects duplicate `CardId`s instead of silently
//!    dropping one definition into a `HashMap` collision.
//! 2. Every definition in `all_cards()` that registers no abilities for a card
//!    with printed rules text carries a non-`Complete` `completeness` marker,
//!    so the deck-build gate can surface it.

use mtg_engine::all_cards;
use mtg_engine::cards::{CardDefinition, CardRegistry, Completeness, RegistryError, TypeLine};
use mtg_engine::state::{CardId, CardType};
use mtg_engine::{
    start_game, start_game_allowing_incomplete, GameState, GameStateBuilder, GameStateError,
    ObjectSpec, PlayerId, ZoneId,
};

fn artifact(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [CardType::Artifact].iter().copied().collect(),
            subtypes: Default::default(),
        },
        ..Default::default()
    }
}

// ── Duplicate CardId detection ────────────────────────────────────────────────

#[test]
/// Two definitions claiming one CardId is a hard error, not a silent overwrite.
///
/// Before this gate, `HashMap::collect` kept whichever definition came last, so a
/// typo'd `cid(...)` in a newly authored card would quietly replace an unrelated
/// card's abilities with no diagnostic anywhere in the build or the test suite.
fn test_try_new_rejects_duplicate_card_id() {
    let defs = vec![
        artifact("sol-ring", "Sol Ring"),
        artifact("mana-crypt", "Mana Crypt"),
        artifact("sol-ring", "Sol Ring (typo'd copy)"),
    ];

    let err = CardRegistry::try_new(defs).expect_err("duplicate CardId must be rejected");
    let RegistryError::DuplicateCardId {
        card_id,
        first_name,
        second_name,
    } = &err;
    assert_eq!(card_id.0, "sol-ring");
    assert_eq!(first_name, "Sol Ring");
    assert_eq!(second_name, "Sol Ring (typo'd copy)");

    // The message must name the colliding id and both definitions.
    let rendered = err.to_string();
    assert!(rendered.contains("sol-ring"), "{rendered}");
    assert!(rendered.contains("Sol Ring (typo'd copy)"), "{rendered}");
}

#[test]
#[should_panic(expected = "duplicate CardId")]
/// `CardRegistry::new` is the infallible convenience wrapper — it panics rather than
/// building a registry that has silently lost a definition.
fn test_new_panics_on_duplicate_card_id() {
    CardRegistry::new(vec![
        artifact("sol-ring", "Sol Ring"),
        artifact("sol-ring", "Sol Ring again"),
    ]);
}

#[test]
/// Distinct CardIds are accepted, and every definition is retrievable.
fn test_try_new_accepts_unique_card_ids() {
    let registry = CardRegistry::try_new(vec![
        artifact("sol-ring", "Sol Ring"),
        artifact("mana-crypt", "Mana Crypt"),
    ])
    .expect("unique CardIds must be accepted");

    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry.get(CardId("mana-crypt".to_string())).unwrap().name,
        "Mana Crypt"
    );
}

#[test]
/// The shipped corpus itself has no CardId collisions. This is the regression test
/// that makes `try_new`'s error unreachable in practice — if a new def collides, this
/// fails by name in CI rather than in a player's game.
fn test_all_cards_have_unique_card_ids() {
    // Run the check on a thread with an enlarged stack. `all_cards()` materialises
    // ~1,749 deeply-nested `CardDefinition` trees, and constructing + dropping them
    // through `try_new` overflows the default 8 MiB test-thread stack when this
    // binary is executed standalone (`target/debug/deps/core-* --exact`). It passes
    // under `cargo test` only because libtest's own runner thread is larger — a
    // latent CI flake that would surface on any toolchain/allocator change. Make the
    // check environment-insensitive by pinning the stack size it runs under.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            CardRegistry::try_new(all_cards())
                .expect("all_cards() must not contain duplicate CardIds");
        })
        .expect("spawn unique-card-id check thread")
        .join()
        .expect("unique-card-id check thread panicked");
}

// ── Completeness markers on the shipped corpus ────────────────────────────────

/// True when a def registers **no behaviour at all** — the condition this file calls
/// "inert": a blank permanent wearing the card's name.
///
/// `abilities.is_empty()` alone is **not** that condition, and assuming it was is how the
/// marker corpus acquired a family of false `inert("no abilities implemented")` markers
/// (scutemob-88 marker sweep). Not every printed ability is an `AbilityDefinition`: a
/// cost-reduction static ("Dragon spells you cast cost {1} less") is not an ability the
/// engine dispatches, it is a `spell_cost_modifier` consumed on the cast path by
/// `apply_spell_cost_modifiers`. Those defs correctly ship `abilities: vec![]` and are
/// fully implemented.
///
/// So this asks the real question — does *any* behaviour-bearing field carry something?
///
/// Excluded on purpose, as identity/characteristics rather than behaviour: `card_id`,
/// `name`, `mana_cost`, `types`, `oracle_text`, `power`, `toughness`, `color_indicator`,
/// `completeness` — and `starting_loyalty`, which is a *characteristic* (CR 208.2), not an
/// ability: a planeswalker with a loyalty number and no loyalty abilities is exactly as
/// inert as a blank permanent, and should be caught here. (A vanilla creature never
/// reaches this check at all: it has no printed rules text.)
///
/// **Adding a behaviour-bearing field to `CardDefinition` means adding it here**, or the
/// gate will demand an `inert` marker on a def that is actually complete.
fn registers_no_behavior(d: &CardDefinition) -> bool {
    d.abilities.is_empty()
        && d.spell_cost_modifiers.is_empty()
        && d.self_cost_reduction.is_none()
        && d.activated_ability_cost_reductions.is_empty()
        && d.spell_additional_costs.is_empty()
        && d.back_face.is_none()
        && d.adventure_face.is_none()
        && d.meld_pair.is_none()
        && !d.cant_be_countered
        && !d.self_exile_on_resolution
        && !d.self_shuffle_on_resolution
}

#[test]
/// A definition that registers zero behaviour for a card with printed rules text is
/// inert: it is a blank permanent wearing the card's name. Every such def must carry a
/// non-`Complete` marker so `validate_deck` rejects it.
///
/// This is the guard that keeps the SR-2 sweep from rotting. A newly authored def with no
/// registered behaviour and a non-empty `oracle_text` fails here until it is either
/// implemented or explicitly marked. See [`registers_no_behavior`] for why "no behaviour"
/// is not the same as "no `abilities`".
fn test_inert_definitions_are_marked_incomplete() {
    let unmarked: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| {
            registers_no_behavior(d)
                && !d.oracle_text.trim().is_empty()
                && d.completeness.is_complete()
        })
        .map(|d| d.name)
        .collect();

    assert!(
        unmarked.is_empty(),
        "these defs have oracle text but register no behaviour at all, and are not marked \
         incomplete (add `completeness: Completeness::inert(\"...\")`): {unmarked:?}"
    );
}

#[test]
/// Non-vacuity for [`test_inert_definitions_are_marked_incomplete`].
///
/// `registers_no_behavior` is a conjunction over a dozen fields, so it is easy to widen it
/// into something that is never true — at which point the inert gate above passes for
/// every def and silently stops guarding anything. Pin both directions against the real
/// corpus: some defs *do* register no behaviour (and are marked), and some defs carry
/// their whole implementation outside `abilities`.
fn inert_gate_is_not_vacuous() {
    let cards = all_cards();

    // Direction 1: the predicate still fires on real inert defs (all of which are marked,
    // which is why the gate above is green).
    let inert_marked = cards
        .iter()
        .filter(|d| registers_no_behavior(d) && !d.oracle_text.trim().is_empty())
        .count();
    assert!(
        inert_marked > 0,
        "registers_no_behavior() matched zero defs — the inert gate is vacuous"
    );

    // Direction 2: the predicate does NOT fire on defs whose behaviour lives outside
    // `abilities`. These are exactly the defs the old `abilities.is_empty()` check
    // mis-flagged as inert. If this hits zero, the exclusion has stopped being exercised.
    let behavior_outside_abilities = cards
        .iter()
        .filter(|d| d.abilities.is_empty() && !registers_no_behavior(d))
        .count();
    assert!(
        behavior_outside_abilities > 0,
        "no def carries behaviour outside `abilities` — the sibling-field exclusion in \
         registers_no_behavior() is untested and may be silently wrong"
    );
}

#[test]
/// The kind↔shape gate (EF-13): a def that registers no behaviour is `Inert` by the
/// `Completeness` taxonomy. `Partial` ("some clauses are implemented and at least one is
/// not") and `KnownWrong` ("every clause is implemented, but one deviates") both assert
/// that behaviour *is* implemented — which a def registering zero behaviour contradicts.
/// So a no-behaviour def marked `Partial`/`KnownWrong` is mis-classified; it is `Inert`.
///
/// This gate exists because the marker corpus drifted into exactly this state: 101 defs
/// were marked `Partial` while `registers_no_behavior` was true for them, misreporting the
/// campaign's `todo`/`empty` buckets (EF-13; original finding
/// `memory/card-authoring/marker-sweep-engine-findings-2026-07-16.md`). They were
/// reclassified `Partial`→`Inert`; this keeps the class from re-forming.
///
/// **`KnownWrong` is gated too, not just `Partial`.** A `KnownWrong` def claims *every*
/// clause is implemented, so a no-behaviour `KnownWrong` is even more contradictory than a
/// `Partial` one, and there is no legitimate no-behaviour `KnownWrong` def — implementing
/// zero clauses cannot also mean implementing all of them. Verified against the compiled
/// registry at reclassification time (`all_cards()` + `registers_no_behavior`): **zero**
/// `KnownWrong` defs register no behaviour, so including `KnownWrong` rejects no real
/// member. Enumerate this class from `all_cards()`, never source text — the regex
/// `abilities:\s*vec!\[\s*\]` also matches `mana_abilities: vec![]` (the recurring corpus
/// trap), which is why a source scan undercounts.
fn test_no_behavior_defs_are_inert_not_partial_or_known_wrong() {
    let offenders: Vec<(String, &'static str)> = all_cards()
        .iter()
        .filter(|d| {
            registers_no_behavior(d)
                && matches!(
                    d.completeness,
                    Completeness::Partial(_) | Completeness::KnownWrong(_)
                )
        })
        .map(|d| (d.name.clone(), d.completeness.kind()))
        .collect();

    assert!(
        offenders.is_empty(),
        "these defs register no behaviour but are marked Partial/KnownWrong — both claim a \
         clause IS implemented, so by the taxonomy they are Inert. Change the marker to \
         `Completeness::inert(...)`: {offenders:?}"
    );
}

#[test]
/// Non-vacuity for [`test_no_behavior_defs_are_inert_not_partial_or_known_wrong`].
///
/// The gate above is green partly because EF-13 emptied its target class, so a
/// corpus-only assertion would keep passing even if the predicate silently stopped
/// distinguishing the shapes it is meant to reject (an SR lesson: a gate that cannot fail
/// is not a gate). Pin the predicate directly against a synthetic corpus: a no-behaviour
/// def marked `Partial` — and one marked `KnownWrong` — must both be flagged, while the
/// same def marked `Inert` or `Complete` must not be.
fn no_behavior_kind_gate_is_not_vacuous() {
    let base = {
        let mut d = artifact("canary", "Canary");
        d.oracle_text = "Do a thing.".to_string();
        d
    };
    assert!(
        registers_no_behavior(&base),
        "the canary must register no behaviour, or this proof tests nothing"
    );

    let is_flagged = |c: Completeness| {
        let mut d = base.clone();
        d.completeness = c;
        registers_no_behavior(&d)
            && matches!(
                d.completeness,
                Completeness::Partial(_) | Completeness::KnownWrong(_)
            )
    };

    // Rejected shapes: the gate fires.
    assert!(
        is_flagged(Completeness::partial("x")),
        "a no-behaviour Partial def must be flagged by the gate"
    );
    assert!(
        is_flagged(Completeness::known_wrong("x")),
        "a no-behaviour KnownWrong def must be flagged by the gate"
    );
    // Allowed shapes: the gate stays silent.
    assert!(
        !is_flagged(Completeness::inert("x")),
        "a no-behaviour Inert def is correctly classified and must NOT be flagged"
    );
    assert!(
        !is_flagged(Completeness::Complete),
        "a Complete def must NOT be flagged"
    );
}

#[test]
/// A vanilla card (no printed rules text, e.g. Memnite) is complete with zero
/// abilities. The inert guard above must not force a marker onto it.
fn test_vanilla_definitions_stay_complete() {
    let marked_vanilla: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| {
            d.abilities.is_empty()
                && d.oracle_text.trim().is_empty()
                && !d.completeness.is_complete()
        })
        .map(|d| d.name)
        .collect();

    assert!(
        marked_vanilla.is_empty(),
        "these defs have no rules text and no abilities — they are vanilla and should be \
         Complete: {marked_vanilla:?}"
    );
}

#[test]
/// Every non-`Complete` marker carries a note explaining what is missing or wrong.
/// An empty note produces a deck-build message the player cannot act on.
fn test_incomplete_markers_carry_a_note() {
    let noteless: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| !d.completeness.is_complete() && d.completeness.note().trim().is_empty())
        .map(|d| d.name)
        .collect();

    assert!(
        noteless.is_empty(),
        "these defs are marked incomplete with an empty note: {noteless:?}"
    );
}

#[test]
/// `Completeness::Complete` is the `Default`, so a def that omits the field is legal
/// in a deck. Every other variant is a deck-build error.
fn test_completeness_default_is_complete() {
    assert_eq!(Completeness::default(), Completeness::Complete);
    assert!(Completeness::default().is_complete());
    assert!(!Completeness::inert("x").is_complete());
    assert!(!Completeness::partial("x").is_complete());
    assert!(!Completeness::known_wrong("x").is_complete());
    assert_eq!(Completeness::inert("x").kind(), "inert");
    assert_eq!(Completeness::partial("x").kind(), "partial");
    assert_eq!(Completeness::known_wrong("x").kind(), "known-wrong");
    assert_eq!(Completeness::known_wrong("why").note(), "why");
    assert_eq!(Completeness::Complete.note(), "");
}

// ── start_game completeness gate (SR-12) ──────────────────────────────────────
//
// validate_deck rejects a non-Complete card, but only where a caller runs it —
// GameStateBuilder, the simulator, and the fuzzer never do. start_game is the
// choke point every game-assembly path shares, so the marker is made
// unbypassable there. These tests prove the gate fires, that the opt-out admits
// what it must, and that the gate's scope is exactly "known but non-Complete."

/// Build a one-player state holding a single hand object that references a def
/// with the given completeness. The def has printed rules text so that a
/// non-Complete marker is semantically warranted.
fn state_with_one_card(completeness: Completeness) -> GameState {
    let mut def = artifact("test-card", "Test Card");
    def.oracle_text = "Do a thing.".to_string();
    def.completeness = completeness;
    let registry = CardRegistry::new(vec![def]);
    GameStateBuilder::new()
        .add_player(PlayerId(1))
        .object(
            ObjectSpec::artifact(PlayerId(1), "Test Card")
                .with_card_id(CardId("test-card".to_string()))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .with_registry(registry)
        .build()
        .expect("state builds")
}

#[test]
/// An inert / partial / knowingly-wrong card in the game aborts start_game with
/// `IncompleteCardsInGame`, reporting the class and note so the failure is
/// actionable. This is the structural companion to `validate_deck`.
fn start_game_rejects_incomplete_cards() {
    for (completeness, expected_kind) in [
        (Completeness::inert("no abilities"), "inert"),
        (
            Completeness::partial("second clause unimplemented"),
            "partial",
        ),
        (
            Completeness::known_wrong("deviates from oracle"),
            "known-wrong",
        ),
    ] {
        let expected_note = completeness.note().to_string();
        let state = state_with_one_card(completeness);
        let err = start_game(state).expect_err("non-Complete card must abort start_game");
        match err {
            GameStateError::IncompleteCardsInGame {
                count,
                first_name,
                first_kind,
                first_note,
            } => {
                assert_eq!(count, 1);
                assert_eq!(first_name, "Test Card");
                assert_eq!(first_kind, expected_kind);
                assert_eq!(first_note, expected_note);
            }
            other => panic!("expected IncompleteCardsInGame, got {other:?}"),
        }
    }
}

#[test]
/// The explicit opt-out admits exactly what the gate rejects. Without it there
/// would be no way to stand up a game with a placeholder def on purpose — and a
/// silent bypass is precisely what SR-12 exists to remove.
fn start_game_allowing_incomplete_admits_incomplete_cards() {
    let state = state_with_one_card(Completeness::partial("second clause unimplemented"));
    start_game_allowing_incomplete(state)
        .expect("the opt-out must start a game containing an incomplete card");
}

#[test]
/// A Complete card passes the gate — the check does not reject faithful defs.
fn start_game_accepts_complete_cards() {
    let state = state_with_one_card(Completeness::Complete);
    start_game(state).expect("a Complete card must start normally");
}

#[test]
/// The gate counts every offender and still reports the first deterministically
/// (imbl::OrdMap iterates in ObjectId order, so "first" is stable).
fn start_game_counts_all_incomplete_cards() {
    let inert = {
        let mut d = artifact("card-a", "Card A");
        d.oracle_text = "text".to_string();
        d.completeness = Completeness::inert("blank");
        d
    };
    let partial = {
        let mut d = artifact("card-b", "Card B");
        d.oracle_text = "text".to_string();
        d.completeness = Completeness::partial("half");
        d
    };
    let registry = CardRegistry::new(vec![inert, partial]);
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .object(
            ObjectSpec::artifact(PlayerId(1), "Card A")
                .with_card_id(CardId("card-a".to_string()))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .object(
            ObjectSpec::artifact(PlayerId(1), "Card B")
                .with_card_id(CardId("card-b".to_string()))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .with_registry(registry)
        .build()
        .expect("state builds");

    match start_game(state).expect_err("two incomplete cards must abort") {
        GameStateError::IncompleteCardsInGame { count, .. } => assert_eq!(count, 2),
        other => panic!("expected IncompleteCardsInGame, got {other:?}"),
    }
}

#[test]
/// Scope guard: a `card_id` absent from the registry is NOT this gate's business
/// (that is the UnknownCard axis; the object already carries synthesised
/// characteristics), and a naked object with no `card_id` is not a card in the
/// game at all. Neither trips the completeness gate — otherwise the hundreds of
/// tests that place naked or empty-registry objects would break.
fn start_game_ignores_unknown_and_naked_objects() {
    // Empty registry, object names a card_id that resolves to nothing.
    let unknown = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .object(
            ObjectSpec::artifact(PlayerId(1), "Ghost")
                .with_card_id(CardId("not-in-registry".to_string()))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .build()
        .expect("state builds");
    start_game(unknown).expect("an unknown card_id is out of this gate's scope");

    // Naked object with no card_id at all.
    let naked = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .object(ObjectSpec::creature(PlayerId(1), "Naked Bear", 2, 2))
        .build()
        .expect("state builds");
    start_game(naked).expect("a naked object is not a card in the game");
}
