# Primitive Batch Plan: PB-24 -- Conditional Statics ("as long as X")

**Generated**: 2026-03-23
**Primitive**: Add `condition: Option<Condition>` field to `AbilityDefinition::Static` and propagate to `ContinuousEffect` runtime struct, enabling ~201 card defs with "as long as" conditional static abilities
**CR Rules**: CR 604.1, 604.2, 604.3a(5), 613.1, 700.5
**Cards affected**: ~201 (all existing card defs with TODO for conditional statics)
**Dependencies**: None (Condition enum already exists with ~30 variants)
**Deferred items from prior PBs**: None

## Primitive Specification

Currently `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef }` creates an
unconditional continuous effect that is always active while the source is on the battlefield.
Many cards have static abilities gated by a condition ("as long as you have 30 or more life",
"as long as it's untapped", "as long as this has 7+ quest counters", etc.). These cannot be
expressed in the current DSL.

**Solution**: Add `condition: Option<Condition>` to both:
1. `ContinuousEffectDef` (the card definition struct)
2. `ContinuousEffect` (the runtime struct in `state/continuous_effect.rs`)

When a condition is present, the layer system's `is_effect_active()` checks the condition
each time characteristics are calculated. If the condition is false, the effect does not
apply. This is correct per CR 604.2: "These effects are active as long as the permanent
with the ability remains on the battlefield **and has the ability**."

Additionally, resolve `EffectFilter::Source` to `EffectFilter::SingleObject(new_id)` in
`register_static_continuous_effects`, so card defs can use `Source` to mean "this permanent"
in their static ability definitions.

**New Condition variants** needed (beyond existing ones):
- `OpponentLifeAtMost(u32)` -- "as long as an opponent has 10 or less life" (Bloodghast)
- `SourceIsUntapped` -- "as long as it's untapped" (Dragonlord Ojutai)
- `IsYourTurn` -- "during your turn" (Razorkin Needlehead, Kaito)
- `YouControlNOrMoreWithFilter { count: u32, filter: TargetFilter }` -- Metalcraft "3+ artifacts", etc.
- `DevotionToColorsLessThan { colors: Vec<Color>, threshold: u32 }` -- Theros gods (single or multi-color)

