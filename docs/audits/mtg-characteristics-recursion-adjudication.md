# Adjudication — the external characteristics-recursion review vs PB-DX19

<!-- last_updated: 2026-08-02 -->

**Date**: 2026-08-02
**Task**: `scutemob-186`
**Adjudicates**: `docs/audits/mtg-characteristics-recursion-findings.md` (external review agent,
2026-08-02, HIGH), against the PB-DX19 dispatch brief
(`memory/primitives/seed-rerank-2026-08-02.md` §4) and against what `scutemob-184` actually
shipped on branch `feat/pb-dx19-the-unbounded-characteristics-recursion-oos-sim2-6-h`.
**Scope**: read-only. This task changed no source, no card def, no test, no coordination file.
The only file it writes is this one.
**Method**: the corpus figures are enumerated from `all_cards()` through a throwaway `serde_json`
dump run outside the repo — **never grepped from def sources** (SR-36). CR text and rulings quoted
verbatim from the `mtg-rules` MCP server.

> ### Two trees, and which claim is anchored to which — read this before checking a line number
>
> `scutemob-184` **merged to main as `451e3517` while this task was running.** Every line number
> in §1 and §2.2 is against this worktree's base, **`62e5699a`** — the *pre-fix* tree. That is the
> correct anchor for §1, because the external doc reviewed pre-fix code and §1 asks whether it
> read that code correctly. It is the **wrong** anchor for main today: `451e3517` moved
> `effects/mod.rs`'s dangerous site from `:10259` to `:10295` and inserted ~90 lines at the top of
> `layers.rs`. §4 is anchored to **`451e3517`** and says so. Anything not explicitly marked is
> `62e5699a`.
>
> Two consequences of the merge that this document absorbed rather than hid: `scutemob-184` grew
> from six commits to **ten** (`ee7a55b4`, `a0d977e5`, `79b94a58`, `4b620c8b`, `cdf194f2`,
> `007a1d1c`, `697606a6`, `a405aa84`, `569087e6`, merge `451e3517`; +2,821 / −96 over 16 files),
> and the last of those **shipped a source gate** that §5.1's first draft proposed as new work.
> §4 and §5.1 are written against what actually landed.

> **Index line not added.** `docs/audits/README.md` carries one row per audit. This task's
> acceptance criterion 6055 confines `git diff` to this file alone, so the row is deliberately
> **not** written here. Adding it is a collect-time chore, not a worker edit.

---

## 0. Executive summary — what changed under the conflict

The task framed a **central conflict**: the external doc's *Rejected Fixes* section rejects the
base-characteristics fix that the v3 brief pre-decided and `scutemob-184` was shipping. Both
halves of that framing needed re-checking, and both moved.

1. **The brief did pre-decide it, in those words.** `seed-rerank-2026-08-02.md` §4, PB-DX19:
   *"The fix is one line: read `&obj.characteristics` at `effects/mod.rs:10259` instead of
   calling `expect_characteristics` … **Take the base-characteristics fix**; a dependency-aware
   fixpoint is a PB of its own and this one should land today."* The external doc's objection is
   aimed at a real, written decision, not a straw man.

