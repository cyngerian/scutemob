# M11-local Session Plan: Web Client & Local Play (1 Human + 3 Bots)

**Generated**: 2026-07-26
**Milestone**: M11-local — Web Client & Local Play, FIRST PLAYABLE
**Sessions**: 8
**Estimated new tests**: ~50 (Rust; the two frontend sessions are manual-checklist only)
**Baseline at plan time**: PROTOCOL_VERSION 27 / fingerprint `f035e797…` / HASH_SCHEMA_VERSION 63 / 3560+ tests

---

## 0. Goal and scope boundary

**Goal**: a human sits at one seat of a 4-player Commander game in a browser, three
`crates/simulator` bots fill the other seats, the engine runs in-process in a local axum
server, and the human can play a game from turn 1 to a natural conclusion.

**M11-local IS**:
- a human-input bridge that lets a human occupy a seat in a bot-driven game;
- a local axum + Svelte 5 client that extends the existing replay-viewer stack;
- deterministic pregame setup (shuffle, opening hands, `Complete`-only deck admission);
- seat-scoped (hidden-information-safe) view models and event feed.

**M11-local is NOT** (all deferred, with owners):
- networking, rooms, matchmaking, reconnection → **M10a**
- WebSocket/SSE push, `GameEvent::private_to()`, state-history ring buffer, rewind UI,
  pause consent, manual state adjustment → **M10b**
- Tauri packaging or any Tauri IPC → post-alpha packaging decision (web-first decided
  2026-07-26, `memory/decisions.md`)
- UI polish, animation, art assets → **M13/M14**
- the two "engine feature additions" still listed under the roadmap's M11 deliverables
  (**turn-control override** / Mindslaver, **step skipping** / Stasis). These are engine
  primitives, not UI work; they belong in the primitive queue, not here. See §8 Risk R10.

**Roadmap inconsistency to fix (Session 8 item)**: `docs/mtg-engine-roadmap.md` lines
843–917 carry a scope-boundary paragraph that excludes rewind/pause/manual-adjust, but the
deliverable checklist below it still contains those four bullets plus the two engine
features. Session 8 moves them to the M10b / primitive-queue sections.

### Standing user-feedback channel

`memory/ui-feedback.md` (on `main`, added 2026-07-26 — not present on this task's branch
until merge) is the inbox for the user's hands-on notes about the play UI. **Read it at the
start of every UI session (5, 6, 7, 8)** and honour anything `queued` there.

One entry is already binding on this plan: the user endorsed the **legal-targets dropdown**
as the target-selection model — "playing a spell and getting a drop-down menu that only
consists of legal targets you need to choose from." That is exactly what Session 3's
`legal_targets_per_slot` + Session 7's `TargetPicker` (one selector per slot, engine-
enumerated candidates only) provide, so no plan change is needed — but it does mean
Arena-style drag-arrows are **lower** priority than M13 previously assumed, and the picker
must not be treated as a placeholder for them.

### Source citations for the decisions this plan encodes

- `docs/mtg-engine-strategic-review.md` **Finding 1** (decouple M11 from M10 — the M10
  dependency was soft; a client can drive the engine in-process) — adopted wholesale.
- **Finding 2** (web UI instead of Tauri) — adopted, *but its stated rationale is stale*:
  "the Tauri app cannot build on the current dev environment (headless Debian)" is no
  longer true (dev is skylarch, a full desktop). The web-first call stands on the other
  two grounds Finding 2 lists — one UI framework instead of two, and faster iteration —
  plus the reuse of the replay viewer's component set. Recorded in `memory/decisions.md`
  2026-07-26.
- **Finding 5** (the simulator already has bots; what is missing is the human input
  bridge) — confirmed by source audit, and *more* is already present than Finding 5
  claims: see §1.
- **Finding 6** (split M10 into M10a/M10b) — M11-local depends on neither half.
- **Revised Critical Path** — M11-local runs in parallel with M10a and is the branch
  labelled "humans can play here".
- `docs/mtg-engine-replay-viewer.md` §"Shared Component Strategy" (props-based components,
  data-fetching adapter is the only thing that differs per host) — this plan uses exactly
  that contract, substituting a play adapter for the Tauri adapter it anticipated.

---

## 1. What already exists (verified by reading source, not assumed)

| Thing | Location | State |
|---|---|---|
| `GameDriver<P>::run_game` | `crates/simulator/src/driver.rs:43` | Blocking, runs a whole game; the problem this plan solves |
| `Bot` trait | `crates/simulator/src/bot.rs` | 6 methods, but **only `choose_action` is ever called by the driver** — `choose_targets`/`choose_attackers`/`choose_blockers`/`choose_mulligan_bottom` have zero call sites outside the two bot impls' own `choose_action` |
| `LegalAction` (22 variants) + `LegalActionProvider` + `StubProvider` | `crates/simulator/src/legal_actions.rs:16,123,168` | Works; documented gaps (Adventure, alt-costs, modes) |
| `action_to_command` | `crates/simulator/src/random_bot.rs:128` | `pub(crate)`; **always emits `targets: Vec::new()`** |
| `solve_mana_payment` | `crates/simulator/src/mana_solver.rs:23` | Greedy; **never consults the player's mana pool**; reads `obj.characteristics.mana_abilities` (not layer-resolved) |
| `random_deck` | `crates/simulator/src/deck.rs:30` | Already filters `completeness.is_complete()` (SR-12) |
| **An existing human-input bridge** | `tools/tui/src/play/` (`app.rs`, `input.rs`, `render.rs`, `panels/`) | **Prior art the strategic review does not mention.** `PlayApp` already: builds a shuffled deck with a 7-card opening hand (`app.rs:122-165`), keeps `human_player: PlayerId(1)`, exposes `is_bot_turn()` / `execute_bot_turn()` / `execute_command()`, auto-taps before casting, and formats an event log. It reimplements the driver loop instead of reusing it, and it **never supplies targets** (`input.rs:98`, `targets: Vec::new()`), so targeted spells are uncastable |
| axum + Svelte 5 stack | `tools/replay-viewer/` (708-line `api.rs`, 895-line `view_model.rs`, 15 Svelte components) | Props-based components; `api.js`/`stores.js` are the only host-specific layer |
| **Router tests without a running server** | `tools/replay-viewer/src/main.rs:198-414` | `tower::ServiceExt::oneshot` against `build_router(...)`. This is the pattern every HTTP test in this milestone must use (see the OOM gotcha, §7) |
| 8 MB tokio worker stacks | `tools/replay-viewer/src/main.rs`'s `fn main` (`:50-65`) | Required: engine trigger chains overflow tokio's default 2 MB stacks in debug builds |
| `crates/network` | `crates/network/src/lib.rs` | 4-line stub, reserved for M10a. **M11-local does not touch it** |

### Facts that shape the design (each verified in source)

1. **The engine has no RNG and never shuffles.** `commander.rs::handle_take_mulligan`
   moves the hand to the library, emits `GameEvent::LibraryShuffled`, and draws 7 —
   with no permutation ("Shuffle library (represented by event; order is not tracked in
   state)"). A mulligan therefore returns a near-identical hand. Shuffling is the
   caller's job at state-build time. → Session 2 does mulligans by pregame rebuild.
2. **Nothing deals opening hands.** `GameStateBuilder` defaults `turn_number = 1`, and
   `StubProvider`'s mulligan branch only fires at `turn_number == 0`
   (`legal_actions.rs:186`). The fuzzer consequently plays games where every player
   starts with an empty hand. The TUI works around this by putting 7 cards in
   `ZoneId::Hand` at build time. → Session 2 makes that a shared, tested helper.
3. **`GameEvent::private_to()` does not exist.** Only `reveals_hidden_info()`
   (`rules/events.rs:1361`) exists, and it is a rewind-checkpoint predicate, not a
   recipient filter. Hidden-information enforcement for M11-local therefore happens in
   the **view-model layer of the server** (Architecture Invariant 7). → Session 4.
4. **Target enumeration is not public.** `validate_targets_with_source`,
   `validate_object_satisfies_requirement`, `target_count_range` are all `pub(crate)`
   (`rules/casting.rs:5756-5803`). Requirements themselves *are* public (they live on
   `AbilityDefinition::Spell { targets }` in `crates/card-types`). Re-deriving target
   legality outside the engine is exactly the drift class OOS-RS-2 was. → Session 3 adds
   a read-only public query that delegates to the existing checkers.
5. **`TakeMulligan` / `KeepHand` are dispatched with only a player-exists guard**
   (`rules/engine.rs:245-256`) — they are reachable pregame, they just cannot shuffle.
6. **Optional payments are never offered.** `pending_echo_payments`,
   `pending_cumulative_upkeep_payments`, `pending_recover_payments`, and
   `Command::OrderBlockers` (CR 509.2, default order used if absent) have no
   `LegalAction`. Not stalls; just silently unavailable. → Session 8 item 2.

---

## 2. The central problem: `run_game` is a closed loop

`GameDriver::run_game(&mut self, initial_state, seed) -> GameResult` (driver.rs:43) owns
the state, loops to conclusion, and calls `bot.choose_action(&state, player, &legal)`
synchronously. A human's decision arrives later, over HTTP. Three candidate shapes:

### (a) Steppable driver — **RECOMMENDED**

Extract the loop body so a caller can advance the game until a designated human seat must
act, get back a pending-decision descriptor, and later hand in the chosen action.

    pub enum AdvanceOutcome {
        AwaitingHuman(PendingDecision),
        GameOver(GameResult),
        Halted(HaltReason),          // MaxTurns | InfiniteLoop | EngineError
    }

    impl<P: LegalActionProvider> LocalGame<P> {
        pub fn advance(&mut self) -> AdvanceOutcome;
        pub fn submit(&mut self, seq: u64, choice: HumanChoice)
            -> Result<Vec<GameEvent>, LocalGameError>;
    }

- **Engine purity**: perfect. No async, no IO, no threads; sync `&mut self` methods on a
  struct that owns a `GameState`. Lives in `crates/simulator`, not `crates/engine`.
- **Testability without HTTP**: perfect. A test scripts the "human" as a function from
  `PendingDecision` to `HumanChoice` and runs a whole game in-process.
- **Async boundary**: entirely inside `tools/play-server` — an axum handler locks the
  session, calls `submit` then `advance`, serializes the result. Nothing async ever
  crosses into simulator or engine code.
- **Sub-decisions**: the five non-`choose_action` `Bot` methods are *not* driver
  callbacks (verified: zero call sites), so there is nothing to intercept. Attackers,
  blockers, mulligan-bottoms and targets are all fields of the `Command` the seat
  returns. They become fields of `ActionParams` (§3), which is a server-side type — **no
  new `Command` or `GameEvent` variant, so PROTOCOL 27 and fingerprint `f035e797…` do not
  move.**
- **Bonus**: it fixes a UX landmine. `run_game` currently reacts to a rejected command by
  silently issuing `PassPriority` instead (driver.rs:233-260). For a human that would
  turn "you mis-clicked a target" into "you passed priority". `submit` returns `Err`.

### (b) Channel-backed `Bot` impl

The human seat is a `Bot` whose `choose_action` blocks on a channel fed by the web layer;
the driver runs on its own thread.

- Requires a second channel to stream state out, because `run_game` returns nothing until
  the game ends — so the "read the board" path is a separate mechanism from the "act"
  path, and the two can disagree.
- Tests need threads and channels: slow, order-dependent, prone to hangs in CI.
- The human seat inherits the silent `PassPriority` fallback on error (above): a rejected
  human action becomes an irreversible pass with no error surface.
- `Bot::choose_action` returns `Command`, with no channel for "invalid, try again" or for
  cancel/abort. Shutdown means killing a blocked thread.
- Only advantage: `driver.rs` is untouched. Not worth the rest.

### (c) Server-side re-implementation of the loop (what the TUI does today)

`tools/tui/src/play/app.rs` already does this. It works, but it has drifted from
`driver.rs` (no turn/command limits, different pass accounting, different error
handling), and doing it a third time in the play server would give three loops with three
sets of bugs.

### Decision

