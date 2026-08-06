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
| W6: Primitive + Card Authoring | — | available (PB-DX25b shipped `scutemob-204` 2026-08-06, v3 ranks 1-7b all shipped, `OOS-DX25-3` CLOSED; **PB-DX25c shipped `scutemob-205` 2026-08-06** (rank 7c, INSERTED 2026-08-06 user-approved — closed `OOS-DX25b-3`, the CR 115.7a redirect-legality gap, HASH 73→74), v3 ranks 1-7c all shipped; **next PB-DX26** at rank 8, read adjudication §5 with v3 §4 — DX42a rides DX8, DX42b rank 13) | 2026-08-02 | **PB-OS queue COMPLETE** (OS1..OS11 + OS4b, `scutemob-116..141`). **Rider-seed queue**: RS1..RS4 SHIPPED (`scutemob-143..146`); plan `memory/primitives/rider-seed-triage-2026-07-19.md`, resume at **R5** per its §5 banner (weigh OOS-RS3-1 insert + OOS-RS2-1 rider). **The PB-DP suite now runs FIRST** (user directive 2026-07-26) — queue `docs/audits/decision-point-audit.md` §8, from the decision-point audit (`scutemob-148`). **PB-DP1 SHIPPED** (`scutemob-149`, merged `f7651bb5`): priority after cast/activate/special action goes to the ACTOR per CR 117.3c; 14 Group-A sites + 8 Group-D sites; entry priority guards added to `handle_turn_face_up`/`handle_activate_loyalty_ability`/`handle_level_up_class`; 19 tests + 15 golden scripts reconciled; PROTOCOL 27 / HASH 63 unchanged; 3,721 tests green. Seeds **OOS-DP1-1..4** filed in the audit doc **§8.1** (durable inventory for this suite — not in `primitive-wip.md`, which the next PB overwrites). Suite tasked out `scutemob-150..158` = PB-DP2..DP10. **PB-DP2 SHIPPED** (`scutemob-150`, commit `f902010f`): mulligan content no-op + bottom-to-top, **CR 103.5/103.5c** (the brief's "103.4b" is a stale cite — see the handoff below); **OOS-M11-1 CLOSED**; 4 probes; PROTOCOL 27 / HASH 63 unchanged; tests 3,721 → **3,725**. Seeds **OOS-DP2-1..6** filed in the audit doc **§8.1**. **PB-DP3 SHIPPED** (`scutemob-151`, DP-4 `min_modes` floor, **CR 601.2b/700.2a**): mode announcement is now mandatory — the fix is a **lift** of the range/duplicate/`min_modes`/`max_modes` checks out of the `!modes_chosen.is_empty()` gate, not the audit's prescribed Spree-guard mirror, so it fixed **40** modal defs (3 commands + 37 `min_modes: 1`) rather than the 3 the row predicted, plus the identical activated-ability bypass in `abilities.rs` (audit §4.2). Narrow CR 702.120a escalate exemption; `resolution.rs`'s `vec![0]` fallback **retained** (4 free-cast producers bypass `handle_cast_spell`). PROTOCOL 27 / HASH 63 unchanged; 0 card-def edits; tests 3,725 → **3,747**. Seeds **OOS-DP3-1..9** filed in the audit doc **§8.1**. **PB-DP4 SHIPPED** (`scutemob-152`, merged `799dcc0a`): DP-10 attack tax now debited (colour-correct; restricted mana excluded per CR 106.6; hybrid/Phyrexian/X tax rejected — OOS-DP4-1); DP-11 enforced as a **deadline** (auto-decline unanswered payments at `handle_all_passed`'s stack-empty branch per CR 118.12a — a priority gate would deadlock the driver); 5 Complete defs made right, 0 def edits; **OOS-DP1-1 + OOS-RS3-4 CLOSED**; seeds OOS-DP4-1..13 filed in audit §8.1; PROTOCOL 27 / HASH 63 unchanged; tests 3,747 → **3,781**. **PB-DP5 SHIPPED** (`scutemob-153`, merged `922252f7`): `pending_draws` on `GameState`, `OrderReplacements` routed by applicability; **3 emit sites fixed, not 2** (`draw_card_skipping_dredge` was a third the audit never named); also fixed a CR 614.11a sequence bug (`DrawCards{count:3}` emitted 3 unanswerable prompts and drew 0 — now one prompt, remainder stashed) and a review-found CR 616.1f loop gap in `determine_action`; **HASH 63 → 64** (gate-forced), PROTOCOL 27 unmoved; tests 3,781 → **3,797**; 0 def edits. NOTE: audit premise falsified — **0 of 1,804 defs register a `WouldDraw` replacement**, so card yield is 0; this is an engine-correctness fix + precondition for authoring the WouldDraw family. **PB-DP6 SHIPPED** (`scutemob-154`, merged `d52fe5b6`): intervening-if now evaluated at trigger-queue time across the queue paths (audit §4.8 queue-time row D→A); no wire change (PROTOCOL 27 / HASH 64); tests 3,797 → **3,809**; seeds OOS-DP6-1..10 filed in audit §8.1 (note OOS-DP6-10: the one hazard this batch INTRODUCED — A9 `WasKicked` suppression, wrong-direction, zero corpus exposure today). **No-wire block DP1..DP6 COMPLETE. PB-DP7 SHIPPED** (`scutemob-155`, merged `8f890611`): cleanup-discard is now a blocking player Command (CR 514.1); the **blocking pending-decision mechanism is proven** for DP8/DP9 reuse; **PROTOCOL 27 → 28, HASH 64 → 65** (both gate-computed); tests 3,809 → **3,837**; 2 fix cycles (18 + 6 findings; 2 HIGH: CR 800.4j dead-player entry skipping CR 514.2, out-of-step answer accepted; plus a TUI auto-pass livelock introduced-and-fixed); seeds OOS-DP7-1..12 in audit §8.1 — **OOS-DP7-11 flags that the SR-19 HashInto gate silently skips path-qualified impls** (gate-integrity seed, rankable). **PB-DP8 SHIPPED** (`scutemob-156`, trigger-target choice, CR **603.3d/601.2c/603.3b**): `Command::ChooseTriggerTargets` + `GameEvent::TriggerTargetChoiceRequired` (disc. 130) + `GameState.pending_trigger_targets` suspend `flush_pending_triggers` MID-BATCH and resume it on the controller's answer; the compliant CR 603.3d fallback survives verbatim as the exported `abilities::default_trigger_targets`, which the CALLER submits as a real Command (engine still knows nothing about seat kind). **PROTOCOL 28 → 29, HASH 65 → 66** (all five fingerprints gate-computed; `TriggerTargetOption` + `SpellTarget` enter the wire closure — a genuine type-count change, unlike DP7's). **Roster 77, not the audit's 84 nor the planner's 74** — enumerated from `all_cards()` per SR-36 and printed by a test. **0 card-def edits, 0 completeness flips**, but **2 live-wrong `Complete` cards fixed by accident** (sword_of_sinew_and_steel, elder_deep_fiend): the plan's premise that a permanent-inner `UpToN` slot 'contributed 0 targets' is FALSE — it returned `None` and the caller removed the WHOLE TRIGGER, so those cast/damage triggers had never once reached the stack. CR 601.2c makes zero targets a legal announcement. Second plan gap found and fixed: §4.1 never says who grants the priority the four guards were about to grant, so a resumed game would have had nobody holding priority — added `grant_priority_on_resume` on the entry. Consult set is **4 guards, not the ~20 DP7's row predicted**, because PB-DP1 moved priority assignment ahead of the flush (all 30 `check_and_flush_triggers` sites verified to need none). Also fixed `local_game.rs`'s latent variant-blindness (hard-coded `DecisionKind::CleanupDiscard`) — now compile-forced. tests 3,837 → **3,858**; 53+1 sentinels re-pinned across 44 files; 1 golden script corrected with CR justification. Seeds **OOS-DP8-1..10** in audit §8.1 (DP8-9/DP8-10 are new relative to the plan). **OOS-M11-4 CLOSED.** OOS-DP3-4 deliberately NOT bundled — ranked as **PB-DP8b** (OOS-DP8-7). **PB-DP9 SHIPPED** (`scutemob-157`, search/scry/surveil, CR **608.2d / 701.23a / 701.22a / 701.25a**): the engine's **first resolution-time decision channel**. `GameState.pending_effect_choice` + `GameEvent::EffectChoiceRequired` (disc. 131) → `Command::AnswerEffectChoice`, backed by an **abort-and-replay** continuation, NOT the "resumable effect-list cursor on the stack object" `pb-plan-DP7.md` §1.6 and audit §8 both prescribed — that is **impossible**, because `resolve_top_of_stack` POPS the stack object before any effect runs. Instead: clone at entry, an effect that needs an unanswered choice records the question and returns, the wrapper restores the clone **wholesale** and emits one event, the answer is banked on `GameState` and the resolution re-runs **from the top**, retracing the identical deterministic path. Consequences worth carrying: no continuation data structure at all (`Sequence`/`Conditional`/`ForEach`/`Repeat` need zero machinery — the replay re-executes them); the re-entrancy audit is **3 units, not 20** (15 of 17 production `execute_effect` callers are inside `resolve_top_of_stack` itself, one is gated by CR 605.4a, one is provably unreachable); **PB-DP8's "a guard that returns early inherits a debt" bug class does not exist here** because a total restore skipped nothing; and "the suspended object leaves the stack" is structurally unreachable rather than a live hazard. **ONE `Command` for all three effects** (CR 608.2d is one rule; 701.22a/23a/25a are three instances) — so one gate entry, one `LegalAction`, one `DecisionKind`, one harness action string, which **corrects OOS-DP8-14's prediction of three**. **PROTOCOL 30 → 31, HASH 67 → 68** (all gate-computed, histories append-only, 44 sentinel files re-pinned via the SYMBOL grep). **Roster 69 / 16 / 7, not the audit's 74 / 16 / 8** — enumerated from `all_cards()` with a RECURSIVE `Effect`-tree walk (a flat scan undercounts). **0 def edits, 0 flips.** Three in-scope correctness fixes beyond agency: CR **701.22b** (`Scry 0` was emitting `Scried{count:0}`; the surveil arm had the mirror guard, the scry arm did not), CR **400.7** (scry-to-bottom RENUMBERED every scried card and consumed `timestamp_counter`, the shuffle seed source — now `Zone::reposition_within`; sweep seeded as OOS-DP9-11), and CR **701.23d** (a quantity-only search with one candidate is determined and asks nothing). Two deliberate deviations, both argued in source and pinned by tests: **the scry/surveil defaults FLIP to the identity** (search keeps its lowest-id default byte-for-byte), and **the three new fields are EXCLUDED from `loop_detection.rs`'s fingerprint** unlike DP7's and DP8's, because they grow between replays of one resolution and could mask a CR 726 loop — recorded as **obligation (7)** on the `BlockingDecision` doc block, the first evidence that list generalises. `GameEvent::private_to()` now exists (OOS-DP8-6's declaration half; a declaration, NOT an enforcement point — nothing consumes it until M10). Benches measured against `48353a36`: `full_turn_4p` 253 → 229 µs, **no regression** from the per-resolution clone. Fallout 25 unit tests + 1 golden script, every repair CR-justified. tests 3,878 → **3,905**; seeds **OOS-DP9-1..12** in audit §8.1 (rank **OOS-DP9-3** first — `SearchLibrary` finds exactly one card, ~7 partial defs, zero new plumbing on this machinery). Merged `d65e7f1e`; on-main verified **3,910 / 0** (5 more than the branch pin — post-merge count). **PB-DP10 SHIPPED** (`scutemob-158`, decision-gate widening, **test-only** — and it **CLOSES THE PB-DP SUITE**): two new files under `crates/engine/tests/core/` — `decision_site_walk.rs` (the canonical serde walk + `ROWS`, all 22 decision sites of audit §3.1 classified **4 SERVED / 15 AUTO-CHOSEN / 2 GATED / 1 NO-DECISION**, each with the engine site that was *read* to establish the class) and `decision_gate.rs` (`BASELINE`, 97 name-keyed entries with exact row sets, + 18 tests). **The headline is a gate-integrity finding, not a feature**: every serde walk in this codebase before now (`effect_choose_gate.rs`, `pb_rs1_roster_sweep.rs`, `pb_dp9_effect_choice.rs::roster`) matched **object keys only** and is therefore blind to a **unit** `Effect` variant — `serde_json::to_value(Effect::Proliferate)` is `Value::String("Proliferate")` — so a verbatim reuse would have reported **0** for Proliferate's 25 `Complete` defs *while looking green*, the exact OOS-DP7-11 failure mode. Fixed with a two-shape walk + a `PROSE_FIELDS` denylist, pinned in both directions against the legacy walk (T2/T3). Measured: all-rows union **267** (the audit's 277 analogue), still-auto union **97**, live denominator 1,139/1,804. **Fail-closed proven end-to-end on a real def**, not just synthetically: adding `Effect::Proliferate` to `lightning_bolt.rs` reddened **two** tests naming the card, the row, the CR and the engine site; restored → green. Two hand-maintained zeros (`AddManaFilterChoice`, `TheRingTemptsYou`) that **nothing** was holding became machine-checked (the SR-33 gate bars a *different* key). **PROTOCOL 31 / HASH 68 unmoved; engine + card-types + card-defs diff vs main EMPTY; 0 def edits, 0 flips.** Review 2 HIGH / 6 MEDIUM / 6 LOW, **all 14 applied** — the HIGHs are worth carrying: (1) `BASELINE` was populated **mechanically** and the plan's class-B/class-D triage was never done, so a spot-check found two class-D defs already inside the frozen baseline (Smuggler's Copter's "you **may** draw" authored as an unconditional `Sequence`; Shambling Ghast with a permanent `-1/-1` counter, an `oracle_text` saying "enters" against a `WhenDies` trigger, and a `Decayed` keyword the printed card does not have) — seeded as **OOS-DP10-8**, not demoted; (2) **the gate can only see a decision the DSL ENCODED**, and that blind class is strictly *worse* than the one it records (**OOS-DP10-9**, and the instrument for it is an oracle-text-vs-DSL cross-check, not a variant walk). tests 3,910 → **3,928**; seeds **OOS-DP10-1..11** in audit §8.1; **closes OOS-DP7-7** (the 277-def re-derivation is now computed, printed and ratcheted every run). Audit §8 now carries a suite-COMPLETE banner and §10 an honest 3-of-8 mechanization ledger. Merged `16ffcfd0`, on-main verified 3,928/0; suite retrospective in the audit doc §8. **Next: re-rank RS5..RS11 against the unranked seeds** (OOS-DP9-3 was the previous first pick; OOS-DP10-8/-9 are new and OOS-DP10-6 is the successor queue's ranked input). **Seed re-rank SHIPPED** (`scutemob-159`) — successor queue `memory/primitives/seed-rerank-2026-07-27.md` §4, PB-DX1..DX18. **PB-DX1 SHIPPED** (`scutemob-160`): OOS-DP6-1 + riders CLOSED; PROTOCOL 31→32 / HASH 68→69; tests 3,928→3,945. **PB-DX2 SHIPPED** (`scutemob-162`): OOS-DP5-7 + OOS-DP7-2 + riders OOS-DP2-1/OOS-DP9-14 all CLOSED — see the Last Handoff section below for detail; PROTOCOL 32 / HASH 69 unmoved; tests 3,945→3,971 (this worktree's own baseline was 3,955 after the intervening M11-local S2 merge, so the batch's own delta is +16). **Fix cycle same day**: review found the implement-phase "fold guard" was a HIGH (unbounded cross-turn accumulation, cashable out-of-priority) — replaced with a discharge design that also closes OOS-DX2-3 as a side effect; 7 doc-vs-code MEDIUMs + 1 coverage-hole MEDIUM + 7 LOWs all applied; PROTOCOL 32 / HASH 69 still unmoved; tests 3,971→**3,974**. **PB-DX3 SHIPPED** (`scutemob-164`, 2026-08-01): **OOS-DP6-3 CLOSED** — `garruks_uprising` + `inventors_fair` both `partial` → **`Complete`**, coverage 1,140 → **1,142** (63.2% → 63.3%), tests 3,988 → **3,998** (+10 probes), **0 engine lines** (empty `git diff` over the whole of `crates/engine/src` *and* `crates/card-types/src`, not just the wire files) and PROTOCOL 32 / HASH 69 unmoved. Review 0 HIGH / 1 MEDIUM / 5 LOW, all applied. **Three things the queue row did not contain, in ascending order of how much they matter.** (1) `inventors_fair`'s upkeep trigger **did not exist at all** — the seed and both blocker notes read as though it were present but ungated, so the batch had to *author* the ability. (2) The runtime `InterveningIf` enum both notes name now has **three** variants, not the two they cite: PB-DX1 added `InterveningIf::CardDef` two batches earlier. The stale notes were stale twice over, and this queue introduced the second staleness itself. (3) **The MEDIUM was the batch reproducing its own subject.** The test module recorded a pre-fix observation for T1 ("the hand count was 1") that **could not have been observed** against T1's own fixture, which had no library object — and an empty-library draw sets `has_lost` (`replacement.rs:1035-1049`) rather than incrementing the hand, so the companion assertion passed whether or not the bug fired. Fixed by giving T1 a real library card and **re-running the pre-fix scenario empirically** (reverting `intervening_if` to `None` and reading the numbers), not by repairing the prose; the same standard was then applied to T3/T5/T6/T7/T8, all of which held. The original claim was right — it had simply never been checked against a fixture where the number meant anything, and that distinction is the whole lesson. `reveal: true` on `Effect::SearchLibrary` is inert (pre-existing **OOS-DP9-9**) and now carries an in-def comment saying so rather than being silently covered by the `Complete` marker. **New seed OOS-DX3-1** (audit §8.1): six more defs sit in the same `pb-plan-DP6.md:395` stale-blocker bucket and **`jadar_ghoulcaller_of_nephalia` is a live-wrong `Complete` def** — `intervening_if: None`, so it makes a 2/2 Zombie **every** end step unconditionally, and its stored `oracle_text` names a token-name filter the printed card never had (MCP: the real text is "if you control no creatures with decayed"). Expressible today as `Not(YouControlNOrMoreWithFilter{count:1, filter: Creature + has_keywords[Decayed]})`; the fix must also reconcile golden script `combat/191`. `ophiomancer` (`partial`, its own note already says "Blocker stale") and `dwynen_s_elite` (`inert`) are two more flips in the same shape. **Next: PB-DX4** (OOS-DP10-8, the 97-entry `BASELINE` triage) — but consider inserting OOS-DX3-1's Jadar half first: live-wrong `Complete`, card-def only.  **PB-DX3b SHIPPED** (`scutemob-166`, 2026-08-01 — a **queue insert ahead of PB-DX4**, taken on the post-DX3 banner's own recommendation): **OOS-DX3-1 CLOSED**. All **seven** remaining defs of the `pb-plan-DP6.md:395` stale-blocker bucket dispositioned explicitly — 4 fixed, 3 deferred with blockers re-affirmed against the *current* `Condition` enum rather than copied forward. `jadar_ghoulcaller_of_nephalia` stays `Complete` and is now CR 603.4-gated; **its stored `oracle_text` was wrong, not merely its blocker note** (the field said "tokens named Shambling Ghast"; MCP says "creatures with decayed"), so the note had been declaring a DSL gap for a filter the card never had — a distinct failure mode from PB-DX3's stale-note class. `ophiomancer` `partial` → `Complete` (`has_subtype: Snake` alone, deliberately not `ControlCreatureWithSubtype`, whose arm hard-requires `CardType::Creature`). `dwynen_s_elite` `inert` → `Complete`, ability **authored from nothing** — the `inventors_fair` shape recurring; expect it. **The seed itself mis-dispositioned a second live-wrong `Complete`**: `emeria_the_sky_ruin` declares no `completeness` field, so it was `Complete` by the `#[default]` derive and reanimated every upkeep regardless of Plains count — the `aurelia_the_warleader` trap from PB-DX1, hit a second time in three batches by a different route. Gated, given an **explicit** `partial` for the DSL-inexpressible "you may" (OOS-DP10-8 class, falsifier search actually run), and a spurious `Legendary` supertype removed (MCP type line is `Land`). **2 flips up, 1 honest flip down — net coverage 1,142 → 1,143, +1 not +3**; 0 engine lines (empty diff over all of `crates/engine/src` + `crates/card-types/src`); PROTOCOL 32 / HASH 69 unmoved; tests 4,008 → **4,022** (this branch's merge base is 4,008, not the 3,998 DX3 pin — `scutemob-165` merged in between). Golden script `combat/191` reconciled by **strengthening** (it had never asserted the Zombie token and passed either way). Review 0 HIGH / 5 MEDIUM / 7 LOW, all 12 applied. New seed **OOS-DX3b-1** (`guardian_project`'s `is_nontoken` half is authorable today; its name-uniqueness half is not, so it stays `known_wrong`). **Durable**: `#[default] Completeness::Complete` is now a twice-demonstrated silent-defect generator — "which defs never declare a marker at all?" is a cheap corpus-wide question nobody has asked. **PB-DX4 SHIPPED** (`scutemob-168`, 2026-08-01): **OOS-DP10-8 CLOSED**, and **OOS-M11-6 closed incidentally**. All 97 `BASELINE` entries read against MCP printed text, roster parsed out of the const array itself (97 → 97 distinct names → 97 unique def files) rather than taken from prose, because this suite has published a wrong roster three times. **Split 84 class-B / 13 class-D** — PB-DP10's 2-of-5 spot-check overstated the D rate ~5x and its own "very noisy sample" caution was right; the queue row's "0 flips" estimate was wrong the other way, since 5 of the 11 had to be demoted. **5 repaired, still `Complete`**: `metastatic_evangel` (4 defects: `{2}{W}`→`{1}{W}`, missing `Human`, P/T transposed 1/3→3/1, and a **stale** in-def note claiming `is_token` is ignored on the ETB path — PB-AC0 had made that false), `grisly_salvage` + `satyr_wayfinder` (`RevealAndRoute` routes ALL matches → `LookAtTopThenPlace{optional:true}`; printed says "**a** card", "you **may**"), `sword_of_truth_and_justice` (bare `TargetCreature` → `controller: You`), `radstorm` (`{2}{U}`→`{3}{U}`). **6 demoted with oracle citations**: `smugglers_copter` → `known_wrong` (20th DP-12 instance; the other 19 already were, so the marker was the defect), `contaminant_grafter` / `grateful_apparition` / `thrasios_triton_hero` → `partial`, and `shambling_ghast` → `partial` **for a defect the fix surfaced** — its three named deviations (phantom `Decayed`, permanent `MinusOneMinusOne` for a printed "until end of turn", `oracle_text` saying "enters" against `WhenDies`) were all FIXED, and the marker is for a fourth: the mode-1 target is flat, so taking the Treasure mode still needs an opponent creature (CR 603.3d). **`mode_targets` is honoured only on the CASTING path** — nothing on the trigger path reads it — so the obvious repair would have DROPPED the requirement rather than scoped it (**OOS-DX4-2**; `hullbreaker_horror` is a second member). **1 left `Complete` deliberately**: `staff_of_compleation`'s "target permanent you own" as `TargetController::You`, allowlisted to match the shipped `nether_traitor` decision for the identical owner-vs-controller class (**OOS-DX4-1**) rather than reporting a corpus class as two cards. **OOS-M11-6 found by accident**: demoting `thrasios_triton_hero` — a legendary creature, i.e. a member of `random_deck`'s own commander pool — re-dealt every seeded deck in the workspace and landed seed 9001 on Rograkh, the corpus's ONLY colourless `Complete` legendary creature (1 of 91). Fixed as that seed preferred (pad from the identity-legal colourless pool; measured 40 colourless lands + 82 nonlands = 122 singletons vs 99 needed), **both** Forest fallbacks removed. The bigger half: the fuzzer feeds `random_deck` straight to `GameStateBuilder` with no validation, so it had been silently **playing** CR 903.5c-illegal decks. Six fixtures across two crates broke; the two play-server rebuild tests lost their only failure trigger exactly as their own maintenance note predicted and now use a sentinel **seed** (a first attempt used a process-global flag that raced with every other test POSTing `/api/game` — green under `-p`, red under `--workspace`, twice). Golden script `baseline/112` **retired**: it tested Decayed on a card that does not have it, citing the card *def* as its authority — a provenance failure. CR 702.147a keeps 12 unit tests; golden-level gap filed as **OOS-DX4-3**. Coverage 1,143 → **1,137** (63.0%), tests 4,040 → **4,048**, `BASELINE` 97 → **91** (moved twice inside the batch, 97→93→92, which is why it was read off the gate not computed), deviation floor 661 → **667**, DP8 roster 76 → **74**, `scry` 16 → **15** — each re-measured against `all_cards()`. **0 engine lines** (empty diff over `crates/engine/src` *and* `crates/card-types/src`), PROTOCOL 32 / HASH 69 unmoved. **PB-DX3b's `#[default]` question answered and bigger than expected: 966 of 1,804 def files never mention `completeness` at all (970 before this batch)** — a clear majority of the `Complete` population, and **eleven of the thirteen** class-D defs were in it; now ratcheted in the growth direction. Durable record `memory/primitives/pb-dx4-baseline-triage.md` (per-def citations + an explicit statement of what the triage does NOT establish: it is a dated claim, it cannot see a decision the DSL never encoded — OOS-DP10-9 stands — and 97 of 1,143 is not a sample the rest can be inferred from). Seeds **OOS-DX4-1..6**. **PB-DX5 SHIPPED** (`scutemob-170`, 2026-08-01): **OOS-OS7-2 CLOSED — CR 611.2c**, `ContinuousEffect` gains `affected_set: Option<OrdSet<ObjectId>>`, populated only by `Effect::ApplyContinuousEffect` via the new `rules::layers::snapshot_affected_set` (never elsewhere) and read as pure membership by `effect_applies_to`; `None` means a static ability (CR 611.3a, unchanged, still live-evaluated). **The dispatch row's roster ("9 defs, 7 `Complete`") was wrong twice over — the sixth consecutive batch in this suite whose published roster was wrong before it started.** `all_cards()`, enumerated fresh: **116** defs generate a resolution-time continuous effect at all; **38** use a mass filter (not 9); and even the premise-verification step's own corrected figure (37, 28 `Complete`) was off by one against its OWN table, which already listed 38 rows summing to 29 — an uncaught arithmetic slip nobody re-added. **Final measured: 38 mass-filter defs, 29 `Complete`, 8 `partial`, 1 `known_wrong`**, from a new self-re-measuring test (`pb_dx5_mass_filter_roster_by_completeness`) rather than a pinned count. Mechanical backfill of `affected_set: None` at all 180 pre-existing `ContinuousEffect` construction sites (49 files, compiler-driven, zero manual judgement calls — every site is either a static registration or a `SingleObject` effect, and `None` is the RULE at the former, a no-op at the latter). **HASH 69 → 70** (mandatory, gate-forced); **PROTOCOL confirmed unmoved at 32** by actually running `--test core protocol_schema`, not assumed (`ContinuousEffect` is outside the SR-8 wire closure). **Yield 0 completeness flips, exactly as pre-committed** — a pure correctness fix for defs already `Complete`; coverage stays 1,137/1,804 (63.0%, byte-identical regen). **Existing-test repair, exactly as flagged a hazard in advance**: `pb_ac3_dynamic_pt_counts.rs`'s `test_set_both_dynamic_locked_at_resolution` was asserting the CR 611.2c bug this batch fixes ("a creature entering after resolution still gets the locked-in X=3") and had been passing; inverted with a CR cite and renamed, not weakened. Every "fails before" claim in the new 14-test probe module was OBSERVED (read-site membership check reverted, actual value recorded, restored) — which caught the runner's OWN first-draft T3 (control-change retention) using the buffed creature as its own effect source, masking the very divergence it claimed to test; fixed with a separate, never-moved source. T11 (zone-scope shortcut vs. brute force) lives as an in-source `#[cfg(test)]` unit test in `rules/layers.rs`, not the integration file — `snapshot_affected_set`/`effect_applies_to_object`/`candidate_ids_for_filter` are all `pub(crate)`. Benchmarks all within ~1% of the merge base; `board_wipe_4p` (flagged most likely to move) measured slightly *faster*. Golden corpus unaffected — Final Showdown's script exercises mode 2 (DestroyAll), not the `AllCreatures|Ability` mode 0 the roster found (a pre-existing, documented DSL-gap omission in the script itself). Six new seeds **OOS-DX5-1..5** + a checked non-finding **OOS-DX5-6** (Mirror Entity is the one Layer ≤4 mass-filter def; unaffected today — nothing in the roster writes `CardType::Creature` via a Layer-4 modification). Tests 4,048 → **4,064**. **Same-day fix cycle** (review `pb-review-DX5.md`, 0 HIGH / 6 MEDIUM / 6 LOW, all 12 applied, none changing observable behaviour): the test-count arithmetic above was itself off by one (the roster file has two `#[test]`s, not one — true implement-phase total was **4,065**, and the fix cycle's own +1 new test (T15) makes the re-run total **4,066**); OOS-DX5-6's "checked non-finding" was FALSE — a real, reachable, CR-correct divergence exists (animate Inkmoth Nexus + Mirror Entity's `AddAllCreatureTypes`), now pinned by T15 and corrected in the seed doc; the fix was found to close a SECOND, larger pre-existing defect (every source-relative mass filter on an instant/sorcery applied to nobody once the spell resolved, CR 400.7) — confirmed empirically, filed as **OOS-DX5-7 (CLOSED as a side effect)**; OOS-DX5-1 widened to name three read sites that ignore `affected_set`; a stale note in `pb_os7_defending_player_continuous_filter.rs` that declared this batch's own seed a live limitation was corrected; T11's fixture was genuinely enriched (phased-out permanent, real `AttachedCreature` match, subtype filter); a vacuous `debug_assert!` hardened; the non-fixed-window-duration question (plan §3 Q4) is now a measured, standing assertion (zero corpus members) rather than a spot check. HASH 70 / PROTOCOL 32 re-confirmed unmoved by re-running both schema gates. **Next: PB-DX6** (OOS-RS2-1 + OOS-DP4-1 — the two mana-cost payment sites PB-RS2 left unflattened; `handle_turn_face_up` pays a raw `def.mana_cost` and `can_spend`'s residue guard is `debug_assert`-only, so in release every hybrid/Phyrexian pip in a `TurnFaceUpMethod::ManaCost` flip is FREE — `kitchen_finks` is `Complete` with two `{G/W}` pips; and `Command::DeclareAttackers` has no `hybrid_choices`/`phyrexian_life_payments` fields at all. One PROTOCOL bump for the batch; also make the residue guard fail loud). Full brief in `memory/primitives/seed-rerank-2026-07-27.md` §"Dispatch briefs", whose stale "Next dispatch: PB-DX5" banner was struck through at PB-DX5 close rather than left to become the N4 re-dispatch hazard a fourth time. Two PB-DX5 residuals worth weighing as inserts first: the probe module rebuilds Craterhoof/Jitte/Mirror Entity encodings by hand rather than instantiating the corpus defs, so def-vs-probe drift would go unnoticed (the `all_cards()` roster test and the fix's filter-agnosticism mitigate but do not close it); and `rules/face.rs`'s deregistration sites match on `source == obj_id && filter == resolved_filter`, which cannot distinguish a static registration from a resolution-generated effect sharing both — `affected_set.is_none()` is now a ready-made discriminator, a different class from the three read sites OOS-DX5-1 names and a candidate to fold into it. | **PB-DX6 SHIPPED** (`scutemob-172`, 2026-08-02): **OOS-RS2-1 + OOS-DP4-1 both CLOSED** — the last two unflattened mana-cost payment sites. `handle_turn_face_up` flattens in **all three** `TurnFaceUpMethod` arms (the brief named only `ManaCost`; all three share one payment block), and `Command::DeclareAttackers` gains the two PB-RS2 payment fields so a hybrid or Phyrexian CR 508.1h attack tax is **payable** rather than rejected — pips replicated **copy-major** into the total, total flattened once, because design (B) (flatten-then-multiply) is *rules-wrong* on the Norn's Annex ruling that each cost is chosen **individually**. `unpayable_tax_defenders` → `x_tax_defenders`, narrowed to X only. New read-only `rules::queries::attack_tax_total` — the attack tax is the one payment cost a client cannot derive — with **exactly one** shared accumulation. `ManaPool::can_spend` is now fail-**closed** on an unflattened residue in every build and `spend` asserts unconditionally: the guard PB-RS2's own review described as firing "NEVER" in release was failing **open**, i.e. silently **undercharging**. **PROTOCOL 32 → 33 computed** from the gate's own output (falsifier named in advance did not occur; closure type count unchanged at 96); **HASH confirmed unmoved at 70 by running the gate**. **0 completeness flips**, pre-committed and held (empty `git diff` over `crates/card-defs`), coverage holds at **1,137/1,804 = 63.0%**, tests 4,066 → **4,099**. Review **1 HIGH / 8 MEDIUM / 6 LOW, all 15 applied** and each re-verified by execution first, because the reviewer had no shell. The HIGH: the copy-major order-pin test **could not fail** under the permutation it existed to catch, while the batch's own freshly-written doc claimed it could — the PB-DX5 "verified: none exist" class reproduced inside the batch citing it. Second finding: this batch silently **removed** PB-DP4's E1 CR 508.1c regression coverage (both pins used a hybrid restriction, no longer a rejection class) — verified by reverting E1, then moved to `x_count: 1`. Seeds **OOS-DX6-1..5**; **OOS-DP4-7 re-dispositioned, not closed**. **QUEUE RE-RANKED 2026-08-02 (`scutemob-182`) — the authoritative queue is now `memory/primitives/seed-rerank-2026-08-02.md` §4 (v3); `seed-rerank-2026-07-27.md` §4 is SUPERSEDED. NEXT IS PB-DX19 (OOS-SIM2-6 + OOS-SIM2-5), NOT PB-DX7** — PB-DX7 survives unchanged at rank 9, displaced by eight entries that are live-wrong on deck-legal `Complete` cards or, in PB-DX19's case, a hard process abort. See the `scutemob-182` handoff below.

## Worker Handoff (PB-DX25c, `scutemob-205`)

**Stage 2 SHIPPED.** Closes `OOS-DX25b-3` (CR 115.7a's "another LEGAL target" at
redirect time). Stage 1 (production code, `cf89a213`) added `StackObject.
target_requirements` (hashed) + `rules::retarget::plan_target_change`, delegating
the whole redirect decision to `casting::validate_targets_inner`. Stage 2: fixed
the 6 fixtures stage 1 left red (real `TargetRequirement`s now recorded), inverted
`t9` + added `t9b`, wrote 9 new probes (`pb_dx25c_retarget_legality.rs`) + 1 bot-path
probe (`pb_dx25c_bot_retarget_is_legal.rs`, S1 only — S2 measured 0/30 fuzz-shaped
games reaching `Effect::ChangeTargets` at 80 turns, so it is NOT shipped, per the
plan's own instruction) + 5 roster/gate tests (`pb_dx25c_retarget_roster.rs`, R1-R5)
+ 1 in-source R6 test in `retarget.rs`, HASH 73 → 74 (gate-computed), `bare_lookup_
ratchet` ceiling 110 → 108, both card-def pointer comments updated (comment-only).
**Tests 4,469 → 4,491 (+22)**; PROTOCOL 35 unmoved; coverage unmoved 1,133/1,803 =
62.8% (proven by regeneration, reverted before commit). Full revert matrix (19
rows, all executed): 15 discriminate (12 exactly as predicted, 3 with a corrected
discriminator — V6/V8/V19); **4 honestly undiscriminated by the full workspace
suite** — V3 and V13 (both predicted-possible by the plan), V7 and V9 (NOT
predicted — `retarget_candidates`'s own `has_conceded` filter is shadowed by
`validate_mapped_targets`'s independent downstream check; the chooser-first
preference is shadowed by a coincidental fixture where the chooser is also first
in seat order). Two structural findings surfaced only by executing tests:
`TargetSpellWithSingleTarget`/`TargetSpellOrAbilityWithSingleTarget` cannot
observe the ACTIVELY-RESOLVING spell as a candidate (its own `StackObject` entry
is popped before its effect runs); `StubProvider`'s offer layer reads `obj.
characteristics.mana_cost` directly, a third instance of the "`ObjectSpec::card()`
is naked" gotcha, in a place `gotchas-infra.md` doesn't mention yet. Filed
**OOS-DX25c-1..4**; closed **OOS-DX25b-3** (4 corrections to its own claims — see
the audit doc). Full measurements, revert matrix results and the R4 non-vacuity-
floor anomaly diagnosis: `memory/primitives/pb-DX25c-execution-notes.md`.

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
    **CLOSED 2026-08-04 by PB-DX21** (`scutemob-200`) — both halves: the engine guard
    (`GameStateError::AlreadyDeclaredAttackers`) and the offer (`legal_actions.rs:878`).

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

## Coordinator note (scutemob-186 collect, 2026-08-02)

The adjudication task was conflict-barred from coordination files; its dispositions live in
`docs/audits/mtg-characteristics-recursion-adjudication.md` (§5 queue insertion PB-DX42a/b,
§6 seeds OOS-ADJ-1..7). The v3 queue memo's §4 table has NOT been re-rowed — the next dispatcher
must read adjudication §5 alongside it. OOS-ADJ-3 warns `OOS-DX19-2`'s "613.8b fixpoint" framing
would make a worker build the wrong thing — re-word at dispatch. OOS-ADJ-7 (blood_moon strips
Artifact card type) rides PB-DX27.

## Worker Handoff (PB-DX25b, `scutemob-204`) — a spell you can target is a spell you can retarget

**What was wrong.** `casting.rs::validate_object_satisfies_requirement` opens by resolving the
announced target id through `state.objects` — so the id must name the **CARD** sitting in
`ZoneId::Stack` — and then, for `TargetSpellWithSingleTarget` and
`TargetSpellOrAbilityWithSingleTarget`, looked the stack object up by `so.id == id`, which is a
**stack-entry** id. `handle_cast_spell` mints the two one line apart (`move_object_to_zone(card,
ZoneId::Stack)` then `state.next_object_id()`), both from the one monotone `timestamp_counter`, so
an id lives in exactly one namespace and the comparison **type-checks while being unsatisfiable**.
`is_spell` was always false and `target_count` always 0: `misdirection` and `bolt_bend` are
`Complete`, deck-legal, and could never resolve a legal target. Same defect as `OOS-SIM3-5`, two
functions apart.

**The brief was short by three sites, and a fix obeying it would have been strictly worse than
HEAD.** The dispatch row said "validation-site only, no stored state". But `Effect::ChangeTargets`
— the effect *both* cards use — resolves the same announced id and matches it against stack-entry
ids at three more places. Repairing only the validator produces a cast that passes announcement,
**takes the mana**, and then hits `continue` at resolution: a silent no-op in place of an honest
refusal. `Effect::CopySpellOnStack` is a fourth site (latent — corpus population zero, re-measured
by enumeration) and `Effect::CounterSpell` had open-coded the *correct* rule as a fifth. The
authoritative id space is settled by the offer layer, not by argument:
`queries::legal_targets_per_slot` enumerates object candidates from `state.objects()` alone, so a
card id is the only thing a player or bot can ever announce.

**The fix is structural.** One `state::stack_registry::stack_index_for_announced_target`, beside
PB-DX25's `card_in_stack_zone`, encoding the rule ONCE —
`so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind) == Some(announced))` — and
consumed by all five sites. Two guards that look incidental and are not:
* **`!so.is_copy` is load-bearing twice.** CR 707.10: `copy.rs` clones the original's `kind`
  wholesale, so a copy's `source_object` names the ORIGINAL's card. Without the guard one card id
  matches the original *and* every copy of it and `position()` silently returns whichever comes
  first. It is also what stops the CR 702.99c cipher-copy exile leak PB-DX25 documented.
* **`is_spell` is KEPT although it is now production-unreachable.** After the repair the helper can
  return a non-spell only via the direct-id clause, which needs an id that is simultaneously a
  `state.objects` row and a stack-entry id — unreachable. So the two requirements are
  behaviourally identical on the real path today. That is the visible shadow of `OOS-DX25b-1`, not
  a new defect, and deleting the guard becomes CR-wrong the day that seed closes. The collapsed-id
  fixture is deliberately kept for the same reason: it is now the only configuration that isolates
  the guard, and its doc says so.

**`stack_card_of` stays uncoupled.** The simulator's exhaustive re-implementation
(`crates/simulator/src/invariants.rs`) is untouched — `check_stack_consistency` exists to catch the
engine getting this classification wrong, and a verifier that reads the engine's own answer goes
silent on exactly the defect it was written for. Zero simulator lines.

**The tests were green while testing a fiction.** `casting.rs`'s `make_test_stack_spell` built
`StackObject { id, kind: Spell { source_object: id } }` — collapsing the two id spaces into a
configuration **no real cast can produce**. Three test files carried that fixture (the two in-src
`casting.rs` tests, `pb_ef11_spell_single_target.rs`, and `pb_ef11`'s Misdirection probe, which
announced a stack-entry id straight into `execute_effect`). All repaired to mint distinct ids, each
proven to discriminate by executed mutation. **`tests/rules/copy_redirect.rs` still has eight more
of the same shape** — including `test_bolt_bend_redirects_single_target_spell`, named after a card
this batch repaired and proving nothing about it. Not repaired (no coverage hole: the real probes
catch the regression), but now disclosed rather than left as a trap.

**Three things worth carrying forward.**
1. **The review's only HIGH was a plan deliverable the implement phase silently dropped** — plan §8
   R2 option (iii) required a wrong-way-round probe for the CR 115.7a redirect, and the execution
   notes then recorded that "the plan scoped this as future work". The plan scoped the **fix** as
   future work and the **probe** as this batch's. Both the probe and the correction shipped.
2. **The reviewer defeated the new R5 gate three ways** — clause-order reversal, a preceding
   statement's `;` landing inside its 150-byte backward window, and a brand-new bare `so.id ==`
   site carrying no `card_in_stack_zone` at all so R5 never looks. The first two are now caught; the
   third is a **permanent structural residual** and is stated as such in the gate's own doc. A gate
   whose doc overclaims its reach is this batch's own subject matter.
3. **The census is complete, and that was verified by the inverse method**, not by agreeing with
   the plan: every `stack_objects` id-comparison across the 18 engine files that mention them,
   classified by provenance. Exactly five raw `so.id ==` comparisons survive outside
   `stack_registry.rs`, every one correctly classified. **No sixth site.** Nothing delegates into
   the `ChangeTargets`/`CopySpellOnStack` arms the way `CounterUnlessPays` delegates into
   `CounterSpell`, so R4's two-arm scope is not blind in the PB-DX25 way.

**Numbers.** Tests **4,452 → 4,469 (+17)**, residual empty, `--workspace --no-fail-fast` to a file.
PROTOCOL **35** / HASH **73** gate-executed and unmoved (fingerprint gate too, not just the version
sentinel). Coverage unmoved **1,133/1,803 = 62.8%**, proven by regeneration; all four card-def edits
comment-only, verified per-line. `crates/simulator/`, `crates/view-model/`, `crates/card-types/`,
`tools/` diffs all empty. Review 1 HIGH / 5 MEDIUM / 6 LOW, all 12 taken.

**Two coordinator scope calls, recorded so they are not re-litigated.** (1) `bolt_bend` **stays
`Complete`** — `completeness` describes the def's fidelity to the printed card, and the def is
faithful; the gap is engine-layer (`OOS-DX20-10` precedent). A demotion would also redden
`pb_dx32_fuzz_output.rs`'s `CORPUS_COMPLETE = 1133` pin and, per `OOS-CARDS2-3`, re-roll every
recorded fuzz seed across five shipped flows — out of all proportion to the finding. (2)
`deflecting_swat`'s requirement was **not widened**; its false comment was corrected in place and
the mismatch filed as `OOS-DX25b-5`.

**Seeds.** `OOS-DX25-3` **CLOSED** — its registry row now carries four corrections to its own
claims (wrong function name; "validation-site only" wrong by three sites; the in-src tests are not
merely negative and fail for a fixture reason, not a rejection reason; `untimely_malfunction`'s
mode 1 fails through the flat/pooled `mode_targets: None` scheme, a more fundamental mechanism than
mode ambiguity). Filed **OOS-DX25b-1..5**. **`OOS-DX25b-3` is LIVE on the same two `Complete`
defs**: this batch is what makes CR 115.7a's unchecked object-target redirect reachable, so a
Misdirected "destroy target creature" now destroys the lowest-`ObjectId` battlefield object —
routinely a basic land — reachable by a human in the browser *and* by the bots, since
`simulator/targeting.rs` routes through the same query. Pinned wrong-way-round for the successor.

**Durable lesson.** *A fixture that collapses two id spaces makes a test green by removing the only
condition under which the code is wrong.* Every test guarding this defect passed, for four months,
because each one hand-built the one state a real cast can never reach. And PB-DX25's enumeration
lesson recurred a third time: the brief, the plan, and the batch's own execution notes were each
short about a different thing.

Full record: `memory/primitives/pb-plan-DX25b.md`, `pb-review-DX25b.md`, and
`pb-DX25b-execution-notes.md` (measurements, the 16-row revert matrix, and §9's corrections to the
plan found by execution).

## Worker Handoff (PB-DX25, `scutemob-203`) — a countered spell is countered, whichever shape it arrived in

**What was wrong.** `Effect::CounterSpell` (`effects/mod.rs`) decided *"does this stack object own
a card in `ZoneId::Stack`?"* by matching the **variant name**. Its `position()` lookup matched
`so.id == id` (the Ward path, CR 702.21a) or `StackObjectKind::Spell { source_object } == id` (the
traditional counter, which targets the **card**) — and nothing else. It then `remove(pos)`ed the
entry **before** matching on the kind, and the match had a `Spell` arm, a combined
`ActivatedAbility | TriggeredAbility` arm, and a `_ =>` catch-all that did nothing. No `is_copy`
check anywhere.

**What the seed and the queue row both got backwards, and the correction is the interesting part.**
`OOS-SIM3-5` ranked (a) — countering a `MutatingCreatureSpell` strands its card — as the live
defect, with (c) as a rider. It is the other way round, and more than that:

* **(a) was never independently reachable.** Ward needs a `GameEvent::PermanentTargeted` naming the
  mutate spell's stack entry, and that event is emitted only for `spell_targets`. A mutate cast's
  target rides in `AdditionalCost::Mutate` and **never enters `spell_targets`** (filed as
  `OOS-DX25-1`), and the SR-36 enumeration measured **0** `Complete` mutate defs declaring a
  spell-level target requirement (roster M3 = 0). So **(a) is what fixing (c) ALONE would have
  created** — a permanent `ZoneId::Stack` leak in place of a silent no-op, reported by
  `stack_consistency` at every subsequent checkpoint for the rest of the game. **A "just fix the
  `position()` lookup" change was strictly worse than HEAD.** This is the single most important
  sequencing fact in the batch and it is why (c) and (a) landed in one commit.
* **(b) is unreachable three ways, not the v3 memo's one.** The ordering argument (`position()`
  returns the lowest index; `copy.rs` pushes the copy above the original), *plus*
  `resolve_effect_target_list_indexed` dropping a dead `DeclaredTarget` — so the window is **empty**,
  not narrow, even under CR 608.2b re-validation — *plus* nothing aiming a counter at a copy at all:
  a copy's stack-entry id is not a `state.objects` key, so `TargetSpell` validation refuses it
  **always**, not merely once the original is gone.
* **(c) is worse than "a silent no-op" sounds.** `TargetSpell` validation resolves the announced id
  through `state.objects` and requires `zone == ZoneId::Stack` — and a mutate spell's card really is
  there. So the engine **offers the target, validates it, takes the mana, and does nothing.**
  Measured exposure: **66** live-wrong pairs.

**What shipped, and why it is structural rather than three patches.** A new engine-side
`crates/engine/src/state/stack_registry.rs::card_in_stack_zone(&StackObjectKind) -> Option<ObjectId>`
— exhaustive over all **27** variants with **no wildcard arm**, `Some` for `Spell` and
`MutatingCreatureSpell` only. Both counter paths consume it: `Effect::CounterSpell` and
`resolution.rs::counter_stack_object`. Adding a 28th card-carrying variant is a **compile error**
until someone classifies it — the same forcing function SR-5 applies to `KeywordAbility`, and it
lives in the engine rather than `card-types` on the `keyword_registry` precedent (which also keeps
the 1,798 card defs `Fresh`). The zone-move moved **out** of the per-kind match and is skipped
entirely when `stack_obj.is_copy` (CR 707.10 — a copy is a spell with no card; `copy.rs` clones the
original's `kind` wholesale, so moving `source_object` would put **someone else's** spell in the
graveyard). A countered copy emits `SpellCountered` with `stack_object_id == source_object_id ==`
its own stack-entry id: CR 707.10 makes the event owed and forbids naming a card id, and the
already-shipped `event_view.rs` fallback renders *"<player>'s spell is countered"* with no renderer
and no wire change.

**The decision worth carrying forward: the verifier was deliberately NOT unified with the thing it
verifies.** The simulator's `invariants::stack_card_of` answers the same question and was the model
for the fix — but it is `check_stack_consistency`'s classification, and that check exists
*specifically* to catch the engine getting this wrong. If the verifier read the engine's own answer
back, a wrong `Some`/`None` would make the check **agree with the defect and go silent**, in exactly
the case it was written for. So there are two implementations on purpose. What keeps them honest:
both are exhaustive with **no wildcard**, so a new variant is a compile error in **both crates
independently** (coverage is machine-synced); the *classification* is deliberately unsynced, so a
disagreement is loud by construction; and one behavioural probe
(`crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs`) proves they agree on the case
that matters by running a real counter-on-mutate game, rather than by sharing code. Doc
cross-references at both functions say all of this, so the next reader does not "simplify" it.
**Contrast with PB-DX20 deliberately**: there, two *consumers* of one derivation were unified,
because disagreement between them was the defect. A *verifier* is not a consumer.

**The `/review` cycle found 0 HIGH / 6 MEDIUM / 3 LOW + 7 folded notes, all taken — and its three
sharpest findings were this batch's own failure mode recurring inside the batch.**

1. **The plan's "FOUR sites that classify a stack object's card/spell-ness" census was short by
   two, and one of the two was wrong in the same direction as the defect being fixed.**
   `abilities.rs:6736`'s `targeting_is_spell` matched `Spell` alone, gating every CR 601.2c
   "becomes the target of a **spell**" trigger — while `casting.rs:6507`, answering the *identical*
   question one function over, pairs both kinds. Two implementations of "is this a spell",
   disagreeing: verbatim the argument the census was written to make. `casting.rs:7126`'s
   `has_split_second_on_stack` was the sixth, and it is `card_in_stack_zone`'s exact question left
   unconverted. Both fixed; the census corrected to six in the plan and the notes.
2. **The SR-36 roster's `P = 48` was an undercount, and the coordinator had already written it into
   the queue memo.** The enumeration walked `Effect::CounterSpell` and was structurally blind to
   **`Effect::CounterUnlessPays`**, which `effects/mod.rs` delegates *straight into the repaired
   arm* — so `mana_leak`, `mana_tithe` and `make_disappear` (all `Complete`, all carrying
   `TargetSpellWithFilter(TargetFilter::default())`, which is unrestricted field-by-field because
   `TargetController::default() == Any`) were equally live-wrong and equally invisible. It also
   missed counters on activated/triggered abilities and on back faces. Re-measured: **C1 29, C2 24,
   C3 11, P 66.** The first re-measure replaced a grep-derived wrong number (144) with a
   differently wrong one, **with the authority of an SR-36 enumeration behind it**.
3. **T6's advertised non-vacuity did not exist.** `assert_eq!(variants.len(), 27)` compared a
   hand-written `vec!` against itself; a 28th variant classified in the registry would leave it
   green. The property actually lives in `g1_scan_is_not_vacuous`, which counts arms **in source**
   — a different subject, in a different crate target. This is the PB-DX24 durable lesson recurring
   in the batch dispatched immediately after it.

Also from the cycle: a doc cross-reference pointed at a comment **that was never written**
(`stack_registry.rs` → `casting.rs:6503`, the MR-M11-12 class — written now); `SpellCountered`'s
type-level doc was false for two of the three shapes that now emit it; a new **G4** gate was added
over `counter_stack_object`, because criterion 6232's "single classification, **both** paths" half
had been resting on argument plus one test rather than on a machine; and G2 was hardened against the
`use StackObjectKind as K` alias form **the registry itself uses**. One note was **declined with a
reason**: a T4 sub-case for the `unwrap_or(controller)` owner fallback would be misleading, because
`move_object_to_zone` does the identical lookup on the same id moments later, so whenever the
fallback could fire the move has already failed (CR 400.7 fizzle) and the fallback value is never
observable — the plan's claim was narrowed instead of a synthetic probe being written.

**An unclaimed positive the reviewer found.** The `is_copy` guard also closes a CR 702.99c hole
nobody was looking for: `resolution.rs:5418-5430` builds a cipher copy as
`Spell { source_object: <a card in EXILE> }` with `is_copy = true`, so countering one through the
Ward clause would previously have pulled the encoded card **out of exile** into a graveyard.

**Two coordinator-side corrections, recorded because both are the batch's own subject.** A shipped
doc cited **PB-DX9** — an *unshipped* queue entry — as the precedent for keeping
`counter_stack_object`; `git log -S` over the quoted sentence found the real one, **PB-DP9**
(`f33aabe2`, `scutemob-157`). And **CR 701.5 is `Cast`, not `Counter`** — `Counter` is **CR 701.6**,
and the widely-cited "CR 701.5g" does not exist; ~337 sites tree-wide carry the stale number, filed
as `OOS-DX25-6` and corrected only inside the region this batch already edited.

**Measurements.** Tests **4,452 / 0 / 5** (+17 over the **4,435** pre-edit baseline measured on this
branch before any edit). PROTOCOL **35** / HASH **73** gate-executed and unmoved — re-executed after
the `abilities.rs`/`casting.rs` edits, along with the SR-5 `keyword_registry` gate (9/0, unmoved).
Coverage unmoved **1,133/1,803 = 62.8%**, proven by **regeneration** rather than by the empty
card-defs diff the plan would have accepted (criterion 6233 asks for regeneration specifically).
`clippy -D warnings`, `fmt --check` and `tools/check-defs-fmt.sh` all clean. Benches within noise
(`full_turn_4p` 214-215 µs). SR-6 scope empty: **0 lines** in `crates/card-defs/`,
`crates/card-types/`, `crates/view-model/` or `tools/`. Every new probe and gate proven
discriminating by **executing** its revert; the matrix and every failure text are in
`memory/primitives/pb-DX25-execution-notes.md`.

**Seeds.** **OOS-SIM3-5 CLOSED**, its row carrying four corrections to its own claims rather than
having them deleted. Filed **OOS-DX25-1..6**. **Read `OOS-DX25-3` before the next batch**: it is
**LIVE on two `Complete`, deck-legal defs** — `misdirection` and `bolt_bend` can never resolve a
legal target, because `validate_target_requirement` keys the announced id on `state.objects` (the
**card**) and then compares it to `so.id` (a **stack-entry** id), two namespaces minted from one
monotone counter that therefore never intersect. Its in-src tests are **negative** tests and pass
**vacuously**, because the requirement refuses everything. It is the same id-space confusion as this
batch's subject, one function over, and it was found by accident.

**Durable lesson.** *An enumeration is only as wide as the variant list it walks, and an exhaustive
match proves nothing about the callers that never ask it.* The batch built a classification that
cannot silently miss a **variant** — and then shipped a roster that silently missed a **delegating
effect**, a gate whose message described a class it could not see, and two live call sites that
never consulted the classification at all. Exhaustiveness is a property of a match, not of a
program.

## Worker Handoff (PB-DX24, `scutemob-202`) — a zone-scoped ability finally functions in its zone

**What was wrong.** `AbilityDefinition::Triggered` carries a `trigger_zone` field.
`TriggeredAbilityDef` — the runtime shape `build_face_ability_vectors` lowers into — has no home
for it, so 33 of the lowering's 34 trigger arms swallowed it. `nether_traitor` pairs
`WheneverCreatureDies` with `trigger_zone: Some(Graveyard)` and is `Completeness::Complete`, so a
deck-legal card had its graveyard ability **installed on the battlefield object** and functioned
from exactly the wrong zone. CR **113.6m** is the load-bearing rule: the ability's effect moves the
card *out* of the graveyard and its trigger condition does not put it *there*, so it functions only
there.

### Read this before trusting the brief, the queue row, or stage 0

**1. The brief was short by a whole half, and the narrow fix alone would have shipped a card that
fires nowhere.** Both the task brief and the v3 queue row read this as "one line, wire-neutral" —
delete the lowering, done. But `collect_graveyard_carddef_triggers` (`abilities.rs:7112`) had
**one** `fires` arm, `PermanentEnteredBattlefield`, written for Bloodghast's landfall. A
`WheneverCreatureDies` graveyard trigger had **no dispatch path at all**. Suppressing the
battlefield lowering without adding the dispatch would have turned a wrong-zone card into a
silent no-op — and a green test suite would have said nothing, because nothing tested it.
Criterion 6205 demanded *both directions*, which is what forced the discovery.

**2. Stage 0's own arm count was wrong, and the fix cycle caught it twice.** Stage 0 said 40 arms;
stage 3 re-measured **34**; the reviewer independently derived 34. The lowering's doc table now
carries the **counting rule** (36 `for ability in abilities` loops minus the 2 mana/activated
loops), so the next reader re-derives it instead of trusting it. Three published numbers for one
census is the same failure mode PB-DX19 recorded — publish the rule, not the number.

**3. The loss was never uniform, and that is why a single-arm fix would have been wrong.** The
`WheneverPermanentEntersBattlefield` arm had always skipped. The repair does not add 33 more
`continue`s — it extracts the whole trigger-lowering region into `build_face_triggered_abilities`,
whose input is filtered **once** at its single call site through `lowers_onto_the_battlefield` (an
exhaustive match on `TriggerZone`, no wildcard), and **deletes** the old per-arm guard so there is
one mechanism rather than two. A 41st arm cannot re-swallow the field; two source gates fail if one
tries, and the comment-stripping in the structural gate was itself proven load-bearing by executing
both variants (PB-DX32's M8 lesson, applied rather than cited).

### What shipped

- **The lowering half** (`testing/replay_harness.rs`): extraction + filter + the corrected lossy
  table row, which now reads *honoured* rather than *dropped*.
- **The dispatch half** (`rules/abilities.rs`): a `WheneverCreatureDies` arm in
  `collect_graveyard_carddef_triggers` mirroring the battlefield `AnyCreatureDies` arm clause for
  clause — CR 108.4a (a graveyard card has no controller; its **owner** stands in), CR 400.7
  (`exclude_self` compares the **graveyard** id, because the battlefield id can never match one and
  a battlefield-only comparison fails **open, silently**), CR 111.7, CR 603.10a/613.1d — plus a
  CR 603.10a look-back guard (`arrived_in_graveyard_this_batch`) applied to **this arm only**. The
  ETB arm must not gain it: CR 603.10a's list does not include ETB triggers, so Bloodghast arriving
  in the graveyard alongside a land entering still triggers. That asymmetry is written at the guard,
  because it is the one place a future reader will be tempted to "unify" the two arms and be wrong.
- **OOS-DX1-4**: six queue sites moved to `def.effective_abilities(<source>.is_transformed)`; Q5
  re-scoped comment-only.

### The thing worth knowing about OOS-DX1-4's closure

**Its "6 latent queue sites" was right for the wrong reason.** The SR-36 enumeration of
`all_cards()` — never a grep — measured **0** corpus defs carrying any of the seven Q-shapes on a
back face (1,803 defs, 15 with a `back_face`). So all seven are latent, live exposure is zero, and
**every probe is a synthetic `back_face` fixture**. The repair is structural, and the closure says
so rather than implying a live repair.

**Q5 is the interesting one, and both the plan and the review got its rule wrong.** The plan cited
CR 712.2 (which is about DFC face *symbols*). The reviewer corrected it to CR **712.16** and noted
CR **712.15** makes the site reachable after all — but stopped there, so it could only conclude
"unreachable in practice, by engine discipline". CR **712.15a** settles it properly: *"if it's
turned face up, it will have its **front** face up"* — so the one DFC that can reach this site
does so on its front face **by rule**, and reading `def.abilities` there is **CR-correct**, not an
unreachable-case accident. Verified against the rules MCP during the fix cycle.

### The review cycle: 0 HIGH / 6 MEDIUM / 7 LOW — all 13 taken, and two were the coordinator's

The reviewer ran **without a shell** — every finding was derived by reading source. Each was
verified by execution before being applied; none turned out wrong.

- **The two that were mine, not the runner's.** (1) `OOS-DX24-1` asserted "live-wrong on 2
  `Complete` defs" — false: `teysa_karlov` and `drivnod_carnage_dominus` are both
  `Completeness::partial`, so `validate_deck` rejects them. (2) The same row framed the doubler
  defect as *"a NEW instance introduced by this batch"* — wrong about the class: the ETB doubler
  arms have had a graveyard-sourced pairing since PB-35 (Bloodghast). Re-measuring past what the
  review concluded gave a third answer, now in the row: **no pairing is deck-legal on both halves
  in either direction**, so exposure is **zero deck-legal pairings** — not "2 `Complete` defs", and
  not the review's "the deck-legal instance predates this batch" either. The fix is also *smaller*
  than first written: one source-zone conjunct above the `match` covers all four
  `TriggerDoublerFilter` arms, not "every death doubler".
- **A gate that was green while the invariant it claimed to pin was already violated.**
  `test_dx24_is_transformed_true_assignment_has_exactly_one_site` matched only a literal
  `is_transformed = true`, but `face.rs:97` writes a **computed** bool and is how
  `Command::Transform` sets it — the batch's *own* Q3/Q4/Q6 probes assert exactly that. Replaced
  with a runtime probe of the real invariant, and the reviewer's claim ("delete `face.rs:67-69` and
  every PB-DX24 test still passes") was checked by **doing it**: the new probe reddens, the old one
  did not.
- **A plan risk that was never discharged.** Plan §10 risk #2 (look-back slice granularity) was
  silently skipped. Now measured per caller: `sba.rs:97` exact, `resolution.rs:8142` coarse (a whole
  resolution's events), `combat.rs`/`engine.rs` **unaudited and stated as such**. Filed
  `OOS-DX24-7`.

### Numbers

- Tests **4,413 → 4,435 / 0 / 5** (+22). Baseline measured on-branch **before any edit**; final run
  `--workspace --no-fail-fast` to a file, residual list empty.
- **PROTOCOL 35 / HASH 73 gate-executed and unmoved** — no `TriggeredAbilityDef` field, no
  `Command`/`GameEvent`/`Effect` variant. `core keyword_registry` green (run, not reasoned about:
  PB-DX20 and PB-DX23 were each caught by that gate).
- Coverage **unmoved at 1,133 / 1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py`
  to an identical body — *not* by an empty card-defs diff, since this batch mandates a comment-only
  def edit. `tools/check-defs-fmt.sh` clean over 1,803 defs (SR-35).
- Benches within noise: `full_turn_4p` 221.5–223.5 µs, `sba_check` 14.92–14.99 µs,
  `priority_cycle_4p` 24.60–24.83 µs.
- Scope: **0 lines** in `crates/simulator/`, `tools/`, `crates/card-types/`; `crates/card-defs/` is
  one comment-only file.

### Seeds

**CLOSED**: `OOS-DX1-3`, `OOS-DX1-4` — both rows also carry corrections to their **own** original
claims ("latent" was false; "all 34 sites" was wrong twice over).
**FILED**: `OOS-DX24-1..9`. The two to read first:

- **`OOS-DX24-1`** — `doubler_applies_to_trigger` is source-blind in **all four** filter arms.
  Latent (no deck-legal pairing), deferred on the plan's own risk #3, fix is one conjunct.
- **`OOS-DX24-9`** — **LIVE on a `Complete` def.** CR 118.12 makes an optional cost a player
  decision; `MayPayThenEffect` auto-pays. The *class* is the pre-existing DP-19 shape, but this
  batch is what makes `nether_traitor`'s instance reachable at all, so it is live **as of this
  batch**. This is also why the T3 probe had to be re-worded: it cited CR 118.12 while asserting
  pay-when-able — citing the rule the engine deviates from as though it implemented it.

### Durable lesson

**A guard, a gate and a claim each have a subject, and "it passes" only tells you about the subject
it actually has.** Three of this batch's findings are one shape: a gate that scanned for a literal
assignment while the real write was computed; a look-back set whose granularity is decided by
whichever caller hands it a slice; and a seed row whose severity tag named a completeness marker
nobody had read. Each was *true about what it examined* and *wrong about what it was taken to
mean*. The batch's own fix has the same shape and is the reason it is trustworthy: the filter is at
the **call site**, so the thing it must be true of is the one thing it can see.

### For the collector

`memory/primitives/pb-plan-DX24.md` (plan), `pb-DX24-stage0.md` (re-verified premise),
`pb-DX24-execution-notes.md` (measurements, revert matrix, per-caller granularity, bench numbers),
`pb-review-DX24.md` (the review, incl. its "what I checked and found correct" section — read it
before re-auditing anything). Next queue row: **PB-DX25** (v3 rank 7).

## Worker Handoff (PB-DX23, `scutemob-201`) — dredge becomes answerable, by anyone

**What was wrong.** `grep -rn "ChooseDredge" crates/simulator/src/ tools/` returned **zero**: the
engine had `Command::ChooseDredge` and `GameEvent::DredgeChoiceRequired` and a gated handler, and
**nothing could reach any of it**. No `LegalAction::ChooseDredge` existed, so neither a bot nor
the human browser seat could answer a dredge offer.

**The consequence is not a lost option, it is a permanent draw-cadence corruption**, and the
probe measured it rather than arguing it. On a real 2-player `LocalGame`, both bot seats, no state
pokes, `golgari_grave_troll` in `p1`'s graveyard, six turns: **two** `DredgeChoiceRequired` events
fired, **one** card was drawn where **two** were owed, and **one** `PendingDraw` survived to the
halt. Each turn the draw step defers and the *next* turn's draw discharges the stale entry before
deferring the current one — forever one behind, off a library that has had a full turn cycle to be
reordered.

### What shipped

**One derivation, two consumers.** `rules::queries::dredge_options(state, player) ->
Vec<(ObjectId, u32)>` (CR 702.52a/b, sorted by `ObjectId`) is now the only dredge-eligibility scan;
`check_would_draw_replacement` calls it instead of keeping its own copy, and the offer layer
consumes the same function. Re-deriving it in `crates/simulator` would have been the `OOS-RS-2`
drift class. **The SR-5 keyword registry caught what the brief missed** — `queries.rs` is a
`Dredge` handling site and the gate failed until it was declared.

**`LegalAction::ChooseDredge { card: Option<ObjectId>, mill: u32 }`**, emitted as an ORDINARY
priority-window action, mapped in `params.rs`, scored in `heuristic_bot`, labelled in
`view.rs`. Bot and human channels both live.

**`OOS-DX2-2`: the tail of a multi-draw is a DIFFERENT draw.** `perform_remaining_draws`'
hard-coded `offer_dredge: false` is now a parameter, and `resolve_declined_pending_draw` gained
`tail_offers_dredge`.

### Three things worth reading before trusting anything here

**1. The brief's two-site framing was short by one, and the naive flip would have shipped a new
bug.** There are **three** resume sites passing `offer_dredge: false`, not two —
`resolve_pending_draw`'s CR 616.1f re-check is a same-draw site and must stay `false`. More
importantly, an *unconditional* `true` at the tail makes the REOPENED `OOS-DX2-3` **live and
reachable from the corpus's only dredge card**: `perform_one_draw`'s implicit stale-entry
discharge would run a tail that pushes a dredge entry, then control returns to the outer call
which pushes its own — two dredge-originated entries for one player, breaking the one invariant
the discharge does establish. The flag is threaded so that discharge alone passes `false`, and
`test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry` pins the exact trace.

**Why PB-DP5 §3.3 does not extend to the tail** (this is acceptance criterion 3, and the
distinction is load-bearing): §3.3 argues for `false` because re-offering dredge mid-chain would
restart a CR 616.1 application the player already began *on the same draw event*. That is a claim
about ONE draw. CR 121.2 makes "draw three" three separate draws, and CR 614.11a / 121.6b say the
replacement completes and *then* the sequence resumes — so each resumed draw is its own fresh
"would draw" event.

**2. The brief's UI prescription was one layer off — the PB-DX20 pattern, again.** It asked for
the play-server's blocking-decision UI. The choice lives in the `LegalAction` itself (the
`PayEcho` shape), so the human channel needed **no** `AnswerShapeView` variant, **no**
`ActionParams`/DTO field, **no** picker, and **zero frontend production lines**. Routing it
through the blocking-decision UI would have meant a fourth `BlockingDecision` variant — CR-wrong
(CR 702.52a is "you **may** instead", and the engine deliberately never blocks) and a HASH bump
for an optional decision. The reviewer adjudicated the divergence ACCEPTABLE. T5.1 proves the
human channel end to end over the real router with a NON-DEFAULT answer, and asserts the option
carries no `decision` key — that assertion *is* the pin on this divergence.

