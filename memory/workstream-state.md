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
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending. **M11-local ACTIVE (web-first)**: S1 `LocalGame` shipped (`scutemob-147`); S2+ HELD until PB-DP suite done. Plan: `memory/m11-session-plan.md` |
| W3: LOW Remediation | — | available | — | LOW Sweep campaign COMPLETE 2026-05-16 (`scutemob-31..38`): 36 LOWs closed, LOW-OPEN 45→6. 6 remain (honestly deferred). Plan: `memory/archive/2026-07/low-sweep-plan.md` (archived 2026-07-18). |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | scutemob-149 | ACTIVE (worker in flight) | worker | **PB-DP suite ACTIVE** (user directive 2026-07-26: DP1..DP10 before RS5/M11-S2; queue `docs/audits/decision-point-audit.md` §8): **PB-DP1 dispatched** (`scutemob-149`, priority-to-actor CR 117.3c). RS queue: RS1..RS4 SHIPPED (`scutemob-143..146`), **HELD after RS4** pending the DP suite + seed re-rank (OOS-M11-1/2/4, OOS-RS3-1, OOS-RS2-1, OOS-RS4-1/2/4 vs RS5..RS11). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-07-26 (oversight session — RS resume, parallel-track expansion, DP pivot; /eot same day)
**Workstream**: W6 (PB-RS4 → **PB-DP suite**) + M11-local playability track (opened)
**Task**: 3 parallel workers dispatched+collected (`scutemob-146/147/148`); PB-DP1 (`scutemob-149`) **in flight at close**.

