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
| **Hardened Scales / Conclave Mentor / Corpsejack Menace** | Counter-doubling replacement effect (CR 122.6)                              | "if N counters would be put on …, M instead" — replacement-effect primitive, separate. |
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

---

## OOS Seeds appended by PB-TS runner (scutemob-16, 2026-04-30)

### OOS-TS-1: Anim Pakal attacker filter

**Card**: Anim Pakal, Thousandth Moon
**Oracle text**: "Whenever Anim Pakal or another nontoken creature you control attacks, create a
1/1 colorless Gnome artifact creature token."
**Gap**: The WheneverYouAttackWithNonTokenNonGnome trigger condition requires filtering by
`is_token: false` AND `has_subtype != Gnome` for the attacking creature. The current
`TriggerCondition::WhenAttacks` and `ETBTriggerFilter` do not cover this pattern. A new
`TriggerCondition::WheneverYouControlledCreatureAttacks { filter: AttackTriggerFilter }` variant
or an extension of the existing attacker-trigger path is needed.
**Blocked on**: Attacker-trigger filter primitive (new `TriggerCondition` variant + dispatch in
`check_triggers` over `CreatureAttacked` or `AttackersDeclared` events).

### OOS-TS-2: Izoni sacrifice-another-creature activated ability

**Card**: Izoni, Thousand-Eyed
**Oracle text**: "{B}{G}, Sacrifice another creature: You gain 1 life and draw a card."
**Gap**: The cost `Sacrifice another creature` is a "sacrifice a creature you control other than
the source" cost distinct from `ActivationCost::sacrifice_self` (which sacrifices the source).
No `ActivationCost` variant for sacrifice-other exists. The card def currently has only the
Undergrowth ETB trigger; the activated ability is left as a TODO.
**Blocked on**: `ActivationCost` variant for sacrifice-another-creature (Cost::SacrificeOther or
ActivationCost::sacrifice_filter excluding the source). Appended to OOS seeds 2026-04-29 by
PB-TS runner.

### OOS-TS-3: CreateTokenAndAttachSource missing replacement-effect call

**Card**: Living Weapon permanents (e.g., Batterskull, Kaldra Compleat)
**Gap**: `Effect::CreateTokenAndAttachSource` currently does NOT call
`apply_token_creation_replacement` (the token-doubling boundary). This means Doubling Season /
Parallel Lives / Anointed Procession do NOT double the Germ token. `Effect::CreateToken` was
fixed by PB-TS (the `resolve_amount` call was added before the replacement boundary), but
`CreateTokenAndAttachSource` only got the `resolve_amount` call — the replacement call is
still absent. The fix is to add `apply_token_creation_replacement(state, ctx.controller, resolved_count)`
inside `CreateTokenAndAttachSource`'s dispatch arm (mirroring `CreateToken`).
**Blocked on**: Engine fix in `effects/mod.rs` `Effect::CreateTokenAndAttachSource` arm; small
isolated change, no new primitives required.

### OOS-TS-4: Pre-death counter snapshot primitive

