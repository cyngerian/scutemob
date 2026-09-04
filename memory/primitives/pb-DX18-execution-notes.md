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
