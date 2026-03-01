# Ability Plan: Aftermath

**Generated**: 2026-03-01
**CR**: 702.127
**Priority**: P4
**Similar abilities studied**: Flashback (CR 702.34) -- `rules/casting.rs`, `rules/resolution.rs`, `state/stack.rs`, `state/types.rs`, `cards/card_definition.rs`; Escape (CR 702.138) -- `rules/casting.rs` graveyard-casting path

## CR Rule Text

**702.127. Aftermath**

> 702.127a Aftermath is an ability found on some split cards (see rule 709, "Split Cards"). It represents three static abilities. "Aftermath" means "You may cast this half of this split card from your graveyard," "This half of this split card can't be cast from any zone other than a graveyard," and "If this spell was cast from a graveyard, exile it instead of putting it anywhere else any time it would leave the stack."

**Related split card rules (709):**

> 709.1. Split cards have two card faces on a single card. The back of a split card is the normal Magic card back.
>
> 709.2. Although split cards have two castable halves, each split card is only one card.
>
> 709.3. A player chooses which half of a split card they are casting before putting it onto the stack.
>
> 709.3a Only the chosen half is evaluated to see if it can be cast. Only that half is considered to be put onto the stack.
>
> 709.3b While on the stack, only the characteristics of the half being cast exist. The other half's characteristics are treated as though they didn't exist.
>
> 709.4. In every zone except the stack, the characteristics of a split card are those of its two halves combined.
>
> 709.4b The mana cost of a split card is the combined mana costs of its two halves. A split card's colors and mana value are determined from its combined mana cost.
>
> 709.4c A split card has each card type specified on either of its halves and each ability in the text box of each half.

## Key Edge Cases

1. **Three static abilities in one keyword** (CR 702.127a): Aftermath encodes:
   - (a) Permission to cast the aftermath half from graveyard.
   - (b) Restriction: the aftermath half CANNOT be cast from any zone other than graveyard (including hand).
   - (c) Exile replacement: if cast from graveyard, exile instead of going anywhere else when leaving the stack.

2. **First half is cast normally from hand** (ruling 2017-04-18): "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you're casting." The first half follows normal casting rules (from hand only, goes to graveyard on resolution).

3. **Aftermath half cannot be cast from hand** (ruling 2017-04-18): "If another effect allows you to cast a split card with aftermath from any zone other than a graveyard, you can't cast the half with aftermath." This means the aftermath half is STRICTLY graveyard-only.

4. **Another effect can grant graveyard casting of either half** (ruling 2017-04-18): "If another effect allows you to cast a split card with aftermath from a graveyard, you may cast either half. If you cast the half that has aftermath, you'll exile the card if it would leave the stack." This is a future interaction concern, not needed for initial implementation.

5. **Exile on ANY stack departure** (ruling 2017-04-18): "A spell with aftermath cast from a graveyard will always be exiled afterward, whether it resolves, it's countered, or it leaves the stack in some other way." This is identical to Flashback's exile behavior. The existing `cast_with_flashback` mechanism covers this pattern.

6. **Timing follows the cast half's type** (ruling 2017-04-18): "If you cast the first half of a split card with aftermath during your turn, you'll have priority immediately after it resolves. You can cast the half with aftermath from your graveyard before any player can take any other action if it's legal for you to do so." This means sorcery aftermath halves follow sorcery timing, instant aftermath halves follow instant timing.

7. **Split card characteristics combine in non-stack zones** (CR 709.4): In hand, graveyard, exile, etc., the card has BOTH names, BOTH mana costs (combined), BOTH types, and ALL abilities from BOTH halves. On the stack, only the chosen half's characteristics exist.

8. **Card is one card, not two** (CR 709.2): Discarding the split card discards one card. Effects counting instants/sorceries in graveyard count this as one card.

9. **Multiplayer**: No special multiplayer considerations. Aftermath works the same in all formats.

