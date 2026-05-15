# PB-EAT — ReplacementModification::EntersAsAdditionalType (Master Biomancer Mutant half)

**Task**: scutemob-25
**Status**: implementation + tests complete (2026-05-15); pending review.
**HASH**: 20 → 21.

## Why

`OOS-EWC-1` (memory/primitives/pb-retriage-CC.md:689) called out the
type-grant half of Master Biomancer's oracle text:

> Each other creature you control enters with a number of additional +1/+1
> counters on it equal to this creature's power **and as a Mutant in addition
> to its other types.**

PB-EWC shipped the counter half via `ReplacementModification::EntersWithCounters
{ count: EffectAmount }` but left the type-grant half blocked — there was no
DSL primitive for "enters as a [subtype] in addition to its other types."

PB-EAT ships the parallel `EntersAsAdditionalType { subtype: SubType }` variant:
enum variant, resolver, hash, and HASH_SCHEMA_VERSION all bumped, and Master
Biomancer's card def completed (TODO removed).

## CR foundation

- **CR 614.1c**: "Effects that read 'This permanent enters the battlefield as
  ...' or 'This object enters as ...' are replacement effects." The subtype
  addition is an entry modification, NOT a Layer 4 continuous type-adding
  effect.
- **CR 205.3**: Subtypes are part of a card's type line. Additive subtypes
  insert into the existing subtype set.
- **CR 613.1d**: Layer 4 type-adding effects apply to permanents already on
  the battlefield. Entry modifications (CR 614.1c) apply during ETB to the
  entering object before `PermanentEnteredBattlefield` is emitted, so ETB
  triggers and SBAs observe the augmented type set.

## Engine surface

### `state/replacement_effect.rs`
- New variant `ReplacementModification::EntersAsAdditionalType { subtype: SubType }`.
- `SubType` carried by value (its inner `String` is already heap-allocated; no
  `Box` needed, mirrors `ChooseCreatureType(SubType)`).

### `rules/replacement.rs::emit_etb_modification`
- New arm: pushes `subtype` into
  `state.objects[new_id].characteristics.subtypes` (OrdSet idempotent insert)
  BEFORE the caller emits `PermanentEnteredBattlefield`.
- Emits `GameEvent::ReplacementEffectApplied { description: "enters as a {} in
  addition to its other types" }` when `effect_id` is provided (global
  replacements). Self-ETB replacements skip the event (mirrors existing arm
  conventions).

### `state/hash.rs`
- Discriminant **22** for `EntersAsAdditionalType`.
- `HASH_SCHEMA_VERSION` bumped from 20 → 21 with a changelog entry.
- Hashes the discriminant + `subtype.0` (the inner String).

## Card defs

- `cards/defs/master_biomancer.rs`: OOS-EWC-1 TODO removed. The second
  `AbilityDefinition::Replacement` block now uses
  `ReplacementModification::EntersAsAdditionalType { subtype: SubType("Mutant".to_string()) }`
  with the same `CreatureControlledBy(PlayerId(0))` placeholder filter (bound
  to the actual controller by `bind_object_filter` at registration). Oracle
  text verified via MCP lookup_card.

## Tests

`crates/engine/tests/primitive_pb_eat.rs` (5 tests):

1. `test_pb_eat_hash_schema_version_is_21` — canary that
   `HASH_SCHEMA_VERSION == 21`.
2. `test_pb_eat_partial_eq_distinguishes_subtype` — two
   `EntersAsAdditionalType { subtype: ... }` values that differ only in subtype
   must not be `PartialEq`-equal.
3. `test_pb_eat_serde_pre_pb_eat_snapshots_deserialize` — pre-PB-EAT
   serialized `ReplacementModification` values (EntersTapped, EntersWithCounters)
   continue to deserialize after the new variant was added (additivity); the
   new variant round-trips.
4. `test_master_biomancer_grants_mutant_subtype_and_counters` — positive
   functional: MB on battlefield, Elvish Mystic (Elf Druid printed) cast and
   resolved → gains `Mutant` subtype AND 2 +1/+1 counters (PB-EWC's count side
   resolving against MB's printed power).
5. `test_eat_idempotent_when_subtype_already_present` — Simic Initiate (Human
   Mutant printed) under MB still has `Mutant` (single entry, OrdSet dedups);
   2 (MB) + 1 (Graft 1) = 3 +1/+1 counters confirms PB-EWC + Graft compose.

Also: all 14 existing PB hash canary tests bumped 20u8 → 21u8 via sed with the
sentinel message rewritten to cite PB-EAT uniformly. One PB-EWC sanity-count
assertion in `primitive_pb_ewc.rs` updated to filter by both trigger filter
AND modification (now that MB has 2 replacements with the same filter, the
EntersWithCounters count is 1 not 2).

## OOS seeds filed

In `memory/primitives/pb-retriage-CC.md`:

- **OOS-EAT-1** — `EntersAsAdditionalCardType { card_type: CardType }`
  (parallel primitive for "enters as a [card type]"). 0 confirmed in corpus.
- **OOS-EAT-2** — `EntersAsAdditionalColor { color: Color }` (distinct from
  `ChooseColor` which writes `chosen_color`, not `characteristics.colors`).
  0 confirmed in corpus.
- **OOS-EAT-3** — `EntersAsAdditionalSupertype { supertype: SuperType }`.
  0 confirmed; lowest priority.

## Verification

- `cargo build --workspace`: clean.
- `cargo test --workspace`: 2789 → 2794 passing (+5 new PB-EAT tests).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.

## Risk register

- **HASH 21 is a wire-format additive change.** Pre-PB-EAT serialized
  `ReplacementModification` values deserialize unchanged (variant addition is
  additive in serde externally-tagged enums). The version bump is required so
  any hash or serialize of state CONTAINING the new variant uses the new
  schema.
- **Idempotency**: `characteristics.subtypes` is `OrdSet<SubType>`; inserting
  an already-present subtype is a no-op. CR 614.5 (a replacement effect
  applies to a given event at most once) is enforced upstream by
  `apply_etb_replacements` / `already_applied` tracking; the OrdSet
  idempotency is a defense-in-depth.
- **Two replacements on Master Biomancer, same trigger filter**:
  `apply_etb_replacements` iterates all applicable replacements; both fire on
  the same ETB. CR 616.1 (multiple replacements: affected player chooses
  order) is satisfied — the order is deterministic (insertion order in
  `state.replacement_effects`); both modifications are commutative (counter
  insertion + subtype set insertion are independent state writes).

## Acceptance-criteria mapping

| AC ID | Description | Status |
|-------|-------------|--------|
| 3862 | Engine surface — new variant | ✅ |
| 3863 | Resolver arm | ✅ |
| 3864 | Hash arm + HASH 20→21 + canary sweep | ✅ |
| 3865 | Master Biomancer card def | ✅ |
| 3866 | tests/primitive_pb_eat.rs (sentinel, PartialEq, serde, functional) | ✅ |
| 3867 | OOS-EAT seeds in pb-retriage-CC.md | ✅ |
| 3868 | Build/clippy/fmt/test green + /review | ✅ build/clippy/fmt; ⏳ /review |