**Adopt (a)**, and make it the single loop: `LocalGame` becomes the core, `run_game` is
re-expressed on top of it (zero human seats), and the TUI is rewired to consume it
(Session 2 does the setup half; the TUI's loop can migrate opportunistically). The
fuzzer's behaviour must be bit-identical after the refactor — Session 1 item 1 captures a
baseline before touching `driver.rs`, item 8 diffs against it.

---

## 3. Architecture

### Crate layout and where the async boundary sits

| Crate | Kind | M11-local change | Async/IO? |
|---|---|---|---|
| `crates/engine` | lib | **Read-only query surface only** (`rules/queries.rs`): target requirements + per-slot legal candidates. No new `Command`/`GameEvent`/`GameState` field. | **Never.** Invariant 1 holds |
| `crates/card-types`, `crates/card-defs` | lib | none | — |
| `crates/simulator` | lib | `local_game.rs` (bridge), `setup.rs` (pregame), `params.rs` (action parameterization) | No. Sync only; `rand` already a dep |
| `crates/view-model` | **new lib** (`mtg-view-model`) | `view_model.rs` moved out of the replay-viewer binary + seat redaction + event views | No. serde only |
| `tools/play-server` | **new bin** | axum server, session state, DTOs, static serving | **Yes — the only place** |
| `tools/play-server/frontend` | **new** Svelte 5 app | play UI; imports replay-viewer components by Vite alias | browser |
| `tools/replay-viewer` | bin | depends on `mtg-view-model` instead of its local module (behaviour-neutral) | unchanged |
| `tools/tui` | bin | consumes `simulator::setup` (dedup) | unchanged |
| `crates/network` | lib stub | **untouched** (M10a) | — |

The async boundary is exactly one function deep: an axum handler acquires the session
mutex, runs sync simulator/engine code inside `tokio::task::block_in_place`, and returns
JSON. Nothing below `tools/play-server/src/api.rs` knows tokio exists.

### Data model (new types, by crate)

`crates/simulator/src/local_game.rs`:

    pub struct LocalGame<P: LegalActionProvider = StubProvider> {
        state: GameState,
        provider: P,
        bots: HashMap<PlayerId, Box<dyn Bot>>,
        // Seats a human occupies. Empty => pure bot game (the GameDriver case).
        human_seats: BTreeSet<PlayerId>,
        limits: LocalGameLimits,
        consecutive_passes: u32,
        command_count: u32,
        // Monotonic. Every emitted PendingDecision carries it; submit() rejects a
        // mismatch so a stale browser tab cannot act on a superseded action list.
        decision_seq: u64,
        journal: Vec<CommandRecord>,
        violations: Vec<InvariantViolation>,
        check_invariants: bool,
    }

    pub struct LocalGameLimits { pub max_turns: u32, pub max_commands: u32,
                                 pub max_consecutive_passes: u32 }

    pub struct PendingDecision {
        pub seq: u64,
        pub player: PlayerId,
        pub kind: DecisionKind,
        pub actions: Vec<LegalAction>,
    }

    // CR 117.3 (priority), CR 103.5 (mulligan), CR 903.9a (commander zone),
    // CR 508.1 / 509.1 (combat declarations).
    pub enum DecisionKind { Priority, Mulligan, CommanderZoneChoice,
                            DeclareAttackers, DeclareBlockers }

    pub struct CommandRecord { pub command: Command, pub events: Vec<GameEvent>,
                               pub turn: u32 }

    pub enum LocalGameError {
        StaleDecision { expected: u64, got: u64 },
        NoPendingDecision,
        UnknownAction(usize),
        BadParams(String),
        Rejected(GameStateError),   // engine said no; state unchanged
        Engine(GameStateError),     // failure while advancing bot seats
    }

`crates/simulator/src/params.rs`:

    // Everything CR 601.2b-601.2h lets a caster announce, as data. Not a Command
    // variant — this is assembled into an existing Command by the function below.
    #[derive(Clone, Debug, Default)]
    pub struct ActionParams {
        pub targets: Vec<Target>,                       // CR 601.2c
        pub x_value: u32,                               // CR 601.2b
        pub modes_chosen: Vec<usize>,                   // CR 700.2
        pub attackers: Vec<(ObjectId, AttackTarget)>,   // CR 508.1
        pub blockers: Vec<(ObjectId, ObjectId)>,        // CR 509.1
        pub cards_to_bottom: Vec<ObjectId>,             // CR 103.5 KeepHand
        pub additional_costs: Vec<AdditionalCost>,
        pub auto_tap: bool,
    }

    pub struct HumanChoice { pub action_index: usize, pub params: ActionParams }

    pub fn action_to_command_with_params(
        state: &GameState, player: PlayerId,
        action: &LegalAction, params: &ActionParams,
    ) -> Result<Command, ParamError>;

`crates/simulator/src/setup.rs`:

    pub struct LocalGameConfig {
        pub player_count: u32,
        pub human_seats: BTreeSet<PlayerId>,
        pub bot_kind: BotKind,          // Random | Heuristic
        pub seed: u64,
        pub decks: DeckSource,          // RandomPerSeat | Fixed(Vec<(PlayerId, DeckConfig)>)
        pub limits: LocalGameLimits,
    }
    pub fn build_initial_state(cfg: &LocalGameConfig)
        -> Result<(GameState, HashMap<PlayerId, String>), SetupError>;
    // CR 103.5 / 103.5c — pregame re-deal, because the engine cannot shuffle (§1 fact 1).
    pub fn redeal(cfg: &LocalGameConfig, seat: PlayerId, mulligan_count: u32)
        -> Result<(GameState, HashMap<PlayerId, String>), SetupError>;

`crates/view-model` (moved + new):

    pub enum Viewer { Omniscient, Seat(PlayerId) }
    impl StateViewModel {
        pub fn from_game_state(state, names) -> Self;              // = Omniscient (shim)
        pub fn from_game_state_for(state, names, viewer) -> Self;  // redacting
    }
    pub struct EventView { pub kind: String, pub text: String }
    pub fn event_view_for(ev: &GameEvent, state: &GameState, viewer: Viewer)
        -> Option<EventView>;   // None => wholly private to another seat

`crates/engine/src/rules/queries.rs` (read-only, no new engine types):

    /// CR 601.2c — the target requirements a spell cast from `card` announces,
    /// honouring Aftermath (CR 702.127a), Overload (CR 702.96b) and per-mode
    /// requirements (CR 700.2c/700.2f). Mirrors casting.rs:3563-3611.
    pub fn spell_target_requirements(state: &GameState, card: ObjectId,
                                     modes_chosen: &[usize]) -> Vec<TargetRequirement>;
    /// CR 602.2b — same, for an activated ability by index.
    pub fn ability_target_requirements(state: &GameState, source: ObjectId,
                                       ability_index: usize) -> Vec<TargetRequirement>;
    /// Per-slot candidates, parallel to `requirements`. Advisory: it applies each
    /// requirement independently and does NOT enforce inter-target distinctness
    /// (CR 601.2c / TargetPermanentDistinctFrom) — `process_command` remains the
    /// authority. Implemented by delegating to the existing
    /// validate_{object,player}_satisfies_requirement, never by re-deriving.
    pub fn legal_targets_per_slot(state: &GameState, caster: PlayerId,
                                  source: ObjectId,
                                  requirements: &[TargetRequirement]) -> Vec<Vec<Target>>;
    pub fn target_count_range(requirements: &[TargetRequirement]) -> (usize, usize);

### Wire-format impact — none

No `Command` variant, no `GameEvent` variant, no `Effect` variant, no `GameState` field is
added anywhere in this milestone. `PROTOCOL_VERSION` stays 27, `PROTOCOL_SCHEMA_FINGERPRINT`
stays `f035e797…`, `HASH_SCHEMA_VERSION` stays 63. Sessions 1, 3 and 5 each end by running
the protocol/hash parity tests as a guard. **If any session finds it needs a new `Command`
or `GameEvent` variant, stop and flag** — that is a wire change requiring a PROTOCOL bump
and an entry in the append-only history in `rules/protocol.rs`, and for M11-local it almost
certainly means the design took a wrong turn (the parameterization belongs in
`ActionParams`, which never crosses the wire).

### Hidden-information filtering point (Architecture Invariant 7)

Because `GameEvent::private_to()` is an M10 deliverable that does not exist, M11-local
enforces Invariant 7 at **two chokepoints inside `crates/view-model`**, both consumed only
by `tools/play-server`:

1. `StateViewModel::from_game_state_for(.., Viewer::Seat(p))` — other seats' hands become
   anonymous placeholders (CR 402.1: the hand is a hidden zone); libraries are already
   size-only in the existing view model (Session 4 adds a test that pins that, CR 401.2);
   face-down permanents and face-down exiled cards not owned by the viewer are
   name-redacted.
2. `event_view_for(ev, state, Viewer::Seat(p))` — the server never ships raw serialized
   `GameEvent`s to the browser (which is what the replay viewer does, correctly, because
   it is an omniscient dev tool). It ships rendered, redacted lines.

The `Viewer::Omniscient` path preserves the replay viewer's current behaviour byte for
byte; Session 4 has a regression test for that.

### How bots fill the other seats

`LocalGameConfig { player_count: 4, human_seats: {PlayerId(1)}, bot_kind: Heuristic }`.
`LocalGame::advance()` runs seats 2-4 through `StubProvider::legal_actions` →
`Bot::choose_action` → `process_command`, exactly as `run_game` does today (including the
`CastSpell` auto-tap pre-pass and the invariant check), until the acting player is a human
seat, the game ends, or a limit trips. Default bot is `HeuristicBot` for the web client
(`RandomBot` remains the fuzzer default).

---

## 4. Session breakdown

### Session 1: Steppable local-game core (8 items)

**STATUS (2026-07-26): all 8 items shipped.** `LocalGame`/`LocalGameLimits`/
`AdvanceOutcome`/`HaltReason`/`PendingDecision`/`DecisionKind`/`CommandRecord`/
`LocalGameError`/`HumanChoice` live in `crates/simulator/src/local_game.rs` (new);
`GameDriver::run_game` re-expressed on top of it in `driver.rs`; all 6 named tests
pass (10 after the review pass below) in `crates/simulator/tests/local_game.rs` (new); `cargo build --workspace
--all-targets`, `cargo test --all`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --check`, and `tools/check-defs-fmt.sh` are all green; PROTOCOL 27 /
HASH 63 confirmed unmoved via `core::protocol_schema::protocol_version_sentinel`
and `core::hash_schema::hash_schema_version_sentinel`.

**Review pass (2026-07-26) — 4 MEDIUM API hazards found and fixed before Session 5 puts
HTTP on this surface.** All four were design hazards rather than bugs in the port itself
(the port's behavioural fidelity was verified statement-by-statement and holds):

1. **`advance()` was not idempotent while a decision was outstanding.** It never inspected
   `self.pending`, so a second call re-enumerated actions, bumped `decision_seq` and
   replaced the decision — silently invalidating the `seq` the client was holding, which
   would then fail with `StaleDecision { expected: <a seq it never saw> }`. A poll or
   keepalive endpoint, or a browser refresh, was enough to trip it. Now `advance()` returns
   the outstanding decision unchanged. **Sessions 5-6 may rely on this**: `advance()` is
   safe to call on every request.
2. **A human seat could deadlock on an empty action list.** The `human_seats` check ran
   *before* the empty-legal-actions auto-pass, so a human in a state with no legal actions
   got an `AwaitingHuman` with nothing to click, re-issued forever, with no counter moving.
   `StubProvider` cannot produce that state, but **Session 3 replaces the provider** — the
   seat is now resolved first and the auto-pass applies to humans too.
3. **`submit()` checked only `seq`, not the acting player.** A client could answer its own
   decision with a command naming a different seat. Now refused via `command_player()`.
   **Session 3 should make this structural rather than checked**: once `submit` takes an
   `action_index` into `pending.actions` and builds the `Command` itself for
   `pending.player`, a cross-seat command becomes unrepresentable and the guard is belt-
   and-braces.
4. **The journal was unconditional, including on the fuzzer path**, where the pre-M11
   driver retained nothing — up to `max_turns * 200` records per in-flight game, each with
   a cloned `Vec<GameEvent>`, across thousands of parallel games. Now gated by
   `LocalGameLimits::record_journal`, `false` for `GameDriver`, `true` for the play server.

Also fixed: the `start_game` failure string regained its pre-M11 shape; the module
docstring no longer advertises CR 103.5 mulligan coverage that is not reachable until
Session 2; and the "behaviour is unchanged" claims in `driver.rs` and `CLAUDE.md` were
reworded to state the actual evidence (see the OOS-M11-3 note below).

Session 1's test count went 6 → 10: `test_local_game_repeated_advance_preserves_pending_decision`,
`test_local_game_submit_rejects_command_for_another_seat`, `test_local_game_journal_can_be_disabled`,
and a `command_player` serialization-shape unit test in `local_game.rs`.

**Deviations from the plan text** (all deliberate, none scope-expanding):

1. `LocalGame::start(state, seed, provider, bots, human_seats, limits,
   check_invariants)` takes the pieces directly rather than a `LocalGameConfig` —
   that type is introduced by Session 2's `setup.rs` and does not exist yet.
   Session 2 should add a `LocalGameConfig`-taking constructor alongside it.
2. `GameDriver::run_game` now takes `self` **by value**, not `&mut self`, because
   `LocalGame` owns `provider` and `bots`. Its only caller (`mtg-fuzzer`'s
   `run_single_game`) builds a fresh `GameDriver` per game and never reuses it, so
   the change cost exactly one `mut` removal in `fuzzer.rs`. `GameResult`'s shape is
   unchanged.
3. The item-1/item-8 fuzzer baseline was captured with `--games 50 --threads 1
   --seed 424242` under `RUST_MIN_STACK=536870912`, **not** the plan's literal
   `--games 200`. Reason in the finding below.

**Item-8 finding — proposed seed `OOS-M11-3` (pre-existing, out of scope here)**

Two separate problems surfaced while establishing fuzzer parity, both reproducing on
**pristine, pre-refactor code** (verified via `git stash`):

- `--games 200` stack-overflows even unmodified: some long games build resolution
  chains deeper than the default thread stack. Hence the reduced-and-stack-raised
  baseline above.
- **The fuzzer is not run-to-run deterministic for long games.** Running the *same
  unmodified binary* twice with identical CLI args produced different outcomes for
  seeds 424250, 424280 and 424287. An isolated single-game replay of seed 424250
  produced a byte-for-byte identical command trace before and after the refactor, so
  the 2/50 divergence observed between baseline and post-refactor runs is this
  nondeterminism, **not** a regression from the port.

This is in `crates/engine`, in very long (150-200+ turn) games, and is out of scope
for a `crates/simulator`-only session. It matters beyond the fuzzer: Tier 1 state
hashing and M10a's authoritative server both assume determinism. Rank it with
`OOS-M11-1` / `OOS-M11-2` at collection.

*Independently reproduced during review* (2026-07-26, coordinator): two consecutive
runs of the **same** post-refactor binary, `--games 40 --threads 1 --seed 424242
--bot random`, differed — 30 wins/10 errors vs 29 wins/11 errors, 70719 vs 70692
violations, and a reordered/extra `stack_consistency` violation at turn 157. Single
-threaded with a fixed seed rules out thread scheduling and RNG seeding.

**Lead for whoever picks up the seed** (not chased further here): `crates/engine`
has ~110 uses of std `HashMap`/`HashSet`. `std`'s `RandomState` is seeded **per
process**, so any site where iteration order feeds a decision is nondeterministic
across runs by construction — while `imbl::OrdMap`/`Vector` (which `GameState`
mostly uses, and which `CLAUDE.md`'s risk register credits for determinism) are
ordered. The acting-player path in `advance()` was checked and is clean:
`pending_commander_zone_choices` is an `imbl::Vector`, so its `.iter().next()` is
deterministic. The two positional `.iter().next()` calls on a `HashSet` found in a
scan are both in `testing/replay_harness.rs` (test-only). Start the hunt at the
other ~108 sites, weighted toward SBA and trigger-ordering code, since the observed
divergence was a stack-consistency violation.

**Crate**: `crates/simulator` **only**. No engine change, no HTTP, no async.
**Files**: `crates/simulator/src/local_game.rs` (new), `src/lib.rs`, `src/driver.rs`,
`crates/simulator/tests/local_game.rs` (new — note the SR-9a no-stray-binaries gate is
scoped to `crates/engine/tests/`, so a new integration target here is fine).

1. **Capture a behavioural baseline before touching `driver.rs`**:
   `~/.cargo/bin/cargo run --profile fuzz -p mtg-simulator --bin mtg-fuzzer -- --games 200
   --seed 424242 --bot random --verbose > /tmp/m11-s1-baseline.txt`. This is the
   regression oracle for item 8.
2. Add `LocalGame`, `LocalGameLimits`, `AdvanceOutcome`, `HaltReason`, `PendingDecision`,
   `DecisionKind`, `CommandRecord`, `LocalGameError` (§3 shapes). `LocalGame::start(cfg,
   state, bots)` calls `mtg_engine::start_game` (which enforces Architecture Invariant 9
   via `check_all_defs_complete`) and maps `IncompleteCardsInGame` to a typed error.
3. `advance()` — port the `run_game` loop body **verbatim** in behaviour: game-over check,
   `max_turns`, `max_commands`, `max_consecutive_passes`, acting-player resolution
   (`pending_commander_zone_choices` first, then `turn().priority_holder`, then
   active-player pass — CR 117.3), empty-legal-actions → pass, `CastSpell` auto-tap
   pre-pass via `mana_solver`, per-command `invariants::check_all`, and the
   rejected-command → `PassPriority` fallback **for bot seats only**. Return
   `AwaitingHuman` the moment the acting player is in `human_seats`.
4. `submit(seq, HumanChoice)` — Session 1 accepts a pre-built `Command`
   (`HumanChoice::Command`); full parameterization arrives in Session 3. Validates `seq`
   against `decision_seq`; on engine rejection returns `LocalGameError::Rejected` and
   leaves `self.state` untouched (clone-then-commit, the same shape `process_command` use
   in `driver.rs` already implies). **Never** falls back to `PassPriority`.
5. Re-express `GameDriver::run_game` on top of `LocalGame` with `human_seats` empty:
   `loop { match self.advance() { GameOver(r) => return r, Halted(h) => return h.into(),
   AwaitingHuman(_) => unreachable } }`. Public signature and `GameResult` shape unchanged
   so `fuzzer.rs` and existing callers compile untouched.
6. Journal: every applied command records `CommandRecord`; `journal()` and
   `journal_since(cursor)` accessors (the play server's event feed and the Session 8 bug
   report both read this).
7. **Tests** (`crates/simulator/tests/local_game.rs`):
   `test_local_game_bot_only_matches_game_driver_for_fixed_seeds` (5 seeds, same winner /
   turn count / command count); `test_local_game_halts_awaiting_human_at_first_priority`
   (CR 117.3a); `test_local_game_submit_illegal_command_returns_err_and_preserves_state`;
   `test_local_game_submit_stale_seq_rejected`; `test_local_game_journal_length_matches_commands`;
   `test_local_game_max_consecutive_passes_halts`.
8. Re-run item 1's fuzzer command and diff against `/tmp/m11-s1-baseline.txt` — must be
   identical. Then `~/.cargo/bin/cargo test -p mtg-engine --test core protocol` and the
   hash parity test to confirm PROTOCOL 27 / HASH 63 unmoved.

**Acceptance**: a 4-bot game and a 1-human/3-bot game both drive from the same struct;
zero HTTP involved in any test; fuzzer output unchanged.

---

### Session 2: Deterministic pregame setup and mulligans (7 items)

**STATUS (2026-07-31, `scutemob-161`): all 7 items shipped.** `LocalGameConfig`/`DeckSource`/`BotKind`/
`SetupError`/`build_initial_state`/`redeal` live in `crates/simulator/src/setup.rs` (new);
re-exported from `crates/simulator/src/lib.rs`; `tools/tui/src/play/app.rs::PlayApp::new`
rewired onto `build_initial_state`; `crates/simulator/src/deck.rs` and
`crates/simulator/src/bin/fuzzer.rs` untouched, as required. 10 new tests in
`crates/simulator/tests/setup.rs` — the 7 named below plus three additions (commander registration,
spec enrichment, and the human-seat path); workspace tests
3,928 → **3,938**; `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo fmt --check`, and `tools/check-defs-fmt.sh` all green; PROTOCOL 31 /
HASH 68 confirmed unmoved via the `core::protocol_schema`/`core::hash_schema` sentinel
tests.

**Premise update on item 4 (binding — see the item text below for what changed):** the
plan's original mulligan rationale ("because `handle_take_mulligan` cannot shuffle") is
**stale**. That gap was filed as `OOS-M11-1` and **closed by PB-DP2** (`scutemob-150`,
2026-07-26) — `handle_take_mulligan` now runs a real seeded `Zone::shuffle` and
`handle_keep_hand` bottoms correctly. `redeal` was implemented anyway, per Q1's pregame-UX
rationale (mulligans before `start_game`, no command issued, no history invalidated), with
the doc comment corrected to say so and to cite CR 103.5/103.5c rather than the stale
"CR 103.4b" (that rule is the Vanguard starting life total, not mulligans).

**A bug found and fixed during implementation, not in the plan text:** a naive
`seed ^ mulligan_count ^ seat.0` perturbation for `redeal` collapses back to the
*original* seed whenever the two terms are equal — which happens on the single most
common case, seat 1's very first mulligan (`mulligan_count == 1 == seat.0`), silently
re-dealing the identical hand the player just rejected. Fixed with a splitmix64-style mix
(`redeal_seed` in `setup.rs`) that runs each term through a distinct odd multiplier before
combining. Caught by writing `test_redeal_produces_a_different_hand` honestly rather than
asserting a placeholder.

**Deviation from item 4's literal text:** `redeal` performs only the "shuffle and draw a
fresh 7" half of CR 103.5. The "let the caller nominate `mulligan_count - 1` cards for the
bottom" half needs `ActionParams` (Session 3, not yet built) to express a card selection,
so it is documented as the caller's responsibility once that lands, not implemented here.

**A live correctness bug in the lifted logic, found and FIXED here (not seeded):** the
pre-Session-2 `PlayApp::new` setup — and therefore the first cut of
`build_initial_state` that lifted it verbatim — placed the commander's *object* in
`ZoneId::Command` but never called `GameStateBuilder::player_commander()` or
`register_commander_zone_replacements()`. `PlayerState::commander_ids` stayed **empty**
for every TUI game, and would have for every `LocalGame`. That field is what every
commander rule keys off: commander tax (`casting.rs:765`), the CR 903.9a/704.6d
command-zone-return SBA (`commander.rs:364/488`), CR 903.10a commander-damage tracking
(`combat.rs:1919`), and CR 903.9b's hand/library redirect replacements
(`replacement.rs:477`, registered by `builder.rs:1198`). None of them fired. The result
is a game that *looks* like Commander — there is a card in the command zone — while the
commander is recastable for free forever, deals no commander damage, and is never
returned to the command zone.

This was initially written up as a seed on the grounds that item 2 scopes
`build_initial_state` to lifting `app.rs`'s logic rather than improving on it. That was
the wrong call and it is fixed instead: the milestone's entire purpose is a *playable
Commander game* (Architecture Invariant 6), a pregame builder that does not register
commanders does not deliver one, and the fix is two calls to existing public engine API
(`builder.player_commander(pid, deck.commander)`, then
`register_commander_zone_replacements(&mut state)` after `build()`) with **zero** engine
edits — the same pairing `testing/replay_harness.rs:257-274` already uses on the script
path. Pinned by an 8th test, `test_setup_registers_commanders_not_just_places_them`,
which asserts one registered `commander_ids` entry per seat, that the registered `CardId`
is the card actually in that seat's command zone, and that the 2-per-commander CR 903.9b
replacements exist before the game starts.

**This is also a TUI behaviour change**, and a deliberate one: `mtg-tui`'s play mode has
been running non-Commander games under a Commander UI, and now runs real ones (tax
applies, commander damage accrues, the commander returns to the command zone). Nothing
else about the TUI's behaviour changed.

**Crate**: `crates/simulator` (+ a call-site swap in `tools/tui`).
**Files**: `crates/simulator/src/setup.rs` (new), `src/deck.rs`, `src/lib.rs`,
`tools/tui/src/play/app.rs`, `crates/simulator/tests/setup.rs` (new).

1. `LocalGameConfig`, `DeckSource`, `BotKind`, `SetupError` (§3). Seeded `StdRng`
   throughout — the same seed must reproduce the same game.
2. `build_initial_state(cfg)` — for each seat: `random_deck` (already `Complete`-only per
   SR-12) or a fixed `DeckConfig`; commander → `ZoneId::Command`; **shuffle `main_deck`
   with the seeded RNG**; first 7 → `ZoneId::Hand` (CR 103.5/402.1 — **corrected**, the plan originally said "CR 103.4", which is the starting LIFE TOTAL; the engine deals no
   opening hand, §1 fact 2); remainder → `ZoneId::Library`; every spec through
   `enrich_spec_from_def`; `first_turn_of_game()`. This is the TUI's `app.rs:122-165`
   logic, lifted and made testable.
3. **Deck admission through the real gate** (Architecture Invariant 9): call
   `mtg_engine::validate_deck(&[commander], &main_deck_plus_commander, &registry, &[])`
   and refuse to build on any `DeckViolation` (CR 903.5a 100-card check included). Assert
   the 99+1 contract `random_deck` produces. `start_game`'s `check_all_defs_complete` stays
   as the second, independent line of defence.
4. `redeal(cfg, seat, mulligan_count)` — the mulligan implementation (CR 103.5, CR 103.5c
   free first mulligan in multiplayer). A mulligan is a **pregame rebuild**: re-shuffle
   under a perturbed seed, re-deal 7, and let the caller nominate `mulligan_count - 1`
   cards for the bottom. This happens strictly before `start_game`, so no command has been
   issued and no history is invalidated.
   > **Two premises in this item were dead by the time it shipped — corrected in place so
   > they stop propagating.** (a) *"Because `handle_take_mulligan` cannot shuffle (§1 fact
   > 1)"* — **false since PB-DP2** (`scutemob-150`), which is also what §8 R2 records; the
   > engine's in-game mulligan is CR 103.5-faithful, and `redeal` is kept for the *pregame
   > UX* reason in Q1 (mulligans before `start_game`), not as a correctness workaround. Do
   > not re-file R2. (b) *"re-shuffle with `seed ^ mulligan_count`"* — **do not do this**:
   > the two terms cancel whenever they are equal, and `seat 1 / mulligan 1` is the single
   > most common case, so it silently re-deals the identical rejected hand. Shipped as
   > `redeal_seed`, a splitmix64-style mix; see its doc comment. Also note the shipped
   > `redeal` implements only the re-deal half — bottoming needs `ActionParams` (Session
   > 3) — and carries two documented limitations of the whole-table rebuild (public
   > command zone, and no representation of a partially-decided table).
5. Rewire `tools/tui/src/play/app.rs::PlayApp::new` to call `setup::build_initial_state`
   — deletes ~45 lines of duplicated setup and gives the TUI deck validation for free.
   TUI behaviour otherwise unchanged.
6. **Leave `crates/simulator/src/bin/fuzzer.rs` alone, deliberately.** Its games start with
   empty hands; changing that changes every fuzz result and invalidates recorded seeds.
   Add a comment in `setup.rs` saying so.
7. **Tests** (`crates/simulator/tests/setup.rs`):
   `test_setup_deals_seven_card_opening_hand_per_seat` (CR 103.5 — corrected from the plan's original "CR 103.4");
   `test_setup_library_holds_the_remainder`; `test_setup_same_seed_same_state_hash`;
   `test_setup_different_seed_different_opening_hand`;
   `test_setup_rejects_deck_with_non_complete_card` (Invariant 9);
   `test_redeal_produces_a_different_hand` (CR 103.5);
   `test_setup_commander_starts_in_command_zone` (CR 903.6).

**Acceptance**: `build_initial_state(seed)` is deterministic and reproducible; a human seat
starts with 7 real cards; no non-`Complete` card can enter a game.

---

### Session 3: Action parameterization + engine target queries (8 items)

**STATUS (2026-08-01, `scutemob-163`): all 8 items shipped — the milestone's crux (§8 R1)
is closed.** A human can cast a targeted spell:
`crates/simulator/tests/local_game.rs::test_human_casts_targeted_spell_through_local_game`
casts one through `LocalGame::submit`, picking its target through the new engine query
surface, and asserts the damage **resolved**. Workspace tests 3,955 → **3,965 / 0**;
`cargo build --workspace`, `clippy --workspace --all-targets -D warnings`, `fmt --check`
and `tools/check-defs-fmt.sh` all green; PROTOCOL **32** / HASH **69** unmoved (the pins
moved from 31/68 on the W6 track via PB-DX1, not here — this session added no
`Command`/`GameEvent`/`Effect` variant and no new public type, and its diff over
`rules/protocol.rs` and `crates/card-types/` is empty).

**Three things the item text below did not contain.**

1. **Item 1's signature is wrong and shipped corrected.** `spell_target_requirements`
   takes a 4th parameter, `alt_cost: Option<AltCostKind>`. `casting_with_overload`
   (`casting.rs:1163`) and `casting_with_aftermath` (`casting.rs:533`) are **caster-intent**
   flags read off the `CastSpell` command — they are not derivable from `GameState`, so
   without the parameter CR 702.96b is unreachable and item 8's
   `test_702_96b_overload_reports_no_target_requirements` is unwritable. `AltCostKind` is
   already public, so no new public type is introduced.
2. **The shared extraction, not the query, is item 1's load-bearing half** — and it is
   worth more than the item implies. `casting.rs` now exports
   `card_def_target_requirements`, `spell_mode_selection` and
   `per_mode_target_requirements`, extracted verbatim (refactor-only, zero behaviour
   change), so the query and the cast path cannot drift.
3. **A gate-churn trap.** The first cut established Overload eligibility by reading
   `KeywordAbility::Overload` off layer-resolved characteristics. That is a *parallel
   re-derivation* of `get_overload_cost(...).is_some()` (`casting.rs:1203`), and it forced
   an SR-5 reclassification of Overload from `Marker` to `Handled`, dragging
   `keyword_registry.rs`, its gate test and `docs/sr-5-keyword-catchall-audit.md` with it.
   Replaced with the same call casting makes; the three collateral files reverted.
   **Generalisable: an SR-5 reclassification forced by a new read is a signal you
   re-derived something instead of delegating to it.**

**Also shipped, beyond the item text:** `HumanChoice` became a struct (item 7), so
Session 1's `command_player` cross-seat runtime guard and its unit test are **deleted** —
the guarantee is structural now, exactly as `submit`'s S1 doc comment predicted. The
tap-then-cast sequence applies to a clone and commits only on full success. Item 6's
parity was *measured*, not asserted: `mtg-fuzzer --games 50 --seed 424242 --bot random`
built at pristine and refactored code produced byte-identical per-seed
Turns/Commands/Winner/Error across all 50 seeds. **`OOS-M11-2`'s pool half is closed;
its layer-resolution half (`mana_solver.rs:35`) remains open and unowned.**

> **Fixture trap for Sessions 4-8:** you cannot pre-fill a player's mana pool before
> `LocalGame::start` and expect it to survive — `start_game` runs through Untap/Upkeep and
> **CR 500.4 empties the pool between steps**. Fund the pool with a real `TapForMana`
> submit inside the same step instead.

> **And treat this section's CR cites as unverified** — Session 2 found `CR 103.4` used
> for the opening hand across six sites when 103.4 is the starting *life total*. That
> family of stale cite has now bitten twice.

**Crates**: `crates/engine` (read-only query module) and `crates/simulator`.
**Files**: `crates/engine/src/rules/queries.rs` (new), `src/rules/mod.rs`, `src/lib.rs`,
`src/rules/casting.rs` (visibility widening only), `crates/simulator/src/params.rs` (new),
`src/random_bot.rs`, `src/local_game.rs`, `crates/engine/tests/rules/queries.rs` (new —
**must be added to `crates/engine/tests/rules/main.rs` as a `mod` line**, or SR-9a's gate
fails and the tests silently vanish).

1. Engine: `spell_target_requirements` — mirror the lookup at `casting.rs:3563-3611`
   (Aftermath CR 702.127a, Overload CR 702.96b → empty, per-mode `mode_targets`
   CR 700.2c/700.2f). Extract that block into a shared private helper so casting and the
   query cannot drift.
2. Engine: `ability_target_requirements` (CR 602.2b) from layer-resolved characteristics —
   `calculate_characteristics(state, source)`, never `card_registry.get()` (standing
   gotcha: registry reads bypass Humility/Dress Down and Layer 6 grants).
3. Engine: `legal_targets_per_slot` — for each requirement, enumerate candidate
   `Target::Object` over all objects in targetable zones plus `Target::Player` over
   active players, keeping those that pass the existing `pub(crate)`
   `validate_object_satisfies_requirement` / `validate_player_satisfies_requirement`.
   Delegate; never re-derive. Doc-comment the advisory caveat (no inter-target
   distinctness) and that `process_command` stays authoritative.
4. Re-export the four query fns from `crates/engine/src/lib.rs`. No new public *type* is
   introduced (returns are `Vec<TargetRequirement>` / `Vec<Vec<Target>>`), so the wire
   fingerprint cannot move.
5. Simulator: `ActionParams`, `HumanChoice`, `ParamError`,
   `action_to_command_with_params`. **It must forward `hybrid_choices` and
   `phyrexian_life_payments` verbatim from the `LegalAction`** the provider produced
   (PB-RS2 precedent — re-deriving them is the OOS-RS-2 drift class), and reject a
   `TapForMana` on an `any_color` ability with no `chosen_color` rather than defaulting to
   `Colorless` (PB-EF12, CR 106.1a/106.1b).
6. Refactor `random_bot::action_to_command` to call
   `action_to_command_with_params(.., &ActionParams::default())` so there is exactly one
   `LegalAction` → `Command` mapping table in the codebase.
7. `LocalGame::submit` takes `HumanChoice { action_index, params }`, resolves
   `action_index` against the pending decision's `actions`, and builds the command.
   Auto-tap becomes conditional: only when `params.auto_tap` **and** the player's existing
   `ManaPool` cannot already cover the cost (today `solve_mana_payment` ignores the pool
   entirely — §1, `mana_solver.rs`).
8. **Tests**: engine `rules::queries::` —
   `test_601_2c_legal_targets_excludes_shroud_and_protected_creatures`,
   `test_601_2c_legal_targets_includes_players_for_target_player`,
   `test_601_2c_target_count_range_up_to_n`,
   `test_702_96b_overload_reports_no_target_requirements`;
   simulator — `test_human_casts_targeted_spell_through_local_game`,
   `test_human_illegal_target_is_rejected_without_state_change`,
   `test_hybrid_payment_plan_is_forwarded_verbatim`,
   `test_auto_tap_skipped_when_pool_already_covers_cost`.
   Close with the protocol/hash parity tests.

**Acceptance**: a human can cast a targeted spell — the single capability whose absence
makes the existing TUI bridge unusable for real play.

---

### Session 4: View-model crate extraction + seat redaction (6 items)

**STATUS (2026-08-01, `scutemob-165`): SHIPPED — all 6 items done.**
`crates/view-model` (package `mtg-view-model`) exists with `lib.rs` (git-recorded rename of
`tools/replay-viewer/src/view_model.rs`, 91% similarity), `redact.rs`, `event_view.rs`,
`tests.rs` and `golden_omniscient_view.json`. All 6 named tests pass and each was proven
non-vacuous by mutation. `cargo build --workspace`, `cargo clippy --all-targets -- -D
warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` and `cargo test --all` are green;
tests 3,988 → **3,998**. `git diff main -- crates/engine crates/card-types crates/card-defs`
is **empty**; PROTOCOL 32 / HASH 69 confirmed by the `core` sentinels.

**The golden snapshot was captured BEFORE the move, from pristine code** (commit
`56d44177`), by running a deterministic 4-player fixture through the untouched
`tools/replay-viewer/src/view_model.rs` and then restoring that file byte-for-byte. It is
therefore a real regression guard rather than a post-hoc record of whatever the new code
does. It is compared as `serde_json::Value`, never as a string, because `StateViewModel`
uses `std::collections::HashMap` and its iteration order is randomized per process.

Deviations from the plan text, all recorded in source:

- **`event_view_for` takes a 4th parameter**, `player_names: &HashMap<PlayerId, String>`.
  The plan's sketch is `(ev, state, viewer)`, but `GameState` carries `PlayerId`s only, so
  every rendered line would read `player_2` and the caller would have to re-render — which
  would put string formatting back *outside* the redaction chokepoint. Display names are
  public, so the parameter carries no hidden data. (Same class as S3's `alt_cost`.)
- **The face-down *battlefield* redaction is belt-and-braces, and the real leak was in
  exile.** The golden snapshot captured from pristine pre-move code already shows the
  face-down morph with `"name": ""`, because `build_zones_view` runs each permanent through
  `calculate_characteristics` and the layer system applies the CR 708.2a override for
  everyone. The face-down *exiled* card leaked its printed name, because
  `objects_in_zone_as_card_views` reads `obj.characteristics.name` raw with no layer pass.
  Both are redacted explicitly at the redact layer so Invariant 7 does not silently depend
  on the layer system; the source records which of the two is live.
- **The golden snapshot was regenerated exactly once**, for the additive `hidden` field. A
  structural diff of old vs new is 12 differences, every one of them `ADDED hidden: false`
  (one per `CardInZoneView` in the fixture) — no other delta.
- **Item 3's plan text is right that the redaction keys on `obj.owner`**, and that is the
  conservative choice rather than the strictly correct one for the battlefield: CR 708.5a
  says a player who *controls* a face-down permanent may look at it, so a thief is denied a
  name they are entitled to. Denying too much never leaks. Noted in `redact.rs`.
- **`GameEvent::SpellCast` carries two `ObjectId`s and only one of them resolves.** The
  first implementation rendered the spell's name from `stack_object_id`, which never
  matches: `handle_cast_spell` mints `stack_entry_id = state.next_object_id()`
  (`rules/casting.rs:4401`) solely to build the `StackObject` pushed onto
  `state.stack_objects()` (`:4529`), and that id is never inserted into `state.objects()`.
  The lookup missed every time, so **every** cast degraded to the name-free fallback
  "alice casts a spell" — never wrong, never present, and indistinguishable from correct
  redaction, which would have quietly made the Session 6 event feed useless for the most
  common action in the game. Fixed to `source_object_id` (the card's new object in
  `ZoneId::Stack`, `:4732`) and pinned by a test proven non-vacuous against the old
  version. The three sibling arms that call `card_name` were audited the same way against
  their emission sites and were already right: `CardDrawn.new_object_id` →
  `Hand(player)` (`replacement.rs:1069`), `LandPlayed.new_land_id` → `Battlefield`
  (`lands.rs:391`), `CardDiscarded.new_id` → `Graveyard(player)` (`casting.rs:4098`).
  Generalisable: in an event-rendering layer, a failed id lookup and a deliberate
  redaction produce the *same* output, so a lookup bug hides inside the privacy
  behaviour. Every `card_name` call site needs its id checked against the emission site,
  not just its entitlement rule reviewed.

**Review cycle (1 HIGH / 1 MEDIUM / 4 LOW, all applied).** The HIGH is the durable one:
**redaction follows the rendering site, not the zone.** The first cut redacted `zones.hand`,
`zones.battlefield` and `zones.exile` — the zones CR calls hidden — and stopped. Four other
sites in `lib.rs` read `obj.characteristics.name` **raw**, with no layer pass and no
entitlement check, and each can be handed a face-down object: `StackItemView::source_name`,
`format_target`, `AttackerView::name` (plus its planeswalker `target`), and
`BlockerView::name`. A morph creature that attacks *is* on the battlefield, so the
battlefield redaction "covered" it in the zone sense while `combat.attackers[i].name`
printed its name to the whole table — the same CR 708.2 violation this session fixed one
zone over, one surface across. `redact_stack` and `redact_combat` now route those four sites
through the already-correct `redact::viewer_may_identify`.

The MEDIUM is *why the HIGH was invisible*: every leak scan viewed from alice's seat, and
alice is the one player whose hand card the fixture also puts on the stack, so her own card
names were never needles. The whole-document-scan technique was right; its instantiation was
one-sided. `test_no_seat_view_leaks_any_other_seats_hidden_card` now loops all four seats.
Proven non-vacuous: with the two new redactions disabled it fails on `seat 2: leaked
"Lightning Bolt"` **while all six plan-named tests stay green**, which is exactly the
blindness. A second added test puts a face-down creature into combat, because the golden
fixture has only face-up attackers and cannot be changed (it is pinned by the pre-move
snapshot) — without it `redact_combat` would have been present, green and unexercised.

**Item 6 (done)**: `memory/gotchas-infra.md` now points the two exhaustive matches
(`StackObjectKind` in `stack_kind_info()`, `KeywordAbility` in `format_keyword()`) at
`crates/view-model/src/lib.rs`, with a MOVED note so a reader arriving from one of the
several historical docs that still cite the replay-viewer path knows it is stale rather
than that they misread. The same pointer in the session-loaded auto-memory index
(outside the repo) was updated too.

**Crates**: new `crates/view-model`; `tools/replay-viewer` becomes a consumer.
**Files**: `crates/view-model/Cargo.toml`, `src/lib.rs`, `src/redact.rs`,
`src/event_view.rs`, `src/tests.rs`; `tools/replay-viewer/src/{main.rs,api.rs,replay.rs,
Cargo.toml}`; root `Cargo.toml` (workspace member).

1. Move `tools/replay-viewer/src/view_model.rs` (895 lines) verbatim into
   `crates/view-model/src/lib.rs` as package `mtg-view-model` (deps: `mtg-engine`,
   `serde`, `serde_json`; `[lints] workspace = true`). Replay-viewer imports it. This must
   be behaviour-neutral: all existing replay-viewer tests pass unchanged.
2. `Viewer` enum + `StateViewModel::from_game_state_for`; keep `from_game_state` as an
   `Omniscient` shim so no call site changes.
3. Redaction in `redact.rs` (Architecture Invariant 7): other seats' hands → placeholder
   `CardInZoneView { object_id: 0, name: "Hidden card", card_types: [], hidden: true }`
   (CR 402.1) — an additive `hidden` field keeps the existing `ZoneHand.svelte` contract
   intact; assert libraries stay size-only (CR 401.2); redact names of face-down
   permanents and face-down exiled objects not owned by the viewer (confirm the exact
   status field — `ObjectStatus` / `FaceDownKind` — while implementing).
4. `event_view.rs`: `EventView` + `event_view_for(ev, state, viewer) -> Option<EventView>`.
   Conservative rules: events naming a card in a hidden zone of another seat return either
   `None` or a name-free line; a `_ =>` arm emits a kind-only line and **must not**
   interpolate any card name. Doc-comment it as the stand-in for the M10 `private_to()`
   deliverable so M10a knows where to look.
5. **Tests** (`crates/view-model/src/tests.rs`):
   `test_seat_view_hides_other_players_hand_card_names`;
   `test_seat_view_never_enumerates_any_library` (CR 401.2);
   `test_seat_view_shows_own_hand`;
   `test_event_view_redacts_other_seats_card_draw`;
   `test_omniscient_view_is_unchanged_for_fixture_state` (JSON equality against a golden
   snapshot captured before the move — the replay-viewer regression guard);
   `test_seat_view_hides_face_down_permanent_name`.
6. Update `memory/gotchas-infra.md`: the two exhaustive matches (`StackObjectKind` in
   `stack_kind_info()`, `KeywordAbility` in the keyword display fn) now live in
   `crates/view-model/src/lib.rs`, not `tools/replay-viewer/src/view_model.rs`, and
   `cargo build --workspace` is still the way to catch a missed arm.

**Acceptance**: one view-model implementation feeds both hosts; a seat view provably
cannot leak another player's hand or any library order.

---

### Session 5: play-server crate skeleton + REST API (8 items)

**STATUS (2026-08-01, `scutemob-167`): SHIPPED — all 8 items done, three review cycles applied.**
`tools/play-server` (package `play-server`) exists as a workspace member with
`Cargo.toml`, `src/{main.rs,api.rs,session.rs,view.rs}` and `README.md`. All 8 named tests
pass through `tower::ServiceExt::oneshot` and **no port is ever bound** —
`TcpListener`/`axum::serve` appear only inside `async_main`, which no test calls, and that
is now **machine-enforced across every `.rs` file in the crate** rather than reviewed (see
fix cycle 2; fix cycle 1's gate read `main.rs` alone and cut in the wrong place). Stated
precisely, after fix cycle 3: a `src/` file is either checked, or the gate goes **red naming
it** — it is not the case that every arrangement of test code is silently understood, and
fix cycle 2's claim that a file Sessions 6/7 add is "covered without editing this test" was
false for two real spellings (see fix cycle 3, MEDIUM 1).
`cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804 defs) and `cargo test --workspace`
are green; tests 4,008 → 4,016 → 4,023 → 4,024 → **4,026** across the three fix cycles. `git diff main --
crates/engine/src crates/card-types/src crates/card-defs/src` is **empty**;
PROTOCOL 32 / HASH 69 unmoved; `crates/simulator` and `crates/view-model` are untouched —
the session needed nothing added to either, and the fix cycle kept it that way even where
the root cause lives in `LocalGame` (see MEDIUM 1).

