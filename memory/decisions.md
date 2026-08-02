# Design Decisions — Last verified: post-M9.5 strategic review (2026-03-07)

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
| 2026-07-16 | CR 613.8 dependency detection is a **static Layer-4-only approximation** (SR-30) | `depends_on` (layers.rs) only encodes Layer-4 type-changing edges (`Set*` depends on `Add*`/`Set*`); no P/T, ability, color, or control-layer dependencies. There is **no CR 613.8c re-evaluation** (order is computed once, not recomputed after each effect applies), and CDA-vs-CDA effects are ordered by timestamp only (CR 613.8a(c) already bars CDA↔non-CDA edges). Consequence: the 613.8b dependency-*loop* fallback is unreachable — no cycle is constructible — so it is `debug_assert`'d unreachable and guarded by a no-symmetric-edge unit test rather than exercised. Revisit if a non-Layer-4 or symmetric dependency arm is ever added. |
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
| 2026-03-07 | Decouple M11 (UI) from M10 (networking) | UI can drive the engine locally with simulator bots (1 human + 3 bots). No need to wait for WebSocket server. Humans can play months earlier. See `docs/mtg-engine-strategic-review.md` |
| 2026-03-07 | Split M10 into M10a (basic multiplayer) and M10b (resilience/social) | M10 scope was too large for one milestone. M10a gets multiplayer working; M10b adds rewind/pause/reconnection. M10b can slip without blocking alpha. |
| 2026-03-07 | Downscope M12 — agent-based card scaling replaces pipeline crate | 193 cards already authored via agents. `card-definition-author` agent + W5 worklist is the active scaling strategy. Scripted converter's 70-80% coverage claim is optimistic given DSL gaps. Revisit post-alpha if needed. |
| 2026-03-07 | Prioritize Transform/Morph before M10 | Transform (CR 712) blocks 4 ability batches; Morph (CR 702.36) blocks 5. Common Commander mechanics that should not be deferred indefinitely. |
| 2026-03-07 | Evaluate web-first UI vs Tauri — decision pending | Replay viewer already has working axum + Svelte 5 stack. Tauri can't build on headless Debian. Web-first avoids maintaining two UI frameworks. Decision needed before M11 starts. |
| 2026-07-17 | SR-33: "{T}: Add {G} or {U}" is authored as **one activated ability per colour**, not `Effect::Choose` (the `tainted_field` pattern) | A mana ability **resolves immediately and never uses the stack** (CR 605.3b), so there is no window in which a resolution-time choice could ever be supplied — the mode choice is *necessarily* made at activation. The engine already has exactly that channel: `enrich_spec_from_def` lowers each `Activated{Tap, AddMana}` into `characteristics.mana_abilities` (excluding them from `activated_abilities` so `ability_index` does not shift), and `Command::TapForMana{ability_index}` selects among them. So one-ability-per-colour is not a workaround for a missing primitive; it is the shape the engine's mana model is built around. The rejected alternative — a general `MakeChoice` Command + `try_as_tap_mana_ability` support for `Choose` — is strictly larger (pending-choice state in `GameState`, new Command, `HASH_SCHEMA_VERSION` bump, and a `PROTOCOL_VERSION` bump because `Effect` is inside the SR-8 wire closure) **and would not fix these 88 cards anyway**, since for a stackless mana ability it degenerates back to index selection. Known limitation accepted: an effect that *copies* an activated ability (Rings of Brighthearth) copies one colour-arm and cannot re-choose on the copy; ability-counting sees N abilities where the card prints one. Recorded, not fixed — the same deviation the deviation-scan allowlist already accepts for `tainted_field`. |
| 2026-07-17 | SR-33: `Effect::Choose` and `Effect::MayPayOrElse` are **gated out of `Complete`** rather than implemented | Both are M7-era stubs: `Choose` unconditionally executes `choices.first()` and `MayPayOrElse` unconditionally declines (`effects/mod.rs`). Implementing real interactive choice is M9+ work with a wire-format blast radius, and is not what SR-33 is scoped to. Gating is the cheap half of the SR ethos ("the sharpest finding is a hole in a checker"): `tests/core/effect_choose_gate.rs` fails any `Complete` def whose serialized effect tree contains either variant, so the stub can never again silently ship as a finished card. Cost is exactly 3 demotions (Cankerbloom, Path to Exile, Rhystic Study) — every other user was already marked. **`MayPayThenEffect` is deliberately NOT gated**: pay-when-able is a documented deterministic-but-legal game choice under CR 118.12, unlike the other two it does honour its `payer`, and gating it would demote 7 `Complete` defs on a debatable premise. Filed as a follow-up instead. Delete this gate when interactive choice lands. |
| 2026-07-18 | PB-EF12 (EF-W-PB2-3): a mana ability's colour choice rides the **activation Command** — `Command::TapForMana { chosen_color: Option<ManaColor> }` — not a resolution-time prompt | Direct extension of the SR-33 precedent (row above): a mana ability resolves immediately and **never uses the stack** (CR 605.3b), so any choice it makes is *necessarily* made at activation (CR 605.3b/605.5, special action). For a fixed "{G} or {U}" the choice channel is `ability_index` (SR-33, one-ability-per-colour). For "{T}: Add one mana of **any** colour" (Command Tower, City of Brass, Chromatic Lantern, Treasure tokens, and *granted* abilities like Cryptolith Rite / Elven Chorus) enumerating one grant-ability per colour is untenable — a grant would push five abilities onto every creature you control, and the corpus already models these as a single `ManaAbility { any_color: true }`. So the colour is carried as a payload on the same activation Command: `chosen_color`, validated in `handle_tap_for_mana` against the ability's offered set (the five real colours WUBRG — `ManaColor::Colorless` is a *type*, not a colour, CR 106.1a/106.1b, so it is rejected), with **no silent `Colorless` default** — a missing choice on an `any_color` ability is a hard `GameStateError`, exactly the SR-37 stub being eliminated. It rides the Command stream, so replay/determinism is preserved with no new `GameState` field (the colour lands in `ManaPool`, which is already per-colour) — hence **PROTOCOL bumps, HASH does not** (`Command` is inside the SR-8 wire closure but not the GameState hash closure). No interactive prompt mechanism is introduced. The SR-33 rejection of a general `MakeChoice` Command still stands for the *fixed* case; this is the narrower "any colour needs a colour payload, and the Command is where it goes" channel. Simulator `LegalActionProvider` emits a concrete legal colour (deterministic WUBRG order) so a bot never suggests what the engine rejects (SR-38 precedent). |

