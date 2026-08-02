# `play-server` — the M11-local play surface

One human seat, three simulator bots, over HTTP.

This is the first-playable host: a browser plays a real Commander game against
`HeuristicBot`s driven by the same `LocalGame` the fuzzer uses. It is the **only
crate in M11-local with async or IO** (`memory/m11-session-plan.md` §3) —
`crates/engine`, `crates/simulator` and `crates/view-model` stay pure, per
Architecture Invariant 1.

Built by M11-local **Session 5** (plan §4); **Session 6** added the Svelte
frontend under `frontend/`, which this binary serves from `dist/`; **Session 7**
added targeting, combat and choice — per-slot target candidates and the CR 508.1
/ CR 509.1 combat payloads on the wire, server-side validation of a submitted
declaration, and the four pickers that fill `params`.

---

## What it is made of

| File | Role |
|---|---|
| `src/main.rs` | clap CLI, hand-built tokio runtime, `build_router`, the inline HTTP tests, the no-socket source gate |
| `src/session.rs` | `PlaySession` lifecycle — build, advance, submit, mulligan. **Synchronous; knows nothing about tokio.** |
| `src/api.rs` | the axum handlers — the only async code in the crate |
| `src/view.rs` | wire DTOs and the server-side rendering that produces them |
| `frontend/` | the Svelte 5 client (Session 6). Builds to `dist/`; **no Rust code and no test target** — see "The frontend" below |

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

## The frontend (`frontend/`)

Built by M11-local **Session 6** (plan §4). Vite + Svelte 5 (runes), the same
versions as `tools/replay-viewer/frontend`: `svelte ^5.45`, `vite ^7.3`,
`@sveltejs/vite-plugin-svelte ^6.2`.

```sh
cd tools/play-server/frontend
npm install
npm run build          # -> tools/play-server/dist/, which the ServeDir fallback mounts
npm run dev            # Vite on :5173, proxying /api -> 127.0.0.1:3040
```

`npm run build` is the gate: `outDir` is `../dist`, and `build_router` mounts a
`ServeDir` on that directory **only if it exists**, so a missing build is the
difference between the play surface and a bare JSON API. The binary prints which
of the three candidate `dist/` paths it found (or that it found none).

| File | Role |
|---|---|
| `vite.config.js` | the `$viewer` alias, the dev proxy, `outDir: ../dist` |
| `src/lib/api.js` | `newGame` / `getGame` / `submitAction` / `mulligan`; unwraps the `{error, kind}` envelope onto the thrown `Error` |
| `src/lib/stores.js` | `seatView` / `decision` / `events` / `loading` / `error`, and the helpers that own every fetch |
| `src/lib/PlayApp.svelte` | layout, pregame block, game-over banner, click-through |
| `src/lib/ActionBar.svelte` | the decision as buttons, the error strip, the keyboard shortcuts |
| `src/lib/EventFeed.svelte` | the rendered, already-redacted history lines |
| `src/lib/TargetPicker.svelte` | CR 601.2c — one selector per target slot, range-checked (Session 7) |
| `src/lib/AttackerPicker.svelte` | CR 508.1a — attacker multi-select plus a per-attacker `AttackTarget` (Session 7) |
| `src/lib/BlockerPicker.svelte` | CR 509.1a — blocker → attacker pairing (Session 7) |
| `src/lib/ValuePrompt.svelte` | CR 601.2b `{X}` and CR 700.2 mode selection (Session 7) |

### `$viewer` imports the replay viewer's components, it does not copy them

