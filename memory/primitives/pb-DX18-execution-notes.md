# PB-DX18 — the trust boundary on ungated commands

**Task**: `scutemob-225` · v4 queue rank 10 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 10)
**Seeds**: `OOS-DP2-7`, `OOS-DP2-4`, `OOS-DP2-8`, `OOS-DX2-4`, `OOS-DX2-1`, `OOS-M11-5`
**Merge base**: `b59b4c83`

---

## §0 — Stage 0: measured before any production line changed

### §0.1 Baseline

`cargo test --workspace --no-fail-fast` on this branch **before any edit**:
**5,015 passed / 0 failed / 5 ignored**, **60** result-producing targets, residual list empty.
Reproduces PB-DX20b's close pin (`scutemob-222`) exactly. Name set saved (5,015 unique names,
no duplicates) for a byte-exact Python set difference at close — **not** `sort` + `comm`, which
fabricates a removal under a UTF-8 locale (`OOS-DX20b-5`).

### §0.2 All six cites reproduce at HEAD

| seed | filed cite | cite at HEAD | reproduces? |
|---|---|---|---|
| `OOS-DX2-4` | `rules/engine.rs:477-489` (row) / `:584-589` (brief) | `rules/engine.rs:584-591` — `Command::TakeMulligan` and `Command::KeepHand` each run `validate_player_exists` and nothing else | **YES** |
| `OOS-DP2-8` | `rules/commander.rs:802` / `:901` / `:864-868` | `handle_take_mulligan` at `:802` (no cap); `required_bottom = mulligan_count.saturating_sub(1)` at `:903`; the draw loop at `:854-874` `break`s on an exhausted library | **YES** |
| `OOS-DX2-1` | `rules/miracle.rs:44-106` | `handle_choose_miracle` at `:44`; validates hand-zone, `KeywordAbility::Miracle`, `cards_drawn_this_turn == 1` — and **not** which object was drawn | **YES** |
| `OOS-M11-5` | `casting.rs::validate_targets_inner` empty-requirements arm | `casting.rs:6298` (fn), `:6311` (`if !requirements.is_empty()` range-check skip), `:6332-6335` (`req_for_target` = all-`None`) | **YES** |
| `OOS-DP2-7` | `rules/replacement.rs:1084-1092` / `:1181-1200` (the 2026-07-31 correction) | `:1383-1391` and `:1480-1500` — **the corrected cites have drifted again, by ~300 lines**; both still push `GameEvent::LibraryShuffled` with no `Zone::shuffle` anywhere | **YES** |
| `OOS-DP2-4` | `rules/commander.rs:843`, `effects/mod.rs:4308`, `:4487`, `:10289` | all four exact, all four `rand::rngs::StdRng::seed_from_u64` | **YES** |

### §0.3 `OOS-M11-5` — the caller census BY CITE (the row asks for exactly this)

`validate_targets_inner` has **six** production call sites (two wrappers plus four direct).
The question is which of them can reach the arm with `requirements.is_empty()` **and** a
non-empty `targets` list. Derived by reading each site, not from the in-source comment:

| # | site | requirements it passes | can be empty with targets? |
|---|---|---|---|
| 1 | `rules/casting.rs:3835` (`handle_cast_spell`, via `validate_targets_with_source`) | `announced_requirements` | **YES — the only one** |
| 2 | `rules/abilities.rs:502` (activated ability) | `target_requirements` | no — the call is inside `else if !target_requirements.is_empty()` |
| 3 | `rules/engine.rs:3792` (loyalty ability) | `ability_targets` | no — the call is inside `if !ability_targets.is_empty()` |
| 4 | `rules/queries.rs:443` (`legal_targets_per_slot`) | `std::slice::from_ref(req)` | no — one element by construction |
| 5 | `rules/resolution.rs:7632` (CR 702.140b mutate re-check) | `&[mutate_target_requirement()]` | no — one element by construction |
| 6 | `rules/retarget.rs:155` / `:173` (CR 115.7a) | `so.target_requirements` (recorded at cast) | only if a stack object was *recorded* with the defect — a **consumer** of it, not a legitimate producer |

`pub(crate) fn validate_targets` (`casting.rs:6249`) is `#[allow(dead_code)]` and has zero
callers, so it is not a seventh site.

