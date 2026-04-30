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
| W6: Primitive + Card Authoring | — | available | — | **PB-LKI-CC shipped 2026-04-29** (`scutemob-17`). `EffectAmount::CounterCountAtLastKnownInformation { counter }` — LKI snapshot threaded `pre_death_counters → PendingTrigger.lki_counters → StackObject.lki_counters → EffectContext.lki_counters → resolve_amount`. HASH 14→**15**. Tests 2725→**2730** (+5). Chasm Skulker + Toothy Imaginary Friend unblocked. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-29
**Workstream**: W6: Primitive (worker, scutemob-17)
**Task**: PB-LKI-CC — `EffectAmount::CounterCountAtLastKnownInformation` for WhenDies/WhenLeavesBattlefield

**Completed**:
- **PB-LKI-CC implemented** (`scutemob-17`, branch `feat/pb-lki-cc-effectamountcountercount-lki-snapshot-for-whendies`). Path A chosen: new `EffectAmount::CounterCountAtLastKnownInformation { counter }` (disc 17). Counter snapshot threaded: `pre_death_counters → PendingTrigger.lki_counters → StackObject.lki_counters → EffectContext.lki_counters → resolve_amount arm`.
- **Cards fixed**: `chasm_skulker.rs` (WhenDies: X Squid tokens) + `toothy_imaginary_friend.rs` (WhenLeavesBattlefield: draw X cards). Both now use the LKI variant instead of `CounterCount{Source}` which returns 0 after zone transition.
- **HASH 14→15**: `PendingTrigger.lki_counters` + `StackObject.lki_counters` + new `EffectAmount` variant all hashed. `HASH_SCHEMA_VERSION` bumped. 8 sentinel-assertion test files updated.
- **Tests 2725→2730** (+5): `crates/engine/tests/primitive_pb_lki_cc.rs` — (a) Chasm Skulker 3 counters → 3 Squids, (b) Toothy 4 counters → 4 draws, (c) zero counters → 0 tokens no panic, (d) multi-type counter discrimination, (e) HASH sentinel.
- **`im` added to dev-dependencies** (`Cargo.toml`) — 14 test files constructing `StackObject` directly needed `im::OrdMap::new()` for the new field. 1 internal test in `casting.rs` also fixed. `primitive_pb37.rs` EffectContext direct construction uses `None`.
- **OOS seeds filed**: `OOS-LKI-1` (Hardened Scales + LKI tokens — confirmed no interaction) + `OOS-LKI-2` (Parallel Lives + LKI token count — confirmed working correctly). Appended to `memory/primitives/pb-retriage-CC.md`.
- Build/clippy/fmt all clean. `cargo build --workspace` clean (no TUI/replay-viewer gaps — `EffectAmount` match arms added in `layers.rs` + `hash.rs`; no new enum variants requiring exhaustive matches elsewhere).

**Not done / deferred**:
- PB-LKI-CC review memo not yet written (plan calls for review after implementation). Worker outputs are complete; review phase is coordinator responsibility.
- Remaining 8 stop-and-flag seeds in `pb-retriage-CC.md` still untouched.
- `docs/project-status.md` Card Health section still stale (canonical: `tools/authoring-report.py`).

**Next session**:
- Coordinator should collect `scutemob-17` (merge to main), then pick next primitive or card authoring wave.
- The next stop-and-flag seed with significant yield is `PB-CC-B` (Armorcraft Judge / `TargetFilter.has_counter_type`) or a Wave authoring run.

**Hazards** (carrying forward):
- **CWD-stickiness in Bash tool**: always use absolute paths.
- **`esm task transition --attest working_branch=<short>` poisons merge**: pass full long-form branch from `esm worktree create`.
- **Worker-worktree `.claude/skills/` deletion artifact**: check `git diff main..HEAD --stat` post-merge.
- **Carried-forward LOWs**: BASELINE-LKI-01, PB-Q4-M01, marisi stale-TODO, 11 PB-T LOWs, 5 PB-P LOWs, 1 PB-D LOW, 4×PB-CC review memo LOWs.

**Commit prefix used**: `scutemob-17:` (worker side)

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