**3. The review found the batch's own overclaim, and it was this batch's own failure mode.** The
plan's suppression rule (no offer when nothing is dredge-eligible) was documented as removing the
decline-forever loop **"structurally"**. It does not: the guard is keyed on the GRAVEYARD, while
the entry `handle_choose_dredge` answers is chosen FIFO, and `PendingDraw` carries no origin
discriminator. With a `NeedsChoice`-origin entry queued ahead of an eligible dredge card the
provider offered, the engine answered the wrong entry, the decline re-deferred, and a bot below
the mill margin declined forever. **That is the same shape of claim `OOS-DX2-3` was wrongly closed
on — made inside the batch dispatched to avoid repeating it.** Fixed with a third conjunct that
asks the engine's own question (would declining THIS FIFO entry discharge it?), with its limits
stated: it is a conservative approximation, exact only at queue depth ≤ 1, which is every
reachable case today. `OOS-DX23-8`.

**The reviewer's first suggested fix was declined on precedent**: a `RepeatKey::ChooseDredge` cap
in `heuristic_bot`. PB-DX21 *deleted* exactly that shape — a bot-side repeat cap masking an offer
the provider should never have made. The offer layer is where a bad offer dies (SR-38).

### Numbers

Baseline **4,398 / 0 / 5** re-measured on this branch at `e490153b` before any edit → **4,413 / 0
/ 5**, residual list empty. +15 reconciles exactly: 1 mandatory probe, 7 engine, 6 simulator, 1
play-server. **PROTOCOL 35 / HASH 73 gate-executed and unmoved** — no state field, no wire type.
Coverage unmoved **1,133/1,803 = 62.8%**, card-def diff comment-only, `check-defs-fmt.sh` run
(SR-35). play-server 80/0. The golden dredge script is byte-unchanged. Every recorded fuzz ratchet
was checked explicitly and **none moved** — diagnosed, not assumed: none of the pinned seeds' deals
reaches a dredge offer.