**The in-source justification is stale, and the row predicted it.** The comment at
`casting.rs:6308-6310` says the empty case is *"used by auras/bestow which validate via a
separate enchant path"*. PB-DX20 (`scutemob-198`) made `aura_spell_target_requirements`
(`casting.rs:5770`) synthesize a real `TargetRequirement` from `KeywordAbility::Enchant`, so an
Aura reaches site 1 with a **non-empty** list. The synthesis has four preconditions
(`casting.rs:5766-5771`); the only surviving way an Aura arrives with an empty list is
precondition 4 failing — an Aura with **no Enchant keyword at all** — measured by PB-DX20 at
**2 `inert` defs**, re-derived by this batch in `r2`.

**Blast radius (the memo's widening).** `casting.rs:4873` calls
`rules::events::push_target_announcement`, which since PB-DX48 derives
`GameEvent::PermanentTargeted` from `stack_obj.targets` and dispatches CR 702.21a Ward. So a
spurious target on a genuinely targetless spell **fires Ward**. Population re-derived in `r3`.

### §0.4 `OOS-DP2-4` — the four sites and their three different contracts

| site | contract on a missing library zone |
|---|---|
| `rules/commander.rs:840-845` | `expect_zone_mut(..).ok_or(GameStateError::ZoneNotFound(..))?` — **propagates** (MR-M9-12) |
| `effects/mod.rs:4305-4311` | `if let Some(zone) = state.expect_zone_mut(..)` — **swallows** |
| `effects/mod.rs:4484-4490` | `if let Some(zone) = ...` — **swallows** |
| `effects/mod.rs:10286-10291` | `if let Some(zone) = ...` — **swallows** |

**The seed names ONE re-permutation channel and there are TWO.** Its addendum says `StdRng` is
not algorithm-stable across `rand` majors. True — and `Zone::shuffle`
(`crates/card-types/src/state/zone.rs:138`) draws its indices with `rng.random_range(0..=i)`,
whose *sampling* algorithm is equally unpinned by `rand`'s own stability policy. Pinning only
the generator leaves the identical defect one layer down. Both are pinned here.

### §0.5 `*_SEED` census re-derived (the brief says 17, the memo says 18+)

`grep -rhoE '\b[A-Z][A-Z0-9_]*_SEED\b'` over `crates/` + `tools/` gives **17 distinct
identifiers**, of which **one is not a seed at all** —
`MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` is a ceiling. So the real figure is **16**:
`COMBAT_SEED`, `DISTINCTIVE_SEED`, `DX15A_SEED`, `DX20B_SEED`, `DX20_T6_SEED`, `DX45H_SEED`,
`ENG1_SEED`, `REBUILD_FAILURE_SEED`, `SIM1_SEED`, `T5_DX23_SEED`, `TARGET_SEED`, `UI1_SEED`,
`UI2_SEED`, `UI3_SPLIT_COMBAT_SEED`, `UI6_RESTRICTED_SEED`, `UI6_SEED`.

**And the `*_SEED` axis is the wrong axis, which is the more useful correction.** What a PRNG
pin can move is a *shuffle permutation*, and `Zone::shuffle` is called from exactly the four
sites above plus two test files. The simulator's opening deal
(`crates/simulator/src/setup.rs:424`, `deck.rs:94/101/133`, `fuzz_setup.rs:212`) uses
`rand::seq::SliceRandom::shuffle` on a `Vec` with its **own** `StdRng` and is **not** one of the
four sites — so the pin does **not** re-deal opening libraries, which is where every `*_SEED`
fixture's sensitivity lives. Predicted blast radius is therefore **small**, and it is *measured*
at close rather than asserted (`OOS-DX21-6`: a moved pin is a measurement, and "nothing moved"
must be measured, never predicted).

### §0.6 `OOS-DX2-1` — the offer channel, and why it is FILED rather than shipped

`grep -rn ChooseMiracle crates/simulator/src tools/` = **0**, exactly as the v4 memo says. The
only non-engine producer in the tree is `testing/replay_harness.rs:843/:853` (the script path).

**Decision: FILE it (`OOS-DX18-*`), do not ship it, and the reason is a CR reading rather than a
budget.** The AC allows shipping "if one LegalAction away". It is not. PB-DX23 could ship dredge
as a `LegalAction` because **CR 702.52a supplies a legal default** — *"you may instead"* — so a
missed offer is a decision the engine can make for the player without changing the game. **CR
702.94a has no such default**: the reveal happens *"as you draw it"*, and a reveal offered at the
next priority grant is a different game action taken with information the player did not have at
the draw (everything the rest of the draw step revealed). Making it honest needs a
`BlockingDecision` variant — a wire change this batch's own gates pin as UNMOVED — which is a
separate batch. Population re-derived from `all_cards()` in `r4`, never from a grep (SR-36).

### §0.7 CR readings that decide the design

* **CR 103.5** — *"Once a player chooses not to take a mulligan, the remaining cards become that
  player's opening hand, and **that player may not take any further mulligans**."* The
  per-player termination is explicit CR text, not a nicety.
* **CR 103.5** — *"A player can take mulligans until their opening hand would be zero cards,
  after which they may not take further mulligans."* With CR 103.5c's free first mulligan and a
  starting hand size of 7, `required_bottom = mulligan_count - 1`, so the opening hand after the
  Nth mulligan is `7 - (N - 1)`; it first reaches zero at **N = 8**. The 8th mulligan is
  therefore the **last legal one** and a 9th must be refused — and at N = 8 `KeepHand` is
  satisfiable (`required_bottom = 7`, hand = 7). At N = 9 it is not, which is the seed's own
  symptom.
* **CR 702.94a** — *"You may reveal this card from your hand **as you draw it** if it's the first
  card you've drawn this turn."* Two conjuncts; the engine checks only the second.

### §0.8 WIRE PREDICTION — committed before any production line changed

**Predicted: HASH 80 → 81 (ONE bump for the whole PB). PROTOCOL 41 UNMOVED.**

*Reason PROTOCOL cannot move:* `crates/engine/tests/core/protocol_schema.rs:116-117` lists
`CLOSURE_MUST_NOT_CONTAIN = ["GameState", "PlayerState", "StackObject", "CardDefinition"]`, and
this batch's two new stored fields live on `GameState` and `PlayerState` respectively. Neither
type is reachable from the `Command` / `GameEvent` / `Effect` / `Characteristics` closure, so no
type, variant or field is added to the wire. The new `PregamePhase` enum is referenced only from
`GameState`. This is the PB-DX21 precedent verbatim (`CombatState.attackers_declared`, HASH
72 → 73, PROTOCOL 35 unmoved).

*Reason HASH must move, exactly once:* both new fields are inside the `GameState` serde closure,
which `hash_schema.rs`'s `decl_fingerprint` source-scan digests, and both are hashed by
`HashInto`, which `stream_fingerprint` digests. Two fields plus one new enum are ONE schema
change; one appended `HASH_SCHEMA_HISTORY` row covers all of it. **Type counts predicted
unchanged where they are counted, and the new `PregamePhase` type is predicted to ADD ONE to the
GameState serde closure's indexed-type list** (it is a genuinely new type, unlike PB-DX20b's
field-only change).

