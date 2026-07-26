# Primitive Batch Plan: PB-DP2 — the mulligan is a content no-op; bottomed cards go to the library TOP

<!-- last_updated: 2026-07-26 -->

**Generated**: 2026-07-26
**Task**: `scutemob-150`
**Branch**: `feat/pb-dp2-mulligan-is-a-content-no-op-bottomed-cards-go-to-libr`
**Primitive**: none — this is a pure **correctness** PB. Two edits in one function pair
(`handle_take_mulligan` / `handle_keep_hand`), plus probes and doc close-out.
**CR Rules**: **103.5**, **103.5c** (and see the numbering verdict in §1 — the criteria's
"CR 103.4b" is *not* the mulligan rule)
**Cards affected**: **0** — do not touch `crates/card-defs/`.
**Dependencies**: none. PB-RS1 (`Zone::top_n` / bottom-write reconciliation) established the
class; the mulligan was simply not in its roster.
**Deferred items from prior PBs**: closes **OOS-M11-1** (filed `memory/m11-session-plan.md`
§8 risk row R2). Widens it to also cover the `handle_keep_hand` half, per audit §7.

**Wire expectation — HARD CONSTRAINT: PROTOCOL stays 27, HASH stays 63.** No new
`Command` / `Effect` / `GameEvent` variant, no new `GameState` field, no new struct field
anywhere. If the implementer concludes a fingerprint must be re-pinned — **STOP AND
RE-SCOPE**. §7 below names the gate tests and explains why they cannot move.

**Scope fence.** Do NOT generalise this into a shuffle/RNG refactor. Do NOT extract a shared
shuffle helper (see §4.4 — recommendation is *inline*, with a seed filed instead). Do NOT
touch card definitions. Do NOT touch `crates/simulator`. Two edits, one file.

---

## 1. CR verification — the numbering verdict (READ THIS FIRST)

Looked up live via the `mtg-rules` MCP on 2026-07-26. **Quoted verbatim.**

### 1.1 CR 103.5 — the entire London mulligan procedure

