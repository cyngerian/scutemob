# Ability Review: Toxic

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.164
**Files reviewed**:
- `crates/engine/src/state/types.rs:722-733` (enum variant + doc comment)
- `crates/engine/src/state/hash.rs:502-506` (hash discriminant 85)
- `tools/replay-viewer/src/view_model.rs:715` (display formatting)
- `crates/engine/src/rules/combat.rs:1114-1337` (DamageAppInfo extraction + application)
- `crates/engine/tests/toxic.rs:1-819` (8 unit tests)

## Verdict: needs-fix

The implementation correctly identifies Toxic as a static ability (no trigger, no stack
object), correctly applies it only to combat damage dealt to players, correctly handles
the Toxic+Infect interaction, and has thorough tests with good CR citations. However, the
Toxic value extraction bypasses `calculate_characteristics()` by reading directly from
`CardDefinition.abilities`. This means ability-removal effects (Humility, Dress Down) and
ability-granting effects would be ignored for Toxic, contradicting the layer system. All
other keywords in the same extraction block (deathtouch, lifelink, wither, infect) correctly
use the layer-resolved `chars` from `calculate_characteristics()`. One MEDIUM finding.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `combat.rs:1159-1188` | **Toxic extraction bypasses layer system.** Uses `CardDefinition.abilities` instead of `calculate_characteristics()`, ignoring ability-removal (Humility) and ability-granting effects. **Fix:** use `chars.keywords` with iteration + sum, matching the pattern used for deathtouch/lifelink/wither/infect. |
| 2 | LOW | `combat.rs:1159-1188` | **Same-N Toxic deduplication in OrdSet.** If a creature gains a second `Toxic(2)` from an effect, `OrdSet` deduplication means `chars.keywords` only has one `Toxic(2)`. Total toxic value would be 2 instead of 4. Not fixable without changing `keywords` from `OrdSet` to `Vector`. Deferred -- no real cards currently produce this scenario in the engine. |
| 3 | LOW | `tests/toxic.rs:266-371` | **Cumulative test exercises CardDefinition path only.** The test sets up a `CardRegistry` with both `Toxic(2)` and `Toxic(1)` but the `ObjectSpec` only has `Toxic(2)` via `with_keyword`. After Finding 1 is fixed, this test should be updated to ensure both keywords appear on the object via `with_keyword`. |

### Finding Details

#### Finding 1: Toxic extraction bypasses layer system

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/combat.rs:1159-1188`
**CR Rule**: 702.164c -- "Combat damage dealt to a player by a creature with toxic..."
**Architecture invariant**: Layer system (CR 613) must be the source of truth for all
characteristic queries at resolution time.

**Issue**: The `source_toxic_total` extraction uses `state.card_registry.get(card_id)` as
its primary path (lines 1159-1172), looking at `CardDefinition.abilities` directly. The
layer-resolved `chars` from `calculate_characteristics()` (line 1131) is only used as a
fallback when the object has no `card_id`.

This creates two correctness problems:

1. **Ability removal ignored.** If Humility, Dress Down, or Sudden Spoiling removes all
   abilities, `calculate_characteristics()` returns `keywords = OrdSet::new()` (empty set --
   see `layers.rs:377`). But the CardDefinition still contains `Toxic(N)`, so the creature
   would incorrectly give poison counters despite having no abilities. All four other
   keywords in this same block (deathtouch, lifelink, wither, infect) correctly use `chars`
   and would be suppressed by Humility. Toxic would be the sole exception.

2. **Granted abilities ignored.** If a continuous effect grants Toxic to a creature (via
   `LayerModification::AddKeyword(Toxic(N))`), `calculate_characteristics()` would include
   it in `chars.keywords`, but the CardDefinition path would not find it (it is not in the
   definition). The creature would fail to give poison counters despite having Toxic.

The comment at lines 1155-1157 explains the motivation: "OrdSet deduplication would
otherwise merge identical-N values in obj.characteristics.keywords." This is a valid
concern for the edge case of two `Toxic(2)` instances, but it is addressed by Finding 2
(LOW) and does not justify bypassing the layer system.

**Fix**: Replace the `source_toxic_total` extraction with the same pattern used for the
other four keywords -- iterate over `chars.keywords`:

```rust
// CR 702.164b: Total toxic value is the sum of all Toxic N values.
// Multiple instances are cumulative. Uses layer-resolved characteristics
// so ability-removal (Humility) and ability-granting effects are respected.
// NOTE: If two identical Toxic(N) values exist, OrdSet deduplication means
// only one is counted. This is a known limitation (see Finding 2, LOW).
let source_toxic_total: u32 = chars
    .as_ref()
    .map(|c| {
        c.keywords
            .iter()
            .filter_map(|kw| match kw {
                KeywordAbility::Toxic(n) => Some(*n),
                _ => None,
            })
            .sum()
    })
    .unwrap_or(0);