*The two fields, designed together so one bump covers both:*

1. `GameState.pregame: PregamePhase` — `Mulligans { kept: OrdSet<PlayerId> }` | `GameStarted`.
   One field carrying **both** CR 103.5 properties: the phase boundary (`OOS-DX2-4`) and the
   per-player *"may not take any further mulligans"* termination.
2. `PlayerState.miracle_pending: Option<ObjectId>` — the just-drawn record CR 702.94a's first
   conjunct needs (`OOS-DX2-1`).

*Stop condition:* if either gate moves in a way this prediction does not explain — PROTOCOL
moving at all, or HASH moving twice — stop and re-derive before re-pinning anything.

### §0.9 Coverage prediction

**0 flips, and 0 card-def edits of any kind are expected** except the `darksteel_colossus`
completeness note and header comment `OOS-DP2-7` names, which are **comment-only** and move no
`Completeness` marker. Coverage predicted **unmoved at 1,137/1,803 = 63.1%**; proven by
regeneration at close, never by an empty diff.

---

## §1 — `OOS-M11-5`: what the row and the memo did not know

### §1.1 The row's own justification is stale for BOTH named cases — verified, not assumed

The memo says PB-DX20 made the aura half stale. It did, and the **bestow** half is stale too,
for a reason a reader is likely to get wrong: `handle_cast_spell`'s Step 1b
(`casting.rs:999-1004`) applies CR 702.103b's transform — *"as a spell cast bestowed is put onto
the stack, it becomes an Aura enchantment and gains enchant creature"* — to its **own** `chars`
binding, **before** `aura_spell_target_requirements` runs at `:3688`. There is a second,
post-push transform at `:4760` that applies the same change to the stack OBJECT; reading only
that one (which is what a `grep casting_with_bestow` surfaces first) leads to the wrong
conclusion that bestow validates existence-only. It does not.

