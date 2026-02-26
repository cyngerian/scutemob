# Ability Plan: Kicker

**Generated**: 2026-02-26
**CR**: 702.33
**Priority**: P2
**Similar abilities studied**: Convoke (`casting.rs:521-642`), Delve (`casting.rs:659-722`), Flashback (`casting.rs:150-166`, `stack.rs:46-51`), `Condition` enum and `check_condition` in `effects/mod.rs:1672-1720`

## CR Rule Text

### 702.33 — Kicker (full text with children)

**702.33a** Kicker is a static ability that functions while the spell with kicker is on the stack. "Kicker [cost]" means "You may pay an additional [cost] as you cast this spell." Paying a spell's kicker cost(s) follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

**702.33b** The phrase "Kicker [cost 1] and/or [cost 2]" means the same thing as "Kicker [cost 1], kicker [cost 2]."

**702.33c** Multikicker is a variant of the kicker ability. "Multikicker [cost]" means "You may pay an additional [cost] any number of times as you cast this spell." A multikicker cost is a kicker cost.

**702.33d** If a spell's controller declares the intention to pay any of that spell's kicker costs, that spell has been "kicked." If a spell has two kicker costs or has multikicker, it may be kicked multiple times. See rule 601.2b.

**702.33e** Objects with kicker or multikicker have additional abilities that specify what happens if they were kicked. These abilities are linked to the kicker or multikicker abilities printed on that object: they can refer only to those specific kicker or multikicker abilities. See rule 607, "Linked Abilities."

**702.33f** Objects with more than one kicker cost may also have abilities that each correspond to a specific kicker cost. Those abilities contain the phrases "if it was kicked with its [A] kicker" and "if it was kicked with its [B] kicker," where A and B are the first and second kicker costs listed on the card, respectively. Each of those abilities is linked to the appropriate kicker ability.

**702.33g** If part of a spell's ability has its effect only if that spell was kicked, and that part of the ability includes any targets, the spell's controller chooses those targets only if that spell was kicked. Otherwise, the spell is cast as if it did not have those targets. See rule 601.2c.

**702.33h** Sticker kicker is a keyword ability that represents a kicker ability and an ability that imposes an additional cost if the spell is kicked. (Not relevant for this engine.)

### 601.2b (additional cost announcement)

"If the spell has alternative or additional costs that will be paid as it's being cast such as buyback or kicker costs (see rules 118.8 and 118.9), the player announces their intentions to pay any or all of those costs (see rule 601.2f)."

### 118.8 (additional costs)

"Some spells and abilities have additional costs. An additional cost is a cost listed in a spell's rules text, or applied to a spell or ability from another effect, that its controller must pay at the same time they pay the spell's mana cost or the ability's activation cost."

**118.8a** "Any number of additional costs may be applied to a spell as it's being cast."
**118.8d** "Additional costs don't change a spell's mana cost, only what its controller has to pay to cast it."

### 607.2i (linked abilities for kicker)

"If an object has an ability printed on it that allows an additional cost to be paid and an ability printed on it that refers to whether that cost was paid, those abilities are linked."

## Key Edge Cases

From CR and card rulings:

1. **Kicker does NOT change mana value** (CR 118.8d, Goblin Bushwhacker ruling 2024-11-08). The spell's mana value remains unchanged regardless of whether kicker was paid. Total cost = mana cost + kicker cost + commander tax.

2. **Copies of kicked spells are also kicked** (Goblin Bushwhacker ruling 2024-11-08): "If you copy a kicked spell on the stack, the copy is also kicked." The `kicked` flag must propagate to copies via Storm/Cascade. IMPORTANT: "If a card or token enters as a copy of a permanent, the new permanent isn't kicked, even if the original was." This distinction matters for copy effects like Clone but NOT for spell copies on the stack.

3. **Kicker only works when CAST** (ruling 2024-11-08): "If you put a permanent with a kicker ability onto the battlefield without casting it, you can't kick it." Permanents entering via effects (e.g., Reanimate) are never kicked.

4. **Single kicker can only be paid once** (ruling 2024-11-08): "The kicker ability doesn't let you pay a kicker cost more than once." Multikicker (702.33c) IS paid multiple times, but standard kicker is at most once.

5. **Conditional kicker targets (CR 702.33g)**: If the kicker effect adds targets, those targets are only announced if the spell was kicked. Not kicked = spell is cast as if those targets don't exist. This is an advanced feature that can be deferred past the initial implementation.

