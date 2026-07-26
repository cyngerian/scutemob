# Primitive Batch Plan: PB-DP5 — the `WouldDraw` multi-replacement prompt is unanswerable

**Generated**: 2026-07-26
**Task**: `scutemob-153` · branch `feat/pb-dp5-woulddraw-multi-replacement-prompt-is-unanswerable-th`
**Finding**: DP-5 (`docs/audits/decision-point-audit.md` §4 L383, §5 L432, §8 L581)
**Class**: CORRECTNESS, Tier 0, class **D**
**CR**: 616.1 / 616.1a / 616.1e / 616.1f · 614.5 · 614.10 · 614.11 / 614.11a · 121.1 / 121.2 · 702.52a
**Primitive**: a new `pub(crate)` `GameState` field `pending_draws: Vector<PendingDraw>` + a
resume path (`resolve_pending_draw`) reachable from the existing `Command::OrderReplacements`
**Wire**: PROTOCOL **27 unchanged** · HASH **63 → 64** (expected; empirically forced, see §6)
**Card yield**: **0** (see §1.4 — DP-5 is not reachable from a legal deck today; this is a
correction to the audit's reachability claim, not a reason to drop the PB)

> **Read before implementing**: `memory/primitive-wip.md` (coordinator brief, hard constraints),
> `memory/conventions.md`, `docs/engine-invariants.md` (SR-3 / SR-4 / SR-8 / SR-9a / SR-17 /
> SR-19 / SR-25), `memory/gotchas-rules.md`, `memory/gotchas-infra.md`.

---

## 0. The bug in one paragraph, verified on this branch

`replacement::check_would_draw_replacement` (`rules/replacement.rs:595`) returns
`DrawAction::NeedsChoice(GameEvent::ReplacementChoiceRequired { .. })` when 2+ `WouldDraw`
replacements apply to one draw (CR 616.1e). **Three** call sites emit that event and return
early **recording no state whatsoever** — there is no draw-pending field on `GameState`.
`handle_order_replacements` (`rules/replacement.rs:139-193`) hard-requires a matching
`pending_zone_changes` entry at `:163-172` and errors without one, so the
`Command::OrderReplacements` the player was just asked for is **rejected**
(`GameStateError::InvalidCommand("player PlayerId(N) is not the affected player of any pending
replacement choice")`) and the draw can never complete. Worse than the audit records: the two
`effects/mod.rs` **loop** sites keep iterating after the deferral, so `Effect::DrawCards
{ count: 3 }` emits **three** unanswerable prompts and draws **zero** cards.

---

## 1. Verified site inventory (line numbers as they exist on this branch)

### 1.1 The producer

| what | file:line | note |
|---|---|---|
| `enum DrawAction` | `crates/engine/src/rules/replacement.rs:567-580` | `Proceed` / `Skip(GameEvent)` / `NeedsChoice(GameEvent)` / `DredgeAvailable(GameEvent)` |
| `pub fn check_would_draw_replacement` | `crates/engine/src/rules/replacement.rs:595` | dredge scan `:600-637`; trigger build `:638-641`; `determine_action` `:642`; match `:643-674` |
| the `NeedsChoice` construction | `crates/engine/src/rules/replacement.rs:662-673` | builds `GameEvent::ReplacementChoiceRequired { player, event_description, choices }` and returns it — **no state written** |
| the `AutoApply` arm | `crates/engine/src/rules/replacement.rs:645-661` | honours **only** `ReplacementModification::SkipDraw`; every other modification falls to the `else` at `:657-660` → `Proceed`, and **emits nothing** |

### 1.2 The three emit sites — the pre-survey's "I believe there are three" is **CONFIRMED**

| # | site | fn | signature | arm |
|---|---|---|---|---|
| 1 | `crates/engine/src/rules/turn_actions.rs` | `pub fn draw_card` (`:1171`) | `Result<Vec<GameEvent>, GameStateError>` | `DrawAction::NeedsChoice` at **`:1189-1192`** (`DredgeAvailable` at `:1193-1196`) |
| 2 | `crates/engine/src/effects/mod.rs` | `fn draw_one_card` (`:8547`) | **plain `Vec<GameEvent>`, no `Result`** | `DrawAction::NeedsChoice` at **`:8556-8559`** (match block `:8553-8564`) |
| 3 | `crates/engine/src/rules/replacement.rs` | `fn draw_card_skipping_dredge` (`:2631`) | `Result<Vec<GameEvent>, GameStateError>` | `ReplacementResult::NeedsChoice` at **`:2666-2676`** |

**Site 3 is a genuine third instance and it is reachable.** Its only caller is
`handle_choose_dredge(state, player, None)` at `rules/replacement.rs:2540`, dispatched from
`rules/engine.rs:302-306` on `Command::ChooseDredge`. That command is reachable from the replay
harness (`crates/engine/src/testing/replay_harness.rs:900-903`), from an approved golden script
(`test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json`) and from the test
suite (`crates/engine/tests/mechanics_a_d/dredge.rs:312-317`). **Include it.** The audit's §4 L383
and §5 DP-5 rows name only sites 1 and 2 and must be corrected.

### 1.3 Callers of the three (the blast radius of any signature change)

`turn_actions::draw_card` — 4 callers:
`turn_actions.rs:754` (monarch end-step draw, CR 724.2) · `turn_actions.rs:1167` (`draw_for_turn`,
CR 504.1) · `rules/resolution.rs:4641` (Ravenous, CR 702.156a) · `rules/resolution.rs:7847`.

`effects::draw_one_card` — 4 callers, **all of them `for _ in 0..n` loops**:
`effects/mod.rs:659` (`Effect::DrawCards`) · `:716` (`WheelDraw::GreatestDiscarded`) ·
`:751` (`WheelDraw::ThatMany | Fixed`) · `:4761` (Connive, CR 701.50e).

`replacement::draw_card_skipping_dredge` — 1 caller (`handle_choose_dredge`, `:2540`).

### 1.4 The answer path, and the state it consults

| what | file:line |
|---|---|
| `pub fn handle_order_replacements` | `rules/replacement.rs:139-193` |
| (a) empty-`ids` reject | `:144-148` |
| (b) unknown-id reject | `:151-158` |
| (c) **the rejection DP-5 is about** — `pending_zone_changes.iter().position(|p| p.affected_player == player)` + `ok_or_else` | **`:163-172`** |
| (d) rebuild `ReplacementTrigger::WouldChangeZone` from the pending entry + require every id in `find_applicable` | `:176-189` |
| (e) `resolve_pending_zone_change(state, ids[0], pending_idx)` | `:191-192` |
| dispatch | `rules/engine.rs:223-227` — `validate_player_active` (which only means *not eliminated*, `engine.rs:2211-2217`; it does **not** require the active player) |
| the mirror to copy | `pub fn resolve_pending_zone_change`, `rules/replacement.rs:907-1055` (CR 616.1f loop + re-pend at `:1034-1052`) |
| `WouldDraw` trigger matching | `rules/replacement.rs:281-288` — `event_player_matches_filter(evt_filter, eff_filter)` |

### 1.5 State plumbing to mirror

| what | file:line |
|---|---|
| `GameState.pending_zone_changes` decl | `crates/engine/src/state/mod.rs:136-138` |
| read accessor | `state/mod.rs:435-438` |
| `_mut` escape hatch | `state/mod.rs:716-719` |
| builder init | `state/builder.rs:311-320` (**the only `GameState` struct literal in the workspace**) |
| `public_state_hash` feed | `state/hash.rs:7703` (section 5, "Vectors of game-wide state") |
| `impl HashInto for PendingZoneChange` | `state/hash.rs:2932-2943` |
| loop-detection fingerprint feed | `rules/loop_detection.rs:141-144` |
| `struct PendingZoneChange` decl | `crates/card-types/src/state/replacement_effect.rs:360-379` |

### 1.6 Gates that will fire

| gate | file:line | current pin |
|---|---|---|
| `HASH_SCHEMA_VERSION` | `crates/engine/src/state/hash.rs:578` | **63** |
| `HASH_SCHEMA_HISTORY` last row | `state/hash.rs:870-878` | v63 (PB-OS11) |
| sentinel test | `crates/engine/tests/core/hash_schema.rs:1192-1198` | `63` |
| **42 further `HASH_SCHEMA_VERSION, 63u8` sentinels** across `tests/primitives/`, `tests/casting/`, `tests/rules/`, `tests/mechanics_e_l/` | see §8.4 | all must go to `64u8` |
| `PROTOCOL_VERSION` | `crates/engine/src/rules/protocol.rs:260` | **27 — must not move** |
| SR-25 `bare_lookup_ratchet` | `tests/core/bare_lookup_ratchet.rs` `SWEPT_FILES` | `src/effects/mod.rs` **111** (L94), `src/rules/replacement.rs` **24** (L126), `src/rules/turn_actions.rs` (L127) — all three are touched files; the ratchet fires on a move **up or down** |
| SR-19 `HashInto` field-coverage | `tests/core/hash_schema.rs:1200-1265` | `NOT_HASHED` is empty — every field of `PendingDraw` must be read as `self.<field>` in its `HashInto` impl |
| SR-9a | `crates/engine/tests/primitives/main.rs` | a new test file needs a `mod` line or its coverage silently vanishes |

### 1.7 What is **not** in the blast radius

`Command::OrderReplacements` appears in exactly 7 files, all inside `crates/` and all engine or
test (`command.rs`, `engine.rs`, `replacement.rs`, `events.rs`, `state/mod.rs`,
`card-types/.../replacement_effect.rs`, `tests/rules/replacement_effects.rs`). **`tools/tui`,
`tools/replay-viewer` and `crates/simulator` contain zero occurrences.** No new `Command` /
`GameEvent` / `Effect` / `StackObjectKind` / `KeywordAbility` variant is added, so the exhaustive
matches in `view_model.rs` and `stack_view.rs` are untouched. `cargo build --workspace` after
every phase anyway (hard constraint 6).

---

## 2. The pending-state shape, and the argument for it

### 2.1 The type

New struct, placed next to `PendingZoneChange` in
**`crates/card-types/src/state/replacement_effect.rs`** (that is where the sibling lives and
where the SR-17 declaration scanner already reaches):

```rust
/// Tracks a card draw that is waiting for the drawing player to choose which
/// `WouldDraw` replacement effect to apply first (CR 616.1 / 614.11).
///
/// When 2+ `WouldDraw` replacements apply to one draw, the draw does not happen
/// and this entry records everything the resume needs. Resolved by
/// `Command::OrderReplacements`; see `resolve_pending_draw`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingDraw {
    /// CR 616.1: the affected player — the player who would draw. They are the
    /// chooser. Unlike a zone change there is no controller/owner split to get
    /// wrong: a draw has exactly one affected player.
    pub player: PlayerId,
    /// CR 614.5: replacement effects already applied to THIS draw event. Threaded
    /// into `find_applicable` on resume so an effect cannot apply twice.
    pub already_applied: Vec<ReplacementId>,
    /// CR 614.11a / 121.2: how many further draws remain in the sequence this draw
    /// belongs to ("draw three cards" = three individual draws). Performed after
    /// this draw resolves, before the sequence is considered finished.
    pub remaining: u32,
    /// Which draw path raised this, so the resume writes the same bookkeeping.
    /// `true` for `turn_actions::draw_card` and `replacement::draw_card_skipping_dredge`
    /// (both set `PlayerState::has_drawn_for_turn`), `false` for `effects::draw_one_card`
    /// (which does not). See §2.4 — the flag preserves an existing divergence rather
    /// than silently unifying it.
    pub sets_has_drawn_for_turn: bool,
}
```

`GameState` gains:

```rust
/// Card draws waiting for the drawing player to choose among applicable
/// `WouldDraw` replacements (CR 616.1 / 614.11). Resolved by `OrderReplacements`.
#[serde(default)]
pub(crate) pending_draws: Vector<PendingDraw>,
```

plus `pending_draws()` (`state/mod.rs`, next to `:435`) and `pending_draws_mut()` (next to
`:716`). **The `_mut` escape hatch is genuinely needed**: `effects/mod.rs` must decrement /
write `remaining` on the entry it just pushed, and tests must construct pending states directly
(exactly why `pending_zone_changes_mut` exists — `tests/rules/replacement_effects.rs:783`,
`:2830`, `:4636`, `:4764` all use it). Gate it with the same doc-comment form as its siblings.

### 2.2 Why a `Vector`, not an `Option`

1. **Two players can have a deferred draw at the same time, inside one effect resolution.**
   `Effect::DrawCards` resolves `resolve_player_target_list` and loops `for p in players`
   (`effects/mod.rs:656-662`); `PlayerTarget::EachPlayer` / `EachOpponent` therefore reaches
   `draw_one_card` for every seat in one call. The two `WheelDraw` branches (`:714-719`,
   `:722-754`) do the same. A `WouldDraw` replacement with
   `PlayerFilter::Any` or `OpponentsOf(..)` applies to more than one of them. An `Option`
   would drop all but one.
2. **One player can have two independent deferred draws.** Nothing forces a player to answer
   before another effect resolves — `OrderReplacements` is an out-of-band command, and a second
   spell can resolve (or the same effect can defer a later draw in the same sequence) before it
   arrives.
3. **Symmetry.** Every sibling pending queue on `GameState` is a `Vector`
   (`pending_zone_changes`, `pending_commander_zone_choices`, `pending_triggers`, the three
   `pending_*_payments`). `imbl::Vector` also keeps Architecture Invariant #2 (persistent
   structure, cheap clone).

