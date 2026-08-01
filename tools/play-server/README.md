# `play-server` — the M11-local play surface

One human seat, three simulator bots, over HTTP.

This is the first-playable host: a browser plays a real Commander game against
`HeuristicBot`s driven by the same `LocalGame` the fuzzer uses. It is the **only
crate in M11-local with async or IO** (`memory/m11-session-plan.md` §3) —
`crates/engine`, `crates/simulator` and `crates/view-model` stay pure, per
Architecture Invariant 1.

Built by M11-local **Session 5** (plan §4). Session 6 adds the Svelte frontend
under `frontend/`, which this binary serves from `dist/`.

---

## What it is made of

| File | Role |
|---|---|
| `src/main.rs` | clap CLI, hand-built tokio runtime, `build_router`, the inline HTTP tests, the no-socket source gate |
| `src/session.rs` | `PlaySession` lifecycle — build, advance, submit, mulligan. **Synchronous; knows nothing about tokio.** |
| `src/api.rs` | the axum handlers — the only async code, and the only place tokio is named |
| `src/view.rs` | wire DTOs and the server-side rendering that produces them |

The async boundary is exactly one function deep: a handler takes the session
mutex and runs the synchronous engine work inside `tokio::task::block_in_place`.
Nothing below `api.rs` may reference tokio.

`LegalAction` is **never serialized**. The client receives an index plus a
server-rendered label and posts the index back; the server maps it through the
`PendingDecision` it is still holding. No engine enum is a wire type, and this
milestone adds **no `Command` / `GameEvent` / `Effect` variant anywhere** —
`PROTOCOL_VERSION` and `HASH_SCHEMA_VERSION` are untouched.

---

## Running it

```sh
cargo run -p play-server -- --port 3040 --players 4 --bot heuristic --seed 0
```

| Flag | Default | Why |
|---|---|---|
| `--port` | `3040` | **not** 3030 — the replay viewer owns that, and the two are meant to run side by side |
| `--host` | `127.0.0.1` | localhost-only by default per **MR-M9.5-06**; pass `0.0.0.0` to expose on the LAN, deliberately |
| `--players` | `4` | CR 903.1 — Commander is a multiplayer format; range is 2..=6 |
| `--bot` | `heuristic` | `RandomBot` makes nonsense plays that read as engine bugs to a human (plan §8 R5); it remains the *fuzzer's* default |
| `--seed` | `0` | the pregame build is deterministic from the seed. `POST /api/game` can override per game. **A bug report needs the mulligan count too** — see below. |

#### Reproducing a table from a bug report

The pregame build is deterministic, but the seed alone is not the whole key. A
mulligan goes through `mtg_simulator::setup::redeal`, which builds the table from
`redeal_seed(seed, human_seat, mulligan_count)` and leaves `cfg.seed` unchanged —
so `GameSummary.seed` keeps reporting the **base** seed while the table in play
came from a derived one.

The reproduction key is therefore four fields, all of them in every `GameSummary`:

| Field | Role |
|---|---|
| `seed` | the base seed (`--seed`, or the `POST /api/game` override) |
| `players` | seat count |
| `bot` | which bot fills the non-human seats |
| `mulligan_count` | how many redeals were taken; `0` means the base seed built the table directly |

The effective seed is documented rather than surfaced as a fifth field because
the derivation is private to `mtg_simulator::setup` and a copy of it here could
drift from the original silently.

The runtime is hand-built with **8 MB worker stacks** for the same reason as
`tools/replay-viewer/src/main.rs`: deep trigger chains overflow tokio's 2 MB
default in debug builds. The **multi-thread** flavor is load-bearing, not a
performance choice — `block_in_place` panics on a current-thread runtime.

Seat 1 is always the human (`Human-1`); the rest are `Bot-2`, `Bot-3`, ….

---

## Routes

The surface is deliberately disjoint from the replay viewer's `/api/step/...`,
so the two servers could one day be merged without a route collision.