10. **Stack departure via CounterSpell effect** (from Flashback gotcha): The `cast_with_flashback` flag on `StackObject` is already checked at all 4 stack departure points (resolution, fizzle, counter in `resolution.rs`, and counter via `Effect::CounterSpell` in `effects/mod.rs`). Reusing this flag for aftermath avoids missing any path.

## Current State (from ability-wip.md)

No prior work exists for Aftermath. The ability-wip.md currently tracks Retrace (a different Batch 4 ability). Aftermath does not appear anywhere in the engine source.

- [ ] Step 1: Enum variant + split card infrastructure
- [ ] Step 2: Rule enforcement (casting)
- [ ] Step 3: Trigger wiring (n/a -- Aftermath is static, not triggered)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Architectural Decision: Split Card Representation

### The Problem

The engine currently has no split card support. `CardDefinition` has a single `name`, single `mana_cost`, single `types`, and a single `abilities` list. Aftermath cards need TWO distinct spell halves, each with their own name, mana cost, types, and spell effect.

### Rejected Approach: Full Split Card Infrastructure

Adding full CR 709 support (two-half `CardDefinition`, combined characteristics in non-stack zones, half-selection at cast time, etc.) is a large infrastructure change that affects `CardDefinition`, `Characteristics`, `calculate_characteristics()`, `enrich_spec_from_def()`, targeting, and many other systems. This is disproportionate for the ~30 aftermath cards in existence and would need to also cover Fuse (CR 702.102) and plain split cards.

### Chosen Approach: Aftermath-Specific Minimal Split Card

Model an aftermath card as a **single `CardDefinition`** with a **new `AbilityDefinition::Aftermath` variant** that encodes the aftermath half's complete spell data (name, mana cost, card type, effect, targets). The card definition's top-level `name`, `mana_cost`, and `types` describe the first (hand) half. The aftermath variant describes the second (graveyard) half.

This approach:
- Requires NO changes to `CardDefinition` struct fields.
- Fits the established pattern of `AbilityDefinition` variants carrying cast-related data (like `Flashback { cost }`, `Escape { cost, exile_count }`, `Ninjutsu { cost }`).
- Naturally expresses "the card has a second spell you can cast from graveyard."
- Can be extended to full split card support later if needed.

**Combined characteristics (CR 709.4)**: For zone-based characteristic queries (name checks, mana value, color identity, type checks), the card should report combined characteristics when not on the stack. This is a known gap that will NOT be implemented now -- it only matters for effects like "cards in your graveyard with mana value 5 or less" which need the combined MV. For initial implementation, the first half's characteristics are sufficient.

**Naming**: The card definition name will be "Cut // Ribbons" (the combined name). The aftermath half stores its own name "Ribbons" for display and future name-matching purposes.

## Implementation Steps

### Step 1: Type System Changes

#### Step 1a: KeywordAbility::Aftermath

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Aftermath` variant to `KeywordAbility` enum after `CommanderNinjutsu` (line 761).
**Pattern**: Follow `KeywordAbility::Flashback` at line 247 -- bare marker keyword with cost stored in `AbilityDefinition`.

```rust
/// CR 702.127: Aftermath -- the second half of a split card can only be cast
/// from the graveyard. If cast from graveyard, exile instead of putting it
/// anywhere else when it leaves the stack.
///
/// This variant is a marker for quick presence-checking (`keywords.contains`).
/// The aftermath half's spell data is stored in `AbilityDefinition::Aftermath`.
Aftermath,
```

**Hash discriminant**: 89 (next after CommanderNinjutsu = 88).

#### Step 1b: AbilityDefinition::Aftermath

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Aftermath` variant to `AbilityDefinition` enum after `CommanderNinjutsu { cost }` (line 278).
**Pattern**: Follow `AbilityDefinition::Spell { effect, targets, modes, cant_be_countered }` at line 96 -- a variant that carries complete spell data.