**Selection rule on resume**: the **first** entry whose `player == player` (FIFO), matching
`handle_order_replacements`' existing `.position(|p| p.affected_player == player)` at `:166`.

### 2.3 Why `remaining` is on the entry, and not "just push N entries"

CR **614.11a**: *"If an effect replaces a draw within a sequence of card draws, all actions
required by the replacement are completed, if possible, before resuming the sequence."* The
sequence must **stop** at the replaced draw, not race past it. So the `for _ in 0..n` loops
**break** on a deferral and stash the remainder on the entry, rather than pushing N entries and
emitting N prompts up front. This is also the only shape that does not regress: today the loop
continues and emits N unanswerable prompts for N lost draws.

Note the resume still produces **one prompt per draw** when several draws in the same sequence
each hit a multi-replacement choice — that is correct, not a smell: CR **121.2** makes each draw
a separate event and CR 616.1 gives each event its own choice.

### 2.4 The `has_drawn_for_turn` question — the pre-survey's third option

The pre-survey asks whether the `draw_card` / `draw_one_card` divergence is "a real distinction
or a latent bug". **It is neither: `PlayerState::has_drawn_for_turn` is write-only dead state.**
Workspace-wide it has exactly six occurrences: three writes
(`turn_actions.rs:1218`, `turn_actions.rs:1432` reset, `replacement.rs:2700`), the declaration
(`card-types/src/state/player.rs:334`), the builder init (`builder.rs:249`), one hash feed
(`hash.rs:2023`) and one test asserting its default (`tests/core/state_foundation.rs:20`).
**No engine logic reads it.** And the flag is already incoherent: `draw_card` is called by the
monarch end-step draw and by Ravenous, neither of which is "the draw for the turn".

Consequences, all of which the plan takes:

- It is **not** safe to unify the three completion bodies without preserving the write, because
  the field **is** fed to `public_state_hash` — unifying would change real game hashes and move
  the SR-9b per-step fingerprints on golden scripts, for no behavioural gain. Hence
  `PendingDraw.sets_has_drawn_for_turn`.
- A second existing inconsistency is exposed and left alone: `draw_card_skipping_dredge`
  (`:2700`) sets the flag, so an **effect**-driven draw routed through a dredge *decline* sets
  it while the direct effect path does not. Seeded (**OOS-DP5-4**), not fixed here.
