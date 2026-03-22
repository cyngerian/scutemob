# Primitive Batch Plan: PB-22 S7 — Adventure (CR 715) + Dual-Zone Search

**Generated**: 2026-03-21
**Primitive**: Adventure casting mechanic (AltCostKind::Adventure, exile-on-resolution, cast creature from exile) + dual-zone search (SearchLibrary zone extension)
**CR Rules**: 715.1-715.5, 715.2a-715.2c, 715.3-715.3d, 715.4, 715.5, 601.3e
**Cards affected**: ~4 (2 existing fixes: Monster Manual, Lozhan; 2 new: Bonecrusher Giant, Lovestruck Beast)
**Dependencies**: None (this is the final PB-22 session, all prior PB sessions complete)
**Deferred items from prior PBs**: Adventure (cast from exile) deferred from PB-13m

## Primitive Specification

### 1. Adventure Casting (CR 715)

Adventurer cards have two sets of characteristics: the normal (creature/permanent) face and an
alternative Adventure face (always Instant or Sorcery subtyped "Adventure"). The Adventure
mechanic requires:

1. A new `adventure_face: Option<CardFace>` field on `CardDefinition` holding the Adventure half's
   characteristics (name, mana cost, types, oracle text, abilities).
2. `AltCostKind::Adventure` — when cast as an Adventure, the spell uses ONLY the adventure face's
   characteristics on the stack (CR 715.3a-715.3b).
3. On resolution, the controller exiles the card instead of putting it into the graveyard
   (CR 715.3d). This is a replacement of the normal instant/sorcery destination.
4. While the card is exiled this way, the controller may cast it as the creature half.
   It CANNOT be cast as an Adventure again from exile (CR 715.3d).
