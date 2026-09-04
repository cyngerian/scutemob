# Primitive Batch Review: PB-DP2 — the mulligan is a content no-op; bottomed cards go to the library TOP

<!-- last_updated: 2026-07-26 -->

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-150`
**Branch**: `feat/pb-dp2-mulligan-is-a-content-no-op-bottomed-cards-go-to-libr`
**Commits reviewed**: `f902010f` (engine + probes), `7aa8914b` (doc close-out)
**CR Rules re-derived live via `mtg-rules` MCP**: 103.5, 103.5a, 103.5b, 103.5c, 103.5d, 103.4, 103.4a-e

**Engine files reviewed**:
- `crates/engine/src/rules/commander.rs` (the two edits + `use rand::SeedableRng` + 2 doc blocks)
- `crates/engine/tests/rules/commander.rs` (4 new probes + `zone_names` helper)

**Corroborating files read (not modified by this PB)**:
`crates/card-types/src/state/zone.rs`, `crates/engine/src/state/mod.rs`,
`crates/engine/src/state/diagnostics.rs`, `crates/engine/src/state/builder.rs`,
`crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/command.rs`,
`crates/engine/src/rules/replacement.rs`, `crates/engine/tests/core/bare_lookup_ratchet.rs`,
`crates/simulator/src/{legal_actions,random_bot,heuristic_bot,local_game}.rs`,
`crates/card-defs/src/defs/darksteel_colossus.rs`,
`docs/audits/decision-point-audit.md`, `memory/m11-session-plan.md`,
`memory/workstream-state.md`, `CLAUDE.md`,
`test-data/generated-scripts/commander/cc32_mulligan_to_six.json`

**Card defs reviewed**: 0 modified by this PB (correct — the plan fenced `crates/card-defs/`).
One *unmodified* def, `darksteel_colossus.rs`, is implicated by Finding 1 below.

---

## Verdict: needs-fix (**0 HIGH**, 1 MEDIUM, 6 LOW)

**There are no HIGH findings, and I want to state that plainly rather than manufacture one.**
Both halves of the fix are correct against the live CR text, re-derived from source rather
than taken from the plan:

- The bottom-write direction is **right, with no reversal**, and I walked the vector
  arithmetic independently (§ Direction proof below). `Zone::top()` is `v.last()`
  (`zone.rs:159-164`), `push_front` is `v.insert(0, id)` (`zone.rs:187-194`),
  `move_object_to_bottom_of_zone` routes to `push_front` (`state/mod.rs:1792`), and
  in-order iteration over `cards_to_bottom` therefore leaves entry 0 **above** later
  entries with the pre-existing library — including its top card — untouched. This matches
  the pre-existing wire doc at `command.rs:245` ("bottom-most last") and the new handler doc
  at `commander.rs:886-890`.
- The shuffle is at the **correct point in the CR 103.5 sequence** (after hand→library, before
  both the `LibraryShuffled` event and the 7 draws), and it **cannot be skipped on any path**
  that emits the event — the only exit between them is the `?` on `expect_zone_mut(..).ok_or(..)`,
  which aborts the whole command.
- The **CR-numbering verdict is correct**. I looked up 103.4 and 103.5 with children
  independently: CR 103.4b is *"In a Vanguard game, each player's starting life total is 20 plus
  or minus the life modifier of their vanguard card"*; the mulligan shuffle **and** the bottoming
  live in one sentence of CR 103.5, with 103.5c supplying the multiplayer free-first adjustment.
  Criterion 5519's cite is stale. The audit rows and CLAUDE.md were edited on a sound basis.
- **Determinism, replay, and the wire closure all hold** (§ Determinism and § Wire below).
- The **SR-25 deviation is not merely defensible, it is strictly better than the plan's snippet**
  (§ Finding on SR-25 below — no finding, it is correct).

The one MEDIUM is a **completeness-of-class** finding, not a defect in what shipped: PB-DP2's
own stated justification (Architecture Invariant 4, "a `LibraryShuffled` event that permutes
nothing is a phantom") applies verbatim to **two more emitters** that were neither fixed nor
seeded, and one of them is reachable in ordinary play. The LOWs are documentation drift and
test-strength observations.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `crates/engine/src/rules/replacement.rs:854`, `:965` | **Two phantom `LibraryShuffled` emitters survive, un-fixed and un-seeded.** `ReplacementModification::ShuffleIntoOwnerLibrary` (CR 701.20) pushes `GameEvent::LibraryShuffled` at both sites and **never calls `Zone::shuffle`** — the exact defect PB-DP2 just fixed in `handle_take_mulligan`, and unlike the mulligan this path is reachable in normal play. **Fix:** file seed **OOS-DP2-7** in `docs/audits/decision-point-audit.md` §8.1. Do not fix in this PB. |
| 2 | LOW | `crates/engine/src/rules/commander.rs:852` | **No CR 103.5 mulligan cap; `required_bottom` can exceed hand size, dead-ending `KeepHand`.** Not filed as a seed alongside the other six. **Fix:** file as **OOS-DP2-8**. |
| 3 | LOW | `crates/engine/src/rules/commander.rs:843` | **`StdRng` is not reproducible across a `rand` major version**, and this PB extends that surface to the opening library order of every game. No gate catches it. **Fix:** add a sentence to **OOS-DP2-4**. |
| 4 | LOW | `crates/card-types/src/state/zone.rs:183-186` | **`push_front`'s doc comment is ambiguous in exactly the direction this PB exists to disambiguate.** **Fix:** reword the parenthetical. |
| 5 | LOW | `crates/engine/tests/rules/commander.rs:1960-1986` | **The determinism probe is the weakest of the four** — it only catches entropy seeding, not a change in the permutation itself. **Fix:** optionally pin the actual resulting order as a golden. |

## Documentation Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 6 | LOW | `docs/audits/decision-point-audit.md:250-251` | **The stale `103.4b` cite the DP-2 row declares stale survives two sections up, in a row about the same defect.** Rows also still class **D** with pre-fix line numbers. **Fix:** correct the CR column to `103.5` and mark both rows shipped. |
| 7 | LOW | `CLAUDE.md:19` vs `:18`/`:54` | **Internal test-count contradiction**: the Tests pin says **3,721**, the Current-State delta and Last-Updated lines say **3,725**. **Fix:** note "PB-DP2's +4 lands at collect", or re-pin. |

---

## Finding Details

### Finding 1 — Two phantom `LibraryShuffled` emitters survive (MEDIUM)

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:848-857` and `:960-966`
**Architecture Invariant**: #4 — "All state changes are Events. Events are the single source
of truth for what happened."
**CR**: 701.20 (shuffle), 103.5 (the analogous mulligan sentence PB-DP2 fixed)