```

This makes Toxic consistent with deathtouch, lifelink, wither, and infect -- all using
the layer-resolved `chars`.

#### Finding 2: Same-N Toxic deduplication in OrdSet

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:1159-1188`
**CR Rule**: 702.164b -- "A creature's total toxic value is the sum of all N values of
toxic abilities that creature has."
**Issue**: `Characteristics.keywords` is an `OrdSet<KeywordAbility>`. If a creature has
two separate instances of `Toxic(2)` (e.g., printed `Toxic 2` plus granted `Toxic 2` from
an effect), the OrdSet would contain only one `Toxic(2)` element, and the total toxic value
would be 2 instead of the correct 4.

This is a structural limitation of the `OrdSet` representation. No known real-world card
combination currently produces this scenario in the engine (different Toxic N values like
`Toxic(2)` + `Toxic(1)` are distinct OrdSet elements and work correctly). Fixing this
would require changing `keywords` from `OrdSet` to `Vector` or adding a parallel tracking
mechanism, which is out of scope for this ability implementation.

**Fix**: Defer. Add a comment noting the limitation. If/when a card or effect creates
duplicate-N Toxic instances, address it holistically (affects all cumulative parameterized
keywords, not just Toxic).

#### Finding 3: Cumulative test exercises CardDefinition path only

**Severity**: LOW
**File**: `crates/engine/tests/toxic.rs:290-302`
**CR Rule**: 702.164b
**Issue**: The test `test_702_164_toxic_multiple_instances_cumulative` builds an `ObjectSpec`
with only `with_keyword(KeywordAbility::Toxic(2))` (line 296), relying on the CardRegistry
path (CardDefinition has both Toxic(2) and Toxic(1)) to produce the correct sum of 3. After
Finding 1 is fixed (switching to `chars.keywords`), the object's `keywords` OrdSet would
only contain `Toxic(2)`, and the test would fail because `Toxic(1)` was never added to the
ObjectSpec.

**Fix**: After applying Finding 1, update the test to add both keywords to the ObjectSpec:

```rust
.object(
    ObjectSpec::creature(p1, "Multi-Toxic Creature", 1, 1)
        .with_keyword(KeywordAbility::Toxic(2))
        .with_keyword(KeywordAbility::Toxic(1))
        .with_card_id(CardId("double-toxic-creature".to_string())),
)
```

This ensures both Toxic values are present in `chars.keywords` (they have different N
values, so OrdSet keeps both).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.164a (static ability) | Yes | Yes | `test_basic` asserts no AbilityTriggered event, empty stack |
| 702.164b (cumulative instances) | Yes* | Yes* | *Uses CardDefinition bypass (Finding 1); test passes but path is wrong |
| 702.164c (combat damage to player) | Yes | Yes | `test_basic`, `test_multiplayer` |
| 702.164c (NOT creatures) | Yes | Yes | `test_damage_to_creature_no_poison` |
| 702.164c (NOT planeswalkers) | Yes (structural) | No | Planeswalker arm has no Toxic code; no explicit test |
| 120.3g (damage must be dealt) | Yes | Yes | `test_zero_damage_no_poison` (0-power creature) |
| 120.3g (in addition to) | Yes | Yes | `test_basic` asserts both life loss AND poison |
| Toxic + Infect | Yes | Yes | `test_toxic_with_infect` -- 3 Infect + 1 Toxic = 4 poison |
| Toxic + Lifelink | Yes | Yes | `test_toxic_with_lifelink` -- life gain + life loss + poison |
| 704.5c (10 poison SBA) | Yes (preexisting) | Yes | `test_toxic_kills_via_poison_sba` |
| Multiplayer correctness | Yes | Yes | `test_multiplayer_correct_player` -- 4 players, only attacked player gets poison |
| Non-combat damage exclusion | Yes (structural) | No | `effects/mod.rs` has no Toxic code; no explicit test |
| Hash coverage | Yes | -- | Discriminant 85, hashes `n` |
| View model display | Yes | -- | `format!("Toxic {n}")` |
| TUI stack_view.rs | N/A | -- | Correctly no variant (static, not triggered) |

## Summary of Positive Observations

1. **Correct categorization as static.** No `StackObjectKind::ToxicTrigger` variant, no
   `TriggerEvent` for Toxic, no trigger dispatch code. This is exactly right per CR 702.164a.

2. **Correct placement in damage loop.** Toxic is applied inline in the
   `CombatDamageTarget::Player` arm of `apply_combat_damage_assignments()`, after the
   `final_dmg == 0` guard. This correctly ensures Toxic only fires when combat damage is
   actually dealt to a player.

3. **Correct Infect + Toxic interaction.** Both the infect branch (giving `final_dmg`
   poison counters) and the Toxic branch (giving `source_toxic_total` poison counters)
   execute independently. A creature with both gives the correct combined total.

4. **Correct Lifelink independence.** Lifelink is applied outside the match (line 1380),
   so it applies regardless of target type, while Toxic only applies inside the Player arm.
   Both operate on the same damage event without interfering.

5. **Thorough test coverage.** 8 tests covering basic positive, negative (creature target),
   cumulative, zero-damage, Infect interaction, Lifelink interaction, SBA kill, and
   multiplayer. All tests have CR citations in doc comments.

6. **Event emission.** `PoisonCountersGiven` events are correctly emitted for Toxic poison,
   with the correct `amount` and `player` fields, enabling UI and replay tracking.
