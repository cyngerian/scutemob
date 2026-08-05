# Primitive Batch Review: PB-DX21 — CR 508.1: attackers may be declared without limit (`OOS-M11-9`)

**Date**: 2026-08-04
**Reviewer**: primitive-impl-reviewer (Opus)
**Task / branch**: `scutemob-200` · `feat/pb-dx21-cr-5081-attackers-may-be-declared-without-limit-oos-`
**CR Rules verified independently via MCP**: 508.1 (+ 508.1a–508.1m), 508.2, 508.3 (+ 508.3a–508.3f),
508.4, 508.4c, 508.6, 508.8, 506.4 (+ 506.4a–506.4e), 500.8/506.5, 509.1a, 117.4, 732
**Card oracle text verified via MCP**: Windbrisk Heights (incl. ruling 2007-10-01), Legion's Landing //
Adanto, the First Fort (incl. rulings 2017-09-29)

**Engine files reviewed**
- `crates/card-types/src/state/combat.rs` (new field + `new()`)
- `crates/engine/src/state/error.rs` (new variant)
- `crates/engine/src/rules/combat.rs` (guard `:69-76`, marker `:751-764`, re-scoped comment `:765-787`)
- `crates/engine/src/state/hash.rs` (`:743-757` History, `:1164-1180` appended epoch, `:4461-4462` feed)
- `crates/engine/tests/core/hash_schema.rs` (`FROZEN_HISTORY_PREFIX_DIGEST` re-pin `:200-203`)
- `crates/simulator/src/legal_actions.rs` (`:876-891` offer suppression)
- `crates/simulator/src/heuristic_bot.rs` (`RepeatKey::DeclareAttackers` deletion + doc rewrite)

**Test files reviewed**
- `crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_combat.rs` (T1–T8, new)
- `crates/engine/tests/primitives/main.rs` (SR-9a registration, lexicographic position correct)
- `crates/engine/tests/rules/static_grants.rs`, `primitive_pb_xa.rs`, `primitive_pb_xa2.rs` (5 literal sites)
- `crates/simulator/tests/local_game_playthrough.rs` (`PolicyState` deletion)
- `crates/simulator/tests/local_game_human_actions.rs` (MR-M11-09 rewrite + new offer probe)

**Card defs reviewed (2, comment-only)**
- `crates/card-defs/src/defs/windbrisk_heights.rs`
- `crates/card-defs/src/defs/legions_landing.rs`

**Docs reviewed**: `docs/mtg-engine-simulator.md`, `tools/play-server/src/api.rs`,
`tools/play-server/README.md`, `docs/audits/decision-point-audit.md`,
`memory/primitives/seed-rerank-2026-08-02.md`

> **Method note / limitation**: this reviewer has **no shell**. Everything below is derived by reading
> source, the CR (MCP) and card oracle text (MCP). No test was executed and no revert was reproduced.
> Claims about test *counts* (4,397/0/5), gate execution (HASH 73, PROTOCOL 35) and A/B measurements
> are taken on the runner's word except where the tree itself contradicts them, which is called out.

---

## Verdict: needs-fix

**The primitive itself is correct and I could not break it.** Re-derived from CR text rather than from
the plan: CR 508.1 ("**First**, the active player declares attackers") followed by CR 508.2
("**Second**, the active player gets priority") makes the declaration a single ordered turn-based
action of the step, and CR 508.1d's "during **each declare attackers step**" plus CR 508.7a's "it
isn't considered to have attacked a second time" both presuppose exactly one declaration per step —
so rejecting a second one is CR-mandated, not policy. The plan's decisive §1.3 verdict is **confirmed
independently**: CR 508.1a's "*if any*" makes the empty choice a completed action, and CR 508.4's
"put onto the battlefield attacking" creatures — which `effects/mod.rs:1502-1504`,
`effects/mod.rs:6331`, `resolution.rs:6020` and `resolution.rs:6480` all insert straight into
`combat.attackers` without ever calling `handle_declare_attackers` — would have made an
`attackers`-keyed guard refuse a player's **first legal** declaration. A real marker field was
required. Guard placement, marker placement, hash feed, history append, frozen-prefix re-pin, the
five struct-literal sites, the error variant's Invariant-7 shape and the offer suppression all check
out under adversarial reading, and I found **no path that keeps a `CombatState` alive across a combat
boundary** (`end_combat` at `turn_actions.rs:2507` is the EndOfCombat step's own turn-based action;
`turn_structure.rs` routes every extra combat through `EndOfCombat → BeginningOfCombat`; the concede
path nulls it at `engine.rs:2962`; there is no "end the turn" effect in the tree), so the MR-M11-09
sticky-marker class does not reproduce.

