# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | LOW Sweep campaign COMPLETE 2026-05-16 (`scutemob-31..38`): 36 LOWs closed, LOW-OPEN 45→6. 6 remain (honestly deferred). Plan: `memory/low-sweep-plan.md`. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | **Card Authoring Campaign ACTIVE** — plan: `memory/card-authoring/campaign-plan-2026-05-16.md` (§0 recalibration 2026-07-07 is authoritative; PB-first sequencing). PB-AC1/AC2/AC3 shipped 2026-07-07/08 (`scutemob-43..45`). Next: dispatch PB-AC4 (modal & optional targeting), then AC5..AC9 in chain order. Clean coverage 951/1,748 = 54.4%. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-07-08 (oversight session 2026-07-07/08 — coordinator dispatching)
**Workstream**: W6: Primitive + Card Authoring — Card Authoring Campaign, PB chain
**Task**: Plan recalibration + PB-AC1/AC2/AC3 dispatched, verified, collected (`scutemob-43..45`)

**Completed** (all merged to main AND pushed — origin in sync at `684e51c0`):
- **Origin push**: cleared the 15-commit unpushed backlog as first act; every merge since pushed same-day.
- **Plan recalibration** (`5c5dccb5`): campaign plan §0 added — measured 4/24 clean (17%) falsifies the "~435 free cards" estimate; **PB-first sequencing**; ETB cluster marked RESOLVED by PB-AC0; W-NOW batches 3+ paused; §1/§4/§6/§7 marked superseded.
- **PB-AC1** (`scutemob-43`, merge `5cd9a662`): `Effect::UntapAll`, `WheneverPermanentUntaps` + `WhenCounterPlaced` triggers, `once_per_turn` limiter, `DoesNotUntap` static. +20 tests (2893). Review 1 HIGH (`triggered_abilities_fired_this_turn` unhashed) + 2 MED — all fixed. Backfill 13 cards: 8 CLEAN / 5 PARTIAL. Coverage 928→934.
- **PB-AC2** (`scutemob-44`, merge `4d819ef4`): `Effect::MayPayThenEffect` (beneficial-pay wrapper — distinct from `MayPayOrElse` tax semantics) + `Effect::CounterUnlessPays`. CR corrected to 118.12/118.12a. +26 tests (2919). Review 0 HIGH / 2 MED — fixed. Backfill 20 cards: 12 CLEAN / 8 PARTIAL (Crossway Troublemakers + Miara riders live). Coverage 934→946.
- **PB-AC3** (`scutemob-45`, merge `0bd7c7a3`): `EffectAmount::{AttackingCreatureCount(19), TappedCreatureCount(20), HandSize(21)}` + `LayerModification::SetBothDynamic(28, Layer 7b)`; fixed pre-existing hash disc-26 collision (`RemoveSuperType`→29); schema 29→30. +21 tests (**2940**). Engine review clean; card review found **4 HIGH wrong-game-state** on PARTIALs (Mishra `Fixed(1)` drain, Ashaya/Multani dying 0/0, Wight self-sac) — all fixed, verdict RESOLVED. Coverage 946→**951 (54.4%)**, todo 621→616.

**Not done / deferred**:
- `tools/authoring-report.py` regex bug (pre-existing, found by AC3 worker): no word boundary, so `mana_abilities: vec![],` matches the empty-abilities pattern — Krenko misclassified "empty" though functionally correct + tested. One-line chore candidate (+1 honest clean).
- PB-AC4..AC9 not started; W-EMPTY / W-MISS derisking batches not run.

**Next session candidates** (highest-yield first):
- Dispatch **PB-AC4** (modal & optional targeting, ~20 cards), then AC5..AC9 in chain order. The AC1-3 rhythm is proven: task brief with hazards → worker runs `/implement-primitive` + backfill → coordinator verifies reviews/commits independently → collect. ~60-90 min per PB.
- Fold the authoring-report.py word-boundary fix into a chore commit.
- Optionally one 12-card derisking batch each of W-EMPTY / W-MISS to measure those cohorts before scheduling them (plan §0.4).

