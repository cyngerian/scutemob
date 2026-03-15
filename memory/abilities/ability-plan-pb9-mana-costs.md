# Ability Plan: PB-9 Hybrid Mana, Phyrexian Mana, and X Costs

**Generated**: 2026-03-14
**CR**: 107.3 (X), 107.4e (hybrid), 107.4f (Phyrexian), 202.2d (hybrid color), 202.3e-g (mana value)
**Priority**: W6-PB-9
**Similar abilities studied**: ManaCost struct (game_object.rs:54-70), pay_cost/can_pay_cost (casting.rs:5256-5348), colors_from_mana_cost (casting.rs:5517-5535), add_colors_from_mana_cost (commander.rs:1028-1044), PipTracker (mana_solver.rs:128-172)

## CR Rule Text

### CR 107.3 — X in Mana Costs
107.3. Many objects use the letter X as a placeholder for a number that needs to be determined. Some objects have abilities that define the value of X; the rest let their controller choose the value of X.

107.3a If a spell or activated ability has a mana cost, alternative cost, additional cost, and/or activation cost with an {X}, [-X], or X in it, and the value of X isn't defined by the text of that spell or ability, the controller of that spell or ability chooses and announces the value of X as part of casting the spell or activating the ability.

107.3g If a card in any zone other than the stack has an {X} in its mana cost, the value of {X} is treated as 0, even if the value of X is defined somewhere within its text.

107.3m If an object's enters-the-battlefield triggered ability or replacement effect refers to X, and the spell that became that object as it resolved had a value of X chosen for any of its costs, the value of X for that ability is the same as the value of X for that spell, although the value of X for that permanent is 0.

### CR 107.4e — Hybrid Mana Symbols
107.4e A hybrid mana symbol is also a colored mana symbol, even if one of its components is colorless. Each one represents a cost that can be paid in one of two ways, as represented by the two halves of the symbol. A hybrid symbol such as {W/U} can be paid with either white or blue mana, and a monocolored hybrid symbol such as {2/B} can be paid with either one black mana or two mana of any type. A hybrid mana symbol is all of its component colors.

### CR 107.4f — Phyrexian Mana Symbols
107.4f Phyrexian mana symbols are colored mana symbols: {W/P} is white, {U/P} is blue, {B/P} is black, {R/P} is red, and {G/P} is green. A Phyrexian mana symbol represents a cost that can be paid either with one mana of its color or by paying 2 life. There are also ten hybrid Phyrexian mana symbols. A hybrid Phyrexian mana symbol represents a cost that can be paid with one mana of either of its component colors or by paying 2 life. A hybrid Phyrexian mana symbol is both of its component colors.

### CR 202.2d — Hybrid/Phyrexian Color Identity
202.2d An object with one or more hybrid mana symbols and/or Phyrexian mana symbols in its mana cost is all of the colors of those mana symbols, in addition to any other colors the object might be.

### CR 202.3e-g — Mana Value
202.3e When calculating the mana value of an object with an {X} in its mana cost, X is treated as 0 while the object is not on the stack, and X is treated as the number chosen for it while the object is on the stack.

202.3f When calculating the mana value of an object with a hybrid mana symbol in its mana cost, use the largest component of each hybrid symbol.

202.3g Each Phyrexian mana symbol in a card's mana cost contributes 1 to its mana value.

### CR 903.4 — Color Identity (Commander)
903.4. The Commander variant uses color identity to determine what cards can be in a deck with a certain commander. The color identity of a card is the color or colors of any mana symbols in that card's mana cost or rules text, plus any colors defined by its characteristic-defining abilities or color indicator.

## Key Edge Cases