6. **Kicker cost stacks with commander tax and other additional costs** (CR 118.8a / 601.2f). Total cost = mana cost + kicker cost + commander tax. Convoke/Delve reduce the total after all additional costs are added.

7. **Multiplayer**: No special multiplayer considerations beyond normal priority rules.

8. **Multikicker** (702.33c) uses a count rather than a boolean. For the initial implementation, model this as `kicker_times_paid: u32` on the StackObject (0 = not kicked, 1 = kicked once, N = multikicked N times). Standard kicker is capped at 1; multikicker is uncapped.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring (propagation of kicked status to resolution)
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

No partial work exists in the codebase. Grep for "Kicker" and "kicked" returns no hits in `crates/engine/src/`.

## Implementation Steps

### Step 1: Types and Data Model

This step adds the kicker cost type, the KeywordAbility variant, the AbilityDefinition variant, the Condition variant, and the StackObject field.

#### 1a. Add `AbilityDefinition::Kicker` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add a new `AbilityDefinition` variant after `Cycling`:

```rust
/// CR 702.33: Kicker [cost]. Optional additional cost that can be paid
/// when casting this spell. If paid, the spell is "kicked" and may have
/// enhanced effects.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Kicker)` for quick
/// presence-checking without scanning all abilities.
///
/// `is_multikicker` indicates multikicker (CR 702.33c) — the cost can
/// be paid any number of times instead of at most once.
Kicker {
    cost: ManaCost,
    #[serde(default)]
    is_multikicker: bool,
},
```

**Pattern**: Follows the `Flashback { cost: ManaCost }` / `Cycling { cost: ManaCost }` pattern at line 144/151.

#### 1b. Add `KeywordAbility::Kicker` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Kicker` variant to `KeywordAbility` enum (after `Delve` at line 243):

```rust
/// CR 702.33: Kicker [cost] — optional additional cost for enhanced effect.
/// This is a marker for quick presence-checking (`keywords.contains`).
/// The kicker cost itself is stored in `AbilityDefinition::Kicker { cost }`.
Kicker,
```

**Pattern**: Follows `Flashback` (line 219) and `Cycling` (line 226) marker variants.

#### 1c. Add `Condition::WasKicked` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add a new variant to the `Condition` enum (after `SourceHasCounters` at line 558):

```rust
/// "if this spell was kicked" (CR 702.33d).
/// Checked at resolution time by reading the `kicker_times_paid` field
/// on the StackObject or the `was_kicked` field on the permanent
/// (for ETB triggers on kicked permanents).
WasKicked,
```

**CR**: 702.33d — "If a spell's controller declares the intention to pay any of that spell's kicker costs, that spell has been 'kicked.'"

#### 1d. Add `kicker_times_paid` field to `StackObject`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add field to `StackObject` struct (after `cast_with_flashback` at line 51):

```rust
/// CR 702.33d: Number of times the kicker cost was paid when this spell
/// was cast. 0 = not kicked. 1 = kicked (standard kicker). N = multikicked
/// N times (CR 702.33c). The value is set at cast time and propagated to
/// copies (CR 702.33d ruling: copies of kicked spells are also kicked).
#[serde(default)]
pub kicker_times_paid: u32,
```

**Pattern**: Follows `cast_with_flashback: bool` at line 51. Uses `#[serde(default)]` for backward compatibility with existing scripts.

#### 1e. Add `was_kicked` field to `GameObject`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add a field to `GameObject` (in the `status` sub-struct or directly on `GameObject`, depending on where `cast_with_flashback`-like data is tracked). Since `cast_with_flashback` is on `StackObject` and does NOT survive to the permanent, we need `was_kicked` on the permanent for ETB triggers (e.g., Goblin Bushwhacker's "When this creature enters, if it was kicked").

Find `GameObject` struct and add:

```rust
/// CR 702.33d: If this permanent was kicked when cast, this records how
/// many times kicker was paid. Used by ETB triggers that check "if it was
/// kicked" (CR 702.33e linked abilities). Set during resolution when the
/// spell moves from stack to battlefield.
///
/// 0 = not kicked (or entered without being cast). Never set for non-cast
/// permanents (CR ruling: "If you put a permanent onto the battlefield
/// without casting it, you can't kick it.").
#[serde(default)]
pub kicker_times_paid: u32,
```

