# Primitive WIP — PB-DP6 (DP-15: intervening-if not checked at queue time) · PLAN

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP5 (DP-5: the `WouldDraw` multi-replacement prompt is
> unanswerable) — SHIPPED** `scutemob-153`, merge `922252f7`, HASH 63→**64**, tests **3,797**.
> Its record lives in `docs/audits/decision-point-audit.md` §5 DP-5 + §8,
> `memory/primitives/pb-plan-DP5.md` + `pb-review-DP5.md`, and the CLAUDE.md changelog entry.

- **PB**: PB-DP6 — **DP-15** (CR **603.4**). A triggered ability with an intervening-if
  clause **does not trigger at all** unless the condition is true at the moment the event
  occurs. This engine only checks the condition at **resolution** on nearly every path, so
  a trigger whose condition is false at event time is still **queued and put on the stack**
  — a false-positive trigger. (The false-negative direction is already handled correctly by
  the resolution-time re-check, which must be **retained**.)
- **Task**: `scutemob-154`
- **Branch**: `feat/pb-dp6-intervening-if-not-checked-at-queue-time-false-positi`
- **Class**: CORRECTNESS (Tier 2, class **D**, promoted into the no-wire block by audit §8).
  Rank 6 of the PB-DP suite.
- **Phase**: fix
- **Binding spec**: `docs/audits/decision-point-audit.md`
  - §4.8 table, **line 333** — "Intervening-if at **queue time** | **D** | Only two paths
    check it: ETB (`rules/replacement.rs:1446-1456`) and graveyard-zone triggers
    (`rules/abilities.rs:6910-6916`). `turn_actions.rs` and `combat.rs` contain zero
    occurrences of `intervening_if`."
  - §4.8 prose, **lines 335-337** — the false-positive framing
  - §5 **line 460** (DP-15 row) — the finding proper
  - §8 **line 590** (PB-DP6 row) — *"wire impact: **none**; already accepted as a known
    limitation in the defs; closing it retires a whole class of def-level caveats"*
  - §8.1 — where new seeds get filed
- **Plan file**: `memory/primitives/pb-plan-DP6.md`
- **Review file**: `memory/primitives/pb-review-DP6.md`

## Acceptance criteria (ESM `scutemob-154`)

1. (5535) Every trigger-queue path evaluates the intervening-if at queue time (condition
   false ⇒ trigger never queued), with tests citing **CR 603.4** covering at least the three
   audited sites **plus** resolution-time re-check retained.
2. (5536) Card-def caveats that cited the queue-time gap are cleared where this fix retires
   them; **list produced by the planner**.
