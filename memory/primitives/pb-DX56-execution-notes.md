# PB-DX56 execution notes (`scutemob-235`)

> v4 queue rank 20, task 2 of 5 of the SECOND user-approved chain.
> Seeds: **OOS-FB1-1** *(prerequisite)* → **OOS-DX32-1** + **OOS-DX22-8**.

---

## §0 Stage 0 — measured and predicted BEFORE any production line

### 0.1 Pre-edit full-workspace baseline (measured, not remembered)

`cargo test --workspace --no-fail-fast` to a file, on this branch **before any edit**:

```
result lines (targets): 72
passed/failed/ignored: 5287 0 5
extracted test lines: 5292 distinct: 5292
duplicate names: 0
```

**Reproduces PB-DX55's close pin exactly** — 5,287 / 0 / 5 on **72** result-producing
targets — the **seventh** consecutive batch in which an inherited pin reproduces with no
correction owed. The extraction regex is deliberately **not** end-anchored
(`OOS-DX42b-6`), so an `#[ignore = "reason"]` test whose line reads `... ignored, <reason>`
is still extracted; the duplicate-name scan the byte-exact method is structurally blind to
(`OOS-DX35-8`) is **EMPTY** (5,292 lines / 5,292 distinct).

### 0.4 Wire prediction — PER HALF, in writing, before any production line

**Prediction: HASH 85 and PROTOCOL 44 BOTH UNMOVED — zero bumps for the whole PB.**

Stated per half with the reason, not as a preference:

* **Half A — `OOS-FB1-1`, the diagnosability tooling.** Everything it touches lives in
  `crates/simulator` and `crates/simulator/src/bin/fuzzer.rs`:
  `InvariantViolation` gains an `evidence` field; `LocalGame`/`GameResult` gain a bounded
  command-history ring; `CrashReport.command_history` stops being `Vec::new()`; the
  in-flight tombstone is a filesystem write in the binary. **Neither gate walks
  `crates/simulator` at all** — `hash_schema.rs` and `protocol_schema.rs` live in
  `crates/engine/tests/core` and close over the engine's
  `Command` / `GameEvent` / `Effect` / `Characteristics` roots. A simulator-side struct is
  not reachable from any of the four. Predicted movement: **none, on either gate.**
* **Half B — the `OOS-DX22-8` engine fix.** The defect is a **dangling `ObjectId` left in
  an already-hashed field**: `GameObject.attached_to` has existed and been hashed since
  long before this batch. A fix that changes **when** that field is cleared adds no type,
  no variant and no field, and `state/hash.rs` hashes the field's VALUE, not the moment it
  was written. Predicted movement: **none, on either gate.**
  **Stop-condition, stated in advance**: if the fix turns out to require a new field or a
  new type, this batch STOPS and posts `COORDINATOR` before re-predicting — it does not
  quietly take a bump the prediction did not cover.
* **Half C — the `OOS-DX32-1` disposition.** Both branches are wire-neutral for the same
  two reasons above: a transient split is a `crates/simulator` bucket change, and an
  engine-side repair of `turn.priority_holder` writes an already-hashed field.
  Predicted movement: **none, on either gate.**

**Counterfactual, to be VERIFIED BY EXECUTION at stage 0 rather than asserted** — "unmoved"
only means something beside what would have moved it. Recorded when run.

### 0.5 Coverage prediction

**0 flips, coverage UNMOVED at 1,140/1,803 = 63.2%.** Reason: this batch authors no card
text and repairs no card-def blocker — it changes the fuzzer's instrumentation and (at
most) one engine attachment/priority path. No `Completeness` marker can move, because no
def's expressible-ness changes. To be confirmed by regeneration rather than by the
empty-diff shortcut.

### 0.2 Both filed figures RE-MEASURED at HEAD — and **NEITHER reproduces, both in the same direction: UP**

The exact filed invocation, run at HEAD before any edit
(`cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200 --threads 1`,
raw output at `memory/primitives/pb-dx56-measurement-head.txt`):

```
Games completed: 20   Wins: 20  Draws: 0  Errors: 0   Avg turns per game: 122.0
Total violations (HARD): 291
Total violations (TRANSIENT, reported -- does not halt --stop-on-error): 553

  HARD (raw 291 / distinct 17), 14/20 game(s)
    player_consistency       189    in 11 game(s): [3, 4, 5, 7, 8, 10, 11, 12, 13, 16, 17]
    attachment_validity      102    in 7 game(s):  [1, 8, 9, 11, 12, 13, 18]
  TRANSIENT (raw 553 / distinct 118), 13/20 game(s)
    no_orphaned_tokens       553    in 13 game(s)
```

| class | as FILED | at the 2026-08-14 re-measure | **at HEAD (this batch)** |
|---|---|---|---|
| `player_consistency` | 114 raw / 5 of 20 games | 84 raw / 4 of 20 [12,13,14,19] | **189 raw / 11 of 20** [3,4,5,7,8,10,11,12,13,16,17] |
| `attachment_validity` | 11 raw / 3 of 20 | 22 raw / 3 of 20 [2,5,10] | **102 raw / 7 of 20** [1,8,9,11,12,13,18] |

