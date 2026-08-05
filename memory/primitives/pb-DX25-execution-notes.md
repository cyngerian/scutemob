# PB-DX25 — execution notes

**Appended to by TWO runners; never rewritten.** The first runner covered
Stages 2, 3, 4 (see that section's original header note below). This second
runner covered Stages 1, 5, 6 (added below the first runner's material, in
Stage order rather than chronological order — Stage 1 was written first by
this runner but appears after the first runner's Stage-2/3/4 material because
Stage numbering, not authorship order, is the organizing axis of this file).
Stage 7 (close-out) is NOT covered by either runner — see the final summary.

## Header note from the first runner (Stages 2, 3, 4 — that runner's scope only)

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

## Stage 1 — the SR-36 corpus roster + G3 (this runner, DONE)

**G3** written into the existing `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`
(the file G1/G2 already lived in — this runner's assignment per plan §10 was
explicitly to add G3 there, not a new file). Enumerated from `all_cards()` via
structural walks over `AbilityDefinition`/`Effect` — **never grepped** — with
helper functions `mutate_defs`, `has_spell_level_target_requirement`,
`effect_contains_counter_spell` (recurses into `Sequence`, `Conditional` (both
branches), `ForEach`, and `Effect::Choose` — the SR-33 stub "modal", distinct
from `AbilityDefinition::Spell.modes`, which is a sibling field walked
separately), `ability_contains_counter_spell`, `counterspell_defs`, and
`counter_target_requirement` (locates the `TargetRequirement` governing the
counter effect: `targets[0]` for a flat `Spell` whose effect tree contains the
counter anywhere — not just at the top level, see below — or
`mode_targets[i][0]` for the modal mode `i` whose effect tree contains it).

### Measured values (ALL recorded, including zeros, per the brief's instruction)

| # | population | plan's grep estimate | **measured** |
|---|---|---|---|
| M1 | Mutate-keyword defs (either face) | 8 | **8** — matches exactly: `{Gemrazer, Sea-Dasher Octopus, Brokkos Apex of Forever, Vulpikeet, Necropanther, Glowstone Recluse, Mindleecher, Nethroi Apex of Death}` |
| M2 | M1 ∩ `is_complete()` | 6 | **6** — `{Gemrazer, Sea-Dasher Octopus, Brokkos Apex of Forever, Vulpikeet, Necropanther, Glowstone Recluse}` (Mindleecher and Nethroi Apex of Death excluded, both `partial`) |
| M3 | M2 with any spell-level target requirement | expected 0 | **0** — confirmed, not a finding. No `Complete` Mutate def declares a non-empty `targets` or a non-empty `mode_targets` slice. Shape (a) stays corpus-unreachable via Ward today, exactly as plan §2.2 argues. |
| C1 | defs with `Effect::CounterSpell` anywhere (recursive walk, front face) | 24 | **23** — **the plan's own grep estimate was itself wrong**, and the reason is an SR-36 textbook case: `transcendent_dragon.rs`'s `completeness: Completeness::partial(...)` note contains the literal substring `"Effect::CounterSpell cannot redirect..."` inside a TODO/blocker *comment string* — a grep for `Effect::CounterSpell` matches it, but the card has **no `Effect::CounterSpell` node anywhere in its actual `abilities` Vec** (its `abilities` list is `[Flash, Flying]` only, the counter clause is entirely unimplemented). The structural walk correctly excludes it. Real C1 (23): `{Abjure, Access Denied, An Offer You Can't Refuse, Arcane Denial, Archmage's Charm, Counterspell, Cryptic Command, Dispel, Dovin's Veto, Fierce Guardianship, Flare of Denial, Force of Negation, Force of Will, Mana Drain, Memory Lapse, Mental Misstep, Negate, Pyroblast, Red Elemental Blast, Rewind, Saw It Coming, Stubborn Denial, Swan Song}` |
| C2 | C1 ∩ `is_complete()` | 18 | **18** — matches exactly |
| C3 | C2 with the UNRESTRICTED `TargetRequirement::TargetSpell` (syntactic) | 8 | **8** — `{Abjure, Archmage's Charm, Counterspell, Cryptic Command, Force of Will, Saw It Coming, Access Denied, Rewind}`. **First pass under-counted this at 6** (missing Access Denied and Rewind) because my first `counter_target_requirement` implementation only matched a `Spell` ability whose TOP-LEVEL `effect` field was `Effect::CounterSpell` directly — `access_denied.rs` and `rewind.rs` both wrap it one level inside `Effect::Sequence([CounterSpell{..}, ..other effects..])`. Fixed by reusing the SAME recursive `effect_contains_counter_spell` predicate used for C1 (rather than a shallow `matches!`), while still reading `targets[0]` for the requirement — every corpus def observed (flat and modal alike) uses `EffectTarget::DeclaredTarget { index: 0 }` for the counter's own target regardless of nesting depth, so slot 0 is always the right requirement to read. |
| **P** | live-wrong pairs = \|M2\| × \|C3\| | ~48 (plan's own estimate) | **48** (6 × 8) — **confirmed exactly**, despite C1's grep estimate being off by one; the discrepancy was entirely in the `partial` Transcendent Dragon, which was never going to reach C2/C3 regardless. The plan's "6 × 24 = 144 is an overcount" framing and its own "~48" replacement estimate are BOTH corrected-and-confirmed by this measurement — no further correction needed to the "~48" number itself, only to the intermediate C1=24 grep figure it was partly derived from. |

**Non-vacuity floor**: `all_cards().len() >= 1_700` asserted in the same test
(measured: 1,803, matching every prior batch's pin).

**Note, not pinned** (plan §5's instruction: "record the extra count as a note
rather than pinning it"): `red_elemental_blast` (`Complete`, `TargetSpellWithFilter`
admitting blue) is excluded from C3 by design (C3 is the syntactic
`TargetRequirement::TargetSpell` subset only). Of M2's 6 mutate defs, 2 are blue
(`Sea-Dasher Octopus`, `Brokkos, Apex of Forever` — mana costs read directly from
each def: `{1}{U}`, `{2}{U}{B}{G}`), so evaluating `matches_filter` against a
synthetic blue creature spell would add up to 2 more live-wrong pairs beyond the
pinned P=48 (not evaluated here, per the plan's own scope note — `pyroblast`,
the other `TargetSpellWithFilter(red)`-carrying def, is `known_wrong` and
already excluded from C2).

**Queue-row correction (I do NOT edit the queue/seed docs myself per the
brief — recording the correction here for the coordinator/close-out runner)**:
`memory/primitives/seed-rerank-2026-08-02.md` §4 row 7 and the `OOS-SIM3-5` row
in `docs/audits/decision-point-audit.md` both currently say "6 × 24 = 144,
overcount". The corrected reading, to be written in by whoever does Stage 7's
close-out: **the live-wrong pair count is a MEASURED 48** (not "~48" and not
144), and the "24" intermediate in the old "6 × 24" framing was itself off by
one against a proper enumeration (23), for the SR-36 reason recorded above
(a grep matched a comment string, not code).

### G3 revert (executed here, see Stage 6 below for the record required by the brief)

Handled together with the rest of Stage 6's revert matrix below, since G3 is a
single gate with one clean revert shape.

---

## Stage 5 — the second counter path (§3.6) + T7 + doc work (§3.2, §3.7) (this runner, DONE)

`crates/engine/src/rules/resolution.rs::counter_stack_object` rewritten in
place: the `Spell | MutatingCreatureSpell` arm plus the 20-variant ability
OR-list are collapsed onto `card_owned = crate::state::stack_registry::
card_in_stack_zone(&stack_obj.kind)`, with `card_to_move = if stack_obj.is_copy
{ None } else { card_owned }` (CR 707.10), replicating the SAME shape as
`effects/mod.rs`'s §3.5 rewrite (per the plan's own "same shape as §3.5"
instruction) — including the copy-aware `named` branch (a countered copy is
named by its own stack-entry id) and the `ActivatedAbility`/`TriggeredAbility`
source-naming branch, both absent from the function's original body but
present in the corresponding effects/mod.rs arm. The whole per-keyword "if
countered by Stifle..." comment block (`:8374-8412` at Stage-4 HEAD) was moved
**verbatim** onto the inner match's `_ => None` fallback arm — read side by
side with the pre-edit body to confirm every line survived, byte-for-byte,
just relocated.

**No information was found that the OR-list carried and the registry does
not** (plan §12 risk 3's stop-and-report condition) — the OR-list was a flat
enumeration of ability/trigger kinds with no per-variant behavioural
divergence beyond the two now-preserved special cases (Activated/Triggered
naming, everything else silent), both carried forward exactly.

**Doc correction** (plan §3.6 point 3): the function's doc comment no longer
claims `"Used by: the fizzle rule (M3-D), counterspell effects (M3-D/E)"` —
confirmed false by the SAME grep the plan's own §0.1 table already recorded
(zero production callers; the two callers are `crates/engine/tests/core/
resolution.rs:630`/`:711`, now joined by T7). The new doc states plainly: a
`pub` API with no production caller, kept as a second independent counter
path on the PB-DP9 precedent already cited in the function's own pre-existing
tail comment (*"routed through the shared helper so a future caller does not
inherit a shipped deadlock"*) — leaving one of two counter paths carrying
PB-DX25's pre-fix defect (the per-kind `Spell`-only lookup) is exactly that
shape of risk.

**T7** (`test_dx25_both_engine_counter_paths_agree`,
`crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs`): two
halves in one test, mirroring T1's and T3's fixtures respectively but routed
through `mtg_engine::rules::resolution::counter_stack_object` instead of
`Effect::CounterSpell`. Half 1 pushes a `MutatingCreatureSpell` stack object
(reusing the file's existing `push_mutating_creature_spell_stack_object`
helper) and asserts the card lands in the graveyard under a fresh id with one
`SpellCountered` event. Half 2 pushes a real `Spell`, copies it via the real
`pub rules::copy::copy_spell_on_stack`, counters the copy by its own
stack-entry id, and asserts the ORIGINAL's card stays in `ZoneId::Stack`
while `SpellCountered.stack_object_id == source_object_id == copy_stack_id`.
**Passed on first run** (no debugging cycle needed — the function's rewrite
mirrors the already-tested effects/mod.rs shape exactly).

### T7 revert — executed

| revert | how | observed failure (verbatim) | rebuild confirmed |
|---|---|---|---|
| delete the `is_copy` guard | `let card_to_move = card_owned;` (dropped `if stack_obj.is_copy { None } else { .. }`) | `panicked at .../pb_dx25_counterspell_stack_shapes.rs:1405:9: assertion \`left == right\` failed: CR 707.10: the ORIGINAL's card must still be in ZoneId::Stack -- counter_stack_object must not move it when countering a copy / left: None / right: Some(Stack)` -- i.e. the ORIGINAL's card was moved out, exactly the predicted defect, mirroring T3's revert row exactly | yes -- `Compiling mtg-engine` present in captured output before the failing test line |

Restored immediately after observing the failure; `git diff --stat --
crates/engine/src/rules/resolution.rs` confirmed **220 insertions/deletions**
(the intentional rewrite) both before and after the revert-and-restore cycle
(i.e. the restore round-tripped to the exact pre-revert edit, not to the
original pre-Stage-5 HEAD) — re-verified by re-running the full File A + File
B + File C suite green.

### CR-citation corrections (§3.7 / F4 / `OOS-UI3-1`)

Verified via MCP before editing: **CR 701.5 is "Cast"**, exactly two
subrules (701.5a, 701.5b); **CR 701.6 is "Counter"**, with 701.6a/701.6b as
quoted in plan §1. No "CR 701.5g" exists anywhere in the rules text.

* `crates/engine/src/effects/mod.rs:171` (the `EffectContext.countered_spell_
  controller` field doc, `pub countered_spell_controller`): `CR 701.5g` →
  `CR 701.6a`, with a note explaining the real warrant (the effect's own
  printed wording plus CR 701.6a — there never was a subrule 701.5g).
* `crates/engine/src/rules/events.rs:159` (`GameEvent::SpellCountered`'s doc):
  `CR 608.2b, 701.5` → `CR 608.2b, 701.6a`.
* `crates/engine/src/effects/mod.rs:2725`/`:2744` (the `Effect::CounterSpell`
  arm's own two `CR 701.5`/`701.5g` cites) and `resolution.rs:8298`/`:8304`
  (the function's opening doc) were **already corrected by the Stage-4
  runner** (confirmed by grep before touching anything -- `grep -n "CR
  701\.5\|CR 701\.6\|701\.5g"` across `effects/mod.rs`, `resolution.rs`,
  `events.rs` found exactly one remaining stale cite, the `171` one above,
  before this stage's edit; zero remain after).

**Scope respected**: only the counter-related cites in these three files were
touched. The ~337 tree-wide `CR 701.5` cites (correct citations to the actual
"Cast" rule, on unrelated code) were left untouched, per the plan's explicit
"NOT this batch" instruction.

### Doc cross-references (§3.2), `crates/simulator/src/invariants.rs`

Three edits, none behavioural:

1. `stack_card_of`'s doc comment gains a paragraph naming
   `mtg_engine::state::stack_registry::card_in_stack_zone` (PB-DX25) as a
   second, independent, DELIBERATELY duplicated classification of the same
   question -- states the "if the verifier read the engine's own answer back,
   a wrong classification would go silent in exactly the case this check
   exists to catch" argument from plan §3.2, and names the behavioural
   cross-check (`crates/simulator/tests/pb_dx25_counter_on_mutate_is_
   consistent.rs`) as what keeps the two honest without coupling them.
2. `t8_mutating_creature_spell_owns_its_stack_card`'s doc comment gains one
   sentence: its discrimination is over THIS crate's own `stack_card_of`, and
   the engine-side registry is a separate classification, cross-referenced by
   name.
3. `check_stack_consistency`'s doc block, right after the existing "Two live
   engine defects... filed as `OOS-SIM3-5`" sentence: a new paragraph stating
   PB-DX25 CLOSES `OOS-SIM3-5` -- **not** deleting the history (the finding
   motivated the fix), explaining that the fix lives in the ENGINE
   (`stack_registry::card_in_stack_zone`), not in this check, and that this
   check "was never the thing that was wrong" (it correctly classified
   `MutatingCreatureSpell` since the S8 rewrite; `Effect::CounterSpell` did
   not).

### Stage 5 verification, all EXECUTED

* `cargo check -p mtg-engine` clean (immediately after the resolution.rs
  rewrite, before writing T7).
* `cargo check -p mtg-simulator` clean (after the invariants.rs doc edits).
* `cargo test -p mtg-engine --test core resolution::` -- **10 / 0**, all
  pre-existing `counter_stack_object` callers (`test_counter_stack_object_
  spell_to_graveyard`, `test_counter_stack_object_permanent_to_graveyard_
  not_battlefield`) pass unchanged -- confirms the rewrite preserves the
  plain-`Spell` behaviour exactly, as the plan's §3.3 "character-for-character
  today's second clause" argument predicted.
* `cargo test -p mtg-engine --test primitives pb_dx25_counterspell_stack_
  shapes::` -- **7 / 0** (T1-T7 all green; T7 new this stage).
* `cargo test -p mtg-simulator --lib invariants::` -- **10 / 0**, all
  pre-existing `t1`-`t10` (incl. `t8_mutating_creature_spell_owns_its_stack_
  card`) pass unchanged after the doc-only edits.
* `cargo fmt -p mtg-engine -p mtg-simulator` then `cargo fmt --check` --
  clean (one reformatting pass on the new T7 block's line-wraps; no
  behavioural change).
* `cargo clippy -p mtg-engine -p mtg-simulator --all-targets -- -D warnings`
  -- clean.
* SR-6 scope: `git diff main..HEAD --numstat -- crates/card-defs/
  crates/card-types/ crates/view-model/ tools/` -- **EMPTY**.

No pre-existing test reddened at any point in this stage.

---

## Stage 6 — the gates (this runner, DONE)

### G3 revert -- executed

| revert | how | observed failure (verbatim) | rebuild confirmed |
|---|---|---|---|
| pinned constant off by 1 | `assert_eq!(p, 48, ...)` -> `assert_eq!(p, 49, ...)` (the P = |M2| x |C3| pin) | `panicked at .../pb_dx25_stack_registry_roster.rs:537:5: assertion \`left == right\` failed: OOS-SIM3-5 roster P (live-wrong pairs = |M2| x |C3|) moved -- expected 48 (6 x 8), got 48 (6 x 8) ... / left: 48 / right: 49` | yes -- `Compiling mtg-engine` present |

Restored immediately; `git diff --stat -- crates/engine/tests/core/
pb_dx25_stack_registry_roster.rs` empty afterward (the file matches its
Stage-1 committed state exactly); G1/G2/G3 all re-run green together.

### Acceptance criterion 6232 -- mapped against the REAL text, not inferred

Fetched verbatim via `esm task get scutemob-203` (the plan's own §6 could only
infer it -- confirmed exact match, word for word):

> "The zone-move is driven by a single per-kind classification (not per-arm
> duplication); the catch-all no longer silently swallows card-carrying kinds
> -- adding a new card-carrying StackObjectKind cannot recreate this bug
> silently (gate or exhaustive match)."

Mapping, clause by clause:

* **"driven by a single per-kind classification (not per-arm duplication)"**
  -- satisfied by `state::stack_registry::card_in_stack_zone` itself (the
  ONE classification, Stage 3) consumed by BOTH engine counter paths: `effects/
  mod.rs`'s `Effect::CounterSpell` arm (Stage 4) AND, as of THIS runner's
  Stage 5, `resolution.rs::counter_stack_object` too. The plan's own §6
  inference (G1+G2+T6) was written before Stage 5 existed and is true only of
  the FIRST path -- Stage 5 is what makes the criterion's "single... not
  per-arm duplication" clause true of the WHOLE engine rather than one of its
  two counter paths, closing the gap the plan itself flagged in §3.6 ("leaving
  one of two counter paths carrying the known-wrong shape is precisely how a
  future caller inherits a shipped defect").
* **"the catch-all no longer silently swallows card-carrying kinds"** --
  satisfied by G2 (`effects/mod.rs`'s arm: the literals `StackObjectKind::
  Spell`/`MutatingCreatureSpell` appear zero times, so there is no per-kind
  catch-all left TO swallow anything) and, by the same shape (not separately
  gated, since the plan named no new gate file for `resolution.rs`), by
  Stage 5's rewrite of `counter_stack_object`'s own catch-all -- its
  `card_owned`/`card_to_move` decision has NO wildcard arm either; only the
  DIAGNOSTICS-only `named` sub-match (which cannot lose a card, per its own
  `OOS-DX25-4` comment) still has one.
* **"adding a new card-carrying StackObjectKind cannot recreate this bug
  silently (gate or exhaustive match)"** -- satisfied by G1 (no wildcard arm
  in `card_in_stack_zone` itself -- a compile error, not a gate, for a new
  variant left unclassified) AND T6 (the classification's CONTENT is pinned
  exhaustively against all 27 variants with a non-vacuity floor, so a new
  variant wrongly classified as `Some` when it should be `None`, or vice
  versa, is caught even though it compiles). G3 additionally pins the CORPUS
  population this bug was live on, so a new card-carrying kind that widens
  the live-wrong class is caught by G3 moving too.

**Confirmed, not assumed**: the plan's inferred G1+G2+T6 triple was correct
as far as it went but incomplete -- it did not anticipate that Stage 5 (a
different runner's assignment) would extend the SAME criterion to a second
function. Recorded here so a reader trusting the plan's own §6 alone would
undercount what actually satisfies 6232 at the end of the batch.

### Full verification, ALL EXECUTED (not predicted)

* `cargo build --workspace` -- clean (7 crates compiled, no warnings).
* `cargo clippy --workspace --all-targets -- -D warnings` -- clean.
* `cargo fmt --check` -- clean (exit 0).
* `tools/check-defs-fmt.sh` -- `card-defs fmt gate: 1803 defs checked / clean`.
* `cargo test -p mtg-engine --test core protocol_schema` -- **17 / 0**, all
  green, incl. `protocol_schema_fingerprint_is_pinned`.
* `cargo test -p mtg-engine --test core hash_schema` -- **21 / 0**, all
  green, incl. `declaration_fingerprint_is_pinned` / `stream_fingerprint_is_
  pinned`.
* `cargo test -p mtg-engine --test core keyword_registry` -- **9 / 0**, all
  green, incl. `registry_sites_match_the_source_tree` (the gate that caught
  PB-DX20/PB-DX23's missed sites -- clean here, as predicted, since PB-DX25
  adds no `KeywordAbility::` literal and `effects/mod.rs` was already a
  declared `Ward` handling site).
* **PROTOCOL / HASH read directly from source, matching the gate output**:
  `grep -n "pub const PROTOCOL_VERSION" crates/engine/src/rules/protocol.rs`
  -> `35`; `grep -n "pub const HASH_SCHEMA_VERSION" crates/engine/src/state/
  hash.rs` -> `73`. **Both confirmed UNMOVED** through the whole batch
  (Stages 1 through 6), gate-executed, never hand-edited.
* `cargo test --workspace --no-fail-fast` to a FILE (never `| tail`) --
  **4,450 / 0 / 5** (37 test binaries ran, `grep -c "FAILED"` on the captured
  file returns 0, residual list EMPTY). **+2 over the Stage-4 baseline of
  4,448** -- exactly this runner's own two additions: G3
  (`g3_corpus_roster_is_pinned`, Stage 1) + T7
  (`test_dx25_both_engine_counter_paths_agree`, Stage 5). Full captured
  output: `/tmp/claude-1000/-home-skydude-projects-scutemob--worktrees-
  scutemob-203/de60b249-271f-4f80-9313-2e03f4ec0af7/scratchpad/
  pb-dx25-stage6-full-suite.txt` (scratchpad, not committed).
* `cargo test -p mtg-simulator` -- **206 / 0**, UNMOVED from the Stage-4-
  recorded pin (Stage 5's `invariants.rs` edits were doc-only).
* `cargo test -p play-server` -- **80 / 0**, UNMOVED, as predicted (this
  batch touches no play-server file).
* SR-6 scope: `git diff main..HEAD --numstat -- crates/card-defs/
  crates/card-types/ crates/view-model/ tools/` -- **EMPTY**, confirmed
  AFTER the G3 revert-and-restore cycle too (not just before it).

No pre-existing test reddened anywhere in Stage 6; every failure observed was
a revert introduced and then restored by this runner.

---

## Summary from the first runner (Stages 2, 3, 4) — kept verbatim as written

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

---

## Summary for the handoff to Stage 7's runner (this second runner, Stages 1/5/6)

* Stages 1, 5, 6 are DONE and committed (two commits so far -- Stage 1 and
  Stage 5; Stage 6 produced no lasting source diff, only a revert-and-restore
  cycle, so it has no commit of its own; this notes-file update is the final
  commit for this runner's scope).
* **Stage 1**: G3 written into the EXISTING `crates/engine/tests/core/
  pb_dx25_stack_registry_roster.rs` (not a new file). Measured, not grepped:
  M1=8, M2=6, M3=0, C1=23 (the plan's grep-derived "24" was itself wrong --
  see the Stage 1 section above), C2=18, C3=8, P=48. **The "6 x 24 = 144"
  correction owed to `memory/primitives/seed-rerank-2026-08-02.md` §4 row 7
  and `docs/audits/decision-point-audit.md`'s `OOS-SIM3-5` row is: the
  measured live-wrong pair count is 48 (not 144, not merely "~48"), and the
  intermediate "24" in that framing should read 23 with the SR-36 comment-
  string reason recorded in the Stage 1 section.** I did NOT edit either doc
  myself, per this runner's brief -- that edit is Stage 7's.
* **Stage 5**: `resolution.rs::counter_stack_object` now drives its zone-move
  off `state::stack_registry::card_in_stack_zone` (same classification as
  `effects/mod.rs`), gains the `is_copy` guard, and its stale "Used by: the
  fizzle rule..." doc claim is corrected. T7 added and passes; its revert
  (deleting the `is_copy` guard) was executed and watched red. CR-citation
  corrections applied at `effects/mod.rs:171` (the one remaining stale
  `701.5g` cite -- the two at `:2725`/`:2744` were already fixed by the
  Stage-4 runner) and `rules/events.rs:159`. Doc cross-references added at
  `crates/simulator/src/invariants.rs`'s `stack_card_of`, its `t8` test, and
  the `check_stack_consistency` `OOS-SIM3-5` paragraph (history kept, not
  deleted; a new paragraph records the closure).
* **Stage 6**: G3's revert executed (pinned `P == 48` flipped to `49`, watched
  red, restored). Acceptance criterion 6232's REAL text fetched via
  `esm task get scutemob-203` (not inferred) and mapped clause-by-clause in
  the Stage 6 section above -- the plan's own §6 inference (G1+G2+T6) was
  correct but INCOMPLETE, because it predates Stage 5's extension of the SAME
  classification to the SECOND counter path; recorded so a future reader
  trusting the plan alone would undercount what satisfies 6232.
* **Full gate suite, ALL EXECUTED**: `cargo build --workspace` clean;
  `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt
  --check` clean; `tools/check-defs-fmt.sh` clean (1,803 defs); `--test core
  protocol_schema` 17/0; `--test core hash_schema` 21/0; `--test core
  keyword_registry` 9/0; PROTOCOL **35** / HASH **73** read directly from
  source and confirmed unmoved through every stage this runner touched.
  `cargo test --workspace --no-fail-fast` to a file: **4,450 / 0 / 5**
  (+2 over the Stage-4 baseline of 4,448 -- exactly G3 + T7, this runner's
  only two new `#[test]` functions), residual list EMPTY.
  `cargo test -p mtg-simulator` **206 / 0** and `cargo test -p play-server`
  **80 / 0**, both UNMOVED from the Stage-4 pin.
* SR-6 scope: `git diff main..HEAD --numstat -- crates/card-defs/
  crates/card-types/ crates/view-model/ tools/` EMPTY, confirmed at the end
  of Stage 6.
* **NOT done here, per this runner's explicit instructions -- Stage 7 is
  owed**: `docs/audits/decision-point-audit.md`'s `OOS-SIM3-5` row
  disposition (CLOSED, with the corrected "6 x 24 = 144" -> measured 48
  reading, and the framing correction that (c) was the live shape and (a) the
  rider, per the plan's own §10 Stage 7 instruction, which this runner did not
  re-derive independently -- see the plan text for the exact wording to use);
  the v3 queue row 7 marked SHIPPED; filing the plan §11 seeds
  (`OOS-DX25-1`..`-6`); the CLAUDE.md delta and workstream-state handoff; and
  the acceptance-criteria `esm task satisfy` calls for criteria 6229-6234 (all
  six read as satisfied by the combined work of both runners, per this file's
  own record, but were not attested by either runner -- Stage 7's job).
* Final test count at the end of THIS runner's work: **4,450 / 0 / 5** on
  this branch. PROTOCOL 35 / HASH 73, both gate-confirmed unmoved.

---

## Stage 7 — coordinator measurements (worker `scutemob-203`)

### Coverage: 0 flips, verified by REGENERATION, not by an empty diff

Criterion 6233 asks for regeneration specifically, so the empty `crates/card-defs/`
diff was not accepted as sufficient. `python3 tools/authoring-report.py` executed at
`13127136`:

```
1,803 files | clean 1,133 (62.8%) | todo 519 | empty 151
plan: 1,501 / 1,636 (91.7%) authored, 135 missing, 321 extras
```

**1,133 / 1,803 = 62.8% — identical to the PB-DX24 pin.** Diffing the regenerated
`docs/authoring-status.md` against the committed one shows **only** self-dating
metadata: the `**Git:**` line (SHA + branch), the rolling "last 7 days" commit
counter, and the tail of the recent-commit list. **No coverage row moved.**

The regenerated files were then **restored with `git checkout`** and are NOT part of
this batch's diff. Note for the next reader: the committed `docs/authoring-status.md`
was generated on the `feat/sim-6-...` branch and carries that SHA — it is stale as a
*header*, current as a *measurement*. Regenerating it is a measurement, not a
deliverable, and committing it would put this branch's SHA in a file every parallel
branch also touches.

### Benches: within noise, and the direction is not a regression

`cargo bench --bench engine_perf -- --warm-up-time 1 --measurement-time 3`
(short measurement window — a sanity check, not a pinned benchmark run):

| bench | this batch | PB-DX24 pin |
|---|---|---|
| `priority_cycle_4p` | 24.3–24.7 µs | 25.5–26.0 µs |
| `full_turn_4p` | 214.1–215.4 µs | 221.5–223.5 µs |
| `sba_check` | 15.1–15.3 µs | — |
| `priority_cycle_6p` | 38.4–38.9 µs | — |
| `full_turn_6p` | 340.1–342.2 µs | — |
| `board_wipe_4p` | 123.8–126.4 µs | — |

Expected: the classification is one non-allocating `match` per stack entry examined,
on a path that runs once per counter resolution — not per priority pass and not per
SBA check. Nothing in the benched paths calls it at all, which is why the two
comparable rows land slightly *below* the prior pin rather than above; read that as
machine/measurement-window variance, **not** as an improvement this batch earned.