**Cards**: Chasm Skulker, Toothy Imaginary Friend (and any future "when [permanent] dies, ...
where X is the number of [counter] counters on it" patterns)
**Oracle pattern**: "When [permanent] dies, create X 1/1 [type] creature tokens, where X is the
number of [counter] counters on [permanent]." / "When [permanent] leaves the battlefield, ..."
**Gap**: `move_object_to_zone` (state/mod.rs:420) creates a new `GameObject` with
`counters: OrdMap::new()`, resetting all counter state on every zone change per CR 400.7. When a
WhenDies / WhenLeavesBattlefield trigger resolves and its effect calls
`EffectAmount::CounterCount { target: EffectTarget::Source, counter: ... }`, `resolve_amount`
reads `state.objects[ctx.source].counters` — which is the **graveyard** object with empty counters.
Result: the resolved count is always 0, producing wrong game state (0 tokens instead of X).
CR 603.10a states that leaves-battlefield triggers "look back in time" for information about the
object as it existed on the battlefield (the "last known information" rule). The engine has no
mechanism to snapshot pre-death counter state and thread it through to trigger resolution.
**Two possible engine paths**:
(a) Add `EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType }` — resolved
    from a counter snapshot stored in `PendingTrigger` at the time the trigger fires (before
    move_object_to_zone returns), or from `EffectContext.lki_counters` populated by the
    WhenDies trigger dispatch. Cleanest approach; aligns with CR 603.10a / 113.7a.
(b) Preserve counters on the graveyard object (copy them from the pre-transition battlefield
    object into the new graveyard object). Smaller change but breaks the "new object, empty
    counters" invariant from CR 400.7 / CR 122.2; risks cascading side effects on other
    counter-check sites.
**Yield**: ≥1 confirmed (Chasm Skulker token-create WhenDies). Toothy Imaginary Friend
(WhenLeavesBattlefield draw X) is also broken by the same gap — it shipped pre-PB-TS but
produces 0 draws, not X; this primitive would fix Toothy retroactively. Sweep for
"when ... dies/leaves" + "number of ... counters" to find remaining cards.
**Status**: Filed by PB-TS fix-phase 2026-04-30. Chasm Skulker death-trigger ability reverted
to TODO comment in chasm_skulker.rs pending this primitive. Toothy deferred to this fix.
**References**: state/mod.rs:420 (counters reset), effects/mod.rs:6011-6012 (comment on
non-battlefield empty counters), CR 603.10a, CR 113.7a, CR 400.7, CR 122.2.

---

## OOS seeds filed by PB-LKI-CC (scutemob-17, 2026-04-29)

### OOS-LKI-1: Hardened Scales + CounterCountAtLastKnownInformation interaction

**Category**: Out-of-scope interaction (replacement effects on counter placement from LKI tokens)
**Discovered during**: PB-LKI-CC test planning
**Description**: Hardened Scales says "If one or more +1/+1 counters would be placed on a
creature you control, that many plus one +1/+1 counters are placed on it instead."
When Chasm Skulker's WhenDies trigger creates N Squid tokens, the tokens themselves enter
with 0 counters — no counter placement, so Hardened Scales does not interact with the token
count at creation time. The `CounterCountAtLastKnownInformation` variant correctly resolves
to the pre-death count; the Scales replacement has no surface here. This OOS item documents
the confirmed no-interaction: the LKI counter read and Scales are orthogonal. No engine
change required.
**Status**: CONFIRMED-NO-INTERACTION. Documented for future reviewer clarity.

### OOS-LKI-2: Parallel Lives / Anointed Procession + LKI token creation count

**Category**: Out-of-scope interaction (token doubling replacement on LKI-driven CreateToken)
**Discovered during**: PB-LKI-CC test planning
**Description**: "If you would create one or more tokens, you instead create twice that many."
When Chasm Skulker's WhenDies trigger creates N Squid tokens via `Effect::CreateToken` with
`count: CounterCountAtLastKnownInformation`, the existing `apply_token_creation_replacement`
boundary in `effects/mod.rs` runs AFTER `resolve_amount(spec.count, ...)` computes N. The
doubling replacement correctly doubles the resolved N — it is not bounded by the LKI path.
No new engine work required; the boundary is already in the right place (post-resolve, pre-create).
**Status**: CONFIRMED-WORKING-CORRECTLY. No regression from PB-LKI-CC. Documented for future
reviewer clarity.

---

## Additional OOS seeds filed by PB-LKI-CC fix-phase (scutemob-17, 2026-04-29)

These are the seeds originally drafted by the planner in pb-plan-LKI-CC.md Step 4.
The runner filed OOS-LKI-1/2 as no-interaction docs; these become OOS-LKI-3/4.

### OOS-LKI-3: Cost-payment LKI counter snapshot for activated abilities

**Cards**: Workhorse (`{T}, sacrifice this: add X mana, X = number of +1/+1 counters on it`), and
any activated ability that sacrifices its source as cost and reads the source's counter count
for the effect.
**Oracle pattern**: "{cost incl. sacrifice this}: [effect] X = number of [counter] counters on this."
**Gap**: PB-P (`PowerOfSacrificedCreature`) snapshots LKI power at cost-payment time
(`EffectContext.sacrificed_creature_powers`) but does NOT snapshot LKI counters. PB-LKI-CC
(HASH 15) ships LKI counter-snapshot for WhenDies / WhenLeavesBattlefield triggers (via
`PendingTrigger.lki_counters`) but NOT for activated-ability cost-payment paths. The two are
mechanically different snapshot sites: trigger-fire snapshot vs. cost-payment snapshot.
**Dispatch chain**: `abilities.rs` pay_costs → `PermanentDestroyed`/`ObjectExiled` → resolution.
A new `EffectAmount::CounterCountAtCostPaymentTime { counter }` variant would be needed, OR
extending `EffectContext.sacrificed_creature_counters: OrdMap<CounterType, u32>` parallel to
`sacrificed_creature_powers`.
**Yield**: Workhorse confirmed + sweep `Cost::SacrificeSelf` activated abilities for
`EffectAmount::CounterCount` references.
**Status**: Filed by PB-LKI-CC fix-phase 2026-04-29.
**References**: pb-plan-LKI-CC.md Step 4 OOS-LKI-1 draft; abilities.rs pay_costs sacrifice path.

### OOS-LKI-4: AnyCreatureDies dying-creature LKI counter access

**Cards**: Hypothetical "Whenever a creature with +1/+1 counters dies, ..." or "Whenever a
creature dies, draw cards equal to the +1/+1 counters that were on it." None confirmed in
current card-def universe.
**Oracle pattern**: AnyCreatureDies trigger reading the dying creature's LKI counter count.
**Gap**: PB-LKI-CC threads LKI counters into `PendingTrigger.lki_counters` ONLY for
SelfDies / SelfLeavesBattlefield triggers. AnyCreatureDies triggers fire on OTHER permanents
(Blood Artist, Zulaport Cutthroat etc.) — the dying creature is the *triggering object*, not
the trigger source. A different snapshot field would be needed: e.g.
`triggering_creature_lki_counters: OrdMap<CounterType, u32>` on `PendingTrigger`, populated
in the `AnyCreatureDies` dispatch arm at `abilities.rs:4318` from the event's `pre_death_counters`.
**Dispatch site**: `abilities.rs:4318` currently has `lki_counters: im::OrdMap::new()` —
intentionally left empty per plan Risk #1 (separate primitive).
**Yield**: 0 confirmed in current pool. File as preventive seed.
**Status**: Filed by PB-LKI-CC fix-phase 2026-04-29.
**References**: pb-plan-LKI-CC.md Step 4 OOS-LKI-2 draft; abilities.rs:4318 AnyCreatureDies arm.

---

## OOS seeds filed by PB-CD (scutemob-18, 2026-05-13)

### OOS-LKI-Power: LKI source-power snapshot for WhenDies / WhenLeavesBattlefield triggers

**Cards**: Conclave Mentor ("When this creature dies, you gain life equal to its power"),
Juri, Master of the Revue ("When Juri dies, it deals damage equal to its power to any
target"), and any future "When [permanent] dies/leaves, [effect] equal to its power /
toughness" patterns.
**Oracle pattern**: SelfDies / SelfLeavesBattlefield trigger reading
`EffectAmount::SourcePower` (or `SourceToughness`) where the source is already in the
graveyard at trigger resolution time.
**Gap**: PB-LKI-CC (HASH 15) ships LKI **counter** snapshots through
`PendingTrigger.lki_counters`, `StackObject.lki_counters`, and
`EffectContext.lki_counters` for SelfDies / SelfLeavesBattlefield triggers. It does NOT
snapshot the source's layer-resolved **power** or **toughness** at trigger-fire time.
`EffectAmount::SourcePower` does not yet exist in the DSL; if added, it would read
`calculate_characteristics(state, ctx.source).power` from the graveyard'd object — which
has lost battlefield-layer state per CR 400.7 / 122.2. Both the DSL variant AND the LKI
snapshot are needed; without them, "gain life equal to its power" resolves to 0.
**Symmetry with PB-LKI-CC**: the snapshot site (sba.rs:540 where `pre_death_counters` is
already computed) would also capture `pre_death_power: i32` and `pre_death_toughness: i32`
into `GameEvent::CreatureDied` and thread them through to `PendingTrigger.lki_power: Option<i32>`.
The dispatch chain is identical to PB-LKI-CC; only the snapshot field and the
`EffectAmount` variant resolution differ.
**Yield**: ≥2 confirmed (Conclave Mentor death trigger + Juri Master death trigger;
Juri's existing card def at `cards/defs/juri_master_of_the_revue.rs:37-38` already
documents the same gap). Sweep `"equal to its power"` + WhenDies/WhenLeavesBattlefield
for additional candidates.
**Status**: CLOSED by PB-LKI-Power (scutemob-19, 2026-05-13). `EffectAmount::SourcePowerAtLastKnownInformation`
(disc 18, HASH 17) ships in this batch. Conclave Mentor death trigger and Juri Master death trigger both
implemented. See `pb-plan-LKI-Power.md` for full dispatch-chain audit.

---

## OOS seeds filed by PB-LKI-Power (scutemob-19, 2026-05-13)

### OOS-LKI-Power-1: SourceToughnessAtLastKnownInformation

**Cards**: hypothetical "When ~ dies, [effect] X = its toughness". None confirmed
in current card-def universe (sweep 2026-05-13 found no SelfDies/SelfLeavesBattlefield
trigger reading source toughness).
**Oracle pattern**: SelfDies/SelfLeavesBattlefield trigger reading source's own
toughness at LKI.
**Gap**: PB-LKI-Power (HASH 17) ships `EffectAmount::SourcePowerAtLastKnownInformation`
(disc 18) and reserves disc 19 for the toughness sibling. The
`pre_death_power: Option<i32>` snapshot infrastructure at sba.rs:540 +
PendingTrigger/StackObject/EffectContext threading + GameEvent payload extension
all generalize trivially: add `pre_death_toughness: Option<i32>` alongside,
add disc 19 variant, add resolve_amount arm reading `ctx.lki_toughness`.
**Yield**: 0 confirmed in current pool. File as preventive seed.
**Status**: Filed by PB-LKI-Power planner 2026-05-13.

### OOS-LKI-Power-2: ReplacementModification::EntersWith(EffectAmount) — Master Biomancer

**Cards**: Master Biomancer ("Each other creature you control enters with a number
of +1/+1 counters on it equal to Biomancer's power"), and any future card with
similar ETB-replacement wording reading the source's live power.
**Oracle pattern**: `EnterFromX` replacement that places counters where the count
is dynamic (= source's power, source's toughness, count of permanents, etc.).
**Gap**: today, `ReplacementModification::EntersWith` accepts a static u32
counter count, not an `EffectAmount`. The source is alive on the battlefield
when the replacement fires (not LKI), so `EffectAmount::PowerOf(EffectTarget::Source)`
would resolve correctly via the live arm — but the replacement DSL doesn't
plumb EffectAmount through. This is the replacement-side mirror of the PB-TS
TokenSpec.count u32→EffectAmount migration.
**Yield**: 1 confirmed (Master Biomancer); broader sweep would surface more
ETB-replacement cards using "equal to X" wording.
**Status**: Filed by PB-LKI-Power planner 2026-05-13.

### OOS-LKI-Power-3: GameEvent LBA hash arms don't hash pre_lba_counters or pre_lba_power

**Cards**: N/A (engine consistency issue, not card-blocking).
**Gap**: `GameEvent::AuraFellOff`, `GameEvent::ObjectExiled`,
`GameEvent::PermanentDestroyed`, and `GameEvent::ObjectReturnedToHand` hash
arms in `state/hash.rs` use `..` and do NOT hash their
`pre_lba_counters` (added by PB-LKI-CC) or `pre_lba_power` (added by
PB-LKI-Power) fields. Only `GameEvent::CreatureDied` hashes its LKI fields.
This is a pre-existing inconsistency that PB-LKI-CC and PB-LKI-Power both
intentionally preserve to minimize blast radius. Replay determinism is
preserved because PendingTrigger and StackObject DO hash these fields, and
GameEvents are derived state recomputable from commands.
**Yield**: 0 (engine-consistency cleanup, no card unblocking).
**Status**: Filed by PB-LKI-Power planner 2026-05-13. Resolution would bump
HASH_SCHEMA_VERSION; defer until a determinism issue is observed in production.

### OOS-LKI-Power-4: AnyCreatureDies + LKI source-power gap

**Cards**: hypothetical cards using "Whenever a creature dies, [effect] equal to
its power" patterns where "its" refers to the dying creature (not the trigger
source). Examples that *don't* exist in the current pool but would hit this gap:
hypothetical "Whenever a creature dies, you gain life equal to its power"
global-trigger card.
**Oracle pattern**: AnyCreatureDies trigger reading the *triggering* (dying)
creature's LKI power, not the trigger *source*'s LKI power.
**Gap**: `EffectAmount::SourcePowerAtLastKnownInformation` (PB-LKI-Power, disc 18)
reads `ctx.lki_power`, which is populated from `PendingTrigger.lki_power` set
in the SelfDies/SelfLeavesBattlefield trigger arms (CR 603.10a / source = trigger
source). The AnyCreatureDies arm (`abilities.rs:4391`) intentionally defaults
`lki_power: None` because the dying creature is the *triggering object*, not
the trigger source. A different snapshot field would be needed: e.g.
`triggering_creature_lki_power: Option<i32>` on `PendingTrigger`, populated
from the GameEvent's `pre_death_power`. Symmetric to OOS-LKI-4 (counter
version filed by PB-LKI-CC).
**Dispatch site**: `abilities.rs:4391` AnyCreatureDies arm — currently
`lki_power: None` (intentionally per plan Site 9 + plan Risk #1).
**Yield**: 0 confirmed in current pool. Preventive seed (mirror of OOS-LKI-4).
**Status**: Filed by PB-LKI-Power reviewer 2026-05-13 (E1 finding).
**References**: pb-review-LKI-Power.md E1; pb-retriage-CC.md OOS-LKI-4 (counter
sibling); pb-plan-LKI-Power.md Step 4 risk register.

### OOS-LKI-Power-5: Non-creature SBA paths hard-code pre_lba_power: None (Layer 4 animation loss)

**Cards**: hypothetical animated-planeswalker / animated-Saga / animated-Aura
that becomes a creature via Layer 4 (e.g. Karn ultimate, Roalesk-style global
animation) AND has a SelfLeavesBattlefield trigger reading
`EffectAmount::SourcePowerAtLastKnownInformation`. None confirmed in current
pool.
**Oracle pattern**: Layer 4 type-grant effect produces a power on a non-creature
permanent that subsequently leaves the battlefield via a non-CreatureDied SBA
path (planeswalker SBA-exile, Saga SBA-sacrifice, Aura SBA-fall-off, Food forage).
**Gap**: 4 SBA sites hard-code `pre_lba_power: None` (sba.rs:733 planeswalker
exile, sba.rs:854 Saga sacrifice, sba.rs:1170 Aura fall-off, abilities.rs:890
Food forage) with comments saying "X are not creatures; no power LKI needed."
This is correct for the printed/base power but loses the layer-resolved power
if a Layer 4 animation effect was active. Symmetric to PB-LKI-CC's identical
pattern for `pre_lba_counters` at the same sites.
**Yield**: 0 confirmed in current pool. Preventive seed.
**Status**: Filed by PB-LKI-Power reviewer 2026-05-13 (E2 finding). Defer
until an animated-non-creature card with a SelfLeavesBattlefield power trigger
surfaces. Mechanical fix: replace `None` with `calculate_characteristics(state, id).power`
at each of the 4 sites.
**References**: pb-review-LKI-Power.md E2; pb-plan-LKI-Power.md Site 6.

---

## OOS seeds filed by PB-EWC (scutemob-20, 2026-05-14)

### OOS-EWC-1: ReplacementModification::EntersAsAdditionalType — Master Biomancer (type-grant half)

**Cards**: Master Biomancer ("...and as a Mutant in addition to its other types").
Future cards with the same pattern (e.g. "creature you control enters as a
[Type] in addition to its other types") would hit the same gap.
**Oracle pattern**: ETB replacement that adds a subtype to the entering
permanent on top of its inherent types (NOT a Layer 4 continuous effect on
permanents already on the battlefield — this is a one-time entry modification).
**Gap**: PB-EWC (HASH 18) ships `EntersWithCounters` with `EffectAmount` count
but no parallel `EntersAsAdditionalType { type: SubType }` modification.
Adding it requires:
  1. A new `ReplacementModification::EntersAsAdditionalType { subtype: SubType }` variant.
  2. A new arm in `emit_etb_modification` (replacement.rs) that pushes the
     subtype into `state.objects[new_id].characteristics.subtypes` BEFORE
     emitting `PermanentEnteredBattlefield`.
  3. HASH bump for the new discriminant.
  4. Author the type-grant half of `master_biomancer.rs` (TODO line preserved
     in the def).
**Yield**: 1 confirmed (Master Biomancer). Combined with `OOS-EWC-2` and
`OOS-EWC-3` below this is the next logical Wave-C primitive for the
replacement family.
**Status**: Filed by PB-EWC 2026-05-14. Author hint reserved in
`master_biomancer.rs` as a TODO referencing this seed.
**References**: pb-plan-EWC; CR 614.1c.

### OOS-EWC-2: EntersWithCounters dynamic count — Golgari Grave-Troll

**Cards**: Golgari Grave-Troll ("This creature enters with a +1/+1 counter on
it for each creature card in your graveyard"). Self-ETB; count = count of
creature cards in controller's graveyard.
**Oracle pattern**: Self-ETB `EntersWithCounters` with
`EffectAmount::CardCount { zone: Graveyard(Controller), player: Controller,
filter: Some(TargetFilter { has_card_types: vec![CardType::Creature], .. }) }`.
PB-EWC's resolver already evaluates `EffectAmount::CardCount` correctly in the
ETB EffectContext, so the only authoring requirement is a card-def edit plus
a dredge-interaction test.
**Gap**: card-def TODO at `crates/engine/src/cards/defs/golgari_grave_troll.rs`.
No engine work required after PB-EWC.
**Yield**: 1 confirmed (Golgari Grave-Troll). Pure card-authoring follow-up.
**Status**: Filed by PB-EWC 2026-05-14 (sweep). Recommended to ship as a
single-card follow-up alongside `OOS-EWC-3` and Eomer's `TargetFilter.exclude_self`.
**References**: pb-plan-EWC; CR 614.1c; existing Dredge test scaffolding.

### OOS-EWC-3: EntersWithCounters dynamic count + subtype receiver — Dragonstorm Globe

**Cards**: Dragonstorm Globe ("Each Dragon you control enters with an
additional +1/+1 counter on it"). Non-self ETB; receiver filter is
"Dragon you control" (subtype + controller).
**Oracle pattern**: Non-self `EntersWithCounters` with static
`EffectAmount::Fixed(1)` BUT receiver filter requires a new
`ObjectFilter::CreatureControlledByOfSubtype(PlayerId, SubType)` variant or
generalization of `CreatureControlledBy` to accept an optional subtype.
**Gap**: PB-EWC unblocked the count side but not the filter side. The
existing `ObjectFilter::CreatureControlledBy(controller)` matches any creature;
no variant gates on subtype today. **Sub-gap (E2 from
`pb-review-EWC.md`)**: `bind_object_filter` does not rebind
`OwnedByOpponentsOf(PlayerId(0))` for `WouldEnterBattlefield` triggers.
The symmetric `WouldChangeZone` arm in
`register_permanent_replacement_abilities` handles this case; the new
`WouldEnterBattlefield` arm does not. No in-scope card hits it (Master
Biomancer uses `CreatureControlledBy`, Ingenious Prodigy uses `Any`),
but a hypothetical "When a creature an opponent controls enters, ..."
would leak the placeholder through registration. Fix: extend
`bind_object_filter` to also rebind `OwnedByOpponentsOf(PlayerId(0))` →
`OwnedByOpponentsOf(controller)` (~3 lines), or add explicit pattern
arm in `register_permanent_replacement_abilities` symmetric to the
existing `WouldChangeZone { filter: OwnedByOpponentsOf(_) }` arm.
**Yield**: 1 confirmed (Dragonstorm Globe). Lower priority; not strictly
required for PB-EWC scope.
**Status**: Filed by PB-EWC 2026-05-14 (sweep). Sub-gap routed from
PB-EWC review LOW E2 2026-05-14 (no in-scope card affected). Defer until
a future replacement-filter expansion (potentially fold into the broader
replacement filter rework alongside Eomer's `TargetFilter.exclude_self`).
**References**: pb-plan-EWC; pb-review-EWC.md E2; CR 614.1c; ObjectFilter
variants in `state/replacement_effect.rs`.

---

## OOS seeds filed by PB-XS (scutemob-21, 2026-05-14)

PB-XS shipped `TargetFilter.exclude_self: bool` (HASH 18→19) for the
"another target X" target-selection family — CR 109.1 / 601.2c. Enforcement
wired at the declarative target-validation path
(`casting::validate_object_satisfies_requirement` w/ already-threaded
`self_id`) and the trigger auto-target picker (`abilities.rs`,
`trigger.source`). Activated abilities now route through
`validate_targets_with_source` (was `validate_targets`, dead-code retained
behind `#[allow(dead_code)]` for callers without a source). In-scope cards
fixed: Roalesk, Samut Voice of Dissent, Torch Courier, Brash Taunter, Ezuri
Renegade Leader, Oath of Teferi, Elderfang Ritualist, Dour Port-Mage,
Thousand-Faced Shadow.

### OOS-XS-1: "different from other declared target" — Hidden Strings, twincast-style

**Cards**: Hidden Strings ("tap or untap target permanent, then you may tap
or untap another target permanent"). Future cards with "two target X, no two
of which are the same" patterns (e.g. Boros Charm-family multi-target choose-one,
Time Stretch, etc.) hit the same gap.
**Oracle pattern**: A spell or ability with multiple TargetRequirement slots
where slot N must reference a different object than slots 0..N-1. This is
NOT exclude_self (the source isn't a battlefield permanent for sorceries);
it is *inter-target distinctness*.
**Gap**: PB-XS only excludes the source. Hidden Strings would need a new
field like `TargetRequirement::TargetPermanentDistinctFrom(usize)` or a
post-pass after declared-target binding that rejects duplicates among
flagged slot indices. Authorship cost ~30 lines + tests; out of scope here.
**Yield**: 1 confirmed (Hidden Strings) + ~3 speculative future cards.
**Status**: Filed by PB-XS 2026-05-14. Defer until a real multi-target
"another" card crosses the priority threshold.
**References**: `crates/engine/src/cards/defs/hidden_strings.rs`; CR 601.2c.

### OOS-XS-2: TargetFilter.is_attacking enforcement at validate sites

**Cards**: Thousand-Faced Shadow ("create a token that's a copy of another
target attacking creature"). Future "target attacking creature" cards
(Aether Tradewinds family, Naya Charm) hit the same gap.
**Oracle pattern**: A target requirement whose filter constrains the target
to currently-attacking creatures (CombatState.attackers membership).
**Gap**: `TargetFilter.is_attacking` exists but per its doc comment is
checked ONLY by `combat_damage_filter` in abilities.rs — the declarative
target-validation path (`validate_object_satisfies_requirement`) and the
trigger auto-target picker silently ignore it. PB-XS authored Thousand-Faced
Shadow with `is_attacking: true` set on the filter so the card-def reads
correctly, but the engine does not yet enforce it during target validation.
**Mechanical fix**: in `validate_object_satisfies_requirement` (and the
auto-target picker), after `matches_filter` returns true, additionally check
`!filter.is_attacking || state.combat.is_attacking(id)`. Mirror the
`exclude_self` pattern PB-XS just added.
**Yield**: 1 confirmed (Thousand-Faced Shadow). Light primitive (~15 lines).
**Status**: Filed by PB-XS 2026-05-14. Recommend bundling with the next
"target X with runtime predicate" primitive (e.g. is_tapped, is_nontoken
target side). Could ship as PB-XA ("eXclude / Attacking / runtime predicates").
**References**: `crates/engine/src/cards/defs/thousand_faced_shadow.rs`;
`TargetFilter.is_attacking` doc comment in `card_definition.rs:2600`.

### OOS-XS-3: Olivia Voldaren {1}{R} multi-effect activated ability

**Cards**: Olivia Voldaren ("{1}{R}: Olivia Voldaren deals 1 damage to
another target creature. That creature becomes a Vampire in addition to its
other types. Put a +1/+1 counter on Olivia Voldaren.").
**Oracle pattern**: A single activated ability that resolves three distinct
effects (damage to declared target + LayerModification::AddSubtype to that
target + AddCounter on Source). Existing DSL `AbilityDefinition::Activated`
takes a single `effect`; sequencing via `Effect::Sequence` works only if
each child Effect is representable. `AddSubtype` LayerModification does not
exist.
**Gap**: PB-XS added `exclude_self: true` would be required ONCE the
activated ability is authored, but the underlying DSL gap is
`LayerModification::AddSubtype { subtype: SubType }` (Layer 4 type-addition,
CR 613.1d). No card-def edit lands today.
**Yield**: 1 confirmed (Olivia Voldaren). Additional cards: Conspiracy,
Arcane Adaptation, Door of Destinies (all "this creature is also X").
**Status**: Filed by PB-XS 2026-05-14. Belongs to a Layer-4 type-grant
primitive batch alongside the ObjectFilter::OwnedByOpponentsOf sub-gap
already on the roadmap.
**References**: `crates/engine/src/cards/defs/olivia_voldaren.rs`; CR 613.1d.

### OOS-XS-4: Skrelv Defector Mite — ChooseColor + protection-from-color + can't-block-by-color

**Cards**: Skrelv, Defector Mite ("{W/P}, {T}: Choose a color. Another
target creature you control gains toxic 1 and hexproof from that color
until end of turn. It can't be blocked by creatures of that color this
turn.").
**Oracle pattern**: An activated ability whose effect (a) prompts the
controller for a color choice, (b) grants conditional hexproof-from-color
to the target until end of turn, (c) attaches a "can't be blocked by
creatures of that color this turn" combat restriction. PB-XS handles only
the "another target" half via `exclude_self`.
**Gap**: Three orthogonal DSL primitives missing:
  1. `ChooseColor` effect / activation-time prompt with the chosen color
     stored on the resulting continuous effect.
  2. `LayerModification::AddProtectionFromColor(ManaColor)` with continuous-
     effect duration UntilEndOfTurn (CR 702.16, color-keyed protection).
  3. A combat-restriction continuous effect referencing the chosen color
     (CR 509.1b — block restrictions evaluated during DeclareBlockers).
**Yield**: 1 confirmed (Skrelv). Adjacent cards: Mother of Runes (color-keyed
protection), Disenchant variants. Color-choice is a broader primitive.
**Status**: Filed by PB-XS 2026-05-14. High complexity; defer until a
post-alpha protection-from-color primitive batch.
**References**: `crates/engine/src/cards/defs/skrelv_defector_mite.rs`;
CR 702.16, CR 509.1b.

### OOS-XS-5: "Whenever another X enters/dies" trigger-side filter — Metastatic Evangel et al.

**Cards**: Metastatic Evangel ("Whenever another nontoken creature you
control enters, proliferate"). Shadow Alley Denizen, Forerunner of the
Legion, Boggart Shenanigans, Athreos God of Passage, Meren of Clan Nel Toth
all have the "another X enters/dies" trigger-side exclusion pattern.
**Oracle pattern**: A `WheneverCreatureEntersBattlefield` /
`WheneverPermanentEntersBattlefield` / `WheneverCreatureDies` trigger whose
trigger object must NOT be the trigger source itself.
**Gap**: `WheneverCreatureDies.exclude_self` already exists (PB-23). The
sibling `WheneverCreatureEntersBattlefield` and
`WheneverPermanentEntersBattlefield` variants in `TriggerCondition` have
only `filter: Option<TargetFilter>` — no `exclude_self` flag at the trigger
level. The trigger-evaluation site silently fires on the source's own ETB.
Note: this is the TRIGGER-side exclusion (which trigger object fires), NOT
the target-side exclusion (which this PB shipped). Cards currently document
the miss via inline comments (see `metastatic_evangel.rs:18-21`).
**Mechanical fix**: add `exclude_self: bool` to
`TriggerCondition::WheneverCreatureEntersBattlefield` and
`WheneverPermanentEntersBattlefield`; gate the matching trigger evaluation
on `triggering_object_id != trigger.source` when set. Mirror PB-23
(`WheneverCreatureDies.exclude_self`).
**Yield**: 6+ confirmed (Metastatic Evangel, Shadow Alley Denizen,
Forerunner of the Legion, Boggart Shenanigans, Athreos, Meren — and more on
sweep). High-yield primitive.
**Status**: SHIPPED 2026-05-15 (PB-XS-E, scutemob-22). The Enters half
(creature + permanent) landed with `TriggerCondition::*.exclude_self: bool`
+ HASH 19→20 + 17 creature card defs migrated. Boggart Shenanigans /
Athreos / Meren remain dies-side (out of PB-XS-E scope); they use the
existing `WheneverCreatureDies.exclude_self` (PB-23) — re-audit pending to
confirm their card defs use the flag correctly.
**References**: `crates/engine/src/cards/defs/metastatic_evangel.rs`;
`WheneverCreatureDies.exclude_self` precedent at `card_definition.rs:2690`;
CR 603.10a. Shipped impl: `crates/engine/tests/primitive_pb_xs_e.rs`.

### OOS-XS-E-1: Three dies-side cards (Boggart Shenanigans, Athreos, Meren)

**Cards**: Boggart Shenanigans, Athreos God of Passage, Meren of Clan Nel
Toth. Listed in the PB-XS-E seed roster but each uses a "Whenever another
creature [Goblin/you own/you control] dies" trigger, not an Enters trigger.
**Oracle pattern**: `WheneverCreatureDies` with `exclude_self: true` —
already supported by the engine via PB-23 (`DeathTriggerFilter::exclude_self`).
**Gap**: No engine gap. The card defs MAY already be using the field
correctly. A future audit should confirm each of these three sets
`exclude_self: true` on its `WheneverCreatureDies` trigger, and that
behavior matches oracle ("another" semantics).
**Yield**: 3 cards, each potentially already correct or a one-line fix.
**Status**: Filed by PB-XS-E 2026-05-15 as a follow-up sweep. Low priority
(no engine change needed; pure card-def verification).
**References**: PB-23; `crates/engine/src/cards/card_definition.rs:2706`
(`WheneverCreatureDies`).

### OOS-XS-E-2: Self-inclusive ETB-trigger correctness regression sweep

**Cards**: Risen Reef ("Whenever this or another Elemental..."), Ayara
First of Locthwain ("Whenever Ayara or another black creature..."),
Bloomvine Regent ("Whenever this creature or another Dragon..."), Satoru
the Infiltrator ("Whenever Satoru and/or one or more other nontoken
creatures..."), and any non-creature-source cards with simple "Whenever a
creature you control enters" wording (Witty Roastmaster).
**Oracle pattern**: Either explicit self-inclusion ("X or another") or
unrestricted ("a creature enters under your control"). With the old
hardcoded `ETBTriggerFilter.exclude_self = true`, these cards latently
failed to fire on their own ETB. PB-XS-E flips the default to `false`, and
for the 4 self-inclusive cards listed above the new behavior matches oracle.
**Gap**: BEHAVIORAL CORRECTNESS regression-fix that landed silently with
PB-XS-E. Existing scripts in `test-data/generated-scripts/` and CardDef
tests may assert the OLD (buggy) semantics; a sweep should re-run scripts
sensitive to Risen Reef / Ayara / Bloomvine / Satoru self-ETBs and update
assertions where the old "trigger never fires on self" expectation is
encoded.
**Yield**: ~5 cards explicitly self-inclusive; possibly more if scripts
assume the old hardcoded behavior.
**Status**: Filed by PB-XS-E 2026-05-15 as a follow-up regression sweep
(low priority unless a failing script surfaces). The 2775-test workspace
suite passed unchanged after the migration, suggesting no in-tree test
encodes the old assumption — but a generated-script audit is still due.
**References**: `crates/engine/src/cards/defs/risen_reef.rs`,
`ayara_first_of_locthwain.rs`, `bloomvine_regent.rs`,
`satoru_the_infiltrator.rs`; CR 207.2c / CR 603.2.