**0 HIGH.** The findings are: one live CR 508.8 residue the batch's own field doc points at but does
not close (M1); one missing probe on the single hardest path the plan itself flagged as risk 4 (M2);
one card-def comment that asserts a defect the card does not have and mis-cites CR 508.6 for it, which
would send the successor batch into a regression (M3); a cluster of test-doc / test-validity issues
where four probes' "pre-fix behaviour" claims describe state the probes structurally cannot observe,
one of them fully vacuous (M4, M5); and the close-out artefacts the plan mandated — seed filing,
`OOS-M11-9` closure, the queue-row banner and line-899 refutation, and the Stage-4 seeded-pin
classification — are absent from the tree (M6, M7).

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| M1 | MEDIUM | `rules/turn_structure.rs:43-51` | **CR 508.8's skip predicate is still a step-end read of `combat.attackers`, and the new field's doc points at it.** Fix: file the residue as a seed with a wrong-way-round pin, and correct the field doc so it does not imply the CR 508.8 site consumes the marker. |
| M2 | MEDIUM | `rules/combat.rs:763` vs `:837-840` | **No probe covers the CR 603.3d suspended-trigger path** — the plan's own §10 risk 4. Fix: add the T2 sub-case the plan suggested, using `pb_dp8_trigger_target_choice.rs` T29's fixture. |
| L1 | LOW | `rules/combat.rs:751-764` | **Marker set inside a fallible `if let Some(combat)`** with no SR-4 diagnostic. Fix: use the `expect_*` idiom or add a `debug_assert`. |
| L2 | LOW | `rules/combat.rs:78-80` | A **non-guard** rejected declaration still installs a `CombatState` before validation (pre-existing; invisible through `process_command`, visible to the direct-handler callers this batch added). Fix: note only, or move the init below the validation loop in a successor. |
| L3 | LOW | sentinel sweep | Tree shows **45** `HASH_SCHEMA_VERSION, 73` across **44** files; plan §6.1 predicted 44/43 and required the runner to report the re-derived count. Fix: report the number and account for the extra site. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| M3 | MEDIUM | `legions_landing.rs:73-81` | **The new comment asserts a residual the card does not have, and cites CR 508.6 for a claim CR 508.6 does not make.** Following it would regress the card. Fix: rewrite the note and re-scope `OOS-DX21-1` to Windbrisk Heights only, with an explicit "do not migrate Legion's Landing" warning. |
| — | none | `windbrisk_heights.rs:7-28` | Comment is **accurate** and matches ruling 2007-10-01 verbatim in substance. 0 completeness flips (no `completeness` field → `Complete` by derive, unchanged). |

