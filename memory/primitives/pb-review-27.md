# Primitive Batch Review: PB-27 -- X-Cost Spells

**Date**: 2026-03-24
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 107.3, 107.3a, 107.3b, 107.3f, 107.3g, 107.3k, 107.3m, 606.4, 606.6, 614.1c
**Engine files reviewed**: `card_definition.rs` (Condition::XValueAtLeast, Effect::Repeat), `effects/mod.rs` (check_condition, execute_effect, resolve_amount), `state/hash.rs` (discriminants 37, 67), `rules/command.rs` (ActivateAbility.x_value, ActivateLoyaltyAbility.x_value), `rules/engine.rs` (x_value extraction), `rules/abilities.rs` (handle_activate_ability x_value wiring, mana cost calculation), `rules/resolution.rs` (x_value propagation to ETB), `testing/replay_harness.rs` (activate_ability, activate_loyalty_ability x_value wiring)
**Card defs reviewed**: 15 (pull_from_tomorrow, awaken_the_woods, ingenious_prodigy, martial_coup, white_suns_twilight, treasure_vault, chandra_flamecaller, the_meathook_massacre, spiteful_banditry, goblin_negotiation, agadeems_awakening, finale_of_devastation, mirror_entity, steel_hellkite, ugin_the_spirit_dragon)

## Verdict: needs-fix

The engine primitives (Condition::XValueAtLeast, Effect::Repeat, x_value on ActivateAbility,
ETB x_value propagation) are correctly implemented and hash discriminants are sequential
with no collisions. The x_value propagation chain from CastSpell through StackObject to
GameObject to ETB EffectContext is complete and correct per CR 107.3m. Replay harness wiring
for both activate_ability and activate_loyalty_ability is present. However, there are 2
MEDIUM issues in card definitions (Ingenious Prodigy ETB modeled as trigger instead of
replacement effect per CR 614.1c; Martial Coup DestroyAll hits its own tokens) and 4 LOW
issues (Finale +X/+X approximation, Ugin missing -10 ability, Ingenious Prodigy upkeep
"may" not modeled, test gap for free-cast X=0).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| (none) | -- | -- | All engine changes are correct. |

No engine findings. Condition::XValueAtLeast correctly compares `ctx.x_value >= *n` (both u32).
Effect::Repeat correctly calls `resolve_amount().max(0) as u32` and loops. Hash discriminants
37 (Condition) and 67 (Effect) are sequential after 36 and 66 respectively. The
check_static_condition fallthrough correctly yields x_value=0 for static contexts per CR 107.3g.
ActivateAbility.x_value is wired through engine.rs to abilities.rs, which sets
`stack_obj.x_value` and adds `x_count * x_value` to generic mana cost. Resolution.rs propagates
x_value from StackObject to GameObject (line 531) and from GameObject to ETB EffectContext
(line 1937).

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `ingenious_prodigy.rs` | **ETB counters modeled as triggered ability instead of replacement effect.** CR 614.1c says "enters with" is a replacement effect, not a trigger. |
| 2 | MEDIUM | `martial_coup.rs` | **DestroyAll hits own Soldier tokens.** Oracle says "destroy all other creatures" -- the tokens should survive. |
| 3 | LOW | `finale_of_devastation.rs` | **+X/+X approximated as fixed +10/+10.** Wrong when X > 10. |
| 4 | LOW | `ugin_the_spirit_dragon.rs` | **Missing -10 loyalty ability entirely.** Only +2 and -X present. |
| 5 | LOW | `ingenious_prodigy.rs` | **Upkeep "you may" not modeled.** Should use optional choice, currently unconditional. |
| 6 | LOW | `chandra_flamecaller.rs` | **+1 and 0 abilities both Effect::Nothing.** Documented as TODO for delayed exile and hand-size draw. |

### Finding Details