2. **`scutemob-184` did not ship it.** Its *first attempt* did, and its own review caught that as
   a HIGH regression (commit `007a1d1c`, "fix a HIGH regression this batch introduced, found by
   review"). What merged is narrower: a new
   `rules::layers::characteristics_for_condition(state, obj)` that returns
   `expect_characteristics` **everywhere except inside a `calculate_characteristics` walk**, where
   it falls back to `obj.characteristics`. The walk is detected by a thread-local depth counter
   (`LAYER_WALK_DEPTH`) set by an RAII `LayerWalkGuard`. Four of the five callers of the shared
   condition evaluators keep full layer resolution; only the one that closes the cycle degrades.

3. **So the live conflict is not the one the task named.** The external doc's *"Use printed/base
   characteristics"* rejection lands on a fix that no longer exists in that form. Its *"Use
   thread-local or global mutable recursion state"* rejection lands squarely on the fix that does.
   That second rejection is the one worth adjudicating, and §3.4 does.

**The measurement settles the size of the argument.** Enumerating `all_cards()`:

| quantity | measured |
|---|---|
| card defs | **1,803** (`Complete`: 1,133) |
| `ContinuousEffectDef` instances (static + `ApplyContinuousEffect`) | **382** |
| …carrying a `condition` at all | **17 instances** across **15 cards** |
| …whose condition reaches a **layer-resolved** characteristics query | **1 instance, 1 card** |
| the card | **`indomitable_archangel`** (`Complete`) |
| `Complete` cards whose Layer-4 effects **add** the Artifact type | **3** |
| `Complete` cards whose Layer-4 effects **remove** it (by replacement) | **3** |
| `Complete` cards where a **DFC face flip** moves it | **1** |
| **deck-legal `Complete` combinations live-wrong under the shipped deviation** | **7**, plus the unbounded CR 708.2a face-down class |

Eleven of the 52 `Condition` variants can reach a layer query; exactly one of them is used as a
`ContinuousEffectDef.condition` anywhere in the corpus. The recursion is a one-card problem
**today** — and a defect *generator* tomorrow, because nothing in the tree stops the population
from growing.

**The CR verdict** (§3): the external doc's **durable** architecture (layer-bounded queries) is
CR-correct and, measured against this corpus, terminates by construction with no cycle-breaking
whatsoever. Its **immediate** patch semantics (treat a recursively-encountered effect as
*inactive*) has **no CR warrant** — CR 613.8b prescribes **timestamp order** for a dependency
loop, never inactivity — and the doc proposes the two in the wrong order for this codebase.

**Disposition** (§5): **accept in substance, reject the sequencing, split the deliverable.**
A follow-up PB is warranted and briefed as **PB-DX42**, ranked **13** in the v3 queue. Its cheap
half — a *corpus* roster gate pinning the layer-querying-condition population at exactly 1 — is
offered as a **rider on PB-DX8** (rank 10, already test-only). Note that `451e3517` shipped a
*source* gate over the same subsystem; §5.1 explains why the two are complementary rather than
duplicative. Seven seeds are identified for filing (§6).

---

## 1. Claim-by-claim verification (AC 6051)

Verdicts: **CONFIRMED** / **PARTIAL** / **REFUTED** / **STALE** (true when written, overtaken by
`scutemob-184`).

### 1.1 The confirmed call chain (external doc §"Confirmed Call Chain")

| # | claim | verdict | evidence |
|---|---|---|---|
| C1 | `calculate_characteristics` collects every active effect before the layer loop, with the quoted `.filter(\|e\| is_effect_active(state, e))` | **CONFIRMED — verbatim** | `crates/engine/src/rules/layers.rs:43-47`. The doc's Rust snippet matches the source token for token. |
| C2 | "the condition of every conditional continuous effect is evaluated for every characteristic query, regardless of which object is being queried" | **CONFIRMED** | The predicate at `layers.rs:46` is `is_effect_active`, which **takes no `object_id`** (`:508`) — the queried object cannot narrow the sweep even though it is in scope at the call. `scutemob-184`'s stage-0 probes prove it empirically: `recursion_is_independent_of_the_object_being_calculated` and `recursion_metalcraft_off_still_terminates` both SIGABRT pre-fix. |
| C3 | `is_effect_active` calls `check_static_condition(state, condition, source_id, controller)` after the duration check | **CONFIRMED — verbatim** | `layers.rs:565`, inside the `if let Some(ref condition) = effect.condition` block opened at `:558`; duration is decided at `:509-555`. |
| C4 | "The nearby comment says conditions are evaluated 'at layer-application time,' but this call occurs while constructing `active_effects`, before any layer is processed" | **CONFIRMED** | The comment is `layers.rs:556-557`: *"CR 604.2: Conditional static abilities … Conditions are evaluated against the current game state at layer-application time."* The layer loop begins at `:49`; `is_effect_active` is called from `:46`, before it. The same *claim* appears at two more sites — `effects/mod.rs:10207` (*"CR 604.2: Called at layer-application time for conditional continuous effects"*, near-identical wording) and `crates/card-types/src/state/continuous_effect.rs:559-560` (*"Evaluated at layer-application time in `is_effect_active`"*, a paraphrase). **Three sites carry the claim; one is verbatim.** |
| C5 | The Archangel def registers the quoted Layer-6 grant with the quoted Metalcraft condition | **CONFIRMED — verbatim** | `crates/card-defs/src/defs/indomitable_archangel.rs:29-43`. Field order, `EffectFilter::ArtifactsYouControl`, `EffectDuration::WhileSourceOnBattlefield`, `count: 3`, `has_card_type: Some(CardType::Artifact)`, `..Default::default()` — all exact. The def declares **no** `completeness` field, so the `#[default] Completeness::Complete` derive makes it deck-legal (`crates/card-types/src/cards/card_definition.rs:197-210`). |
| C6 | `check_static_condition` handles `YouControlNOrMoreWithFilter` by iterating controlled battlefield objects and calling `expect_characteristics`, then `matches_filter` | **CONFIRMED — verbatim** | `crates/engine/src/effects/mod.rs:10238-10270`; the call is `:10259`, `matches_filter` at `:10260`. |
| C7 | The cycle diagram (`calculate_characteristics` → active effects → `is_effect_active` → `check_static_condition` → `expect_characteristics` → `calculate_characteristics`) | **CONFIRMED** | Four hops, closed by `expect_characteristics` (`layers.rs:477`) which is a thin wrapper: `:478` `if let Some(chars) = calculate_characteristics(state, object_id) { return chars; }`. |
| C8 | "The cycle does not require the outer query to be for Archangel or for an artifact" | **CONFIRMED** | Same evidence as C2. This is the sharpest correct observation in the external doc and it is the one the pre-existing in-source comment got wrong. |

### 1.2 The incorrect existing termination argument (external doc §"Incorrect Existing Termination Argument")

| # | claim | verdict | evidence |
|---|---|---|---|
| C9 | A comment above the nested call claims safety from (1) persistent-structure immutability and (2) checking *other* objects | **CONFIRMED — verbatim** | `effects/mod.rs:10247-10258`: *"This is re-entrant but safe: `im-rs` persistent data structures are immutable, so there is no risk of observing partial mutations. Termination is guaranteed because we are checking the types of \*other\* battlefield objects, not the object currently being calculated — there is no direct self-referential cycle."* |
| C10 | "Neither claim establishes termination" / "Immutability prevents observing partial mutation. It does not prevent a pure function from recursively calling itself forever." | **CONFIRMED** | Correct as reasoning, and empirically settled: `scutemob-184` ran `mtg-fuzzer --games 15 --seed 1` under `[profile.fuzz]` at the pre-fix tree — `fatal runtime error: stack overflow`, SIGABRT, exit 134, **0 of 15** games completed. Post-fix: 15/15, avg 189 turns. |
| C11 | "The comment should be removed or rewritten as part of the fix" | **CONFIRMED, and already done** | `scutemob-184` rewrote it with the mechanism and calls that rewrite "worth more than the one-line code change." Its lesson 1 — *"A termination argument in a comment is a claim, and claims rot"* — is the same finding. |
| C12 (unstated, found here) | The comment does not merely fail to prove termination; **it proposes the defective fix as a performance note** — *"If performance becomes an issue, consider using base characteristics (`obj.characteristics`) for the filter check"* (`effects/mod.rs:10256-10258`) | **ADDITIONAL** | This sentence is the direct ancestor of the v3 brief's "take the base-characteristics fix." The external doc read the comment and did not notice that the fix it rejects was written *in* the comment it critiques. Worth recording because it is how the deviation propagated. |

### 1.3 Scope and risk (external doc §"Scope and Risk")

| # | claim | verdict | evidence |
|---|---|---|---|
| C13 | "A legal card definition can trigger a stack overflow" | **CONFIRMED** | `indomitable_archangel` is `Complete`; `validate_deck` rejects only non-`Complete` cards (SR-2). |
| C14 | "The crash is not restricted to one UI, simulator, or networking path… occurs in core rules evaluation" | **CONFIRMED** | The recursion lives in `crates/engine/src/rules/layers.rs`, a pure-library crate (Architecture Invariant 1). A stack overflow is SIGABRT, not a panic, so it is **not** `catch_unwind`-able and the play-server's request boundary cannot contain it. |
| C15 | "Any static condition that (1) is evaluated from `is_effect_active` and (2) calls `calculate_characteristics` or `expect_characteristics` can create the same class of recursion" | **CONFIRMED, and now measured** | **11 of 52** `Condition` variants reach a layer-resolved query — see the inventory in §2.2. Three more (`And`/`Or`/`Not`) reach it transitively through their operands. |
| C16 | "This finding is broader than the `Indomitable Archangel` card definition. The card is a reproducer, not the root cause." | **CONFIRMED as to mechanism; PARTIAL as to live population** | The mechanism is general. The corpus is not: **exactly one** of the 382 `ContinuousEffectDef`s uses a layer-querying condition (§2.1). Every other occurrence of those 11 variants sits in an `activation_condition`, `intervening_if`, `unless_condition`, or bare `Effect::Conditional` — none of which `is_effect_active` reads. `scutemob-184` independently measured the same thing over 57 occurrences and filed it as `OOS-DX19-1`. |
| C17 | The proposed audit command: `rg -n 'check_static_condition\|calculate_characteristics\|expect_characteristics' crates/engine/src/effects crates/engine/src/rules` | **PARTIAL — the surface is wrong** | That command returns **373** hits (rules 295 + effects 78). The same pattern over `crates/ tools/` returns **1,146** — the proposed surface is **32.5%** of it. Outside the two named directories there are **14 real call sites** it cannot see: `crates/engine/src/state/mod.rs:1201` (the SR-24 LKI capture path), 7 in `crates/simulator/src/legal_actions.rs`, `crates/simulator/src/params.rs:307`, `crates/simulator/src/mana_solver.rs:391`, `crates/view-model/src/lib.rs:452`, and `tools/play-server/src/view.rs:1227/1234/1268` (reachable via the `crates/engine/src/lib.rs:27` re-export). It also misses **718 hits across 103 files** in `crates/engine/tests/`, which is where a regression would actually be caught. **Acceptance criterion 8 of the external doc ("A code search identifies and reviews every characteristic query reachable from `check_static_condition`") cannot be satisfied by the command the doc supplies.** |

### 1.4 Feasibility of the proposed design (external doc §"Immediate Remediation" / §"Durable Architectural Repair")

This is the half the task asked to be checked hardest, since the external agent saw four files.

| # | assumption | verdict | evidence |
|---|---|---|---|
| D1 | `ContinuousEffect` has an `EffectId` suitable for keying `evaluating_conditions: HashSet<EffectId>` | **CONFIRMED** | `crates/card-types/src/state/continuous_effect.rs:531-533` — `pub struct ContinuousEffect { pub id: EffectId, … }`. `EffectId` is a real newtype with a `HashInto` impl (`crates/engine/src/state/hash.rs:2335`) and is already used as **logical identity** in preference to `ptr::eq` — `layers.rs:1844-1845` records that choice (MR-M5-03). The doc guessed the design surface correctly without seeing the file. |
| D2 | `calculate_characteristics(state, object_id) -> Option<Characteristics>` — the signature the doc's wrapper/inner split assumes | **CONFIRMED** | `layers.rs:35-38`, exactly. `None` iff the id is absent (`:31-34`). |
| D3 | `expect_characteristics` exists in `rules::layers` | **CONFIRMED** | `layers.rs:477`. |
| D4 | `EffectLayer` has variants named `TypeChange`, `ColorChange`, `Ability`, `PtSwitch` | **CONFIRMED** | `crates/card-types/src/state/continuous_effect.rs:20-41` — ten variants, `Copy`/`Control`/`Text`/`TypeChange`/`ColorChange`/`Ability`/`PtCda`/`PtSet`/`PtModify`/`PtSwitch`. All four named by the doc exist under exactly those names. The doc's ordering assumption (Type < Color < Ability < Pt) matches `layers.rs:49-60`. |
| D5 | `TargetFilter` has `max_power`, `min_power`, `has_keywords` (used in the sketched `required_characteristic_layer`) | **CONFIRMED** | `crates/card-types/src/cards/card_definition.rs:3036-3040` and the serialized field list; `matches_filter` reads them at `effects/mod.rs:9534/9539/9549`. |
| D6 | The sketched `required_characteristic_layer` "account[s] for every `TargetFilter` field used by `matches_filter`" once extended to power/toughness, card types, supertypes, subtypes, colors, keywords | **PARTIAL — the field list is incomplete** | `matches_filter` (`effects/mod.rs:9533-9660`) also branches on `filter.has_name` (`:9600` → `chars.name`) and on `filter.max_cmc` / `filter.min_cmc` (`:9606`/`:9616` → mana value, recomputed per layer from `chars.mana_cost` at `layers.rs:354-358`). **Name and mana value are Layer-1/Layer-3 characteristics** and appear in neither the doc's code sketch nor its prose list. A policy function built to the doc's list would silently return `TypeChange` for a name filter and read a stale name whenever a Layer-1 copy or Layer-3 text-change effect is live. |
| D7 | "Raw `GameObject` properties such as counters, controller, tapped status, combat status, and token status should remain separate from characteristic-layer requirements" | **CONFIRMED, and the separation already exists** | `check_has_counter_type` (`effects/mod.rs:9522-9530`) reads `obj.counters` and is *deliberately* not called from inside `matches_filter` — its own doc comment at `:9517-9521` says so, and `check_static_condition` calls the two side by side at `:10260-10262`. The doc independently re-derived an existing invariant. |
| D8 | "The current `is_effect_active` combines two different questions" (duration/liveness vs conditional applicability) and they should be split | **CONFIRMED as a description** | `layers.rs:509-552` is the duration/liveness half; `:556-573` is the condition half; the early `return false` at `:553-555` is already the seam. The split is a ~10-line refactor, not an architecture change. |
| D9 | The listed duration facts ("source exists; on battlefield; phased in; until-end-of-turn not expired; pairing/control-duration markers remain valid") are evaluable from raw state | **CONFIRMED** | `layers.rs:510-551` evaluates exactly those, plus the CR 702.26e phased-out guard at `:514-515` and the CR 611.2b/c expiry-owned arms (`UntilYourNextTurn`, `WhileYouControlSource`) at `:525-532`. No layer query anywhere in that block. |
| D10 | Threading a `&mut CharacteristicEvalContext` through the recursive branch is a contained change | **PARTIAL — the blast radius is larger than the doc implies** | The nested call the doc rewrites is inside `check_static_condition`, but 10 of the 11 layer-querying variants are reached through `check_condition` (`effects/mod.rs:9662`), a `pub fn` whose other callers are `activation_condition`, `intervening_if`, `Effect::Conditional` and `unless_condition`. A `ctx` parameter must be threaded through **`check_condition` too**, or duplicated. This is why `scutemob-184` reached for ambient state instead — a decision the doc rejects (§3.4) without costing. |
| D11 | "The in-progress marker must always be removed, including on early returns. Use a small helper, guard object, or closure pattern" | **CONFIRMED as necessary, and independently implemented** | `calculate_characteristics` has several early returns (`layers.rs:39` `?`, and the `break` at `:378-380`). `scutemob-184` solved exactly this with an RAII `LayerWalkGuard` whose `Drop` decrements (`451e3517`: `layers.rs:41-56`). The doc's caution was correct and is already satisfied by the shipped shape. |

### 1.5 Claims about the fix that are now STALE

| # | claim | verdict | why |
|---|---|---|---|
| S1 | *Rejected Fixes* → "Use printed/base characteristics in the condition … Do not replace the nested call with `obj.characteristics`" | **STALE as a critique of the shipped code; CONFIRMED as a critique of the brief** | The v3 brief said to do exactly that; `scutemob-184`'s first attempt did; its own review reverted it as a HIGH regression because it broke the four safe call paths (`activation_condition`, `intervening_if`, `Effect::Conditional`, `unless_condition`). The shipped code degrades **only** inside the layer walk. |
| S2 | The doc's implied claim that the deviation is unbounded | **REFUTED — it is bounded, and measured** | §2 measures it at **7** deck-legal `Complete` combinations plus the unbounded CR 708.2a face-down class. |
| S3 | *Required Regression Tests* → the eight named tests | **PARTIAL — five of the eight are answered; three are not** | `crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs` (**17** `#[test]` fns at `451e3517`, plus 8 helper `fn`s) supplies: `recursion_metalcraft_on_grants_shroud_and_terminates` (≈ "crash reproducer" + "Metalcraft true"), `recursion_metalcraft_off_still_terminates` (≈ "Metalcraft false"), `recursion_is_independent_of_the_object_being_calculated` (≈ "query unrelated object"), and `the_deviation_is_scoped_to_the_layer_walk_only` (≈ "no stale context entry" — it asserts the `LayerWalkGuard` decrements on `Drop`). Each was watched failing by an **executed, compiling** revert. It also adds tests with no counterpart in the doc's list: `non_layer_path_reads_layer_resolved_power` / `…_subtypes`, `sibling_condition_on_a_continuous_effect_terminates`, and the `no_condition_evaluator_resolves_characteristics_directly` source gate. **Three of the doc's eight remain open work**: "Layer 4 adds Artifact" exists only *inverted* (`deviation_animated_nexus_does_not_count_toward_metalcraft`, which pins the deviation and tells the next author to invert rather than delete it); "Layer 4 removes Artifact" does not exist; "multiple conditional effects nest deterministically" does not exist and **cannot** be written against the shipped depth counter, because a depth counter is exactly the "single boolean" shape the doc warns about. |
| S4 | *Rejected Fixes* → "Add an arbitrary recursion-depth limit … may be useful as a final panic-prevention assertion, but it is not the semantic fix" | **CONFIRMED, and agreed on both sides** | `scutemob-184` filed `OOS-DX19-4` asking for precisely a depth tripwire as a diagnostic, on the ground that "a stack overflow is not a test failure — it is signal 6, names no test, and takes the binary down." No conflict. |
| S5 | *Rejected Fixes* → "Special-case Indomitable Archangel" | **CONFIRMED, and honored** | No card-specific branch exists in either fix. The only card-def edit in the whole batch is the `greymond_avacyns_stalwart` note the brief mandated — an authoring landmine that instructed a future author to build a second instance of this shape. |

---

## 2. The deviation, measured (AC 6052)

**Method.** A throwaway crate outside the repo (`serde_json` over `mtg_card_defs::all_cards()`)
serialized all **1,803** definitions; every `ContinuousEffectDef` was located structurally by its
field set, which catches both `AbilityDefinition::Static` and the **172**
`Effect::ApplyContinuousEffect` sites nested arbitrarily deep inside effect trees. No def source
was grepped (SR-36). Numbers below are reproducible by rerunning that dump.

### 2.1 The demand side — conditions that query layer-resolved characteristics

| quantity | count |
|---|---|
| `ContinuousEffectDef` instances in the corpus | **382** |
| …with `condition: Some(_)` | **17 instances**, on **15 distinct cards** |
| …whose condition variant reaches a layer-resolved query | **1 instance, 1 card** |

The full condition census over those 17 instances. **Instances and cards differ** — two cards
carry two conditioned effects each, which is why a per-card column does not sum to 17:

| variant | instances | cards | the cards | layer-querying? |
|---|---|---|---|---|
| `SourceHasCounters` | 4 | 3 | `arixmethes_slumbering_isle` (×2), `beastmaster_ascension`, `quest_for_the_goblin_lord` | no |
| `DevotionToColorsLessThan` | 3 | 3 | `athreos_god_of_passage`, `iroas_god_of_victory`, `purphoros_god_of_the_forge` | no — reads **base** `mana_cost` (§2.5) |
| `IsYourTurn` | 3 | 3 | `radha_heart_of_keld`, `razorkin_needlehead`, `triumphant_adventurer` | no |
| `ControllerLifeAtLeast` | 2 | 1 | `serra_ascendant` (×2) | no |
| `OpponentLifeAtMost` | 1 | 1 | `bloodghast` | no |
| `SourceIsUntapped` | 1 | 1 | `dragonlord_ojutai` | no |
| `CompletedADungeon` | 1 | 1 | `nadaar_selfless_paladin` | no |
| `YouControlYourCommander` | 1 | 1 | `skyhunter_strike_force` | no |
| **`YouControlNOrMoreWithFilter`** | **1** | **1** | **`indomitable_archangel`** (`Complete`) | **YES** |
| **total** | **17** | **15** | | |

**None of the 17 uses `And` / `Or` / `Not`**, so no layer-querying variant hides inside a
combinator. That is what makes "exactly one" a closed statement rather than a top-level scan.

**The demand side is a set of size one.** Every other condition on a continuous effect reads
player state, counters, turn structure, or base mana cost — none re-enters the layer system.

### 2.2 Every path reachable from `check_static_condition` (AC 6052's "crossed against")

`Condition` has **52** variants (`crates/card-types/src/cards/card_definition.rs:3676-3927`;
cross-checked against the exhaustive no-`_` match in `condition_is_queue_time_evaluable`,
`effects/mod.rs:10127-10204`). Eleven reach a layer-resolved query:

| variant | call site | dispatch |
|---|---|---|
| `YouControlNOrMoreWithFilter` | `effects/mod.rs:10259` | handled directly in `check_static_condition` |
| `YouControlPermanent` | `effects/mod.rs:9682` | via the `_ =>` arm → `check_condition` |
| `OpponentControlsPermanent` | `effects/mod.rs:9693` | ″ |
| `ControlLandWithSubtypes` | `effects/mod.rs:9791` | ″ |
| `ControlAtMostNOtherLands` | `effects/mod.rs:9807-9809` | ″ |
| `ControlBasicLandsAtLeast` | `effects/mod.rs:9846` | ″ |
| `ControlAtLeastNOtherLands` | `effects/mod.rs:9867` | ″ |
| `ControlAtLeastNOtherLandsWithSubtype` | `effects/mod.rs:9885` | ″ |
| `ControlLegendaryCreature` | `effects/mod.rs:9898` | ″ |
| `ControlCreatureWithSubtype` | `effects/mod.rs:9909` | ″ |
| `OpponentControlsMoreLandsThanYou` | `effects/mod.rs:10077` | ″ — inside a per-player `count_lands` closure, so **N** battlefield sweeps per evaluation |

`Not` / `Or` / `And` (`effects/mod.rs:9768`/`9777`/`9969`) reach the same sites through their
operands; `check_condition` re-delegates five variants back to `check_static_condition`
(`:9938`/`9941`/`9944`/`9947`/`9950`), so a `YouControlNOrMoreWithFilter` nested under a
combinator still lands on `:10259`.

`check_static_condition` has exactly **one** production caller: `layers.rs:565`. Its only other
non-test references are the five self-delegations above and two card-def prose comments
(`ophiomancer.rs:37`, `jadar_ghoulcaller_of_nephalia.rs:57`). **The dangerous edge is a single
line.**

### 2.3 The supply side — what a base-characteristics read gets wrong

`matches_filter` (`effects/mod.rs:9533-9660`) reads nine `Characteristics` fields: `power`,
`toughness` (Layer 7), `card_types`, `subtypes`, supertypes via `legendary`/`basic`/`nonbasic`
(Layer 4), `colors` (Layer 5), `keywords` (Layer 6), `name`, and mana value (Layers 1/3). The
deviation is therefore **not** a "Layer-4 deviation" — the external doc's framing understates it.
Corpus supply, by bucket:

| bucket | cards | `Complete` |
|---|---|---|
| L4 — card types (`SetTypeLine`/`AddCardTypes`/`RemoveCardTypes`/`SetCardTypes`) | 20 | **12** |
| L4 — sub/supertypes | 18 | 14 |
| L5 — colors | 8 | 7 |
| L6 — keywords | 150 | 107 |
| L6 — other granted abilities | 8 | 6 |
| L7 — power/toughness (all `Modify*`/`Set*` arms) | 143 | 110 |
| **union, any layer** | **246** | **181** |

Applied to the *actual* filter the corpus's one condition carries — `has_card_type: Artifact`,
everything else `Default` — the supply set is every Layer-4 modification that can make base and
layer-resolved **Artifact** status disagree. **A "payload contains Artifact" filter is not that
set**: `SetCardTypes` and `SetTypeLine` are *replacements*, so they remove the Artifact type
precisely when their payload does **not** list it. Both directions:

| card | modification | completeness | direction |
|---|---|---|---|
| `blinkmoth_nexus` | `AddCardTypes(Artifact, Creature)` on `Source` | **Complete** | **adds** → **under-count**: an animated Nexus is an artifact creature and must feed Metalcraft (CR 613.1d) |
| `inkmoth_nexus` | `AddCardTypes(Artifact, Creature)` on `Source` | **Complete** | adds → under-count |
| `darksteel_mutation` | `SetCardTypes(Artifact, Creature)` on `AttachedCreature` | **Complete** | adds → under-count |
| `eaten_by_piranhas` | `SetCardTypes(Creature)` on `AttachedCreature` | **Complete** | **removes** → **over-count**. Oracle: *"(It loses all other colors, card types, and creature types.)"* |
| `kenriths_transformation` | `SetCardTypes(Creature)` on `AttachedCreature` | **Complete** | removes → over-count. Ruling (2019-10-04): *"loses any other card types it has (such as artifact)."* |
| `imprisoned_in_the_moon` | `SetTypeLine(Land)` on `AttachedPermanent` | **Complete** | removes → over-count. Oracle: *"loses all other card types"*; enchants creature, land **or** planeswalker |
| `vraska_betrayals_sting` | `SetCardTypes(Artifact)` on `DeclaredTarget` | `Partial` | **not deck-legal** — excluded |
| `oko_thief_of_crowns` | `SetTypeLine(Creature)` on `DeclaredTarget` | `KnownWrong` | **not deck-legal** — excluded |

**Hosts exist in the corpus for the removal direction**: 40 defs are printed Artifact Creatures
(**28** `Complete`), and 2 are printed Artifact Lands (`ancient_den`, `treasure_vault`, both
`Complete`).

**`blood_moon` / `magus_of_the_moon` are deliberately excluded, and the exclusion is a finding.**
Both are `Complete` and both register `SetTypeLine { card_types: [Land], subtypes: [Mountain] }`
over `AllNonbasicLands` — which strips the **Artifact card type** from an artifact land. The
printed card does not do that. The 2020-08-07 ruling on Blood Moon says the effect *"doesn't
affect names or supertypes"* and that nonbasic lands *"lose any other **land types** and
abilities"* — land **subtypes**, not card types; a Darksteel Citadel under Blood Moon is still an
artifact. So under the shipped deviation the base read of `ancient_den` is **accidentally
CR-correct**, and counting these as live-wrong pairs would be double-counting a *different*
defect. That def defect is real and independent of this adjudication — filed as **`OOS-ADJ-7`**.

### 2.4 A supply source neither document accounts for

`obj.characteristics` is **not** merely "pre-layer". `calculate_characteristics` rewrites its
local `chars` from the `GameObject` in **fourteen** ways before or outside the continuous-effect
loop, none of which is stored back on the object. The pre-loop ones are: suspected menace
(`layers.rs:70-72`), ring-bearer legendary (`:82-84`), **DFC back face** (`:97-139`), **meld**
(`:149-204`), **face-down / morph / manifest / cloak / disguise** (`:219-240`, CR 708.2a — empties
name, mana cost, subtypes, supertypes, colors, keywords and all three ability vectors, **replaces**
`card_types` with exactly `{Creature}` at `:223`, and sets 2/2), and **mutate** (`:246-248`, wholesale replacement by the topmost component). Inside the
loop: Changeling (`:255-260`), Devoid (`:268-270`), Impending (`:278-295`), Reconfigure
(`:302-319`), Living Metal (`:329-338`), ±1/+1 counters (`:372-400`). After it: the mutate ability
union (`:413-435`) and derived attack triggers (`:448-459`).

Two of these move the Artifact type on a deck-legal card:

| source | card | direction |
|---|---|---|
| DFC face flip (`:97-139`) | `thaumatic_compass` — front `Artifact`, back (Spires of Orazca) `Land`; **Complete** | **over-count** — a transformed Compass is not an artifact, but `obj.characteristics` still holds the front face |
| face-down override (`:219-240`) | any morph/manifest/cloak/disguise permanent | **over-count** — `obj.characteristics` still holds the *hidden* card's printed types, but a face-down permanent's only card type is Creature (CR 708.2a), so a face-down artifact card is counted as an artifact when it is not one |

`451e3517` names the face-down over-count in `characteristics_for_condition`'s doc comment and
lists CR 712.8d/e (DFC), 712.8g (meld), 729.2a (merge) and 702.73a (changeling) as **UNPINNED**
divergences. **No source names `thaumatic_compass`** — it is the concrete `Complete` card behind
the first of those four, found here.

### 2.5 The answer to AC 6052

**Deck-legal `Complete` combinations live-wrong under the shipped deviation: 7.**

`indomitable_archangel` × seven `Complete` supply cards:

| direction | supply cards | n |
|---|---|---|
| **under-count** (base says not-artifact, layers say artifact) | `blinkmoth_nexus`, `inkmoth_nexus`, `darksteel_mutation` | 3 |
| **over-count** via Layer 4 (base says artifact, layers say not) | `eaten_by_piranhas`, `kenriths_transformation`, `imprisoned_in_the_moon` | 3 |
| **over-count** via a non-continuous-effect base rewrite | `thaumatic_compass` (DFC face flip, §2.4) | 1 |
| **over-count**, unbounded | the CR 708.2a face-down class — any `Complete` card can be played face down, so this has no fixed population and is not tallied | — |

Each pair requires a **conjunction**: both cards in the same game, on the battlefield, controlled
by the same player, with the mis-typed permanent **pivotal** to the count of three. Four of the
seven are Auras, so they need a host of the right printed type as well — the corpus supplies 28
`Complete` artifact creatures and 2 `Complete` artifact lands for the removal direction. That
conjunction requirement, not the raw count, is what places this below the v3 queue's rank-2
through rank-12 entries in §5.2.

**This number superseded a published 4.** The first draft's supply filter asked whether a
modification's *payload contained* `CardType::Artifact`, which is structurally blind to the three
replacement effects that remove Artifact by not listing it. Recorded because the same blind spot
would recur in any future measurement written the same way: **for a replacement modification, the
payload is the whole answer, so "does not contain X" is as much a change to X as "contains X" is.**

**One pre-existing deviation of the same family already ships, banner'd**:
`Condition::DevotionToColorsLessThan` reads base `mana_cost` off battlefield permanents
(`effects/mod.rs:10356`), documented as a CR 700.5a deviation at `:10328-10335`. Three corpus
defs use it as a continuous-effect condition. It is the *only* other
`check_static_condition` battlefield sweep that deliberately skips the layer system; the other
five base-reading variants read graveyard/hand/library, where CR 400.2 makes printed
characteristics correct.

---

## 3. The proposed architecture vs the existing CR 613.8b machinery (AC 6053)

### 3.1 What the engine's 613.8 machinery actually is

- `resolve_layer_order` (`layers.rs:1747-1758`) partitions CDAs out (CR 613.3), sorts them by
  timestamp, and topo-sorts the rest.
- `toposort_with_timestamp_fallback` (`:1764-1854`) stable-sorts by timestamp (`:1785`), builds
  an O(n²) edge set (`:1791-1801`), runs Kahn with a `partition_point` insert that keeps the ready
  queue in timestamp order (`:1814`), and on `result.len() < n` `debug_assert!`s and then appends
  the residual in timestamp order (`:1835-1852`).
- `depends_on` (`:1863-1961`) implements CR 613.8a **(c)** literally (`:1864-1867`, CDA symmetry),
  takes **(a)** as a caller precondition (documented `:1858`, enforced by the per-layer filter at
  `:347`), and approximates **(b)** with a hardcoded five-arm `match` on
  `(&a.modification, &b.modification)` — **every arm Layer 4**. It receives no `&GameState`, no
  `object_id` and no `chars`, so it structurally *cannot* ask "would applying B change the set of
  objects A applies to."
- **`depends_on` never reads `effect.condition`.** `grep -n condition layers.rs` returns nothing
  in `1863-1961`; the only functional reads are `:558` and `:565`, inside `is_effect_active`.

**So the engine does not model condition-mediated coupling as a dependency at all.** It evaluates
every condition once, up front, outside the layer ordering, at `:43-47` — and that
outside-the-ordering evaluation is the recursion.

### 3.2 The CR verdict: treat-as-inactive vs timestamp order

**CR 613.8b, verbatim:**

> An effect dependent on one or more other effects waits to apply until just after all of those
> effects have been applied. If multiple dependent effects would apply simultaneously in this way,
> they're applied in timestamp order relative to each other. **If several dependent effects form a
> dependency loop, then this rule is ignored and the effects in the dependency loop are applied in
> timestamp order.**

**CR 613.8a**, clause (a): a dependency exists only if *"it's applied in the same layer (and, if
applicable, sublayer) as the other effect."*

