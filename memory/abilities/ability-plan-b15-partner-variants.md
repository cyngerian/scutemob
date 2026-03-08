# Ability Plan: Batch 15 -- Commander Partner Variants

**Generated**: 2026-03-08
**CR**: 702.124 (subrules i, k, m)
**Priority**: P4
**Similar abilities studied**: Partner (`KeywordAbility::Partner`, `state/types.rs:259`), PartnerWith (`KeywordAbility::PartnerWith(String)`, `state/types.rs:273`), validation at `rules/commander.rs:485-577`

## CR Rule Text

Full text from CR 702.124 (relevant subrules for B15):

> **702.124a** Partner abilities are keyword abilities that modify the rules for deck construction in the Commander variant (see rule 903), and they function before the game begins. Each partner ability allows you to designate two legendary cards as your commander rather than one. Each partner ability has its own requirements for those two commanders. The partner abilities are: partner, partner--[text], partner with [name], choose a Background, and Doctor's companion.

> **702.124b** Your deck must contain exactly 100 cards, including its two commanders. Both commanders begin the game in the command zone.

> **702.124c** A rule or effect that refers to your commander's color identity refers to the combined color identities of your two commanders. See rule 903.4.

> **702.124d** Except for determining the color identity of your commander, the two commanders function independently. When casting a commander with partner, ignore how many times your other commander has been cast (see rule 903.8). When determining whether a player has been dealt 21 or more combat damage by the same commander, consider damage from each of your two commanders separately (see rule 903.10a).

> **702.124f** Different partner abilities are distinct from one another and cannot be combined. For example, you cannot designate two cards as your commander if one of them has "partner" and the other has "partner with [name]."

> **702.124g** If a legendary card has more than one partner ability, you may choose which one to use when designating your commander, but you can't use both. Notably, no partner ability or combination of partner abilities can ever let a player have more than two commanders.

> **702.124i** "Partner--[text]" means "You may designate two legendary cards as your commander rather than one if each of them has the same 'partner--[text]' ability." The "partner--[text]" abilities are "partner--Father & son," "partner--Friends forever," and "partner--Survivors."

> **702.124k** "Choose a Background" means "You may designate two cards as your commander rather than one if one of them is this card and the other is a legendary Background enchantment card." You can't designate two cards as your commander if one has a "choose a Background" ability and the other is not a legendary Background enchantment card, and legendary Background enchantment cards can't be your commander unless you have also designated a commander with "choose a Background."

> **702.124m** "Doctor's companion" means "You may designate two legendary creature cards as your commander rather than one if one of them is this card and the other is a legendary Time Lord Doctor creature card that has no other creature types."

## Key Edge Cases

- **CR 702.124f**: Different partner abilities CANNOT be combined. FriendsForever + Partner = invalid. ChooseABackground + DoctorsCompanion = invalid. Each variant only pairs with its matching type.
- **CR 702.124g**: A card with multiple partner abilities can use only one.
- **CR 702.124i**: "Friends Forever" is specifically a "partner--[text]" variant. Both commanders must have the same "partner--Friends forever" ability. This is structurally identical to how plain Partner works (both must have it).
- **CR 702.124k**: "Choose a Background" is ASYMMETRIC -- one commander is a legendary creature with "Choose a Background", the other is a "legendary Background enchantment card" (NOT a creature). This means the commander type validation (lines 103-121 of `commander.rs`) must be relaxed for the Background enchantment commander. The Background enchantment does NOT need the "Choose a Background" keyword itself -- it qualifies by being a legendary enchantment with the Background subtype.
- **CR 702.124m**: "Doctor's Companion" is also ASYMMETRIC -- one commander has "Doctor's companion", the other must be a "legendary Time Lord Doctor creature card that has no other creature types." The Doctor commander does NOT need "Doctor's companion" -- it qualifies by being a legendary creature with subtypes exactly [Time Lord, Doctor] and no other creature types.
- **Background is an enchantment subtype** -- represented as `SubType("Background".to_string())` in the engine's `SubType(String)` model.
- **"Time Lord" and "Doctor" are creature subtypes** -- `SubType("Time Lord".to_string())` and `SubType("Doctor".to_string())`.
- **No other creature types check for Doctor's Companion**: The Doctor must have creature types that are exactly {Time Lord, Doctor} -- if it has any additional creature types (e.g., from Changeling), it would NOT qualify. However, per CR 702.124m this is a deck-construction check and uses printed types, not layer-resolved types, so Changeling does not interfere at deck validation time.
- **Multiplayer**: No special multiplayer considerations beyond what Partner already handles. All partner variants use the same infrastructure for combined color identity, independent tax, independent commander damage.