**Pattern**: This is a new concept -- the stack object's `kicker_times_paid` is transferred to the permanent at resolution time, similar to how `cast_with_flashback` affects zone destination but in this case we need the info to survive to the battlefield.

#### 1f. Update `state/hash.rs`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `kicker_times_paid` to the `HashInto` impl for `StackObject` (after `cast_with_flashback` at line 1081):

```rust
self.kicker_times_paid.hash_into(hasher);
```

Also add `kicker_times_paid` to `HashInto` for `GameObject` (find the `impl HashInto for GameObject` block and add the new field).

Also add `Condition::WasKicked` to the `HashInto` impl for `Condition`. Find the existing match arms for `Condition` and add:

```rust
Condition::WasKicked => {
    6u8.hash_into(hasher);
}
```

Also add `AbilityDefinition::Kicker` to the `HashInto` impl for `AbilityDefinition`:

```rust
AbilityDefinition::Kicker { cost, is_multikicker } => {
    7u8.hash_into(hasher); // or next available discriminant
    cost.hash_into(hasher);
    is_multikicker.hash_into(hasher);
}
```

Also add `KeywordAbility::Kicker` to the `HashInto` impl for `KeywordAbility`. Find the match block and add a new discriminant byte.

**Gotcha**: Per `memory/gotchas-infra.md`, every new field on `GameState`, `PlayerState`, `GameObject`, or `StackObject` must be manually added to `hash.rs`.

#### 1g. Add `kicked` field to `Command::CastSpell`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `kicked: bool` field to `CastSpell` (after `delve_cards` at line 80):

```rust
/// CR 702.33: Whether the player intends to pay the kicker cost.
/// For multikicker, use `kicker_times` instead (this field is for
/// standard kicker: true = pay once, false = don't pay).
/// Ignored for spells without kicker.
///
/// For multikicker (CR 702.33c), this field is ignored; use
/// `kicker_times` to specify how many times to pay.
#[serde(default)]
kicked: bool,
/// CR 702.33c: Number of times to pay the multikicker cost. 0 = not
/// kicked. Only used for multikicker spells; ignored for standard kicker.
/// For standard kicker spells, use `kicked: true` instead.
#[serde(default)]
kicker_times: u32,
```

**Alternative (simpler)**: Use a single `kicker_times: u32` field. 0 = not kicked, 1 = kicked once (standard kicker), N = multikicked N times. This avoids having two fields. The `kicked: bool` is just sugar that maps to `kicker_times: 1`. For simplicity, use only `kicker_times: u32`.

**DECISION**: Use a single field `kicker_times: u32` on `Command::CastSpell`. 0 = not kicked, 1 = kicked, N = multikicked N times. This maps directly to `StackObject::kicker_times_paid`.

```rust
/// CR 702.33d: Number of times to pay the kicker cost. 0 = not kicked.
/// 1 = kicked once (standard kicker). N > 1 = multikicker paid N times
/// (CR 702.33c). Validated against the spell's kicker definition:
/// standard kicker rejects values > 1; multikicker accepts any N.
/// Ignored for spells without kicker.
#[serde(default)]
kicker_times: u32,
```

### Step 2: Rule Enforcement (casting.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add kicker cost validation and payment between the flashback/commander-tax cost calculation (line ~166) and the convoke reduction (line ~261).

#### 2a. Update `handle_cast_spell` signature

Add `kicker_times: u32` parameter after `delve_cards`:

```rust
pub fn handle_cast_spell(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
    convoke_creatures: Vec<ObjectId>,
    delve_cards: Vec<ObjectId>,
    kicker_times: u32,  // NEW
) -> Result<Vec<GameEvent>, GameStateError> {
```

#### 2b. Validate kicker eligibility

After the `mana_cost` is determined (line ~166, after flashback/commander-tax), and BEFORE convoke/delve reduction:

```rust
// CR 702.33a / 601.2b: If the player declared intention to pay kicker,
// validate the spell has kicker and add the kicker cost to the total.
let kicker_times_paid = if kicker_times > 0 {
    // Look up the kicker cost from the card definition.
    let kicker_info = get_kicker_cost(&card_id, &state.card_registry);
    match kicker_info {
        Some((kicker_cost, is_multikicker)) => {
            // CR 702.33d: Standard kicker can only be paid once.
            if !is_multikicker && kicker_times > 1 {
                return Err(GameStateError::InvalidCommand(
                    "standard kicker can only be paid once (CR 702.33d)".into(),
                ));
            }
            // CR 118.8d: Additional costs don't change mana cost, but
            // the player must pay mana cost + kicker cost(s).
            // Add kicker cost * times to the total.
            // We accumulate onto the existing mana_cost Option.
            kicker_times
        }
        None => {
            return Err(GameStateError::InvalidCommand(
                "spell does not have kicker".into(),
            ));
        }
    }
} else {
    0
};
```

Then add the kicker mana to the total cost:

```rust
// CR 601.2f: Add kicker cost to total mana cost.
let mana_cost = if kicker_times_paid > 0 {
    let (kicker_cost, _) = get_kicker_cost(&card_id, &state.card_registry)
        .expect("kicker already validated above");
    let mut total = mana_cost.unwrap_or_default();
    for _ in 0..kicker_times_paid {
        total.white += kicker_cost.white;
        total.blue += kicker_cost.blue;
        total.black += kicker_cost.black;
        total.red += kicker_cost.red;
        total.green += kicker_cost.green;
        total.generic += kicker_cost.generic;
        total.colorless += kicker_cost.colorless;
    }
    Some(total)
} else {
    mana_cost
};
```

**CR**: 601.2b + 601.2f — kicker cost announced at cast time, paid as part of total cost. 118.8d — additional cost does not change printed mana cost.

**Position**: AFTER flashback/commander-tax (line ~166), BEFORE convoke reduction (line ~261). This matches the cost stacking order: base -> flashback/tax -> kicker -> convoke/delve reduction.

#### 2c. Add `get_kicker_cost` helper

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add helper function (after `get_flashback_cost` at line ~502):

```rust
/// CR 702.33a: Look up the kicker cost from the card's `AbilityDefinition`.
///
/// Returns `Some((ManaCost, is_multikicker))` if the card has a kicker/multikicker
/// ability, or `None` if it has no kicker.
fn get_kicker_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<(ManaCost, bool)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Kicker { cost, is_multikicker } = a {
                    Some((cost.clone(), *is_multikicker))
                } else {
                    None
                }
            })
        })
    })
}
```

**Pattern**: Follows `get_flashback_cost` at line 487-502.

#### 2d. Set `kicker_times_paid` on the StackObject

In the `StackObject` construction (line ~341-352), add:

```rust
let stack_obj = StackObject {
    id: stack_entry_id,
    controller: player,
    kind: StackObjectKind::Spell {
        source_object: new_card_id,
    },
    targets: spell_targets,
    cant_be_countered,
    is_copy: false,
    cast_with_flashback: casting_with_flashback,
    kicker_times_paid,  // NEW
};
```

#### 2e. Update engine dispatch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add `kicker_times` to the `CastSpell` match arm (line ~70-87):

```rust
Command::CastSpell {
    player,
    card,
    targets,
    convoke_creatures,
    delve_cards,
    kicker_times,  // NEW
} => {
    // ...
    let mut events = casting::handle_cast_spell(
        &mut state,
        player,
        card,
        targets,
        convoke_creatures,
        delve_cards,
        kicker_times,  // NEW
    )?;
```

### Step 3: Propagation of Kicked Status to Resolution

The kicked status needs to be available at two resolution points:

1. **Spell effect resolution** (instants/sorceries): `Condition::WasKicked` checks `StackObject.kicker_times_paid` via `EffectContext`.
2. **Permanent ETB triggers** (creatures/artifacts/enchantments): `Condition::WasKicked` checks `GameObject.kicker_times_paid` on the permanent after it enters the battlefield.

#### 3a. Add `kicker_times_paid` to `EffectContext`

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add field to `EffectContext` struct (after `target_remaps` at line 59):

```rust
/// CR 702.33d: Number of times kicker was paid for this spell.
/// 0 = not kicked. Used by `Condition::WasKicked`.
pub kicker_times_paid: u32,
```

Update `EffectContext::new()` to accept and store this:

```rust
pub fn new(
    controller: PlayerId,
    source: ObjectId,
    targets: Vec<SpellTarget>,
    kicker_times_paid: u32,  // NEW
) -> Self {
    Self {
        controller,
        source,
        targets,
        target_remaps: HashMap::new(),
        kicker_times_paid,  // NEW
    }
}
```

