# Primitive Batch Plan: PB-XA2 — `TargetFilter.is_blocking` + `is_tapped` + `is_untapped` runtime predicates

**Generated**: 2026-05-15
**Worker**: scutemob-26 (`feat/pb-xa2-targetfilter-isblocking-istapped-isuntapped-runtime-p`)
**Predecessor**: PB-XA (`scutemob-24`, shipped 2026-05-15) added enforcement of the
pre-existing `TargetFilter.is_attacking` field at 4 validate sites + 6 trigger
auto-target picker sites. PB-XA2 extends the same pattern with **three new fields**:
`is_blocking`, `is_tapped`, `is_untapped` (OOS-XA-1 + OOS-XA-2 from PB-XA's seed
list, merged into one batch because all three follow the identical mechanical
template). The structural ancestor for `self_id` threading is PB-XS
(`scutemob-21`, 2026-05-14), which introduced `validate_targets_with_source`.
**CR Rules**:
- **CR 109.1** — object identity (target legality at announcement).
- **CR 508.1k** — "attacking creature" defined as a creature in
  `CombatState.attackers` (already enforced for PB-XA; cited for the
  "attacking or blocking" OR semantics).
- **CR 509.1** — declare-blockers turn-based action; a creature is "blocking"
  iff it appears in `CombatState.blockers` for the duration of combat.
- **CR 509.1c** — blocking-requirement check; the live `blockers` map keys on
  blocker `ObjectId`.
- **CR 601.2c** — at cast/activation time each declared target must satisfy
  the `TargetRequirement` predicates (validate site).
- **CR 603.3d** — a triggered ability that requires targets but has no legal
  targets is not put on the stack (trigger-picker no-legal-target skip).
- **CR 701.20a** — "tap" definition; affects `GameObject.status.tapped`.
- **CR 701.21a** — "untap" definition; mutually exclusive with tapped state.

**Cards in scope**: **1** confirmed — Eiganjo, Seat of the Empire (Channel half:
"target attacking or blocking creature"). Zero current defs use the
"target tapped creature" / "target untapped creature" / "target blocking
creature" patterns in isolation (verified via grep at
`crates/engine/src/cards/defs/`; only "blocking creature" appears, only in
Eiganjo's oracle and only conjoined with "attacking"). The worker MUST
re-verify Eiganjo's oracle text via `mcp__mtg-rules__lookup_card` before
authoring the filter change.

**HASH bump**: **21 → 22**. Three new `bool` fields on `TargetFilter`. Each
adds a new arm to `HashInto for TargetFilter` in `state/hash.rs`. All existing
PB hash canary tests across the 16 `primitive_pb_*.rs` / `pbn_*.rs` / `pbd_*.rs`
/ `pbp_*.rs` / `pbt_*.rs` / `effect_sacrifice_permanents_filter.rs` test files
that assert `HASH_SCHEMA_VERSION == 21u8` must be bumped uniformly to `22u8`
and their sentinel messages rewritten to cite PB-XA2.

---

## Executive Summary

PB-XA2 is a **mechanical 3-predicate extension** of PB-XA. Three new bool fields
mirror `is_attacking` exactly: each adds a `passes_<name>` term at the same 10
enforcement sites. Engine surface is symmetric — same comment-then-let-then-AND
structure, same naming convention, same graveyard-arm uniform treatment.

