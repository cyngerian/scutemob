# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

> Detailed PB-by-PB handoffs, hazards, and seed inventories live in `memory/workstream-state.md`.
> Worker sessions: append detail there, not here. CLAUDE.md tracks current snapshot only.

- **Active Milestone**: M9.5 DONE — **Card Authoring Campaign ACTIVE** (plan: `memory/card-authoring/campaign-plan-2026-05-16.md` §0 recalibration 2026-07-07; clean coverage **1,081/1,786 = 60.5%** per `tools/authoring-report.py`; **EF queue ACTIVE (`memory/primitives/ef-batch-plan-2026-07-17.md`) — PB-EF1..EF4 + EF-13 SHIPPED (`scutemob-99`/`101`..`105`, correctness group complete); PB-EF5 in flight `scutemob-106`**; **PB-AC chain COMPLETE — AC0..AC9 all shipped**; **marker sweep COMPLETE — `scutemob-88`**; **SR-33..38 chain COMPLETE**; **W-PB2 + W-EMPTY + W-MISS COMPLETE — `scutemob-95`/`96`/`97`**)
- **Invariant #9 is machine-enforced (SR-2).** `CardDefinition.completeness` (`Complete` by
  Default) marks a def `Inert` / `Partial` / `KnownWrong`; `validate_deck` rejects any
  non-`Complete` card with `DeckViolation::IncompleteCard`. `CardRegistry::try_new` errors on
  duplicate CardIds. Current markers: 62 inert, 570 partial, 97 known-wrong (`scutemob-88`).
  **New card defs must be `Complete` or carry a marker with a note** — an inert def now fails a
  test. **"Inert" means registers no *behaviour*, not `abilities: vec![]`** — a cost-reduction
  static is a `spell_cost_modifier`, not an `AbilityDefinition`, so those defs correctly ship an
  empty `abilities` vec and are Complete. `card_registry_gate::registers_no_behavior` is the
  predicate and it checks every behaviour-bearing field; **adding such a field to
  `CardDefinition` means adding it there**, and `inert_gate_is_not_vacuous` pins both directions.
- **Invariant #3 is machine-enforced (SR-3).** All `GameState` fields are `pub(crate)` with
  one public read accessor each; the `&mut`-handing methods (`player_mut`, `object_mut`,
  `add_object`, `move_object_to_zone`, `next_object_id`, …) are `pub(crate)` too. Outside the
  engine crate a `GameState` is read-only — the only mutation path is a `Command` through
  `process_command`. Tests/benches get mutable access via `state::test_util` + `*_mut()`
  accessors, gated on the `test-util` feature (self dev-dependency). **`cargo build
  --workspace` is the only gate that proves the seal** — `test --all` and `clippy
  --all-targets` enable `test-util` workspace-wide via feature unification. It is a CI step.
- **Tests**: **3383 passing** across 29 suites (SR-9a consolidated 297 test binaries into 9); build/clippy/fmt clean
  — and `fmt` here means `cargo fmt --check` **plus** `tools/check-defs-fmt.sh`, which is the only one
  of the two that looks at the 1,748 card defs (SR-35)
- **CI**: **LIVE and green** since 2026-07-10 (SR-1, merge `e9742dc2`) — single Ubuntu job (fmt + clippy + `build --workspace` + full tests) on push/PR to main + workflow_dispatch; rust-cache@v2, 45m timeout. **Toolchain pinned (SR-11, `scutemob-63`)**: `rust-toolchain.toml` pins exact stable `1.95.0` and CI reads that `channel` from the file (no more floating to latest stable), so local `clippy -D warnings` is an authoritative CI preview. SR remediation track: original SR-1..16 all DONE 2026-07-10; a 2026-07-11 re-audit of the remediated baseline filed **SR-17..SR-32**, all DONE 2026-07-14..16 (16/16 collected; full record: `docs/sr-remediation-plan.md`).
- **Abilities**: ~199 validated; 42/42 P1; 17/17 P2; 40/40 P3; 95/95 P4 implemented (9 permanent-n/a; 1 deferred: Banding)
- **Primitives**: PB-0..PB-37 + named-letter chain (PB-A/B/E/J/M/S/X/Q/Q4/N/D/P/L/T/SFT/CC-{W,B,C,A}/TS/LKI-CC/CD/LKI-Power/EWC/XS/XS-E/XA/EAT/XA2/EWC-D) all DONE. PB-Q2/Q3/Q5 reserved.
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
- **Open primitive seeds**: OOS-XA2-1/2/4/5, OOS-EWCD-1..3, OOS-EAT-1..3, OOS-XS-E-2; older OOS-XS-1/3/4, OOS-LKI-Power-1/4/5, OOS-LKI-1..4, OOS-TS-1..4 — all 0-yield defensives or card-gated; high-confidence backlog exhausted. (OOS-XA-3/XA2-3 RESOLVED by `scutemob-30`; OOS-LKI-Power-3 shipped.) Full list: `memory/primitives/pb-retriage-CC.md`.
- **Known issues**: 0 HIGH; 2 MEDIUM (pre-M8 deferred to M10+); **6 LOW open** (4 M10-gated: MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). Full: `docs/mtg-engine-milestone-reviews.md`.
- **Strategic Review**: `docs/mtg-engine-strategic-review.md` (2026-03-07) — decouple M11 from M10, split M10, downscope M12, web-vs-Tauri decision pending
- **Silent failures are classified in the resolution path (SR-4).** In `effects/mod.rs`
  and `rules/resolution.rs`, a state lookup whose absence is an engine bug goes through
  `state::diagnostics`'s `expect_*` family (`debug_assert!`, `#[track_caller]`); one whose
  absence is a rules-correct fizzle goes through `lki_*` and carries a CR citation.
  `layers::expect_characteristics` is the asserting form of `calculate_characteristics`.
  **New code in these files must pick a side** — a bare `state.objects.get_mut(&id)` no
  longer says which it is. Method: `docs/sr-4-silent-failure-audit.md`. The rest of
  `rules/` is not yet swept (`scutemob-66`).