**Warning**: This changes the signature of `EffectContext::new()`. All call sites must be updated. Search for `EffectContext::new(` in the codebase:

- `resolution.rs` line 150: spell resolution -- pass `stack_obj.kicker_times_paid`
- `resolution.rs` line 327: activated ability resolution -- pass `0` (no kicker)
- `resolution.rs` line 382: triggered ability resolution -- pass `0` (see 3b below)
- `replacement.rs` line 879: ETB triggered effects -- pass from `GameObject.kicker_times_paid` (see 3c)
- Any other callers

#### 3b. Implement `Condition::WasKicked` in `check_condition`

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add match arm in `check_condition` (after `SourceHasCounters` at line 1718):

```rust
Condition::WasKicked => ctx.kicker_times_paid > 0,
```

**CR**: 702.33d -- "If a spell's controller declares the intention to pay any of that spell's kicker costs, that spell has been 'kicked.'"

#### 3c. Transfer `kicker_times_paid` to `GameObject` at resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: In the permanent resolution branch (line ~169-291), after moving the card to the battlefield, set `kicker_times_paid` on the new permanent:

```rust
// CR 702.33d: Transfer kicked status from stack to permanent for ETB triggers.
if let Some(obj) = state.objects.get_mut(&new_id) {
    obj.kicker_times_paid = stack_obj.kicker_times_paid;
}
```

This goes right after the existing `obj.controller = controller;` assignment at line ~179.

Also, in `fire_when_enters_triggered_effects` in `replacement.rs`, pass the permanent's `kicker_times_paid` when creating the `EffectContext`:

```rust
let kicker = state.objects.get(&new_id)
    .map(|o| o.kicker_times_paid)
    .unwrap_or(0);
let mut ctx = EffectContext::new(controller, new_id, vec![], kicker);
```

#### 3d. Copy propagation

