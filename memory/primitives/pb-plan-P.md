# Primitive Batch Plan: PB-P — EffectAmount::PowerOfSacrificedCreature (LKI)

**Generated**: 2026-04-19
**Primitive**: New `EffectAmount::PowerOfSacrificedCreature` variant — an LKI-based read of a sacrificed creature's power, captured at the moment of sacrifice (CR 608.2b) and consumed at effect resolution.
**CR Rules**: 701.16 (Sacrifice), 608.2b (LKI), 400.7 (object identity across zone change), 117.1f / 601.2g (cost-payment ordering), 118.8 (spell additional costs), 602.2 (activated-ability cost payment)
**Cards affected**: **3 confirmed** (Altar of Dementia, Greater Good, Life's Legacy) out of 13 PB-P queue candidates. **Yield 23%** — below the 40-65% filter-PB calibration band; acknowledged narrow per `memory/primitive-wip.md`.
**Dependencies**: none reach PB-P scope. BASELINE-LKI-01 is **adjacent** (same root cause: `calculate_characteristics` against post-zone graveyard objects drops battlefield-gated layer effects), but PB-P side-steps it by capturing power at sacrifice time rather than reading post-sacrifice.
**Deferred items from prior PBs**: none.

---

## Plan Summary (1-paragraph)

PB-P adds a dedicated `EffectAmount::PowerOfSacrificedCreature` variant that reads
a per-resolution-context vector of integers captured AT sacrifice time (capture-by-
value, NOT capture-by-ID) and returns the first entry. **Confirmed yield: 3 cards**
(Altar of Dementia + Greater Good newly authored from `abilities: vec![]`
placeholders; Life's Legacy precision-fixed from `Fixed(1)` placeholder).
**Dispatch verdict: PASS-AS-NEW-EFFECT-AMOUNT-VARIANT** — adding an
`EffectTarget::SacrificedCreature` variant to reuse `EffectAmount::PowerOf` was
considered and rejected because the existing `PowerOf` resolution path requires
the target to live in `state.objects` (which the OLD ObjectId does not
post-`move_object_to_zone`) AND because the layer-resolved characteristics for the
NEW graveyard ObjectId have lost battlefield-gated boosts (BASELINE-LKI-01). A
dedicated EffectAmount variant that reads from a captured integer vector is the
only structurally sound route. **LKI capture decision: CAPTURE-BY-VALUE** — power
is snapshot via `calculate_characteristics(state, sac_id).power` immediately
before `move_object_to_zone` at every sacrifice cost-payment site, then stored as
`Vec<i32>` in two places: (a) `AdditionalCost::Sacrifice` is extended with a
parallel `sacrificed_powers: Vec<i32>` field for spell additional costs, OR a new
`AdditionalCost::SacrificeWithLKI { ids: Vec<ObjectId>, powers: Vec<i32> }`
variant is introduced (planner recommends extending the existing variant — see
Change 1 for rationale); (b) `StackObject` gains a `sacrificed_creature_powers:
Vec<i32>` field for activated abilities, populated at activation time from the
activated-ability sacrifice cost-payment site, and propagated to
`EffectContext.sacrificed_creature_powers: Vec<i32>` at resolution. **Mandatory
test count: 8** (M1-M8). **Deferred cards: 10** (10 candidates in `memory/primitive-wip.md`
re-triage table; one-line reasons each in `## Deferred Cards`). **Hash bump:
5 → 6** (PB-P adds an EffectAmount variant + a StackObject field + an
AdditionalCost field; standard bump per `memory/conventions.md`). **Step 0
stale-TODO sweep result: 5 hits, all accounted-for** — 3 CONFIRMED
(altar/greater/lifes), 2 already in DEFERRED roster (ziatora/ruthless). **No
forced-add misses.** **Warstorm Surge stale-TODO**: confirmed already implemented
(card def at lines 19-42 fully authors the trigger using
`EffectAmount::PowerOf(EffectTarget::TriggeringCreature)`); recommend opportunistic
cleanup (out of PB-P scope unless the runner has spare cycles in fix-phase).

---

## Primitive Specification

A class of cards reads "the sacrificed creature's power" as the magnitude for an
effect. The sacrifice happens as a cost (CR 117.1f, 601.2g), then the effect
resolves later. Three forms exist:

1. **Spell with mandatory additional sacrifice cost** (CR 118.8): Life's Legacy.
   Sacrifice happens during cast (`casting.rs:3820`) before the spell goes on the
   stack. At resolution, the controller draws cards equal to the sacrificed
   creature's LKI power.
2. **Activated ability with sacrifice cost** (CR 602.2): Altar of Dementia,
   Greater Good. Sacrifice happens during activation (`abilities.rs:716`) before
   the ability goes on the stack. At resolution, the effect consumes the LKI
   power.
3. **(Out of scope)** "May sacrifice as part of effect" (Disciple of Freyalise):
   sacrifice happens DURING effect execution, not as a cost. Different mechanism.

CR 608.2b (LKI) requires the read to use the value immediately before the
sacrifice resolved — i.e. the layer-resolved on-battlefield power, not the
graveyard object's base power. This is structurally critical: an anthem (+1/+1)
that pumps the creature on the battlefield must contribute to the read, even
though the anthem's continuous effect no longer applies to the graveyard object
post-sacrifice.

CR 400.7 (object identity) means the OLD ObjectId is dead after
`move_object_to_zone` (`state/mod.rs:409`). The NEW graveyard ObjectId has the
OLD `obj.characteristics` cloned forward (BASE, no layer effects), and
`calculate_characteristics` against it drops battlefield-gated layer effects
(BASELINE-LKI-01). Therefore neither (a) lookup-by-old-ID nor (b)
characteristics-on-new-id are sound. Only a captured integer (snapshot at
sacrifice time, before the zone change) is correct.

### Dispatch unification verdict: PASS-AS-NEW-EFFECT-AMOUNT-VARIANT

**Single new EffectAmount variant**: `EffectAmount::PowerOfSacrificedCreature`.
No new EffectTarget variant. No new EffectContext field reuse — a dedicated
`sacrificed_creature_powers: Vec<i32>` field is added to EffectContext and
populated at resolution time from cost-carrier state (StackObject for activated
abilities, AdditionalCost for spell additional costs).

**Alternative rejected: PASS-AS-NEW-EFFECT-TARGET-VARIANT**
(`EffectTarget::SacrificedCreature` reusing `EffectAmount::PowerOf`).

Reasoning for rejection:

1. The existing `PowerOf` resolution path (`effects/mod.rs:5805-5828`) calls
   `resolve_effect_target_list` which expects to find a live ObjectId in
   `state.objects` or in `ctx.targets` (neither holds for the dead old ID).
2. Even if `EffectTarget::SacrificedCreature` returned the NEW graveyard ObjectId,
   the `PowerOf` body's else-branch reads `obj.characteristics.power` (the
   non-LKI base from `move_object_to_zone`'s `clone()`), giving the wrong answer
   under any continuous boost.
3. Even if `PowerOf` were patched to call `calculate_characteristics`,
   BASELINE-LKI-01 documents that this re-runs filters against the graveyard
   object and silently drops battlefield-gated effects.
4. To fix any of (1)-(3), the runner would need to introduce a captured-integer
   side channel anyway, at which point an explicit `EffectAmount` variant with a
   dedicated context field is cleaner and more explicit than overloading
   `PowerOf` with hidden side-effects.

**Alternative rejected: SPLIT-REQUIRED.** No split needed: one EffectAmount
variant + one EffectContext field + capture sites at the existing 2 sacrifice
cost-payment sites covers all 3 confirmed cards.

---

## CR Rule Text

### CR 701.16 — Sacrifice

> 701.16a To sacrifice a permanent, its controller moves it from the battlefield
> directly to its owner's graveyard. A player can't sacrifice something that
> isn't a permanent, or something that's not under their control. Sacrificing a
> permanent doesn't cause any abilities of that permanent to trigger as it goes
> to the graveyard, unless those abilities specifically refer to the sacrifice.

> 701.16b Effects may also instruct a player to sacrifice a permanent matching
> certain criteria. The player chooses which permanent to sacrifice when the
> effect resolves. If no permanents match the criteria, the player doesn't
> sacrifice anything.

**Engine implication**: Sacrifice as a cost (CR 117.1f) happens before the
spell/ability resolves; sacrifice as an effect happens during resolution. PB-P
covers ONLY the cost-time sacrifice form (Altar of Dementia, Greater Good, Life's
Legacy all use cost-time sacrifice).

### CR 608.2b — Last Known Information (LKI)

> 608.2b If an effect requires information from the game (such as the number of
> creatures on the battlefield), the answer is determined only once, when the
> effect is applied. If the effect requires information from a specific object,
> including the source of the ability itself, the effect uses the current
> information of that object if it's in the public zone it was expected to be in;
> if it's no longer in that zone, or if the effect has moved it from the zone it
> was in to another public zone, the effect uses the object's last known
> information. If an ability states that an object does something, it's the
> object as it exists — or as it most recently existed — that does it, even if
> the ability is removed from the object.

**Engine implication**: For "draw cards equal to the sacrificed creature's
power", the power read MUST be the value immediately before the sacrifice
resolved (the on-battlefield, layer-resolved value). The capture point is BEFORE
`move_object_to_zone` at the sacrifice cost-payment site.

### CR 400.7 — Object identity across zone change

> 400.7 An object that moves from one zone to another becomes a new object with
> no memory of, or relation to, its previous existence. There are seven exceptions
> to this rule: [...] (none of which apply to ordinary sacrifice).

**Engine implication**: The OLD ObjectId stored in
`AdditionalCost::Sacrifice(ids)` is **dead** after `move_object_to_zone`. Any
attempt to look it up in `state.objects` post-sacrifice returns `None`. The NEW
graveyard ObjectId is a distinct object. Capture-by-ID is therefore unsound; only
capture-by-value works.

### CR 117.1f / CR 601.2g — Cost-payment ordering

> 117.1f Players can pay costs in any order they choose. The total cost is
> determined first; then the costs are paid in the order chosen by the player.
> Once a player begins to pay a cost, they must complete the payment.

> 601.2g The player pays the total cost. Partial payments are not allowed.

**Engine implication**: For spell additional costs (CR 118.8) and activated
ability costs (CR 602.2), all costs are paid before the spell/ability hits the
stack. The sacrifice happens BEFORE the StackObject is constructed, giving us a
clean capture window inside the cost-payment block.

### CR 118.8 — Additional costs

> 118.8 Some cards and abilities specify that a player may pay an additional cost
> when they cast a spell, activate an ability, or take some other action. Other
> cards and abilities require that an additional cost be paid to cast a spell or
> activate an ability. [...] If an additional cost includes an action denoted by
> the word "may," that additional cost is optional. Otherwise, the additional
> cost is mandatory.

**Engine implication**: Life's Legacy uses `SpellAdditionalCost::SacrificeCreature`
(mandatory). The sacrifice ID is in `additional_costs: Vec<AdditionalCost>` on
both CastSpell and StackObject (preserved through cast-to-stack transfer at
`casting.rs:4195`). PB-P extends `AdditionalCost::Sacrifice` (or adds a sibling
variant) to also carry the captured power.

---

## Engine Changes

### Change 1: Extend `AdditionalCost::Sacrifice` to carry captured powers

**File**: `crates/engine/src/state/types.rs`
**Action**: Change `AdditionalCost::Sacrifice(Vec<ObjectId>)` to
`AdditionalCost::Sacrifice { ids: Vec<ObjectId>, lki_powers: Vec<i32> }`
(parallel vectors, same length and ordering — ids[i] is the OLD ObjectId,
lki_powers[i] is the captured LKI power at sacrifice time, both referring to
the same sacrificed creature).

**Rationale for extending vs. introducing a new variant**: `AdditionalCost::Sacrifice`
is consumed at ~10 sites (casting.rs ×3 for emerge/bargain/casualty/devour
extraction; resolution.rs ×2 for devour; state/hash.rs ×1; state/types.rs ×1
construction; replay_harness.rs as needed). Adding a sibling variant
(`SacrificeWithLKI { ids, powers }`) would force every reader to handle both
variants; extending the existing variant with a parallel vector lets non-LKI
readers ignore the new field. The cost is one breaking-shape change to the
existing 10 sites; the benefit is all future LKI consumers get the field for
free without further variant-explosion.

**Backward-compat note**: For callers that don't need LKI (emerge, bargain,
casualty, devour), pass `lki_powers: vec![]` or `lki_powers: vec![0; ids.len()]`.
Document the convention in the variant doc-comment: `lki_powers` MAY be empty
for non-LKI consumers; LKI consumers must verify `lki_powers.len() == ids.len()`
before reading.

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditionalCost {
    /// Sacrifice one or more permanents. The `ids` are the (now-dead) ObjectIds
    /// at the time of sacrifice. `lki_powers` is parallel to `ids` and stores
    /// the layer-resolved power of each sacrificed creature captured BEFORE
    /// `move_object_to_zone` (CR 608.2b LKI). Non-LKI consumers (emerge,
    /// bargain, casualty, devour) may set `lki_powers: vec![]` and ignore it.
    /// LKI consumers (PB-P: PowerOfSacrificedCreature) MUST verify
    /// `lki_powers.len() == ids.len()` before reading.
    Sacrifice { ids: Vec<ObjectId>, lki_powers: Vec<i32> },
    // ... other variants unchanged
}
```

**Pattern to follow**: The existing `Sacrifice(Vec<ObjectId>)` tuple variant.
Convert all readers — see Change 7 table for the exhaustive list.

**STOP-AND-FLAG**: If during implementation the runner discovers that converting
the existing tuple variant to a struct variant breaks a serialized JSON test
fixture (the JSON serde representation changes from `[ids]` to `{ "ids": [],
"lki_powers": [] }`), the runner MUST stop and consult oversight. Acceptable
fallback: introduce a sibling variant `SacrificeWithLKI { ids, lki_powers }` for
PB-P consumers only and leave the existing tuple variant untouched. The struct-
variant route is preferred for cleanliness; the sibling-variant route is the
fallback if serde compatibility is at risk.

### Change 2: Add `sacrificed_creature_powers: Vec<i32>` to `StackObject`

**File**: `crates/engine/src/state/stack.rs`
**Line**: After `triggering_creature_id` (around line 449)
**Action**: Add a new field `sacrificed_creature_powers: Vec<i32>` defaulting to
`vec![]`. Populated at activated-ability cost-payment time
(`abilities.rs:716`-ish) for activated abilities with `Cost::Sacrifice(filter)`.
Read at resolution time and copied into `EffectContext.sacrificed_creature_powers`
(see Change 5).

```rust
/// CR 608.2b: Captured LKI powers of creatures sacrificed as a cost when this
/// activated ability or spell was put on the stack. Parallel to the sacrificed
/// IDs in `additional_costs.Sacrifice` (for spells) or populated independently
/// (for activated abilities, which have no AdditionalCost mirror).
/// Read by `EffectAmount::PowerOfSacrificedCreature` at resolution.
/// Empty for stack objects whose costs did not include sacrifice (most cases).
#[serde(default)]
pub sacrificed_creature_powers: Vec<i32>,
```

Update `StackObject::trigger_default` (around line 522) to set
`sacrificed_creature_powers: vec![]`.

**Rationale for a separate field instead of relying on `additional_costs`**:
Activated abilities currently DO NOT populate `StackObject.additional_costs`
(that field is spell-cast-specific — see `casting.rs:4195` vs.
`abilities.rs:918-927` which uses `StackObject::trigger_default` and never
touches `additional_costs`). Plumbing activated-ability sacrifice IDs through
`additional_costs` would require widening the activated-ability path to
populate it everywhere, which is out of scope. A dedicated field on StackObject
is local, additive, and zero-impact on existing code paths.

### Change 3: Add `sacrificed_creature_powers: Vec<i32>` to `EffectContext`

**File**: `crates/engine/src/effects/mod.rs`
**Line**: After `damaged_player` (around line 116) in the `EffectContext` struct
**Action**: Add the field; default to `vec![]` in both `EffectContext::new` and
`EffectContext::new_with_kicker`.

```rust
/// CR 608.2b: LKI powers of creatures sacrificed as a cost for this spell or
/// ability, captured at the cost-payment site BEFORE `move_object_to_zone`.
/// Read by `EffectAmount::PowerOfSacrificedCreature` (PB-P).
/// Empty for spells/abilities whose costs did not include sacrifice.
pub sacrificed_creature_powers: Vec<i32>,
```

Update both `EffectContext::new` (line 133) and `EffectContext::new_with_kicker`
(line 159) to initialize the field to `vec![]`. Update every other ad-hoc
`EffectContext { ... }` literal in the codebase (Change 7 table enumerates these).

### Change 4: Add `EffectAmount::PowerOfSacrificedCreature` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Line**: 2241 (after `DomainCount`, the last variant in the enum)
**Action**: Add the new variant with a doc-comment citing CR 608.2b and CR 701.16.

```rust
/// CR 608.2b LKI: The layer-resolved power of the (first) creature sacrificed as
/// a cost for this spell or activated ability. Captured at sacrifice time and
/// stored on EffectContext.sacrificed_creature_powers. Returns 0 if no creature
/// was sacrificed (defensive default — the card author should not pair this
/// amount with a non-sacrifice cost).
///
/// Used by:
/// - Altar of Dementia: "Sacrifice a creature: Target player mills cards equal
///   to the sacrificed creature's power."
/// - Greater Good: "Sacrifice a creature: Draw cards equal to the sacrificed
///   creature's power, then discard three cards."
/// - Life's Legacy: "As an additional cost to cast this spell, sacrifice a
///   creature. Draw cards equal to the sacrificed creature's power."
///
/// Implementation note: the read uses the LKI value (on-battlefield, layer-
/// resolved), NOT the graveyard object's base characteristics. This is
/// structurally important — see CR 608.2b. Any creature pumped by an anthem at
/// the moment of sacrifice contributes the pumped value, not the printed value.
PowerOfSacrificedCreature,
```

### Change 5: Resolve `PowerOfSacrificedCreature` in `resolve_amount`

**File**: `crates/engine/src/effects/mod.rs`
**Line**: After the `DomainCount` arm in `resolve_amount` (around line 6000+;
runner finds the exact location after compiler errors)
**Action**: Add the resolution arm:

```rust
EffectAmount::PowerOfSacrificedCreature => {
    // CR 608.2b: Read the LKI power of the first sacrificed creature.
    // Defensive: if the cost-payment path didn't capture a power (mismatched
    // card definition), return 0.
    ctx.sacrificed_creature_powers
        .first()
        .copied()
        .unwrap_or(0) as i32
}
```

Note: `as i32` because `sacrificed_creature_powers` stores `i32` directly (not
`u32` — power can be 0 or negative under -1/-1 counters or P/T-set effects, e.g.
Humility, although in practice we sacrifice from battlefield so power should be
≥0 in the common case; using i32 matches `EffectAmount::PowerOf`'s signedness
convention).

### Change 6: Capture LKI power at the spell-additional-cost sacrifice site

**File**: `crates/engine/src/rules/casting.rs`
**Line**: 3820-3853 (the `if let Some(sac_id) = spell_sac_id` block)
**Action**: BEFORE the `state.move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))`
call (line 3833), capture the layer-resolved power:

```rust
// CR 608.2b: Capture LKI power of the sacrificed creature BEFORE the zone move.
// After move_object_to_zone, the OLD sac_id is dead and the NEW graveyard
// object's characteristics have lost battlefield-gated layer effects.
let sac_lki_power: i32 = {
    let chars = crate::rules::layers::calculate_characteristics(state, sac_id)
        .or_else(|| state.objects.get(&sac_id).map(|o| o.characteristics.clone()))
        .unwrap_or_default();
    chars.power.unwrap_or(0)
};
```

Then update the `additional_costs` Vec being passed downstream so its
`Sacrifice { ids, lki_powers }` entry has `lki_powers` populated. The `additional_costs`
parameter is owned by `handle_cast_spell` and threads through to the StackObject
construction at line 4195 — find the construction site of the
`AdditionalCost::Sacrifice` for `spell_sac_id` (it appears earlier in the
function from the `additional_costs` argument as parsed) and patch the `lki_powers`
field there.

**Implementation hint**: Because `additional_costs` is a `Vec<AdditionalCost>`
passed by value into the function and consumed downstream, the cleanest patch is
to mutate it in place after the LKI capture:

```rust
// After capturing sac_lki_power, patch the additional_costs entry in place:
for ac in additional_costs.iter_mut() {
    if let AdditionalCost::Sacrifice { ids, lki_powers } = ac {
        if ids.contains(&sac_id) {
            // Ensure lki_powers is the same length as ids; populate the slot
            // for sac_id with the captured LKI power.
            if lki_powers.len() != ids.len() {
                lki_powers.resize(ids.len(), 0);
            }
            let pos = ids.iter().position(|id| *id == sac_id).unwrap();
            lki_powers[pos] = sac_lki_power;
        }
    }
}
```

**Rationale**: `additional_costs` is constructed by the caller (script harness or
test) with `lki_powers: vec![]` initially (or some default); the engine fills in
the captured value at the cost-payment site. This keeps the harness code simple
(callers don't need to predict LKI) and centralizes capture inside the engine.

**Then in resolution.rs**: `EffectContext.sacrificed_creature_powers` is
populated by reading `stack_obj.additional_costs.iter().find_map(...)` for the
`Sacrifice { lki_powers, .. }` variant — see Change 8.

### Change 7: Capture LKI power at the activated-ability sacrifice site

**File**: `crates/engine/src/rules/abilities.rs`
**Line**: 716 (the `state.move_object_to_zone(sac_id, ZoneId::Graveyard(owner))?` call inside the `if let Some(ref filter) = ability_cost.sacrifice_filter` block)
**Action**: BEFORE the `move_object_to_zone` call, capture the layer-resolved
power and stash it in a local variable:

```rust
// CR 608.2b: Capture LKI power of the sacrificed creature BEFORE the zone move.
let sac_lki_power: i32 = {
    let chars = crate::rules::layers::calculate_characteristics(state, sac_id)
        .or_else(|| state.objects.get(&sac_id).map(|o| o.characteristics.clone()))
        .unwrap_or_default();
    chars.power.unwrap_or(0)
};
let mut sacrificed_lki_powers: Vec<i32> = vec![sac_lki_power];
```

Then at the StackObject construction site (around line 918-927), populate the
new field:

```rust
let mut stack_obj = StackObject::trigger_default(
    stack_id,
    player,
    StackObjectKind::ActivatedAbility {
        source_object: source,
        ability_index,
        embedded_effect: embedded_effect.map(Box::new),
    },
);
stack_obj.targets = spell_targets;
stack_obj.x_value = x_value.unwrap_or(0);
// PB-P: Carry captured LKI powers of cost-sacrificed creatures forward to
// resolution, where EffectAmount::PowerOfSacrificedCreature reads them.
stack_obj.sacrificed_creature_powers = sacrificed_lki_powers;
state.stack_objects.push_back(stack_obj);
```

**Variable scoping**: `sacrificed_lki_powers` must be in scope at the StackObject
construction site. Declare it as `let mut sacrificed_lki_powers: Vec<i32> = vec![];`
near the top of the function (around line 295, alongside `embedded_effect`),
push to it inside the sacrifice block, and read it at the StackObject
construction site. If `sacrifice_filter` is `None`, `sacrificed_lki_powers`
remains empty and the resolution path's `unwrap_or(0)` returns 0.

### Change 8: Propagate sacrificed_creature_powers from StackObject to EffectContext at resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Line 1**: 1768-1800 (the `StackObjectKind::ActivatedAbility` resolution block)
**Action**: After constructing `EffectContext::new(...)` at line 1785, copy the
captured powers from the stack object:

```rust
let mut ctx = EffectContext::new(
    stack_obj.controller,
    source_object,
    stack_obj.targets.clone(),
);
ctx.x_value = stack_obj.x_value;
// PB-P: Propagate captured LKI powers of cost-sacrificed creatures so
// EffectAmount::PowerOfSacrificedCreature resolves correctly.
ctx.sacrificed_creature_powers = stack_obj.sacrificed_creature_powers.clone();
let effect_events = execute_effect(state, &effect, &mut ctx);
```

**Line 2**: The spell-resolution dispatch site (around line 250-400, where
`Spell` resolution happens — runner finds via grep for `EffectContext::new` in
resolution.rs). Apply the same propagation pattern, but read from
`stack_obj.additional_costs` (since spells use `AdditionalCost::Sacrifice` to
carry the IDs and LKI powers, not the new StackObject field):

```rust
// PB-P: Extract LKI powers from additional_costs for spells with mandatory
// SpellAdditionalCost::SacrificeCreature (Life's Legacy).
let lki_powers_from_ac: Vec<i32> = stack_obj
    .additional_costs
    .iter()
    .find_map(|c| match c {
        AdditionalCost::Sacrifice { lki_powers, .. } => Some(lki_powers.clone()),
        _ => None,
    })
    .unwrap_or_default();