- **Hybrid mana value uses the LARGER component** (CR 202.3f): `{2/W}` contributes 2 to mana value, not 1. `{W/U}` contributes 1.
- **Hybrid Phyrexian symbols exist** (CR 107.4f): `{W/U/P}`, `{G/W/P}` etc. can be paid with either color OR 2 life. These are both component colors. Ajani, Sleeper Agent uses `{G/W/P}`.
- **Phyrexian mana contributes 1 to mana value** (CR 202.3g), regardless of whether paid with mana or life.
- **X is 0 everywhere except on the stack** (CR 107.3g, 202.3e). Already handled by `x_value` field on StackObject/CastSpell.
- **X counts as generic for mana value on stack** (CR 202.3e). Current code adds `x_value` to `generic` at cast time (casting.rs:3260-3264), which is correct for payment but means `mana_value()` on the ManaCost struct itself doesn't know about X — it's baked into generic. This is acceptable.
- **Hybrid adds BOTH colors to color identity** (CR 903.4): `{W/U}` in a cost means the card's color identity includes both W and U. Commander validation must check both. Currently `add_colors_from_mana_cost` (commander.rs:1028) only checks the 5 color fields — must be extended.
- **Cost reduction interacts with hybrid**: Cost reduction reduces the total cost. Hybrid pips are individual symbols; reducing generic doesn't affect them. Convoke can tap a creature to pay one half of a hybrid. This is complex — defer convoke/hybrid interaction.
- **Monocolored hybrid `{2/W}`**: Can be paid with either {W} OR {2} generic. Mana value contribution is 2 (CR 202.3f — largest component).
- **`{C/W}` colorless hybrid**: Can be paid with either {C} (colorless specifically) or {W}. These exist per CR 107.4 but no printed cards use them in Commander. Include for completeness but low priority.
- **Phyrexian life payment is a cost, not damage** (CR 107.4f). Uses `pay_life`, not `deal_damage`. Cannot be prevented by damage prevention. Life total can go below 0 (player dies to SBA).
- **Compleated** (CR 702.150): Planeswalker enters with 2 fewer loyalty counters per Phyrexian symbol paid with life. Defer to PB-14 (Planeswalker support).
- **Filter lands** (Flooded Grove et al.): The hybrid mana appears in the ACTIVATION COST, not the spell cost. `Cost::Mana(ManaCost { hybrid: ... })` must be payable for activated abilities too.

## Current State (from ability-wip.md)

- [ ] Step 1: Add HybridMana, PhyrexianMana enums + hybrid/phyrexian/x_count fields on ManaCost
- [ ] Step 2: Update mana_value() for hybrid, phyrexian, X
- [ ] Step 3: Update casting.rs payment logic for all three
- [ ] Step 4: Update mana_solver.rs for hybrid/phyrexian/X
- [ ] Step 5: Exhaustive match updates (view_model.rs, stack_view.rs, helpers.rs, etc.)
- [ ] Step 6: Unit tests (cite CR)
- [ ] Step 7: Fix 12 hybrid card defs
- [ ] Step 8: Fix 3 phyrexian card defs
- [ ] Step 9: Fix 4 X-cost card defs
- [ ] Step 10: Build verification

## Modification Surface

Files and functions that need changes, mapped from current ManaCost usage:

| File | Function/Site | Line | What to add |
|------|--------------|------|-------------|
| `state/game_object.rs` | `ManaCost` struct | L54-63 | Add `hybrid`, `phyrexian`, `x_count` fields |
| `state/game_object.rs` | `ManaCost::mana_value()` | L67-69 | Account for hybrid (largest component), phyrexian (1 each), X (0 off stack) |
| `state/hash.rs` | `HashInto for ManaCost` | L784-793 | Hash new fields |
| `rules/casting.rs` | `can_pay_cost()` | L5256-5288 | Handle hybrid payment options, phyrexian life-or-mana |
| `rules/casting.rs` | `pay_cost()` | L5320-5348 | Deduct hybrid/phyrexian from pool (or life for phyrexian) |
| `rules/casting.rs` | X value block | L3256-3267 | Use `x_count` from ManaCost instead of adding to generic blind |
| `rules/casting.rs` | `colors_from_mana_cost()` | L5517-5535 | Include hybrid/phyrexian colors |
| `rules/commander.rs` | `add_colors_from_mana_cost()` | L1028-1044 | Include hybrid/phyrexian colors |
| `cards/helpers.rs` | prelude exports | L1-32 | Export `HybridMana`, `PhyrexianMana` if needed |
| `simulator/mana_solver.rs` | `PipTracker` + `solve_mana_payment()` | L128-172, L21-124 | Handle hybrid/phyrexian pips |
| `tools/tui/src/play/panels/card_detail.rs` | `format_mana_cost()`, `colored_mana_spans()` | L165-239 | Render hybrid/phyrexian/X symbols |
| `testing/replay_harness.rs` | `translate_player_action()` CastSpell sites | L340+ | Phyrexian life payment choice encoding |

## Design Decisions

### 1. HybridMana Enum

