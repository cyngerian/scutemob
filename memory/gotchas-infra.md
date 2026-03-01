# Infra & Testing Gotchas — Last verified: M9.5 + Batch 1 (2026-03-01)

## Rust / im-rs Gotchas

- **`im-rs` HashMap iteration is not deterministic** across program runs unless using a
  fixed hasher. For deterministic replay, use `im::OrdMap` or sort before iterating.
- **Recursive enums need `Box`** for the recursive variant. `Effect` uses `Box<Effect>`
  inside `Sequence`/`Conditional`.
- **`im` serde support**: `im = { version = "15", features = ["serde"] }` in Cargo.toml.
- **`im::OrdMap` has no `iter_mut`.** Use `iter()` and collect into a new map.
- **`im::Vector<T>` size**: it's a tree structure, not `len() * size_of::<T>`.
- **`HashInto`: when adding new fields to `GameState`, `PlayerState`, `GameObject`, etc.,
  update `state/hash.rs` manually.** Fields not added to the hasher cause non-determinism
  that only shows up in distributed verification.

## Builder Gotchas

- **`CardDefinition` struct literals need `..Default::default()`** for non-creature cards
  after `power`/`toughness` fields were added. Bulk-fix with a script, but manually fix
  `Beast Within`, `Generous Gift`, `Swan Song` (contain nested `TokenSpec { power, toughness }`).
- **`CardRegistry::new()` returns `Arc<CardRegistry>`** — do NOT wrap in `Arc::new()` again.
- **`if let Some(cid) = card_id` moves `card_id`** — use `.clone()` first (`if let Some(cid) = card_id.clone()`) when you need `card_id` again after the pattern. Applies anywhere you match an `Option<T>` where T is not `Copy`.
- **Variables defined inside `{ }` blocks are unavailable after the block.** In `resolution.rs`, `registry` was defined inside the spell-effect block and needed after it for ETB replacements. Fix: define `let registry = ...` before the inner block.
- **`GameStateBuilder::build()` returns `Result`** — must unwrap with `?` or `.unwrap()`.
- **`ObjectSpec::card()` creates naked objects** — no card types, no mana abilities, no
  keywords, no P/T. Always call `enrich_spec_from_def()` before using in scripts. Without
  it: PlayLand fails ("not a land"), TapForMana fails (no ability at index 0), instant-speed
  casts fail for non-active players, permanents go to graveyard instead of battlefield.
- **`EffectAmount::PowerOf(target)` returns 0 if `target.power == None`.** Creatures built
  with `ObjectSpec::card()` have `power: None`; `enrich_spec_from_def` must propagate
  `def.power`/`def.toughness` to fix spells like Swords to Plowshares.

## Layer System Gotchas

- **Gained abilities need their own timestamps, separate from the permanent's timestamp.**
  When a chapter ability (or any "gains [ability]" effect) resolves, the resulting continuous
  effect must be registered with the timestamp of *that resolution*, not the timestamp of the
  permanent. If gained abilities inherit the permanent's timestamp, the Blood Moon + Urza's
  Saga entry-order behavior cannot be resolved correctly: Blood Moon entered after chapters
  resolved should override the gained abilities (later timestamp wins in Layer 6), but Blood
  Moon entered before should not (chapter gains have the later timestamp). Per-permanent
  timestamp storage breaks this entirely.

## ETB Site Gotchas (M9.4)

- **Two ETB sites exist: `resolution.rs` and `lands.rs`.** Any new hook that fires on
  "permanent enters battlefield" must be added to BOTH. Currently: `apply_self_etb_from_definition`,
  `apply_global_etb_replacements`, `register_static_continuous_effects`,
  `fire_when_enters_triggered_effects`. Forgetting `lands.rs` means lands don't benefit.
