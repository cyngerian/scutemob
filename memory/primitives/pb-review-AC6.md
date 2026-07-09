# Primitive Batch Review: PB-AC6 — Phase & opponent-action conditions

**Date**: 2026-07-09
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit reviewed**: `83c6c7d3` (vs parent `83c6c7d3~1`)
**CR Rules (re-verified via mtg-rules MCP)**: 505.1a, 601.2c, 602.2b, 603.2/603.2c,
207.2c, 508.1/508.4, 111.10, 702.21a (Ward precedent)
**Engine files reviewed**: `state/player.rs`, `state/builder.rs`, `state/hash.rs`,
`state/mod.rs`, `state/game_object.rs`, `cards/card_definition.rs`,
`rules/turn_actions.rs`, `rules/combat.rs`, `rules/abilities.rs`, `rules/casting.rs`,
`rules/copy.rs`, `rules/resolution.rs`, `effects/mod.rs`, `testing/replay_harness.rs`
**Card defs reviewed**: none changed in this batch (backfill is an explicit separate
close-phase task per the wip deviation note; card semantics cross-checked against
oracle text anyway — Idol of Oblivion, Land Tax)
**Tests reviewed**: `crates/engine/tests/pb_ac6_phase_action_conditions.rs` (19 tests)

## Verdict: needs-fix (LOW-only)

The engine implementation is correct on every HIGH-probability focus area. All three
new `PlayerState` fields are hashed, reset for all players, and covered by genuine
single-state-mutation hash tests; the `spells_cast_this_game_turn` increment set
exactly mirrors `spells_cast_this_turn` (5 sites, all paired, none spurious); the
main-phase sweeps are generic registry scans; `WhenBecomesTarget` fires at
announcement above the targeting spell with correct spell-vs-ability / by-opponent /
scope gates; and the multiplayer land/opponent conditions use layer-resolved types
and `is_phased_in()`. CR citations in code all check out against the MCP. **Zero HIGH,
zero MEDIUM.** Two LOW observations below are latent (no roster card exercises them)
and both mirror pre-existing systemic patterns; neither blocks collection. This batch
is safe to ship; the LOWs can be addressed opportunistically.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `testing/replay_harness.rs:3235` | **`intervening_if` dropped in `WhenBecomesTarget` enrich conversion.** Explanation below. **Fix:** documentation-only; see detail. |
| 2 | LOW | `effects/mod.rs:8104-8116` | **`SpellMastery` counts objects, not strictly cards, in the graveyard.** Explanation below. **Fix:** optionally add `o.card_id.is_some()` (or `!o.is_token`) guard for defense-in-depth. |

## Card Definition Findings

None — no card defs were modified in this batch.

### Finding Details

#### Finding 1: `intervening_if` dropped in `WhenBecomesTarget` enrich conversion

