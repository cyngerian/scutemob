# PB-CC Re-triage — `EffectAmount::CounterCount*` family

**Verdict**: **UMBRELLA-OF-MICRO-PRIMITIVES** (4 micro-primitives, recommended sequencing below)

**Date**: 2026-04-29
**Worker**: scutemob-9
**Source brief**: rank-6 PB-CC, ~10+ live TODOs across counter-count `EffectAmount` variants
**Discovery scope**: read-only audit; no engine or card-def changes

---

## Executive summary

`EffectAmount::CounterCount { target: EffectTarget, counter: CounterType }` **already exists**
and is fully wired (`effects/mod.rs:6039`, `rules/layers.rs:1492`, `state/hash.rs:4393`),
with three live callsites: Toothy / The One Ring / Everflowing Chalice. `Effect::AddCounterAmount`
also exists and accepts `count: EffectAmount` (`card_definition.rs:1331`,
`effects/mod.rs:1732`). `LayerModification::ModifyBothDynamic { amount: EffectAmount, negate }`
also exists (`continuous_effect.rs:391`, used by Olivia's Wrath).

The brief's framing of a uniform `CounterCount*` family **does not survive verification**.
The "CounterCountOnSelf" gap-variant the brief flagged is in two cases (Mossborn Hydra +
Eomer ETB-counter-half) **already expressible**: those TODOs are stale wire-up. In one case
(Exuberant Fuseling) the gap is **not** in `EffectAmount` at all — it's a missing
`LayerModification::ModifyPowerDynamic` / `ModifyToughnessDynamic` (single-axis dynamic +X/+0).
The "CounterCountOf(player, CounterType)" variant is real and distinct: it requires reading
counters on a `PlayerState` (where `poison_counters: u32` lives as a flat field, not a
counters map). The "permanents-with-counters filter" is a `TargetFilter` field gap, not an
`EffectAmount` gap. All four sub-gaps are mechanically independent — different engine
surface, different dispatch sites, different cards unblocked.

**Calibrated yield (post 2-3x discount per `feedback_pb_yield_calibration.md`,
EffectAmount-PB rate 50–65%)**: across all four micro-PBs, expect **5–7 cards shipped**
out of ~13 candidates surfaced. Single-PB framing would over-promise and under-deliver as
PB-Q (33%) and PB-P (23%) did.

---

## (a) Existing `EffectAmount::CounterCount` — engine surface and callsites

### Definition

`crates/engine/src/cards/card_definition.rs:2200-2203`:

```rust
CounterCount {
    target: EffectTarget,
    counter: CounterType,
},
```

`EffectTarget::Source` (the "self" case for triggered/activated abilities) and
`EffectTarget::DeclaredTarget { index }` are both supported by the resolution paths.

### Dispatch sites

1. **`crates/engine/src/effects/mod.rs:6039-6056`** — `resolve_amount` (effect resolution
   path, used by triggered/activated/spell abilities). Reads `state.objects.get(&id).counters`.
2. **`crates/engine/src/rules/layers.rs:1492-1504`** — `resolve_cda_amount` (CDA path during
   layer calculation). Restricts `target` to `Source`; `debug_assert!(false, …)` on other
   variants because non-Source CDA targets recurse during layer evaluation.
3. **`crates/engine/src/state/hash.rs:4393`** — incremental hashing for game-state
   determinism.

### Live in-engine callsites (production card defs)

- `crates/engine/src/cards/defs/the_one_ring.rs:52,74` — `target: Source`,
  `counter: Custom("burden")`. Used by upkeep `LoseLife` and tap-activated `DrawCards`.
- `crates/engine/src/cards/defs/toothy_imaginary_friend.rs:48` — `target: Source`,
  `counter: PlusOnePlusOne`. Used by leaves-battlefield `DrawCards` (LKI-preserved counters).
- `crates/engine/src/cards/defs/everflowing_chalice.rs` (comment line 6 only) — claims the
  tap ability uses `CounterCount`; in fact the tap ability shipped uses
  `Effect::AddMana { mana_pool(0,0,0,0,0,1) }` (fixed 1) due to a separate gap (mana
  count-as-EffectAmount). The comment is **stale**. (Out of PB-CC scope; flagged.)

### What works today (verified)

- "Counter on self" via `target: EffectTarget::Source` for any battlefield permanent **and**
  for an LKI source still in graveyard/exile (move_object_to_zone preserves counters; see
  Toothy's death trigger).
- "Counters on a targeted permanent" via `target: EffectTarget::DeclaredTarget { index }`
  (untested in production; supported by `resolve_effect_target_list`).
- Use as `count` of `Effect::AddCounterAmount` (closed loop: read N counters, add N more →
  doubling) — see `effects/mod.rs:1738`. **Mossborn Hydra Landfall is wire-up, not gap.**

### What does **not** work today (true gaps)

| Variant                                               | Why                                                                                                                                              |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Counters on a **player** (poison/energy/experience)   | `state.objects` does not contain players; player counters live as `PlayerState::poison_counters: u32` flat field (`state/player.rs:291`)         |
| Per-target dynamic `ModifyPower` / `ModifyToughness`  | `LayerModification::ModifyPower(i32)` only accepts `i32`. `ModifyBothDynamic { amount: EffectAmount }` exists but only for symmetrical +X/+X     |
| `TargetFilter` predicate "has +1/+1 counter"          | `TargetFilter` (`card_definition.rs:2336-2429`) has no `has_counter` / `has_counter_type` field                                                  |
| Counters on a target's controller                     | Composition gap: there is no `EffectTarget`/`PlayerTarget` indirection that says "the player who controls the permanent declared at index N" returning counter count from that player's `PlayerState` |

---

## (b) Gap-variant analysis with concrete oracle examples

### Sub-gap PB-CC-W: stale-TODO wire-up (zero engine surface)

The DSL is sufficient; the card defs have stale TODOs.

- **Mossborn Hydra** Landfall: "double the number of +1/+1 counters on this creature."
  `Effect::AddCounterAmount { target: Source, counter: PlusOnePlusOne,
   count: EffectAmount::CounterCount { target: Source, counter: PlusOnePlusOne } }`
  inside a `WheneverPermanentEntersBattlefield { filter: Land + You }` trigger
  (`card_definition.rs:2506`). Current count of N → adds N more → doubles.
  Ruling 2024-11-08 confirms semantics ("put a number of +1/+1 counters on it equal to the
  number it already has").

### Sub-gap PB-CC-A: `EffectAmount::PlayerCounterCount` (NEW VARIANT)

Counts counters on a `PlayerTarget`. Today players only carry `poison_counters: u32`, but
the variant should be future-proof for energy/experience.

**Engine surface (estimated)**:
- New `EffectAmount::PlayerCounterCount { player: PlayerTarget, counter: CounterType }`
  variant.
- `resolve_amount` arm reading `state.players[id].poison_counters` for `CounterType::Poison`
  and falling back to 0 for unsupported counter kinds. (Consider extending `PlayerState` with
  a generic `OrdMap<CounterType, u32>` later; not required for first ship.)
- `resolve_cda_amount` arm (CDA path: Vishgraz reads opponents' poison as part of its own
  static P/T modification — Layer 7c). Avoid recursion: player counters are not
  characteristics, no layer dependency.
- `state/hash.rs` arm.

**Cards unblocked (in priority order)**:
- **Vishgraz, the Doomhive**: "Vishgraz gets +1/+1 for each poison counter your opponents
  have." Single-blocker: `EffectFilter::Source` + Layer 7c
  `LayerModification::ModifyBothDynamic { amount: PlayerCounterCount { EachOpponent,
  Poison }, negate: false }`. ⚠ See subtlety below.
- **Phyrexian Swarmlord** upkeep: "create a 1/1 token for each poison counter your opponents
  have." Cross-blocker on TokenSpec.count=EffectAmount (separate primitive PB-TS, see
  Stop-and-Flag).
- **Phyresis Outbreak** -1/-1 half: "each creature your opponents control gets -1/-1 for
  each poison counter **its controller has**." Compound blocker: per-target dynamic
  amount (each opponent-controlled creature reads ITS OWN controller's poison count, not a
  shared aggregate). Single `ContinuousEffect` cannot express per-target EffectAmount; would
  need either `ControllerOf(...)` indirection in `PlayerCounterCount` *and* per-target
  evaluation in the layer system, or fanning out as N separate continuous effects (one per
  opponent). Engine architecture decision required.

⚠ **Subtlety on Vishgraz**: `EffectAmount::PlayerCounterCount { EachOpponent, Poison }` reads
"each opponent" — for a 4-player game with 3 opponents, this could be interpreted as
*sum over opponents* or *each opponent*. The oracle text "each poison counter your opponents
have" is unambiguously a sum (every poison counter on any opponent counts once). The
implementation must sum; document this in the variant comment to avoid future ambiguity.

### Sub-gap PB-CC-B: `TargetFilter.has_counter_type` (FIELD ADDITION)

Adds an optional counter-presence predicate to `TargetFilter`.

**Engine surface (estimated)**:
- New `TargetFilter::has_counter_type: Option<CounterType>` field (default `None`,
  `#[serde(default)]`).
- `matches_filter()` (`abilities.rs` / `effects/mod.rs`) — extra predicate over the object's
  `counters` map. NOTE per `card_definition.rs:2410-2413`: like `is_token`/`is_attacking`,
  `counters` are a `GameObject` field, not a `Characteristics` field — so the check must
  happen at *each* `EffectAmount::PermanentCount` callsite that filters by counter, not
  inside `matches_filter(&Characteristics, &TargetFilter)`. Search-library / SBA / target
  validation paths each need an explicit `obj.counters.contains_key(...)` short-circuit.

**Cards unblocked**:
- **Armorcraft Judge** ETB: "draw a card for each creature you control with a +1/+1 counter
  on it." Single-blocker. Ruling 2020-11-10: counts creatures, not counters; +1/+1-or-more
  threshold (`>= 1`).
- **Inspiring Call**: "Draw a card for each creature you control with a +1/+1 counter on it.
  Those creatures gain indestructible until end of turn." Cross-blocker — the second
  sentence requires granting indestructible to *the same matched set* (filter-defined
  multi-target grant), distinct from PB-CC.

### Sub-gap PB-CC-C: `LayerModification::ModifyPowerDynamic` / `ModifyToughnessDynamic` (NEW VARIANT)

Single-axis dynamic P/T modification at Layer 7c, parallel to existing `ModifyBothDynamic`.

**Engine surface (estimated)**:
- New `LayerModification::ModifyPowerDynamic { amount: Box<EffectAmount>, negate: bool }`
  and parallel `ModifyToughnessDynamic`.
- Substitution at `Effect::ApplyContinuousEffect` execution time (mirroring the existing
  `ModifyBothDynamic` substitution at `effects/mod.rs:2305-2315`).
- Layer-application code in `rules/layers.rs` must `panic!` / `debug_assert!` on encountering
  these variants in stored `ContinuousEffect` (mirror the existing pattern at
  `layers.rs:1162-1167`).
- `state/hash.rs` arm.

**Cards unblocked**:
- **Exuberant Fuseling** CDA: "this creature gets +1/+0 for each oil counter on it." Layer
  7c `ModifyPowerDynamic { amount: CounterCount { Source, Oil }, negate: false }` with
  `is_cda: true` and `EffectFilter::Source`. The death-trigger half (counter-on-creature-or-
  artifact dies) is a separate `WheneverCreatureOrArtifactDies` gap, multi-blocker.

### Sub-gap PB-CC-X (collapsed-or-defer): `TargetFilter.exclude_self` extension

Eomer's ETB "for each **other** Human you control" demands self-exclusion in
`PermanentCount`. `exclude_self` exists on `ETBTriggerFilter` /
`DeathTriggerFilter::exclude_self` (`game_object.rs:538,555`) but **not** on `TargetFilter`,
so it is unavailable to `EffectAmount::PermanentCount`. Adding it is independent of
counter-count primitives and is also useful for plenty of other "other creatures you
control"-style cards. **Out of PB-CC scope** — file as a separate primitive seed.

### Verdict label

**UMBRELLA-OF-MICRO-PRIMITIVES**. Four micro-PBs (W, A, B, C). Each has independent engine
surface, independent dispatch sites, and a small distinct slice of unblocked cards.

---

## (c) Per-card classification

| Card                                | Primary gap-variant       | Secondary blockers                                                  | Status                                       |
| ----------------------------------- | ------------------------- | ------------------------------------------------------------------- | -------------------------------------------- |
| **Mossborn Hydra**                  | PB-CC-W (wire-up)         | none                                                                | CONFIRMED-IN-SCOPE-VARIANT-W                 |
| **Vishgraz, the Doomhive**          | PB-CC-A (PlayerCounter)   | Layer 7c CDA path for `PlayerCounterCount` must be wired            | CONFIRMED-IN-SCOPE-VARIANT-A                 |
| **Phyrexian Swarmlord**             | PB-CC-A (PlayerCounter)   | TokenSpec.count=EffectAmount (PB-TS, separate)                      | BLOCKED-BY-OTHER-PRIMITIVE (PB-TS)            |
| **Phyresis Outbreak**               | PB-CC-A (PlayerCounter)   | Per-target EffectAmount in single ContinuousEffect — architectural  | BLOCKED-BY-OTHER-PRIMITIVE (per-target layer) |
| **Vraska, Betrayal's Sting** (-9)   | PB-CC-A (player-poison-diff) | Special-case `MAX(9-x,0)`; loyalty ability shape                  | OUT-OF-SCOPE (one-off, marginal yield)       |
| **Armorcraft Judge**                | PB-CC-B (filter)          | none                                                                | CONFIRMED-IN-SCOPE-VARIANT-B                 |
| **Inspiring Call**                  | PB-CC-B (filter)          | Multi-target indestructible grant on filter-set                     | BLOCKED-BY-OTHER-PRIMITIVE (filter-set grant) |
| **Exuberant Fuseling**              | PB-CC-C (Modify*Dynamic)  | `WheneverCreatureOrArtifactDies` trigger condition (separate)       | CONFIRMED-IN-SCOPE-VARIANT-C (CDA half only) |
| **Éomer, King of Rohan**            | PB-CC-W-ish               | `TargetFilter.exclude_self` (out-of-PB-CC scope)                    | BLOCKED-BY-OTHER-PRIMITIVE (exclude_self)    |
| **Out of the Tombs**                | PB-CC-W                   | Library-empty draw-replacement (separate primitive)                 | BLOCKED-BY-OTHER-PRIMITIVE                   |
| **Anim Pakal, Thousandth Moon**     | (token half) PB-TS        | TokenSpec.count=EffectAmount + non-Gnome-attacker filter            | BLOCKED-BY-OTHER-PRIMITIVE (PB-TS)            |
| **Replicating Ring**                | (threshold trigger)        | Counter-threshold `if N >=` trigger condition (separate primitive) | BLOCKED-BY-OTHER-PRIMITIVE                   |
| **Chasm Skulker**                   | (token half) PB-TS        | TokenSpec.count=EffectAmount via LKI                                | BLOCKED-BY-OTHER-PRIMITIVE (PB-TS)            |

**Summary by gap-variant**:
- W (wire-up): **1 confirmed** (Mossborn).
- A (PlayerCounter): **1 confirmed** (Vishgraz). 3 secondary-blocked.
- B (TargetFilter.has_counter): **1 confirmed** (Armorcraft Judge). 1 secondary-blocked.
- C (ModifyPower/ToughnessDynamic): **1 confirmed** (Exuberant Fuseling, CDA half only).

---

## (d) Yield calibration vs. `feedback_pb_yield_calibration.md`

**Brief planner's claim**: "rank 6, ~10+ live TODOs across counter-count EffectAmount
variants" — implies a single-PB scope of ~10 cards.

**Calibration**: per the feedback file, **EffectAmount/mana PBs run 50–65% yield**
(PB-Q4 36%, PB-Q 33%, PB-P 23% — historical EffectAmount yield is *below* the table's
midpoint when the planner overcounted). Discount factor: 2-3x. Adjusted single-PB
expectation: **3-5 ships**.

**Per-micro-PB yield estimate**:

| Micro-PB | Candidates surfaced | Confirmed-in-scope | Expected ship after discount |
| --- | --- | --- | --- |
| PB-CC-W (wire-up) | 1 (Mossborn) | 1 | **1** (trivial; no engine code) |
| PB-CC-A (PlayerCounterCount) | 4 (Vishgraz, Swarmlord, Phyresis, Vraska -9) | 1 | **1** (Vishgraz only; Phyresis architectural; Swarmlord blocked on TS; Vraska one-off) |
| PB-CC-B (TargetFilter.has_counter) | 2 (Armorcraft, Inspiring Call) | 1 | **1** (Armorcraft only) |
| PB-CC-C (Modify*Dynamic) | 1-2 (Fuseling; possibly Aspect of Hydra-shape if widened, but Aspect uses DevotionTo — already works via ModifyBothDynamic) | 1 | **1** (Fuseling CDA half) |
| **Umbrella total** | 8-9 | **4** | **4** ships |

This is **41-50% of the brief's ~10-candidate framing** — consistent with the historical
EffectAmount-PB midpoint after the 2-3x overcount discount. Each micro-PB ships
~1 card; the umbrella collectively ships **~4 cards** across 4 small engine changes.

**Process implication**: this matches the PB-Q-style outcome (planner counted 6, shipped 2,
spawned 4 micro-PBs Q2–Q5). PB-CC should be planned as **4 sequenced micro-PBs from the
start**, not as one PB with 4 inevitable spawns.

---

## (e) Dispatch-ready scope per micro-primitive

### PB-CC-W — Wire-up Mossborn Hydra (no engine code)

- **Engine surface**: none.
- **Card-def changes**: `mossborn_hydra.rs` — add Landfall ability with
  `WheneverPermanentEntersBattlefield { filter: TargetFilter { has_card_type: Some(Land),
  controller: TargetController::You, .. } }` triggering `Effect::AddCounterAmount` with
  `count: CounterCount { Source, PlusOnePlusOne }`.
- **Mandatory tests**: 1 unit test (token assertions: 1 counter → 2; 2 → 4) + 1 game script
  validating ruling 2024-11-08 ordering with multiple lands entering simultaneously
  (`crates/engine/src/cards/defs/mossborn_hydra.rs` test module).
- **Estimated dispatch sites**: 0 engine, 1 card def, 2 tests.
- **Sequencing**: ship first — validates the engine claim with zero risk.

### PB-CC-B — `TargetFilter.has_counter_type` field addition

- **Engine surface**:
  - `card_definition.rs:2336-2429`: add `pub has_counter_type: Option<CounterType>` with
    `#[serde(default)]`.
  - `abilities.rs` / wherever `matches_filter` is invoked for `EffectAmount::PermanentCount`
    — add an `obj.counters.contains_key(...)` short-circuit at each callsite. (Mirror the
    `is_token` / `is_attacking` pattern.)
  - `state/hash.rs` — extend `TargetFilter` hashing if a hand-rolled `Hash` impl exists.
- **Card-def changes**: `armorcraft_judge.rs` — change the existing approximation
  `PermanentCount { filter: { has_card_type: Creature, controller: You } }` to
  `... has_counter_type: Some(CounterType::PlusOnePlusOne) ...`.
- **Mandatory tests**:
  - 1 unit test: `matches_filter` short-circuit honors `has_counter_type` — both positive
    (creature with counter passes) and negative (creature without counter fails).
  - 1 game script: Armorcraft Judge ETB with 0/1/2 creatures bearing +1/+1 counters →
    correct draw count (ruling 2020-11-10: count of creatures, not counters).
- **Estimated dispatch sites**: 1 engine field + 2-4 callsites + 1 card def + 2 tests.
- **Sequencing**: ship second — small, well-contained.

### PB-CC-C — `LayerModification::ModifyPowerDynamic` / `ModifyToughnessDynamic`

- **Engine surface**:
  - `state/continuous_effect.rs:391`: add `ModifyPowerDynamic { amount: Box<EffectAmount>,
    negate: bool }` and `ModifyToughnessDynamic { amount: Box<EffectAmount>, negate: bool }`.
  - `effects/mod.rs:2305-2315`: extend the substitution arm to handle the two new variants
    (resolve `amount` to `i32`, replace with `ModifyPower(v)` / `ModifyToughness(v)`).
  - `rules/layers.rs:1162-1167`: extend the `panic!`/`debug_assert!` arm so unsubstituted
    variants surface as bugs.
  - `state/hash.rs:1464-1466`: add hash arms.
- **Card-def changes**: `exuberant_fuseling.rs` — add the CDA Layer 7c `ModifyPowerDynamic`
  effect with `is_cda: true`, `EffectFilter::Source`,
  `amount: CounterCount { Source, Oil }`, `negate: false`. Leave the
  WheneverCreatureOrArtifactDies half as TODO (out of PB-CC-C scope).
- **Mandatory tests**:
  - 1 unit test for substitution arm (verify substitution at `ApplyContinuousEffect`).
  - 1 unit test for layer-application `panic!` on unsubstituted variant.
  - 1 game script: Exuberant Fuseling with 0/1/3 oil counters → power 0/1/3, toughness 1.
- **Estimated dispatch sites**: 2 engine variants + 3 dispatch arms + 1 card def + 3 tests.
- **Sequencing**: ship third — extends an existing, trusted pattern.

### PB-CC-A — `EffectAmount::PlayerCounterCount` (NEW VARIANT)

- **Engine surface**:
  - `card_definition.rs:2200-2203` (after existing `CounterCount`): add
    `PlayerCounterCount { player: PlayerTarget, counter: CounterType }`.
  - `effects/mod.rs:6039-6056`: add resolution arm reading `state.players[id].poison_counters`
    when `counter` is `Poison`; sum across resolved players (e.g., `PlayerTarget::EachOpponent`
    sums); explicit no-op (return 0) for unsupported counter kinds. Document the sum
    semantic.
  - `rules/layers.rs:1492-1504`: add CDA arm reading the same. No layer-recursion concern
    (player counters aren't layer-derived characteristics).
  - `state/hash.rs:4393`: add hash arm.
- **Card-def changes**: `vishgraz_the_doomhive.rs` — add CDA Layer 7c
  `ModifyBothDynamic { amount: PlayerCounterCount { EachOpponent, Poison }, negate: false }`
  with `is_cda: true`, `EffectFilter::Source`. (Already-shipped ETB token-creation half is
  unchanged.)
- **Mandatory tests**:
  - 1 unit test: `resolve_amount` with each `PlayerTarget` variant (`Controller`,
    `EachOpponent`, `EachPlayer`, `DeclaredTarget`).
  - 1 unit test: CDA path for Vishgraz P/T (verify the value flows through the layer
    system).
  - 1 game script: Vishgraz P/T scaling with 0/3/8 opponent poison counters across multiple
    opponents.
- **Estimated dispatch sites**: 1 engine variant + 3 dispatch arms + 1 card def + 3 tests.
- **Sequencing**: ship fourth (last) — wider state-access surface; touches both
  `resolve_amount` and `resolve_cda_amount`. Phyresis Outbreak's per-target architectural
  problem is **not** part of PB-CC-A; it remains an open seed for a later
  per-target-EffectAmount layer primitive.

---

## (f) Stop-and-flag — cards touching other unimplemented primitives

The following cards appeared in the brief or in the TODO sweep but are blocked on a
**different** primitive entirely (or in addition to PB-CC). Do **not** attempt them in
any PB-CC micro-PB; treat each as a separate-primitive seed.

| Card                              | Other primitive needed                                                                 | Notes                                                                                  |
| --------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| **Phyrexian Swarmlord**           | **PB-TS** (`TokenSpec.count: u32 → EffectAmount`)                                      | Upkeep "create a token for each poison counter…"; also affects Krenko, Izoni, Chasm Skulker, Anim Pakal, Galadhrim Ambush. **Strong candidate for its own micro-PB**. |
| **Chasm Skulker**                 | **PB-TS** (LKI death-trigger token count)                                              | Death trigger "create X tokens, X = +1/+1 counters" — counter-count exists but TokenSpec.count doesn't accept EffectAmount. |
| **Anim Pakal, Thousandth Moon**   | **PB-TS** + non-Gnome attacker filter                                                  | Token half blocked on PB-TS; trigger filter "non-Gnome creatures attack" is its own gap. |
| **Replicating Ring**              | Counter-threshold trigger condition (`if 8+ counters then …`)                          | Distinct primitive: `TriggerCondition::AtBeginningOfYourUpkeep` works, but the threshold gate inside the effect has no DSL representation. |
| **Phyresis Outbreak** (-1/-1 half)| Per-target dynamic `EffectAmount` in a single `ContinuousEffect`                       | Each opponent-controlled creature reads ITS OWN controller's poison count. Architecturally distinct from PB-CC-A. Likely needs `ModifyBothDynamic` to support per-target evaluation, OR fan out into N continuous effects (one per opponent). Engine architecture decision. |
| **Vraska, Betrayal's Sting** (-9) | Special-case `MAX(9-x, 0)` poison-difference EffectAmount                              | One-off. Marginal yield. Defer.                                                        |
| **Out of the Tombs**              | Draw-replacement (library empty → return creature card)                                | Mill scaling part is wire-up, but the second clause needs a draw-while-empty replacement. Multi-blocker. |
| **Éomer, King of Rohan**          | `TargetFilter.exclude_self` (or new "exclude source" semantics in `PermanentCount`)    | "+1/+1 counter for each *other* Human" — counts source itself otherwise. Independent primitive seed; useful far beyond Eomer. |
| **Inspiring Call**                | Multi-target grant over a filter-defined set ("those creatures gain indestructible")   | Distinct from PB-CC-B; would also need `Effect::GrantUntilEndOfTurn` over a set.       |
| **Everflowing Chalice**           | ETB-with-N-counters from kicker count (multikicker → counter)                          | Kicker-count → counter-on-ETB replacement effect. Comment in card def claims `CounterCount` is in use; **the comment is stale** (the tap ability uses fixed mana, not CounterCount). |
| **Hardened Scales / Conclave Mentor / Corpsejack Menace** | Counter-doubling replacement effect (CR 121.6)                              | "if N counters would be put on …, M instead" — replacement-effect primitive, separate. |
| **Master Biomancer**              | ETB-replacement counter placement based on source's power                              | "each other creature you control enters with +1/+1 counters equal to this creature's power" — replacement, not EffectAmount. |
| **Fathom Mage**                   | `WheneverCounterIsPlacedOn` trigger condition                                          | Distinct trigger primitive.                                                            |
| **Ainok Bond-Kin**                | Layer 6 grant filter "with +1/+1 counter"                                              | Filter applies to a Layer 6 grant, not a count. Different code path from PB-CC-B (which is for `EffectAmount::PermanentCount`). |
| **Aspect of Hydra**               | None — TODO is **stale**                                                               | DevotionTo + ModifyBothDynamic both exist; card is a wire-up regardless of PB-CC. Out of PB-CC scope. |

---

## Recommended PB sequencing

1. **PB-CC-W** — wire up Mossborn Hydra. ~30 lines card-def, 2 tests, no engine. Validates
   the engine claim and clears one TODO immediately.
2. **PB-CC-B** — `TargetFilter.has_counter_type` field + Armorcraft Judge fix. Smallest
   true engine surface (1 field, ~3 callsites).
3. **PB-CC-C** — `ModifyPowerDynamic` / `ModifyToughnessDynamic` + Exuberant Fuseling CDA.
   Extends a trusted pattern (`ModifyBothDynamic`).
4. **PB-CC-A** — `EffectAmount::PlayerCounterCount` + Vishgraz CDA. Largest surface
   (resolve_amount + resolve_cda_amount + hash + state-access pattern).

After all four ship, the *brief's roster* is reduced from ~13 cards to:
- 4 shipped (Mossborn, Armorcraft Judge, Exuberant Fuseling, Vishgraz)
- 9 still parked, but each on a clearly named separate primitive (PB-TS for the token-count
  trio; per-target-EffectAmount seed; counter-threshold trigger; counter-doubling
  replacement; library-empty draw replacement; etc.)

The parked cards are **named, dispatchable seeds** for the next round of micro-PBs — exactly
the outcome the yield-calibration feedback note recommends ("treat PB sizing as 'primitive
scope + likely-shippable cards,' not 'all cards using the primitive.' The remainder are
micro-PB seeds, not failures.").
