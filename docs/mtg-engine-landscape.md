# Open-Source MTG Rules Engines: A Source-Level Survey for scutemob

*Prepared 2026-09-05. Repos cloned at HEAD the same day (phase.rs last commit 2026-09-05 18:32 UTC; manabrew last commit 2026-09-05 15:03 UTC). All file paths below are relative to the repo root and were verified against the checkout.*

## 0. How to use this document

This is written for the scutemob agent, not for a human reader in a hurry. Its job is to give you enough architectural detail, concrete type signatures, and file paths that you can (a) form a defensible opinion about where scutemob sits relative to the field, and (b) know exactly which files to clone and read if you want to go deeper on any one point.

The comparison baseline is scutemob as you know it: a Rust rules engine of roughly 60K+ lines with 130K+ tests, currently in the card-authoring phase, targeting rules-enforced Commander for a small friend group that currently plays on Cockatrice.

Four projects matter. Two are legacy Java engines that define the ceiling of card coverage (XMage, Forge). Two are modern Rust engines that appeared in 2025–2026 and are the real peers (phase.rs, Manabrew). Section 6 is a set of specific questions to answer about scutemob against each axis.

To clone the two modern ones:

```bash
git clone --depth 1 https://github.com/phase-rs/phase.git
git clone --depth 1 https://github.com/witchesofthehill/manabrew.git
# manabrew's forge/ directory is a git submodule (Java Forge source); init it only if you need the reference:
# cd manabrew && git submodule update --init forge
```

---

## 1. Landscape at a glance

| | XMage | Forge | phase.rs | Manabrew | scutemob (baseline) |
|---|---|---|---|---|---|
| Language | Java 8 | Java | Rust (native + WASM) | Rust (native + WASM) + Java Forge interop | Rust |
| First commit era | ~2010 | ~2007 | 2026 (weeks-old per README) | 2025–26 | 2025–26 |
| Card definition strategy | One Java class per card, composed from Ability/Effect/Target objects | Text DSL (`A:SP$ DealDamage \| ValidTgts$ Any \| NumDmg$ 3`) interpreted at runtime | **Oracle text parsed by nom combinators into a typed AST**; per-card JSON overrides possible; optional Forge-script fallback | **Consumes Forge's DSL directly**; compiles common records into typed IR | Hand-authored per card (agent-generated) |
| Card count | 30,000+ unique, near-complete | ~30,000, near-complete | 34,300+ parsed; thousands still `Unimplemented` (fails closed) | Inherits Forge corpus; Rust engine "works for selected matchups" | Card-authoring phase |
| State model | Mutable OO | Mutable OO | Pure reducer `apply(&mut state, actor, action) -> ActionResult`; `im` persistent collections for hot zones | Mirrors Forge's mutable class graph (`Card`, `SpellAbility`, `MagicStack`, `ZoneStore`) | ? |
| Multiplayer / hidden info | Server-authoritative, all rules + hidden info enforced server-side | Host/client, port-forward, buggy >2 players | Axum WebSocket server, per-viewer state filtering, WebRTC P2P option, lobby-only relay mode | Relay + headless room host; can spawn a Java Forge JVM per game | ? |
| Commander | Yes (up to 10) | Yes but multiplayer network play is broken | Yes (`game/commander.rs`, CR 903 tax etc.) | Via Forge semantics | Target format |
| Tests | JUnit, card-level | JUnit | 31,024 `#[test]` functions; insta snapshots; scenario harness; coverage/semantic-audit binaries | 646 `#[test]`; **parity harness vs Java Forge is the primary test** | 130K+ |
| Agent-native? | No | No | Yes: `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, 30 skills in `.claude/skills/`, multi-agent concurrency rules | Yes: `CLAUDE.md`, `AGENTS.md`, `docs/agents/`, "AI assistance welcome for mechanical porting" | Yes |
| License | MIT | GPL-3 | MIT / Apache-2.0 | AGPL-3 (own code), GPL-3 (Forge-derived engine + card data) | — |

The two Rust projects made opposite bets on the hardest problem, which is card coverage:

- **phase.rs** bets that Oracle text is regular enough to parse into a typed AST with a real grammar, and that anything the grammar can't express should fail closed as `Effect::Unimplemented` rather than approximate. Coverage grows by extending the grammar to cover a *class* of cards, never by special-casing one.
- **Manabrew** bets that Forge's twenty years of hand-written card scripts are the asset, and that the right move is to reimplement Forge's *interpreter* in Rust, with Java Forge as a differential-testing oracle, so every existing script keeps working.

scutemob is a third bet: author cards directly against your own engine's primitives, with an agent doing the authoring. That is closer to XMage's model than to either Rust project. Section 6 is about whether that is the right bet.

---

## 2. phase.rs (phase-rs/phase)

### 2.1 Identity

- Repo: https://github.com/phase-rs/phase. Author "Matt", solo-plus-agents. Public alpha server at phase-rs.dev, preview at preview.phase-rs.dev, Tauri desktop releases, Discord.
- README framing: "A Rust-native MTG engine compiling to native and WASM, powering a Tauri desktop app, browser PWA, and WebSocket multiplayer. Implements comprehensive MTG rules using functional architecture — pure reducers, discriminated unions, and immutable state with structural sharing."
- Scale (engine crate only, excluding `snapshots/`): 536 `.rs` files, ~1.63M lines. That number includes very large generated-looking test files (`parser/oracle_effect/tests.rs` alone is 61K lines), so the *logic* footprint is smaller, but the `types/ability.rs` file is 35K lines on its own and `Effect` has 233 variants.
- Dual-licensed MIT/Apache-2.0. No Wizards assets bundled; card metadata from MTGJSON, images from Scryfall at runtime.

### 2.2 Workspace layout

```
crates/
  engine/         Core rules engine: types, game logic, parser, database   (the thing to study)
  engine-wasm/    wasm-bindgen + tsify bridge; thread-local RefCell<Option<GameState>>
  phase-ai/       AI opponent: legal_actions, eval, search, card_hints
  server-core/    Server-side game session mgmt (tokio); per-viewer state filtering
  phase-server/   Axum WebSocket server + lobby
  feed-scraper/   MTGGoldfish metagame scraper (standalone)
