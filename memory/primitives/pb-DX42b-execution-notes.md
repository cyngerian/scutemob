# PB-DX42b — execution notes (`scutemob-233`, 2026-09-05)

v4 queue **rank 18**, the LAST task of the user-approved five-task chain.
Closes **`OOS-ADJ-1` ≡ `OOS-DX19-2`** as ONE defect, plus **`OOS-DX19-1`**'s residue and
**`OOS-DX19-4`**. Rider **`OOS-ADJ-2`** taken (both halves).

Authority: `docs/audits/mtg-characteristics-recursion-adjudication.md` §3.2(iii), §3.3, §5.2.
Plan: `pb-plan-DX42b.md`. Stage 0: `pb-DX42b-stage0-census.md`, `pb-DX42b-wire-prediction.md`.

---

## 1. The defect, and why a depth counter could not fix it

`rules::layers::characteristics_for_condition` returned **printed** `obj.characteristics` for
**any** condition evaluated inside a `calculate_characteristics` walk, because the only thing it
could consult was an ambient `thread_local!` depth counter saying "somewhere inside the layer
system". CR 613.1d says a Layer-6 effect's condition reads characteristics resolved through
Layer 4, because Layer 4 has already run.

**A depth counter suppresses the ENTIRE layer system where an `EffectId` set suppresses the one
self-referential effect — and that difference IS the seven live-wrong pairs.** It is also why
`two_distinct_conditional_effects_nest_without_mutual_suppression` is unwritable against a depth
counter and is the discriminating probe for step 2.

## 2. What shipped (adjudication §5.2 steps 1-3)

1. `is_effect_active` split at its existing seam into `is_effect_duration_active` (verified free
   of any characteristics query — that is what makes step 3 non-circular) and
   `is_effect_condition_satisfied`. `is_effect_active` survives as the exact composition for its
   three non-walk callers (`rules/copy.rs`, `abilities_are_blanked`, `recompute_object_controller`).
2. `CharacteristicEvalContext { in_flight: BTreeSet<EffectId>, bound: Option<EffectLayer> }`, with
   two RAII guards (`BoundGuard`, `InFlightGuard`) that survive the early `return None` and unwind.
   `check_condition` / `check_static_condition` keep their **exact** public signatures — 63 call
   sites and four safe caller classes — as thin wrappers over `pub(crate)` `_ctx` bodies.
3. `calculate_characteristics_through(state, id, through, eval)`, with the **activity sweep bounded
   by the same `through`**. `calculate_characteristics` is now exactly that at `PtSwitch` with a
   fresh context.

`TargetFilter::required_characteristic_layer` is computed per filter **instance** and written as an
**exhaustive destructure of all 33 fields**, so a field added later is a compile error;
`Condition::required_characteristic_layer` is an **exhaustive match with no wildcard arm**.

**`LayerWalkGuard`, `LAYER_WALK_DEPTH`, `in_layer_walk` and the `process_command` balance
`debug_assert!` are RETIRED**, and the reason is a reason rather than a tidy-up: ambient
thread-local depth *can* leak across a command boundary and is sticky for the rest of the thread,
while a `&mut CharacteristicEvalContext` cannot outlive its call — the borrow checker enforces what
the assert used to police at runtime. **`OOS-DX19-4` closes by construction.** What replaces it is
`a_dead_id_early_return_does_not_corrupt_a_later_nested_walk`, a probe on the invariant that
actually can break now.

## 3. Stage-0 measurements (before any production line)

**Baseline 5,231 / 0 / 5 across 68 targets — reproduces PB-DX54's close pin EXACTLY**, the fifth
consecutive batch in which an inherited pin reproduces with no correction owed.

**Demand**: 18 conditioned `ContinuousEffectDef` instances / 16 cards / 9 variants; **2**
layer-querying; **1** deck-legal `Complete`. The v4 row-18 cell's "2 at HEAD — re-measure"
re-measures **correct**. What it does not say is that the second (`the_world_tree`) is `Partial`
and reads **Land** where the Archangel reads **Artifact**, so the supply census does not carry over
— which is `OOS-DX27-9`'s surviving half, unchanged.

