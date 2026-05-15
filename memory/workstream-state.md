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
| W6: Primitive + Card Authoring | — | available | — | **7 PBs shipped 2026-05-15 in coordinator-dispatched chain** — `scutemob-22` PB-XS-E (Enters-trigger sibling of PB-23, HASH 19→20, 3 cards, +11 tests) merged `e0f3f5b0`; `scutemob-23` OOS-EWC-2 (Golgari Grave-Troll pure card-authoring, +4) merged `87c3a306`; `scutemob-24` PB-XA (`TargetFilter.is_attacking` enforcement at 10 sites, +10) merged `b42e06bb`; `scutemob-25` PB-EAT (`ReplacementModification::EntersAsAdditionalType`, HASH 20→21, Master Biomancer Mutant, +5) merged `75302138`; `scutemob-26` PB-XA2 (`is_blocking`/`is_tapped`/`is_untapped`, HASH 21→22, Eiganjo, +17) merged `f3905b62`; `scutemob-27` OOS-XS-E-1 (audit-only, +0) merged `a78e8481`; `scutemob-28` PB-EWC-D (`ObjectFilter::CreatureControlledByOfSubtype` + `bind_object_filter` fix, HASH 22→**23**, Dragonstorm Globe, +7) merged `0a84badc`. Tests 2764→**2818** (+54). HASH 19→23 (4 schema bumps). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

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

## Previous Handoff (preserved for chain context)

**Date**: 2026-05-14 (worker session, dispatched — last PB before the 2026-05-15 7-PB chain)
**Workstream**: W6: Primitive (PB-XS)
**Task**: `scutemob-21` PB-XS — `TargetFilter.exclude_self` for "another target X" spell/ability target selection (CR 109.1 / 601.2c). **Merged `dbc17896` 2026-05-14.**

**Completed**:
- **Engine surface**: `TargetFilter.exclude_self: bool` with `#[serde(default)]` at `card_definition.rs:2639`; doc comment mirrors the `is_token`/`is_attacking` "MUST be checked at each call site" pattern. HASH 18→**19**; field hashed in `state/hash.rs::HashInto for TargetFilter`.
- **Enforcement (per-call-site validator)**: `casting::validate_object_satisfies_requirement` extended to all four filter-bearing TargetRequirement variants via existing `self_id: Option<ObjectId>` parameter threading. Activated-ability call site at `abilities.rs:344` switched from `validate_targets` → `validate_targets_with_source`. Trigger auto-target picker gained `obj.id != trigger.source` checks across 6 sites (battlefield/graveyard scan, top-level + UpToN-inner).
- **9 card defs updated**: roalesk_apex_hybrid, samut_voice_of_dissent, torch_courier, brash_taunter, ezuri_renegade_leader, oath_of_teferi, elderfang_ritualist, dour_port_mage, thousand_faced_shadow. Four migrated from bare `TargetCreature` → `TargetCreatureWithFilter { exclude_self: true }`.
- **10 new tests** in `tests/primitive_pb_xs.rs` including F-1 (positive Elderfang death-trigger graveyard auto-target picker) + F-2 (negative companion). 10 PB hash canary tests bumped 18→19u8.
- **5 OOS seeds filed** (OOS-XS-1..5) — XS-2 (`is_attacking`), XS-5 (Enters-trigger sibling), and the EWC-1/Mutant-half routing all landed in the follow-on 2026-05-15 chain.
- **Review**: `primitive-impl-reviewer` verdict NEEDS-FIX → CLEAN. 1 HIGH (tautological test) + 6 LOW. HIGH fixed by replacing the synthetic test with F-1/F-2 real-trigger discriminators.
- **Tests**: 2754→**2764** (+10). HASH: 18→19.

**Commit prefix used**: worker-side `scutemob-21:`, `merge:` for the merge commit.

## Handoff History

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

### 2026-04-29 ~01:00–05:00 EDT (5-PB autonomous chain — PB-SFT + PB-CC umbrella) — W6: Primitive

- **Phase A re-triage** (`scutemob-8`/`scutemob-9`): PB-SFT verdict PROCEED — FIELD-ADDITION (gap on Effect not Cost); PB-CC verdict UMBRELLA-OF-MICRO-PRIMITIVES (4 micro-PBs).
- **Wave 1 parallel** (`scutemob-10` + `scutemob-11`): PB-SFT (`Effect::SacrificePermanents.filter`) + PB-CC-W (Mossborn Hydra Landfall wire-up). 7+ cards re-authored (Fleshbag/Merciless/Butcher/Dictate/Grave Pact/Liliana DH-4/Blasphemous Edict/etc.).
- **Wave 2 sequential** (`scutemob-12`/`scutemob-13`/`scutemob-14`): PB-CC-B (`TargetFilter.has_counter_type` + Armorcraft Judge), PB-CC-C (`LayerModification::ModifyPower/ToughnessDynamic` — Fuseling deferred Option B), PB-CC-A (`EffectAmount::PlayerCounterCount` — Vishgraz deferred Option B; same trap).
- Tests 2689→2716 (+27). HASH bumped 4×. Pushed `051442bd..fd6c8e6a`.
- **Discovery**: two CDA target cards hit deeper architectural gap — `ModifyBothDynamic`-style substitution locks value at registration but CR 611.3a requires continuous re-eval. Filed PB-CC-C-followup seed (later shipped as `scutemob-15`).

