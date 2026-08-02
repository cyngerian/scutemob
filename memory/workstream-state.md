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
| W6: Primitive + Card Authoring | — | available (PB-DX6 shipped; **next PB-DX19**, NOT PB-DX7 — re-ranked by `scutemob-182`) | 2026-08-02 | **PB-OS queue COMPLETE** (OS1..OS11 + OS4b, `scutemob-116..141`). **Rider-seed queue**: RS1..RS4 SHIPPED (`scutemob-143..146`); plan `memory/primitives/rider-seed-triage-2026-07-19.md`, resume at **R5** per its §5 banner (weigh OOS-RS3-1 insert + OOS-RS2-1 rider). **The PB-DP suite now runs FIRST** (user directive 2026-07-26) — queue `docs/audits/decision-point-audit.md` §8, from the decision-point audit (`scutemob-148`). **PB-DP1 SHIPPED** (`scutemob-149`, merged `f7651bb5`): priority after cast/activate/special action goes to the ACTOR per CR 117.3c; 14 Group-A sites + 8 Group-D sites; entry priority guards added to `handle_turn_face_up`/`handle_activate_loyalty_ability`/`handle_level_up_class`; 19 tests + 15 golden scripts reconciled; PROTOCOL 27 / HASH 63 unchanged; 3,721 tests green. Seeds **OOS-DP1-1..4** filed in the audit doc **§8.1** (durable inventory for this suite — not in `primitive-wip.md`, which the next PB overwrites). Suite tasked out `scutemob-150..158` = PB-DP2..DP10. **PB-DP2 SHIPPED** (`scutemob-150`, commit `f902010f`): mulligan content no-op + bottom-to-top, **CR 103.5/103.5c** (the brief's "103.4b" is a stale cite — see the handoff below); **OOS-M11-1 CLOSED**; 4 probes; PROTOCOL 27 / HASH 63 unchanged; tests 3,721 → **3,725**. Seeds **OOS-DP2-1..6** filed in the audit doc **§8.1**. **PB-DP3 SHIPPED** (`scutemob-151`, DP-4 `min_modes` floor, **CR 601.2b/700.2a**): mode announcement is now mandatory — the fix is a **lift** of the range/duplicate/`min_modes`/`max_modes` checks out of the `!modes_chosen.is_empty()` gate, not the audit's prescribed Spree-guard mirror, so it fixed **40** modal defs (3 commands + 37 `min_modes: 1`) rather than the 3 the row predicted, plus the identical activated-ability bypass in `abilities.rs` (audit §4.2). Narrow CR 702.120a escalate exemption; `resolution.rs`'s `vec![0]` fallback **retained** (4 free-cast producers bypass `handle_cast_spell`). PROTOCOL 27 / HASH 63 unchanged; 0 card-def edits; tests 3,725 → **3,747**. Seeds **OOS-DP3-1..9** filed in the audit doc **§8.1**. **PB-DP4 SHIPPED** (`scutemob-152`, merged `799dcc0a`): DP-10 attack tax now debited (colour-correct; restricted mana excluded per CR 106.6; hybrid/Phyrexian/X tax rejected — OOS-DP4-1); DP-11 enforced as a **deadline** (auto-decline unanswered payments at `handle_all_passed`'s stack-empty branch per CR 118.12a — a priority gate would deadlock the driver); 5 Complete defs made right, 0 def edits; **OOS-DP1-1 + OOS-RS3-4 CLOSED**; seeds OOS-DP4-1..13 filed in audit §8.1; PROTOCOL 27 / HASH 63 unchanged; tests 3,747 → **3,781**. **PB-DP5 SHIPPED** (`scutemob-153`, merged `922252f7`): `pending_draws` on `GameState`, `OrderReplacements` routed by applicability; **3 emit sites fixed, not 2** (`draw_card_skipping_dredge` was a third the audit never named); also fixed a CR 614.11a sequence bug (`DrawCards{count:3}` emitted 3 unanswerable prompts and drew 0 — now one prompt, remainder stashed) and a review-found CR 616.1f loop gap in `determine_action`; **HASH 63 → 64** (gate-forced), PROTOCOL 27 unmoved; tests 3,781 → **3,797**; 0 def edits. NOTE: audit premise falsified — **0 of 1,804 defs register a `WouldDraw` replacement**, so card yield is 0; this is an engine-correctness fix + precondition for authoring the WouldDraw family. **PB-DP6 SHIPPED** (`scutemob-154`, merged `d52fe5b6`): intervening-if now evaluated at trigger-queue time across the queue paths (audit §4.8 queue-time row D→A); no wire change (PROTOCOL 27 / HASH 64); tests 3,797 → **3,809**; seeds OOS-DP6-1..10 filed in audit §8.1 (note OOS-DP6-10: the one hazard this batch INTRODUCED — A9 `WasKicked` suppression, wrong-direction, zero corpus exposure today). **No-wire block DP1..DP6 COMPLETE. PB-DP7 SHIPPED** (`scutemob-155`, merged `8f890611`): cleanup-discard is now a blocking player Command (CR 514.1); the **blocking pending-decision mechanism is proven** for DP8/DP9 reuse; **PROTOCOL 27 → 28, HASH 64 → 65** (both gate-computed); tests 3,809 → **3,837**; 2 fix cycles (18 + 6 findings; 2 HIGH: CR 800.4j dead-player entry skipping CR 514.2, out-of-step answer accepted; plus a TUI auto-pass livelock introduced-and-fixed); seeds OOS-DP7-1..12 in audit §8.1 — **OOS-DP7-11 flags that the SR-19 HashInto gate silently skips path-qualified impls** (gate-integrity seed, rankable). **PB-DP8 SHIPPED** (`scutemob-156`, trigger-target choice, CR **603.3d/601.2c/603.3b**): `Command::ChooseTriggerTargets` + `GameEvent::TriggerTargetChoiceRequired` (disc. 130) + `GameState.pending_trigger_targets` suspend `flush_pending_triggers` MID-BATCH and resume it on the controller's answer; the compliant CR 603.3d fallback survives verbatim as the exported `abilities::default_trigger_targets`, which the CALLER submits as a real Command (engine still knows nothing about seat kind). **PROTOCOL 28 → 29, HASH 65 → 66** (all five fingerprints gate-computed; `TriggerTargetOption` + `SpellTarget` enter the wire closure — a genuine type-count change, unlike DP7's). **Roster 77, not the audit's 84 nor the planner's 74** — enumerated from `all_cards()` per SR-36 and printed by a test. **0 card-def edits, 0 completeness flips**, but **2 live-wrong `Complete` cards fixed by accident** (sword_of_sinew_and_steel, elder_deep_fiend): the plan's premise that a permanent-inner `UpToN` slot 'contributed 0 targets' is FALSE — it returned `None` and the caller removed the WHOLE TRIGGER, so those cast/damage triggers had never once reached the stack. CR 601.2c makes zero targets a legal announcement. Second plan gap found and fixed: §4.1 never says who grants the priority the four guards were about to grant, so a resumed game would have had nobody holding priority — added `grant_priority_on_resume` on the entry. Consult set is **4 guards, not the ~20 DP7's row predicted**, because PB-DP1 moved priority assignment ahead of the flush (all 30 `check_and_flush_triggers` sites verified to need none). Also fixed `local_game.rs`'s latent variant-blindness (hard-coded `DecisionKind::CleanupDiscard`) — now compile-forced. tests 3,837 → **3,858**; 53+1 sentinels re-pinned across 44 files; 1 golden script corrected with CR justification. Seeds **OOS-DP8-1..10** in audit §8.1 (DP8-9/DP8-10 are new relative to the plan). **OOS-M11-4 CLOSED.** OOS-DP3-4 deliberately NOT bundled — ranked as **PB-DP8b** (OOS-DP8-7). **PB-DP9 SHIPPED** (`scutemob-157`, search/scry/surveil, CR **608.2d / 701.23a / 701.22a / 701.25a**): the engine's **first resolution-time decision channel**. `GameState.pending_effect_choice` + `GameEvent::EffectChoiceRequired` (disc. 131) → `Command::AnswerEffectChoice`, backed by an **abort-and-replay** continuation, NOT the "resumable effect-list cursor on the stack object" `pb-plan-DP7.md` §1.6 and audit §8 both prescribed — that is **impossible**, because `resolve_top_of_stack` POPS the stack object before any effect runs. Instead: clone at entry, an effect that needs an unanswered choice records the question and returns, the wrapper restores the clone **wholesale** and emits one event, the answer is banked on `GameState` and the resolution re-runs **from the top**, retracing the identical deterministic path. Consequences worth carrying: no continuation data structure at all (`Sequence`/`Conditional`/`ForEach`/`Repeat` need zero machinery — the replay re-executes them); the re-entrancy audit is **3 units, not 20** (15 of 17 production `execute_effect` callers are inside `resolve_top_of_stack` itself, one is gated by CR 605.4a, one is provably unreachable); **PB-DP8's "a guard that returns early inherits a debt" bug class does not exist here** because a total restore skipped nothing; and "the suspended object leaves the stack" is structurally unreachable rather than a live hazard. **ONE `Command` for all three effects** (CR 608.2d is one rule; 701.22a/23a/25a are three instances) — so one gate entry, one `LegalAction`, one `DecisionKind`, one harness action string, which **corrects OOS-DP8-14's prediction of three**. **PROTOCOL 30 → 31, HASH 67 → 68** (all gate-computed, histories append-only, 44 sentinel files re-pinned via the SYMBOL grep). **Roster 69 / 16 / 7, not the audit's 74 / 16 / 8** — enumerated from `all_cards()` with a RECURSIVE `Effect`-tree walk (a flat scan undercounts). **0 def edits, 0 flips.** Three in-scope correctness fixes beyond agency: CR **701.22b** (`Scry 0` was emitting `Scried{count:0}`; the surveil arm had the mirror guard, the scry arm did not), CR **400.7** (scry-to-bottom RENUMBERED every scried card and consumed `timestamp_counter`, the shuffle seed source — now `Zone::reposition_within`; sweep seeded as OOS-DP9-11), and CR **701.23d** (a quantity-only search with one candidate is determined and asks nothing). Two deliberate deviations, both argued in source and pinned by tests: **the scry/surveil defaults FLIP to the identity** (search keeps its lowest-id default byte-for-byte), and **the three new fields are EXCLUDED from `loop_detection.rs`'s fingerprint** unlike DP7's and DP8's, because they grow between replays of one resolution and could mask a CR 726 loop — recorded as **obligation (7)** on the `BlockingDecision` doc block, the first evidence that list generalises. `GameEvent::private_to()` now exists (OOS-DP8-6's declaration half; a declaration, NOT an enforcement point — nothing consumes it until M10). Benches measured against `48353a36`: `full_turn_4p` 253 → 229 µs, **no regression** from the per-resolution clone. Fallout 25 unit tests + 1 golden script, every repair CR-justified. tests 3,878 → **3,905**; seeds **OOS-DP9-1..12** in audit §8.1 (rank **OOS-DP9-3** first — `SearchLibrary` finds exactly one card, ~7 partial defs, zero new plumbing on this machinery). Merged `d65e7f1e`; on-main verified **3,910 / 0** (5 more than the branch pin — post-merge count). **PB-DP10 SHIPPED** (`scutemob-158`, decision-gate widening, **test-only** — and it **CLOSES THE PB-DP SUITE**): two new files under `crates/engine/tests/core/` — `decision_site_walk.rs` (the canonical serde walk + `ROWS`, all 22 decision sites of audit §3.1 classified **4 SERVED / 15 AUTO-CHOSEN / 2 GATED / 1 NO-DECISION**, each with the engine site that was *read* to establish the class) and `decision_gate.rs` (`BASELINE`, 97 name-keyed entries with exact row sets, + 18 tests). **The headline is a gate-integrity finding, not a feature**: every serde walk in this codebase before now (`effect_choose_gate.rs`, `pb_rs1_roster_sweep.rs`, `pb_dp9_effect_choice.rs::roster`) matched **object keys only** and is therefore blind to a **unit** `Effect` variant — `serde_json::to_value(Effect::Proliferate)` is `Value::String("Proliferate")` — so a verbatim reuse would have reported **0** for Proliferate's 25 `Complete` defs *while looking green*, the exact OOS-DP7-11 failure mode. Fixed with a two-shape walk + a `PROSE_FIELDS` denylist, pinned in both directions against the legacy walk (T2/T3). Measured: all-rows union **267** (the audit's 277 analogue), still-auto union **97**, live denominator 1,139/1,804. **Fail-closed proven end-to-end on a real def**, not just synthetically: adding `Effect::Proliferate` to `lightning_bolt.rs` reddened **two** tests naming the card, the row, the CR and the engine site; restored → green. Two hand-maintained zeros (`AddManaFilterChoice`, `TheRingTemptsYou`) that **nothing** was holding became machine-checked (the SR-33 gate bars a *different* key). **PROTOCOL 31 / HASH 68 unmoved; engine + card-types + card-defs diff vs main EMPTY; 0 def edits, 0 flips.** Review 2 HIGH / 6 MEDIUM / 6 LOW, **all 14 applied** — the HIGHs are worth carrying: (1) `BASELINE` was populated **mechanically** and the plan's class-B/class-D triage was never done, so a spot-check found two class-D defs already inside the frozen baseline (Smuggler's Copter's "you **may** draw" authored as an unconditional `Sequence`; Shambling Ghast with a permanent `-1/-1` counter, an `oracle_text` saying "enters" against a `WhenDies` trigger, and a `Decayed` keyword the printed card does not have) — seeded as **OOS-DP10-8**, not demoted; (2) **the gate can only see a decision the DSL ENCODED**, and that blind class is strictly *worse* than the one it records (**OOS-DP10-9**, and the instrument for it is an oracle-text-vs-DSL cross-check, not a variant walk). tests 3,910 → **3,928**; seeds **OOS-DP10-1..11** in audit §8.1; **closes OOS-DP7-7** (the 277-def re-derivation is now computed, printed and ratcheted every run). Audit §8 now carries a suite-COMPLETE banner and §10 an honest 3-of-8 mechanization ledger. Merged `16ffcfd0`, on-main verified 3,928/0; suite retrospective in the audit doc §8. **Next: re-rank RS5..RS11 against the unranked seeds** (OOS-DP9-3 was the previous first pick; OOS-DP10-8/-9 are new and OOS-DP10-6 is the successor queue's ranked input). **Seed re-rank SHIPPED** (`scutemob-159`) — successor queue `memory/primitives/seed-rerank-2026-07-27.md` §4, PB-DX1..DX18. **PB-DX1 SHIPPED** (`scutemob-160`): OOS-DP6-1 + riders CLOSED; PROTOCOL 31→32 / HASH 68→69; tests 3,928→3,945. **PB-DX2 SHIPPED** (`scutemob-162`): OOS-DP5-7 + OOS-DP7-2 + riders OOS-DP2-1/OOS-DP9-14 all CLOSED — see the Last Handoff section below for detail; PROTOCOL 32 / HASH 69 unmoved; tests 3,945→3,971 (this worktree's own baseline was 3,955 after the intervening M11-local S2 merge, so the batch's own delta is +16). **Fix cycle same day**: review found the implement-phase "fold guard" was a HIGH (unbounded cross-turn accumulation, cashable out-of-priority) — replaced with a discharge design that also closes OOS-DX2-3 as a side effect; 7 doc-vs-code MEDIUMs + 1 coverage-hole MEDIUM + 7 LOWs all applied; PROTOCOL 32 / HASH 69 still unmoved; tests 3,971→**3,974**. **PB-DX3 SHIPPED** (`scutemob-164`, 2026-08-01): **OOS-DP6-3 CLOSED** — `garruks_uprising` + `inventors_fair` both `partial` → **`Complete`**, coverage 1,140 → **1,142** (63.2% → 63.3%), tests 3,988 → **3,998** (+10 probes), **0 engine lines** (empty `git diff` over the whole of `crates/engine/src` *and* `crates/card-types/src`, not just the wire files) and PROTOCOL 32 / HASH 69 unmoved. Review 0 HIGH / 1 MEDIUM / 5 LOW, all applied. **Three things the queue row did not contain, in ascending order of how much they matter.** (1) `inventors_fair`'s upkeep trigger **did not exist at all** — the seed and both blocker notes read as though it were present but ungated, so the batch had to *author* the ability. (2) The runtime `InterveningIf` enum both notes name now has **three** variants, not the two they cite: PB-DX1 added `InterveningIf::CardDef` two batches earlier. The stale notes were stale twice over, and this queue introduced the second staleness itself. (3) **The MEDIUM was the batch reproducing its own subject.** The test module recorded a pre-fix observation for T1 ("the hand count was 1") that **could not have been observed** against T1's own fixture, which had no library object — and an empty-library draw sets `has_lost` (`replacement.rs:1035-1049`) rather than incrementing the hand, so the companion assertion passed whether or not the bug fired. Fixed by giving T1 a real library card and **re-running the pre-fix scenario empirically** (reverting `intervening_if` to `None` and reading the numbers), not by repairing the prose; the same standard was then applied to T3/T5/T6/T7/T8, all of which held. The original claim was right — it had simply never been checked against a fixture where the number meant anything, and that distinction is the whole lesson. `reveal: true` on `Effect::SearchLibrary` is inert (pre-existing **OOS-DP9-9**) and now carries an in-def comment saying so rather than being silently covered by the `Complete` marker. **New seed OOS-DX3-1** (audit §8.1): six more defs sit in the same `pb-plan-DP6.md:395` stale-blocker bucket and **`jadar_ghoulcaller_of_nephalia` is a live-wrong `Complete` def** — `intervening_if: None`, so it makes a 2/2 Zombie **every** end step unconditionally, and its stored `oracle_text` names a token-name filter the printed card never had (MCP: the real text is "if you control no creatures with decayed"). Expressible today as `Not(YouControlNOrMoreWithFilter{count:1, filter: Creature + has_keywords[Decayed]})`; the fix must also reconcile golden script `combat/191`. `ophiomancer` (`partial`, its own note already says "Blocker stale") and `dwynen_s_elite` (`inert`) are two more flips in the same shape. **Next: PB-DX4** (OOS-DP10-8, the 97-entry `BASELINE` triage) — but consider inserting OOS-DX3-1's Jadar half first: live-wrong `Complete`, card-def only.  **PB-DX3b SHIPPED** (`scutemob-166`, 2026-08-01 — a **queue insert ahead of PB-DX4**, taken on the post-DX3 banner's own recommendation): **OOS-DX3-1 CLOSED**. All **seven** remaining defs of the `pb-plan-DP6.md:395` stale-blocker bucket dispositioned explicitly — 4 fixed, 3 deferred with blockers re-affirmed against the *current* `Condition` enum rather than copied forward. `jadar_ghoulcaller_of_nephalia` stays `Complete` and is now CR 603.4-gated; **its stored `oracle_text` was wrong, not merely its blocker note** (the field said "tokens named Shambling Ghast"; MCP says "creatures with decayed"), so the note had been declaring a DSL gap for a filter the card never had — a distinct failure mode from PB-DX3's stale-note class. `ophiomancer` `partial` → `Complete` (`has_subtype: Snake` alone, deliberately not `ControlCreatureWithSubtype`, whose arm hard-requires `CardType::Creature`). `dwynen_s_elite` `inert` → `Complete`, ability **authored from nothing** — the `inventors_fair` shape recurring; expect it. **The seed itself mis-dispositioned a second live-wrong `Complete`**: `emeria_the_sky_ruin` declares no `completeness` field, so it was `Complete` by the `#[default]` derive and reanimated every upkeep regardless of Plains count — the `aurelia_the_warleader` trap from PB-DX1, hit a second time in three batches by a different route. Gated, given an **explicit** `partial` for the DSL-inexpressible "you may" (OOS-DP10-8 class, falsifier search actually run), and a spurious `Legendary` supertype removed (MCP type line is `Land`). **2 flips up, 1 honest flip down — net coverage 1,142 → 1,143, +1 not +3**; 0 engine lines (empty diff over all of `crates/engine/src` + `crates/card-types/src`); PROTOCOL 32 / HASH 69 unmoved; tests 4,008 → **4,022** (this branch's merge base is 4,008, not the 3,998 DX3 pin — `scutemob-165` merged in between). Golden script `combat/191` reconciled by **strengthening** (it had never asserted the Zombie token and passed either way). Review 0 HIGH / 5 MEDIUM / 7 LOW, all 12 applied. New seed **OOS-DX3b-1** (`guardian_project`'s `is_nontoken` half is authorable today; its name-uniqueness half is not, so it stays `known_wrong`). **Durable**: `#[default] Completeness::Complete` is now a twice-demonstrated silent-defect generator — "which defs never declare a marker at all?" is a cheap corpus-wide question nobody has asked. **PB-DX4 SHIPPED** (`scutemob-168`, 2026-08-01): **OOS-DP10-8 CLOSED**, and **OOS-M11-6 closed incidentally**. All 97 `BASELINE` entries read against MCP printed text, roster parsed out of the const array itself (97 → 97 distinct names → 97 unique def files) rather than taken from prose, because this suite has published a wrong roster three times. **Split 84 class-B / 13 class-D** — PB-DP10's 2-of-5 spot-check overstated the D rate ~5x and its own "very noisy sample" caution was right; the queue row's "0 flips" estimate was wrong the other way, since 5 of the 11 had to be demoted. **5 repaired, still `Complete`**: `metastatic_evangel` (4 defects: `{2}{W}`→`{1}{W}`, missing `Human`, P/T transposed 1/3→3/1, and a **stale** in-def note claiming `is_token` is ignored on the ETB path — PB-AC0 had made that false), `grisly_salvage` + `satyr_wayfinder` (`RevealAndRoute` routes ALL matches → `LookAtTopThenPlace{optional:true}`; printed says "**a** card", "you **may**"), `sword_of_truth_and_justice` (bare `TargetCreature` → `controller: You`), `radstorm` (`{2}{U}`→`{3}{U}`). **6 demoted with oracle citations**: `smugglers_copter` → `known_wrong` (20th DP-12 instance; the other 19 already were, so the marker was the defect), `contaminant_grafter` / `grateful_apparition` / `thrasios_triton_hero` → `partial`, and `shambling_ghast` → `partial` **for a defect the fix surfaced** — its three named deviations (phantom `Decayed`, permanent `MinusOneMinusOne` for a printed "until end of turn", `oracle_text` saying "enters" against `WhenDies`) were all FIXED, and the marker is for a fourth: the mode-1 target is flat, so taking the Treasure mode still needs an opponent creature (CR 603.3d). **`mode_targets` is honoured only on the CASTING path** — nothing on the trigger path reads it — so the obvious repair would have DROPPED the requirement rather than scoped it (**OOS-DX4-2**; `hullbreaker_horror` is a second member). **1 left `Complete` deliberately**: `staff_of_compleation`'s "target permanent you own" as `TargetController::You`, allowlisted to match the shipped `nether_traitor` decision for the identical owner-vs-controller class (**OOS-DX4-1**) rather than reporting a corpus class as two cards. **OOS-M11-6 found by accident**: demoting `thrasios_triton_hero` — a legendary creature, i.e. a member of `random_deck`'s own commander pool — re-dealt every seeded deck in the workspace and landed seed 9001 on Rograkh, the corpus's ONLY colourless `Complete` legendary creature (1 of 91). Fixed as that seed preferred (pad from the identity-legal colourless pool; measured 40 colourless lands + 82 nonlands = 122 singletons vs 99 needed), **both** Forest fallbacks removed. The bigger half: the fuzzer feeds `random_deck` straight to `GameStateBuilder` with no validation, so it had been silently **playing** CR 903.5c-illegal decks. Six fixtures across two crates broke; the two play-server rebuild tests lost their only failure trigger exactly as their own maintenance note predicted and now use a sentinel **seed** (a first attempt used a process-global flag that raced with every other test POSTing `/api/game` — green under `-p`, red under `--workspace`, twice). Golden script `baseline/112` **retired**: it tested Decayed on a card that does not have it, citing the card *def* as its authority — a provenance failure. CR 702.147a keeps 12 unit tests; golden-level gap filed as **OOS-DX4-3**. Coverage 1,143 → **1,137** (63.0%), tests 4,040 → **4,048**, `BASELINE` 97 → **91** (moved twice inside the batch, 97→93→92, which is why it was read off the gate not computed), deviation floor 661 → **667**, DP8 roster 76 → **74**, `scry` 16 → **15** — each re-measured against `all_cards()`. **0 engine lines** (empty diff over `crates/engine/src` *and* `crates/card-types/src`), PROTOCOL 32 / HASH 69 unmoved. **PB-DX3b's `#[default]` question answered and bigger than expected: 966 of 1,804 def files never mention `completeness` at all (970 before this batch)** — a clear majority of the `Complete` population, and **eleven of the thirteen** class-D defs were in it; now ratcheted in the growth direction. Durable record `memory/primitives/pb-dx4-baseline-triage.md` (per-def citations + an explicit statement of what the triage does NOT establish: it is a dated claim, it cannot see a decision the DSL never encoded — OOS-DP10-9 stands — and 97 of 1,143 is not a sample the rest can be inferred from). Seeds **OOS-DX4-1..6**. **PB-DX5 SHIPPED** (`scutemob-170`, 2026-08-01): **OOS-OS7-2 CLOSED — CR 611.2c**, `ContinuousEffect` gains `affected_set: Option<OrdSet<ObjectId>>`, populated only by `Effect::ApplyContinuousEffect` via the new `rules::layers::snapshot_affected_set` (never elsewhere) and read as pure membership by `effect_applies_to`; `None` means a static ability (CR 611.3a, unchanged, still live-evaluated). **The dispatch row's roster ("9 defs, 7 `Complete`") was wrong twice over — the sixth consecutive batch in this suite whose published roster was wrong before it started.** `all_cards()`, enumerated fresh: **116** defs generate a resolution-time continuous effect at all; **38** use a mass filter (not 9); and even the premise-verification step's own corrected figure (37, 28 `Complete`) was off by one against its OWN table, which already listed 38 rows summing to 29 — an uncaught arithmetic slip nobody re-added. **Final measured: 38 mass-filter defs, 29 `Complete`, 8 `partial`, 1 `known_wrong`**, from a new self-re-measuring test (`pb_dx5_mass_filter_roster_by_completeness`) rather than a pinned count. Mechanical backfill of `affected_set: None` at all 180 pre-existing `ContinuousEffect` construction sites (49 files, compiler-driven, zero manual judgement calls — every site is either a static registration or a `SingleObject` effect, and `None` is the RULE at the former, a no-op at the latter). **HASH 69 → 70** (mandatory, gate-forced); **PROTOCOL confirmed unmoved at 32** by actually running `--test core protocol_schema`, not assumed (`ContinuousEffect` is outside the SR-8 wire closure). **Yield 0 completeness flips, exactly as pre-committed** — a pure correctness fix for defs already `Complete`; coverage stays 1,137/1,804 (63.0%, byte-identical regen). **Existing-test repair, exactly as flagged a hazard in advance**: `pb_ac3_dynamic_pt_counts.rs`'s `test_set_both_dynamic_locked_at_resolution` was asserting the CR 611.2c bug this batch fixes ("a creature entering after resolution still gets the locked-in X=3") and had been passing; inverted with a CR cite and renamed, not weakened. Every "fails before" claim in the new 14-test probe module was OBSERVED (read-site membership check reverted, actual value recorded, restored) — which caught the runner's OWN first-draft T3 (control-change retention) using the buffed creature as its own effect source, masking the very divergence it claimed to test; fixed with a separate, never-moved source. T11 (zone-scope shortcut vs. brute force) lives as an in-source `#[cfg(test)]` unit test in `rules/layers.rs`, not the integration file — `snapshot_affected_set`/`effect_applies_to_object`/`candidate_ids_for_filter` are all `pub(crate)`. Benchmarks all within ~1% of the merge base; `board_wipe_4p` (flagged most likely to move) measured slightly *faster*. Golden corpus unaffected — Final Showdown's script exercises mode 2 (DestroyAll), not the `AllCreatures|Ability` mode 0 the roster found (a pre-existing, documented DSL-gap omission in the script itself). Six new seeds **OOS-DX5-1..5** + a checked non-finding **OOS-DX5-6** (Mirror Entity is the one Layer ≤4 mass-filter def; unaffected today — nothing in the roster writes `CardType::Creature` via a Layer-4 modification). Tests 4,048 → **4,064**. **Same-day fix cycle** (review `pb-review-DX5.md`, 0 HIGH / 6 MEDIUM / 6 LOW, all 12 applied, none changing observable behaviour): the test-count arithmetic above was itself off by one (the roster file has two `#[test]`s, not one — true implement-phase total was **4,065**, and the fix cycle's own +1 new test (T15) makes the re-run total **4,066**); OOS-DX5-6's "checked non-finding" was FALSE — a real, reachable, CR-correct divergence exists (animate Inkmoth Nexus + Mirror Entity's `AddAllCreatureTypes`), now pinned by T15 and corrected in the seed doc; the fix was found to close a SECOND, larger pre-existing defect (every source-relative mass filter on an instant/sorcery applied to nobody once the spell resolved, CR 400.7) — confirmed empirically, filed as **OOS-DX5-7 (CLOSED as a side effect)**; OOS-DX5-1 widened to name three read sites that ignore `affected_set`; a stale note in `pb_os7_defending_player_continuous_filter.rs` that declared this batch's own seed a live limitation was corrected; T11's fixture was genuinely enriched (phased-out permanent, real `AttachedCreature` match, subtype filter); a vacuous `debug_assert!` hardened; the non-fixed-window-duration question (plan §3 Q4) is now a measured, standing assertion (zero corpus members) rather than a spot check. HASH 70 / PROTOCOL 32 re-confirmed unmoved by re-running both schema gates. **Next: PB-DX6** (OOS-RS2-1 + OOS-DP4-1 — the two mana-cost payment sites PB-RS2 left unflattened; `handle_turn_face_up` pays a raw `def.mana_cost` and `can_spend`'s residue guard is `debug_assert`-only, so in release every hybrid/Phyrexian pip in a `TurnFaceUpMethod::ManaCost` flip is FREE — `kitchen_finks` is `Complete` with two `{G/W}` pips; and `Command::DeclareAttackers` has no `hybrid_choices`/`phyrexian_life_payments` fields at all. One PROTOCOL bump for the batch; also make the residue guard fail loud). Full brief in `memory/primitives/seed-rerank-2026-07-27.md` §"Dispatch briefs", whose stale "Next dispatch: PB-DX5" banner was struck through at PB-DX5 close rather than left to become the N4 re-dispatch hazard a fourth time. Two PB-DX5 residuals worth weighing as inserts first: the probe module rebuilds Craterhoof/Jitte/Mirror Entity encodings by hand rather than instantiating the corpus defs, so def-vs-probe drift would go unnoticed (the `all_cards()` roster test and the fix's filter-agnosticism mitigate but do not close it); and `rules/face.rs`'s deregistration sites match on `source == obj_id && filter == resolved_filter`, which cannot distinguish a static registration from a resolution-generated effect sharing both — `affected_set.is_none()` is now a ready-made discriminator, a different class from the three read sites OOS-DX5-1 names and a candidate to fold into it. | **PB-DX6 SHIPPED** (`scutemob-172`, 2026-08-02): **OOS-RS2-1 + OOS-DP4-1 both CLOSED** — the last two unflattened mana-cost payment sites. `handle_turn_face_up` flattens in **all three** `TurnFaceUpMethod` arms (the brief named only `ManaCost`; all three share one payment block), and `Command::DeclareAttackers` gains the two PB-RS2 payment fields so a hybrid or Phyrexian CR 508.1h attack tax is **payable** rather than rejected — pips replicated **copy-major** into the total, total flattened once, because design (B) (flatten-then-multiply) is *rules-wrong* on the Norn's Annex ruling that each cost is chosen **individually**. `unpayable_tax_defenders` → `x_tax_defenders`, narrowed to X only. New read-only `rules::queries::attack_tax_total` — the attack tax is the one payment cost a client cannot derive — with **exactly one** shared accumulation. `ManaPool::can_spend` is now fail-**closed** on an unflattened residue in every build and `spend` asserts unconditionally: the guard PB-RS2's own review described as firing "NEVER" in release was failing **open**, i.e. silently **undercharging**. **PROTOCOL 32 → 33 computed** from the gate's own output (falsifier named in advance did not occur; closure type count unchanged at 96); **HASH confirmed unmoved at 70 by running the gate**. **0 completeness flips**, pre-committed and held (empty `git diff` over `crates/card-defs`), coverage holds at **1,137/1,804 = 63.0%**, tests 4,066 → **4,099**. Review **1 HIGH / 8 MEDIUM / 6 LOW, all 15 applied** and each re-verified by execution first, because the reviewer had no shell. The HIGH: the copy-major order-pin test **could not fail** under the permutation it existed to catch, while the batch's own freshly-written doc claimed it could — the PB-DX5 "verified: none exist" class reproduced inside the batch citing it. Second finding: this batch silently **removed** PB-DP4's E1 CR 508.1c regression coverage (both pins used a hybrid restriction, no longer a rejection class) — verified by reverting E1, then moved to `x_count: 1`. Seeds **OOS-DX6-1..5**; **OOS-DP4-7 re-dispositioned, not closed**. **QUEUE RE-RANKED 2026-08-02 (`scutemob-182`) — the authoritative queue is now `memory/primitives/seed-rerank-2026-08-02.md` §4 (v3); `seed-rerank-2026-07-27.md` §4 is SUPERSEDED. NEXT IS PB-DX19 (OOS-SIM2-6 + OOS-SIM2-5), NOT PB-DX7** — PB-DX7 survives unchanged at rank 9, displaced by eight entries that are live-wrong on deck-legal `Complete` cards or, in PB-DX19's case, a hard process abort. See the `scutemob-182` handoff below.

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
  resume finished it. **All 10 HIGH/MEDIUM are now closed; of 8 LOW, 1 closed and 7
  open**, each of the seven re-verified as genuinely unchanged rather than assumed. The
  blanket "LOW needs no fix phase" was only half the account and is worth correcting here:
  the reviewer's `memory/m11-fix-session-plan.md` had scoped **four** LOWs into its two
  sessions. **MR-M11-12** was taken (a doc cite pointing at a sentence that does not exist
  — the lying-cite class, doc-only, and the fix documents *both* halves of `OOS-M11-2`,
  the second verified at the read site rather than copied from CLAUDE.md);
  **MR-M11-13/14/17** were deferred with the reason recorded at each item, MR-M11-14 on
  the plan's own advice, since its Session 2 gate names that item as one of the two that
  can perturb the 500-game fuzz parity — and that parity run is the branch's evidence for
  acceptance criterion 5977. The plan's checkboxes are now accurate rather than untouched.
  Five things worth carrying:

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
  - **A fix plan nobody ticks reads as a fix plan nobody ran.** `m11-fix-session-plan.md`
    still had all fourteen boxes unchecked while eleven of its items had shipped — which
    is the same failure as the reviews doc's eighteen `OPEN` rows, in a second file, and
    it is what made the close-out's first account of the LOWs wrong (it said all eight
    were untouched-by-design; four had actually been *scoped into sessions*, so three of
    them needed a stated deferral rather than a blanket rule). Both files are now
    accurate. **The generalisable bit: the artefact a reviewer produces is a second place
    the work has to be recorded, and finishing the work does not update it.**

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
  > **VOIDED as evidence by SIM-3 (`scutemob-177`, 2026-08-02).** Those
  > `stack_consistency` lines were false positives by construction — the check compared
  > `StackObject::id` against the Stack-zone `ObjectId`, two id namespaces CR 400.7
  > guarantees will differ. So "only difference is their line ordering" is a statement
  > about the ordering of noise, and cites `OOS-M11-3` for something that was never
  > evidence of nondeterminism. **The byte-identical Turns/Commands/Winner/Error result
  > stands and is what that bullet's claim actually rests on.**
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

## Worker Handoff (UI-4, `scutemob-185`) — picker Confirm hotfix

**G1 CONFIRMED IN A BROWSER BEFORE ANY EDIT, and the triage's diagnosis was exactly right.**
Headless Chromium (playwright-core, `~/.cache/ms-playwright/chromium-1228`) against a live
`play-server` on **:3041**, seed 116 driven over HTTP to a Three Visits `SearchLibrary`
decision (the Farhaven Elf class), then three clicks in the browser:

```
[pageerror] DataCloneError: Failed to execute 'structuredClone' on 'Window':
            #<Object> could not be cloned.
    at z (…/assets/index-BIZqizQa.js:3:13218)      <- SearchPicker.emit
    at HTMLButtonElement.O (…:3:13344)             <- confirm()
    at HTMLDivElement.za (…:1:30189)               <- Svelte's delegated handler
```

Picker stayed open, **error-strip count 0, zero POST requests**, `command_count` still 171 and
`seq` still 291 after the click. "Did nothing" was literal.

**Fix**: new `frontend/src/lib/plainClone.svelte.js` (a `$state.snapshot` wrapper) called at all
three sites. `$state.snapshot` over `JSON.parse(JSON.stringify(…))` deliberately: it does not
re-serialize, so it cannot coerce anything the wire put there, and it deep-copies plain objects
too — which matters for the harness proposed below, whose fixtures pass non-reactive values.

**Error surfacing, two mechanisms, both demonstrated live by fault injection** (faults reverted;
`dist` rebuilt clean, bundle hash back to `index-CsOwI2Ah.js`):

| Fault | Path | Rendered |
|---|---|---|
| throw inside `SearchPicker.emit` (guarded) | picker `try` → `onError` → `ActionBar.onPickerError` → `PlayApp` → `stores.reportClientError` → strip | *"Something went wrong in this browser — the game is unchanged / could not submit the search answer: injected UI-4 fault"*; chain closed, 0 POSTs |
| throw inside `SearchPicker.select` (**no** `try`) | `window` `error` listener armed by `main.js` | *"unhandled RangeError in the client: …"* |

The second is the load-bearing one: five pickers have no `try` and never will have one for every
handler, so the `window` listener is the guarantee and the per-picker `catch` is only a better
message. **Svelte 5's `<svelte:boundary>` is not a substitute** — it catches render and effect
errors, not DOM handler ones. Checked, not assumed.

**Gates (2, both proven red by executing a revert, then green again)**, in
`tools/play-server/src/main.rs` so they run under `cargo test --all` and therefore CI:

* `test_frontend_never_structured_clones_reactive_state` — walks every `.svelte`/`.js`/`.css`
  under `frontend/src/` (skipping `node_modules/`, `dist/`), bans `structuredClone(`,
  `.postMessage(` and `indexedDB`. **Proven by reverting `SearchPicker`'s clone.** Four
  non-vacuity arms, because a ban with zero permitted uses is the pinned-empty-roster shape:
  named files **and** a ≥14-file floor on the walk; each picker must *import and call*
  `plainClone` (a picker that stopped copying at all would satisfy the ban while mutating its
  parent's state); the helper must really be `$state.snapshot`; and the matcher is fired at a
  synthetic offending line so a typo in a needle cannot hide.
* `test_frontend_picker_failures_reach_the_error_strip` — pins both mechanisms **and the call
  that arms the second**. **Proven by deleting `main.js`'s `installGlobalErrorReporting()`
  call**, which is the failure mode a "does the module export it?" test would have missed.

**All five CR flows verified end to end, each with a NON-DEFAULT answer** so game state
distinguishes the human's choice from the engine's default — the property the whole defect class
turns on:

| Flow | Setup | Posted | Observed in game state |
|---|---|---|---|
| library search CR 701.23 | seed 116, Three Visits | `{SearchLibrary:{found:24}}` | **Dryad Arbor** on the battlefield, not the default `candidates.first()` Forest |
| scry CR 701.22a | seed 28, Preordain (scry 2) | `{Scry:{bottom:[99],top:[98]}}` | drew **Reverse Engineer**, the *second* card; the default order draws the Island |
| surveil CR 701.25a | seed 28, Consider | `{Surveil:{graveyard:[99],top:[]}}` | **Island in the graveyard** |
| sacrifice CR 118.8 | seed 29, Harrow | `{Sacrifice:{ids:[437],lki:[]}}` | battlefield `402,419,437` → `402,419`; **437 chosen over the server default 402** |
| Squad CR 702.157a | seed 1364, Galadhrim Brigade | `{Squad:{count:1}}` | **token copy #486** beside the real #482; template default is `count: 0` = decline |

Zero `pageerror`s and zero error strips across all five.

**Scope**: 9 source files under `tools/play-server` plus 4 doc files; **0 engine lines**
(`git diff main..HEAD --numstat -- crates/` is EMPTY — no engine *and* no simulator), 0 wire
changes — no `Command`/`GameEvent`/`Effect` variant, so PROTOCOL and HASH are untouched by
construction and were not recomputed. (Reconciliation note for the collector: the implementation
commit is 9 files; the branch total is 12+, the difference being `CLAUDE.md`, the play-server
README and this file.) Workspace:
**4,265 passing / 0 failing / 5 ignored** (`--workspace --no-fail-fast`, captured to a file, not
piped to `tail` — 2026-08-02 lesson), which is +2 on main's 4,263 and those 2 are these gates.
`cargo fmt --check`, `tools/check-defs-fmt.sh` (1,803 defs) and
`clippy --workspace --all-targets -D warnings` all clean.

### The R7 frontend test harness — a concrete proposal, deliberately NOT built here

R7 (`memory/m11-session-plan.md` §8) is the debt this defect collected on, and the triage is right
that it is overdue. It is **not** built here because this task had to be small and go first. What
follows is sized from having actually done both halves by hand today.

**Tier 1 — component tests (vitest + jsdom + `@testing-library/svelte`).** 3 devDeps, an
`npm test` script, a `vitest.config.js` reusing the existing `svelte.config.js`, one spec per
picker (8) ≈ 400-600 lines. **The one rule that makes or breaks it: a fixture MUST wrap the
template in `$state()` before passing it as a prop.** A spec that hands a picker a plain object
would have passed green against the broken code — that is precisely why UI-1's and UI-2's HTTP
probes proved the channel and nothing about the component. Reproduce the *reactivity*, not just
the shape. Write that rule into the harness's own module doc, because it is the only part a
future author can get wrong while believing they have covered the bug.

**Tier 2 — real-browser scenarios (`playwright-core`, ~30 lines of setup).** Exactly the shape
used for today's verification, and worth keeping: drive the game to the target decision **over
HTTP**, then do only the last few clicks in the browser. Cheap, no component framework, and it
catches what jsdom structurally cannot — the `DataCloneError` is real-browser structured-clone
behaviour and a jsdom polyfill may not reproduce it. Tier 1 without Tier 2 could have missed this
exact bug.

**The real cost is fixtures, and it is bigger than the harness.** Reaching a scry / surveil /
Squad decision meant scanning **~2,400 seeds** through `POST /api/game` to find an opening hand
holding the right card, because `session.rs:165` hard-codes `DeckSource::RandomPerSeat`. Two
routes: (a) cheap and immediate — commit the tuples below under `test-data/`; (b) the real fix —
let `POST /api/game` accept a fixed decklist so a scenario names its cards instead of hunting a
seed. Recommend (a) now, (b) when someone touches `session.rs` anyway. **Known-good tuples,
handed over so nobody re-scans**: seed **116** → Three Visits (`PickOne`, 33 candidates incl.
Dryad Arbor as a distinguishable non-default); seed **28** → Preordain *and* Consider, two
Islands (`Partition`, both scry 2 and surveil 1 in one game); seed **29** → Harrow, 3 Forests
(sacrifice-a-land `SacrificeCostView`, no creature needed); seed **1364** → Galadhrim Brigade,
5 Forests (Squad `max_count: 1`). Squad is the scarce one — **1 seed in the first 600**, and
`Ultramarines Honour Guard` at 6 mana is not reachable before the human dies, so use Galadhrim
Brigade. Driver scripts are in this session's scratchpad only, not committed; they are ~150 lines
and trivially rewritten from this paragraph.

**CI note, flagged not fixed**: the workflow is a single Ubuntu **cargo** job. Tier 1 needs an
`npm ci && npm test` step and a Node toolchain in that job. Today's two gates need neither —
that is why they are Rust source gates and not a JS lint.

**What today's gates do NOT cover, stated plainly**: they prove the pattern is absent and the
error wiring exists. They cannot prove a picker *renders*, that a template is read correctly, or
that an answer is *right*. `SearchPicker`'s and `PartitionPicker`'s "# Untested" module sections
are still accurate and were left alone.

**`/review` fix cycle — 5 LOW, all 5 taken rather than deferred** (each was a few lines, and two
were real coverage holes):

1. **The gate had a blind spot exactly the size of the shared component library.** It walked
   `frontend/src/` only, but `vite.config.js` aliases `$viewer` →
   `tools/replay-viewer/frontend/src/lib`, imported **in place** and compiled into *this* bundle.
   The walk now covers both, with its own named-file + ≥8-file floor because a `..`-relative path
   is the arrangement most likely to resolve to nothing after a move. **Proven by appending a
   forbidden call to `cardTooltip.js`** — red, naming that file; restored, green. Zero real hits
   today, so this is coverage, not a repair — but the test's own "this is a class, not three
   sites" claim was overreaching by one directory until now.
2. **The silent bail-outs survived.** All three pickers kept malformed-template guards that
   `return` without reporting. Those are *returns*, not throws, so 6048's literal wording was
   already satisfied — but the **symptom** (click Confirm, nothing happens, no message) is the
   thing this task exists to eliminate, and it should not survive from a second cause. All **six**
   sites now report through `onError` before bailing — `SearchPicker` ×2, `PartitionPicker` ×2,
   `CostPicker` ×2 (the two `!entry` checks, which absorb `fillTemplate`'s own two internal
   `return null` paths). Three `onError?.(` calls per picker: two guards plus the `catch`.
3. **`main.js`'s comment overclaimed.** It said arming the net before mount surfaces "a throw
   during the very first render"; the strip lives inside `ActionBar`, so such a throw sets the
   store and renders nothing. Comment narrowed to what is true.
4. **The weakest gate arm matched prose.** The per-picker check was `contains("onError")`, which
   the prop's own doc comment satisfies — a picker that documented the prop and never called it
   would have passed. Anchored on `onError?.(` instead. **Proven by renaming CostPicker's calls**
   — red; restored, green.
5. Count mismatch between CLAUDE.md and this handoff reconciled (above).

All three picker types re-verified in the browser **after** these edits (search → Dryad Arbor;
surveil → Island to graveyard; sacrifice → 437 over the default 402), so the fix cycle did not
regress the thing the fix cycle was protecting.

**Seed**: `OOS-G1-1` (the structured-clone-on-Svelte-state class) is **CLOSED by this task** —
fixed at all three sites and machine-gated against recurrence. Not filed as open in
`docs/audits/decision-point-audit.md` §8.1 for that reason; the gate is the durable artefact.
`OOS-G8-1`-adjacent note for whoever takes **UI-5**: G8's "Concede was the only live control"
premise is now false — the answer button works — so it reverts to an ordinary UX item, exactly as
the triage predicted.
## Worker Handoff (PB-DX19, `scutemob-184`)

**Scope**: the v3 queue's first dispatch — `OOS-SIM2-6` (HIGH) + `OOS-SIM2-5` fold-in.
Plan: `memory/primitives/pb-plan-DX19.md`. Review: `memory/primitives/pb-review-DX19.md`.
Brief: `memory/primitives/seed-rerank-2026-08-02.md` §4, "Dispatch briefs" → PB-DX19.

**Shipped**: `ee7a55b4` (stage-0 repro), `a0d977e5` (fixes), `79b94a58` (tests + deviation pin).
PROTOCOL **33** / HASH **70** gate-executed, both unmoved. Tests **4,274 / 0 / 5** on branch
(+11 over main's 4,263). `clippy -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`
all clean. Coverage unmoved, proven by regeneration (see "claims" below).

### What actually broke, and why it took 4.5 months

The cycle is `calculate_characteristics` → `is_effect_active` → `check_static_condition` →
the `YouControlNOrMoreWithFilter` arm → `expect_characteristics` → back. **The seed's own
description of it was wrong in a way that matters.** It reads as a property of *counting
artifacts* with *that permanent on the battlefield*. It is neither: `calculate_characteristics`
calls `is_effect_active` on **every** `state.continuous_effects` entry, whatever object it was
asked about and whatever zone that object is in. So the recursion runs through the **effect**,
not the object. Two probes prove it — one calculating the Archangel's *own* characteristics (it
is not an artifact; the grant can never reach it) and one with Metalcraft **off** (the condition
is *false*) — and both crashed identically pre-fix.

That distinction is the whole story of the 4.5 months. The in-source comment argued termination
from exactly the disproved invariant ("we are checking the types of *other* battlefield
objects"), and then proposed the correct fix as a **performance** note. Anyone who read it came
away reassured. **The comment, not the code, was the defect that survived** — so the batch
rewrote it with the mechanism, and that rewrite is worth more than the one-line code change.

### Durable lessons

1. **A termination argument in a comment is a claim, and claims rot.** This one was never true.
   If a comment says "this recursion is safe because X", the test that proves X is missing.
2. **A test that names a card by string proves nothing about that card.** `static_grants.rs`
   named Indomitable Archangel and then hand-built the effect with `condition: None` — it
   exercised the filter and never the condition, and the condition was the entire defect. Repaired
   to drive the real def through `register_static_continuous_effects`; it now aborts pre-fix where
   the old shape passed. **That is the test to write: one that fails for the reason you claim.**
3. **`as` casts are not checked arithmetic.** `overflow-checks` does not touch them. The one
   OOS-SIM2-5 site no fuzz profile could ever have caught was a `u32 as i32` counter widening that
   wrapped the counter's **sign** in every profile. Its probe fails by *assertion*; the other five
   fail by *panic*. If a hardening pass converts `+=` to `saturating_add` and leaves the `as`
   casts, it has hardened the sites that were already loud and skipped the silent one.
4. **A stack overflow is not a test failure.** It is signal 6, names no test, and takes the binary
   down: `cargo test` printed `running 3 tests` and named exactly one. Filed `OOS-DX19-4` — a
   depth tripwire would have made this a named debug failure in 2026-03.
5. **`cargo fmt` passed a card-def edit that `tools/check-defs-fmt.sh` rejected.** SR-35 caught a
   real one here, not a hypothetical.

### Claims, and how each was actually established

- **The mandatory experiment is decisive, and was run pre-fix at the real pre-fix tree** (not with
  the card's static commented out — the brief's control does not survive the fix landing).
  `mtg-fuzzer --games 15 --seed 1`, `[profile.fuzz]`: **pre-fix** `fatal runtime error: stack
  overflow` → SIGABRT, exit 134, **0 of 15** games completed; **post-fix** 15 completed, 4.6s,
  avg **189** turns, 12 wins / 3 errors. **This closes `OOS-DP3-9` / `OOS-M11-3`'s stack-overflow
  half** — and note the abort was *immediate*, so that row's "game-count- or game-length-dependent"
  reading was an artefact of which decks the seed drew. `OOS-M11-3`'s **determinism** half is
  untouched and stays open.
- **Every pre-fix failure was OBSERVED via an executed revert, and both reverts compiled** (S8's
  lesson: the first revert of that batch did not compile, so it proved nothing). Two independent
  reverts were run: P/T-only (6 probes fail, 3 recursion probes pass) and recursion-only (6 P/T
  probes pass, recursion probe SIGABRTs). The isolation is the point — it shows neither fix is
  carrying the other's evidence.
- **Coverage unmoved was proven by REGENERATION, not by an empty diff.** The card-defs diff is
  *not* empty: the brief mandated the `greymond_avacyns_stalwart` note edit. So the claim rests on
  `tools/authoring-report.py` producing a byte-identical report body (only the self-dating header
  and recent-commits list differ). The generated docs were then reverted, since the numbers moved
  by nothing and committing them is pure churn.

### The mistake this batch made, and what caught it

**The first fix was a HIGH regression, and the tests could not see it.**
`check_static_condition` is a *shared* evaluator with five callers; only `is_effect_active` (inside
`calculate_characteristics`) closes the cycle. Reading `obj.characteristics` unconditionally fixed
that one and broke the other four, all of which CR 613.1d requires to be layer-resolved:
`garruks_uprising`'s `min_power: 4` intervening-if stops firing on a 2/2 with two `+1/+1` counters;
`bloodline_keeper` rejects a changeling (CR 702.73a expands types *inside* the layer loop);
`mox_opal` **over**-counts a face-down manifest (CR 708.2a — printed types are still the hidden
card's, so this one is a false *positive*, the direction nobody thought to look for).

**All 4,274 tests passed through it.** Not because coverage is thin, but because no existing test
put a counter-pumped or type-changed permanent through a condition filter — the fixture creatures
are plain vanilla bears. A green suite is evidence about the scenarios someone thought to write.

The lesson is not "be careful". It is: **when you change a function, enumerate its callers before
you decide what the change means.** The recursion was a property of ONE call path, and the fix was
applied to the function. `rules::layers::characteristics_for_condition` is the repair — a
re-entrancy guard that decides per caller — and because it decides by shape rather than per site, it
also closed `OOS-DX19-1`'s ten siblings, which the leaf-edit fix would have got wrong in the
opposite direction (several are *correct* as layer-resolved on their real paths).

### What the next worker should know

- **The fix has a known, live cost, and it is asserted in the wrong direction on purpose.**
  `blinkmoth_nexus` / `inkmoth_nexus` are `Complete`-by-derive **colourless** lands that animate
  into **artifact** creatures (Layer-4 `AddCardTypes`), so they share a deck pool with the
  Archangel and an animated Nexus no longer feeds Metalcraft — CR 613.1d says it must.
  `deviation_animated_nexus_does_not_count_toward_metalcraft` pins that, and its message tells you
  to **invert** it rather than delete it when `OOS-DX19-2`'s CR 613.8b fixpoint lands.
- **`OOS-DX19-1` is CLOSED, and the second review is why.** The first routing pass claimed the
  closure while three sites were still unconverted — they spell the call
  `expect_characteristics(state, id)` instead of `(state, obj.id)`, so a pattern-replacement walked
  straight past them, and the reviewer reproduced the original SIGABRT through one. **The durable
  lesson: a closure achieved by editing every site you could find is a claim; a closure backed by a
  gate that fails when a site reappears is a fact.** The gate is
  `no_condition_evaluator_resolves_characteristics_directly`, watched failing on a re-introduced
  miss. Ten more `expect_characteristics` sites in
  `check_condition` are the identical shape and are latent **only because of corpus shape** — all
  **57** corpus occurrences of those ten variants were enumerated and classified by field
  position: every one is an `activation_condition`, `unless_condition`, `intervening_if`, or a bare
  `Effect::Conditional`, and **none** is a `ContinuousEffectDef.condition` — which is the only
  field `is_effect_active` reads. (Do not restate this as "all ~98 `condition: Some(..)`
  occurrences are off the layer path"; that is false — 17 of them *are* continuous-effect
  conditions, `indomitable_archangel`'s among them. The claim is about the ten variants
  only.) The next author who writes "as long as you control a legendary creature, …" as a **static**
  reopens a HIGH with no warning. **Do not fix it by converting the ten leaves**: several are
  *correct* as layer-resolved on their real call paths (there is a `// CR 613.1d … Blood Moon`
  comment saying so). It wants a boundary guard.
- **PB-DX22 must still follow this batch**, per the brief's own sequencing note — shuffling the
  fuzzer makes spells castable at ordinary depths. That constraint is now satisfied.


## Worker Handoff (SEED RE-RANK v3, `scutemob-182`) — doc-only

**Deliverable**: `memory/primitives/seed-rerank-2026-08-02.md`. **This is now the authoritative
primitive queue.** `seed-rerank-2026-07-27.md` §4 is banner'd SUPERSEDED; its §1-§3 remain
canonical. `git diff` confined to `memory/`, `docs/`, `CLAUDE.md` — zero source lines.

**The census is twice the brief's estimate, and the reason is a cutoff.** The brief scoped ~40
seeds (DX6 + the 174-181 run). The real post-2026-07-27 population is **80 rows / 79 distinct
IDs**. v2's census closed **2026-07-31**; every PB-DX batch shipped **2026-08-01** — so the
document that ranks PB-DX1..DX18 has never seen the 29 seeds PB-DX1..DX5 filed, nor
`OOS-M11-5..10`. Four of those 29 are live-wrong on deck-legal `Complete` cards.

**No single source is complete, and a future re-rank must run all three.** Pass A
(workstream-state handoffs) misses **20** rows — the PB-DX1..DX4 handoff sections are rotated out
and only the L18 W6 mega-row survives, naming DX1's seeds not at all. Pass B (the 2026-08 archive)
misses `OOS-M11-5` entirely and records almost every filing as an unresolvable range. Pass C
(`docs/audits/decision-point-audit.md` §8.1, the registry) misses **10** — the CARDS-2 family lives
in `memory/card-authoring/cards2-field-fidelity-2026-08-02.md` under
`## 5. Cross-references and seeds` (**§5**, not §7 — that doc has five sections), and only
`OOS-CARDS2-9` is in §8.1. Wildcards resolved, and two written ranges found stale
(`OOS-DX5-1..7` under-reports by one — `OOS-DX5-8` exists and neither narrative mentions it). **UI-1 (`scutemob-174`) filed zero seeds**;
there is no `OOS-UI1-*` family anywhere.

**Next dispatch is PB-DX19, not PB-DX7.** `OOS-SIM2-6` — the registry's only self-declared HIGH —
was walked hop by hop and confirmed: `layers.rs:35`→`:46` `is_effect_active` → `layers.rs:565`
`check_static_condition` → `effects/mod.rs:10259` `expect_characteristics` → `layers.rs:478`
`calculate_characteristics`. **Unconditional**, because the arm evaluates every candidate
permanent *before* the `exclude_self` test at `:10266` and the source is itself a candidate.
`indomitable_archangel.rs` declares **no `completeness` field** → `Complete`, `validate_deck`
accepts it, `random_deck` pools it for any W-identity seat. Result: `stack overflow` → SIGABRT,
not `catch_unwind`-able, so the play-server cannot contain it. **The class is exactly one card**,
measured two ways (17 of 380 `ContinuousEffectDef`s carry a `condition`; exactly one uses a
recursion-capable variant). **The fix is one line** — and `layers.rs:2291`'s
`EffectAmount::PermanentCount` already made that exact choice for that exact reason
(`:2304-2310`). Live **4.5 months** (`d83ac94d` 2026-03-12 / `aa23d26c` 2026-03-23).

**Two lessons worth carrying, both about why it survived.** (1) `effects/mod.rs:10245-10256`
argues termination from the wrong invariant — "we are checking *other* objects" — when the
recursion is on the *effect*, re-collected at `layers.rs:46` on every nested call; it even proposes
the correct fix as a **performance** note. *A safety argument written next to the code it excuses
is not evidence.* (2) `crates/engine/tests/rules/static_grants.rs:711-760` names Indomitable
Archangel and hand-builds the effect with `condition: None` at `:736`. *A test that names the card
while dodging the field is worse than no test — it reads as coverage.* And a landmine:
`greymond_avacyns_stalwart.rs:38-43` instructs a future author to build a second instance.

**Four seeds filed "latent" are live-wrong.** `OOS-DX1-3` (`nether_traitor`),
`OOS-DX2-5`/`-2`/`-7` (`golgari_grave_troll`), `OOS-DX4-2` (`retreat_to_kazandu`), `OOS-DX4-6`
(**all ten Karoo bounce lands** + two more — scope ×7, and the deviation is exploitable *in the
controller's favour*). **The tempting explanation is wrong and the true one is worse.** The
`#[default] Completeness::Complete` derive — PB-DX1's and PB-DX3b's lesson — accounts for **five**
of the eight live-wrong defs this census caught (`golgari_grave_troll`, `retreat_to_kazandu`, the
ten Karoos, `sigil_of_sleep`, `indomitable_archangel`). The other three (`nether_traitor.rs:60`,
`qarsi_sadist`, `voldaren_epicure`) declare `completeness: Completeness::Complete` **explicitly** —
a one-line grep would have found them. So the shared mechanism is not the derive; it is that **the
latency claim was never checked against the corpus at all**, in three cases not even by the
cheapest possible check. Saying "the derive did it" would let a future triage think that grepping
the explicit marker is sufficient diligence, and for three of these eight it would have been.
**965 of 1,803 defs never declare a marker** (re-measured; PB-DX4 said 966 the day before). Filed as
`OOS-RR3-1`. **Binding for every future batch: a latency claim is not verified until the corpus
has been enumerated — over `all_cards()` where possible (SR-36), missing marker treated as
`Complete` — and "no def does X" is not a finding until someone has actually looked.**

**Other findings that moved a rank.** `OOS-CARDS2-4` — the offer layer cannot see a
`KeywordAbility::Enchant`-carried target requirement (`legal_actions.rs` has zero occurrences of
`Enchant(`/`target_min`; `target_count_range` iterates `TargetRequirement` only), so **13
deck-legal `Complete` Aura defs** 422 on first human contact and the only reason the suite is green
is `KNOWN_FALSE_OFFERS`. `OOS-M11-9` — no once-per-combat guard, and the consequences are three,
not one: attackers **accumulate** (`combat.rs:743` inserts into a map), every attack trigger
**re-fires** (`:795-805`), and the raid count is **clobbered** (`:759`); the blocker side already
has the guard (`:1103`). `OOS-UI2-1` and `OOS-SIM3-1` **reconcile arithmetically** — no opening
hand + 34 basics on top ⇒ first non-land at personal draw ~35-40 ⇒ game turn ≈136-156; "never
casts" is `--max-turns 80`, "casts from turn 143" is the default cap. Quote the cap alongside any
fuzz-parity claim.

**Closures: 11 verified in code, and one closed *further* than recorded.** `OOS-UI2-3`'s third
cause was `OOS-M11-2`'s `can_afford` pool-OR-sources split, which SIM-2 also closed
(`legal_actions.rs:1752-1757` is now one `solve_mana_payment_with_pool` call) — so `OOS-M11-2`'s
residue is **cost MODIFIERS + CR 106.12 restricted mana only**, smaller than CLAUDE.md said.
**Three rows are design records, not work** (`OOS-DX5-2`, `OOS-DX5-6`, `OOS-DX6-3`) — ranking them
wastes a slot. **Six merges** recorded, incl. `OOS-SIM1-2` ≡ `OOS-SIM2-7` (literally the same two
TUI lines) and `OOS-CARDS2-11` ⊂ `OOS-CARDS2-8`.

**Two hard sequencing constraints, derived not asserted.** (1) **PB-DX19 must precede PB-DX22** —
shuffling the fuzzer makes spells castable at ordinary depths, turning the Archangel's turn-191
abort into a routine one that will read as a regression caused by PB-DX22. (2) PB-DX22 and every
card-def batch re-roll every recorded seed (`OOS-CARDS2-3`: `random_deck` indexes a corpus-ordered
vector, so correcting a *type line* re-deals every seeded game, and **no gate exists**) — batch the
card-def work and land the pool-size gate first so the re-deal announces itself.

**Not done, deliberately**: the `mtg-fuzzer --games 15 --seed 1` A/B with and without the
Archangel's static (would settle whether `OOS-SIM2-6` is `OOS-DP3-9`/`OOS-M11-3`'s stack-overflow
mechanism — "very likely" is the honest strength until it runs) and the first-`ZoneId::Stack`
re-measure at HEAD (SIM-1's command-zone loop should put a spell on the stack ~120 turns before
SIM-3's turn-143 measurement; either SIM-3 measured a pre-SIM-1 build or something suppresses the
bot offer). Both are assigned into PB-DX19's and PB-DX22's plans respectively. This task was
doc-only and ran no cargo.

## Worker Handoff (SIM-3, `scutemob-177`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (SIM-3) — **F6 CLOSED**; playtest triage is
now **fully closed** (`memory/playtest-triage-2026-08-02.md`: OPEN = none)
**Task**: `scutemob-177`. Branch
`feat/sim-3-stackconsistency-invariant-is-a-false-positive-by-cons`

**The task was re-scoped before it started, and the re-scope was half right.** The
coordinator's own comment said M11-local S8 had already rewritten the check with the same
diagnosis, so the task shrank to "tests + one doc line". Two of those three residuals were
as described (no test module; `docs/mtg-engine-simulator.md` still wrong in prose). The
third was not: `docs/mtg-engine-runtime-integrity.md` was **also** still wrong — S8 had
corrected neither. And the rewrite itself carried a residual false positive.

**Completed**:

1. **The finding: the S8 rewrite classified on `StackObjectKind::Spell` alone, and its
   stated premise for doing so is false.** Its doc block asserted that the four engine
   sites that move an object into `ZoneId::Stack` "all end in that same `Spell` kind".
   `casting.rs::handle_cast_spell` moves the card at `:4399` and *then* branches on
   `cast_with_mutate` at `:4504`, so a **mutate** cast (CR 702.140a / CR 729.2) puts a card
   in the Stack zone under a `MutatingCreatureSpell` kind — and the check reported it as an
   orphan, on every such cast, in a game with nothing wrong with it. **Generalisable, and
   it is the same shape as `OOS-SIM1-3` and `OOS-SIM2`'s fix-cycle finding one and two
   batches earlier: an enumeration is only as complete as the category it names.** Here the
   category was "kinds that obviously put a card on the stack", read off variant names.
   Classification is now `invariants::stack_card_of`, an **exhaustive match over all 27
   variants** — adding a `StackObjectKind` is a compile error until someone classifies it,
   the forcing function SR-5 already applies to `KeywordAbility`.

2. **Two properties the old set comparison could not express, both added and both
   measured.** Property (3) closes **MR-M11-14** (LOW, deferred): no two non-copy stack
   objects may claim the same card, CR 400.7. Its deferral had asked for a measured run
   before widening the check — this batch was already measuring, so it could pay that
   price, and `memory/m11-fix-session-plan.md` is updated to `[x] DONE`. Property (4) is
   order: the Stack zone's contents are the card-owning stack objects' cards read in stack
   order, ability and trigger entries skipped. Structurally guaranteed (`Zone::Ordered`
   inserts at the back only; every entry site pairs the zone move with a
   `stack_objects.push_back`; every removal takes the pair; CR 608.2d suspension restores
   `restart_point` wholesale) — the `/review` verified that argument independently rather
   than accepting the 10 clean runs as proof.

3. **Measured A/B**, old check restored verbatim from `222ff84f^`, same builds, same seeds:
   `local_game_playthrough` seed 1 **720 → 0** (638 + 82 by direction; the test fails on the
   first seed with the old check, passes on all five with the new); `mtg-fuzzer --games 5
   --seed 1 --max-turns 200` **8,781 → 0** (7,575 + 1,206). Every other check byte-identical
   across the A/B (929 `no_orphaned_tokens`, 9 `player_consistency`), so the measurement
   moves this check and nothing else. **8,781 of that run's 9,719 violations — 90.3% — were
   this one check being wrong.**

4. **Ten probes in a new `#[cfg(test)] mod tests`** (the file had none across 306 lines),
   every one watched failing under a deliberate revert: a **9-revert matrix** in which
   R1–R7 each fail exactly one test and R8/R9 cover the over-firing direction. T2 pins the
   pre-S8 check's two-per-spell false positive as a historical record in code.

**Durable lessons**

- **A redaction of a false positive is not the same as a proof of the truth.** S8 removed
  501 false positives and the number went to zero, which read as done; the *reason* it
  went to zero was that no seed in its evidence cast a mutate spell. Zero is not a proof
  when the population is thin.
- **`OOS-UI2-1` is right about the mechanism and wrong about the horizon** —
  **`OOS-SIM3-1`**, and the most reusable thing here. UI-2 measured 5 games × 80 turns, saw
  no non-land in hand, and concluded "every fuzz parity claim in this project's history is
  a claim about a land-only game". At the fuzzer's **default** `--max-turns 200`, **150
  distinct cards reached `ZoneId::Stack` across 5 games, the earliest on turn 143** — the
  basics run out and real spells start resolving. The deck-order defect is real and
  unchanged; its consequence is a **threshold**, not an absolute. Any future fuzz A/B
  should say which side of turn ~140 it lives on.
- **Every "N violations" figure this project has quoted is checkpoint-weighted**
  (**`OOS-SIM3-3`**): `check_all` re-reports a condition that is still true at every
  command. Measured: 929 `no_orphaned_tokens` reports = **183 distinct tokens**; 9
  `player_consistency` reports = **1 condition**. The inflation factor is not constant, so
  those totals are not comparable to each other.

**Seeds filed** (`docs/audits/decision-point-audit.md` §8.1): **OOS-SIM3-1** (the horizon
qualification above), **OOS-SIM3-2** (two of the twelve documented invariant checks have
never been written — legal-action soundness and SBA idempotency — a third is a no-op, and
`runtime-integrity.md`'s parallel list has four that do not exist), **OOS-SIM3-3**
(checkpoint-weighted totals), **OOS-SIM3-4** (`no_orphaned_tokens` is now the next noise
floor at 929 of the 938 remaining, and OOS-M11-7 says they are *expected* — so the fuzzer
still is not a clean smoke test, for a new reason), **OOS-SIM3-5** (`/review`:
`Effect::CounterSpell` drops `MutatingCreatureSpell` into its `_ =>` arm after already
removing the stack object, stranding the card in `ZoneId::Stack` forever; and countering a
**copy** moves the *original's* card. Both are engine defects that will legitimately trip
this check, neither is in this batch's evidence — **read the next one as a real finding,
not a SIM-3 regression**).

**Also updated**: `OOS-DP3-9`'s `stack_consistency` half **WITHDRAWN** with the A/B
attached (its stack-overflow half stands and now has `OOS-SIM2-6` as a named mechanism; its
`crash-reports/.gitignore` rider re-checked and closed — `.gitignore:52`). The
`memory/workstream-state.md` bot-parity bullet that cited `stack_consistency` line ordering
as evidence of `OOS-M11-3` nondeterminism is annotated as **voided** — it was ordering of
noise; the byte-identical Turns/Commands/Winner/Error result is what that claim rests on.

**Gates**: tests **4,247 → 4,257 / 0 / 5** (+10, exactly the probes). `cargo fmt --check` +
`tools/check-defs-fmt.sh` (1,803 defs) clean; `clippy --workspace --all-targets -D
warnings` clean; `cargo build --workspace` clean (the SR-3 seal gate — the probes use the
`test-util` escape hatches and are `#[cfg(test)]`, so the seal holds). **PROTOCOL 33 /
HASH 70 gate-EXECUTED unmoved** (the criterion's "32" is stale, as UI-1/UI-2/SIM-2 also
found); `decision_gate` 18/18. **Zero engine lines** — the only source file in the diff is
`crates/simulator/src/invariants.rs`.

**Not done, deliberately**: `OOS-SIM3-5`'s two engine defects are left unfixed
(`crates/simulator`-only batch). `OOS-SIM3-2`'s two missing checks are marked in both docs,
not written — #10 (legal-action soundness) is the SR-38 property and is close to free, since
`GameDriver` already distinguishes a rejected command from an applied one.
## Worker Handoff (UI-3, `scutemob-180`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (UI-3) — the UX/layout items the triage
filed under **"Not verified (by design)"**, i.e. feature work rather than claims
**Task**: `scutemob-180`. Branch
`feat/ui-3-play-frontend-ux-polish-batch-playtest-notes-layoutinfo`

**Completed** (all five criteria; 0 engine lines — `git diff main -- crates/engine/src
crates/card-types/src crates/card-defs` is empty):

- **AC 6006 — combat display.** The headline is that **nothing was missing from the
  payload**. `StateViewModel::combat` has carried `attackers[].target` and
  `attackers[].blockers[]` since M9.5, seat-redacted by `redact::redact_combat`. The play
  client rendered `$viewer/StateView.svelte`, which **does not include**
  `CombatView.svelte` — the replay viewer composes those two in its own `App.svelte`, so
  the component existed, the data existed, and the two had never been introduced on this
  surface. `PlayApp` now renders it under the stack. **A real defect fell out of doing so**:
  `AttackerView::target`'s doc comment said `"planeswalker:<id>"` and
  `CombatView.svelte::formatTarget` believed it, rendering `PW #{suffix}` — so an attacked
  planeswalker displayed as **`PW #Chandra, Torch of Defiance`** in *both* surfaces.
  `build_combat_view` has always written a name, and `redact_combat` substitutes
  `FACE_DOWN_NAME` — a name — which is only coherent for a name field. Fixed in place in the
  shared component (deliberately: the replay viewer had the identical bug) and the doc
  corrected with it.
- **AC 6007 — event feed.** The feed was sparse **because the renderer was**:
  `event_view_for` had ~11 rendered arms and a `_ =>` catch-all emitting the bare serde
  variant name with no player and no card, and *every single item the playtest asked for*
  (taps, ETB, deaths, exiles, counters, triggers, resolutions, attacks, blocks, damage) fell
  into it. **49 prose arms** added, each identity routed through the `viewer_may_identify`
  gate — no arm reads `state.objects()`. `EventView` gains `tier`
  (`game`/`player`/`card`/`stack`), assigned **server-side** by a match on the variant with a
  documented `_ => Game` default. The client deliberately does **not** classify by `kind`
  substring: `GameEvent` has ~141 variants, and a stale client-side list would silently hide
  a whole class of event behind a filter chip. (It still substring-matches for *tone*, which
  only picks a colour — the distinction is written down.) `EventFeed` gains tier chips with
  live counts and collapsible per-turn sections; **section boundaries come from the
  unfiltered list**, because `TurnStarted` is itself a `game`-tier event and deriving them
  from the filtered one would make every turn heading vanish the moment someone unticked
  "turn".
- **AC 6008 — layout.** New play-local `PlayBoard.svelte` (2×2 battlefield grid via
  `repeat(auto-fit, minmax(22rem, 1fr))`, so four boards lay out 2×2 and two survivors reflow
  to full width **with no code branch on the count**) and `SeatCard.svelte` (command zone
  folded into the player card, expandable details drawer). The shared
  `$viewer/StateView.svelte` is **untouched** and the replay viewer still uses it — every
  requirement here is the opposite of what a step-debugger wants, and a dead player's board
  must keep rendering there because stepping *backwards* past an elimination is the normal
  thing to do. All four "stay in place" requests fall out of **one** arrangement: seat row,
  action bar and stack dock are flex siblings *above* the scrolling body, the own-hand bar is
  a sibling *below* it, and nothing is `position: sticky` (four stacked sticky strips slide
  under each other). **Commander hover-preview**: measured rather than assumed — the command
  zone was the **only** zone in the codebase without `cardTooltip` (hand, battlefield,
  graveyard, exile and stack all had it).
- **AC 6009 — pass-until.** `stores.js::startPassUntil`, entirely client-side: each iteration
  is one ordinary `POST /api/game/action` naming the `PassPriority` option the server already
  offered, so **no server change, no new route, and no recorded seed moves**. It stops on
  cancel, game over, no decision, **a non-`Priority` decision**, no pass offered, a failed
  request, or 400 passes (below the server's own 500-consecutive-pass guard, so it stops
  first and stops visibly) — and it always **says which**. The non-`Priority` stop is the
  load-bearing one: answering a cleanup discard or a trigger's targets with a default is
  precisely the defect UI-1 existed to delete. Predicates are keyed on a **mode object**, so
  the note's fine-grained form is one more entry (`OOS-UI3-3`).
- **AC 6010 — target segmentation.** `TargetOptionView.owner`, derived inside `NameIndex`
  from the **same already-redacted view** every `label` comes from — never from `GameState`,
  and never re-derived client-side, which would be wrong for exactly the case that matters
  (a stolen permanent sits in its *controller's* battlefield map, CR 109.4). `TargetPicker`
  groups by it, human seat first, unlabelled last. Grouping carries **original candidate
  indices**, so what is submitted is byte-identical to before.

**Tests**: **4,247 → 4,253 / 0 failing / 5 ignored** (baseline measured on this branch at
merge-base `f40c9fb9` before any edit — note this is the post-SIM-2/UI-2/CARDS-2 merged tree,
not CLAUDE.md's 4,218 UI-2 branch pin). +4 view-model (tier classification including the
documented default arm, redaction non-leak across three hidden-zone cases, exact prose), +2
play-server HTTP probes. **Every one watched failing under a deliberate revert**, including
two I re-ran independently rather than trusting the implementing agent's report.

**The fixture lesson worth carrying**: the combat probe first ran on `COMBAT_SEED` (6) and
passed while checking almost nothing — that seed offers **one** eligible attacker, so
"attacker → defender" collapsed to "there is a defender" and a bug swapping two attackers'
defenders would have passed. A sweep of `seed` ∈ 0..24 found that **every** seed offers 3
player targets (just CR 506.2) and **only seed 21 offers two eligible attackers**, because at
the turn the first attack becomes available the boards hold a single creature. New pin
`UI3_SPLIT_COMBAT_SEED = 21`, and **the split itself is asserted** (`distinct_defenders >= 2`)
rather than reported, so a re-deal fails loudly instead of leaving a test that still passes
while checking a strictly weaker property. The blocker half is asserted the same way.

**Gates**: PROTOCOL **33** / HASH **70** gate-EXECUTED unmoved (`--test core` hash_schema +
protocol_schema, 53 passing — not predicted); `decision_gate` 18/18; `cargo clippy --workspace
--all-targets -D warnings` clean; `cargo fmt --check` clean; `tools/check-defs-fmt.sh` clean
(1,803 defs); coverage untouched (0 card-def edits).

**S6 method, both ways**: play frontend **151 → 155 modules**, 0 warnings; replay viewer
**142 modules, unchanged**, and its **CSS bundle hash is byte-identical** across the change
(`index-DYVpLGsR.css` before and after) with the JS differing by 10 bytes — exactly the one
deliberate `formatTarget` string and nothing else. Only **one** `$viewer` file was touched
(`CombatView.svelte`); the four S7 pickers other than `TargetPicker` are byte-identical, and
their six `test_ui1_*` HTTP channel probes re-run green.

**Seeds filed** (`docs/audits/decision-point-audit.md` §8.1): **OOS-UI3-1** (nine wrong CR
citations in `events.rs` doc comments, all in the renumbered 701.x keyword-action block,
verified against the CR text — the largest instance of the OOS-DP6-8 rot class yet, and it
survived because every wrong number points at a *real* keyword action); **OOS-UI3-2** (two
event arms under-disclose because the only id they carry is the destination object — a
battlefield bounce is public in paper and renders name-free; needs a wire change);
**OOS-UI3-3** (fine-grained "until Bot-3 end"); **OOS-UI3-4** (no reveal channel on
`CardInZoneView`, so an opponent's seat card can never show a revealed hand card).

**Limitations 21–25** appended to `tools/play-server/README.md`.

**The fix cycle's finding is the one to read**: `/review` caught that **the 2×2 grid was not
2×2**. `repeat(auto-fit, minmax(22rem, 1fr))` packs as many tracks as *fit*, and four boards
need only ~88rem — so on any display wider than that the batch shipped a squeezed **1×4** row
with empty space to the right, which is *verbatim* the complaint the grid exists to answer.
`auto-fit` was chosen because it delivered the dead-player reflow with no code branch, and it
did; it also silently failed the headline requirement on exactly the machines most likely to
run this. **A CSS idiom that solves the requirement you were thinking about can fail the one
you started from, and neither the build nor any test can tell you** — there is no frontend
harness (plan §8 R7), so the only detector was reading it. Corroboration the reviewer found
and I had not: `--cells` was set inline on the grid and consumed by **no CSS rule** — a hook I
wrote for this and never finished, sitting in the file as evidence. Column count is now
computed. Second MEDIUM, same family: `.top-dock` was the **one uncapped sibling** — I capped
`.stack-dock` and `.hand-bar` and missed the container that hosts every picker, so an expanded
drawer plus a segmented `TargetPicker` could squeeze the board to nothing and push the page
into a *document* scrollbar, destroying the "stay in place on scroll" property the whole flex
arrangement exists to provide.

**One review finding was FALSE and was not actioned**, recorded because a future reader will
meet the same claim: the reviewer's sole HIGH said the `phase-end` predicate captures
`ctx.stackDepth` once at run start, so a resolve-then-recast slips through. It had quoted a
collapsed paraphrase and dropped the `ctx.stackDepth = depth` re-baselining line; its own
worked example fails at its step 3. Verified against source before deciding. (That gap **was**
real one commit earlier — I found and fixed it in a self-review pass before the reviewer ran,
which is presumably why it was reading for it.)

**Also worth carrying**: `MAX_EVENTS` was still **500**, chosen when ~11 `GameEvent` variants
rendered as prose; this batch took that to **60**, multiplying lines per turn. Shipping the
feature that makes a cap bite while leaving the cap alone would have truncated the very
history the feature exists to show. Raised to 2000. **A constant tuned against a behaviour is
part of that behaviour's blast radius.**

---

## Worker Handoff (SIM-2, `scutemob-176`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (SIM-2) — **F3 + F4 + F5 CLOSED**
**Task**: `scutemob-176`. Branch
`feat/sim-2-mana-intelligence-batch---residual-auto-tap-solver-cou`
**Full evidence**: `memory/primitives/sim2-mana-intelligence-2026-08-02.md`

**Completed**:
- **F4 — the mana solver counted SOURCES, not MANA.** `produces` was expanded per unit and
  the expansion never read, so Sol Ring was one mana. Both directions were live and a human
  saw both: over-tap (Sol Ring + two Forests for `{2}{G}`, one mana stranded and destroyed by
  CR 500.4) and, worse, **under-offer** — a `{2}` spell with only a Sol Ring untapped solved
  to `None`, so `can_afford` never offered the cast. A tapped source now credits its whole
  production to a running tally and each pip is paid from that tally.
- **F3 — auto-tap was all-or-nothing.** Pool covers the whole cost → tap nothing; anything
  less → solve for the **entire printed cost** with the pool never subtracted.
  `solve_mana_payment_with_pool` subtracts in `ManaPool::can_spend`'s own order and solves the
  residual; the early return is now the residual-is-zero case of the general rule.
  `advance()`'s bot path calls the same helper, and `can_afford` asks the same question once
  instead of a pool shortcut OR a whole-cost solve **with a gap between them** (a player with
  `{G}` floating and one Forest up was told `{1}{G}` was uncastable).
- **F5 — the bot tapped out every empty upkeep.** `TapForMana` 5 → **0**, below
  `PassPriority`. The demote-vs-gate choice is not arbitrary: every mana-consuming action
  already outscored 5, so a tap was only ever *chosen* when it was the sole alternative to
  passing. Scored 0 rather than removed, so it stays choosable when it is all there is.
- **The layer half of `OOS-M11-2`, recorded as theoretical, was live-wrong.** Changing which
  source the solver reaches for reddened the S8 scripted playthrough on seed 42:
  `"object ObjectId(487) has no mana ability at index 0"`. `layers.rs` clears
  `mana_abilities` for a **face-down** permanent (CR 707.2) and the solver read base
  characteristics. The doc block had illustrated the gap with *granted* abilities
  (Cryptolith Rite) and no urgency. Now `calculate_characteristics`, measured free:
  `mtg-fuzzer --games 60 --max-turns 40` is 6.8 s on both sides.
- **`OOS-CARDS2-9`, which existed only in three source comments and was never filed.** Its
  own statement of the fix — "one place: make the solver ask whether the ability is
  activatable" — was right about the affordability half and silent about the **offer** half:
  `legal_actions`'s `TapForMana` loop checked `life_cost` and nothing else, so an unmet
  activation condition and a summoning-sick creature were offered and refused, and the
  play-server driver carried both refusal strings in `KNOWN_FALSE_OFFERS`. One predicate,
  `tap_ability_is_activatable`, now serves both.
- **The bot half of `OOS-M11-8`.** S8 recorded it CLOSED on a fix living only in
  `auto_tap_commands_for` while `advance()` kept its own printed-cost solve. Latent (no
  shipped bot announces X > 0), but open. Closed by there being one function; pinned by
  `t21`, which drives a purpose-built `XBot`.

**Gates**: workspace **4,214 / 0 / 5**; `play-server` 40/40; clippy `-D warnings` clean;
`cargo fmt --check` + `tools/check-defs-fmt.sh` clean (1,803 defs); `cargo build --workspace`
(SR-3 seal) clean. **PROTOCOL 33 / HASH 70 gate-executed, unmoved** — the criterion's
"PROTOCOL 32" was stale, PB-DX6 moved it before this fork. Coverage unmoved: zero card-def
edits, zero completeness flips.

**Diff scope, stated exactly**: `crates/simulator` (3 source + 4 test files) +
`tools/play-server/src/main.rs` (one seed pin) + docs/memory + **one line of
`crates/engine/src/state/keyword_registry.rs`**. That last one is not scope creep and not
optional: SR-5's gate greps the source tree, so the solver's new CR 302.6 branch on
`KeywordAbility::Haste` must be declared or `core::keyword_registry` fails. It is a data
line; PROTOCOL/HASH are unmoved and `git diff main -- crates/engine/src/rules
crates/engine/src/effects crates/card-types crates/card-defs` is empty.

**Fuzzer A/B** (`--games 100 --seed 1 --max-turns 60`, merge base vs branch): **96/100 games
byte-identical**, 4 differ only in command count, violations 0 → 0, every game ends
`MaxTurnsReached(60)` on both sides. The four are the offer set moving.

**Fix cycle (`/review`, Opus)**: 8 findings, all applied. Two were live SR-38 violations the
batch had *asserted away*: (a) CR 605.3 **stax restrictions** — an opponent's Collector Ouphe
or Stony Silence refuses a Sol Ring's tap, and that class was mirrored in neither the solver
nor the offer loop while four comments claimed the mirror of `handle_tap_for_mana` was
complete (same shape as `OOS-SIM1-3`: an enumeration is only as complete as the category it
names — there enum variants, here the rejections inside one function); (b) an SR-36 **scaled**
ability's marker was called a safe under-count, but the engine adds `resolve_amount(..).max(0)`
with no error, so Itlimoc with no creatures out produces nothing while the marker promises one
mana — over-credit, refused cast. Both fixed and pinned (`t22`, `t23`). The other six were
documentation: a "hoisted" claim the same file contradicted two hundred lines later, the
`OOS-M11-2`/`OOS-M11-8` audit rows (criterion 5 asked for exactly this and the first pass
appended seeds without correcting the rows they contradicted), the playtest-triage F3/F4/F5
banners and roll-up, a defs-vs-ability-rows unit error in a population count, and a
discrimination matrix that claimed no test was decorative while having no row for the one
guarding `pick_least_waste`.

**Two engine findings carried out, both out of scope and both worth someone's attention**:
- **`OOS-SIM2-6` (HIGH)** — `calculate_characteristics` recurses without bound through
  `is_effect_active` → `check_static_condition` → `expect_characteristics`, and
  `indomitable_archangel` (`Complete`, deck-legal) makes that unconditional: its metalcraft
  static's activity depends on counting artifacts, which depends on layer-resolved types,
  which depends on its activity. **Hard, unrecoverable crash** (still overflows at
  `ulimit -s 524288`). Reproduce: `mtg-fuzzer --games 1 --seed 504 --max-turns 200` on this
  branch. Diagnosed by `gdb` backtrace plus a depth probe that named the card. Very likely
  the mechanism behind `OOS-M11-3` / `OOS-DP3-9`, which had the symptom and no cause.
- **`OOS-SIM2-5`** — `layers.rs` P/T arithmetic is unchecked `i32`; Devilish Valet's
  doubling reaches 2^30 and the next doubling panics in debug and **wraps silently in
  release**.

**Seed pin re-derived, for the second time in two days**: `TARGET_SEED` 1 → **13**, by the
rule the pin's own comment states (the pins are a function of the whole corpus *and of the
provider*; SIM-2 changes `can_afford`). Swept 0..24 running the four fixtures against each;
only 13 passes all four. Seed 1 now drives into `OOS-SIM2-5`'s overflow, which is recorded at
the pin so it cannot later read as a property of the fixture.

**Left open, deliberately**: `OOS-SIM2-1` (the solve is greedy — an under-offer is still
possible where source assignment interacts), `OOS-SIM2-2` (20 abilities with their own mana
component are never planned), `OOS-SIM2-3` (a bot still cannot pay an activated ability's
mana cost — `advance()` auto-taps for `CastSpell` only; pre-existing and unchanged in kind),
`OOS-SIM2-4` (SR-36 scaled production and CR 106.6a replacements under-counted — safe
direction), `OOS-SIM2-7` (the two `tools/tui` call sites inherit the production fix but not
the residual). What remains of `OOS-M11-2` after this batch is exactly cost *modifiers* and
CR 106.12 restricted mana.
## Worker Handoff (UI-2, `scutemob-178`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (UI-2) — **F9 CLOSED for `Sacrifice` + `Squad`**
**Task**: `scutemob-178`. Branch
`feat/ui-2-additional-cost-surfacing---sacrifice-squad-offer-descr`

**Completed**:
- **The request wire already existed; the OFFER was blind, and that was the whole
  defect.** `CastSpellData.additional_costs` covers all sixteen cost kinds and
  `ActionParamsDto` deserialized it, so a hand-crafted POST could pay a sacrifice
  before this batch. `StubProvider` simply never read `spell_additional_costs` or
  Squad — zero references — so Life's Legacy was offered on mana affordability alone
  and `casting.rs:3311` then refused it (the human's observed **422**, an SR-38
  violation), and a Squad creature always cast at `count: 0` with the optional cost
  silently lost (CR 702.157a).
- `LegalAction::CastSpell` gains `additional_costs: AdditionalCostPlan`. Eligibility
  mirrors `casting.rs:3300-3369` **gate for gate** — zone, controller,
  `object_cant_be_sacrificed`, then the filter against LAYER-RESOLVED characteristics
  — and deliberately **not** `effects::eligible_sacrifice_targets`, which also checks
  `is_phased_in` and would therefore offer a different set from the one the engine
  validates. `object_cant_be_sacrificed` is re-derived locally because the engine's
  copy is `pub(crate)`; documented as a *necessary* duplicate, explicitly unlike
  `effective_cast_cost`, whose engine copy is public and is consumed.
- **A required cost with nothing eligible suppresses the whole offer**
  (`offerable_cast_plan`, one helper used by BOTH cast loops). That is SR-38 restored,
  and it is F9's actual fix.
- `params.rs` appends the plan's default sacrifice only when the caller announced
  none, so `ActionParams::default()` (every bot) still produces an engine-accepted
  command and a human's choice is never overwritten. Squad is never defaulted —
  absent means declined, which keeps a bot's command byte-identical to the pre-UI-2
  one.
- `ActionOptionView.costs` + `CostPicker.svelte`, inserted between `ValuePrompt` and
  `TargetPicker`. `validate_additional_cost_params` answers **400** for an
  out-of-offer sacrifice id, more than one id, a Squad count above `max_count`, and a
  duplicate entry of either kind; the other fourteen `AdditionalCost` variants
  deliberately fall through to the engine's 422 and the doc says so.

**The card-def repair, which the brief did not anticipate**: `galadhrim_brigade` — the
very card the human tried to Squad — shipped `Complete` and deck-legal carrying
`KeywordAbility::Squad` with **no `AbilityDefinition::Squad { cost }`**, so
`casting.rs::get_squad_cost` returned `None` and *every* non-zero count was refused
with "spell has squad keyword but no squad cost defined". Repaired from the printed
"Squad {1}{G}". `core::ui2_additional_cost_roster` **R3b** now pins that the marker set
and the cost set are the **same set**, in both directions. This is the CARDS-2 shape
again: the knowledge existed per-def and nothing could fail.

**The fix cycle found the sharpest correctness bug**: `effective_cast_cost_with_additional`
**summed** multiple `Squad` entries where `casting.rs` **assigns** (`squad_count = *count`,
so the LAST wins). A two-entry submission therefore made the auto-tap reach for more
mana than the engine charges, the solver found no plan, no taps were issued, and the
engine refused the cast for want of mana — a 422 after a clean offer, which is exactly
the shape this batch exists to delete. Mirrored to last-wins, and duplicates are now
refused at the 400 boundary as well, because the engine resolves the two kinds in
**opposite** directions in silence (Squad last-wins, Sacrifice first-wins via `find_map`).

**The two findings that matter beyond this batch** — both measured, neither UI-2's to
fix:
- **OOS-UI2-1**: **`mtg-fuzzer` has never cast a spell.** `bin/fuzzer.rs` populates its
  libraries through `GameStateBuilder` and **never shuffles them**, while `random_deck`
  appends its ~34 basics LAST and `Zone::Ordered`'s top is the last index. Instrumenting
  the provider over 5 games x 80 turns gave **25,964 hand-card observations and zero
  non-lands**; `build_additional_cost_plan` was reached **0** times in 30 games. UI-2's
  own 360-game A/B came back byte-identical for that reason and is reported as worth
  nothing rather than banked. **Every "fuzz parity" claim in this project's history is a
  claim about a land-only game.**
- **OOS-UI2-2**: `HeuristicBot` scores `TapForMana` 5 against `PassPriority`'s 1, and in
  the upkeep those are the only two actions — so it burns its lands where it cannot
  spend the mana, the pool empties (CR 500.4), and by its own main phase the cast is
  never *offered*. A whole-game bot test therefore passes by never reaching the thing it
  claims to test.

**Numbers**: tests **4,185 -> 4,218 / 0 / 5**. PROTOCOL **33** / HASH **70**
gate-EXECUTED unmoved (the criterion's "PROTOCOL 32" is stale — PB-DX6 moved it before
this fork, the same staleness UI-1 recorded). `decision_gate` 18/18. Coverage unmoved at
**1,133/1,803 = 62.8%**, 0 completeness flips — the Galadhrim repair is an addition to an
already-`Complete` def. `fmt` + `check-defs-fmt.sh` + `clippy -D warnings` clean.

**NOT zero engine lines, and the exception is named**: **9 insertions / 1 deletion in
one file**, `crates/engine/src/state/ability_definition_registry.rs` — one data-only
`sites:` row adding `crates/simulator/src/legal_actions.rs` to `A::Squad`. The SR-15
gate demanded it the moment the provider read the cost-carrying variant; that gate's
`SCAN_ROOTS` includes `crates/simulator/src` **by design** (SR-20), and `A::Bloodrush`
already carries the identical row. `crates/card-types/src` diffs empty.

**Seeds filed** (`docs/audits/decision-point-audit.md` §8.1): **OOS-UI2-1..5** —
the fuzzer's unshuffled libraries; HeuristicBot's upkeep mana burn;
`squad_max_count`'s under-report (capped by playtest triage **F4**, whose test pins
the current wrong value and names the right one); the fourteen unsurfaced
`AdditionalCost` variants; and the TUI receiving the sacrifice default with no picker.

**Also deleted**: `local_game_playthrough.rs`'s `KNOWN_FALSE_OFFERS` register. Its last
entry was F9, its own trailing assertion required deletion once an entry stopped firing,
and the playthrough now asserts `run.error.is_none()` unconditionally — strictly sharper.

## Worker Handoff (CARDS-2, `scutemob-181`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (CARDS-2) — **SR-37 built; F1 + F2 CLOSED**
**Task**: `scutemob-181`. Branch
`feat/cards-2-corpus-field-fidelity-audit-permanent-gate-mana-cost`

**Completed**:
- **A new permanent gate, SR-37**: every `all_cards()` def's printed mana cost, power,
  toughness and type line is diffed against a committed Scryfall fixture. Three pieces —
  `tools/card-field-dump` (enumerates, SR-36), `tools/refresh-card-fidelity-fixture.py`
  (joins `cards.sqlite`, copies **verbatim**), and
  `core::cards2_printed_field_fidelity` (**the only place equality is decided**). The
  fixture is committed because `cards.sqlite` is gitignored and absent in CI; the Python
  does no normalisation on purpose, or the two sides would drift.
- **39 real defects found and repaired** across 31 defs (the gate's raw first run said 51;
  the difference is six false mismatches from its own notation and six more that were the
  design working — see the evidence record's three-column table): 17 mana costs, 5 P/T over 3 defs,
  16 type lines over 16 defs, 1 duplicate card name. **R2 reproduced the playtest-triage
  F2 table exactly, card for card** — first independent confirmation it was reproducible.
- **Boon Satyr (F1) fully repaired**, all four defects incl. the printed "+4/+2" that was
  **never authored** on a def declaring `Complete`. Expressed as two layer-7c statics on
  `EffectFilter::AttachedCreature` — **the shape Rancor already used**; the machinery was
  never missing. T5 proven discriminating **by execution** (revert → the bear stays 2/2).
- **Two more `Complete` defs were implementing a different card's abilities** —
  `backup_agent` (Backup 1 + Lifelink, from another card entirely) and `necron_deathmark`.
  Both repaired, both stayed `Complete`; both were caught because **more than one** printed
  field was wrong, which is the signal for "authored from a misremembered card".
- **Two more `Complete` defs implemented text on NO card at all** — `cyber_conversion`
  ("becomes an artifact + draw a card" for a printed "turn target creature face down")
  and `exalted_angel` (static `Lifelink` for a printed *triggered* "whenever this deals
  damage, gain that much life" — CR 702.15a lifelink cannot be Stifled; the printed clause
  can). Both **honestly demoted** with blocker notes naming the missing primitive
  (**OOS-CARDS2-5/6**), not half-repaired.
- **Zero engine lines**; PROTOCOL/HASH gate-executed unmoved; `decision_gate` 18/18; tests
  **4,185 / 0 / 5** (post-merge with SIM-1). Coverage **1,133/1,803 = 62.8%**, down from 1,137/1,804 — **4
  completeness flips, ALL demotions**. The number went DOWN because the corpus got truer
  (the PB-DX4 pattern): `cyber_conversion` and `exalted_angel` implemented text on no card,
  `braided_net`'s two remaining clauses have no expression (its note first claimed six,
  and a reviewer found four of those to exist), `birchlore_rangers`' mana
  ability has no `Cost` variant.

**Hazards for the next session — read these three:**

0. **A new browser-client defect fell out of the re-derivation: `OOS-CARDS2-4`.** An Aura
   is offered with `target_min: 0` — its target requirement lives in
   `KeywordAbility::Enchant(...)`, which `casting.rs:3720` special-cases (CR 303.4a) and
   the provider never reads — so the engine 422s the cast. **A human clicking any Aura in
   the play client gets an error.** Simulator-only fix; same shape as CARDS-1's equip bug,
   one link earlier in the chain. The S7 test driver now *skips* a refused action, which
   is a workaround in the test and NOT a fix.
1. **The seed pins are a function of the WHOLE CORPUS, not of the completeness markers.**
   Every play-server pin carried the comment "re-read when a batch flips a marker". This
   batch flipped **zero** markers and moved all of them, because
   `simulator/src/deck.rs::random_deck` draws its commander from `Complete` **AND
   Legendary AND Creature** and fills by **colour identity** (computed from the mana
   cost). Measured: commander pool 91 → 90. Correcting a *type line* re-deals every
   seeded game. All the comments now say so. Filed as **OOS-CARDS2-3** (no gate exists).
2. **A fixture predicate broader than the fixture's purpose does not fail when the fixture
   moves — it silently tests something else.** `test_x_value_is_forwarded_to_cast_spell_data`
   had retargeted from a spell onto Deserted Temple's "untap target land", and the failure
   surfaced three assertions later as "the cast is still offered after tapping" (it was
   never a cast). Predicate now says `CastSpell`.
3. **A golden script generated from a card def is not an independent check of that def.**
   Scripts 177 and 164 were written to what the def said, not to the card, and passed for
   two batches while encoding a wrong cost. Script 163 is **retired** — its subject
   (Backup Agent's Backup 1) does not exist.

**Also worth carrying**: the duplicate-name finding (R5) had been written down in
`memory/card-authoring/marker-sweep-2026-07-16.md` **seventeen days earlier**, with the
words "one of the two should be deleted", and nothing happened — because no gate could
fail. `CardRegistry::try_new` rejects a duplicate `CardId` and says nothing about a
duplicate name.

**The fix cycle found the sharpest thing in the batch — read this one.** `tyrranax_rex`,
the gate's own motivating example, shipped `Complete` declaring `KeywordAbility::Ravenous`
— **on no printing of the card** — while omitting haste, Toxic 4 and "can't be countered";
a golden script certified the invented keyword. And that script had already FAILED earlier
in this same batch when the cost was corrected, and was re-baselined by recomputing its
mana pool **without re-reading the oracle** — the exact failure the batch had written down
for scripts 164/177 one commit earlier. Repaired in full (every primitive existed); script
177 retired alongside 163. **The rule: a wrong printed field is reason to re-read the whole
oracle, not to fix the field.** The batch's own "more than one wrong field = misremembered
card" heuristic cannot catch this class, because only one field was wrong.

**A gate-needle gap worth someone's time**: `braided_net` and `windbrisk_heights` both
shipped `Complete` with a printed ability unimplemented and said so in their own comments —
"DSL gap" and "deferred". `completeness_deviation_scan`'s needle set is
`["simplif", "modeled as", "modelled as", "deviation", "approximat"]`, so neither reddened.
Both were also **stale** claims: `Effect::TapPermanent` and
`Condition::YouAttackedWithNOrMore` had both landed since. That is the third and fourth
"not expressible" note this batch found to be false (after `wake_the_dead`'s `x_count` and
`boon_satyr`'s aura static).

**Full evidence record**: `memory/card-authoring/cards2-field-fidelity-2026-08-02.md`
(measurement, every disposition, the four gate-design findings, and seeds
**OOS-CARDS2-1..11** — 7..11 came out of three review fix cycles: **OOS-CARDS2-7** the
`completeness_deviation_scan` needle set has no entry for "DSL gap" or "deferred", the two
phrases the corpus actually uses; **OOS-CARDS2-8** stale "not expressible" notes are a
recurring class, four found false in this batch alone). Gate rationale + refresh procedure:
`docs/engine-invariants.md`
(SR-37).

## Worker Handoff (CARDS-1, `scutemob-179`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (CARDS-1) — **OOS-M11-10 (equip) CLOSED**
**Task**: `scutemob-179`. Branch `feat/cards-1-equip-target-repair-batch---close-oos-m11-10-16-defs`

**Completed**:
- **17 card defs repaired** (not the 16 the seed scoped): every `AbilityDefinition::Activated`
  whose effect is `Effect::AttachEquipment` now declares
  `TargetCreatureWithFilter { controller: You }` (CR 702.6a). **0 engine lines.**
- **Roster re-derived from `all_cards()` per SR-36**, never from the seed's def-source scan. It
  confirmed the seed's counts exactly (17 activated attach sites, 16 empty, `cryptic_coat`'s ETB
  self-attach correctly excluded, 4 prose-only files correctly excluded) — and then broke its
  conclusion, see the lesson below.
- All 17 printed equip lines MCP-verified as plain `Equip {N}`: **no CR 702.6c quality
  restriction anywhere**, so there is no per-def deviation to document.
- New permanent gates: `core::cards1_equip_target_roster` R1–R3 and
  `primitives::cards1_equip_target_repair` T1–T7b (11 tests). Fail-before evidence with verbatim
  pre-fix output: `memory/primitives/cards1-equip-fail-before-2026-08-02.md`.
- Gates: **0 completeness flips** (report body byte-identical, 1,137/1,804 = 63.0%);
  **PROTOCOL 33 / HASH 70 unmoved**, verified by *executing* `core hash_schema` +
  `core protocol_schema`; `decision_gate` 18/18, no pin moves; `cargo fmt --check` and
  `tools/check-defs-fmt.sh` (1,804 defs) both clean.

**Durable lessons** (the reason this handoff is worth reading):
- **The designated reference def was itself wrong.** The seed named `helm_of_the_host` as the one
  def that "declares the `TargetRequirement`" — true, and it was read as "already correct". It
  declared a bare `TargetRequirement::TargetCreature`, dropping CR 702.6a's "you control", so it
  offered opponents' creatures as legal equip targets. *Being the only member with a requirement
  is not the same as being the only member with a correct one.* A batch that trusts its reference
  without re-deriving it inherits the reference's defect — the same shape as PB-DX6's "the brief
  named one arm; all three shared the defect".
- **Two tests written to fail pre-fix passed pre-fix**, and that was information, not noise: the
  legacy `AttachEquipment` special-case in `abilities.rs` *does* validate a **volunteered**
  target. So the defect was never "equip doesn't validate" — it was "**nothing ever asks**".
  That is exactly why the TUI (which volunteered targets) never surfaced this and the browser
  client did on its first human game. The prediction was recorded as wrong rather than smoothed.
- **`OOS-M11-10` names TWO distinct seeds** in `docs/audits/decision-point-audit.md` §8.1 — the
  equip one (closed here) and a still-OPEN loyalty-ability targeting gap filed the same day by
  M11-local S8's close-out. **Every cite of the ID outside that table — CLAUDE.md, the
  milestone-reviews doc, `params.rs`'s in-source comment, and line 183 of this file — means the
  LOYALTY seed.** Both rows are now labelled and a collision note sits under the table.
  Renumbering was declined here: it would rewrite an in-source engine comment, and this batch is
  pinned to zero engine lines. Whoever next touches `params.rs` should renumber the equip row.

**Not done / deferred (deliberate)**:
- **OOS-CARDS1-1** — `darksteel_garrison` has the identical shape for **Fortify** (CR 702.67a).
  Card-def-only and 0 engine lines via
  `TargetPermanentWithFilter { has_card_type: Land, controller: You }` — verified live in
  `casting.rs`, not assumed. Left alone because criterion 6003 required neighbouring attach
  mechanisms be untouched.
- **OOS-CARDS1-2** — **Reconfigure** has it too, but the defective `targets: vec![]` is written in
  *engine* source (`testing/replay_harness.rs`'s `AbilityDefinition::Reconfigure` expansion), so
  zero-engine-lines excluded it by construction. CR 702.151a says "**another** target creature you
  control" — it needs `exclude_self: true`, and copying the equip repair verbatim would be wrong.
- Both rosters are **pinned** by `t7b` (`{"Darksteel Garrison"}`, `{"Lizard Blades"}`), so either
  fix must move a pin in the same change.
- **OOS-CARDS1-3 — the biggest of the three, and it came from the `/review`, not from me.** 21
  Equipment defs print "Equip {N}" and have **no equip ability at all** (`K::Equip` is a
  `KeywordHandling::Marker` that synthesises nothing), **10 of them deck-legal `Complete`**, 9 by
  the `#[default]` derive. That is a larger population than this batch touched and one link
  earlier in the same chain: not "the picker never asks for a target" but "**there is no action
  to pick**". A human can legally deck Umezawa's Jitte or Sword of Feast and Famine today and
  never be offered an equip. Four of the 11 `partial` members already named this gap in their own
  completeness notes — the knowledge existed per-def and had never been aggregated into a seed.
  **Lesson**: R1's exact-17 pin makes a true statement ("all 17 members are correct") that reads
  as a false one ("the equip surface is swept clean"). A roster gate certifies the population it
  enumerates and is silent about the population it does not — and the defs that fall outside it
  are exactly the ones no gate is watching. Whoever takes OOS-CARDS1-3 should also decide whether
  R1 grows a companion pin over marker-only Equipment.

**Hazards for the collector**:
- CLAUDE.md and this file both got a new **appended** section (no existing line grown), per the
  2026-08-02 formatting rule — expect the usual both-sides-edited conflict and take the richer side.
- `docs/authoring-status.md` / `-prev.json` were regenerated to measure flips and then **reverted**,
  because the only delta was the timestamp/SHA header. Do not re-run and commit them.

**Commit prefix used**: `scutemob-179:`

## Worker Handoff (SIM-1, `scutemob-175`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (SIM-1) — **triage F7 CLOSED**
**Task**: `scutemob-175`. Branch `feat/sim-1-commander-castable-from-the-command-zone-legalactionpr`

**Completed** — a human can cast their commander from the command zone. **Zero engine lines**
(`crates/engine/src` + `crates/card-types/src` + `crates/card-defs` diffs all empty and pasted);
PROTOCOL **33** / HASH **70** gate-executed unmoved.

- **The engine was never the problem.** `casting.rs` has supported CR 903.8 since M6 — it derives
  command-zone-ness from the object's zone, admits it past the "not in your hand" gate, gates it
  on `commander_ids`, applies the tax and increments the counter, and emits
  `CommanderCastFromCommandZone`. `StubProvider` simply never looked in the zone, so the browser
  correctly reported that the server had offered nothing. **The frontend was innocent and so was
  the wire**: `params.rs` already forwarded the bare card, and `from_zone` is read *nowhere* in
  the workspace.
- **`effective_cast_cost` — one helper, three call sites.** The brief named one place the tax was
  needed; there were **three**, and `local_game.rs`'s own doc block already described the defect in
  as many words ("Recasting a taxed commander with a pool that covers only the printed cost
  therefore skips tapping and the cast is rejected"). The offer gate, the human `submit` auto-tap
  and the bot `advance()` auto-tap all read the **printed** cost. They now share one helper that
  **consumes `mtg_engine::apply_commander_tax`** rather than re-deriving `generic + 2*tax` — SR-38's
  "only offer what the engine accepts" is only true if the two arithmetics are literally the same
  function. (Contrast `multiply_mana_cost`, a *necessary* duplicate because the engine's copy is
  private. This one is not, so duplicating it would have been a choice, and the wrong one.)
- **The Drannith trap — the finding that would have shipped a fresh SR-38 violation.**
  `casting.rs` rejects **any** non-hand cast while an opponent controls a Drannith Magistrate, and
  `is_cast_restricted_by_stax` says in its own doc that it deliberately does not mirror per-card
  *zone* restrictions. That was harmless for exactly one reason: every offer the provider had ever
  made was a hand cast, and a hand cast always satisfies `zone == Hand(player)`. **Every
  command-zone offer is a non-hand cast**, so without a new mirror the batch would have offered an
  action the engine rejects 100% of the time. `drannith_magistrate.rs` is deck-legal `Complete` by
  the `#[default]` derive. Generalisable: **a guard that is "harmless because unreachable" becomes
  a defect the moment you widen what reaches it — check the reachability argument, not the guard.**
- **Timing is mirrored, not assumed.** A commander is a permanent, so sorcery speed is the *usual*
  answer — but the engine's timing gate is zone-agnostic, so a commander with Flash or under a CR
  601.3b flash grant is legally castable at instant speed. The hand loop's timing block was
  extracted to `can_cast_at_this_time` and is now called by both enumerations, so they cannot drift.
- **Appended after the hand loop on purpose**: `RandomBot` picks by index, so appending leaves every
  pre-existing action's index untouched.

**The regression, and why it is not SIM-1's bug** (the durable lesson of this batch):
`local_game_playthrough` seed 1 halted `InfiniteLoop` at turn 17 having applied exactly 20,000
commands. Diagnosed **by measurement, not by reading code** — a throwaway instrumented copy of the
test printed a per-turn, per-kind histogram: **19,351 of those commands were `DeclareAttackers` in
that single turn.** The cause is the already-open seed **`OOS-M11-9`**: nothing gates "attackers
already declared this combat", so a **vigilant** attacker stays untapped, stays `eligible`, and is
re-offered without limit (CR 508.1 makes it a once-per-combat turn-based action). SIM-1 only made
it *reachable* — seed 1's human commander is `Samut, Voice of Dissent`, which has Vigilance, and
before this branch no commander could ever be cast, so no vigilant commander was ever on the
battlefield to re-declare with. It is the same seed, the same turn range and the same
20,000-command signature the audit already records for the S8 **bot-side** instance.
**The fix location was already decided, in shipped source.** `heuristic_bot.rs` mitigated the
identical loop with a per-combat `RepeatKey` cap and states its reason: put it in the client
"rather than in `StubProvider` … keeps the provider's action list, and therefore every recorded
`mtg-fuzzer` seed, untouched." The scripted human policy is simply the **second client** to need
it, so it got the same cap — reset on the **combat-entry edge**, not the turn number, because
`MR-M11-09` found exactly that regression in the bot (a turn-keyed tally silently disables attacks
in every CR 506.5 extra combat). **No assertion was relaxed.**

**A/B evidence, measured in a separate git worktree at the true merge-base — not reasoned to:**
- **Fuzzer unperturbed.** 60 games, `--seed 42 --max-turns 50 --verbose`: per-game
  `Seed/Turns/Commands/Violations/Error` lines diffed with **zero** differences; the only differing
  line in the entire output is the games/sec throughput counter (58 vs 57), i.e. timing noise.
  This is immunity **by construction** — `fuzzer.rs` never calls `builder.player_commander`, so
  `commander_ids` is empty and the `commander_ids`-gated offer is unreachable there (`OOS-SIM1-4`).
- **Playthrough trajectory essentially unmoved.** Per-seed commands, merge-base → branch:
  1058→1064, 1177→1183, 1164→1172, 1010→1010, 1118→1111 — within ~1% on every seed, with identical
  per-seed action-kind coverage sets.
- **A correction worth carrying**: I first reported these seeds as finishing "below the pre-SIM-1
  baseline". That compared against a **stale comment inside the test file**, written for a
  different `max_commands` config. The measured answer is *unchanged*, which is a stronger result —
  but the lesson is the recurring one here: **a number written in a doc is not a baseline; the
  baseline is what the merge-base actually does when you run it.**
- **Pre-existing failure correctly attributed**: the documented smoke command
  (`--games 100 --seed 42`, default `--max-turns 200`) **stack-overflows on the merge-base too** —
  `OOS-M11-3` / `OOS-DP3-9`, reproduced on pristine code, not SIM-1.

**Seeds filed** (durable rows in `docs/audits/decision-point-audit.md` §8.1, the same table CARDS-1
used): **`OOS-SIM1-1`** (hybrid/Phyrexian commander gated by `can_afford`, not a payment plan —
`CastSpell` has no PB-RS2 channel; note the tax cannot *create* a pip, since `apply_commander_tax`
writes only `generic`), **`OOS-SIM1-2`** (a **fourth** printed-cost auto-tap in `tools/tui`,
outside this batch's scope — which is why `effective_cast_cost` is exported `pub`, so the fix is a
call and not a copy), **`OOS-SIM1-3`** (verified exhaustively against all 9 `GameRestriction`
variants: 7 are cast-relevant, the provider now mirrors 5, and exactly
`MaxNoncreatureSpellsPerTurn` + `MaxNonartifactSpellsPerTurn` remain unmirrored — pre-existing for
hand casts, deliberately not widened), **`OOS-SIM1-4`** (the fuzzer's games are not Commander games
at all: no tax, no CR 903.9a return, no CR 903.10a commander damage is ever fuzzed — deliberately
unfixed, because fixing it moves every recorded seed).

**Scope note the coordinator must record, not swallow**: criterion 5984 requires an HTTP probe and
`tools/play-server` is a **bin** crate with no `lib.rs`, so no `tests/` integration test can reach
`build_router` — every HTTP test in this crate lives in `main.rs`'s `#[cfg(test)] mod tests`. So
criterion 5987's "empty git diff elsewhere" is satisfied as: engine/card-types/card-defs diffs
**empty**, and the `main.rs` diff **proven** test-only by line arithmetic rather than asserted —
the `#[cfg(test)]` cut is at line 207 and the lowest changed line is 3873, so the shipped binary is
behaviourally identical.

**Commit prefix used**: `scutemob-175:`

## Last Handoff

**Date**: 2026-08-02 (oversight session — the full playtest-successor run)
**Workstream**: playtest-triage successor track (UI-1/2/3, SIM-1/2/3, CARDS-1/2)
**Task**: coordinated dispatch of `scutemob-174..181` in four waves of two workers; all 8 collected
same-day. Merges: `f28df527` (174 UI-1), `d04f42a1` (179 CARDS-1), `83bfdba5` (175 SIM-1),
`8cad9c36` (181 CARDS-2), `b30c99f4` (176 SIM-2), `f40c9fb9` (178 UI-2), `a23f0be0` (177 SIM-3),
`b76b1df4` (180 UI-3); bookkeeping `662e4264`.

**Completed**:
- **Playtest triage 2026-08-02 fully closed** — F1–F10, OPEN = none (roll-up in
  `memory/playtest-triage-2026-08-02.md`, rewritten at collect as the union of two
  mutually-blind updates).
- Tests **4,124 → 4,263 / 0 / 5** on main; PROTOCOL **33** / HASH **70** unmoved by every batch,
  gate-executed each time. Coverage **62.8%** (1,133/1,803) after CARDS-2's honest demotions.
- Per-batch detail: the eight Worker Handoff sections above; per-batch narratives rotated to
  `memory/archive/claude-md-changelog-2026-08.md` at the wave-4 collect.
- **Two cross-branch reconciliations happened at collect, not in any worker**: (1) UI-2 × SIM-2
  conflicted in `local_game.rs::advance` — resolved onto SIM-2's unified `auto_tap_commands_for`,
  inside which UI-2's Squad pricing already lived; (2) UI-2's F4 pin test flipped 0 → 1 by its own
  written instruction (SIM-2 closed F4 in parallel) and was renamed
  `squad_max_count_counts_true_production_now_that_f4_is_closed`; OOS-UI2-3 row annotated.

**Not done / deferred**:
- `scutemob-127` (abilities-corpus distillation) — pre-existing backlog, out of the run's scope.
- PB-DX7 (SR-19 gate checks nothing) — still the standing queue's next item, untouched.
- The seeds below — filed, not fixed.

**Next session candidates**:
- **OOS-SIM2-6 (HIGH)**: unbounded `calculate_characteristics` recursion — hard crash from a legal
  deck (`indomitable_archangel`); likely the real cause of OOS-M11-3/OOS-DP3-9.
- **OOS-UI2-1**: the fuzzer has never cast a spell (unshuffled libraries) — closing it together
  with OOS-SIM1-4 (fuzzer games aren't Commander games) re-rolls every recorded seed ONCE.
- **PB-DX7** per the standing queue (`memory/primitives/seed-rerank-2026-07-27.md` §4).
- OOS-UI2-4 (14 remaining additional-cost kinds), OOS-SIM2-5 (i32 P/T wrap).

**Hazards** (carrying forward):
- Parallel workers sharing `crates/simulator` or `tools/play-server` WILL conflict on
  `local_game.rs` + the four coordination docs; collect one at a time and re-check the second
  against the new main. Semantic conflicts (a pin one branch wrote, the other branch's fix flips)
  survive a clean textual merge — run the FULL suite between collects, with output captured
  (a `| tail` pipe destroyed the evidence once this session).
- Every task staged before PB-DX6 cites PROTOCOL 32 in its criteria; main is 33. Brief workers.
- Assertion messages containing mana symbols (`{2}`) are format strings — escape the braces.
- Workers leave throwaway A/B worktrees under scratchpad paths — check `git worktree list` at
  collect, not just `esm worktree list`.

**Commit prefix used**: `scutemob-N:` (workers) / `merge:` / `chore:`

## Worker Handoff (UI-1, `scutemob-174`)

**Date**: 2026-08-02 (worker session, `scutemob-174` — UI-1 blocking-decision pickers)
**Workstream**: M11-local maintenance track (`crates/simulator`, `tools/play-server`)
**Task**: `scutemob-174` — branch `feat/ui-1-blocking-decision-payload-channel-pickers-discard-scrys`

**Completed** — playtest-triage **F8** closed on the browser surface:
- **Three layers, one mechanism.** `StubProvider` bakes the engine-accepted default into every
  blocking-decision `LegalAction` (cleanup discard = the `count` highest `ObjectId`s, scry/surveil
  = the identity partition, search = `candidates.first()`) so a *bot* can submit it and always be
  accepted (SR-38). The candidate data rides along so a *human* client can render a choice. The
  view layer threw it away, so the browser drew one bare button that submitted the default.
- `crates/simulator/src/params.rs`: `ActionParams` gains `discard_cards` / `effect_choice_answer`
  / `trigger_targets`; the three arms forward an announced answer and fall back to the same
  default as before; the three variants join the allowlist and `first_announced_field`.
- `tools/play-server/src/view.rs`: `ActionOptionView.decision` — a generic
  `{question, prompt, answer_field, answer}` envelope whose `answer` is one of **four shapes**
  (`Subset` / `PickOne` / `Partition` / `Slots`). `ActionParamsDto` gains the three answer fields.
- `tools/play-server/src/api.rs`: `validate_decision_params` — an answer naming something the
  response never offered is a **400**, not an engine 422.
- Frontend: `DiscardPicker`, `PartitionPicker` (scry AND surveil), `SearchPicker`; `ActionBar`
  gains a `'decision'` stage dispatching on `answer.shape`; `TargetPicker` now hands back the
  grouped `Target[][]` alongside the flat list.
- **Tests 4,124 → 4,136** (+8 params unit, +5 play-server). Zero engine lines (empty `git diff`
  over `crates/engine/src` + `crates/card-types/src`), PROTOCOL **33** / HASH **70** unmoved
  (gate-executed, not predicted).

**Durable lessons this session paid for**:
1. **CR 400.7 defeats id-following assertions.** The scry and search probes' first drafts followed
   an `ObjectId` from the library into the hand. A card that changes zones is a NEW object. Both
   now assert over the **library**, where the ids survive — and the two answers are distinguished
   by *which* card is still in it.
2. **A probe can pass on a printed keyword.** The trigger-target probe's first version asserted
   "the chosen creature has a keyword" and **passed against the un-fixed code**, because Nezumi
   Prowler is printed with Ninjutsu. It now asserts what each creature *gains* against a baseline
   taken before the answer. Every probe here was re-checked by reverting the fix and watching it
   go red; that check is what caught this one.
3. **A generic payload's extension claim is worth its test.** `Slots` was built so OOS-DP8-2 would
   need no rework, and it did not — but the claim only became evidence once a real pair of
   `Complete` cards (Shadow Alley Denizen + Nezumi Prowler) drove it end to end. Every other
   mono-black route was checked and rejected: PB-EF6 retargeted them all to `TargetOpponent`,
   which has exactly one candidate in a 2-player game and is therefore always forced.
4. **A fourth Invariant-7 channel, opened deliberately.** `StateViewModel` models `library_size`
   and no library *contents*, so `NameIndex` answers `(unknown card)` for every scry/search
   candidate. `view::question_card_label` reads the name off `GameState` for ids drawn out of the
   engine's own `EffectChoiceQuestion` — whose `private_to()` already classifies exactly that id
   set as this seat's. MR-M11-01's lesson applies verbatim (*a redaction gate checks the channel
   it was written for*), so it ships with its own gate: `view.rs`'s production code may read the
   raw object table exactly **twice**, and a third read must be deliberate.
5. **`session::new_game` is the deck-injection seam.** `config_for` hard-codes
   `DeckSource::RandomPerSeat`, but `new_game` takes any `LocalGameConfig` and runs the same two
   Invariant-9 gates — so a `#[cfg(test)]` fixture can install a `DeckSource::Fixed` session and
   still drive every request through the real router. One seed (184) serves two different fixture
   decks because the shuffle permutes *positions* and both probe spells sit at `main_deck[0..2]`.

**Fix cycle (Opus review, 1 HIGH + 1 MEDIUM + 4 LOW, all closed)** — and the HIGH is the one
worth carrying:
- `question_card_label`'s doc **cited a gate test that did not exist**
  (`test_ui1_a_bot_seats_effect_choice_never_reaches_the_human_payload`) and said the channel
  "ships with its own gate rather than with an argument". That is this project's own defect
  class — a claim in prose that no test holds — landing on the one subsystem MR-M11-01's lesson
  is about. It was a draft line left behind when the planned behavioural test was replaced by a
  source-count gate; the README and the archive entry both described the real situation
  correctly, which is how it survived self-review.
- Worse, the premise that test would have asserted was **enforced nowhere**. The channel's safety
  argument needs the `EffectChoiceQuestion` to belong to the seat being rendered, and that held
  only by arithmetic on a one-element set (`config_for` hard-codes `human_seats: [HUMAN_SEAT]`).
  A second human seat — the obvious M10a direction — would have rendered seat A's scried library
  cards, **with real names**, into seat B's payload. `api.rs::seat_view` now filters
  `pending.player == human`, and `test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload`
  holds it two-sidedly. **Generalisable**: when a doc comment says "structural", check that the
  structure is in the code and not in the configuration.
- The new test's own first version mutated `pending.player` and did nothing — every route calls
  `advance()`, which refreshes `pending` straight off `LocalGame`. It moves `PlaySession::human`
  instead. Recorded in the test's doc.
- LOWs: the same doc block said "fourth channel" in its heading and "a fifth" five lines later;
  `question_kind`'s rationale claimed redaction while two functions above it format candidate ids
  into their own 400 bodies (corrected to what it is — message quality); `ActionBar`'s decision
  guard required `currentShape`, so a malformed payload rendered nothing and **skipped the very
  fallback that exists to prevent a dead bar**; the count gate's narrowness (one needle, blind to
  `zones()`/`card_registry()`) is now stated rather than implied.

**Re-review (second pass)**: all 6 confirmed fixed and the new gate confirmed two-sided by
execution — but the fix cycle had left 4 doc defects of its own, **two in the same class it was
convened to fix**. That recurrence is the point, not an aside: writing the correction is a second
chance to assert something no test holds. One was substantive — `seat_view`'s comment justified
dropping a foreign decision with "`submit`'s own `seq` check already refuses to act on it", which
is false (`pending_wire_seq` ignores `human`, and the 409 body discloses the current `seq`, so a
client could learn a hidden decision's seq and submit against it). `post_action` now refuses it
too, and the gate asserts both halves using the seq captured *before* the move. Also corrected:
the gate's own description said it retargets the decision (it retargets the viewer); "asserts
every name is gone" overstated a needle that is the `looked_at` **key** (names are not assertable
— seat 2 legitimately holds Swamps); an unresolvable `[check_ids]` intra-doc link; and both the
code comment and the README now say plainly that this pair is **fail-closed, not M10a-ready** —
`PlaySession::human` is a single `PlayerId`, so a real second human seat would be deadlocked
rather than served, and the missing piece is a per-request viewer.

**Confirmation pass (third)**: all 5 second-cycle findings confirmed fixed, both guard halves
confirmed two-sided **by execution** — and the write half turned out to close a real hole, not a
theoretical one: with `post_action`'s guard deleted the probe gets **HTTP 200 and the other seat's
scry is applied**. No new instance of the "guard that is not there" class; one instance of its
**inverse** — three comments still advertising a `seq`-disclosure channel that the guard's own
placement (above the staleness check) had already closed. Closed the same way as everything else
here: the gate now asserts that a wrong `seq` against a foreign decision answers
`no_pending_decision` rather than `stale_decision`, whose body would carry `expected: <the real
seq>`.

**The through-line of three review rounds, worth more than any one finding**: every round found
prose out of step with the code, in one direction or the other, and every round's fix was the
same — *make a test hold the claim*. A comment that says "structural", "gated", "already
refuses" or "discloses" is an assertion; if no test executes it, it decays at the speed of the
code around it. Two of the three rounds' faults were introduced by the correction to the previous
round.

**Not done / deferred** (all recorded as play-server README limitations 14-17):
- The TUI halves of OOS-DP7-6 / OOS-DP8-2 / OOS-DP9-7 are untouched; those rows are *about* the
  TUI and remain open. OOS-DP9-1 is unchanged and deliberately so — it is about the bot, and the
  bot still submits the default, which is what keeps every recorded fuzzer seed reproducing.
- No picker has an automated test (no frontend harness exists, plan §8 R7).
- `Slots` has no "use the default" button; `PartitionPicker` has no ordering control on the moved
  pile (CR 608.2f says that order is the player's).
- Two pre-existing broken intra-doc links in `tools/play-server/src/view.rs` (`GameSummary::seed`,
  `crate::api::validate_combat_params`) predate this branch and were left alone. CI runs
  fmt/clippy/build/tests, not `cargo doc`, so nothing goes red — noted for whoever wants them.

**Next session candidates**: `scutemob-175` (SIM-1 commander cast) or `scutemob-177` (UI-2
additional costs) — UI-2 is the one UI-1 was meant to pre-shape, and its `CostPicker` should slot
into the same `pickerNeeded` chain.

**Commit prefix used**: `scutemob-174:`


## Prior Handoff (oversight — wave-7 recovery, both collects, playtest triage)

**Date**: 2026-08-02 (oversight session — wave-7 crash recovery, both collects, playtest triage)
**Workstream**: coordinator — W6 (PB-DX6 collect) + M11-local (S8 collect, MILESTONE CLOSED) + triage
**Task**: `scutemob-172`/`173` collected; `scutemob-174..181` created; merges `51878905` + `cb0755bf`

**Completed**:
- Both wave-7 crashed workers restarted per the agreed recovery (173 resumed on its WIP,
  172 fresh from plan `4d367c54`; crashed WIP preserved at `wip/scutemob-172-crash-20260802`).
- **`scutemob-173` COLLECTED (`51878905`) — M11-LOCAL COMPLETE**, on-main 4,097/0.
- **`scutemob-172` COLLECTED (`cb0755bf`) — PB-DX6 SHIPPED**, PROTOCOL 32→**33** / HASH **70**;
  combined S8+DX6 tree measured on main: **4,124 / 0 / 5 ignored**. Both tasks `done` in ESM.
- **OOS-M11-10 filed** (`e4b93ac0`): equip `targets: vec![]` silent fizzle — measured 16 of 17
  real equip activations, 10 `Complete` via the `#[default]` derive.
- **First-human-playtest triage**: every claim in `test-data/bot testing notes.md` verified
  against code — `memory/playtest-triage-2026-08-02.md` (F1–F10, ZERO engine bugs; all
  simulator / play-server / card-defs). Corpus-wide mana-cost audit: **17 wrong costs,
  9 deck-legal `Complete`** (`tyrranax_rex` 3 cheap on a 7-drop).
- **Successor tasks `scutemob-174..181` created** (UI-1 pickers, SIM-1 commander cast, SIM-2 mana
  intelligence, SIM-3 invariant residuals, UI-2 additional costs, CARDS-1 equip batch, UI-3 UX
  polish, CARDS-2 field-fidelity gate). 176/177 carry re-baseline comments — S8 already closed
  OOS-M11-8 and rewrote the false-positive `stack_consistency` check.
- **CLAUDE.md line hygiene** (`fdb872b6`): 12 changelog entries rotated to the monthly archives,
  Current State rewrapped at ~100 chars, formatting rule pinned in the file.

**Not done / deferred**:
- `scutemob-174..181` all in backlog, none dispatched (standing directive: every dispatch needs
  explicit user approval). PB-DX7 (SR-19 gate holes, test-only) next in the W6 queue, undispatched.
- This file (`workstream-state.md`) still has its own mega-lines (the W6 table row is 30k+ chars) —
  same disease CLAUDE.md was cured of; treat in a future chore.

**Next session candidates**:
- Dispatch `scutemob-174` (UI-1 blocking-decision pickers) — biggest agency win, pre-shapes UI-2.
- Then `scutemob-181` (field-fidelity gate) + `scutemob-176` (mana intelligence) — parallel-safe.
- Or PB-DX7 in the W6 lane (disjoint from all of the above).

**Hazards** (carrying forward):
- CLAUDE.md formatting rule is NEW: close-outs append a short delta and rotate detail to the
  monthly archive — never grow an existing line.
- Read the ESM comments on 176/177 before dispatching them (scopes shrank post-S8).
- User's `tools/play-server/frontend/package.json` edit left uncommitted deliberately.

**Commit prefix used**: `merge:` / `chore:` / `scutemob-172:` (worker)


## Prior Worker Handoff (PB-DX6, preserved for chain context)

**Date**: 2026-08-02 (worker session, `scutemob-172`)
**Workstream**: W6 (primitives) — **PB-DX6 SHIPPED**, sixth batch of the PB-DX queue
**Task**: `scutemob-172`. Branch `feat/pb-dx6-the-last-two-unflattened-mana-cost-payment-sites-oos-`, 8 commits.

> **Restart note**: this task crashed mid-implement in a prior session and was redone **from
> scratch** from the plan commit `4d367c54`. The crashed WIP survives at
> `wip/scutemob-172-crash-20260802` for reference only; nothing was cherry-picked from it.
> The redo was staged deliberately (0/A/B/C/D/E/F) with a commit per stage, precisely because
> the previous single-pass attempt ran out of room. That staging is the reusable part.

**Completed**:
- **OOS-RS2-1 and OOS-DP4-1 both CLOSED.** `rules/engine.rs::handle_turn_face_up` paid a **raw**
  `def.mana_cost`, and it is **all three** `TurnFaceUpMethod` arms that share the defective payment
  block — not the `ManaCost` arm the dispatch brief named. `Command::DeclareAttackers` gains the
  two PB-RS2 payment fields, so a hybrid or Phyrexian CR 508.1h attack tax is payable rather than
  rejected.
- **Pre-fix numbers were OBSERVED in both build modes before any line changed**, because plan §2.0
  named a trap purpose-built to produce a plausible false claim. In **debug** — every `cargo test`
  run and all of CI — a manifested Kitchen Finks flip **panics** inside `debug_assert_flattened`
  ("2 hybrid + 0 Phyrexian pip(s) would be paid for free"). The "flips for `{1}`" figure is
  **release-only**, produced by temporarily disabling the guard and reading the pool:
  `{1 colorless, 1 G, 1 W}` → `{0 colorless, 1 G, 1 W}`. **That debug panic is the batch's most
  useful finding**: every test build this project has ever run would have caught the bug, and no
  test ever put a pipped cost through the site.
- **Design (A) shipped on evidence, not taste.** Pips are replicated into the CR 508.1h total and
  the total is flattened **once**. Design (B) — flatten each `cost_per_creature`, then multiply —
  is *rules-wrong* on the Norn's Annex ruling of 2011-06-01 ("that player chooses how to pay each
  cost **individually**"), which (B) structurally cannot express, and it fails in the **quiet**
  direction: it would accept the command and charge a legal-but-not-chosen total.
- **The pip order is copy-major** (`[r1, r2, r1, r2, …]`, never `[r1, r1, …, r2, r2, …]`) so that
  "creature *k*'s pips live at offsets `[k·P, (k+1)·P)`" is true — the only form the ruling or a UI
  can be stated against. Written down in all three required places.
- `unpayable_tax_defenders` → **`x_tax_defenders`**, narrowed to X only; a name asserting
  "unpayable" when hybrid and Phyrexian are now payable is a lying identifier of the class this
  suite keeps re-creating. Message now cites CR 107.3/601.2b and the new **OOS-DX6-1**.
- New read-only **`rules::queries::attack_tax_total`**, because the attack tax is the one payment
  cost a client **cannot** derive — `LegalAction::DeclareAttackers` carries no attacker set. Exactly
  **one** accumulation (`accumulate_attack_tax_total`) serves both it and the validation path.
- **`ManaPool::can_spend`/`spend` stop failing OPEN.** `can_spend` is fail-closed on an unflattened
  residue in every build; `spend` asserts unconditionally. The asymmetry is the argument: a question
  has a truthful conservative answer, an instruction has none, and `spend`'s documented precondition
  is `can_spend`. `Result`-returning signatures were rejected because they **launder an engine bug
  into a rules answer** (every caller would `?` it into `InvalidCommand`) — filed as OOS-DX6-3.
- **PROTOCOL 32 → 33 computed** from the failing gate's own output; the falsifier named in advance
  ("if it passes unchanged, stop") did not occur; closure type count unchanged at 96. **HASH
  confirmed unmoved at 70 by running the gate.** 13 sentinels re-pinned by symbol, then confirmed by
  a full `--workspace --no-fail-fast` run whose residual list was **empty**.
- **0 completeness flips**, pre-committed and held — empty `git diff` over `crates/card-defs`, and a
  coverage regeneration whose body came back byte-identical. Coverage holds at **1,137/1,804 =
  63.0%**; no seeded deck re-dealt, so the play-server pins were never touched.
- Tests 4,066 → **4,099**. clippy / fmt / `tools/check-defs-fmt.sh` (1,804 defs) clean; 210 golden
  scripts green, 0 new skips; benches within noise.

**Hazards for the next worker**:
1. **A batch can silently delete another batch's regression coverage.** PB-DP4's two E1 CR 508.1c
   scoping pins both used a **hybrid** restriction — which stopped being a rejection class the
   moment this batch landed, so E1's fix had lost **all** discriminating power. Found in review,
   verified by reverting E1 and watching them stay green, then moved to `x_count: 1`. **When you
   narrow a rejection class, go find every test that was pinning something else through it.**
2. **The review's HIGH, and the reason to distrust your own new doc comments.** The copy-major
   order-pin test **could not fail** under the permutation it existed to catch — copy- and pip-major
   diverge only when one `add_mana_cost` call has `times > 1` **and** more than one pip, which the
   fixture never produced — while the batch's freshly-written `multiply_mana_cost` doc asserted that
   it could. That is the PB-DX5 "verified: none exist" class, reproduced inside the batch that cites
   it twice. **A test named after an invariant is not evidence that it pins the invariant.** Prove
   discrimination by reverting.
3. **A finding established by reading is a hypothesis.** This batch's reviewer had **no shell** and
   said so per-finding; every finding was re-verified by execution in the fix cycle, and one (the
   TUI site count) was wrong on the numbers. Do not apply an unverified finding.
4. **`tools/tui` still hand-builds `DeclareAttackers` with empty payment vectors** (3 sites), so a
   TUI player facing a pipped attack tax gets a rejection with no way to answer it. Zero exposure
   today (no def carries such a tax) and recorded as **OOS-DX6-5** with in-source comments; the UI
   is M11/M13 work.
5. **`attack_tax_total` returns `None` for an all-X tax**, which is not "no tax". The doc says so
   explicitly and `params.rs` carries the SR-38 note; the real fix needs an X-announcement channel
   (**OOS-DX6-1**).
6. **`OOS-DP4-7` is re-dispositioned, NOT closed.** Do not dedup `add_mana_cost` onto
   `multiply_mana_cost`: the latter is **pip-major**, so the "harmless" dedup would silently
   re-order the tax's pips and re-interpret every `hybrid_choices` vector a client had already
   built — no compile error, and no test failure except the new discriminating fixture.
7. **The SR-31 ratchet gained nothing, deliberately and with the reason checked.**
   `turn_face_up:hybrid` is impossible today because `script_schema.rs`'s `PermanentInitState` has
   no face-down field at all, so the JSON regime cannot build the state;
   `declare_attackers:hybrid` has no honest script because no def produces a pipped tax. Both
   recorded beside `CROSS_VALIDATED_SHAPES` rather than left ambiguous.

**Next**: **PB-DX7** (OOS-DP7-11 + OOS-DP9-13 — the SR-19 gate reports success while checking
nothing; gate integrity, 0 flips, test-only, no wire change). Queue authority:
`memory/primitives/seed-rerank-2026-07-27.md` §4.

---

## Prior Worker Handoff (PB-DX5, preserved for chain context)


## Prior Handoff (wave-7 crash + recovery session, superseded 2026-08-02 — preserved for chain context)

> **ADDENDUM 2026-08-02 (coordinator, post-crash recovery session)** — the crash state below is
> resolved: **`scutemob-173` COLLECTED (merge `51878905`) — M11-LOCAL IS COMPLETE**, on-main
> verified **4,097 / 0** (matches the worker's branch pin exactly); task `done` in ESM.
> **`scutemob-172` (PB-DX6) RESTARTED FRESH** per the agreed recovery: branch reset to plan
> commit `4d367c54`, the unverified 94-file WIP preserved as branch `wip/scutemob-172-crash-20260802`,
> new worker running in the same worktree (1/5 criteria, mid-implement). **The equip finding
> below is FILED**: seed **OOS-M11-10** (`e4b93ac0`, audit §8.1) + repair task `scutemob-179` —
> measured roster is **16 of 17** real equip activations (4 of the 22 grep hits are prose-only,
> 1 is a correct triggered self-attach), 10 of the 16 `Complete` via the `#[default]` derive.
> Also this session: the user's full playtest notes were verified claim-by-claim
> (`memory/playtest-triage-2026-08-02.md`, F1–F10 — **zero engine bugs**, everything is
> simulator/play-server/card-def) and a **corpus-wide mana-cost audit** found **17 wrong costs
> (9 deck-legal `Complete`**, incl. `tyrranax_rex` 3 mana cheap); successor tasks
> **`scutemob-174..181`** created in backlog (pickers, commander cast, mana intelligence,
> invariant fix, additional costs, equip batch, UX polish, field-fidelity gate). S8's merge
> re-baselined 176/177: OOS-M11-8 ({X} auto-tap) is CLOSED in-branch by S8, and S8 already
> rewrote the false-positive `stack_consistency` check (task 177 shrinks to tests + one doc line).
> No dispatch of 174..181 without explicit user approval (standing directive).

**Date**: 2026-08-01..02 (oversight session — parallel two-lane waves; wave 7 lost to a kitty crash; /eot 2026-08-02)
**Workstream**: W6 (PB-DX queue) + M11-local track, run as parallel pairs
**Task**: coordinator chain `scutemob-160..173` (waves 1-6 collected; wave 7 crashed in-flight)

**Completed**:
- **Five waves collected and on-main verified this session** (each pair merged + full workspace run):
  PB-DX2+S3 (**3,988**), PB-DX3+S4 (**4,008**), PB-DX3b+S5 (**4,040** after the seed-pin re-pin
  `b24a9685`), PB-DX4+S6 (**4,048**), PB-DX5+S7 (**4,072**). Main is at `f20823b1` (PB-DX5 merge).
  Detail per batch lives in the entries below and in CLAUDE.md Current State.
- **M11-local S7 SHIPPED** (`scutemob-171`, merge `05849372`) — targeting/combat/X/mode pickers;
  the human can attack, block, and cast targeted/X/modal spells in the browser. CLAUDE.md's
  milestone bullet was a session stale on main and is corrected by this /eot.
- **First human playtest of the browser client happened this session** (frontend `npm install`
  + `npm run build`, `cargo run -p play-server`). It works — and it immediately found a real bug
  (see the equip finding below), which is the whole point of first-playable.
- Stray `/tmp/claude-1000/s8-fuzz-baseline` worktree (left by the S8 worker's fuzz-parity
  comparison) removed; both crashed worktrees WIP-committed so `git status` is clean everywhere.

**Not done / crashed (wave 7 — kitty crash killed both worker sessions)**:
- **`scutemob-172` (PB-DX6, mana-payment flattening)**: died mid-implement. Plan committed
  (`4d367c54`, 1/5 criteria); the 94-file partial implement (mid-PROTOCOL-bump) is preserved as
  WIP `18e89bde` but is **UNVERIFIED — do not build on it**. Agreed recovery: reset branch to
  `4d367c54`, redo implement fresh.
- **`scutemob-173` (M11-local S8, closes the milestone)**: died at **4/5 criteria** with
  substantial verified work committed — scripted-human playthrough (5 seeds), fuzz-parity gate,
  `GET /api/game/report`, Concede + OrderBlockers surfacing, measured test pin **4,092**, seeds
  OOS-M11-7/8/9 handled in-branch. In-flight review-fix edits preserved as WIP `c2013efa`.
  Agreed recovery: fresh worker resumes on the existing commits; only the milestone close-out
  criterion remains. **Collect hazard**: this branch already advances CLAUDE.md past M11-local
  and closes the workstream-state M11 table IN-BRANCH — expect coordination-file conflicts.
- Both tasks remain `in_progress` in ESM with recovery comments attached.

**New finding from the user's playtest (UNFILED — next session should file as an OOS seed)**:
- **Equip is unusable from the browser client, and the root cause is corpus-wide.**
  `accorders_shield.rs` (and ~20 of the 22 `AttachEquipment` defs — `skullclamp`, `lightning_greaves`,
  `swiftfoot_boots`, the swords, etc.) declare the equip `AbilityDefinition::Activated` with
  `targets: vec![]` while the effect reads `EffectTarget::DeclaredTarget { index: 0 }`.
  `abilities.rs:537` has a **legacy special-case** that validates a *volunteered* target
  (`targets.first()`) for `AttachEquipment` but **silently accepts activation with no target** —
  mana is paid, the ability resolves, the attach fizzles. The TUI/old paths volunteered targets;
  S7's browser picker only renders slots from *declared* `TargetRequirement`s → empty → no picker
  → no target submitted → exactly the observed "pay mana, click, nothing happens."
  `crates/simulator` has **zero** equip handling (bots never equip), so no fuzz run ever covered it.
  This is the mirror of OOS-M11-5 (targets accepted without requirements ↔ requirements absent so
  targets never asked). Fix directions to weigh: author a real `TargetRequirement` on the equip
  ability corpus-wide (card-def sweep, likely zero engine lines — the general validation path then
  serves the picker for free); make a no-target `AttachEquipment` activation a **hard rejection**
  (CR 601.2c/702.6a — the ability *requires* a target); and check the two defs that looked
  different (`blade_of_the_bloodchief` is `partial` with equip not even authored; verify
  `blackblade_reforged`).

**Policy change (binding, this session)**:
- **Autonomous wave-chaining RETRACTED by the user.** After wave 1 ("dispatch both") the
  coordinator chained five further waves overnight on the strength of the 2026-07-18
  authorization; the user did not want that. `feedback_queue_autonomous_chaining.md` and the
  MEMORY.md index now record the retraction: **every dispatch — including restarting a crashed
  worker — needs explicit user approval; collect what is in flight, then stop and report.**

**Next session candidates**:
- **Resume `scutemob-173` (S8)** — closest to done; closes M11-local. Fresh worker on the existing
  branch, only the milestone close-out criterion left.
- **Redo `scutemob-172` (PB-DX6)** — reset branch to the plan commit `4d367c54`, fresh implement.
- **File + schedule the equip finding** — could ride PB-DX6's close-out or run as a micro-batch
  (card-def sweep shape, PB-DX3's zero-engine-lines pattern).
- Queue then continues at **PB-DX7** (SR-19 gate integrity) per `seed-rerank-2026-07-27.md` §4.

**Hazards** (carrying forward):
- **kitty remote-control socket loss**: `/tmp/kitty-<pid>` vanished mid-session (likely tmpfiles
  aging at the date rollover) leaving RC unusable while kitty ran; a second detached kitty
  instance (`--listen-on unix:/tmp/kitty-claude-workers`) worked but auto-loads the full session
  config (duplicate tabs). Then kitty itself crashed, killing both wave-7 workers.
- **Two Opus workers + their subagents get heavily API-throttled** — waves 5-7 ran 3-5h wall each;
  single-worker dispatch is materially faster per task.
- **Workers can create throwaway worktrees outside `.worktrees/`** (S8's `/tmp/.../s8-fuzz-baseline`)
  which escape `esm worktree list` hygiene — check `git worktree list` at collect.
- The play-server seed pins re-deal on ANY `Complete`-pool change (precedent `b24a9685`) — now a
  standing coupling between card-def batches and `tools/play-server` tests.

**Commit prefix used**: coordinator `chore:` + `merge:`; worker `scutemob-N:`; crash-preservation `wip:`.

---

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

### 2026-07-26..27 (oversight — PB-DP suite complete + re-rank) [rotated]

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

---

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

