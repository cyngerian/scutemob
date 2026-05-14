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
| W6: Primitive + Card Authoring | — | available | — | **3 PBs shipped 2026-05-13/14 in chain** — `scutemob-18` PB-CD (`ReplacementTrigger::WouldPlaceCounters.counter_filter` + `ObjectFilter::CreatureControlledBy`, 3 cards Hardened Scales/Corpsejack Menace/Conclave Mentor replacement-half, HASH 15→16, +11 tests) merged `36816e0f`; `scutemob-19` PB-LKI-Power (`EffectAmount::SourcePowerAtLastKnownInformation` disc 18 + `lki_power: Option<i32>` LKI snapshot, 2 cards Conclave Mentor death-trigger + Juri Master, HASH 16→**17**, +4 tests via 21-site dispatch chain) merged `12218638`; `scutemob-20` PB-EWC (`ReplacementModification::EntersWithCounters.count` `u32`→`Box<EffectAmount>`, 2 cards Master Biomancer counter half + Ingenious Prodigy CR 614.1c-correct ETB, HASH 17→**18**, +5 tests, OwnedByOpponentsOf rebind sub-gap routed to OOS-EWC-3) signaled-ready 2026-05-14. Tests 2734→**2754** (+20). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-05-14 (worker session)
**Workstream**: W6: Primitive (PB-EWC)
**Task**: `scutemob-20` PB-EWC — `ReplacementModification::EntersWithCounters` count `u32` → `Box<EffectAmount>` (CR 614.1c). **Signaled ready 2026-05-14.**

