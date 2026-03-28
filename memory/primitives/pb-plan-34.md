# Primitive Batch Plan: PB-34 -- Mana Production (Filter Lands, Devotion, Conditional)

**Generated**: 2026-03-27
**Primitive**: New `Effect::AddManaFilterChoice` variant for filter land mana production; verify `AddManaScaled` + `DevotionTo` wiring; extend `try_as_tap_mana_ability` for `AddManaScaled` and filter choice patterns
**CR Rules**: 605 (mana abilities), 106 (mana), 700.5 (devotion)
**Cards affected**: 10 (7 existing fixes + 3 remaining deferred)
**Dependencies**: None
**Deferred items from prior PBs**: None directly applicable

## Primitive Specification

### G-23: Filter Land Mana (new Effect variant)

Filter lands like Fetid Heath have "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}." This is
"pay a hybrid mana, produce 2 mana where each is independently chosen from {color_a, color_b}."

The existing `AddManaChoice` means "N mana of any ONE color" (all same color). Filter lands
need "N mana where EACH is independently from a constrained set of colors." Since interactive
color choice is not yet implemented (M10), the engine simplifies to producing one of each color.

New variant: `Effect::AddManaFilterChoice { player, color_a, color_b }` which produces
1 of color_a + 1 of color_b (the middle option of the 3 choices). This is the fairest
simplification.

### G-24: Devotion-Based Mana (verify existing wiring)

`AddManaScaled { color, count: DevotionTo(color) }` is mechanically complete in effects/mod.rs.
However, Nykthos requires "choose a color" (Command::ChooseColor -- M10 interactive choice).
Three Tree City requires creature-type-count + color choice. Both remain deferred.

**Pre-existing bug**: `AddManaScaled` with `Cost::Tap` is orphaned in `enrich_spec_from_def` --
it's excluded from activated_abilities (line 1886) but `try_as_tap_mana_ability` does NOT
recognize it, so it's never registered as a ManaAbility either. Fix: extend
`try_as_tap_mana_ability` to recognize `AddManaScaled` patterns.

### G-25: Conditional Mana Abilities (scope reduction)

Most G-25 cards require deep infrastructure not in scope for a single PB:
- Springleaf Drum: needs `Cost::TapAnotherCreature` (new Cost variant)
- Cryptolith Rite: needs `LayerModification::GrantManaAbility` (complex)
- Faeburrow Elder: needs dynamic multi-color query
- Arena of Glory: needs exert + mana spend tracking

These are NOT implementable with simple wiring. G-25 is **deferred to PB-37 (residual)**.

**Phyrexian Tower** is already correctly implemented -- `Cost::Sacrifice` + `AddMana` is wired
through the activated ability path (not technically CR 605 compliant since it goes on the stack,
but functionally correct).

## CR Rule Text

### CR 605.1a
> An activated ability is a mana ability if it meets all of the following criteria: it doesn't
> require a target (see rule 115.6), it could add mana to a player's mana pool when it
> resolves, and it's not a loyalty ability.

### CR 605.3b
> An activated mana ability doesn't go on the stack, so it can't be targeted, countered, or
> otherwise responded to. Rather, it resolves immediately after it is activated.

### CR 700.5
> A player's devotion to [color] is the number of mana symbols of that color among the mana
> costs of permanents that player controls.

### CR 106
> Mana is the primary resource in the game. Players spend mana to pay costs.

## Engine Changes

### Change 1: Add `Effect::AddManaFilterChoice` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to the `Effect` enum after `AddManaChoice` (~line 1106):

```rust
/// Add 2 mana where each is independently chosen from two constrained colors.
/// Used by filter lands: "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}."
/// Simplified: produces 1 of each color (the middle option).
AddManaFilterChoice {
    player: PlayerTarget,
    color_a: ManaColor,
    color_b: ManaColor,
},
```

**Pattern**: Follow `AddManaChoice` at line 1103.

### Change 2: Execute `AddManaFilterChoice` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution arm after the `AddManaChoice` handler (~line 1360):

```rust
Effect::AddManaFilterChoice { player, color_a, color_b } => {
    // Simplified: add 1 of each color (middle option of 3 choices).
    // Interactive color choice deferred to M10.
    let players = resolve_player_target_list(state, player, ctx);
    for p in players {
        if let Some(ps) = state.players.get_mut(&p) {
            ps.mana_pool.add(*color_a, 1);
            events.push(GameEvent::ManaAdded {
                player: p,
                color: *color_a,
                amount: 1,
            });
            ps.mana_pool.add(*color_b, 1);
            events.push(GameEvent::ManaAdded {
                player: p,
                color: *color_b,
                amount: 1,
            });
        }
    }
}
```

**CR**: 605.1a -- mana ability resolves and adds mana.

### Change 3: Hash the new variant

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm in `impl HashInto for Effect` after the `AddManaChoice` arm (~line 4504). Use discriminant 73 (next available after 72).

