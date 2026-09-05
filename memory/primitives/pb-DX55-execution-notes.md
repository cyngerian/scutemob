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