Existing `Condition` variants that are reusable:
- `ControllerLifeAtLeast(30)` -- Serra Ascendant
- `SourceHasCounters { counter, min }` -- quest counters, slumber counters
- `OpponentHasPoisonCounters(3)` -- Corrupted (Skrel's Hive)
- `CompletedADungeon` -- Nadaar
- `HasCitysBlessing` -- Ascend cards
- `CardTypesInGraveyardAtLeast(4)` -- Delirium

**New LayerModification variant** needed:
- `RemoveCardTypes(OrdSet<CardType>)` -- Theros gods "isn't a creature" (Layer 4)

## CR Rule Text

**CR 604.1**: Static abilities do something all the time rather than being activated or
triggered. They are written as statements, and they're simply true.

**CR 604.2**: Static abilities create continuous effects, some of which are prevention
effects or replacement effects. These effects are active as long as the permanent with
the ability remains on the battlefield and has the ability, or as long as the object
with the ability remains in the appropriate zone, as described in rule 113.6.

**CR 604.3a(5)**: A CDA "does not set the values of such characteristics only if certain
conditions are met." This means conditional static abilities are NOT CDAs -- they must
be applied through the normal layer system, not as CDAs.

**CR 613.1**: The layer system applies continuous effects in order. Conditional statics
follow all the same layer rules; the condition only gates whether the effect is "active."

**CR 700.5**: A player's devotion to [color] is the number of mana symbols of that color
in the mana costs of permanents that player controls. For multi-color devotion (e.g.,
"devotion to red and white"), count mana symbols that are EITHER of those colors --
each symbol that contains at least one of the listed colors counts once (hybrid symbols
that match either color count). `EffectAmount::DevotionTo(Color)` already implements
single-color counting at `effects/mod.rs` line 4469 with hybrid mana support.

## Engine Changes

### Change 1: Add `condition` field to `ContinuousEffectDef`

**File**: `crates/engine/src/cards/card_definition.rs`
**Line**: ~2402 (struct ContinuousEffectDef)
**Action**: Add `#[serde(default)] pub condition: Option<Condition>` field
**Pattern**: Follow the existing `duration` field

```rust
pub struct ContinuousEffectDef {
    pub layer: crate::state::EffectLayer,
    pub modification: crate::state::LayerModification,
    pub filter: crate::state::EffectFilter,
    pub duration: crate::state::EffectDuration,
    #[serde(default)]
    pub condition: Option<Condition>,  // NEW
}
```

### Change 2: Add `condition` field to `ContinuousEffect` (runtime struct)

**File**: `crates/engine/src/state/continuous_effect.rs`
**Line**: ~220 (struct ContinuousEffect)
**Action**: Add `pub condition: Option<Condition>` field after `is_cda`.
For the import, use the full path `crate::cards::card_definition::Condition` in the
type annotation (this avoids adding a `use` -- the codebase already uses full paths
from state to cards in several places, e.g. `register_static_continuous_effects`).

```rust
pub struct ContinuousEffect {
    // ... existing fields ...
    pub is_cda: bool,
    /// Optional condition that must be true for this effect to be active (CR 604.2).
    /// Used by "as long as X" conditional static abilities. Evaluated at layer-application
    /// time against the current game state. None = always active (unconditional).
    pub condition: Option<crate::cards::card_definition::Condition>,  // NEW
}
```

### Change 3: Add new Condition variants

**File**: `crates/engine/src/cards/card_definition.rs`
**Line**: ~2039 (end of Condition enum, before closing brace)
**Action**: Add 5 new variants

```rust
/// "as long as an opponent has N or less life" (e.g., Bloodghast: N=10).
/// True when ANY living opponent of the controller has life total <= N.
OpponentLifeAtMost(u32),

/// "as long as it's untapped" (e.g., Dragonlord Ojutai).
/// True when the source permanent is untapped on the battlefield.
SourceIsUntapped,

/// "during your turn" / "as long as it's your turn" (e.g., Razorkin Needlehead).
/// True when the active player is the controller.
IsYourTurn,

/// "as long as you control N or more [filter]" (e.g., Metalcraft: 3+ artifacts).
/// True when the controller controls N or more permanents matching the filter.
YouControlNOrMoreWithFilter { count: u32, filter: TargetFilter },

/// "as long as your devotion to [colors] is less than N" (Theros gods).
/// CR 700.5: Counts mana symbols matching ANY of the listed colors in mana costs
/// of permanents the controller controls. For single-color devotion, pass a
/// one-element vec. For multi-color (Athreos: W+B, Iroas: R+W), pass both colors.
/// True when the calculated devotion is < threshold.
DevotionToColorsLessThan { colors: Vec<Color>, threshold: u32 },
```

### Change 4: Add `RemoveCardTypes` to `LayerModification`

**File**: `crates/engine/src/state/continuous_effect.rs`
**Line**: After `AddCardTypes` (~line 151)
**Action**: Add new variant for Theros gods "isn't a creature"

```rust
/// Removes specified card types without affecting other types (Layer 4).
///
/// Used by Theros gods: "As long as your devotion to [color] is less than N,
/// [this] isn't a creature." Removes Creature from the type line while keeping
/// Enchantment (and any other types).
RemoveCardTypes(OrdSet<CardType>),
```

### Change 5: Wire `RemoveCardTypes` in layers.rs

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In the Layer 4 (TypeChange) application logic, add a match arm for
`RemoveCardTypes`. Find the existing `AddCardTypes` application and add nearby:

```rust
LayerModification::RemoveCardTypes(types_to_remove) => {
    for ct in types_to_remove {
        chars.card_types.remove(ct);
    }
}
```

### Change 6: Implement `check_static_condition` function

**File**: `crates/engine/src/effects/mod.rs`
**Line**: After `check_condition` (~line 5170)
**Action**: Add a new public function for evaluating conditions in a static/layer context

The key difference from `check_condition`: no `EffectContext` available. Instead, takes
`source: ObjectId` and `controller: PlayerId` directly. Delegates to `check_condition`
for most variants by constructing a minimal `EffectContext`.

```rust
/// Evaluate a condition in a static ability context (no EffectContext available).
/// CR 604.2: Used at layer-application time for conditional continuous effects.
///
/// Constructs a minimal EffectContext from the source and controller, then delegates
/// to check_condition. Cast-time conditions (WasKicked, WasOverloaded, etc.) always
/// return false in a static context since they have no spell resolution to reference.
pub fn check_static_condition(
    state: &GameState,
    condition: &Condition,
    source: ObjectId,
    controller: PlayerId,
) -> bool {
    match condition {
        // New variants handled directly:
        Condition::OpponentLifeAtMost(n) => {
            state.players.iter().any(|(pid, ps)| {
                *pid != controller && !ps.has_lost && ps.life_total <= *n as i32
            })
        }
        Condition::SourceIsUntapped => {
            state.objects.get(&source)
                .map(|obj| obj.zone == ZoneId::Battlefield && !obj.status.tapped)
                .unwrap_or(false)
        }
        Condition::IsYourTurn => {
            state.turn.active_player == controller
        }
        Condition::YouControlNOrMoreWithFilter { count, filter } => {
            let matching = state.objects.values().filter(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj.is_phased_in()
                    && obj.controller == controller
                    && matches_target_filter(state, obj, filter)
            }).count();
            matching >= *count as usize
        }
        Condition::DevotionToColorsLessThan { colors, threshold } => {
            let devotion = calculate_devotion_to_colors(state, controller, colors);
            devotion < *threshold
        }

        // Delegate everything else to check_condition with a minimal context:
        _ => {
            let ctx = EffectContext {
                controller,
                source,
                targets: vec![],
                target_remaps: HashMap::new(),
                kicker_times_paid: 0,
                was_overloaded: false,
                was_bargained: false,
                was_cleaved: false,
                evidence_collected: false,
                x_value: 0,
                gift_was_given: false,
                gift_opponent: None,
                last_effect_count: 0,
            };
            check_condition(state, condition, &ctx)
        }
    }
}
```

**Helper needed**: `calculate_devotion_to_colors(state, player_id, colors) -> u32`.
Reuse the logic from `EffectAmount::DevotionTo` at line 4469 but generalized for
multiple colors. For each permanent the player controls, count mana symbols matching
ANY of the listed colors (including hybrid symbols where either half matches).

**Helper needed**: `matches_target_filter(state, obj, filter) -> bool`. This may already
exist for target validation. Check `effects/mod.rs` or `targeting.rs`. If not, implement
a simple version that checks `card_type` and `subtype` fields of `TargetFilter` against
the object's characteristics.

### Change 7: Update `is_effect_active` in layers.rs

**File**: `crates/engine/src/rules/layers.rs`
**Line**: ~430 (fn is_effect_active)
**Action**: After checking duration-based activity, also check the condition field.
If condition is `Some(cond)`, call `check_static_condition`. If it returns false,
the effect is not active.

```rust
pub fn is_effect_active(state: &GameState, effect: &ContinuousEffect) -> bool {
    // Existing duration check (restructure into let binding):
    let duration_active = match effect.duration {
        EffectDuration::WhileSourceOnBattlefield => { /* existing logic */ }
        EffectDuration::UntilEndOfTurn => true,
        EffectDuration::Indefinite => true,
        EffectDuration::WhilePaired(a, b) => { /* existing logic */ }
    };
    if !duration_active {
        return false;
    }
    // NEW: Check condition if present
    if let Some(ref condition) = effect.condition {
        if let Some(source_id) = effect.source {
            let controller = state
                .objects
                .get(&source_id)
                .map(|obj| obj.controller)
                .unwrap_or_else(|| PlayerId(0));
            if !crate::effects::check_static_condition(state, condition, source_id, controller) {
                return false;
            }
        } else {
            // No source -- conditional effects without a source are inactive
            return false;
        }
    }
    true
}
```

### Change 8: Propagate condition in `register_static_continuous_effects`

**File**: `crates/engine/src/rules/replacement.rs`
**Line**: ~1698 (AbilityDefinition::Static arm)
**Action**: (a) Pass `condition` from `ContinuousEffectDef` to `ContinuousEffect`.
(b) Resolve `EffectFilter::Source` to `EffectFilter::SingleObject(new_id)` at registration time.

```rust
AbilityDefinition::Static { continuous_effect } => {
    let eff_id = state.next_object_id().0;
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    // Resolve Source filter to concrete ObjectId at registration time
    let resolved_filter = match &continuous_effect.filter {
        EffectFilter::Source => EffectFilter::SingleObject(new_id),
        other => other.clone(),
    };
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(new_id),
        timestamp: ts,
        layer: continuous_effect.layer,
        duration: continuous_effect.duration,
        filter: resolved_filter,  // CHANGED: was continuous_effect.filter.clone()
        modification: continuous_effect.modification.clone(),
        is_cda: false,
        condition: continuous_effect.condition.clone(),  // NEW
    });
}
```

### Change 9: Exhaustive match updates for new Condition variants

Files requiring new match arms for the 5 new `Condition` variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `check_condition` match | L5017 | Add 5 arms for new variants (delegate to check_static_condition logic or inline) |
| `crates/engine/src/state/hash.rs` | `HashInto for Condition` | L3917 | Add 5 hash arms (discriminants 32-36) |

### Change 10: Exhaustive match updates for `RemoveCardTypes`

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/rules/layers.rs` | LayerModification application | varies | Add `RemoveCardTypes` arm in Layer 4 handling |
| `crates/engine/src/state/hash.rs` | `HashInto for LayerModification` | ~L1200 | Add hash arm (discriminant 21) |

### Change 11: Hash updates for modified structs

| File | Struct | Line | Action |
|------|--------|------|--------|
| `crates/engine/src/state/hash.rs` | `HashInto for ContinuousEffect` | L1258 | Add `self.condition.hash_into(hasher)` |
| `crates/engine/src/state/hash.rs` | `HashInto for ContinuousEffectDef` | L4038 | Add `self.condition.hash_into(hasher)` |

### Change 12: Export Condition from helpers.rs

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add `pub use crate::cards::card_definition::Condition;` so card defs can use
`Condition::ControllerLifeAtLeast(30)` etc. without fully qualifying.

### Change 13: Fix all sites that construct ContinuousEffect

Every place that constructs a `ContinuousEffect` struct needs the new `condition` field.
Grep for `ContinuousEffect {` across the codebase:

```
Grep pattern="ContinuousEffect \{" path="crates/engine/src" output_mode="content" -C=2
```

Each site needs `condition: None,` added (for unconditional effects) or the appropriate
condition propagated. Known construction sites:
- `crates/engine/src/rules/replacement.rs` L1702 (register_static_continuous_effects)
- `crates/engine/src/effects/mod.rs` (ApplyContinuousEffect execution)
- Anywhere else that creates ContinuousEffect structs (check with grep)

## Card Definition Fixes

This batch unblocks a large number of cards. The highest-impact fixes are listed below.
The runner should grep for all TODOs mentioning "as long as", "conditional static",
"condition field", "metalcraft", "devotion", "quest counter", etc. and fix every
card it finds where the condition is now expressible.

### Representative Fixes (top priority)

#### serra_ascendant.rs
**Oracle text**: As long as you have 30 or more life, this creature gets +5/+5 and has flying.
**Current state**: Lifelink only; TODO for conditional static
**Fix**: Add two `AbilityDefinition::Static` entries, each with
`condition: Some(Condition::ControllerLifeAtLeast(30))`:
  - Layer PtModify, modification ModifyBoth(5), filter Source, duration WhileSourceOnBattlefield
  - Layer Ability, modification AddKeyword(Flying), filter Source, duration WhileSourceOnBattlefield

#### dragonlord_ojutai.rs
**Oracle text**: Dragonlord Ojutai has hexproof as long as it's untapped.
**Current state**: Flying only; TODO for conditional hexproof
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::SourceIsUntapped)`, layer Ability,
modification AddKeyword(Hexproof), filter Source

#### bloodghast.rs
**Oracle text**: This creature can't block. This creature has haste as long as an opponent has 10 or less life.
**Current state**: TODO for can't-block static + conditional haste
**Fix**: Add unconditional `CantBlock` keyword static (or use
`AbilityDefinition::Keyword(KeywordAbility::CantBlock)` if the keyword exists;
otherwise use `AbilityDefinition::Static` with `AddKeyword(CantBlock)` filter Source).
Add conditional haste: `AbilityDefinition::Static` with
`condition: Some(Condition::OpponentLifeAtMost(10))`, layer Ability,
modification AddKeyword(Haste), filter Source.

#### purphoros_god_of_the_forge.rs
**Oracle text**: As long as your devotion to red is less than five, Purphoros isn't a creature.
**Current state**: TODO for devotion-based type loss
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::DevotionToColorsLessThan { colors: vec![Color::Red], threshold: 5 })`,
layer TypeChange, modification `RemoveCardTypes(ordset![CardType::Creature])`, filter Source.

#### athreos_god_of_passage.rs
**Oracle text**: As long as your devotion to white and black is less than seven, Athreos isn't a creature.
**Current state**: TODO for devotion-based type loss
**Fix**: Same pattern as Purphoros but `colors: vec![Color::White, Color::Black], threshold: 7`.

#### iroas_god_of_victory.rs
**Oracle text**: As long as your devotion to red and white is less than seven, Iroas isn't a creature.
**Current state**: TODO for devotion-based type loss
**Fix**: Same pattern as Athreos but `colors: vec![Color::Red, Color::White], threshold: 7`.

#### nadaar_selfless_paladin.rs
**Oracle text**: Other creatures you control get +1/+1 as long as you've completed a dungeon.
**Current state**: TODO mentioning "AbilityDefinition::Static lacks a condition field"
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::CompletedADungeon)`, layer PtModify,
modification ModifyBoth(1), filter OtherCreaturesYouControl

#### beastmaster_ascension.rs
**Oracle text**: As long as this enchantment has 7+ quest counters, creatures you control get +5/+5.
**Current state**: Attack trigger implemented; TODO for conditional static
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::SourceHasCounters { counter: CounterType::Quest, min: 7 })`,
layer PtModify, modification ModifyBoth(5), filter CreaturesYouControl

#### quest_for_the_goblin_lord.rs
**Oracle text**: As long as this enchantment has 5+ quest counters, creatures you control get +2/+0.
**Current state**: TODO for conditional static
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::SourceHasCounters { counter: CounterType::Quest, min: 5 })`,
layer PtModify, modification ModifyPower(2), filter CreaturesYouControl

