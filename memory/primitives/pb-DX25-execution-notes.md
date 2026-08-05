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

## Stage 4 — the counter arm, atomically (§3.3 + §3.5) + T2-T5 + G2 (DONE)

`crates/engine/src/effects/mod.rs`'s `Effect::CounterSpell` arm rewritten in ONE
edit, both halves together (plan §2.2's binding instruction — shipping the lookup
fix alone would create a permanent `ZoneId::Stack` leak, strictly worse than
HEAD):

* **§3.3 lookup**: `position()`'s second clause now calls
  `state::stack_registry::card_in_stack_zone(&so.kind) == Some(id)` (guarded by
  `!so.is_copy`, CR 707.10) instead of matching the literal `StackObjectKind::Spell`
  pattern. `so.id == id` (the Ward clause) is unchanged.
* **§3.5 zone-move**: the whole per-kind `match stack_obj.kind { Spell {..} => ..,
  ActivatedAbility|TriggeredAbility {..} => .., _ => {} }` is replaced by
  `card_owned = card_in_stack_zone(&stack_obj.kind)`, `card_to_move = if
  stack_obj.is_copy { None } else { card_owned }`, then an `if let Some(source_object)
  = card_to_move { <verbatim pre-PB-DX25 Spell-arm body> } else { <copy-aware named-
  event branch> }`.