- **`handle_play_land` must emit `PermanentEnteredBattlefield` (fixed in e0ab8b7).** `LandPlayed`
  is the land-play-count tracker event; `PermanentEnteredBattlefield` is what `check_triggers`
  listens to for ALL ETB triggered abilities (Hideaway, Landfall, etc.). Without it, any ETB
  trigger on a land silently never fires. Also: `Command::PlayLand` in `engine.rs` must call
  `check_triggers + flush_pending_triggers` — the same pattern as `CastSpell` and `ActivateAbility`.
- **`EffectFilter::AttachedCreature` resolves at characteristic-calc time** via `source.attached_to`.
  The equipment source must be on the battlefield with `attached_to` set; if unattached, filter
  matches nothing. Do NOT pass an `ObjectId` — the source reference is implicit.
- **`EffectFilter::DeclaredTarget { index }` is a placeholder** — it must be resolved to
  `SingleObject` at `ApplyContinuousEffect` execution time in `effects/mod.rs`. Storing it
  unresolved in `state.continuous_effects` is a bug (the layer loop treats it as non-matching).
- **`is_copy: bool` on `StackObject`** — spell copies skip the zone-move step in `resolution.rs`.
  Without this flag, copies try to move the source card (which is on the battlefield/graveyard,
  not the stack), causing a panic or incorrect state.
- **`loop_detection_hashes` is NOT part of the public state hash.** It's bookkeeping state, not
  game state. Hashing it would cause distributed peers to disagree on the public hash mid-game.
- **`cards_drawn_this_turn` and `spells_cast_this_turn` on `PlayerState`** — both reset in
  `reset_turn_state`. If you add a new per-turn counter, add it to `reset_turn_state` too.

## Targeting API Gotchas

- **`TargetSpell` targets use `ObjectId` from the stack, not the card's pre-cast `ObjectId`.**
  After `CastSpell`, the card moves to the stack as a new object with a new ID. Target
  filters must resolve against the stack object ID.
- **`TargetFilter` uses `colors` for required colors, `exclude_colors` for forbidden colors.**
  Using the wrong field silently passes all targets or rejects all targets.
