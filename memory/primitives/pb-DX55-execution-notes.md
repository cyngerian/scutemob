# PB-DX55 — execution notes

v4 queue rank 19. Seeds: **OOS-SIM6-3**, **OOS-SIM5-3**, **OOS-SIM5-5** — none of which had a
registry row at dispatch (the v4 memo's 61-of-208 blind spot, instantiated a second time after
PB-DX42b).

---

## §0 Stage 0 — measured before any edit

### 0.1 Pre-edit baseline

`cargo test --workspace --no-fail-fast` to a file: **5,243 passed / 0 failed / 5 ignored** across
**69** result-producing targets. This **reproduces PB-DX42b's published close pin EXACTLY** — the
sixth consecutive batch in which an inherited pin reproduces with no correction owed.

### 0.2 Registry grep (dispatch hygiene 5)

`grep OOS-SIM6-3 / OOS-SIM5-3 / OOS-SIM5-5 docs/audits/decision-point-audit.md` — **none of the
three has a row of its own.** Every hit is a *mention inside another seed's cell*: all three are
named in `OOS-DX32-9`'s ranked-defect-list cell (`:1318`), `OOS-SIM6-3` additionally inside
`OOS-DX45-8` (`:1470`) and `OOS-DX36-2` (`:1583`). A seed named in another row's prose is not a
filed seed, which is what dispatch hygiene 5 exists to catch. All three are FILED by this batch.

### 0.3 The refusal surface at HEAD — the memo's §2.6 table does NOT reproduce