### Seeds

**CLOSED**: `OOS-DX2-5`, `OOS-DX2-2`. **RECORDED, not closed**: `OOS-DX2-7` — now an AUTO-CHOSEN
row in `docs/audits/decision-point-audit.md` §3.1, NON-DSL, 1 `Complete` def reachable, invisible
to `decision_gate` **by construction** (the walk enumerates card-def `Effect`/`Condition` DSL
variants; dredge is a `KeywordAbility`), a fresh `OOS-DP10-9` instance. This batch made the offer
*answerable*; it did not make the auto-discharge stop being an engine-made decision.
**STAYS REOPENED**: `OOS-DX2-3` — not re-closed, and not on a structural argument; the protected
pin is byte-unedited. `OOS-DP5-2` unchanged: answerable, not bounded.

**Filed**: `OOS-DX23-1` (a non-priority-holder's offer is deferred to their next priority window —
CR 117.3d makes that a deferral, never a loss, but the moment is the engine's), `-2`
(`NeedsChoice`-origin entries), `-3` (the TUI has no dredge channel — it hand-builds commands and
never routes through `params.rs`; same family as `OOS-UI2-5`/`OOS-DX6-5`), `-4` (bot dredge policy
is survival-only, no value evaluation), `-6`, `-7` (both method seeds, below), `-8` (the S1
residual). `-5` deliberately NOT filed: it was conditional on a ratchet moving and none did.

### Two method seeds, both found by executing rather than reasoning

**`OOS-DX23-6`** — `cargo build --workspace` does **not** compile test targets, so this project's
standing "the compiler points at every exhaustive match" assumption is false for matches living in
tests. `local_game_playthrough.rs::kind_of` was caught only by `clippy --all-targets`.

**`OOS-DX23-7`** — the doc-reattachment trap, **second occurrence** in this codebase. A doc comment
attaches to the item that follows it, so inserting a function between an existing doc block and its
function silently reassigns the doc. `attack_tax_total` nearly lost its entire doc, including its
load-bearing PB-DX6 "`None` does NOT always mean free" warning. `perform_remaining_draws` carries a
note recording the same trap from PB-DX2's fix cycle.

### Durable lesson

**A guard keyed on one thing cannot police a decision keyed on another.** The suppression rule
asked about the graveyard; the engine answers FIFO. Both were correct about their own subject and
the pair was wrong — and the doc that said otherwise was written by the same reasoning that
produced the guard, which is why only an adversarial read caught it.

### For the collector

Review: `memory/primitives/pb-review-DX23.md` (0 HIGH / 4 MEDIUM / 9 LOW, all 13 taken). Plan:
`memory/primitives/pb-plan-DX23.md`. Every measurement and the full revert matrix:
`memory/primitives/pb-DX23-execution-notes.md`. **Known gap, stated rather than papered over**: no
browser-level (headless Chromium) pass was run; T5.1 covers the same path in-process over the real
router.

## Worker Handoff (PB-DX21, `scutemob-200`) — declaring attackers is once per combat

**What was wrong.** CR 508.1 makes declaring attackers a turn-based action performed **once** per
declare-attackers step. `combat.rs::handle_declare_attackers` guarded on step, active player,
priority holder and per-attacker legality — and on nothing else — and initialised `CombatState`
only when `None`, so a second `Command::DeclareAttackers` in the same combat reran the whole body.

**Three consequences, not the seed's one.** The seed row said a re-declaration "overwrites
`combat.attackers`". It does not: `:745` **inserts** into an `OrdMap`, so declarations *accumulate*
and a repeated same-id entry overwrites only **that creature's attack target, mid-combat**. (2)
`:795-806` pushes a fresh `AttackersDeclared` and re-runs `check_triggers` +
`flush_pending_triggers`, so **every attack trigger re-fires per declaration** — the one a human
hits first. (3) `:759` *assigns* `attackers_declared_this_turn`, clobbering the raid count read by
`Condition::YouAttackedWithNOrMore` on `windbrisk_heights` and `legions_landing`. **A fourth was
found during the batch**: `:818` resets `state.turn.players_passed` on every accepted declaration,
so a re-declaring client holds the CR 117.4 pass-round open **with no attacker changing** — which
is the *empty* declaration's only consequence, and the reason the guard could not key on the map.

**The brief's preferred one-liner would have shipped a new bug — read this before the next batch
that wants to avoid a HASH bump.** The v3 queue brief said "PREFER reading `combat.attackers` over
adding a field", because a marker mirroring `defenders_declared` moves HASH. Refuted three ways,
any one sufficient:

1. **CR 508.1a** — "chooses which creatures … **if any**" — plus **CR 508.8** make an *empty*
   declaration a **completed** action, and the empty declaration is a live shipped client action
   (`params.rs:474` maps default params to `Command::DeclareAttackers { attackers: vec![] }`;
   `play-server/README.md` already called it "irreversible", aspirationally).
2. **CR 508.4 / 506.3** — "put onto the battlefield attacking" inserts **straight into
   `combat.attackers`** at four sites (`effects/mod.rs:1502`, `:6331`, `resolution.rs:6020`,
   `:6480`) with no declaration at all, and CR 508.4c exempts such creatures from declaration
   requirements. An `attackers`-keyed guard would have refused a player's **first, legal**
   declaration in any combat where such a creature entered attacking first.
3. It cannot see consequence (4) at all.

**The durable form of that lesson: a guard keyed on a collection cannot tell "chose nothing" from
"has not chosen", and cannot tell your own writes from someone else's.** `CombatState` gained
`attackers_declared: bool` (`#[serde(default)]`), hashed beside `defenders_declared`; **HASH 72 →
73 computed from the failing gate's own output**, history row APPENDED, **45** sentinel lines across
**44** files re-pinned (the plan predicted 44 — the extras were two bare-`72` spellings and two
split across two lines, which only a full run surfaced). PROTOCOL **35** gate-executed, unmoved.

**What shipped.** `GameStateError::AlreadyDeclaredAttackers(PlayerId)` mirroring
`AlreadyDeclaredBlockers`, carrying a `PlayerId` and nothing else (it reaches a client as 422 text —
Architecture Invariant 7). The guard sits **after** the priority check and **before** the
`CombatState` init and every tax debit, so a refusal is byte-identical (CR 732). The marker is set
on the **success path only**, inside the attacker-insert block and **before** the CR 603.3d
suspended-trigger early return — a declaration that suspends on a trigger target choice must not be
re-declarable. **CR 509.1a is verified covered and deliberately NOT widened** (`combat.rs:1103`).

**The offer layer had to follow, and that is what makes the closure provable.** Deleting
`local_game_playthrough.rs`'s `PolicyState` cap — mandated — would have turned its own
*"a rejection means the offer was wrong"* assertion red, because the policy would re-declare and the
engine would now refuse. So `legal_actions.rs:878` suppresses the `DeclareAttackers` offer once the
marker is set (SR-38). With **both** mitigations deleted *with their mechanism* (the bot's
`RepeatKey::DeclareAttackers` cap too), that test green **with no cap** is the closure proof for
`OOS-M11-9`. Same shape as PB-DX20's `KNOWN_FALSE_OFFERS` deletion.

**Two review findings worth carrying forward.**

- **A card-def comment asserted a defect the card does not have.** The first-draft note in
  `legions_landing.rs` put it in `OOS-DX21-1` and cited CR 508.6 for a claim CR 508.6 does not make.
  Legion's Landing is a **CR 508.3d per-declaration** trigger — evaluating false in a second combat
  where one creature attacked is *correct*. Following the note would have **regressed** the card.
  `OOS-DX21-1` is re-scoped to `windbrisk_heights` alone (turn-scoped, ruling 2007-10-01) with an
  explicit "do not migrate Legion's Landing" warning.
- **Four probes were reading state their failing call never touched.** `process_command` returns
  `Result<(GameState, …), GameStateError>`, so on `Err` Rust discards every mutation the callee
  made. A probe that clones state, expects an `Err`, then reads the *original* passes identically
  whether the guard is at the top of the function or absent. T6's entry-vs-success-path revert did
  not redden until rewritten to call `handle_declare_attackers(&mut state, ..)` directly, and T4's
  CR 117.4 pin was **fully vacuous** until repaired the same way. Filed tree-wide as
  **`OOS-DX21-7`** — a sweep of existing "rejection leaves state unchanged" probes would likely
  find more.

**Also measured, not asserted** (review M7): suppressing an offered action **reindexes every
subsequent `RandomBot` draw** (`random_bot.rs:63` picks uniformly by index), so offer-layer changes
move seeded fixtures that have nothing to do with them. Gate-config rejection rate moved
**31.081‰ → 6.909‰**, wasted-tap share **89% → 92%**, both inside their ratchets and **neither
constant changed**; `pb_dx32_fuzz_output.rs` T4.1/T4.3 unmoved **proven by an executed ablation**,
not observed. Filed as `OOS-DX21-6`.

**Numbers.** Tests **4,398 / 0 / 5** (+10 over the 4,388 pre-edit on-branch baseline), residual list
empty, independently re-run. Coverage unmoved **1,133/1,803 = 62.8%**, proven by byte-identical
report regeneration (two comment-only card-def edits, so an empty defs diff would have proved
nothing). Benches slightly faster and within noise. Golden scripts: exactly two files carry ≥2
`declare_attackers` and both repeats are **cross-turn**, so no script churn and SR-9b green.
Review **0 HIGH / 7 MEDIUM / 8 LOW, all 15 taken**.

**Seeds.** `OOS-M11-9` **CLOSED**. Filed `OOS-DX21-1..7` in `docs/audits/decision-point-audit.md`
§8.1. The two a successor should read first: **`OOS-DX21-4`** (CR 508.8's skip predicate is a
step-END read of `combat.attackers`, so killing your only attacker mid-step skips declare-blockers
*and* combat damage — pre-existing, and the naive `!attackers_declared` fix is **wrong**) and
**`OOS-DX21-2`** (the CR 509.1a twin of the offer hole, deliberately not widened into).

**Artefacts.** Plan `memory/primitives/pb-plan-DX21.md`; stage 0 `pb-DX21-stage0.md`; review
`pb-review-DX21.md`; revert matrix and every measurement `pb-DX21-execution-notes.md`.

---

## Worker Handoff (PB-DX20, `scutemob-198`) — one derivation, two consumers

**What was wrong.** An Aura's CR 303.4a target requirement lives in `KeywordAbility::Enchant`,
which `casting.rs` special-cased. Every offer-side consumer reads
`mtg_engine::spell_target_requirements`, which reaches `card_def_target_requirements` — and that
function sees `AbilityDefinition::Spell.targets` only. An Aura has no `AbilityDefinition::Spell`,
so the list was empty, `target_count_range` was `(0, 0)`, the browser rendered a zero-target
action, and the human's click 422'd. **13 deck-legal `Complete` Auras**, on first contact.

**The queue brief's prescription was one layer off, and this matters for the next batch.** It said
to synthesize the requirement in `crates/simulator/src/legal_actions.rs`. That file is not on the
browser's path: it decides *which actions exist*, not *what they announce*. The fix landed in
`rules/queries.rs`, and `legal_actions.rs` took **zero** lines — as did every production file
under `tools/play-server/` (`view.rs` already read everything through the query). If you are
chasing an offer-side defect, find the consumer before you patch the producer.

### What shipped

* `casting::enchant_target_to_requirement` — a **total** map over all 9 `EnchantTarget` variants,
  exhaustive `match` with **no wildcard arm**, so a future variant is a compile error rather than
  a silent `vec![]`.
* `casting::aura_spell_target_requirements(chars, base)` — synthesizes only when the object is an
  Aura enchantment, `base` is empty, and the keyword is present. Consumed by **both**
  `handle_cast_spell` and `queries::spell_target_requirements`. That is the SIM-1 lesson taken
  literally (`effective_cast_cost` consuming `apply_commander_tax` rather than re-deriving it):
  the two sides are one arithmetic, not two that agree today.
* **No new `TargetRequirement` variant** — the mapping is expressed in existing variants, so
  PROTOCOL/HASH could not move, and they did not (gate-executed: **35 / 72**).
* Bestow (CR 702.103b) gets the same keyword transform query-side, applied to a *local clone* of
  `chars`, so the two derivations cannot drift the day `StubProvider` learns alt-cost casts.
* Reconfigure (`OOS-CARDS1-2`): the `replay_harness.rs` synth site carries CR 702.151a's
  `TargetCreatureWithFilter { controller: You, exclude_self: true }`. The equip repair was **not**
  copied — CR 702.6a has no "another", CR 702.151a does.
* `KNOWN_FALSE_OFFERS` deleted, and the whole excusal mechanism with it. Any refusal in that
  driver is now unconditionally fatal. That, not an assertion, is what proves the closure.

### The three things worth reading before trusting anything here

1. **The CR 303.4a gate was KEPT, deliberately.** It is now a redundant second check, and the
   reason is that `matches_enchant_target` is the **SBA's own** predicate (CR 704.5m). Keeping it
   at cast time is what guarantees cast-time and SBA-time agree — a *different* property from
   "the offer and the cast agree". Trading one for the other would have been a silent regression.
2. **The Reconfigure symptom was worse than its seed row said.** The row reads as an offer-side
   gap. In fact `abilities.rs`'s legacy `AttachEquipment` guard is an `if let Some(..)` over the
   empty `targets` vec, so a zero-target attach **passed validation, paid the mana, and fizzled at
   resolution with no error and no event**. The two pre-existing `mechanics_m_z/reconfigure.rs`
   tests could never have caught it: both accept `Err(_) | Ok(no attachment)`, so they are green
   under either outcome. T5.2/T5.3 are the strict versions; T5.5 is the discriminating one.
3. **The SR-5 keyword registry caught what both targeted test runs missed.** Two implementation
   dispatches each ran their own scoped `cargo test` and both were green; the full-workspace run
   then failed `core::keyword_registry::registry_sites_match_the_source_tree`, because
   `queries.rs` is now an Enchant **handling site** and had not been declared. A query that reads
   a keyword to decide what may be announced *is* behaviour. Targeted runs are not a substitute
   for the workspace run — this is the second batch in a row to learn that from a gate.

### The review cycle: 1 HIGH / 5 MEDIUM / 7 LOW — all 13 taken

The HIGH is **not the primitive**; the reviewer re-derived the 9-variant / 6-field equivalence by
hand and found it exact in both directions. The HIGH is a **card def inside this batch's own 13**:
`imprisoned_in_the_moon` declares `EnchantTarget::Permanent` for a printed *"Enchant creature,
land, or planeswalker"*. It was half-live but **unreachable** before this batch — `Permanent`'s
arm is a bare `true`, and no offer surface read the keyword — and PB-DX20 opens exactly that door,
so the browser now offers artifacts and enchantments and the cast succeeds. **Filed as
`OOS-DX20-10`, not fixed, and the reason is concrete**: `EnchantFilter` has `has_card_type`
(single) and `has_subtypes` (an OR over **sub**types) but no OR over card *types*, so the printed
line is inexpressible today, and adding a field to `EnchantFilter` moves HASH. A wrong-way-round
deviation pin (the PB-DX19 `deviation_animated_nexus_...` precedent) tells the successor to invert
it, and the pin is a **roster**, so a second instance cannot appear silently.

The most useful MEDIUM: **T1 proved less than the plan claimed**. It asserts `offer == cast`, but
post-fix both sides run the *same* synthesized requirement, and the only Aura-specific cross-check
left can only reject and iterates `Target::Object` only — so T1 was blind to strict **narrowing**
and to every **player-side** error. `Permanent → TargetCreature` and `CreatureOrPlaneswalker →
TargetAny` (the exact mistake the plan warns against in bold) would both have shipped green. The
fix is an exact-shape pin over all 9 arms plus expected accepted-candidate sets. **A differential
probe between two consumers of one function proves consistency, not correctness** — that is the
durable lesson, and it generalises to every "the two cannot drift" claim this project makes.

### Numbers

* Tests **4,373 → 4,388 / 0 / 5** (+15), `--workspace --no-fail-fast` to a file, pre-edit baseline
  measured on this branch BEFORE any edit, residual list empty.
* **PROTOCOL 35 / HASH 72** gate-executed and unmoved. **0 card-def lines** (`git diff --numstat
  -- crates/card-defs/` empty); coverage unmoved at **1,133 / 1,803 = 62.8%**, proven by a
  regeneration whose body came back byte-identical apart from its date/sha stamp.
* `clippy -D warnings`, `cargo fmt --check` and `tools/check-defs-fmt.sh` (1,803 defs) all clean.

### Seeds

**CLOSED**: `OOS-CARDS2-4` (HIGH), `OOS-CARDS1-2`. **Narrowed**: `OOS-SIM4-2` (the Aura clause
only), and `OOS-SIM5-4`'s recorded blocker — *"needs an engine query; `get_enchant_target` is
`pub(crate)`"* — is now **stale**, because `queries.rs` answers it and `TargetPlan::Unsatisfiable`
is reachable for Auras for the first time. **Filed**: `OOS-DX20-1..10` in
`docs/audits/decision-point-audit.md`. The one to read first is **`OOS-DX20-10`** (the HIGH);
after that, **`OOS-DX20-7`** — a roster gate on "Activated + `AttachEquipment` ⇒ non-empty
`targets`" would have caught `OOS-M11-10` and `OOS-CARDS1-2` by construction and would catch the
next one.

### For the collector

Branch `feat/pb-dx20-the-offer-layer-cannot-see-a-keyword-carried-target-`. Plan
`memory/primitives/pb-plan-DX20.md`, review `memory/primitives/pb-review-DX20.md`. The v3 queue
memo's §4 row 2 is shipped and can be struck; **next dispatch is PB-DX21** (CR 508.1, attackers
declared without limit, `OOS-M11-9`). No wire change, so no downstream re-pin is owed.