```rust
/// CR 702.127: Aftermath. The second half of a split card. Can only be cast
/// from the graveyard. When it leaves the stack after being cast from graveyard,
/// it is exiled instead of going anywhere else.
///
/// The aftermath half is a complete spell: it has its own name, mana cost,
/// card type(s), spell effect, and targets. The card definition's top-level
/// fields (name, mana_cost, types) describe the first (hand-castable) half.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Aftermath)` for quick
/// presence-checking without scanning all abilities.
///
/// At cast time from graveyard, the engine uses the aftermath half's mana_cost
/// as the spell cost (alternative cost per CR 118.9). The aftermath half's
/// effect is resolved instead of the card's first-half Spell effect.
Aftermath {
    /// Name of the aftermath half (e.g., "Ribbons" for "Cut // Ribbons").
    name: String,
    /// Mana cost of the aftermath half (paid when casting from graveyard).
    cost: ManaCost,
    /// Card type of the aftermath half (Sorcery, Instant, etc.).
    card_type: CardType,
    /// The spell effect of the aftermath half.
    effect: Effect,
    /// Target requirements for the aftermath half's spell.
    targets: Vec<TargetRequirement>,
},
```

**Hash discriminant for AbilityDefinition**: 24 (next after CommanderNinjutsu = 23). Hash all fields: name, cost, card_type, effect, targets.

#### Step 1c: StackObject flag reuse

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Reuse the existing `cast_with_flashback: bool` field for aftermath spells cast from graveyard.
**Rationale**: The exile-on-stack-departure behavior is IDENTICAL between Flashback (CR 702.34a) and Aftermath (CR 702.127a). Both say "exile instead of putting it anywhere else any time it would leave the stack." Reusing the same flag means all 4 existing stack departure paths already handle aftermath exile correctly.

**Rename consideration**: The field could be renamed to `exile_on_stack_departure` for generality, but this would require touching every existing use (14+ sites). Instead, document the dual-use:

```rust
/// CR 702.34a / CR 702.127a: If true, this spell was cast via flashback or
/// aftermath from the graveyard. When it leaves the stack (resolves, is
/// countered, or fizzles), it is exiled instead of going to any other zone.
```

**No new StackObject field needed.**

#### Step 1d: Hash updates

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Actions**:

1. **KeywordAbility::Aftermath hash** (after `CommanderNinjutsu => 88u8`):
   ```rust
   // Aftermath (discriminant 89) -- CR 702.127
   KeywordAbility::Aftermath => 89u8.hash_into(hasher),
   ```

2. **AbilityDefinition::Aftermath hash** (after `CommanderNinjutsu` discriminant 23):
   ```rust
   // Aftermath (discriminant 24) -- CR 702.127
   AbilityDefinition::Aftermath { name, cost, card_type, effect, targets } => {
       24u8.hash_into(hasher);
       name.hash_into(hasher);
       cost.hash_into(hasher);
       card_type.hash_into(hasher);
       effect.hash_into(hasher);
       targets.hash_into(hasher);
   }
   ```

#### Step 1e: Match arm exhaustiveness

After adding `KeywordAbility::Aftermath`, grep for all exhaustive `match` on `KeywordAbility` and add the new arm. Key locations:

- `state/hash.rs` -- covered above
- `tools/tui/src/play/panels/stack_view.rs` -- if it matches on `KeywordAbility` (check)
- `tools/replay-viewer/src/view_model.rs` -- if it matches on `KeywordAbility` (check)

After adding `AbilityDefinition::Aftermath { .. }`, grep for all exhaustive `match` on `AbilityDefinition`:
- `state/hash.rs` -- covered above
- `state/builder.rs` -- may need to handle the Aftermath variant (e.g., extract keywords)
- `rules/casting.rs` -- `get_flashback_cost` and similar helpers scan abilities

### Step 2: Rule Enforcement -- Casting

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Extend `handle_cast_spell` to support casting the aftermath half from graveyard.
**CR**: 702.127a, 709.3, 709.3a

#### Step 2a: Add `cast_with_aftermath` flag to Command::CastSpell

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add a `cast_with_aftermath: bool` field to `Command::CastSpell` (after `cast_with_overload`).

```rust
/// CR 702.127a: If true, cast the aftermath half of this split card from
/// the graveyard. The aftermath half's mana cost is paid instead of the
/// card's first-half mana cost. The aftermath half's spell effect is used
/// at resolution. The card is exiled when it leaves the stack.
///
/// The card must be in the caster's graveyard and must have the
/// Aftermath keyword. This is an alternative cost (CR 118.9).
#[serde(default)]
cast_with_aftermath: bool,
```

**Propagation**: This field must be passed through `engine.rs` `process_command` -> `handle_cast_spell`. Add `cast_with_aftermath` parameter to `handle_cast_spell` signature.

#### Step 2b: Zone validation in handle_cast_spell

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Location**: After the flashback detection (line ~112-117), add aftermath detection.

```rust
// CR 702.127a: Aftermath -- allowed if card has the Aftermath keyword
// and is in graveyard, and cast_with_aftermath is true.
let casting_with_aftermath = cast_with_aftermath
    && casting_from_graveyard
    && card_obj
        .characteristics
        .keywords
        .contains(&KeywordAbility::Aftermath);
