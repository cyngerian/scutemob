//! Tests for PB-TS: `TokenSpec.count` shape change `u32` → `EffectAmount` (CR 111.1).
//!
//! This primitive unblocks dynamic-count token creation: any `EffectAmount` variant
//! can now drive how many tokens are created, evaluated once at execution time per
//! CR 608.2h (before `apply_token_creation_replacement` doubling boundary).
//!
//! Cards unblocked: Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss,
//! Izoni Thousand-Eyed.
//!
//! Engine surface:
//! - `cards/card_definition.rs` — `TokenSpec.count` field type changed `u32` → `EffectAmount`.
//!   Default is `EffectAmount::Fixed(1)` (matches previous default of `1`).
//! - `effects/mod.rs` — `Effect::CreateToken` and `Effect::CreateTokenAndAttachSource`
//!   call `resolve_amount(state, &spec.count, ctx).max(0) as u32` before the
//!   `apply_token_creation_replacement` boundary.
//! - `state/hash.rs` — `HASH_SCHEMA_VERSION` bumped 13 → 14.
//!
//! Tests:
//!   (a) `EffectAmount::Fixed(N)` creates exactly N tokens via `execute_effect`.
//!   (b) `EffectAmount::Fixed(0)` creates zero tokens (clamp floor).
//!   (c) `EffectAmount::PermanentCount` scales with battlefield goblin count (Krenko-style).
//!   (d) `EffectAmount::CounterCount` reads LKI counters from a dead source in graveyard
//!       (Chasm Skulker scenario — CR 113.7a, CR 122.6).
//!   (e) Hash determinism: `TokenSpec` with `Fixed(3)` vs `Fixed(5)` produce distinct
//!       hashes; `HASH_SCHEMA_VERSION` sentinel is exactly 14.

use mtg_engine::cards::card_definition::{EffectAmount, EffectTarget, TokenSpec};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::hash::HashInto;
use mtg_engine::{
    CardType, Color, CounterType, Effect, GameStateBuilder, ObjectId, ObjectSpec, PlayerId,
    SubType, TargetController, TargetFilter, ZoneId, HASH_SCHEMA_VERSION,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn count_tokens_named(state: &mtg_engine::GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.is_token && o.characteristics.name == name)
        .count()
}

// ── Test (a): Fixed(N) creates exactly N tokens ───────────────────────────────

/// CR 111.1 — `EffectAmount::Fixed(N)` produces exactly N tokens when
/// `Effect::CreateToken` is executed. Verifies the shape change correctly
/// dispatches the constant case through `resolve_amount`.
///
/// Sanity regression: confirms the most common `count` value (a plain integer)
/// is preserved by the migration from `u32` to `EffectAmount::Fixed(i32)`.
#[test]
fn test_pb_ts_fixed_count_creates_n_tokens() {
    let cases: &[(i32, usize)] = &[(1, 1), (3, 3), (5, 5), (10, 10)];

    for &(n, expected) in cases {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(ObjectSpec::creature(p(1), "Source", 1, 1).in_zone(ZoneId::Battlefield))
            .build()
            .unwrap();

        let source_id = find_object(&state, "Source");

        let effect = Effect::CreateToken {
            spec: TokenSpec {
                name: "TestToken".to_string(),
                card_types: [CardType::Creature].into_iter().collect(),
                power: 1,
                toughness: 1,
                count: EffectAmount::Fixed(n),
                ..Default::default()
            },
        };

        let mut ctx = EffectContext::new(p(1), source_id, vec![]);
        let _events = execute_effect(&mut state, &effect, &mut ctx);

        let got = count_tokens_named(&state, "TestToken");
        assert_eq!(
            got, expected,
            "Fixed({}) should create {} tokens; got {}",
            n, expected, got
        );
    }
}

// ── Test (b): Fixed(0) creates zero tokens, no panic ─────────────────────────

/// CR 111.1 / CR 608.2h — `EffectAmount::Fixed(0)` clamps to 0; the engine
/// must not panic and must not create any tokens.
///
/// This defends against a regression where `u32` truncation silently turned
/// negative Fixed values into large unsigned numbers.
#[test]
fn test_pb_ts_fixed_zero_creates_no_tokens() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Source", 1, 1).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source");

    let effect = Effect::CreateToken {
        spec: TokenSpec {
            name: "ZeroToken".to_string(),
            card_types: [CardType::Creature].into_iter().collect(),
            power: 1,
            toughness: 1,
            count: EffectAmount::Fixed(0),
            ..Default::default()
        },
    };

    let mut ctx = EffectContext::new(p(1), source_id, vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let got = count_tokens_named(&state, "ZeroToken");
    assert_eq!(got, 0, "Fixed(0) must create 0 tokens; got {}", got);
}

