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
| W3: LOW Remediation | — | available | — | W3-LOW sprint-1 + sprint-2 shipped 2026-04-25: 13 LOWs closed. ~45 open. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | **PB-CC-C-followup shipped 2026-04-30** (`scutemob-15`, merged `7182a4c8`). Layer-7c continuous CDA primitive landed via Shape A+D hybrid (`AbilityDefinition::CdaModifyPowerToughness` + live-eval branch reusing `resolve_cda_amount`). HASH 12→13. Tests 2716→**2720** (+4). Vishgraz + Exuberant Fuseling unblocked. Review PASS-WITH-NITS, 0 HIGH, 1 MEDIUM (E1 fixed in fix-phase before signal-ready). Also this session: deterministic authoring-status report tooling shipped (`tools/authoring-report.py`, `docs/authoring-status*` — committed `faf1c7e8`). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-29 evening – 2026-04-30 ~00:50 EDT
**Workstream**: W6: Primitive (ESM-managed) + tooling
**Task**: PB-CC-C-followup dispatch + collect (single worker) + canonical authoring-status report tooling

**Completed**:
- **PB-CC-C-followup shipped** (`scutemob-15`, merged `7182a4c8` --no-ff). Worker chose Shape A+D hybrid: new `AbilityDefinition::CdaModifyPowerToughness` variant + live-eval branch reusing `resolve_cda_amount`. Substitution path (CR 608.2h) untouched — PB-CC-C T5 regression preserved. New path (CR 611.3a) re-resolves dynamic EffectAmount in `calculate_characteristics()`. 8 commits in worktree (engine + hash + cards + tests + clippy + fix-phase + memory artifacts).
- **Cards unblocked**: `vishgraz_the_doomhive.rs` (CDA Layer 7c P/T per opponent poison) + `exuberant_fuseling.rs` (CDA Layer 7c power per oil counter on self). Both PB-CC-C-followup TODO citations cleared. Fuseling's `WheneverCreatureOrArtifactDies` death-trigger half remains TODO — out-of-scope separate primitive.
- **HASH 12→13**, sentinel-assertion test files updated (6 test files this batch). Tests 2716→**2720** (+4 mandatory: a-e). Build/clippy/fmt all clean. Verified post-merge cargo test --workspace = 2720 passed.
- **Review verdict**: PASS-WITH-NITS. 0 HIGH, 1 MEDIUM (E1 — asymmetric P/T amounts in `CdaModifyPowerToughness` register dispatch silently dropped one component; Vishgraz unaffected since amounts identical, but engine type-discipline gap). E1 fixed in fix-phase by splitting variant into two-effect dispatch. AC 3723 ("no open HIGH/MEDIUM") satisfied.
- **Authoring-status tooling** (committed `faf1c7e8`, 5 files / 1,323 lines): `tools/authoring-report.py` (deterministic generator, stdlib-only), `docs/authoring-status.md` (auto-regenerated, never hand-edit), `docs/authoring-status-guide.md` (hand-written reading guide; documents intentionally-skipped scope), `docs/authoring-status-missing.txt` (sidecar worklist of 194 plan cards still missing on disk), `docs/authoring-status-prev.json` (snapshot for run-over-run Δ column). Headline at commit time: 1,748 def files; 88.1% plan coverage (1,442/1,636); 321 bonus defs traced to W2 split + W1-B ability batches + W6-prim samples; 915 clean / 652 todo / 181 empty.
- **Discovery (data correction)**: earlier "10 cards added in last month" claim was wrong by an order of magnitude. Actual git log shows 278 NEW card files + 332 modified existing files in last 30 days (Wave A complete 91 cards, A-38 batch 1 53 cards, A-42 batch 1+2 77 cards, etc.). The `docs/project-status.md` "1456 / 1743 (84%)" snapshot from 2026-03-30 is stale — actual is **88.1% plan coverage** with **108% effective coverage** when bonus defs are counted.

**Not done / deferred**:
- 9 named seeds in `memory/primitives/pb-retriage-CC.md` stop-and-flag log still untouched (PB-TS token-count is biggest umbrella, ~5+ cards).
- TODO classifier in `tools/authoring-report.py` covers 47 patterns; OTHER bucket still 682/1187 lines (~57%). Trivial to extend by adding regexes — see "Raw OTHER samples" section of the report.
- `docs/project-status.md` is now demonstrably stale (per authoring-report numbers vs. its 2026-03-30 snapshot). Worth a refresh pass to align with the canonical report.
- `.esm/` and `.worktrees/` not in `.gitignore` — show as untracked every session. Trivial fix queued but not done.