**Completed**:
- **Engine migration**: `ReplacementModification::EntersWithCounters` count migrated `u32` → `Box<EffectAmount>` (boxed to satisfy `clippy::large_enum_variant` — `EffectAmount::Sum`/`CardCount` make it large). Resolver `emit_etb_modification` gained `replacement_source: Option<ObjectId>` parameter; for self-ETB the source is the entering permanent itself (so `EffectAmount::XValue` reads its propagated `x_value`); for global ETB the source comes from `ReplacementEffect.source` (so `PowerOf(EffectTarget::Source)` reads the replacement-owner's live, layer-resolved power per CR 614.12). `resolve_amount` visibility widened to `pub(crate)`. New `WouldEnterBattlefield { filter }` rebind clause in `register_permanent_replacement_abilities` (non-self branch) — without it, Master Biomancer's `CreatureControlledBy(PlayerId(0))` placeholder would leak through registration. HASH 17→**18**.
- **2 cards authored/refactored**: **Master Biomancer** counter half via non-self replacement with `count: PowerOf(EffectTarget::Source)` (type-grant half filed as OOS-EWC-1). **Ingenious Prodigy** ETB clause refactored from triggered-ETB stub (with CR 614.1c DEVIATION comment) to true self-ETB replacement with `count: XValue` — DEVIATION comment removed; upkeep draw trigger preserved.
- **5 new tests** in `tests/primitive_pb_ewc.rs`: (a) live-power base 2/4 → 2 counters, (b) pumped-power 4/6 → 4 counters (calculate_characteristics Layer 7d), (c) Ingenious Prodigy X=5 timing, (d) HASH==18 canary, (e) X=0 absence (discriminating via `contains_key` check). Existing `x_cost_spells.rs:test_x_cost_etb_counters_ingenious_prodigy` and `replacement_effects.rs` call sites migrated to `Box::new(EffectAmount::Fixed(N))`.
- **10 PB hash canary tests** bumped from `HASH_SCHEMA_VERSION, 17u8` to `18u8` via sed sweep.
- **3 new OOS seeds filed** in `pb-retriage-CC.md`: **OOS-EWC-1** (`EntersAsAdditionalType` for Master Biomancer's "as a Mutant" type-grant half), **OOS-EWC-2** (Golgari Grave-Troll self-ETB CardCount over creature graveyard — pure card-authoring follow-up, no engine work needed), **OOS-EWC-3** (Dragonstorm Globe subtype receiver filter; absorbed E2 sub-gap for `OwnedByOpponentsOf(PlayerId(0))` placeholder rebind in `bind_object_filter` for `WouldEnterBattlefield`). OOS-LKI-Power-2 (the originating seed) closed.
- **Review**: `primitive-impl-reviewer` agent verdict PASS-WITH-NITS — 0 HIGH, 0 MEDIUM, 7 LOW (all dispositioned: 5 RESOLVED inline — defensive-default `let-else` cleanup, 4 stale-line-number doc-comment drops, T1 discriminating absence check; 1 ROUTED to OOS-EWC-3 — `OwnedByOpponentsOf` sub-gap; 1 already RESOLVED — comment style). See `memory/primitives/pb-review-EWC.md` § Resolution.
- **Tests**: 2749→**2754** (+5). Build/clippy/fmt clean. **HASH**: 17→18.

**Not done / deferred**:
- OOS-EWC-1/2/3 untouched (Master Biomancer type-grant, Golgari Grave-Troll, Dragonstorm Globe).
- 5 prior OOS-LKI-Power seeds (-1 through -5) untouched.
- 8 prior OOS-TS/LKI seeds untouched.
- `TargetFilter.exclude_self` (Éomer + "for each other [type]" cards) still attractive small primitive.
- `docs/project-status.md` Card Health section still stale (canonical: `tools/authoring-report.py`).

**Next session candidates**:
- **OOS-EWC-2 Golgari Grave-Troll** — pure card-authoring; engine work done after PB-EWC. Single self-ETB with `CardCount { Graveyard(Controller), CreatureCard }`. Should take a single short session.
- **`TargetFilter.exclude_self`** (Éomer + ~5 "for each other [type]" cards) — small contained primitive.
- **OOS-EWC-1 EntersAsAdditionalType** — Master Biomancer type-grant half. New `ReplacementModification` variant + minor resolver arm + HASH bump.

**Hazards** (carrying forward):
- **CWD-stickiness in Bash tool**: same as prior; recipe is `cd /home/skydude/projects/scutemob && <command>` in same bash invocation. Did not bite this session.
- **`feedback_worker_satisfy_before_signal_ready`**: enforced — all 8 criteria satisfied before `signal-ready`.
- **`feedback_verify_full_chain`**: the `register_permanent_replacement_abilities` `WouldEnterBattlefield` filter-rebind gap was caught only because test (a) failed loudly on the first run (got 0 counters, not 2). Without the failing test, the placeholder-leak would have shipped silently. Confirms the recurring pattern: any new `ReplacementTrigger` variant must walk the full registration → rebind → match dispatch chain. New OOS-EWC-3 sub-gap (`OwnedByOpponentsOf` rebind in `bind_object_filter`) is the same shape; left as a tracked seed because no in-scope card hits it today.

---

## Previous Handoff (preserved for chain context)

**Date**: 2026-05-13/14 (oversight session, 2 PBs back-to-back)
**Workstream**: W6: Primitive (coordinator chain)
**Task**: PB-CD + PB-LKI-Power dispatch-collect chain (both shipped)

**Completed**:
- **PB-CD shipped** (`scutemob-18`, merged `36816e0f`). Counter-doubling replacement effects (CR 122.6 / 614.1). Engine: `ReplacementTrigger::WouldPlaceCounters.counter_filter: Option<CounterType>` for counter-type gating + `ObjectFilter::CreatureControlledBy(PlayerId)` disc 8 for receiver scope (layer-resolved creature type per CR 613.1d). Existing Vorinclex/Pir/Lae'zel preserved via `counter_filter: None`. 3 cards: Hardened Scales (AddExtraCounter), Corpsejack Menace (DoubleCounters), Conclave Mentor (replacement half only — death trigger deferred as OOS-LKI-Power seed, closed by next PB). HASH 15→16. Tests 2734→2745 (+11). Review PASS (3 LOW: 1 CR-citation fix 121.6→122.6, 2 false-positives non-reproducible).
- **PB-LKI-Power shipped** (`scutemob-19`, merged `12218638`). LKI source-power snapshot for SelfDies/SelfLeavesBattlefield triggers (CR 603.10a / 122.2 / 400.7). Engine: `EffectAmount::SourcePowerAtLastKnownInformation` (disc 18; disc 19 reserved for toughness variant) + `lki_power: Option<i32>` threaded through `PendingTrigger`/`StackObject`/`EffectContext`. Snapshot at `sba.rs:540` via `calculate_characteristics(state, source_id).power` BEFORE `move_object_to_zone`. 5 `GameEvent` variants extended: `CreatureDied.pre_death_power` HASHED; `AuraFellOff`/`PermanentDestroyed`/`ObjectExiled`/`ObjectReturnedToHand.pre_lba_power` NOT hashed (mirrors PB-LKI-CC LBA precedent). 21-site dispatch chain (full plumbing trace in review memo). 2 cards: Conclave Mentor death-trigger life-gain (closes PB-CD TODO), Juri Master of the Revue death-trigger damage. HASH 16→17. Tests 2745→2749 (+4 — discriminating coverage: per-card + LKI-after-zone-change discriminator + hash determinism canary). Review PASS-WITH-NITS → PASS after worker fix-phase (E1 OOS-LKI-Power-4 seed filed for AnyCreatureDies, E3 stale doc-comment line numbers removed; E2 deferred as OOS-LKI-Power-5).
- **5 OOS-LKI-Power seeds filed** in `memory/primitives/pb-retriage-CC.md`: -1 (toughness variant, disc 19 reserved), -2 (Master Biomancer ETB-replacement EffectAmount from source power), -3 (LBA hash arm symmetric extension), -4 (AnyCreatureDies LKI-power), -5 (Layer-4 animated non-creature SBA paths). Original OOS-LKI-Power closed.
- **Tests**: 2734→**2749** (+15 overall). Build/clippy/fmt clean throughout. **HASH**: 15→17 (two bumps).
- **Bookkeeping**: authoring report regenerated by PB-LKI-Power worker in fix-phase commit; coordinator-side tidy fixed a duplicated phrase ("Master Biomancer ETB-counter-from-source-power" appearing twice) in worker-written CLAUDE.md.

**Not done / deferred**:
- 5 newly-filed OOS-LKI-Power seeds (-1 through -5) untouched.
- 8 prior OOS-TS/LKI seeds still untouched (OOS-LKI-3 Workhorse cost-LKI, OOS-LKI-4 AnyCreatureDies counter axis, OOS-TS-1/2/3/4, etc.).
- `TargetFilter.exclude_self` (Éomer + ~5 "for each other [type]" cards) still attractive small primitive.
- `docs/project-status.md` Card Health section still stale (canonical: `tools/authoring-report.py`).

**Next session**:
- **Master Biomancer ETB-replacement** (OOS-LKI-Power-2) — closes a freshly-filed seed; ETB-side dynamic counter count from source power; different shape from LKI snapshot (live source, not LKI). One named EDH staple.
- **`TargetFilter.exclude_self`** (Éomer) — small contained primitive.
- Or pivot: refresh `docs/project-status.md` Card Health, or tackle one of the older OOS seeds.

**Hazards** (carrying forward, mostly stable):
- **CWD-stickiness in Bash tool**: hit ONCE this session (after `cd /home/skydude/projects/scutemob/.worktrees/scutemob-18 && git log --oneline main..HEAD`, a later bare `git branch --show-current` returned the feature branch because cwd persisted). Recipe still: `cd /home/skydude/projects/scutemob && <command>` in same bash invocation. Less severe than prior session.
- **`feedback_worker_satisfy_before_signal_ready` worked** — both workers correctly ran `esm task satisfy` for all criteria this session. The auto-memory feedback rule is observably improving outcomes.
- **CLAUDE.md bloat**: file now at **407 lines** (size guard is 250). Worker updates added detailed Active Plan + Status entries for each PB. Not addressed this session (would require trimming completed PB detail to topic files); flag as future cleanup pivot.
- **Worktree "uncommitted changes (discarded)" warning** fired on both merges — final commits are clean and reviewed, safe to ignore but documented.
- **`esm task transition --attest working_branch=<short>` poisons merge** (carried forward — used full branch name both times to avoid).
- **Worker-worktree `.claude/skills/` deletion artifact** carried forward but not triggered this session (no contamination cleanup needed post-merge for either PB).
- **Carried-forward LOWs**: BASELINE-LKI-01, PB-Q4-M01, marisi stale-TODO, 11 PB-T LOWs, 5 PB-P LOWs, 1 PB-D LOW, 4×PB-CC review memo LOWs, 3×PB-LKI-CC LOWs (resolved), 3×PB-CD LOWs (1 fixed, 2 false-positives), 3×PB-LKI-Power LOWs (E1/E3 fixed, E2 deferred as OOS-LKI-Power-5).

**Commit prefix used**: worker-side `scutemob-18:` / `scutemob-19:`, `merge:` for merges, coordinator-side `chore:` for both post-collect tidies.

## Handoff History

### 2026-04-30 ~01:00–05:00 EDT (PB-TS + PB-LKI-CC chain) — W6: Primitive

- **PB-TS shipped** (`scutemob-16`, merged `68f4bfbc`). `TokenSpec.count: u32 → EffectAmount` — dynamic token count via `resolve_amount` integration at `effects/mod.rs:540-590` + `601-668` before `apply_token_creation_replacement` boundary. 4 cards re-authored: Phyrexian Swarmlord, Krenko Mob Boss, Izoni Thousand-Eyed, Chasm Skulker (reverted in fix-phase pending PB-LKI-CC). HASH 13→14. Tests +5. Review NEEDS-FIX → PASS. 4 OOS-TS seeds filed.
- **PB-LKI-CC shipped** (`scutemob-17`, merged `a2b24e42`). `EffectAmount::CounterCountAtLastKnownInformation { counter }` (disc 17) — LKI snapshot threaded `pre_death_counters → PendingTrigger.lki_counters → StackObject.lki_counters → EffectContext.lki_counters → resolve_amount`. Fix-phase E1 swept all 5 `SelfLeavesBattlefield` dispatch arms (~35 emit sites across 5 engine files). 2 cards: Chasm Skulker re-authored from PB-TS revert + Toothy Imaginary Friend retroactive correctness fix. HASH 14→15. Tests +9. Review PASS (1 HIGH + 3 LOW resolved). 2 OOS-LKI seeds filed.
- Tests 2720→**2734** (+14). New hazard: worker forgot satisfy step before signal-ready (captured in feedback memory `feedback_worker_satisfy_before_signal_ready.md`).

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