**Supply**: the adjudication's **7** pairs REPRODUCE EXACTLY, re-derived from serialized payloads
without consulting its list — and this is a **CEILING as well as a floor**, because the enumeration
came from the five `LayerModification` arms that actually write `chars.card_types` at the apply site
(`Copy` / `SetTypeLine` / `AddCardTypes` / `RemoveCardTypes` / `SetCardTypes`) rather than from four
remembered variant names. `SetLandTypes` is correctly absent (PB-DX27's `OOS-ADJ-7` repair) and
`Copy` has zero corpus supply. The ceiling holds only for the bounded class: the CR 708.2a
face-down class is unbounded and is **stated, not tallied**. Hosts: 28 `Complete` printed artifact
creatures, 2 artifact lands.

## 4. Wire

**HASH 85 / PROTOCOL 44 both gate-executed (51/51) and UNMOVED**, predicted in writing at
`d90b7994` before any production line. Closure type counts **MEASURED** at **132 / 98** by raising
each gate's `MIN_CLOSURE_TYPES` to 9999 and reading its own panic text. `git diff` over
`state/hash.rs` and `rules/protocol.rs` is **EMPTY**, so no sentinel re-pin, no survivor scan, no
history row and no frozen-prefix re-pin were owed — that is the evidence, not the claim.

**The counterfactual is VERIFIED BY EXECUTION**, because "unmoved" only means something beside what
would have moved it. The rejected design — **store** the required layer on `TargetFilter` as an
`EffectLayer` field instead of computing it — was priced by planting each name in both gates'
`CLOSURE_MUST_NOT_CONTAIN`:

| planted name | `hash_schema` | `protocol_schema` |
|---|---|---|
| `TargetFilter` | **FAILS** | **FAILS** |
| `EffectLayer` | **FAILS** | **FAILS** |

Both the container and the field type are already in **both** closures, so a stored field costs
**+1 HASH and +1 PROTOCOL** plus a ~49-file sentinel re-pin and two history rows. Computing it per
instance costs **zero** on both wires. That measurement is the reason for the design, not a
preference.

## 5. The revert matrix — 9 rows, EXECUTED BY THE COORDINATOR, all three source files restored byte-exactly (`cmp`)

| row | what it undoes | result |
|---|---|---|
| **R1** | the whole fix (base characteristics inside any walk) | **6 RED** — both channel directions, the Nexus inversion, the DFC case, the nesting probe |
| **R2** | the **depth-counter** revert: suppress whenever anything is in flight, ignoring identity | **RED on exactly the nesting probe + its rider**; every channel probe stays GREEN |
| **R3** | the activity sweep's `e.layer <= through` conjunct | **GREEN — a coverage measurement, see §6** |
| **R3c** | the `in_flight` backstop only (sweep bound kept) | **GREEN 23/23 — termination IS by construction** |
| **R3b** | **both** the sweep bound and the backstop | **`fatal runtime error: stack overflow`, SIGABRT** |
| **R4** | the query bound only (ask for `PtSwitch`, sweep still bounded) | GREEN alone — same shape as R3 |
| **R4b** | R4 **plus** the backstop | **stack overflow, SIGABRT** |
| **R5** | per-**TYPE** instead of per-**INSTANCE** required layer | **4 RED** |
| **R6** | the source gate's re-key (scan only the two `pub fn` wrappers) | **the pre-batch shape stays GREEN on a planted `expect_characteristics` inside `check_static_condition_ctx`; the re-keyed gate goes RED** |
| **R7** | `OOS-DX42b-1`'s `unwrap_or(EffectLayer::Copy)` | reddened **only a vocabulary gate** before the new probe; **RED behaviourally** after |

**R1 and R2 are precise complements**, which is the only way to show that two different things are
each load-bearing: R1 proves the bounded query, R2 proves the **`EffectId` keying** — exactly the
information a depth counter loses.

**R6 took two runs to get right, and the first was not a verdict.** The initial R6b reported
`FAILED`; that was a **stale test binary** — the test-file edit and the source edit landed inside
one `cargo test` invocation. Re-run with a proper rebuild it is **GREEN**, which is the finding.
*A stale build turns a green into a red as readily as an over-wide instrument turns a verdict into a
non-verdict.*

## 6. The two rows that were COVERAGE MEASUREMENTS, not passes (`OOS-DX52-2`'s shape)

**R7** reddened exactly one test: `pb_dx39_source_view_gates::r4`, a **vocabulary** gate that
catches `OOS-DX42b-1` only incidentally, because the identifiers `unwrap_or` / `EffectLayer` /
`Copy` come back into the body. A vocabulary gate proves a body is spelled a certain way; it cannot
prove the body does the right thing, and a later batch respelling the fix while keeping the bug
satisfies it completely. Closed by
`a_layer_one_effect_with_a_characteristic_free_condition_does_not_trip_the_assert`.

**R3 reddened NOTHING**, and that is structural rather than a missing test. An effect in a later
layer **cannot change an earlier layer's output** — which is the very fact that makes bounding the
sweep semantically free — and the `in_flight` backstop absorbs the extra recursion. **No assertion
on characteristics can separate the two designs.** What separates them is TERMINATION, measured
with the complementary pair above, and **this is the first time adjudication §3.2(iii)'s claim has
been executed rather than argued**: the sweep bound is what makes the recursion finite, and the
labelled `in_flight` deviation is therefore genuinely **unreachable** rather than merely unused.
Gated by `the_activity_sweep_is_bounded_by_the_same_layer_as_the_query`, a source gate carrying that
table in its own failure message, comment-stripped so the doc paragraph above the filter cannot
satisfy it, with a body-size non-vacuity floor.

## 7. Findings

* **`OOS-DX42b-1`** — the delegated `debug_assert` used `.unwrap_or(EffectLayer::Copy)`, collapsing
  *"reads no characteristic at all"* into *"needs Layer 1"*. `Copy` is the minimum layer, so
  `required < effect.layer` is false for **every** Layer-1 effect: a Layer-1 effect carrying
  `Condition::IsYourTurn` panicked the debug build with a message claiming its condition required
  characteristics it does not require. **Reproduced by execution before it was written down.** Zero
  corpus exposure (`rules/copy.rs` measures zero `EffectLayer::Copy` in the defs), but a debug panic
  on a legitimate configuration is a defect whether or not a card reaches it.
* **`OOS-DX42b-2`** — `indomitable_archangel.rs` cited **CR 702.45a** for Metalcraft from its
  authoring. **CR 702.45a is Bushido.** Metalcraft has no CR 702.x entry at all: CR 207.2c names it
  in its own list of ability words and says ability words have *"no special rules meaning and no
  individual entries in the Comprehensive Rules"*. A wrong cite on the very card this batch is
  about — PB-DX27's shape, one axis over. Verified against the rules server, not recalled.
* **`OOS-DX42b-3`** — **`CR 613.8a(a)` is not a rule NUMBER.** CR 613.8a is a single rule whose
  (a)/(b)/(c) are an internal enumerated list, so a reader who greps the CR for `613.8a(a)` finds
  nothing. The claim the form carries is TRUE. The form is inherited from the adjudication and
  occurs in six further documents; this batch's own two new sites now say **"CR 613.8a clause (a)"**
  and the corpus-wide form is filed rather than swept.
* **`OOS-DX42b-4`** — the labelled deviation's wrong-way-round pin
  (`same_layer_self_reference_is_suppressed_not_resolved`) is `#[cfg(not(debug_assertions))]`, so it
  **does not compile into the debug test binary and never runs in CI or in any batch's close-out
  suite**. It is real coverage that no count delta will ever include. Verified by running
  `cargo test --release`, where it passes.
* **`OOS-DX42b-5`** — `greymond_avacyns_stalwart.rs`'s blocker note asserted in **compiled prose**
  that a registered static reads base characteristics inside the layer walk *"so a Human created by
  another continuous effect's type change is not counted"*, and pointed at *"OOS-DX19-2 for the CR
  613.8b-honest fixpoint"*. Both halves are now FALSE. Rewritten in place: the note was actively
  inviting an author to build a second member of a deviation that no longer exists.

## 8. Two standing gates fired on this batch's own comment edits, and both were answered

PB-DX8's `completeness_deviation_scan` fired on `indomitable_archangel` (the CR-cite correction uses
deviation language while the def ships `Complete`). Answered by **ALLOWLIST with the CONTRACT
WIDENING STATED**, on the `bolt_bend` precedent directly above it — the prose narrates a CITATION
defect and an ENGINE deviation that is now CLOSED, not this card deviating; and the entry is keyed
to a def whose faithfulness is pinned BY EXECUTION in both directions. Rewording the comment to
dodge the needle was rejected for the reason `bolt_bend`'s own entry gives: *a gate you edit prose
to satisfy has stopped measuring.*

