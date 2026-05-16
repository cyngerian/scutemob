# LKI-Completeness Audit + Implementation Plan — BASELINE-LKI-01

**Task**: scutemob-37 (LS-7) — Filtered death triggers do not match creatures whose
filter-relevant characteristic was granted by a continuous effect while on the battlefield.

**Generated**: 2026-05-15
**Issue source**: `docs/mtg-engine-low-issues-remediation.md` — BASELINE-LKI-01

---

## 1. Issue summary + CR basis

### Symptom

A creature is granted a subtype (e.g. "Zombie") by a continuous effect while on the
battlefield, then dies. A filtered death trigger that *should* match it — e.g. "whenever
a Zombie you control dies" — does **not** fire. Reproduced with two continuous-effect
shapes:

- `LayerModification::AddSubtypes` through `EffectFilter::SingleObject`
- `LayerModification::AddSubtypes` through `EffectFilter::AttachedCreature` (aura grant)

### Root cause (confirmed)

The `AnyCreatureDies` dispatch site at **`crates/engine/src/rules/abilities.rs:4357-4361`**
evaluates `triggering_creature_filter` against the **graveyard** object:

```rust
let dying_chars = crate::rules::layers::calculate_characteristics(state, dying_obj_id)
    .unwrap_or_else(|| dying_obj.characteristics.clone());
if !crate::effects::matches_filter(&dying_chars, creature_filter) { continue; }
```

`dying_obj_id` is `new_grave_id` — the **post-`move_object_to_zone`** object, which:

1. Has a **new ObjectId** (CR 400.7), so `EffectFilter::SingleObject(old_id)` no longer
   matches it (`SingleObject(id) => *id == object_id` at `layers.rs:542`).
2. Is in `ZoneId::Graveyard`, so every battlefield-gated filter
   (`AttachedCreature`, `AllCreatures`, `CreaturesYouControl`, …) returns `false`.
3. Its source continuous effect with `EffectDuration::WhileSourceOnBattlefield` is now
   inactive (`is_effect_active` → `obj.zone == ZoneId::Battlefield` false at
   `layers.rs:445`), and an aura source still attached has nothing on the battlefield
   to apply to.

So `calculate_characteristics` returns the **base/printed** subtypes (no granted
"Zombie"), the filter does not match, and the trigger does not fire.

The `.unwrap_or_else(|| dying_obj.characteristics.clone())` fallback is **unreachable** —
`calculate_characteristics` returns `Some(_)` for any valid object, and even if it did
fire, `dying_obj.characteristics` is *also* base/printed values (see §4).

This is confirmed in-code: `pbn_subtype_filtered_triggers.rs` Test 6
(`test_pbn_death_filter_pre_death_lki_color`, lines 405-417) documents the exact
limitation and labels it **ESCALATED to coordinator** — that escalation is this task.

### CR basis

- **CR 603.10a** — "Some zone-change triggers look back in time. These are
  leaves-the-battlefield abilities, abilities that trigger when a card leaves a
  graveyard, and abilities that trigger when an object … is put into a hand or library."
- **CR 603.10** — look-back triggers use "the existence of those abilities and the
  appearance of objects **immediately prior to the event**."
- **CR 613.1d** — Layer 4 (type-changing effects, including subtype) is part of the
  series of layers that determine an object's characteristics.

A filtered "dies" trigger ("whenever a Zombie you control dies") is a
leaves-the-battlefield ability. It must be evaluated against the dying creature's
**layer-resolved characteristics as they last existed on the battlefield** — including
any Layer-4 `AddSubtypes` continuous effect that was applying at that moment. The
engine currently re-derives characteristics in the graveyard, which is a CR divergence.

---

## 2. Filter enumeration — battlefield-gated filters in `layers.rs`

`effect_applies_to` (`crates/engine/src/rules/layers.rs:521`). Every filter variant that
either guards on `obj_zone == ZoneId::Battlefield` or is otherwise zone/identity-fragile
once the object has left the battlefield:

