# Ability Plan: Assist

**Generated**: 2026-03-05
**CR**: 702.132
**Priority**: P4
**Similar abilities studied**: Convoke (702.51) in `casting.rs:2113-2132`, Improvise (702.126) in `casting.rs:2134-2153`, Delve (702.66) in `casting.rs:2155-2173` -- all cost-modification static abilities applied during CastSpell

## CR Rule Text

702.132. Assist

702.132a Assist is a static ability that modifies the rules of paying for the spell with assist (see rules 601.2g-h). If the total cost to cast a spell with assist includes a generic mana component, before you activate mana abilities while casting it, you may choose another player. That player has a chance to activate mana abilities. Once that player chooses not to activate any more mana abilities, you have a chance to activate mana abilities. Before you begin to pay the total cost of the spell, the player you chose may pay for any amount of the generic mana in the spell's total cost.

## Key Edge Cases

- **Only generic mana**: The assisting player can only pay generic mana in the total cost. Colored mana costs must be paid by the spell's controller. (Ruling 2018-06-08 on every assist card)
- **Total cost changes**: "If an effect changes the cost of the spell, the amount that player may pay will be more or less than the amount in the spell's reminder text." (Ruling 2018-06-08) -- assist amount is based on TOTAL cost after all modifications (commander tax, kicker, cost reduction), not the printed reminder text.
- **Another player, not self**: CR 702.132a says "you may choose another player" -- the caster cannot choose themselves.
- **Mana abilities first**: The chosen player activates mana abilities first (CR 601.2g ordering), then the caster activates mana abilities. In the engine's model, mana is pre-tapped before casting -- both players must have mana in pool already.
- **Optional**: "you may choose another player" -- assist is optional. The caster can pay the full cost themselves.
- **Eliminated players**: The assisting player must be active in the game (not eliminated). Per CR 800.4a, eliminated players can't take actions.
- **Multiplayer-native**: This ability was designed for multiplayer (Battlebond). Any non-eliminated, non-caster player may be chosen. In a 4-player Commander game, the caster can choose any of the other 3 players.
- **Interaction with cost modification**: Assist applies to the generic portion of the TOTAL cost. Kicker adds to the total cost before assist. Commander tax adds generic mana that assist can pay. Convoke/Improvise/Delve reduce generic mana, reducing the assist-payable amount.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- assist is static, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Assist` variant after `Casualty(u32)` (line ~939)
**Pattern**: Follow `KeywordAbility::Convoke` at line 294 -- simple marker keyword, no parameters
**Doc comment**:
```
/// CR 702.132: Assist -- another player may pay generic mana in the spell's total cost.
/// "If the total cost to cast a spell with assist includes a generic mana component,
/// before you activate mana abilities while casting it, you may choose another player.
/// [...] the player you chose may pay for any amount of the generic mana in the spell's
/// total cost."
///
/// Static ability. Marker for quick presence-checking (`keywords.contains`).
/// No `AbilityDefinition::Assist` needed -- assist has no per-card data to store.
/// The assisting player and amount are provided via `CastSpell.assist_player` and
/// `CastSpell.assist_amount`.
/// CR 702.132a: Multiple instances are redundant.
Assist,
```

**Hash**: Add to `state/hash.rs` `HashInto` impl for `KeywordAbility`
**Discriminant**: 105 (next after Casualty=104)
```rust
// Assist (discriminant 105) -- CR 702.132
KeywordAbility::Assist => 105u8.hash_into(hasher),
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions and add `Assist` arm. Known locations:
- `state/hash.rs` (hash discriminant)
- Any `match kw { ... }` in `casting.rs`, `combat.rs`, `abilities.rs`, `builder.rs` -- verify with grep

### Step 2: CastSpell Command Extension

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add two new fields to `CastSpell` after `casualty_sacrifice`:

```rust
/// CR 702.132a: The player who assists with the generic mana cost.
/// `None` means no assist (either the spell lacks Assist or the caster
/// chose not to use it). Must be a non-eliminated player other than
/// the caster (CR 702.132a: "another player").
#[serde(default)]
assist_player: Option<PlayerId>,
/// CR 702.132a: The amount of generic mana the assisting player pays.
/// Must be <= the generic component of the spell's total cost (after all
/// cost modifications: commander tax, kicker, affinity, undaunted, convoke,
/// improvise, delve). 0 is valid (no-op assist). Ignored when
/// `assist_player` is `None`.
#[serde(default)]
assist_amount: u32,
```