| Route | Body | Meaning |
|---|---|---|
| `POST /api/game` | `{players?, bot?, seed?}` (all optional; body optional) | start (or restart) a game; the response already carries the human's first decision |
| `GET /api/game` | — | this seat's view, its pending decision, and every event since the last read |
| `POST /api/game/action` | `{seq, action_index, params?}` | answer the pending decision, then let the bots act |
| `POST /api/game/mulligan` | `{take, cards_to_bottom?}` | CR 103.5 pregame redeal. **Pregame only.** |
| `GET /api/healthz` | — | liveness; never takes the session lock |
| everything else | — | `ServeDir` fallback to `dist/`, when that directory exists |

### Error-status semantics

Every failure **this crate's handlers produce**, and every failure of the JSON
body extractor in front of them, is the same envelope:
`{"error": "...", "kind": "..."}`. `kind` is a stable machine tag so a client can
branch without parsing prose.

Two residual exceptions, named rather than glossed and both verified by request:
a path no route matches (**404**) and a wrong method on a routed path (**405**)
are answered by axum's router itself, with an **empty body and no
`Content-Type`**. Both are decided before any handler exists to answer them.
Note the first is distinct from the enveloped `404 no_session` below: that one
comes from a handler and does carry `kind`.

| Condition | Status | `kind` | Reasoning |
|---|---|---|---|
| `seq` does not match the outstanding decision | **409** | `stale_decision` | the client answered a superseded action list; retrying against the current `seq` will work. The message carries `expected` and `got` so the client can resync in one round trip. `seq` is monotonic for the life of the process, **across restarts and mulligans** — see Wire `seq` below. |
| no decision is outstanding | **409** | `no_pending_decision` | well-formed, but conflicts with the current state of the resource |
| `action_index` is not in the list just sent | **400** | `unknown_action` | the request is malformed on its face |
| a param the chosen action has no channel for | **400** | `bad_params` | `ParamError::UnsupportedParam` — wrong against *any* state, so a client error rather than an engine rejection. Refusing beats silently discarding a human's announced targets. |
| the **engine** refused the command | **422** | `rejected` | an illegal target, an unpayable cost. Understood, addressed to a real action, but unprocessable. The `GameStateError` is rendered as **text**. |
| the engine failed while advancing a *bot* seat | **500** | `engine_error` | the human's request was fine; the fault is on the server's side of the boundary |
| no game in progress | **404** | `no_session` | the absence of the resource the route names. Remedy: `POST /api/game`. |
| the request body is not valid JSON | **400** | `malformed_json` | axum's own `JsonSyntaxError`, re-wrapped in the envelope |
| the body is valid JSON of the wrong shape | **400** | `invalid_body` | a missing field, a wrong type, or — every request DTO is `deny_unknown_fields` — a misspelled one. **Remapped from axum's 422**, see below |
| the body is not sent as `application/json` | **415** | `unsupported_media_type` | axum's own status, re-wrapped. Not applicable to `POST /api/game`, where an absent body is legal and means "use the CLI defaults" |
| the session mutex was poisoned | **500** | `session_poisoned` | a previous handler panicked mid-mutation; the session is not trustworthy. **`POST /api/game` is the exception** — see below |

The 400-vs-422 split is the one worth internalising: **400 means the request
never reached the engine, 422 means the engine looked at it and said no.**

That rule is why a deserialization failure is reported as **400 even though
axum's own `JsonDataError` is a 422**. Left alone it collided head-on with the
row above it: a client-side typo (`"target"` for `"targets"`) came back as a 422
with a `text/plain` body and no `kind`, and a client branching on 422 would tell
the user the *engine* had rejected their play. The handlers therefore extract
with `Result<Json<T>, JsonRejection>` and re-wrap.

The same change fixed a quieter one: `POST /api/game` used to take
`Option<Json<NewGameRequest>>`, and `Option<T>`'s `FromRequest` impl is
`T::from_request(..).ok()` — so `{"playerz": 9}` was swallowed and answered
**200 with a default four-player game**. An absent body still means "use the CLI
defaults"; a malformed one is now a 400 and starts no game.

### Poisoning: `POST /api/game` recovers, everything else does not

