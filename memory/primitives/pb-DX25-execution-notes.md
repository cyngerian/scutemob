# PB-DX25 — execution notes (Stages 2, 3, 4 — this runner's scope only)

Worker: `scutemob-203`, branch
`feat/pb-dx25-effectcounterspells-three-stack-object-shapes-counte`. This runner's
assignment was **Stages 2, 3, 4 only** (plan §10) — Stage 1 (corpus roster / G3),
Stage 5 (`counter_stack_object` / T7), Stage 6 (gate-execution close-out for G3),
and Stage 7 (full close-out) are a second runner's scope and are NOT covered here.

Baseline: `memory/primitives/pb-DX25-stage0.md` already pins **4,435 passed / 0
failed / 5 ignored** on this branch before any edit. Not re-measured here per the
brief's instruction.

---

## Stage 2 — fail-before (DONE)

Two new files written against **unmodified HEAD** and executed:

* `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs` — T1 only
  at this point (`test_dx25_counterspell_counters_a_mutate_spell`), real corpus
  cards `gemrazer` x `counterspell`, mirroring
  `crates/engine/tests/mechanics_m_z/mutate.rs`'s cast-for-mutate command shape.
* `crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs` — the File C
  simulator probe, two tests.

### T1's fail-before result (verbatim)

```
thread 'pb_dx25_counterspell_stack_shapes::test_dx25_counterspell_counters_a_mutate_spell' panicked at crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs:284:13:
CR 701.6a: countered Gemrazer should be in p1's graveyard under a fresh ObjectId
```

Full captured output:
`/tmp/claude-1000/-home-skydude-projects-scutemob--worktrees-scutemob-203/de60b249-271f-4f80-9313-2e03f4ec0af7/scratchpad/pb-dx25-t1-head-failure.txt`
(scratchpad, not committed — the failure text above is the load-bearing record).

This matches the plan's prediction exactly: at HEAD, `position()`'s second clause
matches only `StackObjectKind::Spell`, so Counterspell's target-search finds
nothing, the countered card is never moved, and Gemrazer goes on to resolve and
merge with the Wolf on the next stack drain — "nothing happens and nothing is
reported" (plan §2.1).