## Current State (from ability-wip.md)

All three abilities are at `none` status -- no work has been done.

- [ ] Step 1: Enum variants (3 new `KeywordAbility` variants)
- [ ] Step 2: Hash support
- [ ] Step 3: View model / TUI match arms
- [ ] Step 4: Validation logic in `commander.rs`
- [ ] Step 5: Relaxed commander-type check for Background enchantments
- [ ] Step 6: Unit tests

## Implementation Steps

### Step 1: Add KeywordAbility Variants

**File**: `crates/engine/src/state/types.rs`
**Action**: Add 3 new variants after `PartnerWith(String)` (currently at line 273):

```rust
/// CR 702.124i: "Partner--Friends forever" — both commanders must have this ability.
/// Structurally identical to plain Partner but distinct (CR 702.124f).
FriendsForever,
/// CR 702.124k: "Choose a Background" — this commander pairs with a legendary
/// Background enchantment card as the second commander. The Background does NOT
/// need this keyword; it qualifies by being a legendary enchantment with subtype
/// Background.
ChooseABackground,
/// CR 702.124m: "Doctor's companion" — this commander pairs with a legendary
/// Time Lord Doctor creature card that has no other creature types.
/// The Doctor does NOT need this keyword; it qualifies by type.
DoctorsCompanion,
```

**Discriminants**: KW 144 (FriendsForever), 145 (ChooseABackground), 146 (DoctorsCompanion).
**No new AbilityDefinition or StackObjectKind discriminants needed** -- these are deck-validation-only keywords with no in-game triggers or abilities.

### Step 2: Hash Support

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add 3 arms to the `KeywordAbility` match in the `HashInto` impl.
**Pattern**: Follow `KeywordAbility::Partner => 22u8.hash_into(hasher)` at line 357.
**Values**: `FriendsForever => 144u8`, `ChooseABackground => 145u8`, `DoctorsCompanion => 146u8`.

### Step 3: View Model and TUI Match Arms

**File 1**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add 3 arms to the `KeywordAbility` display match (near line 761 where `Partner` is).
- `KeywordAbility::FriendsForever => "Friends forever".to_string()`
- `KeywordAbility::ChooseABackground => "Choose a Background".to_string()`
- `KeywordAbility::DoctorsCompanion => "Doctor's companion".to_string()`

**File 2**: `tools/tui/src/play/panels/stack_view.rs`
**Note**: No changes needed here -- these keywords produce no new `StackObjectKind` variants. The TUI's `StackObjectKind` match is unaffected.

### Step 4: Validation Logic in `commander.rs`

**File**: `crates/engine/src/rules/commander.rs`
**Action**: Extend `validate_partner_commanders()` (lines 485-577) with 3 new cases.

The function currently handles:
- Case 1: Both have plain `Partner` (CR 702.124h)
- Case 2: Both have `PartnerWith` with cross-referenced names (CR 702.124j)
- Case 3: Mixed Partner + PartnerWith rejected (CR 702.124f)
- Case 4: Incomplete PartnerWith pair
- Case 5: Neither has Partner

**Add after existing cases (before the final fallthrough):**

**Case: Friends Forever (CR 702.124i)**
- Check if both commanders have `KeywordAbility::FriendsForever`.
- If both have it, return `Ok(())`.
- This is structurally identical to the plain Partner check.

**Case: Choose a Background (CR 702.124k)**
- Check if one commander has `KeywordAbility::ChooseABackground`.
- The OTHER commander must be a legendary enchantment with subtype "Background".
- It does NOT need `ChooseABackground` keyword itself.
- If one has `ChooseABackground` and the other is a valid legendary Background enchantment, return `Ok(())`.
- Note: The function takes `CardDefinition` refs, so we can check `def.types.card_types`, `def.types.supertypes`, and `def.types.subtypes` directly.

