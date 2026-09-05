# PB-DX53 — execution notes (`scutemob-231`)

> v4 queue rank 16. **`OOS-DX21-1` CLOSED.** Merge base `5182600e`.
> Design, alternatives and the pre-committed predictions: `memory/primitives/pb-DX53-plan.md`
> (written and committed at `a37f8239`/`6b12c513` **before any production line**).

---

## 1. The defect, and why the obvious fix is wrong

`rules/combat.rs` **ASSIGNED** `ps.attackers_declared_this_turn = attackers.len()`. On a turn with
an extra combat phase (CR 500.8) — `aggravated_assault`, `aurelia_the_warleader`,
`combat_celebrant`, `karlach_fury_of_avernus`, all four deck-legal `Complete` — attacking with
three creatures in combat 1 and one in combat 2 dropped the count to **one**, and
`windbrisk_heights` went dead for the rest of the turn.

The ruling is verbatim decisive and answers all three of AC 7368's questions without inference
(Windbrisk Heights, **2007-10-01**, via MCP):

> "you'll get to play the card if you declared three **different** creatures as attackers **at any
> point in the turn**. A creature declared as an attacker in two different attack phases **counts
> only once**. A creature that entered attacking … **doesn't count** because you never attacked
> with it."

**But `Condition::YouAttackedWithNOrMore` had two readers that want OPPOSITE semantics.**
`legions_landing` is CR 508.3d — *"An ability that reads 'Whenever [a player] attacks, . . .'
triggers if one or more creatures that player controls are declared as attackers"* — per
DECLARATION. So making the field accumulate repairs Windbrisk and **regresses** Legion's Landing
(2 in combat 1 + 2 in combat 2 would transform it; the printed trigger never even fires).
**One identifier cannot carry two CR concepts, so the DSL split.**

| symbol | was | means |
|---|---|---|
| `PlayerState.latest_attacker_declaration_size: u32` | `attackers_declared_this_turn` | size of the MOST RECENT declaration; semantics untouched |
| `PlayerState.creatures_declared_as_attackers_this_turn: OrdSet<ObjectId>` | *(new, hashed)* | every creature DECLARED this turn; dedup by `ObjectId` is CR 400.7 identity |
| `Condition::YouAttackedWithNOrMoreThisDeclaration(u32)` | `YouAttackedWithNOrMore` | CR 508.3d |
| `Condition::YouAttackedWithNOrMoreCreaturesThisTurn(u32)` | *(new)* | ruling 2007-10-01 |

**Both old names were lies and both were renamed.** The field said "this turn" and meant "the
latest declaration"; the Condition stated neither scope while two cards read it for opposite ones,
so a card author reaching for the shorter identifier got the per-declaration semantics silently —
the default-choice trap that produced this seed.

**Two properties hold BY CONSTRUCTION rather than by care, which is the point of the shape:**

- **Legion's Landing is byte-identical.** Same field, same assignment, same arm body; zero
  behavioural lines on its path. That is a stronger guarantee than the probe AC 7368 asks for, so
  `t6`/`t7` are a PIN on a property that already holds, not the thing establishing it.
- **CR 508.4's exclusion** (*"Such creatures are 'attacking' but, for the purposes of trigger events
  and effects, they never 'attacked.'"*) holds because the write site reads the **declaration
  command's own** `attackers` parameter, never `combat.attackers` — which PB-DX51 made the shared
  path for the four entrant sites too. An entrant is never a parameter to that function.

---

## 2. Wire — both bumps, both predicted before any code

| | predicted (`a37f8239`) | gate-computed |
|---|---|---|
| HASH | 84 → **85**, one bump | **85** ✓ |
| PROTOCOL | 43 → **44**, one bump | **44** ✓ |
| PROTOCOL closure types | 98, unmoved | **98** ✓ |
| HASH closure types | 132, unmoved | **132** ✓ |