The only design decision PB-XA2 makes is **OR-semantics for "attacking or
blocking creature"** (Eiganjo's oracle). The plan resolves this in
[Section: OR-semantics decision](#or-semantics-decision-ac-3878) below — choose
**option (a) two-bool-OR**, gated by a single `passes_combat_role` term that
collapses the `is_attacking` × `is_blocking` interaction. `is_tapped` and
`is_untapped` are independent AND terms (they are mutually-exclusive states of
the same object; setting both produces an unreachable filter, which
correctly yields "no legal target").

| New field | Predicate | Notes |
|-----------|-----------|-------|
| `is_blocking: bool` | `state.combat.as_ref().is_some_and(\|c\| c.is_blocking(id))` | New helper `CombatState::is_blocking` (`blockers.contains_key`). |
| `is_tapped: bool` | `state.objects.get(&id).is_some_and(\|o\| o.status.tapped)` | No combat lookup; reads `GameObject.status.tapped` directly. |
| `is_untapped: bool` | `state.objects.get(&id).is_some_and(\|o\| !o.status.tapped)` | Same path; negated bool. |

Total engine LOC delta: ~120 lines (3 fields × 10 sites × ~4 lines per term,
minus shared helper extraction opportunities) + 3 hash arms + 1 helper +
1 HASH bump + 1 history doc-comment + 1 doc-comment block on `TargetFilter`
+ ~700-line new `tests/primitive_pb_xa2.rs` test file.

---

## OR-Semantics Decision (AC 3878)

**Decision**: **Option (a) — two bools combined with OR at the validate site,
ONLY for the `is_attacking` × `is_blocking` pair.** All other field interactions
remain AND.

### Rationale

The PB-XA review (post fix-phase) explicitly recommended option (b) (a
`CombatRole` enum) as cleaner long-term. PB-XA2 chooses option (a) because:

1. **Scope economy**: option (b) adds a new discriminant + hash arm + serde
   migration + adds a new enum to `helpers.rs`. For a 1-card driver, option
   (b)'s overhead is higher than the entire mechanical extension.
2. **Symmetry with `is_attacking`**: the field already exists as a bool and
   is consumed in `effects/mod.rs:matches_filter` as a bool. Introducing an
   enum forces a parallel bool-vs-enum coexistence (the existing `is_attacking`
   field can't be deleted without breaking the effect-resolution path used
   by `aetherize.rs` / `blessed_alliance.rs`'s `BounceAll.filter` /
   `SacrificePermanents.filter`).
3. **AND-of-passes preserved at the term level**: option (a) is expressed as
   a single combined term `passes_combat_role` that collapses the
   `is_attacking` × `is_blocking` interaction into one bool, then AND-chained
   with `passes_filter && passes_controller && passes_self && passes_tapped
   && passes_untapped`. The term-level AND structure stays intact — the OR is
   internal to one term.

### Semantics

For each enforcement site, define:

```rust
let passes_combat_role = match (filter.is_attacking, filter.is_blocking) {
    (false, false) => true,                                                  // no restriction
    (true,  false) => state.combat.as_ref().is_some_and(|c| c.attackers.contains_key(&id)),
    (false, true ) => state.combat.as_ref().is_some_and(|c| c.is_blocking(id)),
    (true,  true ) => state.combat.as_ref().is_some_and(|c|
        c.attackers.contains_key(&id) || c.is_blocking(id)
    ),  // OR semantics: "attacking OR blocking"
};
```

`is_tapped` and `is_untapped` use the AND-of-passes pattern unmodified:

```rust
let passes_tapped = !filter.is_tapped
    || state.objects.get(&id).is_some_and(|o| o.status.tapped);
let passes_untapped = !filter.is_untapped
    || state.objects.get(&id).is_some_and(|o| !o.status.tapped);
```

If a card author ever sets BOTH `is_tapped = true` AND `is_untapped = true` on
the same filter, the result is an unreachable filter that yields "no legal
target" — this is correct (no creature is both tapped and untapped), and the
worker MUST add a defensive comment near the field definition noting this.

### Migration Path

`passes_attacking` is **replaced** by `passes_combat_role` in PB-XA2.

- All 10 PB-XA `passes_attacking` definitions (V1-V4 + T1-T6) are renamed and
  the body extended to match the four-way match above. The trailing AND chain
  swaps `passes_attacking` → `passes_combat_role`.
- The graveyard arms (V3, V4, T1, T2) still receive the combined check
  unchanged — graveyard objects are never in either `attackers` or `blockers`,
  so all three of (T,F), (F,T), (T,T) variants reject correctly.
- The graveyard inline filter expressions in `abilities.rs` (lines 6640-6643,
  6658-6661 — single-expression `&& (!filter.is_attacking || ...)`) are
  rewritten to match the four-way pattern by extracting the combat-role check
  into a local closure or `let` binding immediately before the `.find(...)`.

### Tests

The decision is exercised by F-1/F-2 in `tests/primitive_pb_xa2.rs` (a creature
in `attackers` AND a creature in `blockers` AND a non-combatant on the
battlefield — verify the picker accepts the attacker OR blocker but rejects the
non-combatant, with each role tested independently). The (T,T) "attacking OR
blocking" case is the Eiganjo-shaped discriminator.

---

## Full-Chain Verification

### Eiganjo, Seat of the Empire — Channel half (in scope)

**Oracle** (verified via `mcp__mtg-rules__lookup_card`): "Channel — {2}{W},
Discard this card: It deals 4 damage to target attacking or blocking creature.
This ability costs {1} less to activate for each legendary creature you
control."

| Hop | Status | Notes |
|-----|--------|-------|
| Activated ability cast from graveyard | OK | Channel infrastructure works (Channel keyword + DiscardSelf cost). |
| Cost reduction | OK | `activated_ability_cost_reductions` already populated correctly. |
| Target validation `validate_object_satisfies_requirement` | **BITING AFTER PB-XA2** | Currently uses bare `TargetCreature` (overly broad — TODO at line 25-28). After PB-XA2: `TargetCreatureWithFilter { is_attacking: true, is_blocking: true, ..Default::default() }`. V1 path bites; the (T,T) branch of `passes_combat_role` accepts either role. |
| Effect resolution `DealDamage` | OK | Pre-existing. |
| Dispatch chain | OK | Activated-ability target validation → V1 declarative path → `passes_combat_role` returns true iff the candidate is in `attackers` OR `blockers`. |

**Authorability after PB-XA2**: YES. Delete the TODO at lines 24-28; switch
`TargetCreature` to `TargetCreatureWithFilter { is_attacking: true,
is_blocking: true, ..Default::default() }` at line 39.

### Defs/ sweep — verified, no other cards in scope

Verified at plan time via:
```
grep -rn "target.\+\(tapped\|untapped\|blocking\) \(creature\|artifact\|permanent\)" \
  crates/engine/src/cards/defs/ -i
```
Only match: Eiganjo (oracle text mentioning "blocking creature"). Zero
defs use "target tapped creature" / "target untapped creature" / "target
blocking creature" patterns in isolation. The worker MUST re-verify with a
wider grep at implement time and consult MCP `lookup_card` for any candidate
that surfaces.

### Eiganjo Channel cost-reduction filter — verify untouched

Eiganjo's `SelfActivatedCostReduction::PerPermanent { filter: TargetFilter
{ legendary: true, has_card_type: Some(CardType::Creature), .. } }` is on the
COST-REDUCTION side, not the target-side. It must remain unchanged. The
PB-XA2 fields are not relevant here — the cost-reduction filter scans the
controller's battlefield for legendary creatures, not the activation target.

---

## Step-By-Step

### Step 1 — Add three fields to `TargetFilter`

**File**: `crates/engine/src/cards/card_definition.rs`
**Location**: after the existing `exclude_self: bool` field at line 2658.
**Action**: append three new bool fields with `#[serde(default)]` + doc
comments mirroring the `is_attacking` / `exclude_self` "Enforced at:" template.

