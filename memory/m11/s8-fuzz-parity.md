# M11-local S8 — fuzzer parity evidence

<!-- last_updated: 2026-08-01 -->

Plan item 8's last gate: *"a 500-game `--profile fuzz` run to confirm the driver
refactor did not perturb the fuzzer"* (plan §8 R11).

## What was run

Both sides, same command, same seed, `--profile fuzz` (release speed with
`debug-assertions` and `overflow-checks` on, per SR-32):

```
cargo run --profile fuzz --bin mtg-fuzzer -- --games 500 --seed 12345 --max-turns 40 --verbose
```

* **baseline**: a pristine `git worktree` at the merge base `f20823b1`
* **branch**: `feat/m11-local-s8-…` at the S8 head

`--verbose` prints every game's `(seed, turns, commands, violations, outcome)` at the
end, in `results` order, so the comparison is a line-for-line diff and rayon's thread
count does not affect it.

## Result

| | baseline | branch |
|---|---|---|
| games completed | 500 | 500 |
| games differing in **turns / commands / outcome** | — | **0** |
| total invariant violations | **501** | **0** |
| games reporting a violation | 16 | 0 |
| distinct checks that fired | `stack_consistency` (104 lines) | none |

**The fuzzer is unperturbed in what it plays.** Every one of the 500 games reaches the
same turn count, the same command count and the same outcome on both sides. That is the
property R11 asked for, and it is what makes every recorded crash seed still comparable.

**The only difference is what the fuzzer *reports*, and it is a correction.** All 501
baseline violations came from `invariants::check_stack_consistency`, which compared two
different id spaces (a cast spell's card gets `ObjectId` *n* in the Stack zone and its
`StackObject` gets *n+1*), so it fired on ordinary spells and abilities in a game with no
defect. S8 rewrote it against `StackObjectKind::Spell { source_object }` — the id the two
sides actually share. Zero violations afterwards, across 500 games.

That this suite had been carrying ~1 violation per game, all spurious, is itself the
answer to why `OOS-DP3-9`'s "long games trip `stack_consistency`" never resolved into a
real defect.

## The gate could not be run at the plan's default `--max-turns 200`

`--games 500` with the default 200-turn cap **aborts with a stack overflow at the merge
base**, before printing a single game result:

```
thread '<unknown>' (…) has overflowed its stack
fatal runtime error: stack overflow, aborting
```

Reproduced on the pristine baseline worktree, single-threaded and multi-threaded, and
with `RUST_MIN_STACK=128 MiB` (which rayon's pool does not honour). This is the
pre-existing **`OOS-DP3-9`** ("`mtg-fuzzer` aborts on a stack overflow"; CLAUDE.md
records it as reproducing on pristine `main`), not anything S8 introduced — S8 makes no
engine change at all.

`--max-turns 40` is the largest round cap tried that completes on both sides, and it is
what the numbers above are from. **Stated plainly: this gate therefore covers 500 games
of up to 40 turns, not of up to 200.** The deep-resolution recursion that overflows is
reachable only in longer games, so nothing here says anything about those; closing
OOS-DP3-9 is the prerequisite for running the gate as the plan wrote it.