Exact §2.6 invocation: `cargo test -p mtg-simulator --test sim5_bot_cast_discipline -- --nocapture`,
seeds 0/7/42, `AB_MAX_TURNS = 25` (the games run to turn **26**, which is the memo's "26 turns"),
4 `HeuristicBot` seats.

| class (normalised) | seed | §2.6 | **at HEAD** | share |
|---|---|---|---|---|
| `InvalidTarget("modal … per-mode … CR 700.2c")` on `activate` | `OOS-SIM5-5` | 2 | **22** | 31.4% |
| `InsufficientMana` on `activate` | `OOS-SIM6-3` | 76 | **18** | 25.7% |
| `CrossPlayerBlock` | `OOS-SIM5-3` | 14 | **9** | 12.9% |
| `InvalidCommand("The attacking player cannot declare blockers")` | `OOS-SIM5-3` / `OOS-DX51-3` | 13 | **9** | 12.9% |
| `InvalidTarget("expected 1..=1 target(s) but got 0")` | **RESIDUE** | 0 | **10** | 14.3% |
| `InvalidCommand("… cannot block … (attacker has flying …)")` | `OOS-SIM5-3` | — | **2** | 2.9% |
| **total** | | **105** | **70** | |

Per seed: 16 (seed 0) / 29 (seed 7) / 25 (seed 42).

**Three refutations of the memo, each stated rather than reconciled away:**

1. **`OOS-SIM6-3` is 18, not 76.** Its share falls 72.4% → 25.7% and it is no longer the largest
   class. The memo's own cite correction — *"the filed figure '62 of 113' is now **76 of 105** —
   larger in share and in count"* — is itself now stale, in the other direction. The seed is still
   real and still the human-facing one; what has changed is that it is no longer 72% of anything.
2. **`OOS-SIM5-5` is 22, not 2 — it has grown 11× and is now the LARGEST class**, and its message
   text has changed shape. §2.6 measured a bare `1..=1 got 0`; at HEAD the message reads
   *"modal spell with per-mode targets requires exactly 1 target(s) for the chosen mode(s) but got
   0 (CR 700.2c)"*, which is **PB-DX35's** per-mode scoping making the requirement enforceable.
   PB-DX35 made the per-mode requirement REAL on one axis and left the query that would let a
   caller satisfy it unchanged, so the class it could not announce got bigger, not smaller.
3. **The memo's "residue 0" and "cast-side refusals are zero of any kind" are BOTH REFUTED.** Ten
   refusals are the bare `expected 1..=1 target(s) but got 0` class, **three of them cast-side**
   (2 on one unnamed object, 1 on `Revitalizing Repast` — a card that also casts SUCCESSFULLY with
   a target on the same seed, so the class is *no legal candidate at this moment*, not *cannot
   announce ever*). That is `OOS-SIM5-4`'s parked offer-suppression class, which §2.6 priced at
   **0 of 105** and which is worth **10 of 70** at HEAD.

A yield cell is a floor; a *share* cell is neither a floor nor a ceiling, and three of the four
rows in §2.6 moved in different directions in three weeks.

### 0.4 Wire prediction — WRITTEN BEFORE ANY PRODUCTION LINE

Predicted **PROTOCOL 44 / HASH 85 both UNMOVED — ZERO bumps for the whole PB**, per half, with the
reason for each rather than a bare assertion:

- **Half 1 (`OOS-SIM6-3`)** — the change is in `crates/simulator` (`local_game.rs`,
  `legal_actions.rs`). `LegalAction`, `ActivationCostPlan` and every other type it touches are
  **simulator** types, outside the `Command`/`GameEvent`/`Effect`/`Characteristics` closure both
  gates walk. No engine type, variant or field is added.
- **Half 2 (`OOS-SIM5-3`)** — extracting the blocker-legality predicate adds a **free function**
  to `crates/engine`. A free function is not a type; it adds nothing to either closure. The
  `LegalAction::DeclareBlockers` shape change is again a simulator type.
- **Half 3 (`OOS-SIM5-5`)** — `rules/queries.rs` is a **read-only query module and is off-wire**:
  it is not reachable from `Command`, `GameEvent`, `Effect` or `Characteristics`, it declares no
  serialized type, and `TargetRequirement` (what the extended query returns) has been in the wire
  closure since long before this batch. Critically, **`Command::ActivateAbility` ALREADY carries
  `modes_chosen: Vec<usize>`** (`command.rs:124`) — so the per-mode slice needs **no new command
  field**, which is the single fact that keeps this half off the wire.

Both gates are executed against the final tree and the prediction is reported as confirmed or
refuted, never quietly dropped.

---

## §1 The result — the refusal surface, measured at every stage

Every figure below was produced by the coordinator RE-RUNNING the instrument, never accepted
from a delegated report. The invocation is the memo's own:
`cargo test -p mtg-simulator --test sim5_bot_cast_discipline -- --nocapture`, seeds 0/7/42,
`AB_MAX_TURNS = 25` (games run to turn 26), 4 `HeuristicBot` seats.

| class | seed | stage 0 | after H1 | after H3 | **close** |
|---|---|---|---|---|---|
| `InvalidTarget("modal … CR 700.2c")` on `activate` | `OOS-SIM5-5` | 22 | 22 | **0** | **0** |
| `InsufficientMana` on `activate` | `OOS-SIM6-3` | 18 | **0** | 0 | **0** |
| `CrossPlayerBlock` | `OOS-SIM5-3` | 9 | 9 | 7 | **0** |
| `InvalidCommand("The attacking player cannot declare blockers")` | `OOS-SIM5-3` / `OOS-DX51-3` | 9 | 9 | 8 | **0** |
| `InvalidCommand("… cannot block … (attacker has flying …)")` | `OOS-SIM5-3` | 2 | 2 | 1 | **0** |
| `InvalidTarget("expected 1..=1 target(s) but got 0")` | **RESIDUE** | 10 | 9 | 9 | **9** |
| **total** | | **70** | **51** | **25** | **9** |

**All three seeds' classes are ZERO and every remaining refusal is ONE class.**

### The residue is itemised, and the memo's "residue 0" is refuted twice over

The nine survivors are 2 on seed 0 (`activate(p2)`), 7 on seed 42 (4 `activate(p1)`, 2 on an
unnamed object, 1 on `Revitalizing Repast`). **Three are CAST-side**, which §2.6 also says is
impossible (*"cast-side refusals are zero of any kind"*). `Revitalizing Repast` casts
SUCCESSFULLY with a target on the same seed, so the class is *no legal candidate at this
moment*, not *cannot announce ever* — which is `OOS-SIM5-4`'s parked offer-suppression class,
priced by §2.6 at **0 of 105** and worth **9 of 9** at close. Filed as `OOS-DX55-1`.

### Movement that is NOT this batch's fix, attributed rather than claimed

Between stages the untouched classes moved: the residue 10 → 9 after Half 1, and the blocker
classes 9/9/2 → 7/8/1 after Half 3, with seed 42's `ObjectId`s shifting (505 → 507, 521 → 523).
Nothing in Half 1 or Half 3 touches blocking. This is `OOS-DX21-6` trajectory reindexing — a
funded activation and an announceable mode each change what a bot does next — and it is stated
as attribution rather than counted as progress. **The delegated Half-1 report claimed "every
other class is byte-identical in count"; re-executing the instrument refuted that**, which is
why the coordinator re-runs rather than transcribes.

## §2 Fuzz — the PB-DX32 gate config, A/B'd against the merge base

Merge base `70cd2487` built in its own worktree with its own `CARGO_TARGET_DIR`, HEAD in this one.

| | merge base | HEAD |
|---|---|---|
| T2.2 rejections / commands, seeds [1,2,3] | **5 / 2,713 = 1.843‰** | **0 / 2,717 = 0.000‰** |
| T3.1 waste ratio, seeds [1,2,3] | 88 / 97 = 90% | 89 / 97 = 91% |
| T6.3 decision rows REACHED | **4 of 7** | **6 of 7** |

**The gate config's SR-38 rejection rate is zero.** That forced T2.2 to move its seeds — its own
`total_rejections > 0` non-vacuity floor cannot coexist with a measured zero, and a gate whose
floor is unsatisfiable has stopped discriminating — so it now runs `[6, 7, 10]` (51 / 2,572 =
19.829‰) under the SAME ceiling. **Moving the seeds is the right repair for T2.2 and the wrong
place to leave the result**, so the zero is pinned where it happened: new
`test_dx55_the_historical_gate_seeds_now_produce_zero_bot_rejections` asserts `== 0` on
`[1, 2, 3]` with a `total_commands >= 2,150` floor, because a bot that stopped acting also
reports zero. A ceiling of zero is the strongest ratchet this file can hold.

T6.3's reached set goes 4 → 6: `look_at_top_then_place_optional` and `surveil` JOIN (Half 1 — bots
could not previously pay activation costs, so the resolution paths behind those rows were
starved), and Half 2's trajectory shift takes `may_pay_then_effect` back out. Both attributed by
executed ablation, not argued. `decision_site_walk`'s partition is untouched: every row was
already SERVABLE.