```rust
/// PB-XA2: CR 509.1 — Must be currently blocking (a creature in
/// `CombatState.blockers`). Mirrors `is_attacking` for the blocker side
/// of "target attacking or blocking creature" (Eiganjo, Seat of the
/// Empire Channel half). NOT checked inside `matches_filter()`.
///
/// When this AND `is_attacking` are BOTH set on the same filter, the
/// validate sites apply OR semantics (accept either role) — see
/// `passes_combat_role` in `casting.rs` / `abilities.rs`.
///
/// Enforced at:
/// - `casting::validate_object_satisfies_requirement` (declarative
///   target validation, CR 601.2c) for TargetCreatureWithFilter,
///   TargetPermanentWithFilter, TargetCardInYourGraveyard,
///   TargetCardInGraveyard — uses `state.combat.as_ref()
///   .is_some_and(|c| c.is_blocking(id))`.
/// - `abilities.rs` auto-target picker for triggered abilities — same
///   predicate using `obj.id` as the candidate identifier.
#[serde(default)]
pub is_blocking: bool,

/// PB-XA2: CR 701.20a — Must be currently tapped (`GameObject.status
/// .tapped == true`). Runtime `GameObject` field, NOT a Characteristics
/// property. NOT checked inside `matches_filter()`.
///
/// Setting both `is_tapped` AND `is_untapped` on the same filter yields
/// an unreachable filter (no creature is both states simultaneously) —
/// the validate sites will return "no legal target" for any candidate.
///
/// Enforced at:
/// - `casting::validate_object_satisfies_requirement` (declarative
///   target validation, CR 601.2c) — same 4 variants as `is_attacking`.
/// - `abilities.rs` auto-target picker for triggered abilities.
#[serde(default)]
pub is_tapped: bool,

/// PB-XA2: CR 701.21a — Must be currently untapped (`GameObject.status
/// .tapped == false`). Runtime `GameObject` field, NOT a Characteristics
/// property. NOT checked inside `matches_filter()`. See `is_tapped` for
/// the mutually-exclusive-state caveat.
///
/// Enforced at: same sites as `is_tapped` (4 validate + 6 picker).
#[serde(default)]
pub is_untapped: bool,
```

Also update the **existing** `is_attacking` doc comment (lines 2600-2614) to
add a cross-reference noting that when `is_blocking` is also set, the validate
sites apply OR semantics.

### Step 2 — Add `CombatState::is_blocking` helper

**File**: `crates/engine/src/state/combat.rs`
**Location**: after the existing `is_blocked` method at line 112.

```rust
/// PB-XA2: Returns `true` if `id` is currently declared as a blocker
/// (CR 509.1c — `id` keys into `CombatState.blockers`).
///
/// Distinct from `is_blocked(attacker_id)` — this checks whether `id`
/// IS a blocker, not whether `id` IS BLOCKED. Used by
/// `TargetFilter.is_blocking` enforcement at validate sites and the
/// trigger auto-target picker.
pub fn is_blocking(&self, id: ObjectId) -> bool {
    self.blockers.contains_key(&id)
}
```

### Step 3 — Update `HashInto for TargetFilter` (HASH 21→22)

**File**: `crates/engine/src/state/hash.rs`

**Location 1** — `HashInto for TargetFilter` at line 4341-4372. After
`self.exclude_self.hash_into(hasher)` (line 4370), add:

```rust
// PB-XA2: blocking-role runtime predicate (CR 509.1).
self.is_blocking.hash_into(hasher);
// PB-XA2: tapped-state runtime predicate (CR 701.20a).
self.is_tapped.hash_into(hasher);
// PB-XA2: untapped-state runtime predicate (CR 701.21a).
self.is_untapped.hash_into(hasher);
```

**Location 2** — HASH_SCHEMA_VERSION at line 146:

```rust
pub const HASH_SCHEMA_VERSION: u8 = 22;
```

**Location 3** — append schema-version-history block after line 145:

```rust
/// - 22: PB-XA2 (2026-05-15) — `TargetFilter.is_blocking: bool`,
///   `TargetFilter.is_tapped: bool`, `TargetFilter.is_untapped: bool`
///   added (CR 509.1 / 701.20a / 701.21a). Enforced at the 10 PB-XA
///   sites (4 declarative validate + 6 trigger auto-target picker) via
///   `passes_combat_role` (combines `is_attacking` × `is_blocking` with
///   OR semantics for "attacking or blocking creature") and per-field
///   `passes_tapped` / `passes_untapped` AND terms. New `CombatState::
///   is_blocking(id)` helper. Backward compatible via
///   `#[serde(default)] false`. Unblocks Eiganjo, Seat of the Empire
///   Channel half. Replaces PB-XA's `passes_attacking` term at all
///   sites.
```

### Step 4 — Enforcement at validate sites (V1-V4, `rules/casting.rs`)

**File**: `crates/engine/src/rules/casting.rs`

For each of V1 (lines 5707-5733), V2 (lines 5735-5761), V3 (lines 5765-5783),
V4 (lines 5785-5801):

1. **Rename** the existing `passes_attacking` block to `passes_combat_role`
   with the four-way match described in the OR-semantics decision.
2. **Add** two new `passes_tapped` / `passes_untapped` lets immediately after
   the `passes_combat_role` block.
3. **Update** the trailing AND chain to swap `passes_attacking` →
   `passes_combat_role` and append `&& passes_tapped && passes_untapped`.

V1 (TargetCreatureWithFilter, line 5707) example after the change:

```rust
// PB-XA2: CR 508.1k / 509.1c / 601.2c — "target attacking [or
// blocking] X" restricts to creatures in combat.attackers OR
// combat.blockers. When both is_attacking AND is_blocking are set,
// the filter accepts either role (Eiganjo Channel half).
let passes_combat_role = match (filter.is_attacking, filter.is_blocking) {
    (false, false) => true,
    (true,  false) => state.combat.as_ref().is_some_and(|c| c.attackers.contains_key(&id)),
    (false, true ) => state.combat.as_ref().is_some_and(|c| c.is_blocking(id)),
    (true,  true ) => state.combat.as_ref().is_some_and(|c|
        c.attackers.contains_key(&id) || c.is_blocking(id)
    ),
};
// PB-XA2: CR 701.20a — "target tapped X" reads GameObject.status.tapped.
let passes_tapped = !filter.is_tapped
    || state.objects.get(&id).is_some_and(|o| o.status.tapped);