## Worker Handoff (PB-DX32, `scutemob-197`) — the fuzzer's output starts meaning something

**Three seeds closed (`OOS-SIM3-3`, `OOS-SIM3-4`, `OOS-CARDS2-3`), one PARTIAL (`OOS-SIM3-2`).**
Rank 19 of the v3 queue (`memory/primitives/seed-rerank-2026-08-02.md` §4), row 3 of
`docs/mtg-engine-feedback-engineering.md` §2.3 — **promoted from rank 19, user-approved
2026-08-03**. Plan: `memory/primitives/pb-plan-DX32.md`. Full stage-by-stage evidence including
every revert proof: `memory/primitive-wip.md`. Review: `memory/primitives/pb-review-DX32.md`.

### Stage 0 first: every baseline this batch could have quoted was dead

PB-DX22 (`95f53b78`) changed the deal, so no SIM-3 or SIM-5 number survives, and `OOS-DX22-13`
records that several of them were a five-game sample in the first place. Everything was
re-measured at HEAD **before any edit** and committed:
`memory/primitives/pb-dx32-measurement-head-{fuzzer,harness}.txt`.

| measurement at HEAD | value |
|---|---|
| workspace tests, this branch, pre-edit | **4,358 / 0 / 5**, residual empty |
| fuzz violations, 20 games × 200 turns | **426** = 301 `no_orphaned_tokens` + 114 `player_consistency` + 11 `attachment_validity` |
| bot rejections | **542 / 23,613 commands = 22.953‰** (5 games); **1,995 / 94,467 = 21.118‰** (20 games) |
| `RandomBot` wasted taps | **1,986 / 2,641 = 75%** (5 games); **8,423 / 10,720 = 78.6%** (20 games) |
| violations deduped by `(check, description)` | **94 → 20** (4.7×) |
| leaked tokens in the FINAL state | **0**, on 5 seeds and again on all 20 |
| deck pool | `all_cards()` **1,803** / `Complete` **1,133** / commander pool **90** |

**`OOS-SIM3-4`'s "929 of 938" was both stale and a sample.** At HEAD the orphaned-token class is
**70.7%** of the run, and `player_consistency` is a second class at **26.8%** that no seed row
records at anything like that size. Every figure this handoff quotes names its game count.

### What shipped, by criterion

* **(a) SR-38 becomes a run-level invariant.** `GameResult` carries `rejection_count` and a bounded
  `rejections` sample; the fuzzer prints a class histogram and **exits 1** above
  `MAX_BOT_REJECTION_PER_MILLE`. `rejection_count` was already unconditional — only the *sample*
  needed the new `MAX_SAMPLED_REJECTIONS = 8` cap for the journal-off (fuzzer) path, since
  `results` retains every game's `GameResult` and a 256-cap at `--games 1000` would hold 256,000
  cloned `Command`s.
* **(b) the waste instrument is promoted, not copied.** `WasteTally` folds `tap_runs` /
  `wasted_tap_runs` / `wasted_taps` / `total_taps` / `mana_pools_emptied` at the two sites that see
  exactly the journal's command stream — so the streaming fold and `sim5_bot_cast_discipline.rs`'s
  journal walk are provably the same measurement, and `metrics_of` was **kept** as the equivalence
  oracle rather than deleted. `OOS-SIM2-1` is named at the pin.
* **(c) the noise floor.** `check_no_orphaned_tokens` output is split into a `transient_violations`
  bucket at the point of collection; `--stop-on-error` and the crash-report writer key on the hard
  bucket only; counts are deduped by `(check, description)` and printed raw **and** distinct for
  both buckets. Hard violations **426 → 125**, games with ≥1 hard **16/20 → 6/20**, crash files
  **16 → 6**.
* **(d) `OOS-CARDS2-3`.** `CORPUS_DEFS`/`CORPUS_COMPLETE`/`COMMANDER_POOL` pinned exact in both
  directions, the pool recomputed by mirroring `deck.rs`'s own filter clause-for-clause.
* **(e) decision-point runtime coverage.** `crates/simulator/src/decision_coverage.rs` carries
  **ids only**; the roster is kept single-source by a **source gate appended to the existing**
  `decision_gate.rs` (`BASELINE` and `MAX_AUTO_CHOSEN_COMPLETE_UNION = 80` untouched).

### The split is only honest because of what replaces it — read this before widening it

Reclassifying a violation class as non-halting is exactly what SIM-3's `stack_consistency`
withdrawal was about, so the batch bought the right to do it in two ways. First, the **strictly
stronger end-state property** is asserted in the **hard** bucket at both terminal paths
(`invariants::check_no_leaked_tokens` — a new pure-state check, deliberately *not* in `check_all`),
and it measured 0 across all 20 games. Second, **nothing else was reclassified**:
`player_consistency` and `attachment_validity` stay hard, on purpose.

**So criterion (c) is met in its literal wording and not in the colloquial one.**
`--stop-on-error` still halts — now on seed 2's `player_consistency` at turn 123, two games in.
The fuzzer is not yet a clean smoke test; the reason has moved from a false-positive check
(`OOS-SIM3-1`, withdrawn) to a known-transient one (closed here) to an **undiagnosed** one
(`OOS-DX32-1`, filed). That is progress, and stating it as anything more would be false.

### Findings the batch did not go looking for

* **`HeuristicBot` can never leave a tap run open** (`OOS-DX32-5`). It scores `TapForMana` at 0, so
  every tap it makes is an auto-tap prefix inside one atomic sequence. The equivalence probe's
  revert stayed **green** on the plan's own fixture (§7 R8's anticipated failure, hit for real) and
  had to be given a human-`submit()` fixture. Any "0 wasted taps" measured on that bot is weaker
  evidence than it reads as.
* **Every threshold needed two constants** (`OOS-DX32-4`). The debug/25-turn gate and the
  release/200-turn binary measure genuinely different populations, and the gate measures *higher*
  in both cases (31.081‰ vs 21.118‰; 89% vs 78.6%) — which is also the proof the duplication is
  forced rather than a dodge, since an evasive twin loosens for a *lower* number.
* **The SR-38 channel produced its ranked defect list on the first run** (`OOS-DX32-9`):
  `InsufficientMana` (`OOS-SIM6-3`), `InvalidTarget` (`OOS-SIM5-5`), the blocker-refusal family
  (`OOS-SIM5-3`, the largest), and CR 303.4a Aura casts (`OOS-CARDS2-4`). Close any of them and
  ratchet the constant down.
* **Runtime coverage is budget-dependent** (`OOS-DX32-3`): 4 of 5 served rows at the CI gate's
  10×60 debug budget, 5 of 5 at the binary's 20×200 release budget. Both recorded, not just the
  better one.

### The review cycle: 0 HIGH, 8 MEDIUM, 10 LOW — all 18 taken, and one was proven by experiment

The reviewer **had no shell and executed nothing**, and said so; its revert checks were source
inspection. The coordinator closed that gap by executing three things directly, and one of them
found a live hole:

* **`OOS-DX32-6`, the block-comment gate hole.** Wrapping one `UNOBSERVABLE_ROW_IDS` tuple in
  `/* … */` removed it from the **compiled** roster (`ROW_COUNT` 22 → 21) while the source gate's
  `quoted_strings` still found its literals. **The gate stayed green — and so did all 12 probes in
  `pb_dx32_fuzz_output.rs`.** A row silently vanished and nothing in the workspace noticed. Fixed
  (`strip_block_comments` + a raw-count assertion that also catches a duplicated id) and the same
  experiment now fails `left: 21 right: 22`, re-run by the coordinator after the fix. **The open
  half: `strip_line_comments` is used by the other source-reading tests in `decision_gate.rs`, and
  those were not audited for the same hole.** Third appearance of this class in this file's family.
* **T6.1(b) and T4.2 verified genuine** by execution, matching the runner's quoted messages. T4.2's
  revert needed `let _ = state;` as well as `#[allow(unreachable_code)]` — the first attempt failed
  to compile under `-D unused-variables`, which is plan §7 R7's exact class and is worth repeating:
  **a revert whose rebuild failed proves nothing.**
* **M1 — the batch zeroed a diagnostic in its own precedent file.** `local_game_playthrough.rs`
  still split `game.violations()` on `no_orphaned_tokens`, a string that can no longer match, so
  its run report would have printed `0 transient-token reports` forever. Nothing asserted on it, so
  no test went vacuous — but it is the `OOS-DX22-13` "a number's meaning changed silently" class,
  committed in the file the batch cites as its own pattern. Fixed to read
  `transient_violations()`; verified by execution (seeds 7/42 now print 12 and 4).
* **M6 — the thresholds cited the 5-game sample while the 20-game artefact sat in the same commit.**
  Re-quoted from the 20-game run. `MAX_RANDOM_BOT_WASTED_TAP_PCT` **kept at 85** with the real
  headroom (6.4 points over 78.6%, not the ~10 the 75% figure implied) stated as a deliberate
  choice, because the fuzzer is not run-to-run deterministic for long games
  (`OOS-M11-3`/`OOS-DP3-9`) and a single point estimate should not be shaved to the wire.

### For the collector

* Tests **4,373 / 0 / 5** (+15 over the 4,358 pre-edit baseline), full `--workspace --no-fail-fast`
  to a file, residual list **empty** — re-run independently by the coordinator after the fix cycle.
* **PROTOCOL 35 / HASH 72 gate-EXECUTED** (`--test core -- protocol_schema hash_schema`, 38 tests
  green) and unmoved; `PROTOCOL_VERSION = 35` at `protocol.rs:360`, `HASH_SCHEMA_VERSION = 72` at
  `hash.rs:743`.
* **0 wire, 0 engine source, 0 card defs** — `git diff main..HEAD -- crates/engine/src/
  crates/card-defs/ crates/card-types/ crates/view-model/` is EMPTY. Coverage unmoved
  **1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py` to a body identical
  apart from its own git-sha/date stamp lines, then reverting the churn.
* **`tools/` is exactly one file, `+1 -0`** — a `..Default::default()` in a `#[cfg(test)]`
  construction site. `cargo test -p play-server` 78/0 unmoved.
* The only engine-side file touched is the **test** `crates/engine/tests/core/decision_gate.rs`
  (appended to).
* **`memory/primitives/seed-rerank-2026-08-02.md` is untouched** — striking row 19 is the
  coordinator's at collect.
* Successor candidate: **`OOS-DX32-1`** — diagnose `player_consistency` (is it ever true *at
  rest*?). It is the last thing standing between the fuzzer and a usable smoke test, and it is
  26.8% of a run.

## Worker Handoff (PB-DX22, `scutemob-196`) — the fuzzer becomes a real instrument

**Three seeds closed: `OOS-UI2-1`, `OOS-SIM3-1`, `OOS-SIM1-4`.** Rank 4 of the v3 queue
(`memory/primitives/seed-rerank-2026-08-02.md` §4), row 1 of `docs/mtg-engine-feedback-engineering.md`
§2.1. Plan: `memory/primitives/pb-plan-DX22.md`. Full stage-by-stage evidence:
`memory/primitive-wip.md`. Review: `memory/primitives/pb-review-DX22.md`.

### The mandatory pre-plan measurement, and what it settled

The brief made one measurement mandatory *before* acceptance evidence: does a bot cast its
commander around turn 12-24 through SIM-1's command-zone loop (`legal_actions.rs:675-693`), or
is the offer suppressed? It was run first, at HEAD, and committed as the branch's **first**
commit (`891d346c`, raw output `memory/primitives/pb-dx22-measurement-head.txt`) so the
ordering is checkable rather than claimed.

**Answer: SUPPRESSED, and `OOS-SIM1-4` is the cause.** 5 games / seed 1 / `--max-turns 200`:
`commander_ids` populated **0/4 in every seat of every game**, **zero**
`CommanderCastFromCommandZone` in ~56,800 commands, first `SpellCast` at turns
154/143/151/153/151. The provider's own filter says it — "the zone is NOT the filter;
`commander_ids` is" (CR 903.8; CR 408.1 is why) — and `fuzzer.rs` never populated it. So the
brief's disjunction resolves to its second branch: **SIM-3 did not measure a pre-SIM-1 build**,
and `OOS-SIM1-4` and the missing commander cast are ONE defect. That collapsed the batch's
sizing: no provider change was needed. Seed 2's turn 143 reproduces `OOS-SIM3-1` exactly.

### What shipped

* **`crates/simulator/src/fuzz_setup.rs` (new).** The fuzzer's pregame build, lifted out of
  `src/bin/fuzzer.rs`. **This file exists because Cargo compiles `src/bin/*.rs` as its own
  crate, so no integration test could `use` the fuzzer's state build** — which is exactly how
  `crates/simulator/tests/local_game.rs::build_state` came to be a hand-written copy ("Mirrors
  `mtg-fuzzer::run_single_game`'s builder logic") carrying the identical CR 903.6 defect. Both
  callers now share `place_registered_deck`, which does the placement **and** the registration
  in one `if let`, so they cannot separate again.
* **Deliberately NOT in `setup.rs`.** Every play-server seed pin is a function of `setup.rs`;
  keeping the fuzzer's build in its own file makes "this batch cannot move a play-server pin" a
  property a reviewer checks from the diff's **file list**, not its contents. It held:
  `cargo test -p play-server` 78/0, nothing re-derived, nothing adjusted.
* **CR 103.3 / 903.6 shuffle** off the game's own `StdRng`, interleaved per seat
  (`deck₁, shuffle₁, deck₂, shuffle₂, …`) exactly as `setup.rs` does. There the interleaving is
  load-bearing; here it is free, and the free choice is the one that keeps the two paths the
  same shape.
* **CR 903.6 registration + CR 903.9b `register_commander_zone_replacements`.** Both are
  required and they fail differently: omit the second and CR 903.9a still works (it is an SBA
  keyed on `commander_ids`) while CR 903.9b silently does not exist and any count of
  `CommanderZoneRedirect` reads zero **vacuously**. Proven by isolation, not argued: under a
  revert deleting only that call, P6 reddens and P7 stays green.
* **`tests/local_game.rs` fixed in both halves.** Stage 3 rewired it onto the shared helper; a
  follow-up (`eb60cc80`) found it still lacked the CR 903.9b call — the same half-built
  Commander game one link down — and closed it with its own probe.
* **The fuzzer reports its own census.** A constant-size `MechanicsTally`
  (`local_game.rs`, surfaced by `GameDriver::run_game_with_mechanics`) folded from events
  already in hand, so `record_journal: false` and the fuzzer's memory profile are untouched.
  It is **not** a `GameResult` field: `tools/play-server` constructs one and `tools/` was off
  limits. This exists because the review caught the batch committing its "before" and deleting
  its "after" — see the lesson below.

### The A/B, both sides reproducible from committed code

`--games 20 --seed 1 --max-turns 200 --threads 1 --profile fuzz`. After-side raw output:
`memory/primitives/pb-dx22-measurement-after-fixcycle.txt`. **The before side requires building
the merge base** — the shipped binary cannot produce it.

| | before | after |
|---|---|---|
| `CommanderCastFromCommandZone` (CR 903.8) | **0** (~56,800 commands / 5 games) | **36**, in 16/20 games |
| `CommanderReturnedToCommandZone` (CR 903.9a) | 0 | **13** |
| seats with commander damage (CR 903.10a) | 0 | **16/20 games**, max **31** (past the 21 threshold) |
| `CommanderZoneRedirect` (CR 903.9b) | 0 (no mechanism) | **0** (mechanism exists, no game triggered it — `OOS-DX22-9`) |
| first `SpellCast` turn | 143-154 | **3-29**; library-only **5-29**, median 17 |
| `SpellCast` total | 121 / 5 games | **670** / 20 games |
| violations | 1,519 | **426** (301 `no_orphaned_tokens` / 114 `player_consistency` / 11 `attachment_validity` / **0** `stack_consistency`) |
| wins / errors | 9 / 11 `MaxTurnsReached` | **20 / 0** |
| avg turns · cmds/turn | 191.7 · 58.8 | **103.4 · 45.7** |

### Three deliberate remaining divergences from `setup.rs` — read these before "fixing" one

The fuzz path is **not** a `build_initial_state` replacement and was not made into one:
no opening hand (CR 103.5, **`OOS-DX22-1`**), no `validate_deck` (CR 903.5a/903.4,
**`OOS-DX22-5`**), no `DeckSource`. `OOS-DX22-1` is measured, not assumed: the first
`Command::PlayLand` is turn 1-7 on all 20 seeds, so land supply is not the limiter — a seat
starts with zero cards and draws one per *personal* turn, so it has ~T/4 cards by game turn T
at four seats. That is the reason the band is 3-29 rather than 3-12.

### What the repaired instrument immediately found

**`OOS-DX22-8`** — `attachment_validity`: `Object ObjectId(532) attached to ObjectId(677) which
doesn't exist`, 11 violations across seeds 5, 9 and 15 (the batch first recorded "×3, seed 5"
off the binary's 5-game print cap and under-counted 4×). Repro
`cargo run --profile fuzz --bin mtg-fuzzer -- --replay 5 --players 4 --max-turns 200`;
check `invariants.rs:386`; CR 400.7 / **704.5m** (Aura → graveyard; 704.5n is Equipment, which
unattaches and *stays*). **Pre-existing — 0 engine lines in the branch diff** — and deliberately
unfixed. Transient, one turn per game, and all 20 games still ran to a winner, so it is a live
false-positive candidate of the `OOS-M11-7` SBA-lag family SIM-3 withdrew: classify it before
fixing it. A plausible mechanism this batch uniquely enabled: 13 CR 903.9a returns means
commanders changed zones in fuzz games for the first time, and a CR 400.7 zone change is exactly
the orphaning event.

### Durable lessons

1. **A revert-proof written read-only is a hypothesis** (`OOS-DX22-11`). Two of the plan's ten
   predictions were false when executed: P4's stated revert leaves it green (seeds 1 and 2 draw
   different *decklists*), and P9's reddens 1 of 4 seeds, because a registered commander is cast
   from the **command zone** and so is not gated by library order — Stage 3's fix partially
   masked Stage 2's from that probe. Both gates were left where the plan put them and the real
   discrimination was executed and recorded instead.
2. **A universal negative must be measured over its own denominator.** `bin/fuzzer.rs` prints
   per-violation detail for only the first five offending games, and the batch published
   "not one of 426 is `stack_consistency`" off 94 printed lines. The claim survived the real
   tally — but the sample had projected `player_consistency` at ~1% where it is **27%**. Every
   historical "check X never fired" claim in this project came off that same cap
   (**`OOS-DX22-13`**).
3. **Committing the "before" and deleting the "after" is the same defect the batch exists to
   close.** The headline numbers first came from a scratch `examples/` file that was deleted;
   the review called it, and the repair was to make the fuzzer print them. Re-measured, every
   published number matched to the digit — the instrument was accurate, it was *unreproducible*.
4. **A source gate that greps a file body is satisfied by its own comments.** P11 matched
   `player_commander` in the doc comment explaining the rule, so deleting the real call left it
   green while six behavioural probes reddened. It now strips line comments — safe rather than
   merely stricter, because all four genuine placers were checked for a real call first.
5. `memory/gotchas-infra.md`'s stale-binary trap fired **three times** in this batch: a revert
   that fails to compile under `-D warnings` makes `cargo test` run the *previous* binary and
   report a pass. Check for the `Compiling mtg-simulator` line before trusting any red.

### For the collector

Every recorded fuzz seed predating this merge is dead (**`OOS-DX22-7`**); the docs say "the
PB-DX22 merge, `scutemob-196`" in four places where a merge sha would be better once one exists
(`bin/fuzzer.rs` module doc, `workstream-state.md`'s `--seed 504` annotation, the audit §8.1
banner, feedback-engineering §2.1). `memory/primitives/seed-rerank-2026-08-02.md` is
**untouched by design** — the coordinator strikes the PB-DX22 row at collect, and §2.4's "one
open measurement this task could not settle" is settled by the answer above.

## Worker Handoff (UI-6, `scutemob-194`) — the whole-library search view (G9, CR 701.23a)

**G9 of `memory/playtest-triage-2026-08-02b.md` CLOSED, both halves — the LAST row of its
successor table, so that triage is now fully dispatched.** The playtest said *"only showed legal
basic lands — should be able to view whole library when searching — current view is too
cumbersome — should be a list which you can check"*. **The filter was never the defect**:
`candidates` IS the engine's answer space and `handle_answer_effect_choice` refuses anything
outside it, so widening it would be offering illegal answers (SR-38). What was missing is
CR 701.23a's **look** — *"To search for a card in a zone, look at all cards in that zone (even if
it's a hidden zone)"*. So the look and the pick are now two lists, sent separately and rendered
separately. **0 engine lines and 0 simulator lines** (`git diff main..HEAD -- crates/` empty);
PROTOCOL **35** / HASH **72** gate-executed and unmoved. Tests **4,345 / 0 / 5** full workspace
`--no-fail-fast` to a file (+4 on this branch's own pre-edit baseline of 4,341 — two HTTP probes,
one frontend source gate, and the `/review` cycle's restriction probe; the Invariant-7 gate was
**renamed**, not added), residual list empty. `fmt`, `clippy --workspace --all-targets -D warnings` and `tools/check-defs-fmt.sh`
(1,803 defs) all clean. **0 card-def lines**, coverage untouched at **1,133/1,803 = 62.8%**.

**The Invariant-7 gate went red on purpose and that is the interesting part of this batch.**
`test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places` is now
`test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places`, and the re-pin is argued
at the pin site in CR terms rather than being a number bump. Three things bound the new read, and
each is a constraint a careless implementation would have missed:

1. **The searcher's own library only.** `player` is `PendingDecision::player`, which
   `api.rs::seat_view` has already filtered to the viewing seat, and the engine's search effect
   builds candidates from `ZoneId::Library(p)` for that same `p`.
2. **Sorted by NAME, never in library order.** CR 701.23a grants a look at the cards; it does not
   grant a look at the shuffle CR 701.23e exists to protect, and Architecture Invariant 7 names
   library *order* explicitly. Sending `Zone::object_ids()` verbatim would leak draw order to the
   seat that just failed to find — a real defect, in the *right* client rather than the wrong one.
3. **CR 121.1: "all cards in that zone" is not always the whole library.** Under an opponent's
   Aven Mindcensor the searcher *"searches the top four cards instead"*, so the entitlement is
   four cards. This was **found by the `/review` cycle, not by the implementation** — the first
   draft enumerated the library unconditionally, which would have shown 89 cards with 85 marked
   "look only". `library_look_cards` now calls the same `apply_search_library_replacement` the
   engine's search path calls and narrows through the same `Zone::top_n`. This makes it the
   **second** place in `view.rs` that restates an engine rule rather than delegating
   (`action_modes` is the first and says so); it is recorded the same way, and every divergence
   is in the narrowing direction.

**Why the gate had to become a needle SET, measured rather than argued.** The new read spells
`.zone(`, not `.objects()` — and with the channel in the tree, `.objects()` in `view.rs`'s
production region is **still exactly 2**. The pre-UI-6 single-needle gate would have stayed
**green** while a new hidden-information channel opened underneath it. That is MR-M11-01's lesson
arriving a second time in the same file, three sessions later. Worse, the *first* revert run
against the two-needle re-pin replaced `state.zone(..)` with `state.zones().get(..)` — the same
channel one accessor over — and the draft went green. So five needles are now pinned at **0**
(`zones()`, `objects_in_zone(`, `player(`, `object(`, `players()`), two of them added by the
`/review` cycle, which pointed out that the first draft closed the *plural* of one needle and the
*singular* of another while leaving each one's opposite number open. Each zero-pin was proven red
by an executed revert that fires it alone. It is still an enumerated set and not a proof about
every raw read; both the gate and `question_card_label`'s doc say so in those terms.

**The new channel got its OWN behavioural gate, and the sibling could not have covered it.**
`test_ui6_a_foreign_seat_never_receives_the_whole_library_look` mirrors the UI-1 scry gate's
construction (move `PlaySession::human`, not the decision — `advance()` refreshes `pending`
straight back), but it exists separately because the scry gate's raw-body needle is the
`looked_at` **key** and a search payload has none. Proven red by executing the revert: deleting
`seat_view`'s `pending.player == human` filter puts seat 1's entire library, **every card named**,
into seat 2's body, and the assertion quotes it.

**The fixture is new, and the reason is worth carrying.** UI-1's `ui1_install` search is Diabolic
Tutor — *unrestricted*, so its candidate set IS the whole library and `all_cards` would be
set-equal to it. **A fixture like that can never exhibit a look-only card**, so it could never
falsify the claim under test. UI-6 uses **Solemn Simulacrum** (`{4}`, colourless, `Complete`,
ETB `SearchLibrary` with `basic_land_filter()`) at `main_deck[0]` of a mono-black deck, plus six
distinct MV≥6 mono-black fillers whose only job is to sit in the library as cards the search
cannot find. At `UI6_SEED` (= `UI1_SEED`) that yields **89 in library / 33 findable / 56
look-only**. Its `may_fail_to_find` is `true`, the opposite of the UI-1 probe's `false`, so
between them both CR 701.23b/d branches are now exercised over HTTP.

**Browser-verified live** (headless Chromium, playwright-core, release server on :3046, seed 116
→ Three Visits at turn 9 — the UI-4 tuple reused): 89 rendered rows, 33 pickable buttons, 56
look-only, a column list scrolling 2082px inside 224px, the look-only row a `DIV` whose forced
click produced **0 POSTs and 0 selection** with Confirm still disabled, and a non-default pick
posting `{"found":97}` against a server default of `10`. 0 `pageerror`s, 0 error strips,
`command_count` 341 → 342. **Path correction for the next worker: the chromium binary is at
`~/.cache/ms-playwright/chromium-1228/chrome-linux64/chrome`**, not the `chrome-linux` the UI-4
handoff records — that path no longer exists and cost a launch failure.

**CR 400.7 trap, for whoever repeats the browser check**: Three Visits puts the found card onto
the battlefield, where it is a **new object** with a new `ObjectId`, so "the clicked id is on the
battlefield" is false even on success. The captured POST body is the discriminator, not the board.

**The `/review` cycle found 6 and all 6 were taken**, one of them a real rules defect (the CR
121.1 restriction above, live-reachable because `aven_mindcensor.rs` declares no `completeness`
field and is therefore `Complete` by the `#[default]` derive — the same generator PB-DX3b and
PB-DX4 both hit). Two more were gates whose message overstated what they asserted: the
`look-tag` needle was satisfiable by the **stylesheet alone** (deleting the `<span>` left it
green), and `library_look_cards` said it "asserted" a premise it only states in prose. The
restriction fix is pinned by `test_ui6_the_look_narrows_with_a_search_restriction` on a second
fixture — seat 2 holds Aven Mindcensor, seed **29**, read off a 300-seed sweep — asserted against
the engine's own `top_n` rather than against the literal `4`, and proven red by revert (89 ids
vs 4).