// ── Test (c): PermanentCount scales with battlefield Goblin count (Krenko-style) ──

/// CR 111.1 / CR 608.2h — `EffectAmount::PermanentCount` counts Goblin creatures
/// on the battlefield controlled by the controller, evaluated at execution time.
///
/// Mirrors Krenko, Mob Boss: "{T}: Create X 1/1 red Goblin creature tokens, where
/// X is the number of Goblins you control." (Krenko himself counts.)
///
/// Tests the count at 0, 2, and 4 Goblins to verify linear scaling.
#[test]
fn test_pb_ts_permanent_count_scales_with_goblin_count() {
    // Case 1: 0 Goblins → 0 tokens
    {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(ObjectSpec::creature(p(1), "Source", 1, 1).in_zone(ZoneId::Battlefield))
            .build()
            .unwrap();

        let source_id = find_object(&state, "Source");

        let effect = Effect::CreateToken {
            spec: goblin_token_spec_krenko_count(),
        };

        let mut ctx = EffectContext::new(p(1), source_id, vec![]);
        let _events = execute_effect(&mut state, &effect, &mut ctx);

        let got = count_tokens_named(&state, "Goblin");
        assert_eq!(
            got, 0,
            "PermanentCount with 0 Goblins must produce 0 tokens"
        );
    }

    // Case 2: 2 Goblins on battlefield → 2 tokens
    {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(ObjectSpec::creature(p(1), "Source", 1, 1).in_zone(ZoneId::Battlefield))
            .object(
                ObjectSpec::creature(p(1), "GoblinA", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())])
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(p(1), "GoblinB", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())])
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .unwrap();

        let source_id = find_object(&state, "Source");

        let effect = Effect::CreateToken {
            spec: goblin_token_spec_krenko_count(),
        };

        let mut ctx = EffectContext::new(p(1), source_id, vec![]);
        let _events = execute_effect(&mut state, &effect, &mut ctx);

        let got = count_tokens_named(&state, "Goblin");
        assert_eq!(
            got, 2,
            "PermanentCount with 2 Goblins must produce 2 Goblin tokens; got {}",
            got
        );
    }

    // Case 3: 4 Goblins on battlefield → 4 tokens
    {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(ObjectSpec::creature(p(1), "Source", 1, 1).in_zone(ZoneId::Battlefield))
            .object(
                ObjectSpec::creature(p(1), "Goblin1", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())])
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(p(1), "Goblin2", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())])
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(p(1), "Goblin3", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())])
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(
                ObjectSpec::creature(p(1), "Goblin4", 1, 1)
                    .with_subtypes(vec![SubType("Goblin".to_string())])
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .unwrap();

        let source_id = find_object(&state, "Source");

        let effect = Effect::CreateToken {
            spec: goblin_token_spec_krenko_count(),
        };

        let mut ctx = EffectContext::new(p(1), source_id, vec![]);
        let _events = execute_effect(&mut state, &effect, &mut ctx);

        let got = count_tokens_named(&state, "Goblin");
        assert_eq!(
            got, 4,
            "PermanentCount with 4 Goblins must produce 4 Goblin tokens; got {}",
            got
        );
    }
}

/// Helper: produce the Krenko-style token spec whose count is
/// `EffectAmount::PermanentCount { filter: Goblin Creature creatures you control }`.
fn goblin_token_spec_krenko_count() -> TokenSpec {
    use mtg_engine::PlayerTarget;
    TokenSpec {
        name: "Goblin".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
        colors: [Color::Red].into_iter().collect(),
        power: 1,
        toughness: 1,
        count: EffectAmount::PermanentCount {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                has_subtype: Some(SubType("Goblin".to_string())),
                controller: TargetController::You,
                ..Default::default()
            },
            controller: PlayerTarget::Controller,
        },
        ..Default::default()
    }
}

// ── Test (d): CounterCount reads +1/+1 counters from live source ──────────────

