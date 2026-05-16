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
| W6: Primitive + Card Authoring | — | available | — | **2 PBs shipped 2026-05-15 (coordinator session 2)** — `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test) merged `e7d01fda`; `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED) merged `184162df`. Tests 2818→**2819**. Earlier 2026-05-15: 7-PB chain (`scutemob-22..28`, HASH 19→23). High-confidence primitive backlog now exhausted — see Last Handoff. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

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

## Prior Handoff (preserved for chain context)

**Date**: 2026-05-15 (coordinator session 2 — 2-PB sequential chain)
**Workstream**: W6: Primitive
**Task**: 2 PBs dispatched in sequence — `scutemob-29` (HASH-bumping, serialized first), then `scutemob-30` (audit). Both worker-delegated per dispatch policy; both spawned a reviewer before signal-ready.

**Completed**:
- **OOS-LKI-Power-3** (`scutemob-29`, merge `e7d01fda`): hashed `pre_lba_power` on 4 `GameEvent` variants (AuraFellOff, ObjectExiled, PermanentDestroyed, ObjectReturnedToHand) — previously skipped via `..` in `HashInto for GameEvent`. CR 603.10a/113.7a determinism + replay correctness. HASH 23→**24** + history entry. Sentinel sweep across all `primitive_pb_*.rs` canaries 23u8→24u8. +1 determinism test. Review verdict PASS. Sub-bullet (AnyCreatureDies `pre_lba_power: None` promotion) decision documented in `memory/primitives/pb-plan-OOS-LKI-Power-3.md`.
- **OOS-XA2-3** (`scutemob-30`, merge `184162df`): `is_nontoken` target-side enforcement audit. **0-yield** — the only `is_nontoken` in `defs/` (accursed_marauder.rs:26) is effect-side, not inside a `TargetRequirement::Target*WithFilter` block. No target-side consumer → no engine change. OOS-XA-3 and OOS-XA2-3 both marked RESOLVED in `pb-retriage-CC.md`. Audit-only signal-ready (OOS-XS-E-1 precedent).

**Session totals**: 2 PB merges. Tests 2818→**2819** (+1). HASH 23→**24** (1 schema bump). Build/clippy/fmt clean at every merge.

**Not done / deferred**:
- OOS-EWCD-1/2/3 — explicitly NOT dispatched: `pb-retriage-CC.md` says "no in-scope card currently needs this; file for future batch when a card surfaces." Dispatching speculative receiver-filter variants would violate the W6 "no primitive until a card needs it" policy.
- OOS-XA2-4 (CombatRole enum refactor), OOS-XA2-5 (runtime-predicate helper extraction) — LOW refactors, no correctness gap.
- Older OOS-LKI-Power-1/4/5, OOS-LKI-1..4, OOS-TS-1..4 — 0-yield defensives.

**Next session candidates**:
- High-confidence primitive backlog is exhausted; remaining OOS seeds are 0-yield defensives or card-gated. Recommend pivoting workstream: W2 TUI hardening, W3 LOW remediation (~45 open), or W6 card-authoring waves (Wave B re-triage / Wave C).

**Hazards** (carrying forward):
- Bash CWD-stickiness: coordinator shell drifted into a worktree after a verification `cd`; had to `cd` back to repo root before `/collect` (which requires being on main). Always `cd` back explicitly after entering a worktree.
- Both workers correctly spawned a reviewer before signal-ready — the PB-XA2 self-collect hazard did not recur.

**Commit prefix used**: worker-side `scutemob-N:`, `merge:` for merges, `chore:` for end-session.

---

## Previous Handoff (preserved for chain context)

**Date**: 2026-05-15 (coordinator/oversight session, 7-PB autonomous chain)
**Workstream**: W6: Primitive + Card Authoring
**Task**: 7 PBs shipped — `scutemob-22..28`. Each was dispatched in sequence (HASH-bumping work serialized), each worker delegated to `primitive-impl-runner` / `primitive-impl-reviewer` / `bulk-card-author` per the dispatch policy. Coordinator pushed 33 stale commits at session start (`9938ed90..c7580376`); session merged 7 new PBs + 1 stray-artifact commit on top.