3. (5537) `cargo test --all`, clippy, `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
   clean; **PROTOCOL 27 / HASH 64 unchanged** (note: the task text says "HASH 63" — that is
   **stale**, PB-DP5 bumped it to **64** the same day; the binding requirement is
   *unchanged from this branch's parent*, not a literal 63).
4. (5538) Audit DP-15 row + PB-DP6 row updated.

## Hard constraints

1. **No wire change expected**: no new `Command`, `GameEvent`, or `Effect` variant, and no
   new hashed `GameState` field. `PROTOCOL_VERSION` **27** and `HASH_SCHEMA_VERSION` **64**
   must both be unchanged. **If the design appears to require either bump, STOP and say so
   in the plan** — that is a re-scope decision, not a worker call. (PB-DP2's predicted bump
   was falsified; PB-DP3/DP4's "no change" held; PB-DP5's bump was real. Confirm
   empirically via the gate test; never hand-bump.)
2. **The resolution-time re-check stays.** CR 603.4 requires the condition to be checked in
   *both* places. Do not "optimize away" `resolution.rs:~2119-2235` on the theory that the
   queue-time gate makes it redundant — it does not (state changes between queueing and
   resolution).
3. **Do not silently drop triggers.** Every site that gains a gate must be traced: a
   condition that cannot be correctly evaluated at queue time (because the needed context
   isn't available at that point) must **not** be defaulted to `false`. State the chosen
   default per site and defend it. Wrongly suppressing a trigger is worse than the status
   quo, which only over-fires.
4. Architecture invariants #2/#3 (`GameState` sealed `pub(crate)`, mutation only through
   commands) and SR-7 (`PendingTrigger` built through `PendingTrigger::blank` only).
5. SR-4: any new silent-failure site in `effects/mod.rs` / `rules/resolution.rs` must pick a
   side (`expect_*` vs `lki_*`).
6. `crates/simulator`, `tools/tui`, `tools/replay-viewer` have exhaustive matches — run
   `cargo build --workspace` after every phase.
7. **Distinct from DP-12** (costless "you may" triggers have no DSL representation). DP-12
   is explicitly **NOT in scope**.

## Coordinator pre-survey (a hypothesis for the planner to **falsify**, not a fact base)

> PB-DP3's, PB-DP4's and PB-DP5's wip files all recorded pre-survey bullets that were wrong
> in *both* directions (PB-DP3's yield went 3 → 40; PB-DP5 had a third emit site the audit
> never named). Verify every line below against the source **as it exists on this branch**,
> and record in the plan which bullets turned out to be wrong.

### A. There appear to be **two** distinct intervening-if mechanisms, not one

The audit's §4.8 row conflates them. Confirm or refute:

1. **Card-def `Option<Condition>`** — `AbilityDefinition::Triggered { intervening_if:
   Option<Condition>, .. }` at `crates/card-types/src/cards/card_definition.rs:342`.
   Evaluated with `crate::effects::check_condition(state, cond, &ctx)`.
   - Queue-time check exists at exactly **one** place I found:
     `rules/replacement.rs:~1829-1839` (the `WhenEntersBattlefield` arm), plus a hardcoded
     sibling for `TriggerCondition::TributeNotPaid` just below it (`~:1862-1874`).
   - Resolution-time check: `rules/resolution.rs:~2001-2060`.
2. **Runtime `Option<InterveningIf>`** — `TriggeredAbility { intervening_if:
   Option<InterveningIf> }`, a **2-variant** enum at
   `crates/card-types/src/state/game_object.rs:817-827` (`ControllerLifeAtLeast`,
   `SourceHadNoCounterOfType`). Evaluated with `abilities::check_intervening_if`
   (`rules/abilities.rs:9058-9077`).
   - **14** call sites, all in `rules/abilities.rs`; **1** in `rules/resolution.rs:~2226`.

If this two-mechanism reading is right, then the audit's "only two paths check it" is
**wrong about mechanism 2** (which is checked in 14 places, mostly death/graveyard sweeps)
and **right about mechanism 1** (ETB only). The plan must say which mechanism DP-15 is
actually about, and whether the sweep covers one or both. My hypothesis: **mechanism 1 is
the real gap**; mechanism 2 may have its own, smaller holes worth auditing.

### B. The queue-site roster is much larger than the audit's three

`PendingTrigger::blank` appears **82** times in `crates/engine/src/`:

| file | count |
|---|---|
| `rules/abilities.rs` | 46 |
| `rules/turn_actions.rs` | 22 |
| `rules/resolution.rs` | 6 |
| `rules/replacement.rs` | 3 |
| `effects/mod.rs` | 2 |
| `rules/casting.rs`, `rules/mana.rs`, `rules/miracle.rs` | 1 each |

Not all 82 are card-def-driven — many are hardcoded keyword triggers (Suspend, Vanishing,
Echo, Madness, Backup…) with no `intervening_if` to read at all. **The plan must partition
the 82** into (a) sites that read an `AbilityDefinition::Triggered` from the card registry
and therefore *have* an `intervening_if` field in hand, (b) sites that don't, and (c) any
site where the field is in hand and *deliberately* ignored. Only (a) and (c) are in scope.
The audit's three named sites are a **starting roster, not the full set** — the task
description says so explicitly.

Known self-documentation of the gap at `rules/turn_actions.rs:264-266`:

> `// CR 603.4: Intervening-if conditions (if any) are checked at resolution via the Normal`
> `// trigger dispatch path, consistent with all other CardDef trigger handling.`

That comment sits on the **generic CardDef upkeep trigger sweep** and is the clearest
in-source admission. `turn_actions.rs` also owns draw-step, end-step, begin-combat and
several ETB-adjacent sweeps; `combat.rs` owns attack/block/damage triggers. Sweep them all.

### C. Card-def exposure — 25 defs, ~2 explicit caveats

- **25** files contain `intervening_if: Some(...)`. Full list obtainable with
  `grep -rln "intervening_if: Some" crates/card-defs/src/defs/`. The planner must classify
  each by its `TriggerCondition` — an ETB-condition def is **already correct** (mechanism 1's
  one existing gate covers it); a non-ETB one (upkeep, combat, attack, end-step) is
  **live-wrong today**. Expect the live-wrong set to be materially smaller than 25.
  Spot-checks suggesting non-ETB and therefore live-wrong: `loyal_apprentice.rs` and
  `siege_gang_lieutenant.rs` (`AtBeginningOfCombat` + `YouControlYourCommander`),
  `land_tax.rs` (upkeep + `OpponentControlsMoreLandsThanYou`),
  `dragonmaster_outcast.rs` (upkeep + `YouControlNOrMoreWithFilter`),
  `raiders_wake.rs` / `searslicer_goblin.rs` (`YouAttackedThisTurn`),
  `karlach_fury_of_avernus.rs` / `aurelia_the_warleader.rs` (`IsFirstCombatPhase`),
  `simic_ascendancy.rs` / `ingenious_prodigy.rs` (`SourceHasCounters`),
  `revel_in_riches.rs`, `hellkite_tyrant.rs`, `birthing_ritual.rs`,
  `growing_rites_of_itlimoc.rs`, `thaumatic_compass.rs`,
  `tatyova_steward_of_tides.rs`, `case_of_the_locked_hothouse.rs`,
  `acererak_the_archlich.rs`, `contaminant_grafter.rs`, `vivisection_evangelist.rs`.
  **Verify every one — do not trust this list.**
