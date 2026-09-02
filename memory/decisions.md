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

## 2026-08-02 — P/T saturates at `i32` bounds (PB-DX19, `scutemob-184`, closing OOS-SIM2-5)

**Decision**: every power/toughness write in `rules/layers.rs` saturates rather than wrapping or
panicking, and every `u32 -> i32` widening that reaches a P/T write uses
`try_into().unwrap_or(i32::MAX)` rather than `as`.

**Rationale**: the CR puts **no ceiling** on power or toughness; this engine stores both as `i32`.
`devilish_valet` is `Complete` and genuinely doubles — `effects/mod.rs` substitutes its
`ModifyPowerDynamic` to a concrete `ModifyPower(current_power)` per trigger (CR 608.2h), so ~31
triggers reach `i32::MAX`, which is a reachable number of combat triggers in Commander. The two
supported build profiles failed *differently* and neither failed usefully: `[profile.fuzz]` sets
`overflow-checks = true`, so a bare `+=` **panicked** there, while a plain `--release` build
**wrapped silently to negative power** and the creature died to CR 704.5a.

**The deviation, stated plainly**: a creature pinned at `i32::MAX` power is wrong per CR. It is
accepted as far less wrong than one that wrapped negative. Making the ceiling unreachable means
widening the stored type or clamping at the effect layer under an explicit rule — filed
`OOS-DX19-3`, which also records the cost of this decision: the `overflow-checks` panic *was* the
only tripwire that would ever have reported the condition, and this decision removes it.

**The part that is easy to get wrong**: `as` casts are **not** checked arithmetic. `overflow-checks`
does not touch them, so a `u32` counter count above `i32::MAX` wrapped to a *negative* modifier in
**every** profile — inverting the counter's sign — and no fuzz run could ever have surfaced it. A
hardening pass that converts `+=` to `saturating_add` and leaves the casts alone has hardened only
the sites that were already loud. Sixteen sites were converted, not the four the seed named.

**Pinned by**: `crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs` — six probes,
one per group, each watched failing against an executed (and compiling) revert. The counter-widening
probe is the only one that fails by *assertion* rather than by *panic*, which is itself the evidence
for the paragraph above.

## 2026-08-02 — CLAUDE.md close-out bullets go lean: fixed schema, cut explanation never identifiers

**Decision**: from the `scutemob-191` (ENG-1) collect onward, the CLAUDE.md "Last Updated" delta
bullet is written to a fixed ~5-line schema instead of a full narrative:

```
- **Last Updated**: <date> — **<BATCH> SHIPPED** (`<task>`, merge `<hash>`; <queue/row ref>).
  <One sentence: what changed and why it matters.>
  Tests <N/N/N> (<delta>); <engine-lines fact>; PROTOCOL <n> / HASH <n>[; coverage if moved].
  Seeds: <every ID by name — filed / closed / deferred>.
  Full handoff: memory/workstream-state.md.
```

**The one hard rule**: condensing may cut *explanation*, never *identifiers*. Every task ID,
merge hash, seed ID (`OOS-*`), and gate-relevant count appears by name — IDs are the grep index
that routes a future session to the canonical record; prose is the record, and it lives ONCE, in
the worker handoff (`memory/workstream-state.md`, rotated to the monthly archive by `/eot`).

**Rationale**: each batch's narrative was being written four times (ESM comments, workstream-state
handoff, CLAUDE.md mega-bullet, monthly archive) with the CLAUDE.md copy grown to 25+ lines —
drift *away from* this file's own recurrence rule ("append a NEW short delta, rotate detail to the
archive"). The lean form is enforcement of the existing convention, not a new one. Nothing stops
being written except the duplication; canonical homes are unchanged.

**Scope limits (deliberate)**: prospective only — no retroactive edits to existing bullets or
archives; ESM attestations and all provisioned machinery untouched; the seed-decay question is
deferred to the next re-rank; the ID-resolution check script (every `OOS-*`/`PB-*`/`scutemob-*` in
Current State resolves somewhere in `memory/`+`docs/`) is built only if an orphaned reference is
ever actually observed. Dispatch briefs follow the same principle from ENG-2 onward: point at the
triage/plan section, state only the constraints and criteria not already recorded there.

