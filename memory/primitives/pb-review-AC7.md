# Primitive Batch Review: PB-AC7 — Type-changing & Ability-removal

**Date**: 2026-07-09
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit reviewed**: `1caa8cc1` (branch `feat/pb-ac7-type-changing-ability-removal`)
**CR Rules**: 205.1a (set card types / subtypes; correlated-subtype removal), 205.1b,
613.1d (Layer 4), 613.1f (Layer 6), 613.7 (timestamp), 613.8 (dependency), 708.2 (face-down)
**Engine files reviewed**: `state/continuous_effect.rs`, `rules/layers.rs`
(`apply_layer_modification` + `depends_on`), `state/hash.rs`,
`cards/card_definition.rs` + `rules/abilities.rs` (`spell_subtype_filter`)
**Card defs reviewed (present in worktree, live)**: `darksteel_mutation.rs`,
`kenriths_transformation.rs`, `eaten_by_piranhas.rs`, `vraska_betrayals_sting.rs`
**Tests reviewed**: `crates/engine/tests/pb_ac7_type_change_ability_removal.rs` (14)

## Fix pass (2026-07-09, primitive-impl-runner)

**H1, M1, M2 all RESOLVED.** `cargo test --all` = 3034 passed / 0 failed (3032 baseline +
2 new tests: `test_set_card_types_drops_correlated_subtype_when_card_type_removed`,
`test_set_creature_types_layer4_dependency_nondisjoint_creature_subtype`). `cargo build
--workspace`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` all clean.

- **H1 fix** (`crates/engine/src/rules/layers.rs` `SetCardTypes` arm, `crates/engine/src/state/types.rs`):
  Added six `LazyLock<im::OrdSet<SubType>>` statics for the CR 205.3 correlated-subtype
  categories not yet covered by the existing `ALL_CREATURE_TYPES` — `ALL_ARTIFACT_TYPES`
  (205.3g), `ALL_ENCHANTMENT_TYPES` (205.3h), `ALL_LAND_TYPES` (205.3i),
  `ALL_PLANESWALKER_TYPES` (205.3j), `ALL_SPELL_TYPES` (205.3k, correlates to BOTH
  Instant+Sorcery per the CR's "shared" wording), `ALL_BATTLE_TYPES` (205.3q). All six lists
  verified via the mtg-rules MCP (`get_rule`), not from memory. Added
  `correlated_card_types(subtype: &SubType) -> Vec<CardType>` classification helper
  (returns empty for an unrecognized subtype — conservative "always survives" default).
  The `SetCardTypes` arm now filters `chars.subtypes` after assigning `chars.card_types`:
  a subtype survives iff it's uncorrelated (empty) OR at least one of its correlated card
  types is still present. Verified non-vacuous: reverted the arm to the pre-fix bare
  overwrite, reran the two H1-targeted tests, both FAILED (`Equipment` and `Shrine` both
  survived incorrectly), then restored the fix and reran green.
- **M1 fix** (`crates/engine/src/rules/layers.rs` `depends_on`): added three new payload-aware
  dependency arms (none unconditional except where mirroring the existing `SetTypeLine`
  precedent exactly), each justified against the literal CR 613.8a test ("applying B changes
  what A does"):
  1. `(SetCreatureTypes, AddSubtypes)` — depends iff the added subtypes include a creature
     type (payload-aware; a land/artifact/enchantment-only `AddSubtypes` never touches the
     creature-type subset `SetCreatureTypes` replaces, so no dependency is created for that
     case — avoids the "spurious dependency" the review warned against).
  2. `(SetCardTypes, AddCardTypes)` — unconditional, mirrors the `SetTypeLine`/`AddCardTypes`
     precedent exactly (`SetCardTypes` always overwrites `card_types` wholesale, so any
     co-resident `AddCardTypes` is always order-sensitive).
  3. `(SetCardTypes, AddSubtypes)` and `(SetCardTypes, SetCreatureTypes)` — the "additional
     coupling to H1" the review flagged: since H1's fix makes `SetCardTypes` read the current
     `subtypes` at its own application time, an `AddSubtypes`/`SetCreatureTypes` that runs
     AFTER it would bypass the correlation filter. Both arms are payload-aware (dependency
     only exists when the correlation math could actually differ), not blanket — confirmed
     this doesn't force any reordering on the current roster (all three roster Auras keep
     `Creature` in their `SetCardTypes` payload, so arm 3 evaluates false for them; their
     natural push order already produced the correct result either way, verified by hand).
  Verified non-vacuous: temporarily reverted arm 1 to `false`, reran the new non-disjoint
  test, it FAILED (`{Elk, Zombie}` instead of `{Elk}`), then restored and reran green.
- **M2 fix** (`crates/engine/tests/pb_ac7_type_change_ability_removal.rs`): added
  `test_set_card_types_drops_correlated_subtype_when_card_type_removed` (Artifact+Creature+
  Equipment+Golem → `SetCardTypes({Creature})` → Equipment dropped, Golem survives — the
  exact Kenrith's Transformation / Eaten by Piranhas bug shape); modified
  `test_darksteel_mutation_keeps_indestructible`'s target to carry the Enchantment card type
  + `Shrine`/`God` subtypes (the exact Darksteel Mutation Gatherer-ruling example) so the
  existing integration test now also proves H1; added
  `test_set_creature_types_layer4_dependency_nondisjoint_creature_subtype` (the reviewer's
  exact Zombie counterexample, both orders now converge on `{Elk}`).
- **Hash**: no `HASH_SCHEMA_VERSION` bump. Only `LazyLock` statics + a pure helper function
  were added (no new struct field, no new enum variant) — matches the task brief's explicit
  "adding only LazyLock statics + a helper fn requires no bump" case.

## Verdict: needs-fix (original review below, now resolved — see fix pass above)

The hash work, timestamp-ordering, `spell_subtype_filter`, face-down composition, and
duration-expiry are all correct and well-tested; the "already-expressible" scope decision
(no `Effect::LoseAbilities`, no one-shot Layer-4 override variant) is confirmed correct.
**However, `LayerModification::SetCardTypes` omits the CR 205.1a correlated-subtype-removal
clause**, and the three live roster Auras that use it (Darksteel Mutation, Kenrith's
Transformation, Eaten by Piranhas) therefore produce wrong game state on plausible Commander
targets (enchantment-creature "God" subtype, equipment-creature "Equipment" subtype). That is
one HIGH. A MEDIUM dependency gap (the `depends_on` "disjoint sets" justification is incomplete)
and an associated test gap round out the findings.

**Gate note**: build/test (3023 pass)/clippy/fmt independently confirmed green by the
coordinator; not re-run. This review is correctness/CR-fidelity only.

## CR citation correction — runner credited

The task brief and `pb-plan-AC7`'s scope table both cite **205.1b** as the governing rule for
setting card types / subtypes. That is **wrong**. Verified via mtg-rules MCP:

- **205.1a** is the set-and-replace rule: *"when an effect sets one or more of an object's
  subtypes, the new subtype(s) replaces any existing subtypes from the appropriate set
  (creature types, land types, artifact types, ...)"* and *"In most such cases, the new card
  type(s) replaces any existing card types."* — This is the correct basis for both
  `SetCreatureTypes` and `SetCardTypes`.
- **205.1b** is the *opposite* rule (the "in addition to its other types" / retention rule,
  e.g. "artifact creature" cases that RETAIN prior types).

The runner corrected the brief and cited **205.1a** throughout the engine code and doc
comments. **This is correct and should be credited, not dinged.** (The test module and the
Darksteel Mutation test still carry a "CR 205.1b" label in one doc comment — accurate insofar
as Darksteel's "[creature type] artifact creature" wording is literally the 205.1b sentence —
but the operative replace-semantics rule is 205.1a. LOW, cosmetic.)

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| H1 | **HIGH** — **RESOLVED** | `rules/layers.rs:1090` | **`SetCardTypes` never removes subtypes correlated with removed card types (CR 205.1a).** Live cards (Darksteel Mutation, Kenrith, Eaten by Piranhas) produce wrong game state on enchantment-creature / equipment-creature targets. **Fix:** after setting `chars.card_types`, drop any subtype whose correlated card type is no longer present. **Resolved**: added `correlated_card_types()` + 6 CR-205.3 `LazyLock` subtype-set statics in `state/types.rs`, filter added to the `SetCardTypes` arm. See "Fix pass" above. |
| M1 | MEDIUM — **RESOLVED** | `rules/layers.rs:1375` | **No CR 613.8 `depends_on` arm for `SetCreatureTypes`/`SetCardTypes`, justified only for disjoint subtype sets.** A co-resident `AddSubtypes` that adds a *creature* type (Xenograft / Arcane Adaptation / Conspiracy) is non-disjoint and order-dependent — inconsistent with the `SetTypeLine` precedent. **Fix:** add the dependency arm (or document the timestamp choice with a correct CR rationale, not the "disjoint" claim). **Resolved**: 3 payload-aware dependency arms added (`SetCreatureTypes`↔`AddSubtypes`, `SetCardTypes`↔`AddCardTypes`, `SetCardTypes`↔`AddSubtypes`/`SetCreatureTypes`). See "Fix pass" above. |
| M2 | MEDIUM — **RESOLVED** | test file | **Test gap masks H1 and over-generalizes M1.** No test exercises `SetCardTypes` dropping a correlated subtype; the dependency test only covers the disjoint (land-subtype) case. **Fix:** add a correlated-subtype-removal test and a non-disjoint (`AddSubtypes` of a creature type) ordering test. **Resolved**: 2 new tests added, 1 existing test (Darksteel integration) strengthened to carry a droppable subtype. Both H1 and M1 fixes proven non-vacuous by revert-and-rerun. See "Fix pass" above. |
| L1 | LOW | `rules/layers.rs:1090` | `SetCardTypes` ignores CR 205.1a instant/sorcery retention. Not reachable on battlefield permanents (same latent gap in `SetTypeLine`). |
| L2 | LOW | `rules/abilities.rs:3368` | `spell_subtype_filter` reads the stack object's raw `characteristics.subtypes` rather than `calculate_characteristics`. Consistent with the sibling `spell_type_filter` (line 3366) which already reads raw `card_types`; not a W3-LC regression. Noted only. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | (via H1) | `darksteel_mutation.rs`, `kenriths_transformation.rs`, `eaten_by_piranhas.rs` | Card defs are authored correctly *given the primitive's contract*; the wrong game state originates in the engine (H1), and the single engine fix repairs all three. No card-def change required. |

### Finding Details

#### Finding H1: `SetCardTypes` omits CR 205.1a correlated-subtype removal — live wrong game state

**Severity**: HIGH
**File**: `crates/engine/src/rules/layers.rs:1090`
**CR 205.1a**: *"If an object's card type is removed, the subtypes correlated with that card
type will remain if they are also the subtypes of a card type the object currently has;
otherwise, they are also removed for the entire time the object's card type is removed."*
**Oracle/rulings**:
- Darksteel Mutation ruling (2013-10-17): *"If it had any other artifact subtypes (such as
  Equipment), it will retain those. If it had any subtypes other than artifact types and
  creature types (such as Shrine), it won't retain those."*
- Kenrith's Transformation ruling (2019-10-04): *"loses any other card types it has (such as
  artifact)"* — the correlated subtypes of that removed Artifact drop with it.

**Issue**: the arm is a bare overwrite:
```rust
LayerModification::SetCardTypes(new_types) => {
    chars.card_types = new_types.clone();
}
```
Its doc comment even states the (incorrect) contract "leaving supertypes and subtypes
untouched." Per 205.1a, removing a card type must also remove that type's correlated subtypes
unless they remain correlated with a surviving card type. The companion `SetCreatureTypes`
only cleans the *creature*-type subset, so nothing removes a stale *enchantment* / *artifact*
subtype.

**Reachability (confirmed live in the reviewed worktree)** — all three roster Auras are
authored and use `SetCardTypes`:
- `kenriths_transformation.rs` / `eaten_by_piranhas.rs`: `SetCardTypes({Creature})` (removes
  Artifact + Enchantment). Cast on a Reconfigure equipment-creature (e.g. a "…Blades" that is
  an Artifact Creature — Equipment): Artifact card type is removed, so the **Equipment**
  artifact subtype must drop — the engine leaves it on. Wrong game state.
- `darksteel_mutation.rs`: `SetCardTypes({Artifact, Creature})` (removes Enchantment). Cast on
  a God (enchantment creature, subtype **God**) or a Shrine enchantment creature: the
  Enchantment card type is removed, so the enchantment subtype must drop — the engine leaves it
  on. Exactly the "Shrine won't retain" ruling. Wrong game state.

These are plausible Commander board states, and the primitive is *documented for exactly these
cards*. Per the reviewer rubric ("engine change contradicts CR rule text" and "card def
produces wrong game state") and the W6 red line ("no wrong game state"), this is HIGH.

**Fix**: in the `SetCardTypes` arm, after assigning `chars.card_types = new_types.clone()`,
remove any subtype whose owning card-type set is no longer represented in `chars.card_types`.
Use the CR-205.3 correlation (creature subtypes ↔ Creature, land subtypes ↔ Land, artifact
subtypes ↔ Artifact, enchantment subtypes ↔ Enchantment, planeswalker subtypes ↔ Planeswalker,
spell subtypes ↔ Instant/Sorcery). A subtype is kept iff at least one card type it correlates
to is still present. Add the test in M2. Update the doc comment (the "subtypes untouched"
claim is false). This single engine fix repairs Darksteel Mutation, Kenrith, and Eaten by
Piranhas simultaneously — no card-def edits needed.

#### Finding M1: CR 613.8 dependency — "disjoint sets" justification is incomplete

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/layers.rs:1375-1388` (`depends_on`)
**CR 613.8a**: A depends on B if *"applying the other would change … what it does to any of the
things it applies to."*
**Issue**: the runner added no `depends_on` arm, arguing both new variants "only replace ONE
subset of the type line" so a co-resident `AddSubtypes` is "order-independent." That holds
**only when the added subtype is disjoint from the set the SET-effect touches** (e.g. a land
subtype vs `SetCreatureTypes`). It does **not** hold when `AddSubtypes` adds a *creature* type:

Concrete counterexample — `SetCreatureTypes({Elk})` co-resident with `AddSubtypes({Zombie})`
(Zombie ∈ `ALL_CREATURE_TYPES`), same layer:
- `AddSubtypes` older → applied first → `SetCreatureTypes` filters out Zombie → `{Elk}`.
- `AddSubtypes` newer → `SetCreatureTypes` first → `{Elk}` → `AddSubtypes` adds Zombie → `{Elk, Zombie}`.

The engine (`SetCreatureTypes` reads `chars.subtypes` and filters at apply-time) makes the
result timestamp-dependent, and applying `AddSubtypes` first *does* change what
`SetCreatureTypes` does (it now removes an extra Zombie) — a 613.8a dependency. `SetTypeLine`
already carries exactly this dependency vs `AddSubtypes` (line 1369); `SetCreatureTypes` is the
creature-subtype analog and the asymmetry is unjustified. Real cards exist on both sides
(Kenrith's Transformation is `SetCreatureTypes`; Xenograft / Arcane Adaptation / Conspiracy are
creature-type `AddSubtypes`). Not reachable from the *current* roster in a single game (needs a
second card), hence MEDIUM rather than HIGH.

**Additional coupling to H1**: once H1 is fixed, `SetCardTypes` will read `chars.card_types` to
decide which subtypes to drop — so a co-resident `AddCardTypes` (or `SetCreatureTypes` feeding
the Creature correlation) changes what `SetCardTypes` removes. The H1 fix must therefore also
add the appropriate `SetCardTypes`-vs-`AddCardTypes` dependency arm; do not land H1 without
reconsidering `depends_on`.