- **Criterion 5536's caveat set.** Two defs carry a caveat that names the queue-time gap in
  so many words and are the clearest candidates for clearing:
  - `crates/card-defs/src/defs/loyal_apprentice.rs:19-30` — *"`intervening_if` is checked
    only at resolution (resolution.rs:2125-2135), never at queue time, though CR 603.4
    requires both. Divergent case: you do NOT control your commander…"*
  - `crates/card-defs/src/defs/siege_gang_lieutenant.rs:19-20` — same text, cross-references
    loyal_apprentice's top-of-file note.
  Several other defs say *"re-checked at resolution (CR 603.4)"* in a way that is **still
  accurate after the fix** (the resolution re-check is retained) — do **not** blanket-delete
  those. The criterion is "cleared **where this fix retires them**". Produce an explicit
  keep/clear table with a reason per row.
- **A separate class exists and is OUT OF SCOPE**: defs whose caveat says the *`Condition`
  DSL lacks the needed variant* (`dwynen_s_elite.rs:22`, `ophiomancer.rs:22`,
  `guardian_project.rs:6`, `emeria_the_sky_ruin.rs:42`, `inventors_fair.rs:6`,
  `garruks_uprising.rs:23`, `vampire_socialite.rs:28`, `thousand_faced_shadow.rs:6`,
  `jadar_ghoulcaller_of_nephalia.rs:33`). Those are DSL-gap seeds, **not** queue-time-gap
  caveats. This fix does not retire them. **File them as a seed; do not author new
  `Condition` variants here.** Distinguishing the two classes cleanly is a required plan
  deliverable.

### D. Traps to check before writing code

- **`check_condition`'s context.** The ETB gate builds
  `EffectContext::new(controller, new_id, vec![])`. Each new gate site needs the *correct*
  source object and controller — a sweep that iterates `state.objects` has the object in
  hand, but a sweep keyed off an event may not. Getting the source wrong turns a correct
  condition into a silently wrong one.
- **Layer resolution.** Per the W3-LC audit, battlefield reads must go through
  `calculate_characteristics()`, not `obj.characteristics.X`. If `check_condition` (or any
  helper the new gates call) reads raw characteristics, the queue-time answer can differ
  from the resolution-time answer for reasons that have nothing to do with CR 603.4.
  Confirm which one `check_condition` uses and note it.
- **Face-awareness (PB-OS4b / PB-RS4, CR 712.8d/e).** Several sweeps already call
  `effective_abilities(obj.is_transformed)` to read the active face. A new gate must read
  the intervening-if from the **same** face the ability came from, and must use the same
  `ability_index` namespace. `turn_actions.rs:~277-279` shows the established pattern.
- **`Condition` variants that are inherently event-relative.** `WasCast`, `WasKicked`,
  `SourceHasCounters`, `IsFirstCombatPhase`, `YouAttackedThisTurn` — each must be checked
  for "does this evaluate meaningfully at the moment the event occurs?" `WasCast`/`WasKicked`
  in particular are ETB-shaped and may already be fine.
- **SR-25 `bare_lookup_ratchet`** (`crates/engine/tests/core/bare_lookup_ratchet.rs`) fires
  on any change **up or down** in the swept files; `abilities.rs`/`turn_actions.rs` are
  likely swept. Expect to re-pin, and say so.
- **Existing tests/scripts may assert the false positive.** The task text warns of this.
  `crates/engine/tests/mechanics_e_l/evolve.rs:1002`
  (`test_evolve_intervening_if_fails_at_resolution`),
  `crates/engine/tests/mechanics_e_l/graft.rs:857`
  (`test_graft_resolution_recheck_intervening_if`) and
  `crates/engine/tests/primitives/pb_ac8_restrictions_and_wingame.rs:332`
  (`test_wingame_via_intervening_if_upkeep_trigger`) are the obvious candidates. Any test
  that has to change is **evidence the fix is real** — but each change must be justified
  against CR 603.4 in the plan, not adjusted to fit.

### E. Yield calibration

Per `feedback_pb_yield_calibration`, discount any card-yield estimate 2–3×. Also note the
audit's §8 note: this PB is expected to produce **behaviour flips, not `Complete` flips** —
the affected defs are mostly already `Complete` and merely *over-fire*. A def that stops
over-firing does not change its completeness marker. Predict the flip count explicitly and
be prepared for it to be **0**.

## Out of scope — file as seeds in the plan's seed section, do not fix here

- DP-12 (costless "you may" on triggers has no DSL representation) — a different finding.
- Authoring new `Condition` variants to close the DSL gaps in §C's third bullet.
- Mechanism 2's (`InterveningIf`, the 2-variant runtime enum) own coverage holes, if the
  plan finds any that are not part of DP-15 proper — seed them.
- DP-14 (same-controller trigger ordering).

## Plan phase output required

`memory/primitives/pb-plan-DP6.md` containing:

1. Which mechanism(s) DP-15 is about, with the §A hypothesis confirmed or refuted.
2. The **full partitioned inventory** of the 82 `PendingTrigger::blank` sites with line
   numbers **as they exist on this branch** — in-scope / not-applicable / deliberately-ignored.
3. The fix shape: one shared helper vs. per-site gates, with the argument. Include how the
   helper gets the right `EffectContext` and the right face.
4. The per-site default for a condition that cannot be evaluated at queue time (hard
   constraint 3), stated site by site.
5. The card-def classification table: all 25 `intervening_if: Some` defs → ETB (already
   correct) / non-ETB (live-wrong, fixed here), and the separate keep/clear caveat table for
   criterion 5536.
6. The exact hash/protocol gate expectation and what would falsify it.
7. The test list with **per-test fail-before predictions**, including which existing tests
   are predicted to change and why each change is CR-justified.
8. An explicit list of every pre-survey bullet above that turned out to be **wrong**.
9. A seed list for the out-of-scope items.

## Implementation complete (runner close-out)

**Status: SHIPPED (pending review).** All 14 Category-A sites wired, 12 new tests, all
gates green.

### Change summary

- **Phase 1** (`460e7f4e`) — `effects::condition_is_queue_time_evaluable(&Condition) ->
  bool` (`crates/engine/src/effects/mod.rs`, immediately after `check_condition`):
  exhaustive, no `_` arm, 7 `false` variants (`TargetIsLegal`, `WasOverloaded`,
  `WasBargained`, `WasCleaved`, `EvidenceWasCollected`, `GiftWasGiven`,
  `SacrificeFired`), `Not`/`And`/`Or` propagate conservatively (one unanswerable arm
  makes the whole clause unanswerable), every other variant `true`. And
  `abilities::carddef_intervening_if_holds_at_queue_time(state, intervening_if,
  controller, source) -> bool` (`crates/engine/src/rules/abilities.rs`, immediately
  before `check_intervening_if`): `pub(crate)`, `EffectContext::new_with_kicker` +
  `x_value` via `state.fizzle_object(source)` (SR-25, not a bare lookup), defaults
  `true` on `None` or an unanswerable condition. Wired all 14 Category-A sites exactly
  per the plan's §2 table: `turn_actions.rs` A1 (upkeep, `:267-320`), A2 (first main),
  A3 (postcombat main), A4 (end step), A5 (begin combat) — all five destructures grow
  `intervening_if` and a `let sref: &GameState = state;` rebind per §3.4 (the
  borrow-checker workaround was never actually needed — `cargo check` passed on the
  first try at every site, no restructuring); `mana.rs` A6/A6b (`WhenTappedForMana`,
  one gate shared by the immediate-resolution and stack branches, ahead of the
  `targets.is_empty() && is_mana_producing_effect` split); `replacement.rs` A7 (ETB —
  replaces the inline `if let Some(cond)` with the helper, which also repairs the
  `EffectContext::new` → `new_with_kicker` zero-fill bug the plan's §3.3 predicted) and
  A8 (`TributeNotPaid` — AND'd with the hardcoded `!tribute_was_paid` check, later
  refactored to a match guard for clippy's `collapsible_match`); `abilities.rs` A9
  (`WhenYouCastThisSpell`, with an in-source comment on the stack-object-source
  caveat), A10 (`WhenExertedAsAttacks`), A11 (`WhenDealsCombatDamageToPlayer`), A12
  (`WhenTurnedFaceUp`), A13 (`WheneverRingTemptsYou`), and A14 (the graveyard-zone
  sweep, refactored onto the shared helper, behaviour-neutral). **Not** wired:
  `abilities.rs`'s `WheneverYouSacrifice` `retain` post-filter (Category C,
  index-namespace mismatch — OOS-DP6-2, unchanged). Cleared the two caveats
  (`loyal_apprentice.rs:21-30`, `siege_gang_lieutenant.rs:18-23`) per §5.2's CLEAR row;
  the ~20 "re-checked at resolution" caveats left untouched.
- **Phase 2** (`4bdaecfc`, fixed by `c3ff8038`) — 12 new tests in
  `crates/engine/tests/primitives/pb_dp6_intervening_if_queue_time.rs` (registered in
  `tests/primitives/main.rs`), covering the plan's §7 table T1–T12. Also fixed a
  clippy `collapsible_match` on the A8 site (match guard, no behaviour change) and
  de-staled `pb_ac6_card_integration.rs`'s Land Tax test — its OOS-AC6-2 comment
  ("the stack should stay empty here per CR 603.4 but doesn't") documented exactly the
  gap this PB closes; the comment now says so and the test gained a direct
  `stack_objects().is_empty()` assertion instead of only the observable-outcome
  fallback it had settled for.
- **Fix cycle** (`c3ff8038`, discovered by the runner's own fail-before verification,
  not a review finding) — T4, T5, T7 originally asserted only the **final token
  count**, which the pre-existing, RETAINED resolution-time re-check (CR 603.4's
  second sentence, unmodified by this PB) already drives to zero on a false
  condition — a trigger wrongly queued and then fizzled at resolution is
  observationally identical, at the token-count level, to a trigger never queued at
  all. Running the mandated fail-before revert (see below) surfaced that all three
  **passed pre-fix**, i.e. they were silent-skip tests of exactly the pattern
  `conventions.md`'s "Test-validity MEDIUMs are fix-phase HIGHs" section warns about.
  Fixed by adding a direct `state.stack_objects().is_empty()` assertion immediately at
  the step transition, before any resolution occurs, in all three. Re-verified against
  the reverted pre-fix engine a second time: all three now correctly FAIL pre-fix.

### A second, pre-existing bug found (not fixed here — out of scope)

Building T1 (Nullpriest of Oblivion, kicked ETB) surfaced that **even after the
queue-time fix**, the reanimation effect still does not execute: the
**resolution-time re-check** (`resolution.rs`, the `condition_holds` closure inside
the `is_carddef_etb` `TriggeredAbility` resolution arm) builds its `EffectContext` via
`EffectContext::new(...)` — the exact same zero-fills-`kicker_times_paid` bug the plan
diagnosed and fixed at the ETB **queue-time** gate (§3.3) — so `Condition::WasKicked`
reads false at resolution even for a genuinely-kicked permanent. This is a **different
call site** than anything in the plan's 14-site roster (hard constraint 2 requires the
resolution-time re-check be RETAINED, not touched, and the plan never audited its
internal `EffectContext` construction). Per the standing "implement-phase
default-to-defer" rule (`conventions.md`) — new engine surface beyond the declared
scope gets flagged, not silently fixed — this was **not** fixed in this PB. T1's
assertion was narrowed to match the plan's literal wording (queuing only; see the
in-file `NOTE` on the test) rather than asserting the full reanimation. **New seed for
the coordinator to file**: `resolution.rs`'s `condition_holds` closure (in the
`is_carddef_etb` branch) should build its context the same way the main effect
execution path a few lines below it already does (`EffectContext::new_with_kicker` +
propagated `x_value`), not `EffectContext::new`. Live corpus exposure: at minimum
`nullpriest_of_oblivion.rs` and `thieving_skydiver.rs` (both `WhenEntersBattlefield` +
`Condition::WasKicked`) — neither has ever actually reanimated/drawn in the engine,
queue-time bug or not, until this second bug is also fixed.