#### Finding 1: Ingenious Prodigy ETB -- triggered ability vs. replacement effect

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/ingenious_prodigy.rs:22-31`
**CR Rule**: 614.1c -- "Effects that read '[This permanent] enters with . . .' are replacement effects."
**Oracle**: "This creature enters with X +1/+1 counters on it."
**Issue**: The card def models this as a `TriggerCondition::WhenEntersBattlefield` triggered
ability that places counters after the creature enters (going on the stack, requiring priority
passes to resolve). Per CR 614.1c, "enters with" is a replacement effect -- the counters should
be placed as part of entering the battlefield, simultaneously, without using the stack. This
means: (a) opponents cannot respond before counters are placed, (b) SBAs see the creature with
counters already on it. For Ingenious Prodigy (0/1 base), the creature survives either way, but
the timing is incorrect and visible to players (extra priority pass required in tests).
**Fix**: This is a known engine-wide pattern limitation -- the DSL has no `EntersWithCounters`
replacement effect primitive. The current approach is the standard workaround used by Ravenous
(which handles this in resolution.rs specifically). For now, accept the triggered-ability
approximation but add a comment citing CR 614.1c. A proper fix would add an
`EntersWithCounters { counter, count: EffectAmount }` replacement effect, but that is out of
scope for PB-27. **Action**: Add a comment to the card def noting the CR 614.1c deviation.

#### Finding 2: Martial Coup DestroyAll hits own tokens

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/martial_coup.rs:37-47`
**Oracle**: "Create X 1/1 white Soldier creature tokens. If X is 5 or more, destroy all other creatures."
**Issue**: The `DestroyAll` filter is `TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() }` which matches ALL creatures on the battlefield, including the Soldier tokens just created by the preceding Repeat effect. Oracle says "all OTHER creatures" -- meaning all creatures except the tokens created by this spell. The test at line 496 acknowledges this: "the current DestroyAll implementation hits all creatures including the newly created Soldiers." This produces wrong game state when X >= 5.
**Fix**: This requires a `DestroyAllExcept` variant or an `exclude_created_this_effect` flag
on TargetFilter, neither of which exists. This is a DSL gap beyond PB-27 scope. **Action**: Add
a TODO comment in the card def explicitly noting that the DestroyAll incorrectly destroys the
Soldier tokens and citing the oracle text. Also add a note in the test acknowledging wrong game
state.

#### Finding 3: Finale of Devastation +X/+X approximated as +10/+10

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/finale_of_devastation.rs:48`
**Oracle**: "creatures you control get +X/+X and gain haste until end of turn"
**Issue**: `LayerModification::ModifyBoth(10)` is a fixed +10/+10 rather than dynamic +X/+X.
When X > 10 (e.g., X=15), creatures only get +10/+10 instead of +15/+15. Documented with TODO.
**Fix**: No action needed beyond the existing TODO. Requires `LayerModification::ModifyBothDynamic(EffectAmount)`.

#### Finding 4: Ugin missing -10 loyalty ability

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/ugin_the_spirit_dragon.rs:48-53`
**Oracle**: "-10: You gain 7 life, draw seven cards, then put up to seven permanent cards from your hand onto the battlefield."
**Issue**: The -10 ability is entirely absent from the abilities vec. Only +2 and -X are present.
The GainLife and DrawCards portions are expressible but the hand-to-battlefield portion is not.
**Fix**: Add a partial -10 ability with `LoyaltyCost::Minus(10)` implementing `GainLife(7)` and
`DrawCards(7)` as a Sequence, with a TODO for the "put up to seven permanent cards" portion.
Having a partial ability is better than a missing ability because it at least tracks loyalty
cost correctly and delivers the life/cards.