SR-35's `tools/check-defs-fmt.sh` fired on the `greymond` rewrite. `cargo fmt --check` passes on
that file and always will — it checks none of the 1,803 defs and still exits 0.

**The process finding is the durable half.** Both were discovered **by the revert matrix**, because
the coordinator made comment edits after the last full-workspace run and did not re-take the suite,
so R1's first failing set had nine rows of which three were already red on the restored tree. That
is PB-DX28's re-take MEDIUM committed by the **coordinator**, twice in one batch (the first being
the `debug_assert` fix, which reddened `pb_dx39`'s vocabulary gate). The sharper rule:
**a revert matrix whose baseline is not green attributes its own pre-existing failures to the
revert, so the FIRST step of a revert matrix is a green baseline, not the first revert.**

## 9. Benches — a REAL regression, FOUND AND REMOVED rather than published

The first A/B measured a real, non-overlapping regression, and the honest move was to find its
cause rather than to publish it as inherent or explain it away:

> **first A/B — `sba_check` +6.76%, criterion intervals NON-OVERLAPPING** against a same-code band
> of **0.32%** on that bench (base `[15.14, 15.28]` vs head `[15.91, 16.32]`).

**Cause 1, removed.** The bounded layer list was built as
`[..10 layers..].into_iter().filter(|l| l <= through).collect::<Vec<_>>()` — a **heap allocation on
every `calculate_characteristics` call**, where the pre-batch code used a stack array.
`calculate_characteristics` runs once per battlefield permanent per SBA check, so that is one
allocation per permanent per check on the hottest path in the engine. `EffectLayer` is `Ord` and
the array is already in layer order, so the bound is a comparison, not a filter that needs
somewhere to put its result: the array is restored and the loop `break`s on `layer > through`.
**Measured: `sba_check` +6.76% → +2.98%, still non-overlapping.**