- **Every `KeywordAbility` variant must declare where its behavior lives (SR-5).**
  `state::keyword_registry::handling` is an exhaustive match classifying all 166
  variants as `Handled { sites }` (engine code branches on it, at exactly these files)
  or `Marker { carrier, cr }` (presence marker; the rules text is implemented by
  `carrier`, per the cited CR — 18 of these). **Adding a variant is a compile error
  until you classify it**, and `tests/keyword_registry.rs` then checks the claim
  against the source tree: declared site sets must exactly equal a comment-stripped
  scan, so a keyword that loses its last dispatch site — or a `Marker` that gains one —
  fails the suite. Audit: `docs/sr-5-keyword-catchall-audit.md`. The same hazard on
  `AbilityDefinition` / `ZoneChangeAction` is not yet gated (`scutemob-67`).
- **Card defs compile in isolation from the engine (SR-6).** The workspace bottom is
  `crates/card-types` (`mtg-card-types`: the DSL — `cards/{card_definition,helpers,registry}.rs`
  — plus the 11 pure-data `state/` modules it needs). `crates/card-defs` (`mtg-card-defs`:
  1,749 def files + `build.rs` discovery) depends on **card-types only, never on the engine**;
  `crates/engine` depends on both and re-exports them, so every `crate::state::…` and
  `crate::cards::…` path inside the engine, and `mtg_engine::{all_cards, CardDefinition, …}`
  outside it, resolve exactly as before. **Touching an engine file leaves `mtg-card-defs`
  `Fresh`** (`cargo check -p mtg-engine -v`); touching `card-types` correctly rebuilds it.
  The arrow direction is the whole mechanism — putting defs *above* the engine (as the
  pipeline doc originally sketched) would recompile all 1,749 cards on every rules edit.
  Nothing in `card-types` may reference `GameState`. Keyword-registry sites (SR-5) are now
  **workspace-relative** paths and the scan spans both crates.
- **`PendingTrigger` is built through `PendingTrigger::blank` only (SR-7).** The 13
  per-keyword `Option` fields are gone; a trigger kind's payload lives in
  `data: Option<TriggerData>` (`card-types/src/state/stack.rs`), which
  `flush_pending_triggers` reads and threads into `StackObjectKind::KeywordTrigger`.
  `tests/pending_trigger_shape.rs` pins the struct's 16-field set, requires every
  `PendingTrigger { .. }` literal to carry `..PendingTrigger::blank(source, controller, kind)`,
  and asserts each `TriggerData` variant still has a consumer in *both* `abilities.rs` and
  `resolution.rs` — **deleting a `resolution.rs` match arm compiles with zero errors** and
  would otherwise make the trigger a silent no-op. **New per-kind state goes in a
  `TriggerData` variant, never as a field on the struct** — a new field fails the suite.
  `HASH_SCHEMA_VERSION` is now **37**.