**Next session**:
- **Most natural**: pick up one of the 9 stop-and-flag seeds. PB-TS (token-count → EffectAmount) has the largest yield (~5 cards: Phyrexian Swarmlord, Chasm Skulker, Anim Pakal, Krenko, Izoni) and is well-scoped from prior re-triage memos.
- **Alternative**: re-run `python3 tools/authoring-report.py` and use the lagging-group verdicts (`activated-tap` unwritten, `untap-phase` engine-blocked, etc.) to pick a Wave authoring run or a primitive batch.
- **Bookkeeping option**: refresh `docs/project-status.md` Card Health section to point at the authoring-status report as canonical, and gitignore `.esm/` + `.worktrees/`.

**Hazards** (carrying forward + this session's findings):
- **CWD-stickiness in Bash tool** *(re-confirmed this session)*: shell drifted into `.worktrees/scutemob-15` from an earlier `cd` and `git branch --show-current` then returned the worker branch. Fix used: `cd /home/skydude/projects/scutemob` before merge. The hazard from prior handoff is real and recurring — recipe stands. Always reset cwd to absolute repo root before `esm worktree merge`.
- **Dispatch-vs-inline correction** *(this session)*: I started running `/implement-primitive` inline as the coordinator. User correctly intervened — coordinator must `/dispatch` to a worker, never run primitive batches inline. Reinforced in `feedback_dispatch_not_inline.md` (new this session).
- **`esm task transition --attest working_branch=<short>` poisons merge** (recipe carried forward from 2026-04-29). Pass the FULL long-form branch from `esm worktree create`.
- **`bash -c '...'` apostrophe parse error** (carried forward).
- **Worker-worktree `.claude/skills/` deletion artifact** (carried forward; `git diff main..HEAD --stat` post-merge).
- **Carried-forward LOWs**: BASELINE-LKI-01, PB-Q4-M01, marisi stale-TODO, 11 PB-T LOWs, 5 PB-P LOWs, 1 PB-D LOW, 4×PB-CC review memo LOWs. Plus this session's 1 LOW from PB-CC-C-followup review (E1 was MEDIUM but fixed; check `memory/primitives/pb-review-CC-C-followup.md` if it exists).

**Commit prefix used**: worker-agent-generated (`scutemob-15:` worker side, `merge:` for merge commit) + coordinator `chore: authoring-status report tool` for inline tooling work.

## Handoff History

### 2026-04-29 ~01:00–05:00 EDT (5-PB autonomous chain — PB-SFT + PB-CC umbrella) — W6: Primitive

- **Phase A re-triage** (`scutemob-8`/`scutemob-9`): PB-SFT verdict PROCEED — FIELD-ADDITION (gap on Effect not Cost); PB-CC verdict UMBRELLA-OF-MICRO-PRIMITIVES (4 micro-PBs).
- **Wave 1 parallel** (`scutemob-10` + `scutemob-11`): PB-SFT (`Effect::SacrificePermanents.filter`) + PB-CC-W (Mossborn Hydra Landfall wire-up). 7+ cards re-authored (Fleshbag/Merciless/Butcher/Dictate/Grave Pact/Liliana DH-4/Blasphemous Edict/etc.).
- **Wave 2 sequential** (`scutemob-12`/`scutemob-13`/`scutemob-14`): PB-CC-B (`TargetFilter.has_counter_type` + Armorcraft Judge), PB-CC-C (`LayerModification::ModifyPower/ToughnessDynamic` — Fuseling deferred Option B), PB-CC-A (`EffectAmount::PlayerCounterCount` — Vishgraz deferred Option B; same trap).
- Tests 2689→2716 (+27). HASH bumped 4×. Pushed `051442bd..fd6c8e6a`.
- **Discovery**: two CDA target cards hit deeper architectural gap — `ModifyBothDynamic`-style substitution locks value at registration but CR 611.3a requires continuous re-eval. Filed PB-CC-C-followup seed (later shipped as `scutemob-15`).

### 2026-04-25 (W3-LOW sprint-1 + sprint-2 chain dispatch)

- W3-LOW sprint-1 (`scutemob-6`, merged `c6c3592b`): T1 mechanical cleanup, ~14min. SR-FS-01 closed (verified absent), PB-N-L01 indentation reflowed in 5 card defs, BASELINE-CLIPPY-04 deleted + 27 clippy warnings fixed. `cargo clippy --all-targets -- -D warnings` actually exits 0 (PB-T's prior claim was wrong). 4 commits, 54 files, net −117 LOC. Tests still 2686.
- W3-LOW sprint-2 (`scutemob-7`, merged `c7a93c5e` + `afd7c34d` artifact-fixup): T3 behavioral, ~38min. PB-S-L02/L03/L04 base-char→`calculate_characteristics(state, id)` (CR 613.1f), L05 granted-index invariant documented, L06 Humility-before-grant test added. +3 regression tests. Tests 2686→2689. 9 commits.
- Worker-worktree contamination caught + fixed: sprint-2 worker bundled `.claude/skills/` deletion (27 files) + `.esm/worker.md` add. Caught at post-merge `git diff main..HEAD --stat`. Recipe: `git checkout HEAD^1 -- .claude/skills/` + `git rm .esm/worker.md`.
- 13 LOWs closed total. ~45 open. Pushed to origin.

### 2026-04-20 (PB-T single-worker dispatch) — W6: TargetRequirement::UpToN

- W6 re-triage (pre-dispatch): old queue (PB-R/Q3/U/V/W/Y/Q2/Q5) verified 0-1 live TODOs each; new-rank candidates identified (Cost::SacrificeFilteredType rank 3 ~12 live, EffectAmount::CounterCount rank 6 ~10 live). PB-T picked. **PB-T shipped** (`scutemob-5`, merged `2d447e93`): `TargetRequirement::UpToN { count, inner }` per CR 601.2c. Two-pass best-fit validator (out-of-slot-order legal). Auto-target routing for `UpToN{Player}` + nested `UpToN`. HASH 7→8. 22-card oracle sweep → 14 CONFIRMED (64% yield). 14 cards unblocked. 13 tests in `pbt_up_to_n_targets.rs`. Review: needs-fix (1 HIGH validator + 5 MEDIUM) → fix → re-review PASS. Tests 2673→2686. Wall clock ~113 min.

### 2026-04-19 (chain-dispatch session) — W6: PB-P + PB-L shipped sequentially via ESM

- **Push** (`52e2c9dc..872ea5d2`): 11 commits pushed to `origin/main` in two pushes. **PB-P shipped** (`scutemob-3`, merged `8ba9c5b7`): `EffectAmount::PowerOfSacrificedCreature` + `AdditionalCost::Sacrifice` reshape to `{ ids, lki_powers }` for CR 608.2b LKI capture-by-value. 3 cards (altar_of_dementia, greater_good, lifes_legacy). HASH 5→6. Review PASS-WITH-NOTES (5 LOW). **PB-L shipped** (`scutemob-4`, merged `872ea5d2`): Step 0 verdict reversed mid-task (EXISTS → PARTIAL-GAP). No new `TriggerCondition` variant (Landfall = ability word CR 207.2c). Minimal primitive: `ETBTriggerFilter.card_type_filter` + battlefield conversion block in `replay_harness.rs`. 3 cards + 5 TODO rewrites. HASH 6→7. Memo `memory/primitives/pb-note-L-collapsed.md`. **Chain-dispatch pattern validated**: single coordinator ran two `/dispatch` → poll → `/collect` cycles, no user intervention mid-chain. Tests 2655→2673.

### 2026-04-19 (A/B session) — W6: ESM install, PB-D A/B, dispatch skill hardening

- ESM install committed (`aca3035e`, `a253c24f`); PB-D A/B via `/dispatch` (scutemob-1 inline 68 files w/ scope creep, scutemob-2 agent-delegated 14 files PASS-WITH-NOTES, scutemob-2 merged at `72cddb62`); dispatch skill hardened (`7d255645`) to require granular TaskCreate list + specialized-agent delegation; two feedback memory files added. PB-D shipped: `TargetController::DamagedPlayer` + 10 dispatch sites + 6 card defs + 7 MANDATORY tests. Hash 4→5. Tests 2648→2655. 1 LOW carried (marisi_breaker TODO).