**Fix cycle (2026-08-01) — 2 MEDIUM / 5 LOW, all applied; 8 named tests unrenamed, +7 new.**

- **MEDIUM 1 — the `seq` anti-stale-tab guarantee did not survive a session rebuild, and
  that was the milestone's own claim.** `LocalGame::start` sets `decision_seq: 0`
  unconditionally, and the play server calls it on **both** `POST /api/game` and
  `POST /api/game/mulligan` — so game B's first decision reused game A's `seq: 1` and
  `submit` *matched* it. **Observed, not reasoned to**: the pre-fix run answered a stale
  tab's post with **200** and moved the new game's `command_count` 0 → 4, on both paths.
  Fixed in the play server, not the simulator (`decision_seq` is private and
  `crates/simulator` is out of scope): `PlaySession` gained a `seq_base`, set on each
  rebuild to one past the highest **wire** `seq` the previous session ever issued, with
  `checked_sub` on the way in — never a bare subtraction on a client-supplied `u64`. The
  wire `seq` is now monotonic for the life of the process; it *skips* a value per rebuild,
  which nothing depends on. `view::decision_view` takes the wire `seq` as a parameter
  rather than reading `decision.seq`. *(Corrected by fix cycle 2, LOW 12: that is a
  **convention**, not a guarantee — `wire_seq` is a bare `u64`, and nothing stops a caller
  passing `decision.seq`. What holds it is that `api.rs::seat_view` is the sole caller and
  builds the pair with a `zip`. Making it structural would need a newtype.)*