When Storm or Cascade creates copies, the copy inherits `kicker_times_paid` from the original. Check `rules/copy.rs` -- find where `StackObject` copies are created and ensure `kicker_times_paid` is propagated.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/copy.rs`
**Action**: Grep for `StackObject {` in copy.rs and ensure each copy construction includes `kicker_times_paid` from the original stack object.

### Step 4: Script Harness Updates

#### 4a. Add `kicked` field to `ScriptAction::PlayerAction`

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/script_schema.rs`
**Action**: Add field to `PlayerAction` variant (after `delve` at line 227):

```rust
/// CR 702.33: Whether the spell is cast with kicker.
/// For standard kicker, true means "pay kicker once."
/// For multikicker, the value indicates how many times.
/// Defaults to false / 0 for non-kicked casts.
#[serde(default)]
kicked: bool,
/// CR 702.33c: For multikicker, number of times to pay.
/// Only used with action "cast_spell" on multikicker spells.
/// Defaults to 0. If `kicked` is true and `kicker_times` is 0,
/// treated as 1 (standard kicker).
#[serde(default)]
kicker_times: u32,
```

**Simpler alternative**: Just `kicked: bool` for the first implementation. `kicker_times` can be added later for multikicker.

**DECISION**: Add `kicked: bool` only. For standard kicker this suffices. Multikicker support (the `kicker_times: u32` field) is a natural extension but not needed for the first card (Burst Lightning). Map `kicked: true` to `kicker_times: 1` in `translate_player_action`.

#### 4b. Update `translate_player_action`

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `kicked` parameter extraction and pass it through to `Command::CastSpell`:

```rust
// In the function signature, add: kicked: bool
// In the match arm for "cast_spell":
Some(Command::CastSpell {
    player,
    card: card_id,
    targets: target_list,
    convoke_creatures: convoke_ids,
    delve_cards: delve_ids,
    kicker_times: if kicked { 1 } else { 0 },  // NEW
})
```

Also update the `cast_spell_flashback` arm to pass `kicker_times: 0`.

#### 4c. Update `script_replay.rs` caller

**File**: `/home/airbaggie/scutemob/crates/engine/tests/script_replay.rs`
**Action**: Extract `kicked` from the `ScriptAction::PlayerAction` fields and pass to `translate_player_action`.

### Step 5: Replay Viewer Update

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `is_kicked: bool` (or `kicker_times: u32`) field to `StackObjectView` (line ~120), and populate it from `so.kicker_times_paid > 0` in the view model builder (line ~400).

This is a LOW priority update -- the viewer still works without it, it just won't show kicked status.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/kicker.rs` (new file)

**Tests to write**:

1. **`test_kicker_basic_cast_with_kicker`** -- CR 702.33a
   - Card: Burst Lightning (Kicker {4}, deals 2 damage; if kicked, 4 damage instead)
   - Setup: Player has {5}{R} mana, cast Burst Lightning with `kicker_times: 1`
   - Assert: Mana pool deducted by {R} + {4} = {4}{R} total. SpellCast event emitted. After resolution, target takes 4 damage.

2. **`test_kicker_basic_cast_without_kicker`** -- CR 702.33a (negative)
   - Same setup, but `kicker_times: 0`
   - Assert: Mana pool deducted by {R} only. After resolution, target takes 2 damage (not 4).

3. **`test_kicker_insufficient_mana_with_kicker`** -- CR 601.2f
   - Player has only {R} (enough for base cost but not kicker {4})
   - Cast Burst Lightning with `kicker_times: 1`
   - Assert: `InsufficientMana` error.

4. **`test_kicker_non_kicker_spell_rejected`** -- validation
   - Cast a non-kicker spell (e.g., Lightning Bolt) with `kicker_times: 1`
   - Assert: Error "spell does not have kicker".

5. **`test_kicker_standard_kicker_rejects_multiple`** -- CR 702.33d
   - Cast Burst Lightning with `kicker_times: 2`
   - Assert: Error "standard kicker can only be paid once".

6. **`test_kicker_permanent_etb_kicked`** -- CR 702.33e / 702.33d
   - Card: Goblin Bushwhacker or Torch Slinger (creature with kicker ETB trigger)
   - Cast with kicker, resolve
   - Assert: Permanent enters battlefield with `kicker_times_paid: 1`, ETB trigger fires

7. **`test_kicker_permanent_etb_not_kicked`** -- negative
   - Cast same creature WITHOUT kicker
   - Assert: ETB trigger does NOT fire (Condition::WasKicked is false)

8. **`test_kicker_does_not_change_mana_value`** -- CR 118.8d
   - Cast Burst Lightning ({R}, kicker {4}) with kicker
   - Assert: The spell on the stack still has mana_value == 1 (not 5)

9. **`test_kicker_with_commander_tax`** -- CR 118.8a
   - Cast a commander with kicker from the command zone with 1 tax
   - Assert: Total cost = base + kicker + {2} tax

10. **`test_kicker_condition_evaluates_correctly`** -- Condition::WasKicked
    - Directly test `check_condition(state, &Condition::WasKicked, &ctx)` with `kicker_times_paid: 0` and `kicker_times_paid: 1`

**Pattern**: Follow `crates/engine/tests/delve.rs` and `crates/engine/tests/convoke.rs` for test structure. Both use `GameStateBuilder::four_player()`, `ObjectSpec::card()` + `enrich_spec_from_def()`, cast spells, and check events/state.

### Step 7: Card Definition

**Suggested card**: Burst Lightning

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
**Card lookup result**:

```
Burst Lightning {R}
Instant
Kicker {4}
Burst Lightning deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.
```

**CardDefinition sketch**:

```rust
CardDefinition {
    id: "burst-lightning".into(),
    name: "Burst Lightning".into(),
    mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
    color_identity: vec![Color::Red].into_iter().collect(),
    card_types: vec![CardType::Instant].into_iter().collect(),
    oracle_text: "Kicker {4}\nBurst Lightning deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.".into(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Kicker),
        AbilityDefinition::Kicker {
            cost: ManaCost { generic: 4, ..Default::default() },
            is_multikicker: false,
        },
        AbilityDefinition::Spell {
            effect: Effect::Conditional {
                condition: Condition::WasKicked,
                if_true: Box::new(Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(4),
                }),
                if_false: Box::new(Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                }),
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        },
    ],
    ..Default::default()
}
```

The key insight: the spell effect uses `Effect::Conditional` with `Condition::WasKicked` to branch between 2 damage and 4 damage. This is the fundamental pattern for all kicker cards -- the `WasKicked` condition drives the Conditional branch in the effect DSL.

**Second card (for ETB testing)**: Torch Slinger

```
Torch Slinger {2}{R}
Creature -- Goblin Shaman
2/2
Kicker {1}{R}
When this creature enters, if it was kicked, it deals 2 damage to target creature.
```

This card tests kicked status propagation through permanent ETB triggers. The WhenEntersBattlefield trigger uses `intervening_if: Some(Condition::WasKicked)`.

### Step 8: Game Script

**Suggested scenario**: Burst Lightning kicked vs unkicked

**Subsystem directory**: `test-data/generated-scripts/stack/`
**Filename**: `072_burst_lightning_kicker.json` (next available number in stack/)

**Scenario**:
1. Player 1 has Burst Lightning in hand with 5R in pool
2. Player 1 casts Burst Lightning targeting Player 2 with `kicked: true`
3. Priority passes; spell resolves
4. Assert: Player 2 life = 36 (40 - 4 damage)
5. (Second scenario) Player 1 casts another Burst Lightning without kicker
6. Assert: Player 2 takes only 2 damage (life = 34)

## Interactions to Watch

1. **Kicker + Commander Tax**: Both are additional costs. Total = base + kicker + tax. Kicker cost is added BEFORE convoke/delve reduction but AFTER commander tax. This matches the cost stacking order already in `casting.rs`.

2. **Kicker + Flashback**: A spell cast with flashback CAN also be kicked (CR 118.9d: additional costs apply on top of alternative costs). Total = flashback_cost + kicker_cost. The implementation naturally handles this since kicker cost is added to `mana_cost` (which is already set to flashback cost).

3. **Kicker + Copy (Storm/Cascade)**: Copies of kicked spells are also kicked (ruling). `StackObject.kicker_times_paid` must propagate when copies are created in `copy.rs`.

4. **Kicker + Convoke/Delve**: Kicker increases the total cost; convoke/delve reduce it. A kicked spell with convoke could have its combined mana cost reduced by tapping creatures. This works naturally since kicker cost is added before convoke/delve reduction in the cost pipeline.

5. **Permanent kicker + Clone**: "If a card or token enters as a copy of a permanent, the new permanent isn't kicked." Copies that enter via Clone/Rite of Replication do NOT inherit `was_kicked`. Only spell copies on the stack inherit kicked status. This distinction will matter when copy permanents are implemented -- ensure `kicker_times_paid` is NOT copied in the Layer 1 copy effect. For now, this is a future concern.

6. **Multiplayer**: No special considerations. Kicker is paid by the caster and affects only the spell's effect. All opponents are affected equally by the spell's resolution.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Kicker` variant |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Kicker`, `Condition::WasKicked` |
| `crates/engine/src/state/stack.rs` | Add `kicker_times_paid: u32` to `StackObject` |
| `crates/engine/src/state/game_object.rs` | Add `kicker_times_paid: u32` to `GameObject` |
| `crates/engine/src/state/hash.rs` | Hash new fields/variants |
| `crates/engine/src/rules/command.rs` | Add `kicker_times: u32` to `Command::CastSpell` |
| `crates/engine/src/rules/casting.rs` | Validate + pay kicker cost, add `get_kicker_cost` helper |
| `crates/engine/src/rules/engine.rs` | Pass `kicker_times` through dispatch |
| `crates/engine/src/rules/resolution.rs` | Transfer `kicker_times_paid` to permanent, pass to `EffectContext` |
| `crates/engine/src/rules/copy.rs` | Propagate `kicker_times_paid` on spell copies |
| `crates/engine/src/effects/mod.rs` | Add `kicker_times_paid` to `EffectContext`, implement `WasKicked` condition |
| `crates/engine/src/rules/replacement.rs` | Pass `kicker_times_paid` through `fire_when_enters_triggered_effects` |
| `crates/engine/src/testing/script_schema.rs` | Add `kicked: bool` to `PlayerAction` |
| `crates/engine/src/testing/replay_harness.rs` | Extract `kicked`, pass `kicker_times` to `CastSpell` |
| `crates/engine/tests/script_replay.rs` | Pass `kicked` through to `translate_player_action` |
| `crates/engine/tests/kicker.rs` | New test file (10 tests) |
| `crates/engine/src/cards/definitions.rs` | Add Burst Lightning + Torch Slinger card definitions |
| `tools/replay-viewer/src/view_model.rs` | Add `is_kicked` to `StackObjectView` (LOW priority) |
| `test-data/generated-scripts/stack/072_burst_lightning_kicker.json` | Game script |
