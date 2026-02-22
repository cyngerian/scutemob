# Conventions — Last verified: M7

## Rust Style

- **Edition**: 2021
- **Formatting**: `rustfmt` default settings. Run `cargo fmt` before every commit.
- **Linting**: `cargo clippy -- -D warnings`. No warnings allowed in CI.
- **Error handling**: `thiserror` for library errors, `anyhow` in binaries/tools only.
  Engine crate uses typed errors — never `unwrap()` or `expect()` in engine logic. Tests
  may use `unwrap()`.
- **Naming**: Types `PascalCase`, functions/methods `snake_case`, constants
  `SCREAMING_SNAKE_CASE`, modules `snake_case`.

## Comprehensive Rules Citation Format

Every rules implementation MUST cite the CR section it implements:

```rust
/// Implements CR 704.5f: "If a creature has toughness 0 or less, it's put into
/// its owner's graveyard."
fn check_zero_toughness(state: &GameState) -> Vec<GameEvent> { ... }
```

For tests, cite the rule AND the source of the test case:

```rust
#[test]
/// CR 704.5f — creature with 0 toughness dies as SBA
/// Source: CR example under 704.5f
fn test_704_5f_zero_toughness_creature_dies() { ... }

#[test]
/// CR 613.10 — Humility + Opalescence interaction
/// Source: CR example under 613.10, confirmed by Forge engine
fn test_613_10_humility_opalescence() { ... }
```

## Testing Conventions

- **Test location**: `crates/engine/tests/`, not inline `#[cfg(test)]` modules. Black-box
  testing against the public API only.
- **GameStateBuilder**: Always use the builder. Never manually construct `GameState` structs
  — the builder ensures invariants.
- **One assertion focus per test**: single behavior per test; multiple related assertions are
  fine, but the test name should describe the specific behavior.
- **Test naming**: `test_<system>_<scenario>_<expected_behavior>`
  - Good: `test_sba_creature_zero_toughness_goes_to_graveyard`
  - Good: `test_priority_all_four_players_pass_stack_resolves`
  - Bad: `test_combat` (too vague), `test_1` (meaningless)
- **Golden test format**: JSON files in `test-data/golden-games/`. Schema in architecture
  doc §6.4.
- **Property tests**: Use `proptest` crate. Define invariants in `tests/properties/`.

## Commit Conventions

- **Format**: `M<number>: <short description>` (e.g., `M1: implement GameState struct`)
- **PR scope**: One logical change per PR.
- **Tests required**: Every PR touching engine logic must include or update tests.
- **Benchmark check**: If PR touches state cloning, layer calculation, or SBA checks, run
  benchmarks and note any regression.

## Dependencies Policy

- **Engine crate**: `im`, `serde`, `thiserror`. No async runtime, no IO, no network, no UI.
- **Network crate**: `tokio`, `tokio-tungstenite` or `axum`, `serde`, `rmp-serde`.
- **Card-db crate**: `rusqlite`, `serde`.
- **Tauri app**: `tauri`, `serde`, frontend deps.

Engine crate must NEVER depend on network, card-db, or tauri-app crates. Information flows
inward only: app depends on network, network depends on engine. Never the reverse.
