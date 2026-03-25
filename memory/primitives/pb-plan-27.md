# Primitive Batch Plan: PB-27 -- X-Cost Spells

**Generated**: 2026-03-24
**Primitive**: Wire X-cost mana costs through the engine -- ManaCost.x_count on card defs, x_value propagation to ETB triggers, Condition::XValueAtLeast, dynamic token count via Effect::Repeat, x_value on ActivateAbility, and loyalty -X card def fixes.
**CR Rules**: 107.3, 107.3a, 107.3b, 107.3f, 107.3g, 107.3k, 107.3m, 606.4, 606.6
**Cards affected**: ~18 true X-cost cards (from the ~42 estimate; rest are count-based patterns for PB-28)
**Dependencies**: PB-9 (x_value infrastructure -- DONE)
**Deferred items from prior PBs**: PB-9 deferred 2M 7L items (none directly related to X-cost wiring)

## Primitive Specification

PB-9 (Ravenous, Batch 12) added the foundational X-value infrastructure:
- `x_value: u32` on CastSpell, StackObject, GameObject, EffectContext
- `x_count: u32` on ManaCost (structural -- how many {X} symbols)
- `EffectAmount::XValue` (resolves to `ctx.x_value`)
- X-cost integration in casting.rs (adds x_count * x_value to generic mana)
- `LoyaltyCost::MinusX` (fully wired in engine.rs)

What's MISSING (preventing ~18 card defs from being correct):