**Hazards** (carrying forward):
- Disk: each worktree builds ~30G `target/` — run dispatches **strictly sequential**, one worktree at a time.
- `esm worktree check` false `conflicts: True` again (scutemob-45) even though merge-base == main tip; caused by the phantom `.claude/skills` deletions and/or attestation branch-name drift. Verify with `git merge-base <main> <branch>` — if base == main tip, the merged tree is byte-identical to the worker tree where gates ran.
- Run `esm task unlock <id> --agent primary` right after the in_progress transition — coordinator lock blocks worker `signal-ready`.
- New struct fields / mutable runtime fields MUST be added to `state/hash.rs` `HashInto` impls (PB-AC1's HIGH was exactly this). Keep it in every PB task brief.
- Phantom `.claude/skills/*` deletions in fresh worktrees — do NOT commit (all 3 workers complied).
- Dual incomplete markers: grep BOTH `// TODO` and `// ENGINE-BLOCKED`.

**Commit prefix used**: worker `W6-prim:`, `merge:` for merges, coordinator `chore:`.

---

## Previous Handoff (preserved for chain context)

**Date**: 2026-07-07 session close (work merged under 2026-05-16/18 commit stamps)
**Workstream**: W6: Primitive + Card Authoring — **Card Authoring Campaign launched**
**Task**: Campaign triage + two derisking batches + one engine PB — `scutemob-39..42` + 1 chore, 5 merges, all local (origin 14 behind).

**Completed** (all merged to main):
- **scutemob-39** (merge `941e557e`): triage & scope — refreshed DSL gap audit (`memory/card-authoring/dsl-gap-audit-2026-05-16.md`) + campaign plan (`memory/card-authoring/campaign-plan-2026-05-16.md`). Plan estimate: ~435 authorable-now / ~470 engine-blocked behind 9 PBs (PB-AC1..AC9) / ~110 defer; ~75-session critical path.
- **scutemob-40** (merge `80b7cc44`): W-NOW-1 batch 1, 12 stale-TODO cards. Disposition **4 CLEAN / 5 PARTIAL / 3 BLOCKED**. Surfaced that the creature-ETB harness path silently dropped `has_subtype`/nontoken filters (review: `memory/card-authoring/review-scutemob-40.md`).
- **scutemob-41 / PB-AC0** (merge `df997fd2`): engine fix — creature-ETB path forwards `triggering_creature_filter` (`replay_harness.rs:2411`) and `abilities.rs` honors subtype + nontoken. Ganax + Lathliss ETB clauses now live; Miirym + The Great Henge latent over-triggers fixed. Reviewer NEEDS-FIX → PASS. Tests 2860→**2873** (+13).
- **scutemob-42** (merge `a0da201f`): W-NOW-1 batch 2, 12 cards. Disposition **0 CLEAN / 8 PARTIAL / 4 BLOCKED** (review 0 HIGH / 0 MEDIUM / 3 LOW, all addressed).
- **chore** (`fa4d593f`): `tools/authoring-report.py` counts `// ENGINE-BLOCKED` as incomplete — batch-2 cards had been miscounted clean (true clean 928 / 53.1%, not the false 938).

**Not done / deferred**:
- **Push to origin** — local main 14 commits ahead; a month of work exists only locally.
- **Campaign plan recalibration** — measured fully-clean rate is 4/24 (~17%), well under the audit's NOW-EXPRESSIBLE estimate; plan still lists the ETB cluster as "stale" (resolved by PB-AC0) and overstates free-card yield.
- PB-AC1..AC9 not started; W-NOW batches 3+ paused pending recalibration.

**Next session candidates** (highest-yield first):
- Push the 14 commits to origin (backup first).
- Recalibrate `campaign-plan-2026-05-16.md`: mark ETB cluster resolved by PB-AC0; discount authorable-now using the measured 17%-clean / 67%-partial mix; reorder PB-track-first (engine primitives are the bottleneck, not authoring throughput).
- Dispatch **PB-AC1** (untap / counter-placed / once-per-turn — highest-yield PB in the plan), then author its unblocked cohort behind it (the PB-AC0 rhythm: engine fix flips a whole cohort).

**Hazards** (carrying forward):
- Card authors mark incomplete clauses with `// TODO` OR `// ENGINE-BLOCKED` — any tooling or grep for incomplete cards must match BOTH markers (report tool fixed in `fa4d593f`).
- Gap-audit "NOW-EXPRESSIBLE" claims must be verified per card: measured batches show most stale-TODO cards still carry ≥1 genuinely blocked clause.
- Fresh worktrees show phantom `.claude/skills/*` deletions in `git status` — do NOT commit them (both workers correctly excluded; they vanish with the worktree).
- Disk hazard from the LOW-sweep note still applies: run dispatches strictly sequential, one worktree at a time.

**Commit prefix used**: worker `scutemob-N:` / `W5-cards:` / `W6-prim:`, `merge:` for merges, coordinator `chore:`.

---

## Handoff History

### 2026-05-16 (coordinator session — LOW Sweep campaign) — W3: LOW Remediation

- **8 fix sessions** (`scutemob-31..38`, plan `memory/low-sweep-plan.md`): 36 of 42 open LOWs closed, LOW-OPEN 45→**6** (4 M10-gated: MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). New DSL: `Effect::DestroyAndReanimate`, `Effect::PreventNextUntap`, `ProtectionQuality::{FromSuperType, FromName, FromPlayer}`; BASELINE-LKI-01 fixed (`pre_death_characteristics` snapshot, CR 603.10a/613.1e — audit `memory/primitives/lki-completeness-audit.md`, which filed +1 follow-up LOW: continuous-effect-granted triggered abilities lost via SelfDies). Tests 2819→**2860**; HASH 24→**27**. Origin hazards recorded: 4 parallel worktrees filled the disk to 100% (hence strictly-sequential rule); attestation-vs-real-branch-name drift causes false `esm worktree check` conflicts.

### 2026-05-15 (coordinator session 2 — 2-PB chain) — W6: Primitive

- **2 PBs shipped**: `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test, merged `e7d01fda`); `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED, merged `184162df`). Tests 2818→**2819**. High-confidence primitive backlog exhausted at session end.

