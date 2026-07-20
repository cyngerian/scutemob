# Workstream State

> Coordination file for parallel sessions. Read by `/start`, claimed by
> `/start-work`, released by `/eot`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | LOW Sweep campaign COMPLETE 2026-05-16 (`scutemob-31..38`): 36 LOWs closed, LOW-OPEN 45→6. 6 remain (honestly deferred). Plan: `memory/archive/2026-07/low-sweep-plan.md` (archived 2026-07-18). |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | **PB-OS queue ACTIVE** (`memory/primitives/oos-retriage-plan-2026-07-18.md`, from OOS retriage `scutemob-115`): **OS1..OS3 SHIPPED** (`scutemob-116`/`128`/`129`), **OS4 SHIPPED NARROWED** (`scutemob-130`, PROTOCOL 18→19 / HASH 55→56), **OS4b SHIPPED** (`scutemob-134` — face-aware ability gathering, OOS-OS4-2 closed; wire-neutral 19/56), **OS5 SHIPPED/in-review** (`scutemob-135` — dynamic relative-count `EffectAmount` `OtherAttackersSharingCreatureType`, OOS-EF4-1 closed; shared_animosity + goblin_piledriver →Complete; rabblemaster/muxus partial-improvements; **PROTOCOL 19→20 / HASH 56→57**). Next: **OS6** (or OOS-OS4-3 edgar micro-PB). Prior EF queue COMPLETE (`scutemob-99..114`). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-07-19 (oversight session — fully autonomous coordinator chain, user-directed "stop after PB-OS11")
**Workstream**: W6 (PB-OS queue) — **QUEUE COMPLETE**
**Task**: PB-OS4..OS11 + OS4b dispatched/collected (`scutemob-130`/`134`..`141`), audit-#2 DOCB-1..3 executed (`131` inline, `132`/`133` dispatched). Final merge `bd220b00`, close-out `14497516`.