ctx.sacrificed_creature_powers = lki_powers_from_ac;
```

This dual path (StackObject field for activated abilities, additional_costs for
spells) reflects the actual cost-payment plumbing in the engine. Both end up in
the same `ctx` field for unified resolution.

### Change 9: Hash arms for the new variant + new fields

**File**: `crates/engine/src/state/hash.rs`

**9a. EffectAmount::PowerOfSacrificedCreature** (after the `DomainCount` arm at
line 4386, discriminant 15):

```rust
// CR 608.2b: PowerOfSacrificedCreature — LKI power of cost-sacrificed creature — discriminant 15
EffectAmount::PowerOfSacrificedCreature => 15u8.hash_into(hasher),
```

**9b. AdditionalCost::Sacrifice shape change** (find the existing `AdditionalCost`
hash impl via grep; update the `Sacrifice` arm to hash both `ids` and
`lki_powers` Vecs):

```rust
AdditionalCost::Sacrifice { ids, lki_powers } => {
    <discriminant>u8.hash_into(hasher);
    ids.hash_into(hasher);
    lki_powers.hash_into(hasher);
}
```

**9c. StackObject.sacrificed_creature_powers** — append to the existing
`StackObject` hash impl after `triggering_creature_id`:

```rust
self.sacrificed_creature_powers.hash_into(hasher);
```

### Change 10: Bump hash sentinel 5 → 6

**File**: `crates/engine/src/state/hash.rs`
**Line**: 32 (the `pub const HASH_SCHEMA_VERSION: u8`)
**Action**: Bump 5 → 6 and extend the history comment:

```rust
/// - 6: PB-P (2026-04-19) — EffectAmount::PowerOfSacrificedCreature added (disc 15);
///   AdditionalCost::Sacrifice changed to struct variant { ids, lki_powers };
///   StackObject.sacrificed_creature_powers field added; EffectContext gains
///   sacrificed_creature_powers (not hashed — runtime resolution scratch only).
pub const HASH_SCHEMA_VERSION: u8 = 6;
```

**Rationale**: Three serialized-shape changes (EffectAmount variant addition,
AdditionalCost shape change, StackObject field addition). Default action per
`memory/conventions.md` "Hash bump rule" is to bump on every serialized-shape
change. Test `assert_eq!(HASH_SCHEMA_VERSION, 6u8)` is required in any new
hash-parity test (M7).

### Change 11: Update all `AdditionalCost::Sacrifice(...)` construction sites

The variant changes from tuple `Sacrifice(Vec<ObjectId>)` to struct
`Sacrifice { ids: Vec<ObjectId>, lki_powers: Vec<i32> }`. Every construction
site must be updated. Runner uses `cargo build` errors to find sites; the
expected non-exhaustive list:

| File | Approx Line | Construction context | Update |
|------|-------------|---------------------|--------|
| `state/types.rs` | enum def | Variant definition | Convert tuple to struct |
| `state/hash.rs` | AdditionalCost hash impl | Hash arm | Hash both fields (Change 9b) |
| `rules/casting.rs` | ~174 | `sacrifice_from_additional_costs` | Pattern `Sacrifice { ids, .. }` instead of `Sacrifice(ids)` |
| `rules/casting.rs` | ~3956 | `devour_sacrifices` | Same pattern fix |
| `rules/casting.rs` | ~136 | `AdditionalCost::Sacrifice(_) => {}` | `Sacrifice { .. } => {}` |
| `rules/resolution.rs` | ~1101 | `devour_sacrifice_ids` | `Sacrifice { ids, .. }` |
| `cards/card_definition.rs` | (script harness path) | If any | Pattern fix |
| `testing/replay_harness.rs` | grep `AdditionalCost::Sacrifice` | Constructor | `Sacrifice { ids: vec![...], lki_powers: vec![] }` |
| `tools/replay-viewer/src/view_model.rs` | Possible match site | Add arm | Wildcard or struct pattern |
| `tools/tui/src/play/panels/stack_view.rs` | Possible match site | Add arm | Same |

**Worker procedure**: After Change 1 lands, run `cargo build --workspace`,
collect every "expected struct fields, found tuple" error, fix in order. Treat
any surprise consumer as a stop-and-flag (the runner should investigate the
site's semantics before silently filling in `lki_powers: vec![]`).

### Change 12: Exhaustive match sites for the new EffectAmount variant

Adding `EffectAmount::PowerOfSacrificedCreature` will force a non-exhaustive
match error at every exhaustive `match` on `EffectAmount`. Known sites:

| File | Approx Line | Match context | Action |
|------|-------------|--------------|--------|
| `state/hash.rs` | 4321-4387 | HashInto for EffectAmount | Add arm (Change 9a) |
| `effects/mod.rs` | resolve_amount | Resolution dispatch | Add arm (Change 5) |
| `effects/mod.rs` | resolve_cda_amount | CDA dispatch | Add arm (likely defensive `=> 0`; runner verifies) |
| `cards/card_definition.rs` | possible serde derive | (handled by derive) | No change |
| `testing/replay_harness.rs` | possible match | If any | Add arm |
| `tools/replay-viewer/src/view_model.rs` | possible match | If any | Add arm (wildcard ok if displayed as "PowerOfSacrificedCreature") |
| `tools/tui/src/play/panels/stack_view.rs` | possible match | If any | Add arm |

**Worker procedure**: Same as Change 11 — drive from compiler errors.

---

## Card Definition Fixes

Three cards confirmed shippable by PB-P. All verified by Read of current source.

### altar_of_dementia.rs (newly authored from placeholder)

**Oracle text**: "Sacrifice a creature: Target player mills cards equal to the
sacrificed creature's power."

**Current state**: `abilities: vec![]` placeholder. TODO comment at lines 3-5
explicitly names `EffectAmount::PowerOfSacrificedCreature` as the gap.

**Fix**: Replace the empty `abilities` Vec with a single
`AbilityDefinition::Activated`:

```rust
abilities: vec![
    // CR 602.2 + CR 701.16 + CR 608.2b: Sacrifice a creature, target player mills
    // cards equal to the sacrificed creature's LKI power.
    AbilityDefinition::Activated {
        cost: Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }),
        effect: Effect::Mill {
            player: PlayerTarget::DeclaredTarget { index: 0 },
            count: EffectAmount::PowerOfSacrificedCreature,
        },
        timing_restriction: None,
        targets: vec![TargetRequirement::TargetPlayer],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
    },
],
```

Strip the TODO comment block at lines 3-5. Add a CR citation comment.

**Verify before shipping**: `Effect::Mill` exists. The runner Reads
`card_definition.rs` and `effects/mod.rs` to confirm signature
(`Effect::Mill { player: PlayerTarget, count: EffectAmount }`). If the actual
field name differs (e.g., `MillCards`, `LibraryToGraveyard`), use the canonical
name — but this is a planner-time concern for completeness, not a stop-and-flag.

### greater_good.rs (newly authored from placeholder)

**Oracle text**: "Sacrifice a creature: Draw cards equal to the sacrificed
creature's power, then discard three cards."

**Current state**: `abilities: vec![]` placeholder. TODO at lines 3-6 claims
`Effect::DiscardCards` is missing — **STALE** (verified: `Effect::DiscardCards`
exists in `card_definition.rs:1197` and is implemented in `effects/mod.rs:518`).

**Fix**: Replace the empty `abilities` Vec with a single
`AbilityDefinition::Activated` whose effect is a `Sequence`:

```rust
abilities: vec![
    // CR 602.2 + CR 701.16 + CR 608.2b: Sacrifice a creature, draw cards equal to
    // its LKI power, then discard three cards.
    AbilityDefinition::Activated {
        cost: Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        }),
        effect: Effect::Sequence(vec![
            Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::PowerOfSacrificedCreature,
            },
            Effect::DiscardCards {
                player: PlayerTarget::Controller,
                count: 3,
            },
        ]),
        timing_restriction: None,
        targets: vec![],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
    },
],
```

Strip the TODO comment block at lines 3-6. Add a CR citation comment.

**Verify before shipping**:
1. `Effect::DiscardCards` field name and shape — runner confirms via Read of
   `effects/mod.rs:518`. The wip says `count: u32`; verify whether it accepts
   `EffectAmount` or just `u32`. The plan uses `count: 3` literal; if the field
   is `EffectAmount`, switch to `EffectAmount::Fixed(3)`.
2. The "then discard" semantics: discard happens AFTER draw, so any of the just-
   drawn cards may be selected for discard. CR 701.7 (discard) requires the
   active player to choose. The deterministic fallback used elsewhere in the
   engine (lowest-ObjectId-first) is acceptable.
3. **Stop-and-flag** if `Effect::DiscardCards` requires a target requirement
   that's not in the current `targets: vec![]` (e.g., if it requires a target
   for "you may discard a card chosen by you"). Greater Good is *not* targeted —
   the discard is a non-target choice — so this should be fine, but verify.

### lifes_legacy.rs (precision fix from `Fixed(1)` placeholder)

**Oracle text**: "As an additional cost to cast this spell, sacrifice a
creature. Draw cards equal to the sacrificed creature's power."

**Current state**: Already plumbed `SpellAdditionalCost::SacrificeCreature` at
line 20. Has a `Spell` ability at lines 25-34 with `Effect::DrawCards { count:
EffectAmount::Fixed(1) }` as a placeholder. TODO at lines 5-9 and 21-23
explicitly names `EffectAmount::SacrificedCreaturePower` as the gap.

**Fix**: Replace `EffectAmount::Fixed(1)` with
`EffectAmount::PowerOfSacrificedCreature`:

```rust
abilities: vec![
    AbilityDefinition::Spell {
        effect: Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::PowerOfSacrificedCreature,
        },
        targets: vec![],
        modes: None,
        cant_be_countered: false,
    },
],
```

Strip the TODO comment block at lines 5-9 and 21-23. Add a CR 608.2b citation
to the surviving comment block above the ability.

**Backward-compat verification**: After the fix, the existing spell-additional-
cost plumbing in `casting.rs` (lines 3157-3223) feeds `additional_costs.Sacrifice`
forward to the StackObject (line 4195), which is read at resolution time
(Change 8 spell path). Life's Legacy will resolve with `ctx.sacrificed_creature_powers`
populated from the captured LKI power, and the draw count will be the captured
value.

---

## New Card Definitions (none)

PB-P does not add any new card files. All three confirmed cards already exist as
files; PB-P fixes their `abilities` vectors.

---

## Deferred Cards (10)

These cards appeared in the original PB-P queue or as TODO-sweep candidates but
are NOT shipped by PB-P. One-line reason each (full table in
`memory/primitive-wip.md` lines 30-45).

| Card | File | Reason for deferral |
|------|------|--------------------|
| Conclave Mentor | `conclave_mentor.rs` | Different mechanism: `WhenThisDies` trigger + `EffectAmount::PowerOf(Source)`. Not sacrifice-based. |
| Jagged Scar Archers | `jagged_scar_archers.rs` | Different gap: `PowerOf(Source)` (already exists — TODO is stale) + flying-creature filter. Out of PB-P scope. |
| Krenko, Tin Street Kingpin | `krenko_tin_street_kingpin.rs` | Different mechanism: token count = source's power on attack, not sacrificed. |
| Master Biomancer | `master_biomancer.rs` | Different DSL gap: dynamic ETB replacement counter count. |
| The Great Henge | `the_great_henge.rs` | Different DSL gap: `SelfCostReduction::GreatestPower`. |
| Warstorm Surge | `warstorm_surge.rs` | **Already implemented** via `EffectAmount::PowerOf(EffectTarget::TriggeringCreature)`. Stale TODO; recommend opportunistic cleanup. |
| Ziatora, the Incinerator | `ziatora_the_incinerator.rs` | Compound blocker: optional sacrifice as effect (NOT cost) + reflexive trigger + power-based damage. Different mechanic. |
| Ruthless Technomancer | `ruthless_technomancer.rs` | Compound blocker: optional ETB sacrifice + variable `Cost::Sacrifice(X artifacts)`. Different mechanic. |
| Miren, the Moaning Well | `miren_the_moaning_well.rs` | Different stat: `ToughnessOfSacrificedCreature` — needs a parallel PB. |
| Diamond Valley | `diamond_valley.rs` | Same as Miren — `ToughnessOfSacrificedCreature`. |
| Birthing Pod | `birthing_pod.rs` | Different DSL gap: dynamic search filter "MV = sacrificed creature's MV + 1". |

**Adjacent future PB candidates surfaced by this analysis**:

- **PB-P-followup-toughness**: `EffectAmount::ToughnessOfSacrificedCreature`
  for Miren, Diamond Valley. Same plumbing as PB-P (capture toughness instead
  of power; or add `sacrificed_creature_toughnesses: Vec<i32>` parallel field).
  2-3 cards confirmed yield, very narrow but easy to ship if user wants the
  parallel.
- **PB-P-followup-effect-sac**: A primitive for "sacrifice as part of effect
  resolution" (Disciple of Freyalise, Ziatora, Ruthless Technomancer). Different
  mechanic — sacrifice happens during effect, captured via
  `Effect::SacrificeCreature` -> internal LKI capture -> read in subsequent
  effect via this same `EffectAmount::PowerOfSacrificedCreature`. Reuses PB-P's
  EffectAmount variant + capture mechanism, just adds a new effect-side
  sacrifice path.

---

## Pre-existing TODO Sweep (Step 0, MANDATORY)

Grep run against `crates/engine/src/cards/defs/` for the following patterns:
- `EffectAmount::PowerOfSacrificedCreature`
- `SacrificedCreaturePower`
- `power of the sacrificed`
- `sacrificed creature's power`
- `sacrificed.*power`

