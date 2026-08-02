# Primitive Batch Plan: PB-DX19 — the unbounded `calculate_characteristics` recursion

**Generated**: 2026-08-02
**Primitive**: not a DSL widening — a **correctness repair** to two engine arithmetic/read sites.
`Condition::YouControlNOrMoreWithFilter` stops re-entering the layer system, and the ten Layer-7
P/T application sites stop performing unchecked `i32` arithmetic.
**Seeds**: `OOS-SIM2-6` (HIGH — the registry's only HIGH) + `OOS-SIM2-5`
**CR Rules**: 604.2, 613.1 (613.1d, 613.1f, 613.1g), 613.4 (613.4c), 613.8 (613.8a/b/c), 208.1,
208.3, 208.5, 400.7, 608.2h, 611.3a
**Cards affected**: **0 completeness flips.** 1 def source touched (`greymond_avacyns_stalwart.rs`,
comment text inside a `Completeness::inert` string only). 1 def unblocked from a hard process abort
(`indomitable_archangel`, `Complete`, deck-legal).
**Dependencies**: none. Stage 0 (reproduction) is already in the tree.
**Deferred items from prior PBs**: none carried in. This batch **may** close the stack-overflow half
of `OOS-DP3-9` / `OOS-M11-3` — §10 is the experiment that decides it.
**Wire**: **none expected.** No `Effect`, `Command`, `GameEvent`, `Condition` or stored-state
variant is added or changed. Predicted PROTOCOL **33** / HASH **70**, both unmoved — but §9 requires
they be *computed by the gates*, not asserted from this line.

**Authoritative brief**: `memory/primitives/seed-rerank-2026-08-02.md`, "Dispatch briefs → PB-DX19"
(that file's `:761`–`:813`).

---

## §1 Premise verification — every claim re-read against HEAD

This section is the batch's answer to `feedback_verify_cr_before_implement`: the coordinator's
summary is *not* authoritative and was re-derived from source. Two line numbers in the published
brief are off by small amounts; the code they point at is otherwise exactly as described.

### 1.1 The recursion chain

| file:line | what is actually there | brief? |
|---|---|---|
| `crates/engine/src/rules/layers.rs:35` | `pub fn calculate_characteristics(state: &GameState, object_id: ObjectId) -> Option<Characteristics>` | ✅ exact |
| `layers.rs:41` | `let mut chars = obj.characteristics.clone();` — the base-characteristics seed | — |
| `layers.rs:43-47` | `state.continuous_effects.iter().filter(\|e\| is_effect_active(state, e)).collect()` — **no `object_id` is passed to the filter** | ✅ exact (`:46`) |
| `layers.rs:508` | `pub fn is_effect_active(state: &GameState, effect: &ContinuousEffect) -> bool` — signature takes **no object id**, and its own doc comment at `:502-507` says so | ✅ exact |
| `layers.rs:558-567` | `if let Some(ref condition) = effect.condition { … check_static_condition(state, condition, source_id, controller) … }`, the call itself on `:565` | ✅ exact |
| `layers.rs:477` | `#[track_caller] pub fn expect_characteristics(state, object_id) -> Characteristics`, whose body calls `calculate_characteristics` at `:478` | ✅ exact |
| `effects/mod.rs:10212` | `pub fn check_static_condition(state, condition, source, controller) -> bool` | ✅ exact |
| `effects/mod.rs:10238-10270` | the `Condition::YouControlNOrMoreWithFilter { count, filter }` arm | ✅ exact |
| `effects/mod.rs:10247-10258` | the twelve-line `// NOTE:` comment arguing termination | ✅ **coordinator** exact (published brief says `:10245-10256`) |
| `effects/mod.rs:10259` | `let chars = crate::rules::layers::expect_characteristics(state, obj.id);` | ✅ exact |
| `effects/mod.rs:10260` | `matches_filter(&chars, filter)` | ✅ exact |
| `effects/mod.rs:10262` | `&& check_has_counter_type(obj, filter)` | ✅ exact |
| `effects/mod.rs:10265` | `&& (!filter.exclude_self \|\| obj.id != source)` — **evaluated after `:10259`** | ✅ **coordinator** exact (published brief says `:10266`) |

**Correction to record**: the brief cites `effects/mod.rs:9533` for
`matches_filter(&Characteristics, &TargetFilter)`. `:9533` is inside **`check_has_counter_type`**
(declared `:9529`). `matches_filter` is declared at **`effects/mod.rs:9540`**:
`pub fn matches_filter(chars: &Characteristics, filter: &TargetFilter) -> bool`. The *claim* the
citation supports — that `matches_filter` takes `&Characteristics` and is therefore
type-compatible with `&obj.characteristics` — is **true**; only the line number was wrong.

### 1.2 The precedent that made the opposite choice

`layers.rs:2282` `resolve_cda_amount` → `:2290-2317` `EffectAmount::PermanentCount`. The comment at
`:2304-2310` reads, verbatim:

> `// NOTE: We deliberately use base characteristics here (not`
> `// calculate_characteristics) to avoid recursive CDA evaluation.`
> `// CR 604.3: CDA filters typically check card types (Creature, Land)`
> `// or subtypes, which are set in Layers 4-6 (not by other CDAs).`
> `// This avoids an infinite recursion when the CDA creature itself`
> `// is included in the count (e.g., "*/* = creatures you control"`
> `// counts the creature with the CDA).`

and `:2311` is `crate::effects::matches_filter(&obj.characteristics, filter)`. ✅ exact.

**A second, independent in-tree precedent the brief does not name**:
`effects/mod.rs:10336` `calculate_devotion_to_colors` — reached from `check_static_condition`'s
`Condition::DevotionToColorsLessThan` arm (`:10273`), i.e. *on the layer path*, by three `Complete`
god defs (`iroas`, `athreos`, `purphoros`) — reads `obj.characteristics.mana_cost` at `:10356`,
base, no recursion. So the layer path already has **two** sites that made this trade deliberately;
`:10259` is the odd one out, not the norm.

### 1.3 The card, the landmine, the dodging test

| claim | verified |
|---|---|
| `indomitable_archangel.rs` declares no `completeness` field → `Complete` by derive | ✅ file is 47 lines; `..Default::default()` at `:45`; no `completeness:` key |
| it registers `AbilityDefinition::Static` (`:29`) with `ContinuousEffectDef` (`:30`), `layer: EffectLayer::Ability` (`:31`), `LayerModification::AddKeyword(Shroud)` (`:32`), `filter: EffectFilter::ArtifactsYouControl` (`:33`), `condition: Some(Condition::YouControlNOrMoreWithFilter { count: 3, filter: has_card_type: Some(CardType::Artifact) })` (`:35-41`) | ✅ exact |
| `exclude_self` is `false` (it comes from `..Default::default()` at `:39`), so the `:10265` test is a no-op here and is **irrelevant to this defect** | ✅ |
| `greymond_avacyns_stalwart.rs:37-43` — `Completeness::inert(...)` whose note ends *"The +2/+2 conditional static IS now expressible (Condition::YouControlNOrMoreWithFilter + ContinuousEffectDef.condition) and should be wired."* | ✅ exact (brief said `:38-43`; the `completeness:` key opens on `:37`) |
| `crates/engine/tests/rules/static_grants.rs:711-761` `test_artifacts_you_control_grants_shroud` names Indomitable Archangel, builds it with `ObjectSpec::creature(p1(), "Indomitable Archangel", 4, 4)` at `:716` (synthetic, **no `card_id`**), and hand-pushes a `ContinuousEffect` with `condition: None` at `:738` | ✅ exact (brief said `:707-760` / `:736`; the `#[test]` attribute is `:711`, the fn `:712`, `condition: None` is `:738`) |

### 1.4 The ten P/T sites

`grep -n '\*p += \|\*t += ' crates/engine/src/rules/layers.rs` returns exactly ten, and there are
**no** `-=` sites. Re-read individually:

| line | arm | expression |
|---|---|---|
| `:394` | `+1/+1` / `-1/-1` counter path | `*p += net` |
| `:397` | same | `*t += net` |
| `:1658` | `LayerModification::ModifyPower(delta)` | `*p += delta` |
| `:1663` | `ModifyToughness(delta)` | `*t += delta` |
| `:1668` | `ModifyBoth(delta)` | `*p += delta` |
| `:1671` | `ModifyBoth(delta)` | `*t += delta` |
| `:1698` | `ModifyBothDynamic` | `*p += delta` |
| `:1701` | `ModifyBothDynamic` | `*t += delta` |
| `:1715` | `ModifyPowerDynamic` | `*p += delta` |
| `:1729` | `ModifyToughnessDynamic` | `*t += delta` |

Supporting arithmetic re-read: `:385` / `:389` are `…counters.get(…).copied().unwrap_or(0) as i32`
over an **`OrdMap<CounterType, u32>`** (`crates/card-types/src/state/game_object.rs:1013`); `:391`
is `let net = plus_ones - minus_ones;`; `:1696`, `:1713`, `:1727` are
`let delta = if *negate { -raw } else { raw };`. **The `as i32` casts at `:385`/`:389` are a hole
`saturating_add` does not close** — see §8.1.

### 1.5 Profiles and substitution

| claim | verified |
|---|---|
| `Cargo.toml:51-54` `[profile.fuzz] inherits = "release" / debug-assertions = true / overflow-checks = true` | ✅ exact |
| there is **no** `[profile.dev]` or `[profile.test]` block in the workspace root `Cargo.toml` (the file is 55 lines; `[profile.fuzz]` is its only profile) | ✅ — so the **cargo defaults** apply: dev has `debug-assertions = true`, `overflow-checks = true`, and `test` inherits dev. **`cargo test` therefore panics on `i32` overflow today.** This is load-bearing for §8's test design. |
| `effects/mod.rs:3892-3907` substitutes `ModifyBothDynamic`/`ModifyPowerDynamic`/`ModifyToughnessDynamic` → concrete `ModifyBoth(v)`/`ModifyPower(v)`/`ModifyToughness(v)` at resolution (CR 608.2h), with the path-semantics note at `:3880-3891` | ✅ exact (brief said `:3897-3907`; the `match` opens `:3892`) |
| `devilish_valet.rs` is `Complete` by derive and its Alliance trigger applies `ModifyPowerDynamic { amount: PowerOf(Source), negate: false }`, `duration: UntilEndOfTurn`, `filter: Source` (`:37-48`) — so each trigger stacks a **new concrete** `ModifyPower(current_power)`: 1 → 2 → 4 → 8 … | ✅ exact |

### 1.6 Stage 0 is already in the tree

`crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs` exists (238 lines) and is
registered at `crates/engine/tests/primitives/main.rs:32` (SR-9a satisfied — **no new
`mod` line and no new top-level `tests/*.rs` is needed by this batch**). It builds the Archangel
from `all_cards()` with `ObjectSpec::card(..).with_card_id(def.card_id.clone())` +
`enrich_spec_from_def` and registers statics through the production registrar
`mtg_engine::rules::replacement::register_static_continuous_effects`. Three probes:
`recursion_metalcraft_on_grants_shroud_and_terminates`,
`recursion_metalcraft_off_still_terminates`,
`recursion_is_independent_of_the_object_being_calculated`.

Pre-fix observation (verbatim, recorded in the file's doc comments): `running 3 tests` → one test
named → `has overflowed its stack` → `fatal runtime error: stack overflow, aborting` →
`signal: 6, SIGABRT`. **SIGABRT is not `catch_unwind`-able.**

---

## §2 The defect, mechanically

### 2.1 The cycle

```
layers.rs:35   calculate_characteristics(state, ANY object_id)
  → layers.rs:46   .filter(|e| is_effect_active(state, e))      ← for EVERY continuous effect
    → layers.rs:565  check_static_condition(state, cond, source, controller)
      → effects/mod.rs:10259  expect_characteristics(state, candidate.id)
        → layers.rs:478  calculate_characteristics(state, candidate.id)   ← back to the top
```

Four hops, no depth counter, no memo, no visited set.

### 2.2 Why it is *unconditional*, not conditional

`calculate_characteristics` does not filter continuous effects by the object it is calculating. It
filters by *activity* — `is_effect_active` (`layers.rs:508`) takes `(state, effect)` and no object
id at all; the doc comment at `:502-507` says explicitly that a per-object question is not
expressible there. `effect_applies_to` is the per-object question, and it runs **later**, on the
already-collected list.

Consequence: as long as one conditional effect with this `Condition` variant exists anywhere in
`state.continuous_effects`, **every** `calculate_characteristics` call in the game — on any object,
in any zone, for any purpose — re-enters `check_static_condition` for it, which re-enters
`calculate_characteristics` for up to `|battlefield|` candidates, each of which does the same. The
recursion is not merely infinite; its branching factor is the size of the battlefield.

The stage-0 probe `recursion_is_independent_of_the_object_being_calculated` is the direct
falsification: it calculates the **Archangel's own** characteristics — an object the granting
filter (`ArtifactsYouControl`) never matches — and crashed identically.

### 2.3 The old comment, sentence by sentence

Correcting this text is an acceptance criterion, so each sentence is adjudicated:

> "This is re-entrant but safe: `im-rs` persistent data structures are immutable, so there is no
> risk of observing partial mutations."

**True but irrelevant.** Immutability rules out *data races and torn reads*. It says nothing about
*termination*. The comment lets a true statement about memory safety stand in for an unmade
argument about recursion depth — that is the rhetorical move that hid the bug.

> "Termination is guaranteed because we are checking the types of *other* battlefield objects, not
> the object currently being calculated — there is no direct self-referential cycle."

**False on both clauses.** (a) The candidate set is `state.objects.values()` filtered on zone,
phase and controller — the object currently being calculated is *not* excluded, and the source of
the effect is not excluded either (`exclude_self` is tested at `:10265`, *after* the recursive call
at `:10259`, and is `false` on the one live card). (b) Even if the current object were excluded,
"no *direct* self-referential cycle" is the wrong invariant: the cycle here is
`calculate_characteristics(A) → … → calculate_characteristics(B) → … → calculate_characteristics(A)`,
which is indirect and unbounded. What actually terminates a layer computation is *not re-entering
the layer system at all*, which is the invariant `resolve_cda_amount` chose and this site did not.

> "If performance becomes an issue, consider using base characteristics (`obj.characteristics`) for
> the filter check instead of calling `calculate_characteristics` again."

**The correct fix, filed under the wrong severity.** This sentence describes, precisely, the repair
this batch ships — and calls it a performance nicety. A reader triaging by severity skips it. This
is why the defect survived ~4.5 months in a `Complete`, deck-legal card. The rewrite in §5 must
therefore not merely add the constraint; it must remove the invitation to read it as optional.

### 2.4 Blast radius

`indomitable_archangel` is `Complete`, so `validate_deck` accepts it and the simulator's
`random_deck` (`crates/simulator/src/deck.rs:58-71`, `Complete`-only per SR-12) will draw it into
any W-identity seat's pool. Reaching the battlefield is sufficient — no interaction, no opponent
action, no specific board is required. The crash is SIGABRT, so the play-server's per-request
`catch_unwind` boundary cannot contain it: the process dies and the 4-player game with it.

---

## §3 The fix decision, argued

Two candidates.

### 3.1 Option A — base characteristics at `:10259` (**recommended, ships in this batch**)

Read `&obj.characteristics` instead of calling `expect_characteristics`. Terminates trivially: the
condition evaluation never re-enters the layer system, so `calculate_characteristics` has no
recursive edge at all. One line. Matches `layers.rs:2311` and `effects/mod.rs:10356` (§1.2).

### 3.2 Option B — a CR 613.8-shaped dependency-aware fixpoint

CR supplies the machinery. **CR 613.8** (full text, MCP):

> 613.8. Within a layer or sublayer, determining which order effects are applied in is sometimes
> done using a dependency system. If a dependency exists, it will override the timestamp system.
>
> **613.8a** An effect is said to "depend on" another if (a) it's applied in the same layer (and, if
> applicable, sublayer) as the other effect; (b) applying the other would change the text or the
> existence of the first effect, what it applies to, or what it does to any of the things it applies
> to; and (c) neither effect is from a characteristic-defining ability or both effects are from
> characteristic-defining abilities. Otherwise, the effect is considered to be independent of the
> other effect.
>
> **613.8b** An effect dependent on one or more other effects waits to apply until just after all of
> those effects have been applied. If multiple dependent effects would apply simultaneously in this
> way, they're applied in timestamp order relative to each other. **If several dependent effects
> form a dependency loop, then this rule is ignored and the effects in the dependency loop are
> applied in timestamp order.**
>
> **613.8c** After each effect is applied, the order of remaining effects is reevaluated and may
> change if an effect that has not yet been applied becomes dependent on or independent of one or
> more other effects that have not yet been applied.

613.8b's last sentence is the termination rule this site lacks, and the engine already has 613.8
machinery: `layers.rs:1747` `resolve_layer_order` → `:1764` `toposort_with_timestamp_fallback`. But
that machinery orders effects *within one layer of one object's computation*. Making condition
evaluation dependency-aware means lifting the whole computation to a **global fixpoint over all
permanents simultaneously**, with 613.8c's re-evaluation after each application. That is a
rewrite of `calculate_characteristics`'s contract (it currently answers one object at a time, with
`Option` semantics tied to a single id), a rewrite of every one of its ~hundreds of call sites'
cost model, and its own PB. **Not this batch.**

### 3.3 What Option A costs — stated plainly, with a live example

Under CR 604.2 the condition is evaluated against the *current game state*, and CR 613.1d puts
type-changing effects in Layer 4. So CR wants Metalcraft to count a permanent that is an artifact
**after** layers. Option A counts it only if it is an artifact **on the printed card / at
creation**. Concretely:

| interaction | CR answer | Option A answer | live in corpus? |
|---|---|---|---|
| A land animated into an **artifact creature** (`blinkmoth_nexus.rs:42-43`, `inkmoth_nexus.rs:43-44`: Layer-4 `AddCardTypes([Artifact, Creature])`) — is it Metalcraft fuel? | **yes** | **no** (base type is Land) | **YES — both defs are `Complete` by derive, both are colorless, so both fit *any* commander's identity and sit in the same W pool as the Archangel** |
| An artifact **animated into a creature** (artifact→artifact creature) — still Metalcraft fuel? | yes | **yes** (base type still includes Artifact; Layer 4 *added* Creature, it did not remove Artifact) | n/a — **unaffected**, and worth stating because it is the case one instinctively worries about |
| A permanent whose **artifact type is removed** by a Layer-4 effect (e.g. an Iroas/Athreos-shaped `RemoveCardTypes`) — still fuel? | no | **yes** (false positive) | no such def targets Artifact today |
| A Layer-1 copy effect that makes a nonartifact into an artifact after creation | yes | no | no corpus def |

So **Option A ships a known-wrong answer on a two-def, deck-legal, in-corpus interaction**
(Blinkmoth/Inkmoth Nexus animated alongside Indomitable Archangel). That is the honest price. It is
worth paying today, because the alternative is not "the correct answer" — it is **a process abort
whenever the Archangel is on the battlefield at all**, which is strictly worse than a wrong count in
a rare interaction, and because the in-tree precedent (`layers.rs:2304-2310`) already accepted this
exact trade for this exact hazard on a *more* common code path (every `*/*` CDA in the corpus).

**Recorded deviation, not silent.** The comment rewrite (§5) names it, and §12 files
`OOS-DX19-2` for the CR-honest version with the Nexus example attached so the successor batch has a
discriminating test already specified.

---

## §4 Scope of the fix

### 4.1 The edit — `crates/engine/src/effects/mod.rs`, `Condition::YouControlNOrMoreWithFilter` arm

Current `:10246-10266` (comment elided; see §5 for the replacement text):

```rust
                        && {
                            // ← the twelve-line NOTE at :10247-10258, replaced per §5
                            let chars = crate::rules::layers::expect_characteristics(state, obj.id);
                            matches_filter(&chars, filter)
                                // CR 122.1: counter check must be against GameObject (not Characteristics).
                                && check_has_counter_type(obj, filter)
                                // CR 109.1: "you control another [permanent]" excludes the
                                // source (PB-EF1, marker EF-5).
                                && (!filter.exclude_self || obj.id != source)
                        }
```

Target:

```rust
                        && {
                            // ← the replacement NOTE from §5
                            //
                            // CR 109.1: "you control another [permanent]" excludes the source
                            // (PB-EF1, marker EF-5). Tested FIRST — see §4.2 of the PB-DX19 plan.
                            (!filter.exclude_self || obj.id != source)
                                && matches_filter(&obj.characteristics, filter)
                                // CR 122.1: counter check must be against GameObject
                                // (not Characteristics).
                                && check_has_counter_type(obj, filter)
                        }
```

Type-compatibility is established: `matches_filter(chars: &Characteristics, filter: &TargetFilter)`
(`effects/mod.rs:9540`) and `GameObject.characteristics: Characteristics`. `check_has_counter_type`
(`:9529`) already takes `&GameObject` and is unchanged. `obj` in the closure is `&&GameObject`;
field access auto-derefs. **No clone** — the old code allocated an owned `Characteristics` per
candidate per call; the new code borrows.

### 4.2 Should `exclude_self` be reordered? — argued

**Yes, reorder it first — but do not sell it as a safety measure.**

*For*: all three predicates are pure (no interior mutability; `im-rs` throughout), so reordering is
behaviour-preserving by construction. `obj.id != source` is a single integer compare; putting it
first makes the cheap identity test gate the two content tests, and it makes the code read in the
order a rules lawyer reads CR 109.1 ("*another* permanent you control" — establish which objects are
candidates, *then* ask what they are). It also removes the specific structural trap that made this
bug possible to write: today an author reading top-to-bottom meets an expensive resolution call
before meeting the exclusion, and the old comment's "we are checking *other* objects" claim reads as
if the exclusion had already happened. After the reorder, that misreading is not available.

*Against / the honest limit*: it fixes **nothing**. If someone reintroduces a layer-resolved read
here, the recursion returns in full, because the recursive edge runs through the *other* candidates
(≥ 1 in every case that matters) and not through the source. `exclude_self` is `false` on the only
live card, so the reorder would not have prevented this defect and must not be described as a second
line of defence anywhere in the code or the commit message.

*Verdict*: take it, on legibility grounds only, and say exactly that in the comment.

`check_has_counter_type` stays last: it reads `obj.counters` (a `GameObject` field), is already
correct per CR 122.1, and its position among the two content tests is arbitrary.

### 4.3 The sibling-site class — **scope decision: fix `:10259` only; file the class**

`check_static_condition`'s catch-all `_` arm (`effects/mod.rs:10283`) builds a minimal
`EffectContext` and delegates to `check_condition` (`effects/mod.rs:9662`). `check_condition`'s body
contains **ten** further `expect_characteristics` call sites, every one the same shape:

| line | `Condition` variant |
|---|---|
| `:9682` | `YouControlPermanent(filter)` |
| `:9693` | `OpponentControlsPermanent(filter)` |
| `:9791` | `ControlLandWithSubtypes(subtypes)` |
| `:9807` | `ControlAtMostNOtherLands(n)` |
| `:9846` | `ControlBasicLandsAtLeast(n)` |
| `:9867` | `ControlAtLeastNOtherLands(n)` |
| `:9885` | `ControlAtLeastNOtherLandsWithSubtype { count, subtype }` |
| `:9898` | `ControlLegendaryCreature` |
| `:9909` | `ControlCreatureWithSubtype(subtype)` |
| `:10077` | `OpponentControlsMoreLandsThanYou` (twice per call, via the `count_lands` closure) |

Each is reachable from the layer path **iff** its variant appears as a `ContinuousEffectDef.condition`.
`Condition::Not` (`:9768`) and `Condition::Or` (`:9777`) recurse into `check_condition`, so a
wrapped variant is reachable too.

**Liveness measurement (this is the gate the default scope decision turns on).** Enumerated by
grepping every `condition: Some(Condition::…)` occurrence in `crates/card-defs/src/defs/` (97
occurrences) and classifying each by its *field position*:

- The land family (`ControlLandWithSubtypes`, `ControlAtLeast/AtMostNOtherLands`,
  `ControlBasicLandsAtLeast`, `ControlAtLeastNOtherLandsWithSubtype`, `CanRevealFromHandWithSubtype`,
  `HaveTwoOrMoreOpponents`, and the `Or`/`Not` wrappers in `temple_of_the_dragon_queen` /
  `den_of_the_bugbear`) all sit in `unless_condition` on `AbilityDefinition::Replacement` — the
  **CR 614.1c ETB-tapped** path, not `is_effect_active`.
- `mox_jasper.rs:22` (`YouControlPermanent`), `mox_opal.rs:29`, `inventors_fair.rs:94`,
  `bloodline_keeper.rs:69` (`YouControlNOrMoreWithFilter`) are all **`activation_condition`** on
  `AbilityDefinition::Activated` — not the layer path.
- `minas_tirith.rs:22` (`ControlLegendaryCreature`) is **`unless_condition`** — ETB-tapped.
- Every def that *does* put a condition on a `ContinuousEffectDef` uses a non-recursive variant:
  `IsYourTurn`, `SourceIsUntapped`, `SourceHasCounters`, `ControllerLifeAtLeast`,
  `OpponentLifeAtMost`, `CompletedADungeon`, `YouAttackedThisTurn`, `HasCitysBlessing`,
  `CreatedATokenThisTurn`, `OpponentHasPoisonCounters`, `CardTypesInGraveyardAtLeast`,
  `YouControlYourCommander` (`effects/mod.rs:10099` — reads `card_id`, no characteristics),
  `DevotionToColorsLessThan` (`:10273` → `:10336`, base characteristics), **and
  `YouControlNOrMoreWithFilter` on `indomitable_archangel` alone.**
- No engine source registers a `ContinuousEffect` with one of the ten variants
  (`condition: Some(Condition::` outside `defs/` occurs only in `crates/engine/tests/`, all
  non-recursive variants).

**Result: `indomitable_archangel` is the only live instance in the entire tree.** The ten siblings
are **latent**. Therefore the default scope decision from the dispatch holds unchanged:

> **Fix `:10259` in this batch. File the ten siblings as `OOS-DX19-1` with this measurement
> attached.** Do **not** convert them to base characteristics in this batch — several of them
> (`ControlLandWithSubtypes` in particular) carry explicit `// CR 613.1d: Use layer-resolved …`
> comments naming Blood Moon as the reason, and on their *actual* call path (ETB replacement,
> `Effect::Conditional`) the layer-resolved read is **correct** and must not be downgraded. The
> fix for the class is a guard at the boundary (§12, `OOS-DX19-1`), not sixteen edits at the leaves.

---

## §5 The comment rewrite

Replaces `effects/mod.rs:10247-10258` in full. Constraints it satisfies: states the real invariant;
cites CR 613.8b and the `layers.rs` precedent; names the deviation; contains no "other objects"
argument and no framing of the fix as a performance concern.

```rust
                            // INVARIANT — do not resolve characteristics here.
                            //
                            // This closure runs on the layer path:
                            // `calculate_characteristics` -> `is_effect_active` (layers.rs:508)
                            // -> `check_static_condition` -> here. `is_effect_active` takes no
                            // object id and is applied to EVERY entry in
                            // `state.continuous_effects` on EVERY `calculate_characteristics`
                            // call, so any call that resolves an object's characteristics from
                            // inside this closure re-enters the layer system unconditionally --
                            // for every candidate permanent, on every call, regardless of which
                            // object the outer call was made for. There is no exit condition and
                            // no depth guard; it is a stack overflow (SIGABRT, not an unwindable
                            // panic), and it shipped that way for 4.5 months behind an
                            // `expect_characteristics` call at this line. See PB-DX19 /
                            // OOS-SIM2-6 and the probes in
                            // `tests/primitives/pb_dx19_characteristics_recursion.rs`.
                            //
                            // The invariant is therefore: BASE characteristics only, no
                            // re-entry. Same choice, same reason, as
                            // `layers.rs::resolve_cda_amount`'s `EffectAmount::PermanentCount`
                            // arm and `calculate_devotion_to_colors`, which are the other two
                            // sites that read permanents from the layer path.
                            //
                            // DOCUMENTED DEVIATION (CR 604.2 / CR 613.1d). CR evaluates a
                            // conditional static's condition against the current game state,
                            // which includes Layer 1-6 effects. Reading base characteristics
                            // means a permanent whose matching type is GRANTED by a Layer-4
                            // effect is not counted -- an animated Blinkmoth/Inkmoth Nexus
                            // (`AddCardTypes([Artifact, Creature])`) does not feed Indomitable
                            // Archangel's Metalcraft, though CR says it should -- and a
                            // permanent whose matching type is REMOVED is still counted. CR
                            // 613.8b supplies the termination rule a CR-honest implementation
                            // would need ("if several dependent effects form a dependency loop,
                            // ... the effects in the dependency loop are applied in timestamp
                            // order"), but applying it here means lifting characteristics
                            // calculation to a global fixpoint over all permanents with CR
                            // 613.8c re-evaluation, which is a batch of its own: OOS-DX19-2.
                            // Until then this deviation is the accepted price of terminating.
```

---

## §6 `greymond_avacyns_stalwart.rs` disposition

**The landmine**: the current `Completeness::inert` note (`:37-43`) ends *"The +2/+2 conditional
static IS now expressible (`Condition::YouControlNOrMoreWithFilter` + `ContinuousEffectDef.condition`)
and should be wired."* A future author following that instruction authors a **second** instance of
the crashing shape. Pre-fix that is a new process abort; post-fix it is a second instance of the
documented deviation, silently, with no note that a deviation exists.

**Hard constraint on the edit (see §11)**: change **only** the string passed to
`Completeness::inert`. Leave the four `// TODO:` comment lines (`:6`, `:7`, `:34`, `:35`) and every
other line **byte-identical**. Rationale is measured, not assumed:
`tools/authoring-report.py` collects lines matching `TODO_LINE_RE` (`:176-177`) and feeds each to
`classify_todo` (`:235-239`, first-match-wins over `TODO_BUCKETS` at `:41+`), producing the
`total_todos` count and the `todo_classes` histogram in `docs/authoring-status.md`. The
`completeness:` note string is **never** parsed by the report — `MARKER_RE` (`:152`) captures only
the variant name, and `marker_disagreements` (`:197-214`) only asks *whether* a TODO comment exists,
not what it says. So a note-text-only edit provably moves nothing; a TODO-line edit provably moves
two report numbers.

**Draft replacement note** (a drop-in for `:38-42`; keeps the marker `inert`, keeps the leading
`Blocked:` convention):

```rust
        completeness: Completeness::inert(
            "Blocked: 'As this enters, choose two abilities from among first strike, vigilance, \
             and lifelink' — no as-enters ability-choice replacement and no layer grant keyed to \
             a chosen ability set. The +2/+2 conditional static is expressible in shape \
             (Condition::YouControlNOrMoreWithFilter + ContinuousEffectDef.condition, the \
             Indomitable Archangel pattern) and no longer crashes — PB-DX19 removed the \
             unbounded characteristics recursion that shape used to trigger (OOS-SIM2-6). Wiring \
             it would inherit PB-DX19's documented deviation: the count reads BASE \
             characteristics, so a Human created by a Layer-4 type-changing effect is not \
             counted (see the invariant comment on Condition::YouControlNOrMoreWithFilter in \
             effects/mod.rs, and OOS-DX19-2). Whoever wires it must add the deviation to this \
             note; do not resolve characteristics inside the condition. This card stays inert \
             regardless: the as-enters half is still unexpressible.",
        ),
```

Note that this text contains the word "deviation", which is a `DEVIATION_NEEDLES` needle in
`crates/engine/tests/core/completeness_deviation_scan.rs:52` — harmless here, because that gate
(`deviation_language_requires_a_marker_or_allowlist`, `:235`) is satisfied by the presence of
`Completeness::inert`, which is a `MARKER_FRAGMENTS` entry (`:63`). Verified, not assumed.

---

## §7 `static_grants.rs` repair

**File**: `crates/engine/tests/rules/static_grants.rs`, `test_artifacts_you_control_grants_shroud`
(`:711-761`).

**What is wrong**: it names Indomitable Archangel and then tests something the card cannot do. The
object is `ObjectSpec::creature(p1(), "Indomitable Archangel", 4, 4)` (`:716`) — a synthetic
permanent with a matching `name` and **no `card_id`**, so no registrar can find its def. The
`ContinuousEffect` is hand-pushed at `:728-739` with `condition: None` (`:738`). The test therefore
exercises `EffectFilter::ArtifactsYouControl` and **never the Metalcraft condition** — which is
exactly the surface that crashes. It is a test that names the card as cover for not using it.

**Repair**, keeping all three original assertions and adding the Metalcraft-off case:

1. Build the object from the **real def**, using the pattern the stage-0 file already proves works
   (`pb_dx19_characteristics_recursion.rs:79-92`):
   `ObjectSpec::card(p1(), "Indomitable Archangel").in_zone(ZoneId::Battlefield).with_card_id(def.card_id.clone())`
   then `enrich_spec_from_def(base, &defs)`, where `defs: HashMap<String, CardDefinition>` comes
   from `all_cards()`.
2. Register through the **production** registrar, not `continuous_effects_mut().push_back`:
   `mtg_engine::rules::replacement::register_static_continuous_effects(&mut state, angel_id, Some(&card_id), &CardRegistry::new(all_cards()), false)`.
   Delete the hand-built `ContinuousEffect` block (`:728-739`) entirely, along with the now-unused
   `ContinuousEffect` / `EffectId` / `EffectDuration` / `LayerModification` imports **only if** no
   other test in the file uses them (it does — several do; check before touching the `use` at
   `:12-16`, and let `cargo clippy -- -D warnings` arbitrate rather than guessing).
3. **Board**: three P1 artifacts, so Metalcraft is ON. The original test had one P1 artifact; the
   original three assertions are preserved by naming the first of the three `"P1 Artifact"` and
   leaving the P1 creature and P2 artifact as-is.
4. **Assertions — the original three, unchanged in meaning:**
   - P1's artifact **has** Shroud (CR 604.2 + CR 613.1f).
   - P1's creature does **not** (not an artifact — filter axis).
   - P2's artifact does **not** (different controller — controller axis).
5. **New fourth case, `test_artifacts_you_control_no_shroud_below_metalcraft`** (separate `#[test]`,
   so a failure names which axis broke): same construction with **two** P1 artifacts; assert P1's
   artifact does **not** have Shroud. This is the assertion the original file could not make,
   because with `condition: None` there is no threshold to be below.

**Discrimination discipline (non-negotiable)**: both repaired tests must be **watched failing**
against a reverted tree before the batch closes — revert the `:10259` edit, confirm the revert
**compiles** (the S8 `{X}` lesson: PB-DX6's first revert did not, and a non-compiling revert proves
nothing), run, record the verbatim output at the test as a doc comment, restore. Expected pre-fix
symptom for both: the same stack-overflow SIGABRT stage 0 recorded, which takes the whole `rules`
binary down — note that in the doc comment, because a reviewer seeing "1 test named, N reported"
needs to know it is the defect and not a harness fault.

---

## §8 `OOS-SIM2-5` fold-in — the P/T arithmetic

**File**: `crates/engine/src/rules/layers.rs`, all sites. Sixteen edits in three groups.

### 8.1 Group A — the counter path (`:381-399`)

Exercised by every game with a `+1/+1` counter, i.e. nearly all of them.

| line | now | target | why |
|---|---|---|---|
| `:385` | `…copied().unwrap_or(0) as i32` | `i32::try_from(…copied().unwrap_or(0)).unwrap_or(i32::MAX)` | **`counters` is `OrdMap<CounterType, u32>`** (`card-types/src/state/game_object.rs:1013`). A `u32 as i32` cast **wraps silently even under `overflow-checks`** — casts are not checked arithmetic in Rust. A count above `2^31-1` becomes **negative power**, and `saturating_add` downstream cannot undo it. This is a hole the seed does not name and `saturating_*` alone does not close. |
| `:389` | same, `MinusOneMinusOne` | same | same |
| `:391` | `let net = plus_ones - minus_ones;` | `let net = plus_ones.saturating_sub(minus_ones);` | after the clamp both operands are in `[0, i32::MAX]`, so this cannot overflow *today* — take `saturating_sub` anyway so the clamp above is not load-bearing for a second property, and so a future change to either operand's provenance is safe. **This is the only genuine subtraction in the ten sites' neighbourhood.** |
| `:394` | `*p += net` | `*p = p.saturating_add(net)` | |
| `:397` | `*t += net` | `*t = t.saturating_add(net)` | |

### 8.2 Group B — the concrete `Modify*` arms (`:1656-1673`)

Reached both from direct static registration and from the CR 608.2h substitution
(`effects/mod.rs:3892-3907`), so this is where `devilish_valet`'s doubling actually lands.

`:1658`, `:1663`, `:1668`, `:1671`: `*x += delta` → `*x = x.saturating_add(*delta)`.
No subtraction; `delta` is already signed and arrives pre-negated.

### 8.3 Group C — the `*Dynamic` arms (`:1689-1731`)

`:1698`, `:1701`, `:1715`, `:1729`: `*x += delta` → `*x = x.saturating_add(delta)`.

Plus the three negations at `:1696`, `:1713`, `:1727`:
`let delta = if *negate { -raw } else { raw };` → `…{ raw.saturating_neg() }`.
`-i32::MIN` panics in debug and wraps in release. `raw` comes from `resolve_cda_amount`, which today
returns bounded counts, so this is defensive — take it, it is free, and `resolve_cda_amount`'s
return provenance is not a stable contract.

### 8.4 The documented deviation and where it goes

Nothing in CR bounds power or toughness. **CR 208.1** defines them as numbers modifiable by effects;
**CR 208.3/208.5** cover absence (treated as 0, and 208.5: "If a creature somehow has no value for
its power, its power is 0"); **CR 613.4c** orders modifications without bounding them. So clamping
at `i32::MAX` / `i32::MIN` is an **engine deviation from CR**, not a CR rule.

**Where to record it: `memory/decisions.md`.** Justified against the alternative:
`docs/engine-invariants.md` is the reference for the **machine-enforced SR gates** (SR-2/3/4/5/6/7/
8/9a/9b/9c/35/36/37) — every section there describes a gate that *fails a build*. This deviation has
no gate and cannot cheaply have one (there is no way to distinguish a saturated `i32::MAX` from a
legitimately-computed one without carrying a flag). Filing an ungated item into a gates document
dilutes exactly the property that makes that document useful. `memory/decisions.md` is the register
for "we chose X over Y and here is why", which is what this is.

**Additionally**, put a short form of the deviation in a doc comment on the counter path
(`layers.rs`, above `:381`) and on the Layer 7c arms, because the code is what a future reader has
in hand. Cross-reference the `decisions.md` entry by name.

### 8.5 Tests — one discriminating test per group, plus the card vehicle

**They discriminate because the dev/test profile has `overflow-checks = true`** — verified in §1.5:
the workspace `Cargo.toml` contains **no `[profile.dev]` or `[profile.test]` block**, so cargo's
defaults apply (`debug-assertions = true`, `overflow-checks = true` for dev; `test` inherits dev).
Therefore each test below **panics with `attempt to add with overflow` pre-fix** and **returns the
saturated value post-fix**. No `--release` run, and no `--profile fuzz` run, is needed to make them
discriminate. Each must still be watched failing by revert, per §7's discipline.

All four go in `crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs` (already
registered at `primitives/main.rs:32`; **no `main.rs` edit** — SR-9a).

| test | group | construction | assertion |
|---|---|---|---|
| `pt_counter_path_saturates_instead_of_overflowing` | A | creature with `power: i32::MAX`, then `u32::MAX` `+1/+1` counters on it (exercises **both** the `as i32` cast at `:385` and the add at `:394`) | `chars.power == Some(i32::MAX)`; and a second object with `power: i32::MIN` + `u32::MAX` `-1/-1` counters asserts `Some(i32::MIN)` |
| `pt_modify_arms_saturate_instead_of_overflowing` | B | creature with `power`/`toughness` `= i32::MAX`, one hand-registered `ContinuousEffect` with `LayerModification::ModifyBoth(5)` at `EffectLayer::PtModify` | `power == Some(i32::MAX)` and `toughness == Some(i32::MAX)` |
| `pt_dynamic_arms_saturate_instead_of_overflowing` | C | creature with `power = i32::MAX`, one `ModifyPowerDynamic { amount: EffectAmount::Fixed(5), negate: false }` registered with `is_cda: true` so it reaches `:1715` live rather than being substituted away | `power == Some(i32::MAX)` |
| `devilish_valet_doubling_saturates` | B (integration) | the **real** `devilish_valet` def via the §7 real-def spec pattern; drive ~32 Alliance triggers by entering creatures under P1, each substituting (`effects/mod.rs:3898-3902`) to a concrete `ModifyPower(current_power)`: 1 → 2 → 4 → … | `power == Some(i32::MAX)` at the end, never negative. **This is the card-level proof that the seed's "silently wraps to negative power" claim was real.** |

Recording note the brief demands: any artefact quoted in the review must say **which profile it came
from**, because a `--release` fuzz artefact wraps silently to negative power while a `--profile fuzz`
artefact panics.

---

## §9 Wire and gates — computed, never predicted

**Prediction (pre-committed here so a disagreement is a stop signal, per the queue's ordering rule):
PROTOCOL 33 unmoved, HASH 70 unmoved.** No enum variant, struct field, serialized shape or stored
state is added or changed by any edit in §4, §5, §6, §7 or §8.

Gate test names, read from source (`crates/engine/tests/core/`), not guessed:

| gate | file:line | test fn |
|---|---|---|
| PROTOCOL fingerprint | `core/protocol_schema.rs:846` | `protocol_schema_fingerprint_is_pinned` |
| PROTOCOL version sentinel | `core/protocol_schema.rs:872` | `protocol_version_sentinel` |
| PROTOCOL closure non-vacuity | `core/protocol_schema.rs:788` | `protocol_closure_is_not_vacuous_and_is_bounded` |
| PROTOCOL history tail | `core/protocol_schema.rs:980` | `history_tail_matches_the_fingerprint_const` |
| HASH declaration fingerprint | `core/hash_schema.rs:1086` | `declaration_fingerprint_is_pinned` |
| HASH stream fingerprint | `core/hash_schema.rs:1111` | `stream_fingerprint_is_pinned` |
| HASH version sentinel | `core/hash_schema.rs:1212` | `hash_schema_version_sentinel` |

Commands (run from the worktree root; `cargo` is `~/.cargo/bin/cargo`):

```bash
~/.cargo/bin/cargo test -p mtg-engine --test core protocol_schema -- --nocapture \
  2>&1 | tee /tmp/dx19-protocol-gate.txt
~/.cargo/bin/cargo test -p mtg-engine --test core hash_schema -- --nocapture \
  2>&1 | tee /tmp/dx19-hash-gate.txt
```

If either fingerprint gate fails, the failure message **prints the computed value**. Take the value
from that output. **Do not edit a pin to make a gate pass** — a disagreement with the prediction
above means stop and re-read what changed, because nothing in this plan should move either
fingerprint.

The two in-tree HASH sentinels that must stay consistent (found by symbol, not by grepping for the
number): `crates/engine/tests/primitives/pbp_power_of_sacrificed_creature.rs:797` and
`crates/engine/tests/casting/optional_cost_and_counter_tax.rs:1139`, both asserting `70u8`. If HASH
is unmoved they need no edit; re-confirm by execution, not by reading.

Full run, **captured to a file, never piped to `tail`** (2026-08-02 incident: a `| tail` hid a
compile failure and faked a green run):

```bash
~/.cargo/bin/cargo test --workspace --no-fail-fast > /tmp/dx19-full-test.txt 2>&1; echo "exit=$?"
grep -nE '^(error|warning: unused|test result:|failures:)' /tmp/dx19-full-test.txt
~/.cargo/bin/cargo clippy --all-targets -- -D warnings > /tmp/dx19-clippy.txt 2>&1; echo "exit=$?"
~/.cargo/bin/cargo fmt --check
tools/check-defs-fmt.sh          # SR-35 — the only thing that checks the 1,798 defs
```

Baseline to beat: **4,263 passing / 0 failing / 5 ignored** on main at `b76b1df4`. Expected delta:
**+5** (2 repaired/added in `static_grants.rs`, 4 added in the PB-DX19 primitives file, minus the
one `static_grants.rs` test that is repaired in place rather than added — recount from the run, do
not carry this arithmetic forward).

---

## §10 The mandatory fuzzer experiment

**Question**: does this batch close the stack-overflow half of `OOS-DP3-9` / `OOS-M11-3`?

### 10.1 The binary and its flags

`crates/simulator/src/bin/fuzzer.rs`, binary name `mtg-fuzzer` (`:60`). Flags read from the header
(`:7-15`) and clap struct (`:65-93`): `--games <N>` (default 1000), `--players <N>` (default 4),
`--max-turns <N>` (**default 200**), `--seed <SEED>` (default random; each game uses
`base_seed.wrapping_add(game_index)`, `:168`), `--threads`, `--bot random|heuristic`,
`--stop-on-error`, `--replay <SEED>`, `--verbose`.

### 10.2 Profile

**`--profile fuzz` for both arms.** `[profile.fuzz]` (`Cargo.toml:51-54`) is release-optimised with
`debug-assertions` and `overflow-checks` on, so a run sees both phenomena this batch touches: the
SR-4/SR-14 `debug_assert!` tripwires, and — decisively for `OOS-SIM2-5` — a **panic** on P/T
overflow rather than a silent wrap to negative power. Do not mix profiles between arms; a stack
overflow's depth-to-crash is frame-size dependent and therefore profile-dependent.

### 10.3 The ordering subtlety, and how the control arm is actually obtained

The brief's framing ("as-is vs Archangel's static commented out") does not survive the fix landing:
post-fix, the as-is arm no longer overflows *from this cause*, so there is nothing for the control
to control for. **The real experiment is pre-fix vs post-fix**, and the pre-fix measurement must be
taken from a tree that still has the defect. Taking it by editing a committed def is both
unnecessary and unprovable-after-the-fact.

**Procedure — measure the control arm from the merge base, in a separate worktree:**

```bash
BASE=$(git merge-base main HEAD)
git worktree add /tmp/dx19-base "$BASE"

# Arm 1 (PRE-FIX, control)
cd /tmp/dx19-base
RUST_BACKTRACE=1 ~/.cargo/bin/cargo run --profile fuzz --bin mtg-fuzzer -- \
  --games 15 --seed 1 --players 4 --bot random > /tmp/dx19-fuzz-prefix.txt 2>&1; echo "exit=$?"

# Arm 2 (POST-FIX)
cd <worktree root>
RUST_BACKTRACE=1 ~/.cargo/bin/cargo run --profile fuzz --bin mtg-fuzzer -- \
  --games 15 --seed 1 --players 4 --bot random > /tmp/dx19-fuzz-postfix.txt 2>&1; echo "exit=$?"

git worktree remove /tmp/dx19-base
```

**Validity precondition — the two arms must draw the same decks.** `random_deck`
(`crates/simulator/src/deck.rs:30`) filters the pool on `completeness.is_complete()` (`:43`, `:65`),
so the seed→deck mapping depends on the *set of `Complete` defs*. This batch changes no def's
marker and adds/removes no def, so the pools are identical and `--seed 1` draws identically. Prove
it rather than assert it:

```bash
git diff "$BASE"..HEAD --stat -- crates/card-defs   # must be exactly greymond_avacyns_stalwart.rs
git diff "$BASE"..HEAD -- crates/card-defs | grep -E '^[+-]\s*completeness:\s*Completeness::'
# ^ must return NOTHING: the marker line itself is unchanged; only the string argument moved.
```

**No committed def is edited for this experiment.** If a future reader insists on the
"static disabled" arm as well, the only honest way is an uncommitted revert of the single `:10259`
line, run, then `git diff --exit-code -- crates/engine/src/effects/mod.rs` to prove restoration —
but that is a third arm, not a substitute for the two above.

### 10.4 Reading the result — the two traps

1. **The default `--max-turns` is 200, which is the exact configuration `OOS-M11-3` says already
   stack-overflows for an unrelated reason.** A pre-fix overflow is therefore **not** automatically
   this defect. Classify every overflow by its backtrace: this batch's cause shows
   `calculate_characteristics` / `is_effect_active` / `check_static_condition` /
   `expect_characteristics` cycling; `OOS-M11-3`'s does not. `RUST_BACKTRACE=1` is in the invocation
   above for exactly this. **Only if the post-fix arm shows zero overflows and the pre-fix arm's
   overflows all carry the four-hop cycle may `OOS-DP3-9`/`OOS-M11-3`'s stack-overflow half be
   merged into this batch.**
2. **A null result is not evidence of closure.** Per `OOS-UI2-1`/`OOS-SIM3-1` (reconciled in the
   2026-08-02 re-rank), the fuzzer's first non-land is drawn around personal draw 35-40, so a game
   only begins casting deep into the default 200-turn cap. Fifteen games may contain **zero**
   Archangels on the battlefield, in which case both arms are green and the experiment has measured
   nothing. Report that outcome as *"inconclusive — no Archangel reached the battlefield"*, not as
   *"closed"*. If it comes back inconclusive and time allows, re-run with `--games 60` before
   drawing any conclusion.

**Record the result either way**, in the fix-cycle report and in `memory/workstream-state.md`. The
*decisive* evidence for `OOS-SIM2-6` is the three stage-0 probes plus their watched-failing revert,
not the fuzz run; the fuzz run is corroboration and an `OOS-DP3-9` disposition, and the plan says so
rather than letting a green fuzz run be mistaken for the proof.

---

## §11 Risks and pre-committed claims

### Pre-committed claims (state these before running; a disagreement is a stop-and-re-read)

1. **0 completeness flips.** Coverage stays at its collect value. Proof obligations:
   `git diff --stat -- crates/card-defs` shows exactly one file (`greymond_avacyns_stalwart.rs`);
   `git diff -- crates/card-defs` contains no `completeness: Completeness::` line change; and a
   regeneration (`python3 tools/authoring-report.py`) produces a report body **byte-identical** to
   the committed one.
2. **The greymond note-text edit cannot move a report count — verified, not assumed.**
   `tools/authoring-report.py` never parses the `completeness` note string: `MARKER_RE` (`:152`)
   captures only the variant name; `classify_file` (`:185-194`) buckets on that name;
   `marker_disagreements` (`:197-214`) asks only whether a TODO comment *exists*; and
   `classify_todo` (`:235-239`) is applied **only** to lines matching `TODO_LINE_RE` (`:176-177`),
   i.e. `// TODO` / `// ENGINE-BLOCKED` comment lines. **But greymond has four such lines
   (`:6`, `:7`, `:34`, `:35`) that DO feed `total_todos` and the `todo_classes` histogram** — so the
   claim holds *only* under §6's constraint that those four lines are left byte-identical. If the
   runner decides a TODO line must change, claim (1) is void and the report must be regenerated and
   its delta reported.
   `crates/engine/tests/core/authoring_report.rs:83`
   (`authoring_report_buckets_match_the_compiled_registry`) compares report buckets against the
   compiled registry's markers only — unaffected either way.
3. **Wire unmoved: PROTOCOL 33, HASH 70** — computed by §9's commands, not asserted.
4. **Benchmarks improve or stay flat.** The fix *removes* work: `expect_characteristics` per
   candidate per call becomes a field borrow, and the owned `Characteristics` clone disappears.
   `full_turn_4p` / `priority_cycle_4p` should be within noise or slightly faster. A **regression**
   would mean something else changed and is a stop signal.

### Risks

- **The saturating clamp hides a real bug.** A P/T that saturates is indistinguishable from one
  legitimately computed, and no diagnostic fires. This is a genuine loss relative to the current
  (crashing) behaviour under `overflow-checks`, and it is the reason `OOS-DX19-3` is filed rather
  than the deviation being considered closed by documentation alone.
- **The `u32 → i32` cast fix changes behaviour at a value no test currently reaches.**
  `i32::try_from(...).unwrap_or(i32::MAX)` is not a no-op for counts above `2^31-1`; today those
  produce negative power. There is no realistic game state with 2 billion counters, so this is
  hardening, not a repair of an observed defect. Say so; do not overclaim it.
- **The `static_grants.rs` repair touches a shared `use` block** (`:12-16`) that many tests in the
  file depend on. Removing a now-unused import can break sibling tests. Let
  `cargo clippy --all-targets -- -D warnings` decide; do not prune by eye.
- **The revert-to-watch-failing step kills the whole test binary** (SIGABRT), so `--test rules` and
  `--test primitives` both report far fewer tests than they contain during the pre-fix observation.
  That is expected and must be stated in the recorded output, or a reviewer will read it as a
  harness fault. Also: **confirm the revert compiles** before believing its failure (PB-DX6's first
  revert did not).
- **The deviation is live, not theoretical** (§3.3): `blinkmoth_nexus` and `inkmoth_nexus` are both
  `Complete`, both colorless, and both animate into artifacts via Layer 4. Post-fix, an animated
  Nexus does not feed Metalcraft. If the review treats that as a new HIGH rather than as the
  documented, seeded price of terminating, this batch's scope has been misread — §3 is the argument,
  and it should be cited rather than re-litigated.
- **Fuzz-run flakiness**: `OOS-M11-3`'s determinism half is still open, so two runs of the *same*
  arm may differ. Do not treat a single pair of runs as an A/B unless both arms are internally
  reproduced at least twice, or the pre-fix arm's crash carries the four-hop backtrace (which is
  self-identifying and needs no repetition).

---

## §12 Seeds to file

| id | title | evidence to attach |
|---|---|---|
| **OOS-DX19-1** | The sibling recursion class: **ten** `expect_characteristics` sites in `check_condition` (`effects/mod.rs:9682, 9693, 9791, 9807, 9846, 9867, 9885, 9898, 9909, 10077`) are reachable from the layer path via `check_static_condition`'s `_` catch-all (`:10283`), including through `Condition::Not` (`:9768`) / `Condition::Or` (`:9777`) wrappers. **Latent today** — §4.3's enumeration shows zero corpus defs and zero engine sites put any of the ten variants in a `ContinuousEffectDef.condition`. The next author who writes "as long as you control a legendary creature, …" as a **static** reopens the HIGH with no warning. **Proposed fix is a boundary guard, not ten leaf edits**: a `thread_local` re-entrancy flag (or depth counter) set by `calculate_characteristics` and asserted by `expect_characteristics`, so the *shape* is refused rather than each instance patched — several of the ten are **correct** as layer-resolved on their real (ETB-replacement / `Effect::Conditional`) call paths and must not be downgraded. | §4.3 table; the `// CR 613.1d … Blood Moon` comments at `:9784-9785` that make blanket conversion wrong |
| **OOS-DX19-2** | The CR-613.8b-honest fixpoint. Base-characteristics condition evaluation is **live-wrong** on two `Complete`, colorless, deck-legal defs: `blinkmoth_nexus.rs:42-43` and `inkmoth_nexus.rs:43-44` animate a land into an **artifact** creature via Layer-4 `AddCardTypes([Artifact, Creature])`, and post-PB-DX19 they will not feed `indomitable_archangel`'s Metalcraft though CR 604.2 + 613.1d say they must. Fixing it means lifting characteristics calculation to a global fixpoint with CR 613.8b's dependency-loop → timestamp-order termination and CR 613.8c re-evaluation. Machinery partially exists: `layers.rs:1747` `resolve_layer_order` → `:1764` `toposort_with_timestamp_fallback`. | the §3.3 table; the discriminating test is already specified there (animated Nexus + Archangel + 2 real artifacts → CR says shroud, engine says none) |
| **OOS-DX19-3** | A saturated P/T is silently indistinguishable from a legitimately-large one. The `[profile.fuzz]` `overflow-checks` tripwire that would have caught `OOS-SIM2-5` is **removed** by this batch's own fix. Proposal: route the clamp through a `state::diagnostics` `expect_*`-class assertion (SR-4's classification), so a saturation fires a `debug_assert!` naming the object and the layer while still returning a total answer in release. | §8.4; the SR-4 `expect_*` vs `lki_*` split in `docs/engine-invariants.md` |
| **OOS-DX19-4** | `calculate_characteristics` has no recursion tripwire at all. A `thread_local` depth counter with a `debug_assert!` at, say, depth 8 would have surfaced `OOS-SIM2-6` in 2026-03 as a named debug failure instead of a 2026-08 SIGABRT from a live game. Cheap; overlaps `OOS-DX19-1`'s proposed fix and could subsume it. | the stage-0 verbatim output in `pb_dx19_characteristics_recursion.rs:156-166` |

---

## Verification checklist

- [ ] §1 premise table re-confirmed on the branch head before the first edit
- [ ] `effects/mod.rs:10259` reads `&obj.characteristics`; `exclude_self` reordered first (§4.1/§4.2)
- [ ] Comment at `:10247-10258` replaced with §5's text — real invariant, CR 613.8b, precedent,
      named deviation, **no** "other objects" argument, **no** performance framing
- [ ] `greymond_avacyns_stalwart.rs`: **only** the `Completeness::inert` string changed; the four
      `// TODO` lines byte-identical (§6, §11.2)
- [ ] `static_grants.rs::test_artifacts_you_control_grants_shroud` drives the real def through
      `register_static_continuous_effects`; the hand-built `condition: None` block is gone; three
      original assertions preserved; Metalcraft-off case added as a second `#[test]` (§7)
- [ ] All 16 arithmetic edits in `layers.rs` (§8.1-8.3), including the two `u32 → i32` casts and the
      three `saturating_neg`
- [ ] Deviation recorded in `memory/decisions.md` + short form in the `layers.rs` doc comments (§8.4)
- [ ] Four new P/T tests in `tests/primitives/pb_dx19_characteristics_recursion.rs`; **no**
      `primitives/main.rs` edit (SR-9a)
- [ ] Every new/repaired test **watched failing** by a revert that **compiles**; verbatim output
      recorded at the test
- [ ] PROTOCOL and HASH **computed** by the §9 gate commands; predicted 33 / 70; no pin edited
- [ ] `cargo test --workspace --no-fail-fast` captured to a **file**; `grep` for `^error`/`failures:`
- [ ] `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` **and**
      `tools/check-defs-fmt.sh` (SR-35)
- [ ] `git diff --stat -- crates/card-defs` = 1 file; no `completeness:` marker line changed;
      `tools/authoring-report.py` body byte-identical (§11.1)
- [ ] Fuzz experiment run per §10, both arms, `--profile fuzz`, `RUST_BACKTRACE=1`, deck-pool
      identity proven; result recorded **including** an inconclusive one
- [ ] `OOS-DX19-1..4` filed with the evidence in §12
- [ ] Benches spot-checked (`full_turn_4p`, `priority_cycle_4p`) — flat or faster; a regression is a
      stop signal