## 2026-07-18 — DOC-8: fate of the §3 untouchable memory corpus (scutemob-124)

**Decision** (user, 2026-07-18): option (c) + (b) scoped to abilities only.
- `memory/abilities/` (329 files, 5.1MB): W1 closed 2026-03; ability pipeline idle since;
  nothing globs it. **Distillation pass authorized** — extract reusable patterns into
  gotchas/conventions, then archive. Filed as its own follow-up task, NOT a cleanup.
- `memory/primitives/` (198+ files, 4.4MB): **keep untouchable** — demonstrably live
  (OS retriage cited pb-plan-AC7/AC8/pb-retriage-CC the week of this decision).
- `memory/card-authoring/*review*.md`: **keep untouchable**; protection glob widened
  from `review-*.md` to `*review*.md` (audit F5 gap — 9+ review files fell outside the
  prefix glob). `card-fix-applicator`'s own read glob is unchanged.
**Why**: the corpus rules exist for the agents that read them; retention should track
actual readership, not blanket-quarantine 86% of memory/.

## 2026-07-26 — M11-local dispatched in parallel with RS queue; UI is WEB-FIRST (action item 6 resolved)

**Decision** (user, 2026-07-26): begin the playability track now, in parallel with the
paused-then-resumed RS correctness queue (PB-RS4 in flight).
- **Track**: **M11-local first** (web UI + simulator bots + local play, no networking),
  per the strategic review's revised critical path — M10a follows later, in parallel.
- **UI stack**: **web-first** — extend the axum + Svelte 5 stack the replay viewer
  already uses (shared components, single UI framework, becomes the M10a server UI).
  Tauri v2 remains a later packaging wrapper option, not a parallel framework.
**Why**: shortest path to a human playing a game; 1,139 Complete cards + validate_deck
already support legal curated decks; simulator (GameDriver/bots/LegalActionProvider)
exists and needs only a human-input bridge. Note: the review's "headless Debian can't
build Tauri" premise is stale (dev is now skylarch, full desktop) — the web-first call
was made on iteration-speed and single-stack grounds, not the environment constraint.

## 2026-08-01 — M11-local design decisions (recorded at close, `scutemob-173` / S8 item 7)

Four decisions taken during M11-local that the milestone rests on. Each was made inside
a session and is recorded here so the *reason* survives the session log.

### 1. The human-input bridge is a **steppable driver**, not a channel-backed `Bot`

**Decision** (M11-local planning, `scutemob-147`; shipped S1 `scutemob-147`,
`crates/simulator/src/local_game.rs`): a human occupies a seat by the caller *stepping*
the game — `advance()` runs bot seats and returns `AwaitingHuman(PendingDecision)`;
`submit(seq, choice)` answers — rather than by implementing `Bot` for a channel that
blocks waiting on a human.

**Why**: the obvious design is a `HumanBot: Bot` whose `choose_action` blocks on a
channel, because `GameDriver` already takes `Box<dyn Bot>` per seat. It does not work,
for a reason specific to this engine rather than to blocking:

* **`Bot::choose_action` returns a `Command`, and a rejected `Command` is silently
  swallowed.** `driver.rs`'s loop answers a rejection by issuing `PassPriority` on the
  seat's behalf. For a bot that is a reasonable safety valve; for a human it means an
  illegal play is answered by *passing your turn* with no error. `submit` returns
  `Result` precisely so that cannot happen (S8 item 4).
* **Every sub-decision is already a field of the returned `Command`.** Targets, X, modes,
  attacker/blocker sets — `Bot`'s extra `choose_targets` / `choose_attackers` /
  `choose_blockers` callbacks exist for bot convenience, not because the engine asks
  separately. A human client needs to supply them *with* the action, which is what
  `ActionParams` does.