- **The card-def corpus is format-checked by `tools/check-defs-fmt.sh`, not by `cargo fmt` (SR-35).**
  `cargo fmt --all -- --check` exits 0 having checked **zero** of the 1,748 files in
  `crates/card-defs/src/defs/`: rustfmt walks `mod` declarations *textually*, expanding no
  macros and running no build scripts, and `defs/mod.rs` is one
  `include!(concat!(env!("OUT_DIR"), …))` whose `#[path]` mods `build.rs` writes into
  `target/`. Both halves defeat the walk, so the corpus was **unvisited, not clean** — 321
  defs were misformatted, some with plainly broken indentation. The SR-6 layout that causes
  this is worth keeping (one file per card, no shared registry to collide on), so the gate
  hands rustfmt the file list explicitly instead. **Run it, or `cargo test --all` (which
  runs it via `core card_defs_fmt`) — `cargo fmt` will keep lying.** `--fix` reformats.
  **Pointing rustfmt at the files is necessary but not sufficient**, and this is the part
  to remember: rustfmt has two failure modes here that are *indistinguishable from success*
  — no output, exit 0, file untouched. (1) When an expression won't fit `max_width`, rustfmt
  emits the original source verbatim and the fallback propagates to the **enclosing**
  expression; a long `oracle_text: "…".to_string(),` therefore swallows the whole
  `CardDefinition` literal — the whole file. Measured by canary (inject a misindented
  `card_id`, ask rustfmt directly): **1,380 of 1,748 defs were inert**. `format_strings=true`
  splits long strings with `\` continuations, which fits them, which stops the fallback: 0
  inert. (2) An unbreakable over-width line (>~107 cols) does the same thing and *hides other
  errors in the file*; `error_on_line_overflow=true` makes it exit 1. The corpus has zero such
  lines, so that check is hard-fail with **no allowlist** — a def whose formatted output
  overflows 100 columns fails and you split the line by hand. Both flags are passed on the
  command line, never a workspace `rustfmt.toml` (which would restyle the engine crates too).
  **Do not delete either flag to make something pass** — each is pinned by its own canary
  (`gate_catches_a_def_whose_oracle_text_is_one_long_line`,
  `gate_catches_an_unbreakable_over_width_line`), which stands up a throwaway corpus and runs the
  shipped script against it. Those canaries exist because the reformatted corpus **cannot detect its
  own blindness**: with a flag removed, rustfmt leaves the already-`\`-continued defs alone, so the
  gate stays green while every *newly authored* def goes back to invisible. Long
  *comments* do **not** trigger the fallback (245 defs have >100-col comment lines; none inert).
  The reformat was proven semantics-preserving by diffing the full `Debug` of `all_cards()`
  across it: 1,719 files changed, **byte-identical** output.
- **Serialized `Command` / `GameEvent` / replay-log streams carry a version tag (SR-8).**
  Policy is **strict lockstep**: `rules::protocol::Envelope<T>` declares `protocol_version`,
  and a receiver accepts it iff it equals `PROTOCOL_VERSION` **exactly** — older *and newer*
  are rejected with `ProtocolError::VersionMismatch`. No negotiation, no forward compat: per
  invariant #9, a client that tolerates an unknown event variant holds a history it cannot
  rewind and cannot tell that it does. `decode` is **staged** (probe version → reject → parse
  payload) so a mismatch is never an opaque serde error. `ReplayLog` also carries
  `hash_schema_version` and checks it separately. **The version is machine-checked**:
  `PROTOCOL_SCHEMA_FINGERPRINT` pins a blake3 digest of the **transitive type closure** of
  the three wire frames (its size and the digest itself live in `rules/protocol.rs`; do not
  re-quote them here — they move whenever the wire does), and `tests/protocol_schema.rs` recomputes it from source — so
  `#[serde(skip)]`/`rename`/`rename_all` (all invisible to rustc) and any shape change fail the
  build. The closure reaches `Characteristics` → `Effect` → the whole card DSL, so **adding an
  `Effect` variant is a wire change and most PBs will bump `PROTOCOL_VERSION`**; it stops at
  `GameState`, which is why this and `HASH_SCHEMA_VERSION` stay separate. The current
  `PROTOCOL_VERSION` is the `pub const` in `rules/protocol.rs` (read it there rather than
  quoting a number that drifts). Policy: `docs/mtg-engine-protocol-versioning.md`. **This was M10's hard blocker.**
- **Integration tests are 9 targets, not 297 binaries (SR-9a).** `crates/engine/tests/*.rs` became
  `crates/engine/tests/<group>/{main.rs, *.rs}` — `core`, `rules`, `combat`, `casting`,
  `primitives`, `scripts`, `mechanics_{a_d,e_l,m_z}`. Every file moved verbatim; a former
  per-file binary is now a **module**, so `--test run_all_scripts` is `--test scripts
  run_all_scripts` and `--test layers` is `--test rules layers::` (keep the `::`). Warm rebuild
  after an engine edit **34.2s → 11.1s**, `target/` **19 GB → 2.2 GB**. **Never add a top-level
  `tests/*.rs`** — each is another link on every test build, and `tests/no_stray_test_binaries.rs`
  fails the suite. That gate also fails when a file sits in a group dir with no `mod` line in the
  group's `main.rs` — such a file is not compiled and its tests silently cease to exist
  (demonstrated: `--test combat` reports `ok. 69 passed` with six tests missing) — and, because
  that check is textual, it additionally requires a group's `main.rs` to contain **nothing but
  `//!` docs and bare `mod x;` lines** and group dirs to be flat. Layout and the rule for where a
  new test file goes: `docs/sr-9a-test-consolidation.md`. Note `tests/proptest-regressions/` is
  **not** a test group (`NON_GROUP_DIRS`) — `proptest` writes it on its first failure and the group
  check would otherwise redden a second time and bury the real failure.
- **The golden-script corpus is triaged and cannot skip silently (SR-9c).** The 271 scripts in
  `test-data/generated-scripts/` are now **210 `approved` / 61 `retired` / 0 `pending_review`**.
  `ReviewStatus::Retired` carries a **required `retirement_reason`**; a retired script is excluded from the
  run but printed, never absent. `tests/scripts/run_all_scripts.rs` **partitions** the discovered set
  (`approved + retired == discovered`) and fails on a `pending_review`/`disputed`/`corrected` script, a file
  that doesn't deserialize (`discover_scripts` no longer swallows the `Err` — six scripts had been invisible
  since written), an approved script with zero `assert_state`, or one using an untranslatable
  `player_action` outside a **dead-entry-guarded allowlist** (`ALLOWED_UNTRANSLATABLE_ACTIONS` in
  `run_all_scripts.rs` — currently `search_library` only). The replay *checker* (`script_replay.rs`) was itself largely vacuous:
  an unrecognized assertion path returned "no mismatch" (**244** assertions unchecked) and `zones.stack`
  was tested against a hardcoded empty list (**583** `is_empty:true` always passed). Both are now real —
  an unknown path is a hard mismatch, `zones.stack` reads the live depth, power/toughness read through
  `calculate_characteristics`. **A new assertion path must be implemented in `check_assertions`, not just
  written in a script** — an unimplemented path now fails, it does not pass. The 61 retirements are a
  worklist: each names the one missing `CardDefinition`, primitive, or un-wired harness `Command` (combat
  damage assignment, mulligan, commander-zone cast, craft, disturb, order-replacements) that would
  un-retire it.
