# Card Review: Batch A

**Date**: 2026-03-10
**Reviewer**: ability-impl-reviewer (Opus)
**Cards reviewed**: Exotic Orchard, Blasphemous Act, Chaos Warp, City of Brass, Assassin's Trophy

---

## Card 1: Exotic Orchard

**File**: `crates/engine/src/cards/defs/exotic_orchard.rs`
**Oracle**: "{T}: Add one mana of any color that a land an opponent controls could produce."

- **Oracle match**: PARTIAL -- The oracle text string is correct. The behavior is simplified: `AddManaAnyColor` always offers all 5 colors rather than restricting to colors opponents' lands could produce.
- **DSL correctness**: YES -- `AbilityDefinition::Activated` with `Cost::Tap` and `Effect::AddManaAnyColor` is the correct DSL pattern for a mana ability. No mana cost, Land type, no subtypes -- all correct.
- **Findings**:
  - **F1** (LOW): Documented DSL gap -- `AddManaAnyColor` does not filter by opponents' lands. The TODO comment accurately describes the limitation. This is a genuine DSL gap requiring a new Effect variant or runtime query. Acceptable simplification for now.

**Verdict**: Acceptable with documented gap.

---

## Card 2: Blasphemous Act

**File**: `crates/engine/src/cards/defs/blasphemous_act.rs`
**Oracle**: "This spell costs {1} less to cast for each creature on the battlefield. Blasphemous Act deals 13 damage to each creature."

- **Oracle match**: YES -- Oracle text string matches. Mana cost {8}{R} is correct. Sorcery type correct.
- **DSL correctness**: YES -- `Effect::DealDamage` with `EffectTarget::AllCreatures` and `EffectAmount::Fixed(13)` correctly models "deals 13 damage to each creature". Empty `targets` vec is correct (no targeting -- it says "each creature", not "target creature").
- **Findings**:
  - **F2** (LOW): Documented DSL gap -- cost reduction "costs {1} less for each creature on the battlefield" is not implemented. The TODO accurately describes this. Would require a dynamic cost modifier (e.g., `CostReduction::PerCreatureOnBattlefield`). The card works at base cost {8}{R}; the discount is not applied.

**Verdict**: Acceptable with documented gap.

---

## Card 3: Chaos Warp

**File**: `crates/engine/src/cards/defs/chaos_warp.rs`
**Oracle**: "The owner of target permanent shuffles it into their library, then reveals the top card of their library. If it's a permanent card, they put it onto the battlefield."

- **Oracle match**: PARTIAL -- Oracle text string is correct. The reveal-and-put-onto-battlefield portion is correctly documented as a DSL gap.
- **DSL correctness**: NO -- The `MoveZone` destination uses `PlayerTarget::Controller`, which resolves to the Chaos Warp *caster's* library, not the target permanent's *owner's* library. The card explicitly says "the owner of target permanent shuffles it into **their** library."
- **Findings**:
  - **F3** (MEDIUM): **Wrong library destination.** `ZoneTarget::Library { owner: PlayerTarget::Controller, ... }` resolves `Controller` to `ctx.controller` (the Chaos Warp caster). The card says "the **owner** of target permanent shuffles it into **their** library." In a multiplayer game where Player A casts Chaos Warp targeting Player B's permanent (which Player B owns), the permanent would incorrectly go to Player A's library instead of Player B's. The DSL has `PlayerTarget::ControllerOf(Box<EffectTarget>)` which could be used as a workaround -- but the correct fix is either using `ControllerOf` (which resolves to the controller of the target, usually the owner in non-steal scenarios) or adding an `OwnerOf` variant. **Fix**: At minimum, change to `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` which gets the controller of the targeted permanent (correct in most cases; only wrong when the permanent is controlled but not owned by a player -- rare in Commander). Ideally, add `PlayerTarget::OwnerOf(Box<EffectTarget>)` to be fully correct.
  - **F4** (LOW): Documented DSL gap -- reveal top card and conditionally put permanent cards onto the battlefield. TODO is accurate. Would need a new Effect variant combining reveal + type check + conditional zone move.

**Verdict**: Needs fix (F3 is MEDIUM).

---

## Card 4: City of Brass

**File**: `crates/engine/src/cards/defs/city_of_brass.rs`
**Oracle**: "Whenever City of Brass becomes tapped, it deals 1 damage to you. {T}: Add one mana of any color."