CR 702.103b is explicit that the requirement must exist: *"Because the spell is an Aura spell,
its controller must choose a legal target for that spell as defined by its enchant creature
ability and rule 601.2c."*

### §1.2 THE CENSUS WAS SHORT BY A WHOLE MECHANISM, AND IT IS **SPLICE**

Neither the registry row, nor the v4 memo cell, nor this batch's own §0.3 site table names it.
CR 702.47a: *"copy this card's text box onto that spell"* — so a spliced spell gains the spliced
card's **targets** (CR 601.2b). `AbilityDefinition::Splice` (`card_definition.rs:682-686`) carries
`cost`, `onto_subtype` and `effect` and **no `targets` field at all**, so
`card_def_target_requirements` cannot see them and the spliced target rode the existence-only
arm. Corpus population: **1** def, `glacial_ray` (measured by walking the defs, not by grepping
prose). Closing `OOS-M11-5` therefore *requires* shipping the splice contribution — a batch that
only added the rejection would have broken the one shipped splice card.

### §1.3 The rejection population is MEASURED, and 44 of 46 are a test-only shape

Instrumented the new rejection to print the casting object's `card_id` and ran the whole
workspace suite. **46 rejections. 44 have `card_id == None`** — the documented
`ObjectSpec::card()` naked-object gotcha, where a fixture builds a `CardDefinition` **carrying
the right `TargetRequirement`**, registers it, and never links it to the object it casts. Exactly
**2** have a real definition:

| card | why it was rejected | verdict |
|---|---|---|
| `boon_satyr` | golden script `layers/081_bestow_aura_then_falls_off.json` issues `"action": "cast_spell"` while its own metadata, notes and CR cites all say **bestow**. It passed only because the existence-only arm accepted the target the script declared. | **the script was a pin on `OOS-M11-5`** |
| `reach_through_mists` | golden script `stack/146_splice_glacial_ray.json`, the splice case of §1.2 | **a real missing mechanism** |

Architecture Invariant 9 makes the naked shape unreachable in a real game (every object in a game
has a `CardDefinition`), which is why this defect could sit behind 42 green tests: **the coverage
that would have caught it was resting on a shape production cannot produce** — `OOS-DX47-4`'s
class, arriving again.

---

## §2 — What shipped, seed by seed

| seed | verdict | shape |
|---|---|---|
| `OOS-DX2-4` | **CLOSED** | `GameState.pregame: PregamePhase` + one shared `validate_pregame_mulligan_allowed` on both commands |
| `OOS-DP2-8` | **CLOSED** | `MAX_MULLIGANS = STARTING_HAND_SIZE + 1`, derived from the same constant the draw loop counts to |
| `OOS-DX2-1` | **CLOSED** | `PlayerState.miracle_pending`, written unconditionally at the draw site, cleared on either answer and at `reset_turn_state` |
| `OOS-M11-5` | **CLOSED** | CR 601.2c rejection in `validate_targets_inner` + CR 702.47a splice targets + the SR-38 splice offer gate |
| `OOS-DP2-7` | **CLOSED** | the obligation rides on `ZoneChangeAction::Redirect` and is discharged after the move by `GameState::finish_redirect_shuffle` |
| `OOS-DP2-4` | **CLOSED** | one `GameState::shuffle_library_seeded`, and `rand` dropped from both crates |

### §2.1 Three fixture families were pins on the defects

1. **Three `rules::commander` mulligan fixtures** drove `KeepHand` → `TakeMulligan` on one
   player. CR 103.5 forbids that outright. Repaired **in place** (no test name changed) by
   branching each keep off a clone; the escalating-bottom-count property they exist for is
   unchanged, only the command ORDER is.