**Completed (chain order)**:
- **PB-XS-E** (`scutemob-22`, merge `e0f3f5b0`): `TriggerCondition::WheneverCreatureEntersBattlefield/WheneverPermanentEntersBattlefield.exclude_self: bool` mirroring PB-23's `WheneverCreatureDies.exclude_self`. 3 cards (Metastatic Evangel, Shadow Alley Denizen, Forerunner of the Legion). 3 dies-side cards (Boggart Shenanigans, Athreos, Meren) routed to OOS-XS-E-1 follow-up (later closed as audit-only). HASH 19→**20**. Tests 2764→**2775** (+11). Filed OOS-XS-E-1, -2.
- **OOS-EWC-2** (`scutemob-23`, merge `87c3a306`): pure card-authoring. Golgari Grave-Troll — self-ETB `EntersWithCounters` with `EffectAmount::CardCount { zone: Graveyard(Controller), filter: creature }`. Dredge 6 keyword. No engine work (PB-EWC already shipped the count-EffectAmount path). Trample CORRECTION: original printing does NOT have trample per MCP — task description was wrong; worker caught and didn't add it. Tests 2775→**2779** (+4).
- **PB-XA** (`scutemob-24`, merge `b42e06bb`): `TargetFilter.is_attacking` enforcement at 4 validate sites (V1-V4) + 6 trigger auto-target picker sites (T1-T6) in `casting.rs`/`abilities.rs`. Mirrors PB-XS `exclude_self` pattern. Thousand-Faced Shadow inline TODO removed. HASH unchanged (pre-existing field). Tests 2779→**2789** (+10). Filed OOS-XA-1 (`is_blocking`), OOS-XA-2 (`is_tapped`/`is_untapped`), OOS-XA-3 (`is_nontoken` target-side audit).
- **PB-EAT** (`scutemob-25`, merge `75302138`): new `ReplacementModification::EntersAsAdditionalType { subtype: SubType }` variant. Resolver in `emit_etb_modification` pushes subtype into `state.objects[new_id].characteristics.subtypes` BEFORE `PermanentEnteredBattlefield` (CR 614.1c entry modification, NOT Layer 4). Master Biomancer Mutant half authored — closes OOS-EWC-1. HASH 20→**21**. Tests 2789→**2794** (+5). Filed OOS-EAT-1/2/3 (CardType/Color/Supertype — all 0-yield defensive).
- **PB-XA2** (`scutemob-26`, merge `f3905b62`): `TargetFilter.is_blocking` + `is_tapped` + `is_untapped` runtime predicates. Same 10-site enforcement pattern as PB-XA. OR-semantics for "attacking or blocking" implemented as two-bool with explicit 4-way match on `(is_attacking, is_blocking)`: `(F,F)→true, (T,F)→attackers, (F,T)→blockers, (T,T)→attackers||blockers`. Added `CombatState::is_blocking(id)` helper. 1 card (Eiganjo Seat of the Empire Channel half). HASH 21→**22**. Tests 2794→**2811** (+17). Filed OOS-XA2-1..5. **Worker self-collected without spawning pre-merge reviewer**; post-merge review captured in `memory/primitives/pb-review-XA2.md` (committed separately `45f6bac5`). Verdict 0 HIGH / 0 MEDIUM / 3 LOW.
- **OOS-XS-E-1** (`scutemob-27`, merge `a78e8481`): pure audit of Boggart Shenanigans / Athreos / Meren `WheneverCreatureDies` triggers. **Audit result**: all 3 cards LACK the trigger entirely — they're DSL gaps for unrelated reasons (subtype filter, Layer-4 RemoveCardTypes static). No `exclude_self` fix needed. Plan file `pb-plan-XSE1.md` documents oracle wording for each card. Tests 2811 → **2811** (+0). HASH unchanged.
- **PB-EWC-D** (`scutemob-28`, merge `0a84badc`): new `ObjectFilter::CreatureControlledByOfSubtype { controller, subtype }` variant + `bind_object_filter` extension for `OwnedByOpponentsOf(PlayerId(0)) → OwnedByOpponentsOf(controller)` on `WouldEnterBattlefield` triggers (closes E2 from `pb-review-EWC.md`). Dragonstorm Globe authored ("Each Dragon you control enters with an additional +1/+1 counter"). HASH 22→**23**. Tests 2811→**2818** (+7). Filed OOS-EWCD-1/2/3.

**Session totals**: 7 PB merges + 1 chore commit. Tests 2764 → **2818** (+54). HASH 19→**23** (4 schema bumps in one session). Build / clippy / fmt clean at every merge. Branch pushed before session (`origin/main` at `c7580376`) — local now 8 commits ahead at session end.

**Not done / deferred**:
- All filed OOS seeds (XS-E-2, XA-1/2/3 closed via PB-XA2, XA2-1..5, EAT-1/2/3, EWCD-1/2/3) — except XA-1/2 which were closed by PB-XA2.
- Older OOS-LKI-Power-1/3/4/5 (0-yield defensive), OOS-LKI-1..4, OOS-TS-1..4 still untouched.
- High-complexity defers: OOS-XS-1 (inter-target distinctness), OOS-XS-3 (Olivia multi-effect — blocked on AddSubtype Layer 4), OOS-XS-4 (Skrelv ChooseColor + protection-from-color).
- `docs/project-status.md` Card Health still stale (use `tools/authoring-report.py`).