## §3 Wire — PROTOCOL 44 / HASH 85, ZERO bumps, and the counterfactual EXECUTED

Both gates run green against the final tree (`hash_schema` 36/36, `protocol_schema` 17/17),
`git diff` over `state/hash.rs` and `rules/protocol.rs` is EMPTY, so no sentinel re-pin, no
history row and no frozen-prefix re-pin were owed. Closure type counts **MEASURED** by raising
each gate's `MIN_CLOSURE_TYPES` to 9999 and reading its own panic text: **HASH 132 / PROTOCOL
98**, unchanged.

**"Unmoved" only means something beside what would have moved it**, so the counterfactual is
verified by planting each type in both gates' `CLOSURE_MUST_NOT_CONTAIN` and running them:

| planted type | HASH gate | PROTOCOL gate |
|---|---|---|
| `TargetRequirement` | FAILS → already in the closure | FAILS → already in the closure |
| `ModeSelection` | FAILS → already in | FAILS → already in |
| `AttackTarget` | FAILS → already in | FAILS → already in |
| `CombatState` | FAILS → already in | **passes → NOT in** (reachable only through `GameState`) |

So every type the batch's new query surfaces traffic in was **already on both wires**, which is
why returning them adds nothing — and the load-bearing fact for Half 3 is that
`Command::ActivateAbility` already carried `modes_chosen` (`command.rs:124`), so no command field
was added. `rules/queries.rs` is a read-only query module and is off-wire: it declares no
serialized type and is not reachable from `Command`/`GameEvent`/`Effect`/`Characteristics`.

## §4 Benches — NOT measured, and the reason is a mechanism bound, not an estimate

`crates/engine/benches/engine_perf.rs` contains **zero** occurrences of any symbol this batch
touched (`check_block_pair`, `legal_blocks`, `ability_target_requirements`,
`per_mode_target_requirements`, `command_mana_cost`, `auto_tap_commands_for`,
`handle_declare_blockers`), and its only mention of `DeclareBlockers` is a doc line whose very
next sentence is *"No attackers are declared."* — so `handle_declare_blockers` is never called on
any benched path. Everything else this batch changed is in `crates/simulator` and `tools/`, which
the engine benches do not link. Stated as a bound checked by execution rather than as a
prediction; the fuzz A/B above is what covers the paths the benches cannot see.

## §5 Coverage — UNMOVED, 0 flips, 0 card-def edits

`tools/authoring-report.py` regenerated: **1,140 / 1,803 = 63.2%**, clean 1,140 / todo 516 /
empty 147 — every bucket identical to the inherited figure. Self-dating churn reverted.
`git diff --numstat 70cd2487..HEAD` over `crates/card-defs` and `crates/card-types/src/cards` is
**EMPTY**, so the shortcut was available and the regeneration was run anyway. 0 flips, and the
reason is that this batch authors no card text: it repairs the offer/query/funding layers that
sit between a `Complete` def and a client.