5. If countered or fizzled, the card goes to graveyard (NOT exile) — the exile only happens
   on successful resolution (CR 715.3d: "Instead of putting a spell that was cast as an
   Adventure into its owner's graveyard **as it resolves**").
6. In every zone except the stack (while cast as Adventure), the card has only its normal
   characteristics (CR 715.4).

**Key difference from Disturb**: Disturb uses `back_face` because it's a DFC. Adventure cards
are NOT DFCs — they have a single physical face with an inset text box. A separate
`adventure_face` field is cleaner than overloading `back_face`.

**Key difference from Flashback**: Flashback exiles on ALL stack departure (resolution, counter,
fizzle). Adventure exiles ONLY on successful resolution.

### 2. Dual-Zone Search

Extend `Effect::SearchLibrary` with an optional `also_search_graveyard: bool` field (default
false). When true, the search also considers cards in the player's graveyard as candidates.
This covers the "Search your library and/or graveyard" pattern (Finale of Devastation).

Using a boolean is simpler than a `Vec<Zone>` since "library + graveyard" is the only
multi-zone search pattern in MTG.

## CR Rule Text

### CR 715 — Adventurer Cards

- **715.1**: Adventurer cards have a two-part card frame, with a smaller frame inset within their text box.
- **715.2**: The text that appears in the inset frame on the left defines alternative characteristics that the object may have while it's a spell. The card's normal characteristics appear as usual, although with a smaller text box on the right.
- **715.2a**: If an effect refers to a card, spell, or permanent that "has an Adventure," it refers to an object that has the alternative characteristics of an Adventure spell, even if the object currently doesn't use them.
- **715.2b**: The existence and values of these alternative characteristics are part of the object's copiable values.
- **715.2c**: Although adventurer cards are printed with multiple sets of characteristics, each adventurer card is only one card.
- **715.3**: As a player plays an adventurer card, the player chooses whether they play the card normally or as an Adventure.
- **715.3a**: When casting an adventurer card as an Adventure, only the alternative characteristics are evaluated to see if it can be cast.
- **715.3b**: While on the stack as an Adventure, the spell has only its alternative characteristics.
- **715.3c**: If an Adventure spell is copied, the copy is also an Adventure.
- **715.3d**: Instead of putting a spell that was cast as an Adventure into its owner's graveyard as it resolves, its controller exiles it. For as long as that card remains exiled, that player may play it. It can't be cast as an Adventure this way, although other effects that allow a player to cast it may allow a player to cast it as an Adventure.
- **715.4**: In every zone except the stack, and while on the stack not as an Adventure, an adventurer card has only its normal characteristics.
- **715.5**: If an effect instructs a player to choose a card name and the player wants to choose an adventurer card's alternative name, the player may do so.

### CR 601.3e — Alternative Characteristics for Casting

Some rules and effects state that an alternative set of characteristics or a subset of characteristics are considered to determine if a card or copy of a card is legal to cast. These alternative characteristics replace the object's characteristics for this determination.

## Engine Changes

### Change 1: Add `adventure_face` field to CardDefinition

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `adventure_face: Option<CardFace>` field to `CardDefinition` struct, after `back_face`.
**CR**: 715.2 — the alternative characteristics of the Adventure half.

```rust
/// CR 715.2: Alternative characteristics for the Adventure half.
///
/// `None` for non-adventurer cards. `Some(face)` for adventurer cards.
/// The face holds the Adventure spell's name, mana cost, types (Instant/Sorcery — Adventure),
/// oracle text, and abilities (the Spell effect). On the stack when cast as an Adventure,
/// only these characteristics apply (CR 715.3b). In all other zones, only the main
/// face's characteristics apply (CR 715.4).
#[serde(default)]
pub adventure_face: Option<CardFace>,
```

Also add `adventure_face: None,` to the `Default` implementation (after `back_face: None,`).

### Change 2: Add `AltCostKind::Adventure` variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Adventure` variant to `AltCostKind` enum (after `Prototype`).
**CR**: 715.3 — casting as an Adventure is an alternative casting mode.

Note from Scryfall ruling 2022-06-10: "Casting a card as an Adventure isn't casting it for an
alternative cost." This is important — it means Adventure can technically be combined with
other alternative costs (like flashback). However, for simplicity, we model it as an
`AltCostKind` variant since it determines which characteristics are used on the stack.
The key rule is CR 715.3: "the player chooses whether they play the card normally or as
an Adventure." In practice, Adventure cards use the adventure half's mana cost, so
`AltCostKind::Adventure` is the correct abstraction.

### Change 3: Add `was_cast_as_adventure` boolean to StackObject

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_cast_as_adventure: bool` field to `StackObject`, after `was_cleaved`.
**CR**: 715.3d — tracks that this spell was cast as an Adventure so the exile-on-resolution
behavior applies.

```rust
/// CR 715.3d: If true, this spell was cast as an Adventure. On successful resolution,
/// the card is exiled instead of going to the graveyard. From exile, the controller
/// may cast the creature half (but NOT as an Adventure again).
///
/// If countered or fizzled, the card goes to graveyard normally (NOT exile).
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_cast_as_adventure: bool,
```

Also add `was_cast_as_adventure: false,` to the `Default` implementation.

### Change 4: Add `adventure_exiled_by` field to GameObject

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `adventure_exiled_by: Option<PlayerId>` field to `GameObject`.
**CR**: 715.3d — "For as long as that card remains exiled, that player may play it."
The PlayerId records which player exiled it (the spell's controller at resolution time).
Only that player may cast the creature from exile.

```rust
/// CR 715.3d: If set, this card in exile was exiled as a resolved Adventure spell.
/// The value is the player who may cast it from exile (the spell's controller at
/// resolution time). The card can only be cast as the creature half, NOT as an Adventure.
/// Cleared when the card leaves exile for any reason (CR 400.7: new object).
#[serde(default)]
pub adventure_exiled_by: Option<PlayerId>,
```

### Change 5: Casting validation — `casting.rs`

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add Adventure casting validation. Three insertion points:

**5a**: After `let cast_with_disturb = ...` (~line 169), add:
```rust
let cast_with_adventure = alt_cost == Some(AltCostKind::Adventure);
```

**5b**: In the zone validation section (~line 457-470), add Adventure to the allowlist:
Adventure can be cast from hand (normal) AND from exile (if `adventure_exiled_by` is set and
the player matches). When casting the creature from exile, `alt_cost` should be `None` (normal
cast, not Adventure).

Add validation: if `cast_with_adventure`, verify the card has an `adventure_face` on its
CardDefinition. If casting from exile without `cast_with_adventure`, verify
`adventure_exiled_by == Some(player)` on the object.

**5c**: In the alternative cost mana calculation section (~line 2130-2160), add Adventure cost
resolution. When `cast_with_adventure`, use the `adventure_face.mana_cost` instead of the
card's normal mana cost.

**5d**: In the alt-cost mutual exclusion section (~line 1864-1980), add Adventure validation.
Adventure is effectively an alternative casting mode but NOT technically an alternative cost
per Scryfall ruling. For simplicity, treat it as mutually exclusive with other alt costs
(flashback, disturb, etc.) since you can't cast an Adventure from exile as an Adventure
again (CR 715.3d).

**5e**: In the zone guard (~line 457-470), add a path allowing cast from exile when
`adventure_exiled_by == Some(player)` and `alt_cost` is `None` (casting creature from exile).

### Change 6: Stack object creation — `casting.rs`

**File**: `crates/engine/src/rules/casting.rs`
**Action**: At the StackObject creation point (~line 3720), set `was_cast_as_adventure`.

At ~line 3729: `was_cast_as_adventure: cast_with_adventure,`

### Change 7: Resolution — exile on successful resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: At the instant/sorcery post-resolution destination (~line 1697-1708), add
Adventure exile. Insert `was_cast_as_adventure` check BEFORE the graveyard fallback.

```rust
let destination = if stack_obj.cast_with_flashback
    || stack_obj.cast_with_jump_start
    || (has_cipher && cipher_creature.is_some())
{
    ZoneId::Exile
} else if stack_obj.was_cast_as_adventure {
    // CR 715.3d: Adventure spell exiles on resolution (not graveyard).
    ZoneId::Exile
} else if stack_obj.was_buyback_paid {
    ZoneId::Hand(owner)
} else {
    ZoneId::Graveyard(owner)
};
```

After the move_object_to_zone call, if `was_cast_as_adventure`, set `adventure_exiled_by`:
```rust
if stack_obj.was_cast_as_adventure {
    if let Some(obj) = state.objects.get_mut(&new_id) {
        obj.adventure_exiled_by = Some(controller);
    }
}
```

**IMPORTANT**: Do NOT add `was_cast_as_adventure` to the fizzle path (~line 94-96) or the
counter path (~line 7299). Adventure only exiles on successful resolution. When countered or
fizzled, the card goes to graveyard like any other spell.

### Change 8: Resolution — permanent enters battlefield from adventure exile

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: At the permanent ETB section (~line 540-600), when a spell resolves as a permanent
(creature) that was cast from exile via `adventure_exiled_by`, clear the
`adventure_exiled_by` flag on the new object (it's a new object via CR 400.7, so this
happens naturally — but verify the flag is NOT propagated).

No additional change needed here because `adventure_exiled_by` defaults to `None` and
new objects start clean.

### Change 9: Fizzle and counter paths — NO adventure exile

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Verify that the fizzle path (~line 94) and counter path (~line 7299) do NOT
include `was_cast_as_adventure` in the exile conditions. These should already work correctly
since `was_cast_as_adventure` is not `cast_with_flashback` or `cast_with_jump_start`.

Just add a comment at each site for clarity:
```rust
// NOTE: Adventure spells go to graveyard when countered/fizzled,
// NOT exile (CR 715.3d: exile only on resolution).
```

### Change 10: Hash updates

**File**: `crates/engine/src/state/hash.rs`

**10a**: AltCostKind::Adventure — discriminant 27

In the `impl HashInto for AltCostKind` match (~line 2558, after Prototype => 26):
```rust
AltCostKind::Adventure => 27,
```

**10b**: `was_cast_as_adventure` on StackObject

Find the StackObject `HashInto` impl and add:
```rust
self.was_cast_as_adventure.hash_into(hasher);
```

**10c**: `adventure_exiled_by` on GameObject

Find the GameObject `HashInto` impl and add:
```rust
self.adventure_exiled_by.hash_into(hasher);
```

**10d**: `adventure_face` on CardDefinition

Find the CardDefinition `HashInto` impl and add:
```rust
self.adventure_face.hash_into(hasher);
```

### Change 11: Dual-zone search — extend `Effect::SearchLibrary`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `also_search_graveyard: bool` field to `Effect::SearchLibrary` variant:

```rust
SearchLibrary {
    player: PlayerTarget,
    filter: TargetFilter,
    reveal: bool,
    destination: ZoneTarget,
    #[serde(default)]
    shuffle_before_placing: bool,
    /// CR 701.23: If true, also search the player's graveyard (in addition to library).
    /// Used by "Search your library and/or graveyard" effects (e.g., Finale of Devastation).
    #[serde(default)]
    also_search_graveyard: bool,
},
```

### Change 12: Dual-zone search — execution in `effects/mod.rs`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: At the SearchLibrary execution (~line 1896), when `also_search_graveyard` is true,
also collect candidates from `ZoneId::Graveyard(p)` in addition to `ZoneId::Library(p)`.

```rust
let lib_id = ZoneId::Library(p);
let gy_id = ZoneId::Graveyard(p);
let mut candidates: Vec<(ObjectId, ZoneId)> = state
    .objects
    .iter()
    .filter(|(_, obj)| {
        let in_lib = obj.zone == lib_id;
        let in_gy = *also_search_graveyard && obj.zone == gy_id;
        (in_lib || in_gy) && matches_filter(&obj.characteristics, filter)
    })
    .map(|(id, obj)| (*id, obj.zone.clone()))
    .collect();
```

When moving the found card, only shuffle the library (not the graveyard). The shuffle
still applies regardless of which zone the card was found in, per standard MTG rules
("If you search your library this way, shuffle").

### Change 13: Dual-zone search hash update

**File**: `crates/engine/src/state/hash.rs`
**Action**: In the `Effect::SearchLibrary` hash arm (~line 4333), add:
```rust
also_search_graveyard.hash_into(hasher);
```

### Change 14: Exhaustive match updates

Files requiring new match arms or field additions for the new types:

| File | Match expression | Line (approx) | Action |
|------|-----------------|----------------|--------|
| `crates/engine/src/state/hash.rs` | AltCostKind match | ~L2558 | Add `Adventure => 27` |
| `crates/engine/src/state/hash.rs` | StackObject hash | varies | Hash `was_cast_as_adventure` |
| `crates/engine/src/state/hash.rs` | GameObject hash | varies | Hash `adventure_exiled_by` |
| `crates/engine/src/state/hash.rs` | CardDefinition hash | varies | Hash `adventure_face` |
| `crates/engine/src/state/hash.rs` | Effect::SearchLibrary hash | ~L4333 | Hash `also_search_graveyard` |
| `crates/engine/src/cards/helpers.rs` | (exports) | — | No change needed (CardFace already exported) |
| `tools/replay-viewer/src/view_model.rs` | (no exhaustive AltCostKind match) | — | No change needed |
| `tools/tui/src/play/panels/stack_view.rs` | (no exhaustive AltCostKind match) | — | No change needed |
| `crates/engine/src/rules/casting.rs` | alt_cost matching | ~L164 | Add `cast_with_adventure` binding + validation |
| `crates/engine/src/rules/resolution.rs` | destination selection | ~L1697 | Add adventure exile path |
| `crates/engine/src/effects/mod.rs` | SearchLibrary arm | ~L1879 | Destructure `also_search_graveyard` |

**Existing SearchLibrary callsites** that need `also_search_graveyard: false` added:
```
Grep pattern="SearchLibrary" path="crates/engine/src/cards/defs" output_mode="files_with_matches"
```
All existing card defs using `Effect::SearchLibrary` need the new field added. Since `#[serde(default)]`
handles deserialization, only explicit struct construction in Rust code needs updating. Grep for all
`SearchLibrary {` in card defs and add `also_search_graveyard: false,`.

## Card Definition Fixes

### monster_manual.rs
**Oracle text**: "Monster Manual" main face: {3}{G} Artifact, "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield." Adventure half "Zoological Study": {2}{G} Sorcery — Adventure, "Mill five cards, then return a creature card from your graveyard to your hand."
**Current state**: Two TODOs — activated ability (hand-to-battlefield, blocked by TargetCardInHand gap) and Adventure half (blocked by Adventure framework).
**Fix**: Add `adventure_face` with Zoological Study characteristics and Spell effect. The activated ability TODO remains (blocked by TargetCardInHand, separate DSL gap — document with TODO). The Adventure half uses `Effect::Sequence` with `Effect::MillCards` + `Effect::MoveZone` (graveyard creature to hand).

### lozhan_dragons_legacy.rs
**Oracle text**: "Flying\nWhenever you cast an Adventure or Dragon spell, Lozhan deals damage equal to that spell's mana value to any target that isn't a commander."
**Current state**: TODO — triggered ability needs WheneverYouCastSpellWithType(Adventure|Dragon) trigger, EffectAmount::CastSpellManaValue, and TargetFilter::NonCommander.
**Fix**: The Adventure framework enables checking "has an Adventure" (CR 715.2a). However, the full trigger requires `TriggerCondition::WheneverYouCastAdventureOrDragon` and `EffectAmount::CastSpellManaValue` — both are DSL gaps beyond Adventure itself. **Keep TODO** but update it to note that Adventure framework now exists; remaining gaps are trigger condition and effect amount.

## New Card Definitions

### bonecrusher_giant.rs
**Oracle text**: Bonecrusher Giant {2}{R} — Creature — Giant 4/3. "Whenever Bonecrusher Giant becomes the target of a spell, Bonecrusher Giant deals 2 damage to that spell's controller." Adventure half "Stomp" {1}{R} — Instant — Adventure. "Damage can't be prevented this turn. Stomp deals 2 damage to any target."
**CardDefinition sketch**:
```rust
CardDefinition {
    name: "Bonecrusher Giant // Stomp",
    mana_cost: {generic: 2, red: 1},
    types: [Creature], subtypes: ["Giant"],
    power: 4, toughness: 3,
    abilities: [Triggered(WhenTargetedBySpell, DealDamage 2 to spell controller)],
    adventure_face: Some(CardFace {
        name: "Stomp",
        mana_cost: {generic: 1, red: 1},
        types: [Instant], subtypes: ["Adventure"],
        abilities: [Spell(DealDamage 2 to target)],
        // Note: "damage can't be prevented" effect is a DSL gap (prevention shield removal).
        // Document with TODO.
    }),
}
```

### lovestruck_beast.rs
**Oracle text**: Lovestruck Beast {2}{G} — Creature — Beast Noble 5/5. "Lovestruck Beast can't attack unless you control a 1/1 creature." Adventure half "Heart's Desire" {G} — Sorcery — Adventure. "Create a 1/1 white Human creature token."
**CardDefinition sketch**:
```rust
CardDefinition {
    name: "Lovestruck Beast // Heart's Desire",
    mana_cost: {generic: 2, green: 1},
    types: [Creature], subtypes: ["Beast", "Noble"],
    power: 5, toughness: 5,
    abilities: [StaticRestriction(CantAttackUnlessYouControl1_1)],
    // Note: attack restriction is a DSL gap (ContinuousRestriction::CantAttackUnless).
    // Document with TODO.
    adventure_face: Some(CardFace {
        name: "Heart's Desire",
        mana_cost: {green: 1},
        types: [Sorcery], subtypes: ["Adventure"],
        abilities: [Spell(CreateToken(1/1 white Human))],
    }),
}
```

## Unit Tests

**File**: `crates/engine/tests/adventure_tests.rs`
**Tests to write**:

### Adventure Tests (5 minimum)

1. `test_adventure_cast_adventure_half_from_hand` — Cast Bonecrusher Giant's Stomp from hand as an Adventure. Verify the spell uses the Adventure characteristics on the stack (Instant type, {1}{R} mana cost). CR 715.3a, 715.3b.

2. `test_adventure_exile_on_resolution` — Cast an Adventure spell. Verify it goes to exile (not graveyard) on resolution. Verify `adventure_exiled_by` is set on the exiled card. CR 715.3d.

3. `test_adventure_cast_creature_from_exile` — After an Adventure resolves and is exiled, cast the creature half from exile. Verify it enters the battlefield with creature characteristics. CR 715.3d.

4. `test_adventure_countered_goes_to_graveyard` — Cast an Adventure spell, then counter it. Verify the card goes to graveyard, NOT exile. CR 715.3d (exile only on resolution).

5. `test_adventure_cannot_recast_as_adventure_from_exile` — After an Adventure resolves and is exiled, verify the card CANNOT be cast as an Adventure again from exile. Only the creature half can be cast. CR 715.3d.

6. `test_adventure_normal_characteristics_in_other_zones` — Verify that an adventurer card in hand/graveyard/exile has only its normal (creature) characteristics, not the Adventure characteristics. CR 715.4.

### Dual-Zone Search Tests (3 minimum)

7. `test_search_library_only` — SearchLibrary with `also_search_graveyard: false` only finds cards in the library. Standard behavior.

8. `test_search_library_and_graveyard` — SearchLibrary with `also_search_graveyard: true` finds cards in both library and graveyard.

9. `test_search_graveyard_still_shuffles_library` — When a card is found in the graveyard via dual-zone search, the library is still shuffled (per standard "search your library" rules).

**Pattern**: Follow tests for Disturb in `crates/engine/tests/` (likely in `transform_dfc_tests.rs` or similar).

## Verification Checklist

- [ ] `AltCostKind::Adventure` added to types.rs
- [ ] `adventure_face: Option<CardFace>` added to CardDefinition
- [ ] `was_cast_as_adventure: bool` added to StackObject
- [ ] `adventure_exiled_by: Option<PlayerId>` added to GameObject
- [ ] Casting validation in casting.rs (adventure from hand, creature from exile)
- [ ] Resolution exile path in resolution.rs (only on successful resolution)
- [ ] Fizzle/counter paths verified to NOT exile adventure spells
- [ ] `also_search_graveyard: bool` added to Effect::SearchLibrary
- [ ] SearchLibrary execution extended in effects/mod.rs
- [ ] All hash updates in hash.rs (AltCostKind disc 27, StackObject, GameObject, CardDef, Effect)
- [ ] Monster Manual card def updated (adventure_face added)
- [ ] Lozhan TODO updated (Adventure framework exists, remaining gaps documented)
- [ ] Bonecrusher Giant card def authored
- [ ] Lovestruck Beast card def authored
- [ ] All existing SearchLibrary card defs updated with `also_search_graveyard: false`
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)