1. **Card defs don't set `x_count` on ManaCost** -- most X-cost cards have `x_count: 0` and missing {X} from their mana_cost
2. **ETB triggers don't propagate x_value** -- `EffectContext::new_with_kicker` hardcodes `x_value: 0`
3. **No `Condition::XValueAtLeast(u32)`** -- needed for "if X is 5 or more" (Martial Coup, White Sun's Twilight)
4. **No dynamic token count** -- `TokenSpec.count` is `u32`; need `Effect::Repeat` for "create X tokens"
5. **No `x_value` on `ActivateAbility` command** -- X-cost activated abilities can't pass X
6. **Replay harness `activate_loyalty_ability` ignores x_value** -- `x_value: None` hardcoded
7. **Replay harness `activate_ability` has no x_value support**

## CR Rule Text

### CR 107.3 (X in costs)
107.3. Many objects use the letter X as a placeholder for a number that needs to be determined. Some objects have abilities that define the value of X; the rest let their controller choose the value of X.

107.3a If a spell or activated ability has a mana cost, alternative cost, additional cost, and/or activation cost with an {X}, [-X], or X in it, and the value of X isn't defined by the text of that spell or ability, the controller of that spell or ability chooses and announces the value of X as part of casting the spell or activating the ability.

107.3b If a player is casting a spell that has an {X} in its mana cost, the value of X isn't defined by the text of that spell, and an effect lets that player cast that spell while paying neither its mana cost nor an alternative cost that includes X, then the only legal choice for X is 0.

107.3f Sometimes X appears in the text of a spell or ability but not in a mana cost, alternative cost, additional cost, or activation cost. If the value of X isn't defined, the controller of the spell or ability chooses the value of X at the appropriate time.

107.3g If a card in any zone other than the stack has an {X} in its mana cost, the value of {X} is treated as 0, even if the value of X is defined somewhere within its text.

107.3k If an object's activated ability has an {X}, [-X], or X in its activation cost, the value of X for that ability is independent of any other values of X chosen for that object or for other instances of abilities of that object.

107.3m If an object's enters-the-battlefield triggered ability or replacement effect refers to X, and the spell that became that object as it resolved had a value of X chosen for any of its costs, the value of X for that ability is the same as the value of X for that spell, although the value of X for that permanent is 0.

### CR 606.4, 606.6 (loyalty costs)
606.4. The cost to activate a loyalty ability of a permanent is to put on or remove from that permanent a certain number of loyalty counters.
606.6. An ability with a negative loyalty cost can't be activated unless the permanent has at least that many loyalty counters on it.

## Engine Changes

### Change 1: Add `Condition::XValueAtLeast(u32)` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Condition` enum after the last existing variant (~line 2073)
**CR**: 107.3m -- enables "if X is 5 or more" conditional branches

```rust
/// CR 107.3m: "if X is N or more" — true when `ctx.x_value >= n`.
/// Used for Martial Coup ("if X is 5 or more, destroy all other creatures"),
/// White Sun's Twilight, Finale of Devastation ("if X is 10 or more").
XValueAtLeast(u32),
```

### Change 2: Wire `Condition::XValueAtLeast` in check_condition

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm in `check_condition()` (after line ~5340)
**Pattern**: Follow `Condition::SourceHasCounters` pattern

```rust
Condition::XValueAtLeast(n) => ctx.x_value >= *n as u32,
```

### Change 3: Hash `Condition::XValueAtLeast`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add match arm in `Condition` HashInto impl (after line ~4158)

```rust
Condition::XValueAtLeast(n) => {
    35u8.hash_into(hasher); // next discriminant after 34 (IsYourTurn)
    n.hash_into(hasher);
}
```

**Note**: Check the actual next discriminant -- verify last used is 34.

### Change 4: Add `Effect::Repeat` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Effect` enum (after `ForEach` near line ~1210)

```rust
/// Execute an effect N times, where N is resolved from the amount.
/// Used for "create X tokens" patterns where the count is dynamic (e.g., EffectAmount::XValue).
/// CR 107.3m: N resolves to the X value from the casting cost.
Repeat {
    effect: Box<Effect>,
    count: EffectAmount,
},
```

### Change 5: Execute `Effect::Repeat` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm in `execute_effect()` for `Effect::Repeat`
**Pattern**: Follow `Effect::ForEach` execution pattern

```rust
Effect::Repeat { effect, count } => {
    let n = resolve_amount(state, count, ctx).max(0) as u32;
    for _ in 0..n {
        let inner_events = execute_effect(state, effect, ctx);
        events.extend(inner_events);
    }
}
```

### Change 6: Hash `Effect::Repeat`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add match arm in `Effect` HashInto impl. Find the next available discriminant.

### Change 7: Propagate x_value to ETB trigger EffectContext

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: At line ~1924, after creating `EffectContext::new_with_kicker`, set `ctx.x_value` from the permanent's `x_value`.
**CR**: 107.3m -- ETB triggered abilities reference the spell's X value

```rust
let mut ctx = EffectContext::new_with_kicker(
    stack_obj.controller,
    source_object,
    stack_obj.targets.clone(),
    kicker_times_paid,
);
// CR 107.3m: Propagate x_value from the permanent (set during resolution
// from the spell's StackObject.x_value) so ETB effects using
// EffectAmount::XValue resolve correctly.
ctx.x_value = state
    .objects
    .get(&source_object)
    .map(|o| o.x_value)
    .unwrap_or(0);
```

### Change 8: Add `x_value` to `ActivateAbility` command

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `x_value: Option<u32>` field to `Command::ActivateAbility` (after `sacrifice_target` at line ~126)
**CR**: 107.3k -- X for activated abilities is independent

```rust
/// CR 107.3k: For activated abilities with {X} in the activation cost,
/// the chosen value of X. `None` for non-X abilities (defaults to 0).
#[serde(default)]
x_value: Option<u32>,
```

### Change 9: Wire x_value through ActivateAbility handler

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Extract `x_value` from `Command::ActivateAbility` and pass it to `handle_activate_ability` (line ~122-136)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `x_value: Option<u32>` parameter to `handle_activate_ability` (line 128). Set `stack_obj.x_value` after creating the StackObject (line ~798).

```rust
// After: stack_obj.targets = spell_targets;
stack_obj.x_value = x_value.unwrap_or(0);
```

Also propagate x_value in the activation condition check EffectContext (line ~240):
```rust
x_value: x_value.unwrap_or(0),
```

### Change 10: Wire x_value in mana cost calculation for activated abilities

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the cost-payment section of `handle_activate_ability`, when a `Cost::Mana` with `x_count > 0` is encountered, add `x_count * x_value` to generic before paying. Follow the pattern in casting.rs lines 3210-3222.

### Change 11: Wire replay harness x_value for activate_ability

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In the `"activate_ability"` arm (around line ~530), pass `x_value` from the PlayerAction to the Command.

```rust
Some(Command::ActivateAbility {
    player,
    source: source_id,
    ability_index,
    targets: target_list,
    discard_card: discard_card_id,
    sacrifice_target: sacrifice_target_id,
    x_value: if x_value > 0 { Some(x_value) } else { None },
})
```

### Change 12: Wire replay harness x_value for activate_loyalty_ability

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In the `"activate_loyalty_ability"` arm (line ~550), read `x_value` from PlayerAction instead of `None`.

```rust
x_value: if x_value > 0 { Some(x_value) } else { None },
```

### Change 13: Exhaustive match updates

Files requiring new match arms for `Condition::XValueAtLeast` and `Effect::Repeat`:

| File | Match expression | Approx Line | Action |
|------|-----------------|-------------|--------|
| `crates/engine/src/effects/mod.rs` | `check_condition` match | L5078 | Add `Condition::XValueAtLeast(n)` arm |
| `crates/engine/src/effects/mod.rs` | `execute_effect` match | L193+ | Add `Effect::Repeat` arm |
| `crates/engine/src/state/hash.rs` | `Condition` HashInto | L4030 | Add `XValueAtLeast` hash arm |
| `crates/engine/src/state/hash.rs` | `Effect` HashInto | L3500+ | Add `Repeat` hash arm |

No changes needed in view_model.rs or TUI (Condition/Effect/EffectAmount are not matched there).

Also check `Command::ActivateAbility` match sites for the new `x_value` field:

| File | Match expression | Approx Line | Action |
|------|-----------------|-------------|--------|
| `crates/engine/src/rules/engine.rs` | `Command::ActivateAbility` match | L122 | Extract `x_value` |
| `crates/engine/src/testing/replay_harness.rs` | `activate_ability` builder | L530 | Add `x_value` field |

## Card Definition Fixes

### Category A: True X-cost spells (set x_count, use EffectAmount::XValue)

#### the_meathook_massacre.rs
**Oracle text**: When The Meathook Massacre enters, each creature gets -X/-X until end of turn. [+ death triggers]
**Current state**: Missing x_count on mana cost, ETB -X/-X effect absent
**Fix**: Set `mana_cost: Some(ManaCost { black: 2, x_count: 1, ..Default::default() })`. Add ETB triggered ability: `WhenEntersBattlefield` -> `ApplyContinuousEffect` with `LayerModification::ModifyBoth` using a negative XValue. NOTE: `ModifyBoth` takes `i32`, not `EffectAmount`. Will need to use `Effect::ApplyContinuousEffectDynamic` or a workaround -- evaluate at fix time whether an existing pattern handles this or if it needs a separate change.

#### spiteful_banditry.rs
**Oracle text**: {X}{R}{R}. When enters, deals X damage to each creature. [+ treasure trigger]
**Current state**: Missing x_count, empty abilities
**Fix**: Set x_count: 1. Add ETB trigger with `ForEach { over: EachCreature, effect: DealDamage { target: IterationTarget, amount: XValue } }`. The "once each turn" trigger for Treasure creation remains a separate gap (trigger throttle).

#### martial_coup.rs
**Oracle text**: {X}{W}{W}. Create X 1/1 white Soldier tokens. If X >= 5, destroy all other creatures.
**Current state**: Missing x_count, empty abilities
**Fix**: Set x_count: 1. Use `Repeat { effect: CreateToken { spec: soldier_1_1_spec() }, count: XValue }` for X tokens. Use `Conditional { condition: XValueAtLeast(5), if_true: DestroyAll { filter: NonSelf }, if_false: Nothing }` for the conditional wipe. Full Sequence of Repeat + Conditional.

#### awaken_the_woods.rs
**Oracle text**: {X}{G}{G}. Create X 1/1 green Forest Dryad land creature tokens.
**Current state**: Has `count: 3` approximation, missing x_count
**Fix**: Set x_count: 1. Change effect to `Repeat { effect: CreateToken { spec: dryad_token_spec_1() }, count: XValue }` where spec has count: 1.

#### white_suns_twilight.rs
**Oracle text**: {X}{W}{W}. Gain X life. Create X Phyrexian Mite tokens. If X >= 5, destroy all other creatures.
**Current state**: Missing x_count, empty abilities
**Fix**: Set x_count: 1. Sequence of GainLife(XValue) + Repeat(CreateToken, XValue) + Conditional(XValueAtLeast(5), DestroyAll, Nothing).

#### goblin_negotiation.rs
**Oracle text**: {X}{R}{R}. Deal X damage to target creature. Create tokens equal to excess damage.
**Current state**: Missing x_count, empty abilities
**Fix**: Set x_count: 1. Partial fix: DealDamage with XValue to target. Excess damage token creation requires tracking (separate gap -- note with TODO).

#### agadeems_awakening.rs
**Oracle text**: {X}{B}{B}{B}. Return creatures from GY with different MV X or less.
**Current state**: Missing x_count, empty abilities
**Fix**: Set x_count: 1. Full effect requires multi-target graveyard selection with MV filter. Partial: note that this card has ADDITIONAL gaps beyond X-cost (multi-target GY selection, MV filter). Set mana cost correctly; leave TODO for the complex effect.

#### ingenious_prodigy.rs
**Oracle text**: {X}{U}. Enters with X +1/+1 counters.
**Current state**: Missing x_count, TODO for X counters
**Fix**: Set x_count: 1. Add ETB trigger: `WhenEntersBattlefield` -> `AddCounterAmount { target: Source, counter: PlusOnePlusOne, count: XValue }`. This works because Change 7 propagates x_value to ETB EffectContext.

#### pull_from_tomorrow.rs
**Oracle text**: {X}{U}{U}. Draw X, discard 1.
**Current state**: Already uses `EffectAmount::XValue` for draw! But missing x_count on mana cost.
**Fix**: Change mana cost to `ManaCost { blue: 2, x_count: 1, ..Default::default() }`.

#### finale_of_devastation.rs
**Oracle text**: {X}{G}{G}. Search for creature MV X or less; if X >= 10 mass pump.
**Current state**: Missing x_count, search missing MV filter, conditional pump missing
**Fix**: Set x_count: 1. Add `Condition::XValueAtLeast(10)` for the conditional pump. The dynamic MV filter on SearchLibrary (max_cmc from XValue) is a separate gap -- note with TODO.

### Category B: X-cost activated abilities (need ActivateAbility.x_value)

#### mirror_entity.rs
**Oracle text**: {X}: Until EOT, creatures you control have base P/T X/X and gain all creature types.
**Current state**: TODO for X-cost activated
**Fix**: Add activated ability with `Cost::Mana(ManaCost { x_count: 1, ..Default::default() })`. Effect: `ApplyContinuousEffect` with base P/T setting from XValue. NOTE: The continuous effect needs to set P/T to a dynamic value. This requires `LayerModification::SetBoth(EffectAmount)` which doesn't exist. Leave TODO for the P/T-setting-from-XValue gap. At minimum, wire the cost correctly.

#### steel_hellkite.rs
**Oracle text**: {X}: Destroy each nonland permanent with MV X whose controller was dealt combat damage.
**Current state**: TODO
**Fix**: Multiple gaps beyond X-cost (MV=X filter, combat-damage-tracking). Set cost correctly with x_count: 1. Leave TODO for the complex filter/tracking.

#### grim_hireling.rs
**Oracle text**: {B}, Sacrifice X Treasures: Target creature gets -X/-X until EOT.
**Current state**: TODO
**Fix**: "Sacrifice X" is a different kind of X cost -- not mana X. The number of sacrificed permanents determines the effect. This is a separate gap (variable sacrifice count). NOT fixable in PB-27.

#### treasure_vault.rs
**Oracle text**: {X}{X}, {T}, Sacrifice: Create X Treasure tokens.
**Current state**: Has x_count: 2, approximated as 1 token
**Fix**: Change effect to `Repeat { effect: CreateToken { spec: treasure_token_spec(1) }, count: XValue }`. The x_count: 2 and Cost::Mana already correct. x_value on ActivateAbility needed.

#### crucible_of_the_spirit_dragon.rs
**Oracle text**: {T}, Remove X storage counters: Add X mana in any combination.
**Current state**: TODO
**Fix**: "Remove X counters" as a cost is a separate gap (variable counter removal cost). NOT fixable in PB-27.

### Category C: Planeswalker -X abilities (LoyaltyCost::MinusX already wired)

#### chandra_flamecaller.rs
**Oracle text**: -X: Chandra deals X damage to each creature.
**Current state**: Uses `LoyaltyCost::Minus(1)` and `Effect::Nothing`
**Fix**: Change to `LoyaltyCost::MinusX` and `Effect::ForEach { over: EachCreature, effect: DealDamage { target: IterationTarget, amount: XValue } }`. Wire x_value in replay harness for loyalty abilities (Change 12).

#### ugin_the_spirit_dragon.rs
**Oracle text**: -X: Exile each permanent with MV X or less that's one or more colors. -10: Gain 7, draw 7, put up to 7 permanents.
**Current state**: Missing -X and -10 abilities entirely
**Fix**: Implement -X: `LoyaltyCost::MinusX`, effect exiles permanents matching filter. Needs "MV <= X" filter and "is colored" filter -- partial gap. The -10 ability needs "put from hand to battlefield" which is a separate gap. Implement -X partially (MinusX cost + ForEach exile with available filters), leave TODOs for MV filter and -10.

### Cards NOT in PB-27 scope (count-based, not X-cost)

The following 25 cards from the original ~42 estimate use "X = count of something" patterns that need `PermanentCount`-based dynamic amounts (PB-28 CDA territory), not X-cost infrastructure:
elendas_hierophant, scourge_of_valkas, wrenn_and_seven, elenda_the_dusk_rose, hallowed_spiritkeeper, galadhrim_ambush, ruthless_technomancer (Sacrifice X gap), cavern_hoard_dragon (cost reduction), krenko_mob_boss, chandra_flamecaller (+1 and 0 abilities -- separate gaps), xenagos_the_reveler, phyrexian_swarmlord, dockside_extortionist, myrel_shield_of_argive, commissar_severina_raine, access_denied (MV of countered spell), excise_the_imperfect, overwhelming_stampede, craterhoof_behemoth, spymasters_vault, the_ur_dragon, florian_voldaren_scion, promise_of_power, destiny_spinner, eomer_king_of_rohan, mishra_claimed_by_gix, thieving_skydiver (Kicker X gap).

## Summary of Fixable Cards

**Fully fixable** (all TODOs for this card resolved):
1. pull_from_tomorrow.rs -- just add x_count
2. awaken_the_woods.rs -- add x_count, use Repeat
3. ingenious_prodigy.rs -- add x_count, ETB AddCounterAmount
4. martial_coup.rs -- add x_count, Repeat + Conditional
5. white_suns_twilight.rs -- add x_count, GainLife + Repeat + Conditional
6. treasure_vault.rs -- change to Repeat for token count
7. chandra_flamecaller.rs (-X ability only) -- use MinusX + XValue

**Partially fixable** (mana cost corrected, some effects wired, remaining TODOs):
8. the_meathook_massacre.rs -- ETB -X/-X needs dynamic continuous effect amount
9. spiteful_banditry.rs -- ETB X damage fixed, "once each turn" trigger remains TODO
10. goblin_negotiation.rs -- X damage to target fixed, excess damage tokens remain TODO
11. agadeems_awakening.rs -- mana cost fixed, multi-target GY selection remains TODO
12. finale_of_devastation.rs -- mana cost + conditional fixed, MV filter remains TODO
13. mirror_entity.rs -- X cost wired, dynamic P/T setting remains TODO
14. steel_hellkite.rs -- X cost wired, MV filter + damage tracking remains TODO
15. ugin_the_spirit_dragon.rs -- MinusX cost wired, MV filter remains TODO

**NOT fixable in PB-27** (X refers to count/sacrifice, not mana X):
- grim_hireling.rs -- "Sacrifice X Treasures" variable sacrifice count gap
- crucible_of_the_spirit_dragon.rs -- "Remove X counters" variable counter cost gap

## Unit Tests

**File**: `crates/engine/tests/x_cost_spells.rs`
**Tests to write**:
- `test_x_cost_spell_basic` -- Cast a spell with {X}{G}{G} where X=3; verify generic mana paid = 3, x_value on stack = 3. CR 107.3a.
- `test_x_cost_effect_amount_xvalue` -- Cast an X-cost spell, verify EffectAmount::XValue resolves to the chosen X. CR 107.3m.
- `test_x_cost_etb_counters` -- Cast Ingenious Prodigy with X=4; verify it enters with 4 +1/+1 counters. CR 107.3m.
- `test_x_cost_repeat_tokens` -- Cast Awaken the Woods with X=3; verify 3 tokens created. CR 107.3m.
- `test_x_cost_conditional_xvalue_at_least` -- Martial Coup with X=5: create 5 tokens AND destroy creatures. X=4: create 4 tokens, no destroy. CR 107.3m.
- `test_x_cost_loyalty_minus_x` -- Activate Chandra's -X with X=3; verify 3 damage to each creature and 3 loyalty removed. CR 606.4.
- `test_x_cost_activated_ability` -- Activate Treasure Vault with X=2 ({X}{X}); verify 2 Treasures created. CR 107.3k.
- `test_x_cost_zero_on_stack_permanent` -- After X-cost creature resolves, verify obj.x_value retained for ETB but mana_value of permanent treats X as 0. CR 107.3g.
- `test_x_cost_free_cast_x_is_zero` -- When an effect casts an X spell without paying its mana cost, X must be 0. CR 107.3b.

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` for card-specific integration tests.

## Session Estimate

**2 sessions**:
- Session 1: Engine changes (Changes 1-13) + tests
- Session 2: Card definition fixes (18 cards) + verification

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] Condition::XValueAtLeast wired in check_condition and hash
- [ ] Effect::Repeat wired in execute_effect and hash
- [ ] x_value propagated to ETB trigger EffectContext
- [ ] ActivateAbility.x_value added and wired through handler
- [ ] Replay harness activate_ability and activate_loyalty_ability pass x_value
- [ ] All 18 card defs updated (x_count set, effects wired)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in fully-fixable card defs

## Risks & Edge Cases

- **Dynamic continuous effect amount**: The Meathook Massacre's "-X/-X" ETB needs `LayerModification` to support dynamic amounts from `EffectAmount`. Currently `ModifyBoth(i32)` is fixed. This is a partial fix -- the ETB trigger can be wired but the continuous effect amount will be approximate unless a `LayerModification::ModifyBothDynamic(EffectAmount)` is added. Consider whether to add this in PB-27 or defer.
- **Mirror Entity's dynamic P/T setting**: Similar issue -- `LayerModification::SetBoth` needs to support `EffectAmount`. Deferred to a future PB.
- **{X}{X} double-X costs**: Treasure Vault has `x_count: 2`, meaning generic += 2 * x_value. The engine already handles this (casting.rs line 3215-3217). Verify test coverage.
- **X-cost activated abilities and mana payment**: The activated ability cost-payment path in abilities.rs needs to handle `Cost::Mana` with `x_count > 0`. Currently the mana cost for activated abilities is validated/paid differently from spell casting. Verify the x_count * x_value -> generic conversion happens for activated abilities too.
- **Excess damage tracking** (Goblin Negotiation): Not expressible in current DSL. Partial fix only.
- **Once-per-turn trigger throttle** (Spiteful Banditry): Separate gap not addressed in PB-27.
- **"X can't be 0" constraint** (some cards): Not enforced by engine; validated at deck-builder/legal-action level. Low priority.