**Fix**: add `(SetCreatureTypes, AddSubtypes)` and (post-H1) `(SetCardTypes, AddCardTypes)`
dependency arms mirroring the `SetTypeLine` precedent; replace the inline "disjoint" rationale
with the correct scoped statement.

#### Finding M2: test gap masks H1, over-generalizes M1

**Severity**: MEDIUM
**File**: `crates/engine/tests/pb_ac7_type_change_ability_removal.rs`
**Issue**:
- `test_set_card_types_replaces_card_types_preserves_supertypes` keeps a **Golem** (creature)
  subtype while **Creature** stays present, so no correlated subtype is ever eligible to drop —
  it cannot detect H1.
- `test_darksteel_mutation_keeps_indestructible` uses a plain 5/5 flyer with no non-creature
  subtypes, so the Insect result is achieved by `SetCreatureTypes` alone — it also cannot
  detect H1.
- `test_set_creature_types_layer4_dependency_with_add_subtypes` only exercises a **land**
  subtype (disjoint), so its order-independence "proves" exactly the safe case the runner then
  over-generalized in M1.
**Fix**: add (1) a `SetCardTypes` test where a removed card type's correlated subtype must
drop (e.g. Artifact+Creature+Equipment → `SetCardTypes({Creature})` → Equipment gone), and (2)
a non-disjoint dependency test (`AddSubtypes` of a creature type vs `SetCreatureTypes`, both
orders) asserting the CR-correct result once M1 lands.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 205.1a subtype-set (creature) | Yes | Yes | `SetCreatureTypes` blanket-preserves non-creature subtypes — CORRECT per "appropriate set (creature types)". |
| 205.1a card-type-set | Partial | Partial | Overwrite works; **correlated-subtype removal missing (H1)**. |
| 205.1a instant/sorcery retention | No | No | L1, latent/unreachable. |
| 613.1d Layer 4 placement | Yes | Yes | Both variants in the TypeChange block, mirrors `AddAllCreatureTypes`. |
| 613.1f RemoveAllAbilities | Yes (pre-existing) | Yes | Composition validated. |
| 613.7 timestamp ordering | Yes | Yes | `test_granted_then_removed…` both orders, wedge on `keywords` (non-vacuous). |
| 613.8 dependency | Partial | Partial | Disjoint case only; **M1/M2**. |
| 708.2 face-down override | Yes | Yes | `test_lose_abilities_vs_face_down_override` — `power==2` tell proves override ran. |
| 514.2 UntilEndOfTurn expiry | Yes | Yes | Calls real `expire_end_of_turn_effects`; non-vacuous. |
| 205.1a spell_subtype_filter | Yes | Yes | Real CastSpell integration (Equipment/Vehicle/Aura + negative + None regression). |