> **103.5.** Each player draws a number of cards equal to their starting hand size, which is
> normally seven. (Some effects can modify a player's starting hand size.) A player who is
> dissatisfied with their initial hand may take a mulligan. First, the starting player
> declares whether they will take a mulligan. Then each other player in turn order does the
> same. Once each player has made a declaration, all players who decided to take mulligans do
> so at the same time. **To take a mulligan, a player shuffles the cards in their hand back
> into their library, draws a new hand of cards equal to their starting hand size, then puts
> a number of those cards equal to the number of times that player has taken a mulligan on
> the bottom of their library in any order.** Once a player chooses not to take a mulligan,
> the remaining cards become that player's opening hand, and that player may not take any
> further mulligans. This process is then repeated until no player takes a mulligan. A player
> can take mulligans until their opening hand would be zero cards, after which they may not
> take further mulligans.

> **103.5c.** In a multiplayer game and in any Brawl game, the first mulligan a player takes
> doesn't count toward the number of cards that player will put on the bottom of their
> library or the number of mulligans that player may take. Subsequent mulligans are counted
> toward these numbers as normal.

(103.5a Vanguard hand size, 103.5b "any time you could mulligan", 103.5d shared-team-turns —
all out of scope, none contradict the above.)

### 1.2 CR 103.4 — what the task's cite actually is

> **103.4.** Each player begins the game with a starting life total of 20. Some variant games
> have different starting life totals.
>
> **103.4b.** In a Vanguard game, each player's starting life total is 20 plus or minus the
> life modifier of their vanguard card.
>
> **103.4c.** In a Commander game, each player's starting life total is 40.

### 1.3 VERDICT — state this plainly and do not silently substitute

**CR 103.5 is the correct and only cite for BOTH halves of this PB.** It carries the shuffle
*and* the bottoming in a single sentence. **CR 103.4b is starting life totals in a Vanguard
game and has nothing to do with mulligans** — the ESM acceptance-criterion text (5519) and
the retired golden script `cc32_mulligan_to_six.json` both carry a stale, incorrect cite.
This is a real numbering error, not a renumbering artifact: 103.4 has *never* been the
mulligan rule in the current CR, and there is no "London mulligan renumbering" to reconcile.
The engine's own comments (`rules/commander.rs:782-794`, `:856-865`) already cite 103.5 /
103.5c and are **correct as-is**.

**How the implementer must handle this (both, not either):**

- **Test doc comments and in-code comments cite CR 103.5** (plus 103.5c where the free-first-
  mulligan count is involved). This is the authoritative cite.
- **Name the bottom-placement probe `test_dp2_cards_to_bottom_land_on_library_bottom_cr_103_4b`**
  so acceptance criterion 5519 ("citing CR 103.4b") remains legibly satisfied by a grep, and
  in that test's doc comment write, verbatim:

  ```rust
  /// CR 103.5 — "…then puts a number of those cards equal to the number of times that
  /// player has taken a mulligan **on the bottom of their library in any order**."
  ///
  /// NOTE ON THE CITE: task `scutemob-150` criterion 5519 cites "CR 103.4b". That is
  /// stale — 103.4b is the *Vanguard starting life total*. The mulligan bottoming step
  /// lives in CR 103.5 (verified against the live CR, 2026-07-26, PB-DP2). The test name
  /// keeps the criterion's number so the criterion stays greppable; the rule text above
  /// is the authoritative one.
  ```

- Record the same correction in the close-out (§6) so it does not recur.

### 1.4 Is the engine's take/keep split of the CR sentence correct?

**Strictly, the CR puts the bottoming inside "take a mulligan"; the engine defers it to the
keep. The split is a benign, behaviourally-equivalent simplification — leave it alone.**

Argument (put this in the plan record, not in code):

- CR order within one mulligan is: shuffle → draw to starting hand size → bottom N.
- The engine does: `TakeMulligan` = shuffle → draw 7; `KeepHand` = bottom N, where
  N = `mulligan_count.saturating_sub(1)` (the 103.5c free-first adjustment).
- Under the CR, after mulligan #k a player holds `7 - k` cards. Under the engine they hold 7
  and bottom `k` on the keep. Final opening-hand size is identical.
- The only divergence would be the state of the library between mulligans — and the *next*
  mulligan shuffles the whole library anyway (CR 103.5's shuffle re-randomises any card the
  previous step bottomed). So no observable difference survives.

**Therefore: fix the shuffle in `handle_take_mulligan` and the direction in
`handle_keep_hand`, and do NOT move the bottoming step.** Moving it would be a scope
explosion (`Command::TakeMulligan` would need a `cards_to_bottom` field ⇒ PROTOCOL bump),
which the task explicitly forbids.

### 1.5 What the CR says about the *order* of bottomed cards

CR 103.5: "…on the bottom of their library **in any order**." **The player chooses the
order.** The CR does not constrain it, so the engine's obligation is only to *honour the
order the player supplied* in `Command::KeepHand { cards_to_bottom }` under a documented,
stable convention. The existing convention is documented at `rules/commander.rs:864-865`:

> `cards_to_bottom` lists the ObjectIds to put on the bottom of the library
> in order (index 0 = placed first, ends up above later entries).

**This convention is preserved, not inverted, by iterating `cards_to_bottom` in order and
calling `move_object_to_bottom_of_zone` (= `Zone::push_front` = `Vector::insert(0, …)`).**
Worked proof — this is the exact off-by-one-direction question the PB exists to settle:

```
Library storage is Vector<ObjectId>; index 0 is the BOTTOM, the LAST index is the TOP
  (Zone::top() == v.last(),  crates/card-types/src/state/zone.rs:159-164).

Start:               [L0, L1, ..., Ln]      (L0 bottom, Ln top)
cards_to_bottom = [A, B]                    (A is index 0 = "placed first")

  push_front(A)  ->  [A, L0, L1, ..., Ln]
  push_front(B)  ->  [B, A, L0, L1, ..., Ln]

Final: index 0 = B, index 1 = A, index 2.. = original library, TOP is still Ln.
```

A is at index 1, B is at index 0; a higher index is closer to the top; therefore **A (placed
first) ends up ABOVE B (placed later)** — exactly what the doc comment promises. It also
matches physical play: sliding card A under the deck makes it bottom-most, then sliding B
under puts B beneath A.

**REQUIRED FINAL LIBRARY ORDER — assert this literally in the probe:**

| library index | contents |
|---|---|
| `0` (deepest / bottom-most) | the **last** entry of `cards_to_bottom` |
| `1` | the second-to-last entry |
| … | … |
| `cards_to_bottom.len() - 1` | the **first** entry of `cards_to_bottom` (index 0) |
| `cards_to_bottom.len()` .. end | the pre-existing library, **unchanged and in order** |
| last index (top) | **unchanged** — the next card drawn is NOT a bottomed card |

No reversal of `cards_to_bottom` is needed. Do **not** add one. The doc comment at
`:864-865` is correct as written; §3.3 asks only for a tightening, not a semantic change.

---

## 2. The two defects (verified in source)

**(a) `handle_keep_hand` bottoms to the TOP.** `crates/engine/src/rules/commander.rs:886-890`:

```rust
    // Move each card from hand to bottom of library
    let lib_zone_id = ZoneId::Library(player);
    for obj_id in cards_to_bottom.iter() {
        state.move_object_to_zone(*obj_id, lib_zone_id)?;   // <-- push_back == TOP
    }
```

`Zone::insert` on an ordered zone is `v.push_back(id)` (`zone.rs:107-114`) and `Zone::top()`
is `v.last()` (`:159-164`). The comment says "bottom"; the code writes the top. The cards a
player bottoms are the next cards they draw.

**(b) `handle_take_mulligan` never permutes.** Same file, `:808-848`. Hand objects are moved
to the library (landing on top, in ascending-`ObjectId` order — `Hand` is
`Zone::Unordered(OrdSet)`), `GameEvent::LibraryShuffled` is pushed at `:824` with **no
permutation performed**, and 7 cards are drawn straight back off the top. The same seven
cards return, reversed. Two problems: CR 103.5's shuffle does not happen, and the emitted
event is a phantom (Architecture Invariant 4).

---

## 3. Engine changes

Exactly two edits, both in `crates/engine/src/rules/commander.rs`. Plus one file-level `use`
and two doc-comment tightenings.

### Change 1 — the (a) fix: bottom means bottom

**File**: `crates/engine/src/rules/commander.rs`
**Site**: `handle_keep_hand`, the loop at `:886-890`
**CR**: 103.5 ("on the bottom of their library in any order")

Replace the single call:

```rust
        state.move_object_to_zone(*obj_id, lib_zone_id)?;
```

with:

```rust
        state.move_object_to_bottom_of_zone(*obj_id, lib_zone_id)?;
```

**Verified reachability and composition:**

| question | answer | evidence |
|---|---|---|
| Does `GameState::move_object_to_bottom_of_zone` exist? | yes | `crates/engine/src/state/mod.rs:1610` |
| Is it the right helper? | yes — identical to `move_object_to_zone` except it inserts at index 0 (`push_front`) instead of appending | `state/mod.rs:1605-1614`; same CR 400.7 new-object semantics, same LKI capture (`:1636`), same SR-23 error-before-capture ordering (`:1625-1634`) |
| Visibility from `rules/commander.rs`? | **yes** — `pub(crate)`, same crate (`mtg-engine`) | `state/mod.rs:1610` |
| Error type composes with the existing `?`? | **yes** — returns `Result<(ObjectId, GameObject), GameStateError>`, byte-identical signature to `move_object_to_zone` | `state/mod.rs:1610-1614` |
| Prior art for this exact helper choice? | `rules/copy.rs:484` (cascade bottom-write, MR-M9.4-08) | — |

**Fallible vs. diagnostic variant — use the FALLIBLE one.** Do **not** use
`expect_move_object_to_bottom_of_zone` (`state/diagnostics.rs:308`). The site's existing
comment (`commander.rs:816-818`, MR-M9-12) makes a deliberate, recorded choice to *propagate*
move errors rather than swallow them:

> MR-M9-12: propagate move errors instead of silently dropping them with `let _ =`. During a
> pregame mulligan every hand card is in the hand zone, so a failed move signals a real state
> inconsistency that must surface.

`expect_*` is `debug_assert!` + `None` — release-silent. Propagation is strictly stronger and
already the established contract of both handlers (they return `Result<Vec<GameEvent>, …>`).
**Preserve it.** SR-4 classification: **engine-bug class, surfaced as an error** — correct.

**Do not add a reversal.** See §1.5. In-order iteration + `push_front` already produces the
documented and physically correct order.

### Change 2 — the (b) fix: a real seeded permutation

**File**: `crates/engine/src/rules/commander.rs`
**Site**: `handle_take_mulligan`
**CR**: 103.5 ("shuffles the cards in their hand back into their library")

**Exact insertion point**: after the hand→library move loop closes (`:822`), **before** the
`GameEvent::LibraryShuffled` push (`:824`), **before** the 7-card draw loop (`:830`). This is
the CR's own ordering: cards must be in the library before the library is shuffled, and the
draw must come off the shuffled library.

```rust
    let lib_zone_id = ZoneId::Library(player);
    for obj_id in hand_objects {
        state.move_object_to_zone(obj_id, lib_zone_id)?;
    }
    // CR 103.5: taking a mulligan shuffles the hand *into* the library — the library
    // must actually be permuted, or the same seven cards come straight back off the top.
    // MR-M7-17 / PB-DP2: seed from `timestamp_counter` (not entropy) so replay is
    // deterministic (SR-9b). Same idiom as `effects/mod.rs:8697-8703`.
    let seed = state.timestamp_counter;
    state.timestamp_counter += 1;
    let library = state
        .zones
        .get_mut(&lib_zone_id)
        .ok_or(GameStateError::ZoneNotFound(lib_zone_id))?;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    library.shuffle(&mut rng);
    // The shuffle above is what makes this event non-phantom (Architecture Invariant 4).
    events.push(GameEvent::LibraryShuffled { player });
```

**Shuffle unconditionally**, even when the hand is empty. CR 103.5's shuffle is part of
taking a mulligan, not conditional on hand contents; the event is already unconditional; and
the empty-hand case is what makes probe P1 (§5.1) clean.

**Every reachability question, answered:**

| question | answer | evidence |
|---|---|---|
| `state.timestamp_counter` accessible from `rules/commander.rs`? | **yes** — `pub(crate)` field | `state/mod.rs:230` |
| `state.zones` accessible as a field? | **yes** — already used in this very function | `commander.rs:811`, `:832` |
| `expect_zone_mut` accessible? | yes, `pub(crate)` (`state/diagnostics.rs:217`) — but **do not use it**, see below | — |
| Is `rand` an engine dependency? | **yes**, `rand = "0.9"` | `crates/engine/Cargo.toml:15` |
| Is `StdRng` available? | yes — `rand` default features include `std_rng`; three existing engine call sites | `effects/mod.rs:3048`, `:3147`, `:8701` |
| Is `SeedableRng` in scope in `commander.rs`? | **no — must be added** | `commander.rs:5-14` has no `rand` import |
| Is the `Rng` trait needed in scope? | **no** — `Zone::shuffle(&mut self, rng: &mut impl Rng)` takes `impl Trait` in argument position; the caller needs only a concrete `StdRng`. `effects/mod.rs:39` imports only `SeedableRng`, confirming this. | `zone.rs:138`; `effects/mod.rs:39` |

**`expect_zone_mut` vs `zones.get_mut` — use `get_mut(..).ok_or(ZoneNotFound)?`.**
SR-4 classification: this is an **engine-bug-class** site, not an LKI-fizzle site — a missing
library zone during a pregame mulligan is real state corruption, never a legal fizzle.
`expect_zone_mut` *is* the engine-bug-class helper, but it is `debug_assert!` + `None`, i.e.
**release-silent**: in a release build the mulligan would quietly skip the shuffle and
re-emit the phantom event — reintroducing the exact bug this PB fixes. `get_mut(..).ok_or(..)?`
propagates a `GameStateError::ZoneNotFound` in *both* profiles and matches the function's own
MR-M9-12 precedent (§Change 1) and its `Result` return type. It is strictly stronger and
strictly more legible. Choose it.

Borrow note: read and bump `timestamp_counter` *before* taking `&mut state.zones`. Field-level
borrows are disjoint, so this compiles either way, but ordering it this way keeps it obvious
(cf. the disjointness note at `state/diagnostics.rs:94-103`).

### Change 3 — the `use`

**File**: `crates/engine/src/rules/commander.rs`, with the existing `use` block at `:5-14`
**Action**: add

```rust
use rand::SeedableRng;
```

File-level, matching `effects/mod.rs:39`. Do not import `Rng`; it is unused and clippy will
reject it under `-D warnings`.

### Change 4 — doc-comment tightening (no behaviour)

Two small edits so the next reader does not have to re-derive §1.5.

1. `handle_take_mulligan`'s doc block (`:783-794`). The line "This function handles drawing 7
   cards after shuffling" now understates it. Amend to state that the function performs the
   CR 103.5 shuffle itself, with a deterministic `timestamp_counter`-derived seed
   (MR-M7-17 / SR-9b), and that `LibraryShuffled` is emitted only after a real permutation.
2. `handle_keep_hand`'s doc block, the `cards_to_bottom` line at `:864-865`. Keep the
   convention; make the resulting position explicit:

   ```rust
   /// `cards_to_bottom` lists the ObjectIds to put on the bottom of the library in the
   /// player's chosen order (CR 103.5: "in any order"). Index 0 is placed first and
   /// therefore ends up ABOVE later entries: the LAST entry is the bottom-most card in
   /// the library. Implemented with `move_object_to_bottom_of_zone` (`push_front`), so
   /// the pre-existing library — including its top card — is untouched.
   ```

Also fix the stale "put N-1 cards ... on the bottom" phrasing only if it is wrong; it is
correct (103.5c free first mulligan), so leave the arithmetic alone.

### Change 5 — exhaustive match sites

**NONE.** No enum variant is added, no struct field is added, no `HashInto` impl changes.
There is nothing to add to `state/hash.rs`, `tools/replay-viewer/src/view_model.rs`,
`tools/tui/src/play/panels/stack_view.rs`, or `state::keyword_registry::handling`. This is
the whole reason the wire expectation holds.

---

## 4. Determinism, blast radius, and the helper question

### 4.1 Why seeding from `timestamp_counter` preserves replay determinism (SR-9b)

- `build_initial_state` is deterministic (`sorted_zone_entries`, SR-9b), and
  `GameStateBuilder` starts `timestamp_counter` at `0` (`state/builder.rs:333`).
- `timestamp_counter` advances only as a pure function of the command sequence: every
  `next_object_id()` bumps it by exactly 1 (`state/mod.rs:875-878`), plus the handful of
  explicit `+= 1` seed bumps. **No entropy source anywhere.**
- A replay re-executes the same `Command` stream from the same start state ⇒ identical
  counter values at every step ⇒ identical seeds ⇒ identical `StdRng` streams ⇒ identical
  Fisher-Yates permutations (`Zone::shuffle`, `zone.rs:138-147`, `rng.random_range(0..=i)`).
- This is exactly the property MR-M7-17 chose this seed source for, at three existing sites.

### 4.2 Is the extra `timestamp_counter` increment harmless? — YES

- `timestamp_counter` is *already* advanced 14 times per mulligan by this very function
  (7 hand→library moves + 7 draws, each allocating a new `ObjectId` via `next_object_id()`,
  `state/mod.rs:875-878`). One more increment is noise against that.
- Its only semantic consumer is `GameObject.timestamp`, used for CR 613.7 layer ordering
  among continuous effects. The counter is **monotonic**, so an extra increment shifts
  absolute values without changing any *relative* order — no dependency, no layer outcome,
  no timestamp tie can flip.
- During the pregame mulligan the battlefield is empty and there are no continuous effects to
  order at all.
- It is hashed (`state/hash.rs:7662`), so a mulligan's *state hash value* changes. That is a
  value, not a schema — see §7. `loop_detection_hashes` (CR 104.4b) is a turn-loop guard and
  is not consulted pregame.

No safer seed derivation is needed. Use `timestamp_counter`.

### 4.3 Blast radius — verified by grep, not assumed

`Command::TakeMulligan` / `Command::KeepHand` appear in exactly these code locations:

| location | effect of this change |
|---|---|
| `crates/engine/src/rules/command.rs`, `rules/engine.rs:244-256` | dispatch only; unchanged |
| `crates/engine/src/rules/commander.rs` | **the two edits** |
| `crates/engine/tests/rules/commander.rs` | 3 existing tests — see §5.4 |
| `crates/simulator/src/{legal_actions,random_bot,heuristic_bot}.rs` | **unreachable** — see below |
| `test-data/generated-scripts/commander/cc32_mulligan_to_six.json` | **retired** — see §5.5 |

**The simulator, the driver and the fuzzer cannot reach a mulligan.** `legal_actions.rs:186`
gates `LegalAction::TakeMulligan/KeepHand` on
`state.turn().is_first_turn_of_game && state.turn().turn_number == 0`, and the tree contains
an authoritative in-source statement that this is dead today —
`crates/simulator/src/local_game.rs:569-574`:

> The `Mulligan` arm (CR 103.5) is **currently unreachable**: it needs `turn_number == 0`
> (mirroring `StubProvider`'s own mulligan gate), but `GameStateBuilder` defaults
> `turn_number` to 1 and nothing in the tree sets it to 0. Mulligans also need per-player
> resolution … Session 2 owns pregame setup and will make this reachable.

Therefore **no** `GameDriver` game, `LocalGame` test, fuzzer run, `crates/simulator/tests/
local_game.rs` assertion, or `invariants.rs` snapshot can observe this change. The
implementer should still re-run `cargo test --all` (which covers all 31 workspace test
binaries) and report if anything outside `crates/engine/tests/rules/commander.rs` moves —
**a failure there would falsify this analysis and is a stop-and-report event.**

Pre-existing landmine worth knowing but **not fixing here**: `random_bot.rs:240-242` always
sends `KeepHand { cards_to_bottom: vec![] }`, which `handle_keep_hand` rejects after a second
mulligan. Unreachable today; it becomes M11-local Session 2's problem. Note in the handoff.

### 4.4 Factor the shuffle into a shared helper? — **NO. Inline it.** (recommendation, with reasoning)

The seeded-shuffle idiom now appears **four** times: `effects/mod.rs:3048`, `:3147`, `:8701`,
and this new site. A `GameState::shuffle_library_seeded(&mut self, player)` helper is the
right eventual home. **Do not extract it in this PB:**

- The three existing sites have materially different surroundings — `:8701` is inside
  `move_zone_all_then_shuffle`, which also gathers objects by zone filter and uses
  `expect_move_object_to_zone` (error-swallowing, the *opposite* of what this site needs per
  MR-M9-12). Reusing `move_zone_all_then_shuffle` directly is therefore **wrong** here, not
  merely inconvenient.
- Extracting a helper means editing three proven call sites inside a PB whose job is to fix a
  correctness bug. That trades a bounded 2-edit diff for a 4-site refactor with a wider
  clippy/test surface, against an explicit "do not expand this into a general shuffle/RNG
  refactor" directive.

**Action**: inline (as written in Change 2), and file the dedup as a seed (§6, OOS-DP2-4).

---

## 5. Test plan

**File**: `crates/engine/tests/rules/commander.rs` — append after the existing mulligan block
(`:1345-1620`). Do **not** create a new top-level `tests/*.rs` (SR-9a).

**Shared accessors and facts the probes rely on (all verified):**

- `state.zone(&ZoneId::Library(p)).unwrap().object_ids()` is public and, for an ordered zone,
  returns the backing `Vector` in storage order: **index 0 = BOTTOM, last index = TOP**
  (`zone.rs:130-135`, `:159-164`; `ZoneId::is_ordered()` at `zone.rs:57-62` includes Library).
- `ZoneId::Hand(p)` is **Unordered** (`OrdSet`); `object_ids()` yields ascending `ObjectId` —
  deterministic, but position carries no game meaning.
- **`ObjectId`s are NOT stable across a move (CR 400.7).** Every probe must identify cards by
  **name**, never by pre-move `ObjectId`. Use
  `state.object(id).unwrap().characteristics.name.clone()`. `characteristics` is cloned
  verbatim onto the new object (`state/mod.rs:1652`), and `ObjectSpec::card(owner, name)`
  sets that name (`state/builder.rs:1451-1453`). `GameEvent::MulliganKept.cards_to_bottom`
  carries the **pre-move** ids and is therefore useless for position assertions — don't use it.
- Suggested local helper at the top of the new block:

  ```rust
  /// Card names of an ordered zone, BOTTOM-first (index 0 == bottom of library).
  fn zone_names(state: &mtg_engine::GameState, z: &ZoneId) -> Vec<String> {
      state
          .zone(z)
          .unwrap()
          .object_ids()
          .into_iter()
          .map(|id| state.object(id).unwrap().characteristics.name.clone())
          .collect()
  }
  ```

- Reuse `build_state_with_library(player, n)` (`:1348-1360`); it builds
  `"Card 0" … "Card n-1"` into the library with `"Card 0"` at the bottom and `"Card n-1"` on
  top, and leaves the hand empty. **Use `n = 40`** in the new probes (the existing tests use
  20; 40 gives the permutation probes their headroom — see §5.2).

### 5.1 `test_dp2_cards_to_bottom_land_on_library_bottom_cr_103_4b` — the (a) probe

Name per §1.3. CR 103.5 in the doc comment, with the cite note.

```
setup:   build_state_with_library(p1, 40)          // hand empty, library Card 0..Card 39
         TakeMulligan x3                            // required_bottom = 3 - 1 = 2 (CR 103.5c)
         // hand: 7 cards, library: 33
capture: let lib_before = zone_names(&state, &ZoneId::Library(p1));   // 33, bottom-first
         let hand = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids();
         let (a_id, b_id) = (hand[0], hand[1]);
         let name_a = name_of(a_id);  let name_b = name_of(b_id);
act:     KeepHand { player: p1, cards_to_bottom: vec![a_id, b_id] }
assert:  let lib_after = zone_names(&state, &ZoneId::Library(p1));
         assert_eq!(lib_after.len(), 35);
         // CR 103.5: bottom means bottom, and the LAST cards_to_bottom entry is deepest.
         assert_eq!(lib_after[0], name_b, "last cards_to_bottom entry is the bottom-most card");
         assert_eq!(lib_after[1], name_a, "index-0 entry sits ABOVE later entries");
         assert_eq!(&lib_after[2..], &lib_before[..],
                    "pre-existing library must be untouched and in order");
         // The next card drawn must NOT be a bottomed card.
         let top = state.zone(&ZoneId::Library(p1)).unwrap().top().unwrap();
         let top_name = state.object(top).unwrap().characteristics.name.clone();
         assert_eq!(top_name, *lib_before.last().unwrap());
         assert_ne!(top_name, name_a);
         assert_ne!(top_name, name_b);
         assert_eq!(state.zone(&ZoneId::Hand(p1)).unwrap().len(), 5);
```

**Fail-before / pass-after**: pre-fix `move_object_to_zone` appends, so
`lib_after == lib_before ++ [name_a, name_b]` — `lib_after[0]` is `lib_before[0]`, not
`name_b`, and `top()` is `name_b`. Both the position assertion and the next-draw assertion
fail. Post-fix all pass. Deterministic; no randomness in the assertion.

### 5.2 `test_dp2_mulligan_actually_permutes_the_library_cr_103_5` — the (b) probe, non-flaky by construction

**This is the delicate one. The construction below is deterministic AND has a coincidence
probability of ~1e-11 to ~1e-48, so it is not a coin flip in any sense.**

Two independent reasons it cannot be flaky:

1. **The seed is fixed.** `timestamp_counter` is a pure function of the command sequence from
   a deterministic `build_initial_state` (§4.1). The permutation this test produces is the
   *same permutation on every run, on every machine*. The test either always passes or always
   fails — it can never flap. This alone discharges the flakiness concern.
2. **The assertion targets the whole 40-card permutation, not the 7-card hand**, so even the
   *hypothetical* coincidence is astronomically small (identity permutation on 40 elements:
   1/40! ≈ 1e-48).

```
setup:   build_state_with_library(p1, 40)      // hand EMPTY, so nothing moves hand->library
         let pre = zone_names(&state, &ZoneId::Library(p1));   // ["Card 0", ..., "Card 39"]
act:     TakeMulligan { player: p1 }
capture: let lib_after = zone_names(&state, &ZoneId::Library(p1));   // 33, bottom-first
         // draw order: draw #1 took the top, #2 the next, ...  (handle_take_mulligan :830-848)
         // recover the drawn cards in DRAW order from the CardDrawn events:
         let drawn: Vec<String> = events.iter().filter_map(|e| match e {
             GameEvent::CardDrawn { player, new_object_id } if *player == p1 =>
                 Some(state.object(*new_object_id).unwrap().characteristics.name.clone()),
             _ => None,
         }).collect();
         assert_eq!(drawn.len(), 7);
         // Reconstruct the full post-shuffle library order, bottom-first:
         let mut full = lib_after.clone();
         full.extend(drawn.iter().rev().cloned());       // draw #7 ... draw #1 back on top
         assert_eq!(full.len(), 40);
assert:  // (1) nothing was lost or duplicated -- it really is a permutation
         let (mut s_full, mut s_pre) = (full.clone(), pre.clone());
         s_full.sort(); s_pre.sort();
         assert_eq!(s_full, s_pre, "shuffle must be a permutation, not a rewrite");
         // (2) CR 103.5: the library was actually permuted
         assert_ne!(full, pre, "CR 103.5: taking a mulligan must shuffle the library");
         // (3) the sharpest fail-before form: pre-fix the draw is EXACTLY the top 7 in order
         let unshuffled_top7: Vec<String> =
             pre.iter().rev().take(7).cloned().collect();     // Card 39 ... Card 33
         assert_ne!(drawn, unshuffled_top7,
             "pre-fix the mulligan drew the untouched top 7 in order (CR 103.5 violation)");
         // (4) the event is no longer phantom
         assert!(events.iter().any(|e|
             matches!(e, GameEvent::LibraryShuffled { player } if *player == p1)));
```

**Fail-before**: with an empty hand and no shuffle, the library is untouched, so
`full == pre` (assertion 2 fails) and `drawn == ["Card 39", …, "Card 33"]` (assertion 3
fails). **Pass-after**: both hold for the fixed seed. Assertion 1 is a standing guard that
`Zone::shuffle` neither drops nor duplicates.

**If assertion 2 or 3 somehow fails post-fix** (it will not, but state the escape hatch):
that is a *deterministic* failure, not flake — bump the library size in this probe from 40 to
41 and re-run. Do not add retries, do not add `#[ignore]`, do not weaken the assertion to a
set comparison.

### 5.3 `test_dp2_mulligan_returns_a_different_hand_cr_103_5` and `test_dp2_mulligan_permutation_is_deterministic_cr_103_5`

**(i) The literal OOS-M11-1 headline — "the same seven cards return".**

```
setup:   builder with 7 named cards in ZoneId::Hand(p1) ("H0".."H6")
         and 40 named cards in ZoneId::Library(p1) ("Card 0".."Card 39")
capture: let hand_before: BTreeSet<String> = names of hand
act:     TakeMulligan { player: p1 }
assert:  assert_eq!(state.zone(&ZoneId::Hand(p1)).unwrap().len(), 7);
         let hand_after: BTreeSet<String> = names of hand
         assert_ne!(hand_after, hand_before,
             "CR 103.5: a mulligan shuffles the hand into a 47-card library; the same \
              seven cards must not come straight back (OOS-M11-1)");
         assert_eq!(state.zone(&ZoneId::Library(p1)).unwrap().len(), 40);
```

Compare **sets**, not sequences: pre-fix the hand returns *reversed*, so a sequence
comparison would pass for the wrong reason and the probe would not fail-before. Coincidence
probability post-fix: `1 / C(47,7)` ≈ 1/6.2e7, and again the seed is fixed so it is
deterministic either way.

**(ii) Determinism — same start state, same permutation, twice.**

```
setup:   let base = build_state_with_library(p1, 40);
act:     let (s1, _) = process_command(base.clone(), Command::TakeMulligan { player: p1 })?;
         let (s2, _) = process_command(base.clone(), Command::TakeMulligan { player: p1 })?;
assert:  assert_eq!(zone_names(&s1, &ZoneId::Library(p1)),
                    zone_names(&s2, &ZoneId::Library(p1)),
                    "SR-9b: the mulligan shuffle is seeded from timestamp_counter, so two \
                     runs from the same start state must produce the same permutation");
         // and the hands match too (BTreeSet, since Hand is unordered)
         assert_eq!(hand_names_sorted(&s1), hand_names_sorted(&s2));
         assert_eq!(s1.timestamp_counter(), s2.timestamp_counter());
```

Note `process_command` takes ownership of `GameState` — `.clone()` the base for each call
(standard gotcha). `timestamp_counter()` is a public accessor (`state/mod.rs:591`).

This probe passes both before and after the fix (a no-op is trivially deterministic); its
job is to **pin the property against a future entropy-seeded regression**, and to satisfy
acceptance criterion 5520's "deterministic per seed" clause. Say so in its doc comment so a
reviewer does not flag it as a non-fail-before test.

### 5.4 Existing tests — expected impact: NONE. Verify, don't assume.

| test | line | assertion shape | verdict |
|---|---|---|---|
| `test_free_mulligan_then_london_mulligan` | `:1365` | hand/library **counts**, `MulliganTaken`/`MulliganKept` event shapes | **unaffected.** At `:1465` it picks `hand.object_ids()[0]` — the lowest `ObjectId` in an `OrdSet`. Post-shuffle the hand holds different *cards*, but `[0]` still resolves to a live in-hand object, `handle_keep_hand` still moves it, hand still goes 7→6. |
| `test_mulligan_keep_wrong_count_rejected` | `:1499` | rejects `cards_to_bottom: vec![]` after 2 mulligans | **unaffected.** The count check at `:877-885` runs *before* any move; the shuffle changes which cards are where, not how many. |
| `test_mulligan_sequence_four_players` | `:1525` | per-player `mulligan_count`, hand counts | **unaffected.** Same `object_ids()[0]` pattern at `:1575`; per-player libraries; counts only. |

**No existing test assumes the hand is stable across a mulligan** and **none inspects library
position.** That is precisely why the bug survived — record that in the review handoff.

Implementer instruction: run
`~/.cargo/bin/cargo test -p mtg-engine --test rules -- commander` before and after and
confirm the three pre-existing tests pass in both states. If any changes status, **stop and
report** — it falsifies this table.

### 5.5 Golden scripts — none to reconcile. Verified by grep.

`grep -rl mulligan test-data/` returns exactly one file:
`test-data/generated-scripts/commander/cc32_mulligan_to_six.json`. It is
**`"review_status": "retired"`** with retirement reason (SR-9c):

> `mulligan_decision` and `choose_option` have no translation arms; `Command::TakeMulligan`/
> `KeepHand` exist but the pregame mulligan procedure is not scriptable through the harness.

Corroborated at `crates/engine/tests/scripts/run_all_scripts.rs:299`. **It does not execute;
nothing to reconcile.**

*Optional, cosmetic, LOW*: that script's `cr_sections_tested` contains `"103.4"` and
`"103.4b"`, and two `cr_ref` fields say `"103.4b"` — the same wrong cite §1.3 corrects. This
is exactly the **OOS-DP1-3** stale-citation class. The implementer **may** fix the four cite
strings in that one retired file as a one-line-each courtesy, or leave it to the OOS-DP1-3
doc pass. **Do not** un-retire the script and do not touch `review_status`. If unsure, leave
it and note it in the handoff.

### 5.6 Test-count expectation

Baseline **3,721**. This PB adds **4** tests → expect **3,725** passing / 0 failing. If the
delta is anything other than +4, investigate before proceeding.

---

## 6. Doc close-out — the exact edits

| # | file | edit |
|---|---|---|
| 1 | `memory/m11-session-plan.md` — §8 risk table row **R2** (grep `OOS-M11-1`; ~`:913`) and the resolutions bullet (~`:941-947`) | Mark **OOS-M11-1 CLOSED by PB-DP2 (`scutemob-150`)**. R2's "Out of M11-local's scope to fix properly (a caller-supplied permutation would be a new `Command` → wire change)" is **falsified** — the fix needed no wire change at all, because the engine already had a deterministic `timestamp_counter`-seeded PRNG. Say so explicitly; it is the reusable lesson. Note that Session 2's pregame `redeal` workaround is no longer load-bearing for correctness (Session 2 may still want it for UX). Bump `<!-- last_updated: -->` if the file carries one. |
| 2 | `docs/audits/decision-point-audit.md` §5, **DP-2 row** (`:429`) | Prefix with **`SHIPPED (PB-DP2, scutemob-150).`** Record both halves fixed: `handle_keep_hand` → `move_object_to_bottom_of_zone`; `handle_take_mulligan` → real seeded `Zone::shuffle`. Correct the row's own CR column from `103.4b, 103.5` to **`103.5, 103.5c`**, and add a one-clause note that 103.4b is the Vanguard starting life total (§1.3). |
| 3 | `docs/audits/decision-point-audit.md` §8, **PB-DP2 row** (`:561`) | **Correct the falsified wire prediction.** The row says "(b) needs a seed on `GameState` ⇒ **HASH bump**". Replace with "**none** — reuses the existing `timestamp_counter` seed source (`effects/mod.rs:8697-8703`); PROTOCOL 27 / HASH 63 unmoved." Note that (b) did **not** need to trail (a): both shipped together in one 2-edit PB. |
| 4 | `docs/audits/decision-point-audit.md` §7, OOS-M11-1 subsection (`:495-517`) | Add a closing line: confirmed and closed by PB-DP2, including the widening the audit itself recommended (`handle_keep_hand`). |
| 5 | `docs/audits/decision-point-audit.md` **§8.1 seed table** (`:587-592`) | Append the new seeds below. This table is the suite's durable inventory — `memory/primitive-wip.md` is rewritten wholesale by the next run, so a seed recorded only there is lost. |
| 6 | `docs/audits/decision-point-audit.md` header | Bump `<!-- last_updated: 2026-07-26 -->`. |
| 7 | `CLAUDE.md` "Current State" | One-sentence delta on the PB-DP suite line: **PB-DP2 SHIPPED** (`scutemob-150`) — mulligan shuffle + bottom-write, CR 103.5/103.5c, **OOS-M11-1 CLOSED**, 4 probes, no wire change (PROTOCOL 27 / HASH 63), tests 3,721 → 3,725; **next: PB-DP3**. Update the "Last Updated" line's date/summary. **Do not** rewrite the archive; per the recurrence rule, detailed narrative goes to `memory/archive/claude-md-changelog-2026-07.md` at `/collect`, not here. |
| 8 | `memory/workstream-state.md` | Append the PB-DP2 handoff (what shipped, the CR-numbering correction, the seeds, the simulator-unreachability finding). |

**New seeds to file in §8.1** (all discovered while planning; none in scope for this PB):

| seed | finding | class |
|---|---|---|
| **OOS-DP2-1** | **`handle_keep_hand` never verifies that `cards_to_bottom` entries are in the player's hand.** It checks only the *count* (`commander.rs:877-885`) and then moves each id from wherever it is. A malformed or hostile `KeepHand` can bottom a card from the battlefield, the graveyard, or **another player's hand**. Needs an `obj.zone == ZoneId::Hand(player)` guard per entry, plus a duplicate-id check. | correctness, validation gap |
| **OOS-DP2-2** | **Starting hand size is hard-coded to 7.** `handle_take_mulligan:830` draws `for _ in 0..7`; CR 103.5 says "equal to their **starting hand size**" and CR 103.5a (Vanguard) plus any starting-hand-size-modifying effect can change it. No `starting_hand_size` exists on `PlayerState`. Adding one is a **HASH bump**, so it is its own PB. | correctness, deferred (wire) |
| **OOS-DP2-3** | **All engine shuffles are predictable from public state.** `timestamp_counter` is a hashed, replayable field, so any client that can compute the state can compute every future shuffle — including its opponents' libraries. Pre-existing and engine-wide (4 sites), not introduced here; the deck order is already deterministic from `build_initial_state`. Bears on Architecture Invariant 7 and M10's hidden-information story: a networked build needs a server-held secret seed. | security / hidden-info, M10-gated |
| **OOS-DP2-4** | **The seeded-shuffle idiom is copy-pasted at 4 sites** (`effects/mod.rs:3048`, `:3147`, `:8701`, `rules/commander.rs`). A `GameState::shuffle_library_seeded(&mut self, player)` on `state/mod.rs` would dedupe it. Deliberately not done in PB-DP2 (§4.4): the extraction touches three proven call sites with different error-handling contracts. Batch into a cleanup pass. | cosmetic / refactor |
| **OOS-DP2-5** | **`RandomBot`/`HeuristicBot` send `KeepHand { cards_to_bottom: vec![] }` unconditionally** (`random_bot.rs:240-242`), which `handle_keep_hand` rejects after a 2nd mulligan. Unreachable today (mulligans are gated off in the simulator, `local_game.rs:569-574`) but becomes live the moment M11-local Session 2 sets `turn_number = 0`. | correctness, latent; M11-local S2 owns |
| **OOS-DP2-6** | **The engine defers CR 103.5's bottoming from take-time to keep-time.** Behaviourally equivalent (§1.4) because the next mulligan reshuffles the library, but it is a documented divergence from the CR sentence and would matter if any future effect ever observed the library between a mulligan and a keep. Record-only; no action recommended. | documentation / known divergence |

---

## 7. Verification checklist and gates

Run from the worktree root with `~/.cargo/bin/cargo`.

- [ ] `cargo check -p mtg-engine` — the two edits compile
- [ ] `cargo build --workspace` — SR-3 seal gate + the `tools/` exhaustive-match trap (nothing
      expected here, but this is the gate that catches it)
- [ ] `cargo test --all` — **expect 3,725 passing / 0 failing** (baseline 3,721 + 4)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `tools/check-defs-fmt.sh` (SR-35 — also runs inside `cargo test --all` as
      `core card_defs_fmt`; run it standalone anyway)
- [ ] The three pre-existing mulligan tests pass **before and after** (§5.4)
- [ ] No file under `crates/card-defs/` or `crates/simulator/` is modified

**Wire gates — PROTOCOL must stay 27, HASH must stay 63.** Name and confirm green:

| gate test | target | why it cannot move |
|---|---|---|
| `core::protocol_schema::protocol_schema_fingerprint_is_pinned` | `PROTOCOL_SCHEMA_FINGERPRINT` (`rules/protocol.rs:277`) | a blake3 digest of the **transitive type closure** of `Command`/`GameEvent`/`ReplayLog`. No type in that closure changes. |
| `core::protocol_schema::protocol_version_sentinel` | `PROTOCOL_VERSION` | unchanged at 27 |
| `core::protocol_schema::history_tail_matches_the_fingerprint_const` | `PROTOCOL_HISTORY` | no new row appended |
| `core::hash_schema::declaration_fingerprint_is_pinned` | `GameState` hashed-type **declaration** closure | no struct field added or removed anywhere |
| `core::hash_schema::stream_fingerprint_is_pinned` | the hash **byte stream** of a fixed `canonical_fixture()` (`tests/core/hash_schema.rs:711`) | the fixture is a hand-built `GameState`, not one produced by a mulligan; no `HashInto` impl body changes |
| `core::hash_schema::hash_schema_version_sentinel` | hash schema version | unchanged at 63 |
| `core::hash_schema::every_hashed_struct_field_is_hashed_or_allowlisted` | field-coverage scan | no new field |

Both fingerprints are **source-shape digests**, not value digests — the extra
`timestamp_counter` increment changes a runtime *hash value*, which no gate pins. **If any of
these seven tests fails, STOP AND RE-SCOPE — do not re-pin a fingerprint.**

---

## 8. Risks & edge cases

- **The CR-cite trap (highest risk of a wrong artifact).** An implementer who takes "CR
  103.4b" at face value will write a test doc comment citing the Vanguard life-total rule.
  §1.3 is mandatory reading; the required test name and the required doc-comment note are
  spelled out there.
- **Direction inversion (the whole point of the PB).** `push_front` inserts at *vector index
  0*, which is the *bottom*, because `top()` is `last()`. It is easy to talk oneself into a
  reversal of `cards_to_bottom`. **There is no reversal.** §1.5 has the worked proof and §5.1
  the literal assertion; if the probe passes without a reversal, the direction is right.
- **`ObjectId` instability (CR 400.7).** Every move mints a fresh id. Any probe that stores a
  pre-move `ObjectId` and looks for it in the library afterwards will fail for the wrong
  reason. Identify by `characteristics.name` throughout (§5 preamble). Likewise
  `MulliganKept.cards_to_bottom` carries stale ids — not usable for position assertions.
- **Set vs sequence in §5.3(i).** Pre-fix the hand comes back *reversed but identical as a
  set*. A sequence comparison would pass pre-fix and destroy the fail-before property.
  Compare sets.
- **`expect_zone_mut` is release-silent.** Using it would let a release build skip the shuffle
  and re-emit the phantom event — reintroducing the bug in the one profile that matters.
  §Change 2 mandates `get_mut(..).ok_or(ZoneNotFound)?`.
- **Reusing `move_zone_all_then_shuffle` is wrong**, not just inelegant: it uses
  `expect_move_object_to_zone`, which swallows move errors — the exact behaviour MR-M9-12
  removed from this handler. Inline the 5-line idiom.
- **`cargo test --all` surprises outside `crates/engine/tests/rules/commander.rs`** would
  falsify §4.3's unreachability analysis (which rests on an in-source claim at
  `local_game.rs:569-574` and the `turn_number == 0` gate). Treat any such failure as a
  stop-and-report event, not something to paper over.
- **Coordination with M11-local Session 2.** S2 owns pregame setup and will set
  `turn_number = 0`, making mulligans reachable in the simulator for the first time. When it
  does, OOS-DP2-5 (bots always send an empty `cards_to_bottom`) goes live immediately.
  Coordinate; do not block on it — this PB fixes the engine path either way.
- **Do not expand.** Starting hand size (OOS-DP2-2), hand-membership validation
  (OOS-DP2-1), the shared shuffle helper (OOS-DP2-4) and the take-time/keep-time split
  (OOS-DP2-6) are all real and all out of scope. File them; do not fix them.