Once a handler has panicked while holding the session mutex, every route answers
`500 session_poisoned` — except `POST /api/game`, which clears the poison and
starts a new game. The asymmetry is deliberate: that handler overwrites the
session wholesale and reads no game out of the poisoned value, and this surface
runs with `check_invariants: true` and live debug assertions specifically so that
engine panics show up, so costing a process restart per panic is the wrong price.
The one thing it does read across the recovery is the two `u64` wire-`seq`
counters, which are copies rather than invariants — that is what keeps the `seq`
guarantee below true through a panic.

### Wire `seq` is not `LocalGame`'s `seq`

`LocalGame::start` resets its `decision_seq` to 0, and this server calls it on
**both** `POST /api/game` and `POST /api/game/mulligan`. Taken literally that
would make the first decision of every game `seq: 1`, and a tab still rendering
the previous game's `seq: 1` could post against the new one and be *matched* —
applying whatever `action_index` meant in the old list to a table it had never
seen, with the 409 never firing.

`PlaySession` therefore adds a `seq_base`, set on each rebuild to one past the
highest wire `seq` the previous session ever issued, and translates in both
directions. The wire `seq` is monotonic for the life of the process (it skips a
value at each rebuild, which nothing depends on), a `seq` from a superseded game
is always strictly below the current base, and `checked_sub` — never a bare
subtraction on a client-supplied `u64` — turns that into a `stale_decision` 409
with a truthful `expected`/`got`.

---

## Decision record: no WebSocket, no SSE in M11-local

**Decision.** M11-local ships plain HTTP request/response. There is no
WebSocket, no server-sent-event stream, and no long-poll.

**Reasoning, not just the conclusion.** Bots act *synchronously inside the same
request that carries the human's action*: `POST /api/game/action` calls
`LocalGame::submit` and then `LocalGame::advance` before it returns, and
`advance` runs every bot seat until the human must act again. The response
therefore already contains the state the human next has to make a decision
about, together with every event produced along the way. There is no moment at
which the server knows something the client has not been told in a response it
is already waiting for — which is precisely the condition a push channel exists
to remove. Adding one here would buy nothing and cost a second transport, a
reconnection story, and a second serialization path to keep redaction-correct.

The corollaries are worth stating because they are what would change:

- **A second human seat would break the premise.** Two humans means one client
  can be idle while another acts, and then the server *does* hold news the idle
  client is not waiting on. M11-local has exactly one human seat by definition.
- **`GET /api/game` is the whole resync story.** It is idempotent (see
  Limitations) and returns the events since the client's cursor, so a refreshed
  or reconnected tab catches up with one request.
- **Push infrastructure is M10a's problem.** M10a is the networked server
  milestone; it needs event fan-out to N remote clients with per-seat redaction
  at the transport, which is a different design from this one and should not be
  prototyped here.

**Where this is recorded.** In this README, and — per plan item 8 — in
`memory/decisions.md`, which **Session 8** writes when it collects the
milestone's documentation updates. It is deliberately not written there yet.

---

## Known limitations

These are real and deliberate. The code documents each in place; they are
repeated here so this README does not claim more than the implementation does.

1. **The mulligan rebuilds the whole table, not one seat.** `POST
   /api/game/mulligan` delegates to `mtg_simulator::setup::redeal`, which
   re-rolls every seat from a perturbed seed. Two consequences: the command zone
   is public (CR 903.6), so a redeal is *not* invisible to the other players
   because it changes their commanders; and a partially-decided table cannot be
   represented at all, because CR 103.5c gives each player their own mulligan
   count and one `(seat, count)` signature has nowhere to record that seat 2
   already kept. A per-seat model needs each bot seat to be *asked*, which is a
   new decision channel rather than a small addition.

2. **`cards_to_bottom` is refused with 400.** CR 103.5's bottoming half is not
   expressible on the redeal path: `handle_keep_hand` checks
   `cards_to_bottom.len()` against `PlayerState::mulligan_count`, which a rebuild
   always leaves at 0 because no `Command::TakeMulligan` is ever issued. A
   non-empty list is therefore rejected loudly rather than accepted and silently
   discarded.