**Frontend**: `SearchPicker` is a scrollable checkable **list**, not a wrapped button grid — a
fixed left edge is what makes ~99 rows scannable. Rows are the union of `allCards` and
`candidates`, each carrying `pickable = (id in candidates)`; a look-only row is a plain `div`
with a visible `look only` tag, **not a disabled button**, because a disabled control reads as
"not right now" and CR 701.23a's distinction is permanent. `select` and `emit` both re-check
membership (the emit guard explains the refusal in CR terms rather than letting the server's 400
read as "request failed"). A one-click *"hide the N I can't find"* filter is **off by default** —
defaulting it on would restore the exact behaviour that was complained about. Gate:
`test_frontend_search_picker_looks_wider_than_it_picks`, proven red three ways (render candidates
only; accept a look-only id; make the look-only row a disabled button).

### Seeds filed (UI-6)

* **`OOS-UI6-1`** — *The picker opens on a wall of look-only rows.* `all_cards` is
  name-sorted, so in a Swamp/Forest deck every findable card is late in the alphabet and the
  first screen is unpickable cards (observed live: the top 10 rows at seed 116 are `Archetype of
  Endurance` … `Collector Ouphe`, all look-only). The filter box and the "hide the N I can't
  find" toggle each fix it in one action, so this is UX ranking, not correctness. Two candidate
  treatments, both with a cost stated: sort findable-first (loses the single A–Z scan the sort
  exists for), or scroll to the first findable row on open (keeps the sort, costs an effect).
  Deliberately not decided by this batch.
* **`OOS-UI6-2`** — *`all_cards` is filled in at the `SearchLibrary` arm only, and nothing
  gates that.* Any future `PickOne` question gets an empty look list and the client silently
  falls back to candidates-only. That is the safe direction, but a new arm that *should* carry a
  look entitlement would ship without one and no test would notice. A roster gate over the
  `EffectChoiceQuestion` variants routed through `PickOne` would close it — the same shape as
  SR-5's keyword registry.
* **`OOS-UI6-3`** — *The client's graveyard-search union branch is unreachable and therefore
  untested.* `SearchPicker` merges candidates absent from `allCards` because
  `also_search_graveyard` puts graveyard cards in the answer space that are in no library.
  Measured: `finale_of_devastation` is the **only** def with `also_search_graveyard: true`, and
  it is `Completeness::partial`, so `validate_deck` rejects it — the branch cannot fire today.
  Fold into the R7 frontend harness when it exists rather than building a fixture for it.
* **`OOS-UI6-4`** — *The field is named `all_cards`, which overstates it in one case.* It is
  the **library**, narrowed by CR 121.1; a graveyard search's candidates are not in it. The name
  is the triage's own recommendation and the doc is precise, so this is naming, not behaviour —
  `library_cards` or `look_at` would say it. Renaming is a DTO change with a frontend prop to
  match; cheap, and worth doing only alongside another change to this shape.
* **`OOS-UI6-5`** — *The Invariant-7 count gate is still an enumerated needle set.* Seven
  needles now, five of them zero-pins, two of those added because the first draft's own revert
  defeated it with a synonym. A read through an accessor nobody listed stays invisible. The
  durable fix is type-level — a wrapper `view.rs` must go through to reach `GameState` — not more
  needles, and it is the same limitation MR-M11-01 is about.
* **`OOS-UI6-6`** — *`library_look_cards` restates an engine rule and can go stale silently.*
  It is the second such site in `view.rs` (`action_modes` is the first). If the engine's search
  path stops calling `apply_search_library_replacement`, or starts restricting by something other
  than `top_n`, the look narrows wrongly with nothing to catch it. A shared engine query —
  `rules::queries::searchable_library(state, player)` returning the ids the search will actually
  consider — would let both the engine and the view read one implementation. That is an engine
  line, so it was out of scope here.

## Worker Handoff (ENG-2, `scutemob-193`) — targets in the event log (G7, CR 601.2c)