**CR 611.3a**: a static-ability effect *"isn't 'locked in'; it applies at any given moment to
whatever its text indicates."*

Three findings follow.

**(i) The Archangel case is not a CR 613.8 dependency at all.** The Archangel's effect is
Layer 6 (`EffectLayer::Ability`, `AddKeyword(Shroud)`); its condition reads **card types**, which
are set in Layer 4. Layer 4 < Layer 6, so 613.8a(a) fails and no dependency relation is even
constructible. The CR-correct answer is supplied by the *fixed layer order* alone — **613.1**,
which lists the layers and fixes their sequence; that single rule does the work here, and neither
613.3 (CDAs first *within* a layer) nor 613.6 (an effect keeps applying to the same set of objects
across layers) bears on a cross-layer condition read. A Layer-6 effect's applicability is decided
against characteristics resolved through Layer 4 because Layer 4 has already run. The official
Wizards precedent for the same shape is the **Neurok Transmuter / March of
the Machines** ruling (2004-12-01) — *"March of the Machines depends on knowing what is and isn't
an artifact"* — which is the *same-layer* (both Layer 4) case and is therefore resolved by
dependency; the Archangel's is the strictly-cross-layer case and does not need it.

**(ii) "Treat the recursively-encountered effect as inactive" has no CR warrant — but 613.8b is
not the rule that denies it, and the distinction matters.** Two claims must be kept apart:

- *What the CR says about a **dependency loop**.* 613.8b is explicit and quoted above: timestamp
  order, not inactivity. But 613.8a(a) confines that rule to a **single layer**, and (i) has just
  established that the live case is cross-layer. **So 613.8b does not govern the Archangel case,
  and citing it as though it did would be the same category error this document is adjudicating.**
- *What the CR says about a **condition-evaluation cycle**.* Nothing. It is an artefact of an
  implementation that answers "is this effect active?" by a recursive query; the CR's model has no
  such query, only 611.3a's *"applies at any given moment to whatever its text indicates."*

**Verdict, stated at the strength the evidence supports: the CR is silent on condition-evaluation
cycles, so any cycle-breaker — treat-as-inactive included — is an undocumented deviation and must
ship labelled as one.** 613.8b is *evidence about the CR's disposition* when it does face an
unresolvable circularity (it picks a total order rather than deleting an effect), which is a
reason to prefer a timestamp-ordered tiebreak over suppression if the class ever becomes
non-empty — but it is evidence, not governing text.

On this the external doc is honest — *"This should not be declared a complete implementation of
every possible self-referential conditional continuous effect"* — and its §"Same-layer conditions"
proposal to *"detect this class through validation or debug assertions"* is a defensible
engineering posture that matches the engine's existing `debug_assert!` at `layers.rs:1835`. It is
simply not a rules implementation, and should not be described as one. Per (iii), the corpus
contains no case where the difference is observable today.