## Recurring-failure-mode gate results (per task brief)

1. **hash.rs completeness** — CLEAN. Enumerated every `LayerModification` hash discriminant:
   0 CopyOf, 1 SetController, 2 SetTypeLine, 3 AddCardTypes, 4 AddSubtypes, 5 LoseAllSubtypes,
   6 SetColors, 7 AddColors, 8 BecomeColorless, 9 AddKeyword, 10 AddKeywords,
   11 RemoveAllAbilities, 12 RemoveKeyword, 13 SetPtViaCda, 14 SetPtToManaValue,
   15 SetPowerToughness, 16 ModifyPower, 17 ModifyToughness, 18 ModifyBoth,
   19 SwitchPowerToughness, 20 AddAllCreatureTypes, 21 RemoveCardTypes, 22 SetPtDynamic,
   23 AddActivatedAbility, 24 AddManaAbility, 25 ModifyBothDynamic, 26 ModifyPowerDynamic,
   27 ModifyToughnessDynamic, 28 SetBothDynamic, **29 RemoveSuperType**, **30 SetCreatureTypes**,
   **31 SetCardTypes** — 0..31 contiguous, unique, no collision; 30/31 are new and free.
   `WheneverYouCastSpell` arm (hash.rs:5020) destructures `spell_subtype_filter` explicitly and
   calls `spell_subtype_filter.hash_into(hasher)` — hashed, not dropped via `..`.
   `HASH_SCHEMA_VERSION = 34` with changelog entry; no stale `33` sentinel remains (grep clean;
   `test_hash_schema_version_is_34` would fail otherwise). **No new mutable/runtime
   GameState/PlayerState/GameObject field this batch** → the PB-AC1/AC5 mutation-verified-field
   HIGH does not apply; the two hash-distinguishes tests cover the new variant + field.
