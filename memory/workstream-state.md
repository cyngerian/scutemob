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
| W6: Primitive + Card Authoring | — | available | — | **Card Authoring Campaign ACTIVE** — plan: `memory/card-authoring/campaign-plan-2026-05-16.md` (§0 recalibration 2026-07-07 authoritative). **PB-AC chain COMPLETE** — AC0..AC9 all shipped (`scutemob-41..47`, `49..52`). Clean coverage 1,006/1,748 = 57.6% (post-SR track). Next: re-read plan §0 against SR-2's machine-tracked completeness markers, then W-PB2 (~55 cards unblocked by AC4..AC6) / W-EMPTY + W-MISS derisking. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-07-08..10 (oversight session — coordinator dispatching; /eot run 2026-07-16)
**Workstream**: W6: Primitive + Card Authoring — Card Authoring Campaign, PB-AC chain close
**Task**: PB-AC4..AC9 dispatched, verified, collected (`scutemob-46/47/49/50/51/52`) — **AC chain complete (AC0..AC9)**

**Completed** (all merged to main AND pushed same-day):
- **PB-AC4** (`scutemob-46`, merge `dca25ec0`): `ModeSelection.mode_targets` per-mode targeting (CR 601.2c) + Escalate hard-reject fail-safe; UpToN verified already shipped by PB-T, not re-added. Review 0 HIGH / 1 MED fixed. Backfill 11 migrated (Casualties of War was uncastable; Cryptic Command stubs replaced) + 2 cleanups; 2 card HIGHs fixed. Tests 2940→2957; coverage 951→954.
- **PB-AC5** (`scutemob-47`, merge `0ce2c470`): Warp (CR 702.185, exile/recast), Transmute, Exert (attack-cost AND activation-cost shapes), `Cost::ExileFromHand`+`AltCostKind::Pitch`, `CounterSpell.exile_instead` add-on. Review 2 HIGH (`was_warped`+`exert` unhashed — mutation-verified fixes). Backfill 6 clean (Force of Will/Vigor/Negation trio). Tests →2984; coverage →960. Worker corrected 3 brief errors (Warp was in CR; Exert is a keyword action; DoesNotUntap wrong model for Exert).
- **PB-AC6** (`scutemob-49`, merge `0628807e`): first-main + postcombat-main generic CardDef sweeps, `WhenBecomesTarget` at announcement, 5 Conditions, 3 PlayerState trackers with all-player turn-boundary resets (plan's `spells_cast_this_turn` reuse rejected — wrong game state). Review 0 HIGH / 0 MED. Backfill 6 clean + marker-correction sweep over 13 blocked cards. Tests →3009; coverage →965.
- **PB-AC7** (`scutemob-50`, merge `2f214906`): `SetCreatureTypes`/`SetCardTypes` Layer 4 (review HIGH = CR 205.1a correlated-subtype removal; + CR 613.8 payload-aware `depends_on` arms), `spell_subtype_filter`. LoseAbilities + one-shot Layer-4 override verified already-expressible. Backfill 5 clean. Tests →3035; coverage →970.
- **PB-AC8** (`scutemob-51`, merge `a2aea440`): `CantAttackOwner`, `CantBeSacrificed` (cost-payment AND delayed-trigger choke points — review MED caught half-wiring, the verify-full-chain failure mode), `Effect::WinGame` w/ mandatory 4p test (worker corrected brief's inverted CR 104.3h). 2/5 briefed primitives already existed. Backfill 3 clean — all mis-triaged, blocked only by stale markers. Tests →3062; coverage →973.
- **PB-AC9** (`scutemob-52`, merge `a4750cdb`): `WheelHand` + `SetNoMaximumHandSize` (unbriefed co-blocker); 3/5 briefed already existed (`RollDice` d20+results, `DoubleTokens`, `AddManaFilterChoice`); **token doubling rewired 2→13/13 creation sites** (doublers silently failing on Squad/Offspring/Myriad/Embalm/Eternalize/Encore/Living Weapon/Gift/Investigate/Amass — invisible to any marker). Review MED = Amass bypassing `apply_counter_replacement` (pre-existing, CR 701.47a). Backfill 11 clean (doubler trio, wheels, d20 dragons); 1 HIGH = Reforge the Soul stale Miracle marker. Tests →**3090**; coverage →**983 (56.2%)** at chain close.
- Coordinator chores: filed `scutemob-48` (registry gate, invariant #9, from AC5 worker flag — since **CLOSED by SR-2**); fixed `/implement-primitive` skill wip path to `memory/primitives/primitive-wip.md` (`7c88a1be`) after the stale path caused a runner to clobber AC3's close-out; CLAUDE.md snapshot after each collection.

**Not done / deferred**:
- W-PB2 (~55 cards unblocked by AC4..AC6) not started; W-EMPTY / W-MISS derisking batches not run.
- AC8+AC9 workers both recommended a campaign-wide stale-marker sweep — likely superseded by SR-2's machine-tracked completeness markers (68 inert / 627 partial / 47 known-wrong); reconcile before scheduling.

**Next session candidates** (highest-yield first):
- Re-read campaign plan §0 against the SR-2 completeness-marker system (and `docs/sr-remediation-plan.md`), then dispatch **W-PB2** — or a marker-reconciliation batch first if the plan still assumes comment markers.
- W-EMPTY / W-MISS 12-card derisking batches (plan §0.4).

**Hazards** (carrying forward):
- **Recon-first is mandatory**: AC7/AC8/AC9 each found 2-3 of the brief's primitives already existing under other names — a grep proving absence is only as good as the name you guess; planners must verify under multiple names before building.
- HashInto omissions were review HIGHs twice (AC1, AC5); engineered out from AC6 on by baking mutation-verified hash tests into acceptance criteria — keep that criterion in every engine-touching brief.
- Coordinator briefs are advisory: workers correctly overturned brief claims 3x (Warp-in-CR, Exert shape, CR 104.3h). Keep verify-before-implement in every brief.
- `cargo build --workspace` does NOT compile test targets — sub-agents falsely reported green twice; workers must re-run all gates themselves. (Post-SR-3 note: `build --workspace` is ALSO the only gate proving the GameState seal — run both.)
- CR file bare `\r` line endings: rule-number greps silently match nothing — use the mtg-rules MCP, never grep.
- Still applies: strictly-sequential dispatches (~30G `target/` per worktree); `esm task unlock` right after in_progress; phantom `.claude/skills/*` deletions never committed.

**Commit prefix used**: worker `W6-prim:`/`W6-cards:`, `merge:` for merges, coordinator `chore:`.

---

## Previous Handoff (preserved for chain context)

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

## Handoff History

### 2026-07-07 (coordinator session — campaign launch) — W6: Primitive + Card Authoring

- **Campaign triage + 2 derisking batches + PB-AC0** (`scutemob-39..42` + chore, 5 merges): DSL gap audit + campaign plan written (~435 authorable-now estimate, falsified next session at 17% measured clean); W-NOW-1 batches 1-2 (4 CLEAN / 13 PARTIAL / 7 BLOCKED over 24 cards); **PB-AC0** creature-ETB filter forwarding (`df997fd2`, +13 tests, 2860→**2873**); `authoring-report.py` taught to count `// ENGINE-BLOCKED` (true clean 928 / 53.1%). Deferred at close: origin 14 ahead (pushed next session), plan recalibration (done next session as §0).

### 2026-05-16 (coordinator session — LOW Sweep campaign) — W3: LOW Remediation

- **8 fix sessions** (`scutemob-31..38`, plan `memory/low-sweep-plan.md`): 36 of 42 open LOWs closed, LOW-OPEN 45→**6** (4 M10-gated: MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). New DSL: `Effect::DestroyAndReanimate`, `Effect::PreventNextUntap`, `ProtectionQuality::{FromSuperType, FromName, FromPlayer}`; BASELINE-LKI-01 fixed (`pre_death_characteristics` snapshot, CR 603.10a/613.1e — audit `memory/primitives/lki-completeness-audit.md`, which filed +1 follow-up LOW: continuous-effect-granted triggered abilities lost via SelfDies). Tests 2819→**2860**; HASH 24→**27**. Origin hazards recorded: 4 parallel worktrees filled the disk to 100% (hence strictly-sequential rule); attestation-vs-real-branch-name drift causes false `esm worktree check` conflicts.

### 2026-05-15 (coordinator session 2 — 2-PB chain) — W6: Primitive

- **2 PBs shipped**: `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test, merged `e7d01fda`); `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED, merged `184162df`). Tests 2818→**2819**. High-confidence primitive backlog exhausted at session end.

### 2026-05-15 (7-PB autonomous chain) — W6: Primitive

- **7 PBs shipped** (`scutemob-22..28`): PB-XS-E (ETB `exclude_self`, HASH 19→20), OOS-EWC-2 (Golgari Grave-Troll), PB-XA (`is_attacking` enforcement, 10 sites), PB-EAT (`EntersAsAdditionalType`, HASH 20→21), PB-XA2 (`is_blocking`/`is_tapped`/`is_untapped`, HASH 21→22), OOS-XS-E-1 (audit-only, 0-yield), PB-EWC-D (`CreatureControlledByOfSubtype` + Dragonstorm Globe, HASH 22→23). Tests 2764→**2818** (+54). All worker-delegated; PB-XA2 worker self-collected without pre-merge reviewer (post-merge review 0 HIGH/0 MEDIUM/3 LOW). Full detail in git history + `memory/primitives/`.

### 2026-05-14 (PB-XS) — W6: Primitive

- **PB-XS shipped** (`scutemob-21`, merged `dbc17896`). `TargetFilter.exclude_self: bool` for "another target X" spell/ability target selection (CR 109.1 / 601.2c). Per-call-site validator enforcement across 4 filter-bearing TargetRequirement variants + 6 trigger auto-target-picker sites. 9 card defs updated (4 migrated bare `TargetCreature` → `TargetCreatureWithFilter`). HASH 18→**19**. Tests 2754→**2764** (+10). Review NEEDS-FIX → CLEAN (1 HIGH tautological test replaced with F-1/F-2 real-trigger discriminators). 5 OOS-XS seeds filed.



