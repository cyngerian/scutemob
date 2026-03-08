# Ability Plan: Decayed Tokens

**Generated**: 2026-03-08
**CR**: 702.147 (Decayed keyword — already implemented)
**Priority**: P4 (Batch 14, item 14.6)
**Similar abilities studied**: Decayed keyword (B1, validated), Myriad EOC exile pattern, predefined token specs (Treasure, Food, Clue, Blood, Army)

## CR Rule Text

702.147. Decayed

702.147a Decayed represents a static ability and a triggered ability. "Decayed" means "This creature can't block" and "When this creature attacks, sacrifice it at end of combat."

Also relevant:
- CR 122.1b: Decayed is one of the keyword counter types (flying, first strike, double strike, deathtouch, **decayed**, exalted, haste, hexproof, indestructible, lifelink, menace, reach, shadow, trample, vigilance).

## Key Edge Cases

- **Decayed is already fully enforced.** The keyword, can't-block restriction, and EOC sacrifice are all implemented and validated with 8 tests in `decayed.rs`.
- **TokenSpec already supports keywords.** `TokenSpec.keywords: OrdSet<KeywordAbility>` means any token created with `KeywordAbility::Decayed` in its spec will have the keyword, and the existing combat.rs / turn_actions.rs enforcement will apply automatically.
- **No new engine infrastructure is needed.** This is purely a verification task: confirm that a token created via `Effect::CreateToken` with `Decayed` in its keywords is correctly enforced (can't block, sacrificed at EOC when it attacks).
- **Summoning sickness applies** (ruling 2021-09-24): Decayed tokens that enter during your turn can't attack until your next turn (no haste).
- **Keyword counter interaction** (CR 122.1b): A decayed keyword counter on any permanent grants Decayed. This is a separate infrastructure concern (keyword counters) not specific to Decayed tokens.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- `KeywordAbility::Decayed` at `state/types.rs:688` (disc 74 in hash.rs)
- [x] Step 2: Rule enforcement -- can't block in `combat.rs:546+849`, EOC sacrifice in `turn_actions.rs:1541-1562`, flag set in `combat.rs:433-441`
- [x] Step 3: Trigger wiring -- EOC flag pattern (decayed_sacrifice_at_eoc on GameObject), not a trigger dispatch
- [x] Step 4: Unit tests -- 8 tests in `crates/engine/tests/decayed.rs` (all passing)
- [ ] Step 5: Token-specific verification tests
- [ ] Step 6: Card definition (card that creates Decayed tokens)
- [ ] Step 7: Game script

## Implementation Steps

### Step 1: Enum Variant -- DONE

`KeywordAbility::Decayed` exists at `crates/engine/src/state/types.rs:688`, discriminant 74 in `hash.rs:486`. No new variants needed.

### Step 2: Rule Enforcement -- DONE

All enforcement is in place:
- **Can't block**: `crates/engine/src/rules/combat.rs:546` (DeclareBlockers validation) and `combat.rs:849` (Provoke override)
- **EOC sacrifice flag**: `crates/engine/src/rules/combat.rs:433-441` (set on DeclareAttackers)
- **EOC sacrifice execution**: `crates/engine/src/rules/turn_actions.rs:1541-1600`
- **Flag on GameObject**: `crates/engine/src/state/game_object.rs:461` (`decayed_sacrifice_at_eoc: bool`)
- **Flag reset on zone change**: `crates/engine/src/state/mod.rs:341,518`
- **Flag init in token creation**: `crates/engine/src/effects/mod.rs:2967`
- **Hash**: `crates/engine/src/state/hash.rs:851`

### Step 3: Trigger Wiring -- DONE (N/A for tokens specifically)

The Decayed keyword uses the EOC flag pattern, not trigger dispatch. Tokens inherit this behavior automatically because `make_token()` copies keywords from `TokenSpec.keywords` into the token's characteristics.

### Step 4: Token-Specific Unit Tests

**File**: `crates/engine/tests/decayed.rs` (append to existing file)
**Tests to write**:

1. `test_702_147_decayed_token_created_with_keyword` -- Create a token via `Effect::CreateToken` with a `TokenSpec` that includes `KeywordAbility::Decayed`. Verify the token on the battlefield has the Decayed keyword in its characteristics.

2. `test_702_147_decayed_token_cannot_block` -- Create a Decayed Zombie token, attempt to declare it as a blocker, verify the command is rejected. (Same pattern as test 1 but with a token instead of a manually-placed creature.)

3. `test_702_147_decayed_token_sacrificed_at_eoc` -- Create a Decayed Zombie token (without summoning sickness), declare it as an attacker, advance through combat, verify it's sacrificed at EOC. This is the critical end-to-end test for the "Decayed Tokens" feature.

4. `test_702_147_decayed_token_has_summoning_sickness` -- Create a Decayed token with `has_summoning_sickness: true`, attempt to declare it as an attacker, verify the command is rejected. Validates ruling 2021-09-24.

**Pattern**: Follow existing tests 1-8 in `decayed.rs`. The key difference is that tokens are created via `Effect::CreateToken` with a `TokenSpec` rather than `ObjectSpec::creature().with_keyword()`. For test setup, either:
- (a) Use the effect execution system to create the token mid-test, or
- (b) Use `ObjectSpec::creature().with_keyword(Decayed)` and set `is_token: true` (simpler, equivalent for enforcement testing).

**Recommendation**: Option (b) is simpler and equally valid for verifying enforcement. The `make_token()` path is already covered by verifying that `TokenSpec.keywords` propagates to `Characteristics.keywords` (test 1 can do this directly). The enforcement tests (2-4) can use ObjectSpec since the engine doesn't distinguish tokens from non-tokens for Decayed enforcement -- it only checks the keyword.

However, test 1 should specifically test the `make_token()` function to verify `TokenSpec` -> token keyword propagation, since that's the actual "Decayed Tokens" feature being validated.

### Step 5: Card Definition

**Suggested card**: Jadar, Ghoulcaller of Nephalia
- {1}{B}, Legendary Creature -- Human Wizard, 1/1
- "At the beginning of your end step, if you control no creatures with decayed, create a 2/2 black Zombie creature token with decayed."
- Simple triggered ability with an intervening-if condition
- Creates a single Decayed token -- perfect for testing the feature

**Alternative**: Wilhelt, the Rotcleaver ({2}{U}{B}, 3/3) -- "Whenever another Zombie you control dies, if it didn't have decayed, create a 2/2 black Zombie creature token with decayed." More complex (die trigger with filter), but tests a common Decayed token creation pattern.

**Recommendation**: Jadar is simpler and more targeted. Wilhelt requires die-trigger filtering infrastructure that may not be fully supported yet. Use Jadar.

**DSL feasibility check**: Jadar needs:
- `TriggerCondition::AtBeginningOfYourEndStep` -- check if this exists
- Intervening-if: "if you control no creatures with decayed" -- this is a custom condition not yet in the DSL (`Condition::YouControlNoCreaturesWith(KeywordAbility)` or similar). May need a new `Condition` variant.
- Effect: `Effect::CreateToken { spec: zombie_decayed_token_spec() }`

**DSL gap**: The intervening-if condition "you control no creatures with decayed" likely requires a new `Condition` variant. If this is too complex for this batch item, the card definition can omit the intervening-if check (documented as a simplification) or use a different card.

**Simpler card alternative**: A card with a simpler "create a Zombie token with decayed" trigger that doesn't need an intervening-if. Options:
- Ghoulish Procession ({1}{B}, Enchantment): "Whenever one or more creatures you control die, create a 2/2 black Zombie creature token with decayed. This ability triggers only once each turn." -- still complex (once-per-turn trigger).
- None of the Decayed token creators are truly simple -- they all have conditions or filters.

**Final recommendation**: Author Jadar with a TODO noting the intervening-if gap, or simply write a test-only card definition that creates a Decayed token on ETB (for testing purposes). The card-definition-author agent can handle either approach.

### Step 6: Token Spec Helper (optional)

**File**: `crates/engine/src/cards/card_definition.rs` (near `treasure_token_spec`, `food_token_spec`, etc.)
**Action**: Add a `zombie_decayed_token_spec()` helper function.

```
pub fn zombie_decayed_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Zombie".to_string(),
        power: 2,
        toughness: 2,
        colors: ordset![Color::Black],
        card_types: ordset![CardType::Creature],
        subtypes: ordset![SubType::Zombie],
        keywords: ordset![KeywordAbility::Decayed],
        count,
        ..Default::default()
    }
}
```

**Rationale**: Multiple MID/VOW cards create "2/2 black Zombie creature token with decayed" (Jadar, Wilhelt, Ghoulish Procession, Tainted Adversary, etc.). A shared helper prevents duplication, following the pattern of `treasure_token_spec()`, `food_token_spec()`, `clue_token_spec()`, `blood_token_spec()`, and `army_token_spec()`.

**Note**: This is optional. If only one card needs it, inline the `TokenSpec` in the card definition. If multiple cards will use it (likely), add the helper.

### Step 7: Game Script (later phase)

**Suggested scenario**: A creature with Decayed enters the battlefield as a token, attacks next turn, deals combat damage, and is sacrificed at EOC.
**Subsystem directory**: `test-data/generated-scripts/combat/` (Decayed sacrifice is a combat mechanic)
**Note**: Requires the card definition from Step 5 to be complete first. If using Jadar, the script would need end-step trigger infrastructure in the harness.

## Interactions to Watch

- **Token creation propagates keywords correctly**: `make_token()` at `effects/mod.rs:2901-2904` iterates `spec.keywords` and inserts into the token's `Characteristics.keywords`. Decayed will be included automatically.
- **`decayed_sacrifice_at_eoc` flag is initialized to `false` on token creation**: Confirmed at `effects/mod.rs:2967`. The flag is only set when the token attacks (`combat.rs:437-441`).
- **Tokens cease to exist in non-battlefield zones (CR 704.5d)**: After the Decayed token is sacrificed (moved to graveyard), the SBA will clean it up. This is handled generically for all tokens.
- **Multiplayer**: No special considerations. Decayed enforcement is per-object, not per-player.

## Summary

The Decayed keyword is **fully implemented and validated** (B1). The "Decayed Tokens" batch item is primarily about:
1. Adding 2-4 token-specific unit tests to `decayed.rs` confirming the full pipeline works when the Decayed creature is a token (not just a regular creature)
2. Optionally adding a `zombie_decayed_token_spec()` helper
3. Authoring a card definition for a card that creates Decayed tokens (Jadar recommended)
4. Writing a game script

No new enum variants, no new discriminants, no new enforcement code, no new fields on any struct. This is a low-risk verification and card-authoring task.