### 2026-05-15 (7-PB autonomous chain) — W6: Primitive

- **7 PBs shipped** (`scutemob-22..28`): PB-XS-E (ETB `exclude_self`, HASH 19→20), OOS-EWC-2 (Golgari Grave-Troll), PB-XA (`is_attacking` enforcement, 10 sites), PB-EAT (`EntersAsAdditionalType`, HASH 20→21), PB-XA2 (`is_blocking`/`is_tapped`/`is_untapped`, HASH 21→22), OOS-XS-E-1 (audit-only, 0-yield), PB-EWC-D (`CreatureControlledByOfSubtype` + Dragonstorm Globe, HASH 22→23). Tests 2764→**2818** (+54). All worker-delegated; PB-XA2 worker self-collected without pre-merge reviewer (post-merge review 0 HIGH/0 MEDIUM/3 LOW). Full detail in git history + `memory/primitives/`.

### 2026-05-14 (PB-XS) — W6: Primitive

- **PB-XS shipped** (`scutemob-21`, merged `dbc17896`). `TargetFilter.exclude_self: bool` for "another target X" spell/ability target selection (CR 109.1 / 601.2c). Per-call-site validator enforcement across 4 filter-bearing TargetRequirement variants + 6 trigger auto-target-picker sites. 9 card defs updated (4 migrated bare `TargetCreature` → `TargetCreatureWithFilter`). HASH 18→**19**. Tests 2754→**2764** (+10). Review NEEDS-FIX → CLEAN (1 HIGH tautological test replaced with F-1/F-2 real-trigger discriminators). 5 OOS-XS seeds filed.

### 2026-05-14 (PB-EWC) — W6: Primitive

- **PB-EWC shipped** (`scutemob-20`, merged `9ea3ba8c`). `ReplacementModification::EntersWithCounters.count: u32 → Box<EffectAmount>` per CR 614.1c. Resolver builds `EffectContext` pinned to the replacement source and calls live-arm `resolve_amount` (handles `PowerOf(Source)`, `XValue`, `Fixed`). Zero pre-existing call-site reshapes — no live cards used the static u32. 2 cards: Master Biomancer counter half (Mutant-grant deferred to PB-EAT 2026-05-15) + Ingenious Prodigy (refactored from DEVIATION trigger stub to true replacement). HASH 17→**18**. Tests 2749→**2754** (+5). 3 OOS-EWC seeds filed; EWC-1 closed by 2026-05-15 PB-EAT, EWC-2 closed by 2026-05-15 OOS-EWC-2 dispatch, EWC-3 closed by 2026-05-15 PB-EWC-D.
- Review: PASS-WITH-NITS (0 HIGH, 0 MEDIUM, 7 LOW). E1 (defensive-default + race comment) fixed inline; 6 LOW triaged.


