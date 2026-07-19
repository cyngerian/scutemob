# Engine Invariants & Machine-Enforced Gates

<!-- last_updated: 2026-07-18 -->

> **Standing invariant/machinery reference.** These bullets moved verbatim out of
> CLAUDE.md's "Current State" section on 2026-07-18 (DOC-1v2, `scutemob-125`) because
> they are permanent engineering constraints, not a rolling snapshot — they describe
> gates the build enforces and the reasoning behind them, and they change only when the
> corresponding gate changes. CLAUDE.md keeps a one-line pointer to each; the full text
> lives here.
>
> These complement the nine non-negotiable **Architecture Invariants** in CLAUDE.md
> (which state the rules); the entries below document how the build *enforces* those
> rules (the SR-remediation gates) and where each mechanism lives. Read the matching
> entry before touching the subsystem it guards.
>
> Live structural counts (corpus size = 1,798 card defs) are kept current here; numbers
> that appear inside a historical narrative of when a gate was built (e.g. SR-35's
> "1,380 of 1,748 defs were inert") are point-in-time record and are left as written.

---

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
  until you classify it**, and `crates/engine/tests/core/keyword_registry.rs`
  (`--test core keyword_registry::`) then checks the claim
  against the source tree: declared site sets must exactly equal a comment-stripped
  scan, so a keyword that loses its last dispatch site — or a `Marker` that gains one —
  fails the suite. Audit: `docs/sr-5-keyword-catchall-audit.md`. The same hazard on
  `AbilityDefinition` / `ZoneChangeAction` is not yet gated (`scutemob-67`).
- **Card defs compile in isolation from the engine (SR-6).** The workspace bottom is
  `crates/card-types` (`mtg-card-types`: the DSL — `cards/{card_definition,helpers,registry}.rs`
  — plus the 11 pure-data `state/` modules it needs). `crates/card-defs` (`mtg-card-defs`:
  1,798 def files + `build.rs` discovery) depends on **card-types only, never on the engine**;
  `crates/engine` depends on both and re-exports them, so every `crate::state::…` and
  `crate::cards::…` path inside the engine, and `mtg_engine::{all_cards, CardDefinition, …}`
  outside it, resolve exactly as before. **Touching an engine file leaves `mtg-card-defs`
  `Fresh`** (`cargo check -p mtg-engine -v`); touching `card-types` correctly rebuilds it.
  The arrow direction is the whole mechanism — putting defs *above* the engine (as the
  pipeline doc originally sketched) would recompile all 1,798 cards on every rules edit.
  Nothing in `card-types` may reference `GameState`. Keyword-registry sites (SR-5) are now
  **workspace-relative** paths and the scan spans both crates.
- **`PendingTrigger` is built through `PendingTrigger::blank` only (SR-7).** The 13
  per-keyword `Option` fields are gone; a trigger kind's payload lives in
  `data: Option<TriggerData>` (`card-types/src/state/stack.rs`), which
  `flush_pending_triggers` reads and threads into `StackObjectKind::KeywordTrigger`.
  `crates/engine/tests/core/pending_trigger_shape.rs` (`--test core pending_trigger_shape::`)
  pins the struct's 16-field set, requires every
  `PendingTrigger { .. }` literal to carry `..PendingTrigger::blank(source, controller, kind)`,
  and asserts each `TriggerData` variant still has a consumer in *both* `abilities.rs` and
  `resolution.rs` — **deleting a `resolution.rs` match arm compiles with zero errors** and
  would otherwise make the trigger a silent no-op. **New per-kind state goes in a
  `TriggerData` variant, never as a field on the struct** — a new field fails the suite.
  `HASH_SCHEMA_VERSION` was **37** when SR-7 landed; it has since advanced with every
  hash-affecting change (read the live `pub const` in `crates/engine/src/state/hash.rs`
  rather than quoting a number that drifts).
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
  re-quote them here — they move whenever the wire does), and `crates/engine/tests/core/protocol_schema.rs`
  (`--test core protocol_schema::`) recomputes it from source — so
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