```rust
/// A single hybrid mana symbol (CR 107.4e).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HybridMana {
    /// {W/U}, {W/B}, {U/B}, {U/R}, {B/R}, {B/G}, {R/G}, {R/W}, {G/W}, {G/U}
    /// Can be paid with either color.
    ColorColor(ManaColor, ManaColor),
    /// {2/W}, {2/U}, {2/B}, {2/R}, {2/G}
    /// Can be paid with the color OR 2 generic mana.
    GenericColor(ManaColor),
    /// {C/W}, {C/U}, {C/B}, {C/R}, {C/G}
    /// Can be paid with colorless specifically OR the color.
    ColorlessColor(ManaColor),
}
```

Rationale: Enum with 3 variants covers all printed hybrid symbols per CR 107.4. No `{C/X}` cards exist in Commander but the CR defines them. `ColorColor` handles the 10 two-color pairs. `GenericColor` handles the 5 monocolored hybrid `{2/W}` etc.

### 2. PhyrexianMana Enum

```rust
/// A single Phyrexian mana symbol (CR 107.4f).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PhyrexianMana {
    /// {W/P}, {U/P}, {B/P}, {R/P}, {G/P}
    /// Pay with the color OR 2 life.
    Single(ManaColor),
    /// {W/U/P}, {W/B/P}, {U/B/P}, {U/R/P}, {B/R/P}, {B/G/P}, {R/G/P}, {R/W/P}, {G/W/P}, {G/U/P}
    /// Pay with either color OR 2 life.
    Hybrid(ManaColor, ManaColor),
}
```

Rationale: Separate enum for Phyrexian because the life-payment option is fundamentally different from hybrid's two-color choice. `Hybrid` variant handles Compleated symbols like `{G/W/P}`.

### 3. ManaCost Field Additions

```rust
pub struct ManaCost {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
    pub generic: u32,
    /// Hybrid mana symbols (CR 107.4e). Each entry is one hybrid pip.
    #[serde(default)]
    pub hybrid: Vec<HybridMana>,
    /// Phyrexian mana symbols (CR 107.4f). Each entry is one Phyrexian pip.
    #[serde(default)]
    pub phyrexian: Vec<PhyrexianMana>,
    /// Number of {X} symbols in the cost (CR 107.3). Usually 1, but some
    /// cards have {X}{X} (e.g., Treasure Vault). The actual value of X is
    /// chosen at cast time and stored on CastSpell/StackObject.x_value.
    #[serde(default)]
    pub x_count: u32,
}
```

All three new fields have `#[serde(default)]` and derive `Default` (Vec::default() = empty, u32::default() = 0). This preserves the `..Default::default()` pattern used by all 718+ card defs.

### 4. Payment Choice Encoding (Determinism)

For the engine to remain deterministic, hybrid and phyrexian payment choices must be encoded in the Command. Two approaches:

**Option A — Flatten at CastSpell time**: The caller (script harness, simulator, future UI) decides how to pay each hybrid/phyrexian pip BEFORE sending the CastSpell command. Add fields to CastSpell:
```rust
/// For each hybrid pip in the cost, which option was chosen.
/// Length must equal the total hybrid pips in the final cost.
#[serde(default)]
hybrid_choices: Vec<ManaColor>,
/// For each phyrexian pip, true = paid with life, false = paid with mana.
#[serde(default)]
phyrexian_life_payments: Vec<bool>,
```

**Option B — Auto-solve in engine**: The engine picks the cheapest option based on available mana. Simpler but less flexible and may not match player intent.

**Decision: Option A.** The engine is deterministic; choices must come from outside. The harness/simulator makes the choice. For scripts, default to the first color option for hybrid and mana payment for phyrexian (life payment only when explicitly specified).

### 5. Mana Value Calculation

```rust
pub fn mana_value(&self) -> u32 {
    let base = self.white + self.blue + self.black + self.red
             + self.green + self.colorless + self.generic;
    // CR 202.3f: hybrid — use largest component
    let hybrid_mv: u32 = self.hybrid.iter().map(|h| match h {
        HybridMana::ColorColor(_, _) => 1,        // max(1, 1) = 1
        HybridMana::GenericColor(_) => 2,          // max(2, 1) = 2
        HybridMana::ColorlessColor(_) => 1,        // max(1, 1) = 1
    }).sum();
    // CR 202.3g: each phyrexian symbol contributes 1
    let phyrexian_mv = self.phyrexian.len() as u32;
    // CR 202.3e: X is 0 off stack (x_count is structural; actual value is on StackObject)
    base + hybrid_mv + phyrexian_mv
}
```