**Issue.** PB-DP2's justification for the (b) half — recorded in the plan §2, the audit §7,
`commander.rs:845` (`"The shuffle above is what makes this event non-phantom (Architecture
Invariant 4)"`) and the workstream handoff — is that emitting `LibraryShuffled` without
performing a permutation is an Invariant-4 violation. That reasoning is correct. It also
applies, unchanged, to two sites the PB did not touch and did not seed.

A whole-tree grep for `.shuffle(` in `crates/engine/src` returns exactly four call sites:
`effects/mod.rs:3049`, `:3148`, `:8702`, and the new `commander.rs:844`. A grep for
`GameEvent::LibraryShuffled` emissions returns **six**: those four's owners plus
`resolution.rs:6024` (which does shuffle, at `:6005-6006`) and:

```rust
// replacement.rs:848-857
Some(ReplacementModification::ShuffleIntoOwnerLibrary) => {
    // CR 701.24: Redirect to library AND shuffle the library.
    acc_events.push(GameEvent::ReplacementEffectApplied { .. });
    acc_events.push(GameEvent::LibraryShuffled { player: owner });   // <-- no permutation
    current_to = crate::state::zone::ZoneType::Library;
}
```

```rust
// replacement.rs:960-966
// If shuffling into library, emit shuffle event.
if matches!(&modification, ReplacementModification::ShuffleIntoOwnerLibrary) {
    events.push(GameEvent::LibraryShuffled { player: owner });        // <-- no permutation
}
```

Neither path reaches a `Zone::shuffle`. The comment at `:849` even asserts *"Redirect to library
AND shuffle the library"* — the "AND shuffle" half does not exist. The card is inserted by the
ordinary redirect move, i.e. `push_back` = **the top of the library**, which is a second,
independent defect of precisely the top/bottom class PB-RS1 and PB-DP2 swept.

**This is the oracle-vs-dispatch semantic gate failing one link down the chain.**
`crates/card-defs/src/defs/darksteel_colossus.rs:39` correctly selects
`ReplacementModification::ShuffleIntoOwnerLibrary`, and its `completeness` marker at `:56-59`
states:

> `known_wrong("the 'reveal it' clause is not modelled; the shuffle-into-owner's-library
> replacement itself is correct (ReplacementModification::ShuffleIntoOwnerLibrary)")`

Oracle (verified via MCP): *"If Darksteel Colossus would be put into a graveyard from anywhere,
reveal Darksteel Colossus and shuffle it into its owner's library instead."* The def's claim
that the replacement "itself is correct" is **false at the match arm**: the Colossus goes to the
top of its owner's library and is redrawn immediately. The def is `known_wrong` (so
`validate_deck` blocks it — severity is bounded), but the marker's stated reason is wrong, and
the file-header comment at `:9` is separately stale ("simplified to `RedirectToZone(Library)`",
contradicted by `:39`).

The existing test is the canonical phantom-passes-the-test shape —
`crates/engine/tests/core/card_def_fixes.rs:1115-1121` asserts only that a `LibraryShuffled`
event appears in the emitted vector, never that the library moved.

**Fix**: Do **not** fix the code in this PB — the scope fence is correct and the reviewer
agrees with it. File a seed in `docs/audits/decision-point-audit.md` §8.1:

> **OOS-DP2-7** — **Two more phantom `LibraryShuffled` emitters, plus a top/bottom inversion.**
> `ReplacementModification::ShuffleIntoOwnerLibrary` (CR 701.20) emits `GameEvent::LibraryShuffled`
> at `rules/replacement.rs:854` and `:965` without ever calling `Zone::shuffle` — the identical
> Architecture-Invariant-4 defect PB-DP2 fixed in `handle_take_mulligan`, still live at two sites,
> and reachable in ordinary play (unlike the mulligan). The comment at `:849` claims "Redirect to
> library AND shuffle the library"; only the redirect happens, and it lands the card on the library
> **top** (`push_back`), so a Darksteel Colossus that dies is redrawn next turn. `darksteel_colossus.rs`
> is the only def using the variant; it is `known_wrong`, but its completeness note asserts the
> replacement "itself is correct", which is false at the match arm, and its header comment at `:9`
> is stale. The only test (`tests/core/card_def_fixes.rs:1115-1121`) asserts the event's presence
> and never the library's contents. Fix needs a seeded `Zone::shuffle` at both sites plus a
> position assertion in the test; also correct the def's completeness note. | correctness,
> live-wrong (gated by `known_wrong`) | filed by PB-DP2 review (`scutemob-150`)

### Finding 2 — No CR 103.5 mulligan cap (LOW)

**Severity**: LOW
**File**: `crates/engine/src/rules/commander.rs:802-877` (whole handler); draw loop at `:852`
**CR 103.5**: *"A player can take mulligans until their opening hand would be zero cards, after
which they may not take further mulligans."*

**Issue**: `handle_take_mulligan` has no cap. `handle_keep_hand` computes
`required_bottom = mulligan_count.saturating_sub(1)` (`:901`) and requires
`cards_to_bottom.len() == required_bottom` (`:902-910`). After 9 mulligans in multiplayer,
`required_bottom` is 8 while the hand holds at most 7, so **`KeepHand` becomes unsatisfiable
from the hand** and the player is stuck (or must supply out-of-hand ids, which OOS-DP2-1
already documents as unvalidated). Long before that, the draw loop silently short-draws on an
exhausted library (`:864-868`). This is pre-existing, not introduced by PB-DP2, and the audit's
§4.4 row at `:254` notes a *different* mulligan-timing gap but not this count gap. PB-DP2 filed
six seeds about this exact function pair, so this one belongs with them.

**Fix**: File **OOS-DP2-8** in `docs/audits/decision-point-audit.md` §8.1 — "CR 103.5's mulligan
cap ('until their opening hand would be zero cards') is unenforced; past 8 mulligans
`required_bottom` exceeds hand size and `KeepHand` becomes unsatisfiable. Needs a cap in
`handle_take_mulligan`; no wire change. | correctness, latent". No code change in this PB.

### Finding 3 — `StdRng` reproducibility across `rand` majors (LOW)

**Severity**: LOW
**File**: `crates/engine/src/rules/commander.rs:843`; `crates/engine/Cargo.toml:15` (`rand = "0.9"`)
**Invariant**: SR-9b (replay determinism), SR-8 (wire versioning)

**Issue**: §4.1 of the plan proves determinism *within a build* and that proof is sound — the
seed is a pure function of the command stream, `GameStateBuilder` starts `timestamp_counter`
at 0 (`builder.rs:333`), and there is no entropy source. What it does not address: `rand`'s
`StdRng` is explicitly documented as *not* algorithm-stable across major versions. A future
`rand 0.10` bump silently re-permutes every seeded shuffle. `PROTOCOL_SCHEMA_FINGERPRINT` is a
digest of the **type closure** and would not move, so a stored `ReplayLog` would replay to a
different game with no gate firing. This was already true at the three `effects/mod.rs` sites,
but their blast radius was a single effect; PB-DP2 extends it to **the opening library order of
every game**, which is the largest possible divergence. Not introduced here, and correctly not
in scope.

**Fix**: Append to **OOS-DP2-4** in §8.1: "…and the eventual helper should pin the PRNG
(`rand_chacha::ChaCha8Rng`, or an in-tree Fisher-Yates) rather than `StdRng`, whose algorithm is
not stable across `rand` major versions — a `rand` bump would silently invalidate every stored
`ReplayLog` with no fingerprint gate firing (SR-8/SR-9b)."

### Finding 4 — `push_front` doc ambiguity (LOW)

**Severity**: LOW
**File**: `crates/card-types/src/state/zone.rs:183-186`

**Issue**: The doc reads *"this places the object at the 'bottom' (the end furthest from the
top, which is the last element)"*. The trailing relative clause parses most naturally as
modifying **"the end furthest from the top"**, which would make the bottom the last element —
the exact inversion PB-DP2 exists to kill. (It is presumably intended to modify "the top".)
Pre-existing, and the PB touched `commander.rs` rather than `zone.rs`, but this is the single
sentence a future implementer will read when asking "which end is `push_front`?".

**Fix**: Reword to: *"…places the object at index 0, which is the **bottom** — ordered zones
store the top at the **last** index (`Zone::top()` is `v.last()`)."*

### Finding 5 — The determinism probe is the weakest of the four (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/rules/commander.rs:1960-1986`