client/           React + TS + Tailwind v4 + Zustand + Framer Motion; EngineAdapter abstraction
lobby-worker/     Cloudflare-worker style lobby broker
supabase/         Auth/persistence for the hosted service
.claude/skills/   30 agent skills (see 2.9)
```

Dependency flow: `engine <- phase-ai <- engine-wasm / server-core <- phase-server`.

Key crates in `crates/engine/Cargo.toml`: `nom = "8.0"` (parser), `im = "15"` (persistent HAMT/RRB collections for hot `GameState` zones), `insta` (snapshot tests), `proptest` (optional), `rand 0.9`, `serde` with `rc` feature. Note the README says `rpds`; the actual dependency is `im`.

### 2.3 The card pipeline (the most important thing to understand)

phase.rs has **no per-card code**. A card becomes executable through this pipeline:

```
MTGJSON atomic data
  └─> crates/engine/src/database/mtgjson.rs, oracle_loader.rs
        └─> crates/engine/src/parser/oracle.rs  (dispatcher)
              ├─> parser/oracle_nom/*        shared nom 8 combinators (primitives, target, quantity,
              │                                duration, condition, filter, bridge, context)
              ├─> parser/oracle_effect/*     imperative clauses -> Effect AST (lower.rs, imperative.rs,
              │                                sequence.rs, subject.rs, conditions.rs)
              ├─> parser/oracle_trigger.rs   "When/Whenever/At" -> TriggerDefinition
              ├─> parser/oracle_static/*     "creatures you control get +1/+1" -> StaticDefinition
              ├─> parser/oracle_replacement.rs  "If ... would ... instead" -> ReplacementDefinition
              ├─> parser/oracle_cost.rs, oracle_keyword.rs, oracle_casting.rs, oracle_modal.rs,
              │     oracle_saga.rs, oracle_class.rs, oracle_level.rs, oracle_vote.rs, ...
              └─> parser/swallow_check.rs    guards against a parser "swallowing" text it didn't
                                               actually model (fail-closed enforcement)
        └─> types/ability.rs :: AbilityDefinition / Effect / TriggerDefinition / StaticDefinition
              └─> cargo export-cards  ->  client/public/card-data.json  (pre-built, consumed by WASM + server)
                    └─> database::CardDatabase::from_export(path) at runtime
```

A parsed card is a JSON object with `abilities`, `triggers`, `statics`, `replacements`, `extractedKeywords`. Here is the actual insta snapshot for a vanilla flier with an ETB draw (from `crates/engine/src/parser/snapshots/engine__parser__oracle__pipeline_snapshot_tests__pipeline_creature_with_keywords_and_trigger.snap`), abbreviated:

```json
{
  "abilities": [
    { "kind": "Spell",
      "effect": { "type": "Unimplemented", "name": "unknown", "description": "Flying" },
      "description": "Flying", ... }
  ],
  "triggers": [
    { "mode": "ChangesZone",
      "execute": {
        "kind": "Spell",
        "effect": { "type": "Draw", "count": { "type": "Fixed", "value": 1 }, "target": { "type": "Controller" } },
        ... },
      "valid_card": { "type": "SelfRef" },
      "origin": null, "destination": "Battlefield",
      "trigger_zones": ["Battlefield"],
      "description": "When ~ enters, draw a card.", ... }
  ],
  "statics": [], "replacements": [], "extractedKeywords": []
}
```

Two things to notice. First, the `Flying` line in this particular snapshot lands as `Unimplemented` because that snapshot test deliberately feeds a keyword through the effect path (keywords normally go through `extractedKeywords`/`oracle_keyword.rs`); it illustrates the fail-closed default. Second, the trigger vocabulary (`mode: ChangesZone`, `valid_card`, `origin`, `destination`, `execute`) is unmistakably Forge's trigger vocabulary (`Mode$ ChangesZone | ValidCard$ Card.Self | Destination$ Battlefield | Execute$ ...`). phase.rs borrowed Forge's *ontology* while rejecting its string-interpretation runtime.

**Fail-closed is a first-class design rule.** From `CLAUDE.md`: the single authority for "parser couldn't handle this" is `Effect::unimplemented(name, fragment)`; hand-constructing `Effect::Unimplemented { .. }` literals is gated. `parser/swallow_check.rs` (11K lines) exists to catch parsers that consumed text without producing a faithful model. The coverage tooling (`cargo coverage`, `cargo semantic-audit`, `cargo parser-gaps`) reports Unimplemented gaps per card; CI mode exits 1 on gaps in tracked sets. Query for gaps on a card:

```bash
jq '.["card name"] | {abilities: [.abilities[]? | select(.effect.type == "Unimplemented")], triggers: [.triggers[]? | select(.mode == "Unknown")]}' client/public/card-data.json
```

**Forge fallback bridge.** `crates/engine/src/database/forge/` (feature-gated, nothing GPL bundled) reads a user-supplied Forge checkout and "selectively replaces `Unimplemented` entries in Oracle-parsed cards. As the Oracle parser improves, cards naturally graduate away from Forge data." So the project has a pragmatic escape hatch to Forge's corpus while keeping its own parser as the strategic path. Files: `cost.rs, effect.rs, filter.rs, keyword.rs, loader.rs, replacement.rs, static_ab.rs, svar.rs, translate.rs, trigger.rs, types.rs`.

### 2.4 The type system

`crates/engine/src/types/ability.rs` (35,678 lines) is the heart. Core struct, verbatim (comments trimmed):

```rust
pub struct AbilityDefinition {
    pub kind: AbilityKind,
    pub effect: Box<Effect>,
    pub cost: Option<AbilityCost>,
    pub sub_ability: Option<Box<AbilityDefinition>>,
    /// CR 608.2c: Alternative branch executed when the condition on this ability is NOT met.
    pub else_ability: Option<Box<AbilityDefinition>>,
    pub duration: Option<Duration>,
    pub description: Option<String>,
    pub target_prompt: Option<String>,
    /// CR 602.5d: "Activate only as a sorcery." ... single authority for sorcery-speed timing.
    pub activation_restrictions: Vec<ActivationRestriction>,
    pub activation_mana_payment_restriction: Option<ActivationManaPaymentRestriction>,
    /// CR 602.2a: Who may begin to activate this ability.
    pub activator_filter: Option<PlayerFilter>,
    /// CR 602.1: Zone from which this ability can be activated.
    pub activation_zone: Option<Zone>,
    pub ability_tag: Option<AbilityTag>,
    pub condition: Option<AbilityCondition>,
    // ...
}
```

The `Effect` enum (`ability.rs:14129`, 3,555 lines, 233 variants, `#[serde(tag = "type")]`, `strum::IntoStaticStr`). Representative variant:

```rust
DealDamage {
    #[serde(default = "default_quantity_one")]
    amount: QuantityExpr,
    #[serde(default = "default_target_filter_any")]
    target: TargetFilter,
    /// CR 120.3: Override damage source. None = ability source (default).
    damage_source: Option<DamageSource>,
    /// CR 120.4a: Trailing excess-redirect rider ("Excess damage is dealt to
    /// that creature's controller instead").
    excess: Option<ExcessRecipient>,
},
```

The design discipline that keeps 233 variants from becoming 2,000 is spelled out in `CLAUDE.md` under "Parameterize, don't proliferate": before adding a sibling variant, check whether it is a leaf-level parameterization of an existing variant's axis (scope, target, aggregate, condition). `LifeTotal { player: PlayerScope }` instead of `LifeTotal + OpponentLifeTotal + TargetLifeTotal`. A "sibling-cluster smell" (three or more variants sharing a name root) is a refactor trigger. There is a **categorical boundary rule**: the parameterization axis must lie within one CR section (life is CR 119, P/T is CR 208/209; don't unify them at the leaf level; unify at `TargetFilter` or at the handler, e.g. `Effect::DealDamage` per CR 120 handles all damage subjects). An auto-generated `data/engine-inventory.json` (`cargo engine-inventory`) is the canonical index of engine surface so agents grep it before proposing variants.

Other load-bearing types, all in `types/`:

- `TargetFilter` — recursive: `Typed(TypedFilter) | And { filters } | Or { filters } | Not { filter } | ...`, where `TypedFilter` has `controller: Option<ControllerRef>` and `properties: Vec<FilterProp>` (`FilterProp::Owned { controller }`, `FilterProp::InZone { zone }`, etc.). Evaluated at runtime by `game/filter.rs` (15.8K lines).
- `QuantityExpr` = `Fixed(i32) | Ref(QuantityRef)`; `QuantityRef` is a reference to a dynamic game value (`HandSize`, `LifeTotal { player }`, `CountersOnTarget`, `ObjectCount { filter, zone }`). Resolved by `game/quantity.rs`. The layering rule ("`QuantityRef` must not contain `Fixed`; wrap in `QuantityExpr`") is explicit in `CLAUDE.md`.
- `GameAction` (`types/actions.rs:150`): `PassPriority`, `PlayLand { object_id, card_id }`, `CastSpell { object_id, card_id, targets, payment_mode }`, `ChooseMeldPair`, `ChooseEntryAttackTarget`, `Foretell`, ... plus a `Debug(DebugAction)` variant with explicit permission gating.
- `WaitingFor` (`types/game_state.rs:12108`): the engine's "what decision is pending" state machine — `Priority { player }`, `ModeChoice`, `ChooseXValue`, `MeldPairChoice`, `MeldAttackTargetChoice`, `ResolveAllConsent { epoch, representative }`, and many more. This is how the pure reducer models interactive resolution: instead of callbacks, it returns a `WaitingFor` and the next `GameAction` answers it.
- `GameState` is declared through a `declare_game_state!` macro (`types/game_state.rs:17151`) that generates both the struct and a `RawGameStateFields` deserializer so fields can be added with serde defaults without breaking saved games/replays. Fields begin: `turn_number, active_player, phase, players: Vec<Player>, priority_player, turn_decision_controller (CR 723 control effects), ...`, plus `commander_cast_count`, `commander_cast_owners` (CR 903.8).

### 2.5 The reducer and the game loop

`crates/engine/src/game/engine.rs:882`:

```rust
pub fn apply(
    state: &mut GameState,
    actor: PlayerId,
    action: GameAction,
) -> Result<ActionResult, EngineError> {
    apply_action_boundary(state, actor, action, PublicFinalizeMode::Immediate)
}

pub fn apply_with_rejection(
    state: &mut GameState,
    actor: PlayerId,
    action: GameAction,
) -> Result<ActionResult, ActionRejection> { /* maps EngineError -> viewer-safe ActionRejection */ }
```

with

```rust
pub struct ActionResult {
    pub events: Vec<GameEvent>,
    pub waiting_for: WaitingFor,
    pub log_entries: Vec<GameLogEntry>,
}
```

Note the `actor: PlayerId` parameter and its doc comment: adapters that forward actions from a remote peer "must tag the action with the PlayerId associated with the *connection*, not a value copied out of the wire frame. Otherwise a malicious peer can trivially spoof another player's identity." Engine-internal simulation uses `apply_as_current`. There is also `apply_for_simulation` for AI search. `apply_with_rejection` routes errors through `game/visibility.rs::filter_action_rejection_for_viewer` so an error message can't leak hidden info.

The `im` persistent collections make `GameState::clone()` O(log n) structural sharing, which is what makes AI search and replay cheap. `CLAUDE.md` warns `im::Vector::truncate(n)` panics if `n > len`.

Game logic is one module per concern in `crates/engine/src/game/` (`ls` it; ~200 files). The ones a comparison should read:

| Module | What |
|---|---|
| `engine.rs` (23.8K) | `apply`, action boundary, phase auto-advance, debug permissions |
| `priority.rs`, `turns.rs`, `match_flow.rs` | CR 117 priority, CR 500 turn structure |
| `stack.rs` (15K), `engine_stack.rs`, `engine_resolve_batch.rs` | CR 608 resolution; "Resolve All" consent protocol for multiplayer |
| `casting.rs` (22.9K), `casting_costs.rs` (25K), `casting_targets.rs`, `mana_payment.rs`, `mana_abilities.rs` (15K) | CR 601 cast pipeline with `CastPaymentMode`, convoke, X, alternative costs |
| `layers.rs` (24.4K), `static_abilities.rs`, `static_source_index.rs`, `derived.rs`, `derived_views.rs` | CR 613 layer system; derived characteristics computed by the engine and exposed to clients |
| `triggers.rs` (48K), `trigger_index.rs`, `trigger_matchers.rs` (17.7K) | CR 603; indexed trigger lookup by event; batching; ordering |
| `replacement.rs` (21K), `engine_replacement.rs` | CR 614/615, with `ApplyPostReplacementDamage` continuation so a damage batch can pause on a nested choice without re-running replacement logic |
| `sba.rs`, `elimination.rs`, `life_safety.rs` | CR 704 |
| `combat.rs` (16K), `combat_damage.rs`, `engine_combat.rs` | CR 506–510 |
| `zones.rs`, `zone_pipeline.rs`, `exile_links.rs`, `lifecycle.rs`, `off_zone_characteristics.rs` | Zone-change pipeline (`ZoneMoveRequest` → `ZoneMoveResult`, batch moves) |
| `commander.rs` | CR 903: tax, cast-count ownership, command zone |
| `visibility.rs`, `public_state.rs` | Per-viewer projection of state, events, and rejections |
| `replay.rs`, `ledger.rs`, `log.rs` | Deterministic replay; `types/deterministic_serde.rs` |
| `scenario.rs`, `scenario_db.rs`, `test_fixtures.rs` | The test harness (2.8) |
| `coverage.rs` (18.5K), `gap_analysis.rs` | Coverage/gap reporting used by the `coverage-report` binary |
| `planechase.rs`, `archenemy.rs`, `conspiracy.rs`, `attractions.rs`, `stickers.rs`, `contraptions.rs`, `dungeon.rs`, `day_night.rs` | Supplementary/Un-set mechanics — evidence of "build for the class" reaching very far |

Effects are one module per handler in `game/effects/` (~110 files: `deal_damage.rs, counter.rs, draw.rs, destroy.rs, bounce.rs, change_zone.rs, cascade.rs, connive.rs, discover.rs, amass.rs, ...`). A handler receives a `ResolvedAbility` (targets already chosen, `chosen_players` populated by any preceding `Choose` clause) and returns events plus possibly a `WaitingFor`. Excerpt from `effects/bounce.rs` showing how a filter is walked to recover a chosen player:

```rust
/// CR 608.2c + CR 608.2d + CR 109.4 (issue #534): Resolve the *selecting
/// player* for a non-targeted graveyard-return `Bounce` whose filter scopes
/// ownership to a chosen player. ...
fn chosen_player_for_filter(ability: &ResolvedAbility, filter: &TargetFilter) -> Option<PlayerId> {
    fn find_index(filter: &TargetFilter) -> Option<u8> {
        match filter {
            TargetFilter::Typed(tf) => {
                if let Some(ControllerRef::ChosenPlayer { index }) = tf.controller {
                    return Some(index);
                }
                tf.properties.iter().find_map(|prop| match prop {
                    FilterProp::Owned { controller: ControllerRef::ChosenPlayer { index } } => Some(*index),
                    _ => None,
                })
            }
            TargetFilter::And { filters } | TargetFilter::Or { filters } => filters.iter().find_map(find_index),
            TargetFilter::Not { filter } => find_index(filter),
            _ => None,
        }
    }
    let index = find_index(filter)?;
    ability.chosen_players.get(index as usize).copied()
}
```

### 2.6 CR annotation as a hard requirement

Every rule-implementing line carries a `// CR XXX.Yz: description` comment, verified by grepping a local (gitignored) copy of the Comprehensive Rules. From `CLAUDE.md`: "A wrong CR number is worse than no CR number." Format regex `CR \d{3}(\.\d+[a-z]?)?`. There is a `validate-cr-annotations` skill and a `cargo rules-audit` binary (`--features audit`) that produces a CR coverage report. The `layers.rs` imports alone show the granularity: `eval_has_city_blessing, eval_is_monarch, eval_is_initiative, eval_has_enduring_story, count_devotion, effective_speed`.