- **`CardEffectTarget` is the re-exported name** for `cards::EffectTarget` in `lib.rs`. Tests must
  use `mtg_engine::CardEffectTarget`, not `mtg_engine::EffectTarget` (which doesn't exist at root).

## Command Handler Pattern Gotchas

- **Every `Command` handler that can produce triggers must call `check_triggers()` +
  `flush_pending_triggers()` after its action.** `CastSpell` does this; `ActivateAbility`
  was missing it until M9.5 Ward work — activated abilities could not fire triggers at all.
  When adding a new command variant, verify it ends with the trigger flush pair, or triggers
  on the resulting state will silently never fire.

## Activated Ability Harness Gotchas (M9.5+)

- **`ability_index` is 0-indexed into non-mana `activated_abilities` only.** Mana abilities
  (Cost::Tap + Effect::AddMana OR Effect::AddManaAnyColor) are filtered out by `enrich_spec_from_def`.
  Mind Stone's `{1},{T},Sacrifice: Draw a card` lands at `activated_abilities[0]`. Scripts use `ability_index: 0`.
  **Note**: the filter must exclude BOTH `Effect::AddMana` and `Effect::AddManaAnyColor` — if only
  `AddMana` is excluded, cards like Commander's Sphere (which has `{T}: AddManaAnyColor`) get
  that tap ability indexed as 0, pushing the sacrifice ability to index 1 (wrong).
- **Sacrifice-as-cost: source leaves battlefield at activation time (CR 602.2c).** The effect is
  captured as `embedded_effect: Option<Box<Effect>>` before any costs are paid. Resolution uses
  `embedded_effect.as_deref().cloned()` because the source may no longer be in `state.objects`.
  After activation, assert source in graveyard; do NOT assert `zones.battlefield` still has it.
- **After `move_object_to_zone`, the original `ObjectId` is dead (CR 400.7).** Tests that verify
  the destination must search by name: `state.objects.values().any(|o| o.characteristics.name == "X"
  && matches!(o.zone, ZoneId::Graveyard(_)))`. Using the old `ObjectId` returns `None`.
- **`large_enum_variant` clippy error**: Adding `Option<Effect>` to a stack object variant hits
  clippy's size threshold. Fix: box it (`Option<Box<Effect>>`). Access with `.as_deref().cloned()`.
- **DrawCards on empty library is silently a no-op.** If a sacrifice-draw script asserts
  `zones.hand.p1.count: 1` after resolution, the library must have at least 1 card in initial_state.

## Agent Workflow Gotchas (ability pipeline)

- **Keyword actions (Surveil, Scry, etc.) are Effects, NOT `KeywordAbility` enum variants.**
  They produce a game event and optional trigger, but the "do N things and optionally X" logic
  lives in `Effect::Surveil { player, count }` / `Effect::Scry { player, count }`. Adding
  them as `KeywordAbility` variants would not give the ability pipeline (builder.rs triggers)
  a mechanism to encode the count parameter. Card definitions use `AbilityDefinition::Activated`
  or `AbilityDefinition::Triggered` that calls the Effect directly.
- **CDA (Characteristic-Defining Abilities, CR 604.3) must be inlined in `calculate_characteristics`
  at Layer 4**, not registered as `ContinuousEffect`. CDAs like Changeling ("is every creature
  type") apply in ALL zones. If modeled as a `ContinuousEffect`, they only apply on the
  battlefield. Inline check: `if chars.keywords.contains(Changeling) { chars.sub_types = ALL_CREATURE_TYPES.clone(); }`.
  This also automatically handles Humility: Layer 6 removes the keyword → Layer 4 already ran,
  subtypes stay. But if Humility was in effect first (no keyword), Layer 4 check skips. Correct.
- **`card-definition-author` agent silent exit**: The agent sometimes returns only an `agentId`
  with no content (no tool calls, no output). Resume with the same `agentId` — the second
  invocation usually also exits silently, but the card was often written on the first run.
  Verify by reading `definitions.rs` directly and running `cargo test` to confirm compilation.
  Do NOT spawn a new agent without first verifying the card wasn't already inserted.
- **`card-definition-author` adds TODO comments for freshly-implemented keywords.** The agent's
  knowledge cutoff means it doesn't know about `KeywordAbility` variants added earlier in the
  same pipeline run. It will write `// TODO: KeywordAbility::Delve not yet implemented` even
  when the keyword IS implemented. Always verify the generated definition includes the keyword
  ability variant (`AbilityDefinition::Keyword(KeywordAbility::X)`) and delete any stale TODO
  comments before committing.
- **`game-script-generator` stale binary validation**: The generator validates scripts using the
  running replay-viewer binary, not the compiled library. If card definitions were added after
  the binary was last built, the binary won't find them and validation will fail with
  `InvalidCommand`. Always validate scripts via `cargo test -p mtg-engine --test run_all_scripts`
  which uses the library directly. Approve the script if library tests pass.
- **`game-script-generator` SCRIPT_FILTER**: When the HTTP server is DOWN, the agent uses
  `SCRIPT_FILTER=<script_name_without_ext> ~/.cargo/bin/cargo test --test run_all_scripts -- --nocapture`
  (run from workspace root, e.g. `SCRIPT_FILTER=103 ~/.cargo/bin/cargo test -p mtg-engine --test run_all_scripts`).
  This runs ONLY the named script. Do NOT use `cargo test --test script_replay` — that only runs 4 unit tests,
  not the JSON scripts. Do NOT start or build the replay-viewer HTTP server — it causes OOM
  kills (SIGKILL/137) from the Sonnet agent context.
- **`SCRIPT_FILTER=X matched 0 scripts` means serde parse failure**, not a filter miss. The harness
  only runs scripts with `review_status: Approved`. If a script parses but is `pending_review`, SCRIPT_FILTER
  still skips it. If it's Approved but matches 0, `serde_json::from_str::<GameScript>` is silently
  failing — check JSON for schema violations. Common culprit fixed in e0ab8b7: `Dispute.step_index`
  and `action_index` are now `Option<usize>` — null JSON values for script-level disputes caused
  silent parse failure for any script with a `disputes` entry.
- **Upkeep triggers don't auto-fire in the harness.** The harness initializes a raw game state
  snapshot — it never processes a phase transition. Triggers that fire "at the beginning of upkeep"
  (e.g., Suspend's SuspendCounterTrigger) only appear if the engine advances from a previous phase.
  Script workarounds: (a) start with the trigger already on the stack, (b) test via the `suspend_card`
  special action which does fire synchronously, (c) cover the trigger lifecycle via unit tests.
  Same limitation applies to any turn-based trigger (end step, draw step, etc.).
- **Aura attachment order in `resolution.rs`**: `set attached_to/attachments` MUST happen
  BEFORE `register_static_continuous_effects`. If continuous effects register before attachment,
  `EffectFilter::AttachedCreature` finds no target and the Aura's static effects never apply.
- **`ctx.source` is stale after `MoveZone` moves the source object.** `move_object_to_zone`
  creates a new `ObjectId` for the destination object (CR 400.7). Any subsequent effect in a
  `Sequence` that references `EffectTarget::Source` will fail silently unless `ctx.source` is
  updated. Fix: after `MoveZone` resolves for a `Source` target, set `ctx.source = new_id`.
  Applies to persist/undying/blink-style effects where a `Sequence([MoveZone, AddCounter])`
  or similar is used. See `effects/mod.rs:762-767`.
- **Last-known-information for die triggers: capture counters BEFORE `move_object_to_zone`.**
  After the zone move, the old `ObjectId` is dead and `counters` are reset to `OrdMap::new()`.
  For persist/undying-style intervening-if checks, capture `pre_death_counters` from the live
  object before moving, and carry them through the `CreatureDied` event. All 8 `CreatureDied`
  emission sites (sba.rs, abilities.rs, effects/mod.rs, replacement.rs) must do this capture.
  See `rules/events.rs:249` and the `InterveningIf::SourceHadNoCounterOfType` pattern.
- **`TimingRestriction` import in `definitions.rs`**: When adding `AbilityDefinition::Activated`
  with `timing_restriction: Some(TimingRestriction::SorcerySpeed)`, `TimingRestriction` must be
  added to the `super::card_definition` import. The card-definition-author agent may omit this.
  Compile error: `use of undeclared type TimingRestriction`.

## Script Harness Gotchas

- **New harness action types added for abilities**: `cycle_card` (CycleCard command, finds card
  in hand, pays mana), `choose_dredge` (ChooseDredge command, finds named card in player's
  graveyard), `cast_spell_flashback` (CastSpell with flashback cost, card must be in graveyard),
  `declare_attackers` (DeclareAttackers command, `attackers: [{card, target_player}]` array),
  `declare_blockers` (DeclareBlockers command, `blockers: [{card, blocking}]` array),
  `crew_vehicle` (CrewVehicle command, `vehicle`, `crew_creatures: [name, ...]` array),
  `improvise` (CastSpell with `improvise_names: [name, ...]`; maps to `improvise_artifacts`).
  When the generator reports a harness gap for a new action type, add the arm to
  `translate_player_action()` in `crates/engine/src/testing/replay_harness.rs` and revalidate.
- **Convoke scripts: duplicate creature names resolve to the same ObjectId** — rejected as
  "duplicate ObjectId in convoke_creatures." Use distinct card names (e.g., Llanowar Elves,
  Elvish Mystic, Birds of Paradise) rather than three identical "Llanowar Elves" entries.
- **`priority_player` field in action scripts must be set correctly** or the harness routes
  the action to the wrong player.
- **Mana abilities do NOT reset `players_passed`** (CR 605.3a — mana abilities don't use
  the stack, no priority). Only `CastSpell` and similar stack actions reset `players_passed`.
- **`pay_cost` generic mana order: colorless → green → red → black → blue → white.** When
  writing game scripts with multiple sequential casts, provide enough mana of specific colors
  so that generic pips don't consume the color you need next. E.g., casting Rest in Peace
  {1W} then Lightning Bolt {R}: pool `{ white: 2, red: 2 }` works; `{ white: 3, red: 1 }`
  fails because the engine's generic pip eats the single red before Bolt is cast.
- **Commander registration in replay harness**: `build_initial_state` now reads
  `initial_state.players[*].commander` and calls `register_commander_zone_replacements`.
  If you add a new initial_state field that affects pre-game setup, add it to `build_initial_state`
  in `tests/script_replay.rs`.

## EOC Flag Pattern (Decayed, Myriad)

For "sacrifice/exile at end of combat" effects where the trigger is locked in at attack
declaration (even if the ability is later removed):
1. Add a `bool` flag to `GameObject` (e.g., `decayed_sacrifice_at_eoc`)
2. Set the flag in `handle_declare_attackers()` in `combat.rs` — ONLY here has mutable state
3. Check the flag in `end_combat()` in `turn_actions.rs` — collect flagged creatures, sacrifice
4. Reset the flag in BOTH `move_object_to_zone()` sites in `state/mod.rs`
5. Initialize to `false` in: `builder.rs`, `effects/mod.rs` (token creation), `resolution.rs`
6. Hash the new field in `hash.rs`

This is the same pattern as `myriad_exile_at_eoc`. See `game_object.rs:399-408` (Decayed).

## Turn Structure Gotchas

- **`advance_turn()` uses `turn.last_regular_active`, NOT `turn.active_player`.** When manually
  constructing a test state with a non-P1 active player, you must set BOTH `active_player` AND
  `last_regular_active`. If you only set `active_player = P3` but leave `last_regular_active = P1`,
  `advance_turn` will compute next-after-P1 → P2 instead of next-after-P3 → P4.
- **Cleanup discard (CR 514.1a) applies only to the active player**, not all players.
  If you need to test that a non-active player would discard, advance turns to make them active.

## Testing Gotchas

- **`GameStateBuilder::six_player()`** added in M9 alongside `four_player()`. 6-player tests
  cover priority rotation, combat with 5 defenders, APNAP ordering, turn advancement skipping
  eliminated players, concession mid-game — all in `crates/engine/tests/six_player.rs`.
- **`ObjectSpec::card + .with_types([Creature])` creates a creature with `toughness: None`.**
  SBAs 704.5f/g/h skip `None` toughness to avoid false positives. Use
  `ObjectSpec::creature(owner, name, power, toughness)` for creatures SBAs should affect.
- **Don't test implementation details.** Test observable behavior: "player B's life is 37"
  not "the stack has one item of type InstantSpell with damage field 3."
- **Randomness in tests**: use seeded RNG (`StdRng::seed_from_u64`) for deterministic
  library order.
- **Golden tests are fragile**: if you change Event format, all golden test files break.
  Version the golden test schema.
- **1-player `start_game` doesn't reach Cleanup.** `active_players().len() == 1` makes
  `is_game_over()` return true immediately. Tests verifying cleanup (e.g., `UntilEndOfTurn`
  expiry via the full turn cycle) must use 2+ players. Layer tests that only call
  `calculate_characteristics` can safely use 1 player.
- **Combat step turn-based actions fire when ENTERING the step, not exiting it.** Events
  from `FirstStrikeDamage` (damage dealt, creatures dying) appear in the `pass_all` that
  transitions INTO the step. The `pass_all` that exits the step produces events from
  entering the NEXT step. Tests for first-strike damage must capture the first `pass_all`,
  not the second.
- **CR 510.1c: last blocker gets ALL remaining power (no trample cap).** "Assign minimum
  lethal before moving to next blocker" only applies when subsequent blockers exist. The
  final blocker without trample absorbs all remaining attacker power.
- **`pass_all_four` resolves exactly ONE item from the stack per call.** If a test casts a
  permanent that itself triggers a doubled ETB (e.g., Panharmonicon entering puts 2 Watcher
  triggers on the stack), a single `pass_all_four` only drains 1 trigger. Use a drain loop:
  `while !state.stack_objects.is_empty() { let (s,_) = pass_all_four(state, [...]); state = s; }`
- **Sorcery-speed casts require empty stack (CR 307.1).** After a permanent resolves,
  any ETB triggers it generated must be fully resolved before casting the next sorcery-speed
  spell. If the permanent's own ETB doubles triggers, you need multiple drain calls.

## Replay Viewer / Axum Gotchas (M9.5+)

- **Axum 0.7 path params use `/:n` not `/{n}`** — `/{n}` is axum 0.8+ syntax. Using `/{n}` silently returns 404 for every request to that route.
- **`ZoneId::Battlefield` and `ZoneId::Exile` are shared zones** (no player ID). To filter by player on the battlefield, check `obj.controller == pid`. Contrast with `ZoneId::Hand(pid)` and `ZoneId::Graveyard(pid)` which are per-player.
- **`characteristics.name` is `String`, not `Option<String>`.** Use `.map(|o| o.characteristics.name.clone())` not `.filter_map(...)`. The view model's `PermanentView.name` is also `String`.
- **Axum 0.7 integration tests** need `tower = { version = "0.4", features = ["util"] }` + `http-body-util = "0.1"` as dev-deps. Pattern: `app.oneshot(req).await` then `resp.into_body().collect().await.unwrap().to_bytes()` to read body.
- **Replay viewer tests run from `tools/replay-viewer/`**, not workspace root. Script paths in tests must be `../../test-data/generated-scripts/...`.
- **`stack_resolve` script actions are informational only** — no command is sent to the engine. State is identical to the preceding priority-pass step. Real engine events for resolution appear in the priority-pass steps, not the stack_resolve step.
- **`pending_review` game scripts are unvalidated** — auto-generated scripts often misattribute triggers (ETB vs. death vs. activated) and omit interactive commands (e.g. `SearchLibrary` requires an explicit player command the generator doesn't emit). Use the replay viewer to validate before approving.
- **Svelte 5 keyed `{#each entries as e (e.id)}` crashes silently on duplicate keys.** The entire component's reactivity breaks — buttons stop responding, no error shown. Root cause is always a duplicate `metadata.id` across scripts. The `api.rs` `get_scripts` handler now deduplicates and logs a warning, but fix the source script first.
- **`metadata.id` must be unique across all scripts.** Copy-pasted scripts inherit the source's `id` — always update it. Pattern: `script_<subdir>_<NNN>` matching the filename number (e.g., `054_mind_stone...json` → `id: "script_stack_054"`).
- **Tokio worker thread stack overflow with trigger-heavy scripts**: The default tokio worker
  thread stack is 2 MB. In debug builds, the MTG rules engine's deep call chains during
  triggered ability resolution (prowess, ward, ETB cascades) can exceed this limit, causing
  `thread 'tokio-runtime-worker' has overflowed its stack` and aborting the process. Fix:
  replace `#[tokio::main]` in `main.rs` with a custom `tokio::runtime::Builder` that sets
  `.thread_stack_size(8 * 1024 * 1024)` to match the OS default for regular threads. This
  issue affects the HTTP harness only — `cargo test` uses regular threads (8 MB) and passes.
  The `api.rs` `post_run_script` and `post_load` handlers also use `spawn_blocking` to further
  insulate against deep call chains in async handlers.