**Completed**:
- **PB-RS4** (`scutemob-146`, merge `9419d0e9`): face-aware residuals — **OOS-RS-3 + OOS-OS4-2 fully closed**; `deregister_face_statics` 1→all 10 families via `remove_one_registration` + source-scan parity gate; 4th same-root-cause Saga-sweep deviation found in planning, fixed; 0 flips, 2 integrity repairs, 17 fail-before/pass-after probes; PROTOCOL 27 / HASH 63; seeds OOS-RS4-1/2/4 filed.
- **M11-local kickoff** (`scutemob-147`, merge `375ffb67`): user decisions recorded 2026-07-26 (**M11-local first, WEB-FIRST**; `memory/decisions.md`); roadmap restructured (M11-local defined sans networking, M10→pre/a/b, M12→continuous track; strategic-review action items 9/9 closed); `memory/m11-session-plan.md` (8 sessions; **no new wire variants the whole milestone**); **S1 SHIPPED** — `LocalGame` steppable human bridge in `crates/simulator`, `GameDriver::run_game` re-expressed on it, 10 tests, 4 API hazards fixed in review. **S2+ HELD until DP suite done.**
- **Decision-point surface audit** (`scutemob-148`, merge `06bb165c`): `docs/audits/decision-point-audit.md` — DP-1..DP-32; **277/1,139 Complete defs (24.3%) contain an engine-made choice**; 5 Tier-0 correctness findings (**DP-1 priority-after-cast CR 117.3c ~20 sites**, DP-2 mulligan content no-op, DP-4 `min_modes` bypass on 3 Complete cards, DP-5 WouldDraw draw destroyed, DP-10/11 costs checked-never-charged); ranked **PB-DP1..DP10**; M11-loop verdict: `DecisionKind`'s 5 variants are the *complete* reachable set — S3/S5/S7 recommendations 1-11 in §9.4.
- Coordination: OOS-M11-3 ID collision resolved (audit's trigger-target seed → **OOS-M11-4**, `ea5cb314`); `memory/ui-feedback.md` ledger created (user endorses legal-target picker; drag-arrows demoted); combined main verified green post-3-merge (full `cargo test --all`, pipefail-checked); CLAUDE.md 3-way Current State conflict reconciled at 147 collect (took branch structure + re-applied RS4 facts).
- **PB-DP1 DISPATCHED** (`scutemob-149`): priority-to-actor fix, CR 117.3c.

**Not done / deferred**:
- **USER DIRECTIVE 2026-07-26: run PB-DP1..DP10 to completion before RS5 and M11-S2.** Chain sequentially (shared engine files); DP7..DP9 planned together (pending-decision-that-blocks machinery — audit §8 sequencing note); DP10 gate-widening closes.
- Unranked seed pool for the post-suite re-rank: OOS-M11-1/2/4, OOS-RS3-1, OOS-RS2-1, OOS-RS4-1/2/4, vs RS5..RS11.
- `scutemob-127` (abilities-corpus distillation) still backlog.

**Next session candidates**:
- **Collect PB-DP1** (`scutemob-149` — check `esm task get` / worker tab), then chain PB-DP2..DP10 per `docs/audits/decision-point-audit.md` §8, marking each row shipped there at collect.
- After the suite: re-rank RS queue vs seed pool; resume M11 S2 with audit §9.4 recommendations in the dispatch brief.

**Hazards** (carrying forward):
- **DP1 has the suite's widest test blast radius** — many tests/scripts encode active-player priority; every update must cite CR 117.3c; reviewer must catch scripts silently keeping the inversion (PB-RS1 precedent).
- Auto-memory MEMORY.md gotcha "CastSpell resets priority to ACTIVE player" becomes **stale the moment PB-DP1 lands** — flip it at collection.
- Parallel workers editing CLAUDE.md Current State will conflict at collect — expected; coordinator reconciles (branch structure + latest facts).
- Audit §10 re-audit triggers are live: new Effect choice-fields, new Commands, `Zone` API changes, Complete-count jumps.

**Commit prefix used**: merge: / chore: / W6-prim: / scutemob-147:

## Previous Handoff (preserved for chain context)


**Date**: 2026-07-19..20 (oversight session — autonomous coordinator chain, user-directed "stop dispatching after PB-RS3"; /eot 2026-07-26)
**Workstream**: W6 (rider-seed queue PB-RS1..RS11) — **RS1..RS3 SHIPPED, QUEUE PAUSED**
**Task**: rider-seed mini-triage + first three RS batches dispatched/collected (`scutemob-142..145`). Final merge `b1c21909`, close-out `52b59154`.

**Completed**:
- **Rider-seed mini-triage** (`scutemob-142`, `6f50b7f7`): 8 briefed seeds → 11 OS-series IDs (OOS-OS10-1 phantom, OOS-OS7-3 never filed), OOS-OS4-1 restored, **6 new seeds filed (OOS-RS-1..6); 4 correctness-class findings outranked every filed seed**, 2 live-wrong on `Complete` cards. Plan: `memory/primitives/rider-seed-triage-2026-07-19.md` (queue R1..R11).
- **PB-RS1** (`scutemob-143`, `56697a00`): library top/bottom inversion — `Zone::top_n` shared helper across Scry/Surveil/RevealAndRoute/LookAtTopThenPlace (+ a 5th inverted read caught in review); bottom-writes rerouted; camp A (top=last) CR-confirmed by probe; 41-card roster repaired; 5 golden scripts + 2 fixtures + 1 stale-convention test reconciled; no wire bump; OOS-RS1-1 filed (`ZoneTarget::Library` position inert — muxus still gated).
- **PB-RS2** (`scutemob-144`, `86176ff7`): hybrid/Phyrexian pips in activated+mana abilities now charged — `ActivateAbility`+`TapForMana` schema fields (PROTOCOL 26→**27**); flatten relocated to `card-types` as shared method; fail-loud residue guard; simulator non-suicidal payment plans; **birthing_pod inert→Complete (OOS-OS8-1 CLOSED)**; 7 filter lands stop being free (stay `known_wrong`); self-caught CR 119.4 combined-life bug + pre-existing casting.rs 119.4 hole fixed; OOS-RS2-1 filed (TurnFaceUp is the 4th unrouted payment site).
- **PB-RS3** (`scutemob-145`, `b1c21909`): card-def `AtBeginningOfCombat` sweep in `begin_combat` (5th copy of proven sibling template); **3 flips** (loyal_apprentice, siege_gang_lieutenant, probe-earned goblin_rabblemaster — "needs new must-attack GameRestriction" was misframed) + helm_of_the_host integrity repair (explicit `Complete`); mirage_phalanx note honest-amended; no wire bump; OOS-RS3-1..4 filed (**RS3-1 rankable** — CardDefETB sweeps skip queue-time intervening-if, CR 603.4; helper `check_intervening_if` already exists).
- **Totals**: coverage 62.9% → **63.1%** (1,139/1,804); PROTOCOL 26→**27** / HASH **63**; OOS-RS-1, OOS-RS-2, OOS-OS8-1, OOS-OS9-1 all CLOSED; every review clean or fixed (0 HIGH across all three).

**Not done / deferred**:
- **Queue PAUSED after R3 by user.** R4..R11 undispatched; OOS-RS3-1 (rankable insert) + OOS-RS2-1 (cheap rider) filed but unranked.
- OOS-RS3-2 (8 effectively-Complete defs textually admitting unimplemented behavior — emeria_the_sky_ruin is live-wrong; re-marking pass, not a primitive).
- scutemob-127 (abilities-corpus distillation) still backlog; dormant/defer backlog; retired-scripts worklist; M10.

**Next session candidates** (highest-yield first):
- **Resume the RS queue at R4** (face-aware residuals, OOS-RS-3) per the §5 banner in `rider-seed-triage-2026-07-19.md` — but first weigh inserting **OOS-RS3-1** (5 call-sites of an existing helper, correctness) and riding **OOS-RS2-1** (4th payment-site routing, materially smaller than R2).
- Pull forward emeria_the_sky_ruin from OOS-RS3-2 (the one live-wrong member) or run the full re-marking pass.
- scutemob-127, M10 per strategic review, or retired-scripts worklist.

**Hazards** (carrying forward):
- All five prior hazards below still stand (attestation verbatim, poll-loop cap, `esm update` clobber risk, resume state-resync, yield unreliability both directions).
- **Probe-first pays**: RS1's probe settled the fix direction; RS3's probe overturned a card's stated blocker and earned an unplanned flip. Keep step-0 probes in every RS brief.
- **Reviews keep catching real misses** (RS1's 5th inverted read, RS2's 12 findings, RS3's seed-scope corrections) — never skip the reviewer pass even on "template" PBs.

**Commit prefix used**: worker `scutemob-N:`/`W6-prim:`, `merge:`, coordinator `chore:`.

---


**Date**: 2026-07-19 (oversight session — fully autonomous coordinator chain, user-directed "stop after PB-OS11")
**Workstream**: W6 (PB-OS queue) — **QUEUE COMPLETE**
**Task**: PB-OS4..OS11 + OS4b dispatched/collected (`scutemob-130`/`134`..`141`), audit-#2 DOCB-1..3 executed (`131` inline, `132`/`133` dispatched). Final merge `bd220b00`, close-out `14497516`.

**Completed**:
- **PB-OS4** (`scutemob-130`, `7ee96913`, SHIPPED NARROWED): `ExileSourceAndReturnTransformed` (CR 400.7/712.18); reviewer HIGH → OOS-OS4-2; edgar UN-authored (would ship wrong state); PROTOCOL 18→19 / HASH 55→56.
- **PB-OS4b** (`scutemob-134`, `77d411a0`, correctness insert): face-aware ability gathering wire-neutral; **docent + bloodline Complete-but-wrong → verified Complete by execution**; **OOS-OS4-2 was only PARTIALLY closed here** (3 CR 712.8d/e residuals survived — `replacement.rs:1180-1191`, `:1907-1913`, `face.rs:118-148`; tracked as OOS-RS-3, `scutemob-142`) — ✅ **now FULLY CLOSED by PB-RS4 (`scutemob-146`, 2026-07-26)**; OOS-OS4-3 filed.
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
- ~~**PB-RS4**~~ ✅ **SHIPPED `scutemob-146`** (2026-07-26): face-aware residuals — **OOS-RS-3 CLOSED, and with it OOS-OS4-2 is fully closed**. Both `replacement.rs` gathering sites (`apply_self_etb_from_definition`, `register_permanent_replacement_abilities`) now read `def.effective_abilities(entering_is_transformed)` via a live `fizzle_object` read rather than a threaded parameter (justified per-call-site: `is_transformed` is written in exactly two places, both strictly before every consumer). `deregister_face_statics` extended from `Static`-only to **all ten** registered families through a new `remove_one_registration` inverse helper mirroring `register_static_continuous_effects` arm for arm — the AC's "justified subset + re-filed seed" escape hatch was **not** needed — plus a source-scan drift gate (`tests/core/face_dereg_parity.rs`) that fails the build if a family is added to one function and not the other. **A fourth deviation was found during planning and fixed**: the CR 714.3b precombat-main Saga lore sweep and `fire_saga_chapter_triggers`'s `ability_index` namespace both read the front face, the latter disagreeing with all **8** of its consumers (docs had claimed 3) — same CR 712.8d/e root cause, and the only defect in the batch reachable with a shipped card def (`fable_of_the_mirror_breaker`, which stays `partial`). **0 flips** as predicted, 2 integrity repairs; 17 fail-before/pass-after probes with verbatim failure messages recorded; review 0 HIGH / 1 MEDIUM / 11 LOW, all 12 dispositioned (the MEDIUM — degenerate probe field values — took two attempts: non-default values alone proved non-discriminating, so both tests were rebuilt around a phantom same-source entry). Also corrected a pre-existing CR miscitation (614.16a does not exist) and a false claim in the old `replacement.rs` comment (four `Complete` MDFC lands *do* declare back-face self-ETB replacements; they are unreachable only because MDFC face-selection is unimplemented). PROTOCOL 27 / HASH 63 unchanged. Seeds filed: **OOS-RS4-1** (stack craft / `ExileSourceAndReturnTransformed` never register permanent replacements or queue ETB triggers), **OOS-RS4-2** (4 `Complete` MDFC lands with permanently-unreachable back faces), **OOS-RS4-4** (transform between trigger queue and resolution can desync a CardDef ability index, CR 113.7a). Coverage unchanged at **1,139/1,804 = 63.1%**.
- **Next: R5** (Anim Pakal LKI counters, OOS-RS-4) per `rider-seed-triage-2026-07-19.md` §5 banner; weigh OOS-RS3-1 insert (queue-time intervening-if, marked **rankable**) + OOS-RS2-1 rider (TurnFaceUp unflattened cost). The R1-R3 user pause was lifted for R4.
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

## Handoff History

### 2026-07-16..17 (oversight — marker sweep + SR-33..38 + W-waves + EF triage) [rotated] — rotated to History 2026-07-26

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