2. **Golden script `layers/081_bestow_aura_then_falls_off.json`** issued `"cast_spell"`
   while its metadata, its notes and all eight of its `cr_sections_tested` said **bestow**.
   It passed only on the arm this batch closed. Now `cast_spell_bestow`.
3. **`core::card_def_fixes::test_darksteel_colossus_shuffles_into_library`** asserted the
   `LibraryShuffled` EVENT and never the library — exactly as `OOS-DP2-7` says — and its
   fixture gave the player an **EMPTY library**, which is why the phantom was invisible:
   with nothing to permute, "shuffled" and "put on top" are the same state.

### §2.2 The decision on `OOS-DX2-1`'s missing offer channel: FILED, and why

The AC allows shipping it "if one LegalAction away". It is not, and the reason is a CR
reading rather than a budget. PB-DX23 could ship dredge as a `LegalAction` because
**CR 702.52a supplies a legal default** (*"you may instead"*), so a missed offer is a
decision the engine may make for the player without changing the game. **CR 702.94a has
none**: the reveal happens *"as you draw it"*, and an offer surfaced at the next priority
grant is a different game action taken with information the player did not have at the
draw. Making it honest needs a `BlockingDecision` variant — a wire change this batch's own
gates pin as UNMOVED. Population re-derived from `all_cards()`: **3** defs declare Miracle
(`terminus`, `temporal_mastery`, `reforge_the_soul`).

---

## §3 — The revert matrix: 13 rows executed, 13 discriminating

Each row edits ONE production line, runs the named target, and is restored.

| row | revert | target | result |
|---|---|---|---|
| R1 | `finish_redirect_shuffle` performs no shuffle | `card_def_fixes::test_darksteel…` | **RED** |
| R2 | a consumer calls `finish_redirect_shuffle(false, ..)` | `roster::r1` | **RED** *(after the fix — see below)* |
| R2b | a consumer drops the call entirely | `roster::r1` | **RED** |
| R3 | no pregame gate on `TakeMulligan` | `pb_dx18_pregame_command_gates` | **RED** |
| R4 | no pregame gate on `KeepHand` | `pb_dx18_pregame_command_gates` | **RED** |
| R5 | CR 103.5 cap disabled | `pb_dx18_pregame_command_gates` | **RED** |
| R6 | `ChooseMiracle` stops checking the just-drawn record | `mechanics_m_z::miracle` | **RED** |
| R7 | a decline no longer consumes the offer | `mechanics_m_z::miracle` | **RED** |
| R8 | `miracle_pending` assigned inside an `if let Some(..)` | `roster::r3` | **RED** |
| R9 | the CR 601.2c rejection disabled | `pb_dx18_targetless_spell` | **RED** |
| R10 | `glacial_ray`'s splice `targets` emptied | `c2f_splice…` | **RED** |
| R11 | the pinned PRNG seeded differently | `pinned_rng_tests` | **RED** |
| R12 | an Arcane host given a target of its own | `roster::r2` | **RED** |

**R2 IS THE ROW WORTH READING, BECAUSE IT DEFEATED THIS BATCH'S OWN GATE.** `r1`'s first
draft looked for the string `finish_redirect_shuffle` in the arm body. A consumer written
as `state.finish_redirect_shuffle(false, to, &mut events)` **contains that string, drops
the obligation completely, and left `r1` GREEN.** That is `OOS-DX47`'s `r3` shape — *a gate
keyed on a spelling measures the spelling* — committed inside the roster file whose own
module doc states the rule. It was found by **executing** the revert, not by reading the
gate. `r1` now requires the arm's own bound `shuffle_destination_after` to appear inside the
call's argument list; R2 and R2b are both RED.

Two further gate defeats were found the same way and fixed before shipping:

* `r1`'s first draft flagged `resolve_pending_zone_change`'s chained-redirect arm, which
  reads a destination and does **not** move the object — a false positive of the gate's
  SHAPE. Re-keyed on the mechanism (an arm owes a discharge only if it calls one of the
  three move helpers), with a companion assertion that exactly ONE non-moving arm exists,
  so the split cannot silently become "everything looks non-moving".