**Result: 5 cards with matching TODO comments.** All accounted for:

| Card | TODO location | Disposition |
|------|--------------|-------------|
| `altar_of_dementia.rs` | lines 3-5 | **Forced add — confirmed** (newly authored) |
| `greater_good.rs` | lines 3-6 | **Forced add — confirmed** (newly authored) |
| `lifes_legacy.rs` | lines 5-9, 21-23 | **Forced add — confirmed** (precision fix) |
| `ziatora_the_incinerator.rs` | (oracle-text mention) | Forced add but **deferred** (compound: in-effect sacrifice + reflexive trigger) |
| `ruthless_technomancer.rs` | (oracle-text mention) | Forced add but **deferred** (compound: variable Cost::Sacrifice + ETB) |

**Cross-verified**: 0 forced-add misses between the original wip pre-triage table
(13 candidates) and the TODO sweep. The narrow PB-P scope correctly captures all
3 cards that explicitly self-identify as needing the primitive.

**Warstorm Surge stale-TODO recorded**: confirmed implemented at
`warstorm_surge.rs:25-41` using `EffectAmount::PowerOf(EffectTarget::TriggeringCreature)`.
The comment at lines 4-9 claims the path is "approximated as Nothing" and
"untested" — both false. **Recommendation**: opportunistic cleanup of the
warstorm_surge stale TODO during the PB-P fix-phase if the runner has spare
cycles. Not a blocker for PB-P shipping.