#### Finding 5: Ingenious Prodigy upkeep "you may" not modeled

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/ingenious_prodigy.rs:33-51`
**Oracle**: "you may remove a +1/+1 counter from it. If you do, draw a card."
**Issue**: The upkeep ability unconditionally removes a counter and draws. Oracle says "you MAY"
-- this should be optional. The DSL has `Effect::MayPayOrElse` but integrating it with
"remove counter as cost" is non-trivial.
**Fix**: No action needed for PB-27 scope. Add a comment noting the "may" is not modeled.

#### Finding 6: Chandra Flamecaller +1 and 0 abilities

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/chandra_flamecaller.rs:22-35`
**Oracle**: "+1: Create two 3/1 red Elemental creature tokens with haste. Exile them at the beginning of the next end step. 0: Discard all the cards in your hand, then draw that many cards plus one."
**Issue**: Both abilities use `Effect::Nothing`. The +1 needs delayed exile triggers and the 0
needs hand-size tracking. Both are documented with TODO comments.
**Fix**: No action for PB-27. These are separate DSL gaps (delayed triggers, hand-size amount).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 107.3a (X in costs) | Yes | Yes | test_x_cost_spell_basic_mana_payment |
| 107.3b (free cast X=0) | Partial | No | Engine handles via casting.rs but no dedicated test |
| 107.3g (X=0 off stack) | Yes | Partial | test_x_cost_permanent_retains_x_value_for_etb checks permanent.x_value but not mana_value=0 |
| 107.3k (activated ability X) | Yes | Yes | test_x_cost_activated_ability_double_x_treasure_vault |
| 107.3m (ETB X propagation) | Yes | Yes | test_x_cost_etb_counters_ingenious_prodigy, test_x_cost_permanent_retains_x_value_for_etb |
| 606.4 (loyalty costs) | Yes | No | LoyaltyCost::MinusX wired but no test for Chandra -X |
| 606.6 (negative loyalty req) | Yes | No | MinusX handler checks loyalty >= x but no test |
| 614.1c (enters with = replacement) | No | No | Modeled as trigger (Finding 1) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| pull_from_tomorrow | Yes | 0 | Yes | Fully correct |
| awaken_the_woods | Yes | 0 | Yes | Fully correct |
| ingenious_prodigy | Partial | 0 | Partial | ETB is trigger not replacement (F1); upkeep "may" missing (F5) |
| martial_coup | No | 0 | No | DestroyAll hits own tokens (F2) |
| white_suns_twilight | Yes | 0 | Partial | "Can't block" on tokens not modeled (noted in comment) |
| treasure_vault | Yes | 0 | Yes | Fully correct |
| chandra_flamecaller | Partial | 2 | Partial | +1 and 0 are Effect::Nothing (F6); -X is correct |
| the_meathook_massacre | Partial | 1 | Partial | ETB -X/-X TODO (dynamic layer mod); death triggers correct |
| spiteful_banditry | Partial | 1 | Partial | ETB X damage correct; once-per-turn trigger TODO |
| goblin_negotiation | Partial | 1 | Partial | X damage correct; excess damage tokens TODO |
| agadeems_awakening | Partial | 1 | Partial | Mana cost correct; multi-target GY selection TODO |
| finale_of_devastation | Partial | 2 | Partial | XValueAtLeast(10) correct; +X/+X fixed at +10 (F3); MV filter TODO |
| mirror_entity | Partial | 2 | Partial | X cost wired; dynamic P/T TODO; all creature types TODO |
| steel_hellkite | Partial | 3 | Partial | X cost wired; MV filter, damage tracking, once-per-turn TODO |
| ugin_the_spirit_dragon | Partial | 3 | Partial | +2 correct; -X cost wired; -10 missing (F4); MV/color filter TODO |

## Test Assessment

10 tests covering: basic mana payment, XValue draw effect, ETB counters, Repeat token creation,
XValueAtLeast positive and negative cases (X=5 and X=4 for Martial Coup), double-X activated
ability (Treasure Vault), permanent x_value retention, Repeat with X=0, and White Sun's Twilight
below threshold. Good positive and negative coverage. Missing: loyalty -X test (Chandra),
free-cast X=0 test (CR 107.3b), mana_value=0 on permanent test (CR 107.3g).
