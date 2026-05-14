# PB-EWC — ReplacementModification::EntersWithCounters count u32 → EffectAmount

**Task**: scutemob-20
**Status**: implementation + tests complete (2026-05-14); pending review.
**HASH**: 17 → 18.

## Why

OOS-LKI-Power-2 (memory/primitives/pb-retriage-CC.md:604) called out that
`ReplacementModification::EntersWithCounters` accepts a static `u32` counter
count today, which blocks any "enters with N where N is dynamic" wording:

- **Master Biomancer** — "Each other creature you control enters with a
  number of additional +1/+1 counters on it equal to this creature's power."
  Ruling 2013-01-24: "use Master Biomancer's power as that creature is
  entering." This is `EffectAmount::PowerOf(EffectTarget::Source)`.
- **Ingenious Prodigy** — "This creature enters with X +1/+1 counters on it."
  Was previously fudged as a triggered-ETB ability with a CR 614.1c DEVIATION
  comment because the DSL lacked `EntersWithCounters` with `EffectAmount::XValue`.

PB-EWC mirrors PB-TS's `TokenSpec.count u32 → EffectAmount` migration on the
replacement side: enum variant, resolver, hash, and HASH_SCHEMA_VERSION all
bumped.

## Engine surface

### `state/replacement_effect.rs`
- Added `use crate::cards::card_definition::EffectAmount;` (sibling crate
  precedent — `continuous_effect.rs:9` and `dungeon.rs:15` already do this).
- `ReplacementModification::EntersWithCounters` migrated to
  `{ counter: CounterType, count: Box<EffectAmount> }`. `Box` to satisfy
  `clippy::large_enum_variant` (EffectAmount can be large due to
  `CardCount { filter: TargetFilter }` and recursive `Sum`).

### `state/hash.rs`
- `HASH_SCHEMA_VERSION` bumped from 17 → 18 with a changelog entry.
- `EntersWithCounters` hash arm reuses the existing `EffectAmount::hash_into`
  impl (state/hash.rs:4479) — no new discriminant.

### `rules/replacement.rs`
- `emit_etb_modification` gains a `replacement_source: Option<ObjectId>`
  parameter. For self-ETB (`apply_self_etb_modification`), the source is the
  entering permanent itself (`Some(new_id)`). For non-self global ETB
  (`apply_etb_replacements`), the source is pulled from
  `ReplacementEffect.source` (Master Biomancer's ObjectId).
- The `EntersWithCounters` arm builds an `EffectContext` pinned to the
  replacement source, copies the source's `x_value` (so Ingenious Prodigy's
  `EffectAmount::XValue` resolves correctly), and calls
  `crate::effects::resolve_amount(state, &count, &ctx)`. `resolve_amount` was
  promoted from `fn` → `pub(crate) fn`.
- `register_permanent_replacement_abilities` gains a
  `ReplacementTrigger::WouldEnterBattlefield { filter }` rebind clause in the
  non-self branch: card defs use `CreatureControlledBy(PlayerId(0))` as a
  placeholder, which `bind_object_filter` rebinds to the actual controller at
  registration time. Without this Master Biomancer's filter would never match
  (placeholder PlayerId(0) leaks through registration).

### `effects/mod.rs`
- `resolve_amount` signature unchanged; visibility widened to `pub(crate)`.

## Card defs

- `cards/defs/master_biomancer.rs`: counter half authored via
  `AbilityDefinition::Replacement` with `is_self: false`,
  `filter: CreatureControlledBy(PlayerId(0))`, and
  `count: Box::new(EffectAmount::PowerOf(EffectTarget::Source))`. Type-grant
  half (OOS-EWC-1) is preserved as a TODO referencing the new OOS seed.
- `cards/defs/ingenious_prodigy.rs`: ETB counter clause converted from
  `AbilityDefinition::Triggered` (with the CR 614.1c DEVIATION comment) to
  `AbilityDefinition::Replacement { is_self: true,
  count: Box::new(EffectAmount::XValue), filter: Any }`. The DEVIATION comment
  is removed; the upkeep draw trigger is preserved unchanged.

## Tests

`crates/engine/tests/primitive_pb_ewc.rs` (5 tests):

1. `test_master_biomancer_counter_from_live_power_base` — cast MB from hand,
   cast Elvish Mystic, verify 2 +1/+1 counters on the Mystic (= MB's base
   power 2). Also verifies the replacement filter was rebound from `PlayerId(0)`
   placeholder to the actual controller at registration time.
2. `test_master_biomancer_counter_tracks_pumped_power` — cast MB, add 2 +1/+1
   counters to MB (Layer 7d → MB is 4/6), cast Elvish Mystic, verify 4
   counters on the Mystic. Exercises the live-power read via
   `calculate_characteristics`.
3. `test_ingenious_prodigy_x_value_replacement_counts` — cast Ingenious
   Prodigy with X=5, verify 5 +1/+1 counters present immediately on the
   battlefield (replacement, not stack trigger).
4. `test_ingenious_prodigy_x_zero_no_counters` — X=0 must produce no counters
   and suppress the `CounterAdded` event (existing `>0` guard).
5. `test_pb_ewc_hash_schema_version_is_18` — canary that
   `HASH_SCHEMA_VERSION == 18`. All 10 other PB hash canaries were updated
   17 → 18 via sed.

Also fixed the existing `tests/x_cost_spells.rs:test_x_cost_etb_counters_ingenious_prodigy`
to keep working under the new self-ETB-replacement path (no test code
change needed; the assertions on count + CounterAdded event still hold).

## OOS seeds filed

In `memory/primitives/pb-retriage-CC.md`:

- **OOS-EWC-1** — `EntersAsAdditionalType` for Master Biomancer's type-grant
  half.
- **OOS-EWC-2** — Golgari Grave-Troll: self-ETB with
  `count: EffectAmount::CardCount { Graveyard, Controller, CreatureCard }`.
  Pure card-authoring follow-up — engine work is done.
- **OOS-EWC-3** — Dragonstorm Globe: non-self ETB with subtype receiver
  filter. Requires a new `ObjectFilter` variant (creature of subtype X
  controlled by Y).

## Verification

- `cargo build --workspace` clean.
- `cargo test --workspace --lib --tests` — 2754 passing (+5 new tests).
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.

## Risk register

- `Box<EffectAmount>` is a wire-format change baked into HASH 18; old
  serialized states with `u32` are not forward-compatible.
- `WouldEnterBattlefield` filter rebind is a new code path —
  `test_master_biomancer_counter_from_live_power_base` sanity-asserts the
  rebound `CreatureControlledBy(p1)` ends up in
  `state.replacement_effects`.
- The resolver passes `replacement_source.unwrap_or(new_id)` as a defensive
  default. This only matters if a registered `ReplacementEffect.source` is
  `None` — unusual, but defensive.
- `EffectContext.x_value` is read from `state.objects[source].x_value`. For
  Master Biomancer source = MB itself (x_value = 0 — MB is not an X spell).
  For Ingenious Prodigy source = new_id with `x_value` populated at
  resolution.rs:546.