#### arixmethes_slumbering_isle.rs
**Oracle text**: As long as Arixmethes has a slumber counter on it, it's a land. (It's not a creature.)
**Current state**: TODO for conditional type change
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::SourceHasCounters { counter: CounterType::Slumber, min: 1 })`,
layer TypeChange, modification `RemoveCardTypes(ordset![CardType::Creature])`, filter Source.
Also add a second static for AddCardTypes Land (if not already a land type).
**Note**: CounterType::Slumber may not exist yet -- add it to the CounterType enum.

#### razorkin_needlehead.rs
**Oracle text**: First strike during your turn (conditional keyword)
**Current state**: TODO for conditional first strike
**Fix**: Add `AbilityDefinition::Static` with
`condition: Some(Condition::IsYourTurn)`, layer Ability,
modification AddKeyword(FirstStrike), filter Source

#### skrevls_hive.rs
**Oracle text**: Corrupted -- As long as an opponent has 3+ poison counters, creatures you control with toxic have lifelink.
**Current state**: TODO for Corrupted static
**Fix**: Condition is expressible: `Condition::OpponentHasPoisonCounters(3)`. But filter
needs "creatures you control with toxic" which requires a keyword-filtered EffectFilter
(PB-25 scope). Leave TODO for filter gap.

#### indomitable_archangel.rs
**Oracle text**: Metalcraft -- Artifacts you control have shroud as long as you control 3+ artifacts.
**Current state**: TODO for Metalcraft
**Fix**: Condition is expressible: `Condition::YouControlNOrMoreWithFilter { count: 3, filter: artifact_filter }`.
But filter needs "artifacts you control" EffectFilter (PB-25 scope). Leave TODO for filter gap.

### Additional Cards to Fix (grep-identified)

The runner should grep for these patterns and fix all matching cards:
- `TODO.*as long as` -- all conditional statics
- `TODO.*quest counter` -- quest-counter-threshold statics
- `TODO.*metalcraft` -- 3+ artifact control threshold
- `TODO.*devotion.*isn't a creature` -- Theros gods
- `TODO.*hexproof.*untapped` -- untapped-conditional hexproof
- `TODO.*your turn.*first strike|keyword` -- turn-conditional keywords
- `TODO.*city's blessing.*static` -- Ascend conditional statics
- `TODO.*completed.*dungeon.*static` -- dungeon completion statics
- `TODO.*30 or more life` -- life threshold statics
- `TODO.*opponent.*10 or less life` -- opponent life threshold statics