**AC 7369 predicted PROTOCOL UNMOVED and that prediction is REFUTED — with its own ground
verified true.** `PlayerState` really is in `CLOSURE_MUST_NOT_CONTAIN`
(`protocol_schema.rs:116`), so a `PlayerState` field alone moves HASH only. The AC's error is a
SCOPE assumption: the fix cannot be a `PlayerState` field alone (§1), and `Condition` **is** in the
wire closure via `Effect::Conditional`. That is not an inference — `rules/protocol.rs`'s **v21**
history row says it in the tree already ("*`Condition` (already in the closure via
`Effect::Conditional`)*… `PlayerState.attackers_declared_this_turn` … **not the wire closure** —
HASH_SCHEMA_VERSION bump only"), i.e. **the same batch that created this field wrote down both
halves of this prediction five weeks ago** — and it was re-verified BY EXECUTION at stage 0 by
planting `Condition` in both gates' `CLOSURE_MUST_NOT_CONTAIN` (both fail) and `TriggerCondition`
(both pass).

**Sentinels.** Re-pinned by symbol across 49 files, then:
- **Survivor scan on BOTH axes** (`OOS-DX36-8`): axis 1 SHAPE — a ±3-line window, not a
  symbol-adjacent match; axis 2 VALUE — `(84|43)(u8|u32|usize|i32)?` with digit-boundary lookarounds,
  because `\b` between a digit and `u` is not a word boundary. **0 candidates**, whole workspace.
- **`OOS-DX18-3`'s OPPOSITE check** — the one a survivor scan is structurally blind to. All **74**
  added lines carrying `85`/`44` read individually: **58** are symbol-adjacent assertion arguments,
  and the other **16** are the two history rows, their doc paragraphs, the two frozen-prefix note
  lines, the new digest, and 6 bare-continuation assertion arguments (the multi-line spelling).
  **No prose rewritten.**
- History rows APPENDED, never edited; both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned from the
  gates' own output; `history_is_append_only` and `frozen_prefix_is_pinned` green on both.

---

## 3. Census (AC 7370) — and the batch's own gate was wrong on one axis

Three axes, all over `all_cards()`, all PRINTED by their test.

| axis | result |
|---|---|
| declared, per-DECLARATION (CR 508.3d) | `legions_landing` |
| declared, per-TURN (ruling 2007-10-01) | `windbrisk_heights`, **`minas_tirith`** |
| `Effect::AdditionalCombatPhase` declarers | **4**, all derive-`Complete` and deck-legal: `aggravated_assault`, `aurelia_the_warleader`, `combat_celebrant`, `karlach_fury_of_avernus` |

**The inverse ORACLE axis is what found the second member, and it had to be, because the defect on
that card is that its ability was MISSING.** `minas_tirith` prints *"Activate only if you attacked
with two or more creatures this turn"* and was `partial` behind an in-source `ENGINE-BLOCKED` note
demanding `Condition::AttackedWithNCreatures(2)` — **an identifier that had existed as
`Condition::YouAttackedWithNOrMore(u32)` since PB-OS6 (2026-07-19)**. The note was FALSE at HEAD and
outlived the commit that falsified it (`OOS-DX47-6`'s shape; PB-DX27's *"a blocker note is a
claim"*). A declared-axis census structurally cannot see it. Authored; **the batch's single flip**.

**SR-36 has a worked example three times over in this one axis.** `grep -rl AdditionalCombatPhase`
returns **8** files. Four of them declare nothing:
- `windbrisk_heights`, `moraug_fury_of_akoum`, `breath_of_fury` mention it in a `//` comment;
- **`scourge_of_the_throne` mentions it inside its `Completeness::partial(...)` STRING**, which is
  compiled into the def.

That last one broke this batch's own gate — see §4.

**The other four scopes found by the inverse axis, classified and left alone with reasons:**
per-turn BOOLEAN (Raid, `Condition::YouAttackedThisTurn`, a monotone `= true` that cannot be
clobbered downward, so correct by construction): `raiders_wake`, `kaito_shizuki`,
`searslicer_goblin`, `bloodsoaked_champion`, `chart_a_course`, `alesha_who_laughs_at_fate`.
Per-COMBAT (Melee, CR 702.121a — `resolution.rs` reads `state.combat` live, which is replaced at
each `BeginningOfCombat`, so correct on the TURN axis and untouched by this batch):
`skyhunter_strike_force`, `wings_of_the_guard`, `adriana_captain_of_the_guard` — **but it counts
CR 508.4 entrants and does not filter by controller**, filed as `OOS-DX53-1`, not fixed.
Per-combat total power (Pack tactics): `battle_cry_goblin`, blocker still real.
Per-creature ATTACK TALLY: `moraug_fury_of_akoum` — **the opposite structure from the one built
here**, filed as `OOS-DX53-3` so a later batch does not mistake this batch's dedup'd SET for the
missing tally. Per-creature attacked-this-turn: `berserk`, blocked on a delayed-trigger primitive.

---

## 4. THE BATCH'S OWN ROSTER GATE REPRODUCED `OOS-DX36-8` ONE AXIS OVER

`pb_dx53_raid_count_roster.rs`'s first draft matched on `format!("{def:#?}")`, under a module doc
that **argued for the choice** — correctly — on the exhaustiveness axis: a derived `Debug` recurses
through every field by construction, so PB-DX26's `RollDice` lesson (a hand-written recursive
walker can miss a nesting site) does not apply, because there is no variant list to
under-enumerate. **True, and irrelevant to the failure it actually had.**