// PB-XA2: CR 701.21a — "target untapped X" reads !status.tapped.
let passes_untapped = !filter.is_untapped
    || state.objects.get(&id).is_some_and(|o| !o.status.tapped);
passes_filter && passes_controller && passes_self
    && passes_combat_role && passes_tapped && passes_untapped
```

V2/V3/V4 follow identical structure. V3 (graveyard arm at line 5765-5783): the
`passes_combat_role` check correctly rejects all candidates because graveyard
objects are never in `attackers` or `blockers`. The `passes_tapped` /
`passes_untapped` checks are LIVE on graveyard arms — `status.tapped` is a
permanent-only concept (CR 110.5), but reading `GameObject.status.tapped` for
a graveyard object returns the default `false`. **Semantics caveat**: setting
`is_tapped=true` on a graveyard target filter rejects all candidates (correct
— graveyard cards aren't tapped in any meaningful sense). Setting
`is_untapped=true` on a graveyard target ACCEPTS all candidates (the default
`false` satisfies `!tapped`). The worker MUST add a comment near the graveyard
arms noting this is a design quirk; the resolution is that no card legitimately
uses `is_tapped` / `is_untapped` on a graveyard filter, so the behavior is a
defensive degenerate case rather than a correctness bug. If a card ever needs
this, the right fix is a dedicated graveyard-target predicate, not retrofitting
the runtime field.

### Step 5 — Enforcement at trigger auto-target picker (T1-T6, `rules/abilities.rs`)

**File**: `crates/engine/src/rules/abilities.rs`

| Site | Lines (current, PB-XA-bumped) | Variant | Pattern |
|------|------------------------------|---------|---------|
| T1 | 6625-6649 | TargetCardInYourGraveyard (graveyard scan) | inline in `.find()` closure |
| T2 | 6650-6666 | TargetCardInGraveyard (graveyard scan) | inline in `.find()` closure |
| T3 | 6752-6782 | TargetCreatureWithFilter (top-level battlefield) | `let passes_*` then AND |
| T4 | 6783-6810 | TargetPermanentWithFilter (top-level battlefield) | `let passes_*` then AND |
| T5 | 6851-6868 | TargetCreatureWithFilter (UpToN inner) | `let passes_*` then AND |
| T6 | 6869-6884 | TargetPermanentWithFilter (UpToN inner) | `let passes_*` then AND |

For T3-T6, rename `passes_attacking` → `passes_combat_role` (four-way match,
same as casting.rs) and add `passes_tapped` / `passes_untapped` lets. Update
the final AND-chained expression.

For T1/T2 (inline `.find()` closures, lines 6640-6643 and 6658-6661), the
current single-expression `&& (!filter.is_attacking || ...)` clause needs
expanding. Two acceptable refactor paths:

- **Path A (preferred — readability)**: extract the predicate computation into
  a `let` binding ABOVE the `.find(...)` call so the closure body stays compact:

  ```rust
  let filter = filter.clone();           // already in scope
  let combat = state.combat.as_ref();
  state.objects.iter().find(|(_, obj)| {
      let role_ok = match (filter.is_attacking, filter.is_blocking) {
          (false, false) => true,
          (true,  false) => combat.is_some_and(|c| c.attackers.contains_key(&obj.id)),
          (false, true ) => combat.is_some_and(|c| c.is_blocking(obj.id)),
          (true,  true ) => combat.is_some_and(|c|
              c.attackers.contains_key(&obj.id) || c.is_blocking(obj.id)
          ),
      };
      let tapped_ok = !filter.is_tapped || obj.status.tapped;
      let untapped_ok = !filter.is_untapped || !obj.status.tapped;
      obj.zone == controller_gy
          && crate::effects::matches_filter(&obj.characteristics, &filter)
          && (!filter.exclude_self || obj.id != trigger.source)
          && role_ok && tapped_ok && untapped_ok
  })
  ```

- **Path B (minimal diff)**: keep everything inline; one continuation per term.
  More verbose; rejected.

Use Path A. Worker MUST preserve `filter` ownership semantics (the existing
code clones via the surrounding pattern match — verify at implement time).

### Step 6 — Update Eiganjo def

**File**: `crates/engine/src/cards/defs/eiganjo_seat_of_the_empire.rs`

Replace lines 24-39 (the TODO comment block + the `TargetCreature` line) with:

```rust
            // PB-XA2: "target attacking or blocking creature" — filter
            // applies OR semantics when both is_attacking and is_blocking
            // are set (see `passes_combat_role` in
            // rules/casting.rs / rules/abilities.rs).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, white: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(4),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    is_attacking: true,
                    is_blocking: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
