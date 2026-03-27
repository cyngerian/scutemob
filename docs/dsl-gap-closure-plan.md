# DSL Gap Closure Plan

> **Purpose**: Close all remaining DSL gaps blocking card implementations. Extends the
> primitive batch pipeline (PB-23+) with new engine work, then backfills existing TODO
> card defs after each gap closes.
>
> **Problem**: As of 2026-03-23, 814 of 1,452 card defs (56%) have TODOs. The original
> primitive plan (PB-0 through PB-22) underestimated the gaps — authoring is producing
> stubs faster than the DSL can support them. Continuing to author without closing these
> gaps will produce ~1,700+ card files where 60%+ are non-functional.
>
> **Approach**: Pause new card authoring (A-29+). Close gaps as primitive batches.
> After each batch, backfill ALL existing cards unblocked by it. Resume authoring
> only when the backfill is complete.
>
> **Workflow integration**: Each gap batch uses `/implement-primitive` (plan → implement →
> review → fix → close). Backfill sweeps use `/author-wave` for the card def fixes.
> The operations plan (`docs/card-authoring-operations.md`) is updated to insert gap
> closure before remaining authoring groups.
>
> **Created**: 2026-03-23
> **Status**: DRAFT

---

## Current Card Health

| Category | Count | % of 1,452 |
|----------|------:|----------:|
| Fully implemented (no TODO, has abilities) | 583 | 40% |
| Vanilla (intentionally empty abilities) | 55 | 4% |
| Partial (some abilities, some TODOs) | 670 | 46% |
| Stripped (empty abilities + TODO comments) | 144 | 10% |

Plus ~291 cards not yet authored (from ops plan: 1,743 − 1,452).

---

## Gap Inventory

Every TODO in every card def, categorized. Cards affected = distinct card files containing
that gap pattern (a card may appear in multiple categories).

### Tier 1: Structural Gaps (new DSL constructs needed)

| ID | Gap | Cards | Engine Change |
|----|-----|------:|---------------|
| G-1 | Controller-filtered creature triggers | ~145 | Add `controller` field to `WheneverCreatureDies`, `WheneverCreatureEntersBattlefield`; new variants `WheneverCreatureYouControlAttacks`, `WheneverCreatureYouControlDealsCombatDamage` |
| G-2 | Conditional statics ("as long as X") | ~201 | Add `condition: Option<Condition>` field to `AbilityDefinition::Static` |
| G-3 | Continuous effect grants ("creatures you control get/have") | ~98 | Wire `ApplyContinuousEffect` for card defs — add `EffectFilter::CreaturesYouControl`, `CreaturesOpponentsControl`, filtered grant patterns |
| G-4 | Spell-type filter on triggers | ~19 | Add `spell_type_filter: Option<Vec<CardType>>` to `WheneverYouCastSpell`; also covers `WheneverOpponentCastsSpell` |
| G-5 | X-cost spells | ~42 | Add `x_cost: bool` to `ManaCost`; wire `EffectAmount::XValue` into mana cost parsing and legal action generation |
| G-6 | CDA / count-based P/T | ~32 | CharacteristicDefiningAbility mechanism: `PowerToughness::CDA(EffectAmount)` evaluated in Layer 7a |
| G-7 | Cost reduction statics | ~30 | `ContinuousEffectDef` with `EffectLayer::CostModification` — "spells cost {N} less" as a static continuous effect |

### Tier 2: New Trigger Variants