This is the single practice most worth stealing regardless of anything else.

### 2.7 Multiplayer and hidden information

- `crates/server-core/src/filter.rs`: `filter_state_for_player(state, viewer) -> GameState` ("Hides ALL opponents' hand contents and ALL players' library contents") and `filter_events_for_player`. Both delegate to `engine::game::visibility`, so the projection logic lives in the engine, not the transport.
- Protocol is discriminated unions: `ClientMessage::{CreateGameWithSettings, JoinGameWithPassword, Action, Reconnect, Concede, Emote, SubscribeLobby}`, `ServerMessage::{GameCreated, GameStarted, StateUpdate, OpponentDisconnected, GameOver, LobbyUpdate, PlayerCount}`. 10-second reconnect grace.
- Transport-agnostic client `EngineAdapter` with five implementations: `WasmAdapter` (local), `TauriAdapter`, `WebSocketAdapter` (server-authoritative), `P2PHostAdapter` / `P2PGuestAdapter` (WebRTC via PeerJS; host runs the engine).
- Server ops: `PHASE_MAX_CONNECTIONS` (200), `PHASE_MAX_GAMES` (100), Prometheus `/metrics`, `PHASE_REPLICA_ORDINAL` for autoscaling; `PHASE_LOBBY_ONLY=true` runs the public server's matchmaking-broker mode. Docker image `ghcr.io/phase-rs/phase-server`.
- `WaitingFor::ResolveAllConsent { epoch, representative }` / `ResolveAllReady` implement a multiplayer "resolve all" shortcut with a frozen submitter ledger — a Commander-table quality-of-life feature.