```

Verify oracle text via `mcp__mtg-rules__lookup_card` for "Eiganjo, Seat of the
Empire" before authoring (already done at plan time: oracle matches).

### Step 7 — Bump ALL existing PB hash canary tests

**Files** (per grep at plan time — 16 sentinel assertions across 16 files):

| File | Line | Current value |
|------|------|---------------|
| `crates/engine/tests/primitive_pb_ewc.rs` | 401 | `21u8` |
| `crates/engine/tests/pbp_power_of_sacrificed_creature.rs` | 782 | `21u8` |
| `crates/engine/tests/primitive_pb_xs.rs` | 69 | `21u8` |
| `crates/engine/tests/pbn_subtype_filtered_triggers.rs` | 558 | `21u8` |
| `crates/engine/tests/primitive_pb_xs_e.rs` | 160 | `21u8` |
| `crates/engine/tests/primitive_pb_cc_a.rs` | 101 | `21u8` |
| `crates/engine/tests/primitive_pb_cc_c_followup.rs` | 400 | `21u8` |
| `crates/engine/tests/primitive_pb_lki_cc.rs` | 440 | `21u8` |
| `crates/engine/tests/primitive_pb_xa.rs` | 92 | `21u8` |
| `crates/engine/tests/primitive_pb_lki_power.rs` | 385 | `21u8` |
| `crates/engine/tests/primitive_pb_eat.rs` | 143 | `21u8` |
| `crates/engine/tests/pbd_damaged_player_filter.rs` | 597 | `21u8` |
| `crates/engine/tests/effect_sacrifice_permanents_filter.rs` | 136 | `21u8` |
| `crates/engine/tests/primitive_pb_ts.rs` | 369 | `21u8` |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | 411 | `21u8` |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | 867 | `21u8` |

All must be uniformly bumped to `22u8` and their sentinel messages rewritten to
cite PB-XA2 (the predecessor citation must mention PB-XA2 explicitly so the
next reviewer can re-orient).

Also check: `primitive_pb_cc_c.rs` (which was not in the grep — confirms
it has no sentinel) and any tests added between snapshot time and implementation
time (worker MUST re-grep at implement time and bump any new sentinels).

Use the PB-XA pattern message:
> "PB-XA2 bumped HASH_SCHEMA_VERSION 21→22 (TargetFilter.is_blocking/is_tapped/is_untapped, CR 509.1/701.20a/701.21a). If you bumped again, update this test and state/hash.rs history."

### Step 8 — Tests (`tests/primitive_pb_xa2.rs`)

NEW file. Mirror `tests/primitive_pb_xa.rs` structure. Estimated ~750 LOC, 16
tests across sections A-H.

| ID | Section | Test name | Purpose | Discriminator? |
|----|---------|-----------|---------|----------------|
| A-1 | hash sentinel | `test_pb_hash_schema_version_live_sentinel` | `assert_eq!(HASH_SCHEMA_VERSION, 22u8)` | regression |
| A-2 | serde-default | `test_pbxa2_serde_default_deserialize_pre_xa2_snapshot` | old JSON snapshot missing the 3 new fields deserializes with all three `false` | regression |
| B-1 | PartialEq | `test_pbxa2_filter_equality_distinguishes_is_blocking` | filters differing only on `is_blocking` are not equal | regression |
| B-2 | PartialEq | `test_pbxa2_filter_equality_distinguishes_is_tapped` | filters differing only on `is_tapped` are not equal | regression |
| B-3 | PartialEq | `test_pbxa2_filter_equality_distinguishes_is_untapped` | filters differing only on `is_untapped` are not equal | regression |
| C-1 | `is_blocking` validate, **negative** | `test_pbxa2_activated_target_is_blocking_non_blocker_rejected` | `TargetCreatureWithFilter{is_blocking=true}` rejects a creature not in `combat.blockers`. Exercises V1 negative path. | positive |
| C-2 | `is_blocking` validate, **positive** | `test_pbxa2_activated_target_is_blocking_blocker_accepted` | same filter accepts a creature whose ObjectId keys into `combat.blockers`. Exercises V1 positive path. | positive |
| D-1 | `is_tapped` validate, **negative** | `test_pbxa2_activated_target_is_tapped_untapped_rejected` | `TargetCreatureWithFilter{is_tapped=true}` rejects a creature with `status.tapped = false`. Exercises V1 negative path. | positive |
| D-2 | `is_tapped` validate, **positive** | `test_pbxa2_activated_target_is_tapped_tapped_accepted` | same filter accepts a tapped creature. | positive |
| E-1 | `is_untapped` validate, **negative** | `test_pbxa2_activated_target_is_untapped_tapped_rejected` | `TargetCreatureWithFilter{is_untapped=true}` rejects a creature with `status.tapped = true`. | positive |
| E-2 | `is_untapped` validate, **positive** | `test_pbxa2_activated_target_is_untapped_untapped_accepted` | same filter accepts an untapped creature. | positive |
| F-1 | OR-semantics "attacking or blocking", **positive on attacker** | `test_pbxa2_activated_target_attacking_or_blocking_accepts_attacker` | Eiganjo-shaped filter `{is_attacking=true, is_blocking=true}` accepts an attacking creature (in `combat.attackers`, NOT in `blockers`). | positive |
| F-2 | OR-semantics, **positive on blocker** | `test_pbxa2_activated_target_attacking_or_blocking_accepts_blocker` | same filter accepts a blocking creature (in `combat.blockers`, NOT in `attackers`). | positive |
| F-3 | OR-semantics, **negative** | `test_pbxa2_activated_target_attacking_or_blocking_rejects_non_combatant` | same filter rejects a battlefield creature not in either map. | positive |
| G-1 | trigger picker, `is_blocking` discrimination | `test_pbxa2_trigger_picker_selects_blocking_creature_positive` | **ObjectId-ordered discriminator** (per H-XA-01 lesson): non-blocker Sitter added BEFORE blocker Defender so Sitter gets the smaller ObjectId; without PB-XA2 enforcement the picker would short-circuit on Sitter, with enforcement it advances to Defender. Includes `assert!(sitter_id < defender_id)` sanity guard. | **mandatory positive discriminator** |
| G-2 | trigger picker, no-legal-target skip | `test_pbxa2_trigger_picker_skipped_when_no_blocker` | per CR 603.3d, trigger with `is_blocking=true` and empty `blockers` map is skipped (no stack object created). | positive |
| H-1 | graveyard arm rejects all three runtime fields | `test_pbxa2_graveyard_target_with_runtime_fields_rejects` | `TargetCardInYourGraveyard{is_blocking=true}` always rejects; `TargetCardInYourGraveyard{is_tapped=true}` always rejects (graveyard `status.tapped` is default `false`); `TargetCardInYourGraveyard{is_untapped=true}` accepts a graveyard candidate matching `matches_filter` (since `!status.tapped` is true for default `false`) — but this is a degenerate edge case and the test asserts this behavior to lock it in. | regression |

**Discriminator-test discipline** (per H-XA-01 and `feedback_verify_full_chain`):
G-1 MUST be a POSITIVE discriminator. Test setup MUST satisfy:
- Sitter (non-blocker, NOT in `combat.blockers`) is added FIRST → smaller ObjectId.
- Defender (blocker, IN `combat.blockers`) is added SECOND → larger ObjectId.
- `assert!(sitter_id < defender_id)` sanity guard.
- Trigger fires with `TargetPermanentWithFilter { has_card_type: Some(Creature),
  is_blocking: true, ..Default::default() }`.
- Assertion: `target_id == defender_id` AND `target_id != sitter_id`.
- Mental-toggle check: with the `passes_combat_role` enforcement removed at T4,
  the test MUST fail with `Got id ObjectId(<sitter_id>)`. This is the criterion
  that satisfies the test-validity invariant. The worker is expected to verify
  this mental-toggle check during implementation (do NOT skip it).

**Discriminator check for `is_tapped` / `is_untapped`** (D-2 / E-2 positives):
the positive-case tests are not subject to ObjectId-ordering ambiguity because
there is only ONE creature in the scenario — the candidate either passes or
doesn't. D-1 and E-1 are paired with their positive counterparts D-2/E-2 to
form mutual discriminators (a positive set on a negative state must fail; a
positive set on a positive state must succeed).

**Use `combat_with_attacker` helper from PB-XA** as a baseline; add
`combat_with_blocker(blocker_id, attacker_id)` for the new map population.
Pattern:
```rust
fn combat_with_blocker(blocker_id: ObjectId, attacker_id: ObjectId) -> CombatState {
    CombatState {
        attacking_player: p(1),
        attackers: [(attacker_id, AttackTarget::Player(p(2)))].into_iter().collect(),
        blockers: [(blocker_id, attacker_id)].into_iter().collect(),
        ..CombatState::new(p(1))
    }
}
```

**Use `with_tapped()` ObjectSpec builder** (verify it exists; grep at implement
time. If absent, mutate `state.objects[id].status.tapped = true` manually after
build, matching the C-2 pattern for `state.combat = Some(...)`).

### Step 9 — File OOS-XA2 seeds (`memory/primitives/pb-retriage-CC.md`)

Append a `## OOS seeds filed by PB-XA2 (scutemob-26, 2026-05-15)` section.
Suggested OOS entries:

- **OOS-XA2-1** — target-side color predicate audit: `TargetFilter.colors` /
  `TargetFilter.exclude_colors` are pre-existing fields whose enforcement at
  validate sites currently routes through `matches_filter` (which DOES read
  `Characteristics.colors`). Verify this is correct — color is a
  `Characteristics` field, not a runtime field, so the routing should already
  be correct. The audit goal is to spot-check, not implement.
- **OOS-XA2-2** — target-side `has_name` enforcement audit. `TargetFilter
  .has_name: Option<String>` exists; verify `matches_filter` and the validate
  sites enforce it. Likely already correct; audit pass needed.
- **OOS-XA2-3** — target-side `is_nontoken` enforcement audit (carried forward
  from PB-XA OOS-XA-3). Re-audit required: is the field consumed at the
  validate sites? Pre-existing OOS-XA-3 from `pb-retriage-CC.md:1026-1053`
  remains open; PB-XA2 does not address it.
- **OOS-XA2-4** — `CombatRole` enum refactor: the PB-XA review's recommended
  long-term cleanup of replacing `is_attacking: bool` + `is_blocking: bool`
  with a `combat_role: Option<CombatRole>` enum. Filed as a future refactor;
  not needed unless a third combat-role variant appears (e.g. "creature that
  blocked or was blocked").
- **OOS-XA2-5** — runtime-predicate helper extraction (PB-XA E-XA-01 carryforward):
  extract a `runtime_predicates_pass(state, id, filter, self_id) -> bool` helper
  that bundles all current runtime checks (`passes_self`, `passes_combat_role`,
  `passes_tapped`, `passes_untapped`). Reduces 10-site duplication. Light
  refactor (~80 LOC delta net negative).

The worker MUST keep these OOS entries brief (paragraph each) and reference
the original OOS-XA entries by ID. Do not duplicate full PB-XA seed content.

### Step 10 — Gate checks + review spawn

1. `cargo build --workspace` — clean.
2. `cargo test --workspace` — expect `2789 + ~16 = ~2805` tests passing. The
   actual baseline at PB-XA2 start is determined by `cargo test --workspace`
   right after worktree checkout; the new test file contributes 16 tests, the
   16 sentinel bumps don't change the test count (same assertion, new value).
3. `cargo clippy --workspace --all-targets -- -D warnings` — clean. Especially
   verify no `match` exhaustiveness warnings for the new tuple-match in
   `passes_combat_role`.
4. `cargo fmt --check` — clean.
5. Spawn `primitive-impl-reviewer` (Opus, RA). Resolve HIGH/MEDIUM findings
   inline before signal-ready. Likely areas of reviewer scrutiny (pre-empt):
   - G-1 ObjectId-ordering discriminator validity (mental-toggle check).
   - V3/V4 graveyard arm semantics for `is_untapped` accepting default-`false`
     candidates — confirm comment + intentionality.
   - T1/T2 closure refactor preserves `filter` ownership.
   - All 16 sentinel bumps applied uniformly.
   - HASH history doc-comment block at version 22.
6. `esm task satisfy` each AC after green; `esm task signal-ready scutemob-26`.

---

## Files Modified (expected)

| File | Action | Approx LOC delta |
|------|--------|------------------|
| `crates/engine/src/cards/card_definition.rs` | +3 fields + doc comments | +60 |
| `crates/engine/src/state/combat.rs` | +1 helper `is_blocking` | +10 |
| `crates/engine/src/state/hash.rs` | +3 hash arms + HASH bump 21→22 + history block | +20 |
| `crates/engine/src/rules/casting.rs` | rename + extend V1-V4 (4 sites) | +60 |
| `crates/engine/src/rules/abilities.rs` | rename + extend T1-T6 (6 sites; T1/T2 refactor to `let` pattern) | +100 |
| `crates/engine/src/cards/defs/eiganjo_seat_of_the_empire.rs` | delete TODO, switch filter | -4, +10 (net +6) |
| `crates/engine/tests/primitive_pb_xa2.rs` | NEW test file | +750 |
| 16 existing test files (Step 7) | bump sentinel 21u8 → 22u8 | +0 (same line count) |
| `memory/primitives/pb-retriage-CC.md` | append OOS-XA2 section | +80 |

**No changes expected** to:
- `cards/helpers.rs` (no new enum / type to export; `TargetFilter` is already exported)
- `state/game_object.rs` (no change to `ObjectStatus`)
- `effects/mod.rs` (no change to `matches_filter`; runtime fields stay opt-out)
- `replay_harness.rs` (no schema flow change)
- TUI / replay-viewer match arms (no new `StackObjectKind` / `KeywordAbility`)

---

## Acceptance Map

| AC ID | Step(s) | Verifiable by |
|-------|---------|---------------|
| 3875 (TargetFilter 3 fields + `CombatState::is_blocking` helper) | Steps 1, 2 | grep field definitions + helper signature |
| 3876 (HASH 21→22, all canary sentinels bumped) | Steps 3, 7 | grep `HASH_SCHEMA_VERSION, 22u8` count == 17 (16 existing + 1 new A-1) |
| 3877 (V1-V4 + T1-T6 enforcement at 10 sites) | Steps 4, 5 | grep `passes_combat_role` count == 10; `passes_tapped` count == 10; `passes_untapped` count == 10 |
| 3878 (OR-semantics decision documented) | This plan + comments in casting.rs / abilities.rs | code review of `passes_combat_role` |
| 3879 (Eiganjo + sweep) | Step 6 + plan-time sweep | grep Eiganjo def — TODO gone; `is_attacking: true` and `is_blocking: true` present |
| 3880 (new test file w/ HASH-22 sentinel, PartialEq, serde-default, per-predicate +/-, OR-discriminator) | Step 8 | `cargo test --test primitive_pb_xa2 -- --nocapture` |
| 3881 (OOS-XA2-N seeds) | Step 9 | `pb-retriage-CC.md` diff |
| 3882 (gates + /review) | Step 10 | tooling output + reviewer verdict in `pb-review-XA2.md` |

---

## Risks & Edge Cases

1. **Graveyard arm semantics for `is_untapped`**: graveyard objects have
   `status.tapped = false` (default), so `is_untapped=true` will ACCEPT
   matching graveyard cards (vs. `is_tapped=true` which rejects all). This is
   a degenerate edge case. Mitigation: comment in V3/V4 + test H-1 locks in
   the behavior. Future audit (OOS-XA2-N) may revisit.
2. **Tuple-match exhaustiveness**: the four-way `match (is_attacking, is_blocking)`
   is exhaustive (4 of 4 cases enumerated). Clippy should not warn. Verify at
   gate time.
3. **Path A closure refactor (T1/T2)**: the inline `.find()` predicate
   expansion is the most error-prone diff. Worker MUST verify `filter`
   ownership semantics — the current code uses `filter` (borrowed in the
   pattern match arm) within the closure; the closure captures it by
   reference. Path A's `match (filter.is_attacking, filter.is_blocking)` and
   `state.objects.get(&obj.id)` continue to borrow correctly. No `.clone()`
   needed.
4. **`status.tapped` on stack objects / non-permanent zones**: `status.tapped`
   is meaningful only on the battlefield. The validate-site predicate reads
   `state.objects.get(&id).is_some_and(|o| o.status.tapped)` — for non-
   battlefield candidates, the field is default-`false`. For
   `TargetCreatureWithFilter` and `TargetPermanentWithFilter` the candidate
   MUST already be on the battlefield (the `on_battlefield` gate at line 5708
   rejects others), so the read is meaningful. For graveyard arms, see risk 1.
5. **Mental-toggle discrimination for G-1**: worker MUST verify by temporarily
   removing `passes_combat_role` enforcement at T4 and confirming G-1 fails
   with the expected error message. Skipping this verification was the H-XA-01
   failure mode that triggered the PB-XA fix-phase.
6. **HASH bump sentinel sweep completeness**: the 16-file list above was
   generated at plan time. Workers MUST re-run the grep
   (`grep -rn "HASH_SCHEMA_VERSION,\?\s\+\d\+u8" crates/engine/tests/`) at
   implement time and bump any sentinels added by interleaved work. Missing a
   sentinel produces a single isolated test failure that's trivial to debug
   but signals incomplete sweep discipline.
7. **OR-semantics divergence drift**: future authors may set BOTH
   `is_attacking` and `is_blocking` thinking they mean AND ("must be both
   attacking AND blocking"). Mitigation: comment on each field references the
   sibling field and the OR semantics; doc comment in OR-semantics decision
   section calls out the design choice.
8. **Replay-viewer / TUI exhaustive match risk**: PB-XA had zero TUI/replay
   match changes; PB-XA2 likewise adds no new `StackObjectKind` /
   `KeywordAbility` / `Effect` variants. Verify at gate time via
   `cargo build --workspace` (the canonical detector for missed exhaustive
   matches per `gotchas-infra.md`).

---

## Open Questions

1. **Worker should verify Eiganjo oracle text via MCP `lookup_card` at
   implement time.** Plan-time confirmation: oracle is "It deals 4 damage to
   target attacking or blocking creature" (Channel half). If the oracle is
   later corrected (errata), the plan adapts; no schema change expected.

2. **Helper extraction for the four-way tuple match**: should `passes_combat_role`
   be extracted into a free function `combat_role_pass(state, id, filter) -> bool`
   in `state/combat.rs` to deduplicate the 10-site repetition? Recommendation:
   YES, if the worker has spare time and the gate budget allows. Defer to
   OOS-XA2-5 if it threatens the green-gate budget.

3. **Should `is_tapped` / `is_untapped` be merged into a single
   `tap_state: Option<TapState>` enum** (variants `Tapped`, `Untapped`)? This
   eliminates the unreachable-filter edge case (`is_tapped + is_untapped` both
   true). Counter-argument: the current bool pair matches the `is_attacking`
   /`is_blocking` shape (no enum coupling between sibling pairs). The plan
   defers this question to a future OOS entry if it becomes a real authoring
   pain point.