## Unit Tests

**File**: `crates/engine/tests/conditional_statics.rs`
**Tests to write**:
- `test_conditional_static_life_threshold` -- Serra Ascendant: +5/+5 and flying when life >= 30, not when < 30 (CR 604.2)
- `test_conditional_static_untapped` -- Dragonlord Ojutai: hexproof while untapped, loses it when tapped
- `test_conditional_static_counter_threshold` -- Beastmaster Ascension: +5/+5 when 7+ quest counters
- `test_conditional_static_dungeon` -- Nadaar: +1/+1 to others when dungeon completed
- `test_conditional_static_opponent_life` -- Bloodghast: haste when opponent has <= 10 life
- `test_conditional_static_is_your_turn` -- Razorkin Needlehead: first strike during your turn only
- `test_conditional_static_devotion_single` -- Purphoros: not a creature when devotion to red < 5
- `test_conditional_static_devotion_multicolor` -- Athreos: not a creature when devotion to W+B < 7
- `test_conditional_static_remove_type` -- Verify RemoveCardTypes removes Creature but keeps Enchantment
- `test_conditional_static_toggles_midgame` -- Verify that gaining/losing the condition mid-game immediately updates characteristics (no lag)
- `test_conditional_static_source_filter_resolved` -- Verify EffectFilter::Source is resolved to SingleObject at registration

