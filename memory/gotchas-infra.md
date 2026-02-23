# Infra & Testing Gotchas — Last verified: M9.4

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

## ETB Site Gotchas (M9.4)

- **Two ETB sites exist: `resolution.rs` and `lands.rs`.** Any new hook that fires on
  "permanent enters battlefield" must be added to BOTH. Currently: `apply_self_etb_from_definition`,
  `apply_global_etb_replacements`, `register_static_continuous_effects`,
  `fire_when_enters_triggered_effects`. Forgetting `lands.rs` means lands don't benefit.
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

## Script Harness Gotchas

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