### Fail-before / pass-after evidence (OBSERVED, not predicted)

Method: reverted the 7 touched engine/card-def source files
(`crates/engine/src/effects/mod.rs`, `crates/engine/src/rules/{abilities,turn_actions,
mana,replacement}.rs`, `crates/card-defs/src/defs/{loyal_apprentice,
siege_gang_lieutenant}.rs`) to `2deb0402` (the parent commit) via `git checkout
2deb0402 -- <files>`, kept the test file as committed except for two mechanical
compile-only shims needed because the pre-fix engine has no `Condition`-evaluability
API at all: (a) dropped the `condition_is_queue_time_evaluable` import, (b) wrapped
T12 in `#[cfg(any())]` (it is a pure unit test of a function that does not exist
pre-fix — the correct "prediction" for T12 is "does not compile", which is what
happened on the very first attempt before the shim). Ran `cargo test -p mtg-engine
--test primitives pb_dp6`, recorded every pass/fail and panic message, then restored
all 7 files with `git checkout HEAD -- <files>` and confirmed `git diff` was empty
(only `memory/primitive-wip.md`, this close-out edit, showed as modified) before
re-running the full gate suite. This was done **twice** — the first pass caught the
T4/T5/T7 silent-skip bug (see "Fix cycle" above); the table below is from the second,
final pass against the corrected test file.