- **MEDIUM 2 + LOW 5 — axum's own extractor rejections bypassed the JSON envelope, and
  collided with the documented meaning of 422.** `JsonDataError` is 422 with a
  `text/plain` body and no `kind`; this crate documents 422 as "the *engine* refused the
  command". **Observed**: `{"params":{"target":[]}}` (a typo) came back
  `422 Failed to deserialize the JSON body into the target type: …` as plain text, and
  `POST /api/game {"playerz":9}` came back **200 with a full default four-player game**,
  because `Option<T>`'s `FromRequest` impl is `T::from_request(..).ok()`. All three
  body-taking handlers now extract `Result<Json<T>, JsonRejection>` and re-wrap: data
  errors → **400** `invalid_body`, syntax → 400 `malformed_json`, content-type → 415
  `unsupported_media_type`. An **absent** body on `POST /api/game` still means "use the CLI
  defaults" (that arm is `MissingJsonContentType`), and both halves are tested.
- **LOW 4** — the `seat_view` comment claimed no omniscient entry point is "reachable from
  this crate"; `main.rs`'s test module reaches `from_game_state` deliberately as test 7's
  oracle. Now says "the **production paths** of this crate", matching the README.
- **LOW 6** — `POST /api/game`, and only it, recovers from a poisoned session mutex
  (`PoisonError::into_inner()` + `clear_poison()`). Sound because that handler discards the
  corrupt session outright; the one thing it reads out of the poisoned value is
  `next_seq_base()`, which is what keeps MEDIUM 1's guarantee true through a panic. Every
  other route that takes the lock keeps its 500, asserted in the same test. *(Two
  corrections from fix cycle 2. The recovery as shipped here was **not atomic** — see fix
  cycle 2 MEDIUM 1. And `next_seq_base()` reads **one** `u64`, `highest_wire_seq`, not two:
  `seq_base` is never read across the recovery (LOW 9).)*