- The pre-survey's "`draw_one_card` does a subset" is imprecise: `draw_one_card` **does** run the
  CR 702.94a miracle check (`effects/mod.rs:8592-8596`). The difference is exactly one write.

### 2.5 The other pre-survey framing that needs correcting

`draw_one_card` currently returns a plain `Vec<GameEvent>` with no way to tell the caller "I
deferred". The plan does **not** work around that with a state-inspection hack; it changes the
private helper's return type (§3.1). That is an engine-internal signature and touches no wire.

---

## 3. Engine changes

### Phase 1 — the type and the state (no behaviour change yet)

1. **`crates/card-types/src/state/replacement_effect.rs`** — add `PendingDraw` (§2.1) after
   `PendingZoneChange` (`:379`).
2. **`crates/engine/src/state/mod.rs`** — add `pending_draws` after `pending_zone_changes`
   (`:138`); add `pending_draws()` after `:438`; add `pending_draws_mut()` after `:719`. Import
   `PendingDraw` alongside `PendingZoneChange`.
3. **`crates/engine/src/state/builder.rs:320`** — `pending_draws: Vector::new(),`.
4. **`crates/engine/src/state/hash.rs`** — `impl HashInto for PendingDraw` next to
   `PendingZoneChange`'s (`:2932`), reading **every** field via `self.<field>` (SR-19); feed
   `self.pending_draws.hash_into(&mut hasher);` immediately after `:7703` in
   `public_state_hash`.
5. **`crates/engine/src/rules/loop_detection.rs`** — mirror block 6 (`:141-144`):
   ```rust
   // 7. Pending draws (CR 616.1 / 614.11) — a deferred draw is live state; two
   //    states differing only in it are not the same position.
   for pd in &state.pending_draws { pd.hash_into(&mut hasher); }
   ```
   Rationale: without it, "before the prompt" and "after the prompt" fingerprint identically and
   CR 104.4b loop detection could call a legitimate progression a loop.
6. Run `cargo test --all`. **Expect** `hash_schema::declaration_fingerprint_is_pinned` and
   `stream_fingerprint_is_pinned` to fail. Do §6 now, in this phase, so the rest of the work
   runs against a green gate.

### Phase 2 — record the pending state at all three emit sites

Refactor the shared draw body into one helper so the three sites, and the resume, cannot drift.
Put it in `rules/replacement.rs` (which already owns `check_would_draw_replacement` and
`draw_card_skipping_dredge`):

```rust
/// What happened to one attempted draw.
pub(crate) enum DrawStepOutcome {
    /// A card moved to hand (or the library was empty and the player lost).
    Completed,
    /// A replacement (CR 614.10 `SkipDraw`) consumed the draw. No card moved.
    Replaced,
    /// CR 616.1: 2+ replacements applied; a `PendingDraw` was pushed and a
    /// `ReplacementChoiceRequired` emitted. The caller MUST stop the sequence.
    Deferred,
    /// CR 702.52a: a `DredgeChoiceRequired` was emitted. Behaviour unchanged from
    /// pre-PB-DP5 — see §5.3; the caller does NOT stop.
    DredgeOffered,
    /// CR 104.3b: the library was empty; `PlayerLost` emitted.
    LostToEmptyLibrary,
}

/// CR 121.1 / 121.2 / 614.11 / 616.1: perform ONE card draw for `player`.
pub(crate) fn perform_one_draw(
    state: &mut GameState,
    player: PlayerId,
    offer_dredge: bool,
    sets_has_drawn_for_turn: bool,
    already_applied: HashSet<ReplacementId>,
    remaining_after: u32,
) -> (Vec<GameEvent>, DrawStepOutcome)
```

- `check_would_draw_replacement` gains an `already_applied: &HashSet<ReplacementId>` parameter
  (it currently hardcodes `&std::collections::HashSet::new()` at `:641`) **and** an
  `offer_dredge: bool` (so the resume does not re-offer dredge — see §3.3). Both existing
  callers pass `&HashSet::new()` / `true`; behaviour is unchanged for them.
- On `NeedsChoice`, `perform_one_draw` pushes
  `PendingDraw { player, already_applied: already_applied.into_iter().collect(), remaining: remaining_after, sets_has_drawn_for_turn }`
  **before** returning the event, and reports `Deferred`.
  - Determinism (SR-9b): sort the collected `already_applied` by `ReplacementId` before storing,
    so an `imbl` state hash cannot depend on `HashSet` iteration order. **This is load-bearing** —
    `HashSet` iteration order is not stable and the field is hashed.
- The completion body is the union of today's three, with the single divergence gated on
  `sets_has_drawn_for_turn`:
  `move top of library → Hand(player)`, `p.cards_drawn_this_turn += 1` (CR 121.1),
  `if sets_has_drawn_for_turn { p.has_drawn_for_turn = true }`, `GameEvent::CardDrawn`,
  then `miracle::check_miracle_eligible` (CR 702.94a).
  Empty library → `p.has_lost = true` + `GameEvent::PlayerLost { LibraryEmpty }` (CR 104.3b).

Rewire the three sites:

| site | call |
|---|---|
| `turn_actions::draw_card` (`:1171`) | `let (evts, _) = replacement::perform_one_draw(state, player, true, true, HashSet::new(), 0); Ok(evts)` — keep the `Result` signature and the eliminated/conceded guard at `:1176-1180`. |
| `effects::draw_one_card` (`:8547`) | becomes `fn draw_cards_for_player(state, player, n: u32) -> Vec<GameEvent>` owning the loop: `for i in 0..n { let (evts, out) = perform_one_draw(state, player, true, false, HashSet::new(), n - 1 - i); events.extend(evts); if matches!(out, Deferred \| LostToEmptyLibrary) { break; } }`. The four call sites at `:659`, `:716`, `:751`, `:4761` drop their own `for _ in 0..n` and call it with `n`. |
| `replacement::draw_card_skipping_dredge` (`:2631`) | `perform_one_draw(state, player, false, true, HashSet::new(), 0)` — keeps the eliminated/conceded guard at `:2638-2642`. |

**SR-4 classification for every new lookup in `perform_one_draw`** (all three source bodies
already carry the rationale; carry it forward verbatim):
- library zone read → `expect_zone` (SR-14 ground truth 2: the library zone is built pre-turn-1
  and never removed; `top()` returning `None` is the legal CR 104.3b empty case, not an absence);
- `players` reads → `expect_player` / `expect_player_mut` (ground truth 1);
- the hand move → `expect_move_object_to_zone` (the id was just read from the live library top).
  **Note**: `draw_card` currently uses `state.zone(&..)?` and `state.move_object_to_zone(..)?`
  (fallible). Moving to the `expect_*` forms changes its error surface from `Err` to a
  `debug_assert!` + swallow. That is the SR-4-correct classification and matches
  `draw_card_skipping_dredge`'s existing choice, but it **is** a behaviour change in the
  corrupted-state case — state it in the commit message.
- Re-pin the SR-25 `SWEPT_FILES` ceilings for all three files afterwards; the count will move
  (probably **down** in `effects/mod.rs`), and the ratchet fails on a move in either direction.

### Phase 3 — accept the answer and complete the draw

**3.1 `handle_order_replacements` (`rules/replacement.rs:139-193`) grows a second arm.**

Keep steps (a) and (b) unchanged. Replace step (c)'s unconditional `ok_or_else` with routing:

```rust
// CR 616.1: the sender must be the affected chooser of a pending event. Two kinds
// of pending event can be outstanding for the same player at the same time — a zone
// change and a draw — and `Command::OrderReplacements` carries no discriminator.
// Route by APPLICABILITY, which is total: `trigger_matches` requires the effect's
// trigger and the event's trigger to be the SAME `ReplacementTrigger` variant
// (rules/replacement.rs:262-330), so a `WouldChangeZone` replacement can never be
// applicable to a draw and vice versa. A well-formed answer therefore names exactly
// one of the two, and the check that decides "is this a legal answer" is the same
// check that decides "which question is this answering" — no new trust surface.
```