## §6 Tests — 5,286 / 0 / 5 on 72 targets

Baseline **5,243 / 0 / 5** on **69** targets, measured on the merge base in its own worktree, and
**reproducing PB-DX42b's published close pin EXACTLY** — the sixth consecutive batch in which an
inherited pin reproduces with no correction owed.

Delta itemised by test NAME by a **byte-exact Python set difference**, never `sort` + `comm`
(`OOS-DX20b-5`), with the extraction regex **NOT end-anchored** (`OOS-DX42b-6`, so an
`#[ignore = "reason"]` test whose line reads `... ignored, <reason>` is still extracted):
**43 additions, 0 leavers, 0 removals, 0 renames.** Count delta 43 == name-set delta 43, and the
duplicate-name scan the byte-exact method is structurally blind to (`OOS-DX35-8`) is **EMPTY on
both runs** (5,248 lines / 5,248 distinct; 5,291 / 5,291).

`clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
`tools/check-defs-fmt.sh` (1,803 defs) and `cargo build --workspace` (the SR-3 seal gate) all
clean **against the FINAL tree**, and `clippy` FIRED there on `doc_lazy_continuation` in a doc
line opening `1.` — an ordered-list item making the next line its lazy continuation, PB-DX39's
own case one punctuation mark over. Reworded, with the reason recorded at the line.

**`npm run build` was NOT run and that is stated rather than omitted.** Unlike the last several
batches the frontend DOES move here — `BlockerPicker.svelte` (+28/−11) and `ActionBar.svelte`
(+1/−0) — so the criterion is live rather than N/A; `node_modules` is absent from this worktree
(`test -d` executed), so the build cannot run at all. Reported as a gap, not as an N/A.

**Two standing gates fired on this batch's own work and both were answered rather than
weakened**: SR-5's `keyword_registry` site roster, twice (`legal_actions.rs` reads Vigilance for
the funding exclusion; `random_bot.rs` reads Menace for the prune), and SR-25's
`bare_lookup_ratchet`, whose `combat.rs` ceiling genuinely FELL 16 → 14 from the extraction and
was **lowered** rather than left stale-high — a stale-high ceiling is slack a regression hides in
(PB-DX49's rule).

## §7 Engine lines

`crates/engine/src` **+544 / −513**, of which `rules/combat.rs` is **+342 / −473** — a **net
reduction of 131 lines**, which is the second hand-rolled copy collapsing into the first.
`crates/simulator/src` **+857 / −119**; `tools/` **+464 / −29**; `crates/card-types`,
`crates/card-defs` and `crates/view-model` are all **EXACTLY 0**.

## §8 Revert matrix — 13 rows, EXECUTED BY THE COORDINATOR, all files restored byte-exactly

Every row patches production source, runs the batch's own probes across four targets
(`mtg-simulator`'s three new suites, `mtg-engine`'s `primitives` and `core` filtered to
`pb_dx55`, and `play-server`'s `test_dx55`), then restores and verifies with `cmp`.

| row | what it undoes | RED |
|---|---|---|
| **R0** | **CONTROL — no patch at all** | **0** (as required) |
| R1 | `auto_tap_commands_for` narrowed back to `CastSpell` (Half 1 whole) | **3** — `t1`, `t3`, and the HTTP probe |
| R2 | the self-tap funding exclusion dropped | **1** — `t1` alone |
| R3 | the query stops honouring chosen modes | **6** — `c1`/`c2`/`c3` + `t1`/`t7`/`t9` |
| R4 | `legal_blocks`' attacking-player fast path removed | **0** — see below |
| R4b | that fast path **and** the general `CrossPlayerBlock` arm removed together | **3** |
| R5 | the general `CrossPlayerBlock` arm alone | **1** |
| R6 | the offer goes back to the raw scan + flat cross product | **2** |
| R7 | `ability_default_modes` stops being CR 700.2a legality-aware | **1** |
| R8 | a genuine top-level `_ =>` wildcard planted in `command_mana_cost` | **4** incl. the `r1` ceiling gate |
| R9 | a PARTIAL hand-rolled per-pair block predicate planted in `combat.rs` | **0 before the fix / 1 after** |
| R10 | a SIXTH inline copy of `per_mode_target_requirements`' body | **1** |
| R11 | the CR 702.110a menace prune removed | **2** |

**R1 and R2 are precise complements**: R1 reddens all three funding probes, R2 reddens exactly
the one whose fixture puts a mana ability on the permanent that is about to tap itself. That is
the only way to show the funding widening and the self-tap exclusion are each load-bearing.

**R5, R6 and R4b are a complementary triple, and R4's zero is the row worth reading.** Removing
the attacking-player fast path from `legal_blocks` reddens **NOTHING** — and that is structural,
not a missing test: an attacker always attacks somebody else, so `check_block_pair`'s
`CrossPlayerBlock` arm already refuses every pair the attacking player could name, and the early
return is a documented fast path on top of it. Settled by execution rather than argued: with the
fast path AND that arm both removed (R4b) three probes go red, and with the arm alone removed
(R5) exactly the cross-player probe does. **The row that actually carries `OOS-DX51-3` is R6** —
the offer consuming the query instead of a raw battlefield scan.

### R9 is a GATE DEFEAT THAT SUCCEEDED, and it is this batch's durable half

`pb_dx55_block_legality_roster`'s `r1` claims *"exactly one per-pair block-legality predicate
exists in the workspace"*, deciding it by a threshold of 5 of 9 `MARKERS`. **Every one of those
nine is EXOTIC** — horsemanship, skulk, shadow, intimidate, fear, the `CantBeBlockedExceptBy`
filter internals, landwalk, protection. Planting a five-guard hand-rolled predicate in
`combat.rs` itself — controller, tapped, `CantBlock`, flying/reach, protection, i.e. someone
answering *"can this block that?"* for one local purpose and covering only the cases they had in
mind — scored **1 of 9** and left `r1` **GREEN**.

The file's own `r2`/`r3` self-defeats did not see it, and the reason is `OOS-DX54-6` verbatim:
both plant a WHOLESALE renamed copy of `check_block_pair`'s brace-matched body, which carries all
nine markers by construction. *A gate's self-test written by the same author from the same
mental model exercises the inputs that author already thought of.*

**A similarity gate keyed entirely on the RARE members of a set is blind to the partial copy, and
the partial copy is the likely one** — nobody hand-rolls their way to horsemanship. Closed by a
SECOND axis keyed on the COMMON guards, with its threshold chosen from a measurement taken before
the code was written (`check_block_pair` scores 8 of 8; `handle_declare_attackers` scores 2,
reading Flying/Reach for the ATTACKER-side question; nothing else scores above one — so a
threshold of 3 has five points of headroom below the real predicate and one above its nearest
neighbour). `r4` is the axis, `r5` is R9's own plant kept as a test so the repair cannot be undone
silently — and `r5` **asserts that the exotic axis still misses it**, so a later batch that widens
`MARKERS` is sent to re-read why axis B exists rather than quietly deleting it. R9 re-executed
against the fixed gate: **RED**. Filed as `OOS-DX55-3`.

### R8's first plant was a BAD PLANT, and it is disclosed rather than counted

The first R8 planted a guarded arm (`Command::Concede { .. } if false => …`), which is not a
wildcard and leaves the match exhaustive — so its green was the plant failing to test the claim,
not the gate failing to hold it. Re-planted as a genuine top-level `_ => None`: **4 red**,
including `r1` itself. *A gate defeat that does not reproduce the thing the gate forbids is not a
defeat.*

### THE INSTRUMENT WAS WRONG FIRST, AND EVERY ROW BEFORE THE FIX WAS CONTAMINATED

The harness restored patched files with `shutil.copy2`, **which preserves the source mtime**. A
restored file therefore looked OLDER than the artefact compiled from the patched version, cargo
did not rebuild it, and **every row after the first measured the previous row's binary.** It was
found the only way it could be: a probe (`t2_cross_player_block_offer_absent_and_engine_refuses`)
failed on a tree `git status` called clean and passed in the full suite, and the contradiction
could not be explained by anything in the source. The first matrix reported R6 at 3 red and R7 at
2 red; after the fix (`shutil.copy` + `os.utime(dst, None)` on both the patch and the restore)
they are 2 and 1 — the extra red in each was R5's leftover binary. **The whole matrix was
re-executed from R0 and only the re-executed numbers are published above.** Filed as
`OOS-DX55-4`; the generalisation is that a revert harness that restores with metadata-preserving
copy is not a revert harness, and its failure mode is silent contamination rather than an error.

A second instrument error in the same harness: a `str.replace(old, new, 1)` anchor that was not
unique patched the SPELL query's signature instead of the ABILITY query's, twice, producing a
build failure that read like a broken revert. Anchors are now the smallest string unique to the
site.