| # | test | OBSERVED pre-fix behavior | plan's prediction | match? |
|---|---|---|---|---|
| T1 | ETB WasKicked gate | `assertion left == right failed`: `stack_objects().len()` was **0**, expected 1 — the kicked ETB trigger never queued at all | "FAILS before — the kicked case queues nothing today" | ✅ |
| T2 | upkeep false | panics: "equal land counts (condition false) must never queue the upkeep trigger" — trigger was queued (stack non-empty) | "FAILS before — trigger is queued" | ✅ |
| T3 | upkeep true | `ok` | "Passes before and after (non-regression)" | ✅ |
| T4 | end step false | panics: "...must not even be queued (not merely fizzle at resolution)" — trigger was queued pre-fix, then fizzled at the (pre-existing) resolution re-check, landing on 0 tokens either way | "FAILS before" | ✅ (only after the T4 fix — see "Fix cycle"; the original final-token-count-only T4 passed pre-fix, a silent-skip false positive) |
| T5 | begin combat, no commander | panics: "...must not even be queued at the beginning of combat" — same pattern as T4 | "FAILS before — this is the exact divergence loyal_apprentice.rs:23-26 documents" | ✅ (same caveat as T4) |
| T6 | resolution recheck retained | `ok` | "Passes before and after — the pin that hard constraint 2 was honoured" | ✅ |
| T7 | first-main / postcombat-main gates | panics on the first-main assertion: "...must not even queue the first-main-phase trigger" | "FAILS before" | ✅ (same caveat as T4/T5) |
| T8 | unevaluable condition | `ok` | "Passes before (nothing gates) and must still pass after" | ✅ |
| T9 | graveyard gate unchanged | `ok` | "Passes before and after — proves the A14 refactor is behaviour-neutral" | ✅ |
| T10 | Tribute AND-in | panics: "...tribute was not paid but the def's own intervening-if is false -- the trigger must still not queue" — `pending_triggers` contained the source pre-fix | "FAILS before (A8's field is ignored)" | ✅ |
| T11 | face-aware back-face condition | `ok` (both `FrontToken`/`BackToken` counts land at 0, for the pre-existing reason that PB-OS4b/RS4's face selection already reads the back face for the *sweep itself* — the intervening_if gate is simply absent, so the back face's false condition is only caught at resolution) | "Passes before *vacuously* (nothing gates); must pass after" | ✅ |
| T12 | predicate exhaustiveness | **does not compile** — `error[E0432]: unresolved import` `condition_is_queue_time_evaluable` does not exist in `effects` (confirmed before applying the compile-only shim) | n/a in the plan's table (function didn't exist when the plan was written); the only honest fail-before is "cannot compile" | ✅ |

**11 of 12 observed rows matched the plan's literal prediction on the first attempt.**
The one miss was the runner's own construction, not the plan's: **T4, T5, and T7 as
originally written passed pre-fix** (contradicting the plan's "FAILS before" for all
three) because they asserted only the final token count, which the retained
resolution-time re-check already zeroes out regardless of whether the trigger was
queued. This is flagged above as its own fix-cycle entry, not folded silently into the
table, per the task's instruction to say which predictions were wrong. The plan's
*prediction itself* (that these sites over-fire pre-fix) was never wrong — only the
first-draft test's ability to detect it.

### Test counts

- Parent pin (PB-DP5 collect, `2deb0402`): **3,797** passing, 0 failing.
- After PB-DP6 (12 new): **3,809** passing, 0 failing.

### Wire check (read directly from source after the change)

- `crates/engine/src/rules/protocol.rs`: `pub const PROTOCOL_VERSION: u32 = 27;` — unchanged.
- `crates/engine/src/state/hash.rs`: `pub const HASH_SCHEMA_VERSION: u8 = 64;` — unchanged.
- `bare_lookup_ratchet` (`crates/engine/tests/core/`): green with **no** ceiling edits,
  exactly as predicted (§6: "the helper's only lookup is `state.fizzle_object(source)`,
  which the counter does not count as bare").

### Plan deviations

1. **§3.4's borrow-checker workaround was never triggered.** The plan predicted the
   five `turn_actions.rs` closures might fight the borrow checker over `state: &mut
   GameState` and prescribed a `let sref: &GameState = state;` rebind as the fix. The
   rebind was applied preemptively at all five sites and `cargo check` passed
   immediately every time — no actual borrow conflict was ever hit, so it is unclear
   whether the rebind was load-bearing or just harmless-and-consistent. Left in place
   per the plan's explicit instruction (do not restructure into collect-then-check).
2. **A8's `if` was converted to a match guard**, not left as the plan's §3.4 literal
   "keep `if !tribute_was_paid` and AND the helper in" — `cargo clippy`'s
   `collapsible_match` lint (new in this toolchain's lint set, not anticipated by the
   plan) required collapsing the nested `if` into the `match` arm's guard. Behaviour
   is identical (re-verified against T10); the `_ => {}` catch-all arm still receives
   control when the guard is false.
3. **T1, T4, T5, T7 needed a fix cycle** the plan did not anticipate — covered above
   under "Fix cycle" and in the observed-vs-predicted table.
4. **T1's scope was narrowed at the assertion level** (queuing only, not full
   reanimation) because of the second pre-existing bug described above — the plan's
   T1 wording ("Kicked Nullpriest-shaped ETB trigger **is queued**") is honored
   literally; the plan did not ask for a full end-to-end reanimation assertion, and
   attempting one surfaced scope the plan did not authorize fixing.

### Un-enumerated sites hit

None beyond the plan's own inventory. The runner note in the coordinator's brief
(re-derive B1/B2 mechanically via `rg -n "AbilityDefinition::Triggered"
crates/engine/src --glob '!testing/*'`) was followed: the grep returned the same 15
non-`testing` destructure sites the plan's §2 already accounted for (14 Category-A +
1 Category-C), confirming no B1 line needed promotion to Category A.

### Gates (all green)

`cargo build --workspace`, `cargo test --all` (3,809/0, up from 3,797), `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`
(1,804 defs clean), `bare_lookup_ratchet` (no ceiling edits), and a golden-script grep
for all 15 flipping card names (§11 risk 5) — zero hits, no churn to justify.

