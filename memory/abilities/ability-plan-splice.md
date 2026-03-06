# Ability Plan: Splice

**Generated**: 2026-03-06
**CR**: 702.47
**Priority**: P4
**Similar abilities studied**: Kicker (additional cost + effect modification, `casting.rs:1768-1805`, `resolution.rs:198-213`), Replicate (additional cost + copies, `casting.rs:1809-1844`, `command.rs:212-219`)

## CR Rule Text

702.47. Splice

702.47a Splice is a static ability that functions while a card is in your hand. "Splice onto [quality] [cost]" means "You may reveal this card from your hand as you cast a [quality] spell. If you do, that spell gains the text of this card's rules text and you pay [cost] as an additional cost to cast that spell." Paying a card's splice cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

702.47b You can't choose to use a splice ability if you can't make the required choices (targets, etc.) for that card's rules text. You can't splice any one card onto the same spell more than once. If you're splicing more than one card onto a spell, reveal them all at once and choose the order in which their effects will happen. The effects of the main spell must happen first.

702.47c The spell has the characteristics of the main spell, plus the rules text of each of the spliced cards. This is a text-changing effect (see rule 612, "Text-Changing Effects"). The spell doesn't gain any other characteristics (name, mana cost, color, supertypes, card types, subtypes, etc.) of the spliced cards. Text gained by the spell that refers to a card by name refers to the spell on the stack, not the card from which the text was copied.

702.47d Choose targets for the added text normally (see rule 601.2c). Note that a spell with one or more targets won't resolve if all of its targets are illegal on resolution.

702.47e The spell loses any splice changes once it leaves the stack for any reason.

### Related Rules

601.2b: "If the player wishes to splice any cards onto the spell (see rule 702.47), they reveal those cards in their hand." Splice is announced at the same time as mode choices, before costs are paid.

612.10: "A splice ability changes a spell's text by adding the rules text of the card with splice to the spell, following that spell's own rules text. It doesn't modify or replace any of that spell's own text."

## Key Edge Cases

- **Card stays in hand (702.47a)**: The spliced card is revealed but never leaves the player's hand. It is NOT cast. No "whenever you cast" triggers fire for it.
- **Cannot splice onto itself (ruling)**: "A card with a splice ability can't be spliced onto itself because the spell is on the stack (and not in your hand) when you reveal the cards you want to splice onto it."
- **One splice per card per spell (702.47b)**: You can't splice the same card onto the same spell more than once, but you CAN splice multiple different cards onto one spell.
- **Effect order (702.47b)**: Main spell effects happen first; then spliced effects in the order chosen by the caster.
- **Characteristics unchanged (702.47c)**: The spell only gains rules text (effects), not name, mana cost, color, types, or subtypes of the spliced card.
- **Targets for spliced text (702.47d)**: The spliced card's targeting requirements must be satisfiable at announcement time, and targets are chosen for the added text as part of casting. If ALL targets (main + spliced) are illegal at resolution, the spell fizzles.
- **Splice lost on stack departure (702.47e)**: If the spell is bounced, countered, or otherwise leaves the stack, the splice modifications are gone. The spliced card remains in hand regardless.
- **Subtype matching**: "Splice onto Arcane" requires the target spell to have the Arcane subtype. The engine checks `subtypes` on the spell's characteristics.
- **Additional cost (702.47a)**: The splice cost is paid as an additional cost (CR 601.2f-h), NOT an alternative cost. It adds to the total mana cost of the spell.
- **Multiplayer**: No special multiplayer considerations beyond normal targeting rules.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Splice` variant with doc comment citing CR 702.47.
**Pattern**: Follow `KeywordAbility::Replicate` at the end of the enum (around line 963).
**CR**: 702.47a -- "Splice is a static ability that functions while a card is in your hand."

**Note**: Splice is a marker keyword on the card. The splice cost and subtype qualifier are stored in `AbilityDefinition::Splice { cost, onto_subtype }` (see Step 1b).

**Discriminant**: KeywordAbility discriminant 109.

#### Step 1b: AbilityDefinition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Splice { cost: ManaCost, onto_subtype: SubType, effect: Effect }` variant.
**Pattern**: Follow `AbilityDefinition::Replicate { cost: ManaCost }` (around line 423).
**CR**: 702.47a -- stores the splice cost, the required subtype (e.g., "Arcane"), and the effect text (rules text of the spliced card) that gets appended to the target spell.