`vite.config.js` aliases `$viewer` → `tools/replay-viewer/frontend/src/lib`, so
`PhaseIndicator` and `StateView` (and, through the latter, `PlayerPanel`,
`ZoneHand`, `ZoneBattlefield`, `ZoneStack`, `ZoneGraveyard`, `ZoneExile`,
`cardTooltip`) are compiled **in place** from the viewer's tree — 143 modules in
the production build (135 before Session 7's four pickers), against twelve files
of our own. This is the
mechanism `docs/mtg-engine-replay-viewer.md` §"Import Mechanism" anticipated when
it made those components props-based.

A copy would fork. `crates/view-model`'s `stack_kind_info` / `format_keyword`
matches are exhaustive over `StackObjectKind` and `KeywordAbility`, and the
components downstream of them are written against those shapes; a duplicate would
go stale on the next variant with nothing to make it fail. Promotion to a shared
`tools/ui-shared/` package is deferred — plan §8 R8.

The evidence that the alias resolves rather than silently falling back: the
production bundle contains the viewer components' scoped CSS
(`grep zone-battlefield dist/assets/*.css`), and
`find frontend/src -type f` lists twelve files, none of them a `Zone*` or
`PhaseIndicator`.

### Interaction

- **Buttons** — every option in `decision.actions`, labelled server-side.
  `PassPriority` and `Concede` are pulled into their own group so "pass" does not
  move as the list grows. Every button is disabled while a request is in flight.
- **Click-through** — clicking a card in the hand or on the battlefield matches
  `decision.actions` by `object_id`. One match submits; several offer an inline
  chooser; none explains, naming the card and listing what *is* offered. It
  deliberately does **not** invent a rules reason, because the server does not
  send one: a used land drop, an unpayable cost and a sorcery-speed restriction
  are indistinguishable from the client.
- **Pickers (Session 7)** — clicking an option no longer submits `{}`. `ActionBar`
  opens whichever pickers that option's fields call for, in CR order — `ValuePrompt`
  (CR 601.2b announces `{X}` and modes) → `TargetPicker` (CR 601.2c) →
  `AttackerPicker` (CR 508.1) → `BlockerPicker` (CR 509.1) — accumulating one
  `params` object and submitting once at the end. An option needing none of them
  still submits `{}` immediately. Click-through goes through the same entry point,
  so a targeted spell cannot be cast targetless from either path.
- **Keyboard** — `space` submits the `PassPriority` option (found by `kind`,
  never by index), `Esc` cancels the chooser, aborts an open picker chain, and
  dismisses the error strip. Both are ignored while the focus is in an input, so
  typing a seed does not pass priority; `space` is additionally suppressed while a
  picker is open, so it cannot pass priority underneath one.
- **Errors** are shown, not logged. A 422 `rejected` reads as "the engine refused
  this play" and carries the `GameStateError` text; a 409 `stale_decision`
  re-reads `GET /api/game` on its own.

### The one change Session 6 made *outside* `tools/play-server`

`tools/replay-viewer/frontend/src/lib/ZoneHand.svelte` keyed its `#each` on
`card.object_id`. That is fine for the replay viewer, which is omniscient and
gives every hand card a distinct id — and **fatal** for this app, because
`mtg_view_model`'s `redact::redact_hands` replaces each card of a hand the seat
may not read with `redact::hidden_placeholder()`, whose `object_id` is **0**. A
redacted 4-player table therefore hands `ZoneHand` three seven-card hands with one
distinct key each, and Svelte 5's keyed reconciler evaluates `length > keys.size`
and calls `each_key_duplicate`, which **throws in production as well as in DEV**.
With no `<svelte:boundary>` above it, the throw escapes the effect flush and takes
the whole mount down: the play surface rendered *nothing at all*.

Measured on a real payload rather than argued: `Bot-2`/`Bot-3`/`Bot-4` each came
back `length 7, keys.size 1`; the seat's own hand `length 7, keys.size 7`. The
key is now `card.hidden ? \`hidden-${i}\` : card.object_id` — keyed on the flag the
redactor actually sets rather than on the sentinel value 0, and inert for the
replay viewer, which never sets `hidden` on a hand card.

`hidden_placeholder` is called from exactly one site (`redact_hands`), so hands
are the only zone that can contain duplicate ids; the command zone is public
(CR 903.6) and is not redacted this way. That was checked, not assumed.

This is the shared-component tax that plan §8 R8 defers by keeping `$viewer` an
alias rather than a package: a component with two consumers now has to be correct
for both. Fixing it in the viewer rather than working around it here is the whole
point of not copying.

### The three S6 gaps, and what Session 7 did to each

> **All three are closed.** The account below is kept because the *shapes* of the
> three failures are the durable lesson — one loud, two silent — and because the
> S7 pickers are only meaningful as answers to them.

`ActionOptionView.target_slots` and `modes` were empty until Session 7
(Limitation 4 above), and the client sent `params: {}`. So casting a *targeted*
spell from this UI failed with a real 422 — observed verbatim during the S6
verification:

```
{"error":"invalid target: expected 1..=1 target(s) but got 0","kind":"rejected"}
```

That is the correct behaviour for S6 (the engine refused an under-specified
announcement, CR 601.2c) and it is exactly what S7's `TargetPicker` closes. The
error strip surfaces it rather than swallowing it, so the failure is legible
instead of mysterious.

**Combat is the same gap with the opposite failure mode, and it is the more
dangerous one.** `params.rs` maps `LegalAction::DeclareAttackers` with default
params straight to `Command::DeclareAttackers { attackers: vec![] }` — a legal,
irreversible "I attack with nothing" for that combat — and likewise for blockers.
Nothing refuses it, so unlike the targeted spell above there is no 422 to read;
the human's combat step is simply gone. The buttons stay **enabled** (at a
`DeclareAttackers` decision the declaration is usually the only option offered, so
disabling it would deadlock the game, and CR 508.1 makes declaring no attackers a
legal choice), but they are marked `declares none` and their tooltip says so. S7's
`AttackerPicker` / `BlockerPicker` are what actually close it.