**Cause 2, removed.** `abilities_are_blanked` constructed a fresh `CharacteristicEvalContext` for
**every continuous effect**, inside a sweep that is O(permanents × effects) per SBA check. One
context for the whole sweep is not merely cheaper, it is **observationally identical** —
`InFlightGuard` removes each `EffectId` on drop. **Measured: +2.98% → +1.29%, intervals now
OVERLAPPING.**

**Final verdict: NO REGRESSION DEMONSTRATED.** Three merge-base runs and **five** HEAD runs on a
quiet machine, with the same-code band measured FIRST across the three base runs:

| bench | base med | base band | HEAD med | HEAD band | delta | overlap |
|---|---|---|---|---|---|---|
| `priority_cycle_4p` | 24.75 | 0.24% | 24.77 | 1.62% | **+0.05%** | yes |
| `priority_cycle_6p` | 38.72 | 1.12% | 39.18 | 2.97% | **+1.20%** | yes |
| `sba_check` | 15.21 | 0.32% | 15.41 | 4.71% | **+1.29%** | yes |
| `full_turn_4p` | 220.66 | 1.35% | 219.97 | 2.29% | **−0.31%** | yes |
| `full_turn_6p` | 348.20 | 0.42% | 349.87 | 2.35% | **+0.48%** | yes |
| `board_wipe_4p` | 120.76 | 2.95% | 122.58 | 3.59% | **+1.51%** | yes |

Every criterion interval overlaps and every difference is smaller than HEAD's own same-code spread
on that bench. **Nothing is claimed in either direction.**

**No struct grew**, so there is no `size_of` measurement to report: `CharacteristicEvalContext` is a
call-stack local, never a field of any hashed or serialized type — which is the same fact the wire
prediction rests on, so the two results are the same observation seen twice.

## 10. Fuzz — NEUTRAL BY MEASUREMENT, and the output is byte-identical

**PB-DX32 gate config** (seeds 1/2/3 × 25 turns, the in-tree test): every per-seed row is
**byte-identical** before and after — `T3.1` waste tallies and `T2.2` command/rejection counts on
all three seeds.

**Wider run**, matched A/B (`--games 20 --seed 1 --max-turns 200`), merge base built in its own
worktree with its own target dir: the fuzzer's entire program output differs in **exactly one
line**, the wall-clock timing (`Time: 1.9s` vs `2.1s`). Every violation count, every per-seed band
and every histogram row is identical — HARD **88 / distinct 4** across 5 of 20 games, TRANSIENT
**210 / distinct 44**, rejections **2,189 / 94,770 = 23.098‰**, 20 games completed on both sides.

Stated precisely: **no observable divergence in these invocations**, which is not the same as
proving no `public_state_hash` anywhere moved. **No ablation was owed**, because there is no
movement to attribute — and no seeded fixture was re-dealt either, since **no `Completeness` marker
moved anywhere** (checked by `git diff` over the marker, not inferred from an unchanged total).

## 11. Coverage

**UNMOVED at 1,140 / 1,803 = 63.2%**, by regeneration, **0 flips — predicted with the reason per
def before any code changed** and confirmed in **every bucket** (clean 1,140 / todo 516 / empty 147,
all identical). Self-dating churn reverted.

**3 card-def edits, all comment-only** (`indomitable_archangel` and `greymond_avacyns_stalwart`
plus the SR-35 reflow of the latter). `git diff` over the `Completeness::` marker lines in
`crates/card-defs` is **EMPTY**, so the `CORPUS_COMPLETE` SET is unmoved as well as its count and
`OOS-CARDS2-3`'s re-deal budget was checked and found **not owed**.