**(iii) Layer-bounding terminates by construction on this corpus — but only if the activity
check is bounded too, and neither document says so.** The one layer-querying condition sits on a
Layer-6 effect and reads a Layer-4 characteristic. Resolving layers 1–4 requires only effects in
layers 1–4, whose conditions (measured: none is layer-querying) cannot ask for layer 4 or later.
The recursion is not intrinsic to conditional statics; it is an artefact of asking for
**fully-resolved Layer-7d** characteristics when the question was about Layer 4.

> **Load-bearing precondition, stated here because it is stated nowhere else.** The external
> doc's own sketch collects effects **globally** — `state.continuous_effects.iter().filter(|e|
> is_effect_active_with_context(state, e, ctx))` inside `calculate_characteristics_inner`, with
> no layer filter. Adding a `through_layer` parameter to the *query* while leaving the *activity
> sweep* global does **not** terminate: a Layer-4-bounded query would still evaluate the Layer-6
> Archangel effect's condition, which re-enters the same bounded query. Termination requires that
> a `through_layer`-bounded walk evaluate the condition of **only those effects whose layer ≤
> `through_layer`** — which is also the semantically right thing, since an effect in a later layer
> cannot affect an earlier layer's output. §5.2 step 3 carries this as an explicit instruction.

With that precondition, **the external doc's `calculate_characteristics_through(state, id,
through_layer, ctx)` is the CR-correct architecture, and on this corpus it needs no cycle-breaker
at all.** Without it, it is the same recursion with an extra parameter.

The residual same-layer/backward case (a Layer-4 effect whose condition reads Layer 4 or 7) is
genuinely unaddressed by both documents *and*, so far as this task could establish from the MCP
rule text, under-specified in the CR itself for a **backward** (later-layer) read. It is empty in
the corpus today. §6 files it.

### 3.3 Assessment of the three proposed pieces

| piece | verdict |
|---|---|
| **eval context keyed by `EffectId`** | **Feasible** (D1) and **semantically strictly better than what shipped**: it suppresses *one* effect and leaves the rest of the layer system intact, so a nested query still sees Blinkmoth's animation, Darksteel Mutation's type set, and the Compass's flipped face. It would close **all seven** measured pairs and the face-down class. Its cost is threading `ctx` through `check_condition` (D10). Its residual CR gap is (ii): the suppression itself is a deviation, and its outcome is *evaluation-order-dependent* when two such effects coexist — reproducible (the `continuous_effects` iteration order is fixed) but not CR-derived. |
| **duration/condition split of `is_effect_active`** | **Accept, unreservedly.** ~10 lines against an existing seam (D8/D9), no semantic change, and it is the precondition for any layer-bounded design — the duration half must be answerable without a characteristics query or the bounding is circular. Cheapest correct piece in the whole proposal. |
| **layer-bounded `calculate_characteristics_through`** | **Accept as the target architecture** — it is the piece that makes the CR-correct answer *and* the termination argument fall out of the same fact (§3.2(iii)). Two corrections required: (a) `required_characteristic_layer` must cover **name** and **mana value**, which the sketch omits (D6); (b) the "highest layer required" is a property of the *filter instance*, not the `TargetFilter` type — a filter with only `has_card_type` needs Layer 4, and the corpus's one live filter is exactly that, so the common case is cheap. |

### 3.4 The thread-local rejection, adjudicated

The external doc rejects ambient state on four grounds: *"complicates tests, parallelism,
reentrancy, and future caching."* Against the shipped implementation
(`451e3517`: `layers.rs:34-61`, `:111-117`):

- **tests** — *does not survive contact.* `the_deviation_is_scoped_to_the_layer_walk_only` and
  `non_layer_path_reads_layer_resolved_power` test the ambient flag directly. Testability was
  demonstrated, not obstructed.
- **parallelism** — *inverted.* `cargo test` runs test fns on parallel threads; a `thread_local!`
  is the *correct* choice there and a `static mut`/`Mutex` would not be. The engine itself is a
  pure single-threaded library (Architecture Invariant 1: no async runtime).
- **reentrancy** — *handled.* A depth counter plus an RAII `Drop` is precisely the reentrancy-safe
  shape; it survives the several early returns (D11) and unwind.
- **future caching** — ***stands***. `characteristics_for_condition(state, obj)` is not
  referentially transparent: the same `(state, obj)` returns different answers depending on
  ambient depth. Any future memoization of `calculate_characteristics` keyed on `(state, id)` —
  a natural optimization given the O(n) battlefield sweep the Archangel condition performs on
  every query — would return a layer-degraded value to a non-layer caller. Nothing in the tree
  prevents this today.

**And the strongest objection is one the doc does not make.** A depth counter loses information
that an `EffectId` set retains: it suppresses **the entire layer system** rather than **the one
self-referential effect**. That is not a stylistic difference — it is the whole of §2.5's seven
live-wrong pairs. The doc reached the right conclusion (prefer explicit context to ambient depth)
for reasons that are three-quarters wrong about this codebase.

**Also material, and in the shipped fix's favour**: the thread-local touches neither
`GameState` (Architecture Invariant 2) nor the `Command` mutation path (Invariant 3), is not
serialized, not hashed, and cannot differ between two runs of the same command. PROTOCOL **33** /
HASH **70** were gate-executed and both are unmoved. Whatever replaces it must
clear the same bar.

### 3.5 Where the external doc improves on the shipped fix, and where it does not

**Improves**: closes all seven measured pairs plus the face-down class (§3.3); makes the CR
answer and the termination argument the same fact (§3.2(iii)); removes the referential-opacity
hazard (§3.4); makes the "multiple distinct conditional effects" test writable — which against a
depth counter it is not (S3).

**Does not**: it would not have shipped faster (it is a strictly larger change than the one that
landed the same day and closed a HIGH); its audit surface is 32.5% of the real one (C17); its
field list is incomplete (D6); it under-costs the `check_condition` threading (D10); and three
of its four thread-local objections do not hold here (§3.4). Its *"first ship the minimal
cycle-safe repair … then split duration from condition and introduce a layer-bounded query"*
sequencing is right in the abstract and **already one step further along than it knows** — the
minimal repair shipped on 2026-08-02.

---

## 4. What `scutemob-184` shipped, for the record

**Anchored to `451e3517`** (merge of `feat/pb-dx19-…` into main), **not** to this worktree's base.
Ten commits, `+2,821 / −96` across 16 files. The branch was still being edited when this task
began; every figure below is from the merged tree, and this section was rewritten once the merge
landed. Line numbers in this section are main's, not `62e5699a`'s.

- **`rules::layers::characteristics_for_condition(state, obj)`** (`layers.rs:111`) —
  `expect_characteristics` unless `in_layer_walk()` (`:59-61`), in which case
  `obj.characteristics.clone()`. Routed to **11** condition-evaluator sites in `effects/mod.rs` (10 in `check_condition`, 1 in `check_static_condition`, counted at `451e3517`; the commit message for `569087e6` says "14", which is not reproducible against the merged tree),
  as a boundary guard rather than a per-leaf conversion. Non-condition call sites in
  `resolve_amount` also route through it, where the guard is inert: `layers.rs` calls
  `resolve_cda_amount`, never `resolve_amount`, and `resolve_cda_amount`'s own filter arms already
  read base characteristics by a pre-existing documented choice — `EffectAmount::PermanentCount`
  at `layers.rs:2290`+, whose comment names recursive CDA evaluation as the reason.
- **`LAYER_WALK_DEPTH` / `LayerWalkGuard`** — thread-local `Cell<u32>` (`layers.rs:36`), RAII with
  `saturating_add`/`saturating_sub` on enter/`Drop` (`:47`, `:54`), entered at the top of
  `calculate_characteristics`. Plus a `debug_assert!` in `process_command` that the depth is
  **balanced at every command boundary** — a leaked depth is sticky for the thread and would
  silently downgrade every later condition read.
- **The closure was claimed once before it was true, and the fix for that is the most durable
  thing in the batch.** The routing pass was done by pattern replacement and missed three sites
  spelling the call `expect_characteristics(state, id)` because they destructure `(&id, obj)`:
  `ControlAtMostNOtherLands`, `ControlAtLeastNOtherLands`,
  `ControlAtLeastNOtherLandsWithSubtype`. The re-review **reproduced the original SIGABRT through
  one of them on a tree that already recorded `OOS-DX19-1` as CLOSED**. `569087e6` routed all 11
  and added a **source gate**, `no_condition_evaluator_resolves_characteristics_directly`
  (`tests/primitives/pb_dx19_characteristics_recursion.rs:899`), which brace-matches both
  evaluator bodies and fails on any `expect_characteristics` / `calculate_characteristics` call
  inside either — watched failing on a deliberately re-introduced miss, with a non-vacuity
  assertion so a renamed evaluator cannot pass by finding nothing. Its stated lesson: *"a closure
  achieved by editing every site you could find is a claim; a closure backed by a gate that fails
  when a site reappears is a fact."*
- **`OOS-SIM2-5` fold-in** — ten P/T arithmetic sites converted to `saturating_*`, and four
  `u32 → i32` widenings converted from `as` to `try_into().unwrap_or(i32::MAX)`. The `as` casts
  matter separately: they are not checked arithmetic even under `overflow-checks`, so a count
  above `i32::MAX` wraps the **sign** in every profile.
- **Evidence**: pre-fix `mtg-fuzzer --games 15 --seed 1` under `[profile.fuzz]` → SIGABRT, 0/15;
  post-fix 15/15. Two independent reverts, **both compiling**, isolating the recursion fix from
  the arithmetic fix. Tests 4,274/0/5 on branch. PROTOCOL 33 / HASH 70 gate-executed, unmoved.
- **Closes** `OOS-DP3-9` / `OOS-M11-3`'s stack-overflow half (the abort was immediate, so that
  row's "game-length-dependent" reading was an artefact of deck draw). The determinism half
  stands.
- **Files** `OOS-DX19-1` (the ten latent leaves; wants a boundary guard, *not* leaf conversion,
  because several are correct as layer-resolved on their real paths), **`OOS-DX19-2`** (*"the
  CR-honest fix is a CR 613.8b dependency-aware fixpoint — a batch of its own"*), `OOS-DX19-3`
  (P/T ceiling), `OOS-DX19-4` (depth tripwire as a named debug failure).

**One correction to `OOS-DX19-2`'s framing, which §3.2 establishes**: the follow-up is *not*
principally a CR 613.8b dependency fixpoint. 613.8a(a) confines dependency to a single layer, and
the live case is strictly cross-layer. The follow-up wanted is a **layer-bounded query**; the
613.8 fixpoint is a separate, currently-empty concern. Filed as `OOS-ADJ-3`.

---

## 5. Disposition (AC 6054)

**Verdict: accept the external review in substance; reject its sequencing; split the
deliverable in two, and rank the halves differently.**

The review is competent and its central technical objection is correct: the shipped fix trades a
crash for a documented CR 613.1d deviation, and that deviation is real, live, and reachable from
deck-legal `Complete` cards. It is also **seven pairs**, each requiring a two- or three-object
conjunction — which is why the *fix* does not belong near the top of the queue and the *gate*
belongs earlier than the fix.

### 5.1 PB-DX42a — the **corpus** roster gate (offered as a **rider on PB-DX8**, rank 10)

> **✅ SHIPPED 2026-08-12 as the rider on PB-DX8 (`scutemob-208`).**
> `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs`, 10 tests, test-only
> (`git diff --numstat` over `crates/engine/src`, `crates/card-types/src`, `crates/card-defs/src`,
> `crates/view-model/src`, `crates/simulator/src`, `tools/` is empty). Every number this section
> asked to be **re-derived at dispatch rather than transcribed** was re-derived, and every one
> matched: **382** `ContinuousEffectDef` nodes (206 under a `Static` ancestor, **176** reachable
> only by the structural field-set walk), **17** conditioned instances across **15** distinct cards
> and **9** distinct variants, and the layer-querying subset is exactly
> `{ Indomitable Archangel × YouControlNOrMoreWithFilter }`. Both non-vacuity floors are asserted
> (`>= 382`, `>= 17`), plus a third this section did not ask for and which the other two do not
> imply — **`>= 176` nodes reached with no `Static` ancestor**, because a walk that found only
> `AbilityDefinition::Static` nodes would clear both stated floors while missing the entire nesting
> class the structural walk exists for. The failure message states both legal exits and `t8` asserts
> that it does.
>
> **One correction to this section's premise, disclosed rather than papered over.** The gate pins
> the layer-querying set along **two independent axes** (source-read + structural), and the
> structural axis — *"does the condition's payload carry a `TargetFilter`?"* — is **not** a fully
> general proxy for *"reaches `characteristics_for_condition`"*. `Condition::ControlLandWithSubtypes`
> also reaches it, via `check_condition`'s ETB-replacement arm, while carrying no `TargetFilter`.
> It does not affect today's population — that variant never appears inside a
> `ContinuousEffectDef.condition` — but the agreement between the two axes rests on that
> coincidence, so `t7` pins the coincidence itself rather than assuming it, and both the module doc
> and `t6`'s failure message say so. **PB-DX42b (§5.2) inherits this**: its rank argument rests on a
> population of exactly 1, and the second axis alone would not notice a `ControlLandWithSubtypes`
> condition joining that population.

**First, what already exists, so this is not proposed twice.** `569087e6` shipped
`no_condition_evaluator_resolves_characteristics_directly` — a **source** gate over
`check_condition` and `check_static_condition`, which fails if either evaluator ever resolves
characteristics directly again. That closes the regression channel this batch's own routing pass
fell into, and it is the right instrument for that job. **It does not answer the question this
rider asks.** The source gate holds the *shape* fixed; it says nothing about the *population*.

**What is still ungated.** The deviation's blast radius is `{ card defs whose
`ContinuousEffectDef.condition` reaches a layer query }`, measured at exactly **1**. Nothing stops
that set growing. The next author who writes *"as long as you control a legendary creature, …"* as
a **static** routes correctly through `characteristics_for_condition`, passes the source gate, and
gets a silently wrong answer inside the layer walk — no failure, no warning, no signal that the
seven pairs are now more. `greymond_avacyns_stalwart`'s note, rewritten by `569087e6`, *invites*
exactly this ("It is safe to register now, at one cost…"). That is a defect *generator*, and a
corpus roster is the SR-36-shaped instrument for it.

**Scope.** One test file. Enumerate `all_cards()`; walk every `ContinuousEffectDef` **structurally**
(by field set, so it catches the 172 nested inside `Effect::ApplyContinuousEffect` at arbitrary
depth, not only `AbilityDefinition::Static`); collect condition variants, **descending through
`And`/`Or`/`Not`**; assert the layer-querying subset is exactly
`{ indomitable_archangel × YouControlNOrMoreWithFilter }`. **Non-vacuity floor required** — assert
the walk found ≥ 382 `ContinuousEffectDef`s and ≥ 17 conditioned ones, because the pinned set has
one member and a broken walk finding nothing would otherwise pass. (PB-DX6 precedent: two rosters
pinned **empty**, and that is the shape that rots silently.) The failure message must tell the next
author the choice: either the new condition is layer-safe, or the population has grown and
PB-DX42b's rank must be recomputed.

**Derive the pinned numbers from a fresh enumeration at dispatch, not from §2.1.** §2.1's first
published version mis-stated the per-variant census by two rows (it reported distinct *cards*
under a total counting *instances*). The corrected table is above; re-run rather than transcribe.

**Cost**: no engine change, no card-def change, no wire. `Complete` count unmoved. **Rider on
PB-DX8** (rank 10, "oracle-text-vs-DSL cross-check") because that batch is already a
corpus-scanning, test-only, gate-integrity batch — marginal dispatch cost ≈ 0. If PB-DX8 slips, it
rides PB-DX7.

### 5.2 PB-DX42b — layer-bounded condition queries (**insert at rank 13**)

**`PB-DX42: CR 613.1d layer-bounded condition queries — retire the `in_layer_walk` deviation` ·
CORRECTNESS**

Closes **`OOS-DX19-2`** (reframed per §4), **`OOS-DX19-1`**'s residue, and the new `OOS-ADJ-1`.

`rules::layers::characteristics_for_condition` (shipped by `scutemob-184`, merged `451e3517`)
returns `obj.characteristics` for **any** condition evaluated inside a `calculate_characteristics`
walk. That is a CR 613.1d deviation on **seven** measured deck-legal `Complete` pairs (§2.5) and
on the whole CR 708.2a face-down class, and it is asserted **in the wrong direction on purpose** by
`deviation_animated_nexus_does_not_count_toward_metalcraft`, whose message instructs this batch to
**invert** rather than delete it. `451e3517`'s doc comment on `characteristics_for_condition`
already names the four other divergences as **UNPINNED** (CR 712.8d/e DFC, 712.8g meld, 729.2a
merge, 702.73a changeling) — §2.4 supplies the concrete `Complete` card for the first of them
(`thaumatic_compass`), which no source currently names.

**Do it in three steps, in this order.**

1. **Split `is_effect_active`** (`layers.rs:508`) at its existing seam (`:553`) into
   `is_effect_duration_active` (`:509-552`, already free of any layer query — verified) and
   `is_effect_condition_satisfied` (`:556-573`). No behaviour change; this is the precondition
   for step 3 not being circular.
2. **Replace the ambient depth counter with an explicit context.** The `EffectId` exists
   (`continuous_effect.rs:533`) and is already the engine's chosen logical identity over
   `ptr::eq` (`layers.rs:1844-1845`). Thread `&mut CharacteristicEvalContext` through
   `calculate_characteristics_inner` → `is_effect_condition_satisfied` → `check_static_condition`
   → **`check_condition`** (this is the real cost: `check_condition` is `pub` with four other
   caller classes — `activation_condition`, `intervening_if`, `Effect::Conditional`,
   `unless_condition`; pass `None` there rather than duplicating the evaluator). Use an RAII guard
   for insert/remove, as `LayerWalkGuard` already does. **This step alone closes all seven measured
   pairs**, because suppressing one effect leaves the rest of the layer system intact.
3. **Add `calculate_characteristics_through(state, id, through_layer, ctx)`**, and **bound the
   activity sweep by the same `through_layer`** — evaluate `is_effect_condition_satisfied` only
   for effects whose layer ≤ `through_layer`. **This is what makes it terminate**; a bounded query
   over a global activity sweep is the original recursion with an extra parameter (§3.2(iii)).
   Then add `TargetFilter::required_characteristic_layer(&self)`, computed **per filter
   instance**, not per type. It **must** cover `has_name` (Layers 1/3) and `max_cmc`/`min_cmc`
   (mana value, Layers 1/3) in addition to power/toughness, card types, supertypes, subtypes,
   colors and keywords — `matches_filter` branches on all nine (`effects/mod.rs:9533-9660`), and
   the external doc's sketch omits the first two. `debug_assert!` when a condition's required
   layer ≥ its effect's layer; that class is **empty in the corpus** (§3.2(iii)) and the assert is
   how it stays visible.

**Tests.** Invert `deviation_animated_nexus_does_not_count_toward_metalcraft`. Add the three of
the external doc's eight that have no counterpart, **with fixtures that can actually produce the
phenomenon**:

- *Layer-4 **adds** Artifact* — `blinkmoth_nexus` or `inkmoth_nexus` animated, or
  `darksteel_mutation` on a nonartifact creature.
- *Layer-4 **removes** Artifact* — `eaten_by_piranhas`, `kenriths_transformation` or
  `imprisoned_in_the_moon` over one of the 28 `Complete` artifact creatures. **Not**
  `darksteel_mutation` (its `SetCardTypes` payload *is* `[Artifact, Creature]`, so an enchanted
  artifact stays an artifact) and **not** `thaumatic_compass` (a DFC face swap at `layers.rs`'s
  pre-loop rewrite, not a Layer-4 continuous effect — worth its own separate test, but it does not
  discriminate the Layer-4 path).
- *Distinct conditional effects nesting without mutual suppression* — the discriminating test for
  step 2, and **unwritable against a depth counter**, which is exactly the "single boolean" shape
  the external doc warns about.

Keep `non_layer_path_reads_layer_resolved_power`,
`non_layer_path_reads_layer_resolved_subtypes`, `sibling_condition_on_a_continuous_effect_terminates`
and `no_condition_evaluator_resolves_characteristics_directly` green — the last is `451e3517`'s
source gate and must survive the refactor, which means the new internal entry point has to keep
condition evaluators from resolving characteristics *directly* even as it gives them a bounded way
to do it *indirectly*. `the_deviation_is_scoped_to_the_layer_walk_only` needs rewording, not
deletion. Watch each new test fail by an **executed, compiling** revert.

**Wire: none expected.** No `Command`, `GameEvent` or `Effect` variant; `EffectId` is already
hashed; the eval context is call-stack state, not game state. Predict PROTOCOL 33 / HASH 70
unmoved and **gate-execute both** rather than predicting (the queue's standing ordering rule).

**Do not** revert `scutemob-184`. Its arithmetic fixes are orthogonal, its crash closure is real,
and its boundary-guard placement (one guard over 11 condition-evaluator sites, rather than converting leaves) is
the right shape — `OOS-DX19-1` explicitly warns that several leaves are *correct* as
layer-resolved on their real paths.

**Rank: 13**, between PB-DX28 (rank 12) and PB-DX29 (rank 13, which shifts to 14).

**Severity argument.** By the queue's own convention — *live-wrong on a deck-legal `Complete`
path first; then gate/evidence integrity; then cheap high-yield riders; then agency/quality* —
this is live-wrong on `Complete` cards and belongs in the first band. Within that band it ranks
**below** every current member on measured population:

| rank | batch | measured live-wrong population | conjunction required? |
|---|---|---|---|
| 2 | PB-DX20 | 13 `Complete` Auras + 1 Reconfigure, **unplayable on first contact** | no |
| 3 | PB-DX21 | 14 `Complete` vigilant creatures, corrupted by a normal client action | no |
| 5 | PB-DX23 | 1 def, but **permanent** draw-cadence corruption | no |
| 6 | PB-DX24 | 1 def (`nether_traitor`), unconditionally wrong | no |
| 7 | PB-DX25 | 6 `Complete` mutate defs × 24 counter defs, spell resolves anyway | pair, but **any** counter × **any** mutate |
| 12 | PB-DX28 | ≥14 `Complete` defs incl. 10 Karoos, **exploitable** | no |
| **13** | **PB-DX42b** | **7 pairs** (+ the unbounded face-down class) | **yes — both cards, same controller, the mis-typed permanent pivotal to a count of 3, and for 4 of the 7 a host of the right printed type as well** |

It ranks **above** PB-DX29 (rank 13→14, agency) and everything below, because those are agency,
capability or hygiene classes, and this is a rules-correctness class on deck-legal cards. It is
**not** promoted for the defect-generator argument, because §5.1 answers that argument for ~1/20th
of the cost and can ship four ranks earlier.

**Sequencing.** No hard dependency on any other batch. It should follow PB-DX42a so the roster is
pinned before the fix moves it — if the gate ships first and the population is still 1, this
batch's scope is exactly what §2 measured; if the population has grown by then, re-measure before
dispatch and re-rank upward.

### 5.3 Explicitly rejected

- **Reverting `scutemob-184` and shipping the external doc's design instead.** It closed a HIGH,
  its evidence is unusually strong (two compiling reverts, a decisive pre/post fuzz A/B), and
  its deviation is measured, bounded, pinned by a test and documented at the call site. Nothing
  here is worth a revert.
- **Promoting the fix into the top-8 band.** The measurement does not support it; §5.1 buys the
  urgency argument more cheaply.
- **Adopting "treat-as-inactive" as the engine's stated semantics for self-referential
  conditionals.** CR 613.8b says timestamp order (§3.2(ii)). If a cycle-breaker must exist as a
  backstop, it ships as a **documented deviation with a `debug_assert!`**, in the same register
  as the CR 700.5a devotion note and the `layers.rs:1835` loop assert — not as a rules claim.

---

## 6. Seeds identified (AC 6054)

Filed here for the collector to register in the canonical registry
(`docs/audits/decision-point-audit.md` §8.1). This task does not write that file.

| id | severity | statement |
|---|---|---|
| **`OOS-ADJ-1`** | MEDIUM | `characteristics_for_condition` returns base characteristics inside the layer walk, deviating from CR 613.1d on **7** measured deck-legal `Complete` pairs — `indomitable_archangel` × { `blinkmoth_nexus`, `inkmoth_nexus`, `darksteel_mutation` } (under-count) and × { `eaten_by_piranhas`, `kenriths_transformation`, `imprisoned_in_the_moon`, `thaumatic_compass` } (over-count) — plus the unbounded CR 708.2a face-down class. **Merges with `OOS-DX19-2`**; both are PB-DX42b. |
| **`OOS-ADJ-2`** | MEDIUM | Nothing gates the **size of the corpus population** carrying a layer-querying `ContinuousEffectDef.condition`. It is **1** today, measured. `451e3517`'s `no_condition_evaluator_resolves_characteristics_directly` gates the *evaluator source*, not the population, so a new conditional static passes it and silently joins the deviation. `greymond_avacyns_stalwart`'s rewritten note actively invites one. Wants the §5.1 corpus roster gate; this is the seed that justifies the rank-10 rider. |
| **`OOS-ADJ-3`** | LOW | `OOS-DX19-2` is framed as "a CR 613.8b dependency-aware fixpoint". CR 613.8a(a) confines dependency to a **single layer**; the live case (Layer-6 effect, Layer-4 condition) is strictly cross-layer and needs layer-bounding, not a fixpoint. A worker taking `OOS-DX19-2` at its word will build the wrong thing. Re-word at dispatch. |
| **`OOS-ADJ-4`** | LOW | `characteristics_for_condition` is not referentially transparent — the same `(state, obj)` returns different answers by ambient thread-local depth. Any future memoization of `calculate_characteristics` keyed on `(state, id)` (a natural optimization: the Archangel condition performs an O(n) battlefield sweep on **every** query) silently poisons non-layer callers. `451e3517`'s `process_command` balance `debug_assert!` catches a *leaked* depth; it does not catch this, and no comment names the hazard. |
| **`OOS-ADJ-5`** | LOW | The engine's CR 613.8 dependency relation is **Layer-4-only** (five hardcoded `depends_on` arms, `layers.rs:1868-1960`) and `depends_on` receives no `&GameState`, so 613.8a(b) ("would applying the other change *what it applies to*") cannot be evaluated. Dependencies in Layers 1/2/3/5/6/7a–7d resolve by pure timestamp. No test constructs a real dependency cycle or exercises the `layers.rs:1835-1852` fallback; `crates/engine/tests/rules/layers.rs:1251-1258` records in its own comment that the test formerly in that slot "claimed to test the 613.8b cycle but built no cycle at all". Pre-existing, out of scope here, and **not** what PB-DX42b fixes. |
| **`OOS-ADJ-6`** | LOW | No mechanism forces a `Condition` variant, when it is added or when its evaluator gains a characteristics read, to declare which layer it requires. `Condition` has 52 variants; 11 reach a layer query today and the classification lives nowhere but this document. The natural home is the `KeywordAbility`/SR-5 pattern — an exhaustive classification whose omission is a compile error. |
| **`OOS-ADJ-7`** | MEDIUM | `blood_moon.rs` and `magus_of_the_moon.rs` (both `Complete`) register `SetTypeLine { card_types: [Land], subtypes: [Mountain] }` over `AllNonbasicLands`, which **strips the Artifact card type**. The printed cards do not: the 2020-08-07 ruling says the effect *"doesn't affect names or supertypes"* and that nonbasic lands lose *"any other **land types** and abilities"* — subtypes, not card types. A Darksteel Citadel under Blood Moon is still an artifact. Live-wrong on 2 `Complete` defs against the corpus's 2 `Complete` artifact lands (`ancient_den`, `treasure_vault`). **Independent of the recursion work** — a card-def/`LayerModification` scope defect, and it should ride a card-def batch (PB-DX27), not PB-DX42b. Found only because §2.3's supply measurement was redone in both directions. |

> **↻ `OOS-ADJ-2` — PARTIALLY DISCHARGED, re-scoped 2026-08-14 by the seed re-rank v4
> (`scutemob-212`). Do not read the shipped PB-DX42a rider as closing it.** Two independent
> verifications during that task reached **opposite** verdicts on the same gate, and reconciling
> them is the finding. The gate genuinely works for what it covers: `t5_layer_querying_set_is_pinned`
> (`crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs:514-537`) pins the population
> **by name**, states both legal exits in its own failure text, and **fired on its first real
> event** — PB-DX27's The World Tree forced exit (b) rather than joining silently, which is exactly
> the hazard this seed predicted. But axis 1 (`:463-469`) filters on the **literal string**
> `"YouControlNOrMoreWithFilter"`, and axis 2 requires a `TargetFilter` payload that **eight of the
> eleven** layer-querying `Condition` variants do not carry (`ControlLandWithSubtypes`,
> `ControlAtMostNOtherLands`, `ControlBasicLandsAtLeast`, `ControlAtLeastNOtherLands`,
> `ControlAtLeastNOtherLandsWithSubtype`, `ControlLegendaryCreature`, `ControlCreatureWithSubtype`,
> `OpponentControlsMoreLandsThanYou`). `t7` (`:618-633`) pins exactly **one** of the eight absent
> and its own message explains why the structural signal cannot see it. **So the gate covers the
> population as it exists and is blind to seven of the eleven ways it can grow** — *a gate written
> for one variant measures that variant*, arriving at the gate written to close the seed that
> predicted it. **Re-scoped to those seven unpinned variants.** The widening is ~5 lines (extend
> `t7`'s pin to the eight-member set) plus 3 more beside `t9:732` for the `TargetFilter`
> fingerprint half that `OOS-DX28-1` describes and that `t9` does **not** currently cover; carried
> as a rider in the v4 queue (`memory/primitives/seed-rerank-2026-08-14.md` §4 rank 21), not as a
> reason to move PB-DX42b. Full derivation of the eleven-variant set: v4 memo §2.2, §2.3.

**Existing seeds this task touches, and how:**

- **`OOS-DX19-2`** — merged into `OOS-ADJ-1`; its framing corrected by `OOS-ADJ-3`.
- **`OOS-DX19-1`** — CONFIRMED and independently re-measured (11 layer-querying variants; only
  `YouControlNOrMoreWithFilter` is corpus-live as a continuous-effect condition). Its guidance
  ("do **not** fix it by converting the ten leaves — it wants a boundary guard") is endorsed.
  Closed by `569087e6` **on the second attempt**: the first closure was claimed on a tree where
  three sites spelling the call `expect_characteristics(state, id)` still bypassed the guard, and
  the re-review reproduced the original SIGABRT through one of them. All 11 condition-evaluator sites now route, and the closure is held by a source gate rather
  than by an edit sweep.
- **`OOS-DX19-4`** (depth tripwire) — CONFIRMED, and the external doc independently agrees a depth
  limit is legitimate *as a panic-prevention assertion* while not being the semantic fix. No
  conflict; keep as filed.
- **`OOS-SIM2-6`** — its stack-overflow half is closed by `scutemob-184`. `OOS-M11-3`'s
  determinism half is untouched by everything here.

---

## 7. Method, and what this adjudication does not establish

**Established by execution or by direct source read**: every file:line in §1 and §2.2, at
`62e5699a` (the pre-fix anchor — see the note in the header); the corpus counts in §2
(`all_cards()` dump, 1,803 defs, reproducible); the CR text and card rulings in §2.3 and §3.2
(MCP `get_rule` / `lookup_card`, verbatim); the shipped shape in §4, read at `451e3517` after the
merge landed mid-task, having first been read on the unmerged branch.

**This document was adversarially reviewed twice and materially corrected.** Twelve findings
came from an adversarial pass and five more from the acceptance-criteria review; the substantive ones are absorbed above and named where they landed, because a corrected
number with no record of the correction is the same failure mode this document criticises in
`OOS-DX19-1`'s first closure. The corrections that changed a published figure or instruction:

| what changed | from | to |
|---|---|---|
| live-wrong pairs (§0, §1.5, §2.5, §3.3, §3.4, §3.5, §5.2, §6) | 4 | **7**, after redoing the supply measurement in the *removal* direction (§2.5's note) |
| §4 routed-site count | 14 (taken from `569087e6`'s commit message) | **11** — counted at `451e3517`: 10 in `check_condition`, 1 in `check_static_condition`. The batch's own message is not reproducible against the merged tree |
| §1.5 S3 test-file size | "25 test fns" | **17** `#[test]` fns; the 25 counts helper `fn`s too |
| condition census (§2.1) | 9 rows summing to 15 under a stated total of 17 | instances-vs-cards separated; `SourceHasCounters` 3→4, `ControllerLifeAtLeast` 1→2 |
| §5.2 step 3 | layer-bounded *query* | layer-bounded query **plus a layer-bounded activity sweep** — without which it does not terminate (§3.2(iii)) |
| §5.2 test fixtures | "Darksteel Mutation over an artifact, or `thaumatic_compass` transformed" | neither can produce a Layer-4 Artifact removal; replaced with `eaten_by_piranhas` / `kenriths_transformation` / `imprisoned_in_the_moon` |
| §3.2(ii) verdict | "613.8b prescribes timestamp order, so treat-as-inactive is a deviation" | 613.8b does not govern a cross-layer case at all; the CR is **silent**, and 613.8b is evidence, not governing text |
| §4 | branch, 6 commits, +2,145/−89 | merged `451e3517`, **10 commits**, +2,821/−96 — including a source gate §5.1's first draft proposed as new work |
| §3.2(i) | "613.1, 613.3, 613.6" | **613.1** alone; 613.3 and 613.6 do not bear on a cross-layer condition read |