| File:line | Filter variant | Guard / fragility |
|-----------|----------------|-------------------|
| `layers.rs:542` | `SingleObject(id)` | **No explicit zone guard** — but `id` is the *battlefield* ObjectId; after `move_object_to_zone` the graveyard object has a new ID (CR 400.7) → never matches. Fragile by identity, not zone. |
| `layers.rs:544` | `AllCreatures` | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:547` | `AllLands` | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:550` | `AllNonbasicLands` | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:555` | `AllNonAuraEnchantments` / `AllEnchantments` | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:558` | `AllNonAuraEnchantments` (second clause) | `obj_zone == ZoneId::Battlefield` |
| `layers.rs:566` | `AllPermanents` | `obj_zone == ZoneId::Battlefield` |
| `layers.rs:569` | `ControlledBy(player)` | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:577` | `CreaturesControlledBy(player)` | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:595` | `AttachedCreature` | `if obj_zone != ZoneId::Battlefield { return false; }` — **cited in the issue.** |
| `layers.rs:617` | `AttachedLand` | `if obj_zone != ZoneId::Battlefield { return false; }` |
| `layers.rs:633` | `AttachedPermanent` | `if obj_zone != ZoneId::Battlefield { return false; }` |
| `layers.rs:650,664,681,701,717,735` | `CreaturesYouControl` and other creature-scoped filters | `if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(Creature) { … }` |
| `layers.rs:755` | artifact-scoped filter | `obj_zone != ZoneId::Battlefield || !…Artifact` |
| `layers.rs:769,786,803,823,840,872,909,948,966` | further creature-scoped filters | `obj_zone != ZoneId::Battlefield || !…Creature` |
| `layers.rs:864,931` | additional battlefield-scoped filters | `obj_zone == ZoneId::Battlefield && …` |
| `layers.rs:531` | (pre-match phasing guard) | `if obj_zone == ZoneId::Battlefield { … phased_out … }` — not a bug source, noted for completeness |

**Conclusion**: the ONLY filter variants that *could* keep applying to a graveyard
object are `SingleObject`, `AllCardsInGraveyards` (`layers.rs:567`), and `Source` /
`DeclaredTarget` (both resolve to `false` if unresolved). `SingleObject` fails by
**identity** (new ObjectId), not by zone guard. Every battlefield-scope filter fails by
**zone guard**. **No "fix the guards" approach is viable** — these guards are correct CR
behavior for *live* layer application (CR 613 applies continuous effects to objects in
the zone the effect cares about). The fix must capture characteristics **before** the
zone change, not patch the guards.

---

## 3. Dispatch-site enumeration — LKI reads via `calculate_characteristics` on
objects that just left the battlefield

The question for each site: does it apply a filter (`triggering_creature_filter` /
`matches_filter`) against the *characteristics* of an object that has already been moved
to a non-battlefield zone?

