# CLAUDE.md Changelog Archive — 2026-07

> Verbatim archive of the reverse-chronological "Last shipped" and "Last Updated"
> narratives that lived in CLAUDE.md's Current State section, extracted 2026-07-18 by
> DOC-1v2 (`scutemob-125`). Nothing here was summarized or dropped — this is the
> byte-for-byte history moved out of CLAUDE.md so that file could return to a true
> snapshot.
>
> **Recurrence rule:** future `/collect` and milestone-close bookkeeping appends its
> detailed PB/SR narrative here (newest first, matching the prior style), leaving only a
> one-paragraph snapshot delta in CLAUDE.md's Current State. Start a new dated archive
> file (`claude-md-changelog-YYYY-MM.md`) when the month rolls over.

---

## Rotated 2026-08-02 from CLAUDE.md's "Last Updated" window (July-dated entries)

- 2026-07-31 — **SEED RE-RANK SHIPPED (`scutemob-159`): the PB-RS queue is retired and the successor
  queue is `memory/primitives/seed-rerank-2026-07-27.md` §4, PB-DX1..PB-DX18.** Docs-only, 0
  engine/card-def edits, tests **3,928 / 0** unchanged, PROTOCOL 31 / HASH 68 untouched. The census:
  **209** `OOS-*` tokens across the repo → 207 distinct seeds → **204 real** after striking three
  phantoms, one of them found here (**`OOS-RS1-2`**, conditional in `pb-review-RS1.md` and never
  filed because the fix it was the fallback for was applied — `effects/mod.rs:3568-3577` reads
  `Zone::top_n`). All six closures the brief listed as settled were re-verified **against shipped
  code, not banners**, and all six hold — **and a seventh was found that no document records:
  `OOS-RS3-1` was closed by PB-DP6 (`scutemob-154`)**, all five `CardDefETB` sweeps now gating at
  queue time (`turn_actions.rs:310/483/561/781/1945`, 14 gated sites workspace-wide), while the RS
  doc's §5 banner went on naming it the next insert candidate for a week — the `N4 re-dispatch
  hazard`, live. **Four premise corrections changed the ranking.** (1) The RS queue's own next pick,
  **R5 / OOS-RS-4**, is a *LOW* review finding worth 0 flips whose obvious one-line fix is a trap:
  `CounterCountAtLastKnownInformation` reads `ctx.lki_counters`, populated only for
  leave-the-battlefield triggers, and Anim Pakal's is `WheneverYouAttack`, so the swap would produce
  **zero** Gnomes — retired from the queue. (2) **R6 / OOS-OS7-2 is much stronger than its "0 flips"
  filing**: `ContinuousEffect` has no affected-set field and `layers.rs:613` re-evaluates
  `AllCreatures` live, so CR 611.2c is unimplemented and **7 `Complete` defs are live-wrong in
  ordinary play** (a creature entering after a mass -1/-1 wrongly gets it) — re-ranked up to PB-DX5.
  (3) **OOS-DP9-3's yield is 2, not ~7**: `protean_hulk` is `inert` not `partial`, and five of the
  seven carry a second independent blocker a count field does not touch. (4) **OOS-DP6-3 is the
  cheapest item in the inventory and only became so when PB-DP6 landed** —
  `Condition::YouControlNOrMoreWithFilter` is used by 21 defs *and* is queue-time evaluable, so both
  halves of CR 603.4 are available and `garruks_uprising` + `inventors_fair` are authorable today:
  **2 flips, 0 engine lines**. The queue's top five, correctness-first: **PB-DX1** OOS-DP6-1 (the
  intervening-if dropped in the runtime lowering — the only open seed that is live-wrong *and* on a
  `Complete` def *and* unbounded; Aurelia is deck-legal and grants herself infinite combats; HASH
  bump), **PB-DX2** OOS-DP5-7 + OOS-DP7-2 (`ChooseDredge` has no pending-state gate — `card: None`
  is a free card for anyone at any time, and two doc comments claim a pause the code does not
  implement; wire-neutral), **PB-DX3** OOS-DP6-3, **PB-DX4** OOS-DP10-8 (the 97-entry `BASELINE`
  triage; both spot-check class-D defs re-verified against oracle text by MCP — Shambling Ghast's
  printed -1/-1 is *until end of turn* and it has no `Decayed`), **PB-DX5** OOS-OS7-2. **OOS-M11-2's
  exclusion from the primitive queue is confirmed and now recorded as a queue decision**
  (`mana_solver.rs` has zero `mana_pool` references and reads non-layer-resolved `mana_abilities` at
  `:35`; M11-local S3 owns it). Two further cross-links worth carrying: **`OOS-DP10-3` ≡ the legacy
  `OOS-AC9-FILTERMANA`**, the same filter-land gap filed twice nine months apart, and
  **`OOS-DP5-5`'s stated blocker no longer exists** — PB-DP9's abort-and-replay *is* the suspendable
  resolver it was waiting for, so re-scope it rather than copying the dismissal forward. The durable
  lesson, added as an addendum to the 2026-07-19 triage's closing claim that no rider seed had gone
  stale: **"filed too recently to be stale" is a statement about elapsed batches, not elapsed days**
  — ten PB-DP batches ran across the same subsystems while the RS queue was parked.

- 2026-07-27 — **PB-DP10 SHIPPED (`scutemob-158`) and THE PB-DP SUITE IS COMPLETE.** The last row of
  `docs/audits/decision-point-audit.md` §8 was never an instance; it was the **invariant**. DP-INV
  says the engine must either obtain a player's answer through a `Command` or refuse the card at
  deck-build, and SR-33's `effect_choose_gate.rs` enforced that for exactly **three** DSL variants
  while §3.1 counted twenty-one decision sites across 277 `Complete` defs — so the figure grew
  silently with every card authored. PB-DP10 is **test-only** (PROTOCOL 31 / HASH 68 unmoved, `git
  diff` over `crates/engine/src` + `crates/card-types/src` + `crates/card-defs/src` **empty**, 0 def
  edits, 0 completeness flips) and adds two files under `crates/engine/tests/core/`:
  `decision_site_walk.rs`, which classifies **all 22** rows (the audit's 21, with row 4 split) as
  **4 SERVED** (`triggered_targets` 77, `search_library` 73, `scry` 16, `surveil` 8 — the classes
  PB-DP8 and PB-DP9 actually fixed) / **15 AUTO-CHOSEN** / **2 GATED** / **1 NO-DECISION**, each
  carrying the engine site that was *read* to establish it; and `decision_gate.rs`, which freezes
  the **97**-def still-auto union in a name-keyed `BASELINE` and reddens on a new instance. **The
  batch's headline is a gate-integrity finding, and it is the reason this was worth a batch**: every
  serde walk in this codebase — `effect_choose_gate.rs`, `pb_rs1_roster_sweep.rs`, and PB-DP9's own
  `roster` module, the one written *because* a hand-walk had been caught being incomplete — matches
  **object keys only**, and serde serializes a **unit** enum variant as a bare JSON **string**.
  `serde_json::to_value(Effect::Proliferate) == Value::String("Proliferate")`. A verbatim reuse
  would have reported **0** for Proliferate's 25 `Complete` defs *while the suite stayed green* —
  precisely the OOS-DP7-11 "a gate cited as covering something is a claim like any other" failure
  mode, one layer deeper. The canonical walk now matches both shapes, with a `PROSE_FIELDS` denylist
  so a card whose oracle text spells a variant name is not a false positive, and T2/T3 pin both
  mechanisms against the legacy walk. Two "control group" zeros the audit credited to the SR-33 gate
  were held by **nothing** (`AddManaFilterChoice` is a *different serde key* from the
  `AddManaChoice` that gate bars; `TheRingTemptsYou` likewise) and are now machine-checked.
  **All-rows union 267** — the §3.1 "277" re-derived by computation rather than regex, which
  **closes OOS-DP7-7** — with a per-row delta table naming a mechanism for each drift and marking
  three deltas honestly as "unexplained, ±1, within regex noise" rather than inventing a story.
  **Fail-closed was proven end-to-end on a real card def**, not only on a synthetic corpus:
  `Effect::Proliferate` added to `lightning_bolt.rs` reddened **two** tests naming the card, the
  row, the CR and the engine site; restored, green. **The review's two HIGHs are both about the
  gate's own honesty, and both are the durable lesson.** (1) `BASELINE`'s 97 entries were populated
  **mechanically** and the plan's class-B/class-D triage against oracle text was never performed — a
  spot-check found two class-**D** defs already inside the frozen list: **Smuggler's Copter**, whose
  "you **may** draw a card" is authored as an unconditional `Sequence(DrawCards, DiscardCards)` on a
  `Complete` def (the 20th instance of DP-12's class, where the other 19 are `known_wrong`), and
  **Shambling Ghast**, with a *permanent* `-1/-1` counter where the card says "until end of turn",
  an `oracle_text` field reading "enters" against a `WhenDies` trigger, and a `Decayed` keyword the
  printed card does not have. Per plan §5.3 both are **seeded (OOS-DP10-8), not demoted**, and the
  baseline's doc comment and the gate's failure message now say plainly that an entry asserts *only*
  which rows a def hits and nothing about whether the def is otherwise oracle-correct. (2) **The
  gate can only see a decision the DSL encoded.** Every row is a predicate over a variant name, so a
  choice dropped at authoring time — a "you may" written as a bare `Sequence` — leaves no trace and
  passes forever; that class is *strictly worse* than the class the gate records, because a recorded
  auto-choice is at least a legal outcome. Stated in the module doc, in §3.1 and in the §8 row, and
  filed as **OOS-DP10-9** with the instrument that would catch it (an oracle-text-vs-DSL
  cross-check, not a variant walk). All 14 findings applied; both new tests proven non-vacuous by
  execution. Tests 3,910 → **3,928**; seeds **OOS-DP10-1..11**; audit §8 carries a suite-COMPLETE
  banner and §10 an honest **3-of-8** mechanization ledger. **Next: re-rank RS5..RS11** against the
  DP suite's seeds (OOS-DP9-3 was the standing first pick; OOS-DP10-6 is the successor queue's
  measured ranking).

- 2026-07-27 — **PB-DP9 SHIPPED** (`scutemob-157`): **a tutor no longer fetches for you, and scry
  and surveil stop inverting their printed mechanic.** Library search picked the lowest `ObjectId`;
  scry put **every** looked-at card on the bottom, making "keep on top" unreachable; surveil put
  every one in the graveyard, making `Surveil N` exactly `Mill N`. All three now route through a
  blocking CR **608.2d** announcement — `GameState.pending_effect_choice` +
  `GameEvent::EffectChoiceRequired` (disc. 131) → `Command::AnswerEffectChoice` — the engine's
  **first resolution-time decision channel**. **The design the plan inherited was impossible and the
  correction is the batch's headline.** `pb-plan-DP7.md` §1.6 and audit §8 both prescribed "a
  resumable effect-list cursor **on the stack object**"; `resolve_top_of_stack` **pops** the stack
  object (`resolution.rs:39-42`) before a single effect runs, so nothing living there can carry a
  continuation. What shipped is an **abort-and-replay**: clone the state at entry; an effect needing
  an unanswered choice records the *question* and returns; the wrapper **restores the clone
  wholesale** (spell back on the stack, no card moved, no event emitted) and returns exactly one
  event; the answer is banked on `GameState` and the resolution re-runs **from the top**, retracing
  the identical deterministic path. That buys three things a cursor would not: **no continuation
  data structure at all** (`Sequence`/`Conditional`/`ForEach`/`Repeat`/`MayPayThenEffect` and the
  per-player loops need zero machinery — the replay re-executes them, which is *correct* because
  they are pure functions of a restored state); a re-entrancy audit of **3 units, not 20** (15 of
  the 17 production `execute_effect` callers live inside `resolve_top_of_stack` itself, one is CR
  605.4a-gated, one is provably unreachable); and **immunity to PB-DP8's hardest recurring bug
  class** — "a guard that returns early inherits the obligation of the statements it skipped" cannot
  arise when nothing was skipped because nothing happened. **ONE `Command` for all three effects**,
  argued from CR 608.2d being one rule of which 701.22a/701.23a/701.25a are three instances with
  identical timing, actor and validity condition — so one admission-gate entry, one `LegalAction`,
  one `DecisionKind`, one harness action string, correcting OOS-DP8-14's prediction of three.
  **PROTOCOL 30 → 31 / HASH 67 → 68**, all fingerprints gate-computed, both histories append-only,
  44 sentinel files re-pinned via the **symbol** grep. **Roster 69 / 16 / 7, not the audit's 74 / 16
  / 8** — enumerated from `all_cards()` with a *recursive* `Effect`-tree walk (a flat scan
  undercounts; the effects nest). **0 card-def edits, 0 completeness flips.** Three in-scope
  correctness fixes beyond the agency restoration, none in the brief: CR **701.22b** (`Scry 0`
  emitted `Scried { count: 0 }` — the surveil arm had the mirror-image CR 701.25c guard, the scry
  arm did not, so a "whenever you scry" trigger could fire off a scry 0); CR **400.7**
  (scry-to-bottom **renumbered every scried card**, because both `move_object_to_*` helpers mint a
  fresh `ObjectId` unconditionally, and it also consumed `timestamp_counter` — the shuffle seed
  source; replaced with a `Zone::reposition_within` permutation, sweep seeded as OOS-DP9-11); and CR
  **701.23d** (a quantity-only search with exactly one candidate is determined and asks nothing).
  **Two deliberate deviations, both argued in source and pinned in both directions by tests.** (1)
  The scry and surveil **defaults flip to the identity** — search keeps its lowest-`ObjectId`
  default byte-for-byte, but "all to the bottom" / "all to the graveyard" is the systematically
  maximal-harm legal answer and surveil-as-mill can contribute to a CR 704.5b deck-out; the cost was
  paid visibly in 25 unit tests + 1 golden script, every repair CR-justified, none by weakening an
  assertion. (2) The three new `GameState` fields go into `public_state_hash` but **NOT** into
  `loop_detection.rs`'s mandatory-state fingerprint, unlike PB-DP7's and PB-DP8's, because they
  *grow* between successive replays of one resolution and could mask a CR 726 mandatory loop —
  recorded as **obligation (7)** on the `BlockingDecision` doc block, the first evidence PB-DP8's
  six-obligation list generalises (PB-DP9 discharged 6/6 plus the new one).
  `GameEvent::private_to()` now exists (OOS-DP8-6's declaration half) and is honestly labelled a
  **declaration, not an enforcement point** — nothing consumes it until M10. Benches measured
  against `48353a36` in a throwaway worktree: `full_turn_4p` **253 → 229 µs**, no regression from
  the per-resolution `GameState::clone()`. `EffectContext.target_remaps`' `HashMap` audited and
  **clean** (3 inserts, 1 get, nothing iterates — SR-9b safe). Tests 3,878 → **3,905**; 211/211
  golden scripts green, 0 new skips; seeds **OOS-DP9-1..12** filed in audit §8.1 — rank
  **OOS-DP9-3** next (`Effect::SearchLibrary` finds exactly ONE card; CR 701.23 says "one or more";
  ~7 `partial` defs; **zero** new plumbing on this machinery).

- 2026-07-26 — **PB-DP3 SHIPPED** (`scutemob-151`): **a modal spell can no longer be cast without
  announcing its modes** (CR **601.2b/700.2a**). The range / duplicate / `min_modes` / `max_modes`
  checks all lived inside `casting.rs`'s `if !modes_chosen.is_empty()` branch, so supplying
  *nothing* skipped every one of them and both consumers re-derived `vec![0]` — Cryptic Command,
  Austere Command and Incendiary Command (all `Complete`, all `min_modes: 2`) paid full cost and
  resolved **one** mode, silently. The fix **lifts** the checks out of the emptiness gate rather
  than mirroring the Spree guard as the audit prescribed (the Spree guard fires earlier and owns its
  CR 702.172a message, so it was kept intact) — and the lift turned a 3-card fix into a **40-def**
  one: the 3 commands plus the **37** `min_modes: 1` defs that had all been accepting an unannounced
  cast, plus the identical bypass on modal **activated** abilities in `abilities.rs` (audit §4.2
  L214), folded in at zero test cost. Escalate keeps a narrow CR 702.120a exemption — electing to
  pay the additional cost *is* an announcement of the mode **count** — with the derived count now
  bounds-checked (OOS-DP3-1). `resolution.rs`'s `vec![0]` fallback was **retained**: it looks like
  dead code after the fix and is not, because four producers build Spell stack objects without ever
  calling `handle_cast_spell` (cascade, discover, cipher, suspend) — the plan's original list of
  those producers was wrong in *both* directions and the review corrected it. **0 card-def edits**,
  PROTOCOL 27 / HASH 63 unmoved as predicted, 8 fail-before probes (Austere Command's empty-mode
  cast simply *succeeded* pre-fix), review 0 HIGH / 2 MEDIUM / 6 LOW all dispositioned, tests 3,725
  → **3,747**; seeds **OOS-DP3-1..9** filed in audit §8.1 (the last, OOS-DP3-9, found by the closing
  `/review`: `mtg-fuzzer` aborts on a stack overflow at 15 games and long games trip
  `stack_consistency` — pre-existing on `main`, reproduced both there and here, plausibly a shared
  root cause with OOS-M11-3) and audit rows §4.1 L186 (**D→A**), §4.2 L214 (**B→A**), §5 DP-4/DP-20,
  §8, §9 rec 4 (superseded) all updated. **Prior same-day entry:** **PB-DP2 SHIPPED**
  (`scutemob-150`): the mulligan is no longer a content no-op — `handle_take_mulligan` performs a
  real `timestamp_counter`-seeded `Zone::shuffle` (the `LibraryShuffled` event had been a phantom,
  Architecture Invariant 4) and `handle_keep_hand` bottoms with `move_object_to_bottom_of_zone`
  instead of writing to the library top, per **CR 103.5/103.5c** (the suite's "103.4b" cite is stale
  — that rule is the Vanguard starting life total); **OOS-M11-1 CLOSED**, the audit's predicted HASH
  bump **falsified** (PROTOCOL 27 / HASH 63 unmoved), 4 probes, tests 3,721 → 3,725, seeds
  OOS-DP2-1..6 filed. **Prior same-day entry:** **M11-local KICKED OFF (web-first); the playability
  track is now open** (`scutemob-147`). Three commits: (1) `aceba394` roadmap restructure applying
  strategic-review action items 2/3/4 — M11 → **M11-local** (browser + 1 human + 3 bots, no
  networking, M10 dependency removed), M10 → **M10-pre/M10a/M10b**, **M12 downscoped** to a
  continuous track; rewind/pause/manual-adjust UI and the two engine features (Mindslaver turn
  control, Stasis step skipping) carved out of the UI milestone into M13; strategic-review action
  items closed 9/9 with its stale headless-Debian premise corrected. (2) `c6b6b46f`
  `memory/m11-session-plan.md` — 8 sessions from `rules-implementation-planner`; the milestone needs
  **no new `Command`/`GameEvent` variant**, so PROTOCOL 27 / HASH 63 hold throughout. (3) `f2a9647b`
  **Session 1 shipped** — `LocalGame`, the steppable human-input bridge, in `crates/simulator` only;
  `GameDriver::run_game` re-expressed on top of it (parity evidenced by a byte-identical single-seed
  command-trace replay plus a statement-by-statement review — a full fuzzer-baseline diff is **not**
  a usable oracle right now, see OOS-M11-3 below); 10 new tests; **3,683 workspace tests passing, 0
  failing**. A review pass found and fixed four API hazards before HTTP lands on this surface:
  `advance()` is now idempotent while a decision is outstanding (a poll endpoint or browser refresh
  used to silently invalidate the client's `seq`), a human seat now gets the same
  empty-legal-actions auto-pass a bot does (it could otherwise deadlock on an empty decision),
  `submit()` now refuses a command naming another seat, and the journal is opt-in so the fuzzer does
  not pay for it. Three new seeds filed but **not yet ranked into the RS queue**: OOS-M11-1
  (mulligan never actually shuffles — live-wrong vs CR 103.5), OOS-M11-2 (mana solver ignores the
  pool, reads non-layer-resolved abilities), OOS-M11-3 (fuzzer nondeterministic in very long games;
  pre-existing). **Same day, parallel track: PB-RS4 SHIPPED** (`scutemob-146` merge `9419d0e9`) —
  face-aware residuals; **OOS-RS-3 closed, and with it OOS-OS4-2 fully closed**;
  `deregister_face_statics` extended to all 10 registered families + parity gate; a 4th
  same-root-cause Saga-sweep deviation found in planning and fixed; 0 flips, 2 integrity repairs, 17
  probes, PROTOCOL 27 / HASH 63 unchanged; seeds OOS-RS4-1/2/4 filed; **next: PB-RS5**. *(Entries
  older than the five above have rotated out of this window — the 2026-07-19 PB-OS-queue-complete
  entry and everything before it live verbatim in `memory/archive/claude-md-changelog-2026-07.md`.)*

## Last shipped (as of 2026-07-18)

- **Last shipped**: **PB-EF1 — `exclude_self` enforcement sweep** (`scutemob-99`, merge `6202ab81`) —
  first batch off the EF queue. Five executor sites that matched a `TargetFilter` without a threaded
  source ObjectId silently ignored `exclude_self` (`PermanentCount` amount resolver, sacrifice
  cost + `SacrificePermanents` effect paths via `eligible_sacrifice_targets`, `UntapAll`,
  `YouControlNOrMoreWithFilter`, `SacrificeOther`); all five now honor it, each pinned by a decoy
  test that fails on exactly that field. One wire change proved necessary after all:
  the activated-cost path lowers to a lossy `SacrificeFilter`, so "sacrifice ANOTHER creature"
  (Izoni, Yawgmoth) needed `ActivationCost.sacrifice_exclude_self` — **HASH 43→44, PROTOCOL 5→6**
  (Nantuko-Husk-style "sacrifice a creature" bars a default-exclude). 6 cards flipped/authored
  Complete (éomer, Izoni, Korvold, Yawgmoth, Commissar Severina Raine, Copperhorn Scout);
  disciple_of_freyalise stayed partial with a real second blocker filed as **EF-EF1-A**
  (`PowerOfSacrificedCreature` not populated in the `MayPayThenEffect` optional-cost path).
  Closed EF-W-PB2-1, EF-W-EMPTY-1, EF-W-MISS-2, marker EF-4/EF-5, OOS-TS-2. Coverage
  59.8% → **60.1%** (1,071/1,782). Same sitting: **swan_song demoted** Complete → known_wrong
  (`scutemob-100`, EF-W-MISS-1 — Bird minted for the wrong player; PB-EF2 fixes it).
  **EF-13 RESOLVED — Option A shipped** (`scutemob-101`, merge `0096ca65`): the no-behaviour
  `Partial` class enumerated from the compiled registry was **101** defs (drifted from the filed
  105; 0 `KnownWrong` members), all flipped to `inert` with notes preserved;
  `card_registry_gate` now forbids `Partial`/`KnownWrong` on a def where
  `registers_no_behavior` is true, with its own non-vacuity canary. Headline clean coverage
  unchanged (1,070 = 60.0%); buckets honest: todo 655→554, empty 57→158. Prior: **Marker sweep** (`scutemob-88`) — the AC8/AC9 follow-up both workers asked
  for. All **742** non-`Complete` markers audited against the current engine (29 agent batches,
  full coverage, 0 missing). **42% were wrong**: 208 `stale-blocker-shipped` (note cites a
  capability that now exists) + 100 `wrong-or-vague-note`; only 434 still valid. Applied: **13
  upgraded to Complete** (coverage **57.6% → 58.3%**), **54 `partial` → `known_wrong`** (the
  marker understated the card — it ships wrong game state, it does not merely omit a clause),
  266 notes rewritten to the real blocker. **116 ready cards emitted as a worklist across 16
  blocker groups** (`memory/card-authoring/marker-sweep-worklist-2026-07-16.md`) — not authored
  here. Root cause found and fixed: `card_registry_gate`'s inert check tested
  `abilities.is_empty()`, which is **not** the same as "registers no behaviour" — a cost-reduction
  static lives in `spell_cost_modifiers` — so the gate itself minted the false
  `inert("no abilities implemented")` markers it then demanded. Now `registers_no_behavior` +
  `inert_gate_is_not_vacuous`. **Open follow-up (EF-13): 105 defs are marked `partial` but
  register no behaviour at all — they are `Inert` by the taxonomy.** Not a safety issue (both are
  non-`Complete`, so `validate_deck` rejects them alike), but it misreports the campaign's
  todo/empty buckets; deferred because it moves headline numbers and is inherited drift.
  **Count that class from `all_cards()`, never from source text** — the regex
  `abilities:\s*vec!\[\s*\]` also matches inside `mana_abilities: vec![]`, the same trap
  CLAUDE.md already records against the authoring report; it fired twice more during this task.
  Method that made it work (per `feedback_verify_full_chain`):
  **variant existence is not proof a blocker is stale** — a `TriggerCondition` needs a builder arm
  in `enrich_spec_from_def` *and* a dispatch in `check_triggers`, and several `TargetFilter` fields
  are silently ignored by `matches_filter`. Calibration case `megrim.rs`: note false on every
  clause, yet the card is still not Complete (models "deals 2 damage" as `LoseLife`, CR 119.3) —
  "note is false → upgrade" would have shipped a broken card. **12 engine findings filed for
  SR-33+, not fixed inline**: `memory/card-authoring/marker-sweep-engine-findings-2026-07-16.md`.
  **EF-1 is HIGH and needs a coordinator decision**: 88 dual/tri lands are `Complete` but model
  "Add {G} or {U}" as `Effect::Choose`, which is a stub (`effects/mod.rs:3190` always takes
  `choices.first()`) *and* is unknown to `try_as_tap_mana_ability` — so they register **zero mana
  abilities** (CR 605.1a) and only ever make their first colour. Proven empirically; the whole
  original dual + shockland + check/fast/temple cycles. Fix shape exists in-repo (`tainted_field`:
  two abilities, one per colour). 3275 tests. Prior: **PB-AC9** (`scutemob-52`, merge `a4750cdb`) — **closes the AC chain**. Recon: 3/5 briefed primitives already existed (`Effect::RollDice` d20+results CR 706, `ReplacementModification::DoubleTokens` CR 614.1, `Effect::AddManaFilterChoice`); SearchLibrary multi-name 0-yield → OOS seed. Built: `Effect::WheelHand` + `Effect::SetNoMaximumHandSize` (unbriefed co-blocker — flag was recomputed each cleanup, "rest of the game" inexpressible). **Token doubling rewired 2→13/13 creation sites** (Squad, Offspring, Myriad, Embalm, Eternalize, Encore, Living Weapon, Gift keyed to recipient, Investigate, Amass — doublers were silently failing, invisible to any marker/roster). Review 0 HIGH / 1 MEDIUM fixed (Amass bypassed `apply_counter_replacement` — CR 701.47a; fix proven non-vacuous). Backfill: 11 clean incl. token doublers (Parallel Lives, Anointed Procession, Doubling Season), wheels (Echo of Eons, Winds of Change), d20 Ancient dragons; 1 backfill HIGH (Reforge the Soul stale Miracle marker — 2nd consecutive stale-marker HIGH; AC8+AC9 workers both recommend a campaign-wide marker sweep next). New gotcha logged: `timestamp_counter` IS the object-id counter — rewinding it aliases ObjectIds (`3d7e216c`). Prior: PB-AC8..AC1 (`scutemob-51..43`). Next per campaign plan: **W-PB2** (author ~55 cards unblocked by AC4..AC6), W-EMPTY/W-MISS derisking batches. Registry-gate debt **CLOSED** by SR-2 (`scutemob-54`); follow-up `scutemob-64` (SR-12).

---

## Last Updated narrative (as of 2026-07-18)

- **Last Updated**: 2026-07-18 (**PB-OS1 collected, `scutemob-116` merge `db49a0b2`** —
  `recompute_object_controller` wired into `expire_end_of_turn_effects` +
  `expire_until_next_turn_effects`; canary de-vacuoused (failed pre-fix, passes post-fix);
  stacked-control + until-next-turn timing tests; roster from `all_cards()` = **2** affected
  (`sarkhan_vol`, `zealous_conscripts`) — reviewer overturned the plan's karrthus claim
  (`Indefinite` is CR-correct permanent control, 611.2a, no seed filed); 0 golden scripts
  encoded the old behavior; no PROTOCOL/HASH change; OOS-EF9-1 EOT-half closed,
  WhileSourceOnBattlefield half deferred. Same day: doc audit landed
  (`memory/doc-audit-2026-07-18.md` F1-F8 + addendum) — **DOC remediation running before
  PB-OS2** (live tasks: 118/119/121/124/125/126; 117/120/122/123 cancelled-superseded).
  Earlier: **OOS seed retriage collected, `scutemob-115` merge `7d577171` —
  THE PB-OS QUEUE IS ACTIVE.** All 65 open OOS/EF-EF seeds enumerated from source docs (not the
  headline list) and chain-verified against the post-EF-queue engine: 23 resolved/stale (10 newly
  discovered silently closed — the EF/EWC/EAT/AC9 waves shipped primitives that resolved older
  seeds nobody cross-referenced; 3 stale finding banners added), 16 candidates ranked into
  **PB-OS1..OS11** (`memory/primitives/oos-retriage-plan-2026-07-18.md`), 7 deferred, 24 dormant.
  **PB-OS1 is fully specified and dispatchable**: gain-control reversion (OOS-EF9-1) —
  `expire_end_of_turn_effects`/`expire_until_next_turn_effects` drop `SetController` effects but
  never call the already-built `recompute_object_controller`, so sarkhan_vol /
  zealous_conscripts / karrthus keep stolen creatures forever while shipping `Complete`
  (invariant #9); fix wires the idle helper, no wire bump, must de-vacuous
  `test_gain_control_until_eot_expires`. Queue total ~19-22 discounted flips; correctness group
  PB-OS1..OS3 first. Doc-only task, zero code changes. Earlier: **PB-EF12 collected, `scutemob-114` merge `833e54ad` — THE EF
  QUEUE IS COMPLETE.** `chosen_color` rides `Command::TapForMana` (coordinator decision in
  `memory/decisions.md`, CR 605.3b, extending the SR-33 precedent; no Colorless default —
  missing/illegal choice is rejected). **17 defs restored/flipped Complete** (elven_chorus + 16
  any-color rocks/lands — the SR-37-gated `AddManaAnyColor` family is genuinely correct now);
  7 held back on real blockers; deathrite reverted after the gate itself caught it; the
  land-colour gate refined to served-vs-unserved; simulator emits only engine-legal colours.
  EF-W-PB2-3 closed. Coverage 61.2% → **62.1%** (1,117/1,798); 3476 tests; **PROTOCOL 17→18,
  HASH 55**; /review 0 findings. **Queue totals for the day** (scutemob-99..114): 12 PBs +
  EF-13 reclassification + swan_song demote + Cargo.lock chore; coverage 59.8% → 62.1%
  (+52 clean defs, corpus 1,781 → 1,798); tests 3330 → 3476; PROTOCOL 2→18, HASH 43→55;
  all 20 EF findings closed; 11 new OOS seeds filed (see Active Milestone line). Next: no open
  queue — candidates are the OOS seed backlog, W-blocked cohorts, or M10 per the strategic
  review. Earlier: **PB-EF11 collected, `scutemob-112` merge `e991b237`** — both
  singletons: `WheelDraw::GreatestDiscarded` (Windfall Complete; everyone draws the max
  discarded, decoy pins not-per-player) and `TargetSpellWithSingleTarget` + retarget
  (Misdirection restored to Complete after its honest scutemob-97 demotion). **PROTOCOL 15→17,
  HASH 53→55** (one bump per commit). 3466 tests; coverage **61.2%** (1,100/1,798); reviews
  0H/0M/2L fixed. Same sitting: **Cargo.lock is now TRACKED** (`scutemob-113`, merge
  `e1c30acb`) — scutemob-112 found main's tip didn't build in a fresh env (untracked lock →
  fresh resolve picked stricter `equivalent 1.0.2`); the lock pins deps like
  rust-toolchain.toml pins the compiler (SR-11), verified `--locked` green; EF11's COMMIT 1
  also carries the 9-site source-level fix. In flight: **PB-EF12** (`scutemob-114`, granted
  any-color mana choice — **the final EF-queue batch**; coordinator decision recorded: color
  choice rides `Command::TapForMana` per the SR-33 CR 605.3b precedent, no Colorless default).
  Earlier: **PB-EF10 collected, `scutemob-111` merge `3710ad9c`** — all
  three EF-W-MISS-7 sub-gaps via one `SacrificedCreatureLki` struct:
  `EffectAmount::ToughnessOfSacrificedCreature` (layer-resolved LKI captured before
  `move_object_to_zone`, anthem + toughness-not-power decoys), `TargetFilter.max_cmc_amount`
  runtime cap on SearchLibrary (avoids a 99-def edit), `Condition::SacrificeFired` ("if you
  do", CR 608.2b/c/h/i). momentous_fall/eldritch_evolution/victimize Complete + 2 bonus flips
  (miren_the_moaning_well, diamond_valley); birthing_ritual honestly inert → **OOS-EF10-1**
  (top-N dig inexpressible). EF-EF1-A untouched. Coverage **61.1%** (1,098/1,796); 3453 tests;
  **PROTOCOL 14→15, HASH 52→53**; review 0H/0M/2L fixed. In flight: **PB-EF11**
  (`scutemob-112`, Windfall + Misdirection singletons); then EF12 closes the queue.
  Earlier: **PB-EF9 collected, `scutemob-110` merge `abb92654`** —
  `EffectDuration::WhileYouControlSource(PlayerId)` (CR 611.2b/c): "you" captured at creation,
  one-shot `expire_while_you_control_source_effects` **never resumes** (live re-eval would
  wrongly revive on regaining control); the planner found the engine had **no control-reversion
  at all** — this PB built it (`recompute_object_controller`), and the latent
  never-reverts gap on other durations is filed as **OOS-EF9-1**. olivia_voldaren +
  dragonlord_silumgar Complete; roil_elemental/kellogg honestly partial. Tests incl. phase-out
  (CR 702.26e), source-dies-via-SBA, and a decoy proving `WhileSourceOnBattlefield` stays
  active in the same steal scenario. Coverage **61.0%** (1,093/1,792); 3437 tests; **PROTOCOL
  13→14, HASH 51→52**; review 0H/1M(fixed)/2L. In flight: **PB-EF10** (`scutemob-111`,
  sacrifice-driven amounts / runtime max_cmc / if-you-do). Earlier: **PB-EF8 collected, `scutemob-109` merge `4fa6b6f2`** —
  `Cost::ExileSelfFromHand` + hand-zone activation through the mana-ability lowering path;
  **2 flips Complete** (simian_spirit_guide, elvish_spirit_guide); EF-W-PB2-8 closed. Coverage
  **60.9%** (1,091/1,792); **PROTOCOL 12→13, HASH 50→51** (EF7 had taken 11→12/49→50); /review
  0H/0M/1L fixed. In flight: **PB-EF9** (`scutemob-110`, WhileYouControlSource duration).
  Earlier: **PB-EF7 collected, `scutemob-108` merge `104ef5ad`** — modal
  `AbilityDefinition::Activated`: `modes: Option<ModeSelection>` + `modes_chosen` on
  `Command::ActivateAbility`, validated in `handle_activate_ability` (CR 700.2a/d) with per-mode
  target splitting; corpus sweep sized the cohort at **3** — goblin_cratermaker + cankerbloom
  Complete, Umezawa's Jitte stays known_wrong on a real second blocker (combat-damage-to-player
  trigger variant). Coverage 60.7% → **60.8%** (1,089/1,792); /review 4/4, 1 LOW fixed (vacuous
  snapshot assertions dropped). In flight: **PB-EF8** (`scutemob-109`, ExileSelfFromHand).
  Earlier: **PB-EF6 collected, `scutemob-107` merge `359c824d`** —
  `TargetRequirement::TargetOpponent` with caster-threaded opponent-only validation
  (CR 102.2/102.3/601.2c) and no self-fallback in either auto-picker (CR 603.3d). 3 flips
  Complete (shaman_of_the_pack, raiders_wake, vengeful_bloodwitch — a roster recall) + a
  **latent self-target fix on shipped-Complete fell_specter**; 4 stay non-Complete on real
  blockers; **OOS-EF6-1** filed (WhenTappedForMana trigger dispatch gap — blocks
  forbidden_orchard). Coverage 60.5% → **60.7%** (1,087/1,792); **PROTOCOL 10→11,
  HASH 48→49**; /review 0 issues. In flight: **PB-EF7** (`scutemob-108`, modal activated
  abilities). Earlier: **PB-EF5 collected, `scutemob-106` merge `111c4513`** —
  `Effect::TransformSelf` shipped through the existing DFC machinery (CR 701.28/712.8);
  **honest yield well under the plan's ~7–9**: 2 new Complete (docent_of_perfection,
  bloodline_keeper) + delver_of_secrets **integrity-demoted** (shipped Complete but never
  transformed) + 2 partial (thaumatic_compass, growing_rites); 8 of the "11 body-only DFCs"
  were double-blocked by distinct out-of-scope primitives. **Battle + Sephiroth split out with
  justification** (CR 310 is a full card-type subsystem; bare enum would be legal-but-wrong) →
  seeds **OOS-EF5-1..4** filed. /review caught a legal-but-wrong def (thaumatic_compass
  fabricated a Spires ability — fixed+demoted+pinned) and 2 reviewer HIGH/MED were overturned
  against cards.sqlite. Coverage **60.5%** (1,084/1,792); 3396 tests; **PROTOCOL 9→10,
  HASH 47→48**. In flight: **PB-EF6** (`scutemob-107`, TargetOpponent). Earlier: **PB-EF4 collected, `scutemob-105` merge `26421364`** —
  `EffectFilter::TriggeringCreature` + `Effect::DealDamage.source` override; **7 cards
  Complete** (dragon_tempest, scourge_of_valkas, ogre_battledriver, Atarka, Fervent Charge,
  Goblin Piledriver, Muxus — beat the ~4–5 estimate); shared_animosity inert (**OOS-EF4-1**
  filed). EF-W-PB2-6/MISS-5 + PB2-7 closed. Coverage 60.3% → **60.5%** (1,081/1,786); 3383
  tests; **PROTOCOL 8→9, HASH 46→47**; /review 0 issues. In flight: **PB-EF5**
  (`scutemob-106`, self-transform + Battle — highest yield). Earlier: **PB-EF3b collected, `scutemob-104` merge `6439d0ce`** —
  granted trigger-keywords (Melee/Battle Cry/Annihilator) now fire: synthesis moved to
  post-layer characteristics via `calculate_characteristics` + shared helper;
  Melee/Myriad/Provoke tags raw→resolved. Adriana Complete; Skyhunter Strike Force stayed
  partial (**OOS-EF3b-1** filed). EF-W-MISS-3 closed — **the correctness group
  (PB-EF1/EF2/EF3/EF3b) is COMPLETE; all six correctness findings cleared.** Coverage **60.3%**
  (1,076/1,785); 3372 tests; no schema bump. In flight: **PB-EF4** (`scutemob-105`,
  TriggeringCreature subject/source — first capability batch). Earlier: **PB-EF3 collected, `scutemob-103` merge `cae6710a`** — both
  halves in one PB. (A) EF-W-MISS-10: all 30 attack/triggered enrich blocks now forward DSL
  targets (were hardcoded `vec![]`); `flush_pending_triggers` fallback is kind-guarded (Normal →
  runtime `triggered_abilities` authoritative; CardDefETB → def raw-index), 4 mis-tagged sites
  reclassified, latent Throat Slitter path fixed. (B) EF-W-MISS-4:
  `EffectTarget::AttackTarget` (CR 506.4c fizzle) + `PlayerTarget::DefendingPlayer` (CR 508.4),
  captured per-attacker at AttackersDeclared and threaded StackObject→EffectContext — correct in
  4-player, decoys prove non-defending opponents unhit. 3 cards Complete (Ojutai Soul of Winter,
  Hellrider, Raid Bombardment); 5 blocked with real distinct blockers (Silumgar → **OOS-EF3-1**
  filed); Dragonlord Ojutai was mis-listed (combat-damage trigger, not attack). Coverage
  60.1% → **60.2%** (1,075/1,785); 3364 tests; **PROTOCOL 7→8, HASH 45→46**. In flight:
  **PB-EF3b** (`scutemob-104`, granted keyword-triggers fire). Earlier: **PB-EF2 collected, `scutemob-102` merge `3a489f59`** —
  `TokenSpec.recipient: PlayerTarget` (default `Controller`; all 201 existing construction sites
  unchanged) + `PlayerTarget::ControllerOfCounteredSpell`/`ControllerOfTriggeringObject`; token
  doubling applies per-recipient (CR 614.1, forward+reverse decoys). swan_song
  known_wrong→**Complete** (review HIGH also fixed its over-broad `TargetSpell` →
  `TargetSpellWithFilter`); An Offer You Can't Refuse authored Complete. Script
  `tokens/001` **un-retired** (approved again); pre-existing wrong-owner assertion in
  `stack/045` fixed. EF-W-MISS-1 closed. Coverage 60.0% → **60.1%** (1,072/1,783); 3355 tests;
  **PROTOCOL 6→7, HASH 44→45**. In flight: **PB-EF3** (`scutemob-103`, attack-trigger target
  fidelity + defending-player target). Earlier: **EF-13 Option A collected, `scutemob-101` merge `0096ca65`** —
  101 no-behaviour `partial` defs → `inert`, registry gate + canary added; 3346 tests, coverage
  60.0% unchanged, buckets todo 554 / empty 158. Earlier same day: **PB-EF1 collected,
  `scutemob-99` merge `6202ab81`** — see "Last shipped" above; HASH 44 / PROTOCOL 6. Also:
  swan_song demote `scutemob-100` merge `615c4319`. In flight: **PB-EF2** (`scutemob-102`,
  CreateToken recipient — un-demotes swan_song). Next per `ef-batch-plan-2026-07-17.md`:
  PB-EF3 → PB-EF3b → capability batches EF4..EF12.)
  Earlier: 2026-07-17 (**EF triage collected, `scutemob-98` merge `ef82ae45`** — all 20
  post-wave findings (EF-W-PB2-1..8, EF-W-EMPTY-1, EF-W-MISS-1..10, EF-13) deduped and classified;
  **`memory/primitives/ef-batch-plan-2026-07-17.md` is the active engine-primitive queue**:
  correctness-first PB-EF1..EF12 with discounted yields, first dispatch **PB-EF1** + the
  swan_song demote; EF-13 options A/B/C await user decision (plan §3). Campaign plan §0
  repointed. Earlier: **W-MISS collected, `scutemob-97` merge `9cec7673`** — the ~115
  estimate re-derived to **35/194 authorable** (157 blocked with per-card blockers, 2
  false-missing); **33 authored Complete** in 3 reviewed batches, 2 honest mid-wave demotions
  (Ojutai: targeted attack-trigger drops its target; Misdirection: single-target restriction
  inexpressible — EF-W-MISS-9/10). Coverage **59.0% → 59.8%** (1,065/1,781; corpus grew +33
  files). **10 engine findings filed** (EF-W-MISS-1..10, incl. latent `swan_song.rs`
  token-recipient bug) in `memory/card-authoring/w-miss-roster-2026-07-17.md`. 3330 tests. Earlier:
  **W-EMPTY collected, `scutemob-96` merge `a9152c83`** — the plan's
  "~110 authorable empty defs" was stale: after the marker sweep + W-PB2, only 60 inert defs
  remained, **3 authorable** (57 genuinely blocked, truthfully marked). `turn` +
  `sea_gate_restoration` Complete (+2, coverage **59.0%**); `disciple_of_freyalise` stayed partial
  (EF-W-EMPTY-1: exclude_self gap). Wave closed in one batch; 3330 tests. Earlier:
  **W-PB2 collected, `scutemob-95` merge `7c8cdeff`** — 57 cards
  from the marker-sweep worklist authored in 5 reviewed batches: **47 Complete** (coverage 56.2% →
  **58.9%**, new high), 10 truthfully marked with their real blocker; 8 engine findings filed as
  EF-W-PB2-1..8 in `memory/card-authoring/w-pb2-roster-2026-07-17.md`; no gated stub effects used;
  3330 tests. Next per campaign plan §3: W-EMPTY (~110 empty-placeholder cards) / W-MISS (~115
  missing-file cards), or triage EF-W-PB2 findings first. Earlier: **SR-38 collected, `scutemob-94` merge `ac65216a`** — simulator
  `StubProvider` now gates `TapForMana`/`ActivateAbility` suggestions on `life_cost` vs
  `life_total` (CR 119.4b short-circuit), mirroring the engine's own checks — a bot can no longer
  suggest an activation the engine rejects; SG-2's non-Controller refusal pinned by test; SG-3's
  scaled-clause exclusion narrowed to amounts (colours compared on both sides). 3330 tests. This
  clears the SR-33..38 chain that the marker sweep opened — **no open SR tasks**. Earlier:
  **SR-37 collected, `scutemob-93` merge `df49eb61`** — gate hygiene:
  `ManaAbility.activation_condition` added and checked in `handle_tap_for_mana` (CR 602.5b —
  enrich's `..` was silently dropping it; Tainted Field's coloured arms now require a Swamp);
  the `AddManaAnyColor` family (`/Restricted//OfAnyColorAmount`) gated out of Complete — all
  three add `ManaColor::Colorless` — with **18 Complete defs demoted** to known_wrong; the land
  gate now parses "one mana of any color" as all five and reports the invented `{C}` instead of
  skipping. Coverage 57.3% → **56.2%** (983/1748, honest). HASH 42→43, PROTOCOL 4→5. 3326 tests.
  Earlier: **SR-36 collected, `scutemob-92` merge `264f0e9e`** — SF-8 + SF-9, both HIGH,
  both fixed; see the bullet above. The headline is the roster, not the fix: 11 `Complete`
  fetchlands were fetching for free. Filed **SR-38** (`scutemob-94`): SG-1 simulator
  `LegalActionProvider` ignores `life_cost` — unchanged code whose meaning SR-36 changed; bots can
  pick activations the engine now rejects — plus SG-2/SG-3 hardening
  (`memory/card-authoring/sr36-engine-findings-2026-07-17.md`). 3319 tests. Earlier: **SR-35 collected, `scutemob-91`** — the card corpus is
  format-checked for the *first time*: `cargo fmt --all -- --check` exits 0 having checked **zero** of
  the 1,748 defs, and 321 were misformatted. The brief's fix — "run rustfmt over the defs" — would have
  produced a gate **vacuous for 79% of the corpus**: a long `oracle_text` makes rustfmt fall back to
  verbatim for the enclosing expression and leave the whole file untouched at exit 0, canary-measured at
  **1,380/1,748 defs inert** under *direct* rustfmt. `format_strings=true` → 0 inert;
  `error_on_line_overflow=true` kills the residual unbreakable-line case; both proven load-bearing and
  each pinned by its own canary. Reformat proven non-semantic (full `Debug` of `all_cards()`
  byte-identical; reviewer independently re-proved it by parsing 8,082 string literals). Suite 3305.
  See the SR-35 bullet above. Earlier: SR-34 collected, `scutemob-90` merge `ce6f30b0` — composite-cost
  mana abilities (CR 605.1a by what an ability *does*, not what it costs): `ManaAbility` gained
  `mana_cost`/`life_cost`; `mana_ability_lowering` widened from bare `Cost::Tap` to any
  `TapForMana`-payable cost; `handle_tap_for_mana` now checks legality (CR 118.3/119.4, 119.4b
  short-circuit) and collects payment. 27 affected Complete defs probed by *activation* — 7 of 27
  source-traced predictions falsified (incl. Magnifying Glass contradicting its own oracle); 10
  certified with regression tests, 17 honest demotions, +3 horizon lands restored; coverage
  58.1% → **57.1%**. `PROTOCOL_VERSION` 2→3, `HASH_SCHEMA_VERSION` 40→41, history rows appended.
  Filed **SR-36** (`scutemob-92`: SF-8 Gaea's Cradle taps for 1 regardless of board + SF-9
  `Cost::PayLife` silently unpaid on non-mana abilities — both HIGH, live-probed) and **SR-37**
  (`scutemob-93`: SF-10..12 gate hygiene). Findings: `memory/card-authoring/sr34-engine-findings-2026-07-17.md`.
  3300 tests. Earlier: SR-33 collected, `scutemob-89` merge `953cc5a6` — 88 `Effect::Choose`
  dual/tri lands rewritten to one-activated-ability-per-colour (tainted_field pattern; decision in
  `memory/decisions.md`: CR 605.3b makes a general choice Command pointless for stackless mana
  abilities — `TapForMana{ability_index}` IS the choice channel). The new broad gate
  `every_complete_land_registers_each_printed_tap_mana_color` caught **14 more** dead lands (9
  Triomes + 3 surveil lands asserting unimplemented CR 305.6 intrinsic abilities; 2 Hierarchs) —
  fixed in-task, 102 defs total. **`Effect::Choose`/`MayPayOrElse`/`AddManaChoice` are now gated
  out of Complete** (`tests/core/effect_choose_gate.rs`, walks the serde tree): all three are stubs
  — Choose executes `choices.first()`, MayPayOrElse always declines, AddManaChoice adds one
  colorless and ignores count. 7 demotions (path_to_exile, rhystic_study, cankerbloom, Fiery
  Islet/Nurturing Peatland/Silent Clearing, Glistening Sphere); coverage 58.3% → **57.9%**, honest.
  Findings SF-1..SF-7 in `memory/card-authoring/sr33-engine-findings-2026-07-17.md`; filed
  **SR-34** (composite-cost mana abilities never registered — Signets/horizon/filter lands;
  un-demotes the 3 horizon lands) and **SR-35** (`cargo fmt --check` covers ZERO card defs —
  include!/`#[path]` invisible to rustfmt; add explicit CI rustfmt over defs). 3284 tests.
  Earlier: 2026-07-16 (scutemob-88 marker sweep collected — see "Last shipped" above.
  Critical finding filed as **SR-33 (`scutemob-89`)**: 88 dual/tri lands are Complete-but-broken —
  `Effect::Choose` is a stub that always executes `choices.first()` (effects/mod.rs:3190) and
  `try_as_tap_mana_ability` doesn't handle `Choose`, so Tropical Island et al. register **zero**
  mana abilities (CR 605.1a). Also on that task: `path_to_exile`'s deviation-scan ALLOWLIST
  justification is false (`MayPayOrElse` always declines) and `rhystic_study` is Complete while
  its draw always fires. **EF-13 open, needs user call**: 105 `partial` defs register no behaviour
  and are `Inert` by taxonomy — moves headline numbers. Earlier: 2026-07-10 (SR-9c — the golden-script corpus is triaged (94→**210 approved**, **61
  retired** with recorded reasons, **0 pending**) and can no longer skip silently. This closes SR-9. The
  corpus's green was fiction: `run_all_scripts` dropped 175 `pending_review` scripts without a count, six
  scripts never deserialized (`review_status: draft`; `disputes[]` missing `raised_by`) and had been
  invisible since written, and the replay checker passed **244** unimplemented-path and **583**
  `zones.stack: is_empty` assertions **vacuously** (the stack path was checked against a hardcoded empty
  list). All closed by `tests/scripts/run_all_scripts.rs`, which partitions `approved + retired ==
  discovered` and gates pending/undeserializable/vacuous/reason-less scripts, plus a hardened
  `script_replay.rs` where an unknown assertion path is a mismatch, not a pass. New `ReviewStatus::Retired`
  + required `retirement_reason`. Only **one** approved failure was *fixed* rather than retired: `stack/050`
  now asserts `zones.stack.count == 1` (Solemn Simulacrum's dies trigger belongs on the stack, CR 603.3);
  `stack/170` and `cc31` were retired rather than edited to match a possibly-wrong engine. The 61
  retirements each name the one missing card/primitive/harness-Command that would un-retire it — a ready
  worklist for the authoring campaign. Ninth consecutive SR task whose sharpest finding was a hole in a
  *checker*, not engine code — and `/review` then found a tenth: `every_approved_script_asserts_something`
  counted `assert_state` checkpoints, so an empty `assertions: {}` map would have passed vacuously; fixed to
  count assertion entries. 3185 tests. Earlier same day: SR-9b — the JSON-script regime and the hand-written `Command` regime
  now cross-validate. Four divergences, all the harness's, as gotcha SR-9(b) predicted. The load-bearing
  one: **`build_initial_state` was not deterministic** — `RandomState` seeds each `HashMap` instance
  separately, `ObjectId`s are handed out in insertion order, so two deserializations of the same JSON in
  the same process produced different states (40 builds → 2 distinct hashes). Nothing that hashes a
  harness-built state could have worked, which is why this had to land before anything else does.
  Two lessons worth more than the fixes. **Two mutual rejections are equivalent, and worthless**:
  `equivalence_equip` was green because *both* regimes rejected the equip — Grizzly Bears has no
  `CardDefinition`, so `enrich_spec_from_def` returned a bare spec and it was not a creature. The
  non-vacuity test found it; the equivalence test never could. **And a scenario proves nothing about a
  bug it cannot express**: of six adversarial attacks (each asserted to have changed the file first),
  `play_land` silently falling back to the battlefield is caught by *only* the proptest, because it needs
  a two-step sequence; and `equivalence_equip` survives reverting the determinism fix, because only one
  player has permanents in it. Also: `proptest` writes `tests/proptest-regressions/` on first failure and
  SR-9a's group gate read it as a stray group, so one red test became two — live since before SR-9a,
  fixed here. **`/review` then found two perturbations that survived the new gate** — the eighth
  consecutive SR task whose review findings were holes in the gate, not bugs in the code, and both the
  named shape: the determinism check was pointed only at the battlefield map (the scenarios have
  one-owner hands and no graveyards), and `card_names` never read the commander block. A third: the file
  *documented* that the harness's `resolve_targets` drops unresolvable targets while a direct test
  aborts, and that no scenario exercised the difference — writing that scenario made it divergence #4,
  because `filter_map` turned a `cast_spell` at an absent permanent into a targeted spell cast with no
  target (CR 601.2c). **A documented hazard that nothing executes is a hazard, not a note.** 3178 tests.
  Earlier same day: SR-9a — 297 integration-test binaries → 9 targets; warm test-build
  34.2s → 11.1s, `target/` 19 GB → 2.2 GB, test count unmoved (3162 → 3167, the +5 being the new
  gate's own). The gate, `tests/no_stray_test_binaries.rs`, exists because a dropped `mod` line
  converts a test file into a text file and the suite goes green with less coverage than it had
  yesterday — shown, not asserted, across eight attacks. **Seventh consecutive SR task whose review
  findings were holes in the gate, not bugs in the code**: the declaration check was textual, so
  `#[cfg(…)] mod foo;`, `#[path]`, and a nested subdir all satisfied it while deleting coverage.
  Fixed by shrinking the grammar — a group `main.rs` may hold nothing but `//!` docs and bare
  `mod x;`. The demonstration had a hole too: the first attack deleted a `mod` line for a module
  that was in a different group, so `sed` matched nothing and the gate "passed" an attack that
  never happened. Earlier same day: SR-8 — protocol versioning: strict lockstep + a fingerprint that
  makes the version number machine-checked rather than remembered. Two under-inclusion holes were
  found by the gate's own denominator guards while they were being written (a `pub type` alias on
  the wire; a rustfmt-wrapped `#[derive]` that silently dropped a type's serde config out of the
  digest), and `/review` found a third — `ReplayLog` is a wire frame in its own right and was not
  a fingerprint root. Sixth consecutive SR task whose review findings were holes in the *gate*,
  not bugs in the code. 3162 tests. Earlier same day: SR-7 — `PendingTrigger` → `TriggerData` cutover finished: 13
  always-`None`, never-read per-keyword fields deleted (29 fields → 16), 32 hand-rolled
  literals collapsed onto `blank()` (−850 lines in `rules/`), `HASH_SCHEMA_VERSION` 36 → 37
  and 28 sentinel tests bumped; zero behavior change. New `tests/pending_trigger_shape.rs`
  stops the migration un-finishing. Follow-up **`scutemob-68` (SR-16) — DONE 2026-07-10**: those
  `kind`/`data`/`embedded_effect` `#[serde(skip)]` fields are now serialized (option (a);
  `PendingTriggerKind` gained the derive), so a round-tripped keyword trigger keeps its identity and
  payload instead of coercing to anonymous `Normal`. `HASH_SCHEMA_VERSION` 38 → 39 (serde shape
  change; hash stream unchanged, so states hash identically); no `PROTOCOL_VERSION` bump
  (`PendingTrigger` is inside `GameState`, off the SR-8 wire). Gate:
  `pending_trigger_serde_roundtrip`. **This closes the SR remediation track (SR-1..16).**
  Earlier same day: SR-6 — card defs extracted to `mtg-card-defs` + DSL to
  `mtg-card-types`; engine-internal edits no longer re-typecheck the 1,749 defs
  (`CARGO_INCREMENTAL=0` check 7s → 2–3s; defs report `Fresh`). All 1,749 def files moved with
  **zero content edits** via a two-module re-export in `card-defs`. Earlier same day: SR-5 —
  `state::keyword_registry` gates new KeywordAbility variants; the task's "117 KeywordAbility catch-alls" premise was a misattribution — only 2 of them are on that enum, the rest sit on `AbilityDefinition`/`ZoneId`/`ZoneChangeAction`, filed as `scutemob-67`; 3129 tests. Earlier same day: SR-4 — 398 swallow-sites in effects/resolution classified LKI-vs-bug; `state::diagnostics` vocabulary. SR-3 — invariant #3 machine-enforced: GameState sealed, 287 files migrated, `cargo build --workspace` added to CI as the seal gate. SR-2 — invariant #9 registry gate; clean coverage 57.6%. The prior 56.2% was an undercount: the authoring report's `abilities: vec![]` regex also matched nested `mana_abilities: vec![]`. SR-1 — CI live.)

---

## 2026-07-18 — DOC remediation complete (appended per recurrence rule)

Doc audit (`memory/doc-audit-2026-07-18.md` F1-F8) remediated in one sitting after PB-OS1:
- **DOC-5+8** (`scutemob-121`/`124`, merge `e22a836f`, coordinator-inline per gated protocol):
  15 files archived to `memory/archive/2026-07/` (README'd); §3 glob widened to `*review*.md`;
  DOC-8 ruling recorded (abilities distillation authorized → follow-up `scutemob-127`;
  primitives + reviews stay untouchable); Gate A also caught `/implement-primitive` pointing
  at a stale WIP duplicate (frozen at PB-AC9) — 12 refs repointed to live `memory/primitive-wip.md`.
- **DOC-1v2** (`scutemob-125`, merge `4c7995b0`): CLAUDE.md 77.8KB → 34KB; changelog → this
  file (verbatim); invariants → `docs/engine-invariants.md` (routed); counts fixed
  (1,798 / Seventeen agents); recurrence rule added; secondary docs index + 5 skill lines.
- **DOC-3** (`scutemob-119`, auto-memory, empty repo branch): 18/18 MEMORY.md links resolve;
  SR-6 helpers.rs path fixed; removed-skill refs cleaned; archive-move repoints applied.
- **DOC-2** (`scutemob-118`, merge `c0de9550`): 7 stale docs bannered
  (RETIRED/HISTORICAL + successor); `project-status.md` retired outright (user decision);
  CLAUDE.md routing repointed.
- **DOC-6v2** (`scutemob-126`, merge `78a10cc0`): `.claude/docs.yaml` created over the living
  set; `<!-- last_updated -->` stamps adopted (~20 docs); milestone-reviews/sr-plan/corner-case
  stamps fixed; layer-bypass-audit disambiguated; `ability-wip.md` cleared to IDLE;
  ability-coverage stamped-and-deferred.
- Cancelled-superseded: `scutemob-117`/`120`/`122`/`123`. One ~10s ESM outage mid-run, recovered.

## 2026-07-18 — PB-OS2 collected (`scutemob-128`, merge `6fe4f140`)

EF-EF1-A closed: `pay_optional_cost`/`try_pay_optional_cost` now return
`Vec<SacrificedCreatureLki>` and the `MayPayThenEffect` executor threads
`ctx.sacrificed_creature_lki`/`sacrifice_fired` before the `then` effect — pre-zone-move,
layer-resolved (PB-EF10 `SacrificedCreatureLki` reused). `disciple_of_freyalise` front face
authored `Complete` (+1, honest since W-EMPTY). Decoy pinned by anthem + wrong-creature and
proven by revert-and-rerun; decline path proves no stale LKI leak. No PROTOCOL/HASH change
(sentinels assert 18/55 untouched). Reviews: primitive-impl-reviewer clean, /review 4/4 ACs
zero issues. Next: PB-OS3 (OOS-EF6-1 WhenTappedForMana target dispatch).

## 2026-07-18 — PB-OS3 collected (`scutemob-129`, merge `fd922b74`) — CORRECTNESS GROUP COMPLETE

OOS-EF6-1 closed: `fire_mana_triggered_abilities` deferred branch reclassified
`PendingTriggerKind::Normal` → `CardDefETB` so the def raw-index lookup forwards
WhenTappedForMana targets (PB-EF3's Option B pattern; CR 605.5a). `forbidden_orchard`
`known_wrong` → **Complete** — both halves compose: PB-EF12 chosen-color mana + opponent-
targeted token with `recipient: DeclaredTarget{0}`, 4-player decoy pins the recipient.
Roster sweep test derived from `all_cards()`; doubler/immediate-branch regression pinned;
non-vacuity by revert-and-rerun. No PROTOCOL/HASH change. Reviews clean (1 INFO, no action).
**PB-OS1..OS3 (correctness group) all shipped.** Next: PB-OS4 (return-transformed /
enters-transformed, capability, ~2-3 of 4 DFCs, PROTOCOL bump expected).

## 2026-07-19 — PB-OS4 collected SHIPPED-NARROWED (`scutemob-130`, merge `7ee96913`)

Return-transformed as a NEW object ships: `Effect::ExileSourceAndReturnTransformed`
(CR 400.7 + 712.18 — new ObjectId, back-face characteristics layer-resolved, no
counters/auras carried, Saga 714.4 no-sacrifice), PROTOCOL 18→19 / HASH 55→56 (single
bump, digests re-pinned fresh from failing gates). **Reviewer HIGH became the headline**:
the transform machinery gathers FRONT-face abilities regardless of face
(`register_static_continuous_effects`, `queue_carddef_etb_triggers`, upkeep scan) —
filed **OOS-OS4-2** (own PB; likely implicates PB-EF5's shipped TransformSelf
`Complete` markers — correctness candidate to triage AHEAD of OS5). Ship narrowed
accordingly: edgar UN-authored (would register its front anthem on the Coffin — wrong
state), two speculative effect variants removed, honest yield **0 Complete flips + fable
partial with real ch. III usage**; nicol_bolas/grist blocked on **OOS-OS4-1**
(planeswalker-back loyalty). OOS-EF5-3 honestly NARROWED, not closed. An oracle-text
divergence (edgar has no end-step clause) was caught against cards.sqlite mid-implement.
Gates all green; 9 tests. DOCB-1 deferred items completed at this collect:
primitive-wip reset to IDLE, authoring-report regenerated (1,799 defs, 62.2%).

## 2026-07-19 — Audit #2 remediation complete (DOCB-1/2/3, `scutemob-131`/`132`/`133`)

DOCB-1 (coordinator-inline, `888981f8`+`8154db85`+`299bb536`): full state resync — CLAUDE.md:17
trimmed, workstream-state rotated with OOS/OS/DOC handoff, PB-OS1 ✅ banner backfilled
(re-dispatch hazard dead), MEMORY.md fixed, audit files committed, authoring-report on main,
primitive-wip reset at OS4 collect. DOCB-3 (`d3a04a3a` + `7b8efcaa`): 10 polish items (1
worker-missed, coordinator-applied). DOCB-2 (`cf9535d6`, provisioned-override justified):
/implement-primitive + /audit-cards rewired onto live sources, /start-work RETIRED with
banner, /collect gained mandatory PB state-sync step (N4 root-cause fix); risk noted —
esm update may clobber provisioned-skill customizations. PB dispatch gate LIFTED.

## 2026-07-19 — PB-OS4b collected (`scutemob-134`, merge `77d411a0`) — OOS-OS4-2 CLOSED

Face-aware ability gathering shipped **wire-neutral** (PROTOCOL 19 / HASH 56 unchanged):
`def.effective_abilities(is_transformed)` at every gathering site — the 3 filed sites plus
the sweep's finds (activated/mana/triggered population, and reviewer E1's three main-phase/
end-step CardDef trigger sweeps; 8 boundary sites route through one `apply_face_change`).
**docent_of_perfection + bloodline_keeper were Complete-but-WRONG and are now verified
Complete by execution** (pinning tests; the PB-EF5 markers survive honestly). Fable's
back-face Reflection is now activatable (stays Partial, ch. I/II only); growing_rites/
thaumatic back abilities functional. Face-distinction decoys incl. there-and-back and
non-DFC-noop. Reviewer E2 found the non-Static registration family is 10 variants (not 4)
— documented, not extended (no roster card reaches it). Edgar half deferred as
**OOS-OS4-3** (re-add `ReturnSourceToBattlefieldTransformed`, 1 wire bump). Tests 3476 →
**3512**. Reviews: 4/4 PASS, 1 LOW documented in source. Next: PB-OS5.

## 2026-07-19 — PB-OS5 collected (`scutemob-135`, merge `de58b1cc`) — OOS-EF4-1 CLOSED

`EffectAmount::OtherAttackersSharingCreatureType { relative_to }` (discriminant 24) —
resolution-time, layer-resolved shared-type count, Changeling-safe, any-controller per oracle.
`shared_animosity` inert→**Complete**; `goblin_piledriver` authored **Complete** (Sum of
existing counts — zero extra wire surface); rabblemaster pump implemented (partial,
forced-attack blocker); muxus attack-half authored (partial → PB-OS8 dig blocker). Single
PROTOCOL 19→**20** / HASH 56→**57**. Reviewer clean bill. Coverage **1,121/1,801 = 62.2%**.
Next: PB-OS6 (DFC flip-condition sub-batch, OOS-EF5-4).

## 2026-07-19 — PB-OS6 collected (`scutemob-136`, merge `63ca78ce`) — OOS-EF5-4 a/b/g shipped

DFC flip-condition sub-batch: (a) top-card-is-instant/sorcery reveal Condition (delver flip,
CR 702.x reveal semantics; deviation-scan ALLOWLIST entry justified — mandatory-if-true is
faithful), (b) count on `WheneverYouAttack` (legions_landing 3+), (g) `RemoveFromCombat`
(thaumatic_compass back face). **3 Complete**: delver_of_secrets, legions_landing,
thaumatic_compass (the OS4b face-aware work made the back-face halves real). (c) Sacrifice
count deferred → **OOS-OS6-1** (Westvale); (d) folded into PB-OS8 as planned. Two stale
PB-EF5 regression guards updated partial→complete. Single batched PROTOCOL 20→**21** /
HASH 57→**58**, append-only history verified. Review: clean bill (1 LOW). Tests **3538**;
coverage **1,124/1,802 = 62.4%**. Next: PB-OS7.

## 2026-07-19 — PB-OS7 collected (`scutemob-137`, merge `e2dd4c1f`) — OOS-EF3-1 CLOSED

`EffectFilter::CreaturesControlledBy(PlayerId)` (discriminant 36), defending player captured
at effect creation (layer system can't read EffectContext — PB-EF9 precedent).
`silumgar_the_drifting_death` **Complete**: per-Dragon trigger, per-defender scope (4p
bystander decoy), EOT expiry through PB-OS1's reversion machinery, same/different-defender
stacking, SBA toughness-death scoping — 11 execution-probing tests. PROTOCOL 21→**22**
(machine-forced — CC-W had put the filter struct in the wire closure at v14; worker
stop-and-flagged the plan's no-bump misprediction, exemplary per reviewer) / HASH 58→**59**.
Karazikar not authored → **OOS-OS7-1** (defending-player target-validation sibling +
opponent-vs-opponent TriggerCondition; also unblocks kogla R1). **OOS-OS7-2** (doc-only
MEDIUM): CR 611.2c entering/leaving-control edge is engine-wide and pre-existing. Review:
no HIGH, initial needs-fix resolved (2 LOW fixed, negative tests added). Coverage
**1,125/1,803 = 62.4%**. Next: PB-OS8.

## 2026-07-19 — PB-OS8 collected (`scutemob-138`, merge `38246a6e`) — OOS-EF10-1 CLOSED

`Effect::LookAtTopThenPlace` (top-N scope pinned by position-N+1 decoy — the SearchLibrary
distinction; look events private per invariant #7) + `TargetFilter.min_cmc_amount` (both CMC
boundary decoys). **birthing_ritual inert→Complete, growing_rites partial→Complete** (OS6's
deferred (d) closed). birthing_pod honestly blocked → **OOS-OS8-1** (Phyrexian mana in
activated costs — new finding); muxus put-multiple → **OOS-OS8-2**. PROTOCOL 22→**23** /
HASH 59→**60**. Initial needs-fix resolved (large_enum_variant allow per repo precedent).
Coverage **1,127/1,803 = 62.5%**. NOTE: esm worktree check false-flagged conflicts +
provisioned damage on this task — attestation branch-name drift (recorded hazard); real
merge verified clean via git merge-tree first. Next: PB-OS9.

## 2026-07-19 — PB-OS9 collected (`scutemob-139`, merge `6800d924`) — OOS-EF3b-1 CLOSED

`Condition::YouControlYourCommander` (CR 903.3d): controls + battlefield + phased-in + OWNS —
per-owner `commander_ids` makes stolen-commander semantics correct both directions (yours
stolen → OFF; stole-back → ON). One `check_condition` arm serves intervening-if AND
continuous-grant paths; grant drops on leave/steal via layer re-eval.
`skyhunter_strike_force` **Complete**; loyal_apprentice + siege_gang_lieutenant authored
CR-correct but stay partial → **OOS-OS9-1** (AtBeginningOfCombat trigger sweep gap — new
finding). PROTOCOL 23→**24** / HASH 60→**61** (both machine-forced, stop-and-flagged).
Review: clean. Coverage **1,128/1,803 = 62.6%**. Next: PB-OS10, then OS11 closes the queue.

## 2026-07-19 — PB-OS10 collected (`scutemob-140`, merge `dcef1775`) — OOS-XS-1 + OOS-EF7-1 CLOSED

Both seeds verified REAL against CR before building (falsification branch exercised and
passed: CR 601.2c + "another" grammar confirms Hidden Strings distinctness; Jitte oracle =
any recipient). Shipped `TargetRequirement::TargetPermanentDistinctFrom(usize)` (opt-in
per-slot, casting.rs) + `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` (deduped
once-per-source per combat step). **umezawas_jitte `known_wrong`→Complete** (trigger + modal
`Activated::modes` conversion, execution-verified). hidden_strings stays honestly
`known_wrong` — a SURVIVING second blocker (tap-or-untap "may" optionality; distinctness now
enforced+pinned for when it clears). Single PROTOCOL 24→**25** / HASH 61→**62**. Review LOW-only,
dispositioned; 16 tests. Coverage **1,129/1,803 = 62.6%**. Next: PB-OS11 — closes the queue.

## 2026-07-19 — PB-OS11 collected (`scutemob-141`, merge `bd220b00`) — THE PB-OS QUEUE IS COMPLETE

Final batch. **Both seed premises verified stale against oracle and honestly reframed before
building** (feedback_verify_full_chain at its best): OOS-LKI-3's filed "sacrifice for X=counters"
is not modern Workhorse — it is `Cost::RemoveCounter` needing mana-ability LOWERING
(`ManaAbility.remove_counter`); OOS-TS-1's exclude_subtypes already existed — the real gap was
`WheneverYouAttack { filter }` (once-per-batch filtered attack trigger). **6 flips**: workhorse
(new), anim_pakal_thousandth_moon, general_kreat_the_boltbringer, hermes_overseer_of_elpis
(TODO-sweep adds), gemstone_array + druids_repository (RemoveCounter backfills) — beat the ~2
estimate. PROTOCOL 25→**26** / HASH 62→**63**. 19 execution-based tests; review MEDIUM fixed,
both reviews clean. Coverage **1,135/1,804 = 62.9%**.

**QUEUE TOTALS (PB-OS1..OS11 + OS4b, `scutemob-115`..`141`, ~26h)**: 12 PBs + 1 triage +
DOC/DOCB interludes; coverage 62.1% → **62.9%** (+18 clean flips + 4 Complete-but-wrong made
right: sarkhan_vol/zealous_conscripts control reversion, docent/bloodline face-aware);
known_wrong closed: swan-song-class integrity holds — forbidden_orchard, umezawas_jitte,
delver; PROTOCOL 18→**26**, HASH 55→**63** (all machine-forced, every bump justified);
tests 3476 → **3560+**; 8 rider seeds filed for the next triage; 2 seed premises falsified-
and-reframed, 1 falsification branch exercised (Hidden Strings distinctness confirmed real).
Every worker review clean or fixed; zero forced flips.
