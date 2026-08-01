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
| `src/api.rs` | the axum handlers — the only async code in the crate |
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

Two residual exceptions, named rather than glossed: a path no route matches
(**404**) and a wrong method on a routed path (**405**) are answered by axum's
router itself, with an **empty body and no `Content-Type`**. Both are decided
before any handler exists to answer them. Note the first is distinct from the
enveloped `404 no_session` below: that one comes from a handler and does carry
`kind`. Both statements are read off axum's routing behaviour, not held by a test
in this crate.

The `dist/` caveat applies to the **404 only** (third audit LOW 7). An unmatched
*path* is answered by the router's path fallback, and `build_router` mounts a
`ServeDir` fallback only `if dist_dir.exists()` — so with a built frontend the
404 becomes whatever `ServeDir` says. A **405** is decided by the `MethodRouter`
for an already-matched path and never reaches the path fallback at all, so
`dist/`'s presence is irrelevant to it. The inline tests build the router with a
deliberately absent `dist/`, which is why a 404 in them really is "the API said
404".

| Condition | Status | `kind` | Reasoning |
|---|---|---|---|
| `seq` does not match the outstanding decision | **409** | `stale_decision` | the client answered a superseded action list; retrying against the current `seq` will work. The message carries `expected` and `got` so the client can resync in one round trip. `seq` is monotonic for the life of the process, **across restarts and mulligans** — see Wire `seq` below. |
| no decision is outstanding | **409** | `no_pending_decision` | well-formed, but conflicts with the current state of the resource |
| `POST /api/game/mulligan` after a command has been applied | **409** | `not_pregame` | CR 103.5 is a pregame action; a rebuild would discard real play |
| `action_index` is not in the list just sent | **400** | `unknown_action` | the request is malformed on its face |
| a param the chosen action has no channel for | **400** | `bad_params` | `ParamError::UnsupportedParam` — wrong against *any* state, so a client error rather than an engine rejection. Refusing beats silently discarding a human's announced targets. **Also emitted by `POST /api/game/mulligan` for a non-empty `cards_to_bottom`**, which is refused in the handler and never goes near `LocalGame::submit` — see Limitation 2. |
| `players` outside `2..=6` | **400** | `bad_player_count` | a range check this crate makes; wrong against every state and never reaches engine code |
| `bot` is neither `"heuristic"` nor `"random"` | **400** | `bad_bot_kind` | likewise. The comparison is **case-insensitive** (`parse_bot_kind` lowercases first), so `"Heuristic"` is accepted |
| the **engine** refused the command | **422** | `rejected` | an illegal target, an unpayable cost. Understood, addressed to a real action, but unprocessable. The `GameStateError` is rendered as **text**. |
| `validate_deck` refused a seat's deck | **422** | `setup_failed` | Architecture Invariant 9 / CR 903.5c. Reachable from a client-supplied `seed` — see the 400/422 rule below |
| pregame assembly failed some *other* way | **500** | `setup_internal` | `SetupError`'s three non-`validate_deck` variants — no deck could be built for a seat, a `CardId` has no definition in the pool, or `GameStateBuilder::build()` failed. All server-side faults, none reachable today. See below |
| `start_game` refused the assembled table | **500** | `start_failed` | `check_all_defs_complete` rejecting a table *this server itself* put together is a server-side fault, not a bad request |
| an engine failure reaches `From<LocalGameError>` | **500** | `engine_error` | **currently unreachable** — kept as the correct mapping, not as a live contract. See below. |
| no game in progress | **404** | `no_session` | the absence of the resource the route names. Remedy: `POST /api/game`. |
| the request body is not valid JSON | **400** | `malformed_json` | axum's own `JsonSyntaxError`, re-wrapped in the envelope |
| the body is valid JSON of the wrong shape | **400** | `invalid_body` | a missing field, a wrong type, or — every request DTO is `deny_unknown_fields` — a misspelled one. **Remapped from axum's 422**, see below |
| the body is not sent as `application/json` | **415** | `unsupported_media_type` | axum's own status, re-wrapped. Not applicable to `POST /api/game`, where an absent body is legal and means "use the CLI defaults" |
| the session mutex was poisoned | **500** | `session_poisoned` | a previous handler panicked mid-mutation; the session is not trustworthy. **`POST /api/game` is the exception** — see below |