**Pattern**: Follow tests in `crates/engine/tests/continuous_effects.rs` or `crates/engine/tests/layers.rs`

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] `ContinuousEffectDef` has `condition: Option<Condition>` field
- [ ] `ContinuousEffect` has `condition: Option<Condition>` field
- [ ] `is_effect_active` checks condition when present
- [ ] `check_static_condition` function exists and handles all new variants
- [ ] `register_static_continuous_effects` resolves `Source` filter and propagates condition
- [ ] All 5 new Condition variants added with hash arms and check_condition arms
- [ ] `EffectFilter::Source` resolved to `SingleObject(new_id)` at registration time
- [ ] `RemoveCardTypes` LayerModification added and wired in layers.rs
- [ ] `calculate_devotion_to_colors` helper exists
- [ ] `matches_target_filter` helper exists or is reused from targeting
- [ ] Condition exported from helpers.rs
- [ ] All ContinuousEffect construction sites updated with `condition: None`
- [ ] Top-priority card defs fixed (Serra Ascendant, Dragonlord Ojutai, Bloodghast, Purphoros, Athreos, Iroas, Nadaar, Beastmaster Ascension, Razorkin Needlehead, Quest for the Goblin Lord, Arixmethes)
- [ ] All grep-identified card defs with expressible conditions fixed
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in fixed card defs for patterns now expressible