## Risks & Edge Cases

1. **Adventure is NOT technically an alternative cost** (Scryfall ruling 2022-06-10). In theory, an Adventure could be combined with Flashback or other alt costs. We model it as `AltCostKind` for simplicity — this means Adventure+Flashback is blocked. This is acceptable for now since no printed card combines them. If needed later, Adventure could be moved to a separate `is_adventure: bool` flag on CastSpell.

2. **Copies of Adventure spells** (CR 715.3c): If an Adventure spell is copied, the copy is also an Adventure. Copies are already `is_copy: true` and don't move to zones on resolution, so the exile behavior is skipped for copies. The copy should still have Adventure characteristics on the stack. Verify the copy system propagates `was_cast_as_adventure` flag.

3. **Adventure face characteristics on the stack**: The engine needs to know which characteristics to use when an Adventure spell is on the stack. Currently, the stack object holds a `source_object` reference, and characteristics come from the underlying card. When `was_cast_as_adventure` is true, the layer system or resolution code needs to use `adventure_face` characteristics instead of the main face. This may require a check in `calculate_characteristics()` or in the casting code that sets initial characteristics.

4. **SearchLibrary with `also_search_graveyard`**: The deterministic fallback picks the lowest ObjectId. When searching both zones, a graveyard card might have a lower ObjectId than a library card. This is fine for deterministic behavior but may not match player intent. M10 interactive choice resolves this.

5. **Monster Manual's Adventure half**: The oracle text shows "Mill five cards, then return a creature card from your graveyard to your hand." This is a sorcery effect (mill + return). The mill effect exists (`Effect::MillCards`). The return-from-graveyard needs `Effect::MoveZone` targeting a creature in graveyard. The deterministic fallback picks the first creature (by ObjectId) from the graveyard — acceptable for testing.

6. **Existing `SearchLibrary` callsites**: All card defs that construct `Effect::SearchLibrary` inline need the new `also_search_graveyard` field. Since `#[serde(default)]` handles the serde path, only Rust code constructing the enum needs updating. Grep for all sites and add the field.