The 400-vs-422 split is the one worth internalising:

> **400 means this crate refused the request before any engine code judged it.
> 422 means engine code looked at what the request asked for and said no —
> `process_command` for a command, `validate_deck` for a pregame table.**

That is the rule restated (S5 re-review MEDIUM 4). The earlier form said "400
means the request never reached the engine", and `setup_failed` broke it: a
pregame deck-assembly failure never calls `process_command`. The status is right
and the sentence was wrong. `POST /api/game {"seed": 17}` is syntactically
perfect — a legal `u64`, nothing malformed on its face — and fails because the
*table that seed builds* is illegal: `deck::basics_for_colors` pads a colourless
commander's deck with Forests and `validate_deck` refuses them under CR 903.5c.
That is the textbook 422. A sweep of 180 `(players, seed)` pairs found 7 such
tables, so this is a route a client can take, not a theoretical one.

The restated rule grounds 422 in `validate_deck`, and the third audit's LOW 3
pointed out that only **one** of `SetupError`'s four variants is a `validate_deck`
judgment. The other three — `NoDeckForSeat` (the card pool had no legendary
creature), `MissingCardDefinition` (documented in `crates/simulator/src/setup.rs`
as "a defensive check at spec-build time", and this crate never passes a
`DeckSource::Fixed` deck), and `Builder(GameStateError)` — are server-side faults
by the same argument that puts `start_failed` at 500, so they are matched by
variant and answered **500 `setup_internal`**. None is reachable today (`players`
is range-checked to `2..=6` first, and the pool always contains a legendary
creature), so nothing was lying before; the point is that the *rule* is now true
rather than merely narrowed.

The same rule is why a deserialization failure is reported as **400 even though
axum's own `JsonDataError` is a 422**. Left alone it collided head-on with the
`rejected` row: a client-side typo (`"target"` for `"targets"`) came back as a
422 with a `text/plain` body and no `kind`, and a client branching on 422 would
tell the user the *engine* had rejected their play. The handlers therefore
extract with `Result<Json<T>, JsonRejection>` and re-wrap.

#### `engine_error` is unreachable, and is documented rather than removed

`LocalGameError::Engine` is constructed at exactly one site in the workspace,
`LocalGame::start`, which this crate routes through `SessionError::Start` to
**500 `start_failed`**. The only expression feeding `From<LocalGameError>` is the
`play.submit(..)?` in `post_action`, and `LocalGame::submit` returns only
`NoPendingDecision`, `StaleDecision`, `UnknownAction`, `BadParams` and
`Rejected`. Nor is the row's old description right about what *would* happen: an
engine failure while advancing a bot seat becomes
`AdvanceOutcome::Halted(HaltReason::EngineError(..))` — `LocalGame::advance`
returns no `Result` at all — and is answered with **200** and
`game_over.halted == true`. The match arm stays because the `match` is exhaustive
over a plain enum and a wildcard would silently swallow a future variant
(S5 re-review MEDIUM 3).

The same change fixed a quieter one: `POST /api/game` used to take
`Option<Json<NewGameRequest>>`, and `Option<T>`'s `FromRequest` impl is
`T::from_request(..).ok()` — so `{"playerz": 9}` was swallowed and answered
**200 with a default four-player game**. An absent body still means "use the CLI
defaults"; a malformed one is now a 400 and starts no game.

### Poisoning: `POST /api/game` recovers, everything else does not

Once a handler has panicked while holding the session mutex, every route that
takes the lock answers `500 session_poisoned` — except `POST /api/game`, which
clears the poison and starts a new game. (Two carve-outs: `GET /api/healthz`
never takes the lock and keeps answering 200, and the pre-lock 400s —
a body the extractor rejects, a `players` out of range, an unknown `bot`, a
non-empty `cards_to_bottom` — are decided before the lock and keep their 400.)

The asymmetry is deliberate: that handler discards the corrupt session outright
and never plays on with it, and this surface runs with `check_invariants: true`
and live debug assertions specifically so that engine panics show up, so costing
a process restart per panic is the wrong price. The one thing it reads across the
recovery is `next_seq_base()` — a single `u64` high-water mark, a copy rather
than an invariant — which is what keeps the `seq` guarantee below true through a
panic.