**Rationale for two fields**: Unlike convoke/improvise/delve which modify the cost object, assist deducts mana from another player's pool. The engine needs to know WHO pays and HOW MUCH. The "how much" must be explicitly stated because in the engine model mana abilities are resolved before the CastSpell command (no interactive mana ability activation during casting).

### Step 3: Rule Enforcement in casting.rs

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add assist validation and payment in `handle_cast_spell`, AFTER all cost reductions (convoke, improvise, delve) and BEFORE the final `can_pay_cost` / `pay_cost` call.

**CR**: 702.132a -- "the player you chose may pay for any amount of the generic mana in the spell's total cost"

**Pipeline position**: The cost modification pipeline order becomes:
`base_mana_cost -> alt_cost -> commander_tax -> kicker -> affinity -> undaunted -> convoke -> improvise -> delve -> ASSIST -> pay`

**Logic** (pseudocode):
```rust
// CR 702.132a: Apply assist — another player pays generic mana.
let mana_cost = if let Some(assist_pid) = assist_player {
    // Validate: spell must have Assist keyword
    if !chars.keywords.contains(&KeywordAbility::Assist) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have assist".into(),
        ));
    }
    // Validate: assisting player is not the caster
    if assist_pid == player {
        return Err(GameStateError::InvalidCommand(
            "cannot assist yourself — must choose another player".into(),
        ));
    }
    // Validate: assisting player is active (not eliminated)
    if !state.active_players().contains(&assist_pid) {
        return Err(GameStateError::InvalidCommand(
            "assisting player is not active".into(),
        ));
    }
    // Validate: assist_amount <= generic mana remaining in total cost
    let generic_remaining = mana_cost.as_ref().map_or(0, |c| c.generic);
    if assist_amount > generic_remaining {
        return Err(GameStateError::InvalidCommand(
            format!("assist amount {} exceeds generic mana {} in total cost",
                    assist_amount, generic_remaining),
        ));
    }
    if assist_amount > 0 {
        // Deduct generic mana from assisting player's pool
        let assist_pool = &state.player(assist_pid)?.mana_pool;
        if assist_pool.total() < assist_amount {
            return Err(GameStateError::InsufficientMana);
        }
        // Pay generic mana from the assisting player's pool
        let assist_cost = ManaCost {
            generic: assist_amount,
            ..Default::default()
        };
        let assist_player_state = state.player_mut(assist_pid)?;
        pay_cost(&mut assist_player_state.mana_pool, &assist_cost);
        events.push(GameEvent::ManaCostPaid {
            player: assist_pid,
            cost: assist_cost.clone(),
        });
        // Reduce generic in the caster's remaining cost
        let mut reduced = mana_cost.unwrap_or_default();
        reduced.generic = reduced.generic.saturating_sub(assist_amount);
        Some(reduced)
    } else {
        mana_cost
    }
} else {
    mana_cost
};
```

**Note**: The `ManaCostPaid` event for the assisting player is important for event correctness -- it shows that mana was spent from a different player's pool.

### Step 4: Replay Harness Support

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `assist_player_name` (Option<&str>) and `assist_amount` (u32) to `translate_player_action()`. Add a new action type `cast_spell_assist` or extend `cast_spell` to accept optional `assist_player` and `assist_amount` fields in the JSON schema.

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add `assist_player` (Option<String>) and `assist_amount` (Option<u32>) fields to the action schema if the schema has explicit fields for cost modifiers.

**Pattern**: Follow how `bargain_sacrifice_name` and `casualty_sacrifice_name` are threaded through the harness at `replay_harness.rs:250-260`.

### Step 5: Unit Tests

**File**: `crates/engine/tests/assist.rs`
**Tests to write**:

1. `test_assist_basic_another_player_pays_generic` -- CR 702.132a
   - Setup: 4 players. P1 has a {4}{U} Assist sorcery in hand. P2 has 3 generic mana in pool. P1 has {U} + 1 generic.
   - Action: P1 casts with assist_player=P2, assist_amount=3.
   - Assert: P2's pool reduced by 3, P1's pool reduced by {U}+1, spell on stack.