### 2.8 Testing

31,024 `#[test]` functions across the workspace. Layers:

1. **Parser unit tests** (huge, colocated): `parser/oracle_effect/tests.rs` (61K lines), `oracle_static/tests.rs` (35K), `oracle_trigger_tests.rs` (32K), `oracle_tests.rs` (28K). Policy from `CLAUDE.md`: "Test the building block, not the special case. A parser test for 'exile target creature' is more valuable than a test for a single card name."
2. **Insta snapshot tests** of whole-card parse output (`parser/snapshots/`, 21 files) and pipeline snapshots (`game/snapshots/`).
3. **Runtime scenario tests** through a rigid recipe (`.claude/skills/card-test/SKILL.md`), using `game/scenario.rs`'s `GameScenario` + `GameRunner` + `CastOutcome`:

```rust
let mut scenario = GameScenario::new();
scenario.at_phase(Phase::PreCombatMain);
let spell = scenario.add_spell_to_hand_from_oracle(P0, "My Card", /* is_instant */ true, ORACLE).id();
let mut runner = scenario.build();
let outcome = runner.cast(spell)
    .modes(&[0, 2]).x(3)
    .target_player(P1).target_objects(&[victim])
    .convoke_with(&[creature])
    .resolve();                       // drives WaitingFor state machine per CR 601.2a-h
outcome.assert_life_delta(P1, -3);
outcome.assert_zone(&[victim], Zone::Exile);
outcome.assert_hand_drawn(P0, 1);
assert!(matches!(outcome.final_waiting_for(), WaitingFor::Priority { .. }));
```

   The skill enumerates six recurring harness foot-guns (hand-written `TargetRef` vectors, incomplete modal target submission, wrong hand baseline, inline-keyword cards, asserting AST internals, vacuous negative assertions) and makes them structurally impossible via the driver. Integration tests live in `crates/engine/tests/integration/` (1,538 files) behind a single binary; a guard test `no_top_level_test_binaries` prevents each file becoming its own ~130MB link.
4. **Regression tests named after bugs**: `triggers_dedup_regression_tests.rs`, `engine_phase_trigger_regression_tests.rs`, `omnath_tests.rs`, `marksman_tests.rs`.
5. **Audits as binaries**: `cargo coverage`, `cargo semantic-audit` (outputs `data/semantic-audit.json` + `.md`), `cargo parser-gaps`, `cargo rules-audit`, `cargo ai-gate` (paired-seed AI regression).
6. **Measurement hygiene** is documented: the card export is nondeterministic on ~20 faces, so "a delta of ≲20 faces between two full-pool runs is not signal"; snapshot `card-data.json` before touching the parser and diff generated-vs-generated.