A `Debug` render also prints PROSE that is compiled into the def. So `scourge_of_the_throne`'s
`Completeness::partial("... Effect::UntapAll{is_attacking}, Effect::AdditionalCombatPhase. ...")`
was counted as a **declarer**, and R3's published population read **5** when the truth is **4**.

The tree already solves this and the plan (§7) told the batch to use it:
`decision_site_walk::def_contains_variant` walks the serde JSON and suppresses bare strings under a
`PROSE_FIELDS` key — and that list already carries `"Inert"` / `"Partial"` / `"KnownWrong"`, the
`Completeness` variant keys, for exactly this. The batch hand-rolled instead.

**R1 had the same shape and was ONE BLOCKER NOTE away from the same false positive.** That is not a
remote risk in this class specifically: the card this batch repaired, `minas_tirith`, carried a
note naming a `Condition` variant **by identifier**, which is what blocker notes do.

Both rows re-keyed onto `def_contains_variant`; R3's pin corrected 5 → 4; `scourge_of_the_throne`
added to R3's must-be-absent list **as the member that discriminates the two walks**, so the
distinction is pinned rather than merely fixed. Filed as `OOS-DX53-2`.

*A census walk has two axes — how exhaustively it reaches, and whether what it reaches is code or
prose — and defending one of them says nothing about the other.*

---

## 5. Corrections to inherited documents, each reported rather than absorbed

1. **AC 7371's baseline pin of 5,194 does not reproduce; 5,196 does** — and 5,196 is
   `CLAUDE.md:273`'s own published PB-DX39 close-out, so the pin reproduces and the AC's figure is
   a transcription off by two. Reported rather than reconciled away, because a
   "baseline that does not reproduce" is the signal `OOS-DX51-5` exists for and must not be spent
   on a typo.
2. **v4 memo row 16's wire cell says "HASH (MED)"** — both move.
3. **v4 memo row 16's yield cell says "0 flips"** — one flip.
4. **The v4 row title, this task's title and AC 7368's framing all cite CR 508.6.** CR 508.6 is
   verbatim a BOOLEAN per-player predicate (*"A player has 'attacked [a player]' if the first player
   declared one or more creatures as attackers attacking the second player"*) with no count and no
   turn-scope content. It does not warrant this behaviour. **The `OOS-DX21-1` registry row already
   said so**, and PB-DX21's own review had corrected `legions_landing.rs` for exactly this mis-cite
   — after which it propagated into the queue row title and the dispatch title. Governing
   authorities: CR 508.1, ruling 2007-10-01, CR 508.4, CR 400.7, and CR 508.3d.
5. **The plan's own §9.2 said the declared `AdditionalCombatPhase` population is 7** (grep's 8 minus
   `windbrisk_heights`). It is **4**. Two more grep hits are comments and one is a completeness
   note. The plan's §9.2 was itself offered as a worked example of SR-36 and was under-corrected.
6. **The engine agent reported the population as 5** — the Debug-walk figure of §4.

---

## 6. Measurements