**Issue**: It does genuinely exercise **two independent runs** — `process_command(base.clone(), …)`
twice — so it is not comparing a value to itself, and the "coin-flip" concern does not apply.
But because the seed is derived from state with no entropy and no `HashMap` iteration in the
path, equality is near-tautological in-process. It catches a `from_entropy()` regression (its
stated job, honestly disclosed in the doc comment at `:1955-1959`) and nothing else — notably
not Finding 3's cross-version drift, and not a silent change to the permutation. A golden
assertion (pin the literal resulting order, e.g. `assert_eq!(zone_names(&s1, …)[0], "Card 17")`)
would be strictly stronger and is the usual SR-9b shape.

**Fix**: Optional. Either add a golden-order assertion to this probe, or leave as-is — the doc
comment already prevents a future reviewer from over-reading it.

### Finding 6 — Uncorrected stale cite in audit §4.4 (LOW)

**Severity**: LOW
**File**: `docs/audits/decision-point-audit.md:250-251`

**Issue**: The §5 DP-2 row at `:429` was correctly rewritten (CR column `103.5, 103.5c`, plus an
explicit "**CR correction**" paragraph declaring `103.4b` stale). Two sections up, §4.4's own
row for the same defect still reads:

```
| The **shuffle** inside a mulligan | **103.4b/103.5** | **D** | `rules/commander.rs:808-848` — see **DP-2** |
| `cards_to_bottom` **placement**   | **103.5**        | **D** | `rules/commander.rs:886-890` — see **DP-2** |
```

So the document now contradicts itself on the CR number for the very defect whose row says the
number was wrong. Both rows also remain class **D** (undecidable/wrong) with pre-fix line
numbers — post-fix, `handle_keep_hand`'s loop is at `:911-915`, not `:886-890`.

**Fix**: In `:250` change the CR column from `103.4b/103.5` to `103.5`; append
`— **SHIPPED (PB-DP2)**` to both `:250` and `:251` and refresh the line refs to
`rules/commander.rs:826-846` and `:911-915`.

### Finding 7 — CLAUDE.md test-count contradiction (LOW)

**Severity**: LOW
**File**: `CLAUDE.md:19` vs `:18` and `:54`

**Issue**: The **Tests** bullet at `:19` still reads *"**3,721 passing / 0 failing** (re-pinned
2026-07-26 at PB-DP1 collect…)"*, while the Current-State delta at `:18` and the Last-Updated
line at `:54` both already say **3,725**. Defensible under the convention that the pin is
re-taken at `/collect`, but as written the file states two different numbers.

**Fix**: Either re-pin `:19` to 3,725, or append to it: "(PB-DP2's +4 probes land at
`scutemob-150` collect)".

---

## Direction proof — re-derived independently (no finding)

The task brief asked me not to take the plan's arithmetic on faith. Re-derived from source:

| step | source | fact |
|---|---|---|
| storage | `zone.rs:70-75` | `Zone::Ordered(Vector<ObjectId>)` for `ZoneId::Library` (`is_ordered()`, `:57-62`) |
| which end is the top | `zone.rs:159-164` | `top()` → `v.last()` ⇒ **TOP = last index** |
| corroboration | `zone.rs:175-180` + unit test `:215-219` | `top_n` walks `v.iter().rev()`; `ordered(&[1,2,3]).top_n(3) == [3,2,1]` |
| what `push_front` does | `zone.rs:187-194` | `v.insert(0, id)` ⇒ **index 0 = deepest bottom** |
| what the helper routes to | `state/mod.rs:1792` | `move_object_to_bottom_of_zone` → `to_zone.push_front(new_id)` |
| iteration order | `commander.rs:913-915` | `for obj_id in cards_to_bottom.iter()` — in order, no reversal |

```
Start:             [L0, L1, …, Ln]     (index 0 = bottom, Ln = top)
cards_to_bottom = [A, B]               (A is index 0, "placed first")

  push_front(A) →  [A, L0, …, Ln]
  push_front(B) →  [B, A, L0, …, Ln]
```

⇒ **B (last entry) is bottom-most; A (index 0) sits above B; the pre-existing library, including
`Ln` at the top, is untouched.** This matches CR 103.5's "in any order" (the CR leaves the order
to the player; the engine's obligation is only to honour the supplied order under a stable,
documented convention), the pre-existing wire doc at `command.rs:245` ("bottom-most last"), the
new handler doc at `commander.rs:886-890`, and physical play (sliding A under the deck, then B
under that). **No reversal is needed and none was added — correct.**

`zone_names`'s bottom-first claim is likewise correct: `object_ids()` for `Zone::Ordered` is
`v.iter().copied().collect()` (`zone.rs:130-135`), i.e. raw storage order, index 0 first = bottom.

## Shuffle placement and non-phantom guarantee (no finding)

CR 103.5: *"To take a mulligan, a player **shuffles the cards in their hand back into their
library**, draws a new hand of cards equal to their starting hand size, then puts a number of
those cards … on the bottom of their library in any order."*

Engine order in `handle_take_mulligan`:

1. `:817-829` — move every hand object to `ZoneId::Library(player)` (`?`-propagating, MR-M9-12)
2. `:834-844` — read seed, bump counter, `expect_zone_mut(..).ok_or(..)?`, `library.shuffle(&mut rng)`
3. `:846` — `events.push(GameEvent::LibraryShuffled { player })`
4. `:852-870` — draw 7 off the (now shuffled) top

Cards are in the library before the shuffle; the draw comes off the shuffled library. Correct.

**Cannot be skipped**: the only control-flow exit between the `shuffle` and the event push is
the `?` at `:842`, which aborts the entire command. There is no `if`, no early `continue`, no
`let _ =`. The shuffle is unconditional even on an empty hand, which is right — CR 103.5's
shuffle is part of taking a mulligan, not conditional on hand contents. `LibraryShuffled` is
therefore **genuinely non-phantom on every path in this function that emits it**. (The two
*other* emitters in `replacement.rs` are Finding 1.)

**Take/keep split** (plan §1.4, seeded as OOS-DP2-6): I re-checked the equivalence argument
rather than accepting it. CR does shuffle → draw → bottom-N inside one mulligan; the engine does
`TakeMulligan` = shuffle + draw 7 and `KeepHand` = bottom `mulligan_count - 1`. Walking two and
three consecutive mulligans, final hand size and final library multiset are identical in both
schemes, and the next mulligan's shuffle re-randomises anything the previous step bottomed. The
divergence is unobservable, correctly filed as record-only, and correctly not fixed (moving the
step needs a `cards_to_bottom` field on `Command::TakeMulligan` ⇒ PROTOCOL bump).

## Determinism and replay (SR-9b) — no finding

| question | verdict | evidence |
|---|---|---|
| Does the extra `timestamp_counter += 1` perturb layer/timestamp ordering? | **No.** The counter is monotonic; `GameObject.timestamp` feeds CR 613.7 **relative** ordering only, so a uniform shift flips no tie and creates no dependency. The battlefield is empty pregame anyway. | `state/mod.rs:1664`, `:875-878` |
| Does it perturb LKI? | **No.** `capture_lki_snapshot` (`state/mod.rs:1636`) snapshots the pre-move object; it does not read the counter. | — |
| Does it perturb `HashInto`? | **Value only, not schema.** `timestamp_counter` is already hashed (`hash.rs:7662`); no field added or removed. `loop_detection_hashes` (CR 104.4b) is a turn-loop guard, not consulted pregame. | — |
| Does it break an object-id invariant? | **No.** `next_object_id` is `timestamp_counter += 1; ObjectId(timestamp_counter)`, so the bump skips one id value. INV-OB-04 (`counter >= total_objects`) and INV-OB-05 (`obj.timestamp <= counter`) are both `>=`/`<=` and only get safer. | `tests/core/invariants.rs:768-795` |
| Reproducible across a replay of the same command stream? | **Yes.** `builder.rs:333` starts the counter at 0; every advance is a pure function of the command sequence; `build_initial_state` is deterministic (SR-9b `sorted_zone_entries`); `StdRng::seed_from_u64` + `Zone::shuffle`'s Fisher-Yates (`zone.rs:138-147`) are pure. Caveat: Finding 3 (cross-`rand`-major). | — |
| Can an `Err` after the bump but before the shuffle leave observable half-mutated state? | **No.** `process_command` takes `state: GameState` **by value** and returns `Result<(GameState, Vec<GameEvent>), _>`; on `Err` the mutated state is dropped and never handed back. The caller cannot observe it. | `rules/engine.rs:67-76`, `:245-249` |

## SR-25 deviation — verified correct, no finding

The runner used `state.expect_zone_mut(&lib_zone_id).ok_or(GameStateError::ZoneNotFound(lib_zone_id))?`
(`commander.rs:840-842`) instead of the plan's `state.zones.get_mut(..).ok_or(..)?`. I checked
`expect_zone_mut`'s **actual definition** rather than the description:

```rust
// state/diagnostics.rs:215-225
#[track_caller]
pub(crate) fn expect_zone_mut(&mut self, id: &ZoneId) -> Option<&mut Zone> {
    let found = self.zones.get_mut(id);
    debug_assert!(found.is_some(), "engine invariant: ZoneId {id:?} absent …");
    found
}
```

- **The reasoning holds.** `bare_lookup_ratchet.rs:171-178` counts the six needles
  `.objects/.players/.zones.get[_mut](`. `expect_zone_mut` matches none of them at the call
  site, so the count is unchanged. `src/rules/commander.rs` is pinned at **6**
  (`bare_lookup_ratchet.rs:151`) and the gate is an **exact equality** check — both `count > ceiling`
  (`:227`) and `count < ceiling` (`:239`) panic — so the plan's bare form would indeed have
  failed at 7.
- **The ceiling was NOT quietly raised.** It is still `("src/rules/commander.rs", 6)`, and
  `SWEPT_FILES` carries no PB-DP2 bump comment (every prior raise in that array is annotated).
  `bare_lookup_ratchet.rs` was not modified by either commit.