```

**Zone validation update** (line ~184-195): Add `!casting_with_aftermath` to the zone rejection condition:

```rust
if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
    && !casting_with_madness
    && !casting_with_escape_auto
    && !cast_with_escape
    && !cast_with_foretell
    && !casting_with_aftermath  // NEW
{
    return Err(GameStateError::InvalidCommand(
        "card is not in your hand".into(),
    ));
}
```

#### Step 2c: Alternative cost validation

**Location**: After the existing alternative cost mutual-exclusion checks (around line ~260-360).

```rust
// Step 1f: Validate aftermath (CR 702.127a / CR 118.9a).
// Aftermath is an alternative cost -- cannot combine with flashback, evoke,
// bestow, madness, miracle, escape, foretell, or overload.
if casting_with_aftermath {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine aftermath with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    // ... (similar checks for evoke, bestow, etc.)
}
```

#### Step 2d: Cost determination

**Location**: The cost determination section (around line ~255-420).

When `casting_with_aftermath` is true, look up the aftermath cost from `AbilityDefinition::Aftermath { cost, .. }`:

```rust
fn get_aftermath_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Aftermath { cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

In the cost calculation:
```rust
let mana_cost = if casting_with_aftermath {
    get_aftermath_cost(&card_id, &state.card_registry)
} else if casting_with_flashback {
    // ... existing flashback cost
} else {
    // ... existing normal cost
};
```

#### Step 2e: Type validation

The aftermath half may have a different card type than the first half (e.g., Spring is Sorcery, Mind is Instant). When casting the aftermath half, use the aftermath half's card_type for timing validation:

```rust
// CR 702.127a + CR 709.3a: When casting the aftermath half, use its type
// for timing validation, not the first half's type.
if casting_with_aftermath {
    let aftermath_type = get_aftermath_card_type(&card_id, &state.card_registry);
    let is_instant_speed = aftermath_type == Some(CardType::Instant)
        || chars.keywords.contains(&KeywordAbility::Flash);
    // ... timing validation with aftermath type
}
```

Add helper:
```rust
fn get_aftermath_card_type(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<CardType> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Aftermath { card_type, .. } = a {
                    Some(card_type.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

#### Step 2f: Set flashback flag on StackObject

When creating the StackObject (line ~950-960), set `cast_with_flashback` to true when casting with aftermath (reuses flashback's exile mechanism):

```rust
cast_with_flashback: casting_with_flashback || casting_with_aftermath,
```

#### Step 2g: Prevent casting aftermath half from hand

The CastSpell command needs to reject casting the first half of an aftermath card when `cast_with_aftermath` is true and the card is in hand (edge case: prevent misuse). But more importantly, the engine should NOT allow the aftermath half's effect to be used from hand. Since the command only says `cast_with_aftermath: true/false`, and the card definition stores both halves, the enforcement is:

- `cast_with_aftermath: false` + card in hand -> cast first half normally
- `cast_with_aftermath: true` + card in graveyard -> cast aftermath half
- `cast_with_aftermath: true` + card in hand -> ERROR (aftermath can only be cast from graveyard)
- `cast_with_aftermath: false` + card in graveyard -> ERROR (card is not in your hand, unless it has Flashback/Escape)

### Step 3: Rule Enforcement -- Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: When resolving a spell that was cast with aftermath, use the aftermath half's effect instead of the first half's Spell effect.
**CR**: 702.127a, 709.3b

#### Step 3a: Track which half was cast

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `cast_with_aftermath: bool` field to `StackObject` (after `was_overloaded`).

```rust
/// CR 702.127a: If true, this spell was cast as the aftermath half of a
/// split card from the graveyard. At resolution, the aftermath half's
/// effect is used instead of the first half's Spell effect.
///
/// The exile-on-stack-departure behavior is handled by `cast_with_flashback`
/// being set to true (same mechanism as Flashback).
#[serde(default)]
pub cast_with_aftermath: bool,
```

**Hash**: Add `self.cast_with_aftermath.hash_into(hasher)` after `self.was_overloaded.hash_into(hasher)` in `state/hash.rs`.

#### Step 3b: Resolution effect selection

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Location**: The spell resolution code at line ~150-165, where it finds `AbilityDefinition::Spell { effect, .. }`.

Add an aftermath branch BEFORE the normal Spell lookup:

```rust
// CR 702.127a + CR 709.3b: If aftermath half was cast, use the
// aftermath effect instead of the first-half Spell effect.
let spell_effect = if stack_obj.cast_with_aftermath {
    def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Aftermath { effect, .. } = a {
            Some(effect.clone())
        } else {
            None
        }
    })
} else {
    def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Spell { effect, .. } = a {
            Some(effect.clone())
        } else {
            None
        }
    })
};
```

#### Step 3c: Target validation at resolution

The aftermath half may have different targets than the first half. The targets were already validated at cast time using the aftermath half's `TargetRequirement` list. At resolution, the existing fizzle check validates stored targets -- no changes needed since the targets on the `StackObject` are the ones declared at cast time (which were validated against the aftermath half's requirements).

However, the cast-time target validation in `casting.rs` currently looks up targets from `AbilityDefinition::Spell { targets, .. }`. When casting the aftermath half, it must look up targets from `AbilityDefinition::Aftermath { targets, .. }` instead.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Location**: Where target requirements are looked up (grep for `TargetRequirement` in casting.rs).

### Step 4: Trigger Wiring

**Not applicable.** Aftermath is a static ability, not a triggered ability. It represents three static abilities per CR 702.127a:
1. Permission to cast from graveyard (handled in casting.rs)
2. Restriction against casting from non-graveyard zones (handled in casting.rs)
3. Exile on stack departure (handled via `cast_with_flashback` flag)

No changes to `rules/abilities.rs` or `check_triggers`.

### Step 5: Replay Harness

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_aftermath"` action type that finds the card in graveyard and sets `cast_with_aftermath: true`.