**Completed**:
- **PB-OS4** (`scutemob-130`, `7ee96913`, SHIPPED NARROWED): `ExileSourceAndReturnTransformed` (CR 400.7/712.18); reviewer HIGH → OOS-OS4-2; edgar UN-authored (would ship wrong state); PROTOCOL 18→19 / HASH 55→56.
- **PB-OS4b** (`scutemob-134`, `77d411a0`, correctness insert): face-aware ability gathering wire-neutral; **docent + bloodline Complete-but-wrong → verified Complete by execution**; **OOS-OS4-2 only PARTIALLY closed** (3 CR 712.8d/e residuals survive — `replacement.rs:1180-1191`, `:1907-1913`, `face.rs:118-148`; tracked as OOS-RS-3, `scutemob-142`), OOS-OS4-3 filed.
- **PB-OS5..OS11** (`scutemob-135`..`141`): relative-count amount (shared_animosity, piledriver); flip-condition sub-batch (delver, legions_landing, thaumatic_compass); defending-player filter (silumgar); LookAtTopThenPlace + min_cmc (birthing_ritual, growing_rites); YouControlYourCommander (skyhunter); distinctness + Jitte trigger (umezawas_jitte); RemoveCounter lowering + filtered-attack trigger (workhorse, anim_pakal, kreat, hermes + 2 backfills). PROTOCOL 19→**26**, HASH 56→**63**, one justified bump per PB.
- **DOCB-1..3** (audit #2): state resync (inline), skill rewiring off retired docs + /start-work RETIRED + /collect state-sync step (`132`), 10-item polish (`133`, 1 item coordinator-fixed).
- **Totals**: coverage 62.1% → **62.9%** (1,135/1,804); tests 3476 → **3560+**; +18 flips, 4 Complete-but-wrong made right, 3 known_wrong redeemed; 2 seed premises falsified-and-reframed against oracle; every review clean or fixed.

**Not done / deferred**:
- ~~**Rider-seed mini-triage** (8 seeds)~~ ✅ **DONE `scutemob-142`** (2026-07-19) → `memory/primitives/rider-seed-triage-2026-07-19.md`. Was 8 seeds; actually 11 OS-series IDs (OOS-OS10-1 phantom, OOS-OS7-3 never filed) + OOS-OS4-1 restored + **6 new seeds filed (OOS-RS-1..6), of which 4 are correctness-class and outrank every previously-filed seed**, 2 live-wrong on `Complete` cards. Ranked queue R1..R11; first dispatch **PB-RS1 (OOS-RS-1, library top/bottom inversion)** fully specified.
- scutemob-127 (abilities-corpus distillation) still backlog; dormant/defer backlog (`oos-retriage-plan` §1c/§1d); retired-scripts worklist; M10.

**Next session candidates** (highest-yield first):
- ~~**PB-RS1**~~ ✅ **SHIPPED `scutemob-143`** (2026-07-19, merge `56697a00`): camp A (top=last) CR-confirmed; `Zone::top_n` shared helper across all 4 arms + a 5th inverted read caught in review; bottom-writes rerouted; 41-card roster repaired via `all_cards()` (grep's 47 over-counted); 5 golden scripts + 2 harness fixtures + 1 stale-convention test reconciled; PROTOCOL 26 / HASH 63 unchanged; OOS-RS1-1 filed (`ZoneTarget::Library` position inert — muxus/OOS-OS8-2 STILL gated).
- ~~**PB-RS2**~~ ✅ **SHIPPED `scutemob-144`** (2026-07-20, merge `86176ff7`): `Command::ActivateAbility` **and** `TapForMana` gain hybrid/Phyrexian payment fields (PROTOCOL 26→**27**, machine-forced; HASH 63); flatten relocated to `card-types` as shared method; fail-loud residue guard in `can_spend`/`spend`; simulator plan-resolution (non-suicidal, CR 104.3b); **birthing_pod inert→Complete (OOS-OS8-1 CLOSED)**; 7 filter lands stop being free (stay `known_wrong`, output-side mode issue remains); self-caught CR 119.4 combined-life bug + pre-existing casting.rs 119.4 hole fixed; coverage **1,136/1,804 = 63.0%**.
- ~~**PB-RS3**~~ ✅ **SHIPPED `scutemob-145`** (2026-07-20, merge `b1c21909`): card-def `AtBeginningOfCombat` sweep in `begin_combat` (fifth copy of the proven sibling template); **3 flips** (loyal_apprentice, siege_gang_lieutenant, + probe-earned goblin_rabblemaster — its "needs new must-attack GameRestriction" blocker was misframed, all pieces existed) + helm_of_the_host integrity repair (explicit `Complete` marker); mirage_phalanx note amended (now wrong both directions, contained by `known_wrong`); PROTOCOL 27 / HASH 63 unchanged; seeds filed OOS-RS3-1..4 (RS3-1 marked **rankable** — queue-time intervening-if, CR 603.4) + OOS-RS2-1 (TurnFaceUp unflattened cost). Coverage **1,139/1,804 = 63.1%**.
- **QUEUE PAUSED by user after R3** (2026-07-20). On resume: **R4** (face-aware residuals, OOS-RS-3) per `rider-seed-triage-2026-07-19.md` §5 banner; weigh OOS-RS3-1 insert + OOS-RS2-1 rider.
- M10 per strategic review (protocol machinery battle-tested: 8 bumps this queue, all machine-forced).
- Retired-scripts worklist (61 scripts, each names its one blocker).

**Hazards** (carrying forward):
- **Attestation branch-name drift → false `esm worktree check` conflicts AND false provisioned-damage ("unknown (diff failed)")**: always attest the branch verbatim from `esm worktree create` output; `git merge-tree --write-tree` is the arbiter when the check screams.
- Harness kills long background poll loops after ~1 iteration — use the Monitor tool for worker-ready watches, not restart-churn Bash loops.
- `esm update` may clobber the DOCB-2 customizations in provisioned skills (collect/dispatch) — re-apply from `scutemob-132` branch history if doctor/update touches them.
- Pausing a queue must include state resync on resume (audit-#2 N4; now encoded in /collect step 7).
- PB yield estimates stay unreliable in BOTH directions (OS4: 4→0+narrowed; OS11: 2→6) — verify premises against oracle before building; falsified seeds are wins.

**Commit prefix used**: worker `scutemob-N:`/`W6-prim:`, `merge:`, coordinator `chore:`.

---

### 2026-07-18 late (oversight — OOS retriage → OS1..3 → DOC remediation) [rotated]

**Date**: 2026-07-18 (late — oversight session: OOS retriage → PB-OS correctness group → DOC remediation interlude)
**Workstream**: W6 (PB-OS queue) + cross-cutting doc remediation
**Tasks**: `scutemob-115` (OOS retriage → PB-OS1..OS11 queue), `116`/`128`/`129` (PB-OS1..OS3, correctness group COMPLETE), DOC-1..8 remediation (`118`/`119`/`121`/`124`/`125`/`126` done, `127` filed), audit #2 filed (`131`/`132`/`133` = DOCB-1..3). **PB-OS4 (`scutemob-130`) IN FLIGHT at handoff.**

**Completed**:
- **OOS retriage** (`scutemob-115`, `7d577171`): 65 seeds chain-verified — 23 resolved/stale (10 silently closed by EF/EWC/EAT/AC9 waves), 16 → **PB-OS1..OS11** queue, 7 defer, 24 dormant.
- **PB-OS1** (`scutemob-116`, `db49a0b2`): gain-control reverts at EOT/next-turn expiry (idle `recompute_object_controller` wired); roster 2 not 3 (karrthus `Indefinite` CR-correct); vacuous canary de-vacuoused; no wire bump.
- **PB-OS2** (`scutemob-128`, `6fe4f140`): `MayPayThenEffect` sacrifice LKI (EF-EF1-A closed); disciple_of_freyalise Complete; revert-and-rerun proof; no wire bump.
- **PB-OS3** (`scutemob-129`, `fd922b74`): WhenTappedForMana trigger kind → `CardDefETB` (targets forward); forbidden_orchard `known_wrong`→Complete (both halves composed, 4p decoy); no wire bump.
- **PB-OS6** (`scutemob-136`): DFC flip-condition sub-batch (OOS-EF5-4). *(OS4/OS4b/OS5 shipped between OS3 and this — see CLAUDE.md Current State; this handoff block predates them.)* SHIP 3→Complete: (a) delver_of_secrets (`Condition::TopCardIsInstantOrSorcery`), (b) legions_landing NEW (`Condition::YouAttackedWithNOrMore(u32)` + `PlayerState.attackers_declared_this_turn`, CR 508.4 declared-only), (g) thaumatic_compass (`Effect::RemoveFromCombat{target}` + `GameEvent::RemovedFromCombat` + shared `remove_from_combat` helper factored from `apply_regeneration`, CR 506.4). DEFER: (c) westvale→new seed **OOS-OS6-1** (multi-count sacrifice cost needs `Command::ActivateAbility` wire reshape, ~90 edits, single-card yield; kellogg_dangerous_mind rides it), (d) growing_rites→PB-OS8 (`LookAtTopThenPlace`; stays partial). Single **PROTOCOL 20→21 / HASH 57→58**. 12 execution-driven decoy tests; primitive-impl-reviewer + `/review` both clean bill. OOS-EF5-4 SHIPPED-narrowed in ef-batch §9 + OS plan §3 + queue table.
- **PB-OS7** (`scutemob-137`): defending-player-scoped continuous filter (OOS-EF3-1). SHIP 1→Complete: `silumgar_the_drifting_death` NEW→Complete via `EffectFilter::CreaturesControlledByDefendingPlayer` (DSL placeholder, substituted at `Effect::ApplyContinuousEffect` execution into `CreaturesControlledBy(ctx.defending_player)`, `None => return` — never `unwrap_or(ctx.controller)`; per-Dragon trigger, per-defender scope, -1/-1 UntilEndOfTurn, ruling 2014-11-24 stacking). **PROTOCOL 21→22 / HASH 58→59 (both machine-forced** — the plan predicted no PROTOCOL bump but `EffectFilter` was already in the wire closure since PB-EF9/v14 via `ContinuousEffectDef`; runner stopped-and-flagged, then bumped). 11 execution-driven tests (4p decoy, EOT expiry, same/diff-defender stacking, SBA, non-Dragon + planeswalker-scope negatives). DEFER: Karazikar (target-filter + goad + opp-vs-opp trigger) → **OOS-OS7-1**; pre-existing engine-wide CR 611.2c set-snapshot gap (Golgari Charm/Eyeblight Massacre share it) → **OOS-OS7-2**. OOS-EF3-1 CLOSED in ef-batch §6 + OS plan §3 + queue table. primitive-impl-reviewer + `/review` (Opus) both clean bill (all 4 ACs PASS).
- **DOC remediation** (audit `memory/doc-audit-2026-07-18.md`): CLAUDE.md 78→34KB (changelog→archive verbatim, invariants→`docs/engine-invariants.md` routed); 7 stale docs bannered, project-status RETIRED; 15 files archived (gated /cleanup, 4 commits); docs.yaml live (~20 docs stamped); auto-memory links fixed; DOC-8 ruling: abilities distillation authorized (`scutemob-127`), primitives+reviews stay. Execution report: `memory/doc-remediation-report-2026-07-18.md`.
- **Audit #2** (`memory/doc-audit-2026-07-18b.md`): remediation held; found stale next-state (this rotation fixes it) + skills wired to retired docs (DOCB-2 `scutemob-132`) + polish batch (DOCB-3 `scutemob-133`).

**In flight / next**:
- **PB-OS4** (`scutemob-130`): return-transformed DFCs (OOS-EF5-3); plan + engine change committed; PROTOCOL bump expected. At collect: strip any retired-doc writes (its skill copy predates DOCB-2 rewire), reset primitive-wip, regenerate authoring-report on main.
- **DOCB-2/3** (`scutemob-132`/`133`) gate any further PB dispatch; then **PB-OS5..OS11** per the OS plan.
- **PB-OS8** (`scutemob-138`, implement phase complete, awaiting review): `Effect::LookAtTopThenPlace`
  (disc 96, put-≤1 sibling of `RevealAndRoute`) + `TargetFilter.min_cmc_amount` (runtime floor,
  mirror of `max_cmc_amount`). `birthing_ritual` (inert→Complete), `growing_rites_of_itlimoc`
  (partial→Complete, closes PB-OS6 deferred (d)). `birthing_pod` STAYS partial — new blocker
  **OOS-OS8-1** (Phyrexian mana unsupported in activated-ability payment path). `muxus_goblin_grandee`
  re-pointed, STAYS partial — **OOS-OS8-2** (its ETB is `RevealAndRoute`, not this primitive).
  **PROTOCOL 22→23 / HASH 59→60** (both machine-forced, both types already in the SR-8 closure).
  13 new tests (`tests/primitives/pb_os8_look_at_top_then_place.rs`), all green. One unplanned
  knock-on: `min_cmc_amount` pushed `TargetFilter` over clippy's `large_enum_variant` gap for
  `Cost::Sacrifice(TargetFilter)` — fixed with `#[allow(clippy::large_enum_variant)]` on `Cost`
  (boxing would touch ~84 call sites, out of scope) matching existing precedent. Full suite +
  clippy + fmt + check-defs-fmt all clean. OOS-EF10-1 CLOSED in ef-batch §12 + OS plan §3 + queue
  table.

**Hazards** (carrying forward + new):
- **Pausing a queue must include a state resync on resume** (audit-#2 root cause: OS1 collected mid-interlude stranded its plan banner; DOCB-2 adds the process step).
- PB yield overcounting universal; latent Complete-but-wrong surfaces at PB boundaries; poll loops die at the Bash 10-min cap (use Monitor); strictly-sequential PB dispatches; version bumps machine-forced.

**Commit prefix**: worker `scutemob-N:`, `merge:`, coordinator `chore:`.

---

### 2026-07-18 (oversight session — EF queue execution) — W6 [rotated]

**Date**: 2026-07-18 (oversight session — fully autonomous coordinator chain, user-authorized "run the whole queue")
**Workstream**: W6: Primitive + Card Authoring — EF queue execution
**Task**: 16 tasks dispatched/collected (`scutemob-99..114`) — PB-EF1..EF12, EF-13 Option A, swan_song demote, Cargo.lock chore. **EF QUEUE COMPLETE.**

**Completed** (all merged to main AND pushed, every worker review passed):
- **PB-EF1** (`scutemob-99`, `6202ab81`): `exclude_self` honored at 5 executors; unplanned wire change `ActivationCost.sacrifice_exclude_self` ("sacrifice ANOTHER" inexpressible otherwise); 6 cards Complete; EF-EF1-A filed (PowerOfSacrificedCreature unset in MayPayThenEffect path).
- **swan_song demote** (`scutemob-100`, `615c4319`, coordinator one-liner) then **PB-EF2** (`scutemob-102`, `3a489f59`): `TokenSpec.recipient` (201 sites unchanged), doubling per-recipient; swan_song re-Complete, An Offer authored; retired `tokens/001` un-retired, `stack/045` wrong-owner fixed.
- **PB-EF3** (`scutemob-103`, `cae6710a`): all 30 attack/trigger enrich blocks forwarded DSL targets (were `vec![]`); kind-guarded fallback; `EffectTarget::AttackTarget` + `PlayerTarget::DefendingPlayer` (CR 506.4c/508.4) 4p-correct; 3 Complete, OOS-EF3-1.
- **EF-13 Option A** (`scutemob-101`, `0096ca65`, coordinator decision): 101 no-behaviour partials → `inert` (drifted from filed 105); registry gate + canary; headline unchanged, buckets honest (todo 554 / empty 158).
- **PB-EF3b** (`scutemob-104`, `6439d0ce`): granted Melee/Battle Cry/Annihilator triggers fire via post-layer synthesis; Adriana Complete; OOS-EF3b-1.
- **PB-EF4** (`scutemob-105`, `26421364`): `EffectFilter::TriggeringCreature` + `DealDamage.source`; **7 Complete** (beat ~4–5 est.); OOS-EF4-1.
- **PB-EF5** (`scutemob-106`, `111c4513`): `TransformSelf` through existing DFC machinery; honest yield 2+1 demote (8 of 11 DFCs double-blocked); **Battle + Sephiroth split out** (CR 310 = full subsystem, legal-but-wrong risk) → OOS-EF5-1..4; review caught thaumatic_compass fabricated ability.
- **PB-EF6** (`scutemob-107`, `359c824d`): `TargetOpponent` opponent-only validation; 3 flips + latent fell_specter self-target fix; OOS-EF6-1 (WhenTappedForMana).
- **PB-EF7** (`scutemob-108`, `104ef5ad`): modal `Activated{modes}`; sweep sized cohort at 3; Cratermaker + Cankerbloom Complete, Jitte honest 2nd blocker.
- **PB-EF8** (`scutemob-109`, `4fa6b6f2`): `Cost::ExileSelfFromHand` via mana-ability lowering; both Spirit Guides Complete.
- **PB-EF9** (`scutemob-110`, `abb92654`): `WhileYouControlSource` (CR 611.2b/c never-resumes); **engine had NO control-reversion at all — this PB built it**; OOS-EF9-1 (latent never-reverts on other durations).
- **PB-EF10** (`scutemob-111`, `3710ad9c`): 3 sub-gaps via one `SacrificedCreatureLki` (toughness LKI, runtime max_cmc, `Condition::SacrificeFired`); 3 authored + 2 bonus flips; OOS-EF10-1.
- **Cargo.lock chore** (`scutemob-113`, `e1c30acb`): main didn't build in fresh envs (untracked lock → `equivalent 1.0.2`); lock now TRACKED, `--locked` verified; EF11 carried the 9-site source fix too.
- **PB-EF11** (`scutemob-112`, `e991b237`): `WheelDraw::GreatestDiscarded` (Windfall) + `TargetSpellWithSingleTarget` (Misdirection restored).
- **PB-EF12** (`scutemob-114`, `833e54ad`): `chosen_color` on `Command::TapForMana` (coordinator decision, CR 605.3b, in memory/decisions.md); **17 defs restored** (SR-37 AddManaAnyColor family un-gated); simulator emits only legal colours; /review 0 findings.
- **Session totals**: coverage 59.8% → **62.1%** (1,065→1,117 clean, corpus 1,781→1,798); tests 3330 → **3476**; PROTOCOL 2→**18**, HASH 43→**55**; all 20 EF findings closed; CLAUDE.md snapshot chore after every collect.

**Not done / deferred**:
- 11 new OOS seeds unbatched (EF-EF1-A, OOS-EF3-1, EF3b-1, EF4-1, EF5-1..4 incl. Battle subsystem, EF6-1, EF9-1, EF10-1).
- 61 retired-scripts worklist still untouched (minus tokens/001 + stack/045, fixed en route).
- 7 EF12 candidates + assorted per-PB blocked cards held back with recorded blockers.

**Next session candidates** (highest-yield first):
- Triage the 11 OOS seeds into a new ordered batch plan (mirror the EF-triage task shape, `scutemob-98`).
- OOS-EF9-1 (control-reversion for UntilEndOfTurn/WhileSourceOnBattlefield — correctness-flavored, machinery now exists).
- Retired-scripts worklist batches; or start M10 per strategic review (protocol versioning blocker long since cleared).

**Hazards** (carrying forward):
- **PB yield overcounting is universal**: EF5 planned ~7–9, honest 2 (+1 demote); every worker re-derived its roster from `all_cards()` + activation probes — keep mandating this in briefs.
- **Latent Complete-but-wrong keeps surfacing at PB boundaries**: delver (never transforms), fell_specter (self-target), thaumatic_compass (fabricated ability), 4 granted-any-color rocks. New gates catch them; expect more each PB.
- Untracked build inputs bite: Cargo.lock now tracked; if a fresh env breaks again, suspect another floating input, not code.
- Worker kitty-tab cost/time display freezes while a subagent runs — judge liveness by subagent token counter or worktree git status, not the header.
- Poll loops die silently at the Bash 10-min cap — always restart from the state file; a `killed` notification is expected, not an error.
- Still applies: strictly-sequential dispatches (shared hot files + wire bumps); unlock right after in_progress; version bumps machine-forced with history rows appended.

**Commit prefix used**: worker `scutemob-N:`, `merge:` for merges, coordinator `chore:`.

---

## Previous Handoff (preserved for chain context)

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

## Handoff History

### 2026-07-08..10 (oversight session; /eot 2026-07-16) — W6: PB-AC chain close (AC0..AC9 complete)

- PB-AC4..AC9 dispatched/collected (`scutemob-46/47/49/50/51/52`).
- **PB-AC4** (`dca25ec0`): `ModeSelection.mode_targets` per-mode targeting (CR 601.2c) + Escalate fail-safe; backfill 11 migrated. Tests 2940→2957.
- **PB-AC5** (`0ce2c470`): Warp, Transmute, Exert (both shapes), `Cost::ExileFromHand`+Pitch, `CounterSpell.exile_instead`; 2 HIGH unhashed-field fixes. Tests →2984.
- **PB-AC6** (`0628807e`): main-phase sweeps, `WhenBecomesTarget`, 5 Conditions, 3 PlayerState trackers. Tests →3009.
- **PB-AC7** (`2f214906`): `SetCreatureTypes`/`SetCardTypes` Layer 4 (CR 205.1a correlated-subtype HIGH; CR 613.8 depends_on). Tests →3035.
- **PB-AC8** (`a2aea440`): `CantAttackOwner`, `CantBeSacrificed` (both choke points), `Effect::WinGame` (worker corrected inverted CR 104.3h). Tests →3062.
- **PB-AC9** (`a4750cdb`): `WheelHand` + `SetNoMaximumHandSize`; **token doubling rewired 2→13/13 sites** (doublers silently failing); Reforge stale-marker HIGH → both workers recommended the marker sweep (executed this session). Tests →3090; coverage 983 (56.2%) at chain close.
- Hazards that stayed load-bearing: recon-first (2-3 primitives per PB already existed); HashInto omissions as review HIGHs (engineered out via mutation-verified hash tests in criteria); worker-overturns-brief 3×; `build --workspace` ≠ test compile but IS the seal gate; CR file bare `\r` — use MCP, never grep.

### 2026-07-08 (oversight session) — W6: PB-AC1..AC3 + plan recalibration

- **Recalibration** (`5c5dccb5`): §0 added — 4/24 clean (17%) falsified "~435 free cards"; PB-first sequencing. **PB-AC1** (`5cd9a662`): UntapAll, untap/counter triggers, once_per_turn, DoesNotUntap; 1 HIGH unhashed. **PB-AC2** (`4d819ef4`): `MayPayThenEffect` + `CounterUnlessPays` (CR 118.12). **PB-AC3** (`0bd7c7a3`): 3 EffectAmounts + `SetBothDynamic` Layer 7b; hash disc-26 collision fixed; 4 HIGH wrong-game-state PARTIALs fixed. Tests 2873→2940; coverage 951 (54.4%). Hazards: ~30G target/ per worktree → strictly sequential; false `esm worktree check` conflicts (verify merge-base); unlock after in_progress; phantom `.claude/skills` deletions.

### 2026-07-07 (coordinator session — campaign launch) — W6: Primitive + Card Authoring

- **Campaign triage + 2 derisking batches + PB-AC0** (`scutemob-39..42` + chore, 5 merges): DSL gap audit + campaign plan written (~435 authorable-now estimate, falsified next session at 17% measured clean); W-NOW-1 batches 1-2 (4 CLEAN / 13 PARTIAL / 7 BLOCKED over 24 cards); **PB-AC0** creature-ETB filter forwarding (`df997fd2`, +13 tests, 2860→**2873**); `authoring-report.py` taught to count `// ENGINE-BLOCKED` (true clean 928 / 53.1%). Deferred at close: origin 14 ahead (pushed next session), plan recalibration (done next session as §0).

### 2026-05-16 (coordinator session — LOW Sweep campaign) — W3: LOW Remediation

- **8 fix sessions** (`scutemob-31..38`, plan `memory/low-sweep-plan.md`): 36 of 42 open LOWs closed, LOW-OPEN 45→**6** (4 M10-gated: MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). New DSL: `Effect::DestroyAndReanimate`, `Effect::PreventNextUntap`, `ProtectionQuality::{FromSuperType, FromName, FromPlayer}`; BASELINE-LKI-01 fixed (`pre_death_characteristics` snapshot, CR 603.10a/613.1e). Tests 2819→**2860**; HASH 24→**27**. Origin hazards recorded: 4 parallel worktrees filled the disk to 100% (hence strictly-sequential rule); attestation-vs-real-branch-name drift causes false `esm worktree check` conflicts.

### 2026-05-15 (coordinator session 2 — 2-PB chain) — W6: Primitive

- **2 PBs shipped**: `scutemob-29` OOS-LKI-Power-3 (hash `pre_lba_power` on 4 `GameEvent` variants, HASH 23→**24**, +1 test, merged `e7d01fda`); `scutemob-30` OOS-XA2-3 (`is_nontoken` target-side audit — 0-yield, OOS-XA-3/XA2-3 RESOLVED, merged `184162df`). Tests 2818→**2819**. High-confidence primitive backlog exhausted at session end.

