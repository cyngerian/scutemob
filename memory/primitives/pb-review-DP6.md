# Primitive Batch Review: PB-DP6 — DP-15, intervening-if at queue time (CR 603.4)

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 603.4 (primary, verified verbatim via MCP), 603.2/603.3, 605.4a, 702.33d,
702.104b, 712.8d/e, 903.3d
**Branch**: `feat/pb-dp6-intervening-if-not-checked-at-queue-time-false-positi`
(`2deb0402..ba035bcb`)

**Engine files reviewed**:
- `crates/engine/src/effects/mod.rs` (`condition_is_queue_time_evaluable`, `:9334-9415`)
- `crates/engine/src/rules/abilities.rs` (helper `:9123-9164`; gates A9 `:3764`, A10 `:4084`,
  A11 `:5064`, A12 `:5948`, A13 `:6003`, A14 `:7018`; Category-C non-gate `:6252`)
- `crates/engine/src/rules/turn_actions.rs` (A1 `:310`, A2 `:483`, A3 `:561`, A4 `:781`,
  A5 `:1768`)
- `crates/engine/src/rules/mana.rs` (A6/A6b `:847`)
- `crates/engine/src/rules/replacement.rs` (A7 `:1838`, A8 `:1873`)
- `crates/engine/src/rules/resolution.rs` (read-only: resolution-time re-check `:2139-2156`)
- `crates/engine/tests/primitives/pb_dp6_intervening_if_queue_time.rs` (new, 12 tests)
- `crates/engine/tests/primitives/pb_ac6_card_integration.rs` (de-staled Land Tax test)
- `crates/engine/tests/primitives/main.rs` (`mod` registration `:26`)

**Card defs reviewed**: 24 defs carrying `intervening_if: Some(..)` (full roster
re-derived independently — plan §5.1's count of 24 is correct; the pre-survey's 25 was
wrong). 2 defs edited: `loyal_apprentice.rs`, `siege_gang_lieutenant.rs` (caveat clears,
documentation-only).

---

## Verdict: needs-fix

The engine change is **correct and well executed**. All 14 Category-A sites were verified
line by line against plan §4's source/controller table and every one matches; the default is
`true` at every site and through every `Not`/`And`/`Or` recursion; the evaluability predicate
is genuinely exhaustive with no `_` arm and every `true` classification survives independent
CR scrutiny; the resolution-time re-check at `resolution.rs:2139-2156` is byte-for-byte
intact; Category C (`abilities.rs:6252`) and OOS-DP6-1 were both correctly left alone;
PROTOCOL 27 / HASH 64 and the five `bare_lookup_ratchet` ceilings are unmoved, read directly
from source. **I found no case where the gate wrongly suppresses a trigger that CR 603.4 says
must fire, on any live corpus card.** Hard constraints 1, 2, 3, 4 and 6 all hold.

