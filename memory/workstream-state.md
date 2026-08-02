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
| W6: Primitive + Card Authoring | — | available (PB-DP suite COMPLETE) | 2026-07-31 | **PB-OS queue COMPLETE** (OS1..OS11 + OS4b, `scutemob-116..141`). **Rider-seed queue**: RS1..RS4 SHIPPED (`scutemob-143..146`); plan `memory/primitives/rider-seed-triage-2026-07-19.md`, resume at **R5** per its §5 banner (weigh OOS-RS3-1 insert + OOS-RS2-1 rider). **The PB-DP suite now runs FIRST** (user directive 2026-07-26) — queue `docs/audits/decision-point-audit.md` §8, from the decision-point audit (`scutemob-148`). **PB-DP1 SHIPPED** (`scutemob-149`, merged `f7651bb5`): priority after cast/activate/special action goes to the ACTOR per CR 117.3c; 14 Group-A sites + 8 Group-D sites; entry priority guards added to `handle_turn_face_up`/`handle_activate_loyalty_ability`/`handle_level_up_class`; 19 tests + 15 golden scripts reconciled; PROTOCOL 27 / HASH 63 unchanged; 3,721 tests green. Seeds **OOS-DP1-1..4** filed in the audit doc **§8.1** (durable inventory for this suite — not in `primitive-wip.md`, which the next PB overwrites). Suite tasked out `scutemob-150..158` = PB-DP2..DP10. **PB-DP2 SHIPPED** (`scutemob-150`, commit `f902010f`): mulligan content no-op + bottom-to-top, **CR 103.5/103.5c** (the brief's "103.4b" is a stale cite — see the handoff below); **OOS-M11-1 CLOSED**; 4 probes; PROTOCOL 27 / HASH 63 unchanged; tests 3,721 → **3,725**. Seeds **OOS-DP2-1..6** filed in the audit doc **§8.1**. **PB-DP3 SHIPPED** (`scutemob-151`, DP-4 `min_modes` floor, **CR 601.2b/700.2a**): mode announcement is now mandatory — the fix is a **lift** of the range/duplicate/`min_modes`/`max_modes` checks out of the `!modes_chosen.is_empty()` gate, not the audit's prescribed Spree-guard mirror, so it fixed **40** modal defs (3 commands + 37 `min_modes: 1`) rather than the 3 the row predicted, plus the identical activated-ability bypass in `abilities.rs` (audit §4.2). Narrow CR 702.120a escalate exemption; `resolution.rs`'s `vec![0]` fallback **retained** (4 free-cast producers bypass `handle_cast_spell`). PROTOCOL 27 / HASH 63 unchanged; 0 card-def edits; tests 3,725 → **3,747**. Seeds **OOS-DP3-1..9** filed in the audit doc **§8.1**. **PB-DP4 SHIPPED** (`scutemob-152`, merged `799dcc0a`): DP-10 attack tax now debited (colour-correct; restricted mana excluded per CR 106.6; hybrid/Phyrexian/X tax rejected — OOS-DP4-1); DP-11 enforced as a **deadline** (auto-decline unanswered payments at `handle_all_passed`'s stack-empty branch per CR 118.12a — a priority gate would deadlock the driver); 5 Complete defs made right, 0 def edits; **OOS-DP1-1 + OOS-RS3-4 CLOSED**; seeds OOS-DP4-1..13 filed in audit §8.1; PROTOCOL 27 / HASH 63 unchanged; tests 3,747 → **3,781**. **PB-DP5 SHIPPED** (`scutemob-153`, merged `922252f7`): `pending_draws` on `GameState`, `OrderReplacements` routed by applicability; **3 emit sites fixed, not 2** (`draw_card_skipping_dredge` was a third the audit never named); also fixed a CR 614.11a sequence bug (`DrawCards{count:3}` emitted 3 unanswerable prompts and drew 0 — now one prompt, remainder stashed) and a review-found CR 616.1f loop gap in `determine_action`; **HASH 63 → 64** (gate-forced), PROTOCOL 27 unmoved; tests 3,781 → **3,797**; 0 def edits. NOTE: audit premise falsified — **0 of 1,804 defs register a `WouldDraw` replacement**, so card yield is 0; this is an engine-correctness fix + precondition for authoring the WouldDraw family. **PB-DP6 SHIPPED** (`scutemob-154`, merged `d52fe5b6`): intervening-if now evaluated at trigger-queue time across the queue paths (audit §4.8 queue-time row D→A); no wire change (PROTOCOL 27 / HASH 64); tests 3,797 → **3,809**; seeds OOS-DP6-1..10 filed in audit §8.1 (note OOS-DP6-10: the one hazard this batch INTRODUCED — A9 `WasKicked` suppression, wrong-direction, zero corpus exposure today). **No-wire block DP1..DP6 COMPLETE. PB-DP7 SHIPPED** (`scutemob-155`, merged `8f890611`): cleanup-discard is now a blocking player Command (CR 514.1); the **blocking pending-decision mechanism is proven** for DP8/DP9 reuse; **PROTOCOL 27 → 28, HASH 64 → 65** (both gate-computed); tests 3,809 → **3,837**; 2 fix cycles (18 + 6 findings; 2 HIGH: CR 800.4j dead-player entry skipping CR 514.2, out-of-step answer accepted; plus a TUI auto-pass livelock introduced-and-fixed); seeds OOS-DP7-1..12 in audit §8.1 — **OOS-DP7-11 flags that the SR-19 HashInto gate silently skips path-qualified impls** (gate-integrity seed, rankable). **PB-DP8 SHIPPED** (`scutemob-156`, trigger-target choice, CR **603.3d/601.2c/603.3b**): `Command::ChooseTriggerTargets` + `GameEvent::TriggerTargetChoiceRequired` (disc. 130) + `GameState.pending_trigger_targets` suspend `flush_pending_triggers` MID-BATCH and resume it on the controller's answer; the compliant CR 603.3d fallback survives verbatim as the exported `abilities::default_trigger_targets`, which the CALLER submits as a real Command (engine still knows nothing about seat kind). **PROTOCOL 28 → 29, HASH 65 → 66** (all five fingerprints gate-computed; `TriggerTargetOption` + `SpellTarget` enter the wire closure — a genuine type-count change, unlike DP7's). **Roster 77, not the audit's 84 nor the planner's 74** — enumerated from `all_cards()` per SR-36 and printed by a test. **0 card-def edits, 0 completeness flips**, but **2 live-wrong `Complete` cards fixed by accident** (sword_of_sinew_and_steel, elder_deep_fiend): the plan's premise that a permanent-inner `UpToN` slot 'contributed 0 targets' is FALSE — it returned `None` and the caller removed the WHOLE TRIGGER, so those cast/damage triggers had never once reached the stack. CR 601.2c makes zero targets a legal announcement. Second plan gap found and fixed: §4.1 never says who grants the priority the four guards were about to grant, so a resumed game would have had nobody holding priority — added `grant_priority_on_resume` on the entry. Consult set is **4 guards, not the ~20 DP7's row predicted**, because PB-DP1 moved priority assignment ahead of the flush (all 30 `check_and_flush_triggers` sites verified to need none). Also fixed `local_game.rs`'s latent variant-blindness (hard-coded `DecisionKind::CleanupDiscard`) — now compile-forced. tests 3,837 → **3,858**; 53+1 sentinels re-pinned across 44 files; 1 golden script corrected with CR justification. Seeds **OOS-DP8-1..10** in audit §8.1 (DP8-9/DP8-10 are new relative to the plan). **OOS-M11-4 CLOSED.** OOS-DP3-4 deliberately NOT bundled — ranked as **PB-DP8b** (OOS-DP8-7). **PB-DP9 SHIPPED** (`scutemob-157`, search/scry/surveil, CR **608.2d / 701.23a / 701.22a / 701.25a**): the engine's **first resolution-time decision channel**. `GameState.pending_effect_choice` + `GameEvent::EffectChoiceRequired` (disc. 131) → `Command::AnswerEffectChoice`, backed by an **abort-and-replay** continuation, NOT the "resumable effect-list cursor on the stack object" `pb-plan-DP7.md` §1.6 and audit §8 both prescribed — that is **impossible**, because `resolve_top_of_stack` POPS the stack object before any effect runs. Instead: clone at entry, an effect that needs an unanswered choice records the question and returns, the wrapper restores the clone **wholesale** and emits one event, the answer is banked on `GameState` and the resolution re-runs **from the top**, retracing the identical deterministic path. Consequences worth carrying: no continuation data structure at all (`Sequence`/`Conditional`/`ForEach`/`Repeat` need zero machinery — the replay re-executes them); the re-entrancy audit is **3 units, not 20** (15 of 17 production `execute_effect` callers are inside `resolve_top_of_stack` itself, one is gated by CR 605.4a, one is provably unreachable); **PB-DP8's "a guard that returns early inherits a debt" bug class does not exist here** because a total restore skipped nothing; and "the suspended object leaves the stack" is structurally unreachable rather than a live hazard. **ONE `Command` for all three effects** (CR 608.2d is one rule; 701.22a/23a/25a are three instances) — so one gate entry, one `LegalAction`, one `DecisionKind`, one harness action string, which **corrects OOS-DP8-14's prediction of three**. **PROTOCOL 30 → 31, HASH 67 → 68** (all gate-computed, histories append-only, 44 sentinel files re-pinned via the SYMBOL grep). **Roster 69 / 16 / 7, not the audit's 74 / 16 / 8** — enumerated from `all_cards()` with a RECURSIVE `Effect`-tree walk (a flat scan undercounts). **0 def edits, 0 flips.** Three in-scope correctness fixes beyond agency: CR **701.22b** (`Scry 0` was emitting `Scried{count:0}`; the surveil arm had the mirror guard, the scry arm did not), CR **400.7** (scry-to-bottom RENUMBERED every scried card and consumed `timestamp_counter`, the shuffle seed source — now `Zone::reposition_within`; sweep seeded as OOS-DP9-11), and CR **701.23d** (a quantity-only search with one candidate is determined and asks nothing). Two deliberate deviations, both argued in source and pinned by tests: **the scry/surveil defaults FLIP to the identity** (search keeps its lowest-id default byte-for-byte), and **the three new fields are EXCLUDED from `loop_detection.rs`'s fingerprint** unlike DP7's and DP8's, because they grow between replays of one resolution and could mask a CR 726 loop — recorded as **obligation (7)** on the `BlockingDecision` doc block, the first evidence that list generalises. `GameEvent::private_to()` now exists (OOS-DP8-6's declaration half; a declaration, NOT an enforcement point — nothing consumes it until M10). Benches measured against `48353a36`: `full_turn_4p` 253 → 229 µs, **no regression** from the per-resolution clone. Fallout 25 unit tests + 1 golden script, every repair CR-justified. tests 3,878 → **3,905**; seeds **OOS-DP9-1..12** in audit §8.1 (rank **OOS-DP9-3** first — `SearchLibrary` finds exactly one card, ~7 partial defs, zero new plumbing on this machinery). Merged `d65e7f1e`; on-main verified **3,910 / 0** (5 more than the branch pin — post-merge count). **PB-DP10 SHIPPED** (`scutemob-158`, decision-gate widening, **test-only** — and it **CLOSES THE PB-DP SUITE**): two new files under `crates/engine/tests/core/` — `decision_site_walk.rs` (the canonical serde walk + `ROWS`, all 22 decision sites of audit §3.1 classified **4 SERVED / 15 AUTO-CHOSEN / 2 GATED / 1 NO-DECISION**, each with the engine site that was *read* to establish the class) and `decision_gate.rs` (`BASELINE`, 97 name-keyed entries with exact row sets, + 18 tests). **The headline is a gate-integrity finding, not a feature**: every serde walk in this codebase before now (`effect_choose_gate.rs`, `pb_rs1_roster_sweep.rs`, `pb_dp9_effect_choice.rs::roster`) matched **object keys only** and is therefore blind to a **unit** `Effect` variant — `serde_json::to_value(Effect::Proliferate)` is `Value::String("Proliferate")` — so a verbatim reuse would have reported **0** for Proliferate's 25 `Complete` defs *while looking green*, the exact OOS-DP7-11 failure mode. Fixed with a two-shape walk + a `PROSE_FIELDS` denylist, pinned in both directions against the legacy walk (T2/T3). Measured: all-rows union **267** (the audit's 277 analogue), still-auto union **97**, live denominator 1,139/1,804. **Fail-closed proven end-to-end on a real def**, not just synthetically: adding `Effect::Proliferate` to `lightning_bolt.rs` reddened **two** tests naming the card, the row, the CR and the engine site; restored → green. Two hand-maintained zeros (`AddManaFilterChoice`, `TheRingTemptsYou`) that **nothing** was holding became machine-checked (the SR-33 gate bars a *different* key). **PROTOCOL 31 / HASH 68 unmoved; engine + card-types + card-defs diff vs main EMPTY; 0 def edits, 0 flips.** Review 2 HIGH / 6 MEDIUM / 6 LOW, **all 14 applied** — the HIGHs are worth carrying: (1) `BASELINE` was populated **mechanically** and the plan's class-B/class-D triage was never done, so a spot-check found two class-D defs already inside the frozen baseline (Smuggler's Copter's "you **may** draw" authored as an unconditional `Sequence`; Shambling Ghast with a permanent `-1/-1` counter, an `oracle_text` saying "enters" against a `WhenDies` trigger, and a `Decayed` keyword the printed card does not have) — seeded as **OOS-DP10-8**, not demoted; (2) **the gate can only see a decision the DSL ENCODED**, and that blind class is strictly *worse* than the one it records (**OOS-DP10-9**, and the instrument for it is an oracle-text-vs-DSL cross-check, not a variant walk). tests 3,910 → **3,928**; seeds **OOS-DP10-1..11** in audit §8.1; **closes OOS-DP7-7** (the 277-def re-derivation is now computed, printed and ratcheted every run). Audit §8 now carries a suite-COMPLETE banner and §10 an honest 3-of-8 mechanization ledger. Merged `16ffcfd0`, on-main verified 3,928/0; suite retrospective in the audit doc §8. **Next: re-rank RS5..RS11 against the unranked seeds** (OOS-DP9-3 was the previous first pick; OOS-DP10-8/-9 are new and OOS-DP10-6 is the successor queue's ranked input). **Seed re-rank SHIPPED** (`scutemob-159`) — successor queue `memory/primitives/seed-rerank-2026-07-27.md` §4, PB-DX1..DX18. **PB-DX1 SHIPPED** (`scutemob-160`): OOS-DP6-1 + riders CLOSED; PROTOCOL 31→32 / HASH 68→69; tests 3,928→3,945. **PB-DX2 SHIPPED** (`scutemob-162`): OOS-DP5-7 + OOS-DP7-2 + riders OOS-DP2-1/OOS-DP9-14 all CLOSED — see the Last Handoff section below for detail; PROTOCOL 32 / HASH 69 unmoved; tests 3,945→3,971 (this worktree's own baseline was 3,955 after the intervening M11-local S2 merge, so the batch's own delta is +16). **Fix cycle same day**: review found the implement-phase "fold guard" was a HIGH (unbounded cross-turn accumulation, cashable out-of-priority) — replaced with a discharge design that also closes OOS-DX2-3 as a side effect; 7 doc-vs-code MEDIUMs + 1 coverage-hole MEDIUM + 7 LOWs all applied; PROTOCOL 32 / HASH 69 still unmoved; tests 3,971→**3,974**. **PB-DX3 SHIPPED** (`scutemob-164`, 2026-08-01): **OOS-DP6-3 CLOSED** — `garruks_uprising` + `inventors_fair` both `partial` → **`Complete`**, coverage 1,140 → **1,142** (63.2% → 63.3%), tests 3,988 → **3,998** (+10 probes), **0 engine lines** (empty `git diff` over the whole of `crates/engine/src` *and* `crates/card-types/src`, not just the wire files) and PROTOCOL 32 / HASH 69 unmoved. Review 0 HIGH / 1 MEDIUM / 5 LOW, all applied. **Three things the queue row did not contain, in ascending order of how much they matter.** (1) `inventors_fair`'s upkeep trigger **did not exist at all** — the seed and both blocker notes read as though it were present but ungated, so the batch had to *author* the ability. (2) The runtime `InterveningIf` enum both notes name now has **three** variants, not the two they cite: PB-DX1 added `InterveningIf::CardDef` two batches earlier. The stale notes were stale twice over, and this queue introduced the second staleness itself. (3) **The MEDIUM was the batch reproducing its own subject.** The test module recorded a pre-fix observation for T1 ("the hand count was 1") that **could not have been observed** against T1's own fixture, which had no library object — and an empty-library draw sets `has_lost` (`replacement.rs:1035-1049`) rather than incrementing the hand, so the companion assertion passed whether or not the bug fired. Fixed by giving T1 a real library card and **re-running the pre-fix scenario empirically** (reverting `intervening_if` to `None` and reading the numbers), not by repairing the prose; the same standard was then applied to T3/T5/T6/T7/T8, all of which held. The original claim was right — it had simply never been checked against a fixture where the number meant anything, and that distinction is the whole lesson. `reveal: true` on `Effect::SearchLibrary` is inert (pre-existing **OOS-DP9-9**) and now carries an in-def comment saying so rather than being silently covered by the `Complete` marker. **New seed OOS-DX3-1** (audit §8.1): six more defs sit in the same `pb-plan-DP6.md:395` stale-blocker bucket and **`jadar_ghoulcaller_of_nephalia` is a live-wrong `Complete` def** — `intervening_if: None`, so it makes a 2/2 Zombie **every** end step unconditionally, and its stored `oracle_text` names a token-name filter the printed card never had (MCP: the real text is "if you control no creatures with decayed"). Expressible today as `Not(YouControlNOrMoreWithFilter{count:1, filter: Creature + has_keywords[Decayed]})`; the fix must also reconcile golden script `combat/191`. `ophiomancer` (`partial`, its own note already says "Blocker stale") and `dwynen_s_elite` (`inert`) are two more flips in the same shape. **Next: PB-DX4** (OOS-DP10-8, the 97-entry `BASELINE` triage) — but consider inserting OOS-DX3-1's Jadar half first: live-wrong `Complete`, card-def only.  **PB-DX3b SHIPPED** (`scutemob-166`, 2026-08-01 — a **queue insert ahead of PB-DX4**, taken on the post-DX3 banner's own recommendation): **OOS-DX3-1 CLOSED**. All **seven** remaining defs of the `pb-plan-DP6.md:395` stale-blocker bucket dispositioned explicitly — 4 fixed, 3 deferred with blockers re-affirmed against the *current* `Condition` enum rather than copied forward. `jadar_ghoulcaller_of_nephalia` stays `Complete` and is now CR 603.4-gated; **its stored `oracle_text` was wrong, not merely its blocker note** (the field said "tokens named Shambling Ghast"; MCP says "creatures with decayed"), so the note had been declaring a DSL gap for a filter the card never had — a distinct failure mode from PB-DX3's stale-note class. `ophiomancer` `partial` → `Complete` (`has_subtype: Snake` alone, deliberately not `ControlCreatureWithSubtype`, whose arm hard-requires `CardType::Creature`). `dwynen_s_elite` `inert` → `Complete`, ability **authored from nothing** — the `inventors_fair` shape recurring; expect it. **The seed itself mis-dispositioned a second live-wrong `Complete`**: `emeria_the_sky_ruin` declares no `completeness` field, so it was `Complete` by the `#[default]` derive and reanimated every upkeep regardless of Plains count — the `aurelia_the_warleader` trap from PB-DX1, hit a second time in three batches by a different route. Gated, given an **explicit** `partial` for the DSL-inexpressible "you may" (OOS-DP10-8 class, falsifier search actually run), and a spurious `Legendary` supertype removed (MCP type line is `Land`). **2 flips up, 1 honest flip down — net coverage 1,142 → 1,143, +1 not +3**; 0 engine lines (empty diff over all of `crates/engine/src` + `crates/card-types/src`); PROTOCOL 32 / HASH 69 unmoved; tests 4,008 → **4,022** (this branch's merge base is 4,008, not the 3,998 DX3 pin — `scutemob-165` merged in between). Golden script `combat/191` reconciled by **strengthening** (it had never asserted the Zombie token and passed either way). Review 0 HIGH / 5 MEDIUM / 7 LOW, all 12 applied. New seed **OOS-DX3b-1** (`guardian_project`'s `is_nontoken` half is authorable today; its name-uniqueness half is not, so it stays `known_wrong`). **Durable**: `#[default] Completeness::Complete` is now a twice-demonstrated silent-defect generator — "which defs never declare a marker at all?" is a cheap corpus-wide question nobody has asked. **PB-DX4 SHIPPED** (`scutemob-168`, 2026-08-01): **OOS-DP10-8 CLOSED**, and **OOS-M11-6 closed incidentally**. All 97 `BASELINE` entries read against MCP printed text, roster parsed out of the const array itself (97 → 97 distinct names → 97 unique def files) rather than taken from prose, because this suite has published a wrong roster three times. **Split 84 class-B / 13 class-D** — PB-DP10's 2-of-5 spot-check overstated the D rate ~5x and its own "very noisy sample" caution was right; the queue row's "0 flips" estimate was wrong the other way, since 5 of the 11 had to be demoted. **5 repaired, still `Complete`**: `metastatic_evangel` (4 defects: `{2}{W}`→`{1}{W}`, missing `Human`, P/T transposed 1/3→3/1, and a **stale** in-def note claiming `is_token` is ignored on the ETB path — PB-AC0 had made that false), `grisly_salvage` + `satyr_wayfinder` (`RevealAndRoute` routes ALL matches → `LookAtTopThenPlace{optional:true}`; printed says "**a** card", "you **may**"), `sword_of_truth_and_justice` (bare `TargetCreature` → `controller: You`), `radstorm` (`{2}{U}`→`{3}{U}`). **6 demoted with oracle citations**: `smugglers_copter` → `known_wrong` (20th DP-12 instance; the other 19 already were, so the marker was the defect), `contaminant_grafter` / `grateful_apparition` / `thrasios_triton_hero` → `partial`, and `shambling_ghast` → `partial` **for a defect the fix surfaced** — its three named deviations (phantom `Decayed`, permanent `MinusOneMinusOne` for a printed "until end of turn", `oracle_text` saying "enters" against `WhenDies`) were all FIXED, and the marker is for a fourth: the mode-1 target is flat, so taking the Treasure mode still needs an opponent creature (CR 603.3d). **`mode_targets` is honoured only on the CASTING path** — nothing on the trigger path reads it — so the obvious repair would have DROPPED the requirement rather than scoped it (**OOS-DX4-2**; `hullbreaker_horror` is a second member). **1 left `Complete` deliberately**: `staff_of_compleation`'s "target permanent you own" as `TargetController::You`, allowlisted to match the shipped `nether_traitor` decision for the identical owner-vs-controller class (**OOS-DX4-1**) rather than reporting a corpus class as two cards. **OOS-M11-6 found by accident**: demoting `thrasios_triton_hero` — a legendary creature, i.e. a member of `random_deck`'s own commander pool — re-dealt every seeded deck in the workspace and landed seed 9001 on Rograkh, the corpus's ONLY colourless `Complete` legendary creature (1 of 91). Fixed as that seed preferred (pad from the identity-legal colourless pool; measured 40 colourless lands + 82 nonlands = 122 singletons vs 99 needed), **both** Forest fallbacks removed. The bigger half: the fuzzer feeds `random_deck` straight to `GameStateBuilder` with no validation, so it had been silently **playing** CR 903.5c-illegal decks. Six fixtures across two crates broke; the two play-server rebuild tests lost their only failure trigger exactly as their own maintenance note predicted and now use a sentinel **seed** (a first attempt used a process-global flag that raced with every other test POSTing `/api/game` — green under `-p`, red under `--workspace`, twice). Golden script `baseline/112` **retired**: it tested Decayed on a card that does not have it, citing the card *def* as its authority — a provenance failure. CR 702.147a keeps 12 unit tests; golden-level gap filed as **OOS-DX4-3**. Coverage 1,143 → **1,137** (63.0%), tests 4,040 → **4,048**, `BASELINE` 97 → **91** (moved twice inside the batch, 97→93→92, which is why it was read off the gate not computed), deviation floor 661 → **667**, DP8 roster 76 → **74**, `scry` 16 → **15** — each re-measured against `all_cards()`. **0 engine lines** (empty diff over `crates/engine/src` *and* `crates/card-types/src`), PROTOCOL 32 / HASH 69 unmoved. **PB-DX3b's `#[default]` question answered and bigger than expected: 966 of 1,804 def files never mention `completeness` at all (970 before this batch)** — a clear majority of the `Complete` population, and **eleven of the thirteen** class-D defs were in it; now ratcheted in the growth direction. Durable record `memory/primitives/pb-dx4-baseline-triage.md` (per-def citations + an explicit statement of what the triage does NOT establish: it is a dated claim, it cannot see a decision the DSL never encoded — OOS-DP10-9 stands — and 97 of 1,143 is not a sample the rest can be inferred from). Seeds **OOS-DX4-1..6**. **PB-DX5 SHIPPED** (`scutemob-170`, 2026-08-01): **OOS-OS7-2 CLOSED — CR 611.2c**, `ContinuousEffect` gains `affected_set: Option<OrdSet<ObjectId>>`, populated only by `Effect::ApplyContinuousEffect` via the new `rules::layers::snapshot_affected_set` (never elsewhere) and read as pure membership by `effect_applies_to`; `None` means a static ability (CR 611.3a, unchanged, still live-evaluated). **The dispatch row's roster ("9 defs, 7 `Complete`") was wrong twice over — the sixth consecutive batch in this suite whose published roster was wrong before it started.** `all_cards()`, enumerated fresh: **116** defs generate a resolution-time continuous effect at all; **38** use a mass filter (not 9); and even the premise-verification step's own corrected figure (37, 28 `Complete`) was off by one against its OWN table, which already listed 38 rows summing to 29 — an uncaught arithmetic slip nobody re-added. **Final measured: 38 mass-filter defs, 29 `Complete`, 8 `partial`, 1 `known_wrong`**, from a new self-re-measuring test (`pb_dx5_mass_filter_roster_by_completeness`) rather than a pinned count. Mechanical backfill of `affected_set: None` at all 180 pre-existing `ContinuousEffect` construction sites (49 files, compiler-driven, zero manual judgement calls — every site is either a static registration or a `SingleObject` effect, and `None` is the RULE at the former, a no-op at the latter). **HASH 69 → 70** (mandatory, gate-forced); **PROTOCOL confirmed unmoved at 32** by actually running `--test core protocol_schema`, not assumed (`ContinuousEffect` is outside the SR-8 wire closure). **Yield 0 completeness flips, exactly as pre-committed** — a pure correctness fix for defs already `Complete`; coverage stays 1,137/1,804 (63.0%, byte-identical regen). **Existing-test repair, exactly as flagged a hazard in advance**: `pb_ac3_dynamic_pt_counts.rs`'s `test_set_both_dynamic_locked_at_resolution` was asserting the CR 611.2c bug this batch fixes ("a creature entering after resolution still gets the locked-in X=3") and had been passing; inverted with a CR cite and renamed, not weakened. Every "fails before" claim in the new 14-test probe module was OBSERVED (read-site membership check reverted, actual value recorded, restored) — which caught the runner's OWN first-draft T3 (control-change retention) using the buffed creature as its own effect source, masking the very divergence it claimed to test; fixed with a separate, never-moved source. T11 (zone-scope shortcut vs. brute force) lives as an in-source `#[cfg(test)]` unit test in `rules/layers.rs`, not the integration file — `snapshot_affected_set`/`effect_applies_to_object`/`candidate_ids_for_filter` are all `pub(crate)`. Benchmarks all within ~1% of the merge base; `board_wipe_4p` (flagged most likely to move) measured slightly *faster*. Golden corpus unaffected — Final Showdown's script exercises mode 2 (DestroyAll), not the `AllCreatures|Ability` mode 0 the roster found (a pre-existing, documented DSL-gap omission in the script itself). Six new seeds **OOS-DX5-1..5** + a checked non-finding **OOS-DX5-6** (Mirror Entity is the one Layer ≤4 mass-filter def; unaffected today — nothing in the roster writes `CardType::Creature` via a Layer-4 modification). Tests 4,048 → **4,064**. **Same-day fix cycle** (review `pb-review-DX5.md`, 0 HIGH / 6 MEDIUM / 6 LOW, all 12 applied, none changing observable behaviour): the test-count arithmetic above was itself off by one (the roster file has two `#[test]`s, not one — true implement-phase total was **4,065**, and the fix cycle's own +1 new test (T15) makes the re-run total **4,066**); OOS-DX5-6's "checked non-finding" was FALSE — a real, reachable, CR-correct divergence exists (animate Inkmoth Nexus + Mirror Entity's `AddAllCreatureTypes`), now pinned by T15 and corrected in the seed doc; the fix was found to close a SECOND, larger pre-existing defect (every source-relative mass filter on an instant/sorcery applied to nobody once the spell resolved, CR 400.7) — confirmed empirically, filed as **OOS-DX5-7 (CLOSED as a side effect)**; OOS-DX5-1 widened to name three read sites that ignore `affected_set`; a stale note in `pb_os7_defending_player_continuous_filter.rs` that declared this batch's own seed a live limitation was corrected; T11's fixture was genuinely enriched (phased-out permanent, real `AttachedCreature` match, subtype filter); a vacuous `debug_assert!` hardened; the non-fixed-window-duration question (plan §3 Q4) is now a measured, standing assertion (zero corpus members) rather than a spot check. HASH 70 / PROTOCOL 32 re-confirmed unmoved by re-running both schema gates. **Next: PB-DX6** (OOS-RS2-1 + OOS-DP4-1 — the two mana-cost payment sites PB-RS2 left unflattened; `handle_turn_face_up` pays a raw `def.mana_cost` and `can_spend`'s residue guard is `debug_assert`-only, so in release every hybrid/Phyrexian pip in a `TurnFaceUpMethod::ManaCost` flip is FREE — `kitchen_finks` is `Complete` with two `{G/W}` pips; and `Command::DeclareAttackers` has no `hybrid_choices`/`phyrexian_life_payments` fields at all. One PROTOCOL bump for the batch; also make the residue guard fail loud). Full brief in `memory/primitives/seed-rerank-2026-07-27.md` §"Dispatch briefs", whose stale "Next dispatch: PB-DX5" banner was struck through at PB-DX5 close rather than left to become the N4 re-dispatch hazard a fourth time. Two PB-DX5 residuals worth weighing as inserts first: the probe module rebuilds Craterhoof/Jitte/Mirror Entity encodings by hand rather than instantiating the corpus defs, so def-vs-probe drift would go unnoticed (the `all_cards()` roster test and the fix's filter-agnosticism mitigate but do not close it); and `rules/face.rs`'s deregistration sites match on `source == obj_id && filter == resolved_filter`, which cannot distinguish a static registration from a resolution-generated effect sharing both — `affected_set.is_none()` is now a ready-made discriminator, a different class from the three read sites OOS-DX5-1 names and a candidate to fold into it. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## M11-local Track (parallel to W6 — `crates/simulator`, `tools/`, no engine surface)

> **✅ MILESTONE COMPLETE — all 8 sessions shipped, closed by `scutemob-173` on
> 2026-08-01.** This section is now a record, not a queue.
>
> Deliberately its own section, not a W-row: M11-local ran concurrently with the W6
> primitive queue and touched a disjoint set of crates. Plan: `memory/m11-session-plan.md`
> (8 sessions, authoritative, now marked COMPLETE). **No new `Command`/`GameEvent` variant
> anywhere in the milestone** — the wire-neutrality claim held end to end; the pins at
> close are PROTOCOL **32** / HASH **70**, both moved by the W6 track (PB-DX1, PB-DX5) and
> never by M11-local, confirmed by an empty `git diff` over `crates/engine/src` +
> `crates/card-types/src` + `crates/card-defs/src` across the whole S8 branch.

| Session | Task | Status | Notes |
|---------|------|--------|-------|
| S1 steppable local-game core | `scutemob-147` | **SHIPPED** | `LocalGame` in `crates/simulator/src/local_game.rs`; `GameDriver::run_game` re-expressed on top of it |
| S2 deterministic pregame setup + mulligans | `scutemob-161` | **SHIPPED** | `setup.rs`: `build_initial_state` / `redeal` — see handoff below |
| S3 action parameterization + engine target queries | `scutemob-163` | **SHIPPED** | the crux (plan §8 R1) is closed: a human can cast a targeted spell. See handoff below |
| S4 view-model crate extraction + seat redaction | `scutemob-165` | **SHIPPED** | this session — `crates/view-model` (`mtg-view-model`); a seat view provably cannot leak another hand or any library order. See handoff below |
| S5 play-server crate skeleton + REST API | `scutemob-167` | **SHIPPED** (+ 2 review cycles) | this session — `tools/play-server` (axum, port 3040), the only crate in this milestone with async or IO. 5 routes + `ServeDir`, **16 tests** (15 `oneshot` HTTP + the source gate, which is a plain `#[test]` and constructs no router), **no port ever bound and now machine-gated crate-wide**. See handoff below |
| S6 play frontend — render and basic input | `scutemob-169` | **SHIPPED** | this session — `tools/play-server/frontend` (Svelte 5 + Vite 7), dev proxy to `127.0.0.1:3040`, `$viewer` alias importing the replay-viewer components **in place**. **Zero Rust**: `git diff main` over `crates/` + `tools/play-server/src` + `tools/play-server/Cargo.toml` is empty — **zero Rust anywhere**; the only change outside `tools/play-server` is one Svelte component, `tools/replay-viewer/frontend/src/lib/ZoneHand.svelte` (the review HIGH below). PROTOCOL 32 / HASH 69 unmoved, tests **4,040 / 0**. See handoff below |
| S7 targeting, combat and choice UIs | `scutemob-171` | **SHIPPED** | this session — `tools/play-server/src/{view.rs,api.rs}` populate `target_slots` / `target_min`/`max` / `modes` (with per-mode slots and ranges) / `attack` / `block` from `mtg_engine::{spell_target_requirements, ability_target_requirements, legal_targets_per_slot, target_count_range}` and the provider's own `DeclareAttackers`/`DeclareBlockers` payloads; `validate_combat_params` refuses an unoffered pair with a 400; `needs_x` now answers `ActivateAbility` (README Limitation 5 CLOSED). Four picker components + an `ActionBar` chain in CR 601.2b → 601.2c → 508.1 → 509.1 order. **One additive change outside `tools/play-server`**: `StackItemView::source_object_id` in `crates/view-model` — see handoff. PROTOCOL 32 / HASH 69 unmoved; play-server tests 18 → **24**. See handoff below |
| S8 playthrough hardening, docs, acceptance | `scutemob-173` | **SHIPPED — CLOSES THE MILESTONE** | this session — scripted playthrough on 5 seeds, human-only `Concede` + `OrderBlockers`, error audit, `GET /api/game/report`, docs, decisions.md, 8 gates. See the handoff below |

**S8 handoff — MILESTONE CLOSE (2026-08-01, `scutemob-173`)**

- **A scripted human plays five full games with nothing swept under the rug.**
  `crates/simulator/tests/local_game_playthrough.rs` drives seat 1 through four-player
  games on seeds 1/7/42/1234/9001 with a deterministic policy (land → cheapest castable
  → attack → pass), through `LocalGame` alone. All five reach the turn cap with **0
  engine rejections and 0 invariant violations**. The five games run over the **real**
  1,804-def pool through `setup::build_initial_state`, not the 99-Plains fixture the rest
  of `crates/simulator/tests` uses, on a hand-built 64 MiB thread (deep resolution
  exhausts the 2 MiB test stack — pre-existing, OOS-DP3-9).

- **Running it found four defects in one afternoon, which is the argument for the test.**
  None was in the plan; each was fixed at its own layer.
  1. **`invariants::check_stack_consistency` compared two different id spaces.** A cast
     spell's card gets `ObjectId` *n* in the Stack zone and its `StackObject` gets *n+1*
     (`casting.rs` mints them consecutively), so the check fired **twice per spell** and
     once per ability, always, in games with no defect. Measured: **501 spurious
     violations across 500 fuzz games** at the merge base, **0** after. Rewritten against
     `StackObjectKind::Spell { source_object }`, the id the two sides actually share.
     This is what `OOS-DP3-9`'s "long games trip `stack_consistency`" always was.
  2. **`mana_solver` tapped one permanent twice.** It held one entry per (permanent ×
     mana ability) and marked only the chosen entry spent, so a permanent with two mana
     abilities was planned into two `TapForMana`s; the second is refused ("already
     tapped"). Fixed with `spend()`, which marks every entry for the permanent.
  3. **`HeuristicBot` froze the table, twice over.** It scores every real play above
     `PassPriority`, so a *free repeatable* action loops forever: `lightning_greaves`'
     Equip `{0}` (which resolves as a **no-op** — its `ActivatedAbility` declares
     `targets: []` while its effect names `DeclaredTarget { index: 0 }`), and
     re-declaring the same combat. A per-turn preference cap (`RepeatKey`) fixes both,
     **in the bot rather than the provider**, so the fuzzer's `RandomBot` draw sequence
     is untouched.
  4. **The playthrough's own `max_commands` was too tight.** `GameDriver`'s
     `max_turns * 200` is the fuzzer's ratio and the fuzzer's games start with empty
     hands; a real four-player table runs ~260 commands/turn, so the *command* valve
     fired before the turn cap and the plan's terminal state was unreachable.

- **Two of those bottomed out in the engine and were filed, not fixed** (M11-local makes
  no engine change — an empty `git diff` over `crates/engine/src` proves it):
  - **`OOS-M11-7`** — CR 704.3 says SBAs are checked whenever a player *would receive*
    priority. This engine checks them on **step entry** and at **resolution**, not on
    each pass within a step, so a Treasure sacrificed to pay a mana cost sits legally in
    the graveyard for several priority passes. Self-healing, never wrong at rest —
    the playthrough asserts the strictly stronger property that **no token is outside the
    battlefield in the final state**, and reports the transient class separately.
  - **`OOS-M11-9`** — neither `StubProvider` nor `combat.rs::handle_declare_attackers`
    gates "attackers have already been declared this combat". CR 508.1 makes it a
    turn-based action performed **once**; the engine accepts a second, a third, and so on.
    With a vigilant attacker (still untapped, so still `eligible`) this is unbounded.

- **Item 2's premise was stale and that is the reusable part.** The plan (2026-07-26)
  lists Echo / Cumulative Upkeep / Recover as needing new `LegalAction` variants.
  **PB-DP4 (`scutemob-152`) had already shipped all three**, with SR-38 affordability
  gating, later the same day the plan was written. Only `OrderBlockers` (CR 509.2) was
  genuinely unsurfaced. *A plan item that names missing work is a dated claim; check the
  code before building it.* Three tests now verify the existing three reach a human seat
  through `LocalGame`, which is the half `legal_actions.rs`'s own tests do not cover.

- **`Concede` and `OrderBlockers` are offered to human seats ONLY**, appended by
  `local_game::human_only_actions` rather than by `StubProvider`. Two independent reasons,
  both load-bearing: a bot must never auto-concede, and *appending to the provider's list
  re-rolls every `RandomBot` draw downstream of it*, which would change what every
  recorded fuzz seed reproduces. That constraint is what let the R11 gate be **measured**
  rather than argued.

- **The R11 fuzz gate, measured** (`memory/m11/s8-fuzz-parity.md`): 500 games, same seed,
  merge-base worktree vs branch. **0 games differ in turns, commands or outcome.**
  Violations 501 → 0, all of them finding 1's false positives. The gate **cannot** be run
  at the plan's default `--max-turns 200` — `mtg-fuzzer` stack-overflows at the *merge
  base* (OOS-DP3-9), reproduced single- and multi-threaded and with a 128 MiB
  `RUST_MIN_STACK` — so it is 500 games of up to **40** turns, and the record says so.

- **`GET /api/game/report`** ships the repro artefact: `{seed, config, PROTOCOL +
  fingerprint, HASH, final `public_state_hash`, journal}` plus an "Export report" button.
  It is a **pure read** — it uses `journal()` and not `take_new_records()`, so an export
  cannot swallow event lines the live feed has not shipped (tested). It is also the **one
  payload in `play-server` that is not seat-redacted**, deliberately: a redacted repro is
  not a repro. Safe only because M11-local is one human, three bots, one process, no
  networking. **Re-scope it at M10a.**

- **`OOS-M11-8` CLOSED** (the S7 handoff routed it here): `auto_tap_commands_for` now adds
  `x_value × mana_cost.x_count` generic before planning, so a human can cast an `{X}`
  spell. Verified to discriminate by disabling the fix — *and the first attempt at that
  check was invalid*: clippy `-D warnings` failed the disabled build, cargo reused the
  stale test binary, and the test "passed". **A revert-and-rerun proves nothing unless
  the rebuild succeeded.**

- **Gates at close**: tests **4,097 / 0** (merge base measured at **4,072**, so **+25** —
  2 playthrough, 15 human-action, 8 play-server; the implement phase pinned **4,092/+20**
  and the close-out fix cycle added the other 5, so both figures are real and this is the
  final one — measured by running the suite, and the per-file split re-derived against it
  rather than carried), clippy `-D warnings` clean, `cargo fmt
  --check` clean, `tools/check-defs-fmt.sh` 1,804 defs clean, `cargo build --workspace`
  clean, **PROTOCOL 32 / HASH 70 unmoved** (empty diff over `protocol.rs` / `hash.rs` and
  gate-computed by running the `core` suites), fuzz parity as above.

- **CLOSE-OUT ADDENDUM (2026-08-02, same task, after a kitty crash mid-fix-cycle).** The
  `milestone-reviewer` pass filed **MR-M11-01..21** into
  `docs/mtg-engine-milestone-reviews.md`; the fix cycle it opened was interrupted, and the
  resume finished it. **All 10 HIGH/MEDIUM are now closed, all 8 LOW deliberately open**
  (the CLAUDE.md checklist's LOW-no-fix-phase rule), each LOW re-verified as genuinely
  unchanged rather than assumed. Four things worth carrying:

  - **The HIGH is the one nobody's gate could see, and it is the reusable shape.**
    `GameSummary.seed` shipped on **every** seat payload for three sessions. Since
    `setup::build_initial_state` is deterministic in its config alone and
    `session::config_for` fixes every other input, `(seed, players, mulligan_count)`
    *rebuilds* every bot's opening hand and library order — the exact pair Architecture
    Invariant 7 names, and the exact words of the milestone's own acceptance criterion.
    Both Invariant-7 gates stayed green the whole time, because one searches the body for
    card **names** and the other scans source for omniscient **view-model entry points**,
    and a seed is neither. **A redaction gate checks the channel it was written for; a new
    channel is invisible to it.** There are now three gates for three channels — names,
    reconstruction keys, free-form engine strings — and the table is in the play-server
    README so the next surface starts from three rather than rediscovering two.
  - **A status word is not a disposition.** Every one of the 18 findings still read `OPEN`
    while eight had shipped, and three of the eight had landed their *code* fix without
    the *test* the finding asked for — so three behaviour changes were held by prose.
    Those three tests now exist and were proven to discriminate by execution: with each
    fix reverted its test fails and the other 29 stay green. **The first revert did not
    compile** (two helpers went dead under `-D warnings`), which is the S8 `{X}` lesson
    recurring within the same task: *a revert-and-rerun proves nothing unless the rebuild
    succeeded.*
  - **Two findings are closed on part of what they asked for, and the part left is named
    in the reviews doc rather than implied.** MR-M11-04's companion handler-set gate is an
    M10a item (the narrowing, which makes the *existing* claim true, was taken);
    MR-M11-06's code half is a capability addition, not a defect repair, and its seed
    **`OOS-M11-10`** is filed — which was the half the finding actually flagged, since the
    in-source comment had promised "to be filed for S6/S7" through three sessions that all
    shipped without filing it. *A comment asserting a seed exists is not a seed.*
  - **`HASH 69` in the reviews doc was stale in four places.** The claim ("unmoved by
    M11-local") was true; the number was not — HASH moved 69 → **70** in PB-DX5 on the
    parallel W6 track before this branch forked. Found by reading
    `crates/engine/src/state/hash.rs` rather than carrying the figure forward, which is
    the same move that caught three arithmetic slips inside PB-DX5 itself. This file
    already had it right.

- **What M11-local did NOT deliver, stated plainly**: card images come from Scryfall over
  the network rather than a cache (M14); the bug-report artefact has no free-text
  description field; no automated test spans browser + game, because there is no frontend
  test harness (plan §8 R7 — revisit at M13); `StubProvider` still enumerates no
  Adventure, alt-cost, or Convoke/Improvise/Delve casts (R4); `OOS-M11-2`'s
  layer-resolution half is open.

**S7 handoff (2026-08-01, `scutemob-171`)**

- **A human can now attack, block, and cast a targeted / X / modal spell.** Server side,
  `ActionOptionView` gained `target_slots` (populated from
  `mtg_engine::spell_target_requirements` / `ability_target_requirements` +
  `legal_targets_per_slot`), `target_min`/`target_max` from `target_count_range`, `modes`
  with **per-mode** `target_slots` + ranges, `mode_min`/`mode_max`, and `attack` / `block`
  payloads rendered straight out of the provider's own
  `LegalAction::DeclareAttackers { eligible, targets }` /
  `DeclareBlockers { eligible, attackers }`. Frontend side, four pickers chained by
  `ActionBar` in CR order — `ValuePrompt` (601.2b) → `TargetPicker` (601.2c) →
  `AttackerPicker` (508.1) → `BlockerPicker` (509.1) — accumulating one `params` object and
  submitting once. Click-through goes through the same entry point, so a targeted spell
  cannot be cast targetless from either path. PROTOCOL 32 / HASH 69 unmoved; play-server
  tests 18 → **24**; `npm run build` clean at 143 modules, 0 warnings.

- **The S6 review's three MEDIUMs are all closed, and the asymmetry between them is the
  durable part.** The targeted-spell gap announced itself with a 422 every single time. The
  other two — `DeclareAttackers`/`DeclareBlockers` silently submitting an **empty set**, and
  an activated ability's `{X}` silently announced as **0** — were indistinguishable from a
  normal click. The `declares none` and `X = 0` badges in `ActionBar.svelte` are gone,
  because they were warnings about an absence that is now filled.

- **`needs_x` for activated abilities: the S6 note was true and looked in the wrong place.**
  It said `LegalAction::ActivateAbility` does not carry the ability's `ActivationCost`,
  which is correct — but the action carries `source` and `ability_index`, and those reach
  the **layer-resolved** `Characteristics::activated_abilities` entry, whose
  `cost.mana_cost.x_count` is the answer. `mirror_entity` (deck-legal, `x_count: 1`, one
  click makes every creature 0/0) now gets a real prompt. **Generalisable: "the action does
  not carry X" is not the same claim as "X is unreachable from the action".**

- **A real defect surfaced by populating the field, not by reasoning about it:
  `StackItemView::id` is a `StackObject` id and `Target::Object` names a `GameObject`.**
  Nothing bridged the two, so every target that is a spell on the stack — i.e. every
  counterspell's target — rendered as `(unknown card)`. Observed on a real payload before
  the fix (seed 2 offers `Cast Dispel`; its one candidate came back `"(unknown card)"` while
  the stack held `Dark Ritual`). Fixed by adding **`StackItemView::source_object_id`** to
  `crates/view-model` — the id was already being computed in `build_zones_view` for
  `source_name` and thrown away. **This is a deliberate scope deviation** (plan §4 S7 says
  `tools/play-server` and its frontend); it is additive, exposes strictly less than the
  `source_name` already shipped beside it, and the alternative was leaving counterspell
  targets unlabelled. Exposing the bare id leaks nothing: CR 405.1 makes the stack public,
  `redact_stack` blanks a face-down source's *name*, and a face-down **permanent** already
  keeps its real `object_id` for the same reason.

  The same fix removed a latent hazard on the play-server side: `NameIndex` had been writing
  `item.id` — a `StackObject` id — into a map keyed by `ObjectId`. Nothing looked it up, and
  the stack is inserted last, so a numerically-colliding id could have overwritten a real
  permanent's name. **Two id spaces that both count from small integers, in one map.**

- **New seed `OOS-M11-8`: a non-zero `{X}` cannot be paid for through this API.**
  `LocalGame::auto_tap_commands_for` reads the spell's **printed** `mana_cost` and knows
  nothing about `cast.x_value`, so it taps for the base cost and the engine then refuses the
  cast — observed as `422 "player does not have enough mana to pay the cost"`, not inferred.
  The human's workaround exists and works (tap sources manually first; S3 made auto-tap
  conditional on the pool, so a covered base cost leaves the surplus for X) and is the path
  `test_x_value_is_forwarded_to_cast_spell_data` drives. The fix belongs in
  `crates/simulator`, out of S7's scope. **S8 item 2's "surface the invisible optional
  decisions" audit should pick this up** — it is the same family as `OOS-M11-2`.

- **Fixtures were observed, not chosen.** A temporary `#[ignore]`d probe swept
  `players` ∈ {2, 4} × `seed` ∈ 0..12 through `oneshot` (**no port bound**) and reported per
  game whether the human is ever offered a `DeclareAttackers`, a `DeclareBlockers`, or a
  `CastSpell` with a non-empty candidate list. Exactly **one** swept pair reaches both halves
  of combat: `players: 4, seed: 6` (attackers turn 5, blockers turn 6). `seed: 9` reaches a
  targeted removal spell (Doom Blade) with real creature candidates. Both are pinned as
  `COMBAT_SEED` / `TARGET_SEED` with a note to **re-observe rather than guess** when a
  card-def completeness flip re-deals the decks — which PB-DX4 has already demonstrated it
  will. The probe was deleted; `git diff` over `tools/play-server/src` shows no probe.

  The sweep also established two absences worth recording rather than discovering later:
  **no seeded game in the sweep dealt the human a modal spell or an `{X}` spell**, so the
  `modes` path and the `needs_x`-on-`CastSpell` path are right by construction and
  **unexercised by any test**. Said plainly in the README rather than left implied.

- **A non-vacuity check that failed, and the fix.** The first version of
  `test_action_option_target_slots_match_engine_query` stopped on the first action with any
  non-empty slot — which at `seed: 9` is a slot with **one** candidate. Reversing the
  candidate order inside `action_option_view` left the test **green**, because reversing a
  one-element list changes nothing: the per-slot *order* assertion, the whole point of that
  test, was never being exercised. The fixture now demands a slot with **at least two**
  candidates, and the same perturbation turns it red. `validate_combat_params` was checked
  the same way (neuter it → `test_declare_blockers_rejects_ineligible_blocker` goes red).
  **Carry: "the assertion is present" and "the assertion can fail" are different facts, and
  only the second is worth anything.**

- **The `ModeSelection` lookup is the one engine rule this crate restates, and it is
  recorded as such.** `rules::casting::spell_mode_selection` is `pub(crate)`, so
  `view::action_modes` re-derives it through the public `GameState::card_registry` for the
  spell case (the *ability* case reads the layer-resolved `ActivatedAbility::modes` and
  cannot drift). It is confined to which modes to *offer*; the engine re-validates
  `modes_chosen` on the cast path regardless (CR 601.2b, PB-DP3), so a drift is a wrong
  picker, never a wrong game state. Everything else — target requirements, target legality,
  combat eligibility — is delegated, per plan §1 fact 4.

- **`ModeOptionView.label` is a truncated `Debug` of the mode's `Effect`.** There is no
  per-mode oracle text anywhere in the DSL (`ModeSelection.modes` is a bare `Vec<Effect>`),
  so the label is visibly machine-shaped rather than pretending to be printed text.

- **Review cycle: 0 HIGH / 3 MEDIUM / 4 LOW, all 7 applied — and the sharpest one is a
  correctness bug the tests could not have caught.** `TargetRequirement::UpToN { count }`
  is a **single** requirement worth up to `count` targets (`target_count_range` adds
  `count` to the maximum for it; `validate_targets_inner`'s second pass assigns several
  announced targets to that one slot), but `legal_targets_per_slot` returns one entry per
  *requirement*. The first DTO shape was `Vec<Vec<TargetOptionView>>` plus a **collective**
  `(min, max)`, from which a client cannot tell *which* slot the slack belongs to — so the
  obvious one-pick-per-slot reading silently capped `force_of_vigor` (`Complete` by the
  `#[default]` derive, deck-legal, one `UpToN { count: 2 }`) at destroying **one** of its
  "up to two" targets. Fixed by making a slot a struct: `TargetSlotView { min, max,
  candidates }`, each range computed by handing `target_count_range` a one-element slice so
  it cannot drift from the collective one, with `TargetPicker` multi-selecting up to a
  slot's own `max`. **No test could have found it**: no seeded game in the fixture sweep
  deals such a card, so the multi-select branch still ships unexercised and the README says
  so. **Carry: a DTO that flattens a domain concept ("a slot") onto a container shape ("a
  list of candidates") loses whatever the concept carried besides its contents.**
- **The second MEDIUM is the same class as PB-DX3's: a doc comment contradicted by its own
  code.** `action_option_view`'s `# Cost` block claimed the candidate sweep "runs **only**
  for actions that declare at least one target requirement" — while the function's own
  modal branch calls the sweep once per *mode*, and a per-mode-targeting card's
  option-level requirement list is empty **by design**. So the exact actions the sentence
  called free were the ones paying `modes × slots × candidates`. And `queries.rs` asks in
  terms that this be **measured** before a browser polls it; the comment had substituted an
  argument. Measured with a temporary probe: 4 players / seed 9 / turn 17, 12 actions of
  which 1 targeted, 22 candidates → one `decision_view` ≈ **201 µs**, debug build, mean of
  20. **A first draft of the corrected paragraph carried invented numbers ("24 actions, 91
  candidates, under 3 ms") and the probe contradicted every one of them** — which is the
  whole argument for running the probe rather than reasoning.
- **The third MEDIUM: a redaction test whose leak oracle could not fire.**
  `test_target_option_labels_are_seat_redacted` asserted no other seat's hand-card name
  appears in a target label — but target candidates come only from Battlefield / Stack /
  Graveyard (all public), and `redact_hands` rewrites a hidden hand card's `object_id` to 0,
  so no id it collects can key into a hand entry. Deleting `redact_hands` entirely would
  have left it green. Fixed by adding the assertion that **does** bite — every object label
  equals the name the *seat-redacted* `StateViewModel` carries for that id, re-derived from
  the session rather than read off the payload — verified by perturbation (sourcing the
  label from the id instead of `NameIndex` turns it red: `left: "obj-409", right:
  "Vampire"`). The hand-name loop is kept and **relabelled a forward guard** against a
  future widening of `legal_targets_per_slot` into a hidden zone, not evidence that
  redaction works today. The one reachable divergence at this site — a face-down
  battlefield permanent, CR 708.2a — is unfixtured and said so.
- **And the fix cycle's own record overstated that repair, which the re-review caught —
  the exact failure mode this project keeps hitting.** The perturbation cited as proof
  (sourcing the label from the id instead of `NameIndex`) is the *trivial* one. The
  redaction-relevant perturbation is **building `NameIndex` from the omniscient view**, and
  it was then run: `api.rs::seat_view` was edited to do exactly that and **the whole crate
  stayed green — all 23 tests**, including S5's whole-body sweep
  `test_seat_view_over_http_contains_no_other_hand_card_names`. So no behavioural test in
  this crate guarded the chokepoint at all.
  The reason is structural, not a gap in those tests: `NameIndex` is only ever *queried*
  for ids that appear in an action, a target candidate or a combat list, and every one of
  those is in a public zone, so on every id that ever gets labelled the two views **agree**.
  The only construct that separates them is a face-down battlefield permanent (CR 708.2a),
  which no seeded game reaches.
  Closed the way this project closes an unfalsifiable invariant — with a **source gate**:
  `test_production_code_never_builds_an_omniscient_view` scans the production region of
  every `src/*.rs` (comment- and string-blanked by the existing `code_only`, so a doc
  comment naming the symbol neither satisfies nor trips it) for `from_game_state(` and
  `Viewer::Omniscient`, and **was proven to catch the exact edit above** rather than
  assumed to. Its own non-vacuity check went red on first run and taught the batch
  something: the two needles are not in the same position — `from_game_state(` is used for
  real in the test region (the oracle), while `Viewer::Omniscient` appears **nowhere in
  this crate** and is a forward guard whose *mechanism* is pinned instead. play-server
  tests 23 → **24**.
- Four LOWs, all applied: the `{X}` 422 was observed on `casting.rs`'s `x_count == 0`
  fallback path rather than on a real `{X}` card (the seed row and README now say which);
  `StackItemView::source_object_id`'s leak argument did not cover the hidden-zone source
  `redact_stack`'s own doc raises (now does); `BlockerPicker` cannot express CR 509.1b
  "can block an additional creature" while the server deliberately permits it (recorded as
  a client limitation); README limitation numbering and a stale "16 tests" (the pre-S7
  count was 18 — PB-DX4 added two).
- **Still unverifiable headless, and marked so in the README rather than glossed**: every
  DOM and keyboard behaviour — clicking through the picker chain, Escape aborting a chain
  mid-way, `space` being suppressed while a picker is open, `<select>` default rendering in
  the attacker/blocker pickers. There is still no frontend test harness (plan §8 R7), and
  S6's row 2 stands as the proof that a green `npm run build` says nothing about whether a
  component survives a redacted payload.

**S6 handoff (2026-08-01, `scutemob-169`)**

- **Read this first: the session's one real bug was invisible to every gate it had, and the
  fix is in the replay viewer, not here.** `ZoneHand.svelte` keyed its `#each` on
  `card.object_id`. That is right for the omniscient replay viewer and **fatal** for a
  seat-redacted payload: `redact::redact_hands` replaces every unreadable hand card with
  `hidden_placeholder()`, whose `object_id` is **0**, so three bot hands of seven cards each
  arrive with one distinct key apiece. Svelte 5 evaluates `length > keys.size` and calls
  `each_key_duplicate`, which **throws in production as well as DEV**; with no
  `<svelte:boundary>` the throw escapes the effect flush and takes the mount down. **The play
  surface rendered nothing at all** — while `npm run build` was clean at 135 modules and 0
  warnings, the Rust diff was empty, and 4,040 tests were green. Caught in review by
  evaluating Svelte's own condition against the dumped hands (`7 > 1` per bot seat, `7 > 7`
  false for the human's), not by a build and not by a browser. Fixed **in the shared
  component** as `card.hidden ? \`hidden-${i}\` : card.object_id` — the flag the redactor
  sets, not the sentinel 0 — inert for the viewer, and precisely the reason the plan aliases
  the component instead of copying it. `hidden_placeholder` has one call site, so hands are
  the only zone at risk (checked). **Carry into S7: the viewer's components were written
  against an omniscient view model, and every id-uniqueness assumption in them is now also a
  claim about the redacted one.**
- **The play surface has a UI.** `tools/play-server/frontend` — Svelte 5 runes + Vite 7, the
  same versions as the replay viewer's frontend. Eight source files
  (`App.svelte`, `app.css`, `main.js`, `lib/{api,stores}.js`,
  `lib/{PlayApp,ActionBar,EventFeed}.svelte`) and a `package-lock.json`; `npm run build`
  emits `tools/play-server/dist/`, which S5's `ServeDir` fallback already mounts. Build
  clean: 135 modules, 0 warnings.
- **`$viewer` imports the replay viewer's components rather than copying them, and the claim
  is checked.** The alias is `fileURLToPath(new URL('../../replay-viewer/frontend/src/lib',
  import.meta.url))` — absolute at resolve time, because a bare relative alias target
  resolves against the *importing* file and would break for `src/lib/` importers. Evidence
  both ways: `find frontend/src -type f` lists eight files with no `Zone*` and no
  `PhaseIndicator`, while the production CSS bundle contains those components' scoped rules.
  Promotion to `tools/ui-shared/` stays deferred (plan §8 R8).
- **Zero Rust, and the gate is the whole surface rather than the wire files.**
  `git diff main -- crates/ tools/play-server/src tools/play-server/Cargo.toml` is
  **empty** — zero Rust anywhere. The **only** change outside `tools/play-server` is one
  Svelte component, `tools/replay-viewer/frontend/src/lib/ZoneHand.svelte`, and it is the
  review HIGH above; an earlier draft of this bullet claimed an empty `tools/replay-viewer/`
  diff and the fix cycle falsified it in the same commit that introduced the fix.
  PROTOCOL **32** / HASH **69** unmoved; workspace
  `cargo test --all` **4,040 / 0**; `clippy --workspace --all-targets -D warnings`, `cargo
  fmt --check` and `tools/check-defs-fmt.sh` (1,804 defs) all clean. The test count is
  unchanged from the merge base *by construction* — no Rust test target was added and the
  plan explicitly gives this session no frontend test harness (§4 item 7: S5's API tests are
  the automated coverage).
- **The manual checklist was run, not asserted — and that is the part worth carrying.** A
  temporary `#[ignore]`d probe was added to `main.rs`'s existing `mod tests`, driven through
  `tower::ServiceExt::oneshot` (**binding no port**, per plan §7 constraint 1), and
  **removed again**; the frontend was then validated against the dumped `SeatView` payloads
  rather than against a written-down idea of them. Established at the pinned
  `--seed 0 --players 4 --bot heuristic`: a **7-card** opening hand (Island, Mist Intruder,
  Misdirection, Nyxbloom Ancient, Accorder's Shield, Helm of the Host, Swan Song); a land
  drop through `{index: 1, kind: "PlayLand", object_id: 2}` moving hand 8→7 and battlefield
  0→1; 25 `PassPriority` submissions; **10–21 rendered, seat-redacted `EventView` lines per
  response**; **turn 4** reached in 25 submissions. A second run preferring `CastSpell` was
  needed for the stack, because the land-only policy never put anything on it in three turns
  — it produced `zones.stack: [{id: 404, kind: "spell", source_name: "Accorder's Shield"}]`.
  The two steps that genuinely cannot be checked headlessly (launching the binary; keyboard
  and DOM events) are **marked unverifiable in the README**, not glossed.
- **The mulligan `LegalAction`s are unreachable on this surface.** `legal_actions.rs` and
  `local_game.rs::decision_kind_for` both gate `TakeMulligan`/`KeepHand` and
  `DecisionKind::Mulligan` on `is_first_turn_of_game && turn_number == 0`, while
  `setup::build_initial_state` + `GameStateBuilder` leave a fresh table already *in* turn 1
  — `session.rs::is_pregame`'s own doc says the condition is unsatisfiable, and the payload
  agrees (pregame decision `kind: "Priority"`, one option, `Pass priority`). So the UI gates
  its pregame block on `summary.pregame` alone and uses the dedicated
  `POST /api/game/mulligan`. **"Keep this hand" has no server-side representation at all** —
  `take: false` only re-renders and `pregame` is `command_count == 0` — so it is a
  client-side flag, said out loud in `PlayApp.svelte` rather than hidden.
- **The redactor rewrites a hidden card's `object_id` to 0, and click-through must refuse
  those rather than merely fail to match them.** `redact::hidden_placeholder` emits
  `{hidden: true, name: "Hidden card", object_id: 0}`; the playthrough carried **569** such
  entries, every one id 0, while the lowest id any `ActionOptionView` ever carried was 2.
  There is no collision today — but all seven of a bot's hand cards share one id, so a single
  action about object 0 would make all seven of them submit it. Matching on a sentinel is the
  wrong shape whether or not it currently collides. Scope worth knowing: `hidden` is a field
  of `CardInZoneView`, **not** of `PermanentView`, so an opponent's face-down permanent keeps
  its real id and is matched normally — which is right, since an action naming it is about an
  object the seat can point at without knowing what it is.
- **`DeclareAttackers` / `DeclareBlockers` submit an EMPTY set, silently** (review MEDIUM).
  `params.rs` maps default params straight to `Command::DeclareAttackers { attackers: vec![] }`
  — legal, irreversible, and *quieter* than the targeted-spell case, which at least fails
  loudly. The buttons stay enabled (disabling would deadlock a combat where the declaration
  is the only offered action, and CR 508.1 makes "no attackers" legal) but are marked
  `declares none` with a tooltip, and the README says it plainly. S7's pickers are the fix.
- **An activated ability's `{X}` is announced as 0, and the client cannot tell which
  abilities have one** (second review cycle). `params.rs` maps default params to
  `x_value: None`, read as `unwrap_or(0)`; `view.rs::action_needs_x` answers `CastSpell` only
  (README Limitation 5), so `needs_x` is `false` regardless. Reachable and destructive on a
  deck-legal card: **`mirror_entity`** declares no `completeness` field — `Complete` by the
  `#[default]` derive, the same silent-defect generator PB-DX1 and PB-DX3b each hit — and its
  activated ability has `x_count: 1`, so one click makes every creature 0/0 and the board
  dies to SBAs, with no error to read. Annotated `X = 0` **unconditionally** on the kind
  because there is no flag to branch on; the tag goes away when S7 closes Limitation 5.
  **All three silent-degradation paths are the same hole — the client can only send
  `params: {}` — and only the targeted-spell one fails loudly.**
- **Three review LOWs applied**: `jsconfig.json`'s `$viewer/*` path was off by one directory
  (editor-only; `vite.config.js` was right, so the build never noticed); the "omit and take
  the CLI default" rationale did not hold for `players`, pre-seeded to `'4'` and therefore
  overriding a server run with `--players 6` on every New game; and the event feed keyed its
  `#each` on the array index against a front-truncating window, now on a monotonic `seq`
  stamped at append.
- **Two facts recorded for Session 7.** (1) **`ZoneStack` declares `onCardClick` and never
  invokes it** — a dead prop in the viewer, harmless here (no `LegalAction` with an
  `object_id` names a stack object, `view.rs::action_object`) but load-bearing for S7, which
  renders targets on stack items. (2) A **targeted spell cast from this UI fails with a real
  422** — `{"error":"invalid target: expected 1..=1 target(s) but got 0","kind":"rejected"}`,
  observed, not imagined — because `target_slots` is empty until S7 and the client sends
  `params: {}`. Correct S6 behaviour under CR 601.2c; the error strip surfaces it instead of
  swallowing it, and S7's `TargetPicker` is exactly what closes it.
- **One server-side oddity found and deliberately NOT fixed** (fixing it is a Rust diff this
  session's acceptance criteria forbid): `ActionRequest.params` carries a plain
  `#[serde(default)]`, which routes through `ActionParamsDto`'s **derived** `Default` where
  `auto_tap` is `false`, while an explicit `"params": {}` takes the field's
  `#[serde(default = "default_auto_tap")]` and gets `true`. Omitting `params` and sending
  `{}` are therefore not equivalent, which is not what the DTO's doc comment implies. The
  client always sends `{}`, so it is unaffected. **Reasoned from the source, not executed**,
  and nothing in `tools/play-server` names `auto_tap` at all, so nothing pins it either way —
  a one-test job for S7 or S8.

**S5 handoff (2026-08-01, `scutemob-167`)**

- **A full game is now playable over `curl` alone.** `POST /api/game` → `GET /api/game` →
  `POST /api/game/action` → `POST /api/game/mulligan` → `GET /api/healthz`, plus a `ServeDir`
  fallback to `dist/` for S6's frontend. Tests 4,008 → 4,016 → 4,023 → **4,024** across the two
  review fix cycles (+16 in the crate's inline `mod tests`: 15 `oneshot` HTTP tests plus the
  source gate, which is a plain `#[test]` and builds no router). `git diff main -- crates/engine/src
  crates/card-types/src crates/card-defs/src` is **empty**; PROTOCOL **32** / HASH **69**
  unmoved; `crates/simulator` and `crates/view-model` untouched — S5 needed nothing added to
  either, **including its fix cycle**: MEDIUM 1's root cause is `LocalGame::decision_seq`
  restarting at 0, and the fix is an offset in `PlaySession`, not an edit to the simulator.
- **No port is ever bound, and that is now a gate rather than a promise — but the first
  version of the gate cut in the wrong place.** `TcpListener` / `axum::serve` appear only
  inside `async_main`, which no test calls; all 15 HTTP tests drive
  `build_router(state, &PathBuf::from("nonexistent_dist"))` through
  `tower::ServiceExt::oneshot`. `test_no_socket_symbol_appears_in_the_test_region` now walks
  **every `.rs` file** under the crate's `src/` and `tests/` (rooted at `CARGO_MANIFEST_DIR`,
  so it does not depend on the working directory) and fails on any of the four symbols inside
  a test region — line-anchored `#[cfg(test)]` in a `src/` file, the whole file for a
  `tests/` one. Needles are assembled with `concat!` so it does not match its own source.
  **As first shipped it read `main.rs` alone and cut at the first *textual* `#[cfg(test)]`,
  which is the one spelled out in that file's own module doc comment** — the "test region"
  therefore began at a paragraph of prose and the gate passed only because all four needles
  happen to be typed to the left of the marker in that sentence; its non-vacuity guard was
  satisfiable by the same paragraph. Both fixed in fix cycle 2, with three mutation proofs
  run rather than argued. Plan §7 constraint 1 is machine-held crate-wide for S6/S7.
- **Four review findings worth carrying into S6, because the client will encounter all
  four.** (1) The wire `seq` is **not** `LocalGame`'s `seq`: `PlaySession::seq_base` makes it
  monotonic across restarts and mulligans, because without it game B's first decision reused
  game A's `seq: 1` and a stale tab's post was **accepted with 200** (observed — the new
  game's `command_count` moved 0 → 4). (2) A body the extractor rejects is now **400 in the
  JSON envelope**, not axum's bare `text/plain` **422** — the old behaviour collided with
  this crate's own meaning for 422 ("the engine refused it"), so a client-side typo read as
  an engine rejection; and `POST /api/game {"playerz":9}` used to answer **200 with a default
  game** because `Option<T>`'s `FromRequest` is `.ok()`. An **absent** body still means "use
  the CLI defaults". (3) `POST /api/game` — and only it — recovers from a poisoned session
  mutex, so one engine panic no longer costs a process restart on the surface that exists to
  find engine panics. **That recovery had to be made atomic in fix cycle 2**: as first
  written it cleared the poison flag *before* the fallible rebuild, and `session::new_game`
  fails on a client-supplied seed (a colourless commander's deck is padded with Forests,
  which `validate_deck` refuses under CR 903.5c — 7 failing tables in 180 `(players, seed)`
  pairs), so the `?` left the half-mutated session readable at **200** where the unfixed
  code had answered 500. The corrupt session is now `take()`n in the same straight-line
  block that clears the flag. (4) `GameSummary.seed` is the **base** seed; after a mulligan the table
  came from `redeal_seed(seed, seat, count)`, so a reproducible bug report needs
  `seed` + `players` + `bot` + `mulligan_count` — all four are already in every `GameSummary`.
- **The multi-thread runtime flavor is a correctness requirement, not a performance choice.**
  `tokio::task::block_in_place` **panics** on a current-thread runtime — which is exactly what
  a plain `#[tokio::test]` builds. Every async test carries
  `#[tokio::test(flavor = "multi_thread")]`. The 8 MB worker stacks are the separate,
  inherited reason (`tools/replay-viewer/src/main.rs`'s `fn main` (`:50-65`): deep trigger chains overflow
  tokio's 2 MB default in debug builds). Both facts are commented at the runtime builder and
  in `api.rs`'s module doc — **S6/S7 must not "simplify" either one away.**
- **New engine seed, found by a test refusing to lie about itself.** Writing
  `test_post_action_illegal_target_returns_422` against the first castable spell at seed 0
  (`Accorder's Shield`, a `{0}` artifact with no target requirements) returned **200**: the
  engine **accepts a spurious `Player` target on a spell that requires none**, and records it
  on the stack object. The test was rebuilt to drive three deterministic steps to
  `Cast Dispel` ("counter target spell", CR 601.2c), where the target is genuinely refused →
  `Rejected(GameStateError::InvalidTarget)` → 422, with the same params on `PassPriority`
  asserted alongside as the **400** control (`ParamError::UnsupportedParam`, never reaches the
  engine). **The excess-target acceptance is a real engine-side gap, out of scope for
  M11-local, and is FILED as `OOS-M11-5`** (`docs/audits/decision-point-audit.md` §8.1) so the
  next queue re-rank — which enumerates `OOS-*` tokens — actually sees it. Root cause read in
  source: `validate_targets_inner` skips its entire requirement-matching pass when
  `requirements.is_empty()`, an "existence-only" arm added for **aura/bestow** (which declares
  a target while carrying no `TargetRequirement`) and never scoped to it. Zero exposure through
  the bots — `params.rs` only forwards targets a human announced — so it became reachable only
  when S3 gave a human a way to announce targets at all, which is why nine prior batches did
  not see it.
- **Invariant 7 at the HTTP boundary is pinned in both directions.** Omniscient truth is read
  out of band from the session's `GameState`; after excluding the human's own hand and every
  public zone (battlefield, graveyards, command zone per CR 903.6, exile, stack), **20
  distinct other-seat hand card names** remain and the count is asserted **exactly**, so a
  future change cannot quietly empty the set and turn the search into a no-op. Each name is
  searched for in the **raw response body string**, not the parsed `zones.hand` — S4's review
  HIGH ("redaction follows the rendering site, not the zone") applied forward. All seven of
  the human's own names are asserted **present**, so an empty payload fails. Proven by
  mutation, **re-run against the current 16-test module in fix cycle 2**: flipping
  `seat_view` to `Viewer::Omniscient` reddens exactly
  `test_seat_view_over_http_contains_no_other_hand_card_names`, on `"Aggravated Assault"`,
  while the other **fifteen** stay green.
- **Two facts S7 needs.** (1) `mtg_view_model::redact::viewer_may_identify` is `pub(crate)`
  and not re-exported, so a play-server label physically cannot call it — every label goes
  through a `NameIndex` derived from the already-redacted `StateViewModel`, and an unidentified
  id renders `(hidden card)`. **S7's `target_slots` labels must use the same index**, not
  `state.objects()`. (2) `event_view_for` takes **four** params (`ev, state, player_names,
  viewer`), not the plan §3 sketch's three.
- **Known limitations, all deliberate and documented in `tools/play-server/README.md`**: the
  mulligan rebuilds the **whole table** (CR 903.6 makes the command zone public, so a redeal
  is not invisible; and CR 103.5c's per-seat counts cannot be represented) — a per-seat model
  needs each *bot* seat asked, i.e. a new decision channel; `cards_to_bottom` is refused with
  **400** rather than silently discarded, because `handle_keep_hand` checks it against a
  `PlayerState::mulligan_count` a rebuild always leaves at 0; `GET /api/game` calls the
  idempotent `advance()` and consumes `journal_cursor`; `target_slots` / `modes` are empty
  until S7; `needs_x` answers `CastSpell` only; one game per process; and (added by the fix
  cycle) `GameSummary.seed` is the base seed, not the effective one after a mulligan.
- **Second engine/simulator seed, `OOS-M11-6`, found while probing whether `new_game` is
  client-reachably fallible — and it is.** `random_deck` (`crates/simulator/src/deck.rs`)
  applies the CR 903.5c colour-identity filter correctly to the main deck and then **bypasses
  that same filter 37 lines later** when padding to 99 with basics (filter predicate
  `deck.rs:68`, padding loop `deck.rs:105-110`): `basics_for_colors`
  falls back to **Forest** (identity `{Green}`) for a **colourless** commander, so such a deck
  carries ~34 illegal Forests and `validate_deck` — which S2 deliberately routed
  `build_initial_state` through — refuses the whole table. **Measured: 7 failures in a sweep of
  180 `(players, seed)` pairs** (`players: 2, seed: 17` among them), so roughly one
  client-supplied seed in 25 returns a deck-validation failure instead of a game. There are
  **two** Forest fallbacks and the second is dead — the call site's own `if basics.is_empty()`
  arm has a comment saying *"use Wastes (or just any basic)"* and pushes `forest`. **Not a
  one-line fix**: no `wastes.rs` def exists, so it needs either Wastes authored or colourless
  padding drawn from the identity-legal lands already in the pool (prefer the second — no new
  def, no `Complete` flip). **The fuzzer half is CONFIRMED, not suspected** (checked in the
  third audit): `driver.rs` has no deck reference at all, `validate_deck` appears in
  `crates/simulator` only in `setup.rs`, and `bin/fuzzer.rs:296` calls `random_deck` and feeds
  `GameStateBuilder` directly at `:309`+ from the same `all_cards()` pool. So those decks are
  **played** there, not refused — the blast radius is a **silent CR 903.5c deviation in every
  fuzz run that rolls a colourless commander**, not just a play-server 422. The two
  poison-atomicity tests in `tools/play-server` use this bug as their only trigger; closing it
  needs a replacement failure mode (they fail loudly, not vacuously).
- **The no-WebSocket / no-SSE decision is recorded in the crate README with its reasoning**
  (bots act synchronously inside the human's own request, so the server never holds news the
  client is not already waiting on; a second human seat would break that premise; push is
  M10a's problem). `memory/decisions.md` receives it at **S8**, per plan item 8 — deliberately
  **not** written there yet.

**S4 handoff (2026-08-01, `scutemob-165`)**

- **One view-model implementation now feeds both hosts.** `tools/replay-viewer/src/view_model.rs`
  is gone; the file is `crates/view-model/src/lib.rs` (`git mv`, 91% similarity, additive
  changes only). `tools/replay-viewer` is a consumer and **its 15 tests pass unedited**.
  `crates/engine`/`card-types`/`card-defs` diff vs main is **empty**; PROTOCOL 32 / HASH 69
  unmoved. Tests 3,988 → **3,998**.
- **Redaction follows the RENDERING SITE, not the zone — the review's HIGH, and the thing
  most worth carrying into S5-S7.** The first cut redacted `zones.hand`,
  `zones.battlefield` and `zones.exile`, which are the zones CR calls hidden, and stopped.
  Four other sites read `obj.characteristics.name` **raw** — no layer pass, no entitlement
  check — and each can be handed a face-down object: `StackItemView::source_name`,
  `format_target`, `AttackerView::name` (and its planeswalker `target`), `BlockerView::name`.
  A morph creature that attacks *is* on the battlefield, so the battlefield redaction
  "covered" it in the zone sense while `combat.attackers[i].name` printed its name to the
  whole table. `redact_stack` / `redact_combat` now route all four through the
  already-correct `viewer_may_identify`. **S7 populates `ActionOptionView.target_slots` from
  the engine query surface — those labels are a fifth rendering site and must come from the
  seat-redacted view, not from `state.objects()` directly.**
- **"Renders a name" is too narrow a test for a redaction surface — `is_commander` renders a
  boolean and leaks a name.** The re-review's finding, and the seventh site.
  `build_zones_view` derives `PermanentView::is_commander` from the raw `obj.card_id`, and
  CR 903.3 calls the commander designation *"an attribute of the card itself"*, not a
  characteristic — which is precisely why CR 708.2a's face-down override does not touch it
  and why `calculate_characteristics` structurally **cannot**. So a commander cast face down
  for its morph cost comes back with every characteristic correctly blanked and
  `is_commander: true` intact, and since every opponent already knows which card is your
  commander (CR 903.6 — it started in the command zone) that one boolean resolves the
  identity to exactly one card the instant it enters. Now cleared for non-owners; the test
  asserts the omniscient view *does* flag it before asserting the seat view does not.
  `redact.rs`'s module doc now carries the complete site inventory with a disposition for
  each, including the one deliberate non-redaction (`commander_damage_received`, whose inner
  keys are commander names — but a non-zero entry requires that commander to have dealt
  combat damage, at which point CR 903.10a makes the association public in paper too).
- **A single-seat leak scan is a blind leak scan.** Every scan viewed from alice, and alice
  is the one player whose hand card the fixture also puts on the stack, so her own names
  were never needles — which is why the HIGH above passed six whole-document scans. Fixed by
  looping all four seats. With the new redactions disabled the all-seats test fails on
  `seat 2: leaked "Lightning Bolt"` **while all six plan-named tests stay green**.
- **The exhaustive-match gotcha moved with the file.** `stack_kind_info()`
  (`StackObjectKind`) and `format_keyword()` (`KeywordAbility`) now live in
  `crates/view-model/src/lib.rs`. `cargo build --workspace` is still the gate and is now a
  *harder* one: a missed arm breaks a library two binaries depend on, not one binary's
  private module. `memory/gotchas-infra.md` and the session-loaded auto-memory index are
  both updated; several historical docs still cite the replay-viewer path and are stale.
- **The golden snapshot was captured BEFORE the move** (commit `56d44177`), from pristine
  code, then the source file was restored byte-for-byte. That is what makes
  `test_omniscient_view_is_unchanged_for_fixture_state` a regression guard rather than a
  record of whatever the new code happens to do. Compared as `serde_json::Value`, never as
  a string — `StateViewModel` uses `HashMap` and its iteration order is randomized per
  process. It was regenerated exactly once, for the additive `hidden` field; a structural
  diff showed 12 deltas, every one an added `hidden: false`.
- **The leak that mattered was not the one the plan named.** A face-down *battlefield*
  permanent was already safe: `build_zones_view` runs each permanent through
  `calculate_characteristics` and the layer system applies the CR 708.2a override for
  everyone, so the pristine golden already shows `"name": ""`. The face-down *exiled* card
  leaked its printed name, because `objects_in_zone_as_card_views` reads
  `obj.characteristics.name` raw with no layer pass. Both are redacted explicitly so
  Invariant 7 does not silently depend on the layer system continuing to blank the name.
- **A lookup bug can hide inside privacy behaviour — the sharpest thing in this session.**
  `event_view.rs` first rendered a cast spell's name from `SpellCast.stack_object_id`,
  which **never** resolves: `handle_cast_spell` mints `stack_entry_id =
  state.next_object_id()` (`rules/casting.rs:4401`) solely to build the `StackObject` it
  pushes onto `state.stack_objects()` (`:4529`), and that id is never inserted into
  `state.objects()`. Every cast degraded to the name-free fallback "alice casts a spell" —
  never wrong, never present, and **indistinguishable from correct redaction**, which would
  have quietly made S6's event feed useless for the most common action in the game. Fixed
  to `source_object_id` (`:4732`). The three sibling `card_name` call sites were audited the
  same way against their emission sites and were already right (`CardDrawn.new_object_id` →
  `Hand`, `LandPlayed.new_land_id` → `Battlefield`, `CardDiscarded.new_id` → `Graveyard`).
  **Generalisable: in a redacting renderer, a failed id lookup and a deliberate redaction
  produce the same output. Check every id against its emission site, not just its
  entitlement rule.**
- **`event_view_for` takes a 4th parameter** the plan's sketch omits, `player_names:
  &HashMap<PlayerId, String>` — `GameState` carries `PlayerId`s only, so without it every
  line reads `player_2` and the caller must re-render, putting string formatting back
  *outside* the chokepoint. Display names are public. Same deviation class as S3's
  `alt_cost`.
- **Plan §"Hidden-information filtering point" is stale on one premise**: it says
  `GameEvent::private_to()` "does not exist". It does (PB-DP9, `rules/events.rs`), but by
  its own doc it is "a declaration, not an enforcement point" with no consumer, and it is a
  per-*event* verdict that cannot express per-*field* privacy (`CardDrawn` is public; the
  card's identity is not). `event_view.rs` honours it first and then applies per-field
  entitlement, and its module doc says so for M10a.
- **Face-down redaction keys on `obj.owner`, which is conservative rather than strictly
  correct.** CR 708.5a lets a player who *controls* a face-down permanent look at it, so a
  thief is denied a name they are entitled to. Denying too much never leaks; the reverse
  does. Recorded in `redact.rs` for whoever wants the precise version.
- **Escape hatches need the `test-util` feature.** The fixture uses `objects_mut` /
  `players_mut` / `stack_objects_mut` / `combat_mut`, which are `#[cfg(any(test, feature =
  "test-util"))]` per SR-3. They belong in `[dev-dependencies]` only — putting `test-util`
  in `[dependencies]` would break the seal that `cargo build --workspace` enforces.

**S3 handoff (2026-08-01, `scutemob-163`)**

- **The milestone's crux is closed.** The TUI always sent `targets: Vec::new()`, so any
  spell with a `TargetRequirement` was rejected at `casting.rs:3708` — a human literally
  could not cast Lightning Bolt. `test_human_casts_targeted_spell_through_local_game`
  now casts a targeted spell through `LocalGame::submit` and asserts the damage
  **resolved**, picking its target through the new engine query surface end-to-end.
- **Engine half** — new `crates/engine/src/rules/queries.rs` (read-only, 4 fns,
  re-exported from `lib.rs`, **no new public type**). `casting.rs` gains three shared
  helpers extracted verbatim from `handle_cast_spell` (`card_def_target_requirements`,
  `spell_mode_selection`, `per_mode_target_requirements`) so the query and the cast path
  **cannot drift** — that shared extraction, not the query itself, is the load-bearing
  part of plan item 1. `legal_targets_per_slot` delegates one
  `casting::validate_targets_inner` call per candidate, which is what buys
  hexproof/shroud/protection (`casting.rs:6160`) and player-hexproof (`:6114`) for free
  instead of re-deriving them. SR-9a honoured: `tests/rules/queries.rs` **plus** its
  `mod` line in `tests/rules/main.rs`.
- **Signature deviation, argued not assumed:** `spell_target_requirements` takes a 4th
  parameter `alt_cost: Option<AltCostKind>` that the plan's §3 sketch omits.
  `casting_with_overload` (`casting.rs:1163`) and `casting_with_aftermath` (`:533`) are
  **caster-intent** flags derived from the `CastSpell` command, not derivable from state,
  so without it CR 702.96b is unreachable and the named Overload test is unwritable.
  `AltCostKind` is already public → no new public type, wire fingerprint unmoved.
- **A gate-churn trap, avoided.** The first cut checked Overload eligibility by reading
  `KeywordAbility::Overload` from layer-resolved characteristics. That is a *parallel
  re-derivation* of what `casting.rs:1203` establishes with
  `get_overload_cost(...).is_some()`, and it reclassified Overload from SR-5 `Marker` to
  `Handled`, dragging `keyword_registry.rs`, its gate test and
  `docs/sr-5-keyword-catchall-audit.md` along with it. Replaced with the *same call*
  casting makes; the three collateral files reverted. **Lesson: when a new read of a
  keyword forces an SR-5 reclassification, that is a signal you re-derived something
  instead of delegating to it.** (Aftermath genuinely does read its keyword, mirroring
  `casting.rs:533-538`, so `queries.rs` is honestly added to *its* site list.)
- **Simulator half** — new `crates/simulator/src/params.rs` is now the **single**
  `LegalAction` → `Command` mapping table (`random_bot::action_to_command` delegates;
  RNG survives only to fill `attackers`/`blockers`). `hybrid_choices` /
  `phyrexian_life_payments` forwarded **verbatim** from the `LegalAction` (PB-RS2
  precedent — re-deriving is the OOS-RS-2 drift class); an `any_color` `TapForMana` with
  no `chosen_color` is **rejected**, not defaulted to Colorless (PB-EF12, CR 106.1a/b).
  A param announced on an action with no channel for it is rejected rather than silently
  discarded.
- **Bot parity was proven, not asserted.** `mtg-fuzzer --games 50 --seed 424242 --bot
  random` built at pristine and at refactored code: byte-identical per-seed
  Turns/Commands/Winner/Error across all 50 seeds, identical aggregates. Only difference
  is `stack_consistency` violation *line ordering*, which is the known `OOS-M11-3`
  nondeterminism (total violation count identical).
- **`HumanChoice` is now a struct** (`{ action_index, params }`), not
  `enum HumanChoice::Command(Command)`. `submit` builds the command itself for
  `pending.player`, so a **cross-seat command is structurally unrepresentable** — S1's
  `command_player` runtime guard and its unit test are deleted, exactly as `submit`'s own
  S1 doc comment predicted. The tap-then-cast sequence applies to a **clone** and commits
  only on full success, so a succeeded tap never survives a rejected cast.
- **`OOS-M11-2`'s pool half is CLOSED**: auto-tap now fires only when `params.auto_tap`
  **and** the caster's existing `ManaPool` cannot already cover the cost. The layer-
  resolution half (`mana_solver.rs` reads non-layer-resolved `mana_abilities` at `:35`)
  is **still open** and unowned. `advance()`'s bot-seat auto-tap deliberately still fires
  unconditionally — a bot has no reason to prefer its pool, and touching it would perturb
  the fuzzer parity above.
- **A fixture trap worth carrying into S4-S8:** you cannot pre-fill a player's mana pool
  before `LocalGame::start` and expect it to survive — `start_game` runs through
  Untap/Upkeep and **CR 500.4 empties the pool between steps**. Produce a funded pool
  with a real `TapForMana` submit inside the same step instead.
- Workspace **3,955 → 3,965 / 0** (engine +7, simulator +4, −1 deleted `command_player`
  unit). PROTOCOL **32** / HASH **69** unmoved; diff vs main over
  `crates/engine/src/rules/protocol.rs` and `crates/card-types/` **empty**.
- **S4 is unblocked and stays parallel-safe** with the PB-DX queue: it touches a new
  `crates/view-model` + `tools/replay-viewer`, no engine surface.

**S2 handoff (2026-07-31, `scutemob-161`)**

- Shipped `crates/simulator/src/setup.rs`: `LocalGameConfig` / `DeckSource` / `BotKind` /
  `SetupError` / `build_initial_state` / `redeal`, re-exported from the crate root. One
  `StdRng` seeded from `cfg.seed`, consumed in ascending `PlayerId` order — same seed
  reproduces the same `public_state_hash`. Deck admission runs through the **real**
  `mtg_engine::validate_deck` and refuses on any `DeckViolation` (Architecture Invariant
  9); `start_game`'s `check_all_defs_complete` stays as the independent second line.
  `tools/tui/src/play/app.rs::PlayApp::new` rewired onto it (~55 duplicated lines gone);
  `deck.rs` and `bin/fuzzer.rs` untouched by design. **10 tests** in
  `crates/simulator/tests/setup.rs`; workspace **3,928 → 3,938 / 0**; PROTOCOL 31 / HASH
  68 unmoved; engine + card-types + card-defs diff vs main **empty**.
- **A live Commander bug was found in the lifted logic and FIXED, not seeded.** The old
  TUI setup placed the commander *object* in `ZoneId::Command` but never called
  `GameStateBuilder::player_commander`, so `PlayerState::commander_ids` was **empty** in
  every game it built. That field gates commander tax, the CR 903.9a/704.6d
  command-zone-return SBA, CR 903.10a commander damage, and CR 903.9b's hand/library
  redirects — none of them fired. `mtg-tui`'s play mode had been running non-Commander
  games under a Commander UI. Fixed with two calls to existing public engine API (zero
  engine edits), pinned by `test_setup_registers_commanders_not_just_places_them`.
- **CR cite correction, and it is the reusable lesson.** The session plan's own Session 2
  text cites **CR 103.4** for the seven-card opening hand in items 2 and 7. That is wrong:
  **103.4 is the starting life total** (103.4c = Commander's 40); the seven-card draw and
  the mulligan are both **CR 103.5**, and CR 402.1 restates the draw. Verified against the
  CR via MCP, corrected in six places across `setup.rs` and `tests/setup.rs`. This is the
  *same* stale-cite family as the "CR 103.4b" the PB-DP2 handoff already flagged — the
  plan text was never corrected, so the miscite propagated straight into new code.
  **Anyone working Sessions 3-8 should treat the plan's CR cites as unverified.**
- **`redeal` is a v1 UX path with two honest limitations**, documented in source rather
  than papered over: it rebuilds the whole table, so (a) it re-rolls every seat's
  commander — and the command zone is *public* (CR 903.6), so this is not invisible to
  the other seats; and (b) a single `(seat, mulligan_count)` signature cannot represent a
  partially-decided table, so it discards a hand another seat already kept (CR 103.5:
  "once a player chooses not to take a mulligan, the remaining cards become that player's
  opening hand"). A per-seat mulligan state fixes both and belongs with the Session 5
  play-server pregame flow.
- **Premise honored:** plan §8 R2 was **not** re-filed — `OOS-M11-1` was closed by PB-DP2
  (`scutemob-150`), `handle_take_mulligan` really shuffles, and `redeal` is kept for the
  pregame-UX reason in Q1, not as a correctness workaround.
- **Still open from S1:** `OOS-M11-2` (mana solver ignores the pool, reads
  non-layer-resolved `mana_abilities`) — S3 owns the pool half; the re-rank
  (`scutemob-159`) confirmed its exclusion from the primitive queue. `OOS-M11-3` (fuzzer
  nondeterminism in 150-200+ turn games) untouched.

## Last Handoff

**Date**: 2026-08-01 (worker session, `scutemob-170`)
**Workstream**: W6 (primitives) — **PB-DX5 SHIPPED**, fifth batch of the PB-DX queue
**Task**: `scutemob-170`. Branch `feat/pb-dx5-cr-6112c-lock-the-affected-set-of-a-resolution-genera`, 8 commits.

**Completed**:
- **OOS-OS7-2 CLOSED — CR 611.2c is implemented.** `ContinuousEffect` gains
  `affected_set: Option<OrdSet<ObjectId>>`. `Some(set)` = generated by the resolution of a spell
  or ability; `effect_applies_to` answers by **membership alone** and never re-consults `filter`,
  `chars` or `obj_zone`. `None` = generated by a **static** ability (CR 611.3a — genuinely not
  locked in), which keeps the live re-evaluation it always had. Populated at exactly one site,
  `Effect::ApplyContinuousEffect`, via the new `rules::layers::snapshot_affected_set`, called
  before the effect is pushed so `calculate_characteristics` cannot see the effect being created.
- **`is_effect_active` was deliberately NOT changed**, against the dispatch brief and the task's
  own acceptance criterion, which name both functions. It takes no `object_id`, so a per-object
  locked set is not expressible there; and an effect whose locked set is empty is still *active*
  (CR 611.2b describes an outcome, not non-existence). Ruled correct in review. Pinned by
  `test_is_effect_active_is_unchanged_by_the_snapshot`.
- **The dispatch row's roster was wrong twice over — the sixth consecutive batch in this suite
  whose published roster was wrong before it started.** Enumerated from `all_cards()` rather than
  grep: **116** defs generate a resolution-time continuous effect; **38** use a mass filter
  (29 `Complete`, 8 `partial`, 1 `known_wrong`), not "9 defs / 7 `Complete`". The grep conjunction
  missed the entire `CreaturesYouControl*` family (27 defs — Craterhoof Behemoth, Purphoros,
  Mirror Entity, Triumph of the Hordes, Unbreakable Formation) because the filter name does not
  begin with `All`, and it counted `elvish_dreadlord`, whose only `ApplyContinuousEffect` mention
  is inside a **blocker-note string**. Three separate arithmetic slips were then caught inside the
  batch itself — mine (37/28), the plan's (its own table summed to 38/29), and the implement
  phase's test count (+16 vs the true +17) — each by re-measuring rather than re-reading.
- **The batch closed a second, larger defect and did not know it until review (OOS-DX5-7).**
  `effect_applies_to`'s source-relative arms require `state.objects.get(&source_id)` to still
  exist. For an instant or sorcery, `ctx.source` is the spell's card object, which
  `resolve_top_of_stack_inner` moves to the graveyard **after** effects run — a new object under
  CR 400.7. So pre-fix, *Triumph of the Hordes*, *Unbreakable Formation*, *Goblin Surprise* and
  *Return of the Wildspeaker* applied to **nobody at all** the moment they resolved, which is a
  strictly bigger bug than the "newcomer wrongly gets it" the seed described. Verified empirically
  in the fix cycle (membership read reverted, both board creatures observed collapsing to their
  printed power), not inferred. It is also the only mechanism by which the batch's own T12 could
  fail pre-fix, so T12 had been mislabelled about what it demonstrated.
- **Fingerprints computed, not predicted**: `HASH_SCHEMA_VERSION` 69 → **70** (mandatory; the
  field is hashed), append-only history row added, 43 sentinels re-pinned by **symbol** grep —
  two of which the single-line grep could not see and only the full workspace run with
  `--no-fail-fast` caught. `PROTOCOL_VERSION` **confirmed unmoved at 32** by running
  `--test core protocol_schema`, the falsifier the plan named in advance. `ContinuousEffect` is
  outside the SR-8 wire closure; `git diff` over `rules/protocol.rs` is empty. The PB-DX1 lesson
  ("anything reachable from `Characteristics` is PROTOCOL too") was the reason to check, and here
  it did not apply.
- **Yield 0 flips, exactly as pre-committed** (`tools/authoring-report.py`: 1,137/1,804 = 63.0%,
  body byte-identical, only the regenerated-date header moved). This is a pure engine correctness
  fix that makes 29 existing `Complete` defs behave correctly; no marker moved, so the seeded-deck
  re-deal hazard from PB-DX4 did not fire and the play-server seed pins were not touched.
- **One existing test was asserting the bug while citing CR 611.2c as its justification** —
  `pb_ac3_dynamic_pt_counts.rs::test_set_both_dynamic_locked_at_resolution` claimed the rule
  required the *filter membership* to be re-evaluated live while only the *value* stayed locked.
  Inverted with the rule text quoted, renamed to
  `test_611_2c_new_creature_after_resolution_does_not_get_the_locked_value`, and **strengthened**
  (exact `Some(1)`, the newcomer's own printed power) rather than loosened. No assertion anywhere
  in the batch was weakened.
- **Review 0 HIGH / 6 MEDIUM / 6 LOW, all 12 applied.** Every MEDIUM was the same shape: *a claim
  recorded as measured that had been reasoned to*. Two of them put a false statement into engine
  source — `snapshot_affected_set`'s doc block asserted "verified: no Layer-≤4 divergence exists
  in the roster", which asked the wrong question (the divergence comes from any Layer-≤4 effect
  that **writes** the characteristic the filter reads, and `inkmoth_nexus` does exactly that).
  Fixed, and a real test added (animate a Nexus, then activate Mirror Entity), which discriminates.
- **Probes discriminate, and that was verified independently rather than asserted.** With the
  read-site membership block disabled, **8 of the 15** probes fail (mass -1/-1 newcomer,
  Craterhoof newcomer, control-change retention, Jitte, SBA-after-debuff, phased-out exclusion,
  PB-DP9 abort-and-replay, Layer-≤4 divergence) and exactly the 7 that must be insensitive stay
  green (static anthem in **both** directions, `SingleObject` unchanged, `is_effect_active`
  unchanged, CR 400.7 leave-and-return, phase-in).

**Numbers**: tests 4,048 → **4,066** (+18). Benchmarks within ~1% of the merge base
(`full_turn_4p`, `priority_cycle_4p`, `sba_check`, `board_wipe_4p`; the last, flagged as most
likely to move, measured slightly *faster*) — the snapshot runs once per resolution, not per
layer pass. `cargo clippy -D warnings`, `cargo fmt --check` and `tools/check-defs-fmt.sh` (1,804
defs) clean.

**Seeds**: **OOS-DX5-1..7** in `docs/audits/decision-point-audit.md` §8.1. OOS-DX5-6 was filed as
a checked non-finding and **reopened as a real finding by the fix cycle**; OOS-DX5-7 (the
source-retirement class above) was found only by review.

**Durable lesson for the next batch.** Three arithmetic slips and two false "verified" claims all
came from the same move: writing down a number that was derived rather than read. Every one was
caught by re-running the measurement, and none by re-reading the prose. The corollary that cost
the most here is narrower and worth carrying: **a doc comment that says "verified: none exist"
is a dated claim about a question someone chose**, and the question can be wrong even when the
answer to it is right.

**Left for the collector**: `CLAUDE.md` Current State + Last Updated (updated in-branch by this
worker). `main` moved during this session (`scutemob-171`, M11-local S7) — merge base is
`d568615b`; `tools/` is untouched in both directions, so a `git diff main -- tools/` right now
shows S7's work, not this branch's.

**Commit prefix used**: `scutemob-170:`.

---

## Prior Worker Handoff (PB-DX2, preserved for chain context)

**Date**: 2026-08-01 (worker session, `scutemob-162`)
**Workstream**: W6 (primitives) — **PB-DX2 SHIPPED**, second batch of the PB-DX queue
**Task**: `scutemob-162`. Branch `feat/pb-dx2-gate-the-resolution-time-commands-nothing-gates-oos-d`, 6 commits.

**Completed**:
- **OOS-DP5-7 CLOSED**: `Command::ChooseDredge` had NO pending-state gate at all — `card: None` drew a free card for any player at any time (bypassing the pre-fix decline path, which validated only has_lost/has_conceded), `card: Some(x)` dredged at will. Fixed with design **(b)**: `perform_one_draw`'s `DredgeAvailable` arm records/folds a `PendingDraw` entry into the EXISTING `pending_draws` queue (no new type, no new `GameState` field), and `handle_choose_dredge` requires-and-consumes it before doing anything else. `draw_card_skipping_dredge` deleted, folded into the gated decline arm.
- **A second, previously undocumented CR 614.11a bug fixed in the same edit**: a multi-draw sequence (`Effect::DrawCards{count:3}`) with a dredge card in the graveyard emitted ONE `DredgeChoiceRequired` per remaining draw and destroyed all but one — `draw_cards_for_player` now stops on `DredgeOffered`, and a `perform_remaining_draws` helper (extracted from `resolve_pending_draw`'s tail) discharges the entry's `remaining` count from `handle_choose_dredge`.
- **OOS-DP7-2 CLOSED**: 5 doc sites (not the 2 the seed named) reconciled — every comment claiming the engine "pauses" on `DredgeChoiceRequired` corrected to describe the actual deadline-not-block design. The third site (`events.rs:1354`, `CleanupDiscardChoiceRequired`'s doc) cited this exact seed as "not implemented" and would have become a NEW lying comment if left untouched. `MiracleRevealChoiceRequired`'s identical claim was VERIFIED false (not just suspected) and its underlying CR 702.94a violation is real and live — seeded, not fixed (OOS-DX2-1, needs a HASH bump).
- **Rider OOS-DP2-1 CLOSED**: `handle_keep_hand` validated only the COUNT of `cards_to_bottom`, not that named objects were in the sender's hand — a malformed/hostile command could bottom a battlefield permanent, a graveyard card, or a card from ANOTHER PLAYER'S HAND. Fixed with a per-entry `expect_zone` membership + duplicate-id check, all validation before mutation. `bare_lookup_ratchet` unmoved.
- **Rider OOS-DP9-14 CLOSED (defensive hardening)**: `pending_effect_choice` with a dead owner is now reaped at the top of `resolve_top_of_stack`, narrowly (dead owner only — a live owner's entry still trips the entry `debug_assert!`, pinned by a `#[should_panic]` test).
- **PROTOCOL 32 / HASH 69 both UNMOVED** — confirmed by `git diff --stat` over `rules/protocol.rs` + `state/hash.rs` (empty) and the `core` test group's `hash_schema::*`/`protocol_schema::*` suites, all green. Wire-neutrality was a hard acceptance criterion, not merely a prediction — no fallback bump was applied.
- **A genuine surprise found only by running the golden corpus, not by reasoning about it**: golden script `replacement/014_golgari_grave_troll_dredge.json` — which the plan (and the seed row) both predicted would stay green untouched — turned out to depend on the EXACT exploit this batch closes. Its `type: turn_based_action, action: draw_card` entry is (and always was) purely informational per `script_schema.rs`'s documented contract; no driver dispatches an engine `Command` off it. The script's `initial_state` started already inside the Draw step, so no real draw was ever attempted, and pre-fix `choose_dredge` succeeded regardless. Fixed by starting the script at Upkeep and adding a leading `priority_round` that drives the REAL Upkeep→Draw transition and its CR 504.1 turn-based action — mirroring `crates/engine/tests/mechanics_a_d/dredge.rs`'s `pass_all` unit-test pattern exactly. An append-only dispute entry documents the finding with CR citations; the pre-existing dispute record is untouched.
- Plan `memory/primitives/pb-plan-DX2.md`; seeds **OOS-DX2-1..7** filed in audit §8.1. Tests **3,955 → 3,978** across implement (+16) and two fix cycles (+3, +4) in this worktree. 0 completeness flips, 0 card-def edits — the roster is exactly 1 `Complete` card (`golgari_grave_troll.rs`, `Dredge(6)`), as predicted.

**Fix cycle (same day, `scutemob-162`)**: review verdict needs-fix, 1 HIGH / 7 MEDIUM / 7 LOW, all 15 applied. **The HIGH: the implement-phase "fold guard" (bullet above) turned an unanswered dredge offer into an obligation that accumulated WITHOUT BOUND across turns and could be cashed in one command at an arbitrary later moment, out of priority** — while `events.rs`'s own new doc asserted the opposite. **Fixed by replacing the fold with a discharge**: `perform_one_draw` now auto-resolves (as an implicit decline) any stale entry for a player the instant ANOTHER draw arrives for them, unconditionally, before even checking what the new draw needs — so `pending_draws` never holds more than the single most-recently-offered draw's own remainder. This was ALSO claimed to close **OOS-DX2-3** (two entries per player) as a side effect, on the argument that both `push_back` sites are downstream of the discharge and two entries are therefore "structurally impossible, not merely bounded" — **that proof is FALSE and the seed is REOPENED; see fix cycle 2 below.** The residual — a single entry answerable at an arbitrary later moment — is `OOS-DP5-2`'s pre-existing finding, narrowed (not closed) by this fix and noted in its row rather than re-filed. Six MEDIUMs were doc-vs-code (`PendingDraw`'s own declaration doc, the `GameState` field doc, `handle_order_replacements`'s routing doc, `memory/gotchas-rules.md`, `effects/mod.rs`, and a doc-comment-capture bug where a newly-inserted helper silently stole `resolve_pending_draw`'s doc block — the exact OOS-DP7-2 failure mode, reintroduced by the batch that closes it); the seventh was a genuine coverage hole (`dredge.rs` test 9 silently degraded to testing the entry gate instead of the graveyard-zone check it was named for). PROTOCOL 32 / HASH 69 confirmed still unmoved after the fix cycle. Tests 3,971 → **3,974** (+3: cross-player rejection + the two untested cross-kind cells of plan §3.3's four-case table). Full disposition table: `memory/primitives/pb-review-DX2.md`'s "Fix cycle" appendix.

**Fix cycle 2 (same day, `scutemob-162`)** — a re-review of the fix cycle itself, because fix cycle 1's HIGH fix rewrote `perform_one_draw`'s control flow rather than patching it. Verdict needs-fix again: **1 HIGH / 3 MEDIUM / 5 LOW, all 9 applied.** The HIGH is that fix cycle 1's own closure of `OOS-DX2-3` rested on a false proof — "both `push_back` sites live downstream of the discharge" is a claim about **where** the pushes are, not **when** they run. `resolve_declined_pending_draw` **re-enters** `perform_one_draw`; the inner call's discharge check finds the queue already emptied by its caller and skips, but its own `check_would_draw_replacement` can independently return `NeedsChoice` (CR 616.1f excludes only replacements that were *applied*, not merely offered) and push a fresh entry — after which the outer call pushes its own. **Reproduced empirically before any fix was written**: one extra `draw_card` on the existing T19 fixture yields `pending_draws().len() == 2`. The seed is **REOPENED**; the corrected invariant ("at most one *dredge-originated* entry per player") replaces the false one at all seven asserting sites, including two FIFO arguments that dismissed ordering ambiguity because "there is never a second candidate" and a termination proof that assumed its own conclusion. **No engine behaviour changed and no wire moved — the record was wrong, not the code**, and corpus exposure is zero (no card def registers a `WouldDraw` replacement). The real entry count is now pinned by a test instead of by prose. Also fixed: the discharge is itself a new engine-made auto-decline recorded in no decision-point row and *not* outcome-neutral (a discharged draw takes the library top at the LATER moment, so an intervening scry/shuffle/mill changes which card is taken) — filed as **OOS-DX2-7**, a fresh instance of OOS-DP10-9; a missing test for "discharge produced events AND the current draw took `Proceed`", verified non-vacuous by injecting the exact regression; two `handle_choose_dredge` `Some`-arm validations that still had zero coverage repo-wide; a duplicate `PlayerLost` when the discharge decks a player out (Architecture Invariant 4); and **six sites citing CR 104.3b for the empty-library loss when the rule is CR 104.3c** (MCP-verified; 104.3b is life total 0 or less). Tests 3,974 → **3,978**. PROTOCOL 32 / HASH 69 still unmoved. Full disposition: `memory/primitives/pb-review-DX2.md`'s re-review appendix.

**Three things worth carrying into the next batch**:
0. **A fix cycle's own repair is unreviewed work, and it earned a second HIGH here.** Fix cycle 1 replaced a fold with a discharge — a control-flow rewrite on the hottest path in the draw system, larger than any option the review offered — and shipped a false structural proof with it. If a fix cycle rewrites rather than patches, re-review it. Also: **cite by symbol, not by line, in a doc-heavy batch.** Two cite drifts appeared *inside* this batch's own cite corrections, one in a row that claimed the number had been re-verified.
1. **"The golden corpus will stay green" is a claim to verify, not a fact to assume — even when the plan explicitly traced the script's action sequence.** The plan's own §1 P7 walked `replacement/014`'s actions and concluded it "reaches the offer first"; that trace missed that the `turn_based_action` label it read is purely informational and dispatches nothing. Only actually running the full suite (not just the new unit tests) surfaced it. Run the full golden corpus before declaring done, every batch, even when a targeted grep says the roster is narrow.
2. **A batch that adds a legitimate gate can retroactively convict a test fixture that was silently relying on the absence of that gate.** This is the golden-script sibling of PB-DX1's "fixing an engine gap can convert a dormant def-level approximation into a live bug" lesson — here it converted a dormant TEST-FIXTURE approximation into a visible test failure, which is the better outcome (a script that "passes" by exercising an exploit was never really testing what its own name and assertions claimed).

**Next**: **PB-DX3** (OOS-DP6-3 — 2 flips, `garruks_uprising` + `inventors_fair`, 0 engine lines) per `memory/primitives/seed-rerank-2026-07-27.md` §4. Independent of PB-DX1/PB-DX2 and of M11-local.

---

**Date**: 2026-08-01 (worker session, `scutemob-160`)
**Workstream**: W6 (primitives) — **PB-DX1 SHIPPED**, the first batch of the PB-DX queue
**Task**: `scutemob-160`. Branch `feat/pb-dx1-the-intervening-if-dropped-in-the-runtime-lowering-oo`, 6 commits.

**Completed**:
- **OOS-DP6-1 CLOSED** + both riders (**OOS-DP6-5**, **OOS-DP6-9**). CR 603.4 now holds at both ends for the lowered runtime path. **PROTOCOL 31→32 / HASH 68→69** (both gate-computed, histories append-only, 44 sentinel files re-pinned via the symbol grep). Tests **3,928 → 3,945**. Benches within 5% (`full_turn_4p` 214.6→217.4 µs).
- Plan `memory/primitives/pb-plan-DX1.md`; review `memory/primitives/pb-review-DX1.md` (1 HIGH / 5 MEDIUM / 4 LOW, **all 10 applied**); seeds **OOS-DX1-1..6** in audit §8.1.
- Card defs: `karlach_fury_of_avernus` `known_wrong`→**`Complete`** (1 flip); `aurelia_the_warleader` re-authored `once_per_turn: true`; `tatyova_steward_of_tides` threshold `(6)`→`(7)`, **stays `partial`**.

**Four things worth carrying into the next batch**:
1. **A wire prediction on any type reachable from `Characteristics` is a PROTOCOL bump too, not just HASH.** The §4 brief predicted HASH only; `Characteristics` is in `protocol_schema.rs`'s `CLOSURE_MUST_CONTAIN`, so `TriggeredAbilityDef` and `InterveningIf` were in the wire closure all along. Planning caught this *and stated the falsifier in advance* — which is the process working. **This bears directly on PB-DX5's row.**
2. **Fixing an engine gap can convert a dormant def-level approximation into a live bug, in the *suppressing* direction.** Two instances in this one batch: Aurelia's `IsFirstCombatPhase` proxy (the review's HIGH — a regression *this batch introduced*, caught only because the reviewer re-derived the oracle rather than trusting the plan's deferral rationale) and Tatyova's `(6)`-means-6 threshold. **Before any batch that starts evaluating a previously-discarded field, sweep the corpus for arguments authored as "approximations" against it.**
3. **The same lowering has now been caught dropping three fields**: `intervening_if` (the headline), `once_per_turn` (found in planning, fixed in batch, 3 `Complete` defs were over-firing) and `trigger_zone` (OOS-DX1-3, still open). A lossy-lowering table is now a module comment on `build_face_ability_vectors` specifically so a fourth is not discovered the same way.
4. **A plan's rationale for deferring something is a claim like any other.** The plan declined to re-author Aurelia because it "would change which mechanism T1 exercises"; the reviewer falsified that with the batch's own T12b. Reviewers should treat deferral rationales as reviewable, not as scope boundaries.

**Next**: **PB-DX2** (OOS-DP5-7 + OOS-DP7-2 — `ChooseDredge` has no pending-state gate; wire-neutral), then **PB-DX3** (2 flips, 0 engine lines). Both independent of PB-DX1 and of M11-local. **✅ PB-DX2 shipped — see the Last Handoff section above.**

---

**Date**: 2026-07-26..27 (oversight session — autonomous coordinator chain, user-directed "task out the PB suite and run autonomously", then "task it out and rerank"; /eot 2026-07-27)
**Workstream**: W6 (PB-DP suite) — **DP1..DP10 ALL SHIPPED + seed re-rank DONE; queue handoff to PB-DX**
**Task**: `scutemob-149..158` (suite) + `scutemob-159` (re-rank). Final merges `16ffcfd0` (DP10) and `0dd79b5d` (re-rank).

**Completed**:
- **THE PB-DP SUITE IS COMPLETE** — all 10 batches dispatched, collected, merge-verified (full test suite run on main after every merge). Tests 3,683 → **3,928 / 0**; PROTOCOL 27 → **31**; HASH 63 → **68**. All five Tier-0 correctness findings (DP-1..DP-5) closed. Per-batch detail: CLAUDE.md "Last Updated" entries + `docs/audits/decision-point-audit.md` §5/§8 rows (each marked SHIPPED with verified breakdowns) + git merges `f7651bb5`/`68172717`/`3b04bd17`/`799dcc0a`/`922252f7`/`d52fe5b6`/`8f890611`/`48353a36`/`d65e7f1e`/`16ffcfd0`.
- **Seeds closed by the suite**: OOS-M11-1 (DP2), OOS-M11-4 (DP8), OOS-DP1-1 + OOS-RS3-4 (DP4), OOS-DP7-7 (DP10) — plus OOS-RS3-1, discovered closed by DP6 only during the re-rank census.
- **Seed re-rank** (`scutemob-159`, merge `0dd79b5d`, docs-only): 204-seed census, 7 closures source-verified, RS5..RS11 dispositioned (only ex-RS6 gained rank; ex-RS5 demoted — its obvious fix is a trap), phantom seed OOS-RS1-2 struck. **Successor queue PB-DX1..DX18** in `memory/primitives/seed-rerank-2026-07-27.md` §4 (authoritative; rider-seed-triage §5 banner defused). Honest yield ~13-15 flips + ~15 integrity repairs + 3 gate-integrity fixes.

**Not done / deferred**:
- ~~PB-DX queue not started~~ — **PB-DX1 SHIPPED** (`scutemob-160`, 2026-08-01); OOS-DP6-1 + riders DP6-5/DP6-9 all CLOSED. **Next dispatch is PB-DX2**, then PB-DX3.
- M11-local S2 (pregame setup + mulligans) unblocked and parallel-safe; scutemob-127 (abilities-corpus distillation) still backlog; M10 line untouched.

**Next session candidates** (highest-yield first):
- ~~Dispatch PB-DX1~~ **DONE** (`scutemob-160`; PROTOCOL 31→32 / HASH 68→69 — the §4 brief predicted HASH only and was half wrong). **Dispatch PB-DX2** (`ChooseDredge` free-card exploit, wire-neutral) — then chain DX3 (2 flips, 0 engine lines) under the standing autonomous-chaining rule.
- M11-local S2 in parallel (`crates/simulator` only — disjoint from DX1/DX2 engine surface).

**Hazards** (carrying forward):
- All prior hazards stand (attestation verbatim, Monitor-not-poll-loops, `esm update` clobber, probe-first, never skip the reviewer).
- **Merge conflicts in coordination files are routine** on worker branches that update CLAUDE.md/workstream-state (DP1, DP9): resolve by taking the worker's richer version, then reconcile counts in the collect chore commit. `git merge-tree --write-tree` remains the conflict arbiter (`esm worktree check` false-positives persist).
- **Audit rosters are magnitudes, not rosters** — SR-36 enumeration beat the §3.1 regex every time (84→77, 74→73, 7→2). Trust only computed counts; DP10's gate now ratchets them.
- **The CR 800.4 concede/departure priority-strand class bit three batches** before DP9's engine-wide backstop; watch for it in any new blocking-decision work (PB-DX10 adds one).
- **Worker state-sync is inconsistent** — some workers update CLAUDE.md/workstream-state in-branch, some don't; the collect step must check and reconcile every time (N4).

**Commit prefix used**: worker `W6-prim:`, `merge:`, coordinator `chore:`.

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


### PB-DP suite — worker close-out detail

> Rotated out at /eot 2026-07-27. The per-batch close-outs (DP2..DP10 designs, deviations,
> seed lists) live in: CLAUDE.md "Last Updated" (DP9/DP10 verbatim), the audit doc
> `docs/audits/decision-point-audit.md` §5/§8/§8.1 (every row updated at ship time), and the
> merge commits listed in the Last Handoff above.

## Handoff History

### 2026-07-19 (oversight — PB-OS queue complete, OS4..OS11 + OS4b) [rotated]

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
- ~~**PB-DX1**~~ ✅ **SHIPPED `scutemob-160`** (2026-08-01): **OOS-DP6-1 CLOSED**, with both riders (**OOS-DP6-5** TurnFaceUp resolution re-check, **OOS-DP6-9** haunt at both ends — and *both* their §8.1 cites were stale and corrected on closure: `resolution.rs:7369`→`:7564`, `:5351`→`:5494`). Fix **(a)** as a **variant**, `InterveningIf::CardDef(Box<Condition>)`, not a field on `TriggeredAbilityDef`: a field would have forced `None` at ~140 struct literals across 84 files and left a permanent "did you read the other field too?" hazard; the variant costs zero construction-site churn, makes an unclassified case a compile error, and repairs all 13 queue sites + the 1 resolution site at once because every dispatch already routes through `check_intervening_if`. Alternative **(b)** rejected on a ground the seed never anticipated — a registry re-read discards `layers::expect_characteristics`, so Humility/Dress Down would stop suppressing all 34 trigger events (a CR 613.1f regression *larger than the bug*), and it discards every runtime filter the lowering exists to carry and breaks tokens/copies. Alternative **(c)** rejected with evidence stronger than OOS-DP6-2: `replay_harness.rs:2642`/`:2781` already carry an *"Index-namespace fix (2026-07-09)"* comment recording that **this exact trick on this exact function** was the root cause of the Monastery Mentor / Leaf-Crowned Visionary filter-bypass bug — and the fix applied then was (a). A three-valued `InterveningIfMoment { TriggerTime, TriggerTimeLookBack, Resolution }` classifies the 14 call sites (8/5/1), independently re-derived twice. **PROTOCOL 31→32 AND HASH 68→69 — the brief's "HASH only" was half wrong** (`Characteristics` is in `CLOSURE_MUST_CONTAIN`); planning predicted the correction *and stated the falsifier in advance*. **Two things the brief did not contain.** (1) `once_per_turn` is dropped by the **same** lowering at 31 of 34 sites, and three `Complete` deck-legal defs over-fire (`welcoming_vampire`, `elvish_warmaster`, `whispering_wizard` — `elvish_warmaster`'s is a *self-reinforcing cascade*, since the Elf token it makes re-qualifies its own trigger); wire-neutral, fixed in the same batch. (2) The **review's HIGH was a regression this batch introduced**: `aurelia_the_warleader`'s `Condition::IsFirstCombatPhase` is a *proxy* for "attacks for the first time each turn", and once the condition actually started being evaluated the proxy began **suppressing** a legitimate trigger (first attack occurring in an extra combat granted by Aggravated Assault / Moraug / Port Razer) — the one failure direction PB-DP6's hard constraint 3 forbids. The plan had declined to re-author her on the grounds it would "change which mechanism T1 exercises"; the reviewer **falsified that using the batch's own T12b**, which already drives the identical shape on Karlach. Yield honesty: **1 flip, not 2** — `karlach_fury_of_avernus` `known_wrong`→`Complete` (MCP ruling #11 verbatim: *"Karlach doesn't have to be among the attacking creatures"*), `tatyova_steward_of_tides` **stays `partial`** (two untouched blockers), and the review caught its `ControlAtLeastNOtherLands(6)` meaning 6 where oracle says **seven** — inert before this batch because the condition was being discarded, live afterwards. **Next: PB-DX2** (`ChooseDredge` free-card exploit, wire-neutral), then **PB-DX3** (2 flips, zero engine change).
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

### 2026-07-08..10 (oversight session; /eot 2026-07-16) — W6: PB-AC chain close (AC0..AC9 complete)

- PB-AC4..AC9 dispatched/collected (`scutemob-46/47/49/50/51/52`).
- **PB-AC4** (`dca25ec0`): `ModeSelection.mode_targets` per-mode targeting (CR 601.2c) + Escalate fail-safe; backfill 11 migrated. Tests 2940→2957.
- **PB-AC5** (`0ce2c470`): Warp, Transmute, Exert (both shapes), `Cost::ExileFromHand`+Pitch, `CounterSpell.exile_instead`; 2 HIGH unhashed-field fixes. Tests →2984.
- **PB-AC6** (`0628807e`): main-phase sweeps, `WhenBecomesTarget`, 5 Conditions, 3 PlayerState trackers. Tests →3009.
- **PB-AC7** (`2f214906`): `SetCreatureTypes`/`SetCardTypes` Layer 4 (CR 205.1a correlated-subtype HIGH; CR 613.8 depends_on). Tests →3035.
- **PB-AC8** (`a2aea440`): `CantAttackOwner`, `CantBeSacrificed` (both choke points), `Effect::WinGame` (worker corrected inverted CR 104.3h). Tests →3062.
- **PB-AC9** (`a4750cdb`): `WheelHand` + `SetNoMaximumHandSize`; **token doubling rewired 2→13/13 sites** (doublers silently failing); Reforge stale-marker HIGH → both workers recommended the marker sweep (executed this session). Tests →3090; coverage 983 (56.2%) at chain close.
- Hazards that stayed load-bearing: recon-first (2-3 primitives per PB already existed); HashInto omissions as review HIGHs (engineered out via mutation-verified hash tests in criteria); worker-overturns-brief 3×; `build --workspace` ≠ test compile but IS the seal gate; CR file bare `\r` — use MCP, never grep.