2. **CR correctness of SetCreatureTypes/SetCardTypes** — 205.1a citation CORRECT (see above).
   `SetCreatureTypes` supertype-preserve + non-creature-subtype blanket-preserve is CORRECT
   ("appropriate set" = creature types only). `SetCardTypes` fails the correlated-removal
   clause → **H1**.
3. **613.8 dependency** — counterexample constructed → **M1** (MEDIUM).
4. **613.7 timestamp** — stable `sort_by_key(timestamp)`; both-orders test is genuine and the
   assertions differ per order (not vacuous). CLEAN.
5. **Test quality** — the 3 `spell_subtype_filter` tests, face-down test (power-2 tell), and
   duration-expiry test (real `expire_end_of_turn_effects`) are all non-vacuous; the trigger
   source is properly enriched via `enrich_spec_from_def`. Gaps: **M2** (H1 + non-disjoint
   ordering untested).
6. **spell_subtype_filter semantics** — OR-match across the list (`.any`), AND'd with
   `spell_type_filter`/`noncreature_only`/`chosen_subtype_filter` (each an independent
   early-return); `None` preserves prior behavior for all construction sites; reads
   `spell_subtypes` (raw stack characteristics, consistent with the sibling card-type read).
   CORRECT.
7. **W3-LC** — no new battlefield-permanent characteristic read bypasses
   `calculate_characteristics`; the stack-spell raw read at abilities.rs:3368 matches existing
   precedent (**L2**). CLEAN.