Order of evaluation: **zone change first** (pure preservation of today's behaviour, so no
existing test can regress), then draw. Concretely:

1. If a `pending_zone_changes` entry exists for `player` **and** every id in `ids` is in
   `find_applicable(WouldChangeZone-from-that-entry, entry.already_applied)` → today's path,
   byte-for-byte (`resolve_pending_zone_change`).
2. Else if a `pending_draws` entry exists for `player` **and** every id in `ids` is in
   `find_applicable(&ReplacementTrigger::WouldDraw { player_filter: PlayerFilter::Specific(entry.player) }, entry.already_applied)`
   → `resolve_pending_draw(state, ids[0], draw_idx)`.
3. Else → `Err(InvalidCommand(..))` naming which pending events exist for the player and which
   id failed applicability. **Both SR-29 security checks survive in the new arm**: the sender
   must be the affected chooser (there must be an entry with `player == player`), and every
   ordered id must be *currently applicable*, not merely registered.

**3.2 `resolve_pending_draw` — new, modelled on `resolve_pending_zone_change` (`:907-1055`).**

```
resolve_pending_draw(state, chosen_id, idx) -> Result<Vec<GameEvent>, GameStateError>

 1. pending = state.pending_draws[idx].clone()
 2. modification = state.replacement_effects.iter().find(|e| e.id == chosen_id)
       .map(|e| e.modification.clone())
       .ok_or(InvalidCommand("replacement effect {:?} not found"))?     // mirrors :932-934
 3. already_applied = pending.already_applied ∪ {chosen_id}             // CR 614.5
 4. emit GameEvent::ReplacementEffectApplied { effect_id: chosen_id, description }
       // mirrors :936-939. THIS EVENT IS THE ORDER DISCRIMINATOR — see §7 / §(a).
 5. state.pending_draws.remove(idx)                                     // mirrors :980
 6. if modification == SkipDraw:
        // CR 614.10 + CR 616.1f. The draw event has been replaced by nothing, so
        // there is no longer an event for a remaining replacement to modify:
        // "taking into account only replacement effects that would NOW be applicable"
        // yields the empty set. The loop ENDS. No card moves; cards_drawn_this_turn
        // is NOT incremented (a replaced draw is not a draw, CR 121.1).
        outcome = Replaced
    else:
        // Every non-SkipDraw modification is a no-op on a draw today (the else-branch
        // at :657-660). Apply it (i.e. record it in already_applied) and re-check.
        (evts, outcome) = perform_one_draw(
            state, pending.player,
            /* offer_dredge      */ false,          // see §3.3
            pending.sets_has_drawn_for_turn,
            already_applied,                        // CR 614.5 threading
            pending.remaining)
        // perform_one_draw's internal path is exactly CR 616.1f:
        //   find_applicable(WouldDraw{Specific(player)}, already_applied) then
        //   determine_action →
        //     NoApplicable  → perform the draw            (outcome Completed)
        //     AutoApply(id) → SkipDraw? stop, no draw     (outcome Replaced)
        //                     else insert id into already_applied and LOOP
        //                     ** the insert is mandatory: without it find_applicable
        //                        returns the same id forever and this spins **
        //     NeedsChoice   → push a NEW PendingDraw carrying the grown
        //                     already_applied and the SAME `remaining`, emit a second
        //                     ReplacementChoiceRequired, return Deferred
 7. if outcome != Deferred and pending.remaining > 0:
        // CR 614.11a: the replacement is complete; resume the sequence.
        for i in 0..pending.remaining {
            (evts, out) = perform_one_draw(state, pending.player, false,
                            pending.sets_has_drawn_for_turn, HashSet::new(),
                            pending.remaining - 1 - i);
            if matches!(out, Deferred | LostToEmptyLibrary) { break; }
        }
 8. Ok(events)
```

**Termination proof** (state it in the doc comment): `already_applied` grows by at least one
`ReplacementId` on every iteration of the CR 616.1f loop and `find_applicable` excludes it
(`:53-56`), so the loop runs at most `state.replacement_effects.len()` times. The
`remaining` sequence is strictly decreasing. There is no mutual recursion:
`resolve_pending_draw` calls `perform_one_draw`, never the reverse.

**3.3 Why the resume never re-offers dredge.** `check_would_draw_replacement` checks dredge
*first* and returns before ever reaching `determine_action` (`:631-637`), so a `NeedsChoice` can
only arise when the graveyard held no dredge-eligible card. Between the prompt and the answer,
one could arrive (a creature dies in response). Re-offering dredge mid-chain would restart a
CR 616.1 application the player has already begun, and would open a second pause with nowhere to
record it. `offer_dredge: false` on every resume draw. **Stated deviation**: CR 702.52a would
let the player dredge draws 2..N of a resumed sequence. Currently unobservable — see §5.4.
Seeded as **OOS-DP5-3**.

---

## 4. Precedence vs `pending_zone_changes`

**Rule**: applicability-based routing, zone change evaluated first (§3.1).

**Argument.** CR 616.1 governs a *single event*: "if two or more replacement and/or prevention
effects are attempting to modify the way an event affects an object or player, the affected
object's controller … or the affected player chooses one to apply". A pending zone change and a
pending draw are two **different** events, raised at different times. CR 616.1's ordering
apparatus (616.1a–f) says nothing about the order in which a player answers two separate
questions, and CR 101.4 APNAP governs only *simultaneous* choices by *different* players. So the
CR is silent, and the engine is free to accept either answer first.

Given that freedom, the only rule that cannot **misroute a well-formed answer** is to route by
what the answer is about. And that routing is *total*, not heuristic: `trigger_matches`
(`rules/replacement.rs:262-330`) pattern-matches `(effect_trigger, event_trigger)` on identical
variants, so a `ReplacementId` whose trigger is `WouldChangeZone` is never in
`find_applicable(WouldDraw{..})` and vice versa. The two candidate sets are provably disjoint,
so "zone change first" is a tie-break that can never actually fire.

Rejected alternatives:
- *Always zone change, then draw, ignoring applicability* — a player holding both would have
  their draw answer consumed by the zone change, whose applicability check would then reject it,
  losing the draw answer with a misleading error.
- *A discriminator field on `Command::OrderReplacements`* — PROTOCOL bump, forbidden by hard
  constraint 1.
- *Draw first* — gratuitously changes existing behaviour for a case that is provably
  unreachable; costs the "no existing test can regress" guarantee for nothing.

---

## 5. Hard constraint 3 — what a deferred draw means, and why it cannot hang

### 5.1 The semantics chosen

**A deferred draw is a recorded, non-blocking obligation.** Precisely:

- Nothing in the engine ever *waits* on `pending_draws`. Priority is not gated on it, SBAs do not
  consult it, `advance_step` / `advance_turn` do not consult it, `handle_all_passed` does not
  consult it.
- The **sequence** stops (CR 614.11a) — that is `remaining` — but the **effect** does not.
  `execute_effect` returns; the rest of a "draw three, then discard three" resolves; the stack
  object finishes resolving; the game moves on.
- If the entry is never answered, the draw simply never happens and the entry sits inert until
  the game ends.

### 5.2 Why this is safe against every hang mode named in the constraint

- **`ForEach::EachPlayer` draws.** `Effect::DrawCards` loops seats at `effects/mod.rs:657`. Only
  the *inner* per-seat loop breaks; the outer loop continues to the next player. Each seat's
  sequence is independent, which is what CR 614.11a describes.
- **A fuzzer/simulator seat that never sends `OrderReplacements`.** It never has to. The command
  is out-of-band; no code path is blocked on it. There is no equivalent of PB-DP4's
  "`driver.rs` answers a rejected command with a silent `PassPriority`" retry loop here, because
  nothing rejects a pass.
- **`resolve_pending_draw` recursion.** Bounded, proof in §3.2.

### 5.3 The deviation, stated explicitly