### 6. X Cost in ManaCost vs CastSpell

Currently, `x_value` on CastSpell gets added to `generic` at cast time (casting.rs:3260-3264). With `x_count` on ManaCost, the logic becomes:
- `x_count` on ManaCost says "this cost has N copies of {X}" (structural, e.g., Treasure Vault has `x_count: 2` for `{X}{X}`)
- `x_value` on CastSpell says "the player chose X = N"
- At cast time: add `x_count * x_value` to generic before payment (replaces the current `generic += x_value`)
- `mana_value()` on ManaCost does NOT include X (it's 0 off stack; on stack, the X has already been folded into generic by casting.rs)

## Implementation Steps

### Step 1: Add Types and Fields to ManaCost

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**:
1. Add `HybridMana` enum (before ManaCost) with `ColorColor(ManaColor, ManaColor)`, `GenericColor(ManaColor)`, `ColorlessColor(ManaColor)`. Derive: `Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize`.
2. Add `PhyrexianMana` enum with `Single(ManaColor)`, `Hybrid(ManaColor, ManaColor)`. Same derives.
3. Add three fields to `ManaCost`:
   - `hybrid: Vec<HybridMana>` with `#[serde(default)]`
   - `phyrexian: Vec<PhyrexianMana>` with `#[serde(default)]`
   - `x_count: u32` with `#[serde(default)]`
4. Update `ManaCost::mana_value()` per the design above.

**CR**: 107.4e, 107.4f, 202.3e, 202.3f, 202.3g

**Note**: ManaCost derives `Default` — `Vec` defaults to empty, `u32` to 0. All existing `..Default::default()` patterns continue to work.

### Step 2: Update Hash

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**:
1. Add `HashInto` impl for `HybridMana` (discriminant + inner colors).
2. Add `HashInto` impl for `PhyrexianMana` (discriminant + inner colors).
3. Extend `HashInto for ManaCost` (L784-793) to hash `hybrid`, `phyrexian`, and `x_count`.

**Pattern**: Follow existing `HashInto for Vec<T>` pattern (hash length, then each element).

### Step 3: Add Payment Choice Fields to CastSpell

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add to `Command::CastSpell`:
```rust
/// For each hybrid pip in the resolved cost, which color was chosen to pay.
/// For GenericColor hybrids, the color means "pay colored"; absence or a
/// sentinel could mean "pay 2 generic" — but using ManaColor is simplest.
/// Length must match total hybrid pips after cost calculation.
#[serde(default)]
hybrid_choices: Vec<HybridManaPayment>,
/// For each phyrexian pip, true = pay 2 life; false = pay mana.
#[serde(default)]
phyrexian_life_payments: Vec<bool>,
```

Add `HybridManaPayment` enum:
```rust
pub enum HybridManaPayment {
    /// Pay with the first color option (or the colored option for GenericColor).
    Color(ManaColor),
    /// Pay with 2 generic mana (only valid for GenericColor hybrids).
    Generic,
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Mirror `hybrid_choices` and `phyrexian_life_payments` on StackObject if needed for resolution tracking. Likely not needed since payment happens at cast time, but review.

### Step 4: Update Payment Logic in casting.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`

**4a. Update `can_pay_cost()` (L5256-5288)**:
The current signature only takes `ManaPool` and `ManaCost`. With hybrid/phyrexian, the function needs to know the payment choices. Two options:
- **Option 1**: Expand `can_pay_cost` signature to take choices. Complex.
- **Option 2**: "Flatten" hybrid/phyrexian into colored/generic pips based on choices BEFORE calling `can_pay_cost`. The ManaCost passed to `can_pay_cost` would already have hybrid resolved into colored pips and phyrexian resolved into colored pips (life-paid ones removed).

**Decision: Option 2 (flatten first).** Add a helper:
```rust
/// CR 601.2f: Resolve hybrid and phyrexian payment choices into a flat ManaCost.
/// Hybrid pips become colored or generic pips based on choices.
/// Phyrexian pips paid with life are removed from the mana cost (life deducted separately).
/// Phyrexian pips paid with mana become colored pips.
fn flatten_hybrid_phyrexian(
    cost: &ManaCost,
    hybrid_choices: &[HybridManaPayment],
    phyrexian_life_payments: &[bool],
) -> (ManaCost, u32) // (flattened mana cost, life to pay)
```

This function returns a ManaCost with only standard fields (no hybrid/phyrexian) plus a life cost. The existing `can_pay_cost` and `pay_cost` work unchanged on the flattened cost. Life is deducted separately.

**4b. X cost update (L3256-3267)**:
Replace:
```rust
let mana_cost = if x_value > 0 {
    mana_cost.map(|mut c| { c.generic += x_value; c })
} else { mana_cost };
```
With:
```rust
let mana_cost = mana_cost.map(|mut c| {
    // CR 107.3a: Add x_count * x_value to generic.
    c.generic += c.x_count * x_value;
    c.x_count = 0; // consumed
    c
});
```

**4c. Flatten + pay hybrid/phyrexian before standard payment**:
After total cost is determined (after affinity, convoke, etc. reductions), flatten hybrid/phyrexian choices, then pay life for phyrexian-life choices, then pay remaining mana normally.

**CR**: 601.2f

### Step 5: Update Color Identity and Color Derivation

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/commander.rs`
**Action**: Update `add_colors_from_mana_cost()` (L1028-1044) to include colors from `hybrid` and `phyrexian` fields:
- Each `HybridMana::ColorColor(a, b)` adds both colors
- Each `HybridMana::GenericColor(c)` adds that color
- Each `HybridMana::ColorlessColor(c)` adds that color
- Each `PhyrexianMana::Single(c)` adds that color
- Each `PhyrexianMana::Hybrid(a, b)` adds both colors

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Update `colors_from_mana_cost()` (L5517-5535) similarly. A card with `{G/W}` in its cost is both green and white.

### Step 6: Update Mana Solver (Simulator)

**File**: `/home/airbaggie/scutemob/crates/simulator/src/mana_solver.rs`
**Action**:
1. Update `PipTracker::from_cost()` to track hybrid/phyrexian pips.
2. Add hybrid payment: for each `ColorColor(a, b)`, try to pay with `a` first, fall back to `b`. For `GenericColor(c)`, try color first (cheaper), fall back to 2 generic.
3. Add phyrexian payment: always pay with mana (AI should not pay life unless strategically beneficial — for now, mana-first).
4. Return `hybrid_choices` and `phyrexian_life_payments` alongside the Command list so the solver can populate CastSpell fields.

**Note**: The mana_solver currently returns `Vec<Command>` (TapForMana commands). It may need to return a richer struct that includes hybrid/phyrexian choices.

### Step 7: Update Display (TUI)

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/card_detail.rs`
**Action**: Update `format_mana_cost()` (L165-193) and `colored_mana_spans()` (L210-239) to render hybrid/phyrexian/X symbols:
- Hybrid `ColorColor(W, U)` renders as `{W/U}`
- Hybrid `GenericColor(W)` renders as `{2/W}`
- Phyrexian `Single(B)` renders as `{B/P}`
- Phyrexian `Hybrid(G, W)` renders as `{G/W/P}`
- X cost: render `{X}` repeated `x_count` times

### Step 8: Update Replay Harness

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**:
1. Add `hybrid_choices` and `phyrexian_life_payments` to the CastSpell construction sites. Default both to empty (= pay first color, pay mana).
2. For scripts that need phyrexian life payment, add a `phyrexian_pay_life: Vec<bool>` field to `PlayerAction` (script_schema.rs).
3. For X spells, `x_value` already propagates (L355-356).

### Step 9: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/mana_costs.rs` (NEW)
**Tests to write**:

1. `test_hybrid_mana_value_color_color` — CR 202.3f: `{W/U}` contributes 1 to mana value. Kitchen Finks `{1}{G/W}{G/W}` has MV 3.
2. `test_hybrid_mana_value_generic_color` — CR 202.3f: `{2/W}` contributes 2 to mana value (largest component).
3. `test_phyrexian_mana_value` — CR 202.3g: `{W/P}` contributes 1 to mana value.
4. `test_x_mana_value_off_stack` — CR 202.3e: ManaCost with `x_count: 1` has MV = sum of other pips (X=0 off stack).
5. `test_hybrid_payment_color_color` — Cast Kitchen Finks paying {1}{G}{W} or {1}{G}{G} or {1}{W}{W}. All valid.
6. `test_hybrid_payment_generic_color` — Cast a spell with `{2/W}`: can pay {W} or {2}.
7. `test_phyrexian_payment_mana` — Pay `{W/P}` with white mana: pool decreases, life unchanged.
8. `test_phyrexian_payment_life` — Pay `{W/P}` with 2 life: pool unchanged, life decreases by 2.
9. `test_x_cost_payment` — Cast spell with `x_count: 1`, `x_value: 3`: total cost includes 3 generic.
10. `test_x_cost_double_x` — Cast with `x_count: 2`, `x_value: 3`: total cost includes 6 generic (Treasure Vault pattern).
11. `test_hybrid_color_identity` — CR 903.4: `{G/W}` in mana cost adds both G and W to color identity.
12. `test_phyrexian_color_identity` — CR 903.4: `{B/P}` in mana cost adds B to color identity.
13. `test_hybrid_phyrexian_color_identity` — CR 903.4: `{G/W/P}` adds both G and W.
14. `test_hybrid_colors_from_cost` — `colors_from_mana_cost` includes hybrid colors.
15. `test_phyrexian_insufficient_life` — Paying `{W/P}` with life when at 1 life: should fail (life cannot go below... actually MTG allows going to negative life; SBA kills you. Check: paying life as a cost requires having at least that much life? CR 119.4: "a player who has 0 or less life loses as an SBA." But paying life doesn't require > 0; you just can't pay more life than you have. CR 118.3 says "can't pay costs they can't pay." CR 119.4: life loss as cost — if you'd go below 0, you can't pay? Check.)

**Pattern**: Follow existing test files in `crates/engine/tests/`. Use `GameStateBuilder`, `process_command`, assertions on game state.

### Step 10: Fix Hybrid Card Defs (12 cards)

**Cards to fix** (replace approximated costs with proper hybrid):

| Card | Current Approximation | Correct Hybrid Cost |
|------|----------------------|-------------------|
| `kitchen_finks.rs` | `generic: 1, white: 1, green: 1` | `generic: 1, hybrid: vec![ColorColor(G, W), ColorColor(G, W)]` |
| `boggart_ram_gang.rs` | `red: 3` | `hybrid: vec![ColorColor(R, G), ColorColor(R, G), ColorColor(R, G)]` |
| `blade_historian.rs` | `red: 2, white: 2` | `hybrid: vec![ColorColor(R, W), ColorColor(R, W), ColorColor(R, W), ColorColor(R, W)]` |
| `connive.rs` | `generic: 2, blue: 1, black: 1` | `generic: 2, hybrid: vec![ColorColor(U, B), ColorColor(U, B)]` |
| `revitalizing_repast.rs` | empty/zero | `hybrid: vec![ColorColor(B, G)]` |
| `nethroi_apex_of_death.rs` | main cost has `green: 1` approx for hybrid | Fix main cost + mutate cost hybrid |
| `brokkos_apex_of_forever.rs` | main cost has `black: 1` approx for hybrid | Fix main cost + mutate cost hybrid |

**Filter lands** (hybrid in ACTIVATION cost, not mana cost):
| Card | Fix |
|------|-----|
| `twilight_mire.rs` | Add activated ability with `Cost::Mana(ManaCost { hybrid: vec![ColorColor(B, G)], ..Default::default() })` + Tap |
| `rugged_prairie.rs` | Same pattern with `ColorColor(R, W)` |
| `fetid_heath.rs` | Same pattern with `ColorColor(W, B)` |
| `flooded_grove.rs` | Same pattern with `ColorColor(G, U)` |
| `cascade_bluffs.rs` | Same pattern with `ColorColor(U, R)` |
| `sunken_ruins.rs` | Same pattern with `ColorColor(U, B)` |
| `graven_cairns.rs` | Same pattern with `ColorColor(B, R)` |

**Note**: Filter lands also need the 3-way mana output choice. This is a separate DSL gap (modal mana output). PB-9 adds the hybrid cost field; the output choice may need a `Choose` variant or stay as TODO. Assess at implementation time.

### Step 11: Fix Phyrexian Card Defs (3 cards)

| Card | Fix |
|------|-----|
| `skrelv_defector_mite.rs` | Activated ability cost: `Cost::Mana(ManaCost { phyrexian: vec![Single(W)], ..Default::default() })` + Tap |
| `tekuthal_inquiry_dominus.rs` | Activated ability cost: `Cost::Sequence(vec![Cost::Mana(ManaCost { generic: 1, phyrexian: vec![Single(U), Single(U)], ..Default::default() }), ...])` |
| `drivnod_carnage_dominus.rs` | Activated ability cost: `Cost::Mana(ManaCost { phyrexian: vec![Single(B), Single(B)], ..Default::default() })` + exile cost |
| `ajani_sleeper_agent.rs` | Main cost: `ManaCost { generic: 1, green: 1, white: 1, phyrexian: vec![Hybrid(G, W)], ..Default::default() }` — defer Compleated to PB-14 |

**Note**: Tekuthal and Drivnod have other DSL gaps beyond hybrid/phyrexian (remove-counters cost, exile-from-graveyard cost, proliferate doubling, death-trigger doubling). Only fix the mana cost representation; other TODOs remain.

### Step 12: Fix X-Cost Card Defs (4 cards)

| Card | Fix |
|------|-----|
| `mockingbird.rs` | `mana_cost: Some(ManaCost { blue: 1, x_count: 1, ..Default::default() })` |
| `cut_ribbons.rs` | Aftermath half cost: `ManaCost { black: 2, x_count: 1, ..Default::default() }` + `EffectAmount::XValue` for LoseLife |
| `treasure_vault.rs` | Activated ability cost: `ManaCost { x_count: 2, ..Default::default() }` + Tap + SacrificeSelf; effect uses `EffectAmount::XValue` for token count |
| `florian_voldaren_scion.rs` | NOT an X mana cost — the X refers to life opponents lost. Remove from PB-9 scope. |

**Important**: `florian_voldaren_scion.rs` does NOT have X in its mana cost ({1}{B}{R}). The "X" in its oracle text refers to a computed value from game state, not a mana cost X. This card is out of scope for PB-9.

### Step 13: Build Verification

Run `~/.cargo/bin/cargo build --workspace` to catch compile errors in:
- replay-viewer (`view_model.rs` — ManaCost display if any)
- TUI (`card_detail.rs`)
- simulator (`mana_solver.rs`)

Run `~/.cargo/bin/cargo test --all` and `~/.cargo/bin/cargo clippy -- -D warnings`.

## Interactions to Watch

- **Cost reduction + hybrid**: Convoke/Improvise reduce generic mana. They should NOT reduce hybrid pips. The flatten-first approach handles this: hybrid pips are resolved to colored or generic BEFORE convoke/improvise reduction. If a hybrid `{2/W}` is chosen as "pay 2 generic", those 2 generic pips ARE subject to further reduction. This is correct per CR.
- **Affinity + hybrid**: Same as above — affinity reduces generic. Only affects hybrid if the player chose the generic payment option for `{2/W}`.
- **Commander tax + hybrid**: Tax adds to generic, not hybrid. No interaction.
- **Prototype + hybrid/phyrexian**: If a prototype cost contains hybrid mana, the alt cost's ManaCost would have hybrid fields. The existing prototype code uses ManaCost — should work seamlessly.
- **Copy effects on X spells**: `x_value` on StackObject is already copied. No change needed.
- **Layer system**: ManaCost is not used in the layer system. No interaction.
- **Phyrexian life payment + Platinum Emperion**: "Your life total can't change" prevents Phyrexian life payment. This is a future interaction — the engine doesn't yet have "can't change life total" effects.

## Scope Clarification

**In scope for PB-9**:
- `HybridMana` and `PhyrexianMana` enums
- `ManaCost` field additions (hybrid, phyrexian, x_count)
- `mana_value()` update
- `can_pay_cost()` and `pay_cost()` with flatten helper
- CastSpell `hybrid_choices` and `phyrexian_life_payments` fields
- Color identity updates
- Hash updates
- TUI display updates
- Unit tests
- Card def fixes for mana cost representation
- Mana solver update

**Out of scope (deferred)**:
- Compleated keyword (PB-14, Planeswalker support)
- Filter land 3-way mana output choice (separate DSL gap — modal mana production)
- Phyrexian mana interaction with "can't pay life" or "life total can't change"
- `{C/W}` colorless hybrid — include enum variant but no cards use it
- Interactive hybrid/phyrexian choice UI (the engine encodes choices; UI is M10+)