So `OOS-DX32-1`'s own instruction — *"neither number should be carried forward again"* —
is honoured, and it was right: `player_consistency` is **2.25×** its last re-measure and
`attachment_validity` **4.6×**, with the game sets disjoint from the ones recorded a
fortnight ago. Five more PB-DX batches have perturbed bot play on the same seeds since,
which is the recorded reason.

**The HARD bucket is EXACTLY these two classes** (`histogram total (HARD): 291`, and the
by-check breakdown lists two rows), so *"every remaining HARD class on the standard
invocation"* is a closed set of two, not an open-ended sweep. `player_consistency`'s share
of HARD is **189 / 291 = 64.9%** — the row's headline "79.2% of the HARD bucket" does not
reproduce either, because the other class grew faster.

### 0.2a The share that decides the disposition, read off the run rather than assumed

**Every one of the 189 `player_consistency` reports is the ACTIVE-PLAYER arm.**
`grep -c "Priority holder" ` over the whole run output is **0**; `grep -c "Active player
Player"` over the printed sample is 88. The two arms of that check are therefore not one
class, and the registry row treats them as one.

Per-game replays (`--replay <seed>`, which prints ALL of a game's violations rather than
the first five games' worth) give the distinct instances:

| seed | class | instances |
|---|---|---|
| 1 | attachment | `631 → 637` t52 |
| 8 | attachment | `667 → 574` t92, `667 → 918` t101, `667 → 937` t104 |
| 8 | player | active `PlayerId(3)` t79 |
| 9 | attachment | `477 → 756` t109, `477 → 852` t129, `477 → 912` t141, `477 → 1000` t154 |
| 11 | attachment | `490 → 444` t55 |
| 11 | player | active `PlayerId(1)` t146 |
| 12 | attachment | `880 → 786` t159 |
| 12 | player | active `PlayerId(4)` t174 |
| 13 | attachment | `472 → 532` t67 |
| 13 | player | active `PlayerId(4)` t148 |
| 18 | attachment | `803 → 478` t130, `803 → 489` t140 |

**The shape worth reading is seeds 8, 9 and 18**: ONE attacher (`667`, `477`, `803`) dangles
against a SUCCESSION of different dead targets across dozens of turns — `477` at turns
109, 129, 141 and 154, a 45-turn span. A single object that survives that long on the
battlefield and keeps acquiring new attachment targets is Equipment-shaped, not Aura-shaped
(an Aura that lost its target goes to the graveyard under CR 704.5m and never re-attaches).
That is a hypothesis at this point in the batch; the evidence field built in Stage 1 is what
decides it, because the check as shipped reports **two integers and nothing else**.

### 0.4a Half A's wire-neutrality is a DEPENDENCY-DIRECTION fact, not a closure-walk result

Checked rather than asserted: `crates/engine/Cargo.toml` does not depend on
`mtg-simulator`, and neither `crates/engine/tests/core/hash_schema.rs` nor
`crates/engine/tests/core/protocol_schema.rs` mentions it. **The engine crate cannot name
a simulator type at all**, so `InvariantViolation`, `GameResult`, `CrashReport` and the
command-history ring are outside both closures by construction — the dependency arrow
points the other way. The two forbidden lists are

* HASH: `["Command", "ReplayLog", "Envelope", "CardRegistry", "CardDefinition"]`
* PROTOCOL: `["GameState", "PlayerState", "StackObject", "CardDefinition"]`

and neither can be extended with a simulator type even to run the counterfactual — the
plant would not compile. **That is worth saying out loud rather than reporting a vacuous
"we planted it and nothing happened"**: for Half A the counterfactual is not merely
unmoved, it is *unexpressible*, and an unexpressible counterfactual is a different
(stronger) claim than an executed one that came back green.

The counterfactual that IS expressible is the one for Halves B and C, and it is the one
that matters: the fields those halves write — `GameObject.attached_to` and
`TurnState.priority_holder` — are **already on both wires**, which is exactly why writing
them at a different moment adds nothing. Executed at stage 0; result recorded in §0.4b.

---

## §1 The diagnosis — both classes, from source, before any fix

Full census: `memory/primitives/pb-DX56-mechanism-census.md` (read-only, `file:line` or a
verbatim CR sentence behind every claim, each one tagged ESTABLISHED or INFERENCE).

### 1.1 `OOS-DX32-1` — the check has TWO arms and they need OPPOSITE dispositions

The registry row, the v4 memo cell and the dispatch criterion all treat
`player_consistency` as one class. **It is two**, and the run says so before the CR does:
**189 of 189 reports are the ACTIVE-PLAYER arm and the priority-holder arm produced ZERO
hits.**

**The active-player arm is asserting something CR 800.4j explicitly permits.** Verbatim:

> **CR 800.4j** — *"If a player leaves the game during their turn, that turn continues to
> its completion **without an active player**. If the active player would receive priority,
> instead the next player in turn order receives priority, or the top object on the stack
> resolves, or the phase or step ends, whichever is appropriate."*

`TurnState::active_player` is a bare `PlayerId` (`state/turn.rs:95`), not an `Option`, with
**exactly one** production write site — `turn_structure.rs:161`, inside `advance_turn`. So
*"without an active player"* is **inexpressible in this state type**, and the engine
necessarily encodes that turn by leaving the departed player's id in the field. Every
consequence CR 800.4j actually requires is discharged elsewhere and was checked:
`priority::grant_priority_to_active_player` (`priority.rs:172-193`) routes past a dead
active player **citing CR 800.4j by name**, and `validate_player_active`
(`engine.rs:3192-3198`) rejects every command from a departed seat.

So the answer to the criterion's question — *"is it ever true at rest?"* — is **it is true,
it is bounded, and it is not a defect**: it is a representation choice the CR describes.
**Bounded** because `next_player_in_turn_order` (`turn_structure.rs:197-201`) skips
`has_lost || has_conceded`, so CR 800.4k holds and the next turn picks a live player. That
is exactly the observed shape: one turn number per game, repeated across that turn's
commands.

**The priority-holder arm is a real defect and stays HARD.** CR 800.4a's last sentence is
unconditional and has no "continues without" escape:

> *"If the player who left the game had priority at the time they left, priority passes to
> the next player in turn order who's still in the game."*

Its zero hits are the evidence that keeping it hard costs nothing.

**Two holes make the boundedness lucky rather than true, and both are fixed here rather
than asserted away** — this is the difference between classifying a class transient and
merely hoping it is:

* **`advance_turn`'s EXTRA-TURN branch applies no liveness filter** (`turn_structure.rs:149`,
  `turn.extra_turns.pop_back()`), and nothing ever prunes the queue (written at
  `resolution.rs:8822` and `effects/mod.rs:7663`, read only there). An extra turn queued
  for a player who then leaves **begins** — CR 800.4k-wrong, and it is the one route by
  which the active-player condition is UNBOUNDED. → **F2**.