* `r4`'s first draft reported `core/decision_site_walk.rs` as an empty module. It is a
  shared HELPER with no tests of its own. The two shapes are different findings and are now
  asserted separately.

---

## §4 — Wire

**HASH 80 → 81, ONE bump. PROTOCOL 41 UNMOVED.** Both gate-computed
(`hash_schema` 36/36, `protocol_schema` 17/17), both predicted in writing at `82154219`
before any production line changed, and both taken from the failing gates' own output.

The prediction survived something it did not anticipate: `AbilityDefinition::Splice` gained
a field mid-batch (the CR 702.47a discovery). PROTOCOL still did not move, because
`AbilityDefinition` is reachable only through `CardDefinition`, which
`CLOSURE_MUST_NOT_CONTAIN` also excludes — and it moved the STREAM digest but not the
DECLARATION digest, because `card_registry` is `#[serde(skip)]` and so `AbilityDefinition`
is outside the `GameState` **serde** closure while being inside the hashed one.

### §4.1 A NEW failure mode of the sentinel re-pin, and it is the OPPOSITE of the known one

PB-DX50 and PB-DX20b each recorded a re-pin regex that was too **narrow** (a multi-line
spelling, then a `Nu8` type suffix). This batch's regex handled both — and was too **wide**:
it rewrote the prose *"HASH 80 -> 81"* into *"HASH 81 -> 81"* inside the very doc paragraph
announcing the bump, because the literal `80` sat inside its window of the symbol.

**A survivor scan is structurally blind to this.** It looks for what was MISSED, and an
over-replacement leaves no survivor — this batch's survivor scan (a differently-shaped
line-window matcher over `crates/` + `tools/`) correctly reported **0**. The only thing that
catches an over-replacement is reading every changed line of the diff, which is what found
it. Census: **87 sites across 47 files** re-pinned by symbol.

---

## §5 — Standing measurements

* **Fuzz decision partition MOVED**, in the improving direction, and the cause is
  attributed by an **executed A/B** rather than argued: `surveil` is now reached (5 of 6
  served rows instead of 4 of 5). An isolated worktree at `e7dee121` — this batch's tree
  with everything except the PRNG pin — runs `test_dx32_a_fuzz_run_reaches_at_least_one_
  served_row` **GREEN**; `c1132e44` (that plus the pin alone) fails. Re-observed and
  re-pinned, never re-tuned (`OOS-DX21-6`).
* **The `*_SEED` axis is the wrong axis** (§0.5). The PRNG pin cannot move an opening
  library, because the simulator's deal uses `SliceRandom` with its own `StdRng` and is not
  one of the four sites; it moves only in-game shuffles. Measured: exactly **one** pin in
  the whole workspace moved.

---

## §6 — Benches: a REAL uniform regression, published as one

Four runs, matched sets, each revision in its own `git worktree` with its own
`CARGO_TARGET_DIR`. Merge base `b59b4c83`.

| bench | base run 1 | base run 2 | HEAD run 1 | HEAD run 2 | verdict |
|---|---|---|---|---|---|
| `priority_cycle_4p` | 24.590 µs | 23.805 µs | 25.365 µs | 24.772 µs | **~+2.5%, real** |
| `priority_cycle_6p` | 38.086 µs | 37.577 µs | 39.782 µs | 39.420 µs | **~+4.0%, real** |
| `sba_check` | 14.400 µs | 15.044 µs | 15.390 µs | 15.395 µs | **~+2.3%, marginal** |
| `full_turn_4p` | 217.06 µs | 216.42 µs | 224.78 µs | 228.13 µs | **~+4.5%, real** |
| `full_turn_6p` | 345.82 µs | 340.88 µs | 351.11 µs | 353.24 µs | **~+2.5%, real** |
| `board_wipe_4p` | 117.85 µs | 117.23 µs | 120.93 µs | 120.62 µs | **~+2.7%, real** |

**The same-code repeatability band was measured before the verdict was written, not
assumed** (PB-DX20b's lesson, where "everything 2-4% faster" turned out to be
contamination wider than the effect). The two base runs differ by **3.3%** on
`priority_cycle_4p` and **4.5%** on `sba_check`, and by under 1.5% on the other four. On
five of six benches the HEAD interval does not overlap either base interval, so the
regression is real; on `sba_check` the effect sits inside the same-code band and the
honest verdict there is **marginal**.