**0 card-def source edits required by the engine change** — as predicted (§5.1: "Zero
card-def source edits are required by the engine change"); the two caveat edits are
documentation-only.

### Seeds for the coordinator (in addition to the plan's OOS-DP6-1..8)

- **New**: the resolution-time `is_carddef_etb` `condition_holds` closure in
  `resolution.rs` builds `EffectContext::new` instead of `EffectContext::new_with_kicker`,
  so `Condition::WasKicked`/`XValueAtLeast` at resolution read a zero-filled context —
  the same bug class §3.3 fixed at the queue-time ETB gate, at a sibling call site the
  plan did not audit. See "A second, pre-existing bug found" above for the full
  writeup and corpus exposure (`nullpriest_of_oblivion.rs`, `thieving_skydiver.rs`).

## Fix cycle (runner close-out)

**Review**: `memory/primitives/pb-review-DP6.md` (1 HIGH, 2 MEDIUM, 3 LOW). Every
numbered finding's disposition:

| # | Severity | Disposition |
|---|----------|--------------|
| 1 | HIGH | **FIXED HERE.** `resolution.rs`'s `is_carddef_etb` `condition_holds` closure hoisted `kicker_times_paid`/`x_value` into one shared `state.objects.get(&source_object)` read (immediately above the closure) and both the closure's and the effect-execution context's `EffectContext` construction now use `new_with_kicker(...)` + `ctx.x_value = x_value`, mirroring the shape `carddef_intervening_if_holds_at_queue_time` already used. This is the same bug class as §3.3's queue-time fix, at the sibling resolution-time call site the plan never audited. Confirmed the closure is shared by both `is_carddef_etb` and the non-ETB registry-fallback branch, as the review noted (the fix applies to the shared site, so both paths are corrected). T1 restored to its full end-to-end assertion (kicked Nullpriest actually returns "GY Fodder" to the battlefield, not merely queues) and its stale `NOTE` block deleted. |
| 2 | MEDIUM | **DECLINED HERE, PER INSTRUCTION — coordinator's parallel lane.** `docs/audits/decision-point-audit.md` is off limits this cycle; observed it already carries the coordinator's own in-flight edits (OOS-DP6-2..9 rows) at the time this fix cycle started. Not touched, not re-touched. |
| 3 | MEDIUM | **FIXED HERE.** T11 gained a `state.stack_objects().is_empty()` assertion immediately after `advance_to_step(state, Step::Upkeep)` and before `resolve_stack`, mirroring T4/T8's idiom, so the back-token half of the test can no longer be satisfied by "queued, then fizzled at the retained resolution re-check." Re-verified fail-before against the pre-original-PB-DP6 engine (see evidence below): the new assertion FAILS pre-fix, confirming T11 now actually pins the A-site gate rather than passing vacuously. |
| 4 | LOW | **Comment extended, seed left to coordinator.** A9's in-source comment (`abilities.rs:3752` region) now names `WasKicked`/`XValueAtLeast` explicitly, states they read 0 at this site because the object-level fields are written at `resolution.rs:619`/`:628` (not at cast), explains why that is a suppression risk unlike the `SourceOnBattlefield` case the original comment covered, and states zero corpus exposure. Not filed as an audit-doc seed here (out of scope, see finding 2's disposition) — flagged below for the coordinator to file. |
| 5 | LOW | **Note only, per review disposition.** No plan/wip text edited (the plan file is historical once shipped); the correct destructure count (24, not 15) and the `resolution.rs:5351` Haunt gap are reported below for the coordinator to seed — not filed by this runner (audit doc off limits). |
| 6 | LOW | **Note only, code unchanged, per review disposition.** Added one sentence to the `Condition::And` arm's comment in `condition_is_queue_time_evaluable` (`effects/mod.rs`) acknowledging the conservatism is deliberate (an evaluable-false first arm could safely suppress, but the current shape errs toward over-firing, which is the safe direction, and costs nothing today). No logic change — did **not** short-circuit into the unsafe direction. |

