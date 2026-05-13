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
| W6: Primitive + Card Authoring | — | available | — | **2 PBs shipped 2026-04-30 in chain** — `scutemob-16` PB-TS (`TokenSpec.count: u32→EffectAmount`, 4 cards Phyrexian Swarmlord/Krenko/Izoni/Chasm Skulker reverted, HASH 13→14, +5 tests) merged `68f4bfbc`; `scutemob-17` PB-LKI-CC (`EffectAmount::CounterCountAtLastKnownInformation`, 2 cards Chasm Skulker re-author + Toothy retroactive fix, HASH 14→**15**, +9 tests via fix-phase E1 full-LBA sweep) merged `a2b24e42`. Tests 2720→**2734** (+14). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-30 ~01:00–05:00 EDT
**Workstream**: W6: Primitive (coordinator chain — 2 PBs back-to-back)
**Task**: PB-TS + PB-LKI-CC dispatch-collect chain (both shipped)

**Completed**:
- **PB-TS shipped** (`scutemob-16`, merged `68f4bfbc`). `TokenSpec.count: u32 → EffectAmount` — dynamic token count via `resolve_amount` integration at `effects/mod.rs:540-590` + `601-668` before `apply_token_creation_replacement` boundary. 4 cards re-authored: Phyrexian Swarmlord, Krenko Mob Boss, Izoni Thousand-Eyed, Chasm Skulker (Chasm later reverted in fix-phase pending PB-LKI-CC). HASH 13→14. Tests 2720→2725 (+5). Review NEEDS-FIX → PASS (E1 Krenko sorcery-speed + C1 Chasm 0-tokens). 4 OOS seeds filed (OOS-TS-1 through 4). Wall clock ~82min.
- **PB-LKI-CC shipped** (`scutemob-17`, merged `a2b24e42`). `EffectAmount::CounterCountAtLastKnownInformation { counter }` (disc 17) — LKI snapshot threaded `pre_death_counters → PendingTrigger.lki_counters → StackObject.lki_counters → EffectContext.lki_counters → resolve_amount`. Fix-phase E1 swept all 5 `SelfLeavesBattlefield` dispatch arms — added `pre_lba_counters: OrdMap<CounterType, u32>` field to 4 `GameEvent` variants (`AuraFellOff`/`PermanentDestroyed`/`ObjectExiled`/`ObjectReturnedToHand`), updated ~35 emit sites across `casting.rs`/`engine.rs`/`resolution.rs`/`turn_actions.rs`/`abilities.rs`. 2 cards: Chasm Skulker (re-authored from PB-TS revert), Toothy Imaginary Friend (retroactive correctness — was producing 0 draws on bounce/exile/destroy). HASH 14→15. Tests 2725→2734 (+9). Review NEEDS-FIX → PASS (1 HIGH + 3 LOW, all resolved). 2 OOS seeds filed (OOS-LKI-3 Workhorse cost-LKI + OOS-LKI-4 AnyCreatureDies). Wall clock ~128min.
- **Bookkeeping**: `.gitignore` updated to hide `.esm/` + `.worktrees/` (queued from prior session). `.claude/skills/` contamination cleanup applied per recipe after both merges. Authoring report regenerated twice — TODO lines 1187→1182→1180; clean count 915→916.

**Not done / deferred**:
- 8 stop-and-flag seeds in `memory/primitives/pb-retriage-CC.md` still untouched (OOS-LKI-3/4 newly added). Counter-doubling replacement (Hardened Scales et al., CR 121.6) and `TargetFilter.exclude_self` (Éomer) remain attractive next picks.
- `docs/project-status.md` Card Health section still stale (canonical: `tools/authoring-report.py`).
- Toothy Imaginary Friend now has correct draw count on bounce/exile/destroy but should be regression-tested under copy/replication scenarios (worker did a basic regression sweep).

**Next session**:
- Pick another primitive from the OOS seeds. **OOS-LKI-3** (Workhorse cost-payment LKI) is natural follow-up since LKI infrastructure is fresh. Or pivot to a new primitive family: counter-doubling replacement (CR 121.6 — 3 named EDH staples), or `TargetFilter.exclude_self` (5+ cards likely).
- Alternative: refresh `docs/project-status.md` Card Health to point at the canonical authoring-status report.

**Hazards** (carrying forward + reconfirmed this session):
- **CWD-stickiness in Bash tool** *(reconfirmed twice this session — once for each PB)*: `cd` does NOT reliably persist between bash invocations in this tool. Reliable recipe: `cd /home/skydude/projects/scutemob && <command>` in the SAME bash invocation, every time. The second occurrence happened when `esm worktree merge` tried to find the worktree at a doubled path because cwd was inside it.
- **Worker forgot to mark criteria satisfied before signal-ready** *(new this session, scutemob-17)*: worker completed all work (review PASS, tests pass, memos exist) but never ran `esm task satisfy` for the 7 criteria. Coordinator verified independently and applied. Possibly worth strengthening the worker prompt or dispatch skill to require satisfy step before signal-ready.
- **`esm task transition --attest working_branch=<short>` poisons merge** (carried forward).
- **Worker-worktree `.claude/skills/` deletion artifact** (carried forward; `.esm/worker.md` add now suppressed by .gitignore but `.claude/skills/` deletion still happens — recipe: `git checkout HEAD^1 -- .claude/skills/`).
- **Carried-forward LOWs**: BASELINE-LKI-01, PB-Q4-M01, marisi stale-TODO, 11 PB-T LOWs, 5 PB-P LOWs, 1 PB-D LOW, 4×PB-CC review memo LOWs, 3 LOWs from PB-LKI-CC review memo (E1/E2/E3 all resolved but check memo for any LOWs filed).

**Commit prefix used**: worker-agent-generated (`scutemob-16:` / `scutemob-17:` worker side, `merge:` for merge commits) + coordinator `chore:` for both post-collect cleanups.

## Handoff History

### 2026-04-29 evening – 2026-04-30 ~00:50 EDT (PB-CC-C-followup + canonical authoring-status tooling) — W6: Primitive + tooling

- **PB-CC-C-followup shipped** (`scutemob-15`, merged `7182a4c8`). Worker chose Shape A+D hybrid: new `AbilityDefinition::CdaModifyPowerToughness` variant + live-eval branch reusing `resolve_cda_amount`. Substitution path (CR 608.2h) untouched. Static-ability path (CR 611.3a) re-resolves dynamic EffectAmount in `calculate_characteristics()`. HASH 12→13. Tests 2716→2720 (+4). Vishgraz + Exuberant Fuseling re-authored, TODO citations cleared. Fuseling's `WheneverCreatureOrArtifactDies` death-trigger half remains TODO (separate primitive).
- **Review verdict PASS-WITH-NITS**: 0 HIGH, 1 MEDIUM (E1 — asymmetric P/T amounts dispatch dropped one component; fix-phase split variant into two-effect dispatch). All LOWs resolved.
- **Authoring-status tooling shipped** (committed `faf1c7e8`, 5 files / 1,323 lines): `tools/authoring-report.py` (deterministic stdlib-only generator), `docs/authoring-status.md` (auto-regenerated), `docs/authoring-status-guide.md` (reading guide), `docs/authoring-status-missing.txt` (worklist), `docs/authoring-status-prev.json` (Δ snapshot). Headline at commit: 1748 def files; 88.1% plan coverage; 321 bonus defs; 915 clean / 652 todo / 181 empty.
- Coordinator data-correction: earlier "10 cards added in last month" claim was wrong by order of magnitude; actual `git log` shows 278 new + 332 modified in last 30 days.

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