- **The script regime and the direct-`Command` regime cross-validate (SR-9b).**
  `tests/scripts/harness_equivalence.rs` expresses a scenario twice — as a JSON `initial_state` +
  action strings, and as `GameStateBuilder` + `Command` literals — and requires the same **fingerprint**
  (`public_state_hash` **plus every player's `private_state_hash`**, because the public hash omits hand
  and library *contents*) after **every** step, plus a `proptest` over random move sequences. Both
  regimes call `enrich_spec_from_def`, so what is proven is not enrich's inference but everything
  around it: id assignment, insertion order, life/mana/turn/step patching, and that
  `translate_player_action` builds the `Command` a hand-written test would. **`build_initial_state` was
  nondeterministic** until this task — `InitialState`'s zone/player maps are `std::collections::HashMap`
  and `ObjectId`s are assigned in insertion order, so the same script built different states run to run
  (40 builds → 2 hashes). Every loop over a script-supplied map must now go through
  `sorted_zone_entries`. `init.turn_number` was also declared and never read (every script ran on turn 1).
  `resolve_targets` used to **drop** an unresolvable target (`filter_map`), turning a `cast_spell` at an
  absent permanent into a targeted spell cast with **no target** (CR 601.2c); it now returns `None`.
  A script may still name a card with **no `CardDefinition`** — the object enters typeless and silent,
  bypassing invariant #9 — pinned as a shrinking allowlist and handed to SR-9c along with seven other
  `initial_state` fields the harness ignores. **Only 6 of `translate_player_action`'s 60+ `Command`
  shapes are cross-validated**; the alt-cost translations (convoke, delve, escape, kicker, casualty,
  splice, escalate, modal, mutate, ninjutsu…) are not. Adding a scenario is cheap.
