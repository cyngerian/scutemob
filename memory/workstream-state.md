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
| W6: Primitive + Card Authoring | — | available | — | **Card Authoring Campaign ACTIVE** — plan: `memory/card-authoring/campaign-plan-2026-05-16.md`. Launched via `scutemob-39..42` + PB-AC0. Next: recalibrate plan, dispatch PB-AC1. Prior: 2 PBs 2026-05-15 (`scutemob-29..30`) — `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test) merged `e7d01fda`; `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED) merged `184162df`. Tests 2818→**2819**. Earlier 2026-05-15: 7-PB chain (`scutemob-22..28`, HASH 19→23). High-confidence primitive backlog now exhausted — see Last Handoff. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

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

## Previous Handoff (preserved for chain context)

**Date**: 2026-05-16 (coordinator session — W3 LOW Sweep campaign)
**Workstream**: W3: LOW Remediation
**Task**: LOW Sweep campaign — 8 dispatched fix sessions (`scutemob-31..38`) closing the entire actionable open-LOW backlog. Plan/tracker: `memory/low-sweep-plan.md`.

**Completed** (all merged to main):
- **LS-1** `scutemob-31` (`f492f815`): 6 commander/deck-validation LOWs (MR-M9-09/10/11/12/13/16).
- **LS-2** `scutemob-32` (`93f6d3b5`): 7 replay-viewer/harness LOWs (MR-M9.5-05/09/10/12/13, MR-CKP-01, MR-B12-05). Coordinator resolved a parallel `milestone-reviews.md` stats conflict.
- **LS-3** `scutemob-33` (`6010b5c9`): 6 combat LOWs (MR-M6-13, SR-TRM-01/02, SR-FS-02/03, MR-M2-16). SR-TRM-01/02 were 0-yield (already correct, stale doc refs).
- **LS-4** `scutemob-34` (`b760292a`): 5 protection LOWs (SR-PRO-01..04, MR-M9.4-10). New `ProtectionQuality::{FromSuperType, FromName, FromPlayer}`.
- **LS-5** `scutemob-35` (`e3fbd3da`): 6 replacement-effect LOWs (MR-M8-12/16, MR-B12-03/04, MR-B16-07, PB-Q4-L01).
- **LS-6** `scutemob-36` (`db49ddee`): PB-T-L01/L02/L03 unblocked — new `Effect::DestroyAndReanimate` + `Effect::PreventNextUntap` DSL primitives; Sorin/Tamiyo riders + Hands of Binding.
- **LS-7** `scutemob-37` (`d81e107c`): BASELINE-LKI-01 fixed — `pre_death_characteristics` snapshot on `CreatureDied` (CR 603.10a/613.1e). Audit: `memory/primitives/lki-completeness-audit.md`.
- **LS-8** `scutemob-38` (`9b4ed0d2`): MR-B11-08/09 — authored Insatiable Avarice ({B}-base Spree card) + 2 tests.

**Session totals**: 8 session merges + 1 `chore:` (authoring-report regen `019d6ba6`). 36 of 42 open LOWs closed; LOW-OPEN 45→**6**. Tests 2819→**2860**. HASH 24→**27** (LS-6 +2 effect discs, LS-7 +1). Final `cargo test --all` on main green; build/clippy/fmt clean.

**Residual — 6 LOWs deliberately NOT closed** (documented in `memory/low-sweep-plan.md`):
- 4 M10-gated: MR-M8-11 (CR 615.7 interactive shield choice), MR-B16-04/05/06 (interactive targeting / ForEach context). Ride M10.
- 2 permanent perf non-bottlenecks: MR-M1-18 (zone O(n) contains), MR-M6-14 (blockers_for rebuild).
- 1 follow-up LOW filed by LS-7 audit: continuous-effect-*granted triggered abilities* lost via `SelfDies`/`SelfLeavesBattlefield` (same LKI class) — see `lki-completeness-audit.md`.

**Hazards** (carrying forward):
- Disk: each worktree builds a ~30G `cargo target/`. 4 parallel worktrees filled the 396G disk to 100% and zeroed `~/.claude.json` (recovered from `~/.claude/backups/`). **Run LOW/PB sessions strictly sequential** — one worktree at a time, peak ~30G.
- `esm worktree check` reported a false conflict when the `working_branch` attestation didn't match the real branch name (esm worktree create appends `-triggers` etc.). Always read the real branch from `esm worktree list`, not the attestation.
- Worker `signal-ready` is rejected while the coordinator's lock is held — `esm task unlock <id> --agent primary` right after the in_progress transition so workers can self-transition to in_review.

**Commit prefix used**: worker-side `scutemob-N:`, `merge:` for merges, `chore:` for bookkeeping.

---

## Handoff History

### 2026-05-15 (coordinator session 2 — 2-PB chain) — W6: Primitive

