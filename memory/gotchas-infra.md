# Infra & Testing Gotchas — Last verified: M7

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
- **`GameStateBuilder::build()` returns `Result`** — must unwrap with `?` or `.unwrap()`.
- **`ObjectSpec::card()` creates naked objects** — no card types, no mana abilities, no
  keywords, no P/T. Always call `enrich_spec_from_def()` before using in scripts. Without
  it: PlayLand fails ("not a land"), TapForMana fails (no ability at index 0), instant-speed
  casts fail for non-active players, permanents go to graveyard instead of battlefield.
- **`EffectAmount::PowerOf(target)` returns 0 if `target.power == None`.** Creatures built
  with `ObjectSpec::card()` have `power: None`; `enrich_spec_from_def` must propagate
  `def.power`/`def.toughness` to fix spells like Swords to Plowshares.

## Targeting API Gotchas

- **`TargetSpell` targets use `ObjectId` from the stack, not the card's pre-cast `ObjectId`.**
  After `CastSpell`, the card moves to the stack as a new object with a new ID. Target
  filters must resolve against the stack object ID.
- **`TargetFilter` uses `colors` for required colors, `exclude_colors` for forbidden colors.**
  Using the wrong field silently passes all targets or rejects all targets.

## Script Harness Gotchas

- **`priority_player` field in action scripts must be set correctly** or the harness routes
  the action to the wrong player.
- **Mana abilities do NOT reset `players_passed`** (CR 605.3a — mana abilities don't use
  the stack, no priority). Only `CastSpell` and similar stack actions reset `players_passed`.

## Testing Gotchas

- **All existing tests use 1, 2, or 4 players.** 6-player scenarios untested. Priority
  rotation with 6 passes, APNAP with 6, and combat with 5 defenders need tests in M9.
  Add `GameStateBuilder::six_player()` alongside them.
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