- **LOW 7** — `GameSummary.seed` is the **base** seed and stops describing the table after
  a mulligan (`setup::redeal` builds from `redeal_seed(seed, seat, count)`). The
  reproduction key is `seed` + `players` + `bot` + `mulligan_count`, all four already in
  every `GameSummary`; documented in the README and on the field rather than duplicating a
  derivation that is private to `mtg_simulator::setup` and could drift.
- **LOW 8** — the no-socket rule is now a gate:
  `test_no_socket_symbol_appears_in_the_test_region` fails on `TcpListener`, `axum::serve`,
  `bind` or `async_main` inside a test region. Needles are assembled with `concat!` so the
  gate does not match its own source. **Proven non-vacuous by execution**: inserting
  `let _gate_probe = "TcpListener";` into `test_healthz_ok` reddened it with the symbol
  named in the message; removed, green. *(Fix cycle 2 rewrote the mechanism, MEDIUM 2 /
  LOW 5 / LOW 6: as shipped here it read `src/main.rs` alone via `include_str!` and cut at
  the **first textual** `#[cfg(test)]`, which is the one written out in that file's own
  module doc comment — so the region began at a paragraph of prose, and the "each needle
  occurs above the cut" guard was satisfiable by that same paragraph. Both are fixed.)*
- **LOW 9** — `NoPendingDecision` → 409 now has the test plan item 6 implied.
  `test_post_action_after_game_over_returns_409` plays a two-seat table to a real CR 104.2a
  conclusion (~1,000 human actions, ~3 s) because that is the **only** route the HTTP
  surface has to that arm: `LegalAction::Concede` is deliberately never offered by the
  provider, and `HaltReason::MaxTurns` needs 200 turns rather than ~110. Nothing reaches
  into `PlaySession` to manufacture the state.

**Fix cycle 2 (2026-08-01) — a re-review: 4 MEDIUM / 6 LOW, all applied; 8 named tests
unrenamed, +1 new (16 in the module).** All nine of fix cycle 1's findings were confirmed
addressed; these are new, and the first is a **regression fix cycle 1 introduced**.

- **MEDIUM 1 — the poison recovery was not atomic, and the "fix" made one route *worse*
  than before it.** Fix cycle 1 called `clear_poison()` *before* `session::new_game(..)?`.
  `new_game` is fallible on a **client-supplied seed**: `deck::basics_for_colors` pads a
  colourless commander's deck with Forests, and `validate_deck` refuses them under CR
  903.5c (**filed as `OOS-M11-6`** — `random_deck` applies the colour-identity filter to the
  main deck (`deck.rs:68`) and then bypasses it 37 lines later when padding to 99
  (`deck.rs:105-110`); and the call site's own dead `if basics.is_empty()` arm names Wastes in
  a comment and pushes Forest anyway, so the code states the correct fix twice and does
  neither. The third audit confirmed the fuzzer half: `driver.rs` references no deck,
  `bin/fuzzer.rs:296` calls `random_deck` and builds from it with no `validate_deck` in the
  file, so those decks are *played* there rather than refused). Its `?` skipped `*guard = Some(play)`,
  so the half-mutated session survived with
  the flag cleared and the next `GET /api/game` answered **200** — where before fix cycle 1
  it answered 500. **Reproduced empirically before the fix was written**: a sweep found 7
  failing tables in 180 `(players, seed)` pairs (`players: 2, seed: 17` among them), and the
  new test, run against the pre-fix ordering, got a full 200 seat view of the poisoned
  session. Fixed **structurally, not by reordering**: the recovery arm `take()`s the corrupt
  session out of the `Option` in the same straight-line block that clears the flag, with no
  fallible operation between, so "flag clear" and "no untrustworthy session left" are one
  fact. The healthy arm still only *peeks*, so a failed rebuild does not destroy a running
  game. Pinned by `test_poison_recovery_is_atomic_when_the_rebuild_fails`; four documents
  that asserted the old (false) property are corrected.
- **MEDIUM 2 + LOW 5 + LOW 6 — the no-socket gate cut at its own doc comment, read one
  file, and had a prose-satisfiable non-vacuity guard.** The cut was a bare
  `source.find("#[cfg(test)]")`, whose first match is the **prose** occurrence in the test
  module's doc comment, not the attribute below it; it passed only because all four needles
  happen to be typed to the left of the marker in that one sentence, so rewording the
  paragraph would have turned the gate red against its own documentation. Now the cut is
  **line-anchored** (first non-whitespace text on the line, which a `///` line can never
  be), the gate walks **every `.rs` file** under `src/` and `tests/` rooted at
  `CARGO_MANIFEST_DIR`, a `tests/` file is checked in full (an integration test has no
  `#[cfg(test)]` attribute at all), and the non-vacuity guard searches
  **comment-stripped** code. **Three mutation proofs, all observed**: (a) a needle inserted
  into a `#[cfg(test)] mod` in `src/session.rs` — a file the old gate never read — reddened
  it, naming the file; (b) a `tests/tmp_probe.rs` containing `axum::serve` stayed *green*
  against an intermediate version that looked for the attribute everywhere, which is how the
  `tests/`-is-whole-file rule was found, and reddened after; (c) renaming the serving entry
  point in **code only**, leaving both prose mentions, reddened guard (c) — the old guard
  would have stayed green. The gate also caught a plainly-written needle in a draft of its
  own new doc comment, which was its first real find.
- **MEDIUM 3 — a documented error row was unreachable.** See the "errors the plan does not
  enumerate" bullet below: `engine_error` cannot be produced through
  `From<LocalGameError>`, and the row's description of *what* would produce it was wrong
  too. Kept and labelled rather than silently collapsed.
- **MEDIUM 4 — `setup_failed` → 422 violated the crate's own 400/422 rule.** Resolved by
  **restating the rule**, not by changing the status: *400 means this crate refused the
  request before any engine code judged it; 422 means engine code looked at what the request
  asked for and said no — `process_command` for a command, `validate_deck` for a pregame
  table.* `POST /api/game {"seed": 17}` is syntactically perfect and fails on the *table the
  seed builds*, which is the textbook 422; `bad_player_count` stays 400 and is the contrast.
- **LOW 7** — five reachable `kind` values were missing from the README's branch contract
  (`setup_failed`, `start_failed`, `bad_player_count`, `not_pregame`, `bad_bot_kind`), and
  the `bad_params` row credited only `ParamError::UnsupportedParam` when `post_mulligan`
  emits the same tag for a non-empty `cards_to_bottom` without going near `LocalGame::submit`.
- **LOW 8** — "the only place tokio is named" was false and self-contradicting (`main.rs`
  builds the runtime; every `#[tokio::test]` names it; the README row directly above said so).
  Replaced with the formulation `session.rs` already used: **nothing below `api.rs`
  references tokio.**
- **LOW 9** — "the two `u64` wire-`seq` counters" is **one**: `next_seq_base()` reads only
  `highest_wire_seq`; `seq_base` is never read across the recovery.
- **LOW 10** — two stale test-count claims, both corrected by re-running rather than by
  arithmetic. See the test-7 paragraph.
- **LOW 11** — `OOS-M11-5`'s "no `target` anywhere in its oracle text" is false; corrected
  in `docs/audits/decision-point-audit.md` §8.1.
- **LOW 12** — smaller drifts: the "every route answers 500 when poisoned" claim (false for
  `GET /api/healthz` and for the pre-lock 400 paths), the router-404 claim (true only in the
  test configuration where `dist/` is absent, and held by no test — the "verified by request"
  claim is dropped), the `decision_view` "cannot be forgotten" claim (a convention, not a
  guarantee), and `DecisionView.player` missing from the deviations list.

**Fix cycle 3 (2026-08-01) — a third audit, scoped to whether fix cycle 2 repeated fix
cycle 1's mistake of shipping a false proof: 1 MEDIUM / 6 LOW, all applied; 8 named tests
unrenamed, +2 new (18 in the module).** It did **not** repeat it — the poison-recovery
atomicity repair is genuinely structural, its test is genuinely non-vacuous, and the
`engine_error` unreachability claim survives independent verification. **No correctness
defect in shipped behaviour was found.** What the audit found was one gate-coverage hole and
six documentation overstatements, **three of them inside text written to correct
documentation drift**.

- **MEDIUM 1 — the no-socket gate silently skipped any file whose `cfg(test)` spelling it
  did not recognise.** `test_region` matched only the literal `#[cfg(test)]` as a line's
  first non-whitespace text; anything else returned `""` and the loop `continue`d **with no
  signal**, guarded only by `files_with_a_region >= 1`, which `main.rs` alone satisfies
  forever (the count was literally **1**, because `api.rs`/`session.rs`/`view.rs` contain no
  `cfg(test)` at all). Fix cycle 2's claim that "a file Session 6 or 7 adds is covered
  without editing this test" was therefore **false for two real arrangements**, and Sessions
  6/7 are exactly when new files arrive. **Both were reproduced before anything was written**:
  `src/probe_split.rs` (the `#[cfg(test)] mod tests;` split — the body file carries no
  attribute) and `src/probe_cfg_all.rs` (`#[cfg(all(test, feature = "x"))]`), each containing
  `tokio::net::TcpListener::bind`, left the gate **green**; the run did not even recompile,
  since the gate reads source from disk. Fixed two ways: `test_region` now recognises the
  `#[cfg(all(test` prefix, and a `src/` file whose **code** is test-shaped (`#[test]`,
  `#[tokio::test`, `fn test_`, `mod tests`) while its region is empty is now a **failure**
  naming the file, not a skip. Post-fix, probe A reddened with the "test-shaped code but no
  test region this gate recognises" message and probe B reddened through the ordinary region
  path naming `TcpListener`; both probes removed. The coverage claim is restated in the
  README, the plan and the gate's own doc comment as *either checked, or red naming the
  file* — the weaker sentence, which is the true one. **The gate again caught a draft of its
  own documentation**: the new doc text named `async_main` in prose below the cut and went
  red, which is the second time it has found a fault in its own comments.
- **LOW 2 — guard (c)'s `bind` needle was satisfiable by a string literal.** The stripper
  removed only whole lines whose `trim_start()` began with `//`, so the serving path's own
  `format!("Failed to bind to {addr}")` — a *code* line — supplied the fourth needle by
  itself, and the guard would have stayed green if the real call were renamed away. It also
  handled neither `/* */` block comments (converting the module doc block to that form
  restores the exact vacuity the guard exists to close) nor `//` inside string literals.
  Replaced with `code_only`, which blanks comment bodies and string-literal bodies, handling
  line comments anywhere on a line, **nested** block comments, raw strings at any hash count,
  and char literals (`'"'` and `'\\'` included — a naive scanner desynchronises on them).
  **Proven both ways by execution**: with the real call replaced by an
  `unimplemented!("{}", format!("Failed to bind to {addr}"))` that keeps the message and
  drops the call, the **old** line-comment-only stripper ran **green** and the **new** one
  ran **red** on the `bind` needle. Mutation reverted verbatim (`git diff` over
  `async_main` empty). Residual stated rather than glossed: this is a lint over source text,
  not a Rust lexer, so macro-generated text is invisible to it by construction. **And the
  branch-coverage claim was checked rather than assumed**: the only inputs `code_only` ever
  sees in this crate — `main.rs` above the cut, plus `api.rs` / `session.rs` / `view.rs` —
  contain line comments and ordinary strings and nothing else, so its block-comment,
  raw-string and char-literal branches were about to ship as an *unverified doc-comment
  claim*, which is the exact defect shape this cycle exists to remove. Pinned instead by
  `test_code_only_blanks_comments_and_string_bodies`, itself proven non-vacuous by disabling
  the char-literal branch and then the block-comment branch (each reddens it; both
  reverted).
- **LOW 3 — the restated 400/422 rule was false for 3 of the 4 variants behind
  `setup_failed`.** `api.rs` mapped **all** of `SessionError::Setup(_)` to 422 while the rule
  grounds 422 in "`validate_deck` for a pregame table", and only `SetupError::InvalidDeck` is
  a `validate_deck` judgment: `NoDeckForSeat` is pool exhaustion, `MissingCardDefinition` is
  documented in `crates/simulator/src/setup.rs` as a defensive spec-build check, and
  `Builder(GameStateError)` is `GameStateBuilder::build()` failing. By this crate's own
  `Start → 500` reasoning all three are server-side faults. **Matched by variant** (the
  preferred fix — it makes the rule true rather than narrowed) with a new
  **500 `setup_internal`** kind so the README's kind→status table stays one-to-one; the match
  is written out variant by variant rather than with a wildcard, so a future `SetupError`
  variant is a compile error here. None of the three is reachable today, so no status was
  lying; the *rule* was over-broad in exactly the way the finding it answers warned about.
- **LOW 4 — the healthy-path half of the atomicity property was prose-only.** `api.rs`
  claimed "a rebuild that fails below must leave a running game exactly as it was"; the code
  was right (`as_ref`, never `take`) and no test drove it, on the same arm where round 2 found
  a claim-vs-code gap. Added `test_a_failed_rebuild_leaves_a_running_game_untouched`: play one
  action first (so "unchanged" is distinguishable from "silently rebuilt at the defaults",
  which would also report `command_count == 0`), fail a rebuild at `players: 2, seed: 17`,
  then assert the same `seq`, the same `command_count`, the same table, and that the
  outstanding decision is still answerable. **Non-vacuity proven by mutation**: changing the
  healthy arm's `as_ref()` to `take()` reddens it with `404 no_session`; reverted.
- **LOW 5 — the coupling between those fixtures and `OOS-M11-6` was recorded nowhere.** Both
  poison-atomicity tests can only make `session::new_game` fail through the CR 903.5c Forest
  padding filed as `OOS-M11-6`; closing that seed may leave them with no trigger. Recorded in
  both test doc comments, the README and the seed row. They fail loudly rather than rotting
  silently, so it is a maintenance note, not a blocker.
- **LOW 6 — two overstatements in `OOS-M11-6` itself, plus one hedge that is now confirmed.**
  (a) "bypasses that same filter **four lines later**" — the filter predicate is `deck.rs:68`
  and the padding bypass is `deck.rs:105-110`: **37** lines. The four was the distance from
  the `basics_for_colors` *call* to the fallback, written as the distance from the filter.
  (b) The italicised, presented-as-verbatim error text quoted "(34 **violations**)";
  `SetupError`'s `Display` formats `"({} violation(s))"`. Quoted correctly now, with a note
  that the earlier form was presented as verbatim and was not. (c) The `GameDriver` hedge is
  **dropped: it is confirmed.** `crates/simulator/src/driver.rs` contains no deck reference of
  any kind; `validate_deck` occurs in `crates/simulator` only in `setup.rs`;
  `bin/fuzzer.rs:296` calls `random_deck` and feeds `GameStateBuilder` at `:309`+ with no
  validation anywhere in the file; both callers draw from the same `all_cards()` pool. So the
  illegal decks are **played** in the fuzzer rather than refused, which raises the seed's
  blast radius from "a play-server 422" to **a silent CR 903.5c deviation in every fuzz run
  that rolls a colourless commander**.
- **LOW 7 — README nits, and one pre-existing stale cite.** The `dist/`-absent caveat
  over-generalised to 405: the 404 half is right (`fallback_service(ServeDir)` is mounted only
  `if dist_dir.exists()`), but a 405 on a routed path is decided by the `MethodRouter` and
  never reaches the path fallback, so `dist/` is irrelevant to it. The `bot` row now says the
  comparison is case-insensitive (`parse_bot_kind` lowercases first), so `"Heuristic"` is
  accepted. And `tools/replay-viewer/src/main.rs:52-66` is actually `:50-65` — corrected at
  all four cite sites, now written by symbol (`fn main`) with the range secondary; the
  neighbouring router-tests cite `:200-415` was also off the end of a 414-line file and is now
  `:198-414`.

**The meta-point, for the third time.** Three of the six documentation defects above sat
*inside* text written to correct documentation drift. The standing rule this cycle enforced:
before writing a sentence asserting a line distance, a quoted string, or a coverage property,
open the file and look — and where that has not been done, write the weaker sentence that is
true.

**`block_in_place` is why the runtime flavor is load-bearing, not a performance choice.**
The 8 MB worker stacks are inherited from `tools/replay-viewer/src/main.rs`'s `fn main` (`:50-65`) (deep
trigger chains overflow tokio's 2 MB default in debug), but the *multi-thread* flavor is a
correctness requirement: `tokio::task::block_in_place` **panics** on a current-thread
runtime, which is what a plain `#[tokio::test]` builds. Every async test therefore carries
`#[tokio::test(flavor = "multi_thread")]`, and both the runtime builder and `api.rs`'s
module doc say so.

**Test 6 (`test_post_action_illegal_target_returns_422`) needed a real spell, and finding
that out surfaced an engine gap.** The obvious subject — the first castable spell at seed 0,
`Accorder's Shield`, a `{0}` artifact — **succeeds with 200** when handed a spurious
`Player` target: the engine does not validate *excess* targets on a spell with no target
requirements, and the bogus target is recorded on the stack object. The test was rebuilt to
drive three deterministic steps to `Cast Dispel` ("counter target spell", CR 601.2c), where a
`Player` target is genuinely refused → `LocalGameError::Rejected(GameStateError::InvalidTarget)`
→ **422**. The same `targets` on `PassPriority` is asserted alongside as a **400** control
(`ParamError::UnsupportedParam`, never reaches the engine), so the 400/422 split is
demonstrated by observation rather than argued. **The excess-target acceptance is a genuine
engine-side finding, out of scope here, and is filed as `OOS-M11-5`** in
`docs/audits/decision-point-audit.md` §8.1. Root cause read in source, not inferred:
`validate_targets_inner` short-circuits its whole requirement-matching pass when
`requirements.is_empty()` — an "existence-only validation" arm added for the **aura/bestow**
path (which declares a target while carrying no `TargetRequirement`) but **not scoped to it**,
so every spell with genuinely zero requirements inherits it. Per CR 601.2c's parenthetical the
spurious target is not inert: a "becomes the target of a spell" trigger can be made to fire off
a spell that does not target.

**Test 7 is Architecture Invariant 7 at the HTTP boundary and was falsified in both
directions.** The omniscient truth is read out of band from the `PlaySession`'s `GameState`
via `StateViewModel::from_game_state`; after excluding the human's own hand and every public
zone (battlefield, graveyards, command zone per CR 903.6, exile, stack), **20 distinct
other-seat hand card names** remain, and the count is asserted **exactly** so a future change
cannot quietly empty the set and turn the search into a no-op. Each is searched for in the
**raw response body string** of `GET /api/game`, not in the parsed `zones.hand` field — the
S4 review HIGH ("redaction follows the rendering site, not the zone") applied forward.
Opposite-direction guard: all seven of the human's own hand names are asserted **present**,
so an empty payload fails. Proven non-vacuous by mutation, **re-run against the current
16-test module during fix cycle 2** (LOW 10 — the earlier wording said "the other seven
stay green", which described the 8-test module the mutation was first run against and was
never re-observed after the module grew): flipping `seat_view`'s `Viewer::Seat(human)` to
`Viewer::Omniscient` reddens exactly `test_seat_view_over_http_contains_no_other_hand_card_names`,
on `seat view leaked another player's hand card "Aggravated Assault" (CR 402.1)`, while
**the other fifteen stay green** — test 2 among them, correctly, since it does not claim to
be a redaction test.

Deviations from the plan text, all recorded in source and in the crate README:

- **`PlaySession` gained `mulligan_count`** beyond the plan's field list. `setup::redeal` is
  keyed by `(seat, mulligan_count)` and nothing else remembers the count — the engine's
  `PlayerState::mulligan_count` is always 0 on this path, because a whole-table rebuild
  issues no `Command::TakeMulligan`.
- **`SeatView` gained `summary: GameSummary`.** The plan names `GameSummary` as a DTO but
  gives it no home.
- **"Pregame" is `command_count() == 0`, not a turn number.** `build_initial_state` sets
  `first_turn_of_game` and `GameStateBuilder` defaults `turn_number` to 1, so a freshly built
  game is already *in* turn 1 — which is also why `local_game::decision_kind_for`'s
  `Mulligan` arm, gated on `turn_number == 0`, is unreachable.
- **The mulligan keeps the whole-table rebuild.** `setup::redeal`'s own doc comment says the
  per-seat model "belongs with the Session 5 play-server pregame flow"; it does not, because
  a per-seat model requires each *bot* seat to be asked, which is a new decision channel
  rather than a small addition. Both of `redeal`'s honest limitations are restated on the
  handler and in the README.
- **`cards_to_bottom` is refused with 400 rather than accepted and discarded.**
  `handle_keep_hand` checks the count against `PlayerState::mulligan_count`, which a rebuild
  always leaves at 0, so CR 103.5's bottoming half is genuinely inexpressible on this path.
  Loud, not silently wrong.
- **`GET /api/game` calls `advance()`** — a read that can move the game. In practice a no-op
  (`advance` is idempotent while a decision is outstanding), and it is what lets a plain
  `GET` report a concluded or halted game without the session caching its last outcome. It
  *does* advance `journal_cursor`: reading events consumes them.
- **`ActionParamsDto.auto_tap` defaults to `true`** — there is no mana-tapping UI yet, and an
  unpayable cast is the worst failure on this surface.
- **`needs_x` answers `CastSpell` only**, through `calculate_characteristics` (not raw
  `obj.characteristics`). An activated ability's `{X}` lives in its `ActivationCost`, which
  `LegalAction::ActivateAbility` does not carry — S7's problem, alongside `modes`.
- **`GameOverView` also carries `AdvanceOutcome::Halted`** behind a `halted: bool`. The plan
  does not model `Halted`, and the client's question ("is another decision coming?") is the
  same either way.
- **Errors the plan does not enumerate**, chosen and reasoned in source: `BadParams` → **400**
  (wrong against *any* state, so a client error, not an engine rejection), no session →
  **404**, a mulligan after play began → **409** `not_pregame`, a poisoned mutex → **500**,
  a bad `players` or `bot` → **400**, `SessionError::Setup` → **422** `setup_failed`,
  `SessionError::Start` → **500** `start_failed`, `LocalGameError::Engine` → **500**
  `engine_error` — the last of which fix cycle 2 (MEDIUM 3) established is **unreachable
  through that impl**: `Engine` is built only in `LocalGame::start`, which this crate routes
  through `SessionError::Start`, and the sole expression feeding `From<LocalGameError>` is
  `play.submit(..)?`. A bot-seat engine failure is not a 500 at all — `LocalGame::advance`
  returns no `Result`, so it surfaces as `Halted(HaltReason::EngineError)` and a **200** with
  `game_over.halted`. The arm is kept (exhaustive match over a plain enum) and labelled
  unreachable in both the source and the README.
- **`DecisionView.player`** is additive to plan item 4's field list (fix cycle 2, LOW 12), so
  a client need not infer the acting seat from `summary.human`.

**Two facts recorded for Session 7.** (1) `mtg_view_model::redact::viewer_may_identify` is
`pub(crate)` and not re-exported, so a play-server label physically *cannot* call it — every
label is instead rendered through a `NameIndex` derived from the already-redacted
`StateViewModel`, and an id absent from that view (or flagged `hidden`, or empty-named)
renders as `(hidden card)`. S7's `target_slots` labels must go through the same index.
(2) `event_view_for` takes **four** parameters (`ev, state, player_names, viewer`), not the
§3 sketch's three.

**Crate**: `tools/play-server` (new) — the only crate in this milestone with async or IO.
**Files**: `tools/play-server/Cargo.toml`, `src/main.rs`, `src/api.rs`, `src/session.rs`,
`src/view.rs`; root `Cargo.toml`.

1. New bin crate: `axum 0.7`, `tower-http` (fs), `tokio` (full), `serde`, `serde_json`,
   `clap`, `anyhow`, `mtg-engine`, `mtg-simulator`, `mtg-view-model`; dev-deps `tower`
   (util) + `http-body-util`; `[lints] workspace = true`; added to workspace members.
2. `main.rs`: clap CLI (`--port` default 3040 so it can run alongside the replay viewer's
   3030, `--host` default `127.0.0.1` per MR-M9.5-06, `--players`, `--bot`, `--seed`), and
   a hand-built multi-thread tokio runtime with **8 MB worker stacks** — same reason as
   `tools/replay-viewer/src/main.rs`'s `fn main` (`:50-65`) (deep trigger chains overflow tokio's 2 MB
   default in debug builds). `build_router(state, dist_dir) -> Router` must be a free
   function so tests can construct it.