- **2 PBs shipped**: `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test, merged `e7d01fda`); `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED, merged `184162df`). Tests 2818→**2819**. High-confidence primitive backlog exhausted at session end.

### 2026-05-15 (7-PB autonomous chain) — W6: Primitive

- **7 PBs shipped** (`scutemob-22..28`): PB-XS-E (ETB `exclude_self`, HASH 19→20), OOS-EWC-2 (Golgari Grave-Troll), PB-XA (`is_attacking` enforcement, 10 sites), PB-EAT (`EntersAsAdditionalType`, HASH 20→21), PB-XA2 (`is_blocking`/`is_tapped`/`is_untapped`, HASH 21→22), OOS-XS-E-1 (audit-only, 0-yield), PB-EWC-D (`CreatureControlledByOfSubtype` + Dragonstorm Globe, HASH 22→23). Tests 2764→**2818** (+54). All worker-delegated; PB-XA2 worker self-collected without pre-merge reviewer (post-merge review 0 HIGH/0 MEDIUM/3 LOW). Full detail in git history + `memory/primitives/`.

### 2026-05-14 (PB-XS) — W6: Primitive

- **PB-XS shipped** (`scutemob-21`, merged `dbc17896`). `TargetFilter.exclude_self: bool` for "another target X" spell/ability target selection (CR 109.1 / 601.2c). Per-call-site validator enforcement across 4 filter-bearing TargetRequirement variants + 6 trigger auto-target-picker sites. 9 card defs updated (4 migrated bare `TargetCreature` → `TargetCreatureWithFilter`). HASH 18→**19**. Tests 2754→**2764** (+10). Review NEEDS-FIX → CLEAN (1 HIGH tautological test replaced with F-1/F-2 real-trigger discriminators). 5 OOS-XS seeds filed.

### 2026-05-14 (PB-EWC) — W6: Primitive

- **PB-EWC shipped** (`scutemob-20`, merged `9ea3ba8c`). `ReplacementModification::EntersWithCounters.count: u32 → Box<EffectAmount>` per CR 614.1c. Resolver builds `EffectContext` pinned to the replacement source and calls live-arm `resolve_amount` (handles `PowerOf(Source)`, `XValue`, `Fixed`). Zero pre-existing call-site reshapes — no live cards used the static u32. 2 cards: Master Biomancer counter half (Mutant-grant deferred to PB-EAT 2026-05-15) + Ingenious Prodigy (refactored from DEVIATION trigger stub to true replacement). HASH 17→**18**. Tests 2749→**2754** (+5). 3 OOS-EWC seeds filed; EWC-1 closed by 2026-05-15 PB-EAT, EWC-2 closed by 2026-05-15 OOS-EWC-2 dispatch, EWC-3 closed by 2026-05-15 PB-EWC-D.
- Review: PASS-WITH-NITS (0 HIGH, 0 MEDIUM, 7 LOW). E1 (defensive-default + race comment) fixed inline; 6 LOW triaged.

### 2026-05-13 (PB-CD + PB-LKI-Power chain — same oversight, two PBs) — W6: Primitive

- **PB-CD shipped** (`scutemob-18`, merged `36816e0f`). Counter-doubling replacement effects (CR 122.6 / 614.1). Engine: `ReplacementTrigger::WouldPlaceCounters.counter_filter: Option<CounterType>` + `ObjectFilter::CreatureControlledBy(PlayerId)` disc 8 (layer-resolved creature type per CR 613.1d). Existing Vorinclex/Pir/Lae'zel preserved via `counter_filter: None`. 3 cards: Hardened Scales, Corpsejack Menace, Conclave Mentor (replacement half only — death trigger deferred as OOS-LKI-Power seed, closed by PB-LKI-Power). HASH 15→16. Tests +11. Review PASS (3 LOW: 1 CR-citation fix, 2 false-positives).
- **PB-LKI-Power shipped** (`scutemob-19`, merged `12218638`). LKI source-power snapshot for SelfDies/SelfLeavesBattlefield triggers (CR 603.10a / 122.2 / 400.7). `EffectAmount::SourcePowerAtLastKnownInformation` disc 18 (disc 19 reserved for toughness variant) + `lki_power: Option<i32>` through `PendingTrigger`/`StackObject`/`EffectContext`. Snapshot at `sba.rs:540` via `calculate_characteristics(state, source_id).power` BEFORE `move_object_to_zone`. 5 `GameEvent` variants extended (CreatureDied.pre_death_power HASHED; AuraFellOff/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand.pre_lba_power NOT hashed, mirrors PB-LKI-CC LBA precedent). 21-site dispatch chain. 2 cards: Conclave Mentor death-trigger life-gain + Juri Master of the Revue death-trigger damage. HASH 16→17. Tests +4. Review PASS-WITH-NITS → PASS after fix-phase. 5 OOS-LKI-Power seeds filed (-1..-5).
- Tests 2734→**2749** (+15 overall). HASH: 15→17 (two bumps).