* A blocking channel also forces an async or threaded host on `crates/simulator`, which
  Architecture Invariant 1's spirit (and the fuzzer's throughput) argues against.

**Consequence, and it is the milestone's structural win**: `GameDriver::run_game` is
re-expressed on top of `LocalGame` with `human_seats` empty, so there is **one** loop
rather than two that can drift. Verified byte-identical across 500 fuzz games at close
(`memory/m11/s8-fuzz-parity.md`).

### 2. **No WebSocket and no SSE** in M11-local

**Decision** (S5, `scutemob-167`): the play server is plain request/response.
`POST /api/game/action` calls `submit` then `advance` **inside the same request**, so
the bots play their whole turn synchronously and the response already carries the state
the human must next act on.

**Why**: a push channel exists to tell a client something it is not already waiting for.
On this surface there is no such moment — the server never knows anything the client has
not been told in a response it is holding open. Adding a socket would buy nothing and
cost a second state-delivery path to keep consistent with the first. Push infrastructure
is M10a's problem, where there *are* other players acting between your requests.

**Consequence**: no reconnection logic, no message ordering, no heartbeat, and the whole
client is `fetch`. Revisit at M10a, not before.

### 3. The view model is a **shared crate**, `crates/view-model`

**Decision** (S4, `scutemob-165`): `tools/replay-viewer/src/view_model.rs` was moved to
its own workspace crate (`mtg-view-model`) rather than copied into the play server.

**Why**: two consumers needed the same `GameState` → view-model conversion, and the
conversion carries **two exhaustive matches over engine enums** (`StackObjectKind` via
`stack_kind_info`, `KeywordAbility` via `format_keyword`). A copy forks silently the
first time either enum gains a variant — the copy still compiles, and only one of the two
renderings is right. A shared crate makes that a compile error in one place.

The move also let redaction be added as a *second entry point*
(`from_game_state_for(.., Viewer)`) rather than as a change to the omniscient one the
stepper legitimately wants, which is what makes Architecture Invariant 7 a chokepoint
rather than a discipline.

**The Svelte components are shared the same way but by Vite alias**, not by copy — and
the first breakage proved the point: `ZoneHand.svelte` keyed its `#each` on
`card.object_id`, unique for the omniscient viewer and *not* for a redacted payload
(every unreadable card gets `object_id: 0`), so Svelte's `each_key_duplicate` threw the
mount down. Fixed once, in the shared component. **Generalisation: every id-uniqueness
assumption in those components is now a claim about the redacted view model too.**

### 4. Mulligans are a **pregame rebuild**, not `Command::TakeMulligan`

**Decision** (S2 `scutemob-161`, kept at S5): `POST /api/game/mulligan` rebuilds the
whole table from a perturbed seed (`setup::redeal`) instead of issuing the engine's
`Command::TakeMulligan`.

**Why**: M11-local offers mulligans *before* `start_game` is ever called, so no command
has been issued and a rebuild invalidates no history. That is simpler than routing an
in-game command through a game that has not started.

**Note the original rationale was falsified and this is not it.** The M11 plan's R2 said
a real mulligan needed a caller-supplied permutation and therefore a new `Command` — a
wire change. **False**: the engine already had a deterministic seeded PRNG
(`StdRng::seed_from_u64(state.timestamp_counter)`), and PB-DP2 (`scutemob-150`) made
`handle_take_mulligan` shuffle for real with PROTOCOL 27 / HASH 63 unmoved. The rebuild
therefore survives on *simplicity*, not on necessity. **Reusable lesson: check for an
existing in-engine deterministic seed source before concluding a permutation needs a new
Command.**

**Two limitations, both real and both documented at `setup::redeal`**: the rebuild is not
invisible to the other seats (CR 903.6 puts every commander in the *public* command zone
and a rebuild re-rolls them), and it cannot represent a partially-decided table (CR
103.5c gives each player their own mulligan count). CR 103.5's bottoming half is not
expressible at all — `handle_keep_hand` checks `cards_to_bottom` against
`PlayerState::mulligan_count`, which a rebuild leaves at 0 — so a non-empty
`cards_to_bottom` is **refused with 400** rather than accepted and discarded. A per-seat
mulligan model belongs with M10a's real pregame flow.

### 5. `GET /api/game/report` is deliberately **not** seat-redacted

**Decision** (S8, `scutemob-173`): the bug-report export carries every seat's raw
`Command`s and `GameEvent`s, while every other payload in `tools/play-server` goes
through the Architecture Invariant 7 chokepoint.

**Why**: a redacted repro is not a repro. A maintainer replaying a defect needs the
`AnswerEffectChoice` that named a library card, and redacting it makes the artefact
unusable for the one purpose it has. This is safe **only because of what M11-local is**:
one human, three bots, one process, no networking — the only "other players" are
simulator bots in the same process as the person clicking the button.

**This must be re-scoped at M10a**, when the other end of a socket is a real person:
redacted, or single-player-only, or authenticated. Recorded here, at
`view.rs::BugReportView`, and in the crate README so it is not rediscovered by accident.
