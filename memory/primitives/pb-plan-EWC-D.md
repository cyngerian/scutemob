# PB-EWC-D — `ObjectFilter::CreatureControlledByOfSubtype` + `bind_object_filter` `OwnedByOpponentsOf` rebind

**Task**: scutemob-28
**Status**: in_progress (worker: scutemob-28)
**HASH**: 22 → 23.
**Branch**: `feat/pb-ewc-d-objectfilter-subtype-bindobjectfilter-ownedbyoppone`

## Why

`OOS-EWC-3` (per `pb-retriage-CC.md` and the PB-EWC review) blocks
**Dragonstorm Globe**: "Each Dragon you control enters with an additional
+1/+1 counter on it." The current `ObjectFilter` enum can express
`CreatureControlledBy(PlayerId)` (PB-CD) but cannot pin a *subtype*
receiver, so the Dragonstorm Globe stub at
`crates/engine/src/cards/defs/dragonstorm_globe.rs` carries a TODO and
the +1/+1 counter half is unimplemented.

PB-EWC-D adds a new `ObjectFilter` variant for subtype-specific
controlled-creature receivers (CR 614.1c / CR 613.1d) and finally
implements the +1/+1 counter half of Dragonstorm Globe via the PB-EWC
`EntersWithCounters` infrastructure.

It also closes **sub-gap E2** from `pb-review-EWC.md`: the
`bind_object_filter` helper currently rebinds `ControlledBy(0)` and
`CreatureControlledBy(0)` but not `OwnedByOpponentsOf(0)`, so a future
non-self `WouldEnterBattlefield` replacement using
`OwnedByOpponentsOf(PlayerId(0))` would leak the placeholder through
registration. The fix is the symmetric ~3-line extension in
`bind_object_filter`.

## Design choice (AC 3902): new variant vs generalize

Two options were on the table:

- **(a)** Add a new variant
  `ObjectFilter::CreatureControlledByOfSubtype { controller: PlayerId,
  subtype: SubType }`.
- **(b)** Generalize the existing
  `ObjectFilter::CreatureControlledBy(PlayerId)` to
  `ObjectFilter::CreatureControlledBy { controller: PlayerId,
  subtype: Option<SubType> }`.

**Chosen: (a) — additive new variant.**

Rationale:
- Purely additive. Existing `CreatureControlledBy(PlayerId)` call sites
  (PB-CD: Hardened Scales / Conclave Mentor / Corpsejack Menace,
  PB-EWC: Master Biomancer) keep working unchanged.
- Hash arm: one new discriminant (9) added next to the existing
  `CreatureControlledBy` (8). No migration of existing serialized
  states for any non-subtype receiver.
- `bind_object_filter`: one new arm matching
  `CreatureControlledByOfSubtype { controller: PlayerId(0), subtype }`
  rebinding to `{ controller, subtype }`. Pattern mirrors
  the existing `CreatureControlledBy(PlayerId(0))` arm.
- `object_matches_filter`: one new arm checking
  layer-resolved creature type (CR 613.1d), controller equality, AND
  subtype membership.
- No coordinated update to all existing `CreatureControlledBy` call
  sites required (the generalize option (b) would touch the hash arm,
  bind helper, object matcher, every card def, every test).

Trade-offs accepted:
- The two variants are slightly duplicative (Dragon-receiver could be
  expressed as a subtype-filter on top of CreatureControlledBy in
  option (b)). Acceptable given option (b) is invasive and PB-EWC-D
  scope is narrow.
- Future receiver gaps (artifact, supertype, multi-subtype) may want
  their own variants — filed as OOS-EWCD-N seeds.

## Engine surface

### `state/replacement_effect.rs`

- Add new `ObjectFilter::CreatureControlledByOfSubtype { controller:
  PlayerId, subtype: SubType }` variant. Doc-comment references the
  PB-CD `CreatureControlledBy` precedent, calls out the CR 613.1d
  layer-resolved type check, and notes that the `subtype` carries by
  value (its inner `String` is already heap-allocated — mirrors
  `EntersAsAdditionalType.subtype` from PB-EAT).

### `state/hash.rs`

- Add hash arm for the new variant. Discriminant 9 (next after PB-CD's
  `CreatureControlledBy` = 8). Hash both `controller` and `subtype`.
- Bump `HASH_SCHEMA_VERSION` from 22 → 23 with a `- 23:` changelog
  entry citing PB-EWC-D, the new variant, the unblock target
  (Dragonstorm Globe + future subtype receivers), and CR 614.1c /
  CR 613.1d.
- Rewrite the `- 22:` history entry to reflect the prior state — no
  changes required there (it was added by PB-XA2 and is preserved).

### `rules/replacement.rs`

- **AC 3904 — resolver wiring**: Add an arm to
  `object_matches_filter` for the new variant. Pattern: layer-resolved
  type via `calculate_characteristics` (CR 613.1d) → check
  `card_types.contains(&CardType::Creature)` AND
  `subtypes.contains(&subtype)` AND `controller == player`.
- **AC 3905 — bind_object_filter E2 fix**: Extend
  `bind_object_filter` to also rebind
  `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)`.
  Also add the rebind arm for the new
  `CreatureControlledByOfSubtype { controller: PlayerId(0), subtype }`
  → `{ controller, subtype }`.
- The non-self `WouldEnterBattlefield { filter }` arm in
  `register_permanent_replacement_abilities` already calls
  `bind_object_filter(filter, controller)` (PB-EWC infrastructure),
  so picking up the new variant + the OwnedByOpponentsOf fix is
  automatic.

