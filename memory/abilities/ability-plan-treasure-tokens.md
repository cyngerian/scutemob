# Ability Plan: Treasure Tokens

**Generated**: 2026-03-08 (updated; original plan 2026-02-26)
**CR**: 111.10a
**Priority**: P2
**Status in coverage doc**: `validated` (since 2026-02-26)
**Similar abilities studied**: Food tokens (`tests/food_tokens.rs`), Clue tokens (`tests/clue_tokens.rs`)

## CR Rule Text

### CR 111.10a
> A Treasure token is a colorless Treasure artifact token with "{T}, Sacrifice this token: Add
> one mana of any color."

### CR 605.1a (Mana Ability classification)
> An activated ability is a mana ability if it meets all of the following criteria: it doesn't
> require a target (see rule 115.6), it could add mana to a player's mana pool when it resolves,
> and it's not a loyalty ability.

### CR 605.3b (Mana abilities resolve immediately)
> An activated mana ability doesn't go on the stack, so it can't be targeted, countered, or
> otherwise responded to. Rather, it resolves immediately after it is activated.

### CR 602.2c
> Sacrifice is a cost paid before the ability resolves.

### CR 704.5d / CR 111.7
> Tokens in non-battlefield zones cease to exist as a state-based action. (Note that if a token
> changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

## Key Edge Cases

1. **Treasure's ability IS a mana ability (CR 605.1a)**: No target, produces mana, not a loyalty
   ability. Does NOT use the stack. Resolves immediately. Cannot be countered or responded to.
2. **Summoning sickness does NOT prevent activation** (CR 302.6): Treasure is an artifact, not a
   creature. The `handle_tap_for_mana` handler correctly checks `card_types.contains(&Creature)`
   before applying the summoning sickness restriction.
3. **Token ceases to exist after sacrifice** (CR 704.5d): After sacrifice (moved to graveyard),
   the token briefly exists in graveyard (triggers can fire), then SBA removes it.
4. **Multiple Treasures can be activated in sequence** (CR 605.3a): Each is a separate mana
   ability that resolves immediately.
5. **Only controller can activate** (CR 602.2): Validated in tests.
6. **`any_color` adds colorless** (simplified): Consistent with `Effect::AddManaAnyColor`
   throughout the engine. Interactive color choice is a future cross-cutting feature.

## Current State: FULLY IMPLEMENTED AND VALIDATED

All steps are complete. Treasure tokens have been `validated` in the ability coverage doc since
2026-02-26. No further implementation work is needed.

### What Exists

| Component | Location | Description |
|-----------|----------|-------------|
| `ManaAbility::treasure()` | `state/game_object.rs:80-87` | Constructor: `sacrifice_self: true`, `any_color: true`, `requires_tap: true` |
| `treasure_token_spec()` | `cards/card_definition.rs:1465-1480` | Helper: creates TokenSpec with type Artifact, subtype Treasure, 1 mana ability |
| Sacrifice handling | `rules/mana.rs:95-122` | `handle_tap_for_mana` step 7: moves object to graveyard as cost |
| Any-color handling | `rules/mana.rs:124-136` | `handle_tap_for_mana` step 8: adds 1 colorless mana |
| Token creation | `effects/mod.rs:2916-2933` | `make_token` propagates `mana_abilities` from TokenSpec to GameObject |
| Hash support | `state/hash.rs:788` | `HashInto` impl for `ManaAbility` (includes `sacrifice_self`, `any_color`) |
| Re-exports | `lib.rs:9`, `cards/mod.rs:17`, `cards/helpers.rs:16` | `treasure_token_spec` available to card defs and tests |

### Card Definitions Using Treasure Tokens

| Card | File | How |
|------|------|-----|
| Strike It Rich | `defs/strike_it_rich.rs` | Spell effect: `CreateToken { spec: treasure_token_spec(1) }` |
| Prosperous Innkeeper | `defs/prosperous_innkeeper.rs` | ETB trigger: `CreateToken { spec: treasure_token_spec(1) }` |
| Riveteers Requisitioner | `defs/riveteers_requisitioner.rs` | Dies trigger: `CreateToken { spec: treasure_token_spec(1) }` |
| Gift system | `rules/resolution.rs:6575-6587` | `GiftType::Treasure`: creates Treasure token for recipient |

### Unit Tests (9 tests in `tests/treasure_tokens.rs`)

| Test | CR | What it validates |
|------|-----|-------------------|
| `test_treasure_token_spec_characteristics` | 111.10a | Spec correctness: colorless, artifact, Treasure subtype, mana ability flags |
| `test_treasure_token_has_mana_ability` | 111.10a | Battlefield presence, mana ability populated on GameObject |
| `test_treasure_sacrifice_for_mana` | 605.1a, 111.10a | TapForMana: sacrifice + mana production + correct events |
| `test_treasure_mana_resolves_immediately_no_stack` | 605.3b | Stack empty after activation, priority retained |
| `test_treasure_sacrifice_multiple_in_sequence` | 605.3a | 3 Treasures sacrificed in sequence = 3 colorless mana |
| `test_treasure_already_tapped_cannot_activate` | 602.2b | Tapped Treasure returns PermanentAlreadyTapped error |
| `test_treasure_not_affected_by_summoning_sickness` | 302.6 | Artifact not creature, activation succeeds |
| `test_treasure_token_ceases_to_exist_after_sba` | 704.5d | Token removed from graveyard by SBA |
| `test_treasure_cannot_be_activated_by_opponent` | 602.2 | Non-controller gets NotController error |

### Game Script

- `test-data/generated-scripts/stack/073_*` (Strike It Rich) -- `validated`

### Harness Support

- `tap_for_mana` action in `replay_harness.rs:630-637` sends `Command::TapForMana` with
  `ability_index: 0`. Works for Treasure tokens.

## Steps to Close

- [x] Step 1: Enum variant / type extensions -- `ManaAbility` has `sacrifice_self` + `any_color`
- [x] Step 2: Rule enforcement -- `handle_tap_for_mana` handles sacrifice + any-color
- [x] Step 3: Trigger wiring -- N/A (mana abilities don't use triggers)
- [x] Step 4: Unit tests -- 9 tests in `treasure_tokens.rs`
- [x] Step 5: Card definition -- Strike It Rich, Prosperous Innkeeper
- [x] Step 6: Game script -- stack/073
- [x] Step 7: Coverage doc update -- `validated` in coverage doc

## Known Limitations (not blocking, documented)

1. **`any_color` defaults to colorless**: Like `Effect::AddManaAnyColor`, the player doesn't
   interactively choose a color. Cross-cutting concern, not Treasure-specific.
2. **Trigger dispatch from mana ability side effects**: `handle_tap_for_mana` does not call
   `check_triggers()`. Abilities that trigger on "artifact goes to graveyard" won't fire from
   Treasure sacrifice via mana ability. Broader infra gap, not Treasure-specific.
3. **Collector Ouphe / Null Rod**: Static effects preventing artifact activated abilities
   (including mana abilities) are not yet implemented. Future work.

## Conclusion

**No implementation work is needed.** Update `ability-wip.md` to phase=done, verdict=pass.