| | |
|---|---|
| baseline (pre-edit, this branch) | **5,196 / 0 / 5**, 66 targets |
| final tree | **5,209 / 0 / 5**, **67** targets (one new simulator binary) |
| delta by NAME (byte-exact Python set difference, never `sort`+`comm` — `OOS-DX20b-5`) | **13 additions, 0 leavers, 0 removals, 0 renames** |
| count-vs-name reconciliation | 13 == 13 ✓ |
| duplicate-name scan (`OOS-DX35-8`) | 5,201 / 5,201 distinct and 5,214 / 5,214 distinct — **EMPTY on both runs** |
| coverage | **1,139 → 1,140 / 1,803 = 63.2%**, the ONE predicted flip |
| `Completeness` markers moved | exactly **1**, checked by `git diff` over the marker rather than inferred from the total (PB-DX26's lesson) |
| card-def edits | **3** (`legions_landing` rename+comment, `windbrisk_heights` rename+comment rewrite, `minas_tirith` authored) |
| `clippy --workspace --all-targets -- -D warnings` | clean, FINAL tree |
| `cargo fmt --check` | clean, FINAL tree |
| `tools/check-defs-fmt.sh` | clean, 1,803 defs, FINAL tree |
| `cargo build --workspace` (SR-3 seal gate) | clean, FINAL tree |
| `npm run build` | **N/A, and that is stated rather than omitted**: `git diff --numstat 5182600e..HEAD -- tools/` is **EMPTY** and `node_modules` is absent |

**Production lines** (`git diff --numstat`, re-taken against the FINAL tree rather than transcribed
— PB-DX28's re-take MEDIUM): `crates/engine/src` **+150 / −30**; `crates/card-types/src`
**+67 / −14**; `crates/card-defs` **+107 / −71**; and **`crates/simulator/src`,
`crates/view-model` and `tools/` are all EXACTLY 0** — every consumer of the raid gate lives in the
engine and the card defs.

**`size_of::<PlayerState>()` 376 → 400 (+24 bytes, +6.4%)**, executed at both revisions;
`size_of::<GameState>()` **3536 → 3536 UNMOVED** (a `PlayerState` lives in an `OrdMap`, on the
heap). The +6.4% is LARGER than PB-DX18's +16 / +4.4%, which produced a real uniform 2.5–4.5%
regression, so a regression here is expected rather than surprising and the A/B is owed.

---

## 7. Benches — MEASURED, six runs, quiet machine, **and the FIRST A/B was thrown away**

**The first merge-base set is DISCARDED rather than published.** Runs 1-3 were taken while the
engine agent was compiling, and `board_wipe_4p` moved **120.34 → 126.52 → 139.34 µs on IDENTICAL
code** — a **16%** same-code spread, with `full_turn_4p` at 9%. That is PB-DX52's contamination
tell, recognised from the spread rather than from the verdict, and the whole set was moved to
`scratchpad/discarded/` before any comparison was computed.

Re-run on a quiet machine, **same-code band measured FIRST** across three merge-base runs before
any HEAD number was read (PB-DX18's lesson). Each revision in its own worktree with its own
`CARGO_TARGET_DIR`.

| bench | base ×3 (µs) | HEAD ×3 (µs) | same-code band | Δ median | |
|---|---|---|---|---|---|
| `priority_cycle_4p` | 24.48 / 24.66 / 24.59 | 24.03 / 24.52 / 24.11 | 0.72% | **−1.96%** | overlap |
| `priority_cycle_6p` | 38.96 / 38.42 / 38.77 | 38.48 / 38.39 / 38.59 | 1.42% | −0.75% | overlap |
| `sba_check` | 15.33 / 15.37 / 15.16 | 15.39 / 14.70 / 15.46 | 1.39% | +0.39% | overlap |
| `full_turn_4p` | 218.13 / 216.67 / 219.11 | 217.02 / 216.32 / 215.04 | 1.13% | −0.83% | overlap |
| `full_turn_6p` | 348.44 / 347.24 / 348.90 | 345.86 / 345.88 / 344.23 | 0.48% | −0.74% | non-overlapping |
| `board_wipe_4p` | 122.05 / 121.70 / 121.49 | 121.01 / 120.43 / 120.02 | 0.46% | **−1.04%** | non-overlapping |

**Widest same-code band: 1.42%.** Both non-overlapping differences (−0.74%, −1.04%) are *smaller*
than that band.

**Verdict: NO REGRESSION, and the apparent 0.7–2.0% improvement is deliberately NOT claimed**, on
the standing grounds: `priority_cycle_4p`/`6p` and `sba_check` are **controls** — nothing this
batch touches is on the priority loop or the SBA loop — and they move the same order as everything
else, which is a build/layout artefact of two separate compilations rather than an effect. The
mechanism argues the *other* way besides: `PlayerState` grew, so a uniform speed-up is not
something this change can cause.

**And the batch's own expectation of a regression is REFUTED by measurement, which is the
interesting part.** §6 predicted one, on PB-DX18's precedent: that batch published a real uniform
2.5–4.5% regression from `PlayerState` 360 → 376 (+4.4%), and this batch grows it MORE
(376 → 400, **+6.4%**). No regression appeared. The candidate explanation, offered as an inference
with its evidence rather than as a finding: **PB-DX18 grew BOTH structs** —
`size_of::<GameState>()` 3512 → 3536 alongside `PlayerState` 360 → 376 — while **this batch grows
`PlayerState` alone and leaves `GameState` UNMOVED at 3536** (executed at both revisions; a
`PlayerState` lives behind an `OrdMap`, on the heap). So the evidence here points at
**`GameState`'s** size, not `PlayerState`'s, as what drove PB-DX18's regression. Recorded so the
next batch that grows one of the two can predict from the right struct — not asserted as proven,
because this batch did not run PB-DX18's counterfactual.
---

## 8. Revert matrix — **executed by the coordinator**, not accepted from the delegated report

| row | revert | reds (of 7 primitive probes) |
|---|---|---|
| **R1** | delete the accumulation at the declaration site (the headline fix) | **6** — `t1`, `t2`, `t3`, `t4`, `t5`, `t6` |
| **R2** | make `YouAttackedWithNOrMoreCreaturesThisTurn` read `latest_attacker_declaration_size` instead of the set | **1 — exactly `t1`** |
| **R3** | stop clearing the set at the turn boundary | **1 — exactly `t5`** |

**3 rows, 3 discriminating, 0 UNDISCRIMINATED.** R2 and R3 are **precise complements** of R1's
blanket: each isolates one link of the chain, so the matrix distinguishes *"the set is populated"*
from *"the right reader consults it"* from *"it is cleared at the right time"*, rather than proving
only that something in the area matters.

**Two rows in that table need their reason stated rather than their count read.**

- **`t7` is GREEN under R1 and that is a stated CONTROL, not a gap.** It asserts Legion's Landing
  DOES transform on three attackers in one combat — the CR 508.3d behaviour a correct fix must not
  break. R1 does not touch the per-declaration path, so a red there would mean the fix had
  regressed the very card §1 promises is byte-identical.
- **`t6` reddens under R1 on its NON-VACUITY FLOOR, not on its subject.** Its Legion's Landing
  assertions (2 in combat 1 + 2 in combat 2 does **not** transform) stay true under R1 — correctly,
  because the per-declaration count is 2 either way. What reddens is its third assertion, that the
  per-TURN set really reached 4. Without that floor, *"Legion's Landing did not transform"* would be
  satisfied by an engine in which nothing was ever counted at all, which is exactly the pre-fix
  engine. **A differential probe between two cards needs a floor proving the two inputs actually
  diverged**, and reading R1's count without opening the test would have mis-attributed this row.

**A methodological correction inside this matrix, recorded because it failed in the safe-looking
direction.** The first harness classified a row as "BUILD FAILED — verdict void" by matching
`^error(\[|:)`, which also matches cargo's ordinary `error: test failed, to rerun pass …`. All
three rows were reported as void builds when all three had in fact built and produced real reds.
That is `OOS-DX39-8`'s own lesson — *a matrix must distinguish "the gate stayed silent" from "the
crate did not build"* — inverted: **an over-wide build detector converts a real verdict into a
non-verdict**, which is the same failure mode one axis over. Re-keyed on `^error\[E` /
`could not compile` and re-executed; the table above is the re-run.

**And one earlier row was reported by this coordinator and then withdrawn before it was trusted.**
An R1 attempt whose `python3 -c` patch never applied (the literal did not match, because the
accumulation is spelled across a line break) printed seven greens. Those greens were the
UNMODIFIED tree. Caught because the script's own assertion error was read rather than the test
output beneath it. *A revert row that does not apply produces a green run indistinguishable from a
non-discriminating gate* — assert the patch applied before reading any verdict.