**Items explicitly NOT touched here (coordinator's parallel lane, per instruction):**

- `docs/audits/decision-point-audit.md` §4.8/§5/§8 rows and the OOS-DP6-1..9 seed
  filing in §8.1 — observed already in flight on this branch's working tree when this
  fix cycle began (7 seed rows present: OOS-DP6-2..9, i.e. finding 2's seed list plus a
  9th, OOS-DP6-9, that already captures the Haunt/`resolution.rs:5351` gap and the
  "24 not 15" destructure correction from finding 5). Not re-verified in detail by this
  runner (out of scope by instruction) beyond confirming the file was not further
  modified by this fix cycle.
- `CLAUDE.md` — untouched.

**New seeds for the coordinator, beyond what's already in the audit doc's working
tree** (report only, not filed by this runner):

- Finding 4 (A9 `WasKicked`/`XValueAtLeast` false-negative risk at the
  `WhenYouCastThisSpell` cast-trigger site) does not yet have a dedicated OOS-DP6 row
  distinct from OOS-DP6-6 (which covers a different set of variants — `WasBargained`/
  `EvidenceWasCollected`/`GiftWasGiven`/`WasOverloaded`/`WasCleaved` — not `WasKicked`/
  `XValueAtLeast`). Worth its own row: real fix is either `StackObject.kicker_times_paid`/
  `x_value` or writing those fields onto the spell's `GameObject` at cast time.

**Aurelia probe (OOS-DP6-1 lowering-drop claim) — disposition: NOT run, disclosed
plainly.** This fix cycle did not execute the plan §1 / review finding-2(e) throwaway
probe (Aurelia-shaped def, extra combat phase, assert the untap/token fires when it
must not). The seed as filed rests on source reading only: `build_face_ability_vectors`
is confirmed (by both the original runner and independently by the reviewer) to be
called from `rules/resolution.rs:720` and `rules/face.rs:104` on the live
permanent-creation path, and it hardcodes `intervening_if: None` at all 34 push sites —
but no test was executed in this cycle to observe the predicted over-fire in a running
game. This is out of scope for the fix cycle's HIGH/MEDIUM work list; recorded here so
the claim does not silently become "confirmed by execution" when it is not.

### Fail-before / pass-after evidence (this fix cycle)

Method for both rows: reverted only the file(s) needed to isolate the claim, kept the
edited test file in place, ran the specific test, then restored via `git checkout -- `
and confirmed `git status`/`git diff --stat` matched the pre-experiment state before
resuming (the A9/And comment edits in `abilities.rs`/`effects/mod.rs` were caught and
reapplied after being clobbered by the second experiment's restore step — see below).

| test | method | OBSERVED pre-fix result | match? |
|---|---|---|---|
| T1 (restored full assertion) | Stashed only `resolution.rs` (back to this branch's pre-fix-cycle HEAD, `ba035bcb`); kept the fixed test file. | `assertion failed` at the new "GY Fodder … battlefield" assertion — the trigger queued (original PB-DP6 gate already fixed that) but the reanimation effect never executed, exactly finding 1's predicted symptom. | ✅ |
| T1 (after restoring the fix) | Restored `resolution.rs` via `git stash pop`. | `ok` — the full end-to-end reanimation now passes. | ✅ |
| T11 (new pre-resolution assertion) | `git checkout 2deb0402 --` on the 5 original-PB-DP6 engine files + 2 card defs (the same method the original close-out used), kept the edited test file with two compile-only shims (dropped the `condition_is_queue_time_evaluable` import, `#[cfg(any())]`'d T12 — neither function exists pre-PB-DP6). | `panicked` at the new `state.stack_objects().is_empty()` assertion — the back-face trigger *was* on the stack pre-fix (no queue-time gate existed at all), confirming T11 now has real fail-before signal instead of the vacuous pass the review found. | ✅ |
| T11 (after restoring) | `git checkout HEAD --` on the same 7 files, restored the un-shimmed test file from a scratchpad copy. | `ok` — all 12 tests pass. | ✅ |

**Bookkeeping hazard hit and corrected during this cycle**: the `git checkout HEAD --`
restore step for the T11 experiment also reverted `abilities.rs` and `effects/mod.rs`
to their pre-this-fix-cycle state, silently discarding the finding-4 (A9 comment) and
finding-6 (`And` comment) edits made earlier in this same session. Caught by re-reading
the files before running the full test suite; both edits were reapplied and verified
present in the final diff.

### Test counts

- Before this fix cycle (PB-DP6 implement, `ba035bcb`): 3,809 passing, 0 failing.
- After this fix cycle: **3,809 passing, 0 failing** (no new `#[test]` functions —
  T1 and T11 were strengthened in place, not added to).

### Gates (all green, this fix cycle)

- `cargo build --workspace` — clean.
- `cargo test --all` — 3,809/0, all 30 test binaries green (including golden scripts:
  `cargo test -p mtg-engine --test scripts` — 43/43, no card-name churn).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `tools/check-defs-fmt.sh` — 1,804 defs clean.
- `bare_lookup_ratchet` — **moved**, as the HIGH fix predicted it might: merging the
  closure's and the effect-execution context's separate `state.objects.get(&source_object)`
  reads into one shared read took `src/rules/resolution.rs` from 101 → **100** bare
  lookups. Ceiling lowered to 100 in `crates/engine/tests/core/bare_lookup_ratchet.rs`
  with a dated comment, per the ratchet's own instruction ("good, you converted some —
  lower the ceiling to lock in the gain"). All other four ceilings
  (`effects/mod.rs` 110, `abilities.rs` 75, `replacement.rs` 24, `turn_actions.rs` 7,
  `mana.rs` 8) unmoved.
- Wire sentinels, read directly from source after all fixes:
  `crates/engine/src/rules/protocol.rs:260` → `PROTOCOL_VERSION: u32 = 27` (unchanged).
  `crates/engine/src/state/hash.rs:591` → `HASH_SCHEMA_VERSION: u8 = 64` (unchanged).
  `protocol_schema`/`state_hashing` gate tests both green with no hand edit.