### 2.9 Agent workflow (how the project actually gets built)

This is the part most relevant to how scutemob is developed.

- `CLAUDE.md` = `AGENTS.md` (185 lines, identical), plus `GEMINI.md`. Principles are ranked and non-negotiable: idiomatic Rust; rules-correct over convenient; build for the class not the card; engine owns all logic, frontend is display-only; compose from building blocks; parameterize don't proliferate; nom combinators from the first line ("Never write `find()`, `split_once()`, `contains()` for parsing dispatch"); "NEVER match on verbatim Oracle text strings — the single most prohibited pattern"; extend don't hack; trace before you build; verify the card against Scryfall/MTGJSON not from memory; read the card before the code.
- **Multi-agent concurrency rules**: never revert another agent's work; surgical edits only; no whole-file `Write`; never `git stash`; wait ~10 minutes for another agent to fix its own compile error before intervening.
- **Agent team orchestration**: teammates can't spawn subagents; lead spawns an opus review subagent per plan/implementation, max 3 rounds; sequential execution unless file sets are disjoint; shared collision points named (`types/ability.rs`, `effects/mod.rs`, `parser/oracle.rs`).
- **Tilt instead of cargo**: Tilt runs continuously (`clippy`, `test-engine`, `test-ai`, `wasm`, `card-data`, `check-frontend`, `test-frontend`, `server`, manual `coverage`); agents read `tilt logs` rather than run builds, to avoid cargo target-lock contention. `scripts/tilt-wait.sh` distinguishes exit 1 ("your code is broken") from exit 3 ("I could not find out") and the doc is emphatic that conflating them destroys trust in the gate.
- **30 skills in `.claude/skills/`**: `add-engine-effect` (the lockstep checklist: types → parser → resolver → targeting → multiplayer filter → frontend → AI → tests), `add-engine-variant`, `add-keyword`, `add-trigger`, `add-static-ability`, `add-replacement-effect`, `add-interactive-effect`, `oracle-parser` (parser single source of truth), `card-test`, `engine-planner`, `engine-implementer`, `review-engine-plan`, `review-impl`, `batch-mechanics`, `audit-card-parsing`, `bug-triage`, `bug-coverage-classifier`, `pr-review-loop`, `pr-contribution-handler`, `ship-commits`, `validate-cr-annotations`, `unlock-set`, `parser-velocity`, `retrain-ai-weights`, `ai-duel`, `changelog`, `project-reference`.
- **Outside contributors are asked to lend an LLM**: `docs/AI-CONTRIBUTOR.md` (536 lines) gives a copy-paste prompt: read the doc, use `$engine-implementer`, implement a card, open a PR, don't stop for input. There is a `--agent` setup flag that skips Scryfall art.
- Planning docs in `.planning/` per phase (CONTEXT/RESEARCH/PLAN/SUMMARY/VERIFICATION) plus `PROJECT.md` manifest. `docs/proposals/`, `docs/parser-misparse-backlog.md`, `docs/LEGACY-COMPAT.md`.

### 2.10 Honest weaknesses to weigh

- Card coverage is the open question; the README says thousands of cards are unimplemented and the Commander badge measures parse coverage, not "your deck resolves correctly."
- The Oracle-parsing bet has a long tail: `parser/oracle_effect/imperative.rs` is 24K lines, `oracle_replacement.rs` 27K, and the "swallow" problem needed its own 11K-line guard. This is a real grammar-engineering effort, not a trick.
- The `Effect` enum at 233 variants and `ability.rs` at 35K lines is a single compilation unit that every agent touches; the concurrency rules exist because of it.
- Weeks old. Public alpha. Solo maintainer plus agents.

---

## 3. Manabrew (witchesofthehill/manabrew)

### 3.1 Identity

- Repo: https://github.com/witchesofthehill/manabrew; hosted at manabrew.app; docs at docs.manabrew.app. Pre-release.
- README: "started because a small group of friends wanted a modern, open way to play Magic online together." Explicitly "uses Forge as the rules reference instead of defining a new interpretation of the game."
- License: own code AGPL-3.0-or-later; engine and bundled card data are derivative of Forge and therefore GPL-3.0-or-later. Forge Java source is vendored as a git submodule at `forge/` and is the reference implementation.
- Scale: 22 crates, ~235K lines of Rust total; `manabrew-engine` is 155K, `parity` 21K, `manabrew-hub` 10K, `parity-debugger` 7K, `self-hosted-node` 7K, `manabrew-agent-interface` 6K, `manabrew-server` 5.7K. 646 `#[test]` functions — but see 3.5; unit tests are not the primary verification.

### 3.2 Workspace layout

```
manabrew-rs/crates/
  manabrew-engine/        Rust port of forge-game (155K lines)          <- study this
  forge-card-script/      Forge DSL parser (1.9K) ; tree-sitter grammar in ../../tree-sitter-forge-card-script/
  forge-card-script-lsp/  LSP for editing Forge scripts
  forge-carddb/           Card database + script IR (1.4K)
  forge-foundation/       Shared enums (ZoneType, etc.) (4.9K)
  forge-limited/          Draft/sealed
  parity/                 Rust-vs-Java differential harness (21K)        <- and this
  parity-debugger/        GUI debugger for parity divergences (7.3K)
  manabrew-game-runtime/  Headless runtime
  manabrew-server/        Relay / lobby server (5.7K)
  manabrew-hub/           Hub service (10K)
  self-hosted-node/       Headless room host; spawns one Forge JVM per game on the forge backend
  manabrew-protocol/, manabrew-relay-protocol/   ts-rs generated wire types
  manabrew-agent-interface/  Prompt/decision protocol for players and bots (6K)
  manabot/                Bot player
  wasm/                   Browser engine bridge
  networking-tests/, loadgen/
forge/                    Java Forge submodule (reference)
forge-harness/            Java-side harness for parity + Forge-backed sessions
parity_decks/             Deck pairs used by the harness
src/, src-tauri/          React UI, Tauri shell
docs/                     PARITY_AND_IR.md, PARITY_TESTING.md, forge-dsl-grammar.md, forge-dsl-semantics.md, PROTOCOL.md, agents/
```

### 3.3 The card model: Forge's DSL, interpreted then compiled