**Case: Doctor's Companion (CR 702.124m)**
- Check if one commander has `KeywordAbility::DoctorsCompanion`.
- The OTHER commander must be a legendary creature with creature subtypes exactly {Time Lord, Doctor} and no other creature types.
- "No other creature types" means: filter the subtypes to only creature subtypes (this is tricky since `SubType` is a flat string). For deck validation, we check the printed subtypes on the `CardDefinition`. The Doctor will have subtypes like `[Time Lord, Doctor]` -- we need to verify these are the ONLY creature subtypes present. Since all subtypes (creature, enchantment, land, etc.) are stored in the same `subtypes` set as `SubType(String)`, we need to check that the subtypes include both "Time Lord" and "Doctor", and that there are no other creature subtypes.
- **Practical approach**: Since `SubType` is a simple string wrapper, and we don't have a creature-subtype-vs-enchantment-subtype distinction, check: `subtypes` contains `SubType("Time Lord")` AND `SubType("Doctor")`, AND the card is a legendary creature. The "no other creature types" check is best done by checking that the ONLY subtypes present are "Time Lord" and "Doctor" (since creature cards typically only have creature subtypes). If a Doctor card also has a planeswalker subtype or similar, that would be stored in the same set, but that edge case does not arise in practice for Doctor Who commanders.
- If one has `DoctorsCompanion` and the other is a valid Doctor, return `Ok(())`.

**CR 702.124f enforcement**: After all positive matches, if one commander has ANY partner variant but the pair doesn't match, produce an error. The existing fallthrough at lines 559-576 already handles this for Partner/PartnerWith. Extend it to also detect FriendsForever, ChooseABackground, and DoctorsCompanion so the error message is accurate.

**Implementation pattern**: Add helper functions or detection variables at the top of `validate_partner_commanders`:

```rust
let cmd1_has_friends_forever = cmd1.abilities.iter().any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::FriendsForever)));
let cmd2_has_friends_forever = cmd2.abilities.iter().any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::FriendsForever)));

let cmd1_has_choose_background = cmd1.abilities.iter().any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::ChooseABackground)));
let cmd2_has_choose_background = cmd2.abilities.iter().any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::ChooseABackground)));

let cmd1_has_doctors_companion = cmd1.abilities.iter().any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::DoctorsCompanion)));
let cmd2_has_doctors_companion = cmd2.abilities.iter().any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::DoctorsCompanion)));
```

Then add validation cases in order.

**Helper function** `is_legendary_background(def: &CardDefinition) -> bool`:
```rust
def.types.supertypes.contains(&SuperType::Legendary)
    && def.types.card_types.contains(&CardType::Enchantment)
    && def.types.subtypes.contains(&SubType("Background".to_string()))
```

**Helper function** `is_time_lord_doctor(def: &CardDefinition) -> bool`:
```rust
def.types.supertypes.contains(&SuperType::Legendary)
    && def.types.card_types.contains(&CardType::Creature)
    && def.types.subtypes.contains(&SubType("Time Lord".to_string()))
    && def.types.subtypes.contains(&SubType("Doctor".to_string()))
    // "no other creature types" -- for deck validation, we check printed subtypes.
    // Doctor Who Doctor cards are printed with exactly {Time Lord, Doctor}.
    // A more robust check would filter to only creature subtypes, but since
    // SubType is a flat string and we lack a creature-subtype registry, checking
    // that exactly these two are present is sufficient for known Doctor cards.
```

### Step 5: Relaxed Commander-Type Check

**File**: `crates/engine/src/rules/commander.rs`
**Location**: Lines 103-121 (the per-commander type validation loop)
**Action**: When validating partner pairs with "Choose a Background", the Background enchantment commander must NOT fail the `is_legendary && is_creature` check.

**Approach**: Move the partner validation BEFORE the per-commander type check, and pass information about the partner type back. OR: when checking each commander's type, also check if this commander is allowed to be a non-creature by virtue of being a valid Background in a "Choose a Background" pair.