**Not established here**:

- **No test was run.** This task is read-only on `crates/` and `tools/`; the 7-pair claim in §2.5
  is a *corpus* claim (these cards exist, are `Complete`, and their effects move the Artifact
  type), **not** a claim that a live game was set up and observed producing the wrong Metalcraft
  count. PB-DX42b's stage 0 should observe it. The **direction** of each error is derived from
  `matches_filter`'s reads and each card's oracle text/rulings; that is sound derivation, not
  observation.
- **The 7 is a floor for its own filter, and the filter is narrow.** It answers "which `Complete`
  cards move the **Artifact** type", because Artifact is what the corpus's single live condition
  reads. Should the population of §5.1's roster ever exceed one, the supply side must be
  re-measured against whatever fields the *new* filters read — §2.3's per-layer table (246 cards,
  181 `Complete`, across Layers 1–7) is the ceiling that measurement would work down from.
- **The face-down class is not counted.** Any `Complete` card can be played face down, so it has
  no fixed population; it is listed qualitatively.
- **`OOS-ADJ-7` (Blood Moon) is filed on a rulings reading, not a full def audit.** The ruling text
  is quoted and the two artifact lands are enumerated, but no systematic pass was made over the
  corpus's other `SetTypeLine` uses for the same over-strip.
- **`OOS-ADJ-5` is reported, not audited.** The Layer-4-only dependency relation is a real
  limitation but was not systematically crossed against the corpus. Someone should.
- **The 11-variant reachability inventory (§2.2) was traced two to three levels deep** through
  `matches_filter`, `check_has_counter_type` and `calculate_devotion_to_colors` (all three are
  terminal leaves — no further layer call). Deeper indirection through a helper not on those
  paths would not have been seen.
- **The backward-layer class is unresolved, and that is narrower than §3.2(ii)'s verdict.** The
  MCP rule text settles the *forward* cross-layer case (fixed layer order, 613.1) and the
  *same-layer* case (613.8). It gives no explicit mechanism for a condition on a layer-L effect
  that requires a characteristic set in a layer **later** than L. That class is empty in this
  corpus. On *that* class specifically, read this document as saying only that the task did not
  locate the passage, if one exists — **not** that the CR is definitively silent. §3.2(ii)'s
  separate verdict, that the CR provides no rule for a *condition-evaluation cycle*, is a
  stronger claim and is made deliberately: a condition-evaluation cycle is an artefact of this
  engine's recursive-query implementation, and the CR's model contains no such query to legislate
  about.
