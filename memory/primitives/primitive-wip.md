---
pb: PB-AC6
title: Phase & opponent-action conditions
phase: implement
plan_file: memory/primitives/pb-plan-AC6.md
review_file: memory/primitives/pb-review-AC6.md
---

# PB-AC6 — Phase & opponent-action conditions

## Scope (task scutemob-49, campaign-plan §2 PB-AC6 row)

**3 new `TriggerCondition` variants:**
1. `AtBeginningOfFirstMainPhase` — needs a generic CardDef sweep in turn
   structure code, analogous to the existing `upkeep_actions()` /
   `end_step_actions()` sweeps. **The B9 lesson**: hardcoded keyword-only sweeps
   silently drop CardDef triggers.
2. `AtBeginningOfPostcombatMain` — ditto. Must distinguish FIRST main from
   postcombat main precisely (CR 505.1).
3. `WhenBecomesTarget` — fires on **ANNOUNCEMENT** of the targeting spell/ability
   (CR 603.2), NOT on resolution.

**5 new `Condition` variants:**
1. `YouAttackedThisTurn`
2. `CreatedATokenThisTurn`
3. `OpponentCastNSpells`
4. `SpellMastery` — an **ability word** (two or more instants/sorceries in your
   graveyard), not a keyword ability
5. `OpponentControlsMoreLandsThanYou`

## CR refs — ADVISORY ONLY, verify each via mtg-rules MCP
500 (turn structure), 505.1 (main phase), 603.2 (triggered abilities /
becomes-target timing), 700.2 (modal). Do **not** grep the CR file: it has bare
`\r` line endings, so rule-number greps silently match nothing. Use the
mtg-rules MCP for all CR verification.

## Hazards (from task brief)
1. **NEW MUTABLE TRACKING FIELDS are the core of this batch.** Every one MUST be
   added to `state/hash.rs` `HashInto` impls AND get a correct turn-boundary
   reset. This exact omission was a review HIGH in **both PB-AC1 and PB-AC5**
   (twice!). Mutation-verify the hash tests — flip each field, assert the hash
   actually changes.
2. Verify the KW/AbilDef/SOK discriminant chain from *current code* before adding
   variants — do not trust remembered numbers.
3. Exhaustive matches in `tools/tui/src/play/panels/stack_view.rs`
   (`StackObjectKind`) and `tools/replay-viewer/src/view_model.rs`
   (`StackObjectKind` + `KeywordAbility`). Run `cargo build --workspace` after
   every impl phase — runners miss this ~50% of the time.
4. Harness: new trigger conditions may need `script_schema.rs` /
   `translate_player_action` wiring.
5. Do NOT commit phantom `.claude/skills/*` deletions that appear in fresh
   worktrees (restored at session start).
6. Load `memory/gotchas-rules.md` before planning — this batch touches turn
   structure, triggers, and targeting.

## Implementation pointers
- This-turn trackers need new `GameState`/`PlayerState` fields.
  `previous_turn_spells_cast` (Daybound) already exists — study its reset pattern.
- `OpponentControlsMoreLandsThanYou` battlefield counts must respect
  `is_phased_in()` and W3-LC `calculate_characteristics()` discipline.

## Roster
Discounted yield ~18 cards. The **planner** identifies the real roster from
oracle text — grep card defs for BOTH `// TODO` and `// ENGINE-BLOCKED` markers
citing main-phase / becomes-target / spell-mastery / attacked-this-turn /
token-this-turn / opponent-cast / land-count patterns. Card rosters in plan docs
are advisory; oracle text via MCP is authoritative.

## Close includes backfill
PB is not done until every unblocked card is re-authored and its stale markers
are deleted.