**Next session candidates** (highest yield first):
- **OOS-XA2-3 carry-forward `is_nontoken` target-side audit** — small re-audit. Likely 0-yield but resolves an ambiguity.
- **OOS-EWCD-1/2/3 receiver-filter expansion** — byte-for-byte parallel of PB-EWC-D. Defensive; defer until a real card surfaces (none in corpus today).
- **OOS-LKI-Power-3 hash-arm inconsistency sweep** — 4 `GameEvent` variants don't hash their `pre_lba_*` fields. Pre-existing engine-consistency cleanup. Bumps HASH 23→24.
- **Skip 0-yield defensives** until a card surfaces. The high-confidence backlog is exhausted for now.

**Hazards** (carrying forward):
- **Worker self-collect without independent review**: PB-XA2 worker (`scutemob-26`) shipped without spawning a separate `primitive-impl-reviewer` agent before signaling ready. Post-merge review caught 3 LOW issues but found 0 HIGH/MEDIUM. **Coordinator should explicitly include "spawn primitive-impl-reviewer before signal-ready"** in the dispatch prompt for runner-led PBs, OR check `git log <branch> --oneline` for a `fix-phase` commit before collecting.
- **OOS-XS-E-1 audit-only outcome**: a "verify and fix if needed" task can resolve to "no fix needed" — that's a valid signal-ready state. Worker correctly documented and signal-readied with N=0 test additions.
- **Task description accuracy drift**: the Golgari Grave-Troll dispatch criterion incorrectly claimed the card had Trample. Worker caught via MCP verification and ignored the false claim. Reinforces `feedback_oversight_primitive_category_not_cards` — oversight describes the PRIMITIVE; worker verifies card-level attributes from oracle text.
- **CWD-stickiness in Bash tool**: no incidents this session.
- **`feedback_worker_satisfy_before_signal_ready`**: enforced — all workers satisfied all criteria before `signal-ready`.

**Commit prefix used**: worker-side `scutemob-N:` (with `scutemob-N: fix-phase` for review fixes), `merge:` for merge commits, coordinator-side `chore:` for end-session bookkeeping and the stray PB-XA2 review artifact.

---

## Handoff History

### 2026-05-14 (PB-XS) — W6: Primitive

- **PB-XS shipped** (`scutemob-21`, merged `dbc17896`). `TargetFilter.exclude_self: bool` for "another target X" spell/ability target selection (CR 109.1 / 601.2c). Per-call-site validator enforcement across 4 filter-bearing TargetRequirement variants + 6 trigger auto-target-picker sites. 9 card defs updated (4 migrated bare `TargetCreature` → `TargetCreatureWithFilter`). HASH 18→**19**. Tests 2754→**2764** (+10). Review NEEDS-FIX → CLEAN (1 HIGH tautological test replaced with F-1/F-2 real-trigger discriminators). 5 OOS-XS seeds filed.

### 2026-05-14 (PB-EWC) — W6: Primitive

- **PB-EWC shipped** (`scutemob-20`, merged `9ea3ba8c`). `ReplacementModification::EntersWithCounters.count: u32 → Box<EffectAmount>` per CR 614.1c. Resolver builds `EffectContext` pinned to the replacement source and calls live-arm `resolve_amount` (handles `PowerOf(Source)`, `XValue`, `Fixed`). Zero pre-existing call-site reshapes — no live cards used the static u32. 2 cards: Master Biomancer counter half (Mutant-grant deferred to PB-EAT 2026-05-15) + Ingenious Prodigy (refactored from DEVIATION trigger stub to true replacement). HASH 17→**18**. Tests 2749→**2754** (+5). 3 OOS-EWC seeds filed; EWC-1 closed by 2026-05-15 PB-EAT, EWC-2 closed by 2026-05-15 OOS-EWC-2 dispatch, EWC-3 closed by 2026-05-15 PB-EWC-D.
- Review: PASS-WITH-NITS (0 HIGH, 0 MEDIUM, 7 LOW). E1 (defensive-default + race comment) fixed inline; 6 LOW triaged.

### 2026-05-13 (PB-CD + PB-LKI-Power chain — same oversight, two PBs) — W6: Primitive