| ID | Gap | Cards | Engine Change |
|----|-----|------:|---------------|
| G-8 | Combat damage triggers (per-creature) | ~49 | `WheneverCreatureYouControlDealsCombatDamageToPlayer` TriggerCondition; event wiring in combat.rs |
| G-9 | Discard triggers | ~9 | `WheneverYouDiscard`, `WheneverOpponentDiscards` TriggerConditions; event wiring in effects |
| G-10 | Sacrifice triggers | ~6 | `WheneverYouSacrifice { filter: Option<TargetFilter> }` TriggerCondition |
| G-11 | WheneverYouAttack | ~8 | New TriggerCondition; fire at declare-attackers step |
| G-12 | Leaves-battlefield triggers | ~6 | `WhenLeavesBattlefield` TriggerCondition; event wiring in zone changes |
| G-13 | Draw-card trigger filtering | ~16 | Add `player_filter` to `WheneverPlayerDrawsCard` or add `WheneverOpponentDrawsCard` |
| G-14 | Lifegain trigger filtering | ~3 | `WheneverYouGainLife` already exists — verify it covers all patterns; may need amount field |
| G-15 | Cast triggers ("when you cast this spell") | ~5 | `WhenYouCastThisSpell` TriggerCondition; fires from stack before resolution |

### Tier 3: New Cost/Effect Primitives

| ID | Gap | Cards | Engine Change |
|----|-----|------:|---------------|
| G-16 | Cost::RemoveCounter | ~16 | New `Cost::RemoveCounter { counter: CounterType, count: u32 }` variant |
| G-17 | AdditionalCost::SacrificeCreature for spells | ~7 | Extend `AdditionalCost` vec on `CastSpell` — creature sacrifice at cast time |
| G-18 | Additional land plays | ~10 | Static: "you may play an additional land on each of your turns" — increment `lands_per_turn` |
| G-19 | Prevention effects (combat damage prevention) | ~11 | Wire `ApplyContinuousEffect` with damage prevention shield — "prevent all combat damage that would be dealt to/by target" |
| G-20 | Control change effects | ~6 | `Effect::GainControl { target, duration }`, `Effect::ExchangeControl { target_a, target_b }` |
| G-21 | Land animation | ~12 | `Effect::AnimateLand { target, power, toughness, types, duration }` — "becomes a N/N creature" |
| G-22 | Copy/clone (DSL wiring) | ~12 | Wire `Effect::CopyPermanent` for card defs — clone ETB replacement |

### Tier 4: Complex Mana Production

| ID | Gap | Cards | Engine Change |
|----|-----|------:|---------------|
| G-23 | Filter lands | ~20 | Activated ability: pay hybrid mana, produce two mana from a choice set. Pattern: `AddManaChoice` with constrained color pairs |
| G-24 | Devotion-based mana | ~5 | `AddManaScaled` with `EffectAmount::DevotionTo` — exists but verify wiring |
| G-25 | Conditional mana abilities | ~15 | Activated abilities with conditions (e.g., "sacrifice: add {B}{B}") — many are just `Cost::Sequence` + `AddMana` wiring |

### Tier 5: Remaining Long-Tail

| ID | Gap | Cards | Engine Change |
|----|-----|------:|---------------|
| G-26 | Activated abilities (general complex) | ~66 | Not a single gap — these are cards with unique activated abilities that need individual wiring. Many will be unblocked by G-1 through G-25. Re-assess after other gaps close. |
| G-27 | Modal triggered abilities | ~26 | Modal choice on trigger resolution — partially supported (B11), extend to triggered abilities |
| G-28 | Exile/flicker timing | ~27 | Delayed return triggers — "exile until end of turn" / "exile, return at beginning of next end step". Extend `Effect::ExileObject` with `return_timing` |
| G-29 | Graveyard recursion conditions | ~23 | Many are just `MoveZone` + conditions — verify wiring. Some need `ActivationCondition` extensions |
| G-30 | Planeswalker (remaining) | ~11 | Individual planeswalker loyalty abilities — framework exists (PB-14), cards need authoring |
| G-31 | Evasion/protection statics | ~21 | "can't be blocked except by N+ creatures" (Menace variant), player protection — extend `CantBeBlocked` filter |

---

## Execution Plan

### Primitive Batches (PB-23 through PB-37)

Gaps are consolidated into implementable batches. Each batch uses the existing
`/implement-primitive` pipeline: plan → implement → review → fix → close.

