# Design Decisions — Last verified: Engine Core Complete checkpoint (2026-02-23)

| Date | Decision | Rationale |
|------|----------|-----------|
| (project start) | Rust for engine, Tauri for app | Performance for layer calculations; Tauri gives native Rust backend + web UI without Electron overhead |
| (project start) | `im-rs` for immutable state | Structural sharing makes state snapshots O(1); enables free undo/replay; fits Rust ownership model |
| (project start) | Command/Event model | Single pattern for networking, replay, testing, and undo; enforces determinism |
| (project start) | Authoritative host (not P2P) | Hidden information requires a trusted authority; simpler than consensus protocols |
| (project start) | SQLite for card data | Structured queries for card lookup; embedded DB ships with app; no external server needed |
| (project start) | Separate engine/network/UI crates | Engine testable without IO; prevents coupling; allows future WASM compilation of engine alone |
| 2026-02-21 | ~~Distributed verification replaces authoritative host~~ — **superseded 2026-02-23** | Superseded: P2P mesh + Mental Poker deferred as a future upgrade path |
| 2026-02-21 | ~~Three-tier network security (hashing → distributed → Mental Poker)~~ — **superseded 2026-02-23** | Superseded: centralized server eliminates need for Tiers 2-3 for trusted playgroups |
| 2026-02-23 | Centralized WebSocket server for M10 (P2P deferred as future upgrade) | One player with bad internet stalls the whole table in P2P; Mental Poker adds significant complexity for no benefit in a trusted playgroup; centralized server is trivially cheap (~$5-10/mo VPS), simpler to implement, normalises timing, solves reconnection cleanly. P2P + Mental Poker preserved in `docs/mtg-engine-network-security.md` as a documented upgrade path. |
| 2026-02-21 | Deterministic state hashing from M3 onward | Catching non-determinism during engine development is dramatically cheaper than discovering it during M10 networking |
| 2026-02-21 | M4 legendary rule auto-keeps newest permanent (highest ObjectId) | Real player choice requires a Command that doesn't exist until M7; auto-newest is deterministic, testable, matches common play |
| 2026-02-21 | Game script generation deferred to M7; schema defined in M5 | Scripts can't run without the replay harness (M7); schema defined early so it compiles and evolves |
| 2026-02-22 | 6-player test coverage and benchmarks tracked as M9 deliverables | Engine is N-player by design but only tested with 1/2/4 players; 6-player Commander is common in casual play |
| 2026-02-21 | Rewind, pause, and manual mode are network/UI features, not engine features | im-rs structural sharing makes state history free; engine only needs `reveals_hidden_info()` on GameEvent (M9); secret info protection is honour-system |
| 2026-02-21 | SBA check at all four priority-grant sites | CR 704.3: SBAs fire "whenever any player would receive priority" — enter_step, resolve_top_of_stack, fizzle, counter |
| 2026-02-21 | Layer 1 (Copy) and Layer 2 (Control) stubbed in M5 | Copy requires CR 707 copiable-values logic (needs M7 card definitions); control changes live on `GameObject.controller`, not `Characteristics` |
| 2026-02-21 | `SetTypeLine` depends on `AddSubtypes`/`AddCardTypes` in dependency detection | Blood Moon + Urborg fix: set always follows add regardless of timestamp (CR 613.8) |
| 2026-02-22 | `CardDefinition` uses `impl Default` (not `#[derive(Default)]`) | `CardId` doesn't implement `Default`; manual impl avoids adding Default to state types |
| 2026-02-22 | Games cannot start with any unimplemented card | Graceful degradation corrupts state history that rewind/replay depends on; unimplemented cards blocked at deck-build time |
| 2026-02-22 | Card definition pipeline is scripted-first, LLM-assisted second | Scryfall provides structured mana cost, P/T, types, keywords; pattern library handles ~70-80% deterministically; no LLM at game runtime |
| 2026-02-22 | `enrich_spec_from_def` populates ObjectSpec from definitions in scripts | `ObjectSpec::card()` creates naked objects; enrichment ensures scripts work without bespoke per-card setup |
| 2026-02-22 | M9.5 Game State Stepper: web-based (axum + Svelte), placed after engine core | Visual validation before networking; Svelte components reused in M11 Tauri app (props-based, data source is the only difference) |
| 2026-02-22 | `HasCardId(CardId)` filter for commander replacement scope | ObjectId changes on zone change (CR 400.7) but CardId persists; replacement effects scoped to specific commanders need CardId matching |
| 2026-02-22 | ~~Two replacement effects per commander (graveyard + exile)~~ — **superseded by M9** | M9 changed graveyard/exile redirects to SBAs (CR 903.9a correct model). See row below. |
| 2026-02-23 | Commander graveyard/exile redirect is SBA (CR 903.9a); hand/library is replacement (CR 903.9b) | CR 903.9a says players "may put it into the command zone" as an SBA; CR 903.9b explicitly says "instead" (replacement). Mixing models caused incorrect interaction ordering with Rest in Peace. |
| 2026-02-22 | Self-ETB replacements from card definitions applied inline, not registered in state | Registering would create a global effect for all permanents; per-instance ETB (e.g., Dimir Guildgate) is applied at the ETB site by looking up card_id → CardDefinition → `AbilityDefinition::Replacement { is_self: true }` |
| 2026-02-22 | `apply_self_etb_from_definition` is public in `replacement.rs`; called from both `resolution.rs` and `lands.rs` | Both permanent spells and land plays are ETB sites; shared public function avoids duplication and ensures consistent CR 614.15 ordering |