```rust
"cast_spell_aftermath" => {
    // CR 702.127a: Cast the aftermath half of a split card from graveyard.
    let card_id = find_in_graveyard(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        cast_with_evoke: false,
        cast_with_bestow: false,
        cast_with_miracle: false,
        cast_with_escape: false,
        escape_exile_cards: vec![],
        cast_with_foretell: false,
        cast_with_buyback: false,
        cast_with_overload: false,
        cast_with_aftermath: true,  // NEW
    })
}
```

The `find_in_graveyard` helper already exists (added for flashback).

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/aftermath.rs` (new file)
**Pattern**: Follow `crates/engine/tests/flashback.rs` for structure and helpers.

**Test card definitions needed:**

1. **"Cut // Ribbons" test card** -- simplified for testing:
   - First half: "Cut" -- Sorcery, {1}{R}, effect: DealDamage(4) to target creature
   - Aftermath half: "Ribbons" -- Sorcery, {2}{B}{B}, effect: DealDamage(3) to each opponent (simplified from {X}{B}{B} to avoid X cost complexity)
   - Both halves are sorceries (simplest case)

2. **"Spring // Mind" test card** (optional, for timing test):
   - First half: "Spring" -- Sorcery, {2}{G}, effect: SearchLibrary (or simplified)
   - Aftermath half: "Mind" -- Instant, {4}{U}{U}, effect: DrawCards(3)
   - Tests that an instant aftermath half can be cast at instant speed

**Tests to write:**

#### `test_aftermath_basic_cast_first_half_from_hand`
- CR 709.3: First half can be cast normally from hand.
- Setup: Cut // Ribbons in p1's hand, p1 has {1}{R} mana. Main phase.
- Action: CastSpell (normal, no aftermath flag).
- Assert: Spell on stack. Mana deducted. Card is on stack.
- Pass all, resolve.
- Assert: Card in graveyard (normal instant/sorcery resolution).

#### `test_aftermath_cast_second_half_from_graveyard`
- CR 702.127a: Aftermath half can be cast from graveyard.
- Setup: Cut // Ribbons in p1's graveyard, p1 has {2}{B}{B} mana. Main phase.
- Action: CastSpell with `cast_with_aftermath: true`.
- Assert: Spell on stack. Aftermath cost deducted.
- Pass all, resolve.
- Assert: Card in exile (not graveyard) -- exile behavior.

#### `test_aftermath_exile_on_resolution`
- CR 702.127a: "Exile it instead of putting it anywhere else any time it would leave the stack."
- Setup: Cut // Ribbons in graveyard, cast aftermath half, both pass.
- Assert: After resolution, card is in exile zone.

#### `test_aftermath_exile_on_counter`
- CR 702.127a: Exile even when countered.
- Setup: Cut // Ribbons cast aftermath, opponent casts Counterspell.
- Assert: After counter resolves, Cut // Ribbons is in exile.

#### `test_aftermath_exile_on_fizzle`
- CR 702.127a: Exile even when fizzled (all targets illegal).
- Setup: Aftermath half has a target; target becomes illegal before resolution.
- Assert: Card exiled (not graveyard).

#### `test_aftermath_cannot_cast_second_half_from_hand`
- CR 702.127a: "This half of this split card can't be cast from any zone other than a graveyard."
- Setup: Cut // Ribbons in hand. Try CastSpell with `cast_with_aftermath: true`.
- Assert: Error returned.

#### `test_aftermath_cannot_cast_second_half_without_flag`
- Negative: Card with aftermath in graveyard, but `cast_with_aftermath: false` (no flashback/escape either).
- Setup: Cut // Ribbons in graveyard. Try CastSpell without any graveyard-casting flag.
- Assert: Error returned ("card is not in your hand").

#### `test_aftermath_first_half_goes_to_graveyard`
- CR 608.2n: First half cast normally from hand goes to graveyard (not exile).
- Setup: Cut // Ribbons in hand, cast first half, resolve.
- Assert: Card in graveyard after resolution.

#### `test_aftermath_pays_aftermath_cost`
- CR 702.127a: The aftermath half's cost is paid, not the first half's cost.
- Setup: Cut // Ribbons in graveyard. Player has exactly {2}{B}{B} (enough for aftermath but not first half if {1}{R}).
- Assert: Cast succeeds. Aftermath cost deducted.

#### `test_aftermath_card_without_aftermath_in_graveyard_fails`
- Negative: A non-aftermath card in graveyard, `cast_with_aftermath: true`.
- Setup: Lightning Bolt in graveyard. Try CastSpell with `cast_with_aftermath: true`.
- Assert: Error returned.

#### `test_aftermath_uses_aftermath_effect`
- The aftermath half's effect is executed (not the first half's effect).
- Setup: Cut // Ribbons in graveyard. Cast aftermath half (Ribbons). Target: opponent.
- Assert: Opponent lost life (Ribbons effect), not "4 damage to creature" (Cut effect).

#### `test_aftermath_full_lifecycle`
- Complete lifecycle: cast first half from hand -> goes to graveyard -> cast aftermath from graveyard -> exiled.
- Setup: Cut // Ribbons in hand. Cast Cut (first half), resolve. Then cast Ribbons (aftermath) from graveyard, resolve.
- Assert: Card in exile at the end.

### Step 7: Card Definition (later phase)

**Suggested card**: Cut // Ribbons
- Oracle: Cut is "{1}{R} Sorcery -- Cut deals 4 damage to target creature."
  Ribbons is "{X}{B}{B} Sorcery -- Each opponent loses X life." (Aftermath)
- Both halves are sorceries (simpler timing rules).
- Cut is a simple targeted damage spell (DealDamage + target creature).
- Ribbons has X cost which is complex; for the initial card def, could use a fixed cost version for testing, then upgrade to X support later.
- Commander legal and played in some decks.

**Simplified test-only card**: For unit tests, define an inline test helper `cut_ribbons_test_def()` with Ribbons as a fixed-cost spell (e.g., {2}{B}{B} -- each opponent loses 3 life) to avoid X cost infrastructure requirements.

**Card lookup**: Use `card-definition-author` agent for the real card definition once the infrastructure is in place.

### Step 8: Game Script (later phase)

**Suggested scenario**: "Aftermath Full Lifecycle"
- Player 1 casts Cut (first half) from hand targeting a creature.
- Creature takes 4 damage. Cut goes to graveyard.
- Player 1 then casts Ribbons (aftermath half) from graveyard.
- Each opponent loses life. Ribbons + card exiled.

**Subsystem directory**: `test-data/generated-scripts/stack/` (aftermath involves casting and stack resolution, same as flashback)

**Script actions needed:**
1. `cast_spell` -- Cut // Ribbons from hand (cast the Cut half)
2. `pass_priority` x N -- resolve Cut
3. `cast_spell_aftermath` -- Cut // Ribbons from graveyard (cast the Ribbons half)
4. `pass_priority` x N -- resolve Ribbons; verify exile

## Interactions to Watch

### Casting system
- **Alternative cost mutual exclusion** (CR 118.9a): Aftermath is an alternative cost. It cannot combine with Flashback, Evoke, Bestow, Madness, Miracle, Escape, Foretell, or Overload. The existing mutual-exclusion checks in casting.rs need a new block for `casting_with_aftermath`.
- **Commander tax**: Unlikely to apply (aftermath cards are instants/sorceries, not commanders), but if it somehow did, tax applies on top per CR 118.9d. The existing tax logic handles this naturally.

### Resolution system
- **Effect selection**: The resolution code at line ~150-165 uses `find_map` to get the first `AbilityDefinition::Spell` from the card definition. When `cast_with_aftermath` is true, it must find `AbilityDefinition::Aftermath` instead. This is the most critical change.
- **Exile mechanism**: Reuses `cast_with_flashback` flag. All 4 existing stack departure paths (resolution, fizzle, counter in resolution.rs, counter in effects/mod.rs) already check this flag.

### Copy system
- **Storm/Cascade copies**: When creating copies, set `cast_with_aftermath: false` (copies are not cast). The `cast_with_flashback: false` is already set for copies, so aftermath exile does not apply to copies.

### Layer system
- **Combined characteristics (CR 709.4)**: NOT implemented in this plan. In non-stack zones, the card reports only the first half's characteristics. This is acceptable for initial implementation -- the aftermath half's characteristics matter only for zone-based queries (name matching, mana value, color) which are edge cases.
- **Future**: Full split card support in the layer system would require `calculate_characteristics()` to merge both halves when the object is not on the stack.

### Prowess interaction
- Casting the aftermath half IS casting a spell. Prowess triggers normally. The `SpellCast` event is emitted by `handle_cast_spell` regardless of the alternative cost used.

### Stack object display
- The replay viewer's `StateViewModel` serializes `StackObject`. The new `cast_with_aftermath` field will appear in JSON output automatically. No viewer changes needed beyond the serialization.

## File Change Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/engine/src/state/types.rs` | Add variant | `KeywordAbility::Aftermath` (disc 89) |
| `crates/engine/src/cards/card_definition.rs` | Add variant | `AbilityDefinition::Aftermath { name, cost, card_type, effect, targets }` |
| `crates/engine/src/state/stack.rs` | Add field | `StackObject::cast_with_aftermath: bool` |
| `crates/engine/src/state/hash.rs` | Add hash arms | KeywordAbility (89), AbilityDefinition (24), StackObject field |
| `crates/engine/src/rules/command.rs` | Add field | `Command::CastSpell::cast_with_aftermath: bool` |
| `crates/engine/src/rules/casting.rs` | Major changes | Zone validation, cost lookup, type validation, aftermath flag, helpers |
| `crates/engine/src/rules/resolution.rs` | Modify | Effect selection: aftermath vs first-half Spell |
| `crates/engine/src/rules/copy.rs` | Minor | Ensure `cast_with_aftermath: false` on copies |
| `crates/engine/src/rules/engine.rs` | Minor | Pass `cast_with_aftermath` from Command to handle_cast_spell |
| `crates/engine/src/testing/replay_harness.rs` | Add action | `cast_spell_aftermath` harness action |
| `crates/engine/tests/aftermath.rs` | New file | 12 unit tests |
| Match arm additions | Multiple files | Add `Aftermath` arm to exhaustive matches on KeywordAbility and AbilityDefinition |
| `tools/tui/src/play/panels/stack_view.rs` | Minor | Add arm for any new StackObjectKind (none in this case -- no new SOK) |