- **An activation cost is only paid if some code pays it, and nothing was checking (SR-36).**
  Two HIGH defects, both pre-existing, both fixed: (1) `handle_tap_for_mana` had no
  `AddManaScaled` branch and read the registered `produces: {colour: 1}` **marker**
  literally, so Gaea's Cradle tapped for exactly 1 green regardless of board state
  (`ManaAbility::scaled_amount` + step 6c now resolve it via `resolve_amount`, CR 605.1a);
  (2) `flatten_cost_into` mapped `Cost::PayLife(_) => {}` and `ActivationCost` had no life
  field, so **any activated ability not lowered into a `ManaAbility` paid no life at all**
  (`ActivationCost::life_cost` + a payment step in `handle_activate_ability`, CR 118.3/119.4,
  with the CR 119.4b short-circuit — a 0 cost is payable at negative life). The two payment
  paths are **disjoint by construction**: `mana_ability_lowering` is the same call that builds
  the `ManaAbility` and excludes the ability from `activated_abilities`, so neither can
  double-charge. **The rosters were the finding, not the fixes.** SR-34 filed SF-9 around
  Staff of Compleation and said "a full corpus scan is still owed"; running it over
  `all_cards()` found **28 ability rows, 14 on `Complete` defs** — the **entire 11-card
  fetchland cycle** (Arid Mesa, Bloodstained Mire, Flooded Strand, Marsh Flats, Misty
  Rainforest, Polluted Delta, Prismatic Vista, Scalding Tarn, Verdant Catacombs, Windswept
  Heath, Wooded Foothills) plus Doom Whisperer, Razaketh and Warren Soultrader, all shipping
  `Complete` and **free**, in legal decks, invisible to every marker and gate. Conversely
  SF-8's roster is **smaller** than filed: the five cards it speculated about (Everflowing
  Chalice, Elvish Guidance, Brightstone Ritual, Battle Hymn, Black Market) carry
  `AddManaScaled` on a **`Spell`** or **`Triggered`** ability (both resolve through the stack,
  where the amount was always evaluated correctly) or only in aspirational notes — falsified,
  exactly as that finding's own "not re-verified" caveat warned. **A roster from
  "by the same shape" reasoning is a hypothesis; enumerate `all_cards()` and filter on the
  `AbilityDefinition` variant, never grep source** — a grep would have matched all five.
  Deleting SR-34's Finding-A exclusion widened Cabal Coffers / Cabal Stronghold / Crypt of
  Agadeem into real mana abilities, **upgraded `Partial` → `Complete`** (coverage 57.1% →
  **57.3%**, marker drift 6 → 3 — an *increase*, unlike the last three SRs), each proven by
  activation with a decoy its filter must exclude. **A decoy must fail on exactly the field
  under test**: the reviewer caught that `cabal_stronghold_counts_only_basic_swamps` used
  Cabal Coffers, which has *no subtypes*, so `matches_filter` rejected it on `has_subtype`
  and never reached the `basic` check — deleting `basic: true` left the test green. Bayou
  (Land — Forest Swamp, nonbasic) is the real decoy. SF-8 also **exposed an asymmetry in
  SR-33's colour gate**: `printed_tap_mana_colors` drops a "for each" clause from the printed
  side, but `registered_colors` read every mana ability, so the moment Cabal Stronghold's
  scaled arm became real the gate cried `invented [Black]` against a card printing `{B}`
  plainly — fixed by `scaled_amount.is_none()` there, and `scaled_amount` is the only thing
  that lets the registered side identify a dynamic ability at all. `PROTOCOL_VERSION` 3→4,
  `HASH_SCHEMA_VERSION` 41→42, both machine-forced. Review: **0 HIGH**, 3 MEDIUM (all closed,
  incl. a third consecutive stale-marker note — `yawgmoth_thran_physician` said "un-author
  until PayLife is representable"; it now is). Findings for the next SR:
  `memory/card-authoring/sr36-engine-findings-2026-07-17.md` (**SG-1 MEDIUM: the simulator's
  `LegalActionProvider` ignores `life_cost` — harmless while the cost was dropped, now it
  offers bots unpayable actions**).
- **Last Updated**: 2026-07-18 (**PB-EF4 collected, `scutemob-105` merge `26421364`** —
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

### What Exists (M0-M9.5 + Engine Core Complete + all P3/P4 abilities)

- `cards/`: CardDefinition framework (30+ Effect primitives), ~1693 card defs across hand-authored + templated waves; CardRegistry
- `effects/`: Full effect execution engine (DealDamage, GainLife, DrawCards, ExileObject, CreateToken, SearchLibrary, ForEach, Conditional, Scry, Surveil, DrainLife, Goad, Fight, etc.)
- `rules/`: Turn structure, priority, stack, SBAs, dependency-based layer system, combat, casting (Convoke/Improvise/Delve/Evoke/Kicker/Morph/Disturb alt costs), resolution, ETB trigger queueing (CR 603.3/603.4), ETB & global replacements, prevention, Commander (deck validation, command zone, tax, zone-return SBA, mulligan, companion, partner variants), protection (DEBT), copy (Layer 1 + storm + cascade), loop detection (CR 104.4b), Enchant, suspend, Mutate (CR 702.140), Transform/DFC (CR 701.28/712), Daybound/Nightbound, Craft, Morph/Megamorph/Disguise/Manifest/Cloak; Type Consolidation refactor complete (CastSpell 32→13, SOK ~20, AbilDef 55)
- `testing/`: Replay harness (`crates/engine/src/testing/replay_harness.rs` — public, shared with replay viewer), ~112 approved scripts, ~1934 harness tests, 6-player suite, 54 property invariants
- `benches/`: criterion (priority_cycle_4p 23µs, sba_check 14µs, full_turn_4p 205µs)
- `tools/replay-viewer/`: axum + Svelte 5, 5 API endpoints, 12 components, diff highlighting, keyboard nav
- 36 corner cases: 32 COVERED, 4 GAP, 0 DEFERRED

---

## Project Overview

We are building an MTG rules engine targeting **Commander format** (4-player multiplayer) with
**networked play**. The engine is written in **Rust**, the desktop app uses **Tauri v2** with a
**Svelte** frontend.

The engine is a standalone library crate with no UI or network dependencies. It can be tested
entirely in isolation. The network layer wraps the engine. The Tauri app wraps the network layer.

### Primary Documents

| Document | Location | Purpose |
|----------|----------|---------|
| Architecture & Testing Strategy | `docs/mtg-engine-architecture.md` | Why decisions were made; system design; testing approach |
| Development Roadmap | `docs/mtg-engine-roadmap.md` | What to build and in what order; milestone definitions |
| Game Script Strategy | `docs/mtg-engine-game-scripts.md` | Engine-independent test script generation, JSON schema, replay harness design |
| Corner Case Reference | `docs/mtg-engine-corner-cases.md` | 36 known difficult interactions the engine must handle correctly |
| Corner Case Audit | `docs/mtg-engine-corner-case-audit.md` | Living correctness ledger: coverage status, card def gaps, deferred items |
| Network Security Strategy | `docs/mtg-engine-network-security.md` | **Deferred P2P upgrade path** — not the active M10 plan. M10 uses a centralized server. |
| Milestone Code Reviews | `docs/mtg-engine-milestone-reviews.md` | Per-milestone code review findings, file inventories, issue tracking |
| Replay Viewer Design | `docs/mtg-engine-replay-viewer.md` | M9.5 game state stepper: architecture, API, Svelte components, shared-component strategy |
| Ability Coverage Audit | `docs/mtg-engine-ability-coverage.md` | Keyword and pattern coverage tracking |
| LOW Issues Remediation | `docs/mtg-engine-low-issues-remediation.md` | Tiered plan for ~68 open LOW issues with risk assessment |
| Workstream Coordination | `docs/workstream-coordination.md` | Cross-session coordination for 4 parallel workstreams (abilities, TUI, LOWs, M10) |
| Ability Batch Plan | `docs/ability-batch-plan.md` | 16 batches covering all ~75 implementable abilities (P3+P4) with dependency map |
| Card Pipeline & Scaling | `docs/mtg-engine-card-pipeline.md` | Card definition organization, Rust DSL rationale, scaling strategy (112 → 27k), authoring pipeline |
| Strategic Review | `docs/mtg-engine-strategic-review.md` | 2026-03-07 project review: path-to-playable compression, M10/M11/M12 restructuring, action items |
| Card Authoring Operations | `docs/card-authoring-operations.md` | Ordered task list for triage → fix → author → audit (68 tasks) |
| Runtime Integrity | `docs/mtg-engine-runtime-integrity.md` | Watchdog, recovery, bug reporting — pre-alpha requirement |
| Type Consolidation Plan | `docs/mtg-engine-type-consolidation.md` | Pre-M10 refactoring: CastSpell, SOK triggers, AbilityDef, Designations — 8 sessions |
| Cleanup Retention Policy | `docs/cleanup-retention-policy.md` | Two-tier ladder, year-month archive convention, /cleanup skill protocol |
| This file | `CLAUDE.md` | Current project state; session context |

**Read the architecture doc before implementing anything.**

---

## When to Load What

Before starting work, check which files apply to your task:

| Task | Load before starting |
|------|----------------------|
| Touching any file in `rules/` | `memory/gotchas-rules.md` |
| Touching any file in `state/`, `cards/`, `effects/` | `memory/gotchas-infra.md` |
| Writing or modifying tests | `memory/gotchas-infra.md` (testing gotchas) |
| Writing new code or tests | `memory/conventions.md` |
| Questioning a design decision | `memory/decisions.md` |
| Implementing a new subsystem | `docs/mtg-engine-corner-cases.md` (full) |
| Checking correctness gaps | `docs/mtg-engine-corner-case-audit.md` |
| Starting a new milestone | Use `/start-milestone <N>` — reads only the relevant roadmap section via Grep+offset, never the full file. |
| Writing golden tests | `docs/mtg-engine-game-scripts.md` |
| Implementing network features (M10+) | `docs/mtg-engine-roadmap.md` M10 section (centralized server); `docs/mtg-engine-network-security.md` only for deferred P2P upgrade |
| Implementing replay viewer (M9.5) | `docs/mtg-engine-replay-viewer.md` |
| Implementing a keyword ability | `docs/mtg-engine-ability-coverage.md` |
| Checking ability gaps | Use `/audit-abilities` or `/ability-status` |
| Implementing a single ability end-to-end | Use `/implement-ability` — orchestrates plan → implement → review → fix → card → script → close |
| End-of-milestone cleanup pass | Use `/cleanup` — reads `docs/cleanup-retention-policy.md` and runs Gate A → B → dry-run → execute |
| Fixing LOW issues | `docs/mtg-engine-low-issues-remediation.md` |
| Authoring card definitions | `docs/card-authoring-operations.md` (operations plan with ordered tasks); `docs/mtg-engine-card-pipeline.md` (DSL reference) |
| Triaging card defs for TODOs | Use `/triage-cards` — scans defs, reclassifies blocked sessions, consolidates review findings |
| Authoring a group of cards | Use `/author-wave <group>` — orchestrates author → review → fix → commit for one group |
| Auditing all card defs | Use `/audit-cards` — scans for TODOs, empty abilities, known-issue patterns, certifies completion |
| Type consolidation refactoring | `docs/mtg-engine-type-consolidation.md` (must read — this is the active plan) |
| Planning M10, M11, or M12 | `docs/mtg-engine-strategic-review.md` (must read before starting) |
| Deciding what to work on / coordinating workstreams | `docs/workstream-coordination.md` |

Use `/review-subsystem <name>` to load the right file and see open issues in one step.

---

## Card Authoring Wave Process

The remaining A-29+ groups are ordered into three waves by engine risk level.
**Follow this order** — see `docs/card-authoring-operations.md` "Authoring Order and
Engine Risk Assessment" for the full breakdown.

1. **Wave A** (A-29, A-32, A-33, A-34, A-35, A-39): Safe to author now. Minor/no engine changes.
2. **Wave B** (A-38, A-42): Re-triage each group first — split authorable cards from blocked ones.
3. **Wave C** (A-30, A-36, A-40, A-41): Blocked on significant engine work. Treat as PB-style batch.

**Engine review checkpoints**: After each wave completes, batch-review all engine
changes before starting the next wave. Run `git diff <pre-wave-commit>..HEAD -- crates/engine/src/`
and review the accumulated engine additions. Fix any issues found. This is a single
review pass per wave, not per-session — but it is **mandatory** before advancing to
the next wave. The PB pipeline had plan → implement → review → fix; the authoring
pipeline adds engine code inline without review, so these checkpoints catch that gap.

---

## Architecture Invariants

These are non-negotiable. If a change would violate any of these, stop and reconsider.

1. **Engine is a pure library.** No IO, no network, no filesystem access, no async runtime
   in the engine crate. It takes commands in and emits state changes out. Everything else
   is the caller's responsibility.

2. **Game state is immutable.** Use `im-rs` persistent data structures. State transitions
   produce new states; old states are retained for undo/replay. Never mutate state in place.

3. **All player actions are Commands.** There is no way to change game state except through
   the Command enum. This enables networking, replay, and deterministic testing.

4. **All state changes are Events.** The engine emits Events describing what happened.
   The network layer broadcasts these. The UI consumes these. Events are the single
   source of truth for "what happened."

5. **Multiplayer-first.** Priority, triggers, combat — everything is designed for N players.
   1v1 is N=2, not a special case.

6. **Commander-first.** The command zone, commander tax, commander damage, color identity —
   these are core features, not bolted-on extensions.

7. **Hidden information is enforced.** The engine knows everything. The centralized server
   filters events before broadcasting — private events go only to the relevant player via
   `GameEvent::private_to() -> Option<PlayerId>`. Never expose another player's hand or
   library order to the wrong client. (P2P + Mental Poker is a deferred upgrade path —
   see `docs/mtg-engine-network-security.md`.)

8. **Tests cite their rules source.** Every test references the CR section or known
   interaction it validates. Untraceable tests are technical debt.

9. **Every card in a game must have a `CardDefinition` before the game starts.** The deck
   builder enforces this. No mid-game discovery, no graceful degradation during play. The
   rewind/replay/pause system depends on a complete and accurate state history from turn 1 —
   a card whose abilities silently never fired produces a corrupted history that cannot be
   rewound to correctly. Unimplemented cards are surfaced at deck-building time with clear
   messaging, not silently ignored at game time.

---

## MCP Resources
- **Rules search**: query by rule number ("613.8") or concept ("dependency continuous effects")
- **Card lookup**: query by exact card name for oracle text, types, rulings
- **Rulings search**: query by interaction concept ("copy effect on double-faced card")
- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations, incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to override. See your auto-memory MEMORY.md index (rust-analyzer MCP Server section) for details.

---

## Critical Gotchas

These 3 apply to nearly every session. All other gotchas are in `memory/gotchas-rules.md` and `memory/gotchas-infra.md`.

- **Object identity (CR 400.7)**: When an object changes zones, it becomes a NEW object.
  The old ObjectId is dead. Auras fall off. "When this dies" triggers reference the old
  object. This is the #1 source of bugs in MTG engines.
- **Replacement effects are NOT triggers.** They modify events as they happen. They don't
  use the stack. Getting this wrong breaks the entire event system.
- **SBAs are checked as a batch, not individually.** All applicable SBAs happen simultaneously.
  Then triggers from all of them go on the stack together (in APNAP order).

---

## Agents

Fifteen project-scoped agents in `.claude/agents/` encode milestone, ability, primitive, and card authoring workflows:

| Agent | Model | RA | Trigger | Purpose |
|-------|-------|----|---------|---------|
| `rules-implementation-planner` | Opus | yes | "plan M9 implementation" | Session plan with architecture, CR refs, session breakdown |
| `session-runner` | Sonnet | — | "run session 1" / "next session" | Execute one implementation session from the plan |
| `milestone-reviewer` | Opus | yes | "review milestone M9" | Structured code review with HIGH/MEDIUM/LOW findings; creates fix-session-plan |
| `fix-session-runner` | Sonnet | — | "run fix session 3" | Execute 5-8 fixes, run tests, close issues |
| `card-definition-author` | Sonnet | — | "add card definition for X" | Translate oracle text to CardDefinition DSL |
| `bulk-card-author` | Sonnet | — | "author session 5" | Write batch of 8-20 card defs from authoring plan |
| `card-batch-reviewer` | Opus | — | "review cards batch 5" | Review 5 card defs against oracle text |
| `card-fix-applicator` | Sonnet | — | "apply fixes from review" | Apply review findings to card def files, verify build |
| `cr-coverage-auditor` | Sonnet | — | "check CR coverage for 614" | Audit test/script coverage for CR sections |
| `game-script-generator` | Sonnet | — | "generate script for X interaction" | JSON game scripts for replay harness |
| `ability-coverage-auditor` | Opus | — | `/audit-abilities` | Scan engine + card defs + scripts → refresh ability coverage doc |
| `ability-impl-planner` | Opus | yes | `/implement-ability` (plan phase) | CR research, study similar abilities, write implementation plan |
| `ability-impl-runner` | Sonnet | — | `/implement-ability` (implement/fix phase) | Execute steps 1-4 (enum, enforcement, triggers, tests), apply fixes |
| `ability-impl-reviewer` | Opus | yes | `/implement-ability` (review phase) | Verify implementation against CR, check edge cases, write findings |
| `primitive-impl-planner` | Opus | yes | `/implement-primitive` (plan phase) | CR research, study engine architecture, write PB plan |
| `primitive-impl-runner` | Sonnet | — | `/implement-primitive` (implement/fix phase) | Engine changes, card def fixes, tests, apply review fixes |
| `primitive-impl-reviewer` | Opus | yes | `/implement-primitive` (review phase) | Verify engine + card defs against CR/oracle text, write findings |

---

## Session & Workstream Protocol

- `/start` — bootstrap ESM, check local state, orient (also covers what `/start-session` used to do — workstream state is loaded via `esm project bootstrap` and the auto-memory MEMORY.md index)
- `/start-work W1-B3` — claim a workstream before coding (prevents parallel collisions)
- `/eot` — end-of-turn / end-of-session: ESM session close + workstream-state rotation + memory routing (replaces `/end` + `/end-session`)
- State file: `memory/workstream-state.md` (shared across sessions)
- Conventions: `memory/conventions.md` | Decisions: `memory/decisions.md`
- Dev environment: `.claude/CLAUDE.local.md`

### Commit Prefix Convention

| Workstream | Prefix | Example |
|------------|--------|---------|
| W1: Abilities | `W1-B<N>:` | `W1-B3: implement Ninjutsu` |
| W2: TUI & Simulator | `W2:` | `W2: fix blocker declaration` |
| W3: LOW Remediation | `W3:` | `W3: add debug_assert to sba.rs` |
| W4: M10 Networking | `W4:` | `W4: add GameServer skeleton` |
| W5: Card Authoring | `W5-cards:` | `W5-cards: author Skullclamp, Blood Artist` |
| Cross-cutting | `chore:` | `chore: update workstream-state` |

---

## Milestone Completion Checklist

When completing a milestone:

- [ ] All deliverables checked off in the roadmap
- [ ] All acceptance criteria met
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt`
      checks none of the 1,748 card defs and still exits 0; the script is the only thing
      that checks them. `cargo test --all` runs it too, via `core card_defs_fmt`.)
- [ ] Performance benchmarks run (if applicable to this milestone)
- [ ] Update "Current State" section of this file
- [ ] Update "Active Milestone" to the next milestone
- [ ] Check off completed deliverables in `docs/mtg-engine-roadmap.md`
- [ ] Update relevant memory topic files (`memory/gotchas-rules.md`, `memory/gotchas-infra.md`, `memory/conventions.md`, `memory/decisions.md`) with new learnings
- [ ] Review all new/changed files and update `docs/mtg-engine-milestone-reviews.md`:
  - Add file inventory with line counts
  - List CR sections implemented
  - Record findings (bugs, enforcement gaps, test gaps) with severity and issue IDs
  - Place deferred issues in the correct future milestone stub
  - Update the cross-milestone issue index and statistics
- [ ] Commit: `M<N>: milestone complete — <summary>`
- [ ] **Code review → fix phase** (if any HIGH or MEDIUM findings):
  - Run the `milestone-reviewer` agent (Opus) — writes findings to `docs/mtg-engine-milestone-reviews.md`
    and creates `memory/m<N>-fix-session-plan.md` grouping issues into sessions of 5-8 fixes each
  - Work through fix sessions with the `fix-session-runner` agent (Sonnet):
    reads `memory/m<N>-fix-session-plan.md` → applies fixes → `cargo test --all` → `cargo clippy -- -D warnings` → closes issues in reviews doc → commit
  - When all sessions complete, update "Current State" and advance to the next milestone
  - LOW-only findings do not require a fix phase; collect them in the reviews doc and address opportunistically

---

# Scutemob MTG Engine — ESM-Managed Project

This project is managed by ESM (External State Machine). Use the `esm` CLI and slash commands to interact with it.

## Quick Start

Use these slash commands to manage your ESM session:

- **`/start`** — Begin a session. Bootstraps context from ESM, starts session tracking, orients you.
- **`/dispatch <title>`** — **Primary workflow.** Create a task, worktree, and auto-launch a worker in a kitty pane. Use this for all implementation work.
- **`/status`** — Quick snapshot of tasks, sessions, and fleet-wide context.
- **`/collect [task_id]`** — Collect a finished worker's work: merge worktree to main, clean up.
- **`/task <title>`** — Create a task and work on it yourself (for small, self-assigned work only).
- **`/done [task_id]`** — Complete a self-assigned task: transition to done, merge branch to main.
- **`/spawn <title>`** — Like /dispatch, but you launch the worker manually.
- **`/eot`** — End-of-turn / end-of-session: ESM close + workstream-state rotation + memory routing. **Use this instead of `/end`** for scutemob — `/end` still works but skips the project-specific bookkeeping.

**Every session must begin with `/start`** (or manually running `esm project bootstrap scutemob` + `esm session start`).

## Worker Detection

If `.esm/worker.md` exists in the working directory, **you are a worker agent**. Read it
immediately and follow its task/acceptance criteria. The rest of this CLAUDE.md still applies.

## Workflow Rules

1. **Bootstrap first**: `/start` (or `esm project bootstrap scutemob && esm session start --project scutemob --agent primary`).
2. **An `in_progress` task must exist before writing code.** Lifecycle: `backlog → in_progress → in_review → done` (or `blocked` from either active state).
3. **Branch protocol**: feature branch per task; attest `working_branch=<full-name>` on transition; `/done` (self-assigned) or `/collect` (dispatched) merges to main.
4. **Tests are mandatory.** Write alongside implementation. Must pass before `in_review`.
5. **Acceptance criteria**: `esm task satisfy <task_id> <criterion_id> --by <agent>` for each before signaling ready.
6. **Task comments are short status lines** — `Completed: X. Next: Y.` / `Blocked: X. Tried: Y.` / `Decision: X. Reason: Y.` Detailed design notes belong in `docs/` or `memory/`, not comments.
7. **Dispatch, don't implement.** Coordinator creates tasks and dispatches workers via `/dispatch` for PB / ability / card-authoring work. Only implement inline for trivial fixes (<10 lines) or when explicitly told.

ESM CLI reference: `esm --help` or `esm <command> --help`. Sessions without a heartbeat for 10 minutes are auto-ended.

## Required Attestations

When transitioning to `in_progress`:
- `branch_exists`: "true"
- `acceptance_criteria_defined`: "true"
- `working_branch`: "<branch-name>"

When transitioning to `in_review`:
- `tests_passing`: "true"
- `implementation_complete`: "true"

When transitioning to `done`:
- `review_complete`: "true"

When transitioning to `blocked`:
- `blocked_reason`: describe what you need before you can continue

Unblocking requires admin approval — you cannot unblock yourself.

## Advisory Mode

ESM runs in **advisory mode** by default. The hook will warn you about scope violations and missing tasks, but won't block your work. Warnings appear in stderr — pay attention to them.

If this project uses **blocking mode**, scope violations will be denied. Check the project's `enforcement_mode` setting.

## Documentation Management

If `.claude/docs.yaml` exists, this project uses ESM documentation management.
Managed docs have a `<!-- last_updated: YYYY-MM-DD -->` comment that tracks freshness.

- **`/docs status`** — Quick health overview of all managed docs
- **`/docs check`** — Audit docs for drift (checks triggers against git history)
- **`/docs init`** — Interactive setup: scan existing docs, detect features, scaffold new ones

When you update a managed doc, always update the `<!-- last_updated: YYYY-MM-DD -->`
comment to today's date. Only update it for substantive changes — not typo fixes.

The `/done` and `/eot` skills automatically check for stale docs based on which
files you changed. Follow their recommendations or dismiss with a reason.

## Project Info

- **ESM Project ID**: `scutemob`
- **Agent ID**: `primary`
- **ESM Server**: `http://tower:8765`