* **§3.7 CR-citation corrections** applied to the two lines this edit already
  touches: `CR 701.5` → `CR 701.6a` at the arm's opening comment and at the
  EF-W-MISS-1 note (the non-existent "CR 701.5g" citation deleted, since the
  effect's own wording plus CR 701.6a is the real warrant per the plan).

`cargo check -p mtg-engine` clean immediately after the edit.

### T1 and File C flip green

Both re-run against the NEW source with zero test-file changes (they were already
written to assert the fixed shape at Stage 2/3):
* `test_dx25_counterspell_counters_a_mutate_spell` — **PASS**.
* `test_dx25_counter_on_mutate_produces_no_stack_consistency_violations` — **PASS**
  (both the violation-count half AND the behavioural half now).

### T2, T3, T4, T5 — written and green on first run

All four new tests in
`crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs` passed on
their first execution (no debugging cycle needed, unlike T1's fixture in Stage 2):

* **T2** `test_dx25_ward_path_counter_on_a_mutate_spell_moves_the_card` — route
  used: **hand-built `EffectContext`**, not a hand-built trigger `StackObject`.
  `ward.rs:136-260` was read first per the plan's instruction; a real Ward
  creature cannot reach a mutate spell's target today (`OOS-DX25-1`, roster M3 =
  0), so the fallback route was used, and it is a faithful shortcut (not a
  different mechanism) because `EffectContext.targets` is exactly what
  `EffectTarget::DeclaredTarget` reads at resolution — same value a real Ward
  trigger's context would carry.
* **T3** `test_dx25_countering_a_copy_moves_no_card` — uses the real, `pub`
  `rules::copy::copy_spell_on_stack` (not a hand-rolled copy). Both halves
  present: the copy's own stack-entry id is used for both `stack_object_id` and
  `source_object_id` on `SpellCountered` (§4.3), and the non-vacuity sibling
  (countering the ORIGINAL afterward, same fixture) confirms the effect path
  really can move a card.
* **T4** `test_dx25_countered_spell_destination_is_preserved` — three sub-cases
  (exile_instead, cast_with_flashback, neither with owner != controller) plus a
  fourth STRUCTURAL sub-case (no fixture, just a doc-comment citing
  `rules/command.rs:792`'s single `Option<AltCostKind>` and the mutually
  exclusive alt-cost dispatch) pinning that `MutatingCreatureSpell` and
  `cast_with_flashback` can never co-occur on one `StackObject`.
* **T5** `test_dx25_uncounterable_mutate_spell_still_sets_the_controller` — newly
  reachable by this batch (before PB-DX25, `position()` never found a
  `MutatingCreatureSpell`, so this line of the arm never ran for one).

**A cleanup during writing, not a defect**: T4's first draft called an unused
`build()` closure once and then immediately discarded its result before
overwriting `state` with a second, differently-configured builder call in
sub-case 1 — dead code left over from an earlier draft. Removed before running
any test (caught by inspection, not by a failing assertion).

### G2 — written and green on first run

`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs` gains
`extract_match_arm_body` (finds a match arm's pattern, then its `=> {`, then
brace-balances the body — a sibling of `extract_function_body` for when the
gated region is one arm inside a giant `match`, not a whole function) and
`EFFECTS_MOD_PATH`, both re-added (they were removed at the end of Stage 3
specifically to avoid dead-code warnings before this arm existed to gate).
`g2_counter_spell_arm_does_not_reclassify_by_kind` and `g2_scan_is_not_vacuous`
both pass on first run.

### Full revert matrix — T1-T5 and G2, ALL EXECUTED against the real source

Every row: reverted, rebuilt (confirmed via `Compiling mtg-engine` in captured
output), watched fail, restored, `diff` against a pre-edit backup copy confirmed
byte-identical, then the full File A + File B + File C suite re-run green before
moving to the next row.

| id | revert | observed failure (verbatim) |
|---|---|---|
| **T1** | restored the `Spell { source_object } == id`-only clause in `position()` | `panicked at .../pb_dx25_counterspell_stack_shapes.rs:367:13: CR 701.6a: countered Gemrazer should be in p1's graveyard under a fresh ObjectId` |
| **T2** | restored the `_ => {}` shape: replaced the whole `if let Some(source_object) = card_to_move {..} else {..}` with a no-op (nothing happens after removal) | `panicked at .../pb_dx25_counterspell_stack_shapes.rs:484:13: CR 701.6a / CR 702.140a: a MutatingCreatureSpell countered via the Ward-shaped so.id == id lookup must move its card to the graveyard, not strand it in ZoneId::Stack` |
| **T3** | deleted the `is_copy` guard: `let card_to_move = card_owned;` (dropped the `if stack_obj.is_copy { None } else { .. }`) | `panicked at .../pb_dx25_counterspell_stack_shapes.rs:645:5: assertion \`left == right\` failed: CR 707.10: the ORIGINAL's card must still be in ZoneId::Stack -- the copy has no card of its own to move / left: None / right: Some(Stack)` -- i.e. the original's card WAS moved out (its `ZoneId` lookup returned `None`), exactly the predicted defect |
| **T4** | hard-coded `let destination = ZoneId::Graveyard(owner);`, dropping the `exile_instead`/`cast_with_flashback`/`cast_with_jump_start` branch | `panicked at .../pb_dx25_counterspell_stack_shapes.rs:812:9: CR 701.6a: exile_instead should send the card to Exile` |
| **T5** | moved the `ctx.countered_spell_controller = ..` assignment to AFTER the `cant_be_countered` check | `panicked at .../pb_dx25_counterspell_stack_shapes.rs:955:5: assertion \`left == right\` failed: EF-W-MISS-1 / An Offer: countered_spell_controller must be set to the (uncounterable) target's controller even though nothing was countered / left: None / right: Some(PlayerId(1))` |
| **G2** | restored the ENTIRE original (pre-PB-DX25) `Effect::CounterSpell` arm verbatim (via `git show HEAD` at the Stage-3 commit) | `panicked at .../pb_dx25_stack_registry_roster.rs:188:5: the zone-move is driven off state::stack_registry, never off a per-kind match -- do not add an arm, extend the registry. Expected >= 2 calls to card_in_stack_zone (lookup + move) in the Effect::CounterSpell arm, got 0` |

**A mechanical note on running these reverts under `-D warnings`**: two of the
revert edits (T2, T4) left an unused `exile_instead`/`controller` binding behind,
which this workspace's `-D warnings` turns into a **compile error**, not a
warning — so the FIRST attempt at each of those two reverts failed to build at
all (not "passed vacuously on a stale binary", the opposite failure mode from
the PB-DX32/PB-DX24 lesson, but worth recording: it is possible for a
revert-proof's own mechanics to be blocked by the SAME gate the batch's
Post-Implementation Verification requires). Fixed by renaming the now-unread
bindings to `_exile_instead` / prefixing with `_`/adding an explicit `let _ = ..;`
line, which does not change the revert's semantics, only silences the warning so
the REAL assertion failure could be observed. Recorded so a future reader
re-executing this matrix isn't surprised by a compile error that looks like it
might be hiding something -- it isn't; it is exactly what `-D warnings` is
supposed to do.

### Full-suite verification (Stage 4 close)

* `cargo check -p mtg-engine` clean.
* `cargo build --workspace` clean.
* `cargo clippy --workspace --all-targets -- -D warnings` clean.
* `cargo fmt` (one run — reformatted line-wraps in the two new test files and the
  roster gate file; `cargo fmt -- --check` clean after).
* `tools/check-defs-fmt.sh` — 1803 defs, clean.
* `cargo test --workspace --no-fail-fast` — **4,448 / 0 / 5** (+13 over the
  Stage-0/pre-edit 4,435 baseline: T1-T6 = 6, G1 = 3, G2 = 2, File C = 2; residual
  list empty).
* `cargo test -p mtg-engine --test core protocol_schema` / `--test core
  hash_schema` — all sub-tests pass; **PROTOCOL 35 / HASH 73 unmoved**,
  gate-executed (not predicted).
* `cargo test -p mtg-engine --test core keyword_registry` — **9 / 0**, unmoved
  and green (the plan's predicted-unmoved SR-5 gate; `effects/mod.rs` was already
  a declared `Handled` site for `KeywordAbility::Ward`, and this batch adds no
  new keyword read).
* `cargo test -p mtg-simulator` — **206 / 0** (+2 over the pre-batch count, the
  new File C tests).
* `cargo test -p play-server` — **80 / 0**, unmoved by this batch (pre-existing
  count on this branch, untouched by any file this stage edited).
* `git diff main..HEAD --numstat -- crates/card-defs/ crates/card-types/
  crates/view-model/ tools/` — **EMPTY** (SR-6 / exhaustive-match-sweep scope,
  confirmed at Stage 4 too, not just Stage 3).

**No pre-existing test reddened at any point in Stage 4** — every failure
observed in this stage was a revert I introduced and then restored; the
"if a pre-existing test reddens, it was asserting the defect" instruction never
applied here.

---

## Summary for the handoff to Stage 1 / Stage 5 / Stage 6 / Stage 7's runner

* Stages 2, 3, 4 are DONE and committed (three commits, `W6-prim:` prefix,
  `scutemob-203`).
* **NOT done here, still owed**: Stage 1 (SR-36 corpus roster, G3, the "6 x 24"
  overcount correction in the plan/queue rows), Stage 5 (`resolution.rs::
  counter_stack_object` fix + T7 + the `invariants.rs` t8 doc cross-reference),
  Stage 6 (G3's revert execution + acceptance criterion 6232 mapping), Stage 7
  (close-out: `docs/audits/decision-point-audit.md`'s `OOS-SIM3-5` row, the §11
  seed filings, CLAUDE.md delta).
* Test count at the end of this runner's work: **4,448 / 0 / 5** on this branch.
  The next runner's own Stage-0-style re-measurement should start FROM this
  number, not from the original 4,435 pre-batch baseline.
* PROTOCOL 35 / HASH 73 confirmed unmoved through Stage 4; the plan's wire
  prediction (§7) holds so far. Stage 5's `counter_stack_object` refactor is
  ALSO predicted wire-neutral (no type change) but must be gate-executed again,
  not assumed.