```rust
Effect::AddManaFilterChoice { player, color_a, color_b } => {
    73u8.hash_into(hasher);
    player.hash_into(hasher);
    color_a.hash_into(hasher);
    color_b.hash_into(hasher);
}
```

### Change 4: Extend `try_as_tap_mana_ability` for AddManaScaled and AddManaFilterChoice

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `try_as_tap_mana_ability` (~line 2870), add recognition for `AddManaFilterChoice`:

```rust
// Filter land pattern: {T}: AddManaFilterChoice { color_a, color_b }
if let Effect::AddManaFilterChoice { color_a, color_b, .. } = effect {
    let mut produces = im::OrdMap::new();
    produces.insert(*color_a, 1);
    // If both colors are the same, produces 2 of that color
    *produces.entry(*color_b).or_insert(0) += 1;
    return Some(ManaAbility {
        produces,
        requires_tap: true,
        sacrifice_self: false,
        any_color: false,
        damage_to_controller: 0,
    });
}
```

Also add recognition for `AddManaScaled` (fixes pre-existing orphan bug):

```rust
// Scaled mana: {T}: AddManaScaled { color, count }
// Can't know amount at enrichment time, so register with produces={color: 1}
// as a marker. The actual mana production happens via the effect system.
if let Effect::AddManaScaled { color, .. } = effect {
    return Some(ManaAbility {
        produces: {
            let mut p = im::OrdMap::new();
            p.insert(*color, 1);
            p
        },
        requires_tap: true,
        sacrifice_self: false,
        any_color: false,
        damage_to_controller: 0,
    });
}
```

**Pattern**: Follow existing `AddManaAnyColor` pattern at line 2876.

### Change 5: Update `is_tap_mana_ability` skip list

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In the `is_tap_mana_ability` check (~line 1881), add `AddManaFilterChoice` to the
`matches!` list alongside the existing variants:

```rust
let is_tap_mana_ability = matches!(cost, Cost::Tap)
    && (matches!(
        effect,
        Effect::AddMana { .. }
            | Effect::AddManaAnyColor { .. }
            | Effect::AddManaScaled { .. }
            | Effect::AddManaFilterChoice { .. }  // <-- NEW
    ) || try_as_tap_mana_ability(effect).is_some());
```

### Change 6: Handle non-Tap filter land costs in try_as_tap_mana_ability

Filter lands use `Cost::Sequence([Mana(hybrid), Tap])`, NOT `Cost::Tap` alone. The current
mana-ability registration in `enrich_spec_from_def` only checks `matches!(cost, Cost::Tap)`.
Filter land abilities therefore won't be caught by the mana-ability path -- they'll go through
the activated ability path, which is actually correct behavior because:

1. The ability has a mana cost component (the hybrid mana)
2. The `Cost::Sequence` properly sets `mana_cost` + `requires_tap` via `flatten_cost_into`
3. The effect still adds mana when the activated ability resolves

**No change needed here.** The filter land ability is registered as an activated ability
(not a mana ability), and when activated via `Command::ActivateAbility`, the effect
`AddManaFilterChoice` executes and adds the mana. This is technically CR 605.1a compliant
(it IS a mana ability that should resolve immediately), but functionally using the stack
has negligible gameplay impact for single-player testing.

**However**, the `is_tap_mana_ability` skip list MUST NOT catch filter land abilities because
they use `Cost::Sequence`, not `Cost::Tap`. Verify this is the case (it should be since the
`matches!(cost, Cost::Tap)` guard excludes Sequence costs).

### Exhaustive match updates

Files requiring new match arms for `Effect::AddManaFilterChoice`:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `match effect` | ~1320 | Execute: add 1 of each color |
| `crates/engine/src/state/hash.rs` | `match self` (Effect) | ~4500 | Hash with discriminant 73 |

No other files exhaustively match on `Effect` variants (TUI and replay-viewer do not).

## Card Definition Fixes

### fetid_heath.rs
**Oracle text**: "{T}: Add {C}. {W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}."
**Current state**: TODO -- second ability defaults to fixed `mana_pool(1, 0, 1, 0, 0, 0)` ({W}{B}) instead of choice. Wrong game state (always produces same combo).
**Fix**: Replace `Effect::AddMana { mana: mana_pool(1, 0, 1, 0, 0, 0) }` with `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::White, color_b: ManaColor::Black }`. Remove TODO comment.

### rugged_prairie.rs
**Oracle text**: "{T}: Add {C}. {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}."
**Current state**: TODO -- defaults to `mana_pool(1, 0, 0, 1, 0, 0)` ({R}{W}).
**Fix**: Replace with `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::Red, color_b: ManaColor::White }`. Remove TODO.

### twilight_mire.rs
**Oracle text**: "{T}: Add {C}. {B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G}."
**Current state**: TODO -- defaults to `mana_pool(0, 0, 1, 0, 1, 0)` ({B}{G}).
**Fix**: Replace with `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::Black, color_b: ManaColor::Green }`. Remove TODO.