## Risks and Mitigations

1. **Risk: Resolution uses wrong spell effect.** If the aftermath branch in resolution.rs is not reached, the first half's Spell effect fires when the aftermath half was intended.
   - **Mitigation**: The `cast_with_aftermath` flag on StackObject explicitly gates effect selection. Test `test_aftermath_uses_aftermath_effect` verifies the correct effect fires.

2. **Risk: Missing `cast_with_aftermath` in Command propagation.** The new field on `Command::CastSpell` must be passed through engine.rs to casting.rs.
   - **Mitigation**: Compilation will fail if the field is missing in the Command destructure pattern (exhaustive match). The test suite ensures the field reaches casting.rs.

3. **Risk: Combined characteristics gap (CR 709.4).** Name checks, mana value queries, and color checks in non-stack zones will only see the first half's data.
   - **Mitigation**: This is a known limitation, documented above. It only affects effects that query characteristics of cards in graveyard/hand/exile. No initial tests exercise this path. Can be addressed when full split card support is added.

4. **Risk: Target validation at cast time uses wrong target list.** The casting code looks up `AbilityDefinition::Spell { targets, .. }` for target validation. When casting the aftermath half, it must use `AbilityDefinition::Aftermath { targets, .. }`.
   - **Mitigation**: Add an aftermath-aware target lookup before the existing Spell target lookup. Test with a targeted aftermath half.

5. **Risk: `cast_with_aftermath` not propagated to copies.** Storm copies might inherit the flag.
   - **Mitigation**: Explicitly set `cast_with_aftermath: false` when creating copies in copy.rs. Copies are never cast, so they should never have this flag.
