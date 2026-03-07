# Ability Plan: Spree

**Generated**: 2026-03-07
**CR**: 702.172
**Priority**: P4
**Similar abilities studied**: Escalate (702.120) in `crates/engine/src/rules/casting.rs:1955-2011`, `crates/engine/tests/escalate.rs`; Modal choice (700.2) in `casting.rs:2871-2930`, `resolution.rs:271-309`

## CR Rule Text

702.172. Spree

702.172a Spree is a static ability found on some modal spells (see rule 700.2) that
applies while the spell on the stack. Spree means "Choose one or more modes. As an
additional cost to cast this spell, pay the costs associated with those modes."

702.172b Cards with the spree ability have a plus sign icon in the upper right corner
of the card, and use a plus sign (+) rather than traditional bullet points. These symbols
are a visual reminder that this card requires an additional cost to be cast, and do not
have additional rules meaning.

**Supporting rule (CR 700.2h):** "Some modal spells have one or more modes with a cost
listed before the effect of that mode. This indicates that the mode has an additional
cost that must be paid as the spell is cast if that mode is chosen. If more than one
such mode is chosen, all additional costs must be paid to cast that spell. Paying these
costs follows the rules for paying additional costs in rules 601.2b and 601.2f-h."

## Key Edge Cases

From card rulings (all 2024-04-12):
- **"Without paying its mana cost" still requires mode costs.** If an effect allows you to
  cast a spell with spree "without paying its mana cost," you must still choose at least one
  mode and pay the associated additional costs. (Important: alt-cost bypasses base cost but
  NOT per-mode additional costs.)
- **Can't choose the same mode more than once** (standard CR 700.2d behavior).
- **Must choose at least one mode.** You cannot cast a spree spell choosing zero modes.
- **Mode order is fixed.** No matter which modes are chosen, effects execute in printed order
  (ascending mode index). Same as all modal spells (CR 700.2a).
- **Copies cannot change modes** (CR 700.2g). If a spree spell is copied, the copy uses the
  same modes. The copier may choose new targets but not new modes.
- **Mana value is based on printed mana cost only.** The additional mode costs do not change
  the spell's mana value (CR 118.8d).
- **Mode targeting.** If a mode requires a target, you can select that mode only if there's a
  legal target. Ignore targeting for unchosen modes (CR 700.2c).
- **All modes resolve together.** No player can cast spells or activate abilities between modes.
- **Multiplayer:** No special multiplayer considerations beyond standard modal spell rules.
  Each chosen mode's targets are chosen at cast time per CR 601.2c.

## How Spree Differs from Escalate