Forge card scripts are flat key-value records. The canonical example (illustrative; the submodule isn't initialized in a shallow clone):

```
Name:Lightning Bolt
ManaCost:R
Types:Instant
A:SP$ DealDamage | ValidTgts$ Any | NumDmg$ 3 | SpellDescription$ CARDNAME deals 3 damage to any target.
Oracle:Lightning Bolt deals 3 damage to any target.
```

Four rule types (`docs/forge-dsl-semantics.md` §2): `A:` abilities (`SP$` spell, `AB$` activated, `DB$` sub-ability reachable only through SVars), `T:` triggers (event matchers with `Mode$` + filter params + `Execute$`), `S:` statics (continuous, layer-applied), `R:` replacements (produce *new* event instances; the original is discarded and never seen by triggers). The doc describes the DSL as "a declarative, event-driven rule system... not a programming language," with a `GameState` / event stream / rules triad, and is cross-checked against the CR with Forge's deviations flagged (§11.1 consolidated list). That semantics doc is itself a valuable artifact: it is the closest thing that exists to a written spec of what Forge's interpreter actually does.

Manabrew's engine started as a faithful Rust re-implementation of the string interpreter and is now growing typed IR underneath (`docs/PARITY_AND_IR.md`):

- parse common ability records into `SpellAbilityIr` (`manabrew-engine/src/ability/ability_ir.rs`);
- compile trigger/replacement/static params into `TriggerIr`, `ReplacementEffectIr`, `StaticAbilityIr` at construction;
- keep a lazy parsed-SVar cache on card state, because **SVar resolution is late-bound** ("Eagerly expanding the whole SVar graph at card load time is wrong because transforms, copies, LKI, and runtime SVar mutation can change what a later lookup should see");
- type `DefinedRef` forms and produced-mana domains (`Produced$`, `Combo ColorIdentity`);
- keep `Raw(String)` / `Unsupported(String)` buckets explicit and inventoried.

"This is an implementation divergence from Java Forge, but it must not become a behavior divergence." Contributors are told *not* to start with IR: "First find the missing Java rule and port it."

### 3.4 Engine structure mirrors Forge's Java

`manabrew-engine/src/` is organized as `ability/, card/, combat/, cost/, event/, game_loop/, keyword/, mana/, mulligan/, parsing/, phase/, player/, replacement/, spellability/, staticability/, svar/, trigger/, zone/` plus `game.rs, game_object.rs, game_snapshot.rs, game_rng.rs, lki.rs, game_log*.rs`. `spellability/` has `ability_activated.rs, ability_static.rs, ability_sub.rs, alternative_cost.rs, optional_cost.rs, spell_permanent.rs, target_choices.rs, target_restrictions.rs, spell_ability_stack_instance.rs, spell_ability_variables.rs, valid_sa.rs` — these are Forge's `SpellAbility`, `AbilityActivated`, `AbilitySub`, `TargetChoices`, `SpellAbilityStackInstance` class names, one-to-one. `game_loop/` has `cast_spell.rs, cost_payment.rs, mana_payment.rs, mana_action_undo.rs, priority.rs, stack_resolution.rs, trigger_handler.rs, trigger_replacement_base.rs, phase_handler.rs, combat_phase.rs, action_space.rs, state_observer.rs`.

`game.rs` opens with a `TypeRegistry` backed by `OnceLock<Vec<String>>` loaded from Forge's `TypeLists.txt`, with a doc comment: "Mirrors Java's `CardType.Constant.CREATURE_TYPES` etc., populated once by `FModel.loadDynamicGamedata()` → `CardType.Helper.parseTypes()`." Almost every module has this kind of "mirrors Java X" annotation. The state model is `Game { zones: ZoneStore, stack: MagicStack, players: Vec<PlayerState>, turn: TurnState, ... }` with `CardZoneTable`, `CardDamageMap`, `CostPaymentStack`, `ExtraTurn` — mutable OO translated into Rust, `Arc`/`BTreeMap`/`VecDeque` heavy, not a reducer.

### 3.5 Parity harness: differential testing as the primary correctness mechanism

`manabrew-rs/crates/parity/` runs the Rust engine and Java Forge with the same deck pair, seed, and deterministic choices, compares state snapshots, and reports the first field-level divergence (phase, turn, player, object, differing values).

```bash
yarn build:harness
yarn parity:test -- --deck1 red_burn --deck2 green_stompy --seed 42 --max-turns 20
yarn parity:test -- --matrix --seeds 42,100,999
yarn parity:gui        # parity-debugger
```

From `PARITY_AND_IR.md`: "The harness is not only a test runner. It is the core development tool for the engine." From `README.md`: "A good fix restores the missing general rule in Rust by reading the corresponding Java file, not by special casing the card that exposed the bug." Note the convergence with phase.rs on "class not card," reached from the opposite direction.

This is why the `#[test]` count (646) is low and misleading: correctness is defined as agreement with Forge on a corpus of seeded games, not as passing hand-written assertions.

### 3.6 Multiplayer and the Java backend path

Two backends behind one client stack and one prompt protocol (`manabrew-agent-interface`, `manabrew-protocol`):

- **Rust parity path** — the Rust engine, used where it has reached parity.
- **Java backend path** — `self-hosted-node` "spawns one Forge JVM per concurrent game," so players get full Forge card coverage through the modern Tauri/web client today. README: "The client and Java Forge backend path are the most practical way to play today."

Topology: relay (`manabrew-server`, `manabrew-relay-protocol`) + headless room hosts (`self-hosted-node`) that connect out to the relay and run games for whoever joins. Signals: `SIGUSR1` drains (finish games, close rooms, exit 0); `--shutdown-on-stale` polls a version manifest and exits when idle so a supervisor can respawn updated. Wire types are generated from Rust via ts-rs and documented at docs.manabrew.app/protocol/. Prerequisite for the Java path and parity: JDK 18–21 + Maven.

### 3.7 Agent workflow

`CLAUDE.md`, `AGENTS.md`, `docs/agents/PARITY_PHILOSOPHY.md`, `docs/STYLE_GUIDELINES.md`. README: "This repository welcomes the use of AI assistance for mechanical porting, parity fixes..." The workflow is deliberately mechanical: failing parity run → find Java owner of the mechanic → mirror in Rust → parity green. That is a very good fit for agents because the oracle is executable and the target text (Java) is concrete.

### 3.8 Honest weaknesses to weigh

- AGPL/GPL. If scutemob ever wants to be anything other than GPL-family, Manabrew's engine and card data are not reusable; its *documents* (`forge-dsl-semantics.md` is CC-BY-4.0 per the SPDX header on `PROTOCOL.md`; check each file) may be.
- Rust engine "works for selected matchups"; the playable product today is really Forge-in-a-JVM behind a nice client.
- The engine inherits Forge's rules deviations from the CR by design (documented in §11.1 of the semantics doc). Parity means parity with Forge, including Forge's bugs, until Forge fixes them.
- Mutable OO state model makes AI search, replay, and hidden-information projection harder than in a reducer architecture.

---

## 4. Legacy baselines (for calibration only)

### XMage (magefree/mage)
Java 8, MIT. One class per card composed from a library of `Ability`/`Effect`/`Target`/`Condition` objects; roughly:

```java
public final class LightningBolt extends CardImpl {
    public LightningBolt(UUID ownerId, CardSetInfo setInfo) {
        super(ownerId, setInfo, new CardType[]{CardType.INSTANT}, "{R}");
        this.getSpellAbility().addEffect(new DamageTargetEffect(3));
        this.getSpellAbility().addTarget(new TargetAnyTarget());
    }
    private LightningBolt(final LightningBolt card) { super(card); }
    @Override public LightningBolt copy() { return new LightningBolt(this); }
}
```

Server-authoritative; all rules and hidden info enforced server-side; ~30,000 unique cards; Commander up to 10 players; actively released (2026-08-12 build). Swing UI. The reason XMage matters to scutemob: it is the existence proof that a **per-card, composed-from-primitives** model (scutemob's model) can reach full coverage — at the cost of ~15 years of volunteer labor and a primitives library in the thousands of classes. The per-card approach's coverage curve is linear in effort; phase.rs's is meant to be sublinear once the grammar covers a class; Manabrew's is front-loaded (port the interpreter) then near-free.

### Forge (Card-Forge/forge)
Java, GPL-3. The DSL described in 3.3. Excellent single-player AI, adventure mode, ~30,000 cards. Network play exists (`Online Multiplayer > Lobby`, port 36743, host must port-forward) but issue #9266 (Nov 2025) reports non-host players auto-skipping their turns in >2-player games, so it is not a Commander option. Forge's real contribution to the ecosystem is the script corpus and the trigger/replacement vocabulary that both Rust projects reuse.

---

## 5. Cross-cutting design comparison

| Axis | phase.rs | Manabrew | Question for scutemob |
|---|---|---|---|
| **Source of card semantics** | Oracle text via grammar; per-card JSON overrides; Forge fallback | Forge scripts; typed IR grows underneath | Hand-authored. What is the *format*? Is it data (JSON/RON/DSL) or Rust code? Is it diffable and auditable by an agent without reading the engine? |
| **Fail-closed policy** | `Effect::Unimplemented` is a typed value; swallow checks; coverage CI | `Raw(String)`/`Unsupported(String)` buckets, inventoried | When scutemob doesn't support a clause, what happens? Silent no-op, panic, typed gap, or the card can't be loaded? |
| **State mutation** | Pure reducer + `WaitingFor` state machine; persistent collections | Mutable OO mirroring Java | Which is scutemob? If mutable, how are replays, undo, AI simulation, and per-viewer projection handled? |
| **Interactive resolution** | Engine returns `WaitingFor`; next `GameAction` answers | Prompt protocol (`manabrew-agent-interface`) | How does scutemob suspend mid-resolution for a choice (modes, X, targets on resolution, "choose a player", ordering triggers)? |
| **Rules provenance** | Mandatory `CR xxx.y` annotations, verified by grep; rules-audit binary | Java-file provenance ("mirrors `CardType.Helper.parseTypes()`"); CR deviations listed | Does scutemob code carry CR citations? Can you produce a report of which CR sections are implemented? |
| **Layer system** | `layers.rs` 24K lines, `static_source_index.rs`, derived views exposed to client | Forge's `StaticAbilityContinuous` port | How complete is CR 613 in scutemob? Timestamps, dependency, CDAs, copy effects, control-change, text-change, type-change, P/T layers 7a–7d? |
| **Hidden information** | Engine-side `visibility.rs`; server filters state, events, and *error messages* per viewer; actor is bound to connection | Server-hosted rooms; relay | Does scutemob have a per-viewer projection of state and events? Are rejections viewer-safe? Is the acting player derived from the connection, not the payload? |
| **Testing** | 31K unit tests (grammar-level), snapshots, scenario driver, regression files, audit binaries | Differential parity vs Forge | 130K tests: what are they *of*? Building blocks or per-card replays? Is there a snapshot layer? A scenario DSL? Is there any external oracle? |
| **Coverage measurement** | `cargo coverage` per format; badges; noise-floor documented | Parity matrices; raw/typed inventories | Can scutemob answer "what percentage of the cards in these four decks resolve correctly" mechanically? |
| **Agent process** | Ranked principles; lockstep skill checklists; multi-agent rules; Tilt gate; review loop; contributor-LLM prompt | Parity-driven mechanical porting | scutemob has ESM + skills; are the *engine-specific* checklists (add effect → parser → resolver → targeting → filter → client → AI → tests) written down? |
| **Commander specifics** | `commander.rs` (CR 903.8 tax, owner tracking), `ResolveAllConsent` multiplayer shortcut, 10-player | Forge's Commander rules | Command zone, tax, commander damage (CR 903.10a), color identity deck validation, partner/background, "Resolve All"-style table shortcuts? |
| **Client** | React display-only; engine exposes all derived values | React + Tauri; prompt protocol | Cockatrice today. Is the plan a Cockatrice-compatible protocol, a phase.rs-style adapter, or your own client? |
| **License** | MIT/Apache | AGPL/GPL | What is scutemob's? Determines which of these you can borrow *code* from (phase.rs: yes; Manabrew: only if GPL-family). |

---

## 6. Assessment tasks for the scutemob agent

Do these in order; each produces a concrete artifact.

1. **Classify scutemob's card model.** Write one paragraph placing it on the XMage ↔ Forge ↔ phase.rs spectrum. Then take three cards of increasing difficulty (Lightning Bolt; a modal ETB with a "choose a player" clause; a replacement effect with an "instead" rider and a layer-dependent static) and show, side by side, scutemob's representation, phase.rs's parsed JSON (`cargo run --bin oracle-gen -- data --filter "<name>"` in a phase.rs checkout, or read the snapshots in `crates/engine/src/parser/snapshots/`), and the Forge script. Note which representation an agent could author fastest *and* which one an agent could audit for correctness fastest. Those are different properties.

2. **Audit fail-closed behavior.** Grep scutemob for every place an unsupported clause, keyword, or trigger mode is encountered. Categorize each as: typed gap (like `Effect::Unimplemented`), silent no-op, panic, or load failure. Compare to phase.rs's single-authority `Effect::unimplemented(name, fragment)` and its swallow check. Produce a list of silent no-ops; those are the ones that will lose games at the table.

3. **Compare the reducer.** Read `phase/crates/engine/src/game/engine.rs` lines ~860–1010 (`apply`, `apply_with_rejection`, `apply_for_simulation`) and `types/game_state.rs` `WaitingFor` and `ActionResult`. Diff against scutemob's top-level action entry point. Specifically check: is the acting player an explicit parameter bound to the connection; does every interactive decision have a typed `WaitingFor`-equivalent; are events the only output channel or does the client read mutable state directly.

4. **CR annotation census.** Count `CR \d{3}` occurrences in scutemob. If low, decide whether to adopt phase.rs's rule (mandatory, verified by grep against a local `MagicCompRules.txt`, `validate-cr-annotations` skill). Estimate cost: it can be retrofitted by an agent as a pure-comment pass, one module at a time, and it materially improves later agent work because every function announces which rule it claims to implement.

5. **Layer-system parity check.** Enumerate CR 613 sublayers and, for each, list whether scutemob implements it, with a test name. Compare to `phase/crates/engine/src/game/layers.rs` (read the import block and the top-level `apply_layers`-style function) and Manabrew's `staticability/`. Report gaps.

6. **Hidden-information audit.** Trace what a scutemob opponent's client can see in: state, events, error/rejection messages, and log entries. Compare to `phase/crates/engine/src/game/visibility.rs` and `crates/server-core/src/filter.rs`. If scutemob has no per-viewer projection yet, note that phase.rs's design puts it *in the engine crate* so that P2P, server, and local modes share one implementation.

7. **Characterize the 130K tests.** Sample 200 at random and bucket them: grammar/building-block tests, per-card resolution tests, regression tests, property tests, snapshot tests. Compare the distribution to phase.rs (2.8) and to Manabrew's parity-first model (3.5). Then answer: does scutemob have any *external oracle*? Neither hand-written assertions nor snapshots catch "we agreed with ourselves." Options: a parity harness against XMage (MIT, server-side, scriptable via its test framework), or against Forge via Manabrew's `forge-harness` approach.

8. **Build a coverage report.** Take the four Commander decks the friend group actually plays. For each card, produce supported / partially supported (which clause) / unsupported. This is the number that matters for leaving Cockatrice, and it is the number phase.rs's `cargo coverage` and Manabrew's parity matrices exist to produce.

9. **Process comparison.** Read `phase/CLAUDE.md` and `phase/.claude/skills/add-engine-effect/SKILL.md` end to end. List every rule or checklist item that scutemob's ESM/skills lack. Prioritize: (a) lockstep registration checklist for a new effect; (b) "parameterize, don't proliferate" with the sibling-cluster smell; (c) multi-agent file-collision rules; (d) engine-inventory JSON as the canonical surface index; (e) measurement-noise-floor documentation before claiming coverage wins.

10. **Strategic recommendation.** Given 1–9, argue for one of: (A) continue hand-authoring cards against scutemob's primitives (XMage's path — linear, but you control everything; MIT-compatible); (B) add an Oracle-text parser front end modeled on phase.rs's `oracle_nom` and treat hand-authored cards as overrides (sublinear once the grammar covers a class; can borrow MIT/Apache code); (C) add a Forge-script front end (fast coverage; GPL contamination if you copy Manabrew's engine, but Forge's script *format* is just a file format — the scripts themselves are GPL); (D) stop building an engine and contribute cards to phase.rs, pointing scutemob's agent workflow at their `docs/AI-CONTRIBUTOR.md`. Be explicit about what would change the recommendation.

---

## 7. Source map: what to read, in order

**phase.rs** (MIT/Apache; safe to read and borrow from)

1. `CLAUDE.md` — principles, the parameterization rule, multi-agent rules
2. `.claude/skills/project-reference/SKILL.md` — architecture, pipeline, env vars
3. `.claude/skills/add-engine-effect/SKILL.md` — the lockstep checklist
4. `.claude/skills/oracle-parser/SKILL.md` — parser architecture and AST
5. `.claude/skills/card-test/SKILL.md` — test recipe
6. `crates/engine/src/types/ability.rs` — `AbilityDefinition` (~22705), `Effect` (14129–17684), `TargetFilter`, `QuantityExpr`
7. `crates/engine/src/types/game_state.rs` — `GameState` macro (17151+), `WaitingFor` (12108), `ActionResult` (15838)
8. `crates/engine/src/types/actions.rs` — `GameAction` (150)
9. `crates/engine/src/game/engine.rs` — `apply` family (~882)
10. `crates/engine/src/game/{layers,triggers,replacement,stack,casting,commander,visibility,zone_pipeline}.rs`
11. `crates/engine/src/game/effects/{deal_damage,bounce,change_zone,counter}.rs` — handler pattern
12. `crates/engine/src/parser/oracle.rs` and `parser/oracle_nom/` — dispatcher and combinators
13. `crates/engine/src/parser/snapshots/*.snap` — what parsed cards look like
14. `crates/engine/src/database/forge/mod.rs` — the Forge fallback bridge
15. `crates/server-core/src/filter.rs`, `crates/phase-server/` — multiplayer
16. `docs/AI-CONTRIBUTOR.md` — how they onboard other people's agents

**Manabrew** (AGPL/GPL; read for ideas, do not copy code unless scutemob is GPL-family)

1. `README.md` — status, two-backend design
2. `docs/PARITY_AND_IR.md` — the whole strategy in 200 lines
3. `docs/forge-dsl-semantics.md` — the best written spec of Forge's runtime semantics that exists; §11.1 lists CR deviations
4. `docs/forge-dsl-grammar.md` and `tree-sitter-forge-card-script/` — a real grammar for Forge scripts
5. `docs/PARITY_TESTING.md`, `docs/agents/PARITY_PHILOSOPHY.md`
6. `manabrew-rs/crates/parity/` — the harness
7. `manabrew-rs/crates/manabrew-engine/src/{game.rs, game_loop/, spellability/, svar/, staticability/, replacement/, trigger/}`
8. `manabrew-rs/crates/manabrew-agent-interface/` — prompt protocol
9. `manabrew-rs/crates/self-hosted-node/README.md` — room-host operations

**XMage / Forge** — only for coverage calibration and as parity oracles. XMage: `Mage/src/main/java/mage/abilities/effects/` (effect library), `Mage.Sets/src/mage/cards/` (card classes), `Mage.Tests/` (the test framework, which is scriptable and would make a usable external oracle). Forge: `forge-gui/res/cardsfolder/` (scripts), `forge-game/src/main/java/forge/game/` (engine).