| File:line | Dispatch | Filter applied to dying object's chars? | Affected by bug? |
|-----------|----------|------------------------------------------|------------------|
| `abilities.rs:4357` | `AnyCreatureDies` → `triggering_creature_filter` via `matches_filter(&dying_chars, …)` | **YES** — `dying_chars` from `calculate_characteristics(new_grave_id)` | **YES — primary bug site.** |
| `abilities.rs:4044-4099` | `SelfDies` (dying object's own `triggered_abilities`) | No — iterates `obj.characteristics.triggered_abilities`; trigger source *is* the dying object, no creature-type filter applied to it | No (but see note A) |
| `abilities.rs:4102-4133` | `SelfLeavesBattlefield` (dying object) | No — same as `SelfDies` | No (note A) |
| `abilities.rs:5312-5333` | `SelfLeavesBattlefield` on `ObjectExiled` (exile object) | No — own triggered abilities | No (note A) |
| `abilities.rs:5368-5389` | `SelfLeavesBattlefield` on second exile path | No | No |
| `abilities.rs:5424-5445` | `SelfLeavesBattlefield` on `ObjectReturnedToHand` (bounce) | No | No |
| `abilities.rs:5892-5912` | `SelfLeavesBattlefield` on `PermanentSacrificed` | No | No |
| `abilities.rs:4419-4427` | `AnyCreatureDies` / `SelfLeavesBattlefield` aura-fell-off path (`AuraFellOff`) | Check: does the aura-fell-off arm filter by dying creature subtype? It iterates aura's own triggered abilities → No | No |
| `abilities.rs:6102-6105` | `AnyCreatureYouControlAttacks` → `triggering_creature_filter` | Object is still on the battlefield (attacker) — `calculate_characteristics` returns correct layer-resolved chars | **No** (correct) |
| `abilities.rs:4272` (AnyCreatureDies preamble, `resolved_chars` at `:4302`) | reads the **watcher's** chars (battlefield object) to find `AnyCreatureDies` trigger defs | Watcher is on battlefield | No (correct) |

**Note A — latent identical bug not in scope but recommended fix-along:** the `SelfDies`
/ `SelfLeavesBattlefield` sites read `obj.characteristics.triggered_abilities` on the
graveyard object. If a creature were *granted a triggered ability* by a Layer-6
continuous effect while on the battlefield (e.g. "enchanted creature has 'when this dies,
…'"), that granted trigger would be **lost** the same way — `move_object_to_zone` copies
only base `characteristics`, and these sites never call `calculate_characteristics` at
all. This is the same LKI gap for a different characteristic (granted abilities vs.
granted subtypes). It is **out of strict scope** for BASELINE-LKI-01 (acceptance
criterion 4043 is subtype-specific) but the recommended fix (§5) makes the snapshot
available to these sites too, so they *can* be corrected in the same change at low extra
cost. See §6 Step 7 (optional).

**Conclusion**: exactly **one** dispatch site is in-scope and affected:
`abilities.rs:4357` (`AnyCreatureDies` filtered death trigger). LTB self-triggers are
structurally immune to the *subtype-filter* form of the bug because they never filter
the dying creature by characteristic.

---

## 4. Characteristics-snapshot investigation

### Does `obj.characteristics` ever hold layer-applied values?

**No.** The layer system is **non-destructive**: `calculate_characteristics`
(`layers.rs:32`) does `let mut chars = obj.characteristics.clone();` and applies layers
to the *clone*, returning it. It **never writes back** to `obj.characteristics`. A grep
for `.characteristics =` in `layers.rs` finds zero assignment of layer output to a
GameObject. Therefore `obj.characteristics` is always **base/printed** values (plus
whatever the builder/copy code set as the copiable base).

### What does `move_object_to_zone` do with characteristics?

`crates/engine/src/state/mod.rs:382` — `move_object_to_zone`:

```rust
characteristics: old_object.characteristics.clone(),   // line 415
```

It copies the **base** characteristics of the old object. Layer effects are *not* baked
in. So the graveyard object's `characteristics` field contains base subtypes only.

### Is fix candidate (a) viable?

**Fix (a) — "dispatch reads `dying_obj.characteristics.clone()` directly and skips
`calculate_characteristics`" — is NOT viable.** `dying_obj.characteristics` on the
graveyard object is base/printed values. It does **not** contain the granted "Zombie"
subtype. Reading it directly produces the same wrong answer as today (it would only
*accidentally* work for subtypes that were in the printed type line, which is what
Test 6's weakened version exploits). Fix (a) as literally described in the issue cannot
solve the continuous-effect case.

### Is there an existing LKI snapshot mechanism?

**Yes — and it is the correct model.** The engine already snapshots specific
characteristics at death time and threads them through the trigger:

- `GameEvent::CreatureDied.pre_death_counters` (`events.rs:221`) — counter map captured
  **before** `move_object_to_zone` (`sba.rs:540-551`).
- `GameEvent::CreatureDied.pre_death_power` (`events.rs:229`) — **layer-resolved** power
  captured via `calculate_characteristics(state, id)` **while the object is still on the
  battlefield** (`sba.rs:548`).
- These are read at the dispatch site (`abilities.rs:4094-4097`) and stored into
  `PendingTrigger.lki_counters` / `lki_power`, then threaded into `EffectContext`
  (`abilities.rs:7678`) and consumed by `EffectAmount::CounterCountAtLastKnownInformation`
  / `SourcePowerAtLastKnownInformation` (`effects/mod.rs:6780`).

`sba.rs:548` proves the capture point: `calculate_characteristics` called on `id` (the
**battlefield** ObjectId, before the move) returns the fully layer-resolved
characteristics — including Layer-4 `AddSubtypes`. This is exactly the data the death
filter needs.

**The infrastructure for fix (b) already exists in shape** — we extend the same
`pre_death_*` snapshot pattern to carry subtypes/colors/card-types (or the whole
`Characteristics`).

---

## 5. Recommended fix — variant (b), implemented as a pre-death `Characteristics` snapshot

**Recommendation: fix candidate (b)** — but with a precise interpretation. The issue
phrases (b) as "teach `calculate_characteristics` to honor preserved chars for
non-battlefield zones." Do **not** modify `calculate_characteristics` itself — that
function correctly models live layer application and is called from ~60 sites; making it
zone-aware would risk every one of them. Instead, implement (b) as:

> **Capture a layer-resolved `Characteristics` snapshot of the dying creature at death
> time (before `move_object_to_zone`), carry it on `GameEvent::CreatureDied`, and have
> the `AnyCreatureDies` dispatch evaluate `triggering_creature_filter` against the
> snapshot instead of re-deriving from the graveyard object.**

This is the same architecture as `pre_death_power` — already reviewed, already hashed,
already CR-cited. It is the lowest-risk way to satisfy CR 603.10a.

### Justification

- **Affected dispatch sites**: exactly one (`abilities.rs:4357`). Blast radius is tiny.
- **`characteristics` does not hold layer values** → fix (a) is impossible (§4).
- **`calculate_characteristics` must not change** → it is correct for live application
  and has ~60 callers; a zone-aware branch there is a far larger blast radius and would
  need every caller audited.
- **Precedent**: `pre_death_power` solved the *identical* shape of problem (layer-
  resolved value needed after the object left the battlefield) by snapshotting at
  `sba.rs:548`. Following the precedent keeps the codebase consistent and the review
  cheap.
- **Other zone-change triggers (LTB / exile / bounce)**: structurally immune to the
  *subtype-filter* form because they never filter the dying creature by characteristic
  (§3). They share a *latent* gap for granted *abilities* (Note A) — the snapshot makes
  fixing that trivial later, but it is out of scope here.

### Snapshot scope decision

Capture the **full layer-resolved `Characteristics`** (the value
`calculate_characteristics(state, id)` already returns at `sba.rs:548`), not just
subtypes. Rationale:

- `calculate_characteristics` is **already being called** at `sba.rs:548` to get
  `pre_death_power` — capturing its full result costs nothing extra.
- `triggering_creature_filter` is a `TargetFilter` that can filter on subtype, color,
  **and** card type (`has_subtype`, `colors`, plus type predicates). A subtype-only
  snapshot would leave color/type-granted filters still broken. The full snapshot closes
  all three at once and matches `matches_filter`'s input type exactly.
- It supersedes `pre_death_power` (power lives inside `Characteristics`) — but to keep
  the change minimally invasive and avoid touching the ~30 `CreatureDied` construction
  sites' existing `pre_death_power` field, **keep `pre_death_power` as-is** and add the
  new field alongside it. (A later cleanup can fold `pre_death_power` into the snapshot.)

---

## 6. Step-by-step implementation plan

### Step 1 — Add `pre_death_characteristics` to `GameEvent::CreatureDied`

**File**: `crates/engine/src/rules/events.rs` (variant at `:207`)

Add a field after `pre_death_power`:

```rust
/// CR 603.10a / CR 613.1: layer-resolved characteristics of the dying creature
/// as they last existed on the battlefield (last known information). Captured by
/// `calculate_characteristics` BEFORE `move_object_to_zone` (e.g. sba.rs:548).
/// Used by the `AnyCreatureDies` dispatch to evaluate `triggering_creature_filter`
/// against the creature's pre-death subtypes/colors/types — including any subtype
/// granted by a continuous effect (AddSubtypes via SingleObject/AttachedCreature)
/// that no longer applies once the object is in the graveyard.
/// `None` when the snapshot is unavailable (defensive; dispatch falls back to the
/// graveyard object's base characteristics, preserving today's behavior).
#[serde(default)]
pub pre_death_characteristics: Option<Characteristics>,
```

Ensure `Characteristics` is imported in `events.rs` (it is in
`crate::state::game_object` — add `use` if absent).

### Step 2 — Capture the snapshot at every `CreatureDied` construction site

There are ~30 `GameEvent::CreatureDied { … }` literals across:
`sba.rs` (2), `turn_actions.rs` (5), `engine.rs` (4), `resolution.rs` (~14),
`effects/mod.rs` (~8), `abilities.rs` (2: `:648`, `:809`), `mana.rs` (1),
`casting.rs` (1), `replacement.rs` (1).

For each site, the dying object's battlefield ObjectId (the value used as `object_id`)
is in scope **before** the `move_object_to_zone` call. The capture is:

```rust
let pre_death_chars = crate::rules::layers::calculate_characteristics(state, id);
```

…taken at the **same point** the site already captures `pre_death_power` (most sites
capture power immediately before the move). Add `pre_death_characteristics: pre_death_chars`
to the event literal.

- **`sba.rs:540-554`** is the model: it already calls `calculate_characteristics(state, id)`
  for `pre_death_power`. Change that block to keep the **whole** result:
  ```rust
  let pre_death_chars = crate::rules::layers::calculate_characteristics(state, id);
  let lki_power = pre_death_chars.as_ref().and_then(|c| c.power)
      .or(obj.characteristics.power);
  ```
  and add `pre_death_characteristics: pre_death_chars.clone()` to both `CreatureDied`
  literals in that function (`:576`, `:617`). Use `.clone()` because the block builds
  two events.
- For sites that do **not** already snapshot power (some `resolution.rs` / `effects`
  sites pass `pre_death_power: None`), it is acceptable to also pass
  `pre_death_characteristics: None` — these are paths where the object may already be
  mid-transition. **However**, prefer capturing where the battlefield object is still
  live: grep each site; if `state.objects.get(&id)` still returns a battlefield object
  at that point, capture the snapshot. The runner should inspect each of the ~30 sites
  and capture wherever the object is still on the battlefield, falling back to `None`
  only where it genuinely is not.
- The `abilities.rs:648` and `:809` sites (`destroy_*` / animated-permanent death):
  capture before their `move_object_to_zone`.

**Compile-driver tactic**: make the field non-`#[serde(default)]`-only by leaving it a
plain field; `cargo check` will then enumerate every construction site that needs the
field — work through the errors. (The `#[serde(default)]` attr only affects
deserialization, not struct literal exhaustiveness, so the compiler still lists every
site.)

### Step 3 — Thread the snapshot into the `AnyCreatureDies` dispatch

**File**: `crates/engine/src/rules/abilities.rs`

In the `GameEvent::CreatureDied { … }` match arm (`:4031`), add
`pre_death_characteristics` to the destructured fields (`:4035` area).

At the filter site (`:4335-4364`), replace the `calculate_characteristics` call:

```rust
if let Some(ref creature_filter) = trigger_def.triggering_creature_filter {
    let dying_obj = match state.objects.get(&dying_obj_id) {
        Some(o) => o,
        None => continue,
    };
    if creature_filter.is_token && !dying_is_token { continue; }
    // CR 603.10a / CR 613.1: evaluate the filter against the dying creature's
    // layer-resolved PRE-DEATH characteristics (captured before move_object_to_zone).
    // This honors subtypes/colors/types granted by a continuous effect that no
    // longer applies once the object is in the graveyard. Falls back to the
    // graveyard object's base characteristics if no snapshot was captured.
    let dying_chars = pre_death_characteristics
        .clone()
        .unwrap_or_else(|| dying_obj.characteristics.clone());
    if !crate::effects::matches_filter(&dying_chars, creature_filter) {
        continue;
    }
}
```

Delete the obsolete `TODO(BASELINE-LKI-01)` comment block (`:4346-4356`).

### Step 4 — Update the hash

**File**: `crates/engine/src/state/hash.rs` — `GameEvent::CreatureDied` arm (`:3504`)

Add `pre_death_characteristics` to the destructure and hash it:

```rust
GameEvent::CreatureDied {
    object_id, new_grave_id, controller,
    pre_death_counters, pre_death_power, pre_death_characteristics,
} => {
    27u8.hash_into(hasher);
    object_id.hash_into(hasher);
    new_grave_id.hash_into(hasher);
    controller.hash_into(hasher);
    for (ct, count) in pre_death_counters.iter() { ct.hash_into(hasher); count.hash_into(hasher); }
    pre_death_power.hash_into(hasher);
    // CR 603.10a / CR 613.1: LKI characteristics snapshot for filtered death triggers.
    pre_death_characteristics.hash_into(hasher);
}
```

`Characteristics` must implement `HashInto` (or `Option<Characteristics>` must).
**Verify**: grep `impl HashInto for Characteristics` in `hash.rs`. If it exists, done.
If not, add an `impl HashInto for Characteristics` that hashes its fields in a
deterministic order (name, mana_cost, card_types, subtypes, supertypes, colors,
keywords, power, toughness, and the `triggered_abilities`/`activated_abilities` if those
are hashed elsewhere). Mirror an existing struct `HashInto` impl. **Bump
`HASH_SCHEMA_VERSION` 26 → 27** in `state/hash.rs` and record the reason in the version
history comment block.

### Step 5 — Update the `HASH_SCHEMA_VERSION` assertion in the PB-N test

**File**: `crates/engine/tests/pbn_subtype_filtered_triggers.rs:557-560`

Update the assertion to `27u8` and add a history line:
`// BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (CreatureDied.pre_death_characteristics, CR 603.10a LKI subtype snapshot).`

### Step 6 — Strengthen / unblock PB-N Test 6

**File**: `crates/engine/tests/pbn_subtype_filtered_triggers.rs`

Test 6 (`test_pbn_death_filter_pre_death_lki_color`, `:418`) currently uses *base*-
characteristic Vampire and documents the ESCALATED limitation. Once the fix lands, the
new regression tests (Step §7) supersede it. Update its doc comment to remove the
"ESCALATED" / "engine limitation" paragraph (`:405-417`) and note that
BASELINE-LKI-01 closed the gap; the test itself can remain as a base-subtype sanity
check.

### Step 7 — (Optional, low-cost) propagate snapshot to `SelfDies` granted-ability gap

Out of strict scope (Note A). If time permits: `SelfDies` / `SelfLeavesBattlefield`
sites that iterate `obj.characteristics.triggered_abilities` on the graveyard object
could instead iterate `pre_death_characteristics.triggered_abilities` so that
continuous-effect-*granted* triggered abilities survive the zone change. **Recommend
deferring** — it widens the change and has no acceptance criterion here. Note it in the
handoff as a follow-up LOW.

---

## 7. Regression test plan

**File**: `crates/engine/tests/pbn_subtype_filtered_triggers.rs` (append; reuses the
file's existing helpers — `pass_all`, `stack_trigger_count`, `library_card`,
`death_trigger_draw_subtype`).

Acceptance criterion 4043 requires **two** tests, one per continuous-effect shape. Both
follow the Test 4 pattern (0-toughness creature dies via SBA, assert
`stack_trigger_count > 0`).

### Test A — `test_lki_death_filter_subtype_granted_via_single_object`

CR 603.10a / 613.1d. Setup:

- Watcher: `ObjectSpec::creature(p1, "Zombie Watcher", 1, 4)` with
  `death_trigger_draw_subtype("Zombie")`.
- Dying creature: `ObjectSpec::creature(p1, "Granted Zombie", 1, 0)` with **base
  subtype `Human`** (explicitly NOT Zombie in printed types) — toughness 0 so it dies
  via SBA.
- A continuous effect granting Zombie via `SingleObject`: build it the way Test sites
  in `effects/mod.rs:5054`/`abilities.rs` build `ApplyContinuousEffect` — a
  `CardContinuousEffectDef { layer: EffectLayer::TypeChange, modification:
  LayerModification::AddSubtypes(vec![SubType("Zombie")]), filter:
  EffectFilter::SingleObject(<dying_id>), duration: EffectDuration::Indefinite,
  condition: None }`. Inject it onto the dying creature.
  - The cleanest harness path: give a **second** permanent ("Zombie Lord") an
    ETB/static that applies the effect, OR push the `ContinuousEffect` directly into
    `state.continuous_effects` via the builder if a builder hook exists. Check
    `GameStateBuilder` for a `with_continuous_effect` / `continuous_effect` method;
    `layer_correctness.rs` tests construct continuous effects directly — **copy that
    file's pattern** (grep `continuous_effects` in `tests/layer_correctness.rs`).
  - Use `EffectDuration::Indefinite` so the effect is active while the creature is on
    the battlefield (`is_effect_active` returns true for `Indefinite`).
- Pre-condition assert: `calculate_characteristics` on the live battlefield creature
  shows the Zombie subtype (sanity check the effect is wired).
- Act: `pass_all(state, &[p1, p2])` → SBA kills the creature.
- Assert: creature left the battlefield AND `stack_trigger_count(&state) > 0` — the
  Zombie-filtered death trigger fired because the pre-death snapshot carried the granted
  subtype. **Pre-fix this fails (0 triggers); post-fix it passes.**

### Test B — `test_lki_death_filter_subtype_granted_via_aura`

CR 603.10a / 613.1d / 702.6 (aura grant). Setup:

- Watcher: same Zombie-filtered death trigger watcher.
- Dying creature: base subtype `Human`, toughness 0.
- An **Aura** attached to the dying creature whose static ability is a
  `WhileSourceOnBattlefield` continuous effect with `filter: EffectFilter::AttachedCreature`,
  `layer: EffectLayer::TypeChange`, `modification: AddSubtypes(["Zombie"])`. The Aura's
  `source` is set so `effect.source` resolves and `attached_to` points at the dying
  creature. Model on existing aura-grant tests — grep `AttachedCreature` in
  `tests/` (and `layer_correctness.rs`) for the construction pattern; if none, build the
  Aura `ObjectSpec` attached via the builder and push the `ContinuousEffect` with
  `source: Some(aura_id)`.
- Pre-condition assert: live creature shows Zombie via `calculate_characteristics`.
- Act: `pass_all` → SBA kills the enchanted creature.
- Assert: `stack_trigger_count(&state) > 0`. **Pre-fix fails; post-fix passes.**

### Negative guard (recommended, not required by 4043)

`test_lki_death_filter_no_false_positive_after_grant_removed` — grant Zombie via a
`WhileSourceOnBattlefield` effect whose **source is removed before** the creature dies,
so the creature is plain Human at death; assert the Zombie filter does **not** fire
(`stack_trigger_count == 0`). Confirms the snapshot captures the *actual* pre-death
state, not a stale grant.

### Verification checklist

- [ ] `cargo check -p mtg-engine` — all ~30 `CreatureDied` sites updated
- [ ] `cargo test -p mtg-engine --test pbn_subtype_filtered_triggers` — Tests A, B,
      negative guard pass; existing Tests 1-9 still pass
- [ ] `cargo test --all` — full suite green (HASH bump may shift hash-dependent
      golden tests; update any `HASH_SCHEMA_VERSION` literals — grep for `26u8` /
      `HASH_SCHEMA_VERSION` across `crates/` and `test-data/`)
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo build --workspace` — replay-viewer / TUI compile (the `CreatureDied`
      field addition may surface in `view_model.rs` if it matches `GameEvent` — grep
      `CreatureDied` in `tools/`)
- [ ] PB-N Test 6 doc comment de-escalated; TODO(BASELINE-LKI-01) block deleted from
      `abilities.rs`
- [ ] BASELINE-LKI-01 marked resolved in `docs/mtg-engine-low-issues-remediation.md`

---

## 8. Risks & edge cases

- **~30 construction sites** — the bulk of the work and the main compile-error churn.
  Use the compiler as a checklist (Step 2). Risk: passing `None` too liberally; the
  runner must check each site for whether the battlefield object is still live and
  capture the real snapshot there.
- **Hash schema bump** — `HASH_SCHEMA_VERSION` 26→27 will change state fingerprints;
  any golden/replay test that hard-codes a hash or the sentinel must be updated. Grep
  `26u8` and `HASH_SCHEMA_VERSION` workspace-wide.
- **`Characteristics` `HashInto`** — if no impl exists, the new one must be
  deterministic (sorted `OrdSet`/`OrdMap` iteration — `im` collections already iterate
  in sorted order, so this is safe).
- **`Option<Characteristics>` size on `GameEvent`** — `Characteristics` is a moderately
  large struct; `CreatureDied` already carries an `OrdMap`. Boxing is unnecessary
  (`GameEvent` is not in a hot per-frame path). Leave unboxed for simplicity unless a
  clippy `large_enum_variant` lint fires — if it does, `Box<Characteristics>`.
- **Latent granted-ability LKI gap (Note A)** — explicitly out of scope; record as a
  follow-up LOW so it is not silently forgotten.
- **`pre_death_power` redundancy** — the new snapshot contains power; `pre_death_power`
  becomes derivable. Do **not** remove `pre_death_power` in this task (avoids touching
  its consumers); note the redundancy as a future cleanup.
- **Other zone-change triggers (exile/bounce/sacrifice)** — verified immune to the
  subtype-filter form (§3); no `ObjectExiled` / `ObjectReturnedToHand` /
  `PermanentSacrificed` changes needed for criterion 4043.