**An activated ability's `{X}` is announced as 0, and the client cannot even tell
which abilities have one.** `params.rs` maps `LegalAction::ActivateAbility` with
default params to `x_value: None`, which `abilities.rs` reads as `unwrap_or(0)`;
`action_needs_x` answers `CastSpell` only (Limitation 5), so `needs_x` is `false`
here whether or not there is an `{X}` to announce. Reachable and destructive on a
deck-legal card, not theoretical: `mirror_entity` is `Complete` (by the `#[default]`
derive) and its activated ability has `x_count: 1`, so one click makes every
creature 0/0 and the board dies to state-based actions with no error to read.
Because there is no flag to branch on, the `X = 0` tag on those buttons is
**unconditional** rather than conditional — noisier than it should be, and the
right trade until S7 closes Limitation 5 and can populate `needs_x` for abilities.

The three paragraphs above are the same underlying hole in three shapes: the
client can only send `params: {}`. Only the first of them fails loudly.

**Session 7 closes all three, and the asymmetry is why it is worth recording.**
The targeted-spell case announced itself every time it happened; the other two
were indistinguishable from a normal click. `TargetPicker` fills `params.targets`
from the server's per-slot candidate lists; `AttackerPicker` / `BlockerPicker`
fill `params.attackers` / `params.blockers` from the provider's own eligibility
lists, and **`api.rs::validate_combat_params` refuses a pair the decision never
offered with a 400** so a picker bug cannot quietly become a different legal
declaration; `ValuePrompt` fills `params.x_value` and `params.modes_chosen`, and
`needs_x` now answers `ActivateAbility`, so `mirror_entity` gets a prompt instead
of a silent `X = 0`. The `declares none` and `X = 0` badges are gone from
`ActionBar.svelte` — they were warnings about an absence, and the absence is
filled.

### The second change outside `tools/play-server` (Session 7)

S6's was `ZoneHand.svelte`. S7's is `crates/view-model`: `StackItemView` gains
**`source_object_id`**.

`StackItemView::id` is the **`StackObject`** id. `mtg_engine::Target::Object`
names the underlying **`GameObject`**. They are different id spaces, and nothing
in the view model bridged them — so a target that is a spell on the stack, which
is every counterspell's target, had no entry in `NameIndex` and rendered as
`(unknown card)`. Observed on a real payload before the fix, not reasoned about:
seed 2 offers `Cast Dispel` with one candidate, and its label came back
`"(unknown card)"` against a stack holding `Dark Ritual`.

The id was already being computed in `build_zones_view` (for `source_name`) and
discarded. Exposing it leaks nothing: CR 405.1 makes the stack public, the view
already ships `source_name` for every entry with `redact_stack` blanking a
face-down source's *name*, and a face-down **permanent** already keeps its real
`object_id` for exactly the same reason.

The fix also removed a latent hazard on this side: `NameIndex` used to write
`item.id` — a `StackObject` id — into a map keyed by `ObjectId`. Nothing ever
looked it up, and the stack is inserted last, so a numerically-colliding id could
have overwritten a real permanent's name.

### Manual checklist (plan item 7)