## Deliberately-not-added decisions — confirmed correct

- `Effect::LoseAbilities`: expressible as `Effect::ApplyContinuousEffect` +
  `LayerModification::RemoveAllAbilities` with any duration/filter — proven by
  `test_lose_abilities_one_shot_until_eot` (real `execute_effect`) and the Darksteel
  composition. Correct to omit.
- One-shot Layer-4 override with duration: expressible via `Effect::ApplyContinuousEffect`
  over a `ContinuousEffectDef { layer: TypeChange, … }`. Correct to omit.

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| darksteel_mutation | Yes (structure) | 0 (roster clause) | **No on God/Shrine target** | H1 — engine fix, not card-def fix |
| kenriths_transformation | Yes (structure) | 0 (roster clause) | **No on equipment-creature target** | H1 |
| eaten_by_piranhas | Yes (structure) | 0 (roster clause) | **No on equipment-creature target** | H1 |
| vraska_betrayals_sting | −2 clause only | −9 clause (OOS-AC7-1) | −2 clause OK | uses SetTypeLine; not affected by H1 |

## Recommendation

Fix **H1** (engine `SetCardTypes` correlated-subtype removal) before backfill sign-off — it is
the W6 wrong-game-state red line and repairs all three roster Auras at once. Address **M1/M2**
in the same fix pass (the H1 fix forces a `depends_on` reconsideration). Everything else —
hash, timestamp, face-down, duration, `spell_subtype_filter`, scope decisions, CR 205.1a
citation — is correct as shipped.