**G7 of `memory/playtest-triage-2026-08-02b.md` CLOSED, event-log half.** Before this batch no
cast/activate/trigger event carried its targets, and a **player**-targeting trigger emitted
nothing at all — the playtester watched a bot's Fell Specter hit them and the feed said only that
a triggered ability went on the stack. One additive `GameEvent::TargetsAnnounced` (discriminant
132) now fires at announcement time from all twelve stack-push sites, and the view-model renders
it. **PROTOCOL 34 → 35, HASH 71 → 72**, both gate-computed from the failing gate's own output on
this branch (the triage's "33" is stale — ENG-1 moved both after it was written). Tests
**4,341 / 0 / 5** full workspace `--no-fail-fast` to a file, residual list **empty**. **0 card-def
lines**; coverage unmoved **1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py`
to a body byte-identical below its self-dating header, not by an empty diff.

**Shape: option (2) of the triage's three, and the rejections are the useful part.** Option (3)
(the triage's own "cheaper third option" — widen `PermanentTargeted` to cover `Target::Player`)
was evaluated first, as the brief demanded, and rejected on a structural ground, not a taste one:
`PermanentTargeted` is **Ward's dispatch channel**, and `flush_sorted` — the reported defect's own
site — emits none, so widening it **structurally cannot reach the defect**. Option (1) (add a
`targets` field to each of `SpellCast`/`AbilityActivated`/`AbilityTriggered`) was rejected as
unfalsifiable: a forgotten site emits `vec![]`, which is indistinguishable from a genuinely
targetless announcement, so no gate can tell the two apart. Option (2)'s separate event makes
"announced nothing" and "announced no targets" the same observable, and the census gate below is
what keeps the site list honest.

**The census gate is the deliverable that outlives the batch.**
`crates/engine/tests/primitives/pb_eng2_targets_announced.rs::every_announcement_site_is_classified`
enumerates all 26 `SpellCast`/`AbilityActivated`/`AbilityTriggered` push sites from source and
requires each to be classified `ANNOUNCES` or `NEVER_TARGETS` **with a reason inline**. A new
emission site fails the test until someone decides which it is. Part 3 additionally pins that the
`NEVER_TARGETS` sites have not quietly grown targets.

**Where targets can actually come from: 8 sites, not 12.** The only places a `StackObject` acquires
non-empty targets today are two struct literals (`casting.rs:4532`, `engine.rs:3703`) and six
`.targets =` assignments (`abilities.rs:1395/1778/1993/8559/9181/10682`). All eight announce. Four
more sites are wired anyway — `copy.rs:474/699` (cascade, discover) and `resolution.rs:5463/6183`
(cipher-copy, suspend free-cast) — because those four hardcode `targets: vec![]` **unconditionally**
today, which is itself a bug (`OOS-ENG2-3`); wiring them now means the announcement is correct the
day that seed is closed, rather than being a second thing to remember. The ninth target-carrying
construction, `copy.rs:163` (`copy_spell_on_stack`), is deliberately **excluded**: CR 707.10, a
copy of a spell is not *cast*, so there is no CR 601.2c announcement to report.

**Invariant 7 is honoured by reusing the existing chokepoint, not by a new rule.** `private_to()`
stays `None` and `reveals_hidden_info()` needs no arm — correct, and reasoned rather than omitted:
CR 601.2c declares targets as part of putting the object on the stack, and CR 400.2 makes the stack
a public zone, so the *event* is public. The *identity* of an object target may still be private
(CR 708.2, a face-down permanent), which is a per-FIELD verdict a per-EVENT `private_to()` cannot
express — so it is decided in `event_view.rs`'s existing `card_or` gate, which routes
`card_name` → `may_name` → `redact::viewer_may_identify`. Player targets are never redacted.
`crates/view-model/src/tests.rs` proves both directions on a face-down permanent, with a
non-vacuity assertion on the omniscient view so the test cannot pass by rendering nothing.

**Downstream, and the "zero changes expected" claim, confirmed by measurement.** `event_view.rs`
gets the prose arm plus an `event_tier` entry (**neither is compiler-forced** — the tier match is
non-exhaustive, so a new variant silently lands in `Game` and the feed's `stack` filter would never
show it; that class is `OOS-ENG2-7`). `tools/tui/src/play/app.rs` gets an arm (its `_ =>
String::new()` would otherwise drop the line silently — class filed as `OOS-ENG2-8`).
`tools/replay-viewer/frontend/src/lib/eventFormat.js` gets a raw dev-tool line. **`tools/play-server`
carries zero source changes and the play frontend zero changes** — verified, not assumed:
`grep "GameEvent::" tools/play-server/src` returns only doc comments, and the +60 lines in
`main.rs` are entirely inside `mod tests`. The existing `event_view_for` → `EventView` → JSON
pipeline carries the new variant unmodified. `state/hash.rs` is the one arm the compiler demands.

**Rider taken while in the file (§4.5/§4.6).** `GameEvent::TargetsChanged` (CR 115.7) had **no**
`event_view` arm at all and rendered as the bare kind string; it now renders `old → new` through
the same `card_or` gate (`OOS-ENG2-4` filed and closed in the same breath). And three shipped
comments cited **CR 108.1** — the *Oracle-text* rule — for "a player target is public"
(`OOS-ENG2-5`).

**That citation rider was itself wrong once, and the correction is the lesson.** The first
replacement was `CR 102.1 / 115.1 / 400.2`. The `/review` cycle caught it: 102.1 merely defines
what a player *is*, and 400.2 is about whether *cards' faces* are visible in a *zone* — a player is
neither a card nor a zone. Only one rule actually says a player can be a target, and it is the one
in this task's own title: **CR 601.2c**, *"an appropriate object **or player** for each target"*.
Shipped chain is now `CR 601.2c / 400.2` at all **six** sites (the original count of three was also
wrong). **Replacing a wrong citation with a plausible one is not a fix** — verify the replacement
against the CR text, which is what the reviewer did and the implementer did not.

### The crash, and what verifying inherited work actually caught

The first worker process died after committing stage E (PROTOCOL/HASH) but before verification and
close-out. The relaunched worker was told to trust the commits and verify them. The engine work
survived that scrutiny intact — but **every doc-comment count in it was wrong**, and one gate was
weaker than its own comment admitted:

| Finding | What it was | Why it mattered |
|---|---|---|
| The gate's Part 2 was `body.contains("push_target_announcement(")` | Two functions carry **two** announcement sites each (`flush_sorted`: T6 modular, T7 main; `resolve_top_of_stack_inner`: S4 cipher, S5 suspend), so either call alone satisfied it | **T6 has no behavioural probe anywhere in the suite** — deleting it was invisible to all 4,341 tests. Now counts per function; **proven red by executing the revert** (deleting the T6 call fails `left: 1, right: 2`; the old assertion passes on that same tree) |
| `events.rs` SR-4 comment said "8 call sites" | There are 12 | The invariant it asserted was **true at all twelve** — only the count rotted. Restated without a count, pointing at the gate as the thing that keeps it true |
| `event_tier` comment claimed the match "has no `_` arm" | The `_ => EventTier::Game` default is three lines below; the paren was also unclosed | A comment that argues from a false invariant is the PB-DX19 failure mode verbatim — that batch's HIGH survived 4.5 months behind exactly this |
| `FROZEN_HISTORY_PREFIX_DIGEST` in both schema gates | Values moved; no ENG-2 line appended to their running attribution logs | The logs are append-only by convention; a silent value move breaks the audit trail |

**Generalisable**: the crashed worker's *code* was trustworthy and its *prose about the code* was
not. Counts in comments are the first thing to rot and the last thing anyone re-derives.

### Browser verification (live headless Chromium, independently re-run)

Both runs are recorded as ESM task comments. The second was run by the relaunched worker precisely
because the first rested on evidence nobody could re-inspect.

| Run | Seed | Observed in the DOM |
|---|---|---|
| Original (comment 1310) | 12, :3041 | `Scrawling Crawler targets Human-1` ×3, kind `TargetsAnnounced`, tier stack, player Bot-4, immediately after the `AbilityTriggered` line; turn 26, 250 pass clicks, 1,984 feed lines |
| Re-run (comment 1314) | **193193**, :3047, release build | **`Omnath, Locus of the Roil targets Human-1`**, class `feed-line tone-plain tier-stack`, player **Bot-2**, turn 14, 129 pass clicks, 1,416 feed lines, **0 uncaught page errors** |

The re-run is a bot-controlled **triggered** ability (Omnath's ETB, "deals damage to any target")
naming the **human player** — the exact Fell Specter class G7 reports, which emitted nothing at all
before this batch. Two other DOM lines named objects (`Shrieking Drake targets Shrieking Drake`,
`… targets Foundry Street Denizen`), so the sentence is not a fixed string. Corroborated at the
wire level by an HTTP-only drive of the same seed finding the identical line in the human seat
payload's `events` array.

**Recipe for the next batch that needs this** (two things cost the re-run most of its time):
the page boots with **no game** — deal a table through the real pregame controls (fill the two
`input[inputmode="numeric"]`, click `button.primary`), and **"Use the default" in `DiscardPicker`
FILLS the selection, it does not submit** (`DiscardPicker.svelte:78`), so clicking it in a loop
spins forever — click `button.secondary` then `button.confirm`. Driving over HTTP *instead of*
clicking does not work for a feed check: `GET /api/game` drains the event cursor, so an external
driver steals the events the browser was going to render.

### Seeds

Filed: **`OOS-ENG2-1`** (MEDIUM — CR 702.21a: `flush_sorted` emits no `PermanentTargeted`, so
**Ward never fires on a triggered ability**; pinned wrong-way-round by a probe with an instruction
to the successor), **`OOS-ENG2-2`** (MEDIUM — same class, four more sites the recon missed:
`handle_activate_forecast`, `handle_scavenge_card`, the loyalty handler, `flush_sorted`'s modular
arm), **`OOS-ENG2-3`** (MEDIUM — cascade/discover/cipher-copy/suspend free-casts hardcode
`targets: vec![]`, so a free-cast targeted spell reaches the stack with no targets; three of the
four admit it in an in-source comment), **`OOS-ENG2-6`** (LOW — the "cards sections" highlight ask;
a derived `PermanentView` field, no engine change, explicitly out of scope), **`OOS-ENG2-7`** (LOW —
`event_tier` is non-exhaustive by design and nothing asks; proposes a count-only-grows ratchet),
**`OOS-ENG2-8`** (LOW — the TUI's `_ => String::new()` silently drops any new event; mitigated for
this variant, class stands), **`OOS-ENG2-9`** (LOW — the feed now carries two lines per
battlefield-object target, `PermanentTargeted` + `TargetsAnnounced`; the superset proof is recorded
so the follow-up can delete the `PermanentTargeted` prose arm without re-deriving it).

Closed: **`OOS-G7-1`** (this batch, event-log half; the triage's stack half was already REFUTED),
**`OOS-ENG2-4`** and **`OOS-ENG2-5`** (both by their own riders, `-5` twice — see the citation
paragraph above).

**Untouched by design**: `OOS-M11-10` (the loyalty-ability targeting gap) — this batch *announces*
loyalty targets (site A13) but does not touch that seed's substance.

**Successor candidate: `OOS-ENG2-1`/`-2` together.** Ward not firing on any triggered ability is a
game-outcome bug, the two seeds are one mechanism at five sites, and this batch's own census has
already enumerated every site it touches. It will move fuzz and golden parity — budget for that.

**Benches within noise, as predicted**: `full_turn_4p` **221.2 µs** (PB-DX6 pinned 220–222),
`priority_cycle_4p` **24.4 µs**, `sba_check` **14.2 µs**, `full_turn_6p` 351.3 µs, `board_wipe_4p`
107.0 µs. Expected — the helper is one `stack_objects()` scan per *announcement*, not per priority
cycle, and it returns before allocating when the target list is empty.

Full plan and per-stage reasoning: `memory/primitives/pb-plan-ENG2.md`.

## Worker Handoff (ENG-1, `scutemob-191`) — effect-driven discard is a real player choice (G3, CR 701.9b)

**G3 of `memory/playtest-triage-2026-08-02b.md` CLOSED.** `Effect::DiscardCards` used to execute
inline and call `discard_cards`, which picks `min_by_key(|id| id.0)` — the human's leftmost/oldest
card — and moved it. CR 701.9b: *"By default, effects that cause a player to discard a card allow
the affected player to choose which card to discard."* No def in the corpus prints "at random" or
"another player chooses", so the default covers the **entire** live corpus and the violation was
unconditional. It now suspends into a new `EffectChoiceQuestion::Discard` through PB-DP9's
existing suspend-and-replay machinery. **PROTOCOL 33 → 34, HASH 70 → 71**, both gate-computed
from the failing gate's own output. Tests **4,330 / 0 / 5** full workspace (`--workspace
--no-fail-fast` to a file, never tail-piped) against a **pre-edit baseline of 4,317 / 0 / 5
measured on this branch** — +13, being 11 new engine tests and 2 new play-server probes. `fmt`,
`clippy --workspace --all-targets -D warnings` and `tools/check-defs-fmt.sh` (1,803 defs) clean.
Coverage **unmoved at 1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py`
to a byte-identical body — **0 card-def lines changed in the whole batch**, which is a positive
assertion, not an omission: `fell_specter.rs` was `Complete`, correct and innocent, and the
defect was 100% engine-side.

### The one decision that shaped everything else: the ask lives in the ARM, not in the helper

The dispatch brief reasoned as if the ask went inside `discard_cards`, and concluded that the
full-hand short-circuit is what stops `Effect::WheelHand` double-counting across a suspend/replay.
**That reasoning is replaced.** The ask is in the `Effect::DiscardCards` arm
(`effects/mod.rs:1267`), so:

- **`Effect::WheelHand` cannot suspend, by construction** — it calls the helper directly and the
  helper never asks. Not "because the short-circuit catches it".
- **`Cost::DiscardCard` cannot suspend, by construction**, and this one is not a nicety: that call
  is inside `pay_optional_cost`, on a cost-payment path with **no resolution wrapper to roll back
  to**. An ask there would record a `pending_effect_choice` nothing can discharge — the trap-state
  class `OOS-DP9-14` was filed for. Placement makes "cost discards do not ask" structural rather
  than promised. (CR 701.9c also gives a cost discard rules of its own; it is a *harder* problem
  than a resolution discard, not an easier one.)

Because both guarantees are structural and structural guarantees rot silently,
`test_eng1_wheel_hand_discards_the_whole_hand_exactly_once_and_never_suspends` asserts the
**structure**, not the arithmetic. If a later batch "simplifies" by moving the ask into
`discard_cards`, that test goes red.

### Shape, and why each field is named what it is

`EffectChoiceQuestion::Discard { hand: Vec<ObjectId>, count: u32 }` — the **whole** hand,
ascending, because CR 701.9b restricts nothing, so the whole hand *is* the legal answer space.
`hand`/`count` rather than `candidates` to match `GameEvent::CleanupDiscardChoiceRequired.hand`
and `LegalAction::DiscardToHandSize.count`: the engine's two discard channels should use one
vocabulary.

`EffectChoiceAnswer::Discard { chosen: Vec<ObjectId> }` — **`chosen`, not `discarded`**. The three
sibling answers name a *destination* (`found`, `bottom`/`top`, `graveyard`/`top`) because those
questions are about where cards go. This one is not a partition — the unchosen cards stay in hand
— and a destination name would be actively **wrong**: CR 702.35a sends a chosen Madness card to
**exile**, so at answer time nothing has been discarded and the destination is not yet known.

**One question for all `n` cards, not `n` questions.** Nothing between picks can change the answer
space (no priority during a resolution, CR 608.2; a Madness trigger lands after, CR 603.3), it
matches CR 514.1's cleanup discard, and `DiscardPicker` already renders it. What it forfeits — an
effect whose k-th pick depends on the (k-1)-th — is seeded as `OOS-ENG1-4`.

### The short-circuit, and the two loop exits that are not compile errors

`n == 0 || n >= hand.len()` (which includes the empty hand) short-circuits on **CR 601.2c's
principle**, the same argument the search arm already makes: when the answer space admits exactly
one legal answer the announcement is *determined*, so there is nothing to announce. That is what
keeps a full-hand discard from costing a round trip and from perturbing a fuzz seed.

The `for p in players` loop has **two different exits and neither is a compile error**: `continue`
for the determined case (later seats still get asked) and `return` for the suspension (the whole
pass is discarded; every later seat's question is re-derived by the replay). Getting them
backwards is the easiest way to break this arm, so
`test_eng1_multiplayer_discard_exercises_both_loop_exits` drives both in **one** resolution — and
it also shows the rollback undoes a determined seat's already-applied discard, which is the
property that makes `return` correct.

### The bot/fuzz default is zero-churn, and it is the opposite end of the hand from its sibling

`default_discard_answer` takes the `count` **lowest** ids from an ascending hand — byte-identical
to `min_by_key(|id| id.0)` applied `n` times. No game *outcome* changes in any bot-only game; only
the **command trace** grows an `AnswerEffectChoice`. Note it is the **opposite** of
`rules::turn_actions::default_cleanup_discard`, which takes the `count` **highest**. Both are
faithful reproductions of two auto-picks that genuinely differed (CR 514.1's took `obj_ids.last()`;
CR 701.9b's took `min_by_key`). Do not "unify" them —
`test_eng1_defaults_reproduce_both_pre_batch_picks` pins both in one place and says why.

### Fixtures that moved, enumerated — three, all repaired by ANSWERING, none by weakening

All three drive resolution with a local `pass_all` helper that never pumps blocking decisions, so
the new suspension went unanswered and the resolution appeared to do nothing.

| Test | Card | Why it moved | Change |
|---|---|---|---|
| `casting/x_cost_spells.rs::test_x_cost_spell_basic_mana_payment` | Pull from Tomorrow | draw X then discard 1; post-draw hand of 3 makes `count=1 < hand.len()`, so it asks | new shared `resolve_through_any_discard_choice` answers with the default |
| `casting/x_cost_spells.rs::test_x_cost_effect_amount_xvalue_draw` | Pull from Tomorrow | same | same, **plus** the helper merges the replay's events — the suspension rolls the whole resolution back, so the `CardDrawn` events the test counts now appear on the replay pass, not the first |
| `primitives/pbp_power_of_sacrificed_creature.rs::test_greater_good_draws_by_sacrificed_power_then_discards_three` | Greater Good | discard 3 against a 4-card hand is no longer determined | answers with the default inline (this one reads final zone counts, not events) |

The default reproduces the pre-ENG-1 pick byte for byte, so **every original assertion keeps its
meaning**. Nothing else in the workspace moved: no seeded simulator fixture, no golden script (the
harness's `auto_answer_blocking_decisions` pump already answers any `EffectChoice` with the
default), and no fuzz outcome — the honest prediction there is "no fuzz change **because there is
no fuzz coverage here**" (`OOS-UI2-1`: the fuzzer has never cast a spell), not "no fuzz change
because it is zero-churn".

### The decision-gate yield: 91 → 80, read off the gate, not computed

`decision_site_walk.rs`'s `discard_cards` row flips `AutoChosen` → `Served { by: "ENG-1" }`.
`decision_gate.rs` loses the 11 `BASELINE` entries whose only auto-chosen row was `discard_cards`
and shrinks Izzet Charm to `["counter_unless_pays"]`; `MAX_AUTO_CHOSEN_COMPLETE_UNION` is set to
**80**, the number T6's own panic printed — deliberately **not** `91 − 12`, because the union is
over *defs*, not `(def, row)` pairs. `MIN_BASELINE = 50` clears with 30 headroom and was not
lowered. **Correction to the plan**: it said 13 baseline rows; T9's reconciliation says **12**
(11 solo + Izzet Charm), and the plan's number was off by one.

Worth reading before the next audit: `decision_site_walk.rs:317-326` has carried a **verbatim
statement of this defect** since 2026-07-27 — `why_not_flagged_is_wrong: "CR 701.9b: the affected
player chooses which card, by default; the engine picks the lowest ObjectId"` — green in the suite
the whole time. The audit found it and classified it as expected. That is the corpus-scale form of
the comment-debt failure below.

### Architecture Invariant 7: the first question that names HAND objects

`EffectChoiceQuestion`'s type doc used to say *"Every `ObjectId` in every variant names a card in a
HIDDEN zone — the library."* ENG-1 **falsifies that sentence**, and it is rewritten to state the
two premises separately rather than folding the new one into the old: the three library variants
are entitled by the *effect* (the player may see those ids only because this effect is resolving),
while `Discard` names cards the answerer **already holds**. Same conclusion — `private_to()` stays
`Some(player)` — different, weaker premise, stated so a reviewer can check it. The premise rests on
`entry.player` being enforced in three independent places (the `process_command` admission gate,
`handle_answer_effect_choice` check 2, and the play-server read guard), and the doc names all
three so relaxing one is visibly a leak.

`view.rs` routes hand labels through **`NameIndex`**, not `question_card_label` — these are the
answerer's own cards, already in the seat-redacted view, exactly as the CR 514.1 arm does it.
Routing an owned-hand question through the library channel would enlarge a channel that
`test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places` counts.

**The new-channel gate exists**: `test_eng1_a_foreign_seats_discard_question_never_reaches_this_payload`,
the hand-zone analogue of the UI-1 gate. Its revert (removing the `pending.player == human` filter
in `api.rs::seat_view`) makes it red, and the leaked payload's candidates render as
`(unknown card)` from the foreign seat — the leak is real and the gate catches it. **The shipped
`GameSummary.seed` HIGH is precisely what a redaction gate checking only the channel it was
written for costs, and a hand is a new channel.**

### The two SILENT plumbing sites — neither is a compile error

`handle_answer_effect_choice` check 4 is a `matches!`, which is not exhaustive: a miss refuses
**every** discard answer with "does not answer question". Check 5 sits before
`_ => unreachable!("variant agreement checked above")`: a miss **panics the engine** on the first
real answer. Both were extended; both are exercised by tests (b) and (f). `api.rs`'s
`validate_decision_params` has a `_ =>` catch-all with the same property — a miss there is a silent
400 on every discard — and it was extended too. `validate_partition` is deliberately **not** reused
for the discard: it is not a partition (the unchosen cards stay in hand) and its message strings
would give a false diagnosis. That is said in a comment so nobody "deduplicates" it later.

### `OOS-ENG1-9` — the batch's biggest discovery, measured and NOT fixed

Building the browser probe reddened its real-name assertion on Faithless Looting, and the cause
generalises: **CR 608.2d's suspend rolls the WHOLE resolution back** (`rules/resolution.rs`, `*state
= restart_point`). For a **draw-then-discard** printing the recorded question names hand objects
that the *restored* state does not contain — the draw was rolled back and CR 400.7 minted new ids
— so every candidate DRAWN IN THAT RESOLUTION renders as the unknown-label placeholder (corrected
by the /review fix cycle: the original wording overstated its own measurement — pre-existing hand
cards render their real names; the probe saw 5 of 7 correct on Faithless Looting, not 0 of 7). The
answer still applies correctly on submission (the replay re-draws deterministically and re-mints
the same ids), so this is a **display gap, not a correctness gap**.

**It is new to this variant, and not by design**: the three library questions name cards that
already existed before the resolution began, so they are immune **by accident**.

**Blast radius, measured, not guessed.** Of the **21** def files that actually carry
`Effect::DiscardCards` (a plain grep says 23 — `reforge_the_soul` AND `nezahal_primal_tide` each
mention it only in a comment explaining that they use `Effect::WheelHand` instead; the first review
cycle caught only the first of those two, correcting the figure a second time here, and derived it
by grep-minus-exceptions rather than an `all_cards()` enumeration, SR-36 — the method, not just the
number, is why it was wrong twice), **14 draw in the same effect**. The number that matters for
playability today is the deck-legal one: of the **12** `Complete` defs, **7 draw** — Chart a
Course, Faithless Looting, Frantic Search, Geier Reach Sanitarium, Greater Good, Izzet Charm, Pull
from Tomorrow — against 5 that do not (Burglar Rat, Consign // Oblivion, Fell Specter, Raiders'
Wake, Sword of Feast and Famine). **A clear majority of the cards a human can actually play, and
the dominant printing, not a corner case** — the loot effect is what "discard" mostly means in
Magic.

**Deferred deliberately, with the reason**: the correct fix is not a discard patch but a general
LKI-for-questions mechanism — capture each candidate's identity at the moment of the ask (where the
objects still exist) onto `PendingEffectChoice`, and widen `BlockingDecision`, `LegalAction` and
the view to carry it. That is a second wire-adjacent surface in a batch already bumping PROTOCOL
and HASH, and it generalises beyond discard to any future question whose answer space is created
mid-resolution. **The coordinator should weigh whether it is the immediate successor**: for those
7 deck-legal cards the human now gets a picker with unlabelled options where they previously got a silent
auto-pick, which is arguably worse for them until this closes. Filed in
`docs/audits/decision-point-audit.md`.

**The /review fix cycle closed the sharpest edge of this, without closing the seed itself**
(review Finding 2): two same-resolution-drawn candidates used to render as two buttons with
IDENTICAL text (`(unknown card)` twice), which read as a redaction bug in the seat's own hand.
`view.rs`'s `PickN` arm now gives each unlabelled candidate a distinguishing placeholder —
`(card drawn this resolution #N)` — so the human can still make a fully informed choice among the
pre-existing hand cards, which is strictly more agency than the pre-ENG-1 silent auto-pick had.
`OOS-ENG1-9` itself (the general LKI-for-questions fix) is still open.

### Comment debt — the thing this batch is a lesson about

`discard_cards`' doc read *"Discard n cards from a player's hand (first by ObjectId,
deterministic)"* — it stated a **placeholder as a design property**. Every one of the ~13 sibling
auto-pick sites the triage census found carries a `deferred to M10+` comment. **This one did not,
which is exactly why the PB-DP decision-point audit's greps missed it and a human found it in a
live game.**

> **A deliberate placeholder that documents its MECHANISM instead of its DEBT is invisible to every
> audit that greps for the debt.**

`discard_cards`' doc now says plainly that its `min_by_key` is the auto-pick path only, and
`Effect::Connive`'s inline comment — the last remaining copy of the exact comment shape that hid
this for a year — now carries `deferred, OOS-ENG1-2` and its CR cite.

### PROTOCOL / HASH, and the sentinel re-pin

PROTOCOL **33 → 34**, fingerprint `2cda8c05…`; **closure type count unchanged at 96** —
`EffectChoiceQuestion`/`EffectChoiceAnswer` have been in the closure since v31, only their declared
shape moved. HASH **70 → 71**, `decl_fingerprint` `ce89c998…` over 129 types, `stream_fingerprint`
`c2845544…`, and both frozen-prefix digests re-pinned to what their gates printed once the v33 and
v70 rows joined the prefix. New history rows appended in both files; **no shipped row edited**.

Sentinels re-pinned **by symbol** across 46 test files. **Two multi-line survivors** —
`pb_dx2_command_gates.rs:1478-1492` and `pb_dp5_pending_draw_choice.rs:1244-1253`, each carrying
both a HASH and a PROTOCOL sentinel split across lines — were invisible to every single-line grep
and were found only by reading the files. That is the exact failure PB-DX5 shipped with.
**Residual list after the pass: EMPTY**, confirmed by execution, not by inspection.

**One surprise worth carrying forward**: `stream_fingerprint_is_pinned` was still **green** before
the version bump even though the new `HashInto` arms already existed — the canonical fixture
carries no `Discard`-shaped `pending_effect_choice`, so the new arms were unexercised and only the
version byte moved the stream. **A hash arm can ship unhashed and unnoticed here**, and `hash.rs`'s
own warning already says the SR-19 gate scans structs only, so an enum arm dropping a field feed
passes every gate green (`OOS-DP9-13`).

### Browser verification (live headless Chromium, real clicks)

Seed **22**, 4 players, heuristic bots, human seat 1. The asking card is **Burglar Rat controlled
by Bot-2** — a bot-controlled `DiscardCards` resolving against the human — reached at step 48 of a
pass-only drive. 28 seeds tried, two hits (seed 22 Burglar Rat, seed 24 Fell Specter). Payload:
shape `PickN`, `answer_field: "effect_choice_answer"`, `chosen_key: "chosen"`, template
`{"Discard":{"chosen":[2]}}`, `default: [2]`, and **all seven candidate labels are real card
names**. Clicked **Elspeth, Storm Slayer (id 3)** — non-default, since the default is id 2, the
lowest `ObjectId`. Server-side proof: Elspeth left the hand and is in the graveyard; **the default
card (id 2, Plains) is still in hand**. Also verified: "Use the default" selects but does **not**
auto-submit (0 POSTs, `command_count` unchanged); "Back" leaves the decision intact at the same
`seq` and re-openable, not wedged; and the console carried exactly **one** message across the whole
session, a 404 for `/favicon.ico` — **no `DataCloneError`, no uncaught exception**, so the UI-4
class does not recur in `DiscardPicker`'s new template branch.

**A trap for the next author of a test like this**: the graveyard entry was id **427, not 3** — CR
400.7 mints a new object on the zone change, so an id-equality assertion across a discard always
reads false. Match by name.

### Seeds filed

- **`OOS-ENG1-1`** — `Cost::DiscardCard` (`effects/mod.rs`, inside `pay_optional_cost`) still
  auto-picks the lowest id. CR 701.9b covers a cost discard too. Excluded **structurally**: a cost
  is paid outside any resolution wrapper, so an ask there records a `pending_effect_choice` nothing
  can discharge, and CR 701.9c adds cost-specific rules an announcement must respect.
- **`OOS-ENG1-2`** — `Effect::Connive`'s inlined discard duplicates the `min_by_key` because it
  needs per-card nonland accounting. Now trivially closable *except* that the nonland counter must
  survive a suspend/replay — a real design question, not a rename.
- **`OOS-ENG1-3`** — no `chooser` field on `Effect::DiscardCards`. **Do not add one.** 21 def files
  carry the effect, 12 are deck-legal `Complete`, and **zero** print "at random" or "another player
  chooses". The two corpus cards that would need it (`gamble.rs`, `grief.rs`) are blocked TODO defs
  carrying no `Effect::DiscardCards` at all, so the field would ship with no reader.
- **`OOS-ENG1-4`** — the one-question shape forfeits a sequenced per-card choice. No current
  printing needs it; filed so a future "discard a card, then discard a card" does not silently
  inherit the wrong shape.
- *(`OOS-ENG1-5` is deliberately unused — the filed set skips it. Noted here per review Finding
  10 so a future reader does not go hunting for a seed that was never filed.)*
- **`OOS-ENG1-6`** — `Effect::MillCards` is the only sibling of the missing `.max(0)` fixed here
  (`resolve_amount(...) as usize` with no clamp wraps a negative to ~1.8e19; `discard_cards`' loop
  has no empty-hand break, so it was an effective hang in release from a legal `EffectAmount`). Not
  fixed — a drive-by in an adjacent arm is how review scope-creep starts. Check whether
  `mill_cards` has an empty-library break before deciding severity.
- **`OOS-ENG1-7`** — `DiscardPicker` submits ascending ids, not click order. CR 608.2f/404.3 make
  discard order a real player payload; shipped ascending because `check_ids` treats the list as a
  set and no card in the corpus reads graveyard order.
- **`OOS-ENG1-8`** — `fable_of_the_mirror_breaker` (`partial`) TODO names this primitive
  (*"DiscardCards has no player-choice bound"*) and is **NOT closed by ENG-1** — chapter II needs
  *optional* + *up-to-N* + a count-driven draw, and this question asks for **exactly** `count`.
  With the variant in the tree, closing it is a min/max widening plus an `EffectAmount` source, not
  a new primitive.
- **`OOS-ENG1-9`** — the draw-then-discard label gap above. **The successor candidate.**
- **`OOS-ENG1-10`** — the second `/review` pass's find: `tools/tui/src/play/input.rs`'s `'r'` key
  still submits the engine's default `EffectChoiceAnswer` verbatim for a discard, with no picker —
  pre-existing and identical to its scry/surveil/search handling (the `OOS-DP7-6`/`OOS-DP8-2`/
  `OOS-DP9-7` family), so NOT a regression, but "effect-driven discard is a real player choice" is
  now true on the browser only. Not fixed. See `tools/play-server/README.md` limitation 28.
- **`OOS-G3-2`** — the "engine picks for the player" census. The triage's list in
  `memory/playtest-triage-2026-08-02b.md` §G3 has never been machine-checked, and
  `decision_site_walk.rs`'s `AutoChosen` rows are the machine-checkable version of it. Reconcile
  the two and make the source comments derive from the table rather than the reverse.

`OOS-G3-1` (the defect itself) is **CLOSED**. `Effect::SacrificePermanents` remains the named
cheapest follow-on and is genuinely cheaper now, but it is a different rule with a **public**
answer space and therefore a different hidden-info argument — its own batch, not a rider.

### Roster-recall gate

TODO sweep over `crates/card-defs/src/defs/`: 27 hits, exactly **1** names this primitive
(`fable_of_the_mirror_breaker`, recorded above as a NOT-a-forced-add with its reason). The other 26
are a different primitive each. **0 forced adds, 0 card-def lines changed.**

### The `/review` cycle — two reviewers, 0 HIGH, 2 MEDIUM, 8 LOW, and **all of them taken**

Full findings: `memory/primitives/pb-review-ENG1.md`. Both reviewers looked specifically for a HIGH
in the four places most likely to hide one — the question-equality determinism premise, the two
loop exits, the three non-compile-error plumbing sites, and the `HashInto` field feeds — and each is
correct. **The MEDIUMs are worth reading even after they are closed**, because one of them is a
standing hole in a gate and the other is a lesson about how a deferral should be shaped.

**MEDIUM 1 — a hash arm can ship unhashed, and the warning that says otherwise is half false.**
`grep 'pending_effect_choice\|EffectChoiceQuestion\|EffectChoiceAnswer'` over
`crates/engine/tests/core/hash_schema.rs` returned **zero**: `canonical_fixture()` had never
populated `pending_effect_choice`, so **all four** arms of both enum impls had been unexercised
since PB-DP9 — not a new hole, an inherited one. `hash.rs`'s own warning claimed the enum impls were
"held by review and by `stream_fingerprint`, nothing else"; the second half was **not true**,
because `stream_fingerprint` is computed over a fixture that never reaches them. Dropping
`count.hash_into(...)` would have made two states differing only in a pending discard's count hash
**identically**, with `cargo test --workspace` fully green — SR-19's gate scans structs only, and
SR-9b's `harness_equivalence` cross-validates green because *both* regimes drop the same field. That
is an undetectable desync in exactly the state M10's network layer most needs to detect one.
**Closed here rather than seeded, and the timing is the whole argument**: closing it re-pins
`stream_fingerprint` on the **v71 row this batch was already writing and which has not shipped to
main**. A successor batch would have paid a HASH 71 → 72 bump plus a 46-file sentinel re-pin for a
test-fixture change — an order of magnitude more, for identical correctness. `canonical_fixture()`
now carries a `Discard`-shaped `pending_effect_choice` (hand of 3, `count: 2`) and a one-entry
bank; new `stream_fingerprint` `923b1ff8…`, **proven by executing the revert** — dropping `count`
turned the gate red printing `1edc655e…`, restored, green.

**MEDIUM 2 — the deferral was right, the placeholder was not.** See the `OOS-ENG1-9` section above:
two same-resolution-drawn candidates rendered as two buttons with *identical* text. Both reviewers
independently endorsed deferring the general fix and both flagged the placeholder as separately
fixable at zero wire cost. **Generalisable: when you defer a fix, the thing you ship in its place
is a deliverable too, and it can have its own defect.**

Of the eight LOWs, three were errors in **my own write-up** and are the ones worth naming, because
they are the batch's own thesis turned on itself: the def-file denominator was wrong **twice**
(23 → 22 → 21; `reforge_the_soul` *and* `nezahal_primal_tide` each mention `Effect::DiscardCards`
only in a comment, and I caught one of the two), and it was wrong because it was derived by
grep-minus-exceptions instead of an `all_cards()` enumeration — **SR-36 exists to say exactly
that**; and the `OOS-ENG1-9` summary overstated its own measurement ("every candidate" where the
evidence said 5 of 7 rendered correctly). The 12-`Complete` and 7-draw figures were right
throughout. Two more LOWs were **comment debt inside the batch whose thesis is comment debt** —
`rules/events.rs` still said the question's ids were library-only, and `view.rs`'s arm comment
asserted "the CANDIDATES are library cards" directly above the arm that disproves it. The last
structural one: `Cost::DiscardCard`'s guarantee — which plan §2.4 calls the *more* dangerous of the
two — had **no named guard**, and test (d) does not cover it (a future batch moving both the ask and
the short-circuit into `discard_cards` leaves (d) green because `WheelHand` passes
`n == hand_size`, while `Cost::DiscardCard` passes `n = 1` against a larger hand and would begin
recording undischargeable entries). `test_eng1_a_cost_discard_never_suspends` now exists, proven red
by an executed revert.

## Worker Handoff (UI-5, `scutemob-190`) — UX polish batch 2: G8, G10, G11, G12, G13

**All five UX rows of `memory/playtest-triage-2026-08-02b.md` closed. Frontend only: 0 engine
lines (`git diff main..HEAD -- crates/` is empty), 0 wire change, PROTOCOL 33 / HASH 70
gate-executed and unmoved.** Tests **4,317 / 0 / 5** full workspace (+4 over SIM-6's 4,313 —
the four new gates), measured with `--workspace --no-fail-fast` to a file. `fmt`, `clippy
--workspace --all-targets -D warnings` and `tools/check-defs-fmt.sh` all clean.

### The one decision the brief asked for up front, made once and applied three times

G11/G12/G13 all land in the `$viewer` components the two surfaces share in place. **The rule:
edit the shared file in place; where the two surfaces genuinely want opposite behaviour,
express the difference as a PROP rather than as a copy.**

| Item | Shared? | Why |
|---|---|---|
| G11 caption | in place, unconditional | the native-`title` collision is identical in the replay viewer — same anchor, same chrome |
| G12 board order | in place, unconditional | pure sibling-block order; the replay viewer has no opposing requirement |
| G13 land stacking | in place, behind `stackLands` (default **false**) | the replay viewer is a step *debugger*: `App.svelte`'s `openCard` opens the object you clicked, and folding five Forests into one chip deletes four of the objects you are stepping to inspect |

A fork of `ZoneBattlefield` would have duplicated 476 lines **including G11's and G12's fixes**
and forked again on the next `PermanentView` field — precisely what `PlayBoard.svelte`'s module
doc says the *leaf* components must not do. That is one rule with one exception criterion, not
three answers in one file.

### What shipped, item by item

**G8 — Concede placement + confirmation.** Out of the action row (filtered from **both** groups,
not just `controls` — dropping it from `controlKinds` alone would have re-shown it mid-play-list),
into the header beside "New game", behind a two-step confirm. Same `option.index`, routed through
`ActionBar.beginExternal` so there is no second code path to the most destructive control on the
surface. **Disabled with a visible reason rather than hidden** — a control that blinks in and out
of the header on every bot turn reads as a bug, or gets hunted for in the one moment it is
dangerous. The reason is rendered as text, not a `title`: **a native tooltip does not open on a
disabled button**, because a disabled control fires no pointer events, so "disabled with a reason"
written as a `title` is a reason nobody can read. Pickers' "Cancel" → **"Back"** at all eight plus
the unknown-shape fallback.

**G10 — mana sources.** A `▸ mana sources (N)` disclosure, collapsed by default, one row per
source *name* with a count (`Tap Mountain for mana ×4`), folded on the server's own label. **Not
hidden**, and the gate asserts *both* sides — collapsed **and** still submitting — because a later
tidy-up that deletes the group would satisfy the playtest note and break every activation cost,
every echo/cumulative-upkeep/recover payment, and every float-ahead-of-a-cost-increase (CR 608.2g).
`OOS-SIM6-3` untouched. Side effect worth knowing: `plays` can now be empty while mana sources
exist, so the empty state says *"No plays available beyond tapping for mana"* instead of lying.

**G11 — tooltip caption.** `cardTooltip` accepts `{name, caption}` and renders the caption inside
the floating div. All nine triage-named sites cleared — **plus roughly ten `title=` on the badges
nested inside those anchors** (`CMD`, `TAP`, `SICK`, `ATT`, counters, keyword abbreviations). Those
were not in the triage and produce the *identical* collision over a smaller hit area on a ~70px
chip; every one existed to expand an abbreviation, so they folded into a second caption line and
lost nothing. Shared `zoneCaption` for the four `CardInZoneView` sites so they cannot drift apart
again — writing the same template four times is how they drifted in the first place.

**G12 — board order.** **Lands moved down, rather than Artifacts/Enchantments moved up** — sliding
A/E up would also have pushed it above Planeswalkers, changing an order nobody complained about.
Result: Creatures, Planeswalkers, Artifacts/Enchantments, Lands, Other. **Artifact lands stay in
the Lands row, deliberately**, documented at the classifier with the one-line reversal named: a
player reads an artifact land as a land — it is what you tap for mana and what CR 305.2 limits —
and nothing here touches `card_types`, so it is still an artifact for Metalcraft and for artifact
removal. Only where the chip is drawn changed.

**G13 — land stacking.** Key is `(name, tapped)` **plus** sorted counters, `attached_to`,
`is_commander`, `is_token`, `summoning_sick`, `damage_marked` — a deliberate superset of what the
land block renders, because the failure mode of a too-narrow key is a silent lie about the board
and of a too-wide key is a chip that does not stack. The `#each` key is the fungibility string and
**not** the representative's `object_id`: tapping one Forest of five moves that permanent into a
different stack, and a key derived from a member that just left would destroy and rebuild a chip
that only changed its count.

**Click path, decided rather than implicit.** The chip nominates `members[0]` — arbitrary *and
immaterial*, since the key already required every member to be indistinguishable, and since tap
state is *in* the key a stack is wholly tapped or wholly untapped, so "first untapped" collapses to
"first". It hands the **whole group** up as a second argument, and `PlayApp.representativeFor`
falls through to a sibling carrying an offered action — the caller is the only party that knows
what the server offered. The extra argument is inert for the replay viewer, whose `openCard(card)`
takes one parameter.

### Gates: four, each proven red by executing a revert

All in `tools/play-server/src/main.rs`, so they run under `cargo test --all` and therefore CI.
Source-level for the standing reason: there is still no frontend test harness (plan §8 R7).

| Gate | Pins |
|---|---|
| `test_frontend_card_elements_carry_no_native_title` | per-**element**, via a tag walk over each `use:cardTooltip` anchor |
| `test_concede_lives_in_the_header_behind_a_confirmation` | out of both action groups; header arm/confirm; same entry point; eight pickers say Back |
| `test_tap_for_mana_is_grouped_and_still_reachable` | collapsed **and** still submits |
| `test_land_stacking_key_is_not_just_the_name` | every field of the key by name; `stackLands` default off; every play-surface instance opts in |

Nine reverts executed, all red, tree green again: `title=` restored on `ZoneHand`; `Concede` back
in `controlKinds`; `CostPicker` Back→Cancel; `concedeArmed` renamed; `manaOpen` default `true`;
the mana row's `onclick` deleted; `p.tapped` dropped from the key; `stackLands` default `true`;
`representativeFor` renamed.

**The G11 gate is the one worth reading, and it is per-element rather than per-file on purpose.**
`title` is fine and useful on a control that is not a tooltip anchor — the Export-report button,
`SeatCard`'s drawer toggle, `StepControls`' whole row — so banning the attribute outright would
have deleted working affordances to fix an unrelated bug. It walks each opening tag carrying
`use:cardTooltip`, tracking `{}` depth and quote state because a Svelte attribute value can
legally contain `>` (`class:pt-damaged={p.damage_marked > 0}`) and stopping at the first `>` would
truncate the tag and read as "no title here". **Its own first run found a bug in itself**: a
component's module doc *names* `use:cardTooltip` in prose, and walking back from there finds the
nearest `<` — `<script` itself, or a `<` comparison operator in code — and reports a tag that does
not exist. Now template-only with HTML comments blanked, and the synthetic non-vacuity case
carries both shapes, so the extractor is proven by execution rather than argued.

### Browser verification — 24/24 live, plus 10/10 on the shared components

Headless Chromium (playwright-core, `/usr/bin/chromium`) against a live `play-server` on **:3045**,
seed **190190**, 4 seats, heuristic bots. Driven over HTTP to turn 23 and stopped **while a
decision was still live** — the first attempt ran to turn 59 and the game was over, which leaves
nothing to concede and no mana source to offer. Stop condition: the human holds ≥4 untapped lands
of one name, a `TapForMana` is offered, and some seat's board carries an artifact/enchantment
*and* lands (an ordering assertion over a board of nothing but lands is vacuous).

| Item | Evidence |
|---|---|
| G11 | 0 elements matching `.permanent-card,.hand-card,.gy-card,.exile-card,.cmd-card,.chip,.stack-item` carry `title`; battlefield hover → `"Legendary Creature — Human Soldier\n2/1 · commander · First Strike"`; hand hover → `"Contagion Clasp\nArtifact"` |
| G12 | `["Creatures (2)","Artifacts/Enchantments (1)","Lands (6)"]` |
| G13 | `Plains×4` untapped, `Swamp×3` **tapped**, `Swamp×2` **untapped**, `Mountain×4` untapped — Swamp and Mountain each render as two chips because each exists in both tap states; clicking the human's own stack acted, `command_count 819 → 820` |
| G10 | `▸ mana sources (3)` collapsed; 0 `kind-TapForMana` in the plays group; expanded → `["Tap Mountain for mana ×3"]` |
| G8 | header shows `New game` / `Export report` / `Concede`; 0 concede in the action row; an **open** `TargetPicker` shows `["Confirm (0/1)","Back"]` and Back submits nothing (`820 → 820`); first click arms `"Concede — end your game? Yes, concede / Keep playing"`; **declining** leaves `game_over=false`, commands `824 → 824`; **confirming really concedes** (`winner Bot-4, 48 turns`); afterwards the button is disabled with `"the game is already over"` |

**Shared components, mounted against a fixture** rather than through the replay viewer's own
binary — `memory/gotchas-infra.md` records that starting that binary from an agent context gets
SIGKILLed (137). A throwaway Vite entry mounted `ZoneBattlefield` **twice on one page**, with and
without `stackLands`, over a 6-Forest fixture (3 plain untapped, 2 tapped, 1 untapped carrying a
charge counter) plus a Sol Ring. Results: viewer mode **6 chips, all count 1**; play mode
**`Forest×3` untapped / `Forest×2` tapped / one lone Forest** — the counter Forest correctly
refusing to merge with its otherwise-identical siblings; artifacts above lands; zero `title`;
caption `"Basic Land — Forest\ncharge counter ×1"` (the badge title that used to be native); and a
stacked chip handing up `[representative_id, group_length] = [1, 3]`. **This is a working
proof-of-concept of the R7 tier-1 harness** and took ~15 minutes; the recipe is: a directory beside
`tools/replay-viewer/frontend/src` containing `index.html` + `main.js` (`mount(Harness, …)`) +
`Harness.svelte` + a `vite.config.js` whose `root` is that directory, built with
`npx vite build --config <dir>/vite.config.js` from the frontend package (so `node_modules`
resolves), then served by `python3 -m http.server`. It was **not** committed — R7 is deferred and
the brief did not ask for it — but the next batch that wants a frontend harness should start here
rather than from scratch. Both production bundles were also rebuilt and both succeed (156 and 142
modules).

### The `/review` cycle found 8 and all 8 were taken — two were real defects, both in G8

1. **MEDIUM — the armed confirmation survived the decision it was armed against.** `local_game.rs`
   appends `Concede` to **every** decision it builds for the human, so a disarm `$effect` keyed on
   `concedeAction` being null essentially never fired. Arm Concede, change your mind, pass priority
   instead — and the red "Yes, concede" bar stayed up, live, across the next decision and the one
   after. **That is the accidental-concede class G8 exists to close, reintroduced by the guard
   meant to prevent it**, and the effect's own doc comment claimed the property the code did not
   have. Reproduced in the browser before the fix (armed, `seq 1446 → 1447`, `stillArmed=true`).
   Now keyed on `$decision?.seq`.
2. **MEDIUM — the header Concede was a silent dead control while a picker chain was open.**
   `beginChain` early-returns on `if (loading || chainOpen)`, and `chainOpen` is `ActionBar`-
   internal, so the button rendered enabled: click "Yes, concede", the bar vanishes, nothing
   happens, no error. **The same silent-dead-button shape UI-4 was dispatched to fix — and the
   shape that made the playtester reach for Concede in the first place.** `ActionBar` gains an
   `onChainOpenChange` push (a method call on a `bind:this` handle is not reactive, and this is
   read inside a `$derived`), and the disabled-reason list gains a fifth entry. Both fixes proven
   by revert: each reverted fix reddens its browser check, 24/24 → 23/24.
3. MEDIUM — stale README and no handoff. Both written (this file; README's Interaction section
   rewritten, and the "one change outside `tools/play-server`" heading generalised).
4. LOW — `position()` floored at the nominal image height when an image is expected. `onEnter`
   assigns `src` and positions synchronously, so on the first frame `offsetHeight` was
   caption-height alone (~30px) and the box could be centred with the image off-screen until the
   first `mousemove`.
5. LOW — `render()` no longer re-assigns an identical `src` on update.
6. LOW — the `ZoneStack`/`onCardClick` doc block had been orphaned by inserting `representativeFor`
   between it and `handleCardClick`. Moved back.
7. LOW — `StateView.svelte` and `CombatView.svelte` still carry card-element `title`s and are
   knowingly out of scope: neither anchors `cardTooltip`, so neither collides, and giving them a
   caption would mean giving them a tooltip (a feature, not this batch's repair). **The exemption
   is now machine-checked** — the gate asserts they are NOT anchors, so the day one grows a
   `use:cardTooltip` it goes red and the per-element ban starts applying to it. That is the only
   honest way to write an exemption down.
8. LOW — gate brittleness. The two array-literal assertions now read the *literals* (whitespace-
   and order-insensitive) rather than whole source lines, and the `stackLands` check became "every
   `<ZoneBattlefield>` instance opts in" with HTML comments blanked first — otherwise the prose
   explaining the prop counts as an opt-in and an added instance that forgot it passes.

### Durable lessons

- **A confirmation step is only as good as the event that disarms it.** The guard was written, was
  documented, and was keyed on the wrong signal, and the wrong signal was one that essentially
  never fires. A two-step confirm whose second step stays live across unrelated decisions is worse
  than no confirm, because it is a live destructive button you have stopped looking at.
- **"Disabled with a reason" written as a `title` is a reason nobody can read** — a disabled
  control fires no pointer events, so the native tooltip never opens. Same lesson as G11, from the
  other direction, and a reviewer will not catch it because the attribute is right there in the
  source.
- **A gate that is worth writing is worth firing at a synthetic offender.** The G11 tag walk was
  wrong on its first run in a way that would have made it green-on-nothing for two of six files;
  the non-vacuity arm caught it in the same minute it was written.
- **Commit before running revert experiments.** A `git checkout -- <file>` used to undo a revert
  also discarded four uncommitted `/review` fixes to the same file. They were reapplied and the
  rebuilt bundle hashed identically (`index-DlGFzzL8.js`), which is how the reapply was verified
  rather than assumed — but the cheap habit is to commit first.

### Seeds

- **`OOS-UI5-1`** — `StateView.svelte:139` (command-zone chip) and `CombatView.svelte:67/79/90`
  (attacker/blocker boxes) carry the native `title` that G11 removed everywhere else. Harmless
  today because neither anchors `cardTooltip`; the gate pins that premise. If either grows a card
  preview, the text must move to a caption first.
- **`OOS-UI5-2`** — land stacking is limited to the Lands group. Creature tokens are the other
  population that arrives in identical multiples (a board of nine Saprolings is nine chips), and
  `PermanentView` carries everything the key would need. Not done here because a creature's chip
  renders P/T, damage and summoning sickness, so the fungibility key has more to say, and because
  combat selection (`AttackerPicker` / `BlockerPicker`) picks per-`object_id` and would need the
  same representative decision made a second time.
- **`OOS-UI5-3`** — `manaSourceRows` folds on the server's rendered **label**, so two different
  cards would merge if `view.rs` ever printed the same sentence for both. It does not today
  (`format!("Tap {} for mana", card(source))` over the card name), and a same-named pair is
  fungible for this purpose anyway — but the fold is on presentation rather than on identity, and
  that is the kind of coupling that is invisible until it is wrong.
- **`OOS-UI5-4`** — the R7 frontend harness remains unbuilt. This batch proved the tier-1 shape
  works in ~15 minutes (recipe above) and then threw it away, which is the right call for a batch
  that was not asked to build it and the wrong outcome to repeat a third time. Every UI batch since
  UI-4 has paid for its absence in source-level gates that cannot prove a component renders.

## Worker Handoff (SIM-6, `scutemob-189`) — activation costs are payable, and the offer stops lying

**G4 CLOSED, both components.** The triage's chain was correct end to end and is
re-verified against HEAD: `LegalAction::ActivateAbility` (`legal_actions.rs:93-102`) had no
cost field; the offer loop (`:883-918`) checked mana/hybrid/Phyrexian/life and never
`ability.cost.sacrifice_filter`; `view.rs`'s `additional_costs_view` early-returned for
anything that was not a `CastSpell`, so `ActionBar`'s cost stage never opened; and
`params.rs:339-345` hardcoded `sacrifice_target: None` / `discard_card: None`. The engine
was innocent throughout — the wire fields have existed since PB-EF1.

**The fix is the UI-2 shape, one command over**: a new `ActivationCostPlan` on the action
(`ActivationSacrificeOption` / `ActivationDiscardOption`), an SR-38 suppression gate when
either eligible set is empty, the choice forwarded through `params.rs` (falling back to the
plan's own default so a *bot* submission is engine-legal), a picker block in
`additional_costs_view`, a validator arm in `api.rs`, and two new props on `CostPicker`.
**0 engine lines** (`git diff main..HEAD -- crates/engine/` empty), **0 wire changes** —
`LegalAction`/`ActionParams` are simulator types. PROTOCOL **33** / HASH **70**
gate-executed and unmoved.

### Three things this batch found that the brief did not predict

1. **The brief's refusal attribution is wrong, and the correction is the useful part.** It
   said "~95 InsufficientMana-on-ActivateAbility + 40 'activation condition not met' … your
   subject is ~80% of all bot command refusals". Re-running the SIM-5 A/B instrument
   (`crates/simulator/tests/sim5_bot_cast_discipline.rs`, seeds 0/7/42, 25 turns) at the
   merge base and printing every rejection class: **not one of the 166 refusals is a
   sacrifice- or discard-cost refusal.** `"sacrifice_target must be Some"` appears **zero**
   times. Those 135 are two *different* SR-38 gaps in the same loop (below). The
   cost-payment channel's refusals never appeared because the *bots* were never reaching
   them — which is exactly why this defect needed a human playtest to surface.

2. **The heuristic bot had to be taught to decline, and a seeded fixture caught it.** With
   the channel open and `ActivateAbility` scored at 40 (vs `PassPriority`'s 1) under a
   2-per-turn repeat cap, a bot ate two of its own creatures per turn, every turn.
   `test_ui3_combat_view_maps_attackers_to_defenders_and_blockers` went red — seed 21 no
   longer reached a declared blocker, because the blockers had been sacrificed. The bot now
   scores an activation whose cost NAMES an object below `PassPriority` (the established
   "0" idiom: below passing, above nothing, so it is still chosen when it is all there is,
   and `params.rs`'s default keeps that command legal). The dispatch brief's own guidance —
   do not teach bots sacrifice strategy; declining is acceptable. `RandomBot` still picks
   these uniformly, so the fuzzer keeps exercising the channel. **Verify this by reverting
   the score, not by reading it**: the UI-3 test is the instrument.

3. **The browser verification found a live 422 of its own, in the same loop.** Driving a
   real game to a Rummaging Goblin discard activation (`{T}`, Discard a card: Draw a card),
   the picker rendered perfectly, the human picked a non-default card, and the POST came
   back **422 — `"object ObjectId(499) has summoning sickness and cannot use abilities with
   {T}"`**. The offer loop mirrored none of the three refusals `handle_activate_ability`
   makes that are knowable from state alone: CR 302.6 summoning sickness, CR 602.5b
   `activation_condition`, CR 118.3 remove-counter. SIM-2 had built exactly this predicate
   for the MANA path (`mana_solver::tap_ability_is_activatable`, OOS-CARDS2-9) and the
   non-mana sibling was never written. It is now
   (`legal_actions::activated_ability_is_activatable`), and **that alone closes the 40
   "activation condition not met" refusals**.

### A/B, measured both ways (instrument: `sim5_bot_cast_discipline.rs`, seeds 0/7/42, 25 turns)

| | merge base | this branch |
|---|---|---|
| total bot command refusals | 30 + 44 + 92 = **166** | 30 + 28 + 55 = **113** |
| `activate: InsufficientMana` | **95** | **62** |
| `activate: "activation condition not met"` | **40** | **0** |
| `activate: sacrifice/discard cost` | **0** | **0** |
| wasted taps / `ManaPoolsEmptied` | 0 / 1 | 0 / 1 (SIM-5's gate holds) |

The 62 residual `InsufficientMana` are **`OOS-SIM6-3`** and are the largest single refusal
class left in the simulator.

### Browser verification — three flows, each with a NON-DEFAULT answer

Seed-scanned `POST /api/game` over 0..400 for a human opening hand holding any of the 37
`Complete` defs with an object-naming activation cost (**hand lives at
`state.zones.hand["Human-1"]`**, not `zones.hand`). **Known-good tuples, handed over so
nobody re-scans**: seed **79** → Yahenni, Undying Partisan; seed **62** → Altar of
Dementia; seed **219** → Rummaging Goblin (discard); seed **282** → Vampiric Rites; seed
**63/70/73/106** → High Market / Spawning Pit / Scavenger Grounds / Viscera Seer. Driver
and playwright scripts were scratchpad-only (~60 lines each, trivially rewritten from this
paragraph).

* **Yahenni (seed 79), activated IN RESPONSE to a bot's Dismember on the stack** — exactly
  the playtest report. The picker offered `Jadar` and `Zombie` and **not Yahenni itself**;
  the prompt read "Sacrifice **another** creature"; picking the non-default `Zombie`
  POSTed `{"cost_sacrifice_target":418}` → **200**; `Zombie` went to the graveyard, `Jadar`
  (the default) did not, the ability resolved above Dismember and Yahenni came back with
  `keywords: ["Haste","Indestructible"]`.
* **Altar of Dementia (seed 62)** — cost stage *then* target stage in one chain, one POST
  carrying `cost_sacrifice_target` **and** `targets` → 200. Archmage Emeritus (power 2)
  sacrificed, the non-default target Bot-3 milled exactly 2, Bot-2 milled 0.
* **Rummaging Goblin (seed 219)** — the discard half. Non-default `Balefire Dragon`
  discarded, goblin tapped, draw resolved. This is the flow that produced finding 3.

No error strip, no `pageerror`, no console error in any of the three.

### Card defs: 8 one-line repairs, and a stale belief that produced them

`yahenni_undying_partisan` was the mandated fix, but it is not alone: **8** activated
abilities print "Sacrifice **another** …" and carried `exclude_self: false`, so all 8 would
have started legally sacrificing themselves the moment this channel opened —
`yahenni_undying_partisan`, `ayara_first_of_locthwain`, `bartolome_del_presidio`,
`razaketh_the_foulblooded`, `umbral_collar_zealot`, `warren_soultrader`, `woe_strider`,
`baron_bertram_graywater`. Coverage is **unmoved at 1,133/1,803 = 62.8%** (regenerated;
only the header date, git SHA and rolling commit log moved).

**Why 8 and not 1**: three defs (`woe_strider`, `wight_of_the_reliquary`,
`vampire_gourmand`) carry notes asserting that "`Cost::Sacrifice` has no 'another' /
exclude-self semantics". **That has been false since PB-EF1** — `TargetFilter.exclude_self`
lowers to `ActivationCost.sacrifice_exclude_self` via `flatten_cost_into`
(`replay_harness.rs:4622`) and `handle_activate_ability` enforces it. Two of those notes
are corrected; the third pair still OMIT their abilities entirely on the stale belief
(`OOS-SIM6-2`). Same shape as PB-DX19's comment: the note, not the code, is why this
survived.

### Seeds filed

* **`OOS-SIM6-1`** (MEDIUM, engine) — `flatten_cost_into` reads only
  `TargetFilter.has_card_type` (**singular**) and ignores `has_card_types` (plural) and
  `colors`. `bartolome_del_presidio` / `umbral_collar_zealot` / `baron_bertram_graywater`
  print "creature **or artifact**" and lower to `SacrificeFilter::Creature`;
  `ayara_first_of_locthwain` prints "another **black** creature" and loses the colour.
  `SacrificeFilter::ArtifactOrCreature` **already exists** and the lowering never emits it.
  The direction is *narrowing* (legal plays refused), so it is not a wrong-game-state bug —
  but it makes three defs' printed text unreachable. Out of scope here only because of the
  0-engine-lines constraint.
* **`OOS-SIM6-2`** (LOW, card defs) — `wight_of_the_reliquary`, `vampire_gourmand` and
  `ruthless_technomancer` omit their sacrifice abilities on the disproved claim above.
  Re-authoring them moves coverage, so it belongs in a card batch, not here.
* **`OOS-SIM6-3`** (HIGH, simulator + human-facing) — **auto-tap covers `CastSpell` and
  nothing else.** `local_game.rs:738` returns `None` for every other command, on both the
  bot path (`advance()`) and the human path (`submit`). `can_afford` offers an activation
  whose cost is solvable *with taps*, the engine charges the *pool*, and the command is
  refused `InsufficientMana`: **62 of the 113 remaining bot refusals**, and a browser human
  activating a mana-cost ability gets a 422 unless they happened to have floating mana.
  This is the largest remaining SR-38 violation on this surface and the obvious successor.
* **`OOS-SIM6-4`** (LOW, simulator) — two engine refusals still unmirrored by the offer
  loop: `forage` (CR 701.61a, `abilities.rs:1235` — needs a Food artifact or three
  graveyard cards; **1 def** in the corpus) and `sacrifice_self` on a source under
  `CantBeSacrificed` (CR 701.21a, `abilities.rs:917`). Both are the same class as the three
  `activated_ability_is_activatable` now covers; neither has measured traffic.
* **`OOS-SIM6-6`** (LOW, latent — filed by the `/review` cycle) — the offer-time
  `activation_condition` evaluation uses `x_value: 0`, because `{X}` is not announced until
  command construction. The engine evaluates the same condition with the command's own
  `x_value` (`abilities.rs:261-271`), so an "Activate only if X is N or more" ability would
  be wrongly **suppressed** — the silent-unplayable direction, not the 422 direction.
  Unreachable today: every `Condition::XValueAtLeast` in the corpus is spell-side. Recorded
  in `activated_ability_is_activatable`'s own doc rather than left to be rediscovered.
* **`OOS-SIM6-5`** (LOW, TUI) — `tools/tui/src/play/input.rs`'s `'e'` key now routes
  through `action_to_command_with_params` (so the costs, modes and hybrid/Phyrexian plans
  are filled), but the TUI still has **no picker** for any of them: it always submits the
  plan's default. A human TUI seat cannot choose which creature to sacrifice.

### What this batch did NOT do, stated plainly

* **Multi-sacrifice is untouched** (`OOS-OS6-1` → PB-DX12). `sacrifice_target` is a single
  `ObjectId` on the wire and stayed one; nothing here reshapes it.
* **No frontend test harness still exists** (R7). The three browser flows were verified by
  hand with playwright-core; nothing automated covers `CostPicker`'s new block. The
  play-server probes cover the *channel* end to end and prove nothing about the component —
  the same limitation UI-2 and UI-4 both recorded.
* **The discard candidate list is the whole hand, unfiltered**, which is what
  `handle_activate_ability` accepts (it checks the zone and nothing else). If a def ever
  needs "discard a *land*", this descriptor has no field for it.

### `/review` cycle — 5 LOW, all 5 taken

The reviewer re-executed every load-bearing gate independently (4,312/0/5, PROTOCOL 33 /
HASH 70, fmt + clippy + defs-fmt, 0 engine lines) and confirmed by three separate reverts
that the suppression gate, the Yahenni `exclude_self` fix and the new activatable mirror
each have a test that goes red without them. All five findings were LOW; all five taken:

1. **The discard channel had no HTTP probe.** The sacrifice half did; the discard half was
   covered only by unit calls and the `params.rs` engine round-trip, so
   `activation_costs_view`'s discard block was verified in a browser by hand and by nothing
   automated. Added `test_sim6_activation_discard_is_answered_over_http` on a new mono-RED
   fixture (Lathliss commander, 99 Mountains, Rummaging Goblin) — deliberately a `{T}`-only
   ability, because an activation that ALSO costs mana fails on this surface for the
   unrelated `OOS-SIM6-3`, which would have made the probe pass or fail for the wrong cause.
   It also pins the CR 302.6 gate incidentally: the offer does not appear on the turn the
   goblin lands.
2. **An `additional_costs` array on an `ActivateAbility` was dropped in silence** — the
   mirror image of a guard this batch had just added in the same function. `params.rs`'s
   activation arm never reads that field and `ActivateAbility` sits inside its consuming
   allowlist, so `first_announced_field` could not catch it either. Now a 400, with a
   both-ways test (and a control that an activation announcing nothing is still accepted).
3. **`OOS-SIM6-6` filed** — see the seed list above.
4. README limitation numbering (the new item was inserted before, not after, item 22).
5. `docs/authoring-status.md` had been regenerated at the batch's first commit rather than
   at HEAD, so its rolling commit block was three commits stale. No count was wrong — no
   card def changed after that commit — but regenerated at HEAD anyway.

### Numbers

Tests **4,313 / 0 / 5** full workspace (+18 over SIM-5's 4,295): 11 simulator (10 for the
channel, 1 for the SR-38 mirror) + 7 play-server. Every suppression gate proven **red by
reverting the gate and watching the assertion fail**, not by inspection. `cargo fmt`,
`tools/check-defs-fmt.sh`, `clippy --workspace --all-targets -D warnings` all clean.

## Worker Handoff (SIM-5, `scutemob-188`) — bots stop wasting mana, and start announcing targets

**G5 CLOSED for its (1)/(2)/(3) halves; (4) DEFERRED with measurements (`OOS-SIM5-4`).**
The triage's chain was correct end to end and is re-verified against HEAD (pre-edit line
numbers, the ones the brief cites): the bot path built `[taps…, cast]` at
`local_game.rs:462-468` and applied them **one at a time** at `:471-472`; on failure `:474-491`
committed the taps, discarded `e`, and passed. The human path has never had that failure mode —
`submit` (`:549`) hands the identical vector to `apply_sequence` (`:700`), whose doc at `:694`
says it exists precisely to stop "a tap-then-cast sequence where the tap succeeded but the cast
was rejected". The cast was rejected because `random_bot::action_to_command` (`:142-193`) built
`ActionParams::default()` and filled only `attackers`/`blockers`, so `params.rs`'s `CastSpell`
arm (`:262`) forwarded `targets: []` and `casting.rs:5931` refused. `HeuristicBot` shares that
function (`heuristic_bot.rs:19`, called at `:346`), so **neither bot had ever cast a targeted
spell**.

### What shipped

* **(1) atomicity, `local_game.rs`** — the bot loop is now one `self.apply_sequence(commands)`
  call. Two deliberate behaviour deltas, both documented at the call site: invariants are
  checked once per *sequence* rather than once per command (the states no longer checked are
  mid-payment ones), and a recorded seed moves **only where a cast is rejected** — per
  `OOS-UI2-1` the fuzzer has never cast at all, so no fuzz seed can reach the changed branch.
* **(3) the refusal is kept** — `RejectedCommand { player, turn, command, error }`, with
  `LocalGame::rejections()` (retained, capped at `MAX_RETAINED_REJECTIONS = 256`) and
  `rejection_count()` (never truncated, so the cap is visible rather than silent). Exported on
  `GET /api/game/report` as `rejections` / `rejection_count` — that endpoint's `journal` records
  applied commands only, which is exactly the limit the triage hit ("the rejected command and
  its error string are unrecoverable").
* **(2) targeting, new `crates/simulator/src/targeting.rs`** — `plan_targets` returns
  `NotTargeted` / `Announce(Vec<Target>)` / `Unsatisfiable`, one target per **mandatory**
  requirement. Every legality decision is delegated to `crates/engine/src/rules/queries.rs`
  (`spell_target_requirements`, `ability_target_requirements`, `legal_targets_per_slot`,
  `target_count_range`); nothing re-derives a targeting rule outside the engine (the `OOS-RS-2`
  drift class). `random_bot::action_to_command` fills `params.targets` from it, and
  `HeuristicBot` inherits it through the shared function.

### Three decisions a successor should not re-litigate blind

1. **Not `Bot::choose_targets`.** The dead trait method takes `&[ObjectId]` and returns
   `Vec<ObjectId>`, so it cannot express `Target::Player` — half of what spells target. It is
   still dead; widening it is `OOS-SIM5-1`'s business, not a legality fix.
2. **Deterministic first-legal candidate, no RNG.** `legal_targets_per_slot` already enumerates
   deterministically (live players in seat order, then objects ascending). Drawing here would
   re-roll every recorded fuzz seed and every seeded play-server fixture for a *strategy* gain,
   and no layer here knows a spell's polarity anyway (removal wants an opponent's creature, a
   pump spell wants its own). Bots therefore target the lowest-`ObjectId` legal candidate,
   which for a `TargetPlayer` slot is often themselves — `OOS-SIM5-1`.
3. **Modes are queried as `spell_default_modes(state, card)`, not `&[]`.** This is the one place
   this module deliberately differs from `view.rs`'s `action_target_requirements`, which passes
   `&[]` because the *human* has not chosen yet. `params.rs` fills a bot's `modes_chosen` with
   exactly that default list, so querying with `&[]` would return `vec![]` for a
   per-mode-targeting card (`queries.rs` divergence 1) and the bot would announce nothing for a
   cast whose command *does* select a mode.

### A/B, measured both ways (instrument: `crates/simulator/tests/sim5_bot_cast_discipline.rs`)

Seeds 0/7/42, 25 turns, four heuristic bots, no human seat; the same journal walk the triage
did on `GET /api/game/report`.

| seed | wasted tap runs | wasted taps | ManaPoolsEmptied | taps | casts | targeted casts |
|------|-----------------|-------------|------------------|------|-------|----------------|
| 0    | 10 → **0**      | 20 → **0**  | 10 → **0**       | 65 → 46 | 17 → 20 | 0 → **2** |
| 7    | 15 → **0**      | 15 → **0**  | 15 → **1**       | 68 → 69 | 23 → 27 | 0 → **4** |
| 42   | 5 → **0**       | 10 → **0**  | 5 → **0**        | 55 → 60 | 19 → 22 | 0 → **1** |

The BEFORE column reproduces the triage's live 1:1 match exactly — `ManaPoolsEmptied` equals
wasted tap runs on all three seeds (10/10, 15/15, 5/5), as the triage measured 18/18.
**The one residual is explained, not waved at**: seed 7 keeps a single `ManaPoolsEmptied` at
T14, and its journal context shows a four-tap run whose cast **succeeded**, part of the
remainder spent on a second cast ~20 commands later and the rest destroyed at the step
boundary — greedy-solver slack (`OOS-SIM2-1`), not a wasted plan. `emptied_pool_context()`
uses a 40-command window for exactly this reason; a 5-command one showed only passes.

**Journal-verified targeted casts by bots** (impossible before this batch): T7 `Glacial Ray` →
player 1 and T18 `Damn` → a permanent (seed 0); T2 `Burst Lightning` → player 1, T3 `Goblin War
Strike` → player 1, T10 `Vandalblast` → a permanent (seed 7); T12 `Doom Blade` → a creature
(seed 42).

### What the recorded rejections immediately revealed (the point of fix (3))

166 refusals across the three seeds, now classifiable instead of inferable:
**~95 `InsufficientMana` on `ActivateAbility`** and **40 `activation condition not met`** — i.e.
the *activation-cost payment channel*, which is **SIM-6's** subject and untouched here; ~25
blocker-declaration refusals (`CrossPlayerBlock`, "the attacking player cannot declare
blockers", CR 508.1d must-attack) — `OOS-SIM5-3`; **4** modal `ActivateAbility` refusals
("requires exactly 1 target(s) for the chosen mode(s)", CR 700.2c) — `ability_target_requirements`
documents that a modal ability's per-mode slice is out of its scope, so a bot cannot announce
for one (`OOS-SIM5-5`); and **1** genuinely unsatisfiable cast (`Victimize`, no creature card in
the graveyard). Cast-side refusals are now ~3% of the total.

### Why fix (4) is deferred rather than shipped (`OOS-SIM5-4`)

Full argument and numbers in `targeting.rs`'s `TargetPlan::Unsatisfiable` doc. In short: the
predicate exists and the filter is short, but it would have suppressed **1 of 166** refusals;
it does **not** cover `OOS-CARDS2-4` (an Aura's restriction is a `KeywordAbility::Enchant`, not
a `TargetRequirement`, and `rules::sba::get_enchant_target`/`matches_enchant_target` are
`pub(crate)` — covering Auras needs an **engine** query this batch may not add); it costs a full
candidate sweep per offered cast per priority window on a path `queries.rs` itself says to
measure and cache first; and shortening the action list re-rolls every recorded fuzz seed and
seeded fixture, since `RandomBot` picks `rng.random_range(0..legal.len())`. Post-(1) an
unsatisfiable offer costs nothing anyway. Scope it as an engine query plus caching.

### Gates (each new gate proven to discriminate by executing a revert, not by assumption)

* `crates/simulator/tests/sim5_bot_cast_discipline.rs` — `seeded_four_bot_game_wastes_no_taps`
  (the A/B instrument; red both on the pre-fix per-command loop **and** on pre-fix zero-target
  params), `bot_announces_a_legal_target_and_the_engine_accepts_the_cast` (a black and a
  colourless creature on board: the bot must pick the non-black one *and* `process_command` must
  accept the command), `plan_targets_reports_an_unsatisfiable_requirement`,
  `a_rejected_bot_cast_commits_no_taps` (no land tapped, no mana floating, no tap in the
  journal, refusal recorded — pinned with a frozen `ZeroTargetCastBot` so it keeps testing
  ATOMICITY even as targeting improves).
* `tools/play-server/src/main.rs` `test_sim5_report_exposes_bot_command_rejections` — asserts
  the two report fields on every iteration (so a dropped field fails even with no rejection) and
  has a non-vacuity floor requiring a real refusal; went red when `record_rejection` was stubbed.
* **0 engine lines** (`git diff main..HEAD --numstat -- crates/engine/` is empty),
  PROTOCOL **33** / HASH **70** unmoved and gate-executed. Workspace suite **4,295 / 0 / 5**
  (+5 = this batch's gates), captured to a file, never tail-piped. `fmt`, `clippy -D warnings`
  and `tools/check-defs-fmt.sh` all clean. **No seeded fixture moved** — the `UI1_SEED`/
  `UI2_SEED`/`SIM1_SEED` pins and the six SEED-0 play-server probes were green untouched, which
  is why nothing in this handoff explains a moved pin.

### Seeds filed

* **`OOS-SIM5-1`** — bot target *choice* is "first legal candidate", and `legal_targets_per_slot`
  lists players before objects **in seat order**, so every player-eligible slot (`TargetPlayer`,
  `TargetAny`, `TargetCreatureOrPlayer`) resolves to **seat 1** — the human's seat in a
  play-server game, and the bot's own seat when the bot is seat 1. **Not a cosmetic seed**: it
  points every bot burn spell at one player, which changes the character of a seeded game and
  not merely its strategic quality. `Bot::choose_targets` is still dead and cannot express
  player targets at all. A real policy (opponent-preferring for removal, self-preferring for
  buffs) needs spell polarity, which is a `HeuristicBot` scoring project.
* **`OOS-SIM5-2`** — `TargetRequirement::UpToN` slots are announced empty (legal: min 0), so a
  bot never uses an optional target.
* **`OOS-SIM5-3`** — ~25 of 166 refusals are blocker declarations the provider offered and the
  engine refused (`CrossPlayerBlock`, "the attacking player cannot declare blockers", CR 508.1d
  must-attack). Pre-existing SR-38 residue in `legal_actions.rs`'s combat surface; now visible
  because rejections are recorded.
* **`OOS-SIM5-4`** — fix (4) deferred; see above and `targeting.rs`.
* **`OOS-SIM5-5`** — a modal **activated ability** with per-mode targets is unannounceable by any
  caller of `ability_target_requirements` (which documents the per-mode slice as out of scope),
  so bots refuse 4× per A/B run. Needs an engine query change, not a simulator one.
* **`OOS-CARDS2-4` unchanged** — Auras still cannot be announced; post-(1) the attempt is a
  harmless no-op that now shows up in `rejections()`.

### The `/review` cycle: 5 PASS, 4 LOW, all 4 taken

The reviewer re-ran everything rather than trusting the numbers — it reverted both fixes in a
scratch tree and reproduced the BEFORE column exactly, then reproduced AFTER on HEAD, then
re-ran the full workspace suite. Four LOW findings, all applied:

1. An in-source A/B summary in `local_game.rs` said "30 wasted taps across 30 tap runs". The
   verified figures are **45 wasted taps across 30 wasted runs, of 82 tap runs in all** — 30 is
   the wasted-*run* count, which is what `ManaPoolsEmptied` matches 1:1. Comment corrected; the
   handoff table and task comment were already right.
2. `record_rejection` retained records regardless of `LocalGameLimits::record_journal`, while
   `driver.rs` sets that flag `false` specifically so the fuzzer retains nothing. Retention is
   now gated on the same flag (the **count** is not gated, so a crash report keeps the number).
3. `OOS-SIM5-1` was under-stated: players are enumerated first *in seat order*, so every
   player-eligible slot resolves to seat 1 for every bot. Seed text strengthened above and in
   `plan_targets`' doc — it changes a seeded game's character, not just its quality.
4. Measured: with targeting kept and only `apply_sequence` reverted, only seed 42 reddens the
   whole-game A/B test, because fix (2) removed nearly all cast-side refusals. So the A/B test
   is **not** the primary atomicity gate — `a_rejected_bot_cast_commits_no_taps` is, and it
   freezes a `ZeroTargetCastBot` into the fixture exactly so it keeps discriminating however
   good targeting becomes. Recorded in that test's doc so a future seed re-pick cannot lose it.

## Worker Handoff (SIM-4, `scutemob-187`) — the mulligan stops re-rolling the table

**G2 CLOSED. CR 103.5: a mulligan permutes a FIXED library-plus-hand multiset.** The
triage's chain was correct end to end; re-verified against HEAD (post-edit line numbers):
`PlayApp.svelte:478` (`Take a mulligan`) → `main.rs:184` (`.route("/game/mulligan", …)`) →
`api.rs:1236` `post_mulligan` → `:1260` `play.mulligan()` → `session.rs:422` →
`session.rs:428` `setup::redeal(&self.cfg, …)` → `setup.rs:503` `redeal` (perturbed seed,
`..cfg.clone()`) → `setup.rs:319` `build_initial_state` → `deck.rs:53`
`commanders[rng.random_range(..)]`. **The load-bearing link is `self.cfg`**: it held
`DeckSource::RandomPerSeat`, a seeded *recipe* in which every card of every seat — the
commander included — is a function of `cfg.seed`, so a perturbed seed re-rolled all four
decklists and all four commanders. CR 903.6 puts the commander in the **public** command
zone, which is why the playtester saw it on three opponents at once.

### The brief's fix was implemented, measured, and replaced — read this before re-proposing it

The brief said: factor `setup.rs`'s deck-resolution block into `resolve_decks(cfg)`, have
`session::new_game` store `DeckSource::Fixed(resolved)`. That was built first. **It reddens
seven tests**, and the reason is structural rather than incidental: `build_initial_state`
draws a seat's deck and then shuffles *that seat* before drawing the next seat's deck, all
off one `StdRng`, so **seat 2's decklist depends on seat 1's shuffle**. Any two-pass
factoring moves the stream, and moving the stream re-rolls every table every existing seed
builds. Measured, not predicted:

* six `tools/play-server` probes that pin card names at `SEED = 0`
  (`test_get_game_returns_seat_view_with_seven_card_hand`,
  `…pass_priority_advances_and_bots_act`, `…no_other_hand_card_names`,
  `test_ui3_combat_view_…`, `test_x_value_is_forwarded_…`, `…illegal_target_returns_422`);
* `local_game_playthrough` seed 1, which landed on a deck holding an **Aura** and died on
  `"engine rejected a just-offered action (CastSpell): Aura spells require exactly one
  target (CR 303.4a)"` — a **pre-existing** engine/legal-action defect the new table merely
  exposed, unfixable inside a 0-engine-lines task and not something to paper over by
  changing the test's seeds. Filed as **OOS-SIM4-2**.

**Shipped instead: `setup::dealt_decks(&state, &cfg)` (`setup.rs:238`)** — read the
decklists that were *actually dealt* back out of the built `GameState` (hand ∪ library, plus
the registered `commander_ids`). `session::new_game` (`session.rs:237`) builds from the
unmodified cfg, then pins the result: `:240` `dealt_decks` → `cfg.decks = Fixed(dealt)`.
This moves **no table at all** (all six SEED-0 pins stayed green, which is the evidence),
and it is the stronger guarantee: the multiset a mulligan permutes is the one the player was
literally dealt, not one re-derived from a config believed to agree. `setup::redeal` needed
**zero** changes — with `Fixed` decks its perturbed seed reaches only the shuffle.

### Gates

* `crates/simulator/tests/setup.rs:392` `test_redeal_preserves_every_seats_deck_and_commander`
  — the pin the brief demanded: for **every** seat, the 100-card multiset and the registered
  commander are identical across a redeal (with a 100-card non-vacuity floor), the
  command-zone *object* is still that commander, and seat 1's hand still changes and is
  still 7. Plus `:505` round trip, `:551` determinism + refusal, `:583` the shape floors.
* `tools/play-server/src/main.rs:7149` (P1, over the real router — two mulligans, all four
  public command zones compared) and `:7232` (P2, direct — the session *holds* `Fixed`, and
  every seat's hidden 100-card multiset survives). **Both proven red by executing the
  revert**: pre-fix P1 reports all four commanders replaced on mulligan 1.
* **The simulator gate alone could never have caught this** and it is worth knowing why:
  `DeckSource::Fixed` was always immune, so a simulator-level test passes whatever the play
  server chooses to store. The defect lived in what the *session kept*. A gate on the
  primitive does not gate the caller's choice of argument.
* `test_redeal_on_an_unresolved_recipe_still_rerolls_the_decks` (`:484`) deliberately pins
  the un-fixed path, so the caller's obligation is visible rather than folklore.

### Deferred, with reasons

* **Per-seat RNG streams** (the brief's optional residual): NOT implemented. Two reasons,
  both concrete. (1) Keying each seat's shuffle on `(seed, pid)` does not isolate anything —
  `redeal` perturbs `cfg.seed`, so every derived stream still moves; real isolation needs
  per-seat mulligan *counts* in the config, i.e. the per-seat pregame model that needs a
  decision channel for bot seats. (2) Re-deriving the shuffle seed at all would move the
  opening hands that `UI1_SEED`/`UI2_SEED`/`SIM1_SEED` pin **by original index** — five
  shipped CR flows rest on those fixtures.
* **OOS-G2-3** (dead `Command::TakeMulligan`/`KeepHand`, turn-0 gate never satisfiable):
  UNCHANGED, out of scope by the brief.

### Seeds filed

* **OOS-SIM4-1** — `setup::redeal` still accepts a `RandomPerSeat` config silently, so G2 is
  prevented by caller discipline, not by construction. `tools/tui/src/play/app.rs:132` builds
  exactly such a config and would reintroduce the defect verbatim the day the TUI grows a
  mulligan (a pointer comment now sits at that construction site). The structural fix is a
  `redeal` that takes the dealt state, or a `DeckSource` that cannot be a recipe past the
  first build.
* **OOS-SIM4-2** — `local_game_playthrough`'s policy submits a just-offered `CastSpell` for
  an **Aura** and the engine rejects it with CR 303.4a. Pre-existing (the seed-1 table only
  changed because of an experiment that was reverted), engine-side, and a genuine
  legal-action bug: an offered action must be applicable. Reproduce by two-passing
  `build_initial_state`'s deck/shuffle loop and running `--test local_game_playthrough`.
* **OOS-SIM4-3** — `dealt_decks` refuses a two-commander seat (CR 903.3 partner/background)
  because `DeckConfig` has one `commander` field. Correct today (nothing builds one), but it
  is the shape that will bite when partner decks arrive.

### Durable lesson

**A limitation documented by its mechanism does not warn anyone; document the consequence.**
Four separate doc blocks (`setup.rs`, `session.rs`, `api.rs`, `PlayApp.svelte`) described the
whole-table rebuild — one even named the commander re-roll — and none said "the players' decks
change". The playtester was the first to say it in those words. All four now do, plus
`tools/play-server/README.md`'s known-limitation 1 and its bug-report reproduction procedure,
which had quietly become **wrong** (rebuilding at the derived seed with the recipe no longer
reproduces a mulliganed table; it now takes base-seed build → `dealt_decks` → rebuild at the
derived seed with `Fixed`).

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
  branch. **DEAD REPRO across the PB-DX22 merge (`scutemob-196`, `95f53b78`)**: that batch shuffles the
  fuzz libraries and registers the commanders, so seed 504 deals a different game and no
  longer reproduces this one. The seed is a pre-merge artefact — see `OOS-DX22-7`; the
  defect itself is closed by PB-DX19 (`451e3517`) regardless.
  Diagnosed by `gdb` backtrace plus a depth probe that named the card. Very likely
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

**Date**: 2026-08-05 (oversight session #6 — v3 rank 7, single dispatch)
**Workstream**: W6 correctness queue (v3)
**Task**: `scutemob-203` (PB-DX25, merge `f8ed9618`), dispatched and collected same evening.

**Completed**:
- **PB-DX25 shipped** (rank 7): counter-on-mutate silent no-op closed. Structural fix — a new
  engine-side `state::stack_registry::card_in_stack_zone`, exhaustive over `StackObjectKind`
  with no wildcard, consumed by BOTH counter paths, so a 28th kind is a compile error until
  classified. The simulator's `stack_card_of` deliberately NOT unified with it (a verifier
  reading the engine's own answer goes silent on exactly the defect it exists to catch).
- **The seed and the queue row had the live shape backwards**: (c) was the only live shape;
  (a) was never independently reachable — Ward cannot reach a mutate spell because the mutate
  target rides `AdditionalCost::Mutate` and never enters `spell_targets` (`OOS-DX25-1`) — so
  (a) is what fixing (c) ALONE would have created, a permanent `ZoneId::Stack` leak in place
  of a silent no-op. (b) is unreachable three independent ways. Live-wrong population
  re-measured: **66** pairs, not the row's implied 144. All corrections written into the
  registry row and the v3 §3 row in place.
- Tests **4,435 → 4,452 / 0 / 5**; PROTOCOL **35** / HASH **73** gate-executed and unmoved
  (prediction held); coverage unmoved **1,133/1,803 = 62.8%** proven by regeneration; benches
  within noise (`full_turn_4p` 214-215 µs).
- Review 0 HIGH / 6 MEDIUM / 3 LOW + 7 folded notes, **all taken** — its sharpest findings
  were the batch's own failure mode recurring inside it (a census short by two sites; a roster
  blind to a delegating variant; a non-vacuity assertion comparing a fixture to itself).
- **OOS-SIM3-5 CLOSED**; **OOS-DX25-1..6 filed** (registry grep-checked per the dedup rule).
  Worker did FULL collect state-sync (queue row struck, W6 row, CLAUDE.md delta, registry) —
  verified, not assumed, at `/collect`.

**Not done / deferred** (inherited set, unchanged):
- Feedback doc rows 2 (FUZZ-CRASH) / 4 / 5 / 6 / 7 / 8 undispatched; **OOS-DX22-8**
  unclassified; **OOS-DX32-1** undiagnosed; v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not
  rowed into §8.1; `scutemob-127` still backlog.

**Next session candidates** (highest-yield first):
- **Read `OOS-DX25-3` first — LIVE on 2 deck-legal `Complete` defs**: `misdirection` and
  `bolt_bend` can NEVER resolve a legal target (`TargetSpellWithSingleTarget` compares a card
  id to a stack-entry id across disjoint id namespaces; the in-src negative tests pass
  vacuously). Weigh as an insert before PB-DX26.
- **PB-DX26** (rank 8 — the equip surface one link earlier; ~4-6 flips; re-measure the
  21/18/10 roster from `all_cards()` at dispatch per v3 §2.7).
- **OOS-DX32-1 diagnosis** or **FUZZ-CRASH** (feedback row 2, cheapest row).

**Hazards** (carrying forward):
- The three standing #5 hazards (registry-grep dedup rule; Monitor over bash poll loops;
  verify worker state-sync at `/collect`) all held this session — PB-DX25's worker synced
  fully.
- New from PB-DX25: `next_object_id` mints stack-entry ids and card ids from ONE counter, so
  an id lives in exactly one namespace — any `so.id == <card id>` comparison type-checks and
  can never match. `OOS-DX25-3` is a second instance of the same class one function over from
  the seed's. Grep for the pattern before trusting any stack-lookup-by-id.

**Commit prefix used**: `scutemob-203:` (worker) + `merge:` + `chore:` (eot)

## Previous Handoff (preserved for chain context)

**Date**: 2026-08-04..05 (oversight session #5 — correctness-queue run, v3 ranks 2/3/5/6)
**Workstream**: W6 correctness queue (v3)
**Task**: five tasks: `scutemob-199` (OOS-FB1 filing — DUPLICATE of `scutemob-195`, deduped at
`/eot`, see hazards; `e7edcdd1`), `scutemob-198` (PB-DX20, merge `ecd7b119`), `scutemob-200`
(PB-DX21, `e490153b`), `scutemob-201` (PB-DX23, `49958549`), `scutemob-202` (PB-DX24, `7b3d7d58`).

**Completed**:
- **PB-DX20 shipped** (rank 2): offer layer sees keyword-carried target requirements — ONE shared
  derivation (`casting::enchant_target_to_requirement`); 13 `Complete` Auras castable in the
  browser; Reconfigure synth site carries `exclude_self: true` (CR 702.151a); the whole
  `KNOWN_FALSE_OFFERS` excusal mechanism deleted. Brief correction: the "4 no-Enchant Auras" set
  was a grep artefact — the T4 roster gates over `all_cards()` (SR-36).
- **PB-DX21 shipped** (rank 3): CR 508.1 once-per-combat guard. The brief's PREFERRED mechanism
  (read `combat.attackers`) was **refuted three ways** (CR 508.1a "if any" + CR 508.8: an EMPTY
  declaration is a completed action, live via `params.rs:474`) → `CombatState::attackers_declared`
  bool, **HASH 72 → 73 gate-computed**. Both client-side mitigations deleted; 3 discriminating
  probes; refuted advice left standing in the brief with the reasoning.
- **PB-DX23 shipped** (rank 5): `LegalAction::ChooseDredge` end-to-end (bot + browser);
  Grave-Troll draw-cadence probe on a real game; OOS-DX2-2 tail flip with the PB-DP5 §3.3
  distinction argued in the commit; OOS-DX2-7 AUTO-CHOSEN row added; **OOS-DX2-3 stays REOPENED**,
  pin byte-unedited.
- **PB-DX24 shipped** (rank 6): `trigger_zone` honoured structurally at the single lowering call
  site (not 34 per-arm edits); graveyard death dispatch built — beyond the brief, the sweep
  handled only `PermanentEnteredBattlefield`; OOS-DX1-4 six of seven sites fixed, seventh
  re-scoped with reason; both seeds CLOSED with their own row-claim corrections.
- Tests **4,373 → 4,435 / 0 / 5**; full suite re-verified on merged main after EVERY collect
  (4,388 / 4,398 / 4,413 / 4,435, all exit 0); PROTOCOL **35** unmoved throughout; HASH **72 → 73**
  (PB-DX21 only); coverage unmoved **1,133/1,803 = 62.8%**.
- **OOS-FB1 double-filing found and deduplicated at `/eot`**: `scutemob-199` re-filed what
  `scutemob-195` had already filed (stale "NOT filed" CLAUDE.md bullet); nine duplicate rows
  removed, the chain-verified `scutemob-199` set kept with the older set's two unique facts
  folded in; banners corrected in registry + feedback doc + CLAUDE.md.

**Not done / deferred**:
- Feedback doc rows 2 (FUZZ-CRASH) / 4 / 5 / 6 / 7 / 8 still undispatched; **OOS-DX22-8** still
  unclassified; **OOS-DX32-1** still undiagnosed.
- Inherited: v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not rowed into §8.1; `scutemob-127`
  still backlog.

**Next session candidates** (highest-yield first):
- **PB-DX25** (rank 7 — `Effect::CounterSpell`'s three stack-object shapes; a countered spell
  resolves anyway, silently). Table-only rank: write the brief at dispatch from the seed rows,
  re-verify premise first.
- **OOS-DX32-1 diagnosis** or **FUZZ-CRASH** (feedback row 2, cheapest row, OOS-DX22-7 feeds it).
- **OOS-ADJ-1..7 rowing into §8.1** (small, closes an inherited deferral) — grep the registry for
  each ID first, per the new dedup rule.

**Hazards** (carrying forward):
- **Seed-filing dedup rule (new, learned the hard way)**: before filing any OOS seed, grep
  `docs/audits/decision-point-audit.md` for the ID — the registry is ground truth; status bullets
  in CLAUDE.md/handoffs lag it (OOS-FB1-1..9 was double-filed exactly this way).
- Monitor tool over bash poll loops for worker watches — bash loops were killed within ~2 min
  repeatedly this session; one persistent Monitor per worker was reliable.
- Workers now do their own collect state-sync inconsistently (DX21/DX24 fully, DX20/DX23
  partially) — `/collect` step 7 must still verify the queue-memo row strike + brief banner.

**Commit prefix used**: `scutemob-N:` (workers/self-task) + `chore:` (collects, eot) + `merge:`

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

## Handoff History

### 2026-08-03..04 (oversight #4 — FEEDBACK-1 + first two feedback-buildout batches) [rotated]

**Date**: 2026-08-03..04 (oversight session #4 — FEEDBACK-1 + the first two feedback-buildout
batches, user-directed "stop after 3 tasks for a check-in")
**Workstream**: W6 correctness queue + feedback-engineering track
**Task**: four tasks dispatched/collected serially: `scutemob-192` (FEEDBACK-1 planning, merge
`d55e74cc`), `scutemob-195` (OOS-FB1 seed filing, coordinator-inline, `9aa4f220`),
`scutemob-196` (PB-DX22, `95f53b78`), `scutemob-197` (PB-DX32, `685aa1c4`).

**Completed**:
- **FEEDBACK-1 shipped** (doc-only): `docs/mtg-engine-feedback-engineering.md` — 14-channel
  inventory, 8-row ranked proposal table, alpha-loop ownership table, 18 from/to corrections.
  Registered in `.claude/docs.yaml` (25 templates) + the CLAUDE.md primary-docs table. Its four
  coordinator-notes (decision gate exists / crash pipeline absent / two rows already queued /
  rejection channel bot-only) are in the ESM task comments (scutemob-183 pattern).
- **OOS-FB1-1..9 filed** into `docs/audits/decision-point-audit.md` §8.1 (`scutemob-195`).
- **PB-DX22 shipped** (v3 rank 4): fuzzer shuffles from the game's seeded RNG + registers
  commanders in both builders (new shared `crates/simulator/src/fuzz_setup.rs`); first-cast
  turn 143-154 → **3-29 band**; CR 903.8/903.9a/903.10a fuzzed for the first time; fuzz games
  END (20 wins / 0 errors vs 9/11 timeouts). The §2.4 open measurement settled: the commander
  offer was SUPPRESSED (empty `commander_ids`), OOS-SIM1-4 the cause. **OOS-UI2-1 / OOS-SIM3-1 /
  OOS-SIM1-4 CLOSED**; OOS-DX22-1..11 filed; every pre-merge fuzz seed dead (OOS-DX22-7); the
  repaired instrument's first real find is **OOS-DX22-8** (attachment_validity transient).
- **PB-DX32 shipped** (v3 rank 19, PROMOTED per feedback doc §2.3): `GameResult` carries the
  SR-38 rejection invariant + promoted waste tally behind measured-at-HEAD ratchets (2.30%
  rejection rate; wasted taps 1,986/2,641); orphan-token noise floor gets the transient/end-state
  treatment; violations deduped by condition; fuzz deck pool size gated (**OOS-CARDS2-3 CLOSED**);
  decision-point runtime coverage counter (reached-vs-ROWS). **OOS-SIM3-3 / OOS-SIM3-4 CLOSED,
  OOS-SIM3-2 PARTIAL**; OOS-DX32-1..10 filed. Review 0 HIGH / 8 MEDIUM / 10 LOW, all 18 taken.
- Tests **4,345 → 4,373 / 0 / 5**; PROTOCOL **35** / HASH **72** unmoved by every batch,
  gate-executed each time; coverage unmoved **1,133/1,803 = 62.8%**.
- **Lean-bullet evaluation gate PASSED** at `/start` (UI-6 reconstructed from its bullet plus one
  pointer-follow) — the lean form stands, no rollback.

**Not done / deferred**:
- Feedback doc rows undispatched: **2 FUZZ-CRASH** (now the cheapest row; OOS-DX22-7 feeds it),
  **4 HTTP-FUZZ** (yield gated on OOS-SIM6-3), **5 R7-HARNESS**, **6 DECK-CHANNEL** (re-rolls
  seeds again — batch with card-def work), **7 CI-POLICY** (needs the OOS-FB1-6 timing
  measurement first), **8 REPORT-LOOP**.
- **OOS-DX22-8** unclassified (classify before fixing — OOS-M11-7 SBA-lag family candidate);
  **OOS-DX32-1** undiagnosed (player_consistency = 26.8% of a run, now what --stop-on-error
  halts on).
- Inherited from oversight #2: v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not rowed into
  §8.1; `scutemob-127` still backlog.

**Next session candidates** (highest-yield first):
- **OOS-DX32-1 diagnosis** (PB-DX32's flagged successor) or **FUZZ-CRASH** (feedback row 2).
- **PB-DX20** (standing queue next — 13 `Complete` Auras unplayable in the browser).
- **OOS-SIM6-3** (unlocks HTTP-FUZZ row 4's yield and 62 of 113 residual bot refusals).

**Hazards** (carrying forward):
- `esm task create --criteria` is REPEATABLE, not pipe-separated — a pipe-joined string becomes
  ONE mega-criterion (scutemob-196 shipped that way; workable, avoid). A `backlog` task cannot
  be archived; reuse it rather than recreate.
- Every fuzz baseline pinned before `95f53b78` is dead (OOS-DX22-7) — re-measure at HEAD, never
  quote SIM-3/SIM-5 numbers.

**Commit prefix used**: `scutemob-N:` (workers/self-task) + `chore:` (collects) + `merge:`

### 2026-08-02 (oversight #3 — playtest-triage-2 successor run, rows 2-8) [rotated]


**Date**: 2026-08-02 (oversight session #3 — playtest-triage-2 successor run, rows 2-8)
**Workstream**: playtest-triage-2 successor track (SIM/ENG/UI)
**Task**: seven tasks dispatched serially and collected same-day: `scutemob-187` (SIM-4, merge
`dcb1fe55`), `scutemob-188` (SIM-5, `e185a2ff`), `scutemob-189` (SIM-6, `ee99929d`),
`scutemob-190` (UI-5, `08dc4e6a`), `scutemob-191` (ENG-1, `a3b5e56b`), `scutemob-193` (ENG-2,
`4ab68fdc`), `scutemob-194` (UI-6, `dd5cb47d`). **The triage-2 successor table is COMPLETE (8/8
rows shipped)**; every row ✅-marked in `memory/playtest-triage-2026-08-02b.md`.

**Completed**:
- **G2/G5/G4/G8+G10-13/G3/G7/G9 all CLOSED** — per-batch detail in each Worker Handoff above and
  the lean CLAUDE.md bullets (per the new `memory/decisions.md` 2026-08-02 lean-bullet schema,
  first applied this session; ENG-1/ENG-2/UI-6 workers wrote theirs in-schema unprompted).
- Tests **4,263 → 4,345 / 0 / 5** across the run; PROTOCOL **33 → 35** / HASH **70 → 72** (ENG-1
  and ENG-2, both gate-computed); coverage unmoved **1,133/1,803 = 62.8%**.
- **FEEDBACK-1 created** (`scutemob-192`, backlog): planning task for the alpha feedback-loop
  buildout (HTTP browser-path fuzzer, rejection/waste/decision-point invariants, R7 harness,
  steered decks, CI integration). **Deliberately NOT dispatched — user wants a fresh session.**
- Ceremony decision recorded (`memory/decisions.md` 2026-08-02): lean close-out bullets, cut
  explanation never identifiers, lean dispatch briefs from ENG-2 onward. Evaluation gate = next
  `/start` reconstructing the run from lean bullets.
- Mid-run incident: kitty crashed during ENG-2 (`scutemob-193`); worktree survived with 9 clean
  commits; worker relaunched with a verify-don't-reimplement resume prompt (user-approved) and
  re-ran the browser verification whose evidence died with the crash.

**Not done / deferred**:
- **FEEDBACK-1 (`scutemob-192`) dispatch** — waits for a fresh Claude Code session by user request.
  - **→ DONE 2026-08-03** (oversight #4): dispatched, collected, merge `d55e74cc`, doc-only
    (`docs/mtg-engine-feedback-engineering.md`); handoff lives in ESM task comments
    (scutemob-183 pattern); OOS-FB1-1..9 specified in doc §5 but NOT yet filed.
- Inherited from oversight #2 (see Previous Handoff): v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7
  not rowed into `decision-point-audit.md` §8.1; `scutemob-127` still backlog.
- Successor candidates flagged by workers, unranked: **OOS-SIM6-3** (bot/human mana-cost
  activation auto-tap — 62 of 113 residual refusals), **OOS-ENG1-9** (suspend-rollback question
  labels), **OOS-ENG2-1+2** (Ward never fires on a triggered ability).

**Next session candidates** (highest-yield first):
- **Dispatch FEEDBACK-1** (`scutemob-192`) from the fresh session — brief is complete in ESM.
- **PB-DX20** (v3 queue next) or the worker-flagged seeds above once FEEDBACK-1's plan lands.
- Third human playtest — the run closed every functional finding from playtest 2; the success
  criterion adopted for the feedback plan is "playtest 3 triages to UX-only".

**Hazards** (carrying forward):
- All oversight-#2 hazards stand (verbatim working_branch attests; commit brief inputs to main
  pre-dispatch; both-append CLAUDE.md conflicts → union-merge, demote to Prior).
- kitty crash kills all worker tabs but NOT worktrees — recovery = relaunch in the same worktree
  with a resume prompt; check `git log main..HEAD` + `git status` before assuming loss. Worker
  relaunch requires explicit user approval (retraction rule).
- `~/.local/bin` can drop off the coordinator shell PATH after a kitty crash — `export
  PATH="$HOME/.local/bin:$PATH"` per call.

**Commit prefix used**: `scutemob-N:` (workers) / `merge:` / `chore:`


### 2026-08-02 (oversight #2 — OOS pivot: re-rank v3, triage 2, PB-DX19, UI-4, adjudication) [rotated]

**Date**: 2026-08-02 (oversight session #2 — OOS pivot: re-rank v3, triage 2, PB-DX19, UI-4, adjudication)
**Workstream**: W6 correctness queue + playtest-triage-2 track
**Task**: five tasks dispatched and collected same-day: `scutemob-182` (seed re-rank v3, merge
`131716d6`), `scutemob-183` (playtest triage 2, `99aba4a8`), `scutemob-184` (PB-DX19, `451e3517`),
`scutemob-185` (UI-4, `b031d39e`), `scutemob-186` (adjudication, `8b069ae2`).

**Completed**:
- **Queue re-ranked twice with evidence**: v3 memo (`seed-rerank-2026-08-02.md`, PB-DX7..DX41; the
  v2 queue had never seen PB-DX1..DX5's 29 seeds), then adjudication `scutemob-186` inserted
  PB-DX42a (rider on DX8) / PB-DX42b (rank 13) — v3 §4 table NOT re-rowed, read with adjudication §5.
- **OOS-SIM2-6 (only HIGH) + OOS-SIM2-5 CLOSED** (PB-DX19): fuzzer 0/15 SIGABRT → 15/15 completed;
  29 checked-arithmetic edits incl. two sign-wrapping `as i32` casts; known pinned deviation:
  animated Nexus no longer feeds Metalcraft (OOS-ADJ-1/OOS-DX19-2 → PB-DX42b).
- **UI-4 (G1) SHIPPED**: Confirm was dead in all three pickers (`structuredClone` on `$state`
  proxy); five CR flows (search/scry/surveil/sac-costs/Squad) work in a browser for the first time;
  R7 harness proposed with the `$state()` fixture rule; two source gates + `$viewer` scan hole fixed.
- **Playtest triage 2** (`playtest-triage-2026-08-02b.md`, G1-G13): 5 new defects (G1 UI-4 done;
  G2 mulligan re-rolls decks CR 103.5; G3 effect-discard has no decision point; G4 activation-cost
  payment channel absent; G5 non-atomic auto-tap), 1 known limitation (G6=R4), 6 UX items; proposed
  tasks UI-5/UI-6/SIM-4/5/6/ENG-1/2 with sequencing constraints (SIM-5∦SIM-6; ENG-1+2 may merge).
- **Adjudication**: external review's durable architecture CR-correct, its immediate patch has no
  CR warrant (613.8b = timestamp order, never inactivity); deviation measured at 7 deck-legal pairs;
  seeds OOS-ADJ-1..7 (registry-of-record: adjudication §6) incl. OOS-ADJ-7 blood_moon strips
  Artifact card type (ride PB-DX27).
- CLAUDE.md wave-4 rotation completed (CARDS-1/SIM-1 bullets to archive, 711→678 lines); external
  findings doc + testing notes 2 committed (`277e60d7`).

**Not done / deferred**:
- v3 §4 table not re-rowed with DX42a/b (pointer note in this file instead).
- OOS-ADJ-1..7 not rowed into `decision-point-audit.md` §8.1 (adjudication §6 is
  registry-of-record until then).
- Triage-2 successor tasks (SIM-4/5/6, ENG-1/2, UI-5/6) not created in ESM yet.
- Tests full-tree re-measure after UI-4 merge pending (4,281/0/5 at `451e3517`; play-server 57
  green at `b031d39e`; nominal 4,283).

**Next session candidates**:
- **SIM-4** (G2 mulligan deck-swap, ~40-60 lines, needs the deck-unchanged-across-redeal gate) —
  highest user-visible value.
- **PB-DX20** (v3 queue next; re-word OOS-DX19-2 framing per OOS-ADJ-3 before any DX42b dispatch).
- **PB-DX8 + DX42a rider** (test-only gate pair).
- UI-5 UX batch (brief must forbid hiding TapForMana; resolve shared-component question up front).

**Hazards** (carrying forward):
- Attest `working_branch` with the LITERAL string from `esm worktree create` output — a command
  substitution can race and record empty, and an empty attest breaks `esm worktree check/merge`
  (fall back: `git merge-tree --write-tree main <branch>` + manual merge; hit on `scutemob-186`).
- Any input doc a task brief references MUST be committed to main BEFORE dispatch — worktrees
  branch from main and do not see untracked coordinator files (hit on the external findings doc).
- Both-append CLAUDE.md/workstream-state conflicts remain routine in parallel waves: union-merge,
  demote the older bullet to Prior.

**Commit prefix used**: `scutemob-N:` (workers) / `merge:` / `chore:`

### 2026-08-02 (oversight — playtest-successor run 174-181) [rotated]

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

### PB-DP suite — worker close-out detail

> Rotated out at /eot 2026-07-27. The per-batch close-outs (DP2..DP10 designs, deviations,
> seed lists) live in: CLAUDE.md "Last Updated" (DP9/DP10 verbatim), the audit doc
> `docs/audits/decision-point-audit.md` §5/§8/§8.1 (every row updated at ship time), and the
> merge commits listed in the Last Handoff above.