**The uniformity is the informative part, and it points at a mechanism on EVERY path
rather than at anything this batch put on a hot path.** Nothing here touches the SBA loop,
the priority cycle or combat: the pregame gate runs on two commands a game never sends,
the CR 601.2c rejection is one `is_empty()` on the cast path, and
`finish_redirect_shuffle` is a `bool` test inside redirect arms that fire only when a
replacement applies.

**One candidate mechanism is bounded by measurement rather than argued.**
`size_of::<GameState>()` moves **3512 → 3536** (+24 bytes) and `size_of::<PlayerState>()`
moves **360 → 376** (+16 bytes), executed at each revision. `GameState` is cloned on every
`process_command` and `PlayerState` is copied on every mutation through the `OrdMap`, so a
**+4.4% `PlayerState`** is on literally every benched path, which is consistent with a
uniform few-percent. `public_state_hash` also gains one enum discriminant and one
`Option<ObjectId>` per player per call, and loop detection hashes state on every priority
cycle and SBA batch.

**Stated rather than mitigated.** Both new fields are load-bearing state, not caches: the
pregame phase is the only thing that can distinguish "before the game began" from "turn
14", and the just-drawn record is CR 702.94a's first conjunct. Shrinking
`Option<ObjectId>` to a sentinel `ObjectId(0)` would save 8 bytes and trade a measured
correctness gate for an unmeasured 8-byte saving, which is the wrong trade to make
silently. Recorded here so the next batch measuring these numbers knows where the step
came from.

---

## §7 — Stated non-changes and disclosed residuals

Recorded here rather than left for a reader to notice missing.

### §7.1 The mulligan OFFER layer is unchanged, and that is not a new SR-38 problem

`crates/simulator/src/legal_actions.rs` offers `LegalAction::TakeMulligan` / `KeepHand` only
when `state.turn().is_first_turn_of_game && state.turn().turn_number == 0`, and
**`GameStateBuilder` defaults `turn_number` to 1 with nothing in the tree setting it to 0** —
`crates/simulator/src/local_game.rs`'s own `decision_kind_for` doc says so about its
`DecisionKind::Mulligan` arm ("currently unreachable"). So the offer is dead code today, and
this batch's gate can only refuse a command the offer never makes. Deliberately **not**
re-pointed at `state.pregame()`: doing so would start offering both commands in every
`GameStateBuilder`-built simulator fixture (all of which have `pregame = Mulligans`), which
is the SR-38 hazard in the other direction.

### §7.2 The gate's reach is bounded by who calls `start_game`, and that is stated

`PregamePhase::GameStarted` is set by `rules::engine::start_game_allowing_incomplete`, which
`start_game` delegates to — both documented entry points, so neither can drift. Production
callers: `simulator::local_game::LocalGame::start` (the browser and every bot game) and
`tools/tui/src/play/app.rs`.

**`crate::testing::replay_harness::build_initial_state` does NOT call it**, so a golden
script's state stays `PregamePhase::Mulligans` for its whole run and a script issuing
`TakeMulligan` mid-script would still be accepted. That is deliberate — it is what makes this
change **safe in the refusing direction only** (the gate can refuse more than HEAD did and
never accept more), and it is why the whole 208-script corpus is unaffected. The same is true
of every `GameStateBuilder` fixture. Said plainly rather than left implicit: the trust
boundary is closed on every path that actually starts a game, and open on the two paths that
deliberately assemble a mid-game position without one.

### §7.3 `Option<ObjectId>` was not shrunk, and the reason is stated

`PlayerState` grows 16 bytes because `ObjectId` is a `u64` newtype with no niche. A sentinel
`ObjectId(0)` would save 8 of them and trade a measured correctness gate for an unmeasured
saving; not taken, and recorded so the next reader knows it was considered.

### §7.4 The `r1` gate's residual, stated in the gate itself

`r1` walks `crates/engine/src` and `r1b` checks that claim by proving no other crate names
`ZoneChangeAction::Redirect`. What neither can catch is a consumer that moves the object
through a helper not in `MOVE_HELPERS` — so `r1`'s non-moving-arm count is asserted at
exactly **1**, which turns "a new move helper appeared" into a red test rather than a silent
gap.