**Recommended approach**: After the partner validation succeeds and we know the pairing type, skip the creature check for the Background commander. This requires restructuring slightly:

1. Run `validate_partner_commanders` first (already at line 77-86).
2. Extend it to return a `PartnerType` enum: `PlainPartner`, `FriendsForever`, `PartnerWith`, `ChooseABackground`, `DoctorsCompanion`, or `None` (for single commander).
3. In the per-commander loop, if the partner type is `ChooseABackground`, allow one commander to be a legendary enchantment (not creature) as long as it has the Background subtype.

**Simpler alternative**: Change `validate_partner_commanders` to return `Result<PartnerVariant, String>` where `PartnerVariant` indicates which type matched. Then in the commander-type loop, check whether the current commander is the Background half and skip the creature requirement if so.

**Simplest alternative (recommended for minimal diff)**: In the commander-type loop (lines 103-121), add a special case: if we have 2 commanders, one has `ChooseABackground`, and the current commander is a legendary enchantment with Background subtype, skip the creature check.

```rust
// In the per-commander loop, before the is_legendary || is_creature check:
let is_background_commander = commander_card_ids.len() == 2
    && def.types.card_types.contains(&CardType::Enchantment)
    && def.types.subtypes.contains(&SubType("Background".to_string()))
    && def.types.supertypes.contains(&SuperType::Legendary);

if !is_background_commander && (!is_legendary || !is_creature) {
    // existing error push
}
```

This works because `validate_partner_commanders` already verifies that if one commander is a Background enchantment, the other MUST have `ChooseABackground`. A Background enchantment without a matching `ChooseABackground` partner will be rejected by `validate_partner_commanders` as "neither has partner."

**Wait -- ordering issue**: The partner validation at lines 77-86 runs BEFORE the per-commander type check at 103-121. If the partner validation rejects the pair (because neither has a recognized partner ability), the type check will ALSO fire (producing a second redundant violation for "not a creature"). This is fine -- both violations are correct and complementary. The Background enchantment truly is not a valid commander unless paired with ChooseABackground.

### Step 6: Unit Tests

**File**: `crates/engine/tests/partner_variants.rs` (new file)
**Pattern**: Follow `crates/engine/tests/partner_with.rs` and `crates/engine/tests/deck_validation.rs`

**Tests to write**:

#### Friends Forever (CR 702.124i)
- `test_friends_forever_both_have_ability_valid_pair` -- Two legendary creatures both with FriendsForever pass validation.
- `test_friends_forever_only_one_has_ability_rejected` -- One has FriendsForever, other has nothing -- rejected.
- `test_friends_forever_mixed_with_plain_partner_rejected` -- One has FriendsForever, other has Partner -- rejected (CR 702.124f).

#### Choose a Background (CR 702.124k)
- `test_choose_a_background_creature_plus_background_enchantment_valid` -- Creature with ChooseABackground + legendary Background enchantment passes validation.
- `test_choose_a_background_missing_background_subtype_rejected` -- Creature with ChooseABackground + legendary enchantment WITHOUT Background subtype -- rejected.
- `test_choose_a_background_non_legendary_background_rejected` -- Creature with ChooseABackground + non-legendary Background enchantment -- rejected.
- `test_choose_a_background_two_creatures_both_choose_rejected` -- Two creatures both with ChooseABackground but neither is a Background enchantment -- rejected (you need one creature, one Background).
- `test_choose_a_background_background_without_choose_ability_rejected` -- A Background enchantment paired with a creature that does NOT have ChooseABackground -- rejected.
- `test_choose_a_background_commander_type_check_allows_enchantment` -- Verify that `validate_deck` does NOT reject the Background enchantment as "not a creature."
- `test_choose_a_background_mixed_with_partner_rejected` -- One has ChooseABackground, other has Partner -- rejected (CR 702.124f).