- **SR-4 classification is satisfied.** The plan's worry was that `expect_*` alone is
  `debug_assert!` + `None`, i.e. release-silent — which would let a release build skip the
  shuffle and re-emit a phantom event. Pairing it with `.ok_or(..)?` yields **both** halves: a
  `debug_assert!` firing in test/debug builds *and* a propagated `GameStateError::ZoneNotFound`
  in release. That is **strictly stronger than either the plan's snippet or bare `expect_*`**,
  and it matches this handler's recorded MR-M9-12 propagate-don't-swallow contract
  (`commander.rs:823-825`) and its `Result` return type. The comment at `:836-839` documents
  exactly this. Engine-bug class, surfaced as an error — correct.

## Test review

| probe | pins what it claims? | fail-before? | notes |
|---|---|---|---|
| `test_dp2_cards_to_bottom_land_on_library_bottom_cr_103_4b` (`:1770-1834`) | **Yes, strongly.** Asserts `lib_after[0] == name_b`, `lib_after[1] == name_a`, **and** `&lib_after[2..] == &lib_before[..]` (whole-library identity), **and** `top()` is unchanged and is neither bottomed card. | **Yes.** Pre-fix `move_object_to_zone` appends, so `lib_after[0] == lib_before[0]` — a library card, necessarily disjoint from the hand-card `name_b` — and `top()` is `name_b`. Both the position and next-draw assertions fail. | Shuffle-independent by construction (all assertions are relative to a captured `lib_before`), so the 3 preceding mulligans cannot make it flaky. Best of the four. |
| `test_dp2_mulligan_actually_permutes_the_library_cr_103_5` (`:1845-1906`) | **Yes.** Reconstructs the full 40-card post-shuffle order (`lib_after ++ drawn.rev()`, correct: draw #1 was the top, so reversing puts #7 nearer the bottom), asserts sorted-multiset equality (no loss/duplication), asserts `full != pre`, and asserts `drawn != pre.rev().take(7)`. | **Yes**, on assertions (2) and (3). Pre-fix `full == pre` and `drawn == [Card 39 … Card 33]`. | **Not a coin flip.** Seed is a pure function of a deterministic build + command stream, so it always passes or always fails; and the target is a 40-element permutation, not the 7-card hand. Assertion (4) (event present) passes pre-fix too and is a standing guard, not the discriminator. |
| `test_dp2_mulligan_returns_a_different_hand_cr_103_5` (`:1913-1952`) | **Yes.** Correctly compares `BTreeSet<String>`, not sequences — pre-fix the hand returns *reversed but set-identical*, so a sequence compare would have passed pre-fix and destroyed fail-before. | **Yes.** | The one the brief flagged as coin-flip risk. It is not: the seed is fixed, so it is deterministic, and even for an arbitrary seed the coincidence probability is `1/C(47,7) ≈ 1.6e-8`. It would remain meaningful at any `timestamp_counter` start value. |
| `test_dp2_mulligan_permutation_is_deterministic_cr_103_5` (`:1960-1986`) | Partially — see **Finding 5**. | **No, by design**, and the doc comment says so explicitly at `:1955-1959`. | **Two genuinely independent runs** (`base.clone()` twice), not a value compared to itself. Weakest probe but honestly labelled. |

**Card identity**: every probe identifies cards by `characteristics.name` (via `zone_names` or an
inline `.map`), never by a pre-move `ObjectId` — correct under CR 400.7, since
`move_object_to_bottom_of_zone` mints a fresh id (`state/mod.rs:1647`) and clones
`characteristics` verbatim (`:1652`). No probe reads `MulliganKept.cards_to_bottom` for position
(it carries stale pre-move ids). Correct.

**`zone_names` bottom-first claim**: verified against `Zone::object_ids()` (`zone.rs:130-135`) —
raw `Vector` order, index 0 first, and index 0 is the bottom because `top()` is `last()`. Correct.

## Regression surface — verified, not relayed

- **Whole-tree grep** for `TakeMulligan|KeepHand|handle_take_mulligan|handle_keep_hand` returns
  19 files; the only **executing test surface** is `crates/engine/tests/rules/commander.rs`.
  Everything else is engine dispatch (`command.rs`, `engine.rs`), the simulator (unreachable,
  below), memory/doc prose, `card-types/src/state/player.rs` (the `mulligan_count` field), and
  the retired script.
- **The four pre-existing mulligan tests** (`:1365`, `:1499`, `:1525`, `:1633`) assert **hand and
  library counts, `mulligan_count`, and event shapes only** — never library position, never hand
  identity. I re-walked each: none passes for a different reason now. `test_free_mulligan_then_london_mulligan:1465`
  and `test_mulligan_sequence_four_players:1575` pick `hand.object_ids()[0]`, which post-shuffle
  resolves to a *different card* but still a live in-hand object, so the 7→6 count assertion is
  unchanged. `test_mulligan_keep_wrong_count_rejected` short-circuits at the count check
  (`:902-910`) before any move.
- **Library exhaustion** under repeated real shuffles: I traced each test's arithmetic.
  `:1369` (20 cards) peaks at lib 13; `:1501` (20) at 13; `:1543` (20/player) at 13;
  `:1637` (25) at 18. None can hit the `None => break` short-draw at `:864-868`. No test
  silently changes shape.
- **Simulator unreachability — verified at source, not relayed.** `legal_actions.rs:186` gates
  `LegalAction::TakeMulligan/KeepHand` on `state.turn().is_first_turn_of_game && state.turn().turn_number == 0`;
  `local_game.rs:576` repeats the same gate for `DecisionKind::Mulligan`; and
  `crates/engine/src/state/builder.rs:59` sets `turn_number: 1`. Nothing in the tree sets it to 0.
  The in-source claim at `local_game.rs:569-574` is therefore **accurate**. No `GameDriver` game,
  `LocalGame` test, fuzzer run, or `invariants.rs` property can observe this change.
- **Golden scripts**: `test-data/generated-scripts/commander/cc32_mulligan_to_six.json` is the
  only mulligan script and is `review_status: "retired"` — it does not execute. Nothing to
  reconcile. (Its four stale `103.4`/`103.4b` cites at `:8`, `:9`, `:143`, `:162` remain; the
  plan made fixing them optional and the handoff explicitly notes them as OOS-DP1-3 class. Fine.)

## Wire closure — PROTOCOL 27 / HASH 63 are genuinely correct

Not merely "the gates pass" — nothing in this change *should* have moved a fingerprint:

- **No new type or variant.** No `Command`, `GameEvent`, `Effect`, `Condition`, or `Zone` variant
  added. `GameEvent::LibraryShuffled` already existed (`rules/events.rs:495`) with an existing
  `HashInto` arm (`hash.rs:4524`) and an existing `reveals_hidden_info()` classification
  (`events.rs:1378`, pinned by `tests/core/six_player.rs:497-506`).
- **No new struct field anywhere.** `timestamp_counter` is a pre-existing `pub(crate)` field
  (`state/mod.rs:230`) already hashed at `hash.rs:7662`. So
  `every_hashed_struct_field_is_hashed_or_allowlisted` and the declaration fingerprint cannot move.
- **No `HashInto` impl body changed.** The stream fingerprint is taken over a hand-built
  `canonical_fixture()` (`tests/core/hash_schema.rs`), not a mulligan-derived state, so the extra
  runtime counter increment — which changes a hash *value* — is invisible to it. Both fingerprints
  are **source-shape** digests, not value digests.
- **No exhaustive-match site to update.** Nothing in `tools/replay-viewer/src/view_model.rs`,
  `tools/tui/src/play/panels/stack_view.rs`, or `state::keyword_registry::handling`.
- The only added dependency edge is a file-level `use rand::SeedableRng;` (`commander.rs:15`);
  `rand = "0.9"` was already a direct dependency (`crates/engine/Cargo.toml:15`). `Rng` is
  correctly **not** imported (`Zone::shuffle` takes `impl Rng` in argument position, so the caller
  needs only a concrete `StdRng`) — importing it would have failed `clippy -D warnings`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 103.5 — "shuffles the cards in their hand back into their library" | **Yes** | **Yes** | `commander.rs:834-844`; `test_dp2_mulligan_actually_permutes_the_library_cr_103_5`, `test_dp2_mulligan_returns_a_different_hand_cr_103_5` |
| 103.5 — "draws a new hand of cards equal to their starting hand size" | **Partial** | Yes (as 7) | Hard-coded `for _ in 0..7` at `:852`; correctly seeded as **OOS-DP2-2** (needs `PlayerState.starting_hand_size` ⇒ HASH bump) |
| 103.5 — "puts … on the bottom of their library in any order" | **Yes** | **Yes** | `commander.rs:911-915`; `test_dp2_cards_to_bottom_land_on_library_bottom_cr_103_4b` asserts literal library indices *and* whole-library identity |
| 103.5 — "can take mulligans until their opening hand would be zero cards" | **No** | No | Finding 2 — no cap; seed recommended (OOS-DP2-8) |
| 103.5 — declaration order (starting player, then turn order; simultaneous resolution) | No | No | Pre-existing; the engine has no mulligan-round sequencer. Out of scope; the audit's §4.4 row `:249` classes take-or-keep as **A** |
| 103.5a — Vanguard starting hand size | No | No | Folded into OOS-DP2-2 |
| 103.5b — "any time you could mulligan" | No | No | No card in the corpus uses it; not in scope |
| 103.5c — first mulligan free in multiplayer | **Yes** (pre-existing) | **Yes** (pre-existing) | `:814` `is_free`, `:901` `saturating_sub(1)`; `test_free_mulligan_then_london_mulligan`, `test_mulligan_three_times_escalating_bottom_count` |
| 103.5d — shared team turns | n/a | n/a | Commander does not use the option |
| **103.4b** | **n/a — correctly identified as NOT a mulligan rule** | n/a | Verified live: Vanguard starting life total. Plan §1.3's verdict is **correct**; the audit rows and CLAUDE.md were edited on sound ground. Residual stale cite: Finding 6 |
| 400.7 — new object identity on zone change | **Yes** (pre-existing helper) | **Yes** | `move_object_to_bottom_of_zone` mints a fresh id (`state/mod.rs:1647`) with the same LKI capture (`:1636`) and SR-23 error-before-capture ordering (`:1625-1634`) as `move_object_to_zone`; every probe identifies by name |
| 701.20 — shuffle into owner's library | **No** | No (event-only assertion) | **Finding 1** — out of PB-DP2's scope, but un-seeded |

## Documentation accuracy spot-check (commit `7aa8914b` vs `f902010f`)

| doc edit | claim | verdict |
|---|---|---|
| audit §5 DP-2 row (`:429`) | "SHIPPED"; both halves fixed; seeded shuffle before event and draws; 4 probes; PROTOCOL 27 / HASH 63; CR correction | **Accurate.** Every clause matches `f902010f`. |
| audit §8 PB-DP2 row (`:571`) | HASH-bump prediction "falsified"; both halves shipped together; S2's `redeal` no longer load-bearing | **Accurate.** No `GameState` field was added; verified above. |
| audit §7 OOS-M11-1 (`:519-527`) | Closed, including the widening the audit recommended | **Accurate.** |
| audit §8.1 seeds OOS-DP2-1..6 (`:603-608`) | 6 new seeds | **All six are real, correctly classified, and genuinely out of scope.** Spot-verified each: **-1** `handle_keep_hand:897-915` checks only the count then moves from any zone — confirmed, and `move_object_to_bottom_of_zone`'s only membership check is `from_zone.contains(&object_id)`, which happily passes for another player's hand; **-2** `for _ in 0..7` at `:852` — confirmed; **-3** `timestamp_counter` is hashed and replayable — confirmed, and pre-existing; **-4** four `zone.shuffle(` sites — confirmed by whole-tree grep (see Finding 1 for the *other* direction of this count); **-5** `random_bot.rs:239-246` sends `cards_to_bottom: Vec::new()` unconditionally, `heuristic_bot.rs:71-72` scores the same actions — confirmed, and latency-gated exactly as described; **-6** the take/keep split — confirmed equivalent by my own walk-through. |
| audit `last_updated` (`:3`) | bumped | Present (2026-07-26). |
| `m11-session-plan.md` R2 (`:948-954`) | OOS-M11-1 closed; "new `Command` → wire change" premise falsified | **Accurate.** |
| `workstream-state.md:56-118` | full handoff | **Accurate and unusually thorough.** The "3 of 4 fail on pristine code" claim checks out (probes 1/2/3 fail-before; probe 4 does not, as disclosed). The SR-25 note, the simulator-unreachability note, and the retired-script note are all correct. |
| `CLAUDE.md:18`, `:54` | PB-DP2 SHIPPED, OOS-M11-1 CLOSED, 3,725, next PB-DP3 | Accurate except the internal contradiction with `:19` — **Finding 7**. |
| test count 3,721 → 3,725 | +4 | Consistent with the 4 probes added; **not independently executed by this review** (read-only pass). |

## Fix-cycle dispositions (2026-07-26)

| # | Severity | Disposition |
|---|----------|-------------|
| 1 | MEDIUM | **Filed as seed** — `OOS-DP2-7` added to `docs/audits/decision-point-audit.md` §8.1 (two phantom `ShuffleIntoOwnerLibrary` emitters at `replacement.rs:854`/`:965`, plus the stale `darksteel_colossus.rs` completeness note). Code **not** patched, per directive. |
| 2 | LOW | **Filed as seed** — `OOS-DP2-8` added to §8.1 (no CR 103.5 mulligan cap; `required_bottom` can exceed hand size). No code change. |
| 3 | LOW | **Applied as an addendum** — appended to the existing `OOS-DP2-4` row in §8.1 rather than filing a new row, noting `StdRng`'s lack of cross-`rand`-major stability and that PB-DP2 widens the blast radius to the opening library order of every game. |
| 4 | LOW | **Applied** — reworded `Zone::push_front`'s doc comment (`crates/card-types/src/state/zone.rs:181-186`) to state plainly that index 0 is the bottom, since `top()` is `v.last()`. Doc-only, no behavior change. |
| 5 | LOW | **Declined, with reason recorded** — no golden-order assertion added; a hard-coded permutation would be brittle against exactly the `rand`-major instability Finding 3 describes. Instead added one sentence to `test_dp2_mulligan_permutation_is_deterministic_cr_103_5`'s doc comment (`crates/engine/tests/rules/commander.rs`) noting it pins seeding-from-state, not the permutation, and cross-referencing OOS-DP2-4. |
| 6 | LOW | **Applied** — `docs/audits/decision-point-audit.md` §4.4 rows (`:250`/`:251`) corrected: CR column now `103.5` (was `103.4b/103.5`), both rows annotated `SHIPPED (PB-DP2)`, line refs refreshed to `rules/commander.rs:826-846` and `:911-915`. |
| 7 | LOW | **Applied** — `CLAUDE.md`'s Tests pin (line ~19) amended to state the pinned baseline (3,721, re-pinned at PB-DP1 collect, merge `f7651bb5`) **and** that PB-DP2's +4 probes are on branch `scutemob-150` and land at collect (3,725). Provenance of the pin was not overwritten. |