## Risks & Edge Cases

- **Circular evaluation**: `is_effect_active` calls `check_static_condition`, which may
  read game state that itself depends on continuous effects. For example, if a condition
  checks "you control a creature with flying" and flying is granted by another continuous
  effect, we need the layer system to have already applied that effect. Since conditions
  are checked in `is_effect_active` BEFORE the layer loop in `calculate_characteristics`,
  they see the BASE characteristics, not layer-resolved ones. This is correct for most
  conditions (life totals, counters, controller checks) but could be wrong for
  type-based conditions. Document this limitation.

- **Performance**: `check_static_condition` is called once per conditional effect per
  `calculate_characteristics` call. If there are many conditional effects, this could
  slow down characteristic calculation. For now this is acceptable (typical games have
  few conditional statics on the battlefield simultaneously).

- **Serde compatibility**: Adding `#[serde(default)]` to both new fields ensures backward
  compatibility with serialized game states that don't have the field (they'll deserialize
  as `None`).

- **RemoveCardTypes interaction with other type-changing effects**: RemoveCardTypes is a
  Layer 4 effect. It must be applied in timestamp order with other Layer 4 effects
  (Blood Moon's SetTypeLine, Opalescence's AddCardTypes, etc.). If Blood Moon makes
  something a Mountain (SetTypeLine) and then a Theros god's devotion effect tries to
  RemoveCardTypes(Creature), the RemoveCardTypes applies AFTER SetTypeLine (assuming
  later timestamp), which is correct. If the god entered before Blood Moon, Blood Moon's
  SetTypeLine would override everything anyway.