**A real bug in my OWN first draft was found and fixed here, not in the primitive
under test.** My `drain_stack` helper initially took a fixed player-pass order
(`&[p2, p1]`) and reused it every iteration of its drain loop. After the FIRST
resolution, CR 117.3b resets priority to the ACTIVE player (p1 throughout this
fixture, since it never changes turn) — not to "the next name in whatever list the
caller wrote down". The second iteration then tried `PassPriority{player: p2}`
against a state whose actual holder was `p1`, and `priority::pass_priority`
correctly rejected it: `NotPriorityHolder { expected: Some(PlayerId(1)), actual:
PlayerId(2) }`. Fixed by having `drain_stack` always read
`state.turn().priority_holder` fresh and pass as WHOEVER currently holds it,
rather than trusting a caller-supplied rotation. This is a fixture defect, not an
engine finding — recorded because it consumed real debugging time and the fix
(read current state, don't assume a rotation) is the correct general shape for any
future multi-stack-item drain fixture in this file family.

### File C's fail-before result — the predicted asymmetry, confirmed by execution

Plan §12 risk 1 predicted: the "zero `stack_consistency` violations" assertion
would be GREEN at HEAD (shape (c) produces no divergence — the card and its entry
both survive, consistently), while the BEHAVIOURAL assertion (no merge happened)
would be the one that reddens. **Confirmed exactly**:

```
running 2 tests
test test_dx25_an_unclaimed_stack_zone_card_is_a_real_violation ... ok

thread 'test_dx25_counter_on_mutate_produces_no_stack_consistency_violations' panicked at crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs:226:5:
CR 701.6a / CR 729.2: Gemrazer was countered and must NOT have merged with the Wolf -- merged_components should be empty, got [MergedComponent { card_id: Some(CardId("gemrazer")), ... }, MergedComponent { card_id: Some(CardId("mock-wolf")), ... }]

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Full captured output:
`/tmp/claude-1000/-home-skydude-projects-scutemob--worktrees-scutemob-203/de60b249-271f-4f80-9313-2e03f4ec0af7/scratchpad/pb-dx25-filec-head-failure.txt`.

The "zero violations across the whole game AND at the terminal state" assertions
(the FIRST two `assert!`s in
`test_dx25_counter_on_mutate_produces_no_stack_consistency_violations`) both
**passed** at HEAD — non-discriminating for shape (c), exactly as predicted. The
sibling non-vacuity test
(`test_dx25_an_unclaimed_stack_zone_card_is_a_real_violation`) passed, proving the
`check_all`/`GameStateBuilder` wiring used above is capable of detecting a real
`stack_consistency` violation when one exists — so the "zero violations" reading
above is a genuine "clean", not a broken check silently returning nothing.

**Recorded honestly per the plan's own instruction**: T1 fails at HEAD because
nothing happens (the counter is a no-op); File C's violation-count half is GREEN
at HEAD because shape (c) produces no stack/card-count divergence for
`stack_consistency` to see; File C's behavioural half (merge did not happen) is
what actually reddens, and it reddens for the SAME underlying reason T1 does.

Commit for this stage: test-only, no `src/` changes. `git diff --stat -- crates/engine/src/ crates/card-defs/ crates/card-types/` is empty at this point (confirmed before committing).

---

## Stage 3 — the registry (`state::stack_registry`) + T6 + G1 (DONE)

New file `crates/engine/src/state/stack_registry.rs`, `pub fn card_in_stack_zone(kind:
&StackObjectKind) -> Option<ObjectId>`, exhaustive over all **27** `StackObjectKind`
variants with **no wildcard arm** (2 `=> Some(...)`, 25 `=> None`), `pub mod
stack_registry;` added in `crates/engine/src/state/mod.rs` beside `keyword_registry`.
Doc comment states the "this is NOT is-it-a-spell" warning (pointing at `casting.rs`'s
`is_spell` check for `TargetSpellWithSingleTarget`) and the deliberate-duplication
note pointing at `mtg_simulator::invariants::stack_card_of`, per plan §3.1/§3.2.

**Variant count independently confirmed at 27**, matching the plan's own count:
`awk '/^pub enum StackObjectKind/{flag=1} flag' crates/card-types/src/state/stack.rs
| grep -E "^    [A-Za-z]+ \{"` lists 27 names. **My own first draft of T6's fixture
was missing `TransformTrigger`** (26 entries, not 27) until this re-derivation caught
it — the registry itself was correct (built by mirroring the simulator's own
`stack_card_of`, which already includes it), only my hand-typed T6 roster had drifted.
Fixed before running T6.

**T6** (`test_dx25_stack_registry_classifies_every_kind`,
`crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs`): one instance of
every variant, asserts `card_in_stack_zone` returns `Some` for exactly `["Spell",
"MutatingCreatureSpell"]`, non-vacuity via `variants.len() == 27`. PASSES (T1 is
unaffected — still red, as expected, since the counter arm itself hasn't been
rewritten yet).

**G1** (`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`, new file,
`mod pb_dx25_stack_registry_roster;` added to `crates/engine/tests/core/main.rs`):
comment-stripping (`strip_line_comments`/`strip_block_comments`/`strip_comments`) +
`extract_function_body` mirror `pb_dx24_trigger_zone_roster.rs`'s idiom exactly.
`g1_stack_registry_has_no_wildcard_arm` asserts no `_ =>`/`_ |` in
`card_in_stack_zone`'s body; `g1_scan_is_not_vacuous` pins the arm count at 27;
`g1_line_comment_stripping_does_not_hide_the_wildcard_it_is_meant_to_find` is an
inverse sanity check on the stripping helpers themselves (a genuine wildcard next to a
line comment must still be found after stripping). **This file's scope is G1 only —
G2 is deliberately deferred to Stage 4** (a gate over the `Effect::CounterSpell` arm
calling `card_in_stack_zone` cannot be meaningfully green OR red before that arm is
rewritten to call it at all), noted in the file's own module doc and its "G2" section
stub.

### G1 revert matrix — both variants EXECUTED against the real source file

| revert | how | observed failure (verbatim) | rebuild confirmed |
|---|---|---|---|
| bare wildcard | replaced the tail arm `K::DelayedActionTrigger { .. } => None,` with `_ => None,` | `a new StackObjectKind must be classified here, not defaulted -- Effect::CounterSpell and counter_stack_object both drive their zone-move off this answer. Found a wildcard arm in card_in_stack_zone's body.` (`g1_stack_registry_has_no_wildcard_arm`, panicked at `pb_dx25_stack_registry_roster.rs:107`) | yes — `Compiling mtg-engine` present in captured output before the failing test line |
| `/* */`-wrapped variant | kept `K::ClassLevelAbility { .. } => None,` as a real arm, replaced the tail arm with `/* a real block comment sitting right before the wildcard */ _ => None,` — i.e. a REAL (uncommented, compiled) wildcard sitting immediately after a real block comment, proving `strip_block_comments` doesn't over-strip and accidentally swallow the live wildcard code along with the comment | identical failure text/location to the row above | yes — `Compiling mtg-engine` present |

Both reverts restored immediately after observing the failure; `cargo test -p
mtg-engine --test core pb_dx25_stack_registry_roster::` and `--test primitives
pb_dx25_counterspell_stack_shapes::test_dx25_stack_registry_classifies_every_kind`
re-run green after each restore (confirmed identical to the pre-revert state — no
`git diff` tracked yet at this point since the file is new/untracked, so the restore
was confirmed by re-running the full G1 + T6 test set, not by `git diff`).

**Note on the `/* */` revert's semantics** (recorded because the brief's own wording
for this row could be read two ways): the interpretation used here is "does
`strip_block_comments` correctly leave a REAL wildcard intact when a real block
comment sits immediately before it" (a false-negative risk on the STRIPPING
function itself), not "is a commented-OUT wildcard correctly ignored" (which would
be a *different*, weaker experiment — a `/* _ => None, */`-only replacement with no
real arm for the removed variant would fail to COMPILE at all, catching the same
class by a stronger, but different, mechanism than G1's own assertion). Both
readings protect the same invariant; this one exercises G1's own scanner rather than
routing the failure through `rustc`'s exhaustiveness check.

**Stage gates, all EXECUTED**: `cargo check -p mtg-engine` clean; `cargo build
--workspace` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean;
`cargo fmt` (one run, reformatted the two new test files -- multi-line struct
literals and a `vec![...]` -- then `cargo fmt -- --check` clean); `git diff --stat --
crates/card-defs/ crates/card-types/` **EMPTY** (SR-6, card-defs and card-types
untouched).

---

## Stage 4 — the counter arm, atomically (§3.3 + §3.5) + T2-T5 + G2

(written below as the stage is executed)

---

## Stage 4 — the counter arm, atomically (§3.3 + §3.5) + T2-T5 + G2

(written below as the stage is executed)