**Deviation from CR 614.11a / 121.1**: within the deferral window the draw has not happened,
though the CR would have completed it (or completed its replacement) before anything else. An
effect that draws and then acts on what was drawn ("draw three cards, then discard three") will
perform the second half against a hand that does not yet contain the drawn cards. This is a
strict improvement on the status quo — today the draws are *destroyed*, so the second half runs
against the same wrong hand **and** the cards are lost permanently — but it is a deviation and
must be recorded in the audit's DP-5 row. Seeded as **OOS-DP5-5**.

### 5.4 What is deliberately NOT built, and why

PB-DP4 established a *deadline* (`force_resolve_overdue_payments`, `rules/engine.rs:1220`, hooked
into `handle_all_passed`'s stack-empty branch at `:1914`). The symmetric move here would be to
auto-pick `choices[0]` for any unanswered `PendingDraw` at that same boundary. **Do not build
it in PB-DP5.** Reasons, in order of weight:

1. **It has zero benefit today.** DP-5 is unreachable from a legal deck (§9.1) — no corpus card
   registers a `WouldDraw` replacement — so the sweep could never fire in a real game.
2. **It would be the *primary* path, not a fallback.** `OrderReplacements` has no `LegalAction`
   (out of scope per the brief), so no bot can ever answer. A deadline would therefore fire in
   100% of automated games, converting a PB whose whole purpose is restoring player agency into
   a new DP-25-class "the engine chose for you" site.
3. **Not building it costs nothing relative to the status quo.** Unanswered ⇒ draw lost ⇒
   exactly today's outcome. No regression, no hang.

The deadline is the correct thing to build **at the same time as** the `LegalAction`, as one
follow-up PB. Filed as **OOS-DP5-1** + **OOS-DP5-2**.

---

## 6. Hash / protocol gate expectation, and what falsifies it

**Prediction**: `HASH_SCHEMA_VERSION` **63 → 64**; `PROTOCOL_VERSION` **27, unchanged**.

**Mechanism.** `tests/core/hash_schema.rs::declaration_fingerprint_is_pinned` digests the
normalized declaration text of the transitive **serde** closure of `GameState`
(`compute_decl_fingerprint`, `:667`). Adding a field to `GameState` and a new struct
`PendingDraw` to that closure moves `decl_fingerprint` by construction.
`stream_fingerprint_is_pinned` also moves, because `HASH_SCHEMA_VERSION` is folded in as the
stream's first byte (the v40 mechanism documented at `state/hash.rs:650-667`) — note the
`canonical_fixture` (`:711-773`) does **not** populate `pending_zone_changes` and need not
populate `pending_draws` either.

`PROTOCOL_SCHEMA_FINGERPRINT` is rooted in `Command`, `GameEvent` and `ReplayLog`
(`rules/protocol.rs:27-31`); `ReplayLog` is `{ hash_schema_version: u8, commands: Vec<Command> }`
(`:628-633`) and never embeds `GameState`. `PendingDraw` is reachable only from `GameState`.
PROTOCOL stays 27 — the same "off the wire closure ⇒ HASH-only bump" reasoning the v9 history
line already records for `EffectFilter`.

**Procedure (do it in Phase 1, before the behaviour work):**
1. Add the field/struct. Run `cargo test -p mtg-engine --test core hash_schema`.
2. Set `HASH_SCHEMA_VERSION = 64` (`state/hash.rs:578`) and add a `- 64:` History line naming
   `GameState.pending_draws` + `PendingDraw`.
3. Re-run; paste the **freshly printed** `decl_fingerprint` and `stream_fingerprint` into a new
   appended `HashSchemaEpoch { version: 64, .. }` row. **Order matters**: the stream digest must
   be read *after* the constant is bumped, or the pasted value is stale and the next run fails.
4. Update all **43** `HASH_SCHEMA_VERSION` sentinels (§8.4).

**Falsifiers — if any of these happens, STOP and report rather than papering over it:**

- `declaration_fingerprint_is_pinned` **passes without a bump.** The scanner did not see
  `PendingDraw`. Most likely causes: the struct was placed outside the scan roots, or a field
  carries a bare `#[serde(skip)]` (which `blank_serde_skip_field_types`, `:356`, deliberately
  blanks). Check `state_closure_is_not_vacuous_and_bounded` (`:870`) and
  `every_referenced_type_resolves` (`:907`) first. **Do not hand-bump the constant to make it
  look right** — PB-DP2's falsified prediction is the precedent for reporting rather than
  forcing.
- `PROTOCOL_SCHEMA_FINGERPRINT` **moves.** Something put `PendingDraw` (or a new variant) on the
  wire. That is hard constraint 1 territory: **stop, do not bump PROTOCOL, report to the
  coordinator.**
- `not_hashed_allowlist_has_no_dead_entries` fails ⇒ a `PendingDraw` field is missing from the
  `HashInto` impl (SR-19). Hash it; do not allowlist it.

---

## 7. Tests, with per-test fail-before predictions

**Placement (SR-9a)**: new file
`crates/engine/tests/primitives/pb_dp5_pending_draw_choice.rs`, with a `mod
pb_dp5_pending_draw_choice;` line added to `crates/engine/tests/primitives/main.rs` **in the
same commit** (the existing roster is alphabetical, `:8-…`). Never add a top-level `tests/*.rs`.
The one existing test is strengthened **in place** so the audit's citation stays valid.

Every test cites CR 616.1 and/or 614.11 in a doc comment (Architecture Invariant #8).

**Fail-before protocol** (PB-DP4 close-out shape): `git stash` the engine changes, run the
probes that compile against pre-fix API, record the *actual observed* behaviour per test, restore
byte-identically. Tests that name `pending_draws()` cannot compile pre-fix — for those, the
pre-fix evidence is the paired command-level probe noted in the table.

| # | test | what it proves | pre-fix prediction |
|---|---|---|---|
| **T0** | *(strengthen)* `test_draw_needs_choice_emits_replacement_choice_required` — `tests/rules/replacement_effects.rs:2938-3002` | keeps its existing three assertions; **adds** `pending_draws().len() == 1`, `player == p1`, `remaining == 0`, `already_applied.is_empty()` | does not compile pre-fix (new API); the *existing* three assertions pass pre-fix and post-fix — that is exactly the vacuity criterion 5532 targets |
| **T1** | `test_dp5_order_replacements_after_deferred_draw_is_accepted` — 2 `SkipDraw` `WouldDraw` effects (ids 600, 601), `draw_card`, then `process_command(OrderReplacements { p1, ids: vec![601, 600] })` | **the headline**: the command is no longer rejected | **FAILS**: `Err(GameStateError::InvalidCommand("player PlayerId(1) is not the affected player of any pending replacement choice"))` from `replacement.rs:167-172` |
| **T2** | `test_dp5_chosen_replacement_is_the_one_applied` — same setup, submit `[601, 600]`, assert the **first** `ReplacementEffectApplied` has `effect_id == ReplacementId(601)` | order discriminator half A. 601 is the **second**-registered id, so passing rules out "used `choices.first()`", "used registration order" and "used `applicable[0]`" | **FAILS**: command rejected, zero `ReplacementEffectApplied` events |
| **T3** | `test_dp5_chosen_replacement_is_the_one_applied_mirrored` — identical but `[600, 601]`, asserts `600` | order discriminator half B. T2+T3 differ **only** in the submitted order and assert **different** `effect_id`s ⇒ non-vacuous (§(a)) | **FAILS**: as T2 |
| **T4** | `test_dp5_draw_completes_through_chosen_order` — two **non-`SkipDraw`** `WouldDraw` effects (`RedirectToZone(ZoneType::Exile)` and `DoubleTokens`, both draw no-ops); after `OrderReplacements` assert: card in `Hand(p1)`, `Library(p1).len() == 0`, `cards_drawn_this_turn == 1`, a `CardDrawn` event, `pending_draws()` empty | **criterion 5532** — the draw *actually completes*, not merely defers; also exercises the CR 616.1f re-check to `NoApplicable` | **FAILS**: command rejected; hand empty, library 1, `cards_drawn_this_turn == 0` |
| **T5** | `test_dp5_effect_draw_path_records_pending_state` — `Effect::DrawCards { count: 1 }` via `execute_effect` | **criterion 5531's "both emit sites"** — covers `effects/mod.rs:8556` | does not compile pre-fix; paired probe: pre-fix, `OrderReplacements` after the same `execute_effect` is rejected identically to T1 |
| **T6** | `test_dp5_draw_sequence_stops_and_resumes` (CR **614.11a**, **121.2**) — `Effect::DrawCards { count: 3 }`, 3 library cards, two non-`SkipDraw` effects. Assert **exactly one** `ReplacementChoiceRequired` and `pending_draws()[0].remaining == 2`; answer; assert 1 card in hand + a fresh entry with `remaining == 1`; answer twice more; assert 3 cards in hand and `pending_draws()` empty | the sequence stops at the replaced draw and resumes through it | **FAILS loudly**: pre-fix emits **three** `ReplacementChoiceRequired` events and draws **zero** cards. Record this — it is worse than the audit's "the draw is eaten" and belongs in the DP-5 row |
| **T7** | `test_dp5_dredge_decline_path_records_pending_state` — dredge card in graveyard + 2 `SkipDraw` effects; draw → `DredgeChoiceRequired`; `ChooseDredge { card: None }` → `ReplacementChoiceRequired` + pending entry; `OrderReplacements` accepted | covers the **third** emit site (`replacement.rs:2666`) the audit never named | **FAILS**: the prompt is emitted, the `OrderReplacements` is rejected |
| **T8** | `test_dp5_order_replacements_rejects_non_affected_player` — p1 has the pending draw, **p2** sends `OrderReplacements` | SR-29 trust boundary preserved in the new arm | passes pre-fix for the wrong reason (everything is rejected). Post-fix it must still `Err` — this is a *guard*, so also assert the error message names the missing pending event, not "not applicable" |
| **T9** | `test_dp5_order_replacements_rejects_inapplicable_id` — a registered `WouldGainLife` replacement id (or a `WouldDraw` scoped to p2) submitted against p1's pending draw | SR-29 applicability check preserved | passes pre-fix for the wrong reason; post-fix must `Err` with the applicability message |
| **T10** | `test_dp5_precedence_zone_change_and_draw_coexist` (§4) — p1 has **both** a pending zone change and a pending draw. Submit the zone-change ids → the zone change resolves and `pending_draws()` is **untouched**; then submit the draw ids → the draw resolves | routing is applicability-based, not positional | first command passes pre-fix (existing path); second **FAILS** pre-fix (rejected) |
| **T11** | `test_dp5_616_1f_recheck_stops_at_skip_draw` (CR **616.1f**, **614.5**, **614.10**) — `{SkipDraw(A), RedirectToZone(B)}`. Choose **B** first → `ReplacementEffectApplied{B}`, then the re-check auto-applies A, no `CardDrawn`, card stays in library, `pending_draws()` empty. Choose **A** first → the chain stops immediately, no `CardDrawn` | the re-check loop runs and terminates; an effect applies at most once | **FAILS**: both commands rejected |
| **T12** | `test_dp5_unanswered_pending_draw_does_not_deadlock` (**hard constraint 3**) — create a pending draw in the draw step, then pass priority through to the next turn without ever answering. Assert: the step/turn advances, no `Err`, no infinite loop, and the entry is still present (the chosen semantics, §5.1) | the deadlock probe | passes pre-fix (there is nothing to deadlock on) — this is a **regression guard**, and its post-fix value is that it fails if anyone later gates progress on `pending_draws` |
| **T13** | `test_dp5_wire_version_sentinels` — `assert_eq!(HASH_SCHEMA_VERSION, 64u8)`, `assert_eq!(PROTOCOL_VERSION, 27)` | §6 | n/a |

**Test-validity note** (`memory/conventions.md`, "Test-validity MEDIUMs are fix-phase HIGHs"):
T2/T3 are the pair that makes criterion 5532 non-vacuous. If the reviewer finds they pass against
a build that ignores the submitted order, that is a **fix-phase HIGH**, not a LOW.

---

## 8. Answers to the two questions the coordinator singled out

### (a) Criterion 5532 — is a distinguishable scenario reachable without widening the draw path?

**Yes. Two complementary discriminators, neither of which needs a new
`ReplacementModification`.** But the honest caveat below is load-bearing and must be recorded in
the audit row.

**D1 — order is observable in the event stream, even with two `SkipDraw`s.**
`resolve_pending_draw` emits `GameEvent::ReplacementEffectApplied { effect_id: chosen_id, .. }`
for the **chosen** effect before anything else (mirroring
`resolve_pending_zone_change:936-939`). With ids 600 and 601 registered in that order, submitting
`[601, 600]` must produce `effect_id: 601` and submitting `[600, 601]` must produce
`effect_id: 600`. Because 601 is the *second*-registered id, T2 passing rules out every
plausible wrong implementation at once: `choices.first()`, registration order, `applicable[0]`,
and the ETB path's `NeedsChoice ⇒ choices.first()` shortcut (`replacement.rs:1116-1120`). This
is a real discriminator on a real degree of freedom, not a tautology.

**D2 — the draw genuinely completes (card in hand) with two non-`SkipDraw` replacements.**
Two `WouldDraw` effects whose modifications are draw no-ops (e.g. `RedirectToZone(Exile)` and
`DoubleTokens`) produce `NeedsChoice` from `determine_action` (`:118-122`, CR 616.1e) and, after
the answer, the CR 616.1f re-check settles on `NoApplicable` and **the draw is performed**. T4
asserts card-in-hand, library decremented, `cards_drawn_this_turn` incremented and a `CardDrawn`
event. That is criterion 5532's literal text — "the draw actually completes (card in hand /
replacement applied)" — satisfied on both clauses.

**The caveat, stated plainly.** *Game-state-level* discrimination between the two orders is
impossible today. `SkipDraw` is the only modification the draw path honours (`:651-660`), it is
terminal, and every other modification is a no-op — so the two orders can differ in **which
effect is credited** and in **how many effects are credited**, but never in the resulting board,
hand or library. **This is not a reason to widen for PB-DP5**: D1+D2 discharge 5532 without it,
and widening would be a much larger PB.

**The minimum widening, if the coordinator wants state-level discrimination** (recommended as a
*separate* PB, seeded **OOS-DP5-6**): add a `ReplacementModification` that modifies a draw
without ending it. The real cards that need it, verified against MCP oracle text:

- **Alhammarret's Archive** / **Teferi's Ageless Insight** — *"If you would draw a card except
  the first one you draw in each of your draw steps, draw two cards instead."* ⇒
  `DrawAdditionalCards(u32)`. `teferis_ageless_insight.rs` is `Completeness::inert` **today**,
  with the TODO *"Draw replacement effect with draw-step exception too complex for DSL"*.
- **Notion Thief** — *"…instead that player skips that draw and you draw a card."* ⇒ a
  skip-and-redirect variant. Genuinely order-sensitive against Alhammarret's Archive; this is the
  canonical CR 616.1 draw-order interaction.
- **Laboratory Maniac** — *"If you would draw a card while your library has no cards in it, you
  win the game instead."* `laboratory_maniac.rs` is `inert` on exactly this.
- **Out of the Tombs** — `out_of_the_tombs.rs:32-35` names the blocker verbatim: *"WouldDraw's
  sole `ReplacementModification` is `SkipDraw`… Same blocker as Laboratory Maniac."*

So the widening PB would unblock **≥3 `inert` defs** and would be the first thing that makes
DP-5 reachable in a real game. It is a real, well-motivated follow-up — just not this one.

### (b) Hard constraint 3 — what a deferred draw means inside synchronous effect resolution

Full answer in **§5**. Summary of the position and its defence:

- **Semantics**: a deferred draw is a **recorded, non-blocking obligation**. `draw_one_card`'s
  synchronous `Vec<GameEvent>` contract is preserved; nothing suspends.
- **What actually stops**: only the *draw sequence* (CR 614.11a), via `PendingDraw.remaining` and
  a `break` in the per-player loop. The effect, the resolution, the stack and priority all
  continue.
- **Why that is not a hang**: no code path is gated on `pending_draws`. A seat that never answers
  loses the draw — **exactly today's outcome**, so the fix is a strict improvement with no new
  failure mode. There is no analogue of PB-DP4's silent-`PassPriority` retry loop, because
  nothing rejects a pass.
- **`ForEach::EachPlayer`**: the *inner* per-seat loop breaks; the outer seat loop continues.
  Per-seat sequences are independent, which is what CR 614.11a describes.
- **Stated deviation** (§5.3): the rest of the effect runs before the deferred draw completes.
  Recorded in the DP-5 row and seeded as OOS-DP5-5.
- **Deliberately not built** (§5.4): a PB-DP4-style deadline sweep. Zero benefit today
  (unreachable from a legal deck), and with no `LegalAction` for `OrderReplacements` it would
  become the primary path in every automated game — a new "the engine chose for you" site inside
  a PB about restoring agency. Bundled with the `LegalAction` as OOS-DP5-1/2.

---

## 8.4 Mechanical blast radius of the HASH bump (43 sites)

`tests/core/hash_schema.rs:1194` (`63`) plus 42 × `HASH_SCHEMA_VERSION, 63u8` in:
`tests/casting/optional_cost_and_counter_tax.rs:1139` ·
`tests/mechanics_e_l/effect_sacrifice_permanents_filter.rs:136` ·
`tests/rules/loyalty_target_validation.rs:355` ·
and 39 files under `tests/primitives/` (`pb_ac1`, `pb_ac3`, `pb_ac4`, `pb_ac5`, `pb_ac6`,
`pb_ac7_type_change_ability_removal`, `pb_ac8`, `pb_ac9`, `pb_ef1`, `pb_ef2`, `pb_ef6`, `pb_ef7`,
`pb_ef10`, `pb_ef11_spell_single_target`, `pb_ef11_wheel_greatest_discarded`, `pb_os5`, `pb_os6`,
`pb_os7`, `pb_os8`, `pb_os9`, `pb_os10`, `pbd_damaged_player_filter`, `pbn_subtype_filtered_triggers`,
`pbp_power_of_sacrificed_creature`, `pbt_up_to_n_targets` ×2, `primitive_pb_cc_a`,
`primitive_pb_cc_c_followup`, `primitive_pb_eat`, `primitive_pb_ewc`, `primitive_pb_ewcd`,
`primitive_pb_lki_cc`, `primitive_pb_lki_power`, `primitive_pb_oos_lki_power_3`, `primitive_pb_ts`,
`primitive_pb_xa`, `primitive_pb_xa2`, `primitive_pb_xs`, `primitive_pb_xs_e`).
Regenerate the list with `rg -n "HASH_SCHEMA_VERSION, 63" ` rather than trusting this snapshot.

---

## 9. Pre-survey bullets that turned out to be **WRONG**

Per the brief, this section is a required output.

**W1 — "Reachable with any two `WouldDraw` replacements, including in the draw step" is
materially wrong as a *reachability* claim (and so is audit §5's "DP-5 needs two `WouldDraw`
replacements on the board").**
**Zero card definitions in the 1,804-card corpus register a `WouldDraw` replacement effect.**
Evidence: `rg "WouldDraw" crates/card-defs/src/defs/` returns exactly one hit — a
`Completeness::inert` *note* in `out_of_the_tombs.rs:32`. Workspace-wide, the only non-test
`ReplacementTrigger::WouldDraw` constructions are the two **consumers** in `replacement.rs`
(`:638`, `:2647`) and one synthetic sample in the SR-15 discriminant registry
(`state/ability_definition_registry.rs:458`). The DSL *supports* it —
`register_permanent_replacement_abilities` (`replacement.rs:~2000-2077`) will register any
`AbilityDefinition::Replacement` and binds `PlayerFilter` placeholders — the corpus just never
uses it. **Consequence**: DP-5 is reachable only from a test-constructed state, the card yield is
**0 `Complete` defs made right**, and the honest framing of the PB is *"a class-D correctness bug
and a precondition for ever authoring Notion Thief / Alhammarret's Archive / Teferi's Ageless
Insight / Laboratory Maniac"*, not *"live-wrong in games today"*. The DP-5 row's reachability
sentence must be corrected.

**W2 — "That difference is either a real distinction or a latent bug" is a false dichotomy.**
There is a third answer and it is the true one: `PlayerState::has_drawn_for_turn` is
**write-only dead state**, never read by any engine logic anywhere in the workspace (§2.4). The
divergence is currently unobservable *except through the state hash*, which is precisely why the
plan preserves it byte-for-byte rather than unifying it.

**W3 — "`draw_one_card` does a subset" is imprecise.** `draw_one_card` **does** run the CR 702.94a
miracle check (`effects/mod.rs:8591-8596`). It differs from `draw_card` by exactly one write.

**W4 — "with two `SkipDraw`s… the chosen order is unobservable" is only half true.** It is
unobservable in *game state*; it is fully observable in the *event stream* via
`ReplacementEffectApplied.effect_id`, because the resume emits the chosen id explicitly (§8(a)
D1). Criterion 5532 is therefore satisfiable with **no** widening — a materially better outcome
than the pre-survey anticipated.

**W5 — the finding understates the damage on the effect path.** The wip and the audit both say
"the draw is eaten" (singular). At `effects/mod.rs:659`/`:716`/`:751`/`:4761` the caller is a
`for _ in 0..n` loop that **keeps iterating after the deferral**, so `Effect::DrawCards
{ count: 3 }` emits **three** unanswerable `ReplacementChoiceRequired` events and draws **zero**
cards. Same shape for the dredge pause. Update the DP-5 row.

**W6 — minor line-number drift** (recorded so the audit can be re-pinned): the audit's
`turn_actions.rs:1186-1189` is really the `use` at `:1185` and the `NeedsChoice` arm at
`:1189-1192`; `effects/mod.rs:8553-8564` is the whole match, the arm is `:8556-8559`; the
pre-survey's `draw_card_skipping_dredge (~:2666)` is the *arm* — the fn starts at `:2631`.

**Bullets that were RIGHT and should be recorded as confirmed**: the third emit site exists and
is reachable (§1.2); `handle_order_replacements`' step breakdown (a)–(e) at `:144/:151/:163/:176/:191`;
the resume must mirror `resolve_pending_zone_change`'s CR 616.1f loop; §8's "HASH bump, no new
`Command` if it reuses `OrderReplacements`" (§6); and the SR-25 ratchet warning — `replacement.rs`
**is** a swept file (ceiling 24), and so are `effects/mod.rs` (111) and `turn_actions.rs`.

---

## 10. Seeds — file in `docs/audits/decision-point-audit.md` §8.1

| seed | finding | class |
|---|---|---|
| **OOS-DP5-1** | **`Command::OrderReplacements` has no `LegalAction`.** `crates/simulator/src/legal_actions.rs` never offers it (zero occurrences of `OrderReplacements` outside `crates/engine` + `crates/card-types`), so no bot or M11-local seat can ever answer a CR 616.1 prompt — for a **draw or a zone change**. The pre-existing `pending_zone_changes` path has the same hole. Same class as PB-DP4's §9 recommendation. Simulator-only, no wire change. | agency / move-generation gap |
| **OOS-DP5-2** | **No deadline for an unanswered `PendingDraw` / `PendingZoneChange`.** PB-DP4's `force_resolve_overdue_payments` (`rules/engine.rs:1220`, hooked at `:1914`) has no CR 616.1 analogue. Deliberately not built in PB-DP5 (§5.4): with no `LegalAction` it would fire in 100% of automated games and become a new "the engine chose for you" site. Should ship **together with** OOS-DP5-1 as one PB. | correctness, deferred (design) |
| **OOS-DP5-3** | **The resume never re-offers dredge.** `resolve_pending_draw` draws the deferred draw and the `remaining` sequence with `offer_dredge: false` (§3.3), so CR 702.52a is not offered on draws 2..N of a resumed sequence. Deliberate — re-offering would restart a CR 616.1 chain the player already began and would open a second pause with nowhere to record it. Presently unobservable: the intersection of "has a dredge card" and "has 2+ `WouldDraw` replacements" is empty in the corpus (there are **no** `WouldDraw` cards at all). | correctness, stated deviation |
| **OOS-DP5-4** | **`PlayerState::has_drawn_for_turn` is write-only dead state, and incoherently written.** Never read by any engine logic (§2.4); written by `turn_actions::draw_card` (`:1218`) — which also serves the monarch end-step draw (`:754`) and Ravenous (`resolution.rs:4641`), neither of which is "the draw for the turn" — and by `draw_card_skipping_dredge` (`:2700`), so an **effect** draw routed through a dredge decline sets it while the direct effect path does not. It **is** hashed (`hash.rs:2023`), so deleting it is a HASH bump and cannot be a drive-by. | cosmetic / dead state (wire) |
| **OOS-DP5-5** | **A deferred draw does not stop the rest of the effect.** §5.3. "Draw three, then discard three" runs its second half against a hand that does not yet hold the drawn cards — a CR 614.11a / 121.1 timing deviation. Strictly better than the status quo (today the draws are destroyed). Closing it needs a suspendable effect resolver, i.e. the M10/M11 pending-decision machinery §8's sequencing note calls for. | correctness, stated deviation |
| **OOS-DP5-6** | **`WouldDraw` honours only `SkipDraw`.** `replacement.rs:651-660` and `:2659-2664`: every other `ReplacementModification` on a draw is a silent no-op that does not even emit `ReplacementEffectApplied`. This is why two draw replacements can never differ in *outcome* (§8(a)). ≥3 `inert` defs blocked on it: `laboratory_maniac.rs`, `teferis_ageless_insight.rs`, `out_of_the_tombs.rs` (which names the blocker verbatim at `:32-35`). Minimum widening: `DrawAdditionalCards(u32)` (Alhammarret's Archive / Teferi's Ageless Insight) + a skip-and-redirect variant (Notion Thief) + a replace-with-effect variant (Laboratory Maniac). Own PB. | DSL gap / card yield |
| **OOS-DP5-7** | **`Command::ChooseDredge` has NO pending-state gate — a live, reachable exploit.** `rules/engine.rs:302-306` calls only `validate_player_exists`; `handle_choose_dredge` (`replacement.rs:2527`) validates the *card* (in graveyard, has `Dredge(n)`, library ≥ n) but **never that a draw is pending**. So any player, at any time, can send `ChooseDredge { card: None }` and take a **free extra card** (`draw_card_skipping_dredge` draws unconditionally), or `ChooseDredge { card: Some(x) }` and dredge at will. Exactly DP-5's trust-boundary class, but **live and reachable today** — dredge defs exist (`golgari_grave_troll.rs`) and the command is script-reachable. Fix reuses PB-DP5's machinery: give the `DredgeAvailable` pause its own `PendingDraw`-style entry and require+consume it in `handle_choose_dredge`. Existing dredge tests (`tests/mechanics_a_d/dredge.rs`) and golden script `replacement/014` all reach `DredgeChoiceRequired` first, so they would stay green. **Deliberately not folded into PB-DP5** (implement-phase default-to-defer). Arguably a higher-severity finding than DP-5 itself. | correctness / trust boundary, **live** |
| **OOS-DP5-8** | **The `AutoApply` draw arm applies an effect and emits nothing.** `replacement.rs:657-660`: with exactly one applicable non-`SkipDraw` `WouldDraw` replacement, the effect is neither applied nor recorded — no `ReplacementEffectApplied`, no `already_applied` entry, no event at all. A player watching the log cannot tell the replacement exists. Same diagnosability class as OOS-DP4-5. Fold into the OOS-DP5-6 widening PB. | diagnosability |

---

## 11. Verification checklist

- [ ] Phase 1 lands **first**, including the HASH 63→64 bump and all 43 sentinels (§6, §8.4)
- [ ] `PROTOCOL_VERSION` still **27**; `PROTOCOL_SCHEMA_FINGERPRINT` unmoved (if it moves: **STOP**, hard constraint 1)
- [ ] `HASH_SCHEMA_HISTORY` has an appended v64 row + a `- 64:` History line; **no existing row edited**
- [ ] `impl HashInto for PendingDraw` reads every field as `self.<field>`; `NOT_HASHED` still empty (SR-19)
- [ ] `already_applied` is sorted before being stored (determinism, SR-9b)
- [ ] SR-25 `SWEPT_FILES` ceilings re-pinned for `effects/mod.rs`, `rules/replacement.rs`, `rules/turn_actions.rs`
- [ ] SR-4: every new lookup in `perform_one_draw` / `resolve_pending_draw` picks a side (`expect_*` vs `fizzle_*`) with a stated rationale
- [ ] `mod pb_dp5_pending_draw_choice;` added to `crates/engine/tests/primitives/main.rs` (SR-9a)
- [ ] T0 strengthened in place at `tests/rules/replacement_effects.rs:2938` (the audit cites that line)
- [ ] Fail-before/pass-after evidence recorded per test, with **observed** pre-fix behaviour (not predicted)
- [ ] Both SR-29 security checks present in the new `handle_order_replacements` arm (affected-chooser + currently-applicable), covered by T8/T9
- [ ] `cargo build --workspace` after **every** phase (hard constraint 6)
- [ ] `cargo test --all` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` · `tools/check-defs-fmt.sh` (SR-35)
- [ ] Golden-script suite green (`cargo test --test run_all_scripts`), especially `replacement/014_golgari_grave_troll_dredge.json` — do **not** start the replay-viewer HTTP server to validate
- [ ] `docs/audits/decision-point-audit.md` updated: §4 L383 (third site), §5 DP-5 row (SHIPPED + the W1/W5 corrections + the §5.3 deviation), §8 PB-DP5 row (prediction confirmed/falsified), §8.1 (OOS-DP5-1..8)
- [ ] **0 card-def edits expected.** If the runner finds itself editing `crates/card-defs/`, stop — that is out of scope and means the plan was wrong

## 12. Risks

1. **The `perform_one_draw` factoring is the riskiest part of the PB**, not the pending state.
   Three call sites with three slightly different completion bodies, one of which
   (`has_drawn_for_turn`) is hash-visible. Mitigation: the `sets_has_drawn_for_turn` flag; a
   golden-script run before/after (SR-9b per-step fingerprints will catch any drift the unit
   tests miss). If the runner cannot keep the three byte-identical, **stop and report** rather
   than "improving" the semantics.
2. **`draw_card`'s error surface changes** if it moves from `state.zone(..)?` /
   `move_object_to_zone(..)?` to the `expect_*` forms. SR-4-correct, but a behaviour change in
   the corrupted-state case. Call it out in the commit message.
3. **Infinite loop in the CR 616.1f re-check** if an auto-applied no-op modification is not
   inserted into `already_applied`. `find_applicable` would return it forever. Guarded by the
   termination note in §3.2 — make it a real doc comment, not a mental note.
4. **`HashSet` iteration order leaking into the state hash** via `already_applied`. Sort. §11.
5. **43 sentinel edits** are a mechanical trap: miss one and a distant test target fails with a
   message that does not mention PB-DP5. Regenerate the list with `rg`, do not hand-copy §8.4.
6. **Scope creep toward OOS-DP5-7** (the dredge gate). It is tempting — same machinery, ~30
   lines — and it is deliberately out of scope. `memory/conventions.md`, "implement-phase
   default-to-defer".