3. **`GET /api/game` calls `advance()`.** It is a read that can move the game.
   In practice it is a no-op: `LocalGame::advance` is idempotent while a decision
   is outstanding (it re-issues the same `seq` rather than minting a new one),
   and every request leaves the game parked at `AwaitingHuman` / `GameOver` /
   `Halted`. It is what lets a plain `GET` report a concluded game consistently
   without the session caching its last outcome. It *does* advance
   `journal_cursor` — reading the events consumes them.

4. **`target_slots` and `modes` are always empty this session.** The DTO fields
   ship now so the wire shape is settled before the frontend lands; **Session 7**
   populates them from `crates/engine/src/rules/queries.rs`
   (`spell_target_requirements` + `legal_targets_per_slot`). Until then a client
   must supply targets itself in `params.targets`.

5. **`needs_x` answers `CastSpell` only.** An activated ability's `{X}` lives in
   its `ActivationCost`, which `LegalAction::ActivateAbility` does not carry;
   Session 7 answers that half alongside `modes`.

6. **One game per process.** The session is a single
   `Arc<Mutex<Option<PlaySession>>>`; `POST /api/game` replaces it. That is the
   shape M11-local wants (one local player) and explicitly not a lobby.

7. **`GameSummary.seed` is the base seed, not the effective one.** After a
   mulligan the table was built from a derived seed; the reproduction key is
   `seed` + `players` + `bot` + `mulligan_count`. See "Reproducing a table from a
   bug report" above.

---

## Hidden information

Architecture Invariant 7 is enforced at one chokepoint, `api.rs::seat_view`: the
state is built with `StateViewModel::from_game_state_for(.., Viewer::Seat(human))`
and every event line goes through `event_view_for(.., Viewer::Seat(human))`.
Neither omniscient entry point (`from_game_state`, `Viewer::Omniscient`) is
reachable from the production paths of this crate.

Every *label* this crate renders is built from a `NameIndex` derived from that
already-redacted view, never from `state.objects()`. This is the Session 4 review
finding applied forward: **redaction follows the rendering site, not the zone.** A
face-down attacker is on the battlefield, so a zone-shaped redaction "covers" it
while a label that names it leaks. An id the redacted view does not identify
renders as `(hidden card)`.

---

## Testing

```sh
cargo test -p play-server
```

**The hard rule: no test in this crate may bind a port.** Every HTTP test drives
`build_router(state, &PathBuf::from("nonexistent_dist"))` through
`tower::ServiceExt::oneshot`. Nothing in `#[cfg(test)]` may reach
`tokio::net::TcpListener`, `axum::serve`, or `async_main` — those exist only on
the `main` path. This is session plan §7 constraint 1; an agent context that
starts the real binary gets SIGKILLed (the replay-viewer OOM/137 note in
`memory/gotchas-infra.md`).

That rule is **machine-enforced**, not merely stated:
`test_no_socket_symbol_appears_in_the_test_region` reads `src/main.rs` with
`include_str!`, cuts at the `#[cfg(test)]` attribute, and fails on any of the
four symbols below the cut. Its own needles are assembled with `concat!` so the
gate does not match its own source — keep any needle you add in that form.

Two further rules the tests depend on:

- **`#[tokio::test(flavor = "multi_thread")]` on every async test.** The handlers
  use `tokio::task::block_in_place`, which panics on the current-thread runtime a
  plain `#[tokio::test]` builds.
- **Fixtures are seed-pinned.** Every card name and count asserted in
  `main.rs`'s test module was read off a real run at seed 0, not reasoned to, and
  the seat-redaction test cross-checks the HTTP payload against the omniscient
  `StateViewModel` obtained out of band rather than against its own expectations.

One test is deliberately slow: `test_post_action_after_game_over_returns_409`
plays a two-seat table to a real CR 104.2a conclusion (~1,000 human actions,
~3 s) because that is the only way the HTTP surface reaches
`NoPendingDecision`. `LegalAction::Concede` is never offered by the provider and
`HaltReason::MaxTurns` needs 200 turns rather than ~110, so there is no cheaper
route to the *same* arm.

`test_post_game_recovers_from_a_poisoned_lock` panics a helper thread on
purpose. Its panic message on stderr is expected output, not a failure.