- **Not all ~201 cards will be fixable in this batch**: Some "conditional static" cards
  also need missing `EffectFilter` variants (e.g., "artifacts you control", "creatures
  with keyword X") which are PB-25 scope. The runner should fix what's expressible and
  leave clear TODOs for the remaining filter gaps.

- **EffectFilter::Source resolution timing**: By resolving `Source` to `SingleObject(new_id)`
  at registration time in `register_static_continuous_effects`, the filter permanently
  binds to that ObjectId. This is consistent with existing `SingleObject` behavior and
  CR 400.7 (zone changes create new objects).

- **Multi-color devotion (CR 700.5)**: "Devotion to red and white" counts symbols that
  are EITHER red or white. A {R/W} hybrid symbol counts once (not twice). The existing
  `EffectAmount::DevotionTo` logic handles hybrid mana for single colors. The new
  `calculate_devotion_to_colors` must generalize this: for each mana symbol, check if
  it matches ANY of the listed colors. Each matching symbol increments the count by 1,
  regardless of how many listed colors it matches. Base color pips: check `mc.red`,
  `mc.white`, etc. Hybrid pips: check both halves against the color list.

- **CounterType::Slumber and CounterType::Quest**: Verify these exist in the
  `CounterType` enum. Quest exists (used by Beastmaster Ascension's trigger). Slumber
  may not -- add it if needed for Arixmethes.

- **CantBlock keyword**: Verify `KeywordAbility::CantBlock` exists for Bloodghast's
  unconditional "can't block" static. If not, check how other "can't block" cards
  (e.g., Decayed tokens) implement this -- it may be enforced via combat.rs checks
  on a designation flag rather than a keyword.
