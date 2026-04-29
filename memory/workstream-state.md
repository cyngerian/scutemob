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
| W6: Primitive + Card Authoring | — | available | — | PB-SFT + PB-CC umbrella shipped 2026-04-29 via 5-PB autonomous chain (scutemob-10..14). Tests 2689→2716. ~10 cards unblocked (PB-SFT 7+; PB-CC-W Mossborn; PB-CC-B Armorcraft Judge; PB-CC-C/PB-CC-A engine machinery shipped, target CDA cards Vishgraz + Exuberant Fuseling deferred to **PB-CC-C-followup** seed = "Layer 7c dynamic CDA with continuous re-evaluation per CR 611.3a"). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-29 ~01:00–05:00 EDT (5-PB autonomous chain — PB-SFT + PB-CC umbrella)
**Workstream**: W6: Primitive (ESM-managed)
**Task**: pre-impl re-triage of PB-SFT + PB-CC (parallel) → 5-PB autonomous impl chain (1 parallel + 3 sequential) → push → end-session

**Completed**:
- **Phase A re-triage** (`scutemob-8` `700f13c8` + `scutemob-9` `037c3547`): two parallel re-triage memos. PB-SFT verdict PROCEED — FIELD-ADDITION (planner mis-named; gap is on Effect, not Cost). PB-CC verdict UMBRELLA-OF-MICRO-PRIMITIVES (4 micro-PBs: W/B/C/A).
- **Wave 1 parallel** (`scutemob-10` `da566418` + `scutemob-11` `0a46e1b4`): PB-SFT (`Effect::SacrificePermanents.filter: Option<TargetFilter>`, 7+ cards re-authored: Fleshbag Marauder, Merciless Executioner, Butcher of Malakir, Dictate of Erebos, Grave Pact, Liliana DH-4, Blasphemous Edict, Roiling Regrowth, etc.) + PB-CC-W (Mossborn Hydra Landfall stale-TODO wire-up, no engine).
- **Wave 2 sequential** (`scutemob-12` `06d02990`, `scutemob-13` `e2d75672`, `scutemob-14` `fd6c8e6a`): PB-CC-B (`TargetFilter.has_counter_type` field + Armorcraft Judge ETB), PB-CC-C (`LayerModification::ModifyPower/ToughnessDynamic` engine variants — Exuberant Fuseling **deferred** Option B per reviewer), PB-CC-A (`EffectAmount::PlayerCounterCount` engine variant — Vishgraz CDA **deferred** Option B per reviewer, same trap as PB-CC-C).
- **Tests 2689→2716** (+27 net). HASH bumped 4×. All clippy clean. Pushed `051442bd..fd6c8e6a` to origin/main.
- **Discoveries**: (1) two CDA target cards (Exuberant Fuseling, Vishgraz) hit a deeper architectural gap — `ModifyBothDynamic`-style substitution **locks the value at registration**, but CR 611.3a requires CDAs to be **continuously re-evaluated** ("static ability, NOT locked-in"). Engine machinery shipped is useful for spell-instant pump effects; CDAs need new primitive **PB-CC-C-followup**: "Layer 7c dynamic CDA with continuous re-evaluation". TODOs documented in vishgraz_the_doomhive.rs and exuberant_fuseling.rs with full citation. (2) Yield calibration tracked: planner estimated 12-16, actual ~10 unblocked — matches `feedback_pb_yield_calibration.md` 2-3x discount.

**Not done / deferred**:
- **PB-CC-C-followup** (new primitive seed): Layer 7c dynamic CDA with continuous re-evaluation. Cards: Vishgraz the Doomhive, Exuberant Fuseling (and likely all "+X/+0 for each counter" CDAs). Architecturally distinct from PB-CC-A/C — needs Layer 7c CDA path that re-resolves the EffectAmount on every layer recomputation, not at registration time.
- Marisi authoring — DamagedPlayer goad ForEach shippable; "opponents can't cast spells during combat" remains DSL gap.
- ~45 LOWs open (unchanged this session).
- Re-triage candidates from previous handoff still untouched: 9 named seeds in pb-retriage-CC.md stop-and-flag log (PB-TS token-count, per-target dynamic, counter-doubling replacement, library-empty draw replacement, counter-threshold trigger gate, exclude-self filter, multi-target grant over filter set, replacement-effect counter substitution, kicker-driven ETB counters).

**Next session**:
- Two natural candidates: (a) **PB-CC-C-followup** — design + ship the Layer 7c continuous CDA primitive (unblocks Vishgraz + Exuberant Fuseling that this session deferred); (b) re-triage of any of the 9 named seeds from the PB-CC stop-and-flag log (PB-TS token-count is biggest umbrella — 5+ cards: Phyrexian Swarmlord, Chasm Skulker, Anim Pakal, Krenko/Izoni token scaling).

**Hazards**:
- **CWD-stickiness in Bash tool**: `cd .worktrees/scutemob-N` keeps the cwd across subsequent bash calls in the same session. `esm worktree merge scutemob-N` then fails with double-pathed error. Fix: always use absolute paths with `esm` commands, or `cd /home/skydude/projects/scutemob` to reset before merge.
- **`esm task transition --attest working_branch=<short>` poisons the merge**: ESM uses the attested branch name as the merge target. `esm worktree create` returns the FULL long-form branch name — pass it verbatim. If the short form was attested, fall back to manual `git merge --no-ff <full-branch-name>` + `git worktree remove --force .worktrees/scutemob-N` + `git branch -D <full-branch>`. Recipe used at scutemob-13.
- **`bash -c '...'` apostrophe parse error**: long claude prompts inside single-quoted bash break on `don't`/`won't`/etc. Fix: write prompt to `/tmp/<task>-prompt.txt` then `PROMPT=$(cat /tmp/<task>-prompt.txt)` and `claude "$PROMPT"`. Used at scutemob-14.
- **Worker-worktree `.claude/skills/` deletion artifact** (still relevant; same recipe). `git diff main..HEAD --stat` post-merge check.
- **Carried-forward LOWs**: BASELINE-LKI-01, PB-Q4-M01, marisi stale-TODO (authorable), 11 PB-T LOWs, 5 PB-P LOWs, 1 PB-D LOW. Plus new LOWs from this session: PB-SFT, PB-CC-B, PB-CC-C, PB-CC-A review memos each carry 2-5 LOWs (see memory/primitives/pb-review-*.md).

**Commit prefix used**: worker-agent-generated (`scutemob-N:` / `W3:` style) + coordinator `chore:` for end-session updates.

## Handoff History

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

### 2026-04-13 (PB-D planner session) — W6: PB-D plan phase

- Opus planner run (`b9f43bf1`): `memory/primitives/pb-plan-D.md` written. Verdict PASS-AS-NEW-VARIANT (`TargetController::DamagedPlayer`), 6 confirmed cards of 15 classified (~40% yield), ~10 dispatch sites across casting/abilities/effects/hash, hash bump 4→5, 7 mandatory + 2 optional tests. Step 0 stale-TODO sweep returned positive null; Step 1 PB-P pre-check found PB-P is real-but-narrow (real gap is `EffectAmount::PowerOf(SacrificedCreature)` LKI read — Altar of Dementia / Greater Good). BASELINE-LKI-01 verified structurally NOT reaching PB-D. 0 stop-and-flags. `memory/primitive-wip.md` halted at phase=plan-complete pending oversight greenlight.

