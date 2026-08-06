# Primitive Batch Review: PB-DX25c — CR 115.7a's "another **legal** target"

**Date**: 2026-08-06
**Reviewer**: primitive-impl-reviewer (Opus)
**Task / branch**: `scutemob-205` / `feat/pb-dx25c-the-object-target-redirect-ignores-cr-1157as-anothe`
**CR Rules**: 115.7 / 115.7a / 115.7b / 115.7c / 115.7d / 115.7e / 115.7f, 115.3, 115.4, 109.5,
102.3, 104.3a, 601.2c, 608.2b, 608.2m, 702.11b, 702.16b, 707.10, 400.7
**Engine files reviewed**: `crates/engine/src/rules/retarget.rs` (new),
`crates/engine/src/effects/mod.rs` (`Effect::ChangeTargets` arm),
`crates/engine/src/rules/casting.rs` (`announced_requirements` hoist + push site),
`crates/engine/src/rules/abilities.rs` (5 `.targets` sites), `crates/engine/src/rules/engine.rs`
(3 literals + loyalty push), `crates/engine/src/rules/copy.rs` (3 literals),
`crates/engine/src/rules/resolution.rs` (1 literal),
`crates/card-types/src/state/stack.rs` (`StackObject.target_requirements` + `trigger_default`),
`crates/card-types/src/cards/card_definition.rs` (`Effect::ChangeTargets` doc),
`crates/engine/src/state/hash.rs` (v74 row + `HashInto`),
`crates/simulator/src/decision_coverage.rs` (1 doc line)
**Test/gate files reviewed**: `tests/primitives/pb_dx25c_retarget_legality.rs`,
`tests/core/pb_dx25c_retarget_roster.rs`, `retarget.rs::tests::r6_…`,
`crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs`,
`tests/primitives/pb_dx25b_announced_stack_target_space.rs` (T9 inverted + T9b),
`tests/rules/copy_redirect.rs`, `tests/primitives/pb_ef11_spell_single_target.rs`,
`tests/core/bare_lookup_ratchet.rs`, `tests/core/decision_site_walk.rs`,
`tests/core/pb_dx25b_announced_target_roster.rs`
**Card defs reviewed**: 2 modified (`misdirection.rs`, `bolt_bend.rs`) + 2 verified untouched
(`deflecting_swat.rs`, `untimely_malfunction.rs`, `hydroelectric_specimen.rs`)

## Verdict: needs-fix

The engine change is **CR-correct on its declared scope and the fix shape is the right one**.
`rules::retarget::plan_target_change` encodes the redirect decision once, delegates legality to
`casting::validate_targets_inner` with `so.controller` as `caster` (CR 109.5 — independently
verified against **every** `caster` consumer inside the validator family, see §"Where it is
right"), implements CR 115.7a's all-or-nothing clause, re-validates the final set per CR 115.7e,
and rebuilds `zone_at_cast` per CR 608.2b. Both card defs are comment-only, faithful to their
printed text, and correctly stay `Complete`. The HASH 73→74 bump is forced, gate-computed and
documented to this project's standard; SR-9a `mod` lines are present; the revert matrix is
executed with four honestly-disclosed undiscriminated rows. **I re-derived the census by an
inverse method (§"AC 6302") and confirm there is no sixth retarget site.**

Findings are **0 HIGH / 5 MEDIUM / 12 LOW**. Two MEDIUMs are the batch's recurring failure mode
recurring inside it: an in-source claim ("every production `.targets`-writing site records a real
list", `t9c`) that **four sites in this same commit refute**, and an assertion message (T4)
stating a counterfactual **the batch's own V7 revert measured to be false**. Two more MEDIUMs are
CR-relevant gaps the batch **discovered and then designed around in a fixture instead of filing**:
a victim with a plain `TargetSpell`/`TargetSpellWithFilter` requirement can now be redirected onto
its **own card**, and the resolving spell's own stack entry is popped before its effect runs, so
Misdirection's own 2004-10-04 ruling ("This spell is still on the stack when new targets are
selected") is unimplementable for the single-target requirements.

---

## AC 6302 — inverse re-derivation of the redirect-candidate consumer census

The plan's §2.1 census is three **forward** methods (grep the decision's vocabulary; grep
`.targets =`; grep the `TargetsChanged` emitter). All three start from a name. I used three
**inverse** methods that start from the *effect*:

**Inverse method I — every mutation of `GameState.stack_objects`, of any field, after push.**
Not `.targets` specifically: every `imbl::Vector` operation on the field, plus the `_mut()`
accessor. Measured (`Grep 'stack_objects'` over `crates/engine/src`, plus `stack_objects_mut`
tree-wide):

| operation | sites | verdict |
|---|---|---|
| `push_back` | 24 (casting, abilities ×13, engine ×4, copy ×3, resolution ×2) | construction, not mutation |
| `pop_back` | 1 (`resolution.rs:189`) | resolution |
| `.remove(pos)` | 2 (`effects/mod.rs:2768` counter, `resolution.rs:8339` `counter_stack_object`) | removal |
| **`.get_mut(pos)`** | **1 — `effects/mod.rs:7594`** | **the one retarget write** |
| `iter_mut` | 0 | — |
| `GameState::stack_objects_mut()` | 0 production callers — the fn is inside the `#[cfg(any(test, feature = "test-util"))]` block at `state/mod.rs:802`; all 62 callers are tests, `crates/simulator/src/invariants.rs` `#[cfg(test)]`, and `crates/view-model/src/tests.rs` | not a production channel |

This is strictly wider than the plan's `.targets =` grep (which would miss a whole-`StackObject`
replacement or a `..spread` reassignment) and it agrees: **one** post-push mutation site.

**Inverse method II — every production caller of the legality-validator family.** Any second
retarget must either call one of these or open-code the rule. Measured across `crates/`:
`validate_targets` (`casting.rs:6020`), `validate_targets_with_source` (`:6031`),
`validate_targets_positional` (`:6234`), `validate_targets_inner` (`:6062`),
`validate_mapped_targets` (`:6248`), `validate_object_satisfies_requirement` (`:6437`). Production
consumers: `casting::handle_cast_spell` (cast), `abilities.rs:487/501` (activation),
`engine.rs:3635` (loyalty), `queries::legal_targets_per_slot:245` (offer), and
`retarget.rs:131/149` (redirect). **Five decision points, one of which is the redirect.** No sixth.

**Inverse method III — every path by which the target *used* can differ from the target
*recorded*.** This is the method that finds a retarget implemented as "resolve differently"
rather than "mutate the stack object", which methods I and II would both miss. Two such paths
exist, and **neither is named in the plan's §2.4 out-of-scope table**:

* `EffectContext::target_remaps` (`effects/mod.rs:69`, written at `:2659`, `:2704`, `:3588`, read
  at `:7639` **before** the declared target). This is CR 400.7 identity-following inside one
  resolution — an effect moves the target to a new zone, and later effects in the same resolution
  follow the new `ObjectId`. It selects a *different id for the same object*, never a *different
  object*, is scoped to one `EffectContext`, is never written back to the stack object, and emits
  no `TargetsChanged`. **Correctly out of scope; should be listed as such** (LOW-12).
* `resolution.rs:636` `ctx.targets = slice` — a per-mode narrowing of the resolving object's
  target list. A read-side slice of an already-validated list; not a choice.

**Conclusion: I confirm there is no sixth site.** The method by which I *would* have found one is
Inverse I (any `get_mut`/`iter_mut`/whole-object replacement on `stack_objects`) combined with
Inverse III (any consumer that consults something other than `StackObject.targets` when deciding
what a resolving object targets). R5 (one `TargetsChanged` emitter) is the narrowest machine check
on this and its own doc correctly says a mutation without the event is invisible to it — that
residual is real, and Inverse I is the manual method that closes it.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **MEDIUM** | `rules/retarget.rs:124-143` | **The greedy loop validates MIXED trial sets, and one already-illegal original target aborts the whole plan.** Contradicts CR 115.7e and CR 115.7a's "even if the original target is itself illegal by then". `OOS-DX25c-1`'s wording covers only backtracking incompleteness, a different mechanism. **Fix:** widen `OOS-DX25c-1` to name the mixed-trial mechanism, and correct `retarget.rs:108-114`'s "Each trial is validated as a WHOLE SET (… CR 115.7e)". |
| E2 | **MEDIUM** | `rules/casting.rs:6450-6473` (consumed by `retarget.rs:131`) | **A `TargetSpell` / `TargetSpellWithFilter` victim can be redirected onto its OWN card** — those two arms take no `self_id` check at all. Reachable from two `Complete` deck-legal defs. Discovered by this batch (T7's own doc names it) and worked around in the fixture rather than filed. **Fix:** file it as a seed (`OOS-DX25c-5`). |
| E3 | **MEDIUM** | `rules/resolution.rs:187-190` | **Misdirection's own 2004-10-04 ruling is unimplementable**: the resolving spell's `StackObject` entry is `pop_back`ed before its effect runs, so `TargetSpellWithSingleTarget`/`TargetSpellOrAbilityWithSingleTarget` can never see it as a redirect candidate. Discovered by this batch, recorded in T7's doc, the execution notes and CLAUDE.md — **not filed in the registry**. **Fix:** file it (`OOS-DX25c-6`). |
| E4 | LOW | `rules/retarget.rs:89-101` | **`source_chars`/`victim_card` do NOT "match what `handle_cast_spell` passed".** Cast-time validation (`casting.rs:3727-3743`) runs *before* the zone move at `casting.rs:4440`, so it passes the pre-move (hand) `card` id. The retarget's values are **more** correct. **Fix:** reword to say the retarget reads the stack-resident object (CR 608.2b/613), not that it reproduces the cast-time argument. |
| E5 | LOW | `rules/retarget.rs:210-216` | `Target::Player(chooser)` is pushed unconditionally on `!has_lost && !has_conceded`, even if `chooser` is absent from `state.turn.turn_order`; `legal_targets_per_slot` (`queries.rs:223`) enumerates `turn_order` only. R6's fixture cannot see the divergence. **Fix:** either gate the chooser push on `turn_order` membership or state the deviation in R6's doc. |
| E6 | LOW | `card-types/src/cards/card_definition.rs:2453`, `:2457-2460` | Doc says `must_change: true` is "Used by Bolt Bend, Untimely Malfunction" — R1 in the same commit measures `{Bolt Bend, Misdirection, Untimely Malfunction}`; and the candidate order is described as "smallest PlayerId/ObjectId" while `retarget_candidates` walks `turn.turn_order` (seat order, not id order). **Fix:** add Misdirection; say "seat order, then ascending ObjectId". |
| E7 | LOW | `rules/copy.rs:408`, `:649` | `target_requirements: vec![]` carries no reason line of its own (plan §8 R8: "every production `vec![]` must carry a one-line reason"); it relies on the adjacent `targets: vec![]` comment. Every other production `vec![]` site does carry one. **Fix:** add the one line. |
| E8 | LOW | `tests/core/bare_lookup_ratchet.rs:98-104` | The 110 → 108 ceiling drop is framed as a conversion; it is a **relocation** — the two lookups now live in `rules/retarget.rs:81`/`:170`, a file that is not in `SWEPT_FILES`, so the ratchet's denominator quietly shrank. **Fix:** add `("src/rules/retarget.rs", N)` to `SWEPT_FILES`, or say at the comment that the sites moved out of the roster. |

## Test / Gate Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| T1 | **MEDIUM** | `tests/primitives/pb_dx25c_retarget_legality.rs:1176-1180` | **`t9c`'s doc claim is false, refuted by four sites in the same commit.** It says the fail-closed configuration "is unreachable through any real cast after §3.1's population work (every production `.targets`-writing site records a real list)". `abilities.rs:1799` (Forecast), `:2017` (Bloodrush), `:8837` (Modular), `:10975` (Scavenge) each write a **non-empty** `targets` with `target_requirements: vec![]`, deliberately and with a reason. `stack.rs:183-185` gets this right ("Empty means … OR no list was recorded at this push site"); `t9c` contradicts it. **Fix:** reword to "unreachable *on any path `plan_target_change` can reach*, because a `ChangeTargets` victim is always a `Spell`/`MutatingCreatureSpell` (`OOS-DX25b-1`)", and name the four ability sites as the reason the guard is load-bearing the day that seed closes. |
| T2 | **MEDIUM** | `tests/primitives/pb_dx25c_retarget_legality.rs:738-742` | **T4's assertion message states a counterfactual the batch measured to be false.** It says "A version that checks only `has_lost` would offer p1 BEFORE p2/p4 and this assertion would see `Player(p1)` instead." V7 (executed) shows T4 stays **green** with the `has_conceded` conjunct removed, because `validate_mapped_targets:6265` re-enforces it downstream. **Fix:** reword to what T4 actually discriminates (the whole legality delegation) and state that `retarget_candidates`' own `has_conceded` filter is defense-in-depth with no discriminating probe, exactly as the execution notes concluded. |
| T3 | LOW | `rules/retarget.rs:194-199` + all probes | The **chooser-first preference** — a deliberately preserved observable (§3.3) — has zero discriminating coverage (V9 undiscriminated). No fixture has a chooser who is legal, is not the current target, and is not first in `turn_order`. **Fix:** add such a fixture (a 4-player T3 variant with `chooser = p3`, current target = p4) or file the gap. |
| T4 | LOW | `tests/core/pb_dx25c_retarget_roster.rs:203` | R2's name claims a CR 115.7b/115.7c population pin the body never asserts (it asserts walker liveness + the `must_change: false` roster). **Fix:** rename to what it pins, or add an assertion over the `Effect` enum's own source that no `ChangeSomeTargets`-shaped variant exists. |
| T5 | LOW | `tests/core/pb_dx25c_retarget_roster.rs:261-310` | **R3 is mostly redundant with rustc.** A `StackObject { … }` literal that omits `target_requirements` does not compile unless it uses a `..base` spread, so the only shape R3 adds over the compiler is the spread form. Its doc presents it as the population's main protection. Also: it scans `crates/engine/src` only — a production literal in `crates/simulator`/`crates/view-model`/`tools/` is invisible (measured: none exist today, all use `trigger_default`). Finally the predicate at `:289-290` is redundant (`has_targets` already contains the outer conjunct). **Fix:** state both residuals in the doc; simplify the predicate. |
| T6 | LOW | `tests/core/pb_dx25c_retarget_roster.rs:326` | `extract_match_arm_body` takes the **first textual** `Effect::ChangeTargets {`. A future non-comment `matches!(e, Effect::ChangeTargets { .. })` earlier in `effects/mod.rs` silently retargets the gate to a different arm body. It fails closed (`plan_calls` would be 0) but with a misleading message. **Fix:** assert the marker occurs exactly once in the stripped source, or anchor on `=> {` proximity. |
| T7 | LOW | `crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs:239`, `:276` | S1 computes `p3_life_before` and then discards it (`let _ = p3_life_before;`); it asserts no life-total observable, and never asserts the new target **differs** from the old (V2 showed an event can fire with an unchanged set). **Fix:** assert `new_target[0].target != Target::Player(p3)` and the p1 life delta, or delete the dead binding. |
| T8 | LOW | `memory/primitives/pb-DX25c-execution-notes.md:404-416` | The §9 checklist item "T9 run **unchanged at HEAD** and recorded green" is asserted, but cross-referenced to the baseline section — which records T9 **RED** (at the post-stage-1 tree, for the fail-closed reason). The actual HEAD-green run exists only in stage 1's commit message, not reproduced here. **Fix:** paste the HEAD run's result into the notes so the checklist item is self-evidencing. |
| T9 | LOW (correction) | `tests/core/pb_dx25b_announced_target_roster.rs:409` | **Factual correction to the review brief:** PB-DX25b's R4 floor was **not** re-aimed — it is still `body.len() >= 200`. The 400 floor is the **new** DX25c R4 (`pb_dx25c_retarget_roster.rs:336`). Both extract the same 2,121-char arm body, so PB-DX25b's floor now carries 10.6× slack. Neither floor is a "the arm did not shrink" check and both docs correctly call themselves extraction-sanity floors. **Fix:** none required; optionally re-aim PB-DX25b's to 400 for consistency, with its own revert proof. |
| T10 | LOW | `tests/primitives/pb_dx25c_retarget_legality.rs:9-21` | **CR 115.3 inter-target distinctness at retarget is now tested nowhere.** T5's drop is correctly reasoned and correctly documented (n is always 1), but the plan's "CR 115.3-at-retarget comes for free" is a structural claim with no probe. **Fix:** state that in `OOS-DX25c-1`'s row alongside the greedy caveat, so a future n>1 batch knows it inherits an untested rule. |
| T11 | LOW | `tests/core/pb_dx25c_retarget_roster.rs:179-190` | R1's per-member single-target check greps the **whole def's** sanitized Debug for `TargetSpellWithSingleTarget`/`…OrAbility…`. A future def with `must_change: true` on one ability and a single-target requirement on an unrelated ability would pass. **Fix:** scope the needle to the same ability, or say in the doc that it is a def-level approximation. |
| T12 | LOW | `memory/primitives/pb-plan-DX25c.md` §2.4 | The out-of-scope table omits the two Inverse-III channels (`EffectContext::target_remaps`, `resolution.rs:636`'s per-mode `ctx.targets` slice). Both are correctly out of scope; both are the shape a *forward* census cannot see. **Fix:** add them to the plan's §2.4 (or to `retarget.rs`'s module doc) so the next batch inherits the wider census. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `misdirection.rs:7`, `:61` | Comments cite "CR 115.7a/115.7b". Misdirection's printed text is "Change **the** target", i.e. CR **115.7a** only; CR 115.7b is "change **a** target" (a different, unimplemented rule with no corpus user, as R2 pins). Pre-existing, but the batch rewrote the surrounding block. **Fix:** drop the `/115.7b`. |

## Bookkeeping Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| B1 | LOW | `CLAUDE.md:45`, `:122`; `memory/workstream-state.md:18` | All three still read "**next PB-DX25c**" / "next dispatch: PB-DX25c" after PB-DX25c shipped. The new "Last Updated" delta (`CLAUDE.md:327-350`) is accurate and correctly appended as a new bullet; the *pointer* lines were not advanced. **Fix:** advance all three to "next dispatch: **PB-DX26**" (v3 row 8). |

---

## Finding Details

### E1 — the greedy loop validates mixed trial sets

**Severity**: MEDIUM · **File**: `crates/engine/src/rules/retarget.rs:124-143`
**CR Rule**: 115.7a — "If a target can't be changed to another legal target, the original target
is unchanged, **even if the original target is itself illegal by then**." · 115.7e — "**only the
final set of targets is evaluated** to determine whether the change is legal."

**Issue.** The task's hypothesis is **confirmed, and it is a different mechanism from the one
`OOS-DX25c-1` records.** Each trial at index `i` is `next.clone()` with `trial[i] = candidate` —
so every index `> i` still holds its **original** target. `validate_targets_inner` validates the
whole slice: its two-pass best-fit assignment requires *every* element to be assignable to some
requirement (`casting.rs:6101-6144`). Therefore if any original target at an undecided index is
already illegal, **every** candidate at index 0 fails, `?` fires, and the entire plan aborts —
even when a fully-legal final assignment exists. CR 115.7e says intermediate sets are exactly what
is *not* evaluated, and CR 115.7a explicitly contemplates a now-illegal original.

`retarget.rs:108-114`'s comment states the opposite of the rule it cites: *"Each trial is
validated as a WHOLE SET (CR 115.3 inter-target distinctness, CR 115.7e)"*. A trial set is not the
final set; the code at `:145-157` knows this ("the greedy loop above validated MIXED trial
sets"), so the two comments in the same function disagree.

**Reachability: zero today, measured not assumed.** R1 pins the `must_change: true` roster at
`{Bolt Bend, Misdirection, Untimely Malfunction}` and, per member, that each carries a
single-target requirement; `casting.rs:6510`/`:6580` enforce `target_count == 1` on the victim.
With `next.len() == 1` the trial set contains no originals at all, so the defect cannot fire.

**Fix.** Widen `OOS-DX25c-1`'s row to name **two** incompletenesses, not one: (a) no backtracking
(already there); (b) intermediate-set validation, which is stricter than CR 115.7e and can abort
on an illegal original. Correct `retarget.rs:108-114` to say the trials are a *heuristic filter*,
with CR 115.7e satisfied by the final re-validation at `:149-157` alone.

### E2 — a victim can be redirected onto its own card

**Severity**: MEDIUM · **File**: `crates/engine/src/rules/casting.rs:6450-6473`
**Oracle / ruling**: Misdirection, 2004-10-04 — *"You can't make a spell which is on the stack
target itself."*

**Issue.** `validate_object_satisfies_requirement`'s `TargetSpell` / `TargetSpellWithFilter` arm
(`:6451-6473`) checks the zone and, for the filter variant, `matches_filter` — and **returns
`Ok(())` without ever consulting `self_id`**. The two single-target arms (`:6487`, `:6527`) do
check it, and `TargetFilter.exclude_self` covers the permanent family; this arm covers neither.

Before PB-DX25c `self_id` was effectively inert at cast time for spells (cast-time validation runs
at `casting.rs:3727` with the **pre-zone-move** `card` id, which can never equal a stack-zone card
id). PB-DX25c is what makes `self_id` a **live** discriminator, because `plan_target_change` passes
`victim_card` — the victim's stack-resident card id, which *is* in the candidate universe.

**Concrete failure scenario.** p2 casts Counterspell (`TargetSpell`, exactly one target) at p3's
Shock. p1 Misdirects the Counterspell. Candidates in order: players (rejected — `TargetSpell` is
object-only), then objects ascending: Shock's card (== current, excluded), **Counterspell's own
card** (`ZoneId::Stack`, passes the zone check, no self check) → picked. The Counterspell now
targets itself. At resolution `is_target_legal` passes (same zone), and `Effect::CounterSpell`'s
`stack_index_for_announced_target` returns `None` (its own entry is already popped), so it
**silently does nothing**. The CR-correct choice was Misdirection's own card (the ruling above,
positively), or the CR 115.7a fallback.

The batch **found this** — `pb_dx25c_retarget_legality.rs:896-904` says it in as many words
("with no self-exclusion at all … a plain `TargetSpell` victim would legally redirect onto ITSELF
before ever reaching Misdirection's card") — and then chose `TargetSpellWithFilter` with a colour
filter so the fixture would not hit it. That is the correct fixture decision and the wrong
disposition for the finding.

**Fix.** File `OOS-DX25c-5` (grep the registry first — no `OOS-DX25c-5` exists today): the
`TargetSpell`/`TargetSpellWithFilter` arms take no `self_id` check, so PB-DX25c's redirect can
produce a self-targeting victim on two `Complete` deck-legal defs. Note in the row that the fix is
a two-line `self_id` guard in that arm, that it also corrects the (currently inert) cast-time path,
and that it interacts with E3 — the two together are what the 2004-10-04 ruling actually requires.

### E3 — the resolving spell is not on the stack when new targets are chosen

**Severity**: MEDIUM · **File**: `crates/engine/src/rules/resolution.rs:184-191`
**Ruling**: Misdirection, 2004-10-04 — *"You can choose to make a spell on the stack target this
spell … The new target for the deflected spell is not chosen until this spell resolves. **This
spell is still on the stack when new targets are selected for the spell.**"*
**CR**: 608.2m (an instant is put into its owner's graveyard as the **final** part of its
resolution).

**Issue.** `resolve_top_of_stack_inner` does `state.stack_objects.pop_back()` **before** running
the effect. The card stays in `state.objects` with `zone == Stack` (so plain `TargetSpell` sees
it, per E2), but the **`StackObject` entry is gone**, and both single-target requirements resolve
through `stack_index_for_announced_target(&state.stack_objects, id)` (`casting.rs:6505`, `:6562`),
which therefore returns `None` → `is_spell = false` → `target_count = 0` → rejected. So a
`TargetSpellWithSingleTarget` or `TargetSpellOrAbilityWithSingleTarget` victim can **never** be
redirected onto the Misdirection/Bolt Bend that is redirecting it, contrary to the ruling.

The batch found this empirically (execution notes §5.2 design detour 2, T7's doc, and it is named
in the CLAUDE.md delta), and it is what sank T7's and T8's first drafts. **It is not in the
registry.**

**Fix.** File `OOS-DX25c-6` with the mechanism (pop-before-execute), the two affected
requirements, the ruling, and the note that it also explains why T7 had to use
`TargetSpellWithFilter` and why T8 needed a fourth stack object. Flag the shape of a fix
(resolve-in-place, or a "currently resolving" shadow entry) as a resolution-architecture change
well out of this batch's scope.

---

## Where the implementation is right — with the evidence

These are unqualified. I checked each independently rather than reading the plan's claim.

1. **CR 109.5 / `caster = so.controller` is correct for *every* consumer, not just
   `TargetOpponent`.** I enumerated every use of the `caster` parameter reachable from
   `validate_targets_inner`: `validate_mapped_targets:6265` (liveness), `:6285` (CR 702.16k
   `FromPlayer` protection source-controller), `:6298` (CR 702.11d player hexproof "opponents"),
   `:6353` → `validate_target_protection` (CR 702.11b/16b/18a "your opponents control"),
   `validate_player_satisfies_requirement:6418` (CR 102.3 `TargetOpponent` self-exclusion),
   `validate_object_satisfies_requirement:6622/6623/6669/6670` (`TargetController::You|Opponent`),
   `:6709` ("your graveyard"). **Every one of them means "the controller of the object whose
   targets these are"** — CR 109.5 exactly — so `so.controller` is right at all nine, and passing
   `ctx.controller` would be wrong at all nine. V6's executed revert confirms it behaviourally
   (T4 reddens with `left: Player(2) != right: Player(4)`).

2. **`zone_at_cast` is rebuilt correctly for every target kind, and nothing downstream broke.**
   The rebuild (`retarget.rs:166-179`) mirrors `validate_mapped_targets:6322-6366` exactly:
   `Some(obj.zone)` for objects, `None` for players. It is total — a candidate that passed
   validation is in `state.objects` by construction. I enumerated every consumer of the field:
   `resolution.rs:8293` (`is_target_legal`, `Some(obj.zone) == zone_at_cast`),
   `effects/mod.rs:9908-9912` (the same comparison), `casting.rs:4450` and `abilities.rs:1397`
   (cast/activation-time `Some(Battlefield)` checks on the caster's own declared targets, never
   reached by a redirect), and `hash.rs:4373`. **HEAD's copy-the-old-zone is a real CR 608.2b bug
   the moment old and new zones differ**, and T6's `assert_eq!(zone_at_cast, None)` after an
   object→player redirect discriminates it (V11 executed: `left: Some(Battlefield) != right:
   None`).

3. **The population is correct at every one of the nine `.targets`-writing sites, and I checked
   each against the list that site actually validated against — not just that a list is present**
   (the task's §5 request; R3 cannot establish this and its doc says so):

   | site | recorded value | validated against | verdict |
   |---|---|---|---|
   | `casting.rs:4576` | `announced_requirements.clone()` | the **same binding**, hoisted at `:3703` and consumed by both `validate_targets_positional:3730` and `validate_targets_with_source:3739` | correct **by construction**, not by agreement — the strongest available form |
   | `engine.rs:3710` | `ability_targets.clone()` | `ability_targets` at `:3638` | correct |
   | `abilities.rs:1416` | `announced_requirements` hoisted at `:515-518` | the same binding via `:490`/`:504` | correct by construction |
   | `abilities.rs:9474` | `trigger_target_requirements` computed at `:8457` | `has_ability_targets:8493` is that list's own emptiness check | correct, **with the one non-governing case disclosed in source** (`:9465-9473`: the Ward `targeting_stack_id` / `triggering_player` shortcuts take precedence and could in principle pair with a CardDef requirement; no corpus trigger does, and it is moot behind `OOS-DX25b-1`) |
   | `copy.rs:170` | `original.target_requirements.clone()` | CR 707.10 | correct |
   | `abilities.rs:1799` (Forecast), `:2017` (Bloodrush), `:8837` (Modular), `:10975` (Scavenge) | `vec![]` (from `trigger_default`) | **nothing** — each validates its target ad-hoc with no `TargetRequirement` | correct **and each carries a one-line reason**, exactly as plan §8 R8 requires. See T1: `t9c`'s doc claims the opposite. |

   The four `vec![]` sites are the honest answer, not a gap: there is no `TargetRequirement`
   variant expressing "attacking creature" (Bloodrush) or "artifact creature chosen by the
   deterministic scan" (Modular), and fabricating one would have made the recorded list a lie —
   which is the failure mode the fail-closed guard exists to convert into a visible loss of
   function.

4. **CR 115.7a's all-or-nothing clause is implemented and discriminated.** Step 6's `?`
   (`retarget.rs:141`) aborts the whole plan on any index with no legal replacement; V2's executed
   revert captures the exact bug shape it prevents (`TargetsChanged { old_targets: […Object(2)…],
   new_targets: […Object(2)…] }` — an event firing with an unchanged set). Its reachability is
   correctly stated as **zero** and **measured** by R1 rather than asserted.

5. **The T9 inversion is done properly and T9b closes the "always return None" hole.** T9's
   wrong-way-round banner and its "successor must invert" instruction are **removed** (not left to
   rot); the fixture is byte-preserved so the diff is legible; T9b adds a second creature and
   asserts the redirect lands on it, its `zone_at_cast` is `Some(Battlefield)`, the land survives
   and the original survives. A `plan_target_change` returning `None` unconditionally passes T9 and
   fails T9b. This is exactly what plan §5.1 asked for.

6. **The HASH bump is forced, gate-computed and documented to standard.** `hash.rs:757-779` (the
   doc row) and `:1204-1220` (the history row) both name the field, its `#[serde(default)]`, its
   `GameState` reachability, that `decl_fingerprint` moves and `stream_fingerprint` moves only by
   the v40 version-byte mechanism (`canonical_fixture()` cannot populate `stack_objects`), that
   T10 is the **only** thing covering the field's own bytes, that `PROTOCOL` cannot move because
   `StackObject` is in `protocol_schema.rs`'s `CLOSURE_MUST_NOT_CONTAIN`, and that
   `loop_detection.rs:144-146` picks the field up with no CR 104.4b false-negative risk because it
   is fixed at construction. V12 executed both failures (T10 **and** the coverage gate naming
   `StackObject.target_requirements`).

7. **Both card defs are faithful and comment-only.** Verified against MCP oracle text:
   Misdirection `{3}{U}{U}` Instant, pitch a blue card (CR 118.9, no life — correctly distinguished
   from Force of Will in the def's own comment), "Change the target of target spell with a single
   target" → `TargetSpellWithSingleTarget` + `must_change: true`; Bolt Bend `{3}{R}` Instant,
   cost reduction on power ≥ 4, "target spell **or ability**" → `TargetSpellOrAbilityWithSingle
   Target` + `must_change: true`. Both correctly stay `Complete` on the `OOS-DX20-10` precedent
   (completeness describes fidelity to the printed card), both correctly record `OOS-DX25b-3` as
   **closed** while leaving `OOS-DX25b-1`/`-2` explicitly open, and neither touches a
   `Completeness::partial(...)` string. `hydroelectric_specimen.rs`, `deflecting_swat.rs` and
   `untimely_malfunction.rs` are untouched, as plan §2.4 required.

8. **S1's non-vacuity anchor genuinely anchors.** It asserts `plan_targets` returns
   `TargetPlan::Announce(vec![Target::Object(life_loss_card_id)])` — an exact-value assertion
   against a real `StubProvider::legal_actions` offer — then that `RandomBot::choose_action`'s
   `Command::CastSpell` carries the identical target, then that the engine accepts it, then that
   a `TargetsChanged` fired (panic otherwise), then membership in `legal_targets_per_slot`'s own
   `TargetOpponent` answer computed with `caster = p2` (the victim's controller — correct per CR
   109.5). It cannot pass with the redirect never firing. Its weaknesses are LOW-7, not vacuity.

9. **The T5 drop and T11 fold are both permitted and both documented, not silent.** T5's reasoning
   is airtight and I re-derived it: `casting.rs:6510`/`:6580` force `target_count == 1`, so no
   real cast reaches `plan_target_change` with `so.targets.len() > 1`, so a
   `TargetPermanentDistinctFrom` CR 115.3 probe is unbuildable without the forbidden hand-built
   fixture. T11's fold into T6 is flagged in the execution notes as a judgment call rather than
   taken silently, and T6 really does carry the cross-zone assertion (`Some(Battlefield)` →
   `None`).

10. **The four undiscriminated revert rows are honest residuals, not vacuous probes.** V3 (final
    re-validation) — the plan predicted it; with n = 1 the final set *is* the last trial set, so
    the line is genuinely unreachable-to-discriminate and is kept as CR 115.7e insurance for n > 1.
    V13 (copy propagation) — `OOS-DX25b-2` makes a copy unannounceable, so no probe exists; the
    plan pre-authorised recording it rather than manufacturing a fixture. V7 — root-caused to
    `validate_mapped_targets:6265`'s independent downstream check, i.e. the filter is real
    defense-in-depth; the probe (T4) still tests the delegation, only its *message* overclaims
    (T2). V9 — root-caused to a coincidence of seat order; the preference is real and untested
    (T3). Each was confirmed by a **full `--workspace --no-fail-fast` run on the mutated tree**,
    not just the named tests, which is the right standard and stronger than the plan asked for.

11. **R4's re-aimed floor is defensible and its diagnosis of the PB-DX25b anomaly is correct.** I
    verified the mechanism independently: `extract_match_arm_body` anchors on `=> {` after the
    pattern marker, so it measures the **whole** arm — the `for` loop, the `pos` resolution, the
    `!must_change` guard, the delegation, the id capture, the mutation and the event push — of
    which the ~130-line candidate scan was only a part. The un-shrunk remainder alone (7539-7607,
    ~2.1 k comment-stripped chars) clears any sensible floor, so PB-DX25b's 200 was never at risk
    from this edit. The 400 floor is documented in the gate's own body as an *extraction-sanity*
    floor, not a "the arm did not shrink" check — which is the honest framing, and the check that
    actually catches a reintroduced decision is (b), the five zero-count needles (V15 executed:
    `left: 1 != right: 0`).

12. **Gate-defeat attempts I ran, and what happened.** R1: a def with `must_change: true` and an
    unrelated single-target ability slips the per-member check (T11) — everything else holds, and
    the 1,700-def non-vacuity floor is in the same test. R2: the name overclaims (T4) but both
    assertions are live and V17 proved the walker-liveness control fires **first**. R3: defeatable
    by the `..spread` form it is written for and by the `.targets =` form it says it cannot see;
    both disclosed (T5 adds the directory-scope residual). R4: defeatable by moving the decision
    into a helper (disclosed) and by an earlier textual marker (T6 — fails closed). R5: I confirmed
    the `push(` back-window heuristic is what makes it 1 rather than 2 (`state/hash.rs`'s
    per-variant destructure is the second textual hit), that V16 proved comment-stripping
    load-bearing on **both** the positive and negative checks, and that its doc already names the
    two ways to defeat it (build-then-push, and mutate-without-event). **In every case the gate
    either fails closed or its own doc says what it cannot reach** — the PB-DX25b R5 standard,
    met.

13. **Bookkeeping is accurate where it matters.** v3 queue row **7c** is struck with a correct
    summary (`seed-rerank-2026-08-02.md:725`), including the honest note that the PLAYER-branch
    defect "was ALSO closed in the same fix, not named by this row originally". `OOS-DX25b-3`'s
    closure row carries four corrections to its own claims and I verified each is substantive.
    `OOS-DX25c-1..4` are filed, and I grep-checked the registry: **each ID appears exactly once,
    no collision** (dispatch hygiene 5). The CLAUDE.md delta is a new short bullet, does not grow
    an existing line, and — unusually and to its credit — *reports the two structural findings and
    the four undiscriminated revert rows in the delta itself* rather than burying them.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 115.7a "another legal target" | Yes | Yes | T1 (hexproof), T2 (protection), T3 (TargetOpponent), T9-inverted, T9b, T4 |
| 115.7a fallback (no legal target ⇒ unchanged) | Yes | Yes | T9-inverted; Misdirection 2004-10-04 ruling asserted directly |
| 115.7a "even if the original is itself illegal" | Yes for n=1 | No | n=1 makes it vacuous; **E1** for n>1 |
| 115.7a all-or-nothing across targets | Yes | Indirectly | `retarget.rs:141`'s `?`; V2 discriminates; population measured zero by R1 |
| 115.7b / 115.7c | No — no DSL shape | R2 (partially, see T4) | Correctly not implemented; no corpus user |
| 115.7d ("choose new targets") | Unchanged no-op | `copy_redirect.rs:353`, `:408` stayed green | `OOS-DX25b-4` stays open, R2 restates it |
| 115.7e final-set evaluation | Yes | No discriminating probe | `retarget.rs:149-157`; V3 undiscriminated (n=1); **E1** |
| 115.7f divided/distributed | Preserved by construction | No | Only `SpellTarget.target` is rewritten; correctly not claimed as work |
| 115.3 distinctness at retarget | Delegated | **No** | T5 dropped with a correct reason; **T10** |
| 115.4 "any target" cross-kind | Yes | Yes | T6 (object → player) |
| 109.5 "you" = victim's controller | Yes | Yes | T3, T4; V6 executed |
| 102.3 / 601.2c TargetOpponent self-exclusion | Yes | Yes | T3, T4 |
| 104.3a conceded player | Yes (twice) | T4 (not discriminating — see T2) | `retarget_candidates:222` + `validate_mapped_targets:6265` |
| 601.2c self-targeting | Partly | T8 | Only the two single-target arms + `exclude_self`; **E2** |
| 608.2b `zone_at_cast` | Yes | Yes | T6, T9b; V11 executed |
| 702.11b hexproof | Delegated | Yes | T1 |
| 702.16b protection | Delegated | Yes | T2 (the only probe; V4 executed) |
| 707.10 copy carries requirements | Yes | No (V13 undiscriminated) | `copy.rs:170`; `OOS-DX25b-2` blocks a probe |
| Misdirection 2004-10-04 "target this spell" | **No** | T7 tests a proxy | **E3** |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `misdirection.rs` | Yes (MCP-verified: cost, type, both lines) | 0 | Yes | Comment-only edit; correctly stays `Complete`; C1 (115.7b cite) |
| `bolt_bend.rs` | Yes (MCP-verified: cost, type, cost reduction, both lines) | 0 | Yes for the spell half; `OOS-DX25b-1` open for the ability half, pinned wrong-way-round | Comment-only edit; correctly stays `Complete` |
| `untimely_malfunction.rs` | n/a (untouched) | 0 | `partial` for mode 2, unchanged | Correctly untouched |
| `deflecting_swat.rs` | n/a (untouched) | 0 | `must_change: false` no-op, `OOS-DX25b-4` open | Correctly untouched; R2 restates the caveat |
| `hydroelectric_specimen.rs` | n/a (untouched) | 0 | unchanged | Plan §2.4's "leave byte-unchanged" honoured |

## Seed Registry Check

| ID | Status | Collision? | Verdict |
|---|---|---|---|
| `OOS-DX25b-3` | CLOSED by PB-DX25c, `decision-point-audit.md:1370` | no | Closure is honest; the four self-corrections are each substantive. **Incomplete in one respect**: it does not record that the fix cannot make Misdirection itself a candidate for single-target victims (E3), nor the self-redirect gap (E2). |
| `OOS-DX25c-1` | filed `:1373` | no (single occurrence) | Accurate but **short by a mechanism** — see E1 |
| `OOS-DX25c-2` | filed `:1374` | no | Accurate |
| `OOS-DX25c-3` | filed `:1375` | no | Accurate; matches `retarget.rs:89-101`'s in-source note |
| `OOS-DX25c-4` | filed `:1376` | no | Accurate |
| `OOS-DX25c-5` | **filed by the fix cycle**, `decision-point-audit.md:1377` | no (grep-confirmed before filing) | Closes **E2** |
| `OOS-DX25c-6` | **filed by the fix cycle**, `decision-point-audit.md:1378` | no (grep-confirmed before filing) | Closes **E3** |
| `OOS-DX25b-1`, `-2`, `-4`, `-5` | open, unedited | — | Correct — all four are genuinely untouched by this batch |

## Previous Findings

Not a re-review. `memory/primitives/pb-review-DX25b.md` was read for house style only; its
findings are closed against `scutemob-204`.

---

## Fix cycle (`scutemob-205`, 2026-08-06) — all 22 findings taken

Per this project's standing precedent (PB-DX21 15/15, PB-DX23 13/13, PB-DX24 13/13, PB-DX25
all, PB-DX25b 12/12), every finding in this review was taken. The dispatch brief's own tally
("18 findings") undercounts the review's own tables (E1-E8 = 8, T1-T12 = 12, C1 = 1, B1 = 1 = 22);
all 22 rows are dispositioned below.

| # | Disposition | What changed |
|---|---|---|
| E1 | **taken** | `retarget.rs:108-143`'s comment corrected — it had claimed trial-set validation itself satisfied CR 115.7e (backwards: CR 115.7e says intermediate sets are NOT what decides legality). `OOS-DX25c-1`'s registry row widened to name TWO failure mechanisms (no backtracking, and mixed-trial poisoning), not one. |
| E2 | **taken** | Filed `OOS-DX25c-5` (grep-confirmed not pre-existing) — `TargetSpell`/`TargetSpellWithFilter` has no `self_id` check, live on 2 `Complete` defs. No code fix, per the finding's own directive ("Fix: file it as a seed"). |
| E3 | **taken** | Filed `OOS-DX25c-6` — the resolving spell's `StackObject` is popped before its effect runs, so Misdirection's own 2004-10-04 ruling ("this spell is still on the stack when new targets are selected") is unimplementable for the two single-target requirements. No code fix (resolution-architecture change, out of scope), per the finding's directive. |
| E4 | **taken** | `retarget.rs:89-101` reworded: the retarget reads the stack-resident (post-zone-move) object, and does NOT reproduce what `handle_cast_spell` passed at cast time (that's the PRE-move value) — corrected, with the exact line ranges on both sides of the move cited. |
| E5 | **taken (documented, not code-changed)** | `retarget_candidates`'s doc now states the deviation explicitly (chooser pushed without a `turn_order` membership check) rather than silently. Declined the alternative (gating the push) because it would be an unproven behaviour change with no failing test to justify it — the same standard PB-DX23 used to decline a reviewer-suggested change on precedent. |
| E6 | **taken** | `card_definition.rs`'s `ChangeTargets` doc: added Misdirection to the `must_change: true` user list; corrected "smallest PlayerId/ObjectId" to the actual order (seat order for players, then ascending ObjectId for objects). |
| E7 | **taken** | Both `copy.rs` `target_requirements: vec![]` sites (`:408`ish, `:649`ish, now shifted by the added comments) gained their own one-line reason, matching plan §8 R8's rule that every production `vec![]` carries one. |
| E8 | **taken** | `bare_lookup_ratchet.rs`'s `SWEPT_FILES` gained `("src/rules/retarget.rs", 0)` with a comment explaining the relocation and why the measured ceiling is 0 (the file's reads are spelled via the `.objects()` accessor method, not the bare `.objects.get(` field-access idiom this ratchet's needles match) rather than merely commenting on the drop. |
| T1 | **taken** | `t9c`'s module doc in `pb_dx25c_retarget_legality.rs` reworded: no longer claims every production `.targets`-writing site records a real list (false, refuted by 4 ability sites); now says the fail-closed config is unreachable on any path `plan_target_change` can reach (`OOS-DX25b-1`), and names the four ability sites as the reason the guard is load-bearing once that seed closes. |
| T2 | **taken** | T4's assertion message (`pb_dx25c_retarget_legality.rs`) reworded to state what T4 actually discriminates (the whole legality delegation) and to state that `retarget_candidates`'s own `has_conceded` filter is defense-in-depth with no discriminating probe (V7), instead of the disproven counterfactual claim. |
| T3 | **taken** | New `t3b_chooser_first_preference_beats_seat_order` added (4 players, chooser p3 not first in seat order, unconditional `TargetPlayer` victim) — discriminates the chooser-first preference from plain seat order. Proven by re-executing V9's exact mutation against it: reddens naming `Player(1)` (seat order) vs `Player(3)` (chooser), restored, `git diff --stat` clean. V9's row and the execution-notes summary both updated: 16 of 19 rows now discriminate (was 15). |
| T4 | **taken** | R2 renamed to `r2_effect_enum_has_no_115_7b_115_7c_variant_and_must_change_false_roster_is_pinned` AND gained a genuine source-level assertion (extracts the `Effect` enum body from `card_definition.rs` and asserts no `ChangeSomeTargets`/`ChangeATarget`/`ChangeAnyTargets` variant exists) — the population claim the old name made but the old body never checked. |
| T5 | **taken** | R3's doc now states both residuals honestly: (1) it is mostly redundant with rustc — its only real value is catching a `..spread` literal that relies on the spread's default for `target_requirements`; (2) it scans `crates/engine/src` only, measured (not assumed) that no production literal exists outside it today. The redundant `body.contains("targets:") &&` outer conjunct (already contained in `has_targets`) was simplified away. |
| T6 | **taken** | R4 gained an assertion that `"Effect::ChangeTargets {"` occurs EXACTLY ONCE in the comment-stripped source before calling `extract_match_arm_body`, guarding against the "first occurrence != only occurrence" failure mode. |
| T7 | **taken** | `pb_dx25c_bot_retarget_is_legal.rs` S1: added `assert_ne!(new_target[0].target, Target::Player(p3), ...)` (the redirect must differ from the original), and replaced the dead `let _ = p3_life_before;` with real life-delta assertions for p1/p2/p3, plus a pinned assertion that the redirect lands on p1 specifically (p2 can never be legal per CR 102.3 self-exclusion). |
| T8 | **taken** | See the dedicated note below — the true pre-PB-DX25c HEAD run (`a071e4ba`) was re-executed via a `git stash` + `git checkout` cycle (recovered mid-fix-cycle from a session-wide Bash outage) and its result — `1 passed; 0 failed` — pasted directly into the execution notes next to the checklist claim, rather than left to cross-reference the (differently-caused) Stage-1 baseline red. |
| T9 | **taken as a correction only, no source change** | Confirmed: PB-DX25b's own R4 floor (`pb_dx25b_announced_target_roster.rs:409`) is untouched, still `>= 200`; the 400 floor is DX25c's own new R4. **Declined the optional consistency re-aim of PB-DX25b's floor to 400** — it would need its own revert proof or it becomes an unverified assertion of "still discriminates", and PB-DX25b's file is not otherwise in scope for this batch (plan §2.4's byte-unchanged discipline for out-of-scope files). Recorded here rather than silently skipped. |
| T10 | **taken** | `OOS-DX25c-1`'s registry row widened again to state that CR 115.3 inter-target distinctness at retarget is tested NOWHERE — it "comes for free" only structurally (the same `validate_targets_inner` call T5 was dropped from), with no probe, for the identical zero-reachability reason as (a)/(b). |
| T11 | **taken (documented, not code-changed)** | R1's doc in `pb_dx25c_retarget_roster.rs` now states the residual: the single-target check greps the WHOLE def's sanitized Debug, not the specific ability carrying `must_change: true`, so it is a def-level approximation. Not scoped to the specific ability — today's 3 roster members each have exactly one relevant ability, so scoping is not yet load-bearing; documenting was judged sufficient over a structural rewrite for a LOW with zero live exposure. |
| T12 | **taken** | Plan `pb-plan-DX25c.md` §2.4 gained two new rows: `EffectContext::target_remaps` and `resolution.rs:636`'s per-mode `ctx.targets` slice — both Inverse-III channels the forward census could not see, both confirmed correctly out of scope. |
| C1 | **taken** | `misdirection.rs`: all three `CR 115.7a/115.7b` cites corrected to `CR 115.7a` alone (lines citing the alt-cast note, the ability comment, and the completeness-decision block) — Misdirection's printed text is "Change THE target" (115.7a), not "change A target" (115.7b, unimplemented, no corpus user per R2). Comment-only. |
| B1 | **taken** | `CLAUDE.md:45`/`:122` and `memory/workstream-state.md:18` all advanced from "next PB-DX25c" / "next dispatch: PB-DX25c" to record PB-DX25c as shipped (`scutemob-205`) and point at "next dispatch: PB-DX26" (v3 rank 8). The already-correct "Last Updated" delta bullet was left untouched, per the brief's explicit instruction. |

### T8 detail — the self-evidencing HEAD run

The checklist claim ("T9 run unchanged at HEAD and recorded green") pointed a reader at the
baseline section, which shows T9 **RED** — but that section is the STAGE-1 baseline (after
stage 1's production code landed, before stage 2's test repairs), where T9 fails for the
fail-closed reason (the field now exists but the OLD fixture doesn't populate it). That is a
different claim from "T9 was green at the TRUE pre-PB-DX25c HEAD", which is what the checklist
item is actually about.

**Execution note**: a first attempt via `git worktree add` at `a071e4ba` (in `/tmp`) was
defeated by a session-wide Bash outage (host-level `/tmp` tmpfs per-user quota exhaustion,
`EDQUOT`, confirmed independently by three diagnostic sub-agents). Once Bash recovered
partway through the fix cycle, the fix was completed by a method that never touches `/tmp`:
`git stash push -u` (this worktree, on persistent disk) → `git checkout a071e4ba` → run the
test → `git checkout` back to the working branch → `git stash pop` → `cargo check
--workspace --all-targets` (clean, confirming no corruption from the cycle). Result, captured
verbatim:

```
test pb_dx25b_announced_stack_target_space::t9_object_target_redirect_ignores_the_original_requirement ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 0.00s
```

**T9 was GREEN at the TRUE pre-PB-DX25c HEAD**, exactly as the §9 checklist claimed — the
wrong-way-round pin doing its job, now confirmed by execution rather than assumed from a
cross-reference. Full method and output: `memory/primitives/pb-DX25c-execution-notes.md`'s
own T8 section.

## Fix-cycle final gates — all EXECUTED

- `cargo check --workspace --all-targets` — clean.
- `cargo test --workspace --no-fail-fast` — **4,485 passed / 2 failed / 2 ignored**, captured
  to a file (not committed). The 2 failures are BOTH `core::card_defs_fmt` — a pre-existing
  gate this batch never touched — failing with `Os { code: 122, kind: QuotaExceeded, message:
  "Disk quota exceeded" }` while copying a script to a temp location, the identical cause as
  the session-wide Bash outage. `cargo`'s own `error: 9 targets failed:` summary lists exactly
  that one target plus 8 doctest targets (all failing the same way, before even collecting a
  test count) — no `primitives`, `simulator`, `rules`, or `play-server` target appears. The
  `primitives` binary (which houses `pb_dx25c_retarget_legality`) reports `1137 passed; 0
  failed; 2 ignored` in the same run. Pass/fail reconciles exactly against the pre-fix-cycle
  pin once the 1 new `t3b` test and the 2 quota-artifact failures are accounted for (4,486 + 1
  = 4,487 expected; 4,485 + 2 = 4,487 measured). The ignored-count (2 vs. the historical 5) is
  disclosed as unreconciled — full reasoning in the execution notes' own gates section.
- `cargo test -p mtg-engine --test core hash_schema` — 21/21 green.
  `cargo test -p mtg-engine --test core protocol_schema` — 17/17 green. **HASH 74 / PROTOCOL
  35 gate-EXECUTED and unmoved.**
- `cargo clippy --workspace --all-targets -- -D warnings` — clean, zero warnings.
- `cargo fmt --check` — clean. `tools/check-defs-fmt.sh` — clean, 1,803 defs.
- Coverage regeneration (`tools/authoring-report.py`) — **1,133/1,803 = 62.8% unmoved**, 135
  missing, byte-identical to the pre-fix-cycle count; the only diff was the self-dating
  stamp/commit-list churn, reverted with `git checkout --` before commit.
- Scope: `git status --short` shows exactly the 13 tracked files this doc's disposition table
  names, plus this review doc itself (previously never committed) and the execution notes.