**After each batch completes, sweep all existing card defs it unblocks** — this is the
backfill step. Use `card-fix-applicator` agent or manual edits. Target: every card
unblocked by the batch has its TODO removed and ability implemented in the same commit.

| Batch | Gaps | Summary | Backfill Cards |
|-------|------|---------|---------------:|
| **PB-23** | G-1 | Controller-filtered creature triggers | ~145 |
| **PB-24** | G-2 | Conditional statics ("as long as") | ~201 |
| **PB-25** | G-3 | Continuous effect grants | ~98 |
| **PB-26** | G-4, G-13, G-9, G-10, G-11, G-12, G-14, G-15 | Trigger variants (all remaining) | ~72 |
| **PB-27** | G-5 | X-cost spells | ~42 |
| **PB-28** | G-6 | CDA / count-based P/T | ~32 |
| **PB-29** | G-7 | Cost reduction statics | ~30 |
| **PB-30** | G-8 | Combat damage triggers | ~49 |
| **PB-31** | G-16, G-17 | Cost primitives (RemoveCounter, AdditionalSacrificeCost) | ~23 |
| **PB-32** | G-18, G-19, G-20, G-21 | Static/effect primitives (lands, prevention, control, animation) | ~39 |
| **PB-33** | G-22, G-28 | Copy/clone + exile/flicker timing | ~39 |
| **PB-34** | G-23, G-24, G-25 | Mana production (filter lands, devotion, conditional) | ~40 |
| **PB-35** | G-27, G-29, G-30 | Modal triggers, graveyard conditions, planeswalker abilities | ~60 |
| **PB-36** | G-31 | Evasion/protection extensions | ~21 |
| **PB-37** | G-26 (residual) | Complex activated abilities — re-assess after PB-23 through PB-36 | TBD |

### Recommended Execution Order

Ordered by: cards unblocked (highest first), dependency safety, implementation simplicity.

1. **PB-23** (G-1: controller-filtered triggers) — highest leverage, unblocks ~145 cards
2. **PB-26** (G-4/G-9-15: trigger variants) — consolidate all remaining trigger gaps in one batch
3. **PB-30** (G-8: combat damage triggers) — extends trigger work from PB-23/PB-26
4. **PB-24** (G-2: conditional statics) — unblocks ~201 cards, large but self-contained
5. **PB-25** (G-3: continuous effect grants) — unblocks ~98 cards
6. **PB-27** (G-5: X-cost spells) — unblocks ~42 cards
7. **PB-28** (G-6: CDA) — unblocks ~32 cards, layer system change
8. **PB-29** (G-7: cost reduction) — unblocks ~30 cards, continuous effect on costs
9. **PB-31** (G-16/17: cost primitives) — small, targeted
10. **PB-34** (G-23/24/25: mana production) — mostly wiring
11. **PB-32** (G-18-21: static/effect primitives) — mixed bag
12. **PB-33** (G-22/28: copy + exile timing) — complex interactions
13. **PB-35** (G-27/29/30: modal + graveyard + PW) — cleanup
14. **PB-36** (G-31: evasion extensions) — small
15. **PB-37** (G-26: residual) — re-assess what's left

### Interleaving with Card Authoring

The operations plan currently has A-24 through A-42 unchecked. The new order:

1. **Pause new authoring** at A-28 (current stopping point)
2. **Run PB-23 through PB-37** with backfill after each batch
3. **After gap closure**: re-triage remaining A-29+ groups — many cards will already
   be implemented via backfill
4. **Resume authoring** for genuinely new cards only (not previously blocked stubs)
5. **Phase 3 audit** as planned

---

## Operations Plan Integration

The following changes are needed to `docs/card-authoring-operations.md`:

### Insert after A-28 (current stopping point):

New section: **Phase 2.5: DSL Gap Closure**