## Card defs

- `cards/defs/dragonstorm_globe.rs`: Author the counter half via
  `AbilityDefinition::Replacement` with `is_self: false`,
  `trigger: WouldEnterBattlefield { filter:
  CreatureControlledByOfSubtype { controller: PlayerId(0),
  subtype: SubType("Dragon".into()) } }`, `modification:
  EntersWithCounters { counter: PlusOnePlusOne, count:
  Box::new(EffectAmount::Fixed(1)) }`. Remove the inline TODO.
- Verify Dragon entering at the same time as Dragonstorm Globe does
  NOT get the counter (ruling 2025-04-04). This is automatic: per the
  PB-EWC review, `register_permanent_replacement_abilities` runs
  AFTER `apply_etb_replacements` at every ETB call site, so when
  Globe enters, its own replacement is not yet registered — the same
  exclude_self property that Master Biomancer relies on.

## Tests

`crates/engine/tests/primitive_pb_ewcd.rs`:

1. `test_pb_ewcd_hash_schema_version_is_23` — HASH-23 sentinel.
2. `test_pb_ewcd_partial_eq_discriminates_subtype_variant` —
   verifies the new variant `PartialEq`s itself, differs from
   `CreatureControlledBy` and from another subtype value.
3. `test_pb_ewcd_serde_default_backward_compat` — verifies the new
   variant round-trips via JSON serde (proves the additive pattern
   does not break wire format for the existing variants — the new
   discriminant doesn't appear in any pre-EWC-D serialized state).
4. `test_dragonstorm_globe_dragon_etb_gets_extra_counter` — cast Dragonstorm
   Globe (so its ETB replacement is registered), then have a Dragon
   creature enter; verify the entering Dragon has exactly 1
   `+1/+1` counter (the EWC modification from Globe).
5. `test_dragonstorm_globe_non_dragon_etb_no_counter` — cast Globe,
   then have a non-Dragon creature enter; verify zero
   `+1/+1` counters.
6. `test_bind_object_filter_rebinds_owned_by_opponents_of_for_wouldenterbattlefield`
   — register a synthetic non-self `Replacement` ability with
   `trigger: WouldEnterBattlefield { filter:
   OwnedByOpponentsOf(PlayerId(0)) }`; resolve the source onto the
   battlefield (or directly invoke `register_permanent_replacement_abilities`);
   assert the registered `ReplacementEffect.trigger.filter` is
   `OwnedByOpponentsOf(actual_controller)` (not the placeholder).

All 11 existing PB hash canary sentinels bumped 22 → 23 across:
- `primitive_pb_cc_a.rs`, `primitive_pb_xa.rs`,
  `primitive_pb_cc_c_followup.rs`, `primitive_pb_xs_e.rs`,
  `primitive_pb_xs.rs`, `primitive_pb_lki_power.rs`,
  `primitive_pb_ts.rs`, `primitive_pb_eat.rs`,
  `primitive_pb_lki_cc.rs`, `primitive_pb_ewc.rs`,
  `primitive_pb_xa2.rs`.

Plus 5 non-`primitive_pb_` sentinels at the same `22u8` literal:
- `pbt_up_to_n_targets.rs` (×2), `pbn_subtype_filtered_triggers.rs`,
  `pbd_damaged_player_filter.rs`, `pbp_power_of_sacrificed_creature.rs`,
  `effect_sacrifice_permanents_filter.rs`.

Sentinel messages rewritten to cite PB-EWC-D uniformly.

## OOS seeds expected (AC 3908)

- **OOS-EWCD-1** — receiver filter for card-type variants (`"Each
  artifact you control enters with..."`, `"Each enchantment you
  control enters with..."`). Would require either `HasCardType` +
  `ControlledBy` AND-composition or a dedicated
  `PermanentControlledByOfCardType` variant.
- **OOS-EWCD-2** — receiver filter for supertype variants (`"Each
  legendary creature you control enters with..."`). Would require
  a `SuperType` field on a controlled-creature variant or a new
  variant.
- **OOS-EWCD-3** — receiver filter for multi-subtype AND
  composition (`"Each Elf Warrior you control enters with..."`).
  The current new variant carries a single `SubType`; multi-subtype
  would require either a `Vec<SubType>` field or a new variant.

## Verification

- `cargo build --workspace` clean.
- `cargo test --workspace --lib --tests` — expected 2811 → 2811+6
  passing.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.

## Risk register

- Adding a new `ObjectFilter` variant means every exhaustive match on
  `ObjectFilter` must add an arm. The 1M context grep already
  located the call sites (`object_matches_filter`,
  `bind_object_filter`, hash arm). No exhaustive matches were found
  outside `replacement.rs` and `hash.rs` — but the build will catch
  any missed.
- The `subtype` field is `SubType(String)` — a heap allocation.
  Comparison is `String == String`. The PB-EAT precedent
  (`EntersAsAdditionalType { subtype: SubType }`) is identical and
  has shipped cleanly.
- HASH-23 is a wire-format bump. Pre-PB-EWC-D serialized states are
  not forward-compatible.
- The Dragon-ruling check (entering simultaneously with Globe does
  NOT get the counter) is implicit in `apply_etb_replacements` /
  `register_permanent_replacement_abilities` ordering. Verified
  semantically in PB-EWC review; not re-tested here (PB-EWC test (a)
  covers this property for Master Biomancer; PB-EWC-D's
  `dragon_etb_gets_extra_counter` test sequences Globe BEFORE the
  Dragon, so the property is exercised but not as a discriminator).
