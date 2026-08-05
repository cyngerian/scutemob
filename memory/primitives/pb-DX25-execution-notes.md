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

## Stage 3 — the registry (`state::stack_registry`) + T6 + G1

(written below as the stage is executed)

---

## Stage 4 — the counter arm, atomically (§3.3 + §3.5) + T2-T5 + G2

(written below as the stage is executed)
