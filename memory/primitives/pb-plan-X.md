# Primitive Batch Plan: PB-X — A-42 Tier 1 Micro-PB

> ## DO NOT IMPLEMENT THIS SESSION
> This is a plan-phase deliverable. Implementation runs in a separate session
> after oversight resolves the open questions at the bottom.

**Generated**: 2026-04-11
**Primitive bundle**: Three small primitives that unblock A-42 Tier 1 card authoring.
**CR Rules**: 118 (costs), 602.1 (activated ability shape), 608.2h (effects determine
numeric values once at apply time), 611.3 (continuous effects from spells), 613.1f
(Layer 6), 701.10 (Exile as cost / action), 205.3m (creature subtypes).
**Cards affected**: 4 directly authored as Tier 1 (Crippling Fear, Eyeblight Massacre,
Olivia's Wrath, Balthor the Defiled), 2 verified-already-authorable and authored in
the same session (Obelisk of Urd, City on Fire), 1 deferred (Metallic Mimic), 1
out-of-scope (Heritage Druid).
**Dependencies**: None new — builds on `Effect::ApplyContinuousEffect`,
`ContinuousEffectDef`, `ActivationCost`, existing `resolve_amount`, existing
`ctx.chosen_creature_type` plumbing.
**Deferred from prior PBs**: None claimed for closure. PB-S residual LOWs L01..L06
remain in mana_solver / abilities.rs and are NOT in PB-X scope.

---

## Executive Summary

PB-X bundles three independent primitives. Estimated total: **~140 LOC engine** +
**~120 LOC card defs** + **~14 unit tests**.

| # | Primitive | Where | LOC | Unblocks |
|---|-----------|-------|-----|----------|
| 1 | `EffectFilter::AllCreaturesExcludingSubtype(SubType)` + `EffectFilter::AllCreaturesExcludingChosenSubtype` | `state/continuous_effect.rs`, `rules/layers.rs`, `state/hash.rs` | ~50 | Crippling Fear, Eyeblight Massacre, Olivia's Wrath |
| 2 | `LayerModification::ModifyBothDynamic(Box<EffectAmount>)` | `state/continuous_effect.rs`, `effects/mod.rs` (resolve-time substitution), `state/hash.rs` | ~40 | Olivia's Wrath |
| 3 | `Cost::ExileSelf` + `ActivationCost.exile_self` + payment block | `cards/card_definition.rs`, `state/game_object.rs`, `rules/abilities.rs`, `testing/replay_harness.rs`, `state/hash.rs` | ~50 | Balthor the Defiled |

A fourth design question — Metallic Mimic's "is the chosen type in addition to its
other types" — is answered **explicitly out of PB-X scope** by the planning phase
(see "Stop-and-flag finding" below). It needs a fifth, distinct primitive
(`LayerModification::AddChosenCreatureType` reading the source's
`chosen_creature_type` at apply time). Spawning that into PB-X would violate the
"three primitives only" boundary set by oversight.

---

## Full-Chain Verification Per Card

This walks every card in scope from oracle text → effect → filter → layer
→ cost → resolution dispatch. The PB-S session repeatedly broke on stopping at
"the field exists"; this section traces the entire dispatch chain end-to-end.

### Crippling Fear — `{2}{B}{B}` Sorcery — **Tier 1, needs Primitive #1 (chosen variant)**

**Oracle**: "Choose a creature type. Creatures that aren't of the chosen type get
-3/-3 until end of turn."

| Hop | Status | Notes |
|-----|--------|-------|
| Spell-level chosen-type | OK | `Effect::ChooseCreatureType { default }` exists (used by Kindred Dominance); sets `ctx.chosen_creature_type` (`effects/mod.rs:2632`). |
| Pump effect | OK | `Effect::ApplyContinuousEffect { effect_def: ContinuousEffectDef { ... } }`. |
| EffectFilter for "creatures not of chosen type, all controllers" | **MISSING** | `CreaturesYouControlExcludingSubtype(SubType)` is the closest existing variant — wrong scope (only "you control") and takes a static SubType, not the chosen one. PB-X adds `AllCreaturesExcludingChosenSubtype`, substituted at `Effect::ApplyContinuousEffect` time using `ctx.chosen_creature_type`. |
| LayerModification for -3/-3 | OK | `LayerModification::ModifyBoth(-3)` (existing). |
| EffectDuration | OK | `EffectDuration::UntilEndOfTurn`. |
| Stack/source dispatch | OK | The stored `ContinuousEffect.source` is the spell stack object id at the moment of execution. Filter resolution doesn't need the source after substitution because the SubType is concrete. |

**Authorability after PB-X**: YES.

### Eyeblight Massacre — `{2}{B}{B}` Sorcery — **Tier 1, needs Primitive #1 (static variant)**

**Oracle**: "Non-Elf creatures get -2/-2 until end of turn."

| Hop | Status | Notes |
|-----|--------|-------|
| Pump effect | OK | `Effect::ApplyContinuousEffect`. |
| EffectFilter "non-Elf creatures, all controllers" | **MISSING** | `CreaturesYouControlExcludingSubtype` is "you control" only. PB-X adds `AllCreaturesExcludingSubtype(SubType)` for the all-player static-subtype case. |
| LayerModification for -2/-2 | OK | `LayerModification::ModifyBoth(-2)`. |
| Duration | OK | `EffectDuration::UntilEndOfTurn`. |

**Authorability after PB-X**: YES.

### Olivia's Wrath — `{4}{B}` Sorcery — **Tier 1, needs Primitives #1 + #2**

**Oracle**: "Each non-Vampire creature gets -X/-X until end of turn, where X is the
number of Vampires you control."

| Hop | Status | Notes |
|-----|--------|-------|
| Pump effect | OK | `Effect::ApplyContinuousEffect`. |
| EffectFilter "non-Vampire creatures, all controllers" | **MISSING (Primitive #1)** | Same as Eyeblight Massacre. |
| LayerModification dynamic -X/-X | **MISSING (Primitive #2)** | `ModifyBoth(i32)` is static. CR 608.2h says X is determined once when the effect applies (i.e., at spell resolution). PB-X adds `ModifyBothDynamic(Box<EffectAmount>)` that is **resolved at `Effect::ApplyContinuousEffect` execution time** into a concrete `ModifyBoth(i32)`. The stored `ContinuousEffect` therefore always carries the existing primitive — no layer-time arm needed. |
| EffectAmount for "Vampires you control" | OK | `EffectAmount::PermanentCount { filter: TargetFilter { has_subtype: Some(Vampire), ... }, controller: PlayerTarget::Controller }` (existing). |
| Sign handling | NOTE | Need to negate the resolved value before passing to `ModifyBoth`, OR introduce a sign discriminant on `ModifyBothDynamic`. Plan picks the latter — `ModifyBothDynamic { amount: Box<EffectAmount>, negate: bool }` — to keep the substitution path mechanical. Alternative: `Negate(Box<EffectAmount>)` combinator on `EffectAmount` itself. **Open question, see below.** |
| Duration | OK | `UntilEndOfTurn`. |

**Authorability after PB-X**: YES.

### Balthor the Defiled — `{2}{B}{B}` Legendary Creature — Zombie Dwarf — **Tier 1, needs Primitive #3**

**Oracle**: "Minion creatures get +1/+1. {B}{B}{B}, Exile Balthor: Each player
returns all black and all red creature cards from their graveyard to the
battlefield."

| Hop | Status | Notes |
|-----|--------|-------|
| Static "Minion creatures get +1/+1" | OK | `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef { layer: PtModify, modification: ModifyBoth(1), filter: AllCreaturesWithSubtype(SubType("Minion")), ... } }` — uses existing primitives. |
| Activated ability cost: `{B}{B}{B}, Exile Balthor` | **MISSING** | No `Cost::ExileSelf` variant. PB-X adds it + `ActivationCost.exile_self: bool`. Cost-payment block in `abilities.rs` mirrors `sacrifice_self` block (~lines 586-620): captures pre-move data, calls `state.move_object_to_zone(source, ZoneId::Exile)`, emits `PermanentExiled` event. |
| LKI / source-after-exile | OK | `embedded_effect` plumbing in `ActivatedAbility` resolution path (`resolution.rs:1767-1798`) is **already** the LKI mechanism for sacrificed sources. The capture happens at `abilities.rs:294-313` BEFORE any cost is paid (`ab.effect.clone()` into `embedded_effect`). For ExileSelf, the same `embedded_effect: Some(Box::new(...))` carries the resolution effect onto the stack object. After exile, `state.objects.get(&source_object)` returns `None`, but the resolution path falls back to `embedded_effect`. **No new dispatch wiring needed** beyond the cost payment block. |
| Mass reanimate effect | OK | PB-H added `Effect::ReturnAllFromGraveyardToBattlefield` (or `LivingDeath`-style). Verify which exact effect variant: **per-player, filtered by color**. Plan-phase grep needed at impl time — see open question. If a sufficient variant exists, fine; if not, this card moves to "needs Primitive #4" and is BLOCKED in PB-X. |
| Duration | n/a (one-shot effect) |  |

**Authorability after PB-X**: **YES** if the existing `ReturnAllFromGraveyardToBattlefield` (PB-H) supports per-player + color filter. **OPEN VERIFICATION** — see open question 4. If it doesn't, the engine path needs a small extension or the card stays partially authored (cost block done, body TODO'd).

### Metallic Mimic — `{2}` Artifact Creature — Shapeshifter — **OUT OF PB-X SCOPE**

**Oracle**: "As this creature enters, choose a creature type. This creature is the
chosen type in addition to its other types. Each other creature you control of the
chosen type enters with an additional +1/+1 counter on it."

| Hop | Status | Notes |
|-----|--------|-------|
| ETB chosen-type | OK | `Effect::ChooseCreatureType` exists; can be wired as a `SelfEntersBattlefield` triggered ability that sets `chosen_creature_type` on the source object (`effects/mod.rs:2628`). |
| "This creature is the chosen type in addition to its other types" | **MISSING — DISTINCT FOURTH PRIMITIVE** | `LayerModification::AddSubtypes(OrdSet<SubType>)` only takes a fixed set (`rules/layers.rs:1016`). No `AddChosenCreatureType` variant exists that reads the source's `chosen_creature_type` at apply time. The `EffectFilter::CreaturesYouControlOfChosenType` shape (`layers.rs:908`) demonstrates the dynamic-read pattern — but no parallel `LayerModification` exists. |
| EntersWithCounters replacement on other matching creatures | OK (likely) | `ReplacementModification::EntersWithCounters` exists. Filter `OtherCreaturesYouControlOfChosenType` exists. |

**Verdict**: **Stop-and-flag**. Adding `LayerModification::AddChosenCreatureType` (or
a `LayerModification::AddSubtypesDynamic { source: ChosenTypeRef }`) would be a
**fourth primitive** in this micro-PB. Per oversight scope rules, PB-X is bounded to
three primitives; spawning a fourth here violates the boundary.

**Recommendation**: Defer Metallic Mimic to a follow-up micro-PB **PB-Y** (or fold
into PB-Q ChooseColor when that lands, since both are "ETB choice that the source
itself reads as a layer effect"). Document in `memory/card-authoring/a42-retriage-2026-04-10.md`.

### Obelisk of Urd — `{6}` Artifact (Convoke) — **VERIFY → AUTHOR IN SAME SESSION**

**Oracle**: "Convoke. As this artifact enters, choose a creature type. Creatures
you control of the chosen type get +2/+2."

| Hop | Status | Notes |
|-----|--------|-------|
| Convoke | OK | `KeywordAbility::Convoke` exists (CR 702.51) — used by ~30 cards. |
| ETB chosen-type | OK | Same as Metallic Mimic above — `Effect::ChooseCreatureType` sets `obj.chosen_creature_type` on the source permanent (not the spell ctx). |
| Pump filter "creatures you control of the chosen type" | OK | `EffectFilter::CreaturesYouControlOfChosenType` (`continuous_effect.rs:209`) reads `source.chosen_creature_type` dynamically at layer-application time (`layers.rs:908`). |
| LayerModification +2/+2 | OK | `LayerModification::ModifyBoth(2)`. |
| Duration | OK | `WhileSourceOnBattlefield` (static ability). |
| Self-typing concern | n/a | Obelisk is an Artifact, not a creature — its own type is irrelevant; the pump filter is over creatures. Unlike Metallic Mimic, Obelisk doesn't need to add the chosen type to itself. |

**Verdict**: **Already authorable.** The 2026-04-10 retriage's "needs verification"
was overcautious. PB-X session authors Obelisk of Urd as a free win.

### City on Fire — `{5}{R}{R}{R}` Enchantment (Convoke) — **VERIFY → AUTHOR IN SAME SESSION**

**Oracle**: "Convoke. If a source you control would deal damage to a permanent or
player, it deals triple that damage instead."

| Hop | Status | Notes |
|-----|--------|-------|
| Convoke | OK | Same as above. |
| Replacement effect | OK | `ReplacementTrigger::DamageWouldBeDealt { target_filter: DamageTargetFilter::FromControllerSources(controller), ... } modification: ReplacementModification::TripleDamage` — exact same shape as Fiery Emancipation (`cards/defs/fiery_emancipation.rs:18-22`). |
| Source filter "you control" | OK | `DamageTargetFilter::FromControllerSources(PlayerId(0))` placeholder gets resolved to the actual controller at registration. Verified by reading Angrath's Marauders, Fiery Emancipation. |

**Verdict**: **Already authorable.** The 2026-04-10 retriage cited "needs an 'any
source' filter not appropriate" — that was a misread; `FromControllerSources` is
exactly "any source you control". PB-X session authors City on Fire as a free win.

### Heritage Druid — `{G}` Creature — Elf Druid — **OUT OF PB-X SCOPE (BLOCKED)**

**Oracle**: "Tap three untapped Elves you control: Add {G}{G}{G}."

The activation cost is "tap three other untapped permanents matching a filter" —
no existing `Cost` variant supports this. `Cost::Tap` taps the source only. This is
a substantial cost-framework extension and was explicitly excluded from PB-X by the
oversight scope. Defer to a separate `TapNCreatures` cost-framework PB.

**Verdict**: stays blocked. Not authored in PB-X session.

---

## Per-Primitive Design

### Primitive 1: Exclusion EffectFilter variants (static + chosen)

**Files & changes**:

- `crates/engine/src/state/continuous_effect.rs` — add 2 enum variants in
  `EffectFilter` after `OtherCreaturesYouControlOfChosenType` (line 218):

  ```rust
  /// Applies to all creature permanents on the battlefield (any controller) that
  /// do NOT have the specified subtype.
  ///
  /// Used for "Non-Elf creatures get -2/-2" (Eyeblight Massacre), "non-Vampire
  /// creatures" (Olivia's Wrath after dynamic-amount substitution).
  AllCreaturesExcludingSubtype(SubType),
  /// DSL placeholder: "creatures that aren't of the chosen type" — substituted
  /// at `Effect::ApplyContinuousEffect` execution time into
  /// `AllCreaturesExcludingSubtype(ctx.chosen_creature_type)`.
  ///
  /// Used for "Choose a creature type. Creatures that aren't of the chosen type
  /// get -3/-3" (Crippling Fear).
  ///
  /// Note: this variant should never appear in a stored `ContinuousEffect`. It
  /// only exists in `ContinuousEffectDef` literals on card definitions and is
  /// substituted before storage. Layers code does not handle it.
  AllCreaturesExcludingChosenSubtype,
  ```

- `crates/engine/src/rules/layers.rs` — add **one** match arm in `is_effect_active`
  (after `CreaturesYouControlExcludingSubtype` at line 836). The chosen-subtype
  variant is unreachable post-substitution; document the unreachable arm with
  `unreachable!("AllCreaturesExcludingChosenSubtype must be substituted at apply time")`
  to fail fast if substitution is forgotten:

  ```rust
  EffectFilter::AllCreaturesExcludingSubtype(subtype) => {
      obj_zone == ZoneId::Battlefield
          && chars.card_types.contains(&CardType::Creature)
          && !chars.subtypes.contains(subtype)
  }
  EffectFilter::AllCreaturesExcludingChosenSubtype => {
      // CR 608.2h: this placeholder must be substituted at Effect::ApplyContinuousEffect
      // execution time. Reaching it during layer application is a substitution bug.
      debug_assert!(
          false,
          "AllCreaturesExcludingChosenSubtype must be substituted before storage"
      );
      false
  }
  ```

- `crates/engine/src/effects/mod.rs` — extend the `resolved_filter` match at line
  2220 to substitute `AllCreaturesExcludingChosenSubtype`:

  ```rust
  CEFilter::AllCreaturesExcludingChosenSubtype => {
      match ctx.chosen_creature_type.clone() {
          Some(subtype) => CEFilter::AllCreaturesExcludingSubtype(subtype),
          None => {
              // No chosen type bound — effect produces no result. Skip silently
              // to mirror DeclaredTarget no-target handling.
              return;
          }
      }
  }
  ```

- `crates/engine/src/state/hash.rs` — add 2 hash arms in `impl HashInto for EffectFilter`
  after discriminant 31 (`OtherCreaturesYouControlOfChosenType`):

  ```rust
  EffectFilter::AllCreaturesExcludingSubtype(st) => {
      32u8.hash_into(hasher);
      st.hash_into(hasher);
  }
  EffectFilter::AllCreaturesExcludingChosenSubtype => 33u8.hash_into(hasher),
  ```

  **Discriminants**: 32, 33. Next free EffectFilter discriminant after PB-X = 34.

**Exhaustive match audit**: ran `rg "match .*EffectFilter|EffectFilter::AllCreatures\b"`.
Engine match sites:
- `state/hash.rs:1196` (HashInto)
- `rules/layers.rs:541` (`is_effect_active`)
- `effects/mod.rs:2220` (`Effect::ApplyContinuousEffect` resolved_filter)

Test files reference variants by name but don't `match` exhaustively. **TUI and
replay-viewer do NOT match on `EffectFilter`** (verified `rg EffectFilter:: tools/`
returns nothing). No cross-crate cascade.

### Primitive 2: `LayerModification::ModifyBothDynamic`

**Decision: new variant, not migration.** `ModifyBoth(i32)` has **76 call sites**
across `crates/engine/src/cards/defs/` (verified via `rg "LayerModification::ModifyBoth"`).
A migration to `ModifyBoth(EffectAmount)` would touch every single one. A new
variant isolates the change to the dynamic-amount sites only.

**Files & changes**:

- `crates/engine/src/state/continuous_effect.rs` — add variant after
  `ModifyBoth(i32)` at line 346:

  ```rust
  /// DSL placeholder: dynamic +X/+X (or -X/-X) where X is an `EffectAmount`
  /// resolved at `Effect::ApplyContinuousEffect` execution time (CR 608.2h).
  ///
  /// Substituted into `ModifyBoth(resolved_value)` before the `ContinuousEffect`
  /// is stored, so layer-application code never sees this variant. Used for
  /// "creatures get -X/-X where X is the number of Vampires you control"
  /// (Olivia's Wrath).
  ///
  /// `negate=true` produces `-X` from a non-negative amount; `negate=false`
  /// produces `+X`. Boxed to avoid `large_enum_variant` clippy warnings.
  ModifyBothDynamic {
      amount: Box<EffectAmount>,
      negate: bool,
  },
  ```

- `crates/engine/src/effects/mod.rs` — extend the `Effect::ApplyContinuousEffect`
  arm (after the `resolved_filter` block at line 2236) with a `resolved_modification`
  binding:

  ```rust
  use crate::state::continuous_effect::LayerModification as LM;
  let resolved_modification = match &effect_def.modification {
      LM::ModifyBothDynamic { amount, negate } => {
          let raw = resolve_amount(state, amount, ctx);
          let v = if *negate { -raw } else { raw };
          LM::ModifyBoth(v)
      }
      other => other.clone(),
  };
  // ... use resolved_modification instead of effect_def.modification.clone() in the
  // ContinuousEffect builder at line 2256.
  ```

- `crates/engine/src/rules/layers.rs` — add an `unreachable`/`debug_assert` arm in
  `apply_layer_modification` (after `ModifyBoth` at line 1117) to catch
  substitution bugs:

  ```rust
  LayerModification::ModifyBothDynamic { .. } => {
      debug_assert!(
          false,
          "ModifyBothDynamic must be substituted at Effect::ApplyContinuousEffect time"
      );
      // Production behavior: silently no-op rather than panic.
  }
  ```

- `crates/engine/src/state/hash.rs` — add hash arm with discriminant 25 (next free
  after `SetPtDynamic = 22`, `AddActivatedAbility = 23`, `AddManaAbility = 24`;
  RemoveSuperType claims 26 already):

  ```rust
  LayerModification::ModifyBothDynamic { amount, negate } => {
      25u8.hash_into(hasher);
      amount.hash_into(hasher);
      negate.hash_into(hasher);
  }
  ```

  **Discriminant**: 25. Next free LayerModification discriminant after PB-X = 27
  (since 26 = RemoveSuperType).

**Exhaustive match audit**: `rg "match .*LayerModification" crates/`. Match sites:
- `state/hash.rs:1278` (HashInto)
- `rules/layers.rs:962` (`apply_layer_modification`)
- `rules/layers.rs:1234`+ (dependency check pairs — ModifyBothDynamic does not
  participate in dependencies; no new pair arms needed)

Card defs construct variants but don't match on the enum. TUI/replay-viewer don't
match. No cross-crate cascade.

### Primitive 3: `Cost::ExileSelf`

**Files & changes**:

- `crates/engine/src/cards/card_definition.rs` — add variant after `SacrificeSelf`
  at line 1064:

  ```rust
  /// Exile this permanent as a cost (CR 701.10, CR 602.2). Used for activated
  /// abilities like Balthor the Defiled's "{B}{B}{B}, Exile Balthor: ..."
  ///
  /// LKI behavior: the ability's effect is captured into `embedded_effect` on
  /// the stack object before the source is moved to exile, mirroring the
  /// `SacrificeSelf` plumbing in `rules/abilities.rs`. Resolution falls back
  /// to the embedded effect when `state.objects.get(source)` returns None.
  ExileSelf,
  ```

- `crates/engine/src/state/game_object.rs` — add field on `ActivationCost` after
  `sacrifice_self` at line 224:

  ```rust
  /// CR 701.10 / CR 602.2: True if activating this ability requires exiling the
  /// source permanent as a cost. Mirrors `sacrifice_self`; differs only in the
  /// destination zone and the emitted event.
  #[serde(default)]
  pub exile_self: bool,
  ```

- `crates/engine/src/rules/abilities.rs` — add a payment block immediately after
  `sacrifice_self` block (line 620). Mirrors lines 586-620 but moves to exile:

  ```rust
  // CR 701.10 / CR 602.2c: Pay exile-self cost. Move source to its owner's
  // exile zone before pushing the ability to the stack. Embedded_effect already
  // captured at line 309 carries the resolution effect, so the ability resolves
  // correctly after the source ID is dead.
  if ability_cost.exile_self {
      let (is_creature, owner, pre_death_controller, pre_death_counters) = {
          let obj = state.object(source)?;
          (
              obj.characteristics.card_types.contains(&crate::state::types::CardType::Creature),
              obj.owner,
              obj.controller,
              obj.counters.clone(),
          )
      };
      let (new_id, _) = state.move_object_to_zone(source, ZoneId::Exile)?;
      events.push(GameEvent::PermanentExiled {
          object_id: source,
          new_exile_id: new_id,
          controller: pre_death_controller,
      });
      // Note: no PermanentSacrificed event (sacrifice and exile are distinct
      // CR actions). Creature death is not triggered by exile-as-cost since
      // the creature does not "die" (CR 700.4: dies = battlefield → graveyard).
      // is_creature, pre_death_counters captured for parity with sacrifice path
      // and for any future "leaves the battlefield" trigger emission.
      let _ = (is_creature, pre_death_counters);
  }
  ```

  **VERIFY at impl time**: the exact name of `GameEvent::PermanentExiled` (or
  whether a generic `GameEvent::ZoneChanged` is the canonical event for exile).
  Grep `enum GameEvent` and use whatever the existing exile path uses (ETB exile
  replacements, Effect::ExileObject). If no specific event exists, emit
  `GameEvent::PermanentLeftBattlefield` or whatever the current convention is.

- `crates/engine/src/testing/replay_harness.rs` — add a match arm in
  `flatten_cost_into` at line 3071 alongside `Cost::SacrificeSelf`:

  ```rust
  Cost::ExileSelf => ac.exile_self = true,
  ```

- `crates/engine/src/state/hash.rs` — add hash arm in `impl HashInto for Cost` at
  line 4638. Next discriminant after `SacrificeSelf = 7u8`. Verify the chain at
  impl time; assume **discriminant 11** (after Tap, Mana, SacrificeSelf, Sacrifice,
  PayLife, DiscardCard, DiscardSelf, Forage, Sequence, RemoveCounter — count
  precisely at impl time):

  ```rust
  Cost::ExileSelf => N_u8.hash_into(hasher), // discriminant TBD at impl
  ```

  Also extend `impl HashInto for ActivationCost` to hash the new `exile_self: bool`
  field. **CRITICAL**: this is the same failure mode as PB-S H1 (forgot
  `once_per_turn`). The reviewer must field-count the impl against the struct
  definition.

**Exhaustive match audit**: `rg "match .*Cost\b" crates/engine/`. Match sites that
need new arms:
- `state/hash.rs:4638` (HashInto for Cost)
- `testing/replay_harness.rs:3072` (flatten_cost_into)

**No** mainline `process_command` match on `Cost` itself — costs flow through
`ActivationCost` (the bool struct), so the dispatch is by `if ability_cost.exile_self`
in `abilities.rs`. TUI/replay-viewer do not match on `Cost`.

Also extend `state/hash.rs` `impl HashInto for ActivationCost` to include
`exile_self.hash_into(hasher)`. Field-count the impl during implementation.

---

## Hash Version Bump

**Yes, bump.** Adding three discriminants (LayerModification 25, EffectFilter 32 +
33, Cost +1) plus a new `ActivationCost` field changes the hash space. Replays
recorded against the pre-PB-X hash will not validate. Bump the hash version
constant per the project's hash-version policy. (Check `state/hash.rs` for the
version constant; PB-S did not bump because it was a HIGH fix mid-cycle, but
clean primitives traditionally do.)

---

## Card Definition Patches

### crippling_fear.rs (NEW)

```rust
// Crippling Fear — {2}{B}{B} Sorcery
// Choose a creature type. Creatures that aren't of the chosen type get -3/-3 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crippling-fear"),
        name: "Crippling Fear".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a creature type. Creatures that aren't of the chosen type get -3/-3 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::ChooseCreatureType { default: SubType("Human".to_string()) },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(-3),
                            filter: EffectFilter::AllCreaturesExcludingChosenSubtype,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
```

### eyeblight_massacre.rs (NEW)

```rust
// Eyeblight Massacre — {2}{B}{B} Sorcery
// Non-Elf creatures get -2/-2 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eyeblight-massacre"),
        name: "Eyeblight Massacre".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Non-Elf creatures get -2/-2 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(-2),
                        filter: EffectFilter::AllCreaturesExcludingSubtype(SubType("Elf".to_string())),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
```

### olivias_wrath.rs (NEW)

```rust
// Olivia's Wrath — {4}{B} Sorcery
// Each non-Vampire creature gets -X/-X until end of turn, where X is the number of Vampires you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("olivias-wrath"),
        name: "Olivia's Wrath".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each non-Vampire creature gets -X/-X until end of turn, where X is the number of Vampires you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBothDynamic {
                            amount: Box::new(EffectAmount::PermanentCount {
                                filter: TargetFilter {
                                    has_card_type: Some(CardType::Creature),
                                    has_subtype: Some(SubType("Vampire".to_string())),
                                    ..Default::default()
                                },
                                controller: PlayerTarget::Controller,
                            }),
                            negate: true,
                        },
                        filter: EffectFilter::AllCreaturesExcludingSubtype(SubType("Vampire".to_string())),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
```

### balthor_the_defiled.rs (NEW)

```rust
// Balthor the Defiled — {2}{B}{B} Legendary Creature — Zombie Dwarf — 2/2
// Minion creatures get +1/+1.
// {B}{B}{B}, Exile Balthor: Each player returns all black and all red creature
// cards from their graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("balthor-the-defiled"),
        name: "Balthor the Defiled".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types_with_subtypes(
            &[CardType::Creature],
            &[SubType("Zombie".to_string()), SubType("Dwarf".to_string())],
            &[SuperType::Legendary],
        ),
        oracle_text: "Minion creatures get +1/+1.\n{B}{B}{B}, Exile Balthor: Each player returns all black and all red creature cards from their graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Static "Minion creatures get +1/+1"
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AllCreaturesWithSubtype(SubType("Minion".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {B}{B}{B}, Exile Balthor: ...
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { black: 3, ..Default::default() }),
                    Cost::ExileSelf,
                ]),
                effect: Effect::ReturnAllFromGraveyardToBattlefield {
                    // VERIFY shape at impl time — see open question 4.
                    // Per-player scope, filter creatures with color in {Black, Red}.
                    ..Default::default()
                },
                targets: vec![],
                sorcery_speed: false,
                once_per_turn: false,
                activation_zone: None,
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
```

### obelisk_of_urd.rs (NEW)

```rust
// Obelisk of Urd — {6} Artifact (Convoke)
// As this artifact enters, choose a creature type.
// Creatures you control of the chosen type get +2/+2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("obelisk-of-urd"),
        name: "Obelisk of Urd".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Convoke (Your creatures can help cast this spell. ...)\nAs this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +2/+2.".to_string(),
        abilities: vec![
            Keyword(KeywordAbility::Convoke),
            // ETB: choose a creature type and store on the source permanent
            AbilityDefinition::Triggered {
                trigger: TriggerCondition::SelfEntersBattlefield,
                effect: Effect::ChooseCreatureType { default: SubType("Human".to_string()) },
                ..Default::default()
            },
            // Static pump using chosen type from the source permanent
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::CreaturesYouControlOfChosenType,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
```

### city_on_fire.rs (NEW)

```rust
// City on Fire — {5}{R}{R}{R} Enchantment (Convoke)
// If a source you control would deal damage to a permanent or player, it deals
// triple that damage instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("city-on-fire"),
        name: "City on Fire".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Convoke (...)\nIf a source you control would deal damage to a permanent or player, it deals triple that damage instead.".to_string(),
        abilities: vec![
            Keyword(KeywordAbility::Convoke),
            // Replacement: triple damage from sources you control
            // (Pattern from cards/defs/fiery_emancipation.rs)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::DamageWouldBeDealt {
                    target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
                },
                modification: ReplacementModification::TripleDamage,
                duration: EffectDuration::WhileSourceOnBattlefield,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
```

### Cards NOT authored in PB-X

- **metallic_mimic.rs** — blocked on `LayerModification::AddChosenCreatureType`
  (fourth primitive). File a TODO `.rs` stub or leave unauthored, with the gap
  documented in `memory/card-authoring/a42-retriage-2026-04-10.md`.
- **heritage_druid.rs** — blocked on `TapNCreatures` cost framework. Not a PB-X
  primitive; defer.

---

## Unit Tests

**File**: `crates/engine/tests/primitive_pb_x.rs` (new)
**Pattern**: Follow `crates/engine/tests/primitive_pb37.rs` and the layered-effect
tests in `tests/layers.rs`.

### Filter tests (Primitive #1)

1. `test_all_creatures_excluding_subtype_static` — battlefield with Elf, Goblin,
   Vampire creatures across two players. Apply
   `AllCreaturesExcludingSubtype(SubType("Elf"))` with `ModifyBoth(-2)`. Expect:
   Goblin and Vampire (both controllers) get -2/-2; Elves unaffected. CR 613.1f.
2. `test_all_creatures_excluding_chosen_subtype_substituted` — set
   `ctx.chosen_creature_type = Some("Vampire")` via a `ChooseCreatureType` effect
   in a `Sequence`, then run the `ApplyContinuousEffect`. Expect: stored
   `ContinuousEffect.filter == AllCreaturesExcludingSubtype("Vampire")`, not the
   chosen-subtype placeholder. Walk `state.continuous_effects` to verify.
3. `test_chosen_subtype_no_choice_skips` — `ApplyContinuousEffect` with the chosen
   variant when `ctx.chosen_creature_type = None`. Expect: no continuous effect
   stored, no panic.
4. `test_eyeblight_massacre_card` — full integration: cast Eyeblight Massacre
   into a 4-creature board (1 Elf, 3 non-Elf). Expect P/T deltas on the 3 non-Elf,
   Elf untouched.
5. `test_crippling_fear_card` — full integration: cast Crippling Fear, choose
   "Goblin" via the harness `choose_creature_type` action (or default), expect
   non-Goblin creatures get -3/-3.

### Dynamic-amount tests (Primitive #2)

6. `test_modify_both_dynamic_resolved_at_apply_time` — battlefield with 3
   Vampires you control. Apply `ModifyBothDynamic { amount: PermanentCount(Vampires
   you control), negate: true }`. After execution, find the stored
   `ContinuousEffect` and assert its modification is `ModifyBoth(-3)` (not the
   dynamic variant). Verify CR 608.2h: kill a Vampire on a later turn within the
   same EOT window — the -3 stays locked in (does not become -2).
7. `test_olivias_wrath_card` — full integration: cast Olivia's Wrath with 2
   Vampires you control. Verify non-Vampire creatures get -2/-2; Vampires
   untouched. Then sacrifice a Vampire mid-turn, recheck non-Vampire P/T:
   still -2/-2 (locked).
8. `test_modify_both_dynamic_zero` — edge case: 0 Vampires you control. Resolved
   value = 0. Effect stored as `ModifyBoth(0)`. No P/T change. No panic.

### ExileSelf tests (Primitive #3)

9. `test_exile_self_cost_moves_source_to_exile` — minimal activated ability
   `Cost::ExileSelf : Effect::DrawCard`. Activate it; assert source object is in
   `ZoneId::Exile`, ability is on stack with `embedded_effect = Some(...)`.
10. `test_exile_self_ability_resolves_after_source_dead` — same setup; resolve
    the stack. Assert: card is drawn (effect ran via `embedded_effect`), source
    still in exile, no `ObjectNotFound` error from resolution.
11. `test_exile_self_with_mana_sequence` — `Cost::Sequence([Mana, ExileSelf])`.
    Player has insufficient mana → activation fails before exile (cost atomicity).
12. `test_balthor_activation_returns_creatures` — full integration: Balthor on
    battlefield, opponent + you have Black and Red creature cards in graveyard +
    Green creature card. Activate Balthor's ability. Assert: Black and Red
    creatures returned to their owners' battlefields; Green creatures stayed in
    graveyard; Balthor in exile; static "Minion +1/+1" no longer applies (Balthor
    is gone).
13. `test_balthor_static_minion_pump` — Balthor + 2 Minion creatures + 1 non-Minion
    creature. Verify Minion creatures get +1/+1, non-Minion unchanged. (Tests the
    static ability separately from the activated.)
14. `test_exile_self_hashed` — sanity test that the `exile_self: true` field
    participates in `HashInto for ActivationCost`: build two ActivationCosts that
    differ only in `exile_self`, hash both, assert hashes differ. Defends against
    the PB-S H1 failure mode.

**Test count**: 14. Trim or merge if duplicative; minimum 11 (4 filter + 4 dynamic
+ 3 cost) for full coverage.

---

## Edge Cases & Risks

1. **`AllCreaturesExcludingChosenSubtype` substitution missed**: if a card def or
   future feature stores this variant in a `ContinuousEffect` without going through
   `Effect::ApplyContinuousEffect`, the layer code's `debug_assert!(false)` arm
   fires in tests and silently no-ops in release. Mitigation: the assertion + test
   #2 + an audit of any new storage paths during impl.

2. **`ModifyBothDynamic` stored without substitution**: same risk, same mitigation.
   Both variants are DSL-only.

3. **Sign handling on `ModifyBothDynamic`**: oversight should approve the
   `negate: bool` design vs. an `EffectAmount::Negate(Box<EffectAmount>)` combinator
   on EffectAmount itself (more general but touches more files). Plan picks
   `negate: bool` for scope minimality. Open question 1.

4. **Olivia's Wrath: Vampire dies mid-turn**: locked value persists per CR 608.2h.
   Test #7 verifies. Without locking (e.g., if we naively re-resolve at layer
   time), the engine would silently do the wrong thing for X-spells across the
   board — getting this wrong is the kind of subtle bug PB-S overlapped with.

5. **Olivia's Wrath: Vampire enters mid-turn**: same — locked value. New Vampires
   that enter after resolution do NOT increase X. Verifies CR 608.2h once.

6. **Crippling Fear chosen type = empty string / non-existent type**: harness
   `choose_creature_type` is deterministic and validated; the `default` argument
   on `Effect::ChooseCreatureType` ensures a real subtype. No new edge.

7. **`ExileSelf` LKI on a state-based-action interrupt**: between cost payment
   and stack push, an SBA could fire? No — cost payment and stack push are within
   `process_command` and SBAs run between commands. Same risk profile as
   `SacrificeSelf`. Verified by reading `abilities.rs:586-917`.

8. **`ExileSelf` interaction with ETB-replacement effects**: exile is a zone
   change; the `ETBReplacement` system and `LeavesTheBattlefield` triggers must
   fire correctly. **VERIFY at impl time**: walk `move_object_to_zone` for the
   Battlefield → Exile transition and confirm any "leaves battlefield" trigger
   fires (Mikaeus undying interactions, etc.). Same as Sacrifice path — should
   already be plumbed via `move_object_to_zone`.

9. **`ExileSelf` for non-permanents**: meaningless (Cost::ExileSelf is only
   sensible for activated abilities of permanents). Validate at activation time:
   if `obj.zone != Battlefield`, the cost cannot be paid. Mirror sacrifice_self
   validation (which currently has no explicit guard — it relies on the ability
   only being available from the battlefield).

10. **Creature dying via exile**: per CR 700.4 "dies = battlefield → graveyard".
    Exile is NOT death. No `CreatureDied` event; no Mikaeus-undying trigger; no
    "When ~ dies" trigger. Reflected in the payment block: emits
    `PermanentExiled`-style event only.

11. **Hash version bump risk**: any in-flight replays will not validate against
    PB-X-built engine. Acceptable; document in commit message.

12. **Marvin reflection pattern (scope-creep canary)**: verified during
    full-chain walk that NONE of the PB-X cards reach into reflection. Safe.

13. **`ReturnAllFromGraveyardToBattlefield` per-player + color filter**: open
    question 4 below — the exact effect variant Balthor needs MUST be verified
    before the activated body can be authored. If absent, Balthor partially
    authored (cost done, body TODO) until a follow-up.

---

## Stop-and-Flag Findings

Per oversight scope rules, raising rather than absorbing:

- **F1: Fourth-primitive discovery (Metallic Mimic)**. Walking the chain showed
  Mimic needs `LayerModification::AddChosenCreatureType` (or equivalent dynamic
  AddSubtypes). This is a fourth primitive. Plan recommendation: defer to a
  follow-up micro-PB **PB-Y** or fold into PB-Q (ChooseColor) since both share
  the "ETB choice the source itself reads in a layer effect" pattern. **Not in
  PB-X.**

- **F2: Retriage corrections.** The 2026-04-10 retriage flagged Obelisk of Urd
  and City on Fire as "needs verification" — both are actually authorable
  today. Plan-phase verification confirms. PB-X session authors them as
  free wins.

- **F3: Balthor body unverified.** The static `+1/+1 Minion creatures` ability
  composes with existing primitives, but the activated ability's body
  (`ReturnAllFromGraveyardToBattlefield` per-player + color filter) was not
  verified during this plan phase. Open question 4. If the existing PB-H effect
  variant doesn't support per-player scope + color filter, the activated body
  goes TODO and Balthor is partially authored. **Cost::ExileSelf still ships
  to unblock the cost-framework gap.**

- **F4: No CR contradiction with oversight scope.** CR 608.2h confirms numeric
  values resolve once; the substitution-at-execution design is correct. CR
  701.10 establishes Exile as an action; the cost-payment block aligns with
  CR 602.2 (cost paid before ability is on stack).

- **F5: Marvin reflection pattern not encountered.** None of the in-scope cards
  trip the PB-S scope-creep trigger.

---

## Open Questions for Oversight

1. **`ModifyBothDynamic` sign handling**: `negate: bool` on the variant (proposed)
   vs. `EffectAmount::Negate(Box<EffectAmount>)` combinator (more general,
   broader surface). Plan default: `negate: bool`. Approve or override.

2. **Hash version bump**: yes (proposed). Confirm whether the project tracks a
   hash version constant; if so, where it lives and what value to bump to. PB-S
   did not bump.

3. **Metallic Mimic disposition**: defer to PB-Y micro-PB (proposed) vs. fold
   into PB-Q ChooseColor. Plan default: spawn PB-Y as a one-primitive micro-PB
   after PB-X lands, since PB-Q ChooseColor is a different mechanic (color
   choice ETB) and bundling muddies it.

4. **Balthor's `ReturnAllFromGraveyardToBattlefield` verification**: implementer
   must grep `Effect::ReturnAllFromGraveyardToBattlefield` and confirm whether
   per-player scope + color filter is supported. If not, two options:
   - (a) Author Balthor with the cost wired and the activated body TODO'd. Cost
     primitive ships, Balthor partially authored.
   - (b) Add a small filter extension to the existing effect as part of PB-X.
     This risks scope creep — flag and ask oversight.

5. **`Cost::ExileSelf` discriminant in HashInto for Cost**: confirm exact next
   discriminant by reading `state/hash.rs:4638-4700` at impl time. Plan assumes
   "11" but didn't count precisely.

6. **Test for face-down Cost::ExileSelf**: face-down creatures with morph + exile
   activation? None of our cards exercise this. Probably no test needed; flag if
   trivial.

---

## Verification Checklist

- [ ] Two new `EffectFilter` variants added + 1 layer arm + 1 substitution arm + 2 hash arms
- [ ] One new `LayerModification` variant added + substitution arm in `effects/mod.rs` + debug_assert arm in `layers.rs` + 1 hash arm
- [ ] `Cost::ExileSelf` variant added + `ActivationCost.exile_self` field added + payment block in `abilities.rs` + replay_harness flatten arm + 2 hash arms (Cost + ActivationCost field)
- [ ] All 6 card defs author cleanly (Crippling Fear, Eyeblight Massacre, Olivia's Wrath, Balthor the Defiled, Obelisk of Urd, City on Fire); Metallic Mimic + Heritage Druid documented as deferred
- [ ] 11+ unit tests in `tests/primitive_pb_x.rs` pass
- [ ] `cargo test --all` green
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo build --workspace` builds replay-viewer + TUI (no exhaustive-match cascade)
- [ ] Hash version bump applied (or open question 2 resolved)
- [ ] No new TODOs introduced; deferred Metallic Mimic gap recorded in
      `memory/card-authoring/a42-retriage-2026-04-10.md`
- [ ] `memory/primitive-wip.md` advanced from `plan` → `implement`
- [ ] Carry-forward LOWs from PB-S NOT touched (out of scope)

---

## Discriminant Chain

| Enum | New variants | Discriminants | Next free after PB-X |
|------|--------------|---------------|----------------------|
| `EffectFilter` | `AllCreaturesExcludingSubtype(SubType)`, `AllCreaturesExcludingChosenSubtype` | 32, 33 | 34 |
| `LayerModification` | `ModifyBothDynamic { amount, negate }` | 25 | 27 (26 = `RemoveSuperType`) |
| `Cost` | `ExileSelf` | TBD (count at impl) | TBD+1 |
| `ActivationCost` | `exile_self: bool` (struct field, not enum) | n/a | n/a |

---

## Files Touched (impl-phase preview)

| File | Reason | Approx LOC |
|------|--------|------------|
| `crates/engine/src/state/continuous_effect.rs` | Add 2 EffectFilter variants + 1 LayerModification variant | +35 |
| `crates/engine/src/state/game_object.rs` | Add `exile_self: bool` to ActivationCost | +5 |
| `crates/engine/src/cards/card_definition.rs` | Add `Cost::ExileSelf` | +5 |
| `crates/engine/src/rules/layers.rs` | Add filter match arm + debug_assert arms | +20 |
| `crates/engine/src/rules/abilities.rs` | Add exile-self payment block | +30 |
| `crates/engine/src/effects/mod.rs` | Substitute resolved_filter + resolved_modification | +30 |
| `crates/engine/src/state/hash.rs` | 5 new hash arms + ActivationCost field | +25 |
| `crates/engine/src/testing/replay_harness.rs` | flatten_cost_into arm | +2 |
| `crates/engine/src/cards/defs/{6 new files}.rs` | Card defs | ~250 |
| `crates/engine/tests/primitive_pb_x.rs` | New test file | ~400 |
| **TOTAL** | | **~800 LOC** |

Engine code (excluding card defs + tests): **~152 LOC**, matching the
"~140 LOC engine" estimate in the executive summary.