```
### Phase 2.5: DSL Gap Closure

Requires: A-28 complete.

Close all remaining DSL gaps before authoring more cards. Each PB-N batch uses
`/implement-primitive`. After each batch, backfill all existing card defs it unblocks.
Full plan: `docs/dsl-gap-closure-plan.md`.

- [ ] **PB-23**: Controller-filtered creature triggers (~145 cards unblocked)
- [ ] **PB-24**: Conditional statics ("as long as X") (~201 cards)
- [ ] **PB-25**: Continuous effect grants (~98 cards)
- [ ] **PB-26**: Trigger variants (spell-type, discard, sacrifice, attack, LTB, draw, cast) (~72 cards)
- [ ] **PB-27**: X-cost spells (~42 cards)
- [ ] **PB-28**: CDA / count-based P/T (~32 cards)
- [ ] **PB-29**: Cost reduction statics (~30 cards)
- [x] **PB-30**: Combat damage triggers (~49 cards)
- [x] **PB-31**: Cost primitives (RemoveCounter, AdditionalSacrificeCost) (~23 cards)
- [x] **PB-32**: Static/effect primitives (additional lands, prevention, control change, land animation) (~39 cards)
- [x] **PB-33**: Copy/clone + exile/flicker timing (~39 cards)
- [ ] **PB-34**: Mana production (filter lands, devotion, conditional) (~40 cards)
- [ ] **PB-35**: Modal triggers + graveyard conditions + planeswalker abilities (~60 cards)
- [ ] **PB-36**: Evasion/protection extensions (~21 cards)
- [ ] **PB-37**: Complex activated abilities — residual after PB-23 to PB-36 (TBD)
- [ ] **BF-1**: Post-gap-closure re-triage — scan all TODO cards, re-classify against new DSL
- [ ] **BF-2**: Commit: `W6-prim: gap closure complete — PB-23 through PB-37`
```

### A-29 through A-42 remain but will shrink:

After gap closure + backfill, many cards currently planned for A-29+ will already be
implemented. The authoring groups should be re-triaged (BF-1) to remove cards already
fixed by backfill. Only genuinely new cards (not yet authored at all) remain in A-29+.

---

## Backfill Protocol

After each PB-N batch closes:

1. **Grep for TODOs** mentioning the closed gap pattern
2. **List all affected card files** (typically 20-100+)
3. **For each card**: look up oracle text via MCP, implement the previously-blocked ability, remove the TODO
4. **Build**: `cargo build --workspace`
5. **Review**: `card-batch-reviewer` agent on batches of 5-6
6. **Fix**: any HIGH/MEDIUM findings
7. **Commit**: `W6-prim: PB-<N> backfill — <N> cards fixed`

Backfill is part of the PB-N close step — the primitive batch is not "done" until all
cards it unblocks have their TODOs removed.

---

## Success Criteria

After PB-37 + backfill + re-triage:

- TODO card count drops from **814 → target <100** (remaining = genuinely complex cards
  needing individual attention in Phase 3 X-2)
- Empty-abilities (stripped) cards drop from **144 → target <20**
- Card authoring A-29+ groups are significantly smaller (many cards already done)
- Phase 3 audit (X-1) should find minimal gaps

---

## Dependency on Existing Infrastructure

| Component | Status | Notes |
|-----------|--------|-------|
| `/implement-primitive` skill | EXISTS | Plan → implement → review → fix → close pipeline |
| `primitive-impl-planner` agent | EXISTS | Opus; CR research + engine study |
| `primitive-impl-runner` agent | EXISTS | Sonnet; engine changes + card fixes + tests |
| `primitive-impl-reviewer` agent | EXISTS | Opus; verify against CR/oracle |
| `card-fix-applicator` agent | EXISTS | Sonnet; apply review fixes to card defs |
| `card-batch-reviewer` agent | EXISTS | Opus; review card defs against oracle text |
| `primitive-card-plan.md` | NEEDS UPDATE | Add PB-23 through PB-37 batch details |
| `card-authoring-operations.md` | NEEDS UPDATE | Insert Phase 2.5 gap closure section |
| `workstream-state.md` | NEEDS UPDATE | Reflect gap closure as current W6 task |
