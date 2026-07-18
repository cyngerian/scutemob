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
| W6: Primitive + Card Authoring | — | available | — | **Campaign wave queue COMPLETE 2026-07-17**: marker sweep + SR-33..38 + W-PB2 + W-EMPTY + W-MISS all collected (`scutemob-88..98`). Clean coverage **1,065/1,781 = 59.8%**. **Active queue: `memory/primitives/ef-batch-plan-2026-07-17.md`** (PB-EF1..EF12, correctness-first). EF-13 decision pending (plan §3). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-07-16..17 (oversight session — coordinator dispatching, user-authorized autonomous chaining)
**Workstream**: W6: Primitive + Card Authoring (+ SR follow-on chain)
**Task**: 11 tasks dispatched/collected (`scutemob-88..98`) — marker sweep, SR-33..38, W-PB2, W-EMPTY, W-MISS, EF triage

**Completed** (all merged to main AND pushed same-day):
- **Marker sweep** (`scutemob-88`, `1a7f8c4f`): all 742 non-Complete markers audited vs the shipped engine; **42% of notes wrong**; 13 upgrades, 266 rewrites, 54 partial→known_wrong; 116-card blocker-grouped worklist; `registers_no_behavior` + `inert_gate_is_not_vacuous` replace the false-minting inert check.
- **SR-33** (`scutemob-89`, `953cc5a6`): 102 Complete-but-dead lands (88 filed + 14 gate-caught Triomes/surveil/Hierarchs) rewritten to one-ability-per-colour; `Effect::Choose`/`MayPayOrElse`/`AddManaChoice` gated out of Complete (serde-tree walk); 7 honest demotions incl. rhystic_study/path_to_exile.
- **SR-34** (`scutemob-90`, `ce6f30b0`): composite-cost mana abilities register + collect payment (CR 605.1a "by what it does"); 27 defs probed by activation, 7/27 source-traced predictions falsified; PROTOCOL 2→3, HASH 40→41.
- **SR-35** (`scutemob-91`, `7b2310dd`): card-def corpus format-checked for the first time — `cargo fmt` covered ZERO of 1,748 defs, 321 misformatted; `tools/check-defs-fmt.sh` + CI step; `format_strings`/`error_on_line_overflow` each pinned by canaries (naive gate was blind for 79% of corpus).
- **SR-36** (`scutemob-92`, `264f0e9e`): SF-8 scaled mana (Gaea's Cradle 0→0, N→N, ×Nyxbloom) + SF-9 PayLife collected; **blast radius ~7× the filing — entire 11-card fetchland cycle fetched for free**; Cabal Coffers/Stronghold/Crypt upgraded; PROTOCOL 3→4, HASH 41→42.
- **SR-37** (`scutemob-93`, `df49eb61`): `ManaAbility.activation_condition` honored (enrich's `..` silently dropped it); `AddManaAnyColor` family gated, 18 demotions; land gate parses "any color"; HASH 42→43, PROTOCOL 4→5.
- **SR-38** (`scutemob-94`, `ac65216a`): simulator `StubProvider` gates suggestions on `life_cost` (CR 119.4b), mirroring engine checks.
- **W-PB2** (`scutemob-95`, `7c8cdeff`): 57-card roster from sweep worklist, 47 Complete in 5 reviewed batches, EF-W-PB2-1..8 filed. Coverage → 58.9%.
- **W-EMPTY** (`scutemob-96`, `a9152c83`): plan's "~110" was stale — 3 authorable of 60 remaining inert (+2 Complete; disciple stayed partial, EF-W-EMPTY-1).
- **W-MISS** (`scutemob-97`, `9cec7673`): 194 missing re-derived → 35 authorable; 33 Complete, 2 honest mid-wave demotions (Ojutai, Misdirection); EF-W-MISS-1..10 filed incl. latent swan_song.rs token-recipient bug. Coverage → **59.8%** (corpus 1,781).
- **EF triage** (`scutemob-98`, `ef82ae45`): all 20 findings deduped/classified → **`memory/primitives/ef-batch-plan-2026-07-17.md`** (PB-EF1..EF12 + PB-EF3b, correctness-first, discounted yields); campaign plan §0 repointed.
- Coordinator chores: CLAUDE.md snapshot after every collection; SR handoff note saved to auto-memory (`project_sr_track_closure_handoff.md`).

**Not done / deferred**:
- **PB-EF1** (recommended first dispatch) + swan_song demote not started.
- **EF-13 decision pending** (105 partial-but-inert defs; options A/B/C in ef-batch-plan §3 — user call).
- 61 retired scripts worklist untouched.

**Next session candidates** (highest-yield first):
- Dispatch **PB-EF1** per ef-batch-plan (correctness-first; swan_song demote rides along).
- Get the **EF-13 decision** from the user, then execute the chosen option (cheap).
- Retired-scripts worklist batches (each names its un-retire blocker).

**Hazards** (carrying forward):
- **Activation probes beat source-tracing, every time**: W-EMPTY 110→3, W-MISS 115→35, SR-34 falsified 7/27, SR-36 blast radius 2→14 then ~7×. Rosters must be probed from `all_cards()` + activation, never regex/plan estimates.
- Version bumps are machine-forced: new `Effect` variant → PROTOCOL + history row; `GameState`/`HashInto` change → HASH + history row. Never re-pin a fingerprint without bumping.
- `cargo fmt` still checks zero defs — `tools/check-defs-fmt.sh` (or `cargo test --all`) is the def format gate; don't delete its two `--config` flags (canary-pinned).
- Stub effects are gated: Choose / MayPayOrElse / AddManaChoice / AddManaAnyColor family cannot appear in a Complete def.
- Count marker classes from the compiled registry — the `abilities: vec![]` regex trap fired 3× more this session (documented in CLAUDE.md; still bites sub-agents).
- Coordinator+worker both editing CLAUDE.md Last-Updated causes merge conflicts (hit once, SR-35) — resolve by stacking entries, keep worker's Tests line.
- Still applies: strictly-sequential dispatches; `esm task unlock` right after in_progress; recon-first (SR-36's 3 stub family members found under other names).

**Commit prefix used**: worker `scutemob-N:`, `merge:` for merges, coordinator `chore:`.

---

## Previous Handoff (preserved for chain context)

**Date**: 2026-07-08..10 (oversight session — coordinator dispatching; /eot run 2026-07-16)
**Workstream**: W6: Primitive + Card Authoring — Card Authoring Campaign, PB-AC chain close
**Task**: PB-AC4..AC9 dispatched, verified, collected (`scutemob-46/47/49/50/51/52`) — **AC chain complete (AC0..AC9)**

- **PB-AC4** (`dca25ec0`): `ModeSelection.mode_targets` per-mode targeting (CR 601.2c) + Escalate fail-safe; backfill 11 migrated. Tests 2940→2957.
- **PB-AC5** (`0ce2c470`): Warp, Transmute, Exert (both shapes), `Cost::ExileFromHand`+Pitch, `CounterSpell.exile_instead`; 2 HIGH unhashed-field fixes. Tests →2984.
- **PB-AC6** (`0628807e`): main-phase sweeps, `WhenBecomesTarget`, 5 Conditions, 3 PlayerState trackers. Tests →3009.
- **PB-AC7** (`2f214906`): `SetCreatureTypes`/`SetCardTypes` Layer 4 (CR 205.1a correlated-subtype HIGH; CR 613.8 depends_on). Tests →3035.
- **PB-AC8** (`a2aea440`): `CantAttackOwner`, `CantBeSacrificed` (both choke points), `Effect::WinGame` (worker corrected inverted CR 104.3h). Tests →3062.
- **PB-AC9** (`a4750cdb`): `WheelHand` + `SetNoMaximumHandSize`; **token doubling rewired 2→13/13 sites** (doublers silently failing); Reforge stale-marker HIGH → both workers recommended the marker sweep (executed this session). Tests →3090; coverage 983 (56.2%) at chain close.
- Hazards that stayed load-bearing: recon-first (2-3 primitives per PB already existed); HashInto omissions as review HIGHs (engineered out via mutation-verified hash tests in criteria); worker-overturns-brief 3×; `build --workspace` ≠ test compile but IS the seal gate; CR file bare `\r` — use MCP, never grep.

---

## Handoff History

### 2026-07-08 (oversight session) — W6: PB-AC1..AC3 + plan recalibration

- **Recalibration** (`5c5dccb5`): §0 added — 4/24 clean (17%) falsified "~435 free cards"; PB-first sequencing. **PB-AC1** (`5cd9a662`): UntapAll, untap/counter triggers, once_per_turn, DoesNotUntap; 1 HIGH unhashed. **PB-AC2** (`4d819ef4`): `MayPayThenEffect` + `CounterUnlessPays` (CR 118.12). **PB-AC3** (`0bd7c7a3`): 3 EffectAmounts + `SetBothDynamic` Layer 7b; hash disc-26 collision fixed; 4 HIGH wrong-game-state PARTIALs fixed. Tests 2873→2940; coverage 951 (54.4%). Hazards: ~30G target/ per worktree → strictly sequential; false `esm worktree check` conflicts (verify merge-base); unlock after in_progress; phantom `.claude/skills` deletions.

### 2026-07-07 (coordinator session — campaign launch) — W6: Primitive + Card Authoring

- **Campaign triage + 2 derisking batches + PB-AC0** (`scutemob-39..42` + chore, 5 merges): DSL gap audit + campaign plan written (~435 authorable-now estimate, falsified next session at 17% measured clean); W-NOW-1 batches 1-2 (4 CLEAN / 13 PARTIAL / 7 BLOCKED over 24 cards); **PB-AC0** creature-ETB filter forwarding (`df997fd2`, +13 tests, 2860→**2873**); `authoring-report.py` taught to count `// ENGINE-BLOCKED` (true clean 928 / 53.1%). Deferred at close: origin 14 ahead (pushed next session), plan recalibration (done next session as §0).

### 2026-05-16 (coordinator session — LOW Sweep campaign) — W3: LOW Remediation

- **8 fix sessions** (`scutemob-31..38`, plan `memory/low-sweep-plan.md`): 36 of 42 open LOWs closed, LOW-OPEN 45→**6** (4 M10-gated: MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). New DSL: `Effect::DestroyAndReanimate`, `Effect::PreventNextUntap`, `ProtectionQuality::{FromSuperType, FromName, FromPlayer}`; BASELINE-LKI-01 fixed (`pre_death_characteristics` snapshot, CR 603.10a/613.1e). Tests 2819→**2860**; HASH 24→**27**. Origin hazards recorded: 4 parallel worktrees filled the disk to 100% (hence strictly-sequential rule); attestation-vs-real-branch-name drift causes false `esm worktree check` conflicts.

### 2026-05-15 (coordinator session 2 — 2-PB chain) — W6: Primitive

- **2 PBs shipped**: `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test, merged `e7d01fda`); `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED, merged `184162df`). Tests 2818→**2819**. High-confidence primitive backlog exhausted at session end.

### 2026-05-15 (7-PB autonomous chain) — W6: Primitive

- **7 PBs shipped** (`scutemob-22..28`): PB-XS-E (ETB `exclude_self`, HASH 19→20), OOS-EWC-2 (Golgari Grave-Troll), PB-XA (`is_attacking` enforcement, 10 sites), PB-EAT (`EntersAsAdditionalType`, HASH 20→21), PB-XA2 (`is_blocking`/`is_tapped`/`is_untapped`, HASH 21→22), OOS-XS-E-1 (audit-only, 0-yield), PB-EWC-D (`CreatureControlledByOfSubtype` + Dragonstorm Globe, HASH 22→23). Tests 2764→**2818** (+54). All worker-delegated; PB-XA2 worker self-collected without pre-merge reviewer (post-merge review 0 HIGH/0 MEDIUM/3 LOW). Full detail in git history + `memory/primitives/`.