#### Doctor's Companion (CR 702.124m)
- `test_doctors_companion_with_valid_doctor_valid_pair` -- Creature with DoctorsCompanion + legendary Time Lord Doctor creature passes validation.
- `test_doctors_companion_doctor_missing_time_lord_subtype_rejected` -- Doctor creature missing "Time Lord" subtype -- rejected.
- `test_doctors_companion_doctor_has_extra_creature_types_rejected` -- Doctor creature has {Time Lord, Doctor, Human} -- rejected ("no other creature types").
- `test_doctors_companion_only_companion_no_doctor_rejected` -- One has DoctorsCompanion, other is a regular legendary creature -- rejected.
- `test_doctors_companion_mixed_with_friends_forever_rejected` -- CR 702.124f cross-variant rejection.

#### Cross-variant rejection (CR 702.124f)
- `test_cross_variant_friends_forever_plus_choose_background_rejected` -- One has FriendsForever, other has ChooseABackground -- rejected.
- `test_cross_variant_friends_forever_plus_doctors_companion_rejected` -- Rejected.

**Test helpers needed** (add to test file):
- `legendary_creature_with_ability(id, name, cost, ability)` -- creates a legendary creature CardDefinition with a specific AbilityDefinition::Keyword.
- `legendary_background_enchantment(id, name, cost)` -- creates a legendary enchantment with SubType("Background").
- `legendary_time_lord_doctor(id, name, cost)` -- creates a legendary creature with subtypes {Time Lord, Doctor}.
- `legendary_time_lord_doctor_extra_type(id, name, cost, extra_subtype)` -- same but with an extra creature subtype.

### Step 7: Card Definition (not needed for B15)

Per the batch plan, B15 has 0 cards. These are deck-validation-only keywords. Card definitions for actual commander cards with these abilities (e.g., Eleven, the Mage; Dungeon Delver; The Thirteenth Doctor) would be authored in the W5 card authoring workstream when needed.

### Step 8: Game Script (not needed for B15)

Per the batch plan, B15 has no game scripts. These abilities have no in-game effects beyond deck construction rules. The deck validation is tested via unit tests, not game scripts.

## Interactions to Watch

- **Commander type validation ordering**: The per-commander type check (lines 103-121) must be aware that a Background enchantment can be a valid commander in a ChooseABackground pair. This is the only place where a non-creature can be a valid commander.
- **CR 702.124f enforcement**: The fallthrough error messages in `validate_partner_commanders` must be updated to mention all partner variant names for clear error reporting. Currently only mentions "Partner" and "Partner with [name]".
- **Combined color identity (CR 702.124c)**: Already handled by existing infrastructure -- the color identity computation in `validate_deck` (lines 123+) already unions both commanders' identities regardless of partner variant.
- **Independent tax/damage (CR 702.124d)**: Already handled by existing commander infrastructure -- commander tax and commander damage are tracked per-commander-card, not per-partner-pair.
- **Changeling and Doctor's Companion**: Changeling is a CDA that applies in all zones. A creature with Changeling has ALL creature types, including Time Lord and Doctor. However, CR 702.124m requires "no other creature types." A Changeling creature has ALL types, so it would fail the "no other creature types" check. At deck validation time, we use the CardDefinition's printed subtypes, and Changeling is expressed as a keyword, not as explicit subtypes -- so the check is against printed subtypes. A Changeling card's printed subtypes do NOT include Time Lord/Doctor (unless explicitly printed), so a Changeling creature would NOT qualify as a Doctor. This is correct behavior -- deck construction rules use printed characteristics.
- **"Time Lord" is a two-word subtype**: Ensure the `SubType("Time Lord".to_string())` comparison works correctly. Since `SubType` is a string wrapper, this should work fine as long as both sides use the exact same string.

## Summary of Changes by File

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | 3 new `KeywordAbility` variants (FriendsForever, ChooseABackground, DoctorsCompanion) |
| `crates/engine/src/state/hash.rs` | 3 new match arms (disc 144, 145, 146) |
| `tools/replay-viewer/src/view_model.rs` | 3 new `KeywordAbility` display arms |
| `crates/engine/src/rules/commander.rs` | Extended `validate_partner_commanders` + relaxed type check for Background |
| `crates/engine/tests/partner_variants.rs` | ~15 new unit tests (new file) |

No changes to: `AbilityDefinition`, `StackObjectKind`, `builder.rs`, `abilities.rs`, `combat.rs`, `resolution.rs`, `effects/mod.rs`, TUI `stack_view.rs`, or any game script infrastructure.
