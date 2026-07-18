# Primitive Batch Review: PB-EF8 — `Cost::ExileSelfFromHand` (activation from hand)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: scutemob-109  **Commit**: `3a5f1678`  (diff `104ef5ad..HEAD`)
**CR Rules**: 605.1a, 605.3b, 605.5, 106.12, 400.7, 118, 602.1/602.2/602.2a, 601.2h
**Engine files reviewed**: `rules/mana.rs`, `testing/replay_harness.rs`,
`effects/mod.rs`, `state/hash.rs`, `rules/protocol.rs`,
`card-types/src/cards/card_definition.rs`, `card-types/src/state/game_object.rs`,
`rules/events.rs` (consumer)
**Card defs reviewed**: 2 — `simian_spirit_guide.rs`, `elvish_spirit_guide.rs`
**Test file reviewed**: `tests/primitives/pb_ef8_exile_self_from_hand.rs` (7 tests)

## Verdict: needs-fix

The implementation is correct and faithful on every one of the nine review axes. The
stackless mana path is genuinely used (the abilities lower to `ManaAbility` and are
excluded from `activated_abilities`); the highest-risk change — relaxing the SR-34 no-tap
guard — is correctly and provably scoped to `exile_self_from_hand` alone (a dedicated
negative-control test pins it); both decoys reject on the exact error variant and are
demonstrably non-vacuous; CR 400.7 object identity, CR 106.12 replacement/trigger
suppression, and CR 605.5 priority retention are all handled correctly-by-construction and
tested; the `ObjectExiled` event mirrors the shipped pitch/embalm hand-exile shape; the
version bumps (HASH 50→51, PROTOCOL 12→13) are machine-forced and consistently applied with
no stale sentinels; all six exhaustive `Cost`/`ActivationZone` match sites are updated with
no `todo!()`/`unreachable!()` shortcuts. **Elvish Spirit Guide's prior free-infinite-{G}
bug is genuinely gone.** The only finding is one LOW: Elvish's `oracle_text` string does not
match the current Oracle wording (the *behavior* is fully correct). Verdict is "needs-fix"
solely because a LOW exists; there is **no HIGH, no MEDIUM, and no legal-but-wrong risk.**

## Engine Change Findings

None.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | LOW | `elvish_spirit_guide.rs` | **`oracle_text` stale vs current Oracle.** Field reads "Exile this **card** from your hand: Add {G}." The current Oracle (MCP `lookup_card`) is "Exile this **creature** from your hand: Add {G}." The file's own header comment (line 2) already uses the correct "creature" wording, so the comment and the `oracle_text` field disagree. `oracle_text` is documentation-only (behavior is driven by `abilities`), so game state is unaffected. **Fix:** change the `oracle_text` string at `elvish_spirit_guide.rs:22` to `"Exile this creature from your hand: Add {G}."`. (Simian's string already matches its Oracle exactly — no change there.) |

### Finding Details

#### Finding 1: Elvish Spirit Guide oracle_text uses "card" not "creature"