### flooded_grove.rs
**Oracle text**: "{T}: Add {C}. {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}."
**Current state**: TODO -- defaults to `mana_pool(0, 1, 0, 0, 1, 0)` ({G}{U}).
**Fix**: Replace with `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::Green, color_b: ManaColor::Blue }`. Remove TODO.

### cascade_bluffs.rs
**Oracle text**: "{T}: Add {C}. {U/R}, {T}: Add {U}{U}, {U}{R}, or {R}{R}."
**Current state**: TODO -- second ability entirely omitted.
**Fix**: Add second ability with `Cost::Sequence([Mana(hybrid U/R), Tap])` and `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::Blue, color_b: ManaColor::Red }`. Remove TODO.

### sunken_ruins.rs
**Oracle text**: "{T}: Add {C}. {U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}."
**Current state**: TODO -- second ability entirely omitted.
**Fix**: Add second ability with `Cost::Sequence([Mana(hybrid U/B), Tap])` and `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::Blue, color_b: ManaColor::Black }`. Remove TODO.

### graven_cairns.rs
**Oracle text**: "{T}: Add {C}. {B/R}, {T}: Add {B}{B}, {B}{R}, or {R}{R}."
**Current state**: TODO -- second ability entirely omitted.
**Fix**: Add second ability with `Cost::Sequence([Mana(hybrid B/R), Tap])` and `Effect::AddManaFilterChoice { player: PlayerTarget::Controller, color_a: ManaColor::Black, color_b: ManaColor::Red }`. Remove TODO.

## New Card Definitions

None. The remaining 3 Shadowmoor/Eventide filter lands (Fire-Lit Thicket, Wooded Bastion,
Mystic Gate) are not in the card registry and are out of scope.

## Deferred Items

### G-24: Devotion-based mana
- **Nykthos, Shrine to Nyx**: Requires "choose a color" (Command::ChooseColor, M10). Deferred.
- **Three Tree City**: Requires creature-type-count + color choice. Deferred.
- `AddManaScaled` + `DevotionTo` execution path verified working in effects/mod.rs (~line 4847).

### G-25: Conditional mana abilities
- **Springleaf Drum**: Needs `Cost::TapAnotherCreature` -- new Cost variant, significant harness work.
- **Cryptolith Rite**: Needs `LayerModification::GrantManaAbility` -- grants an ability to all creatures.
- **Faeburrow Elder**: Needs dynamic multi-color mana query (colors among permanents).
- **Arena of Glory**: Needs exert + mana spend tracking.
- All deferred to PB-37 (residual complex activated abilities).

### Pre-existing bug: AddManaScaled orphan in enrich_spec_from_def
- `AddManaScaled` with `Cost::Tap` is skipped from activated_abilities (line 1886) AND not
  recognized by `try_as_tap_mana_ability` (returns None). These abilities are never registered.
- Affects: Gaea's Cradle, Cabal Coffers, Circle of Dreams Druid, Marwyn, Priest of Titania,
  Elvish Archdruid, Crypt of Agadeem, Cabal Stronghold, Howlsquad Heavy.
- **Fix in this batch**: Extend `try_as_tap_mana_ability` (Change 4 above).

## Unit Tests

**File**: `crates/engine/tests/mana_filter.rs` (new file)
**Tests to write**:
- `test_filter_land_produces_two_mana` -- CR 605.1a: activate filter land ability, verify 2 mana added (1 of each color)
- `test_filter_land_requires_hybrid_mana_cost` -- CR 602.2: verify hybrid mana is spent to activate
- `test_filter_land_requires_tap` -- CR 605.1a: verify land must be untapped
- `test_add_mana_scaled_registered_as_mana_ability` -- verify AddManaScaled abilities are properly registered on objects (regression for pre-existing bug fix)

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` for card-specific behavior tests.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All 7 filter land TODOs resolved
- [ ] `AddManaScaled` orphan bug fixed
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs

## Risks & Edge Cases

- **Hash discriminant collision**: Discriminant 70 is already used by both `ExileWithDelayedReturn` and `PreventCombatDamageFromOrTo`. Use 73 for the new variant to avoid collision. Note the existing collision as a pre-existing issue (not blocking).
- **AddManaScaled registration**: The fix to register AddManaScaled as a ManaAbility means the `produces` map will show 1 mana of the color, but actual production is dynamic. This doesn't affect gameplay (mana ability activation still goes through the command handler which executes the effect), but the `ManaAbility.produces` field becomes a marker rather than an accurate count. Scripts using `tap_for_mana` for Gaea's Cradle etc. need to verify the actual mana pool change via assertions.
- **Filter lands as activated abilities vs mana abilities**: Filter lands with `Cost::Sequence([Mana, Tap])` are registered as activated abilities (go on stack) rather than true mana abilities (CR 605 says resolve immediately). This is a pre-existing architectural limitation shared with Phyrexian Tower and similar cards. Negligible gameplay impact.
- **Interactive color choice**: The `AddManaFilterChoice` simplification (always produce 1 of each) loses the ability to produce 2 of one color. When M10 adds interactive choice, this variant should be upgraded to offer the 3 options.