There is no frontend test harness in this repo — the API tests are the automated
coverage (18 before Session 7 — 16 from Session 5 plus 2 PB-DX4 added, 24 after Session 7's five plus the omniscient-view source gate), and Session 6 added no
test target. What follows is the
checklist the plan asks for, with **each step marked by what was actually done**,
not by what a browser would presumably show.

**Method for the steps marked "payload".** A temporary `#[ignore]`d probe was
added to `src/main.rs`'s existing `mod tests`, run, and **removed again** (`git
diff` over `tools/play-server/src/` is empty). It drove `build_router(..)` through
`tower::ServiceExt::oneshot` — **binding no port**, like every other test in this
crate — with the checklist's own policy ("play a land if one is offered, else
pass"), at the pinned `--seed 0 --players 4 --bot heuristic`, and dumped every
`SeatView` to JSON. The frontend was then checked against those real payloads
rather than against a written-down idea of them. A second run preferring
`CastSpell` produced the stack observation and the 422 above.

| # | Step | Status | What was actually established |
|---|---|---|---|
| 1 | Launch: `cargo run -p play-server`, open `http://127.0.0.1:3040` | **unverifiable headless** | Starting the binary binds a port. Forbidden by plan §7 constraint 1, and an agent context that starts a server like this gets SIGKILLed (the replay-viewer OOM/137 note in `memory/gotchas-infra.md`). Nothing was run. |
| 2 | The page is served at all | **verified (build)**, and the rendering is **partly** checked | `npm run build` emits `dist/index.html` referencing `/assets/index-*.js` and `/assets/index-*.css`; `build_router` mounts `ServeDir::new(dist).append_index_html_on_directories(true)` as the path fallback when `dist/` exists. The bytes exist and the route that serves them is the one already tested in S5. **The rendering itself was never observed in a browser** — and this row is exactly where the session's one real bug hid: a green build says nothing about whether the components survive a *redacted* payload. See "The one change outside `tools/play-server`" above; it was caught by evaluating Svelte's own `length > keys.size` condition against the dumped hands (`7 > 1` for each bot seat, `7 > 7` false for the human's), not by a build and not by a browser. |
| 3 | See a 7-card hand | **verified (payload)** | `POST /api/game` at the pinned seed returns `state.zones.hand["Human-1"]` with exactly 7 entries — Island, Mist Intruder, Misdirection, Nyxbloom Ancient, Accorder's Shield, Helm of the Host, Swan Song — each `hidden: false` with a real `object_id`. `PlayApp` passes that state to `$viewer/StateView`, which renders hands through `ZoneHand`. |
| 4 | Play a land | **verified (payload)** | The decision offered `{index: 1, kind: "PlayLand", object_id: 2, label: "Play Island"}`. Submitting index 1 moved the hand 8 → 7 and the battlefield 0 → 1, and emitted `LandPlayed` + `PermanentEnteredBattlefield`. Click-through matches on that same `object_id: 2`, so the button path and the click path submit the identical index. |
| 5 | Pass priority | **verified (payload)** | 25 `PassPriority` submissions, all 200, each returning the next decision with a fresh `seq`. |
| 6 | Watch the bots act in the event feed | **verified (payload)** | Each response carried 10–21 `EventView` lines (`{kind, text, player}`), e.g. `[TurnStarted] Turn 2 — Bot-2`, `[PriorityPassed] Bot-3 passes`. All are pre-rendered and seat-redacted by `event_view_for(.., Viewer::Seat(human))`; `EventFeed` prints `text` and adds no formatting of its own. `stores.js` **accumulates** them, which matters because the server sends only what is new since the last read. |
| 7 | See the battlefield update | **verified (payload)** | `state.zones.battlefield["Human-1"]` went 0 → 1 on the land drop; by turn 4 the bots' battlefields had grown too (`Bot-2: 1`, `Bot-3: 1`). |
| 7b | See the stack update | **verified (payload, second run)** | The land-only policy never put anything on the stack in three turns, so the claim was re-established rather than assumed: a second run preferring `CastSpell` cast Accorder's Shield and the next payload carried `zones.stack: [{id: 404, kind: "spell", source_name: "Accorder's Shield", controller: "Human-1"}]`. `StateView` renders a non-empty stack through `ZoneStack`. |
| 8 | Reach turn 3 | **verified (payload)** | Turn 3 reached after 16 submissions and turn **4** after 25, with `summary.turn` and `command_count` advancing monotonically. |
| 9 | Error surfacing | **verified (payload)** | A real 422 was produced and its body is the `{error, kind}` envelope quoted above. `api.js` lifts `error` onto `Error.message` and `kind` onto `Error.kind`; `ActionBar` renders both. |
| 10 | `space` = pass priority, `Esc` = cancel | **unverifiable headless** | Needs a browser event loop. The handler is bound on `window` in a `$effect` with cleanup, finds the pass option by `kind === 'PassPriority'` rather than by index, and returns early when the event target is an `input` / `textarea` / `select` / `contenteditable`. **Read, not observed.** |
| 11 | Clicking a card actually fires | **unverifiable headless** | The wiring was read end to end — `StateView` threads `onCardClick` into `ZoneHand`, `ZoneBattlefield`, `ZoneGraveyard` and `ZoneExile`, each calling `onCardClick?.(card)` on a `div` inside an `#each` keyed by `card.object_id` — and the *matching* it feeds was checked against real payloads (step 4). The DOM event itself was never dispatched. **`ZoneStack` is the exception and it is not a bug here**: it declares `onCardClick` as a prop and never invokes it, so a stack item is inert. Nothing in S6 needs it — every `LegalAction` with an `object_id` names a card or a permanent, never a stack object (`view.rs::action_object`) — but S7, which does render targets on stack items, should know the prop is dead rather than discover it. |
| 12 | Mulligan | **partly verified (payload)** | The route itself is S5's: `test_seq_from_before_a_mulligan_is_stale` posts `take: true`, and the `not_pregame` test posts `take: false`. What S6 established is which *path* the UI must use. The pregame block is gated on `summary.pregame` alone and **not** on `decision.kind === "Mulligan"`, because that kind is unreachable: `legal_actions.rs` and `local_game.rs::decision_kind_for` both gate the mulligan actions on `turn_number == 0`, and a freshly built table is already in turn 1 — `session.rs::is_pregame` says so in as many words. Confirmed in the payload: the pregame decision's `kind` is `Priority` and its only option is `Pass priority`. "Keep this hand" has **no server-side representation** (a `take: false` post only re-renders), so it is tracked client-side; that is recorded in `PlayApp.svelte` rather than hidden. |

Two things the checklist deliberately does not claim: that the layout looks
right, and that any of it works in a browser at all. Both need a human with the
server running, which is exactly what the checklist is *for*.

### Manual checklist, Session 7 additions (plan item 7)

The plan asks for three more steps: attack a bot, block a bot's attacker, and
cast a targeted removal spell on a bot's creature. **All three are now covered by
permanent automated tests rather than by a one-off probe**, which is a stronger
claim than the S6 rows above could make — but only for the *server* half. The
browser half is as unverifiable as it was, and is marked so.

The fixtures below were found by a temporary `#[ignore]`d probe that swept
`players` ∈ {2, 4} × `seed` ∈ 0..12 through `oneshot` (**no port bound**) and
reported which games ever offer the human a `DeclareAttackers`, a
`DeclareBlockers`, or a `CastSpell` with a non-empty candidate list. The probe was
then deleted; the numbers it produced are pinned as `COMBAT_SEED` / `TARGET_SEED`
in `src/main.rs` with a note saying to re-observe rather than guess them, because
a completeness flip in any card-def batch re-deals every seeded deck.

| # | Step | Status | What was actually established |
|---|---|---|---|
| 13 | Attack a bot | **verified (automated)** | `test_declare_attackers_through_api_emits_attackers_declared`. At `players: 4, seed: 6` the human is offered a `DeclareAttackers` with one eligible creature (Memnite) and three player targets. The test builds the declaration **only out of what the payload offered** — first `eligible` entry, first `targets` entry, echoing that target's `value` verbatim, exactly as `AttackerPicker` does — and asserts `GameEvent::AttackersDeclared` reaches the client as an `EventView`. It asserts the event, not the 200, because an *empty* declaration also answers 200: a status-only test would have passed against precisely the S6 bug this closes. |
| 14 | Block a bot's attacker | **verified (automated)** | `test_declare_blockers_rejects_ineligible_blocker`, same seed, one turn later. A blocker id the decision never offered is refused **400 `bad_params`** with a message naming CR 509.1a and the offending id; the decision is then confirmed still outstanding with `command_count` unchanged, so the refusal cost the human nothing. The control half asserts the pairing the payload *did* offer is **not** a `bad_params` — deliberately not asserted to be a 200, because the engine may still refuse it on a rule the provider does not model (flying, menace), which would be a legitimate 422. Non-vacuity was checked by neutering `validate_combat_params` and watching this test go red. |
| 15 | Cast a targeted removal spell on a bot's creature | **verified (automated)** | `test_action_option_target_slots_match_engine_query` at `players: 4, seed: 9` (Doom Blade). The wire payload is compared against a **second, independent** call to `spell_target_requirements` + `legal_targets_per_slot` made against the same `GameState`, on slot count, per-slot order, and `(min, max)`. The fixture requires a slot with **at least two** candidates: a first version stopped on a one-candidate slot, and reversing the candidate order in `view.rs` left it green — reversing a one-element list changes nothing. With the stronger fixture, that same perturbation turns it red. `test_x_value_is_forwarded_to_cast_spell_data` then drives the whole chain to `CastSpellData.x_value` by reading the applied `Command` out of `LocalGame::journal()`. |
| 16 | The pickers themselves | **unverifiable headless** | Every DOM and keyboard behaviour: clicking through the `ValuePrompt` → `TargetPicker` → `AttackerPicker` → `BlockerPicker` chain, Escape aborting a chain mid-way, `space` being suppressed while a picker is open, and the `<select>` default rendering in the attacker/blocker pickers. There is still no frontend test harness in this repo (plan §8 R7); `npm run build` is the only gate, and S6's row 2 is the standing proof that a green build says nothing about whether a component survives a redacted payload. **Read, not observed.** |

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
| `GET /api/game/report` | — | the bug-report / repro artefact. A **pure read** — see below |
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

4. ~~**`target_slots` and `modes` are always empty this session.**~~ **CLOSED by
   Session 7.** Both are populated from `crates/engine/src/rules/queries.rs`
   (`spell_target_requirements` / `ability_target_requirements` +
   `legal_targets_per_slot`), alongside `target_min`/`target_max` from
   `target_count_range` and the CR 508.1 / CR 509.1 combat payloads. What
   *remains* is narrower and is listed as limitations 6-9 and 12-13 below.
   (This pointer went stale once already, inside the very fix cycle that
   renumbered the list — the same cross-reference rot, one level up.)

5. ~~**`needs_x` answers `CastSpell` only.**~~ **CLOSED by Session 7.** The
   S6 note said `LegalAction::ActivateAbility` does not carry the ability's
   `ActivationCost`, which is true and was the wrong place to look: the action
   carries `source` and `ability_index`, and those are enough to reach the
   **layer-resolved** `Characteristics::activated_abilities` entry and read its
   `cost.mana_cost.x_count`. `mirror_entity` — the deck-legal card that made this
   destructive rather than theoretical — now reports `needs_x: true` and gets a
   real prompt.

6. ~~**A non-zero `{X}` cannot actually be paid for through this API yet.**~~
   **✅ FIXED in Session 8 — `OOS-M11-8` is CLOSED.** `LocalGame::auto_tap_commands_for`
   used to read the spell's **printed** `mana_cost` and know nothing about
   `cast.x_value`, so it tapped for the base cost and the engine then refused the
   cast — observed by S7 as `422 "player does not have enough mana to pay the cost"`.

   CR 107.3 / 601.2b: X is announced at cast time and is part of the cost from that
   moment, so `x_value × mana_cost.x_count` generic is now added before both the
   pool check and the solve. `x_count`, not a bare `+ x_value`, because `{X}{X}`
   costs 2X; saturating, so a hostile `x_value` cannot overflow into a small cost
   that then looks payable.

   The S7 note this replaces made a distinction worth keeping: its observation was
   on a spell with `x_count == 0` (where `casting.rs`'s documented fallback adds
   `x_value` to the generic cost), and the real `x_count > 0` path was inferred
   rather than exercised. It is exercised now —
   `crates/simulator/tests/local_game_human_actions.rs::test_s8_x_value_is_included_in_the_auto_tap_plan`
   casts a `{X}{1}` sorcery at X = 2 with four one-mana sources and asserts exactly
   three tap, which distinguishes it from both "tapped the printed cost" and
   "tapped everything". Verified to fail with the fix disabled.

7. **A modal action's target slots are per-mode and the option-level range is
   `(0, 0)` for such a card.** `spell_target_requirements` is queried at render
   time with an empty `modes_chosen`, because the human has not announced modes
   yet, and it deliberately answers `vec![]` for a card whose targets live in
   `ModeSelection.mode_targets` (its own divergence 1). Each `ModeOptionView`
   therefore carries its own `target_slots` + `target_min`/`target_max`, and the
   client sums the chosen modes'. **Untested against a live modal spell**: no
   game in the S7 fixture sweep (`players` ∈ {2,4} × `seed` ∈ 0..12) dealt the
   human one, so this path is right by construction and unexercised.

8. **`ModeOptionView.label` is a truncated `Debug` of the mode's `Effect`.**
    There is no per-mode oracle text anywhere in the DSL — `ModeSelection.modes`
    is a bare `Vec<Effect>` — so the label is machine-shaped ("Mode 2:
    DealDamage { .. }") rather than printed text, visibly so rather than
    pretending otherwise.

9. **The stack's `ModeSelection` lookup is the one engine rule this crate
    restates.** `rules::casting::spell_mode_selection` is `pub(crate)`, so
    `view::action_modes` re-derives it through the public
    `GameState::card_registry`. It is confined to *which modes to offer*; the
    engine re-validates `modes_chosen` on the cast path regardless (CR 601.2b,
    PB-DP3), so a drift here is a wrong picker, never a wrong game state.
    Everything else — target requirements, target legality, combat eligibility —
    is delegated.

10. **One game per process.** The session is a single
   `Arc<Mutex<Option<PlaySession>>>`; `POST /api/game` replaces it. That is the
   shape M11-local wants (one local player) and explicitly not a lobby.

11. **`GameSummary.seed` is the base seed, not the effective one.** After a
   mulligan the table was built from a derived seed; the reproduction key is
   `seed` + `players` + `bot` + `mulligan_count`. See "Reproducing a table from a
   bug report" above.

12. **`TargetPicker`'s multi-target slot is unexercised.** A
    `TargetRequirement::UpToN { count }` slot is one requirement worth up to
    `count` targets, and `TargetSlotView` carries its own `min`/`max` so the
    client can offer that. No seeded game in the S7 fixture sweep dealt such a
    card, so the multi-select branch ships correct-by-construction and untested.
    The reachable case is `force_of_vigor` (`Complete`, deck-legal, one
    `UpToN { count: 2 }`), which before the fix would have destroyed at most one
    of its "up to two" targets.

13. **`BlockerPicker` cannot express CR 509.1b "can block an additional
    creature".** Its model is one attacker per blocker. The *server* deliberately
    permits more — `validate_combat_params` rejects only the identical
    `(blocker, attacker)` pair twice, precisely so a creature with the ability is
    not blocked by the validator — so this is a client limitation, not a rules
    one. A blocker with the ability can still be assigned once through the UI.

---

## `GET /api/game/report` — and the one place Invariant 7 does not apply

Session 8, plan item 5, per `docs/mtg-engine-runtime-integrity.md` Layer 3. Returns
`{seed, config, protocol_version, protocol_fingerprint, hash_schema_version, state_hash,
turn, command_count, violations, journal}`; the frontend's **Export report** button saves
it as `scutemob-report-seed<N>-mull<M>.json`.

**It is a pure read.** It does not call `advance()` and does not move `journal_cursor` —
it reads `LocalGame::journal()`, not `take_new_records()` — so pulling a report can
neither change the game nor swallow event lines the live feed has not yet delivered.
Both properties are tested (`test_s8_report_is_a_pure_read`). It can therefore be
requested while a decision is outstanding, which is the moment you most want it.

**Reproducing from it**: `seed` + `config` rebuild the exact table
(`setup::build_initial_state` is deterministic in `cfg.seed`; after `mulligan_count`
redeals the effective seed is `redeal_seed(seed, human_seat, mulligan_count)`), and
replaying `journal`'s commands in order reaches `state_hash`. `protocol_version` /
`hash_schema_version` are what make that checkable rather than hopeful: a repro is valid
only against an engine build carrying the same two numbers.

**⚠️ This is the one payload in this crate that is NOT seat-redacted, and it is
deliberate.** It carries every seat's raw `Command`s and `GameEvent`s, because a redacted
repro is not a repro — a maintainer needs the `AnswerEffectChoice` that named a library
card. That is safe **only because of what M11-local is**: one human, three bots, one
process, no networking. The only "other players" whose hidden information it exposes are
simulator bots running in the same process as the person who clicked the button.

**It must be re-scoped at M10a**, when the other end of a socket is a real person —
redacted, or restricted to a single-player game, or authenticated. Recorded here, at
`view.rs::BugReportView`, and in `memory/decisions.md` so it is not rediscovered by
accident. Note this also means the Invariant 7 source gate below (which scans for
`from_game_state(` / `Viewer::Omniscient`) does **not** cover this route: the report
never builds a view model at all, omniscient or otherwise — it serializes the journal
directly. The gate is not weakened; it simply has nothing to say here.

---

## Hidden information

> **One route is exempt, by design**: `GET /api/game/report` is not seat-redacted.
> Everything in this section describes the *seat* payloads — `GET /api/game`,
> `POST /api/game`, `POST /api/game/action`, `POST /api/game/mulligan`. See the
> section immediately above for the exception, why it is safe inside M11-local's
> scope, and the M10a obligation it carries.

Architecture Invariant 7 is enforced at one chokepoint, `api.rs::seat_view`: the
state is built with `StateViewModel::from_game_state_for(.., Viewer::Seat(human))`
and every event line goes through `event_view_for(.., Viewer::Seat(human))`.
Neither omniscient entry point (`from_game_state`, `Viewer::Omniscient`) is
reachable from the production paths of this crate.

**Three channels, not one — and two of them were found by review, not by a gate**
(review MR-M11-01 / MR-M11-08). A payload can identify a hidden card without ever
naming it:

| Channel | What carries it | What holds it |
|---------|-----------------|---------------|
| **Names** — a label that says the card | the view model and `NameIndex` | `test_production_code_never_builds_an_omniscient_view` (source) + `test_seat_view_over_http_contains_no_other_hand_card_names` (body) |
| **Reconstruction keys** — data that *rebuilds* a hidden zone | `GameSummary.seed`, until MR-M11-01 removed it | `test_mr_m11_01_seat_payload_carries_no_reconstruction_key`, which asserts over the raw body so a rename or a nested copy is caught too |
| **Free-form strings** — engine text spliced in after redaction | `game_over.violations` / `.reason`, until MR-M11-08 reduced them | `test_mr_m11_08_game_over_payload_carries_no_engine_debug`, which plants a card name in both carriers |

`seed` shipped on **every** seat response for three sessions with both name-channel
gates green, because `setup::build_initial_state` is deterministic in its config alone
and `session::config_for` fixes every other input — so `(seed, players,
mulligan_count)` rebuilt every other seat's opening hand *and* library order. It is now
on `BugReportView` and nowhere else. The transferable lesson is why it survived review
and two gates: **a redaction gate checks the channel it was written for, and a new
channel is invisible to it.**

**That chokepoint is machine-enforced as of Session 7**, not asserted in prose:
`test_production_code_never_builds_an_omniscient_view` scans the production
region of every `src/*.rs` — comment- and string-blanked, so a doc comment naming
the symbol neither satisfies nor trips it — for `from_game_state(` and
`Viewer::Omniscient`. The test module is exempt on purpose; it reaches the
omniscient path as the out-of-band oracle the redaction tests check against.

It is a *source* gate rather than a behavioural one, and the reason was measured:
`seat_view` was edited to build its `NameIndex` from the omniscient view and the
whole crate stayed green, all 23 tests. `NameIndex` is only ever queried for ids
that appear in an action, a target candidate or a combat list, and every one of
those is in a public zone — so on every id that ever gets labelled, the two views
agree. The only construct that separates them is a face-down battlefield
permanent (CR 708.2a), and no seeded game reaches one. The invariant is real and
currently unfalsifiable by any payload this crate can produce, which is exactly
the kind of claim that rots.

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
