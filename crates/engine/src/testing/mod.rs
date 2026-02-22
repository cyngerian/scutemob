/// Test infrastructure for game script generation and replay.
///
/// The `script_schema` module defines the `GameScript` type — the contract between
/// script generation (Claude Code + MCP tools) and the replay harness (M7).
/// Scripts are stored in `test-data/generated-scripts/` and run via `cargo test`
/// once the replay harness is built in M7.
///
/// See `docs/mtg-engine-game-scripts.md` for the full generation + validation workflow.
pub mod script_schema;