**Severity**: LOW (latent; systemic pre-existing pattern)
**File**: `crates/engine/src/testing/replay_harness.rs:3213-3245` (`intervening_if: None`
at 3235)
**Issue**: The `enrich_spec_from_def` block that converts
`AbilityDefinition::Triggered { trigger_condition: WhenBecomesTarget {..}, .. }` into a
`TriggeredAbilityDef` ignores the source ability's `intervening_if` (captured by `..`)
and hardcodes `intervening_if: None`. The becomes-target dispatch
(`abilities.rs:6250`) *does* check `trigger_def.intervening_if` at trigger time (CR
603.4), so that check is effectively dead for any enriched card — a hypothetical
"Whenever ~ becomes the target of a spell, if <condition>, ..." card would not enforce
its condition.
**Why LOW, not MEDIUM**: (a) Every one of the ~34 sibling event-driven enrich blocks
in this file already sets `intervening_if: None` (see the comment at line 2259: the
CardDef `Condition` type and the runtime `InterveningIf` type are distinct, so a
straight carry-through isn't possible). This is not a becomes-target-specific
regression. (b) No roster card for this batch (Venerated Rotpriest, Goldspan Dragon,
Bonecrusher Giant, Scalelord Reckoner, Tectonic Giant) uses an intervening-if on its
becomes-target trigger, so there is zero near-term impact. (c) The first-main /
postcombat sweeps push `CardDefETB` triggers, whose intervening-if IS re-checked from
the registry at resolution (CR 603.4), so those conditions (e.g. Land Tax-style, if
ever authored on a main-phase trigger) work correctly.
**Fix**: No code change required for correctness now. Optional: add a short comment at
`abilities.rs:6250` noting the enriched path always supplies `intervening_if: None`, or
file an OOS seed to unify CardDef-`Condition`→runtime-`InterveningIf` conversion across
all event-driven triggers. Do not special-case only becomes-target.

#### Finding 2: `SpellMastery` filters by zone+card_type without a card guard

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:8104-8116`
**CR**: 207.2c — "two or more instant and/or sorcery **cards** in your graveyard."
**Issue**: The filter is `o.zone == Graveyard(controller) && (types.contains(Instant)
|| types.contains(Sorcery))` and counts all matching objects. It does not exclude
non-card objects (tokens). In practice a token instant/sorcery in a graveyard is
removed by SBA (CR 111.7) before the condition is meaningfully evaluated, and copies
never reach the graveyard, so the count is correct in all realistic cases. This
mirrors the existing `CardTypesInGraveyardAtLeast` pattern.
**Fix**: Optional defense-in-depth — add `&& o.card_id.is_some()` (or `&& !o.is_token`)
to the closure. Not required for correctness.

## Focus-Area Verification (all confirmed)

1. **Hash completeness** — GREEN. All three new `PlayerState` fields are in
   `PlayerState::hash_into` (`hash.rs:1438/1440/1442`). Every new enum variant with
   inner data hashes its fields: `TriggerCondition::WhenBecomesTarget` (disc 47,
   hash.rs:5108-5117), `TriggerEvent::PermanentBecomesTarget` (disc 47,
   hash.rs:2548-2557), `Condition::OpponentCastNSpells(n)` (disc 45, hash.rs:5299-5302).
   No other struct fields were added anywhere in the diff (enumerated: only
   `PlayerState` gained fields; `GameObject`/`StackObject`/`GameState`/`CastSpell`
   untouched). The three hash tests (H1/H2/H3, test file lines 184-247) each build ONE
   state, snapshot `public_state_hash()`, mutate exactly one field on that same state,
   and `assert_ne!` — genuine single-state mutation, not two independently-built
   states. `HASH_SCHEMA_VERSION = 33` (hash.rs:274); zero `32u8` sentinels remain
   anywhere in `crates/` (grep confirmed all sentinel test files carry `33u8`).

2. **Turn-boundary reset** — GREEN. All three trackers reset in the all-players loop
   of `reset_turn_state` (turn_actions.rs:1707/1710/1714), alongside
   `cards_drawn_this_turn`/`life_lost_this_turn`. `reset_turn_state` is the only reset
   path; it is called on every turn start (engine.rs:1593/1733/1811/2228 — normal
   advance, extra turns, setup). The multiplayer test
   (`test_all_players_trackers_reset_at_turn_boundary_multiplayer`) sets all four
   players' trackers then calls `reset_turn_state(&mut state, p3)` and asserts p1/p2/p4
   (non-active relative to p3) all reset — genuinely exercises non-active players.

3. **`spells_cast_this_game_turn` increment-site completeness** — GREEN. Independent
   grep found `spells_cast_this_turn` is incremented at exactly 5 sites:
   `casting.rs:4709` (primary normal cast), `copy.rs:462` (cascade free-cast),
   `copy.rs:690` (discover free-cast), `resolution.rs:5133` (cipher copy — a genuine
   cast per ruling 2013-04-15), `resolution.rs:5789` (suspend cast, CR 702.62a). Every
   one has a paired `spells_cast_this_game_turn.saturating_add(1)` (4712/464/692/5135/
   5791), and `spells_cast_this_game_turn` is incremented at *only* those 5 sites — no
   more, no fewer. The runner's flagged 5th site (`casting.rs:4709`, uncited by the
   plan) is correct and essential. Storm/replicate copy paths (CR 706.10 — a copy put
   on the stack is not cast) correctly do NOT increment either field.

4. **`WhenBecomesTarget` timing & semantics** — GREEN. Dispatched from the
   `GameEvent::PermanentTargeted` arm (abilities.rs:4188), which is emitted at target
   announcement (casting.rs after `SpellCast`; abilities.rs:1675 after
   `AbilityActivated`) — not resolution. Test
   `test_becomes_target_fires_at_announcement_not_resolution` asserts the trigger sits
   ABOVE the spell on the stack before resolution (CR 601.2c verified verbatim via MCP:
   triggers "wait to be put on the stack until the spell has finished being cast").
   Spell-vs-ability detection looks up `targeting_stack_id` in `state.stack_objects`
   and matches `StackObjectKind::Spell`, with `unwrap_or(false)` — the safe fallback
   (a missing lookup treats it as not-a-spell, so a spell-only trigger conservatively
   does not fire; at announcement the object is still on the stack so the lookup
   succeeds). `by_opponent` and `scope`/`include_abilities` gates all correct
   (abilities.rs:6219-6248), using layer-resolved characteristics for both the source's
   triggered abilities and the scoped target filter. The multi-slot double-fire
   (same permanent in two "target" slots → two triggers) is CR-CORRECT per 601.2c
   ("if the spell uses the word 'target' in multiple places, the same object can be
   chosen once for each instance") and consistent with Ward — not a bug.

5. **Main-phase sweeps** — GREEN. `AtBeginningOfFirstMainPhase` fires from
   `precombat_main_actions` (turn_actions.rs:551-597), reached only on
   `Step::PreCombatMain` which occurs once per turn (CR 505.1a verified). 
   `AtBeginningOfPostcombatMain` fires from the new `postcombat_main_actions`
   (turn_actions.rs:610-661) on every `Step::PostCombatMain` including effect-created
   extra mains (CR 505.1a verified — all additional main phases are postcombat mains,
   all represented as `Step::PostCombatMain`). Both are generic CardDef registry scans
   (identical pattern to `carddef_upkeep_triggers`, the MR-B9-01 lesson), gated
   `controller == active` and `is_phased_in()`. `Step::PostCombatMain` was correctly
   promoted to an explicit dispatch arm (turn_actions.rs:31); the `_ => Ok(Vec::new())`
   fallthrough remains intact for all other steps. Tests confirm first-main fires once,
   postcombat fires on postcombat but NOT precombat, and a non-active player's
   first-main trigger does not fire on the active player's main.

6. **`OpponentControlsMoreLandsThanYou`** — GREEN. Uses
   `calculate_characteristics(...).card_types.contains(Land)` (NOT raw
   `obj.characteristics`), excludes phased-out permanents via `is_phased_in()`, counts
   any living opponent (`!ps.has_lost`), and is strictly greater (`count_lands(pid) >
   mine`). Oracle wording matched: Land Tax reads "if an opponent controls more lands
   than you" (verified via MCP). Test covers equal→false, +1→true, phased-out→excluded.

7. **`SpellMastery`** — GREEN (with LOW #2). Ability word confirmed in CR 207.2c's list
   (verified via MCP). Uses printed graveyard characteristics (CR 400.2, no layer calc),
   `Instant OR Sorcery` union (not intersection), count ≥ 2. Test covers 0/1→false,
   1 instant + 1 sorcery→true (union), 2 creatures→false.

8. **`YouAttackedThisTurn`** — GREEN. Set in `handle_declare_attackers`
   (combat.rs:561-565) only when `!attackers.is_empty()`, for the attacking player
   only, after attackers are committed. Token-enters-attacking does NOT set it (CR 508.4
   verified via MCP: such creatures "never 'attacked'"); test
   `test_token_entering_attacking_does_not_set_attacked_this_turn` confirms via a
   `enters_attacking: true` token. Declaring zero attackers correctly leaves it false.

9. **`created_token_this_turn`** — GREEN. Set at the single `add_object` chokepoint
   (state/mod.rs:361-368) inside the `Battlefield && is_token` block, controller-scoped.
   Does not false-positive on token zone-moves back to battlefield or merged-permanent
   splits (both route through `move_object_to_zone`, not `add_object`). A token *copy*
   of an existing permanent does set the flag — correct (creating a token copy is
   creating a token). Oracle confirmed: Idol of Oblivion reads "Activate only if you
   created a token this turn" (verified via MCP), matching the condition exactly.

10. **`Box`ing for clippy `large_enum_variant`** — GREEN. `scope:
    Option<Box<TargetFilter>>` on both `TriggerCondition::WhenBecomesTarget` and
    `TriggerEvent::PermanentBecomesTarget`. `Box<T>` serde-roundtrips transparently
    (serialized identically to `T`); `#[serde(default)]` on the CardDef variant fields
    handles pre-bump deserialization. The enrich conversion clones cleanly
    (`scope: scope.clone()`, replay_harness.rs:3231). Hash impls hash through the box.

**Regression checks** — GREEN. Ward is untouched: becomes-target dispatch is a
separate scan appended AFTER the Ward block (abilities.rs:4181-4194) using the distinct
`PermanentBecomesTarget` event; Ward keeps using `SelfBecomesTargetByOpponent`.
Storm/Daybound untouched: `spells_cast_this_turn`, `storm_count`, and
`previous_turn_spells_cast` are not modified by this diff (only reads + the new
independent `spells_cast_this_game_turn` field added). No `.unwrap()`/`.expect()`
introduced into engine logic — all new code uses `unwrap_or`/`unwrap_or_else`/
`unwrap_or(false)`/`if let`. Tests cite CR sections throughout.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 505.1a (first vs postcombat main) | Yes | Yes | first-main-once, postcombat-not-precombat tests |
| 601.2c (target announcement timing) | Yes | Yes | announcement-not-resolution test |
| 602.2b (ability target = spell process) | Yes | Yes | include_abilities:false ability-case test |
| 603.2c (same target multiple slots) | Yes (fires per slot) | — | CR-correct; consistent w/ Ward |
| 207.2c (spell mastery ability word) | Yes | Yes | union + card-type discrimination tests |
| 508.1 (Raid / you attacked) | Yes | Yes | declare-attackers sets flag |
| 508.4 (put onto bf attacking ≠ attacked) | Yes | Yes | token-enters-attacking negative test |
| 111.10 (created a token) | Yes | Yes | add_object chokepoint test |
| 702.21a (opponent gate precedent) | Yes | Yes | by_opponent gate test |

## Card Def Summary

No card defs changed in this batch (backfill deferred to the close phase per the wip
deviation note). Primitive shapes cross-checked against oracle text for their intended
consumers: Idol of Oblivion ("Activate only if you created a token this turn") and
Land Tax ("if an opponent controls more lands than you") both match their conditions
exactly. Backfill review (Searslicer Goblin, Bloodsoaked Champion, Dark Petition,
Venerated Rotpriest, etc.) should occur when those defs are authored.