## Phases
- [x] plan (primitive-impl-planner → pb-plan-AC6.md)
- [x] implement (primitive-impl-runner, 2026-07-09)
  - **PLAN CORRECTION APPLIED** (per task brief, overrides plan Change 10): added
    a dedicated `PlayerState::spells_cast_this_game_turn: u32`, reset for ALL
    players in `reset_turn_state` (NOT the storm-scoped `spells_cast_this_turn`,
    which is deliberately reset only for the incoming active player). Verified in
    code: `turn_actions.rs` resets `spells_cast_this_turn` only for the incoming
    active player (storm scoping); reading it for a non-active opponent would
    accumulate across intervening turns — rejected as wrong game state.
  - **DEVIATION FROM PLAN**: the plan's correction cited 4 increment sites
    (`resolution.rs:5133`, `resolution.rs:5787`, `copy.rs:462`, `copy.rs:688`).
    Grepped ALL `spells_cast_this_turn +=`/`saturating_add` sites and found a
    **5th, uncited site**: `casting.rs:4709` — the PRIMARY normal-cast path (every
    ordinary `CastSpell`). The plan's 4 sites are all secondary/free-cast paths
    (cipher, suspend, cascade, discover). Incremented
    `spells_cast_this_game_turn` at all 5 sites; omitting `casting.rs:4709` would
    have made `OpponentCastNSpells` never fire for ordinary spell casts — the
    single most common case. Flagged for reviewer verification.
  - **Engine changes**:
    - `state/player.rs`: +3 `PlayerState` fields — `attacked_this_turn: bool`,
      `created_token_this_turn: bool`, `spells_cast_this_game_turn: u32`.
    - `state/builder.rs`: init all 3 fields to `false`/`false`/`0`.
    - `state/hash.rs`: hash all 3 new `PlayerState` fields; `HASH_SCHEMA_VERSION`
      32→33 + changelog entry; hash arms for `TriggerCondition` disc 45
      (`AtBeginningOfFirstMainPhase`), 46 (`AtBeginningOfPostcombatMain`), 47
      (`WhenBecomesTarget`); `Condition` disc 43-47 (`YouAttackedThisTurn`,
      `CreatedATokenThisTurn`, `OpponentCastNSpells`, `SpellMastery`,
      `OpponentControlsMoreLandsThanYou`); `TriggerEvent` disc 47
      (`PermanentBecomesTarget`). Updated the `HASH_SCHEMA_VERSION, 32u8`
      sentinel to `33u8` in 24 test files (grep-and-replace).
    - `cards/card_definition.rs`: +3 `TriggerCondition` variants
      (`AtBeginningOfFirstMainPhase`, `AtBeginningOfPostcombatMain`,
      `WhenBecomesTarget { scope: Option<Box<TargetFilter>>, by_opponent,
      include_abilities }` — `scope` boxed per clippy `large_enum_variant`); +5
      `Condition` variants (`YouAttackedThisTurn`, `CreatedATokenThisTurn`,
      `OpponentCastNSpells(u32)`, `SpellMastery`,
      `OpponentControlsMoreLandsThanYou`).
    - `state/game_object.rs`: +1 `TriggerEvent::PermanentBecomesTarget { scope:
      Option<Box<TargetFilter>>, by_opponent, include_abilities }` (boxed, same
      clippy reason).
    - `rules/turn_actions.rs`: `execute_turn_based_actions` +
      `Step::PostCombatMain => Ok(postcombat_main_actions(state))` arm (was
      falling to `_ => Ok(Vec::new())`); generic CardDef sweep added to
      `precombat_main_actions` for `AtBeginningOfFirstMainPhase` (mirrors the
      B9/B14 upkeep/end-step sweep template, gated `controller == active`); new
      `postcombat_main_actions` fn with the same sweep pattern for
      `AtBeginningOfPostcombatMain` (fires on every `Step::PostCombatMain`
      including extra mains — no per-turn dedup needed, CR 505.1a); all-players
      loop in `reset_turn_state` resets the 3 new fields.
    - `rules/combat.rs`: `handle_declare_attackers` sets `attacked_this_turn =
      true` for the attacking player when `!attackers.is_empty()`, right after
      attackers are recorded in combat state. Explicitly does NOT touch the
      token-enters-attacking path (CR 508.4, Bloodsoaked Champion ruling).
    - `state/mod.rs`: `add_object` sets `created_token_this_turn = true` inside
      the existing `zone_id == Battlefield && object.is_token` block (single
      chokepoint for all 13 `TokenCreated` emission sites).
    - `rules/casting.rs`, `rules/copy.rs` (x2), `rules/resolution.rs` (x2):
      increment `spells_cast_this_game_turn` alongside `spells_cast_this_turn` at
      all 5 sites (see deviation note above).
    - `effects/mod.rs`: 5 new `check_condition` arms. `OpponentControlsMoreLandsThanYou`
      uses `calculate_characteristics` (layer-resolved, W3-LC discipline) +
      `is_phased_in()` exclusion. `SpellMastery` uses printed graveyard
      characteristics (CR 400.2), mirroring `CardTypesInGraveyardAtLeast`.
      `check_static_condition`'s existing `_ =>` fallback covers all 5 with no
      changes needed there.
    - `rules/abilities.rs`: new `collect_permanent_becomes_target_triggers` fn,
      called from the `GameEvent::PermanentTargeted` arm of `check_triggers`
      immediately after the existing Ward block. Scans all battlefield
      permanents' layer-resolved `triggered_abilities` for
      `TriggerEvent::PermanentBecomesTarget`, applies per-card
      scope/by_opponent/include_abilities gates (spell-vs-ability determined by
      looking up `targeting_stack_id` in `state.stack_objects` and checking
      `StackObjectKind::Spell`), tags `targeting_stack_id` on the pushed
      `PendingTrigger` (same convention as Ward, enables
      `DeclaredTarget{index:0}` to resolve to the targeting spell/ability).
    - `testing/replay_harness.rs`: `enrich_spec_from_def` gains one new
      conversion block for `TriggerCondition::WhenBecomesTarget` →
      `TriggeredAbilityDef { trigger_on: TriggerEvent::PermanentBecomesTarget {
      .. }, .. }`. `AtBeginningOfFirstMainPhase`/`AtBeginningOfPostcombatMain` do
      NOT get enrich blocks (fire via registry-scan sweeps, like upkeep/end-step).
  - **Tests**: new file `crates/engine/tests/pb_ac6_phase_action_conditions.rs`
    — 19 tests (hash sentinel + 3 mutation-verified hash tests for all 3 new
    fields; first-main/postcombat-main trigger firing + step-discrimination +
    active-player-only; 4 becomes-target tests covering spell-vs-ability scope,
    you-control scope, by-opponent gate, announcement-vs-resolution timing; 5
    condition tests (YouAttackedThisTurn incl. CR 508.4 negative case,
    CreatedATokenThisTurn, SpellMastery, OpponentControlsMoreLandsThanYou incl.
    phased-out exclusion, OpponentCastNSpells); 1 multiplayer turn-boundary
    reset test (4-player, verifies a NON-active player's trackers reset via
    direct `reset_turn_state` call); 1 filter-discrimination sanity check).
  - **Scope deviation (per task instructions)**: card-definition backfill
    (Searslicer Goblin, Bloodsoaked Champion, Idol of Oblivion, Dark Petition,
    Land Tax, Venerated Rotpriest, etc.) explicitly NOT done — that is a later
    phase run by a different agent. Card-integration tests from the plan's test
    list are replaced with synthetic `CardDefinition`/`TriggeredAbilityDef`
    fixtures that validate the primitives directly.
  - Gates: `cargo build --workspace` clean (0 new StackObjectKind/KeywordAbility
    variants, as predicted — TUI/replay-viewer untouched, verified not just
    assumed), `cargo test --all` 3003 passed / 0 failed (2984 baseline + 19
    new), `cargo clippy --all-targets -- -D warnings` clean (1 `large_enum_variant`
    finding self-fixed by boxing `scope` in both new enum variants),
    `cargo fmt --check` clean (ran `cargo fmt` once to apply formatter output).
- [ ] review (primitive-impl-reviewer → pb-review-AC6.md)
- [ ] fix (primitive-impl-runner)
- [ ] backfill (bulk-card-author + card-batch-reviewer)
- [ ] close