| Aspect | Escalate (702.120) | Spree (702.172) |
|--------|-------------------|-----------------|
| Cost model | Single flat cost per extra mode beyond first | Each mode has its OWN cost |
| First mode | Free (included in base cost) | Paid (its mode cost is added) |
| Cost storage | `AbilityDefinition::Escalate { cost: ManaCost }` | Per-mode costs on `ModeSelection` |
| Mode selection | `escalate_modes: u32` (count of extras) | `modes_chosen: Vec<usize>` (explicit indices) |
| Resolution | modes 0..=N | chosen modes in printed order |

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- Spree is static, not triggered)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Spree` unit variant after `Fuse` (line ~1230).
**Discriminant**: 134
**Pattern**: Follow `KeywordAbility::Escalate` at line 1031 -- unit variant, no parameter.

```
/// CR 702.172a: Spree -- static ability on modal spells. "Choose one or more
/// modes. As an additional cost to cast this spell, pay the costs associated
/// with those modes." Each mode has its own additional cost (CR 700.2h).
///
/// Static ability. Marker for quick presence-checking (`keywords.contains`).
/// The per-mode costs are stored in `ModeSelection.mode_costs`.
///
/// Discriminant 134.
Spree,
```

**Hash**: Add to `state/hash.rs` HashInto impl for KeywordAbility, after Fuse (line ~656):
```
// Spree (discriminant 134) -- CR 702.172
KeywordAbility::Spree => 134u8.hash_into(hasher),
```

**Match arms to update**:
1. `tools/replay-viewer/src/view_model.rs` -- keyword display function (line ~853): add
   `KeywordAbility::Spree => "Spree".to_string(),`
2. `tools/tui/src/play/panels/stack_view.rs` -- check if KeywordAbility is matched here
   (unlikely since KW is not a SOK, but verify)

### Step 2: ModeSelection Extension (Per-Mode Costs)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a `mode_costs` field to `ModeSelection` (line ~1212).

```rust
/// CR 700.2h / 702.172a: Per-mode additional costs for spree spells.
/// When present, `mode_costs[i]` is the additional cost that must be paid
/// when mode `i` is chosen. Must have the same length as `modes`.
/// `None` for standard modal spells (no per-mode costs).
#[serde(default)]
pub mode_costs: Option<Vec<ManaCost>>,
```

**Why Option<Vec> instead of Vec**: Standard modal spells (Abzan Charm, etc.) have no per-mode
costs. Making it `Option` avoids changing every existing card definition. `None` = no per-mode
costs. `Some(vec![...])` = spree-style per-mode costs.

**Hash**: Update `HashInto for ModeSelection` in `state/hash.rs` (line ~3257) to hash
`mode_costs`:
```rust
if let Some(costs) = &self.mode_costs {
    true.hash_into(hasher);
    for cost in costs {
        cost.hash_into(hasher);
    }
} else {
    false.hash_into(hasher);
}
```

**No new AbilityDefinition variant needed.** Unlike Escalate which stores its flat cost in
`AbilityDefinition::Escalate { cost }`, Spree's costs live directly on `ModeSelection.mode_costs`.
The `KeywordAbility::Spree` marker is sufficient for identification. AbilityDefinition next
discriminant stays at 52.

### Step 3: Casting Enforcement

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add a Spree cost-addition block AFTER the Escalate block (~line 2011) and BEFORE
the Splice block (~line 2013).

**CR**: 702.172a + 700.2h -- "As an additional cost to cast this spell, pay the costs
associated with those modes."

**Logic**:
1. Check if the spell has `KeywordAbility::Spree`.
2. If yes, look up `ModeSelection` from the card definition.
3. Validate `mode_costs` is `Some` and has the right length.
4. For each mode index in `modes_chosen`, look up `mode_costs[index]` and add it to the
   total mana cost.
5. If `modes_chosen` is empty, reject -- Spree requires at least one mode (CR 702.172a).

**Important interaction with existing modes_chosen validation** (line 2874):
The existing modal validation at line 2874 already validates `modes_chosen` indices, count
against min_modes/max_modes, and no-duplicate rules. Spree spells will have `min_modes: 1`
and `max_modes: <mode_count>` in their ModeSelection, which the existing validation handles
correctly. The Spree cost block just adds the per-mode costs on top.

**Alt-cost interaction** (CR ruling): "without paying its mana cost" skips the base mana cost
but NOT the Spree per-mode costs. The existing alt_cost path zeroes the base cost. Since Spree
costs are additional costs (CR 601.2f-h), they are added after the base cost is determined,
so they survive alt-cost zeroing. This matches the Escalate pattern -- alt_cost zeroes base,
then escalate/spree adds on top.

**Pseudo-code**:
```rust
// CR 702.172a / 700.2h: Spree -- add per-mode additional costs for each chosen mode.
let mana_cost = if chars.keywords.contains(&KeywordAbility::Spree) {
    // Spree requires modes_chosen to be non-empty (at least one mode).
    if validated_modes_chosen.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "spree spell requires at least one mode to be chosen (CR 702.172a)".into(),
        ));
    }
    // Look up per-mode costs from ModeSelection.
    let mode_costs = card_id.as_ref().and_then(|cid| {
        state.card_registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Spell { modes: Some(m), .. } = a {
                    m.mode_costs.clone()
                } else {
                    None
                }
            })
        })
    });
    match mode_costs {
        Some(costs) => {
            let mut total = mana_cost.unwrap_or_default();
            for &idx in &validated_modes_chosen {
                if let Some(cost) = costs.get(idx) {
                    total.white += cost.white;
                    total.blue += cost.blue;
                    total.black += cost.black;
                    total.red += cost.red;
                    total.green += cost.green;
                    total.generic += cost.generic;
                    total.colorless += cost.colorless;
                }
            }
            Some(total)
        }
        None => {
            return Err(GameStateError::InvalidCommand(
                "spree spell has no per-mode costs defined (CR 700.2h)".into(),
            ));
        }
    }
} else {
    mana_cost
};
```

**Ordering concern**: The Spree cost block must come AFTER `validated_modes_chosen` is computed
(line ~2930) but BEFORE `pay_cost`. Currently the cost pipeline is:
1. Base cost
2. Alt cost (Flashback, etc.) -- may zero the base
3. Commander tax
4. Kicker
5. Entwine
6. Escalate
7. Splice
8. Convoke/Improvise/Delve (reductions)
9. Pay

Spree fits at position 6.5 (after Escalate, before Splice). However, note that
`validated_modes_chosen` is computed AFTER the Escalate block. The Spree block must use
`validated_modes_chosen`, so it must be placed after the modes_chosen validation block
(line ~2930). This means the Spree cost addition may need to happen in a second pass,
or the modes_chosen validation block needs to be moved before the Escalate block.

**Recommended approach**: Place the Spree cost block immediately after the
`validated_modes_chosen` computation (after line ~2930), as a new section. This is fine
because it's still before the pay_cost call. The existing mode validation + Spree cost
addition form a natural pair.

### Step 4: Resolution (No Changes Needed)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: No changes needed. The existing mode dispatch at line 271-309 already handles
`modes_chosen` correctly:

```rust
} else if !stack_obj.modes_chosen.is_empty() {
    // CR 700.2a: execute the explicitly chosen modes in index order.
    stack_obj.modes_chosen.iter()
        .filter_map(|&idx| modes.modes.get(idx).cloned())
        .collect()
}
```

Spree spells use `modes_chosen` just like any other modal spell. The modes execute in
ascending index order (printed order), which is correct per the ruling "No matter which
modes you choose, you always follow the instructions in the order they are written."

### Step 5: Harness Action (cast_spell_spree)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `cast_spell_spree` action type. This is syntactic sugar for
`cast_spell_modal` with the Spree keyword enforced. Alternatively, `cast_spell_modal`
already works for Spree -- the per-mode costs are added automatically in casting.rs
when `KeywordAbility::Spree` is detected. A separate action type is NOT strictly needed.

**Recommendation**: Do NOT add a new harness action. `cast_spell_modal` already works:
it passes `modes_chosen` and the casting pipeline handles the rest. This is simpler and
consistent with how modal spells work. The game-script-generator can use `cast_spell_modal`
for Spree spells, just like for any other modal spell.

### Step 6: Unit Tests

**File**: `crates/engine/tests/spree.rs`
**Tests to write**:

1. `test_spree_single_mode_adds_mode_cost` -- CR 702.172a / 700.2h
   Cast a Spree spell choosing mode 0 only. Total cost = base + mode[0] cost.
   Verify: spell resolves, mode[0] effect executes.

2. `test_spree_two_modes_adds_both_costs` -- CR 702.172a / 700.2h
   Cast choosing modes 0 and 1. Total cost = base + mode[0] cost + mode[1] cost.
   Verify: both effects execute in order.

3. `test_spree_all_three_modes` -- CR 702.172a
   Cast choosing all three modes. Total cost = base + all three mode costs.
   Verify: all three effects execute in printed order.

4. `test_spree_zero_modes_rejected` -- CR 702.172a
   Cast with `modes_chosen: []`. Must be rejected (Spree requires at least one mode).

5. `test_spree_insufficient_mana_rejected` -- CR 601.2f
   Cast choosing two modes but only provide enough mana for one mode's cost.
   Verify: rejected for insufficient mana.

6. `test_spree_duplicate_mode_rejected` -- CR 700.2d
   Cast choosing mode 0 twice. Must be rejected (no duplicate modes by default).

7. `test_spree_mode_order_ascending` -- Ruling "always follow the instructions in the
   order they are written"
   Choose modes [2, 0]. Verify effects execute in order 0, 2 (ascending index).

8. `test_spree_keyword_marker_present` -- CR 702.172a
   Verify that the test card has `KeywordAbility::Spree` in its keywords after enrichment.

9. `test_spree_mana_value_unchanged` -- CR 118.8d
   Verify that the spell's mana value on the stack equals its printed mana cost,
   regardless of which mode costs were paid.

**Test card definition** (synthetic, self-contained):
```
"Spree Test Spell"
Sorcery {1}{W}
Spree (Choose one or more additional costs.)
+ {1} -- Controller gains 4 life.
+ {2} -- Controller draws 2 cards.
+ {1}{W} -- Target creature gets -3/-3 until end of turn.
```

**Pattern**: Follow `escalate_test_spell_def()` in `tests/escalate.rs` for structure.

### Step 7: Card Definition (later phase)

**Suggested card**: Requisition Raid ({W}, Sorcery, Spree with 3 modes each costing {1})
- Simple, uniform per-mode costs ({1} each)
- Effects are expressible in the DSL: destroy artifact, destroy enchantment, put +1/+1
  counters on creatures target player controls

**Alternative**: Final Showdown ({W}, Instant, Spree) -- modes cost {1}, {1}, {3}{W}{W}
- Tests mixed costs per mode
- Effects: all creatures lose abilities, indestructible on target, destroy all creatures

### Step 8: Game Script (later phase)

**Suggested scenario**: Cast Requisition Raid choosing 2 of 3 modes. Verify both effects
resolve, correct mana was paid (base + both mode costs).
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Alt-cost casting** (e.g., Flashback + Spree): The base mana cost is replaced by the
  alt cost, but per-mode additional costs are still added. This is critical -- the ruling
  explicitly says "without paying its mana cost" still requires mode costs. The engine's
  cost pipeline handles this correctly because alt-cost modifies the base and per-mode costs
  are added afterward.
- **Copy effects** (CR 700.2g): Copies of Spree spells copy the chosen modes. The copier
  cannot choose different modes. The existing `modes_chosen` propagation on StackObject
  handles this.
- **Cost reduction effects**: Per-mode costs are additional costs (CR 601.2f-h). Cost
  reducers that apply to "total cost" or "additional costs" should affect them. Cost
  reducers that only affect "mana cost" should not (CR 118.8d).
- **Escalate + Spree**: These should be mutually exclusive on cards. A card shouldn't
  have both keywords. But if it did, both cost additions would stack. No special handling
  needed.
- **Entwine + Spree**: Entwine says "choose all modes." If a Spree spell had entwine
  (unlikely), all mode costs would need to be paid. The current entwine path skips
  modes_chosen and uses all modes. The Spree cost block should also handle this case:
  if `was_entwined` and `KeywordAbility::Spree`, add all mode costs. This is an edge
  case that likely never occurs on real cards but should be coded defensively.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Spree | 134 |
| AbilityDefinition | (none -- uses existing Spell + ModeSelection) | n/a |
| StackObjectKind | (none -- no new trigger) | n/a |
| Effect | (none) | n/a |
| GameEvent | (none) | n/a |

## Files Modified (Summary)

1. `crates/engine/src/state/types.rs` -- add `KeywordAbility::Spree` (disc 134)
2. `crates/engine/src/state/hash.rs` -- hash `KeywordAbility::Spree`; hash `ModeSelection.mode_costs`
3. `crates/engine/src/cards/card_definition.rs` -- add `mode_costs: Option<Vec<ManaCost>>` to `ModeSelection`
4. `crates/engine/src/rules/casting.rs` -- add Spree cost block after modes_chosen validation
5. `tools/replay-viewer/src/view_model.rs` -- add `KeywordAbility::Spree` arm in keyword display
6. `crates/engine/tests/spree.rs` -- 9 unit tests (new file)