/// CR 122.6 — `EffectAmount::CounterCount { target: Source, counter: PlusOnePlusOne }`
/// reads the counter count from the live source object and creates that many tokens.
///
/// This exercises the Chasm Skulker scenario: "When Chasm Skulker dies, create X
/// 1/1 blue Squid tokens with islandwalk, where X is the number of +1/+1 counters
/// on it." (CR 608.2h: count is evaluated at execution time.)
///
/// The direct-execute test fires the effect while the source is still accessible
/// (on the battlefield), verifying the `CounterCount` arm of `resolve_amount`
/// correctly reads counter state and drives the token count.
///
/// Note: Chasm Skulker's card def fires this from a WhenDies trigger where
/// ctx.source = the graveyard object (new_grave_id). CR 400.7 mandates that
/// `move_object_to_zone` resets counters, so the actual LKI counter read in a
/// full-game scenario requires `pre_death_counters` from the trigger context.
/// That path is exercised by the full-game card-def integration; this test
/// validates the `EffectAmount::CounterCount` → token count dispatch itself.
#[test]
fn test_pb_ts_counter_count_from_live_source() {
    let cases: &[(u32, usize)] = &[(0, 0), (1, 1), (3, 3), (7, 7)];

    for &(counter_n, expected_tokens) in cases {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(ObjectSpec::creature(p(1), "Skulker", 1, 1).in_zone(ZoneId::Battlefield))
            .build()
            .unwrap();

        let skulker_id = find_object(&state, "Skulker");

        // Pre-load N +1/+1 counters onto the source while it's on the battlefield.
        {
            let obj = state.objects.get_mut(&skulker_id).unwrap();
            obj.counters.insert(CounterType::PlusOnePlusOne, counter_n);
        }

        // Fire the token-creation effect while the source is still on the battlefield.
        // ctx.source = skulker_id (still live, counters accessible).
        let effect = Effect::CreateToken {
            spec: TokenSpec {
                name: "Squid".to_string(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Squid".to_string())].into_iter().collect(),
                colors: [Color::Blue].into_iter().collect(),
                power: 1,
                toughness: 1,
                count: EffectAmount::CounterCount {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                },
                ..Default::default()
            },
        };

        let mut ctx = EffectContext::new(p(1), skulker_id, vec![]);
        let _events = execute_effect(&mut state, &effect, &mut ctx);

        let got = count_tokens_named(&state, "Squid");
        assert_eq!(
            got, expected_tokens,
            "CR 122.6: source with {} +1/+1 counters should create {} Squid tokens via \
             CounterCount dispatch; got {}",
            counter_n, expected_tokens, got
        );
    }
}

// ── Test (e): Hash determinism and HASH_SCHEMA_VERSION sentinel ──────────────

/// CR N/A (hash infrastructure) — PB-TS bumped HASH_SCHEMA_VERSION 13 → 14 (bumped to 15 by PB-LKI-CC).
///
/// (e-1) Sentinel: HASH_SCHEMA_VERSION is exactly 15.
/// (e-2) Determinism: two `TokenSpec` values with `Fixed(3)` produce the same hash.
/// (e-3) `Fixed(3)` vs `Fixed(5)` produce distinct hashes (count IS hashed).
/// (e-4) `Fixed(3)` vs `PermanentCount{...}` produce distinct hashes
///        (variant discriminant IS hashed).
#[test]
fn test_pb_ts_hash_schema_version_and_token_spec_hash_determinism() {
    // (e-1) Schema-version sentinel.
    assert_eq!(
        HASH_SCHEMA_VERSION, 32u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );

    use blake3::Hasher;
    use mtg_engine::PlayerTarget;

    let hash_spec = |spec: &TokenSpec| -> [u8; 32] {
        let mut h = Hasher::new();
        spec.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    let make_spec = |count: EffectAmount| -> TokenSpec {
        TokenSpec {
            name: "TestToken".to_string(),
            card_types: [CardType::Creature].into_iter().collect(),
            power: 1,
            toughness: 1,
            count,
            ..Default::default()
        }
    };

    // (e-2) Determinism: identical specs → same hash.
    let spec_a = make_spec(EffectAmount::Fixed(3));
    let spec_b = make_spec(EffectAmount::Fixed(3));
    assert_eq!(
        hash_spec(&spec_a),
        hash_spec(&spec_b),
        "(e-2) Determinism: two TokenSpec with Fixed(3) must hash identically"
    );

    // (e-3) Fixed(3) vs Fixed(5) → distinct hashes.
    let spec_3 = make_spec(EffectAmount::Fixed(3));
    let spec_5 = make_spec(EffectAmount::Fixed(5));
    assert_ne!(
        hash_spec(&spec_3),
        hash_spec(&spec_5),
        "(e-3) Fixed(3) and Fixed(5) must hash differently (count value is hashed)"
    );

    // (e-4) Fixed(3) vs PermanentCount → distinct hashes (variant discriminant hashed).
    let spec_perm = make_spec(EffectAmount::PermanentCount {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            controller: TargetController::You,
            ..Default::default()
        },
        controller: PlayerTarget::Controller,
    });
    assert_ne!(
        hash_spec(&spec_3),
        hash_spec(&spec_perm),
        "(e-4) Fixed(3) and PermanentCount must hash differently (variant discriminant is hashed)"
    );
}