**The recovery is atomic.** The corrupt session is `take()`n out of the `Option`
in the same straight-line block that clears the flag, with no fallible operation
between the two. The first fix cycle got this wrong: it cleared the flag and
relied on the assignment at the *end* of the handler to remove the corrupt value,
with the fallible `session::new_game` in between. Because `new_game` fails on a
client-supplied seed (see the 400/422 rule above), `POST /api/game {"players": 2,
"seed": 17}` against a poisoned lock left the half-mutated session in place with
the flag cleared, and the next `GET /api/game` answered **200** with its full seat
view — where before that "fix" it answered 500. Observed on a real run;
`test_poison_recovery_is_atomic_when_the_rebuild_fails` pins it. A failed rebuild
now leaves **no** session, so the next `GET` is `404 no_session`.

The healthy-path half is the mirror image and is now pinned too
(`test_a_failed_rebuild_leaves_a_running_game_untouched`, third audit LOW 4): on a
**non**-poisoned lock the handler only *peeks* at the seq counter, so a rebuild
that fails leaves a running game exactly as it was — same outstanding `seq`, same
`command_count`, still answerable. That was true in the code and asserted nowhere.

Both tests reach a failing rebuild the only way a client can: the CR 903.5c Forest
padding filed as `OOS-M11-6`. Closing that seed will need a replacement failure
mode for them; they fail loudly rather than passing vacuously if it disappears.

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

That rule is **machine-enforced across the whole crate**, not merely stated:
`test_no_socket_symbol_appears_in_the_test_region` walks every `.rs` file under
`src/` and `tests/` (rooted at `CARGO_MANIFEST_DIR`, so the walk does not depend
on the process's working directory) and fails on any of the four symbols inside a
test region. In a `src/` file the region starts at the first **line-anchored**
`#[cfg(test)]` or `#[cfg(all(test…` attribute; a `tests/` file is checked in
full, because an integration test carries no such attribute. Its own needles are
assembled with `concat!` so the gate does not match its own source — keep any
needle you add in that form.

**What "crate-wide" does and does not promise.** The gate recognises two
spellings of the attribute, and a file it cannot cut is no longer skipped in
silence: a `src/` file whose *code* is test-shaped (`#[test]`, `#[tokio::test`,
`fn test_`, `mod tests`) but whose region comes out empty is a **failure**, and
the message names the file. So the honest claim is *a file Session 6 or 7 adds is
either checked, or the gate goes red naming it* — not "every arrangement is
understood automatically". The earlier claim was the second, and it was false:
before the third audit's MEDIUM 1, a `#[cfg(test)] mod tests;` split with the
body in `src/tests.rs`, and a `#[cfg(all(test, feature = "x"))]` module, each
carried a `TcpListener::bind` call **past a green gate**. Both were run, not
reasoned about.

The non-vacuity guard that requires each needle to occur in real code above the
cut strips **comments and string-literal bodies** (`code_only`), not just
whole-line `//` comments. That matters concretely: the serving path's own
`format!("Failed to bind to {addr}")` is a code line whose *message* contains the
`bind` needle, so the older line-comment-only stripper let that guard pass on a
diagnostic string rather than on the call. Demonstrated both ways — with the real
call removed and the message kept, the old stripper stayed green and the new one
went red. `code_only` handles line comments, nested block comments, raw strings and char
literals. Only the first two of those are exercised by any file in this crate —
the rest are defensive against source that does not exist here yet, and are
therefore pinned directly by `test_code_only_blanks_comments_and_string_bodies`
rather than left as an unchecked claim in a doc comment. It is a lint over source
text, not a Rust lexer, so macro-generated text is invisible to it by
construction.

The earlier widenings came from the S5 re-review and were likewise proven by
execution: a forbidden symbol inserted into a `#[cfg(test)]` module in
`src/session.rs`, and into a new `tests/tmp_probe.rs`, each reddened the gate and
named the offending file; both were removed again. The version before that read
`src/main.rs` alone **and** cut at the first textual `#[cfg(test)]`, which is the
one written out in that file's own module doc comment — so the "test region"
began at a paragraph of prose and the gate passed only because all four symbols
happen to be typed to the left of the marker in that one sentence. Rewording the
paragraph would have turned the gate red against its own documentation.

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