What blocks a clean verdict is that the PB's own headline repair is **not observable**. The
runner self-reported a second, pre-existing bug at `resolution.rs:2148` and deferred it; I
verified the claim against source and it is real, and its consequence is stronger than the
close-out states: Nullpriest of Oblivion and Thieving Skydiver are both `Complete`
(`Completeness::default() == Complete`), and their entire ETB clause has **identical
observable behaviour before and after this PB** — nothing. Plan §3.3 and §5.1 rows 5/6 both
assert this batch makes those two cards "start triggering at all"; as shipped, that claim is
false. The fix is ~6 lines in the same `else` block, touches no wire surface, and does not
violate hard constraint 2 (repairing the re-check's context is not removing the re-check).
Additionally, acceptance criterion 5538 (audit doc updates + seed filing) is entirely
unsatisfied, and one test (T11) still carries the exact silent-skip defect the runner
correctly found and fixed in T4/T5/T7.

**1 HIGH, 2 MEDIUM, 3 LOW.**

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `rules/resolution.rs:2148` | **Resolution-time intervening-if re-check builds a zero-filled `EffectContext`.** `Condition::WasKicked` / `XValueAtLeast` read 0 at resolution, so the two `Complete` defs this PB claims to repair still never execute their ETB. **Fix:** here. |
| 2 | MEDIUM | `docs/audits/decision-point-audit.md` | **Criterion 5538 unsatisfied.** §4.8 L333, §5 DP-15 L460 and §8 PB-DP6 L590 are all unchanged; OOS-DP6-1..8 + the finding-1 seed are unfiled in §8.1. **Fix:** here. |
| 4 | LOW | `rules/abilities.rs:3752-3771` (A9) | **`WasKicked`/`XValueAtLeast` are structurally unanswerable at the `WhenYouCastThisSpell` site**, but classified evaluable. Zero corpus exposure. **Fix:** extend the in-source comment + seed it. |
| 5 | LOW | `memory/primitives/pb-plan-DP6.md` §2 / wip close-out | **The "15 non-testing `AbilityDefinition::Triggered` destructures" count does not reproduce** — there are 24. No Category-A site was missed. **Fix:** note only. |
| 6 | LOW | `effects/mod.rs:9364-9366` | **`And` propagation is stricter than necessary**: an evaluable-false arm could safely suppress. Conservative in the correct direction; zero corpus exposure. **Fix:** note only. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1a | **HIGH** | `nullpriest_of_oblivion.rs`, `thieving_skydiver.rs` | **`Complete` defs whose ETB clause never executes.** Consequence of engine finding 1, not of a def error. Both defs match oracle text exactly. **Fix:** fix engine finding 1; do not touch the defs. |
| — | none | `loyal_apprentice.rs`, `siege_gang_lieutenant.rs` | Caveat clears verified accurate against the post-fix engine; oracle text matches; no TODOs. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `pb_dp6_..._queue_time.rs:1049-1061` (T11) | **Still a silent-skip test for its stated claim** — the `DP6BackToken == 0` assertion is taken after `resolve_stack`, which the retained resolution re-check zeroes regardless. **Fix:** here. |

---

### Finding Details

#### Finding 1: Resolution-time re-check builds `EffectContext::new`, so the PB's headline repair is unobservable

**Severity**: HIGH
**File**: `crates/engine/src/rules/resolution.rs:2145-2155`
**CR Rule**: 603.4 — *"If the ability triggers, it checks the stated condition again as it
resolves. If the condition isn't true at that time, the ability is removed from the stack and
does nothing."*
**Oracle** (Nullpriest of Oblivion): *"When this creature enters, if it was kicked, return
target creature card from your graveyard to the battlefield."*

**Issue** — verified against source, the runner's claim is real and its scope is slightly
wider than reported:

```rust
let condition_holds = triggered_carddef_iif
    .as_ref()
    .map(|cond| {
        let ctx = EffectContext::new(              // <-- zero-fills kicker_times_paid AND x_value
            stack_obj.controller,
            source_object,
            stack_obj.targets.clone(),
        );
        check_condition(state, cond, &ctx)
    })
    .unwrap_or(true);
```

`check_condition` answers `Condition::WasKicked` as `ctx.kicker_times_paid > 0`
(`effects/mod.rs:8952`) and `XValueAtLeast(n)` as `ctx.x_value >= *n` (`:9170`).
`EffectContext::new` zero-fills both (`effects/mod.rs:190/195`). Five lines *below* this
closure, the effect-execution path at `:2160-2177` does the right thing —
`new_with_kicker(...)` plus an explicit `ctx.x_value = …` — which makes the omission clearly
unintentional rather than a deliberate asymmetry. Note this closure is **shared by both the
`is_carddef_etb` and the non-ETB registry-fallback branches** (it sits outside the
`if !is_carddef_etb` split at `:1995`), so it is not narrowly an ETB-path bug as the
close-out's wording ("the `is_carddef_etb` `condition_holds` closure") implies.

Consequence:

| card | pre-PB-DP6 | post-PB-DP6 | correct |
|---|---|---|---|
| Nullpriest of Oblivion, **kicked** | never queued (queue-time ctx zero-filled) → nothing | queued, then fizzles at `:2156` → **nothing** | reanimate |
| Nullpriest of Oblivion, unkicked | never queued → nothing | never queued → nothing | nothing ✓ |
| Thieving Skydiver, **kicked** | never queued → nothing | queued, then fizzles → **nothing** | gain control |

Both defs carry no `completeness` field, so `..Default::default()` gives them
`Completeness::Complete` (`card-types/src/cards/card_definition.rs:196-200`) — they pass
`validate_deck` and are legal in a real game while an entire printed clause silently never
fires. That is Architecture Invariant 9's stated failure mode ("a card whose abilities
silently never fired produces a corrupted history").

Plan §3.3 claims *"Building the context correctly in the shared helper fixes it"* and §5.1
rows 5/6 label both cards **"FLIP — starts triggering at all"**. As shipped, neither claim is
true; the net observable change for these two cards is zero. Since these are the only two
corpus defs pairing `WasKicked`/`XValueAtLeast` with `intervening_if` (verified: `rg
'intervening_if: Some\(Condition::(WasKicked|XValueAtLeast)'` → exactly 2 hits), fixing them
is what turns the ETB half of this PB from a no-op into a repair.

The "out of scope" argument is weak on the merits. Hard constraint 2 requires the
resolution-time re-check be **retained**, not that its internals be untouched; repairing the
context it builds strengthens it rather than optimizing it away. There is no wire surface
(no `Command`/`GameEvent`/`Effect` variant, no hashed field), no new dispatch, and no new
silent-failure site — `.objects.get(&source_object)` is already used twice immediately below
for exactly these two values.

**Fix**: **here.** Hoist the `kicker_times_paid` / `x_value` reads from `:2160-2177` above the
`condition_holds` closure (a single `state.objects.get(&source_object)` yielding both, to
avoid moving the `bare_lookup_ratchet` count for `resolution.rs`), and build the closure's
context with `EffectContext::new_with_kicker(stack_obj.controller, source_object,
stack_obj.targets.clone(), kicker_times_paid)` followed by `ctx.x_value = x_value;` — i.e.
the same shape as `abilities::carddef_intervening_if_holds_at_queue_time`. Then strengthen T1
back to the plan's real intent: assert the kicked case actually returns "GY Fodder" to the
battlefield, and delete the `NOTE` block at `pb_dp6_..._queue_time.rs:203-210`. Re-run
`bare_lookup_ratchet` and re-pin only if it genuinely moves (it should not, if both values
come from one lookup).

---

#### Finding 2: Acceptance criterion 5538 is unsatisfied — audit rows unchanged and 9 seeds unfiled

**Severity**: MEDIUM
**File**: `docs/audits/decision-point-audit.md:333`, `:460`, `:590`, §8.1 (`:605`)
**Architecture invariant**: ESM criterion 5538 ("Audit DP-15 row + PB-DP6 row updated");
plan §10 verification checklist, penultimate box.

**Issue**: read directly from the branch, none of the three rows moved:

- §4.8 L333 still reads `| Intervening-if at **queue time** | **D** | Only two paths check
  it: ETB (`rules/replacement.rs:1446-1456`) and graveyard-zone triggers
  (`rules/abilities.rs:6910-6916`) …` — still class **D**, still carrying the stale cites the
  plan's own OOS-DP6-8 identifies as 300-400 lines off.
- §5 L460 (DP-15 row) still reads `| **DP-15** | D | … |` with the same stale cites.
- §8 L590 (PB-DP6 row) is unchanged.
- §8.1 exists and is the established filing location — PB-DP4 and PB-DP5 filed 30 `OOS-DP4-*`
  / `OOS-DP5-*` rows there. PB-DP6 filed **zero**. The plan's OOS-DP6-1..8 and the runner's
  new resolution-context seed exist only inside `pb-plan-DP6.md` §9 and
  `memory/primitive-wip.md`, both of which are rotated/overwritten by the next PB.

OOS-DP6-1 in particular is load-bearing and independently verified during this review:
`build_face_ability_vectors` is **not** test-only despite the "that only happens in
`enrich_spec_from_def` for tests" comments scattered through `abilities.rs` — it is called
from `rules/resolution.rs:720` and `rules/face.rs:104` on the live permanent-creation path,
and it hardcodes `intervening_if: None` at all 34 push sites. So Aurelia the Warleader's and
Karlach's `IsFirstCombatPhase` really is checked in neither place in real games. Losing that
seed would be a material regression in the audit's value.

Separately, plan §1 issued an explicit runner instruction — *"before writing §9's seed text,
prove the claim with one throwaway probe (Aurelia-shaped def, extra combat phase, assert the
untap/token fires when it must not). If the probe fails to reproduce, say so in the review"* —
and the close-out records no probe. My source reading corroborates the claim, but the
directed empirical check was skipped and never disclosed as skipped.

**Fix**: **here.** (a) Flip §4.8 L333 to class **A**, replace the stale cites with
`replacement.rs:1838` / `abilities.rs:7018` / `resolution.rs:2139-2156`, and rewrite the cell
to state that all 14 card-def queue sites now gate. (b) Update the §5 DP-15 row to
SHIPPED/PB-DP6 with the same corrected cites. (c) Update the §8 PB-DP6 row to shipped with
the observed outcome (0 wire change, 0 completeness flips, 15 defs stop over-firing, the 2
`WasKicked` defs **pending finding 1**). (d) File OOS-DP6-1..8 verbatim from plan §9 into
§8.1, plus a new row for the resolution-context bug if finding 1 is deferred rather than
fixed — if it is fixed here, file it as closed-on-arrival with the commit reference. (e)
Either run plan §1's Aurelia probe and record the result, or state explicitly in the OOS-DP6-1
seed text that the claim is source-derived and unexecuted.

---

#### Finding 3: T11 retains the silent-skip defect the runner fixed in T4/T5/T7

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dp6_intervening_if_queue_time.rs:1046-1061`
**CR Rule**: 603.4; project convention `memory/conventions.md` — "Test-validity MEDIUMs are
fix-phase HIGHs"

**Issue**: T11 advances to `Step::Upkeep`, **immediately calls `resolve_stack`**, then asserts
`count_tokens(&state, "DP6BackToken") == 0` with the message *"the back face's OWN condition
… should gate its trigger **at queue time**"*. That assertion cannot support that claim. The
retained resolution-time re-check (`resolution.rs:2145`) evaluates
`OpponentControlsMoreLandsThanYou`, finds it false (1 land each), and removes the ability from
the stack — producing 0 tokens whether the queue-time gate ran or not. This is precisely the
pattern the runner correctly diagnosed and repaired in T4, T5 and T7 during the fail-before
cycle; T11 was left behind, and the close-out's own observed-vs-predicted table records it as
`ok` pre-fix, i.e. as a passing test that detects nothing about the gate.

The first assertion (`DP6FrontToken == 0`) is sound but is PB-OS4b/PB-RS4 face-selection
coverage, not PB-DP6 coverage. Net: T11 currently contributes zero fail-before signal for the
thing its docstring says it pins.

The fix is trivial and already proven to work in this file — T8 asserts
`stack_objects().len() == 1` at exactly `Step::Upkeep`, which establishes that the upkeep
sweep has flushed to the stack and nothing has resolved at that point.

**Fix**: **here.** Insert, between `let state = advance_to_step(state, Step::Upkeep);` and
`let state = resolve_stack(...)`:

```rust
assert!(
    state.stack_objects().is_empty(),
    "CR 712.8d/e + 603.4: the back face's own false condition must gate at QUEUE time -- \
     nothing may reach the stack (a post-resolution token count cannot distinguish this \
     from a resolution-time fizzle)"
);
```

Then re-run the mandated fail-before revert for T11 alone and record the observed pre-fix
result. Expected: still passes pre-fix, because pre-fix nothing gates and the front face is
not scanned — so the *back*-token trigger **is** queued pre-fix and the new assertion fails.
If it does not fail pre-fix, say so; that would mean the back-face sweep is not reaching this
def at all and T11 is vacuous for a second, unrelated reason.

---

#### Finding 4: A9's `WasKicked` / `XValueAtLeast` are structurally unanswerable at the cast-trigger site

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:3752-3771`
**CR Rule**: 603.4 first sentence; PB-DP6 hard constraint 3

**Issue**: `condition_is_queue_time_evaluable` classifies `WasKicked` and `XValueAtLeast` as
**evaluable** (`effects/mod.rs:9379`, `:9401`), and the helper sources them from
`state.fizzle_object(source).kicker_times_paid / .x_value`. Those are `GameObject` fields,
written exactly once, at `resolution.rs:619` (and `:628` for `x_value`) — when the spell
*resolves* into a permanent. At A9 the `source` handed to the helper is `*source_object_id`,
the spell's stack-zone `GameObject` (obtained via `fizzle_object` at `:3741`), so both fields
are still 0. A hypothetical *"When you cast this spell, if it was kicked, …"* would therefore
be **suppressed** — the exact false-negative direction hard constraint 3 forbids.

Zero live exposure: no corpus def pairs `WhenYouCastThisSpell` with any `intervening_if`
(verified across all 24 `intervening_if: Some` defs). The in-source comment at `:3752-3763` is
otherwise excellent, but it enumerates only `SourceOnBattlefield` / `SourceHasCounters` as
the surprising answers and asserts *"CR 603.4 asks the question against the game state as it
actually is"* — which is a valid defence for `SourceOnBattlefield` (the spell genuinely is not
on the battlefield) and **not** a valid defence for `WasKicked` (the spell genuinely *was*
kicked; the engine just stores that fact on the wrong object at this moment).

