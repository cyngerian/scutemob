# Review: W3-LOW sprint-2 — PB-S T3 base-characteristic sweep

**Date**: 2026-04-25
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/w3-low-cleanup-sprint-2-pb-s-t3-abilitiesrs-base-char-sweep-`
**Recipe**: `docs/mtg-engine-low-issues-remediation.md:173-195`
**CR rules**: 613.1f, 613.1e, 603.10a, 613.3, 613.7, 602.2c, 602.5b, 601.2f, 702.34 (citation note: in-codebase Channel naming; CR 702.34 is Flashback)
**Engine files reviewed**: `crates/engine/src/rules/abilities.rs`
**Test files reviewed**:
- `crates/engine/tests/animated_creature_sacrifice_cost.rs` (new, +330 LOC)
- `crates/engine/tests/grant_activated_ability.rs` (test 12 added at lines 824-878)
**Card defs spot-checked**: `otawara_soaring_city.rs`, `eiganjo_seat_of_the_empire.rs`

---

## Summary

**Verdict: PASS with one MEDIUM correctness-of-comment finding (not a runtime bug)
and several LOW process/style nits.**

Engine fixes for L02/L03/L04 are correct: each site replaces a `obj.characteristics.X`
read with `calculate_characteristics(state, id).unwrap_or_else(|| obj.characteristics.clone())`
on the relevant code path. The new regression tests for L03 and L04 exercise the right
code paths (`Command::ActivateAbility` with `sacrifice_self` and `sacrifice_filter`),
correctly install a Layer 4 `AddCardTypes(Creature)` effect, and assert
`GameEvent::CreatureDied` is emitted. They would have caught the pre-fix bug (which
emitted `PermanentDestroyed`).

L05 was correctly handled as "documented invariant + deferred refactor"; the debug_assert
was correctly removed (it would have false-positived on `ObjectSpec`-level activated
abilities, which DO exist — the L03 test itself uses that pattern). However, the inline
example in the L05 rationale is factually wrong: it cites "channel lands" as the
example of `ObjectSpec`-installed abilities, but Otawara and Eiganjo both store their
Channel abilities via `AbilityDefinition::Activated` in `card_def.abilities`. The
underlying logic is still sound (ObjectSpec-installed abilities exist; tests use them);
only the example is incorrect.

L06 test correctly inverts the timestamp ordering relative to test 9 and asserts
the grant survives.

Tests delta is consistent with +3 (L03 + L04 + L06 = 3 new tests).

---

## HIGH findings

None.

---

## MEDIUM findings

### MED-1 — L05 doc/inline comment cites incorrect example for `ObjectSpec`-level abilities

**Severity**: MEDIUM (comment-only, no runtime impact, but misleading)
**Location**: `crates/engine/src/rules/abilities.rs:546` (inline comment) and
`crates/engine/src/rules/abilities.rs:8264-8266` (doc comment on `get_self_activated_reduction`).

**Claim in comment**: *"channel lands use the ObjectSpec path, not AbilityDefinition::Activated"*
and *"Channel lands (Boseiju, Otawara, etc.) use the ObjectSpec path, so card_def.abilities
alone does not reflect the full native count."*

**Reality**: Spot-checked `crates/engine/src/cards/defs/otawara_soaring_city.rs` and
`crates/engine/src/cards/defs/eiganjo_seat_of_the_empire.rs`. Both store BOTH
their tap-for-mana and Channel abilities as `AbilityDefinition::Activated` entries
in `card_def.abilities`. The cost-reduction entry `(0, ...)` keys against the
native ability index AFTER `enrich_spec_from_def` translates the tap-for-mana entry
into `mana_abilities` (so Channel ends up at `activated_abilities[0]`).

**Why the broader rationale is still correct**: `ObjectSpec::with_activated_ability()`
is a real code path that DOES bypass `card_def.abilities` — the L03 regression test
uses exactly this pattern (`ObjectSpec::artifact(..).with_activated_ability(..)`).
So the conclusion (a debug_assert can't verify the native count from card_def alone)
is true. Only the cited example is wrong.

**Fix**: Update both comments to use a correct example. Suggested wording:

> "A debug_assert is not feasible because objects can also acquire activated abilities
> via `ObjectSpec::with_activated_ability()` at construction time (used by some
> token specs and tests), so `card_def.abilities` alone does not enumerate the full
> native ability count for an arbitrary object."

Drop the "channel lands" example since Otawara/Eiganjo do not actually exhibit it.

---

## LOW findings

### LOW-1 — `unwrap_or_else(|| obj.characteristics.clone())` fallback is dead code on all five sites

**Severity**: LOW (defensive but unreachable)
**Locations**: `abilities.rs:170-171`, `219-220`, `232-233`, `622-623`, `695-696`,
`751-752`.

`calculate_characteristics(state, id)` returns `Option<Characteristics>` and only
returns `None` when `state.objects.get(&id)` is `None`. In every site the fallback
is reached, the immediately-preceding `let obj = state.object(id)?` (or
`state.object(sac_id)?`) has already validated the object exists. So the
`unwrap_or_else` branch is never entered.

This is harmless but the inline comment in L02 says *"unwrap_or_else falls back to
base characteristics for objects not on the battlefield (LKI path)"* — that is
inaccurate. `calculate_characteristics` does not return `None` based on zone; it
returns `None` only on missing-object. The "LKI path" comment is misleading.

**Fix**: Either drop the fallback (let it `.expect()` since failure is a true bug)
or update the comments to say "defensive fallback in case the object disappears
between the `state.object()` call and this line — currently unreachable but
preserved for safety."

This is consistent with the existing PB-P LKI block at lines 766-773, which uses
`.or_else(|| state.objects.get(&sac_id).map(...)).unwrap_or_default()` for the
genuine LKI path (after `move_object_to_zone`). The non-LKI sites do not need
fallback at all.

### LOW-2 — Citation `CR 702.34` for Channel is misleading

**Severity**: LOW (citation accuracy)
**Locations**: `abilities.rs:161`, `:178`, `:600`.

Code comments cite `CR 702.34` for Channel abilities. CR 702.34 is **Flashback**.
Channel is an ability-word from Kamigawa: Neon Dynasty without an evergreen-keyword
CR rule number — Scryfall lists it under `Keywords: ["Channel"]` but the CR has
no §702.34. The dispatch logic itself is correct (zone-gated activation from hand
with discard-self cost). The citation is just wrong; it was wrong before this
sprint and is now duplicated in the new comment at line 161.

**Fix** (not required this sprint, but flag): replace `CR 702.34` with a more
accurate citation, e.g., `CR 602.2 (general activated-ability rules) + Boseiju
ruling`. Or keep the codebase convention and note that the project uses 702.34
internally for Channel as a stable identifier even though Scryfall/CR do not.

### LOW-3 — Process: commit `4c77bf50` bundles unrelated changes

**Severity**: LOW (process, not correctness)

The fmt-cleanup commit incorporated pre-existing worktree-pending changes
(`.claude/skills/` reorg, `.esm/worker.md` add) alongside legitimate `cargo fmt`
output. The user already flagged this as a known process issue. The W3 fmt
content within the commit is correct. Going forward, prefer `git add <specific
files>` over `git add -A` when staging fmt cleanup.

### LOW-4 — Test 12's CR citations: 613.3 vs 613.7

**Severity**: LOW (citation precision)
**Location**: `grant_activated_ability.rs:830,865,872`.

The L06 test docstring cites *"CR 613.1f + CR 613.3"* and *"CR 613.3 timestamp
ordering"*. CR 613.3 is the rule about CDA ordering ("CDA effects apply first
within their layer"). CR 613.7 is the rule about timestamp order for
non-dependency-related effects. Both Cryptolith Rite and Humility are non-CDA
(`is_cda: false` in the test), so the relevant rule is **613.7**, not 613.3.
The behavior is correct; only the citation is slightly off.

**Fix**: Change `CR 613.3` to `CR 613.7` in the docstring and inline comment.

### LOW-5 — L03/L04 test pattern verification: tests would fail pre-fix

**Severity**: LOW (test-quality observation, no action required)

Pre-fix, the bug was that `obj.characteristics.card_types.contains(Creature)`
read the printed type (Artifact) for an animated artifact. The artifact's death
event would be `PermanentDestroyed`, not `CreatureDied`, so the witness trigger
(`AnyCreatureDies`) would not fire. The L03 test asserts BOTH:
- `GameEvent::CreatureDied` present in activation events (direct event-emission test)
- `p2_life == 38` (witness trigger fired, dealing 1 to p2)

Both assertions exercise the post-fix code path. The first would have failed
pre-fix outright; the second is a redundant cross-check via the witness trigger.
This is good test design (not a finding — calling it out as a positive).

---

## Build/lint status (not run in this review session)

The user's worker reports:
- `cargo test --all` passes with 2689 tests (+3 from 2686 baseline; matches L03
  + L04 + L06).
- `cargo clippy --all-targets -- -D warnings` exits 0.
- `cargo fmt --check` exits 0.

I did not re-execute these in this review session. The diff itself is small and
the test additions are self-contained, so the worker's report is plausible.
**Recommendation**: trust but verify on the next merge attempt.

---

## Scope creep check

Confirmed: changes are confined to `crates/engine/src/rules/abilities.rs`
(engine fix), `crates/engine/tests/animated_creature_sacrifice_cost.rs` (new
test file), `crates/engine/tests/grant_activated_ability.rs` (single test
addition). No card-definition modifications. No HIGH-disguised changes. The only
out-of-scope content is the unintended bundling in commit `4c77bf50` (process
issue noted as LOW-3).

The recipe at `docs/mtg-engine-low-issues-remediation.md:173-195` specifies
exactly L02/L03/L04/L05/L06; all five are addressed in scope.

---

## Verdict

**READY TO CLOSE** with the following actionable item:

1. **MED-1**: Update the inline comment at `abilities.rs:546` and the doc comment at
   `abilities.rs:8264-8266` to remove the incorrect "channel lands" example. The
   broader rationale stays — only the example needs replacement. (~3 LOC.)

The remaining LOW findings are cleanup/precision nits and can be deferred or
folded into the same touch-up. They are not blockers.

---

## Findings table

| # | Severity | Location | Title |
|---|----------|----------|-------|
| MED-1 | MEDIUM | `abilities.rs:546, 8264-8266` | Comment cites wrong example for `ObjectSpec` activated abilities |
| LOW-1 | LOW | `abilities.rs:170, 219, 232, 622, 695, 751` | `unwrap_or_else` fallback is dead code; comment about LKI is misleading |
| LOW-2 | LOW | `abilities.rs:161, 178, 600` | `CR 702.34` is Flashback, not Channel (pre-existing) |
| LOW-3 | LOW | commit `4c77bf50` | fmt commit bundles unrelated worktree changes |
| LOW-4 | LOW | `grant_activated_ability.rs:830, 865, 872` | CR 613.3 should be CR 613.7 in test docstring |
| LOW-5 | LOW | `animated_creature_sacrifice_cost.rs` | Test design observation (positive) |