The `effect` field is the key differentiator from Kicker/Replicate: the spliced card's entire rules text (as an `Effect`) is added to the target spell's resolution. This is the engine's representation of CR 702.47c's "gains the text of this card's rules text."

**Discriminant**: AbilityDefinition hash discriminant 38.

#### Step 1c: Hash Implementation

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Splice => 109u8.hash_into(hasher)` in the KeywordAbility HashInto impl. Add `AbilityDefinition::Splice { cost, onto_subtype, effect }` arm with discriminant 38u8 in the AbilityDefinition HashInto impl, hashing cost, onto_subtype, and effect.
**Pattern**: Follow Replicate/Cleave hashing pattern.

#### Step 1d: CastSpell Command Field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `splice_cards: Vec<ObjectId>` field to `CastSpell` command (default empty vec). Each ObjectId is a card in the caster's hand that they wish to splice onto this spell.
**Pattern**: Follow `replicate_count: u32` field (around line 218).
**CR**: 601.2b -- player announces splice choices at cast time.

**Note**: The Vec order determines effect order (702.47b: main spell first, then spliced effects in announced order).

#### Step 1e: StackObject Field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `spliced_effects: Vec<(Effect, Vec<SpellTarget>)>` field to `StackObject` (default empty vec). Each entry is an effect from a spliced card paired with its targets. These are appended to the spell's effect execution at resolution time (CR 702.47b).
**Pattern**: Add after `was_cleaved` field (around line 224).
**CR**: 702.47e -- "The spell loses any splice changes once it leaves the stack for any reason." Stored on the StackObject, which is dropped when the spell leaves the stack.

Also add `spliced_card_ids: Vec<ObjectId>` to track which cards were spliced (for display/debugging, and to ensure 702.47b's "can't splice the same card more than once" validation at cast time). This stores the ObjectIds of the cards in hand that were spliced.

**Hash**: Add both new fields to the StackObject HashInto impl in `hash.rs`.

#### Step 1f: Update All StackObject Construction Sites

**Action**: Grep for all sites that construct `StackObject { ... }` and add `spliced_effects: vec![], spliced_card_ids: vec![]` to each.
**Files**: `crates/engine/src/rules/casting.rs` (multiple sites), `crates/engine/src/rules/resolution.rs` (suspend cast, copy creation sites).
**Pattern**: Follow how `was_cleaved: false` was added to all construction sites.

#### Step 1g: Update TUI stack_view.rs

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No new `StackObjectKind` variant is needed for Splice (unlike Replicate which has a trigger). Splice is handled entirely within the existing `Spell` kind by appending effects. No TUI changes needed unless there's an exhaustive match on StackObject fields (unlikely).

### Step 2: Rule Enforcement (Casting)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In `handle_cast_spell`, after the replicate cost block (around line 1844), add a splice validation and cost block.

**CR 702.47a validation**:
1. For each `splice_card` ObjectId in `splice_cards`:
   a. Verify the card is in the caster's hand (not the card being cast -- CR ruling: can't splice onto itself).
   b. Verify the card has `KeywordAbility::Splice` in its keywords.
   c. Look up `AbilityDefinition::Splice { cost, onto_subtype, effect }` from the card registry.
   d. Verify the spell being cast has `onto_subtype` in its characteristics' `subtypes` (e.g., check `SubType("Arcane".to_string())` is in `chars.subtypes`).
   e. Verify no duplicate ObjectIds in `splice_cards` (702.47b: can't splice the same card more than once).
   f. Add the splice `cost` to the total mana cost (additional cost, CR 601.2f-h).
   g. Collect the `effect` from each spliced card for later attachment to the StackObject.

**CR 702.47d targeting**: Targets for spliced effects need to be chosen at cast time. This requires extending the `targets` field to include targets for spliced effects. For the initial implementation, spliced effects that need targets will use the `targets` vec on the CastSpell command -- targets are ordered: main spell targets first, then spliced card targets in splice order. The casting code will validate and distribute targets to spliced effects accordingly.

**Alternative simplified targeting**: For the initial implementation (P4 priority), store targets for spliced effects as additional entries in the main `targets` vec. At resolution time, the effect execution context routes targets to the correct effect. This mirrors how the engine already handles multi-target spells.

**Key function**: Add `fn get_splice_info(card_id: &Option<CardId>, registry: &CardRegistry) -> Option<(ManaCost, SubType, Effect)>` helper (follow `get_kicker_cost` and `get_replicate_cost` patterns).

**Mana cost pipeline position**: Splice costs are additional costs (CR 601.2f-h), same category as kicker. Insert after replicate cost addition, before affinity/undaunted/convoke reductions. Pipeline order:
```
base_mana_cost -> alt_cost -> commander_tax -> kicker -> replicate -> SPLICE -> affinity -> undaunted -> convoke -> improvise -> delve -> assist -> pay
```

**StackObject construction**: When building the StackObject for the spell, populate `spliced_effects` with the collected (effect, targets) pairs and `spliced_card_ids` with the splice card ObjectIds.

### Step 3: Rule Enforcement (Resolution)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: After the main spell effect executes (around line 213, after `execute_effect`), iterate over `stack_obj.spliced_effects` and execute each spliced effect in order.

**CR 702.47b**: "The effects of the main spell must happen first." This is naturally handled by executing spliced effects after the main effect.

**Implementation**:
```
// CR 702.47b: Execute spliced effects after main spell effect.
for (spliced_effect, spliced_targets) in &stack_obj.spliced_effects {
    let mut splice_ctx = EffectContext::new(
        controller,
        source_object,  // CR 702.47c: text refers to the spell, not the spliced card
        spliced_targets.clone(),
    );
    // Propagate kicker/overload/bargain/cleave from the main spell
    splice_ctx.was_overloaded = stack_obj.was_overloaded;
    splice_ctx.was_bargained = stack_obj.was_bargained;
    splice_ctx.was_cleaved = stack_obj.was_cleaved;
    let splice_events = execute_effect(state, spliced_effect, &mut splice_ctx);
    events.extend(splice_events);
}
```

**CR 702.47c**: "Text gained by the spell that refers to a card by name refers to the spell on the stack, not the card from which the text was copied." In the engine, effects use `EffectTarget::Source` which resolves to `ctx.source` -- since we set `ctx.source = source_object` (the spell's source), this is automatically correct.

**CR 702.47e**: No special handling needed -- `spliced_effects` lives on the StackObject which is consumed/dropped when the spell resolves, is countered, or fizzles. The splice modifications are inherently lost.

### Step 4: Trigger Wiring

**Not applicable.** Splice is not a triggered ability. It is a static ability that functions during casting (702.47a). The spliced card stays in hand (no zone change, no trigger). No `StackObjectKind` variant needed, no trigger dispatch needed.

### Step 5: Harness Support

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `"cast_spell_splice"` action type that accepts `splice_card_names: Vec<String>` (names of cards in hand to splice). The harness resolves each name to an ObjectId in the caster's hand and passes them as `splice_cards` in the `CastSpell` command.

**Pattern**: Follow `"cast_spell_replicate"` (around line 1244).

**Also**: Update `translate_player_action` to pass `splice_cards: vec![]` in all existing `CastSpell` action types (same pattern as how `replicate_count: 0` was added to all existing paths).

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add `splice_card_names: Option<Vec<String>>` to the action schema.

### Step 6: Unit Tests

**File**: `crates/engine/tests/splice.rs`
**Tests to write**:

1. `test_splice_basic_onto_arcane` -- CR 702.47a: Cast an Arcane spell, splice Glacial Ray onto it. Verify: Glacial Ray stays in hand, target takes 2 damage from spliced effect, main spell effect also resolves.

2. `test_splice_cost_added` -- CR 702.47a/601.2f: Verify that the splice cost is added to the total mana cost. Casting an Arcane spell ({1}{U}) with splice ({1}{R}) requires {1}{U} + {1}{R} = {2}{U}{R} total.

3. `test_splice_card_stays_in_hand` -- CR 702.47a: After resolution, the spliced card remains in the caster's hand (it was revealed but not cast, not discarded, not exiled).

4. `test_splice_wrong_subtype_rejected` -- CR 702.47a: Attempt to splice onto a non-Arcane instant/sorcery. Verify the engine rejects with an error.

5. `test_splice_same_card_twice_rejected` -- CR 702.47b: Attempt to splice the same card onto a spell twice. Verify rejection.

6. `test_splice_not_in_hand_rejected` -- CR 702.47a: Attempt to splice a card that isn't in the caster's hand (e.g., on the battlefield or in graveyard). Verify rejection.

7. `test_splice_multiple_cards` -- CR 702.47b: Splice two different cards onto one Arcane spell. Verify both effects resolve after the main effect, in the declared order.

8. `test_splice_main_effect_first` -- CR 702.47b: Verify that the main spell's effect resolves before any spliced effects.

9. `test_splice_onto_itself_rejected` -- Ruling: The spell being cast can't have itself spliced onto it (it's on the stack, not in hand). This is naturally handled by checking that splice cards are in hand, but worth a test.

**Pattern**: Follow tests in `crates/engine/tests/replicate.rs` and `crates/engine/tests/kicker.rs` for test structure (GameStateBuilder setup, card registration, CastSpell command, assertion of effects and hand contents).

**Test setup**: Tests need:
- At least one card with `SubType("Arcane".to_string())` in its type line (the target spell).
- At least one card with `AbilityDefinition::Splice { cost, onto_subtype, effect }` and `KeywordAbility::Splice` (the splice source).
- For basic tests, define test-only CardDefinitions inline (no need for full card defs yet).

### Step 7: Card Definition (later phase)

**Suggested card**: Glacial Ray
- Type: Instant -- Arcane
- Cost: {1}{R}
- Effect: Deal 2 damage to any target
- Splice onto Arcane {1}{R}
- Use `card-definition-author` agent

**Also needed for testing**: A simple Arcane instant to cast (e.g., "Lava Spike" -- Sorcery Arcane, {R}, deal 3 damage to target player).

### Step 8: Game Script (later phase)

**Suggested scenario**: P1 has Glacial Ray and an Arcane spell (e.g., Lava Spike) in hand. P1 casts the Arcane spell targeting P2, splicing Glacial Ray onto it. After resolution: P2 takes damage from both the main spell and Glacial Ray's spliced effect, and Glacial Ray remains in P1's hand.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Splice + Storm/Replicate**: If a spell with spliced effects is copied (e.g., by Storm), do the copies also have the spliced effects? Per CR 707.2, copies copy the characteristics of the spell on the stack, and 702.47c says the spell gains the rules text. So copies should include the spliced text. However, 702.47e says "loses splice changes once it leaves the stack" -- copies are new stack objects, not the same one. This is an edge case to test later but NOT required for initial implementation.
- **Splice + Kicker**: A spell can be both kicked and have cards spliced onto it. The kicker cost and splice costs are all additional costs (CR 601.2f-h). Total cost = base + kicker + splice costs.
- **Splice + Counterspell**: If the spell is countered, splice modifications are lost (702.47e), but the spliced card stays in hand. The caster doesn't "lose" the spliced card.
- **Splice + Flashback**: A spell cast with flashback can still have cards spliced onto it (splice is an additional cost, not an alternative cost). The spliced cards stay in hand; the flashback spell is exiled.
- **Splice cost reduction**: Splice costs are subject to cost reduction effects that affect additional costs (like Helm of Awakening reducing all spells). This is handled by the cost pipeline naturally.
- **Targeting for spliced effects**: CR 702.47d says targets are chosen normally. If the spliced effect requires targets and no legal targets exist, the player can't choose to splice that card (702.47b: "can't choose to use a splice ability if you can't make the required choices").
- **Multiplayer**: No special considerations beyond normal targeting rules. Any player can be targeted by spliced effects.

## Design Decision: Effect Storage

The key design decision is how to represent the "added rules text" on the stack object. Two approaches:

**Chosen approach**: Store `spliced_effects: Vec<(Effect, Vec<SpellTarget>)>` directly on `StackObject`. At resolution, iterate and execute each effect after the main effect. This is simple, explicit, and naturally handles 702.47e (effects vanish when the StackObject is consumed).

**Alternative (rejected)**: Modify the spell's `Spell` effect to be a `Sequence` wrapping the original effect + spliced effects. This would require mutating the card definition at cast time, which violates the immutable-card-definition principle and makes it harder to distinguish main vs. spliced effects.

## Discriminant Summary

- KeywordAbility::Splice = 109
- AbilityDefinition::Splice = 38
- No new StackObjectKind variant needed