## Test / Doc Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| M4 | MEDIUM | `pb_dx21_...rs:144-148, 227, 279-282, 311-313` | **Four "pre-fix behaviour" doc claims describe state the probes cannot observe** — the second declaration is issued on a `state.clone()` and `expect_err` panics pre-fix. T6's own doc proves the runner knew this. Fix: reword as positive controls, or re-express through T6's direct-handler idiom. |
| M5 | MEDIUM | `pb_dx21_...rs:449-494` | **T4 step (4) is structurally vacuous** — the CR 117.4 half of the seed is unpinned. Fix: express it through `handle_declare_attackers(&mut state, ..)`. |
| M6 | MEDIUM | `docs/audits/decision-point-audit.md`, `memory/primitives/seed-rerank-2026-08-02.md:873-907` | **Close-out artefacts absent**: no `OOS-DX21-*` rows, no `OOS-M11-9` CLOSED marking, no SHIPPED banner, and line 899's refuted "PREFER reading `combat.attackers`" is uncorrected. Fix: do all four. |
| M7 | MEDIUM | seeded fuzz fixtures | **Stage 4's mandatory seeded-pin classification has no record anywhere in the tree.** Fix: record the measured before/after at each named pin, even when the answer is "unmoved". |
| L4 | LOW | `tools/play-server/src/api.rs:308-316` + `pb_dx21_...rs:18-21` | **Dangling self-reference and a false quotation** — the word *irreversible* is not in `api.rs`; it is at `README.md:297`, which got no PB-DX21 note. Fix: move/retarget the note and correct the test header. |
| L5 | LOW | `pb_dx21_...rs:510-573` (T5) | **T5 does not redden under the revert §4 prescribes for T1–T6.** Fix: state T5's real falsifier. |
| L6 | LOW | — | **Revert failure texts are recorded in no tracked artefact** (§9 requires them verbatim; `memory/primitive-wip.md` is still PB-DX32's). Fix: add a PB-DX21 execution-notes file or point at the commit messages. |
| L7 | LOW | `docs/mtg-engine-simulator.md:358-359` | `OOS-DX21-2` mis-attributed to "each defending player may still declare independently"; the seed is about the **offer layer** not suppressing `DeclareBlockers`. Fix: reword. |
| L8 | LOW | `crates/simulator/tests/local_game_playthrough.rs:299-302` | Duplicated `record_journal` comment ("5,000-command" then "1,000-command"). Probably pre-existing; adjacent to the `PolicyState` deletion. Fix: delete one. |

---

## Finding Details

### M1 — CR 508.8's skip is still keyed on `combat.attackers`, and the new field's doc points at it

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_structure.rs:43-51`; doc claim at
`crates/card-types/src/state/combat.rs:51-53`
**CR Rule**: 508.8 — "If **no creatures are declared as attackers** or put onto the battlefield
attacking, skip the declare blockers and combat damage steps." CR 506.4 — a creature is removed from
combat if it leaves the battlefield, changes controller, phases out, stops being a creature, etc.

**Issue**: `advance_step` computes

```rust
let no_attackers = state.combat.as_ref().map(|c| c.attackers.is_empty()).unwrap_or(true);
...
let next = if turn.step == Step::DeclareAttackers && no_attackers { Step::EndOfCombat } else { ... }
```

CR 508.8 is a **declaration-time** predicate; this is a **step-end** read. Declare one attacker, then
kill it (or phase it out, or Turn to Frog it) with an instant while the declare-attackers step is
still open: `remove_from_combat` (`combat.rs:1063`) empties `combat.attackers`, `no_attackers` becomes
`true`, and the engine skips the declare-blockers **and combat-damage** steps in a combat where
creatures *were* declared. That skips other creatures' blocks and any "put onto the battlefield
attacking" entrant that arrives later, and it skips CR 510 entirely.

This is **pre-existing**, not introduced by PB-DX21 — but PB-DX21 is the batch that added the state
which distinguishes the two readings, and the new field's own doc says, verbatim, that the marker is
`true` on an empty declaration because "CR 508.8 defines the game's own downstream behaviour for it".
A reader will conclude the CR 508.8 site consumes the marker. It does not. Note also that the naive
"fix" (`!attackers_declared`) is **wrong** — an empty declaration sets the marker and CR 508.8 still
demands the skip — so this needs a third piece of state (the size of the declaration), which is why it
is a seed and not a one-liner.

**Fix**: File it (`OOS-DX21-N`) with the wrong-way-round pin naming `turn_structure.rs:43-51`, CR
508.8 and CR 506.4, and stating that the correct predicate is "was the declaration empty **at
declaration time**, and has nothing been put onto the battlefield attacking since". Amend
`combat.rs:51-53`'s doc so the CR 508.8 sentence reads as motivation for the marker's *value*, not as
a claim that the skip site reads it.

### M2 — no probe covers the CR 603.3d suspended-trigger path

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/combat.rs:763` (marker) vs `:837-840` (early return)
**CR Rule**: 603.3d / 603.3b

**Issue**: The placement is **correct** — I traced it: the last `return Err` in the function is the
exert check at `:671`, the marker lands at `:763`, and the suspended-batch early return
(`if state.pending_trigger_targets.is_some() { mark_flush_resume_site(..); return Ok(events) }`) is at
`:837-840`, so a declaration that suspends on a trigger-target choice is already marked. But **no test
observes this**. Move the assignment to between `:840` and `:844` and every one of T1–T8 stays green,
because none of their fixtures suspends: T1/T3/T4/T6 use vanilla or non-targeting creatures, T2's
Nadaar venture is explicitly documented as choice-free, T5's Aurelia trigger is choice-free, T7 is a
unit hash test and T8 is the blockers side. The plan itself named this as §10 risk 4 ("the hardest
variant to notice") and §9's checklist has an explicit line for it; the code satisfies the checklist,
nothing pins it.

**Fix**: Add a T2 sub-case (or T9) on the `pb_dp8_trigger_target_choice.rs:2061-2132` fixture — which
already stages exactly a `handle_declare_attackers` flush that suspends and owes a priority grant —
asserting that a second `Command::DeclareAttackers` submitted while `pending_trigger_targets` is
`Some` is `Err(AlreadyDeclaredAttackers)` (or, if the PB-DP7 `BlockedByPendingDecision` gate fires
first, that the marker is already `true` on the suspended state). Prove it red by moving the
assignment below `:840`.

### M3 — `legions_landing.rs`'s new comment asserts a residual the card does not have

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/legions_landing.rs:73-81`
**Oracle**: "Whenever you attack with three or more creatures, transform Legion's Landing."
**Rulings (MCP, 2017-09-29)**: "The last ability of Legion's Landing only counts creatures that **you
declare as attacking creatures**. Creatures that enter the battlefield attacking won't count." /
"**Once you've attacked with three or more creatures**, Legion's Landing will transform even if some
of those creatures leave the battlefield or are removed from combat."
**CR Rule**: 508.3d — "An ability that reads 'Whenever [a player] attacks, …' triggers if one or more
creatures that player controls **are declared as attackers**." CR 508.6 — "A player *has* 'attacked
[a player]' if the first player declared one or more creatures as attackers attacking that player."

**Issue**: The new note says:

> so attacking with 1 creature in combat 2 after 3 in combat 1 overwrites the count to 1 and this
> trigger evaluates false, even though **CR 508.6's "attacked with three or more creatures" is a
> whole-turn count across every combat phase**.

Both halves are wrong.

1. **CR 508.6 says nothing about counts or turns.** It defines the predicate "*has attacked [a
   player]*" — a per-player boolean about declaration, with no numeric or turn-scope content. It is
   the wrong citation for this claim.
2. **Legion's Landing is a per-declaration trigger, not a turn-scoped one.** It is a
   `WheneverYouAttack` trigger in the CR 508.3d family: it fires on a declaration and counts the
   creatures declared **in that declaration**. Attacking with 1 creature in combat 2 after 3 in combat
   1 *should* evaluate false in combat 2 — a second combat is a second, separate attack. The engine's
   assign-per-declaration `attackers_declared_this_turn` is therefore **correct** for this card.

Windbrisk Heights is the genuinely turn-scoped one and its own comment gets it right, matching ruling
2007-10-01 exactly ("if you declared three different creatures as attackers **at any point in the
turn**. A creature declared as an attacker in two different attack phases counts only once.").

The concrete hazard: `OOS-DX21-1` as filed in the plan §8 tells the successor to make
`attackers_declared_this_turn` "a per-turn accumulation with per-creature dedup", and this comment
names Legion's Landing as a second member of that class. Doing so would **regress** Legion's Landing —
attacking with 3, then 1, then 1 would transform on each of the second and third combats. The one
field is currently serving two different semantics (per-declaration for CR 508.3d triggers,
per-turn-deduped for Windbrisk Heights' "this turn" condition), and the successor needs **two**
predicates, not one migrated field.

**Fix**: Rewrite `legions_landing.rs:73-81` to state that the per-declaration count is *correct* for
this card (cite ruling 2017-09-29 and CR 508.3d, drop the CR 508.6 cite), and that it is deliberately
**not** a member of `OOS-DX21-1`. Re-scope `OOS-DX21-1` to `windbrisk_heights` alone and add the
warning that closing it requires a second, per-declaration reader for the CR 508.3d family rather than
a migration of the existing field.

### M4 — four probes' "pre-fix behaviour" docs describe state the probes cannot observe

**Severity**: MEDIUM (fix-phase HIGH under `memory/conventions.md:173-186`, "test-validity MEDIUMs are
fix-phase HIGHs")
**File**: `crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_combat.rs:144-148` (T1),
`:227` + `:279-282` + `:293-298` (T2), `:311-313` (T3), `:418-421` (T4)

**Issue**: Every one of T1–T4 issues the second declaration as

```rust
let err = process_command(state.clone(), declare_cmd(..)).expect_err(..);
```

`process_command` takes `GameState` **by value** and returns no `GameState` on `Err`, so the clone is
discarded whatever happens — and pre-fix `expect_err` panics on the `Ok`, so **every assertion after
it is unreachable**. The probes are genuinely discriminating (the `expect_err` is the discriminator),
but their doc comments claim more:

- T1: "Pre-fix behaviour … the second command returned `Ok`, and `combat.attackers.insert(..)`
  **OVERWROTE Samut's target from p2 to p3**". The overwrite happens in the *discarded* clone; the
  probe's assertion (3) reads the untouched original and passes for any engine.
- T2 `:293-298`: "the `WhenAttacks` trigger must fire exactly once … **pre-fix, a second (accepted)
  declaration re-fired it, producing two `VenturedIntoDungeon` events**". It would not — pre-fix the
  second declaration's events go with the discarded clone, so `ventured_count` is 1 either way. The
  assertion is a positive control on a single declaration, not evidence about the re-fire.
- T3 `:311-313` and its assertion (4) (Windbrisk activation): same shape.
- T4 `:418-421`: see M5.

T6's own doc (`:586-602`) records this exact discovery — *"ANY mutation performed before an `Err`
return is UNCONDITIONALLY discarded by Rust's ownership model … proven, not presumed"* — so the runner
knew. The other four docs were not brought into line, which is the `memory/conventions.md:216-230`
aspirational-comment class inside the batch's own new file.

**Fix**: Reword T1's, T2's and T3's "pre-fix behaviour" paragraphs to say the consequence assertions
are **positive controls** on the accepted first declaration (and that the discriminator is the
`expect_err`), or re-express them through T6's direct `handle_declare_attackers(&mut state, ..)` idiom
so they really observe the pre-fix consequence. Do not simply delete them.

### M5 — T4 step (4) is structurally vacuous; the CR 117.4 consequence is unpinned

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_combat.rs:449-494`
**CR Rule**: 117.4; engine site `combat.rs:844` (`state.turn.players_passed = OrdSet::new()`)

**Issue**: The stage-0 memo's "**A FOURTH consequence**" — a rejected re-declaration must not reset the
CR 117.4 pass-round — is the plan's §4 T4 step 4 and is currently pinned by nothing. The test does:

```rust
state.turn_mut().players_passed.insert(p2);
let passed_before_reject = state.turn().players_passed.clone();
let err = process_command(state.clone(), declare_cmd(..)).expect_err(..);
assert_eq!(state.turn().players_passed, passed_before_reject, "...");
```

`state` was never passed to the failing call — a clone was — so `state.turn().players_passed` is
trivially equal to a clone of itself taken two lines earlier. The assertion holds under **any** engine
behaviour, including one where `handle_declare_attackers` resets `players_passed` before rejecting.
This is a test that cannot fail.

**Fix**: Rewrite step (4) using the direct-handler idiom T6 already establishes: seed
`players_passed`, call `mtg_engine::rules::combat::handle_declare_attackers(&mut state, ..)` on the
**same** instance, assert `Err(AlreadyDeclaredAttackers)` **and** that `state.turn().players_passed`
is unchanged. Prove it red by moving the `players_passed = OrdSet::new()` assignment above the guard.

### M6 — close-out artefacts mandated by plan §7.3/§8/§9 are absent from the tree

**Severity**: MEDIUM
**Files**: `docs/audits/decision-point-audit.md` §8.1;
`memory/primitives/seed-rerank-2026-08-02.md:873-907`

**Issue**: Grepped the whole tree: **zero** occurrences of `OOS-DX21-1` … `OOS-DX21-7` in
`docs/audits/decision-point-audit.md`, and `OOS-M11-9` appears there only inside a 2026-08-02
re-rank banner (`:944-953`), not as a row marked CLOSED. `seed-rerank-2026-08-02.md`'s PB-DX21 row
(`:873-907`) carries no `✅ SHIPPED` banner — unlike its PB-DX22 neighbour at `:911-916` — and line
`:899`'s "**Prefer reading `combat.attackers` over adding a field**" is still standing uncorrected.
Plan §7.3 makes the in-place refutation mandatory ("do not delete it") precisely because a stale brief
line is the re-dispatch hazard this queue keeps re-filing, and §9 lists both closures as checklist
items. `docs/mtg-engine-simulator.md:350-364` **was** updated and is accurate; the audit doc and the
queue memo were not.

If this is intended as Stage-6 work, say so at the pin — but the row's brief is exactly the artefact a
future dispatcher will read.

**Fix**: File `OOS-DX21-1..N` (including M1's CR 508.8 residue and M3's re-scoping) in
`docs/audits/decision-point-audit.md` §8.1; mark `OOS-M11-9` CLOSED there with the merge SHA; banner
the PB-DX21 queue row SHIPPED; correct `:899` in place with the §1.3 refutation.

### M7 — Stage 4's seeded-pin classification has no record, and the suppression is likely to move seeds

**Severity**: MEDIUM
**Files**: `crates/simulator/tests/pb_dx32_fuzz_output.rs` (T2.2/T3.1/T4.1/T4.3/T6.3),
`crates/simulator/tests/sim5_bot_cast_discipline.rs` (T3.3),
`crates/simulator/tests/pb_dx22_fuzz_instrument.rs`, `tools/play-server/src/main.rs` seeded probes

**Issue**: Plan §2.7 says the blast radius must be **measured, not predicted**, and §5 Stage 4 says
"Triage **every** new failure … Never re-pin without classifying" and "Record the before/after of any
pin that moves". Grepping `PB-DX21|DX21` across `crates/simulator/` returns **six source sites and
three test sites, none of them a seeded fixture** — no pin carries a PB-DX21 note and no value shows a
re-measurement.

That silence is surprising rather than reassuring. `RandomBot::choose_action`
(`random_bot.rs:38-64`) has an 80% attack bias whose subset can be **0-count** (`:174`,
`rng.random_range(0..=eligible.len())`), i.e. a legal empty declaration that now permanently retires
the `DeclareAttackers` offer for that combat; the remaining 20% path picks **uniformly by index** from
`legal`, so removing one element re-indexes every subsequent choice and diverges the whole trajectory.
T4.1 pins an exact transient-report count on a specific seed, T4.3 has a real-seeded half, and T6.3
asserts an **exact** reached/never-reached partition — all three are exactly the shape that moves.

Either they genuinely did not move (which deserves recording, because it is evidence about how often
that window is reached) or they moved and were re-pinned silently.

**Fix**: Run each named fixture and record the measured before/after **at the pin**, including
"unmoved" results. If any moved, add the PB-DX21 reason at the constant, per `MOVED_MSG`'s own
instruction.

### L1 — the marker set is a silent no-op if `state.combat` is `None`

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:751-764`
**Invariant**: SR-4 (silent failures in `effects/mod.rs` + `rules/*` are classified LKI-fizzle vs
engine-bug; new code must pick a side)

**Issue**: `combat.attackers_declared = true` lives inside `if let Some(combat) = state.combat.as_mut()`.
It is provably `Some` (`:78-80` installs it and nothing between `:80` and `:751` clears it — I checked
every `state.combat =` site in the crate: `turn_actions.rs:1897`, `turn_actions.rs:2507`,
`combat.rs:79`, `engine.rs:2962`). But if that ever changes, the declaration **succeeds unmarked** and
is re-declarable — the exact defect this batch closes, reintroduced silently. This is the engine-bug
side of SR-4, not the LKI-fizzle side.

**Fix**: Either use the `state::diagnostics` `expect_*` idiom, or add a `debug_assert!(state.combat.is_some())`
before the block with the CR cite. (The sibling `attackers.insert` loop has the same shape and the same
argument; a one-line assertion covers both.)

### L2 — a rejected (non-guard) declaration still installs a `CombatState`

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:78-80`

**Issue**: The plan's §2.3 table claims the guard is placed "**Before** `state.combat = Some(CombatState::new(player))` … that assignment is a state mutation. Guarding after it would install a `CombatState` as a side effect of a rejected command." True of the guard's own rejection. But every *other* rejection (illegal attacker at `:88`–`:196`, tax at `:291`/`:328`/`:356`, enlist at `:546`–`:628`, exert at `:646`–`:671`) still happens **after** `:78-80`, so a rejected declaration does install a `CombatState`. Invisible through `process_command` (which drops the moved state on `Err`) but visible to the two direct-handler callers this batch now has (`pb_dx21_...rs:638`, `pb_dx22_fuzz_instrument.rs:886`) — and T6's own step (2) has to assert a *disjunction* because of it. Pre-existing.

**Fix**: Note only, or move the init below the validation loop in a successor batch.

### L4 — dangling self-reference in `api.rs` and a false quotation in the test header

**Severity**: LOW
**Files**: `tools/play-server/src/api.rs:308-316`;
`crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_combat.rs:18-21`;
`tools/play-server/README.md:294-303`

**Issue**: `api.rs:308` opens "**The word *irreversible* above** was aspirational until PB-DX21".
Grepping the tree: the word does not appear in `api.rs` at all. It is at
`tools/play-server/README.md:297` ("a legal, **irreversible** 'I attack with nothing' for that
combat"), which received no PB-DX21 note. The plan §7.3 row and the stage-0 memo both mis-cited it as
`api.rs:298-306`, and the new test module header propagates the error as a **verbatim quotation** —
`"(params.rs:474/api.rs:298-306, \"a legal, irreversible 'I attack with nothing'\")"` — of text that
is not there. Also, `README.md:299-302`'s "the buttons stay **enabled** (at a `DeclareAttackers`
decision the declaration is usually the only option offered, so disabling it would deadlock the game)"
is partially falsified: the offer now disappears after the declaration, and it does not deadlock
because `legal_actions.rs:515` pushes `PassPriority` unconditionally.

**Fix**: Put the "now true" note at `README.md:294-303` where the word lives, re-scope its
button-enablement paragraph, and correct both the `api.rs` paragraph ("the README's word *irreversible*")
and the test header's citation.

### L5 — T5 has no falsifier under the revert the plan prescribes

**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_combat.rs:510-573`

**Issue**: Plan §4 says "The revert for T1–T6 is *delete the guard block from `combat.rs`*". Deleting
the guard leaves T5 **entirely green**: it asserts (a) combat 1's declaration is `Ok`, (b) the marker
is set, (c) `in_extra_combat`, (d) the extra combat's marker is `false`, (e) combat 2's declaration is
`Ok`. None of the five changes when the guard is removed. T5 is a *marker-scope* probe (it would redden
if `begin_combat` reused a stale `CombatState`, or if the marker were stored per turn), not a guard
probe. That is fine as a test; what is missing is the statement of which revert reddens it.

**Fix**: Record T5's real falsifier in its doc (e.g. change `begin_combat`'s `if state.combat.is_none()`
to unconditionally preserve, or move the marker to `PlayerState`), and note that the §4 T1–T6 revert
leaves it green.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 508.1 (once-per-combat turn-based action) | Yes — `combat.rs:69-76` | Yes — T1, T2, T3, T4 | Verified independently from CR text; the 508.1/508.2 "First/Second" ordering is the warrant |
| 508.1a (the empty choice is a choice) | Yes — marker set on the empty path (`:751-764`) | Yes — T4 steps (1)-(3) | The decisive verdict; independently confirmed |
| 508.1f / 508.1h / 508.1j (tap, total, pay) | Untouched, guard precedes all three | Yes — T6 | Guard at `:74`, tapping at `:682`, tax debit at `:721-749` — order verified by reading |
| 508.2a / 508.3a–508.3e (triggers fire once) | Yes, by consequence of the guard | Partial — T2 (positive control only; see M4) | The re-fire consequence is proven by the `expect_err`, not by the venture count |
| 508.4 / 508.4c ("never attacked") | Yes — marker untouched by all 4 direct-insert sites | **No** | `effects/mod.rs:1502`, `:6331`, `resolution.rs:6020`, `:6480` verified by reading; no probe declares after a CR 508.4 entrant |
| 508.6 | N/A | N/A | Mis-cited in `legions_landing.rs` — see M3 |
| 508.8 | **Partial / pre-existing deviation** | No | See M1 |
| 500.8 / 506.5 (per combat phase) | Yes — `CombatState` lifetime | Yes — T5 | `end_combat` / `begin_combat` / `advance_step` all traced; no sticky-marker path found |
| 506.4 (removed from combat) | Unaffected by the guard | No | Feeds M1 |
| 509.1a (blockers side untouched) | Yes | Yes — T8 + `combat/combat.rs:1701` | Deliberately not widened (`OOS-DX21-2`) |
| 603.3d (suspended batch) | Yes — marker precedes `:837-840` | **No** | See M2 |
| 117.4 (pass-round reset) | Yes — `:844` on success path only | **No** (T4 step 4 vacuous) | See M5 |
| 732 (illegal action, no state change) | Yes for the guard's own rejection | Yes — T1 assertion (3) as positive control | Not for other rejections; see L2 |

**Coverage gap worth naming**: nothing declares attackers in a combat where a CR 508.4 entrant already
populated `combat.attackers`. That is the exact scenario the plan's §1.3(3) uses to refute the brief's
`attackers`-keyed guard, and the tree contains no test for it. Not a defect (the marker is
structurally untouched by all four insert sites, which I verified by reading each), but the batch's
single strongest architectural argument is unpinned. Recommend a cheap T9 on `enters_attacking` +
declaration.

---

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `windbrisk_heights` | Yes (verified vs MCP oracle + ruling 2007-10-01) | 0 | Yes (unchanged; residual accurately stated) | Comment-only; `completeness` unchanged (derive → `Complete`) |
| `legions_landing` | Yes (def unchanged) | 0 | Yes | Comment-only; the **comment** is wrong — see M3. `completeness: Complete` explicit, unchanged |

Coverage impact: **0 flips**, consistent with the plan's pre-commitment. Both edits are inside `//`
comment blocks; neither touches `completeness`, `abilities`, `types`, `mana_cost` or `oracle_text`.
(`tools/check-defs-fmt.sh` is the gate that matters here, per SR-35 — not executed by this reviewer.)

---

## Items verified clean (explicitly, so the absence is not mistaken for an oversight)

- **Guard placement.** No mutation precedes `:74`. The three checks above it (`:51`, `:57`, `:63`) are
  pure reads. Error precedence mirrors the blockers side (`:1112` step → `:1123` player → `:1130`
  already-declared).
- **Marker placement.** Last `return Err` in the function is at `:671`; marker at `:763`; suspended
  early return at `:839`; final `Ok` at `:847`. No path reaches `:847` or `:839` on success without
  passing `:763`, and no `return Err` follows it.
- **No clearing needed.** `state.combat` is written at exactly four sites
  (`turn_actions.rs:1897`, `turn_actions.rs:2507`, `combat.rs:79`, `engine.rs:2962`). `end_combat`
  is `Step::EndOfCombat`'s own turn-based action (`turn_actions.rs:28`), and `advance_step` routes
  **every** exit from the combat phase through `EndOfCombat` — including the CR 508.8 skip (`:50-51`)
  and both extra-combat pops (`:55-80`, `:81-97`). There is no "end the turn" effect in the tree
  (grepped). `advance_turn` does not null `combat`, but cannot be reached mid-combat.
- **Hash.** Field is in the `HashInto` feed at `hash.rs:4462`, in struct-field order (between
  `first_strike_participants` and `defenders_declared`, matching `combat.rs:47/83/86`). History line
  appended at `:743-756`; `HashSchemaEpoch { version: 73 }` **appended** at `:1164-1180` with the v72
  row byte-identical above it; `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned at `hash_schema.rs:200-203`
  with the required note; `BASELINE_VERSION`/`BASELINE_*_FINGERPRINT` (the FROZEN v39 anchors)
  untouched. **No shipped row was edited.** T7 is confirmed to be the only coverage of the field's
  bytes — `hash_schema.rs:713-716` names `combat` as one of `canonical_fixture()`'s five exclusions.
- **PROTOCOL.** Zero occurrences of `CombatState` in `crates/engine/src/rules/protocol.rs`; the SR-8
  note on the new error variant (`error.rs:72-74`) is accurate.
- **Architecture Invariant 7.** `AlreadyDeclaredAttackers(PlayerId)` renders as
  `"player PlayerId(N) has already declared attackers this combat phase"`. Seat identity is public;
  no `ObjectId`, no card name. Correct, and correctly documented at `error.rs:67-70`.
- **Offer suppression correctness.** Keyed on `attackers_declared`, **not** on `attackers`, so a CR
  508.4 entrant cannot suppress a player's first declaration. Uses `state.combat()`, the same sealed
  accessor as its `DeclareBlockers` neighbour at `:937`. `PassPriority` is pushed unconditionally at
  `:515`, so suppression cannot strand a seat. `decision_kind_for` (`local_game.rs:1439-1456`)
  classifies from the action list, so a post-declaration window correctly becomes
  `DecisionKind::Priority`.
- **Struct-literal sites.** Re-derived by grep: `defenders_declared:` appears at 6 literal sites
  (5 pre-existing + T8's new one) and `attackers_declared: false` at exactly the 5 pre-existing ones,
  with T8 deliberately using `true`. Plan's table is exact.
- **`heuristic_bot.rs` deletion.** `RepeatKey::DeclareAttackers` gone from the enum, `cap()` and `of()`;
  `DeclareBlockers` still capped at 1 (`:54`) and still combat-scoped (`refresh_repeat_scope:161-173`,
  `in_combat:85`); the MR-M11-09 note is **kept and correctly re-scoped** (`:128-143`); the S8 stuck-game
  instance is preserved as a historical record (`:101-114`). Nothing needed was lost.
- **`local_game_playthrough.rs` deletion.** `PolicyState` and its threading are gone; step 3's comment
  is rewritten with the CR/PB cite (`:212-226`); the rejection-is-fatal assertion survives **verbatim
  and unweakened** at `:364-375` ("Any error at all is a failure — `Rejected` included"). The closure
  proof is intact.
- **MR-M11-09 test rewrite.** Not vacuous: it drives the real `HeuristicBot` through
  block → capped-pass → combat-exit → block-again, which is exactly the property MR-M11-09 named, on
  the surviving combat-scoped key.
- **New offer probe** (`local_game_human_actions.rs:781-823`) is three-state and discriminating on the
  added condition alone.
- **Golden scripts.** Two scripts carry ≥2 `"action": "declare_attackers"`
  (`combat/069`, `combat/070`); both are `"review_status": "retired"` and are excluded from
  `run_all_approved_scripts`. No golden fallout.
- **Direct-handler callers.** Only two exist (`pb_dx21_...rs`, `pb_dx22_fuzz_instrument.rs:886`); the
  latter declares once. `engine.rs:445` remains the only production caller.

---

## Previous Findings

Not applicable — first review of PB-DX21.