**Evaluation gate**: the next `/start` after a lean collect must reconstruct the batch from the
bullet plus one pointer-follow. If orientation stumbles, write the missing detail back into that
one bullet and resume the long form — rollback is one commit.

---

## 2026-09-02 — A `Complete` marker and CR 118.12's optional cost (PB-DX45, `scutemob-217`)

**Decision.** After PB-DX45, an `Effect::MayPayThenEffect` (or an `Effect::LookAtTopThenPlace`
`place_cost`) **does not bar a def from `Complete`**. The engine no longer takes CR 118.12's
choice: both of the engine's `try_pay_optional_cost` call sites ask the payer an
`EffectChoiceQuestion::PayOptionalCost` on PB-DP9's CR 608.2d suspend-and-replay channel, and a
DECLINE is producible from every client — the browser, the TUI, a bot, and a golden script.

**Why the ruling was needed.** `OOS-DX27-5` recorded that the corpus did not treat this
consistently: `disciple_of_freyalise` shipped `Complete` on the same `MayPayThenEffect` +
`Cost::Sacrifice` shape that left two other defs at `partial`. The row said *"One of the two
readings is wrong and nothing decides which"*. Both readings are now moot — the deviation is gone
— so the markers are re-adjudicated on one rule rather than left to be compared.

**The three defs, re-adjudicated:**

| def | before | after | why |
|---|---|---|---|
| `disciple_of_freyalise` | `Complete` | **`Complete`** (unchanged) | its printed clause is now fully expressed; it was `Complete` on a premise that has since become true |
| `vampire_gourmand` | `partial` | **`Complete`** (flip) | its marker named exactly one blocker — pay-when-able — and that blocker is gone |
| `ruthless_technomancer` | `partial` | **`partial`** (unchanged) | **`OOS-DX27-5`'s framing is wrong here.** The row says PB-DX27 *"left `ruthless_technomancer` and `vampire_gourmand` at `partial` on the same shape"*. Read at HEAD, this def's marker names its **activated** ability — *"no `Cost` variant for a player-chosen variable-X sacrifice count, and `TargetFilter.max_power` is a static `i32` with no `max_power_amount` sibling"* — a different, still-live gap that PB-DX45 does not touch |

`ezuri_stalker_of_spheres` and `mana_vault`, the other two non-`Complete` `MayPayThenEffect`
carriers, are likewise blocked on unrelated gaps and are unchanged. **One flip, not two**, and it
was predicted and named before regeneration (`memory/primitives/pb-DX45-execution-notes.md` §1.5).

**The residual the ruling explicitly does NOT treat as a blocker.** WHICH permanent a
`Cost::Sacrifice` optional cost eats is still the engine's lowest-`ObjectId` pick
(`OOS-DX45-1`). That is held not to bar `Complete`, and the reason is consistency rather than
convenience: the identical auto-pick governs `Effect::SacrificePermanents`, whose ten-def
Fleshbag / Grave Pact family has shipped `Complete` for the life of the corpus (re-measured by
PB-DX15a, which found that family *"makes no per-player choice at all"*). Ruling one class of
which-permanent auto-pick fatal while the other ships would recreate exactly the inconsistency
`OOS-DX27-5` was filed about. If that pick is ever ruled a blocker, it must be ruled so for both
sites in one commit.

**What the rule is, stated so the next author can apply it without re-reading this.** A printed
"you may pay X. If you do, Y" bars `Complete` iff the engine **cannot express the decline**. It
can, now, wherever the clause is authored as `MayPayThenEffect` or a `place_cost`. It still
cannot where the clause is authored as something else — `OOS-DX45-3` names three deck-legal
`Complete` defs (`teneb_the_harvester`, `crypt_ghast`, `syndic_of_tithes`) whose printed cost is
not merely auto-taken but **never charged at all**, and those markers are wrong today. They are
filed rather than fixed here, per this batch's scope line: PB-DX45 repairs every caller of
`effects::try_pay_optional_cost`, not every printed "you may pay".