**TODO sweep result recorded positively**: 5 hits, 3 confirmed yield, 2 deferred,
0 misses. Gate satisfied per planner-agent definition Step 3a.

---

## Unit Tests

**File**: `crates/engine/tests/pbp_power_of_sacrificed_creature.rs` (new)

All tests numbered **MANDATORY** (M#) or **OPTIONAL** (O#). No silent skips.
Every test cites CR 608.2b. The hash-parity test cites the conventions doc.

### MANDATORY tests (8)

**M1** — `test_altar_of_dementia_mills_by_sacrificed_power`

Setup: 4-player game. Player 1 controls Altar of Dementia and a 5/5 Goblin.
Player 1 activates Altar of Dementia, sacrificing the 5/5 Goblin, targeting
Player 2 (who has 30+ cards in library).

Assert:
- The Goblin is in Player 1's graveyard (sacrificed).
- Player 2 has milled exactly 5 cards (5/5 power = 5 mill).
- The cards milled are the top 5 of Player 2's library (deterministic).

**Discriminator**: Pre-PB-P engine fails at compile (Altar of Dementia has
`abilities: vec![]`). Post-PB-P, the activated ability fires, the LKI power is
captured at sacrifice, and the mill count uses the captured value (5), not the
graveyard object's base power, not 0, not 1.

CR: 608.2b, 701.16, 602.2.

**M2** — `test_greater_good_draws_by_sacrificed_power_then_discards_three`

Setup: 4-player game. Player 1 controls Greater Good and a 4/4 Hippo. Player 1
has 4 cards in hand and 10+ cards in library. Player 1 activates Greater Good,
sacrificing the 4/4 Hippo.

Assert:
- The Hippo is in Player 1's graveyard.
- Player 1 drew exactly 4 cards (hand size: 4 + 4 = 8 momentarily).
- Player 1 then discarded exactly 3 cards (final hand size: 8 - 3 = 5).
- The discarded cards are removed (deterministic — lowest ObjectId first per
  the engine's discard fallback).

**Discriminator**: Pre-PB-P engine fails at compile (Greater Good has
`abilities: vec![]`). Post-PB-P, the Sequence runs both effects in order; the
draw uses the captured LKI power (4), and the discard executes second.

CR: 608.2b, 701.16, 602.2, 701.7 (discard).

**M3** — `test_lifes_legacy_draws_by_sacrificed_power_on_resolve`

Setup: 4-player game. Player 1 controls a 6/6 Beast and has Life's Legacy in
hand. Player 1 has 1 untapped Forest and 1 untapped Plains (provides {G} + 1
generic for the {1}{G} cost). Player 1 has 0 cards in hand (other than Life's
Legacy) and 10+ in library. Player 1 casts Life's Legacy, sacrificing the 6/6
Beast as additional cost. Spell resolves.

Assert:
- The Beast is in Player 1's graveyard (sacrificed as additional cost).
- Life's Legacy is in Player 1's graveyard (resolved).
- Player 1 drew exactly 6 cards (hand size: 0 + 6 = 6).

**Discriminator**: Pre-PB-P, Life's Legacy resolves with `Fixed(1)` placeholder
and Player 1 draws 1 card. Post-PB-P, the captured LKI power flows from
`additional_costs.Sacrifice.lki_powers` into `ctx.sacrificed_creature_powers`,
and the draw count is 6.

CR: 608.2b, 118.8, 117.1f.

**M4** — `test_lki_correctness_anthem_boosted_creature_sacrifice`

Setup: 4-player game. Player 1 controls Altar of Dementia and a 2/2 Bear (base
P/T). Player 1 also controls Glorious Anthem (creatures get +1/+1). The Bear is
layer-resolved as 3/3 on the battlefield. Player 1 activates Altar of Dementia,
sacrificing the Bear, targeting Player 2.

Assert:
- The Bear is in Player 1's graveyard.
- The graveyard Bear's base `obj.characteristics.power` is 2 (printed value).
- `calculate_characteristics(state, graveyard_bear_id).power` returns 2 (the
  Anthem doesn't apply to graveyard objects — BASELINE-LKI-01).
- Despite this, **Player 2 mills exactly 3 cards** (the LKI value at sacrifice
  time, when the Bear was on the battlefield boosted by the Anthem).

**Discriminator**: This is the CR 608.2b correctness test. Pre-PB-P, the engine
cannot express this card. Post-PB-P (capture-by-value at sacrifice site), the
mill count is 3 — proving the capture happens BEFORE the zone change, while the
Anthem is still applying.

If PB-P were implemented as capture-by-ID (looking up the new graveyard ID and
calling `calculate_characteristics`), the mill count would be 2 (BASELINE-LKI-01
silently drops the Anthem boost). M4 specifically discriminates capture-by-value
from capture-by-ID. **This test is the load-bearing correctness anchor for PB-P.**

CR: 608.2b, 613.1.

**M5** — `test_zero_power_creature_sacrifice_mills_zero`

Setup: 4-player game. Player 1 controls Altar of Dementia and a 0/4 creature
(Wall of Mulch or similar). Player 1 activates Altar of Dementia, sacrificing
the 0/4, targeting Player 2.

Assert:
- The Wall is in Player 1's graveyard.
- Player 2 mills exactly 0 cards (Mill 0 is a no-op; no library reduction).
- Player 2's library size is unchanged from pre-activation.

**Discriminator**: Edge case — sacrificing a creature with 0 power. The
captured LKI power is 0, the resolution path returns 0, and `Effect::Mill { count:
0 }` is a no-op. Validates the defensive `unwrap_or(0)` at the resolution arm
(Change 5) and the engine's `Effect::Mill` handles count=0 cleanly.

CR: 608.2b, 701.16.

**M6** — `test_sacrifice_no_capture_returns_zero_defensive`

Setup: synthetic test. Construct an `EffectContext` with
`sacrificed_creature_powers: vec![]`. Call `resolve_amount(state,
&EffectAmount::PowerOfSacrificedCreature, &ctx)` directly.

Assert: the function returns 0 (the defensive default). No panic, no
out-of-bounds.

**Discriminator**: Defensive coverage — confirms that a card author using
`EffectAmount::PowerOfSacrificedCreature` without a sacrifice cost gets a
deterministic 0, not a panic. This shapes the documentation contract for the
new variant.

CR: N/A (defensive infra test).

**M7** — `test_hash_parity_power_of_sacrificed_creature_distinct`

Setup: build five `EffectAmount` values:
1. `EffectAmount::Fixed(0)`
2. `EffectAmount::PowerOf(EffectTarget::Source)`
3. `EffectAmount::ToughnessOf(EffectTarget::Source)`
4. `EffectAmount::PowerOfSacrificedCreature`
5. `EffectAmount::CombatDamageDealt`

Hash each via the engine's HashInto trait.

Assert:
- All 5 hashes are distinct.
- `assert_eq!(HASH_SCHEMA_VERSION, 6u8)` (the bumped sentinel).

**Discriminator**: Forces the sentinel assertion to fail if Change 10 is not
made. Discriminates the new EffectAmount variant from the existing four.

CR: N/A (hash infrastructure).

**M8** — `test_backward_compat_existing_powerof_cards_still_work`

Setup: regression sweep. Build three test scenarios reusing existing
`EffectAmount::PowerOf(EffectTarget::X)` cards:
1. **Swords to Plowshares**: cast on a 5/5 creature; assert controller gains 5
   life (existing PowerOf path with Source target — actually
   `EffectTarget::DeclaredTarget { index: 0 }` here, not Source, but verifies
   the PowerOf dispatch path is unaffected by PB-P).
2. **Souls' Majesty** (or Eomer if simpler): triggered ability that draws cards
   equal to a creature's power. Assert the existing PowerOf path returns the
   correct value.
3. **Warstorm Surge** (if practical): ETB trigger using
   `EffectAmount::PowerOf(EffectTarget::TriggeringCreature)`. Assert the
   damage amount equals the entering creature's power.

Assert: all 3 scenarios produce the same game-state outcomes as pre-PB-P.

**Discriminator**: Backward-compat sentinel. PB-P should not alter any existing
`PowerOf` resolution path. If M8 fails, the runner has accidentally regressed
the existing dispatch logic during the AdditionalCost shape change or
EffectContext field addition.

CR: N/A (regression).

### OPTIONAL tests (3)

**O1** — `test_sacrifice_negative_power_creature_under_curses`

Setup: a creature whose layer-resolved power is negative (e.g., a 0/3 hit by
"target creature gets -2/-0 until end of turn"). Sacrifice it via Altar of
Dementia.

Assert: Player 2 mills 0 cards (negative mill rounds to 0; check the engine's
`Effect::Mill` semantics for negative count handling).

**Discriminator**: Edge case for the i32-vs-u32 question on `count`. If the
engine's `Effect::Mill` requires `count: u32` and the resolution arm returns
i32, the conversion semantics matter. Most likely outcome: negative power milled
as 0 (saturating to 0 at the count boundary).

CR: 608.2b, 107.1b (negative quantities).

**O2** — `test_multi_sacrifice_carries_only_first_power`

Setup: a hypothetical card that sacrifices TWO creatures as cost (e.g., Devour
or a custom test card). Activate it; verify that
`EffectAmount::PowerOfSacrificedCreature` returns ONLY the first sacrificed
creature's power, not a sum.

Assert: `ctx.sacrificed_creature_powers.first().copied().unwrap_or(0)` is the
power of the first-sacrificed creature.

**Discriminator**: Confirms the documented semantics — `PowerOfSacrificedCreature`
returns the FIRST sacrificed creature's power only. If a future card needs
"sum of sacrificed powers", a new variant (`SumPowerOfSacrificedCreatures`) would
be added; PB-P does not anticipate that need.

CR: N/A (semantics).

**O3** — `test_lifes_legacy_with_zero_power_creature_draws_zero`

Setup: cast Life's Legacy sacrificing a 0/4 creature. Verify Player 1 draws 0
cards.

**Discriminator**: End-to-end edge case mirroring M5 but for the spell path.
Validates the additional_costs LKI propagation works for 0-power.

CR: 608.2b.

**Test pattern**: Follow `crates/engine/tests/pbd_damaged_player_filter.rs` for
overall structure (helper builders + 4-player setup + LKI assertions). Reuse any
helper imports.

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check --workspace`)
- [ ] `EffectAmount::PowerOfSacrificedCreature` enum variant added with CR doc
  comment
- [ ] `AdditionalCost::Sacrifice` shape converted from tuple to struct with
  `lki_powers: Vec<i32>` field (or sibling variant `SacrificeWithLKI` if struct
  conversion breaks fixtures)
- [ ] `StackObject.sacrificed_creature_powers: Vec<i32>` field added with
  default `vec![]`
- [ ] `EffectContext.sacrificed_creature_powers: Vec<i32>` field added; both
  `EffectContext::new` and `EffectContext::new_with_kicker` initialize it
- [ ] LKI capture at `casting.rs:3820`-ish before `move_object_to_zone` for
  spell additional cost
- [ ] LKI capture at `abilities.rs:716`-ish before `move_object_to_zone` for
  activated ability sacrifice cost
- [ ] Activated-ability StackObject construction at `abilities.rs:918`-ish
  populates `sacrificed_creature_powers`
- [ ] Resolution propagation in `resolution.rs:1768`-ish (activated ability) and
  the spell-resolution dispatch site for spells with additional sacrifice cost
- [ ] Hash sentinel bumped 5 → 6; M7 uses `assert_eq!(HASH_SCHEMA_VERSION, 6u8)`
- [ ] All 3 confirmed card defs updated (Altar of Dementia, Greater Good, Life's
  Legacy)
- [ ] All 8 MANDATORY tests pass (`cargo test --all`)
- [ ] No new clippy warnings beyond BASELINE-CLIPPY-01..06 baseline
  (`cargo clippy --all-targets -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`) — including replay-viewer and
  TUI exhaustive-match consumers
- [ ] No remaining TODOs in the 3 affected card defs
- [ ] `cargo fmt --check` clean
- [ ] Commit message lists: 3 cards shipped, primitive variant added,
  AdditionalCost shape change, hash bump 5→6, test count delta (+8 mandatory,
  +3 optional)

---

## Risks & Edge Cases

### R1: `AdditionalCost::Sacrifice` shape change blast radius

Converting `Sacrifice(Vec<ObjectId>)` to `Sacrifice { ids, lki_powers }` touches
~10 sites. Most are mechanical pattern fixes (`Sacrifice(ids)` →
`Sacrifice { ids, .. }`). The risk is missing a site or accidentally changing
the JSON serde representation in a way that breaks an existing test fixture
(serialized scripts). **Mitigation**: drive from compiler errors after Change 1;
run `cargo test --all` after the engine compiles to catch any JSON
round-trip failures. **Stop-and-flag** if a serialized script fixture rejects the
new shape — fall back to the sibling-variant route (Change 1 alternative).

### R2: Capture-by-value vs capture-by-ID — chosen capture-by-value

The plan picks **capture-by-value** for the structural reasons documented in the
"Primitive Specification" section: capture-by-ID is unsound under CR 400.7
(old ID dead) AND under BASELINE-LKI-01 (new ID's `calculate_characteristics`
drops battlefield-gated layer effects). Capture-by-value is correct by
construction.

**Trade-off**: if a future card needs to read MULTIPLE characteristics of the
sacrificed creature (e.g., "draw cards equal to its power, then create a token
with its toughness"), capture-by-value would need parallel vectors for each
field (`sacrificed_creature_powers`, `sacrificed_creature_toughnesses`,
`sacrificed_creature_subtypes`). Capture-by-snapshot (storing a `Characteristics`
clone per sacrificed creature) would scale better. **Decision**: capture-by-value
for PB-P; revisit if the toughness/subtype follow-up PB ships.

### R3: Spell-resolution dispatch site for spells with additional sacrifice cost

Change 8 line 2 instructs the runner to find the spell-resolution dispatch site
in `resolution.rs` and add the additional_costs read. The exact line number is
not enumerated because the spell resolution dispatch in resolution.rs has many
branches (Spell with effect, Spell with modes, Spell with X, etc.) and the
runner needs to identify the right `EffectContext::new(...)` site for spell
resolution. **Implementation hint**: grep `resolution.rs` for
`EffectContext::new(`, identify the Spell-kind dispatch (not ActivatedAbility,
not TriggeredAbility), and apply the propagation pattern there. If multiple
Spell-kind sites exist (e.g., one for normal spells, one for split spells, one
for fused spells), apply to all that resolve a spell with effects.

**Stop-and-flag** if the runner cannot identify a single spell-resolution
EffectContext construction site cleanly. Possible mitigation: add the
propagation inside `resolve_amount` itself (read `stack_obj` directly via
context lookup) — but that requires `stack_obj` to be in scope, which it isn't
inside `resolve_amount`. The cleaner fix is at the resolution dispatch site.

### R4: Hash bump and replay-fixture rebakes

Bumping the hash sentinel 5 → 6 means any replay fixture or test that asserts
a specific public_state_hash will fail. **Expected outcome**: the runner runs
`cargo test --all`, observes the failures, updates the expected hash values in
the test fixtures, and recommits. This is standard PB protocol — both PB-N
(sentinel 3 → 4) and PB-D (sentinel 4 → 5) followed the same pattern.

### R5: BASELINE-LKI-01 reach check

BASELINE-LKI-01 documents that `calculate_characteristics` against a graveyard
object drops battlefield-gated layer effects. **Does this reach PB-P?**

Analysis:

- PB-P captures power BEFORE `move_object_to_zone`. At that moment, the source
  is still on the battlefield, so `calculate_characteristics(state, sac_id)`
  applies all layer effects correctly (it's the standard battlefield case).
- After capture, the integer is stored. The graveyard read is never invoked.
- Therefore PB-P is **immune to BASELINE-LKI-01**.

This is precisely why capture-by-value is the chosen path. **Confirmed: PB-P is
safe from BASELINE-LKI-01.** (Compare PB-D, which was also safe via a different
mechanism — DamagedPlayer reads `obj.controller`, not layer-resolved chars.)

### R6: Effect ordering inside Greater Good's Sequence

Greater Good's effect is `Sequence([DrawCards, DiscardCards])`. CR 101.4 (turn-
based actions) and CR 608.2 (effect resolution) require the draw to fully
complete before the discard begins. The Sequence wrapper handles this correctly
in the existing engine — `Effect::Sequence` runs sub-effects in order, each
fully completing before the next starts. **Verify during implementation** that
the draw resolves to completion (cards added to hand) before the discard
selection happens. This is not a PB-P-specific risk; it's existing engine
behavior, but worth a sanity check during M2.

### R7: Mill 0 / Draw 0 / Discard with empty hand

If `EffectAmount::PowerOfSacrificedCreature` returns 0 (M5 case), the downstream
effect must handle 0 cleanly:

- `Effect::Mill { count: 0 }`: should be a no-op. Verify in `effects/mod.rs`.
- `Effect::DrawCards { count: 0 }`: should be a no-op. Verify.
- `Effect::DiscardCards { count: 3 }` with hand size < 3: discard all available.
  CR 701.7c (you can't discard more than you have). Verify the engine's
  DiscardCards saturates at hand size, not panics.

These are not PB-P bugs if they exist — they're pre-existing engine behavior —
but the M5 test will surface any regression.

### R8: AdditionalCost is constructed in script harness — wire-format compatibility

The `replay_harness.rs` constructs `AdditionalCost::Sacrifice(...)` from
JSON-script `cast_spell` actions. If the harness reads sacrifice IDs from JSON
and constructs the variant, it must now construct the struct form
`Sacrifice { ids, lki_powers: vec![] }` (the engine fills `lki_powers` at
cost-payment time). The harness should NOT attempt to populate `lki_powers` —
that's the engine's job.

**Stop-and-flag**: if the runner finds the harness has a code path that
synthesizes an `AdditionalCost::Sacrifice` and feeds it directly to
`StackObject` (bypassing the engine's cost-payment path), the captured
`lki_powers` would never get populated and the spell would resolve with 0 power.
This is unlikely (the harness goes through `process_command(CastSpell)` which
re-runs `casting.rs`'s cost-payment path) but worth verifying during M3 setup.

### R9: Confirmed yield (3) is well below the calibration band

PB-P's yield is 23% (3/13), below the 40-65% filter-PB calibration band per
`memory/feedback_pb_yield_calibration.md`. The wip file already acknowledges
this. **Flagged here for record-keeping**. The narrowness is acceptable because:
1. The 3 cards have explicit TODOs naming the primitive (no over-claim).
2. The primitive is real and structurally well-defined (CR 608.2b LKI).
3. The engine plumbing is reusable for the toughness/subtype follow-ups.
4. Re-triage from PB-D's pre-check correctly identified that the broader
   "PowerOfCreature" PB queue entry was overcounted (existing
   `PowerOf(EffectTarget)` covered most cases).

### R10: BASELINE-CLIPPY-01..06 are out of scope

Per `memory/primitive-wip.md` line 149, PB-P does NOT fix the baseline clippy
warnings. The runner runs `cargo clippy --all-targets -- -D warnings` and
verifies that NO NEW warnings beyond the baseline are introduced. If new
warnings appear, fix them; if old baseline warnings are touched, leave them.

---

## Implementation Notes (for the runner)

- **Step order**: do Change 1 (AdditionalCost shape change) first, then Change 4
  (EffectAmount variant), then Change 2 (StackObject field), then Change 3
  (EffectContext field). Run `cargo build --workspace` after each. Drive
  remaining sites from compiler errors.
- **Hash bump last**: update `HASH_SCHEMA_VERSION` only after all code changes
  compile, so the parity test (M7) doesn't need multiple edits.
- **LKI capture sites**: Change 6 (casting.rs spell additional cost) and Change
  7 (abilities.rs activated ability sacrifice cost) are the load-bearing capture
  points. Both must use `calculate_characteristics(state, sac_id).power`
  BEFORE `move_object_to_zone`. The `or_else` fallback to base characteristics
  is for the rare case where `calculate_characteristics` returns None (e.g., the
  sacrifice target has been corrupted between validation and payment); in
  practice this should be unreachable.
- **Test order**: write M7 (hash parity) first — smallest, validates sentinel
  bump path. Then M6 (defensive zero) — smallest unit test. Then M5 (zero power
  end-to-end) — easiest end-to-end. Then M1 (Altar end-to-end). Then M2 (Greater
  Good end-to-end). Then M3 (Life's Legacy end-to-end). Then M4 (LKI anthem
  correctness — load-bearing). Then M8 (backward compat regression). Optional
  tests last.
- **Card def edits come AFTER the engine compiles cleanly**. Don't mix engine
  and card changes. Card edits are 3 small files; once the engine's primitive
  is in place and tests pass, the card def changes are drop-in.
- **Backfill check**: after the EffectAmount variant lands, grep one more time
  for `EffectAmount::PowerOf` to confirm no new sites have been added between
  plan time and implement time that would also need a `PowerOfSacrificedCreature`
  arm.
- **Expected commit size**: ~200 LOC engine diff (heavier than PB-D due to the
  AdditionalCost shape change blast radius) + ~60 LOC card def diff + ~350 LOC
  test diff. Larger than PB-D's shape but still single-PB-sized.
- **Opportunistic warstorm_surge cleanup**: if time permits in fix-phase, strip
  the stale TODO from `warstorm_surge.rs:4-9` and replace with a one-line CR
  citation. NOT required for PB-P merge.