- **PB-CD shipped** (`scutemob-18`, merged `36816e0f`). Counter-doubling replacement effects (CR 122.6 / 614.1). Engine: `ReplacementTrigger::WouldPlaceCounters.counter_filter: Option<CounterType>` + `ObjectFilter::CreatureControlledBy(PlayerId)` disc 8 (layer-resolved creature type per CR 613.1d). Existing Vorinclex/Pir/Lae'zel preserved via `counter_filter: None`. 3 cards: Hardened Scales, Corpsejack Menace, Conclave Mentor (replacement half only — death trigger deferred as OOS-LKI-Power seed, closed by PB-LKI-Power). HASH 15→16. Tests +11. Review PASS (3 LOW: 1 CR-citation fix, 2 false-positives).
- **PB-LKI-Power shipped** (`scutemob-19`, merged `12218638`). LKI source-power snapshot for SelfDies/SelfLeavesBattlefield triggers (CR 603.10a / 122.2 / 400.7). `EffectAmount::SourcePowerAtLastKnownInformation` disc 18 (disc 19 reserved for toughness variant) + `lki_power: Option<i32>` through `PendingTrigger`/`StackObject`/`EffectContext`. Snapshot at `sba.rs:540` via `calculate_characteristics(state, source_id).power` BEFORE `move_object_to_zone`. 5 `GameEvent` variants extended (CreatureDied.pre_death_power HASHED; AuraFellOff/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand.pre_lba_power NOT hashed, mirrors PB-LKI-CC LBA precedent). 21-site dispatch chain. 2 cards: Conclave Mentor death-trigger life-gain + Juri Master of the Revue death-trigger damage. HASH 16→17. Tests +4. Review PASS-WITH-NITS → PASS after fix-phase. 5 OOS-LKI-Power seeds filed (-1..-5).
- Tests 2734→**2749** (+15 overall). HASH: 15→17 (two bumps).

### 2026-04-30 ~01:00–05:00 EDT (PB-TS + PB-LKI-CC chain) — W6: Primitive

- **PB-TS shipped** (`scutemob-16`, merged `68f4bfbc`). `TokenSpec.count: u32 → EffectAmount` — dynamic token count via `resolve_amount` integration at `effects/mod.rs:540-590` + `601-668` before `apply_token_creation_replacement` boundary. 4 cards re-authored: Phyrexian Swarmlord, Krenko Mob Boss, Izoni Thousand-Eyed, Chasm Skulker (reverted in fix-phase pending PB-LKI-CC). HASH 13→14. Tests +5. Review NEEDS-FIX → PASS. 4 OOS-TS seeds filed.
- **PB-LKI-CC shipped** (`scutemob-17`, merged `a2b24e42`). `EffectAmount::CounterCountAtLastKnownInformation { counter }` (disc 17) — LKI snapshot threaded `pre_death_counters → PendingTrigger.lki_counters → StackObject.lki_counters → EffectContext.lki_counters → resolve_amount`. Fix-phase E1 swept all 5 `SelfLeavesBattlefield` dispatch arms (~35 emit sites across 5 engine files). 2 cards: Chasm Skulker re-authored from PB-TS revert + Toothy Imaginary Friend retroactive correctness fix. HASH 14→15. Tests +9. Review PASS (1 HIGH + 3 LOW resolved). 2 OOS-LKI seeds filed.
- Tests 2720→**2734** (+14). New hazard: worker forgot satisfy step before signal-ready (captured in feedback memory `feedback_worker_satisfy_before_signal_ready.md`).

### 2026-04-29 evening – 2026-04-30 ~00:50 EDT (PB-CC-C-followup + canonical authoring-status tooling) — W6: Primitive + tooling

- **PB-CC-C-followup shipped** (`scutemob-15`, merged `7182a4c8`). Worker chose Shape A+D hybrid: new `AbilityDefinition::CdaModifyPowerToughness` variant + live-eval branch reusing `resolve_cda_amount`. Substitution path (CR 608.2h) untouched. Static-ability path (CR 611.3a) re-resolves dynamic EffectAmount in `calculate_characteristics()`. HASH 12→13. Tests 2716→2720 (+4). Vishgraz + Exuberant Fuseling re-authored, TODO citations cleared. Fuseling's `WheneverCreatureOrArtifactDies` death-trigger half remains TODO (separate primitive).
- **Review verdict PASS-WITH-NITS**: 0 HIGH, 1 MEDIUM (E1 — asymmetric P/T amounts dispatch dropped one component; fix-phase split variant into two-effect dispatch). All LOWs resolved.
- **Authoring-status tooling shipped** (committed `faf1c7e8`, 5 files / 1,323 lines): `tools/authoring-report.py` (deterministic stdlib-only generator), `docs/authoring-status.md` (auto-regenerated), `docs/authoring-status-guide.md` (reading guide), `docs/authoring-status-missing.txt` (worklist), `docs/authoring-status-prev.json` (Δ snapshot). Headline at commit: 1748 def files; 88.1% plan coverage; 321 bonus defs; 915 clean / 652 todo / 181 empty.
- Coordinator data-correction: earlier "10 cards added in last month" claim was wrong by order of magnitude; actual `git log` shows 278 new + 332 modified in last 30 days.