2. `test_assist_no_assist_player_pays_full_cost` -- Assist is optional
   - Setup: P1 has enough mana to pay full cost alone.
   - Action: P1 casts with assist_player=None.
   - Assert: P1's pool drained, spell on stack.

3. `test_assist_cannot_assist_self` -- CR 702.132a "another player"
   - Setup: P1 tries to cast with assist_player=P1.
   - Assert: InvalidCommand error.

4. `test_assist_only_generic_mana` -- Ruling 2018-06-08
   - Setup: {2}{U} Assist spell. P2 tries to pay 3 (exceeds generic=2).
   - Assert: InvalidCommand error (assist_amount > generic).

5. `test_assist_eliminated_player_cannot_assist` -- CR 800.4a
   - Setup: P2 is eliminated (life <= 0, removed from active_players).
   - Action: P1 tries assist_player=P2.
   - Assert: InvalidCommand error.

6. `test_assist_with_commander_tax` -- CR 702.132a total cost interaction
   - Setup: Commander with Assist keyword, 2 commander tax accrued. Base cost {3}{U}, total cost {5}{U}.
   - Action: P2 assists with 5 generic mana.
   - Assert: P2 pays 5, P1 pays {U}. Commander tax is payable by assist.

7. `test_assist_with_convoke_reduces_assist_ceiling` -- interaction
   - Setup: {4}{G} Assist+Convoke spell. P1 taps 2 creatures for convoke (reducing generic by 2). Generic remaining = 2.
   - Action: P2 assists with 2.
   - Assert: P2 pays 2, P1 pays {G} only.

8. `test_assist_amount_zero_is_noop` -- edge case
   - Setup: P1 casts with assist_player=P2, assist_amount=0.
   - Assert: P2 pool unchanged, P1 pays full cost, spell on stack.

9. `test_assist_insufficient_mana_assisting_player` -- error case
   - Setup: P2 has only 1 mana but assist_amount=3.
   - Assert: InsufficientMana error.

10. `test_assist_spell_without_keyword_rejected` -- validation
    - Setup: Non-assist spell with assist_player=P2.
    - Assert: InvalidCommand error.

11. `test_assist_multiplayer_any_opponent_can_assist` -- multiplayer
    - Setup: 4 players. P3 assists P1's spell.
    - Assert: P3's pool reduced, spell on stack.

**Pattern**: Follow tests in `crates/engine/tests/convoke.rs` for structure (helper functions, GameStateBuilder usage, process_command assertions).

### Step 6: Card Definition (later phase)

**Suggested card**: Huddle Up (simple: {2}{U} Sorcery, Assist, two target players each draw a card)
**Card lookup**: use `card-definition-author` agent
**File**: `crates/engine/src/cards/defs/huddle_up.rs`

### Step 7: Game Script (later phase)

**Suggested scenario**: 4-player Commander game where P1 casts Huddle Up with P2 assisting by paying 2 generic mana. P1 pays {U}. Both target players draw a card.
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Sequence number**: Next available in stack/ directory

## Interactions to Watch

- **Cost modification order**: Assist reads the generic component AFTER convoke/improvise/delve have reduced it. The pipeline order must be strictly maintained. Assist cannot increase the generic cost -- it can only redistribute who pays it.
- **Mana pool deduction from another player**: This is the ONLY ability in the engine where `pay_cost` is called on a player OTHER than the caster. Verify that `state.player_mut(assist_pid)` correctly mutates the assisting player's state without interfering with the caster's state.
- **ManaCostPaid event attribution**: The event should correctly attribute the payment to the assisting player (not the caster). This matters for any future "whenever a player pays mana" triggers.
- **No triggers**: Assist is purely static. It modifies the cost-payment procedure. No PendingTrigger, no StackObjectKind variant needed.
- **No AbilityDefinition variant needed**: Assist has no per-card parameters (unlike Convoke which is also parameterless). The keyword marker on the spell is sufficient. The assist player/amount come from the CastSpell command.
- **`process_command` ownership**: The function takes ownership of GameState. The assist player's mana pool modification happens on the same mutable state as the caster's mana deduction -- no dual-access issue.