**Severity**: LOW
**File**: `crates/card-defs/src/defs/elvish_spirit_guide.rs:22`
**Oracle** (MCP): "Exile this creature from your hand: Add {G}."
**Issue**: The `oracle_text` field says "…this card…"; the current Oracle says "…this
creature…". The header comment on line 2 uses the correct "creature" form, so the file is
internally inconsistent. No behavioral impact — the ability is `Cost::ExileSelfFromHand` +
`Effect::AddMana { green: 1 }` regardless of the descriptive string. The plan (§Card
Definition Fixes) explicitly flagged this as "a note, not a blocker" and preferred the
Oracle "creature" wording.
**Fix**: Set `oracle_text: "Exile this creature from your hand: Add {G}.".to_string(),`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1a (no target + could add mana ⇒ mana ability) | Yes | Yes | `pb_ef8_lowering_gate_is_not_vacuous` — bare `ExileSelfFromHand` lowers to a `ManaAbility`, excluded from `activated_abilities` |
| 605.3b (mana ability doesn't use the stack) | Yes | Yes | happy path asserts `state.stack_objects().is_empty()` |
| 605.5 (priority/players_passed not reset) | Yes | Yes | `from_hand_mana_ability_does_not_reset_priority_or_players_passed` seeds `players_passed={p(2)}`, asserts unchanged + `priority_holder==Some(p(1))`. (Citation nuance below.) |
| 106.12 (tap-for-mana ⇒ replacements/triggers) | Yes (by construction) | Yes | steps 7b/10 in `mana.rs` gated on `requires_tap` (false); `from_hand_mana_ability_does_not_fire_tapped_for_mana_replacements` asserts `requires_tap==false` |
| 400.7 (new object after zone change) | Yes | Yes | happy path asserts `state.objects().get(&source).is_none()` and exactly one new named object in `ZoneId::Exile` |
| 118 / 601.2h (exile paid as a cost before mana) | Yes | Yes | step 7c exiles before step 8 produces mana; `ObjectExiled` event emitted (asserted) |
| 602.1/602.2 (owner activates a hand card) | Yes | — | from-hand branch checks `obj.owner == player` (defensive; a `Hand(player)` object is owner-owned by CR 400.3) |
| 602.2a (reveal from hidden zone) | Not modeled (accepted) | — | scoped out in plan; the exile makes the card public, so **no wrong/hidden state results** — informational only |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `simian_spirit_guide.rs` | Yes (exact) | 0 | Yes | inert→Complete; `Cost::ExileSelfFromHand` + `activation_zone: Some(Hand)`, `Add {R}`, marker dropped |
| `elvish_spirit_guide.rs` | **No (LOW-1)** | 0 | Yes | known_wrong→Complete; **prior free-infinite-{G} battlefield ability removed**; `Add {G}`; only the descriptive `oracle_text` string is stale |

## Per-axis verification (task's 9 points)

1. **Stackless path** — CONFIRMED. `mana_ability_lowering` (`replay_harness.rs:3805`) lowers
   both cards to `ManaAbility` and the same predicate excludes them from
   `activated_abilities` (`replay_harness.rs:2130`/`2154`). The mana-lowering loop
   destructures with `..` and does NOT read `activation_zone`, so `Some(Hand)` does not
   suppress lowering (design decision #3 holds). `handle_tap_for_mana` handles the from-hand
   case. `Command::ActivateAbility` never sees these abilities.
2. **No-tap-guard relaxation scope** — CONFIRMED SAFE. `replay_harness.rs:3779`:
   `if !acc.requires_tap && !acc.exile_self_from_hand { return None; }`. A `Cost::Mana`-only
   or `SacrificeSelf`-only no-tap cost (Food Chain) still returns `None`. The negative-control
   test (`Cost::Sequence([DiscardCard, ExileSelfFromHand])` does not lower) additionally
   proves the relaxation does not widen to arbitrary no-tap costs. No path can register a
   free *repeatable* stackless mana ability — the exile is self-consuming (CR 400.7).
3. **Zone legality both directions, exact error variant** — CONFIRMED. From-hand branch
   rejects a battlefield source with `InvalidCommand("…from hand…")` (decoy A asserts the
   variant + message substring); battlefield branch rejects a hand source with
   `ObjectNotOnBattlefield(source)` (decoy B asserts `id == source`). Both decoys are
   non-vacuous (documented delete-the-check experiments; priority is held by p(1) so the
   rejection is genuinely on the zone check, not priority/index).
4. **CR 400.7 object identity** — CONFIRMED. `move_object_to_zone(source, Exile)` retires
   `source`; happy path asserts the dead id and a distinct new exile object.
5. **CR 106.12** — CONFIRMED correct-by-construction. Both the mana-production replacement
   block (`mana.rs:372`) and the `WhenTappedForMana` trigger block (`mana.rs:455`) are gated
   on `ability.requires_tap`, which is `false` for these abilities. Nyxbloom/Mana Reflection
   do not multiply.
6. **`ObjectExiled` fields** — CONFIRMED. `mana.rs:360` uses `player`, `object_id: source`,
   `new_exile_id`, `pre_lba_counters: OrdMap::new()`, `pre_lba_power: None` — identical to
   the shipped hand/graveyard-exile shape (embalm `abilities.rs:2384`). The
   `WhenSourceLeavesBattlefield` consumer (`abilities.rs:6139`) is benign: a hand card's
   ObjectId was never a battlefield permanent, so no delayed trigger references it.
7. **Both card defs** — CONFIRMED Complete and behaviorally faithful (see table). Elvish's
   free-infinite-{G} bug (`Cost::Mana(default)`, battlefield) is gone. Only LOW-1 remains.
8. **Version bumps** — CONFIRMED. `HASH_SCHEMA_VERSION = 51` (changelog cites PB-EF8; both
   new bool fields hashed at `hash.rs:1591` and `:2856`); `PROTOCOL_VERSION = 13`. No stale
   `…, 50` / `…, 12` sentinels remain (grep clean). Both new fields carry `#[serde(default)]`.
9. **Exhaustive-match completeness** — CONFIRMED. All six sites updated:
   `mana_ability_cost_components` (accepting arm), `flatten_cost_into` (arm),
   `can_pay_optional_cost` (`=> false`), `pay_optional_cost` (`=> {}`), `Cost` HashInto
   (`=> 13u8`), `ActivationZone` HashInto (`Hand => 1u8`). No other exhaustive `Cost` or
   `ActivationZone` match exists (`abilities.rs:189` uses non-exhaustive `if let
   Some(Graveyard)`). No `todo!()`/`unreachable!()`/`unwrap()` introduced in `mana.rs`.

## Notes (informational, not findings)

- **CR 605.5 citation nuance.** The priority-retention behavior is cited as CR 605.5 in the
  new test and throughout `mana.rs`. CR 605.5 (verified via MCP) actually states which
  abilities *aren't* mana abilities; the "doesn't use the stack / resolves immediately"
  property is CR 605.3b, and priority is governed by the priority rules (CR 117 / 605.3a).
  This citation choice is **inherited from the pre-existing `handle_tap_for_mana` docstring**
  (not introduced by PB-EF8), so it is out of scope to re-cite here — flagged only so it is
  not mistaken for a new defect.
- **CR 602.2a reveal-from-hidden-zone** is not modeled as a distinct event. Because the card
  is exiled (public zone) with an `ObjectExiled` event, no hidden information is retained and
  no wrong game state results. Explicitly scoped out in the plan; acceptable.
- **`obj.owner != player` in the from-hand branch** is defensively redundant (a
  `Hand(player)` object is owner-owned per CR 400.3) but harmless.

## Legal-but-wrong assessment

**No legal-but-wrong risk.** The two cards produce correct multiplayer game state: they
activate only from hand (decoy A pins battlefield rejection), exile the source as the cost
(one-shot, self-consuming — cannot be re-activated because the exiled object is in Exile,
not Hand), produce exactly one mana of the correct color, resolve stacklessly, and do not
reset priority. The prior infinite-mana bug on Elvish is eliminated. The residual gaps
(reveal event, mid-cost-payment activation without formal priority) are pre-existing/scoped
and produce no incorrect state for these cards.