- **Oracle match**: PARTIAL -- Oracle text string is correct. The card has TWO abilities: (1) a triggered ability "Whenever City of Brass becomes tapped, it deals 1 damage to you" and (2) a mana ability "{T}: Add one mana of any color." The implementation models these as a single activated ability that deals damage then adds mana.
- **DSL correctness**: NO -- The triggered ability fires whenever City of Brass becomes tapped by *any* means (opponent's Icy Manipulator, etc.), not just when you activate the mana ability. Modeling damage inside the activated ability means: (a) damage is part of the mana ability resolution rather than a triggered ability using the stack, and (b) tapping City of Brass by other means does not deal damage.
- **Findings**:
  - **F5** (MEDIUM): **Damage modeled as part of mana ability instead of triggered ability.** The oracle text says "Whenever City of Brass becomes tapped" -- this is a triggered ability that fires on ANY tap event, not just the mana ability activation. The current implementation puts `DealDamage` inside the `Activated` effect sequence, which means: (1) damage resolves as part of the mana ability (doesn't use the stack, can't be responded to -- this is actually correct for mana abilities), but (2) if an opponent taps City of Brass via another effect, no damage is dealt. The TODO in the file accurately describes this gap. **Fix**: This requires a `TriggerCondition::WhenBecomesTapped` (or similar) which does not exist in the DSL. Until that trigger condition is added, the current approximation is the best available. Downgrading to LOW since the TODO is documented and the DSL gap is genuine.

    Revised severity: **LOW** (documented DSL gap, no fix possible without new trigger condition).

**Verdict**: Acceptable with documented gap.

---

## Card 5: Assassin's Trophy

**File**: `crates/engine/src/cards/defs/assassins_trophy.rs`
**Oracle**: "Destroy target nonland permanent. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle."

- **Oracle match**: PARTIAL -- Oracle text string is correct. The search-for-basic-land portion is documented as a DSL gap.
- **DSL correctness**: NO -- The targeting uses `TargetRequirement::TargetPermanent` but the card says "target **nonland** permanent." The `TargetFilter` struct has a `non_land: bool` field and `TargetRequirement::TargetPermanentWithFilter(TargetFilter)` exists, so this IS expressible in the current DSL.
- **Findings**:
  - **F6** (HIGH): **Wrong target restriction -- allows targeting lands.** The card says "target nonland permanent" but the implementation uses `TargetRequirement::TargetPermanent`, which allows targeting any permanent including lands. The DSL supports this restriction via `TargetRequirement::TargetPermanentWithFilter(TargetFilter { non_land: true, ..Default::default() })`. The TODO comment claims `TargetNonlandPermanent` doesn't exist, but `TargetPermanentWithFilter` with `non_land: true` does exist and is the correct way to express this. **Fix**: Replace `targets: vec![TargetRequirement::TargetPermanent]` with `targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter { non_land: true, ..Default::default() })]` and remove the stale TODO comment.
  - **F7** (LOW): Documented DSL gap -- "its controller may search their library for a basic land card" is not implemented. Would need a search effect targeting the destroyed permanent's controller. The `SearchLibrary` action exists but wiring it to the target's controller (not the caster) would need `PlayerTarget::ControllerOf`.

**Verdict**: Needs fix (F6 is HIGH).

---

## Summary

| # | Card | Severity | Description |
|---|------|----------|-------------|
| F1 | Exotic Orchard | LOW | AddManaAnyColor doesn't filter by opponents' lands (documented DSL gap) |
| F2 | Blasphemous Act | LOW | Cost reduction not implemented (documented DSL gap) |
| F3 | Chaos Warp | MEDIUM | Wrong library destination -- uses caster's library instead of target's owner's library |
| F4 | Chaos Warp | LOW | Reveal-and-put-onto-battlefield not implemented (documented DSL gap) |
| F5 | City of Brass | LOW | Damage modeled as activated ability effect, not triggered ability (documented DSL gap) |
| F6 | Assassin's Trophy | HIGH | TargetPermanent allows targeting lands; should use TargetPermanentWithFilter { non_land: true } |
| F7 | Assassin's Trophy | LOW | Search-for-basic-land not implemented (documented DSL gap) |

**Overall verdict**: **needs-fix** -- 1 HIGH (F6: fixable now with existing DSL) and 1 MEDIUM (F3: fixable with `ControllerOf` workaround).
