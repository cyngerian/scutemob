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
| W6: Primitive + Card Authoring | — | available (PB-DP suite COMPLETE) | 2026-07-31 | **PB-OS queue COMPLETE** (OS1..OS11 + OS4b, `scutemob-116..141`). **Rider-seed queue**: RS1..RS4 SHIPPED (`scutemob-143..146`); plan `memory/primitives/rider-seed-triage-2026-07-19.md`, resume at **R5** per its §5 banner (weigh OOS-RS3-1 insert + OOS-RS2-1 rider). **The PB-DP suite now runs FIRST** (user directive 2026-07-26) — queue `docs/audits/decision-point-audit.md` §8, from the decision-point audit (`scutemob-148`). **PB-DP1 SHIPPED** (`scutemob-149`, merged `f7651bb5`): priority after cast/activate/special action goes to the ACTOR per CR 117.3c; 14 Group-A sites + 8 Group-D sites; entry priority guards added to `handle_turn_face_up`/`handle_activate_loyalty_ability`/`handle_level_up_class`; 19 tests + 15 golden scripts reconciled; PROTOCOL 27 / HASH 63 unchanged; 3,721 tests green. Seeds **OOS-DP1-1..4** filed in the audit doc **§8.1** (durable inventory for this suite — not in `primitive-wip.md`, which the next PB overwrites). Suite tasked out `scutemob-150..158` = PB-DP2..DP10. **PB-DP2 SHIPPED** (`scutemob-150`, commit `f902010f`): mulligan content no-op + bottom-to-top, **CR 103.5/103.5c** (the brief's "103.4b" is a stale cite — see the handoff below); **OOS-M11-1 CLOSED**; 4 probes; PROTOCOL 27 / HASH 63 unchanged; tests 3,721 → **3,725**. Seeds **OOS-DP2-1..6** filed in the audit doc **§8.1**. **PB-DP3 SHIPPED** (`scutemob-151`, DP-4 `min_modes` floor, **CR 601.2b/700.2a**): mode announcement is now mandatory — the fix is a **lift** of the range/duplicate/`min_modes`/`max_modes` checks out of the `!modes_chosen.is_empty()` gate, not the audit's prescribed Spree-guard mirror, so it fixed **40** modal defs (3 commands + 37 `min_modes: 1`) rather than the 3 the row predicted, plus the identical activated-ability bypass in `abilities.rs` (audit §4.2). Narrow CR 702.120a escalate exemption; `resolution.rs`'s `vec![0]` fallback **retained** (4 free-cast producers bypass `handle_cast_spell`). PROTOCOL 27 / HASH 63 unchanged; 0 card-def edits; tests 3,725 → **3,747**. Seeds **OOS-DP3-1..9** filed in the audit doc **§8.1**. **PB-DP4 SHIPPED** (`scutemob-152`, merged `799dcc0a`): DP-10 attack tax now debited (colour-correct; restricted mana excluded per CR 106.6; hybrid/Phyrexian/X tax rejected — OOS-DP4-1); DP-11 enforced as a **deadline** (auto-decline unanswered payments at `handle_all_passed`'s stack-empty branch per CR 118.12a — a priority gate would deadlock the driver); 5 Complete defs made right, 0 def edits; **OOS-DP1-1 + OOS-RS3-4 CLOSED**; seeds OOS-DP4-1..13 filed in audit §8.1; PROTOCOL 27 / HASH 63 unchanged; tests 3,747 → **3,781**. **PB-DP5 SHIPPED** (`scutemob-153`, merged `922252f7`): `pending_draws` on `GameState`, `OrderReplacements` routed by applicability; **3 emit sites fixed, not 2** (`draw_card_skipping_dredge` was a third the audit never named); also fixed a CR 614.11a sequence bug (`DrawCards{count:3}` emitted 3 unanswerable prompts and drew 0 — now one prompt, remainder stashed) and a review-found CR 616.1f loop gap in `determine_action`; **HASH 63 → 64** (gate-forced), PROTOCOL 27 unmoved; tests 3,781 → **3,797**; 0 def edits. NOTE: audit premise falsified — **0 of 1,804 defs register a `WouldDraw` replacement**, so card yield is 0; this is an engine-correctness fix + precondition for authoring the WouldDraw family. **PB-DP6 SHIPPED** (`scutemob-154`, merged `d52fe5b6`): intervening-if now evaluated at trigger-queue time across the queue paths (audit §4.8 queue-time row D→A); no wire change (PROTOCOL 27 / HASH 64); tests 3,797 → **3,809**; seeds OOS-DP6-1..10 filed in audit §8.1 (note OOS-DP6-10: the one hazard this batch INTRODUCED — A9 `WasKicked` suppression, wrong-direction, zero corpus exposure today). **No-wire block DP1..DP6 COMPLETE. PB-DP7 SHIPPED** (`scutemob-155`, merged `8f890611`): cleanup-discard is now a blocking player Command (CR 514.1); the **blocking pending-decision mechanism is proven** for DP8/DP9 reuse; **PROTOCOL 27 → 28, HASH 64 → 65** (both gate-computed); tests 3,809 → **3,837**; 2 fix cycles (18 + 6 findings; 2 HIGH: CR 800.4j dead-player entry skipping CR 514.2, out-of-step answer accepted; plus a TUI auto-pass livelock introduced-and-fixed); seeds OOS-DP7-1..12 in audit §8.1 — **OOS-DP7-11 flags that the SR-19 HashInto gate silently skips path-qualified impls** (gate-integrity seed, rankable). **PB-DP8 SHIPPED** (`scutemob-156`, trigger-target choice, CR **603.3d/601.2c/603.3b**): `Command::ChooseTriggerTargets` + `GameEvent::TriggerTargetChoiceRequired` (disc. 130) + `GameState.pending_trigger_targets` suspend `flush_pending_triggers` MID-BATCH and resume it on the controller's answer; the compliant CR 603.3d fallback survives verbatim as the exported `abilities::default_trigger_targets`, which the CALLER submits as a real Command (engine still knows nothing about seat kind). **PROTOCOL 28 → 29, HASH 65 → 66** (all five fingerprints gate-computed; `TriggerTargetOption` + `SpellTarget` enter the wire closure — a genuine type-count change, unlike DP7's). **Roster 77, not the audit's 84 nor the planner's 74** — enumerated from `all_cards()` per SR-36 and printed by a test. **0 card-def edits, 0 completeness flips**, but **2 live-wrong `Complete` cards fixed by accident** (sword_of_sinew_and_steel, elder_deep_fiend): the plan's premise that a permanent-inner `UpToN` slot 'contributed 0 targets' is FALSE — it returned `None` and the caller removed the WHOLE TRIGGER, so those cast/damage triggers had never once reached the stack. CR 601.2c makes zero targets a legal announcement. Second plan gap found and fixed: §4.1 never says who grants the priority the four guards were about to grant, so a resumed game would have had nobody holding priority — added `grant_priority_on_resume` on the entry. Consult set is **4 guards, not the ~20 DP7's row predicted**, because PB-DP1 moved priority assignment ahead of the flush (all 30 `check_and_flush_triggers` sites verified to need none). Also fixed `local_game.rs`'s latent variant-blindness (hard-coded `DecisionKind::CleanupDiscard`) — now compile-forced. tests 3,837 → **3,858**; 53+1 sentinels re-pinned across 44 files; 1 golden script corrected with CR justification. Seeds **OOS-DP8-1..10** in audit §8.1 (DP8-9/DP8-10 are new relative to the plan). **OOS-M11-4 CLOSED.** OOS-DP3-4 deliberately NOT bundled — ranked as **PB-DP8b** (OOS-DP8-7). **PB-DP9 SHIPPED** (`scutemob-157`, search/scry/surveil, CR **608.2d / 701.23a / 701.22a / 701.25a**): the engine's **first resolution-time decision channel**. `GameState.pending_effect_choice` + `GameEvent::EffectChoiceRequired` (disc. 131) → `Command::AnswerEffectChoice`, backed by an **abort-and-replay** continuation, NOT the "resumable effect-list cursor on the stack object" `pb-plan-DP7.md` §1.6 and audit §8 both prescribed — that is **impossible**, because `resolve_top_of_stack` POPS the stack object before any effect runs. Instead: clone at entry, an effect that needs an unanswered choice records the question and returns, the wrapper restores the clone **wholesale** and emits one event, the answer is banked on `GameState` and the resolution re-runs **from the top**, retracing the identical deterministic path. Consequences worth carrying: no continuation data structure at all (`Sequence`/`Conditional`/`ForEach`/`Repeat` need zero machinery — the replay re-executes them); the re-entrancy audit is **3 units, not 20** (15 of 17 production `execute_effect` callers are inside `resolve_top_of_stack` itself, one is gated by CR 605.4a, one is provably unreachable); **PB-DP8's "a guard that returns early inherits a debt" bug class does not exist here** because a total restore skipped nothing; and "the suspended object leaves the stack" is structurally unreachable rather than a live hazard. **ONE `Command` for all three effects** (CR 608.2d is one rule; 701.22a/23a/25a are three instances) — so one gate entry, one `LegalAction`, one `DecisionKind`, one harness action string, which **corrects OOS-DP8-14's prediction of three**. **PROTOCOL 30 → 31, HASH 67 → 68** (all gate-computed, histories append-only, 44 sentinel files re-pinned via the SYMBOL grep). **Roster 69 / 16 / 7, not the audit's 74 / 16 / 8** — enumerated from `all_cards()` with a RECURSIVE `Effect`-tree walk (a flat scan undercounts). **0 def edits, 0 flips.** Three in-scope correctness fixes beyond agency: CR **701.22b** (`Scry 0` was emitting `Scried{count:0}`; the surveil arm had the mirror guard, the scry arm did not), CR **400.7** (scry-to-bottom RENUMBERED every scried card and consumed `timestamp_counter`, the shuffle seed source — now `Zone::reposition_within`; sweep seeded as OOS-DP9-11), and CR **701.23d** (a quantity-only search with one candidate is determined and asks nothing). Two deliberate deviations, both argued in source and pinned by tests: **the scry/surveil defaults FLIP to the identity** (search keeps its lowest-id default byte-for-byte), and **the three new fields are EXCLUDED from `loop_detection.rs`'s fingerprint** unlike DP7's and DP8's, because they grow between replays of one resolution and could mask a CR 726 loop — recorded as **obligation (7)** on the `BlockingDecision` doc block, the first evidence that list generalises. `GameEvent::private_to()` now exists (OOS-DP8-6's declaration half; a declaration, NOT an enforcement point — nothing consumes it until M10). Benches measured against `48353a36`: `full_turn_4p` 253 → 229 µs, **no regression** from the per-resolution clone. Fallout 25 unit tests + 1 golden script, every repair CR-justified. tests 3,878 → **3,905**; seeds **OOS-DP9-1..12** in audit §8.1 (rank **OOS-DP9-3** first — `SearchLibrary` finds exactly one card, ~7 partial defs, zero new plumbing on this machinery). Merged `d65e7f1e`; on-main verified **3,910 / 0** (5 more than the branch pin — post-merge count). **PB-DP10 SHIPPED** (`scutemob-158`, decision-gate widening, **test-only** — and it **CLOSES THE PB-DP SUITE**): two new files under `crates/engine/tests/core/` — `decision_site_walk.rs` (the canonical serde walk + `ROWS`, all 22 decision sites of audit §3.1 classified **4 SERVED / 15 AUTO-CHOSEN / 2 GATED / 1 NO-DECISION**, each with the engine site that was *read* to establish the class) and `decision_gate.rs` (`BASELINE`, 97 name-keyed entries with exact row sets, + 18 tests). **The headline is a gate-integrity finding, not a feature**: every serde walk in this codebase before now (`effect_choose_gate.rs`, `pb_rs1_roster_sweep.rs`, `pb_dp9_effect_choice.rs::roster`) matched **object keys only** and is therefore blind to a **unit** `Effect` variant — `serde_json::to_value(Effect::Proliferate)` is `Value::String("Proliferate")` — so a verbatim reuse would have reported **0** for Proliferate's 25 `Complete` defs *while looking green*, the exact OOS-DP7-11 failure mode. Fixed with a two-shape walk + a `PROSE_FIELDS` denylist, pinned in both directions against the legacy walk (T2/T3). Measured: all-rows union **267** (the audit's 277 analogue), still-auto union **97**, live denominator 1,139/1,804. **Fail-closed proven end-to-end on a real def**, not just synthetically: adding `Effect::Proliferate` to `lightning_bolt.rs` reddened **two** tests naming the card, the row, the CR and the engine site; restored → green. Two hand-maintained zeros (`AddManaFilterChoice`, `TheRingTemptsYou`) that **nothing** was holding became machine-checked (the SR-33 gate bars a *different* key). **PROTOCOL 31 / HASH 68 unmoved; engine + card-types + card-defs diff vs main EMPTY; 0 def edits, 0 flips.** Review 2 HIGH / 6 MEDIUM / 6 LOW, **all 14 applied** — the HIGHs are worth carrying: (1) `BASELINE` was populated **mechanically** and the plan's class-B/class-D triage was never done, so a spot-check found two class-D defs already inside the frozen baseline (Smuggler's Copter's "you **may** draw" authored as an unconditional `Sequence`; Shambling Ghast with a permanent `-1/-1` counter, an `oracle_text` saying "enters" against a `WhenDies` trigger, and a `Decayed` keyword the printed card does not have) — seeded as **OOS-DP10-8**, not demoted; (2) **the gate can only see a decision the DSL ENCODED**, and that blind class is strictly *worse* than the one it records (**OOS-DP10-9**, and the instrument for it is an oracle-text-vs-DSL cross-check, not a variant walk). tests 3,910 → **3,928**; seeds **OOS-DP10-1..11** in audit §8.1; **closes OOS-DP7-7** (the 277-def re-derivation is now computed, printed and ratcheted every run). Audit §8 now carries a suite-COMPLETE banner and §10 an honest 3-of-8 mechanization ledger. Merged `16ffcfd0`, on-main verified 3,928/0; suite retrospective in the audit doc §8. **Next: re-rank RS5..RS11 against the unranked seeds** (OOS-DP9-3 was the previous first pick; OOS-DP10-8/-9 are new and OOS-DP10-6 is the successor queue's ranked input). |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

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

### PB-DP suite — worker close-outs appended since this handoff

**PB-DP2** (`scutemob-150`, commit `f902010f`, 2026-07-26) — **SHIPPED**. DP-2 from
`docs/audits/decision-point-audit.md` §5 (Tier 0, class D). Two edits in
`crates/engine/src/rules/commander.rs`:

- **(a) `handle_keep_hand` bottomed to the TOP.** `move_object_to_zone` is `Zone::insert` =
  `push_back`, and `Zone::top()` is `v.last()` — so the cards a player bottomed during the
  London mulligan were the next cards they drew. Now uses `move_object_to_bottom_of_zone`
  (`push_front`). Index 0 of `cards_to_bottom` ends up **above** later entries (the documented
  convention, preserved — **no reversal was needed**), and the pre-existing library including
  its top card is untouched.
- **(b) `handle_take_mulligan` never permuted.** It moved hand→library, emitted a **phantom**
  `GameEvent::LibraryShuffled`, then drew 7 off the top — the same seven cards came back,
  reversed. Now runs a real seeded Fisher-Yates `Zone::shuffle` after the moves and **before**
  both the event and the draws, so the event is no longer phantom (Architecture Invariant 4).

**Closes OOS-M11-1** (filed in `memory/m11-session-plan.md` §8 row R2), widened per audit §7 to
cover the (a) half. M11-local Session 2's pregame `redeal` workaround is no longer load-bearing
for correctness.

- **No wire change: PROTOCOL 27 / HASH 63 unmoved.** The audit §8 PB-DP2 row predicted (b)
  "needs a seed on `GameState` ⇒ HASH bump" — **falsified**. The existing `state.timestamp_counter`
  sufficed (`StdRng::seed_from_u64`, the MR-M7-17 idiom already used at three sites in
  `effects/mod.rs`), so replay determinism (SR-9b) holds with no new field. Reusable lesson:
  check for an existing in-engine deterministic seed source before concluding a permutation
  needs a caller-supplied `Command`.
- **CR-numbering correction.** ESM criterion 5519 and the audit's DP-2 row both cited
  "CR 103.4b". That is stale — live **CR 103.4b is the Vanguard starting life total**. Both the
  shuffle and the bottoming live in a single sentence of **CR 103.5**; **CR 103.5c** is the
  multiplayer free-first-mulligan adjustment. The engine's own source comments already cited
  103.5 correctly. The bottom-placement probe keeps `cr_103_4b` in its **name** so criterion
  5519 stays greppable, and carries the correction in its doc comment. Both audit rows are now
  corrected. (The one matching golden script, `commander/cc32_mulligan_to_six.json`, carries the
  same wrong cite — cosmetic, OOS-DP1-3 class, left alone; it is `review_status: "retired"` and
  does not execute, so **there was nothing to reconcile**.)
- **SR-25 note the plan did not anticipate**: the shuffle uses `expect_zone_mut(..).ok_or(..)?`
  rather than a bare `.zones.get_mut(..)`, which keeps this file's `bare_lookup_ratchet` ceiling
  at 6 while still **propagating in release builds** per MR-M9-12 (`expect_*` alone is
  `debug_assert!` + `None`, i.e. release-silent — using it would have let a release build skip
  the shuffle and re-emit the phantom event).
- **Simulator-unreachability finding**: the whole defect class is **unreachable from the
  simulator today** — `crates/simulator/src/local_game.rs:569-574` documents that mulligans
  cannot fire because `GameStateBuilder` defaults `turn_number` to 1 while the gate needs 0.
  It goes live the moment **M11-local Session 2** sets `turn_number = 0`, which is also when
  OOS-DP2-5 (bots send an empty `cards_to_bottom` unconditionally) becomes a real bug.
- **Tests**: 4 new probes in `crates/engine/tests/rules/commander.rs` —
  `test_dp2_cards_to_bottom_land_on_library_bottom_cr_103_4b`,
  `test_dp2_mulligan_actually_permutes_the_library_cr_103_5`,
  `test_dp2_mulligan_returns_a_different_hand_cr_103_5`,
  `test_dp2_mulligan_permutation_is_deterministic_cr_103_5`. 3 of 4 fail on pristine code; the
  determinism probe passes pre-fix because a no-op is trivially deterministic (its job is to pin
  the property against a future entropy-seeded regression). **3,721 → 3,725 passing / 0
  failing**; clippy `-D warnings`, `cargo fmt --check` and `tools/check-defs-fmt.sh` (1,804
  defs) all clean.
- **Seeds filed — `docs/audits/decision-point-audit.md` §8.1** (the suite's durable inventory;
  `memory/primitive-wip.md` is rewritten wholesale by the next PB): **OOS-DP2-1**
  (`handle_keep_hand` never checks that `cards_to_bottom` entries are in the player's hand —
  a hostile `KeepHand` can bottom another player's card), **OOS-DP2-2** (starting hand size
  hard-coded to 7; fixing it is a HASH bump), **OOS-DP2-3** (all engine shuffles are predictable
  from public state — Architecture Invariant 7 / M10 hidden-info), **OOS-DP2-4** (the
  seeded-shuffle idiom is copy-pasted at 4 sites; deliberately not extracted here),
  **OOS-DP2-5** (bots' empty `cards_to_bottom`, latent until M11-local S2), **OOS-DP2-6** (the
  engine defers CR 103.5's bottoming from take-time to keep-time — behaviourally equivalent,
  record-only).

**PB-DP3** (`scutemob-151`, 2026-07-26) — **SHIPPED**. DP-4 from
`docs/audits/decision-point-audit.md` §5 (Tier 0, class D). **Mode announcement is now
mandatory** (CR 601.2b / 700.2a).

- **The defect.** `rules/casting.rs` gated *all* mode validation behind
  `if !modes_chosen.is_empty()`. Range (700.2a), duplicates (700.2d), `min_modes` and
  `max_modes` were checked correctly — but only if you supplied modes at all. Supply none and
  the empty vector fell through, and both consumers re-derived `vec![0]`. **Cryptic Command,
  Austere Command and Incendiary Command** (`min_modes: 2, max_modes: 2`, all `Complete`) paid
  full cost and resolved exactly **one** mode, silently.
- **The fix is a LIFT, not the audit's prescribed "mirror the Spree guard".** The checks moved
  out of the emptiness gate into a three-way match on
  `(entwine_paid, mode_selection_opt, modes_chosen.is_empty())`, so validation runs whenever
  the object is modal. **That made the yield much larger than the audit row predicted**: not 3
  cards but **40** — the 3 commands plus the **37** `min_modes: 1` defs that had all been
  accepting an unannounced cast. The Spree guard (`casting.rs:2938-2945`) was deliberately
  **kept**: it fires earlier, during cost computation, and owns the CR 702.172a message that
  `spree.rs:854` asserts. **Reusable lesson**: when a validation block is gated on "did the
  caller supply anything", the bug is usually the gate, not a missing check inside it — lifting
  beats bolting on, and the real blast radius is every card the gate was silently excusing.
- **Scope widened in planning, twice, both same-root-cause.** (1) `rules/abilities.rs` had the
  identical bypass for modal **activated** abilities (audit §4.2 line 214 said so); folded in at
  **zero** test/script cost, since every in-repo activation already passed explicit modes.
  (2) The `min_modes: 0` "choose up to N" shape: on the **activated** path it now correctly
  resolves *no* mode; on the **Spell** path it is **unrepresentable** (`StackObject.modes_chosen`
  is a bare `Vec<usize>` with no way to distinguish "chose zero" from a free-cast that never
  announced) and is hard-rejected fail-safe. That asymmetry is deliberate and documented at both
  code sites — see **OOS-DP3-2**.
- **The escalate exemption is the load-bearing judgement call.** Escalate's backward-compat path
  casts with an empty `modes_chosen` and derives `0..=count` at resolution. A naive hard reject
  would have killed it. PB-DP3 exempts `escalate_modes > 0` on CR 702.120a grounds — electing to
  pay the additional cost *is* an announcement of the mode **count** — and bounds-checks the
  derived count against `min_modes`/`max_modes`. Only the mode **identities** stay
  engine-derived (**OOS-DP3-1**). The reviewer upheld this with a stronger argument than the
  plan's: both escalate defs are `Completeness::partial`, so `validate_deck` blocks them and
  **no `Complete` card is live-wrong through that path**.
- **`resolution.rs`'s `vec![0]` fallback was RETAINED, and this is the highest-risk thing to get
  wrong here.** It looks like dead code after the fix and is not: four producers build
  `StackObjectKind::Spell` with an empty `modes_chosen` *without* calling `handle_cast_spell` —
  `copy.rs:386` cascade, `copy.rs:614` discover, `resolution.rs:5167` cipher copy,
  `resolution.rs:5837` suspend. Deleting it would make every suspended or ciphered modal spell
  resolve nothing. **The plan's original producer list was wrong in both directions** (it named
  four `engine.rs` sites that build Ring/Room/Loyalty/ClassLevel objects and can never reach the
  arm, and missed the two `trigger_default` ones); the review caught it and the corrected list is
  now in the code comment and in **OOS-DP3-3**.
- **No wire change: PROTOCOL 27 / HASH 63 unmoved**, as the audit §8 row predicted. No
  `Command`/`GameEvent`/`Effect` variant, no `GameState` field.
- **Blast radius, enumerated not estimated**: 3 engine test lines, 2 golden scripts (`stack/147`
  entwine, `stack/148` escalate — both stay `approved`, both now cite CR 601.2b), 1 replay-harness
  line (`cast_spell` now *forwards* `modes_chosen` instead of silently discarding the script's
  `modes` field), new `spell_default_modes`/`ability_default_modes` helpers in
  `crates/simulator/src/legal_actions.rs` wired into 4 `random_bot.rs` sites and 2
  `tools/tui/src/play/input.rs` sites, and **0 card-def edits**. `heuristic_bot` needed nothing —
  it routes through the single `action_to_command` chokepoint.
- **One un-enumerated gate fired** (the plan's §4.7 negative-space clause working as intended):
  the SR-15 `ability_definition_registry` gate failed because `spell_default_modes` is a new real
  dispatch site on `AbilityDefinition::Spell`. Declaring the site is the gate's *purpose*, so it
  was declared, not worked around.
- **Tests**: new `crates/engine/tests/primitives/pb_dp3_modal_mode_announcement.rs` (18 tests,
  `mod` line registered per SR-9a) + 4 simulator unit tests. **8 probes verified failing on
  pristine code** by reverting the guard — the two most telling: Austere Command's empty-mode
  cast simply *succeeded*, and the modal activated ability's mode 0 fired (life 40→43 where it
  should have stayed 40). `ability_default_modes` is tested against Umezawa's Jitte specifically
  because its `def.abilities[0]` is **not** the activated ability, so a `def.abilities`-indexed
  implementation fails the test — the PB-RS4 index-namespace bug class, pinned.
  **3,725 → 3,747 passing / 0 failing**; clippy `-D warnings`, `cargo fmt --check` and
  `tools/check-defs-fmt.sh` (1,804 defs) all clean.
- **Review**: 0 HIGH / 2 MEDIUM / 6 LOW, verdict "ship after fixes"; all dispositioned (5 fixed,
  1 declined-with-reason as seed-text-only, 2 folded into seeds). Both MEDIUMs were about the
  *record* rather than the behaviour — the wrong producer list above, and zero coverage on the
  two new escalate bounds branches (now covered by synthetic-card probes).
- **Seeds filed — `docs/audits/decision-point-audit.md` §8.1**: **OOS-DP3-1** (escalate derives
  contiguous mode identities), **OOS-DP3-2** (`min_modes: 0` Spell unrepresentable ⇒ HASH bump),
  **OOS-DP3-3** (4 free-cast producers bypass announcement — DP-20 scope, and DP-20's §5 row now
  cross-references it), **OOS-DP3-4** (modal *triggered* abilities auto-select mode 0; the
  "choose up to one" branch is literally `if x { vec![0] } else { vec![0] }` — bundle with
  PB-DP8), **OOS-DP3-5** (cast-time `ModeSelection` lookup is neither face- nor
  aftermath-aware — OOS-OS4-2/RS-3 root-cause class), **OOS-DP3-6** (escalate count over-payment
  is clamped, not rejected), **OOS-DP3-7** (~28 alt-cost harness cast arms can no longer cast a
  modal card at all), **OOS-DP3-8** (the entwine arm is now the only unvalidated one).
- **Audit rows updated**: §4.1 line 186 **D → A**, §4.2 line 214 **B → A**, §5 DP-4 SHIPPED,
  §5 DP-20 cross-reference, §8 PB-DP3 SHIPPED, §8.1 eight seeds, §9 recommendation 4 marked
  **superseded** (the M11 play server no longer needs a compensating check — it needs a
  mode-selection **UI**, and the simulator's default-modes helpers are the placeholder session 7
  must replace).

**PB-DP4** (`scutemob-152`, 2026-07-26) — **SHIPPED**. DP-10 + DP-11 from
`docs/audits/decision-point-audit.md` §5, bundled because they are the same bug shape: *an
affordability check is not a payment*. Both fixes amount to **making the check and the payment
the same predicate**. Commits `5c463339` (engine), `b213aeec` (simulator + tests), `084477ef`
(fix cycle).

- **DP-10 — the attack tax was inspected and never charged** (CR **508.1c** restriction,
  **508.1h/i/j** payment; the audit's "508.1g" cite was wrong — that rule is *optional* "as it
  attacks" costs like exert). `combat.rs` summed `cost_per_creature`'s six colour fields into a
  `u32`, compared it against `total_with_restricted()`, and returned `Ok` without touching the
  pool. Now a real per-defender summed `ManaCost` debited via `casting::pay_cost` in the
  mutation section, colour preserved, reusing `GameEvent::ManaCostPaid`. **Restricted mana no
  longer counts toward affordability** (CR **106.6** — every `ManaRestriction` variant is
  spell-scoped, so `spell: None` is correct; this is a deliberate behaviour flip and a player
  whose only mana is restricted can no longer attack past a Propaganda). Hybrid/Phyrexian/X
  taxes are **rejected** rather than silently contributing 0 (they were free before — the
  OOS-RS-2 class). The in-code claim that interactive payment *"requires a new
  `DeclareAttackers` command field"* is **falsified and deleted**.
- **DP-11 — the "otherwise, sacrifice" was never enforced** (CR 702.30a / 702.24a / 702.59a).
  `resolution.rs` claimed "the game pauses until a `Command::PayEcho` is received"; nothing
  implemented that pause, and the three `pending_*` vectors were inert queues no priority, SBA
  or step-advancement code ever read. **The design decision is the substance of this PB**: the
  fix is a **deadline, not a gate**. `force_resolve_overdue_payments` runs in
  `handle_all_passed`'s **stack-empty** branch and applies the CR 118.12a "didn't pay" branch to
  any unanswered payment. Gating priority was rejected because it **deadlocks** — `driver.rs`
  answers a rejected command with a silent `PassPriority`, so a refused pass is an infinite
  retry with no error, strictly worse than the bug. Deciding at resolution was rejected because
  it destroys the CR 608.2d/608.2g choice and makes `Command::PayEcho` unreachable. Auto-
  **decline**, never auto-pay (auto-pay is DP-19's bug class). Accepted deviation: the choice is
  deferred by one priority round, stated in-code and in the audit.
- **Yield larger than filed**: **5 `Complete` defs were live-wrong and are made right with 0
  card-def edits** (`propaganda`, `ghostly_prison`, `mogg_war_marshal`, `avalanche_riders`,
  `grim_harvest`); `mystic_remora`'s `known_wrong` note becomes accurate. The one card-def edit
  is a **comment** in `goblin_rabblemaster.rs`.
- **Two seeds closed**: **OOS-DP1-1** by *deletion* — all three `priority_holder =
  Some(active_player)` bodges are gone; they were identity writes for echo/CU (whose controller
  is the active player) but for **recover** the controller can be non-active and the write was
  actively yanking priority. **OOS-RS3-4** by Change 1c — `has_uncosted_attack_target` (CR
  508.1d) in both must-attack blocks, ending the "declaring is illegal AND omitting is illegal"
  deadlock.
- **Two bugs the audit had not filed**, found in planning/review: an unguarded life subtraction
  in the cumulative-upkeep `Life` arm (CR **119.4** — `PayCumulativeUpkeep{pay:true}` could
  drive a player to negative life), and §4.5's "Attack requirements" row being **mis-rated A**.
- **No wire change: PROTOCOL 27 / HASH 63 unmoved**, as predicted. Three new `LegalAction`
  variants are simulator-internal. Notably, audit §9 rec 3's `advance()` work turned out
  **unnecessary** — the payments arrive inside the existing `PendingDecision` as
  `DecisionKind::Priority`, so `local_game.rs` needed no edit at all.
- Review 0 HIGH / 5 MEDIUM / 17 LOW (banner said 13; the tables list 17 — discrepancy noted in
  the review), verdict "ship after fixes"; all 5 MEDIUM fixed, 10 LOW fixed, 6 declined with
  reasons, 1 no-fix-needed. Two of the MEDIUMs were **tests that could not discriminate** (an
  APNAP probe both orderings satisfied; a vacuous `players_passed` assertion) — the fix cycle
  strengthened both and verified the strengthened versions fail against a deliberately wrong
  implementation. Tests 3,747 → **3,781**.
- **Audit rows updated**: §4.5 attack-cost row **D → A** + CR cite corrected, §4.5
  attack-requirements row **mis-rated A → A since PB-DP4**, §4.11 echo row **D → A** enforcement,
  §5 DP-10 and DP-11 **SHIPPED**, §8 PB-DP4 **SHIPPED**, §8.1 OOS-DP1-1 **CLOSED** + twelve
  `OOS-DP4-*` seeds appended, §9 recs 3 and 6 annotated, §7 OOS-M11-2 rider. Cross-queue:
  `memory/primitives/rider-seed-triage-2026-07-19.md` marks **OOS-RS3-4 CLOSED** (status marker
  only — the RS queue's ordering and its §5 pause banner are untouched).

---

## Previous Handoff (preserved for chain context)

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
- ~~**Next: R5** (Anim Pakal LKI counters, OOS-RS-4) per `rider-seed-triage-2026-07-19.md` §5 banner; weigh OOS-RS3-1 insert + OOS-RS2-1 rider.~~ **STRUCK 2026-07-31 by the re-rank (`scutemob-159`).** The **PB-RS queue is RETIRED**; the authoritative queue is `memory/primitives/seed-rerank-2026-07-27.md` §4, **PB-DX1..PB-DX18**. Dispositions: **R5 retired** (LOW, 0 flips, and the obvious `CounterCountAtLastKnownInformation` swap returns 0 — that variant is LBA-only and Anim Pakal's trigger is `WheneverYouAttack`); R6→**PB-DX5** (re-ranked *up*: CR 611.2c is unimplemented and **7 `Complete` defs are live-wrong in ordinary play**, not the "0 flips" its filing claimed); R7→PB-DX13; R8→PB-DX12; R9→PB-DX16; R10→PB-DX14; R11→PB-DX17 (and `karazikar` has no def at all, so its "1 flip" is a new authoring). **OOS-RS3-1 was already CLOSED by PB-DP6** — all five `CardDefETB` sweeps gate at queue time; the banner had advertised it as the next insert for a week. **OOS-RS2-1 re-ranked up to PB-DX6** (still live: `can_spend`'s residue guard is `debug_assert`-only, so `kitchen_finks`'s two `{G/W}` pips are free in release). **`OOS-RS1-2` is a phantom** — never filed; strike it.
- **Next dispatch: PB-DX1** — OOS-DP6-1, the intervening-if dropped in the runtime lowering (`build_face_ability_vectors` hardcodes `intervening_if: None` at 34 sites and both the queue and resolution ends read that field, so CR 603.4 is checked at neither). `aurelia_the_warleader` is `Complete` by `#[default]`, deck-legal, and grants herself unbounded extra combats. HASH bump expected (the field lives inside `Characteristics`). Then **PB-DX2** (`ChooseDredge` free-card exploit, wire-neutral) and **PB-DX3** (2 flips, zero engine change).
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

### 2026-07-16..17 (oversight — marker sweep + SR-33..38 + W-waves + EF triage) [rotated]

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