* **`enter_step`'s cleanup-SBA-round grant is unconditional** (`engine.rs:2723-2726`) — the
  single live route by which the priority-holder arm can fire. Already filed as
  `OOS-DP9-19`, and named as the surviving exception in `engine.rs:3079`'s own comment.
  → **F3**.

### 1.2 `OOS-DX22-8` — the check watches the side that heals and is blind to the side that does not

**Supply, ESTABLISHED**: `GameState::move_object_to_zone` (`state/mod.rs:1632`) and
`move_object_to_bottom_of_zone` (`:2146`) retire the departing object and mint a new one.
They perform exactly **two** cross-object fix-ups — CR 702.95e soulbond `paired_with` and
the MR-M8-16 `replacement_effects` GC — and **touch the attachment relation in neither
direction**.

**Direction A** (a HOST leaves; its attachers' `attached_to` dangles) is what
`check_attachment_validity` reports. It is **cleared by an SBA**, and the iff is exact:

> an attacher holds a dangling `attached_to` indefinitely **iff** at every later sweep it
> is phased out (`sba.rs:1134`; `sba.rs:186` → `:1304-1306`) **or** its **layer-resolved**
> subtypes contain none of `Aura` / `Equipment` / `Fortification` (`sba.rs:1140`,
> `:1307-1312`).

Neither holds for the ordinary case, so the ordinary case heals — but **not before the
invariant runs**. The engine sweeps SBAs at **nine** call sites and
`abilities.rs`, `casting.rs`, `combat.rs`, `mana.rs`, `turn_actions.rs`,
`turn_structure.rs` and `replacement.rs` contain **zero** of them, so a permanent that
leaves the battlefield while paying a cost (`abilities.rs:1216`, `mana.rs:493`, 21 sites in
`casting.rs`) dangles across the checkpoint and heals at the next step entry or resolution.
That is **`OOS-M11-7`'s recorded shape, one field over** — the same CR 704.3 timing
deviation that made `no_orphaned_tokens` transient.

**Direction B is the engine defect, it is AT REST, and nothing has ever looked at it.**
When an *attacher* leaves the battlefield by any route other than the six that clean up
(`sba.rs:1239`, `sba.rs:1406`, `effects/mod.rs:2060`, `:6074`, `:6214`, `:6277` — two SBAs
and four equip effects), its host keeps the dead `ObjectId` in `host.attachments`
**permanently**. Destroy an Equipment with a Disenchant, bounce an Aura, exile either: the
host is corrupted for the rest of the game. Consequences, each read off a real consumer
rather than asserted:

* `attachments` is **HASHED** (`state/hash.rs:2670`), so a stale entry perturbs
  `public_state_hash` **and** `loop_detection::compute_mandatory_state_hash` — CR 104.4b
  mandatory-loop detection can fail to recognise a repeated board state;
* it is read by the CR 510.3a equipped-creature combat-damage trigger family
  (`abilities.rs:6013`, `:7232`);
* it is walked by CR 702.26g/h phasing (`turn_actions.rs:1105`, `:1128`), where a dead id
  reaches **`expect_object_mut`** — an IMPOSSIBLE-class SR-4 lookup that fires a
  `debug_assert` — so a stale entry is a **latent debug-build panic** on the phasing path;
* it is rendered to the browser (`crates/view-model/src/lib.rs:472`).

**So the diagnosis is not "the reported class is a bug" and not "the reported class is
noise" — it is that the check was pointed at the healing direction of a two-directional
relation, and the direction that never heals had no check at all.** → **F1**.

The other direction is deliberately NOT fixed at the zone-move site, and the reason is a CR
reason rather than a scope one: CR 704.5m puts an illegal Aura into its **owner's
graveyard** while CR 704.5n merely **unattaches** an Equipment or Fortification and leaves
it on the battlefield. **Opposite dispositions for the same input**, already implemented as
two separate SBA arms. Clearing `attached_to` eagerly would be performing a state-based
action early, and it would silently delete the transient class rather than classify it.

### 1.3 Three findings the census produced that no document names

* **Three of the four `attached_to = Some(..)` production writers never check the SOURCE's
  subtype at all.** `Effect::AttachEquipment` (`effects/mod.rs:6007-6090`) validates the
  target's zone, phasing, controller and layer-resolved creature-ness and checks **nothing**
  about the source — not the `Equipment` subtype, not even that it is on the battlefield.
  `effects/mod.rs:2050-2065` carries a comment reading *"Verify source is still on the
  battlefield and is an Equipment"* while checking only the battlefield half — **a false
  comment on this batch's own subject matter**, PB-DX47's `OOS-DX47-6` shape again.
* **The aura ATTACH site reads RAW characteristics where the aura SBA reads LAYER-RESOLVED
  ones** (`resolution.rs:2015-2026` vs `sba.rs:1138-1142`), so an Aura resolving under a
  subtype-stripping continuous effect is attached by the first and **permanently invisible**
  to the second. That is blind spot (b) with an established code-level route.
* **CR 800.4a's object procedure is implemented nowhere.** `sba.rs:300` and
  `diagnostics.rs:134` both carry a comment asserting *"CR 800.4a removes their objects, not
  the PlayerState"* while no site removes or exiles a departed player's objects. A comment
  describing a procedure the engine does not run.

---

## §2 The three engine fixes, and the CR that decides the shape of F1

Delegated to a `primitive-impl-runner` on `crates/engine/` alone (disjoint from the
tooling agent's file set), then reviewed and re-verified by the coordinator.

**All six CR cites re-verified against the rules server BY THE COORDINATOR**, because the
implementing agent reported it had no `mcp__mtg-rules__*` tool available and **said so
rather than proceeding as if it had** — the PB-DX52 precedent. CR 400.7, CR 704.5m,
CR 704.5n, CR 800.4a, CR 800.4j and CR 800.4k all read exactly as the census quoted them.

### F1 — CR 400.7 attachment symmetry (`state/mod.rs`, +39 / −0)

One private helper `GameState::detach_from_host_on_departure`, called from BOTH zone-move
helpers beside the existing CR 702.95e soulbond fix-up — not two hand-rolled copies, which
is the class of defect this queue keeps closing. It removes the departing id from its
host's `attachments` through `fizzle_object_mut`, because the host may itself have left in
the same SBA batch and that is a legal fizzle rather than an engine bug (SR-4's
classification, mirroring the soulbond site one statement above).

**And there is a CR rule that makes the one-directionality load-bearing rather than merely
conservative, which the census did not have.** Beyond the fact that CR 704.5m and CR 704.5n
prescribe opposite dispositions, **CR 400.7f** exists *specifically* to let a
leaves-the-battlefield trigger find an Aura in its owner's graveyard *"as a result of being
put there as a state-based action for not being attached to a permanent. (See rule
704.5m.)"* — i.e. the CR has an exception rule whose antecedent is that the Aura reached
the graveyard **through 704.5m**. Clearing `attached_to` eagerly at the zone-move site would
change which 704.5m arm fires (from "target gone" to "not attached") and, more importantly,
would be an SBA performed outside an SBA sweep. So *"do not finish the job"* is a
CR requirement with its own citable rule, not a scope decision — and it is pinned
wrong-way-round by `t3` so a later batch cannot quietly symmetrise the helper.

### F2 — CR 800.4k extra-turn liveness (`rules/turn_structure.rs`, +21 / −1)

`advance_turn`'s extra-turn branch pops until it finds a live entry, **discarding** dead
ones (CR 800.4k: that turn *"doesn't begin"* — it is not deferred and not requeued), and
falls through to normal turn order if the whole queue is departed players. LIFO and
`last_regular_active` semantics unchanged for live entries.

**A pre-existing test was a PIN ON THIS DEFECT and its own docstring said so.**
`mechanics_e_l::extra_turns::test_extra_turn_eliminated_player_skipped` documented
*"the eliminated player may briefly be set as active_player … effectively a no-op turn"*
as the expected behaviour — which is the CR 800.4k violation verbatim. It is **corrected,
not re-pinned**, and the correction makes it STRONGER: its headline `assert_ne!(…, p1)`
becomes `assert_eq!(…, p3)`, an exact assertion, because with the fix the departed
player's entry is structurally unable to become `active_player` at all. The test NAME is
unchanged, so this is an in-place inversion that the byte-exact NAME delta cannot see —
disclosed here rather than left for the name set to hide (PB-DX48's rule).

### F3 — CR 800.4a cleanup-SBA-round grant, closing `OOS-DP9-19` (`rules/engine.rs`, `rules/priority.rs`)

The cleanup-SBA-round grant inside `enter_step` hand-rolled
`grant_initial_priority` + two unconditional field writes. It now calls
`priority::grant_priority_to_active_player`, **the helper that already existed and whose
own doc named this exact site as the one deliberately-unrouted hole**. So F3 is finishing
the wiring of a helper built for it, not new logic — and both doc comments that asserted
the hole still existed (`engine.rs:~3079` and `priority.rs`'s grant inventory) are rewritten,
because they become false the moment the fix lands. A comment left asserting a hole the
code no longer has is `OOS-DX47-6`'s shape, and this batch's own §1.3 found two more of
them.

**Every one of the six probes was watched RED under a temporary revert**, with the panic
line and message recorded, and every reverted file restored byte-exactly. `t3` and `t5`
correctly stayed GREEN under F1's and F2's reverts respectively — stated as controls, not
as gaps, because they pin different properties.

---

## §3 The disposition, and the number that decides it

### 3.1 `--stop-on-error` no longer halts: HARD **291 → 0** on the standard invocation

| bucket | at HEAD (pre-edit) | after |
|---|---|---|
| **HARD total** | **291 raw / 17 distinct**, 14/20 games | **0 raw / 0 distinct, 0/20 games** |
| `player_consistency` | 189 (11 games) | — split, see below |
| `attachment_validity` | 102 (7 games) | 102, **TRANSIENT** |
| `departed_active_player` (CR 800.4j) | — | 189, **TRANSIENT** |
| `departed_priority_holder` (CR 800.4a) | — | **0**, HARD |
| `attachment_symmetry` (new, HARD) | — | **0** |
| `dangling_attachment_at_rest` (new, end-state HARD) | — | **0** |
| TRANSIENT total | 553 | 844 |

Every remaining class is transient **with its own strictly stronger property**, and every
new hard check is silent — which is a result rather than a vacuity, because each one was
proven to fire on a planted violation under its own revert row.

### 3.2 **The transient-vs-at-rest question is settled by an ARITHMETIC, not an argument**

The census predicted this in writing before any measurement: *"a dangle that never heals
would be re-reported after every subsequent tracked command for the rest of the game —
hundreds, not fifteen — because the check is per-command and the predicate is stateless."*

Revert row **R-E** (F1 disabled at both call sites, `#[allow(dead_code)]` on the helper so
the row is a verdict and not a build failure) executes exactly that:

| class | raw | distinct | raw / distinct |
|---|---|---|---|
| `attachment_validity` (direction A, heals) | 102 | ~13 | **≈ 8 checkpoints** |
| `attachment_symmetry` under R-E (direction B, never heals) | **10,290** | **7** | **≈ 1,470 checkpoints** |

**Two orders of magnitude, on the same fuzz run, from the same per-command stateless
checker.** That ratio IS the discriminator: ~8 checkpoints is one cost-payment window
between SBA sweeps (`OOS-M11-7`'s shape); ~1,470 is "for the rest of the game". So
*"direction A is transient and direction B is at rest"* is **measured**, and the
prediction that produced it was written down first.

**R-E is also the proof that F1 is load-bearing at run scale and that the defect was
live**: 7 distinct dangling-`attachments` conditions across 5 of 20 games at HEAD, and
**0** with F1 in. Both fuzz outputs are committed
(`memory/primitives/pb-dx56-measurement-revert-F1.txt`,
`pb-dx56-measurement-after.txt`).

**The first attempt at R-E was a NON-verdict and is reported rather than quietly redone**:
removing both call sites made the helper dead code and `-D warnings` turned it into
`error: method is never used`, so the row produced a build failure where it looked like it
would produce a violation count. `OOS-DX39-8`'s exact shape one axis over. Re-run with
`#[allow(dead_code)]`.

### 3.3 The tooling caught a bug in the code that consumes it, on its first run, and it was this batch's own

After the disposition landed, HARD read **1**, not 0: the CR 800.4k turn-boundary promotion
fired on seed 5. **Reading the artefact's evidence rather than its count showed the
promotion was a FALSE POSITIVE of this batch's own code.**

`LocalGame::promote_if_it_crossed_a_turn` extracted the departed seat with
`strip_prefix("player=")` — and `check_all` **prepends** `state_context`, which emits one
`player=PlayerId(n) life=… has_lost=…` line **per seat**. So the key was the FIRST
state-context line: a value identical for every violation in the game. It keyed
`PlayerId(4)`'s turn-154 report against `PlayerId(1)`'s turn-133 one and reported
*"turns_crossed=21"* for **two different seats**. Confirmed against the pre-edit
measurement, which records seed 5's turn-133 report as `PlayerId(1)`.

Fixed by renaming the arm's own key to `arm_player=`, with the incident recorded at both
the emit site and the consumer, and pinned by
`t_arm_player_key_is_not_shadowed_by_state_context` — which asserts **both** that
`arm_player=` names the right seat **and** that a `player=` lookup finds a different seat
first, so the reason for the odd key survives a later "tidy-up".

**That is `OOS-FB1-1`'s entire argument in one incident.** The count said *"one hard
violation, diagnosed"*; the evidence said *"your own key is wrong"*. Nothing in the
pre-batch fuzzer — which printed two integers and an empty `command_history` — could have
told the difference.

---

## §4 Fuzz A/B against the merge base, and the wire

### 4.1 The A/B, in an isolated worktree with its own `CARGO_TARGET_DIR`

Merge base `e0da3cc9` checked out under the scratchpad with its own target dir (both
deleted afterwards — `/tmp` is quota'd, dispatch hygiene 11), same invocation
(`--games 20 --seed 1 --max-turns 200 --threads 1`):

**The merge-base run and this batch's own PRE-EDIT run differ in EXACTLY ONE LINE — the
wall clock** (19.2s vs 19.0s). Every violation count, every distinct count, every per-check
game list is identical. So the pre-edit figure in §0.2 is confirmed to be the merge base's,
not a stale binary's.

| | merge base | HEAD | attribution |
|---|---|---|---|
| HARD total | **291** / 17 distinct, 14/20 games | **0** / 0, 0/20 | the disposition |
| `player_consistency` | 189 (11 games) | — | renamed |
| `departed_active_player` | — | **189** (11 games, same list) | reclassified TRANSIENT |
| `departed_priority_holder` | — | 0 | new hard class, silent |
| `attachment_validity` | 102 (7 games) | **102** (7 games, same list) | reclassified TRANSIENT |
| `attachment_symmetry` | — | 0 | new hard class, silent (10,290 under R-E) |
| `dangling_attachment_at_rest` | — | 0 | new end-state class, silent |
| `no_orphaned_tokens` | 553 (13 games) | **553** (13 games, same list) | unchanged |
| TRANSIENT total | 553 | **844** | 553 + 189 + 102, exactly |
| wins / draws / errors / avg turns | 20 / 0 / 0 / 122.0 | 20 / 0 / 0 / **122.0** | unchanged |

**Every per-class RAW count and every per-class game list is IDENTICAL across the boundary,
and 844 = 553 + 189 + 102 exactly.** So the whole `291 → 0` movement is the
RECLASSIFICATION plus three new hard checks measuring zero — **and the three engine fixes
are trajectory-neutral on these twenty seeds**, which is a measurement rather than a hope:
identical wins, identical avg turns to one decimal, identical per-seed game lists. That is
also why no seeded pin moved and none had to be re-tuned.

### 4.2 The PB-DX32 gate config: ratchets ANSWERED, not loosened

`cargo test -p mtg-simulator --test pb_dx32_fuzz_output` — **13 / 13 green**, including
`test_dx32_sr38_bot_rejection_rate_is_ratcheted`,
`test_dx32_random_bot_waste_ratio_is_bounded` and
`test_dx55_the_historical_gate_seeds_now_produce_zero_bot_rejections` (PB-DX55's
zero-ceiling pin, the strongest ratchet that file holds). **No ratchet constant was
touched** — `git diff` over `report.rs` shows the only `const`-adjacent change is a doc
cross-reference to the new command-history bound.

**One assertion MESSAGE in that file was corrected rather than left standing**, because it
became an overclaim the moment the transient set grew: *"transient_violations() must
contain ONLY no_orphaned_tokens"* was a CLASS fact when tokens were the only transient
class and is now a fact about seed 162 at 25 turns. Its sibling — the exhaustiveness half —
was generalised from one literal name to `is_transient_check`, so it now checks the whole
set instead of one member. A message that claims more than its assertion checks is the
defect PB-DX47 was dispatched for.

### 4.3 The wire: HASH 85 / PROTOCOL 44, BOTH UNMOVED — zero bumps, as predicted

Gate-executed: `hash_schema` **36/36**, `protocol_schema` **17/17**.
`git diff main..HEAD` over `state/hash.rs` and `rules/protocol.rs` is **EMPTY**, so no
sentinel re-pin, no survivor scan, no history row and no frozen-prefix re-pin were owed.

**The counterfactual is VERIFIED BY EXECUTION, and it reproduces PB-DX51's finding.**
Planting `GameObject` and `TurnState` — the two types whose already-existing fields F1, F2
and F3 write — in each gate's `CLOSURE_MUST_NOT_CONTAIN`:

* **HASH FAILS** (*"GameObject entered the GameState serde closure"*), i.e. `attachments`,
  `active_player`, `extra_turns` and `priority_holder` were **already on that wire**, which
  is exactly why writing them at a different moment adds nothing;
* **PROTOCOL stays GREEN**, because both are reachable only through `GameState`, which that
  list already excludes — the same asymmetry PB-DX51 measured with `CombatState`.

For **Half A** the counterfactual is not merely unmoved but **unexpressible**: `crates/engine`
does not depend on `mtg-simulator`, so a simulator type cannot be named in either list and
the plant would not compile (§0.4a).

### 4.4 The rest of the standard gates, against the FINAL tree

`clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean (it
**FIRED** once on the final tree, on a `pub const` line one character over the width, and
was fixed rather than swept — "clean against the FINAL tree" is only worth something if the
final tree is the one checked); `tools/check-defs-fmt.sh` clean (1,803 defs);
`cargo build --workspace` clean (the SR-3 seal gate).

**Coverage UNMOVED at 1,140/1,803 = 63.2%** by regeneration, **0 flips**, predicted with
the reason before any code: this batch authors no card text and repairs no card-def
blocker. **0 card-def edits of any kind** — `git diff main..HEAD --numstat` over
`crates/card-defs` and `crates/card-types/src/cards` is EMPTY, so the shortcut was
available and the regeneration was run anyway; self-dating churn reverted.

**`npm run build` was NOT run, and it is an N/A rather than a gap**:
`git diff main..HEAD --numstat -- tools/play-server/frontend` is **EMPTY** (the only
`tools/` change is one `..Default::default()` in `play-server/src/main.rs`'s `#[cfg(test)]`
`GameResult` literal), and `node_modules` is absent from this worktree.

**Benches: NOT measured, and the reason is a mechanism bound rather than an estimate.**
`crates/engine/benches/engine_perf.rs` contains **zero** occurrences of any symbol this
batch's engine half touches (`detach_from_host_on_departure`, `advance_turn`,
`grant_priority_to_active_player`, `extra_turns`, `attachments`). F1 adds one `Option`
test and, only when it is `Some`, one `retain` over a `Vector` that is empty for every
unattached permanent — on the zone-move path, which no bench drives (the six benches are
priority cycles, full turns, a board wipe and an SBA check). Everything else changed is in
`crates/simulator` and `bin/`, which the engine benches do not link.

---

## §5 The revert / bypass matrix, EXECUTED BY THE COORDINATOR

Rows R-A..R-E are this batch's own revert proofs (§3.2 for R-E). Rows A1..D3 are the
adversarial bypass pass. **Every file was restored byte-exactly (`cmp`) after every row.**

### 5.1 The delegated bypass agent had no shell, and said so

The `primitive-impl-reviewer` reported `Bash is disabled for this session, in subagents as
well as here`, planted nothing, executed nothing, and **put that at the top of its report
rather than presenting reasoning as results**. It produced 16 traced predictions instead.
**The coordinator then executed them.** That is worth recording as a positive: the
alternative — a report of 16 confident GREEN/RED verdicts that were actually inferences —
is precisely the failure mode this project keeps filing.

### 5.2 Result: **SEVEN of eight plants bypassed the shipped gates.** All seven are closed.

| row | plant | before | after the fix | what it means |
|---|---|---|---|---|
| **A1** | delete `check_attachment_symmetry(..)` from `check_all` | **GREEN** (74/74 + 6/6) | **RED** ×2 | both its probes called the PRIVATE fn directly, so nothing asserted it was dispatched. `check_stack_consistency` has had exactly this gate since SIM-3 and the new check did not get one. Closed by a front-door dispatch gate. |
| **A2** | make the `Err(_)` (dead-`ObjectId`) arm unreachable | **GREEN** | **RED** | that arm is the WHOLE of `OOS-DX22-8`'s direction B and **no test drove it** — both probe branches drive `Ok(att)`. |
| **B1''** | delete `check_no_dangling_attachment_at_rest` from `result_snapshot` | **GREEN** | **RED** | an end-state check with no call-site gate. |
| **B1b** | delete `check_no_leaked_tokens` from `result_snapshot` | **GREEN** | **RED** | **the hole is INHERITED from PB-DX32, not introduced here.** One gate keyed on the `check_no_` prefix covers both and a third. `OOS-DX56-5`. |
| **C1''** | add the promoted class to `is_transient_check` | **GREEN** | **RED** | routes the CR 800.4k promotion back into the transient bucket, silently voiding the entire justification for calling the CR 800.4j class transient. |
| **C2'** | make the promotion never fire | **GREEN** | **RED** | it had **no test of any kind** — the constant occurred in exactly two places workspace-wide, its declaration and its assignment. |
| **D1'** | remove F1 from ONLY the bottom-of-library site | **GREEN** | **RED** | `move_object_to_bottom_of_zone` is `pub(crate)`, so an integration test cannot call it. |
| **D3'** | drop the `!has_conceded` conjunct from F2's liveness test | **GREEN** | **RED** | every F2 probe used `has_lost`. CR 104.3a concession is one of the ways to leave. |

### 5.3 Two plants were NON-VERDICTS and are reported as such rather than counted

* **D2 (`while let` → `if let`)** does not compile — the `break` inside becomes
  `error[E0268]: break outside of a loop`. So its first two "GREEN" results were **build
  failures wearing a pass's clothes**, `OOS-DX39-8`'s shape for the third time in this
  batch (R-E was the first, a fmt-rewrapped plant the second). Re-planted as `if let` WITH
  the `break` removed — a plant that compiles — and closed by `t7`.
  **D2 is not merely a coverage gap: it is a fresh CR 500.7 violation.** With a dead entry
  queued ON TOP of a live one, an `if let` pops the dead one, abandons the live player's
  extra turn entirely, and falls through to normal order. CR 800.4k discards the DEAD entry
  and says nothing that would justify discarding the live one underneath it.
* **Two plants FAILED TO APPLY** because `cargo fmt` had rewrapped their target lines
  between the first pass and the second. Both were reported as non-verdicts by the harness's
  own `PLANT FAILED TO APPLY` line and redone against the wrapped spelling. *A plant that
  does not apply produces a green run, and a green run that nobody checked applied is
  indistinguishable from a gate that works.*

### 5.4 Two of the NEW gates caught themselves before shipping, both on their non-vacuity floors

* `t_every_class_constant_is_classified_by_its_own_name` parsed `pub const` **line by
  line** and found 3 hard constants instead of 4, because `cargo fmt` wraps
  `HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN`'s declaration across two lines. That is the
  multi-line-spelling blind spot PB-DX45's re-pin and PB-DX50's sentinel census each hit
  once. **The non-vacuity floor is what caught it** — which is the whole argument for
  putting one on a parsing gate.
* `t_every_end_state_check_is_called_from_result_snapshot` matched **its own source**
  through `include_str!` and extracted the "name" `").skip` from the literal
  `"pub fn check_no_"` inside its own body. A self-referential source gate that scans the
  file it lives in has to exclude itself; the cheapest honest way is to insist the capture
  is spellable as an identifier.

### 5.5 One row is honestly a SOURCE GATE and says so in its own module doc

`core::pb_dx56_departure_hygiene_roster` asserts both zone-move helpers call the CR 400.7
fix-up. It is a source gate rather than a probe because `move_object_to_bottom_of_zone` is
`pub(crate)` and an integration test is an external crate — **and because tracing that
helper's callers finds none that reaches it with an attached battlefield permanent**, so a
behavioural probe is not merely unwritten but unwritable today. The honest statement is
*"no probe AND no currently reachable path"*, not *"an untested defect"*, and it is
disclosed in the test file itself rather than only in `memory/` (`OOS-DX54-5`'s convention).
Its bodies are **brace-matched, not byte-windowed**, so it fails closed (`OOS-DX49-2`), and
each has a non-vacuity floor on body size.

---

## §6 Close-out figures (re-taken AFTER the bypass fix cycle — dispatch hygiene 8)

**Tests: 5,312 / 0 / 5** full-workspace, **+25** over the **5,287** baseline (which
reproduced PB-DX55's close pin exactly — the **eighth** consecutive batch in which an
inherited pin reproduces with no correction owed), on **72** result-producing targets
(unmoved: the new engine probes join existing targets and the simulator ones are unit
tests). Residual list empty.

**Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs**
— never `sort` + `comm` (`OOS-DX20b-5`), with the extraction regex deliberately NOT
end-anchored (`OOS-DX42b-6`), and **re-taken AFTER the bypass fix cycle rather than before
it** (dispatch hygiene 8 — the cycle added 10 tests, so the pre-cycle figure of 15 is
superseded by this line rather than left standing beside it): **25 additions, 0 leavers,
0 removals, 0 renames.** Count delta 25 == name-set delta 25, and the duplicate-name scan
the byte-exact method is structurally blind to (`OOS-DX35-8`) is **EMPTY on both runs**
(5,292 / 5,292 distinct; 5,317 / 5,317).

**"0 leavers" must NOT be read as "nothing was touched"** — two tests were edited IN PLACE
and their names are unchanged, so the name-set delta cannot see either, and both are
disclosed here rather than left for it to hide:

* `mechanics_e_l::extra_turns::test_extra_turn_eliminated_player_skipped` — its docstring
  DOCUMENTED the CR 800.4k violation F2 closes (*"the eliminated player may briefly be set
  as active_player … effectively a no-op turn"*) as the expected behaviour. **Corrected,
  and the correction is STRICTLY STRONGER**: its headline `assert_ne!(…, p1)` becomes
  `assert_eq!(…, p3)`, because with the fix the departed player's entry is structurally
  unable to become `active_player` at all.
* `invariants::tests::t_check_all_prepends_state_context_before_the_checks_own_evidence` —
  its `.find(|v| v.check == "player_consistency")` was repointed at
  `HARD_DEPARTED_PRIORITY_HOLDER` when the class split.

**Engine lines**: `crates/engine/src` **+82 / −20** across four files —
`state/mod.rs` +39/−0 (the shared CR 400.7 helper and its two call sites),
`rules/turn_structure.rs` +21/−1, `rules/engine.rs` +16/−11, `rules/priority.rs` +6/−8
(two doc inventories that became false). **`crates/card-types`, `crates/card-defs` and
`crates/view-model` are all EXACTLY 0.** `crates/simulator/src` is the bulk of the batch;
`tools/` is one `..Default::default()` inside a `#[cfg(test)]` `GameResult` literal.