3. `session.rs`: `PlaySession { game: LocalGame, human: PlayerId, names:
   HashMap<PlayerId,String>, cfg: LocalGameConfig, journal_cursor: usize, pending:
   Option<PendingDecision> }` behind `Arc<std::sync::Mutex<Option<PlaySession>>>`; every
   engine call wrapped in `tokio::task::block_in_place` so a long resolution cannot stall
   the reactor.
4. `view.rs` DTOs: `GameSummary`, `SeatView { state: StateViewModel, decision:
   Option<DecisionView>, events: Vec<EventView>, game_over: Option<GameOverView> }`,
   `DecisionView { seq, kind, actions: Vec<ActionOptionView> }`, `ActionOptionView { index,
   kind, label, object_id, target_slots: Vec<Vec<TargetOptionView>>, needs_x, modes }`.
   Labels are built server-side from the view model (card names), so the client never
   needs engine types. `LegalAction` itself is **not** serialized — the client submits an
   index plus params, and the server maps back through the stored `PendingDecision`.
5. Routes (a route surface disjoint from the replay viewer's `/api/step/...`):
   `POST /api/game` (new game), `GET /api/game` (seat view + pending decision + events
   since cursor), `POST /api/game/action` (`{seq, action_index, params}`),
   `POST /api/game/mulligan` (`{take, cards_to_bottom}`, pregame only, calls
   `setup::redeal`), `GET /api/healthz`; `ServeDir` fallback to `dist/`.
6. Stale/invalid handling: `seq` mismatch → **409**; unknown `action_index` → **400**;
   engine rejection → **422** with the `GameStateError` rendered as text; no pending
   decision → **409**.
7. **Tests** (inline `#[cfg(test)] mod tests` in `main.rs`, exactly like the replay
   viewer's, using `tower::ServiceExt::oneshot` — **never start a listener**, see §7):
   `test_post_game_creates_session_and_returns_decision`;
   `test_get_game_returns_seat_view_with_seven_card_hand`;
   `test_post_action_pass_priority_advances_and_bots_act`;
   `test_post_action_stale_seq_returns_409`;
   `test_post_action_unknown_index_returns_400`;
   `test_post_action_illegal_target_returns_422`;
   `test_seat_view_over_http_contains_no_other_hand_card_names` (Invariant 7 at the HTTP
   boundary);
   `test_healthz_ok`.
8. Record the decision **no WebSocket / no SSE in M11-local**: bots act synchronously
   inside the same request that carries the human's action, so request/response is
   sufficient and push infrastructure is M10a's problem. Note it in the crate README and
   in `memory/decisions.md` (Session 8 collects doc updates).

**Acceptance**: a full game can be played through `curl` alone; every HTTP test runs
without binding a port.

---

### Session 6: Play frontend — render and basic input (7 items)

**STATUS (2026-08-01, `scutemob-169`): SHIPPED — all 7 items done, 1 review cycle applied
(1 HIGH / 2 MEDIUM / 3 LOW).**

- **The HIGH is the session's real lesson, and a green build had nothing to say about it.**
  `tools/replay-viewer/frontend/src/lib/ZoneHand.svelte` keyed its `#each` on
  `card.object_id` — correct for the omniscient viewer, **fatal** for a seat-redacted
  payload, because `redact::redact_hands` replaces every unreadable hand card with
  `hidden_placeholder()`, whose `object_id` is **0**. Three bot hands of seven cards each
  therefore arrived with one distinct key apiece, Svelte 5's keyed reconciler evaluates
  `length > keys.size` and calls `each_key_duplicate` — which **throws in production, not
  only in DEV** — and with no `<svelte:boundary>` the throw escapes the effect flush and
  takes the mount down. **The play surface rendered nothing at all**, and every gate this
  session had (build clean, 135 modules, 0 warnings, zero Rust diff, 4,040 tests green) was
  green while it did. Measured rather than argued: `Bot-2/3/4` each `length 7, keys.size 1`;
  the human's own hand `length 7, keys.size 7`. Fixed **in the shared component** as
  `card.hidden ? \`hidden-${i}\` : card.object_id` — keyed on the flag the redactor sets, not
  on the sentinel 0 — which is inert for the viewer and is the whole reason not to copy the
  component. `hidden_placeholder` has exactly one call site (`redact_hands`), so hands are
  the only zone at risk; checked, not assumed. **Generalisation for S7: the replay viewer's
  components were written against an omniscient view model, and every assumption they make
  about id uniqueness is now a claim about the redacted one too.**
- **MEDIUM: `DeclareAttackers` / `DeclareBlockers` submit an EMPTY set and nothing complains.**
  `params.rs` maps default params straight to `Command::DeclareAttackers { attackers: vec![] }`
  — legal, irreversible, and silent, which is *worse* than the targeted-spell case that at
  least fails loudly with a 422. The buttons stay enabled (disabling them would deadlock a
  combat where the declaration is the only offered action, and CR 508.1 makes "no attackers"
  legal) but are marked `declares none` with a tooltip saying so, and the README says it
  plainly. S7's pickers are the actual fix.
- Three LOWs: `jsconfig.json`'s `$viewer/*` path was off by one directory (editor-only —
  `vite.config.js` was right, so the build never noticed); the "omit and take the CLI
  default" rationale did not hold for `players`, which was pre-seeded to `'4'` and so
  overrode a server started with `--players 6` on every New game; and the event feed keyed
  its `#each` on the array index against a front-truncating window, now keyed on a monotonic
  `seq` stamped at append.

- **Zero Rust.** `git diff main -- crates/ tools/play-server/src tools/play-server/Cargo.toml
  tools/replay-viewer/` is **empty**; PROTOCOL 32 / HASH 69 unmoved; workspace tests
  **4,040 / 0** (unchanged from the merge base by construction — this session adds no Rust
  test target and no frontend test harness, exactly as item 7 says). `npm run build` is the
  gate and it is clean: 135 modules, 0 warnings.
- **`$viewer` resolves rather than copies, and that is checked rather than asserted.**
  `find frontend/src -type f` lists **eight** files, none of them a `Zone*` or
  `PhaseIndicator`, while the production CSS bundle contains the viewer components' scoped
  rules. The alias is built with `fileURLToPath(new URL(..))` and not a bare relative
  string, because a relative alias target resolves against the *importing* file and would
  break for imports from `src/lib/`.
- **The mulligan `LegalAction`s are unreachable on this surface, and the plan's item 5/6
  sketch did not know it.** `legal_actions.rs` and `local_game.rs::decision_kind_for` both
  gate `TakeMulligan`/`KeepHand` and `DecisionKind::Mulligan` on
  `is_first_turn_of_game && turn_number == 0`, and `setup::build_initial_state` +
  `GameStateBuilder` leave a fresh table already *in* turn 1 —
  `session.rs::is_pregame`'s own doc says the condition is unsatisfiable. Confirmed in a
  real payload: the pregame decision's `kind` is `Priority` and its only option is
  `Pass priority`. The UI therefore gates its pregame block on `summary.pregame` alone and
  uses the dedicated `POST /api/game/mulligan` route. **"Keep this hand" has no server-side
  representation at all** (a `take: false` post only re-renders, and `pregame` is
  `command_count == 0`), so it is a client-side flag — recorded in `PlayApp.svelte` rather
  than hidden.
- **The redactor rewrites a hidden card's `object_id` to 0, which click-through must refuse
  rather than merely fail to match.** `redact::hidden_placeholder` emits
  `{hidden: true, name: "Hidden card", object_id: 0}`; a seed-0 4-player playthrough carried
  **569** such entries, all id 0, while the lowest id any `ActionOptionView` ever carried was
  2. No collision today — but all seven of a bot's hand cards share one id, so one action
  about object 0 would make all seven submit it. `PlayApp::isUnidentifiable` refuses them
  before matching. Note the scope: `hidden` is a field of `CardInZoneView`, **not** of
  `PermanentView` — an opponent's face-down permanent keeps its real id and is matched
  normally, which is correct.
- **`ZoneStack` declares `onCardClick` and never calls it.** A dead prop in the viewer,
  found while wiring item 6. Harmless here (no `LegalAction` with an `object_id` names a
  stack object — `view.rs::action_object`), but **Session 7 renders targets on stack items
  and should know before it relies on it.**
- **A targeted spell cast from this UI fails with a real 422**, because `target_slots` is
  empty until S7 and the client sends `params: {}`. Observed verbatim:
  `{"error":"invalid target: expected 1..=1 target(s) but got 0","kind":"rejected"}`. That
  is correct S6 behaviour (CR 601.2c) and the error strip surfaces it rather than swallowing
  it. It is the exact hole S7's `TargetPicker` closes.
- **The manual checklist was actually run, not asserted.** A temporary `#[ignore]`d probe was
  added to `main.rs`'s existing `mod tests`, driven through `tower::ServiceExt::oneshot`
  (**binding no port**), and **removed again**; the frontend was then checked against the
  dumped `SeatView` payloads. 7-card hand, land drop (hand 8→7, battlefield 0→1), 25 passes,
  10–21 redacted event lines per response, a non-empty stack, and turn 4 reached — all
  recorded in `tools/play-server/README.md` §"Manual checklist" with the two steps that are
  genuinely unverifiable headless (launching the binary; keyboard and DOM events) marked as
  such rather than glossed.
- **One server-side oddity found and NOT fixed here** (it would be a Rust diff): `ActionRequest.params`
  carries a plain `#[serde(default)]`, which routes through `ActionParamsDto`'s *derived*
  `Default` where `auto_tap` is `false`, while an explicit `"params": {}` takes the field's
  `#[serde(default = "default_auto_tap")]` and gets `true`. So omitting `params` and sending
  `{}` are not equivalent. The client always sends `{}` explicitly, so it is unaffected.
  Reasoned from the source, **not executed**, and nothing in `tools/play-server` pins it
  either way — a one-test job for S7 or S8.

**Crate**: `tools/play-server/frontend` (new Svelte 5 app). No Rust change beyond serving
`dist/`.
**Files**: `tools/play-server/frontend/{package.json,vite.config.js,index.html,
src/main.js,src/App.svelte,src/lib/api.js,src/lib/stores.js,src/lib/PlayApp.svelte,
src/lib/ActionBar.svelte,src/lib/EventFeed.svelte}`, `tools/play-server/README.md`.

1. Scaffold Vite + Svelte 5 mirroring `tools/replay-viewer/frontend` (same versions:
   svelte ^5.45, vite ^7.3, `@sveltejs/vite-plugin-svelte` ^6.2), with a dev proxy to
   `127.0.0.1:3040`.
2. `vite.config.js`: `resolve.alias` `$viewer → ../../replay-viewer/frontend/src/lib`, so
   the props-based components are imported, not copied — the mechanism
   `docs/mtg-engine-replay-viewer.md` §"Import Mechanism" anticipated. (Promotion to a
   shared `tools/ui-shared/` package is deferred; see §8 R8.)
3. `src/lib/api.js` — the play adapter (`newGame`, `getGame`, `submitAction`, `mulligan`),
   same shape as the replay viewer's adapter; `stores.js` — `seatView`, `decision`,
   `events`, `loading`, `error`.
4. `PlayApp.svelte` — layout: `$viewer/PhaseIndicator` (turn/step/priority),
   `$viewer/StateView` fed the seat-scoped state, `EventFeed`, `ActionBar`.
5. `ActionBar.svelte` — renders `decision.actions` as buttons, submits `{seq, index,
   params}`, disables while a request is in flight, surfaces 4xx/422 text; keyboard
   shortcuts (`space` = pass priority, `Esc` = cancel a picker).
6. Hand and battlefield click-through: clicking a card in `$viewer/ZoneHand` /
   `ZoneBattlefield` selects the matching action by `object_id`; clicking with no matching
   action shows why ("no legal action for this card right now").
7. Manual checklist in `tools/play-server/README.md` (there is no frontend test harness in
   this repo — Session 5's API tests are the automated coverage): launch → see 7-card hand
   → play a land → pass priority → watch bots act in the event feed → see the battlefield
   and stack update → reach turn 3.

**Acceptance**: a human can play lands and pass priority in a browser and watch a
4-player game progress.

---

### Session 7: Targeting, combat and choice UIs (7 items)

**Crates**: `tools/play-server` (DTO population + validation) and its frontend.
**Files**: `tools/play-server/src/{view.rs,api.rs}`,
`tools/play-server/frontend/src/lib/{TargetPicker.svelte,AttackerPicker.svelte,
BlockerPicker.svelte,ValuePrompt.svelte}`.

1. Server: populate `ActionOptionView.target_slots` from
   `mtg_engine::legal_targets_per_slot` + `spell_target_requirements`, rendering each
   candidate as `TargetOptionView { kind: "object"|"player", id, label }` (labels from the
   *seat-redacted* view model — a target picker must not leak a hidden card's name).
2. Server: populate attacker/blocker option payloads from the `LegalAction::DeclareAttackers
   { eligible, targets }` / `DeclareBlockers { eligible, attackers }` the provider already
   emits (CR 508.1 / CR 509.1); validate submitted pairs against them and return 400 with
   a readable message on mismatch.
3. `TargetPicker.svelte` — one selector per slot, min/max from `target_count_range`
   (CR 601.2c, `UpToN` allows fewer).
4. `AttackerPicker.svelte` — multi-select of eligible creatures plus a per-attacker
   `AttackTarget` (player or planeswalker) (CR 508.1a).
5. `BlockerPicker.svelte` — blocker→attacker pairing for the human when a bot attacks
   (CR 509.1a).
6. `ValuePrompt.svelte` — X value (CR 601.2b) and mode selection (CR 700.2) when the
   action option declares `needs_x` / `modes`.
7. **Tests** (server-side, `oneshot`): `test_action_option_target_slots_match_engine_query`;
   `test_declare_attackers_through_api_emits_attackers_declared`;
   `test_declare_blockers_rejects_ineligible_blocker`;
   `test_target_option_labels_are_seat_redacted`;
   `test_x_value_is_forwarded_to_cast_spell_data`.
   Plus manual-checklist additions (attack a bot; block a bot's attacker; cast a targeted
   removal spell on a bot's creature).

**Acceptance**: the human can attack, block, and cast targeted/X/modal spells.

---

### Session 8: Playthrough hardening, docs, acceptance (8 items)

**Crates**: `crates/simulator`, `tools/play-server`, docs.
**Files**: `crates/simulator/tests/local_game_playthrough.rs` (new),
`tools/play-server/src/api.rs`, `docs/mtg-engine-roadmap.md`,
`docs/mtg-engine-replay-viewer.md`, `docs/mtg-engine-simulator.md`, `CLAUDE.md`,
`memory/decisions.md`, `memory/gotchas-infra.md`.

1. **Scripted-human playthrough test**: a deterministic policy (prefer land → prefer
   cheapest castable spell → attack when able → otherwise pass) drives seat 1 through a
   full 4-player game via `LocalGame` alone (no HTTP). Assert: no
   `LocalGameError::Engine`, zero `InvariantViolation`s, and the game reaches `GameOver`
   or the turn cap. Run it for 5 fixed seeds.
2. Surface the currently-invisible optional decisions, or record an explicit deferral with
   the reason: Echo (CR 702.30, `pending_echo_payments`), Cumulative Upkeep (CR 702.24),
   Recover (CR 702.58), and blocker damage-assignment order (CR 509.2,
   `Command::OrderBlockers` — the engine falls back to `OrdMap` order today). These need
   new `LegalAction` variants (simulator-internal, **not** a wire change). If they exceed
   the session, defer with a filed seed rather than half-wiring them.
3. Concede + game-over: `Command::Concede` (CR 104.3a) exposed as an action;
   `GameOverView` rendered with winner / turn count / reason.
4. Error surfacing audit: grep the play-server and `LocalGame` for any path that swallows
   a `GameStateError` on a human action. There must be none — the bot-seat
   `PassPriority` fallback must not be reachable from `submit`.
5. Bug-report export: `GET /api/game/report` serializes `{seed, config, journal (commands
   + events), final state hash}` as JSON, per `docs/mtg-engine-runtime-integrity.md`, plus
   a frontend button. This is also the seed-repro artefact for anything the playthrough
   finds.
6. Docs: strike the rewind/pause/manual-adjust and turn-control/step-skipping bullets from
   the roadmap's M11-local deliverables (they contradict its own scope-boundary paragraph)
   into M10b / the primitive queue; check off what shipped; note in
   `docs/mtg-engine-replay-viewer.md` that the view model now lives in
   `crates/view-model`; add the play client to `docs/mtg-engine-simulator.md`; update
   `CLAUDE.md` Current State.
7. `memory/decisions.md` rows: steppable driver over channel-backed bot (with the
   reasoning in §2); no WebSocket in M11-local; view-model extracted to a shared crate;
   mulligan-by-pregame-rebuild because the engine has no RNG.
8. Gates: `~/.cargo/bin/cargo test --all`, `clippy --all-targets -- -D warnings`,
   `fmt --check`, `tools/check-defs-fmt.sh`, protocol/hash parity, and a 500-game
   `--profile fuzz` run to confirm the driver refactor did not perturb the fuzzer.

**Acceptance**: a human plays a full game end to end; the milestone's gates are green.

---

## 5. Supporting agents

- **`card-definition-author` / `bulk-card-author`**: **not needed.** M11-local adds no
  cards; it plays the 1,139 `Complete` defs that exist.
- **`game-script-generator`**: **not needed.** No new *mechanic* or rules interaction is
  introduced — every rules behaviour exercised here already has engine coverage. (It
  *would* be required if Session 8 item 2 adds Echo/Cumulative-Upkeep decision paths that
  change resolution behaviour; it does not, it only surfaces existing ones.)
- **`cr-coverage-auditor`**: optional, scoped to Session 3's `rules/queries.rs` (CR 601.2b/c,
  602.2b, 700.2c, 702.96b, 702.127a) — the only rules-adjacent engine addition.
- **`milestone-reviewer`**: run after Session 8, as per the Milestone Completion Checklist.
  Two areas deserve its attention specifically: the Invariant-7 redaction chokepoints
  (Session 4) and the `run_game` re-expression (Session 1).

---

## 6. Acceptance criteria checklist

From the roadmap's M11-local section, minus the carved-out bullets:

- [ ] Human input bridge: a human occupies one seat; the driver advances bot seats
      autonomously and yields when the human must act
- [ ] Engine purity preserved: no async/HTTP/IO in `crates/engine` (Invariant 1)
- [ ] axum server skeleton with a route surface separate from the replay viewer's stepper
- [ ] Local game setup: `Complete`-only deck via `validate_deck`, 1 human + 3 bots
      (Invariant 9)
- [ ] Game state rendering: all zones, players, life totals
- [ ] Card display from cached/Scryfall images with text fallback
- [ ] Hand display, clickable
- [ ] Battlefield display with tapped state
- [ ] Stack display with source card info
- [ ] Phase/step indicator and priority indicator
- [ ] Basic input: cast, pass priority, select targets
- [ ] Life totals and per-opponent commander damage
- [ ] A human player can play a Commander game through the UI against bots
- [ ] PROTOCOL_VERSION still 27, fingerprint still `f035e797…`, HASH_SCHEMA_VERSION still 63
- [ ] All tests pass: `~/.cargo/bin/cargo test --all`
- [ ] Zero clippy warnings: `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`
- [ ] Formatted: `~/.cargo/bin/cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35)
- [ ] `cargo build --workspace` clean (catches the view-model exhaustive-match arms)

---

## 7. Non-negotiable constraints for every session

1. **Never start the play-server or replay-viewer HTTP binary to validate anything.**
   Agent contexts get SIGKILL/137 (`memory/gotchas-infra.md`). Use
   `tower::ServiceExt::oneshot` against `build_router(...)`, the pattern already proven in
   `tools/replay-viewer/src/main.rs`.
2. **Architecture Invariant 1**: nothing async, no IO, no network, no filesystem in
   `crates/engine`. The only engine change in this whole milestone is a read-only query
   module.
3. **Architecture Invariants 3 and 4**: every state change goes through
   `process_command`; the UI is fed from `GameEvent`s and derived view models. `GameState`
   stays sealed `pub(crate)` (SR-3).
4. **Architecture Invariant 7**: the human seat's payload must never contain another
   player's hand contents or any library order. Every session that touches
   `crates/view-model` or `tools/play-server` ships a test asserting this.
5. **Architecture Invariant 9**: every deck path goes through `validate_deck` +
   `start_game`'s `check_all_defs_complete`.
6. **No new `Command` / `GameEvent` / `Effect` variant.** If one seems necessary,
   stop and flag: it is a wire change (SR-8) requiring a PROTOCOL bump, and for this
   milestone it signals a design error.
7. **SR-9a**: a new engine test file goes in an existing `crates/engine/tests/<group>/`
   **and gets a `mod` line in that group's `main.rs`**. Never add a top-level
   `crates/engine/tests/*.rs`. (New targets under `crates/simulator/tests/` and
   `tools/play-server/src` are outside that gate.)
8. `cargo build --workspace` after every session — `crates/view-model` carries the two
   exhaustive matches (`StackObjectKind`, `KeywordAbility`) that runners miss ~50% of the
   time.

---

## 8. Risks and open questions

| # | Risk / question | Assessment | Recommendation |
|---|---|---|---|
| R1 | **Targeted spells are currently uncastable by a human.** The TUI proves it: `input.rs` always sends `targets: Vec::new()`, so any spell with a `TargetRequirement` is rejected at `casting.rs:3708`. | This, not the loop, is the real blocker for "a person played a game". | Session 3 is the crux of the milestone. If `legal_targets_per_slot` grows past one session, split it: requirements lookup first, candidate enumeration second. |
| R2 | ~~**`handle_take_mulligan` emits `LibraryShuffled` while permuting nothing** (engine has no RNG). A mulligan today returns the same hand — a live-wrong rules path (CR 103.5 requires a shuffle).~~ ✅ **CLOSED by PB-DP2 (`scutemob-150`, 2026-07-26)**, widened per decision-point-audit §7 to also cover `handle_keep_hand` bottoming to the library TOP. | Real correctness finding, discovered while planning. ~~Out of M11-local's scope to fix properly (a caller-supplied permutation would be a new `Command` → wire change).~~ **This assessment was FALSIFIED.** No wire change was needed at all: the engine already had a deterministic seeded PRNG (`StdRng::seed_from_u64(state.timestamp_counter)`, the MR-M7-17 idiom at `effects/mod.rs:8697-8703`), so `handle_take_mulligan` could just permute the library in place with `Zone::shuffle`. PROTOCOL 27 / HASH 63 unmoved. **Reusable lesson: check for an existing in-engine deterministic seed source before concluding a permutation needs a caller-supplied `Command`.** | ~~File as a primitive seed (proposed id **OOS-M11-1**)~~ — filed, ranked into the PB-DP suite, and **shipped as PB-DP2**. Session 2's pregame `redeal` workaround is **no longer load-bearing for correctness** (mulligans are now CR 103.5-faithful); keep it only if Session 2 wants it for UX. |
| R3 | **`solve_mana_payment` ignores the mana pool and reads non-layer-resolved `characteristics.mana_abilities`.** A human who taps manually then casts gets over-tapped; an animated land or a Cryptolith-Rite-granted ability is invisible to auto-tap. | The second is a standing-gotcha violation (dispatch sites must use `calculate_characteristics`). Simulator-side, not engine-side. | Session 3 item 7 fixes the pool half (cheap). File the layer-resolution half as **OOS-M11-2**; note that the *engine* payment paths are already layer-correct, so this is a suggestion-quality bug, not a wrong-game-state bug. |
| R4 | **`StubProvider` gaps**: no Adventure (documented TODO at `legal_actions.rs:158`), no alt-costs (Spectacle/Surge/Escape/Flashback…), no modes, no Convoke/Improvise/Delve. A human will hit these. | Expected; the provider is the bot's move generator, not a rules-complete action enumerator. | Ship v1 provider-driven. Open question for the user: add a dev-only "raw command" escape hatch (submit a hand-built `Command` and let the engine judge) so a play-tester can exercise paths the provider misses? Recommend yes, behind `--dev` |
| R5 | **Bot play quality.** `RandomBot` makes nonsense plays; games look broken to a human even when the engine is right. | Cosmetic but affects "is this playable" judgement. | Default the web client to `HeuristicBot`; keep `RandomBot` as a `--bot random` option |
| R6 | **Stall guards vs. a thinking human.** `max_consecutive_passes = 500` and `max_commands = max_turns * 200` are fuzzer safety valves. | A human game legitimately passes a lot. | Count only *bot* passes toward the guard in `LocalGame`, and make the limits config fields (`LocalGameLimits`) rather than constants |
| R7 | **No frontend test harness exists in this repo.** Sessions 6 and 7 are validated by manual checklist only. | Accepted for M11-local. | Keep the API contract fully covered by `oneshot` tests so a UI regression cannot hide a server regression. Revisit (Vitest/Playwright) at M13 |
| R8 | **Component sharing by Vite alias couples the play client to the replay viewer's directory layout.** | Cheap now, brittle if a third consumer appears. | Alias now; promote `frontend/src/lib` to `tools/ui-shared/` only when a third host exists or the first breakage occurs |
| R9 | **`CardDisplay.svelte` fetches images from Scryfall directly** — the browser makes external requests during local play. | Fine locally; wrong for an offline/packaged build. | Note it; the cached-image path is M14 (assets) |
| R10 | **Roadmap M11 lists two engine features** (turn-control override / Mindslaver; step skipping / Stasis) among UI deliverables. | They are engine primitives with layer/turn-structure blast radius; bundling them into a UI milestone is how a "first playable" slips. | **Recommend carving them out to the primitive queue.** Needs the user's assent — flagged, not assumed |
| R11 | **Driver refactor could perturb the fuzzer** and invalidate crash-seed comparability. | Real; the fuzzer is the engine's main adversarial harness. | Session 1 items 1 + 8: capture a 200-game baseline before the refactor and diff after. Any difference stops the session |
| Q1 | Should mulligans be offered at all in v1, given R2? | Pregame `redeal` is faithful to CR 103.5 and costs one function. | Yes — offer them; they're the difference between "a demo" and "a game" |
| Q2 | One human seat only, or several (hotseat)? | `human_seats` is a `BTreeSet` so multi-seat is free in the core, but the *view* would have to switch seats and hidden info becomes honour-system. | Core supports N; the web client exposes exactly one seat in M11-local |
| Q3 | Persist a game across server restarts? | The journal already contains everything needed to rebuild by replay. | Out of scope; the Session 8 bug-report export covers the repro need |

### Resolutions applied 2026-07-26 (`scutemob-147`)

- **R10 — already done, no user call needed.** This plan was written against the roadmap as
  it stood before commit `aceba394` earlier the same day. That commit had already carved
  turn-control override (Mindslaver) and step skipping (Stasis) **out of M11-local and into
  M13**, along with the rewind UI, the pause-for-rules-discussion button, manual state
  adjustment, and the integrity-error display (those four need M10b's server machinery).
  The roadmap's M11-local section now carries an explicit "Deferred out of M11-local" block
  naming all six. Session 8's proposed doc edit for R10 is therefore **already satisfied** —
  re-check the roadmap before redoing it.
- **Q1 — yes, offer mulligans** via the pregame `redeal` (Session 2 item 4), as recommended.
- **R4 / Q1's escape hatch — deferred to the Session 5 dispatch, not Session 1.** The
  `--dev` raw-command hatch is a play-server concern and cannot be decided from inside
  Session 1's crate. Session 5 should raise it again with the coordinator.
- **R2 (`OOS-M11-1`) and R3's layer half (`OOS-M11-2`) are filed here as proposed seeds
  only.** They are *not* yet entered in `memory/primitives/rider-seed-triage-2026-07-19.md`
  — that queue is paused at PB-RS4 and belongs to the coordinator. Surface both at
  collection so they can be ranked against the existing RS backlog. R2 in particular is a
  **live-wrong rules path** (CR 103.5 requires a shuffle; `handle_take_mulligan` emits
  `GameEvent::LibraryShuffled` while permuting nothing), which is the class that outranks
  everything else in that queue.
  - **Update 2026-07-26 — `OOS-M11-1` is CLOSED by PB-DP2 (`scutemob-150`).** It was ranked
    into the PB-DP suite (decision-point audit §5 **DP-2**, Tier 0) rather than the RS queue,
    and shipped as two edits in `rules/commander.rs`: `handle_keep_hand` now bottoms with
    `move_object_to_bottom_of_zone` (`push_front`), and `handle_take_mulligan` now runs a
    real seeded Fisher-Yates `Zone::shuffle` before the `LibraryShuffled` event and the
    draws, so the event is no longer phantom (Architecture Invariant 4). **The row's
    "would be a new `Command` → wire change" premise was falsified** — the existing
    `state.timestamp_counter` was a sufficient seed source, PROTOCOL 27 / HASH 63 unmoved.
    4 probes; tests 3,721 → 3,725. Correct cite is **CR 103.5** (+ 103.5c for the free first
    mulligan); "CR 103.4b" as it appears in the task criteria and older notes is stale —
    live 103.4b is the *Vanguard starting life total*. `OOS-M11-2` remains open and
    unranked.

---

## 9. Key CR references

| CR | Summary | Session |
|---|---|---|
| 103.5 / 402.1 | Each player draws a starting hand of seven (**corrected** — 103.4 is the starting *life total*, 103.4c = Commander's 40; do not cite it for the opening hand) | 2 |
| 103.5 / 103.5c | Mulligan procedure; free first mulligan in multiplayer | 2, 8 |
| 104.3a | A player who concedes leaves the game | 8 |
| 117.3 / 117.3a–d | Who has priority; passing priority | 1, 6 |
| 401.2 | The library is a hidden zone kept face down | 4 |
| 402.1 / 402.2 | The hand is hidden; maximum hand size | 4 |
| 400.7 | An object that changes zones becomes a new object (stale ObjectIds in the UI) | 5, 6 |
| 508.1 / 508.1a | Declare attackers; attack targets | 3, 7 |
| 509.1 / 509.1a | Declare blockers | 3, 7 |
| 509.2 | Damage assignment order for multiple blockers | 8 |
| 601.2b | Announce modes and X | 3, 7 |
| 601.2c | Announce targets; target count range; distinctness | 3, 7 |
| 602.2b | Activated-ability targets | 3 |
| 605.3b / 106.1a-b | Mana abilities resolve immediately; colour chosen at activation | 3 |
| 700.2c / 700.2f | Per-mode target requirements | 3 |
| 702.24 / 702.30 / 702.58 | Cumulative Upkeep / Echo / Recover optional payments | 8 |
| 702.96b | Overloaded spells have no targets | 3 |
| 702.127a | Aftermath half's target requirements | 3 |
| 903.5a | Commander deck size is exactly 100 | 2 |
| 903.9a | Commander zone-change choice | 1 |

---

## 10. Start here

**Session 1**, unmodified. It has no dependency on any other session, touches exactly one
crate, adds no engine surface, adds no wire surface, and is fully testable without an HTTP
server. Its output — a `LocalGame` that stops and asks a human what to do — is the thing
every later session builds on, and it is independently useful the moment it lands (the TUI
can adopt it, and the fuzzer keeps running on it).