Note this is a distinct hazard from finding 1 and would not be closed by fixing it.

**Fix**: **note + seed.** Extend the A9 comment with a sentence naming `WasKicked` and
`XValueAtLeast` specifically and stating that they read 0 here because the object-level fields
are written at `resolution.rs:619/628`, not at cast. File as a new OOS-DP6 seed in audit §8.1
(the real repair is to read `StackObject.kicker_times_paid` at this site, or to write the
fields onto the spell's `GameObject` at cast time — both larger than a comment).

---

#### Finding 5: The "15 non-testing destructures" count does not reproduce

**Severity**: LOW
**File**: `memory/primitives/pb-plan-DP6.md` §2 (`:106-109`); `memory/primitive-wip.md`
"Un-enumerated sites hit"

**Issue**: plan §2 makes the partition mechanically reproducible via
`rg -n "AbilityDefinition::Triggered" crates/engine/src --glob '!testing/*'` and asserts it
returns **15** sites (14 Category A + 1 Category C). The close-out reports *"the grep returned
the same 15 non-`testing` destructure sites"*. Running it now returns **24** genuine
destructures (excluding comment lines and `state/hash.rs`):

| file | destructure lines | disposition |
|---|---|---|
| `turn_actions.rs` | 291, 466, 545, 766, 1757 | 5 = A1–A5 ✓ |
| `replacement.rs` | 1824, 1868 | 2 = A7, A8 ✓ |
| `mana.rs` | 822 | 1 = A6/A6b ✓ |
| `abilities.rs` | 3746, 4079, 5059, 5939, 5994, 6955 | 6 = A9–A14 ✓ |
| `abilities.rs` | 6252 | Category C ✓ |
| `abilities.rs` | 7119, 7197, 7323, 8492 | `flush_pending_triggers` post-queue reads (once-per-turn, declared-targets presence, modal modes) — **correctly out of scope** |
| `resolution.rs` | 2021, 2052, 2090 | resolution-time re-check + modal lookup — out of scope ✓ |
| `resolution.rs` | 5351 | Haunt resolution (`HauntedCreatureDies`) — reads the def but ignores `intervening_if` at **both** ends |
| `resolution.rs` | 7369 | `TurnFaceUpTrigger` resolution — already correctly seeded as OOS-DP6-5 |

**No Category-A site was missed** — the partition's conclusion is right, so this is a
bookkeeping error, not a correctness one. But the close-out presents a reproduction check that
does not reproduce, which weakens the evidentiary value of the "un-enumerated sites: none"
claim. The `resolution.rs:5351` Haunt row is a genuine (if latent) gap of the same shape as
OOS-DP6-5 and is not currently seeded anywhere.

**Fix**: **note only**, except: add `resolution.rs:5351` (Haunt, `HauntedCreatureDies`
intervening-if unchecked at queue and resolution) to the §8.1 seed batch from finding 2, and
correct the §2 count when the plan is archived.

---

#### Finding 6: `And` propagation is stricter than CR 603.4 requires

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:9364-9366`
**CR Rule**: 603.4 first sentence

**Issue**: `And(a, b) => f(a) && f(b)` means an `And` whose first arm is evaluable-and-**false**
is treated as unanswerable and the trigger is queued, even though the conjunction is
definitively false regardless of `b`. Suppression would be CR-correct there. The chosen
behaviour errs toward over-firing, which is the correct direction under hard constraint 3, and
it costs nothing today (zero corpus defs use any of the 7 unevaluable variants — verified
across all 24 `intervening_if: Some` defs). `Or` has no equivalent asymmetry: an
evaluable-true arm yields the same outcome either way.

**Fix**: **note only.** Do not change it in this batch — the current shape is trivially
auditable and any short-circuit refinement would need its own probe. Worth one sentence in the
predicate's doc comment acknowledging the conservatism is deliberate.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.4 sentence 1 (queue-time check) | Yes — 14 sites | Yes | T1 (A7), T2/T3 (A1), T4 (A4), T5 (A5), T7 (A2/A3), T9 (A14), T10 (A8) |
| 603.4 sentence 2 (resolution re-check retained) | Yes — `resolution.rs:2139-2156` verified intact | Yes | T6; also pinned by the pre-existing `pb_os9` / `pb_rs3` commander-removal tests, which correctly did **not** need adjustment (both remove the commander *after* the trigger is on the stack) |
| 603.4 + unanswerable condition ⇒ queue anyway | Yes — default `true` at all 14 sites and through `Not`/`And`/`Or` | Yes | T8 (regression pin), T12 (predicate unit test) |
| 605.4a (triggered mana ability, no stack) | Yes — A6b gated ahead of the immediate/stack split | No direct test | Latent; plan seeded the resolution-half asymmetry as OOS-DP6-7 |
| 702.104b (Tribute) + 603.4 conjunction | Yes — A8 match guard, behaviourally identical to the plan's prescribed `if` | Yes | T10, both directions |
| 712.8d/e (face-aware `intervening_if` source) | Yes — inherited from the caller's `effective_abilities(is_transformed)` walk; helper explicitly refuses to re-derive by index | **Weakly** | T11 — see finding 3 |
| 702.33d (`WasKicked` at ETB) | Queue half yes; **resolution half no** | T1 (queue only) | Finding 1 |
| 603.4 on lowered triggers (`TriggeredAbilityDef`) | **No** — OOS-DP6-1, correctly deferred (HASH bump) | No | Correct scope discipline; but see finding 2(e) — the required probe was not run |

## Card Def Summary

All 24 `intervening_if: Some(..)` defs were re-derived from source and each condition checked
for queue-time answerability. Oracle text verified via MCP `lookup_card` for the six named
cards; none of the plan's or the runner's characterisations were wrong.

| Card | Oracle Match | TODOs | Game State Correct | Notes |
|------|-------------|-------|-------------------|-------|
| `loyal_apprentice` | Yes (verified) | 0 | Yes | Caveat CLEARED accurately; stops over-firing at begin combat |
| `siege_gang_lieutenant` | Yes (verified) | 0 | Yes | Caveat CLEARED accurately; both abilities present |
| `land_tax` | Yes (verified) | 0 | Yes | Stops over-firing; `pb_ac6` test de-staled with a real new assertion |
| `searslicer_goblin` | Yes (verified) | 0 | Yes | `YouAttackedThisTurn` reset at untap (`turn_actions.rs:1491`), set at declare-attackers (`combat.rs:710`) — reads correctly at end step, no wrongful suppression |
| `nullpriest_of_oblivion` | Yes (verified) | 0 | **No** | `Complete`; ETB clause still never executes — finding 1 |
| `thieving_skydiver` | Yes (verified) | 0 | **No** | `Complete`; same — finding 1 |
| `dragonmaster_outcast`, `revel_in_riches`, `birthing_ritual`, `case_of_the_locked_hothouse`, `contaminant_grafter`, `growing_rites_of_itlimoc`, `raiders_wake`, `thaumatic_compass` | Yes | 0 | Yes | Stop over-firing; all conditions are pure state reads, safe at queue time |
| `hellkite_tyrant`, `simic_ascendancy`, `ingenious_prodigy` | n/a | — | latent | `partial`/`known_wrong`, deck-blocked; flips are latent as predicted |
| `acererak_the_archlich`, `geological_appraiser`, `the_one_ring`, `vivisection_evangelist` | Yes | 0 | Yes | ETB path, already gated pre-fix; unchanged |
| `aurelia_the_warleader`, `karlach_fury_of_avernus`, `tatyova_steward_of_tides` | Yes | 0 | **No** (pre-existing) | OOS-DP6-1 lowering-drop — correctly NOT fixed here; confirmed live (not test-only): `build_face_ability_vectors` is called from `resolution.rs:720` and `face.rs:104` |

**0 card-def source edits required by the engine change**, exactly as plan §5.1 predicted.
**0 completeness-marker flips**, as predicted.

## Gate / Invariant Verification (read directly from source, not from the close-out)

| Gate | Expected | Observed | Verdict |
|---|---|---|---|
| `PROTOCOL_VERSION` | 27 | `rules/protocol.rs:260` = **27** | ✓ |
| `HASH_SCHEMA_VERSION` | 64 | `state/hash.rs:591` = **64** | ✓ |
| `bare_lookup_ratchet` ceilings | unmoved | `effects/mod.rs` 110, `abilities.rs` 75, `replacement.rs` 24, `turn_actions.rs` 7, `mana.rs` 8 — all exactly the plan §6 values | ✓ no shortcut |
| SR-9a registration | `mod` line present | `tests/primitives/main.rs:26` | ✓ |
| SR-7 (`PendingTrigger::blank` only) | held | every push in the 14 sites uses `..PendingTrigger::blank(..)` | ✓ |
| SR-5 idiom (no `_` arm) | exhaustive | `effects/mod.rs:9345-9414`, closes on `… | Condition::YouControlYourCommander => true,` with no catch-all | ✓ |
| SR-4 (no new silent-failure site) | held | helper's only lookup is `fizzle_object` with a documented rules-correct `unwrap_or((0,0))` | ✓ |
| Hard constraint 2 (re-check retained) | intact | `resolution.rs:2139-2156` present and reachable; T6 pins it | ✓ (but see finding 1 re: its context) |
| Hard constraint 3 (never default false) | held | all 14 sites default `true`; `Not`/`And`/`Or` propagate conservatively; T8 + T12 pin it | ✓ |
| Scope: OOS-DP6-1 not attempted | held | `TriggeredAbilityDef` unchanged; 34 `intervening_if: None` pushes in `replay_harness.rs` untouched | ✓ |
| Scope: Category C not gated | held | `abilities.rs:6252` reads only `trigger_condition` + filters | ✓ |

## Deviation Assessment

| # | Deviation | Verdict |
|---|-----------|---------|
| 1 | `let sref: &GameState = state;` rebind applied at all 5 `turn_actions.rs` sites though no borrow conflict materialised | **Sound.** Harmless, consistent, and the plan explicitly instructed it in preference to a collect-then-check restructure. Honest disclosure. |
| 2 | A8's `if !tribute_was_paid` → match guard (clippy `collapsible_match`) | **Sound.** Verified at `replacement.rs:1868-1878`: guard-false falls through to `_ => {}`, byte-equivalent control flow. T10 exercises both directions. |
| 3 | T4/T5/T7 fix cycle from the runner's own fail-before revert | **Sound and creditable.** The corrected assertions are genuinely non-vacuous — each fires at the step transition before any resolution, and T8's `len() == 1` at the same point proves the trigger would be visible if wrongly queued. |
| 4 | T1's assertion narrowed to queuing only | **Unsound as a permanent state.** Defensible as a stopgap, but it converts the batch's headline card repair into an unobservable one. Resolved by fixing finding 1 and restoring the end-to-end assertion. |

## Test Validity Audit (adversarial pass over all 12)

| # | Non-vacuous? | Evidence |
|---|---|---|
| T1 | Yes (queue half) | Kicked branch asserts `stack_objects().len() == 1`; runner observed pre-fix `0`. Scope narrowed — see finding 1. |
| T2 | Yes | Asserts absence from the stack at `Step::Upkeep`, before any resolution; runner observed pre-fix panic. |
| T3 | Yes | Positive regression pin, resolves to 1 token. |
| T4 | Yes (post-fix cycle) | `stack_objects().is_empty()` at `Step::End` before `resolve_stack`. |
| T5 | Yes (post-fix cycle) | Same idiom at `Step::BeginningOfCombat`. |
| T6 | Yes | Asserts `len() == 1` at queue time, then destroys the commander, then 0 tokens — genuinely pins the retained re-check. |
| T7 | Yes (post-fix cycle) | Two pre-resolution emptiness assertions, one per phase. |
| T8 | Yes | `len() == 1` with `TargetIsLegal { index: 0 }` — fails immediately if anyone flips the unanswerable default to `false`. Value is entirely post-fix, correctly stated. |
| T9 | Yes | Calls `check_triggers` directly and inspects the returned `PendingTrigger` vec — bypasses resolution entirely. Both directions. |
| T10 | Yes | Calls `queue_carddef_etb_triggers` directly and inspects `pending_triggers`. Both directions. |
| **T11** | **No** for its stated claim | Finding 3. Front-token half is sound; back-token half cannot distinguish gate from fizzle. |
| T12 | Yes | Pure unit test; all 7 `false` variants, both `Not` directions, both `And`/`Or` directions, one `true` control. Would not compile pre-fix. |

## Disposition Summary

| # | Severity | Disposition |
|---|----------|-------------|
| 1 | HIGH | **fix here** — `resolution.rs` context repair + restore T1's end-to-end assertion |
| 2 | MEDIUM | **fix here** — audit §4.8/§5/§8 rows + file OOS-DP6-1..8 (+2 new) in §8.1; run or explicitly disclaim plan §1's Aurelia probe |
| 3 | MEDIUM | **fix here** — add the pre-resolution stack assertion to T11 and re-run its fail-before |
| 4 | LOW | **seed it** + extend the A9 in-source comment |
| 5 | LOW | **note only** — except add the `resolution.rs:5351` Haunt row to the §8.1 seed batch |
| 6 | LOW | **note only** |

No finding requires a wire bump. PROTOCOL 27 / HASH 64 should still hold after all fixes.
