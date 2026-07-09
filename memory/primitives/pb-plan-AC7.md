# Primitive Batch Plan: PB-AC7 — Type-changing & Ability-removal

**Generated**: 2026-07-09
**Primitive(s)**: See "Scope reframing" — the brief lists 4 primitives; research shows
only **1.5 are genuinely new engine capability**. The other 2.5 are already expressible.
**CR Rules**: 613.1 (layer list), 205.1a/205.1b (setting card types & subtypes),
613.6/613.7 (timestamps), 613.8 (dependency), 611.2a/c/e (continuous effects from
resolved spells/abilities), 708.2 (face-down).
**Cards affected**: 5 fully-CLEAN + 2 PARTIAL-clause improvements (see roster). Advisory
brief said ~14; discounted real yield is 5 fully-clean.
**Dependencies**: PB-AC2 (`Effect::MayPayThenEffect` — needed by Leaf-Crowned Visionary),
PB-AC3 (SetBothDynamic / Layer-4 `TypeChange` placement precedent). Both present.
**Deferred items from prior PBs**: none carried in; this PB *creates* 4 OOS-AC7 seeds.

---

## TODO SWEEP RESULT (roster-recall gate — MANDATORY)

Ran `Grep` over `crates/engine/src/cards/defs/` for TODO/ENGINE-BLOCKED markers naming
lose-abilities / becomes-type / type-change / creature-type / Aura-Equipment-Vehicle-filter
patterns. **Findings that are forced adds (self-identified as needing this PB):**

- `sram_senior_edificer.rs` — ENGINE-BLOCKED naming the exact gap: "`spell_type_filter`
  accepts `Vec<CardType>` only. There is no spell-subtype filter." → **forced add** (primitive #4).
- `leaf_crowned_visionary.rs` — ENGINE-BLOCKED comment literally says *"Genuine remaining
  gap (PB-AC7 territory per pb-plan-AC2.md)"*. → **forced add** (primitive #4).
- `eaten_by_piranhas.rs`, `kenriths_transformation.rs`, `darksteel_mutation.rs`,
  `final_showdown.rs`, `vraska_betrayals_sting.rs` — TODOs naming LoseAbilities / type
  override. Research shows these are **stale** (see next section) — expressible with
  *existing* primitives. → forced adds to the *backfill* list, not to the engine-change list.
- `frodo_saurons_bane.rs` — needs `SetCreatureTypes` but ALSO several other unbuilt
  primitives → remains PARTIAL (see roster).

Gate assertion: TODO sweep run, **7 marker cards found**, all classified below. Not 0.

---

## Scope reframing (READ FIRST — the single most important finding)

The brief lists four primitives. Grounding each against the *actual engine* (not the
advisory brief) yields:

| # | Brief primitive | Verdict | Reason |
|---|-----------------|---------|--------|
| 1 | `Effect::LoseAbilities` (Layer 6 ability removal) | **ALREADY EXPRESSIBLE — do NOT add** | `LayerModification::RemoveAllAbilities` (Layer 6) exists (`continuous_effect.rs:311`, applied `layers.rs:1093`). One-shot "loses all abilities until EOT" = `Effect::ApplyContinuousEffect { layer: Ability, modification: RemoveAllAbilities, filter: <AllCreatures/DeclaredTarget/AttachedCreature>, duration: UntilEndOfTurn }`. Static Aura form uses the same modification via `AbilityDefinition::Static`. Oko (`oko_thief_of_crowns.rs:35`) already ships this exact composition. |
| 2 | `Effect::SetCreatureTypes` (Layer 4 set-creature-types) | **GENUINELY NEW — add as `LayerModification::SetCreatureTypes`** (not an `Effect` variant) | No existing modification replaces creature subtypes while preserving card types / supertypes / non-creature subtypes. `SetTypeLine` wipes all three sets; `AddSubtypes` only adds; `LoseAllSubtypes` clears all. **But 0 fully-clean roster cards need it** (all roster type-changers are full "becomes X creature" resets that `SetTypeLine` already handles). Frodo needs it but is blocked on other primitives. See "SetCreatureTypes rationale". |
| 3 | One-shot Layer-4 type override with duration | **ALREADY EXPRESSIBLE — do NOT add an `Effect` variant** | `Effect::ApplyContinuousEffect { effect_def: ContinuousEffectDef { layer: TypeChange, modification: <SetTypeLine/AddCardTypes/SetCreatureTypes>, duration: <UntilEndOfTurn/Indefinite>, filter: DeclaredTarget{..} } }` already registers a Layer-4 continuous effect with any duration. Proven by `cyber_conversion.rs` (AddCardTypes + UntilEndOfTurn) and `oko_thief_of_crowns.rs` (SetTypeLine + Indefinite). The `ApplyContinuousEffect` handler (`effects/mod.rs:2792`) is fully generic over layer/duration. |
| 4 | Aura/Equipment/Vehicle subtype filter (TargetFilter + EffectFilter) | **PARTIALLY EXISTS; the real gap is a spell-cast TRIGGER filter** | `TargetFilter.has_subtype/has_subtypes/exclude_subtypes` already match `chars.subtypes` (`effects/mod.rs:7654-7727`), and `SubType` is a `String` newtype, so "target Aura/Equipment/Vehicle" already works. The genuine blocker (Sram, Leaf-Crowned) is that the **cast trigger** `TriggerCondition::WheneverYouCastSpell` filters only by `Option<Vec<CardType>>`, not subtypes. **Add `spell_subtype_filter: Option<Vec<SubType>>` to that variant.** |

**Net engine work for PB-AC7:**
1. `LayerModification::SetCreatureTypes(OrdSet<SubType>)` — Layer 4 (brief-mandated; low yield).
2. (RECOMMENDED companion) `LayerModification::SetCardTypes(OrdSet<CardType>)` — Layer 4;
   makes #1 load-bearing and lets the roster Auras be authored CR-faithfully (preserving
   `Legendary`). Optional; see rationale.
3. `spell_subtype_filter: Option<Vec<SubType>>` field on `TriggerCondition::WheneverYouCastSpell`.
4. HASH_SCHEMA_VERSION bump 33 → 34.

**Do NOT add** `Effect::LoseAbilities` or any new `Effect` variant for one-shot type
override. If the runner is tempted, stop — these are redundant per the analysis above and
per `memory/gotchas-infra.md` ("Adding a new field/variant should be a last resort").

---

## CR Rule Text (verified via mtg-rules MCP — quoted)

**613.1** — layer order. Quoting the relevant layers:
> 613.1d Layer 4: Type-changing effects are applied. These include effects that change an
> object's card type, subtype, and/or supertype.
> 613.1f Layer 6: Ability-adding effects, keyword counters, ability-removing effects, and
> effects that say an object can't have an ability are applied.

Note: the layer *list* is 1–7 (7 layers), not the "eight layers" the code doc-comment
mentions. The `EffectLayer` enum splits Layer 7 into 7a/7b/7c/7d — consistent with CR
613.3/613.4 sublayers. No action needed; noted for accuracy.

**205.1a** (the governing rule for BOTH new modifications):
> Some effects set an object's card type. In most such cases, the new card type(s) replaces
> any existing card types. ... Similarly, when an effect sets one or more of an object's
> subtypes, the new subtype(s) replaces any existing subtypes **from the appropriate set**
> (creature types, land types, artifact types, enchantment types, planeswalker types, or
> spell types). If an object's card type is removed, the subtypes correlated with that card
> type will remain if they are also the subtypes of a card type the object currently has;
> otherwise, they are also removed ... Removing an object's subtype doesn't affect its card
> types at all.

→ This is the CR basis for `SetCreatureTypes`: setting creature subtypes replaces ONLY the
creature-type set, leaving land/artifact/enchantment subtypes and all card types intact.

**205.1b** (the "[creature type] artifact creature" case — Darksteel Mutation family):
> Some effects state that an object becomes a "[creature type or types] artifact creature";
> these effects also allow the object to retain all of its prior card types and subtypes
> other than creature types, but replace any existing creature types.

→ Confirms Darksteel Mutation's Insect replaces prior creature types; Artifact/Creature are
ADDED. (Darksteel Mutation's own text further says "loses all other card types," overriding
the "retain prior card types" default — so its card_types end up exactly {Artifact, Creature}.)

**613.6** — an effect that spans layers keeps applying in each even if its source ability is
removed. (Relevant: RemoveAllAbilities doesn't remove *its own* continuous effect.)

**613.7 / 613.7a / 613.7b / 613.7d** — timestamp ordering. Static-ability effects share the
object's timestamp (613.7a); resolved-spell/ability effects get a timestamp at creation
(613.7b); an object gets a timestamp on entering a zone (613.7d). **613.7 is the basis for
the granted-then-removed ordering test.**

**613.8 / 613.8a / 613.8b** — dependency overrides timestamp; A depends on B if applying B
changes what A does/applies to; loops fall back to timestamp order. **Relevant to whether
`SetCreatureTypes` should depend on `AddSubtypes` — see Risks.**

**611.2a/c/e** — continuous effect from a resolved spell/ability lasts as stated (611.2a);
the set of affected objects is fixed when the effect begins (611.2c); "becomes/gains" applies
*after* the permanent is on the battlefield, "is [characteristic]" applies simultaneously
with ETB (611.2e). Not blocking here (all roster type-changers target existing permanents).

**708.2** — face-down objects have only the characteristics granted by the enabling ability;
these are the copiable values. The face-down 2/2-no-text override in `layers.rs:216-237` runs
BEFORE the layer loop, so a subsequent Layer-6 `RemoveAllAbilities` composes correctly (it
removes from an already-empty ability set — see the LoseAbilities-vs-face-down test).

---

## Engine Changes

### Change 1 — `LayerModification::SetCreatureTypes(OrdSet<SubType>)` (primitive #2)

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add a new variant in the "Layer 4: Type-changing" group, after
`AddAllCreatureTypes` (line ~296):
```rust
/// Layer 4 (CR 205.1a): SETS the object's creature subtypes to exactly this set,
/// removing all prior creature types but PRESERVING card types, supertypes, and
/// non-creature subtypes (land/artifact/enchantment/planeswalker/spell types).
/// Distinct from `AddAllCreatureTypes`/`AddSubtypes` (additive) and `SetTypeLine`
/// (replaces card types & supertypes too). Used by "becomes a [creature type]"
/// effects that keep the object's other types (Frodo; CR-faithful Kenrith/Eaten-by-Piranhas).
SetCreatureTypes(OrdSet<SubType>),
```

**File**: `crates/engine/src/rules/layers.rs` — `apply_layer_modification` (line ~1000)
**Action**: Add an arm in the Layer-4 block (after `AddAllCreatureTypes`, ~line 1071). Reuse
the creature-subtype filter idiom already used for Reconfigure (`layers.rs:308-313`):
```rust
LayerModification::SetCreatureTypes(new_types) => {
    // CR 205.1a: replace only the creature-type subtypes; keep all non-creature subtypes.
    let mut kept: OrdSet<SubType> = chars
        .subtypes
        .iter()
        .filter(|st| !crate::state::types::ALL_CREATURE_TYPES.contains(*st))
        .cloned()
        .collect();
    for s in new_types {
        kept.insert(s.clone());
    }
    chars.subtypes = kept;
}
```

**File**: `crates/engine/src/state/hash.rs` — `impl HashInto for LayerModification` (~1614)
**Action**: Add arm with **discriminant 30** (next free — 0..28 + 29 used):
```rust
LayerModification::SetCreatureTypes(subtypes) => {
    30u8.hash_into(hasher);
    subtypes.hash_into(hasher);
}
```

**File**: `crates/engine/src/rules/layers.rs` — `depends_on` (~1328)
**Action**: See Risks — evaluate whether `SetCreatureTypes` should depend on `AddSubtypes`
(so an "each land is a Swamp"-style add applies before a creature-type SET within Layer 4).
Default: leave as `_ => false` (pure timestamp order). Only add a dependency arm if a test
demonstrates a wrong ordering. Document the decision either way.

### Change 2 (RECOMMENDED, optional) — `LayerModification::SetCardTypes(OrdSet<CardType>)`

**Rationale**: lets the roster Auras (Kenrith, Eaten by Piranhas, Darksteel Mutation) be
authored CR-faithfully — `SetCardTypes({Creature})` + `SetCreatureTypes({Elk})` preserves the
target's `Legendary` supertype, which the current Oko-style `SetTypeLine{supertypes: empty}`
silently drops. This is the ONLY way to give `SetCreatureTypes` load-bearing roster use this
batch. If the runner/reviewer prefer minimal scope, skip this and author the Auras with
`SetTypeLine` (accepted Oko precedent) — then `SetCreatureTypes` is unit-test-only.

**File**: `continuous_effect.rs` — add `SetCardTypes(OrdSet<CardType>)` (Layer 4). Applies:
`chars.card_types = new.clone();` (supertypes & subtypes untouched).
**File**: `layers.rs` `apply_layer_modification` — add arm.
**File**: `hash.rs` — discriminant **31**.

### Change 3 — `spell_subtype_filter` on `WheneverYouCastSpell` (primitive #4, the real gap)

**File**: `crates/engine/src/cards/card_definition.rs` — `TriggerCondition::WheneverYouCastSpell`
(struct-variant at ~line 2980)
**Action**: Add field:
```rust
/// OR-semantics spell-subtype filter (CR 205.1a subtypes). Fires only if the cast
/// spell has at least one of these subtypes. `None` = no subtype restriction.
/// Enables "Whenever you cast an Aura/Equipment/Vehicle spell" (Sram) and
/// "Whenever you cast an Elf spell" (Leaf-Crowned Visionary).
spell_subtype_filter: Option<Vec<SubType>>,
```

**File**: `crates/engine/src/rules/abilities.rs` — the cast-trigger post-processing `retain`
closure, `WheneverYouCastSpell` arm (lines 3392–3428). `spell_subtypes` is ALREADY computed
at line 3368. **Action**: after the `spell_type_filter` block (~line 3412), add:
```rust
if let Some(sub_filter) = spell_subtype_filter {
    if !sub_filter.iter().any(|st| spell_subtypes.contains(st)) {
        return false;
    }
}
```
Destructure the new field in the match pattern (add `spell_subtype_filter,` alongside the
existing fields).
**CR**: 205.1a (spell subtypes) — a spell's subtypes are read from the stack object's
layer-resolved `characteristics.subtypes`, already done at 3368-3372.

**File**: `crates/engine/src/state/hash.rs` — `WheneverYouCastSpell` arm (lines 4995–5006)
**Action**: add `spell_subtype_filter,` to the destructure and `spell_subtype_filter.hash_into(hasher);`.
`Option<Vec<SubType>>` already has a `HashInto` impl path (mirrors `spell_type_filter`).

### Change 4 — HASH_SCHEMA_VERSION bump

**File**: `crates/engine/src/state/hash.rs:274`
**Action**: `pub const HASH_SCHEMA_VERSION: u8 = 34;` (was 33). Add a changelog comment:
"34 (PB-AC7): + LayerModification::SetCreatureTypes (disc 30) [+ SetCardTypes disc 31];
+ WheneverYouCastSpell.spell_subtype_filter."
**Sentinel updates**: grep for `HASH_SCHEMA_VERSION, 33` and `HASH_SCHEMA_VERSION == 33`
across `crates/engine/tests/` and update to 34.

### Change 5 — Exhaustive-match / construction-site fan-out (the #1 compile-error source)

**New field on `WheneverYouCastSpell`** — every explicit construction must add
`spell_subtype_filter: None`. **21 card-def sites** (verified via grep `WheneverYouCastSpell {`):

| File |
|------|
| `cards/defs/zendikar_resurgent.rs` |
| `cards/defs/whispering_wizard.rs` |
| `cards/defs/vanquishers_banner.rs` |
| `cards/defs/talrand_sky_summoner.rs` |
| `cards/defs/storm_kiln_artist.rs` |
| `cards/defs/slickshot_show_off.rs` |
| `cards/defs/oketras_monument.rs` |
| `cards/defs/murmuring_mystic.rs` |
| `cards/defs/monastery_mentor.rs` |
| `cards/defs/lys_alana_huntmaster.rs` |
| `cards/defs/inexorable_tide.rs` |
| `cards/defs/hazorets_monument.rs` |
| `cards/defs/hermes_overseer_of_elpis.rs` |
| `cards/defs/hullbreaker_horror.rs` |
| `cards/defs/guttersnipe.rs` |
| `cards/defs/chulane_teller_of_tales.rs` |
| `cards/defs/bontus_monument.rs` |
| `cards/defs/beast_whisperer.rs` |
| `cards/defs/archmage_emeritus.rs` |
| `cards/defs/archmage_of_runes.rs` |
| `cards/defs/alela_cunning_conqueror.rs` |

Plus the NEW authored `sram_senior_edificer.rs` and `leaf_crowned_visionary.rs` (they SET it).
`testing/replay_harness.rs:2434` uses `{ .. }` — no change. **Runner must also grep
`WheneverYouCastSpell` in `rules/`, `state/`, `cards/builder.rs`** for any other explicit
destructure/construction that isn't `{ .. }`.

**New `LayerModification` variant(s)** — every non-`_`-covered match must add an arm:

| File | Match | Action |
|------|-------|--------|
| `state/continuous_effect.rs` | enum def | add variant(s) |
| `rules/layers.rs` `apply_layer_modification` | exhaustive `match modification` | add execution arm(s) |
| `rules/layers.rs` `depends_on` | `match (&a.modification, &b.modification)` has `_ => false` wildcard | no new arm required unless a dependency is needed (see Risks) |
| `state/hash.rs` `impl HashInto for LayerModification` | exhaustive | add discriminant 30 [+31] |

`LayerModification` is engine-internal — the TUI (`stack_view.rs`) and replay-viewer
(`view_model.rs`) do NOT match on it (they match `StackObjectKind` + `KeywordAbility`, which
are UNTOUCHED this batch). Still run `cargo build --workspace` to be certain.

**Discriminant chains — UNAFFECTED this batch** (verified from current code):
- `KeywordAbility`, `AbilityDefinition`, `StackObjectKind`: **no new variants** — chains
  unchanged. Do not touch KW/AbilDef/SOK discriminants.
- `Effect`: **no new variants** (primitives #1/#3 are redundant).
- `LayerModification` HashInto next-free discriminant is **30** (then 31).

---

## SetCreatureTypes rationale & honest yield

`SetCreatureTypes` is brief-mandated and small, but **0 fully-clean roster cards require it**:
every confirmed type-changer in scope is a full "becomes an X creature, loses all other card
types" reset that `SetTypeLine` already expresses (Oko precedent, shipped). Its would-be user
`frodo_saurons_bane.rs` needs it BUT is also blocked on: conditional-on-own-subtype activated
abilities, self-scoped characteristic-override from an activated ability, granting a triggered
ability via continuous effect, and Ring mechanics → Frodo stays PARTIAL.

**Recommendation**: implement `SetCreatureTypes` (+ optional `SetCardTypes`) with unit tests,
and author Kenrith / Eaten by Piranhas / Darksteel Mutation using `SetCardTypes` +
`SetCreatureTypes` (CR-faithful, preserves Legendary) instead of `SetTypeLine`. That gives
both new Layer-4 variants immediate roster use. If minimal scope is preferred, use
`SetTypeLine` for those Auras and keep `SetCreatureTypes` unit-test-only — flag as
`OOS-AC7-4` for a future card that needs partial creature-type replacement.

---

## Card Definition Fixes (backfill) — all against oracle text via MCP

Legend: **CLEAN** = fully authorable after this PB; **PARTIAL** = the type-change/ability-
removal clause is now authorable but another clause remains blocked (flagged OOS).

### 1. `kenriths_transformation.rs` — CLEAN (stale TODO; needs NO new primitive)
Oracle: "Enchant creature / When this Aura enters, draw a card. / Enchanted creature loses
all abilities and is a green Elk creature with base power and toughness 3/3."
Fix (keep existing Enchant + ETB-draw): add four `AbilityDefinition::Static` effects on
`EffectFilter::AttachedCreature`, `WhileSourceOnBattlefield`, matching the Oko pattern:
- Layer `Ability`: `RemoveAllAbilities`
- Layer `TypeChange`: `SetTypeLine{ supertypes: {}, card_types: {Creature}, subtypes: {Elk} }`
  (or CR-faithful `SetCardTypes({Creature})` + `SetCreatureTypes({Elk})` if Change 2 adopted)
- Layer `ColorChange`: `SetColors({Green})`
- Layer `PtSet`: `SetPowerToughness{ power: 3, toughness: 3 }`
Delete the stale TODO comment.

### 2. `eaten_by_piranhas.rs` — CLEAN (stale TODO; needs NO new primitive)
Oracle: "Flash / Enchant creature / Enchanted creature loses all abilities and is a black
Skeleton creature with base power and toughness 1/1. (It loses all other colors, card types,
and creature types.)"
Fix (already has RemoveAllAbilities + SetPowerToughness(1/1)): ADD `SetColors({Black})`
(Layer ColorChange) and the Layer-4 type set (`SetTypeLine{card_types:{Creature},subtypes:{Skeleton}}`
or `SetCardTypes` + `SetCreatureTypes`). Delete both TODO lines.

### 3. `darksteel_mutation.rs` — CLEAN (currently `abilities: vec![]`)
Oracle: "Enchant creature / Enchanted creature is an Insect artifact creature with base power
and toughness 0/1 and has indestructible, and it loses all other abilities, card types, and
creature types."
Fix — five Static effects on `AttachedCreature`, `WhileSourceOnBattlefield`, **ORDER MATTERS**
(see Risks — the indestructible grant must have a LATER timestamp than the removal so "loses
all OTHER abilities" preserves it; `register_static_continuous_effects` assigns incrementing
timestamps in vec order, so list removal BEFORE grant):
1. Keyword Enchant(Creature) [already implied by type line — add `AbilityDefinition::Keyword(Enchant(Creature))`]
2. Layer `Ability`: `RemoveAllAbilities`  ← list first (earlier timestamp)
3. Layer `Ability`: `AddKeyword(Indestructible)`  ← list after removal (later timestamp → survives)
4. Layer `TypeChange`: `SetTypeLine{card_types:{Artifact,Creature},subtypes:{Insect}}`
   (or `SetCardTypes({Artifact,Creature})` + `SetCreatureTypes({Insect})`)
5. Layer `PtSet`: `SetPowerToughness{0,1}`

### 4. `sram_senior_edificer.rs` — CLEAN (needs primitive #4)
Oracle: "Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card."
Fix: one `AbilityDefinition::Triggered` with
`TriggerCondition::WheneverYouCastSpell { during_opponent_turn: false, spell_type_filter: None,
noncreature_only: false, chosen_subtype_filter: false, spell_subtype_filter:
Some(vec![SubType("Aura"), SubType("Equipment"), SubType("Vehicle")]) }`, effect
`DrawCards{ Controller, 1 }`. Delete ENGINE-BLOCKED comments.
**Verify** the cards' spell type lines carry these subtypes at cast time — Aura enchantments,
Equipment artifacts, and Vehicle artifacts must have the subtype on their `types`. (These are
populated by `enrich_spec_from_def` from each card's `types_sub([...],["Aura"])` etc.)

### 5. `leaf_crowned_visionary.rs` — CLEAN (needs primitive #4 + PB-AC2 MayPayThenEffect)
Oracle: "Other Elves you control get +1/+1. / Whenever you cast an Elf spell, you may pay {G}.
If you do, draw a card."
Fix (keep the existing +1/+1 static): add `Triggered` with
`WheneverYouCastSpell { ..default fields.., spell_subtype_filter: Some(vec![SubType("Elf")]) }`
and effect `Effect::MayPayThenEffect { cost: {G}, effect: DrawCards{Controller,1} }` (PB-AC2).
Delete ENGINE-BLOCKED comment.

### 6. `final_showdown.rs` — PARTIAL (mode 0 clean; mode 1 stays blocked)
Oracle mode 0: "All creatures lose all abilities until end of turn."
Fix mode 0 (replace the `Effect::Sequence(vec![])` placeholder): `Effect::ApplyContinuousEffect
{ effect_def: ContinuousEffectDef { layer: Ability, modification: RemoveAllAbilities, filter:
AllCreatures, duration: UntilEndOfTurn, condition: None } }`. Delete mode-0 TODO.
Mode 1 ("Choose a creature you control. It gains indestructible until end of turn") needs a
non-target "choose a creature you control" selection + grant → **OOS-AC7-2**; leave mode-1
TODO. Mode 2 (DestroyAll) already done.

### 7. `vraska_betrayals_sting.rs` — PARTIAL (−2 clause clean; −9 stays blocked)
Oracle −2: "Target creature becomes a Treasure artifact with '{T}, Sacrifice this artifact:
Add one mana of any color' and loses all other card types and abilities."
Fix −2: add `LoyaltyAbility{ cost: Minus(2), targets: [TargetCreature], effect: Sequence([
  ApplyContinuousEffect{ TypeChange, SetTypeLine{card_types:{Artifact},subtypes:{Treasure}},
    DeclaredTarget{0}, Indefinite },
  ApplyContinuousEffect{ Ability, RemoveAllAbilities, DeclaredTarget{0}, Indefinite },
  ApplyContinuousEffect{ Ability, AddActivatedAbility(<{T},Sac: add any color>), DeclaredTarget{0}, Indefinite },
]) }` — **removal listed before the AddActivatedAbility grant** (same timestamp; stable sort
preserves push order so grant applies after removal — see Risks; verify with a test). The −9
clause needs `EffectAmount::PoisonDifference` → **OOS-AC7-1**; leave that TODO.

**Frodo, Sauron's Bane** — NOT authored this batch. Needs `SetCreatureTypes` PLUS
conditional-on-own-subtype activated abilities, self-characteristic-override, triggered-ability
grants, and Ring mechanics → **OOS-AC7-3**. Leave as-is.

---

## New Card Definitions
None net-new files; all seven above already exist as stubs/partials and are re-authored.

---

## Unit Tests

**File**: `crates/engine/tests/pb_ac7_type_change_ability_removal.rs` (new)
Pattern: follow the layer tests in `crates/engine/tests/` that call
`calculate_characteristics` directly (1-player builds are fine for pure layer calc, per
gotchas-infra "1-player start_game doesn't reach Cleanup" — use 2 players only for the
cleanup-expiry test).

Tests (each cites its CR):
- `test_set_creature_types_replaces_creature_subtypes_keeps_card_types` — CR 205.1a. Build an
  artifact creature with subtypes {Golem}; apply `SetCreatureTypes({Elk})`; assert subtypes ==
  {Elk}, card_types still contains Artifact+Creature, supertypes preserved.
- `test_set_creature_types_preserves_noncreature_subtypes` — CR 205.1a. Target has a land
  subtype (e.g. a creature-land) + creature subtype; SetCreatureTypes replaces only the
  creature subtype, land subtype survives.
- `test_lose_abilities_one_shot_until_eot` — CR 613.1f + 611.2a. ApplyContinuousEffect
  RemoveAllAbilities on AllCreatures, UntilEndOfTurn; creature loses flying; after a full turn
  cycle to cleanup, flying returns. (2-player for cleanup.)
- `test_granted_then_removed_ordering_by_timestamp` — CR 613.7. Static AddKeyword(Flying)
  timestamp T1 on a creature; then RemoveAllAbilities timestamp T2 > T1 → flying removed.
  Reverse (removal T1, grant T2) → flying survives. Validates the Darksteel-Mutation
  "loses all OTHER abilities but keeps indestructible" ordering. **Wedge on `keywords`
  (the property the Layer-6 effect actually reads), not an incidental field** (gotcha #39).
- `test_darksteel_mutation_keeps_indestructible` — CR 205.1b + 613.7. Full Aura integration:
  attach Darksteel Mutation to a 5/5 flyer; assert result is Insect artifact creature, 0/1,
  has Indestructible, has NO other abilities (flying gone).
- `test_lose_abilities_vs_face_down_override` — CR 708.2. A morph face-down 2/2 (empty ability
  set from the pre-loop override at `layers.rs:216`) under RemoveAllAbilities → still 2/2 with
  no abilities; assert no panic and no negative interaction (removal from an already-empty set).
- `test_set_creature_types_layer4_dependency_with_add_subtypes` — CR 613.8. Urborg-style
  `AddSubtypes({Swamp})` (a land subtype, non-creature) co-resident with `SetCreatureTypes` on
  the same object → the land subtype survives regardless of order (they target disjoint subtype
  sets). If Change 2's `depends_on` arm is added, assert the documented ordering; otherwise
  assert timestamp-order independence.
- `test_spell_subtype_filter_positive` — CR 205.1a. Sram on battlefield; cast an Equipment
  spell → draw trigger fires. Cast a Vehicle spell → fires. Cast an Aura spell → fires.
- `test_spell_subtype_filter_negative` — cast a plain creature/sorcery spell with Sram out →
  NO draw trigger.
- `test_spell_subtype_filter_none_matches_all` — a `WheneverYouCastSpell` with
  `spell_subtype_filter: None` (existing cards) still fires on every qualifying spell
  (regression guard for the 21 `None` sites).
- `test_hash_schema_version_is_34` — sentinel: `assert_eq!(HASH_SCHEMA_VERSION, 34)`.
- `test_hash_distinguishes_set_creature_types_payload` — two `ContinuousEffect`s identical
  except `SetCreatureTypes({Elk})` vs `SetCreatureTypes({Frog})` hash differently (validates
  the new HashInto arm + discriminant 30). Also one with `SetCreatureTypes` vs `SetTypeLine`
  same subtypes → different discriminants → different hashes.
- `test_hash_distinguishes_spell_subtype_filter` — two `WheneverYouCastSpell` TriggerConditions
  identical except `spell_subtype_filter: None` vs `Some(vec![SubType("Elf")])` hash
  differently (validates the new destructure/hash line). NOTE: no new *mutable runtime field*
  is added to GameState/PlayerState/GameObject this batch, so the PB-AC1/AC5 "mutation-verified
  runtime field" HIGH does not apply; these two hash tests cover the new enum variant + field
  per the "every new variant/field needs a hash arm + verification" rule.

---

## Verification Checklist

- [ ] `LayerModification::SetCreatureTypes` (+ optional `SetCardTypes`) compiles (`cargo check`)
- [ ] `spell_subtype_filter` added to `WheneverYouCastSpell`; all 21 `None` sites updated;
      Sram/Leaf-Crowned SET it
- [ ] abilities.rs post-processing subtype check added; hash.rs arms updated (both
      LayerModification + WheneverYouCastSpell)
- [ ] HASH_SCHEMA_VERSION = 34; changelog comment added; all test sentinels updated 33→34
- [ ] Backfill: Kenrith, Eaten by Piranhas, Darksteel Mutation, Sram, Leaf-Crowned re-authored
      CLEAN; Final Showdown mode 0 + Vraska −2 clause authored; stale TODOs deleted
- [ ] `cargo test --all` (NOT just `cargo build` — build skips test targets, PB-AC6 note)
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo build --workspace` (catches TUI/replay-viewer even though untouched)
- [ ] `cargo fmt --check`
- [ ] `python3 tools/authoring-report.py` → post clean-coverage delta as task comment
- [ ] No `Effect::LoseAbilities` / redundant one-shot-type-override Effect variant was added
- [ ] Remaining TODOs only on OOS-flagged clauses (Vraska −9, Final Showdown mode 1, Frodo)

---

## Risks & Edge Cases

- **Granted-then-removed timestamp ordering (Darksteel Mutation, Vraska −2).** For Static Aura
  effects, `register_static_continuous_effects` (`resolution.rs:~4952`) assigns *incrementing*
  timestamps in ability-vec order — so list `RemoveAllAbilities` BEFORE `AddKeyword(Indestructible)`.
  For the one-shot Vraska sequence, `Effect::ApplyContinuousEffect` does **NOT** advance
  `timestamp_counter` (verified `effects/mod.rs:2871-2896`) — every effect in the Sequence
  shares one timestamp. The layer sort is a **stable** `sort_by_key(|e| e.timestamp)`
  (`layers.rs:1271`), and `depends_on` returns `false` for (Add*, RemoveAllAbilities) pairs
  (`layers.rs:1354`), so the effects apply in `continuous_effects` push order = Sequence order.
  Listing removal before the grant makes the grant survive. **This is correct but fragile and
  undocumented — the `test_granted_then_removed_ordering_by_timestamp` and
  `test_darksteel_mutation_keeps_indestructible` tests lock it in.** If the reviewer wants
  robustness, consider having `ApplyContinuousEffect` increment `timestamp_counter` per push
  so intra-Sequence ordering is explicit rather than stable-sort-incidental (flag as a
  follow-up, do not change silently — it would shift many existing effect timestamps and could
  perturb hashes/goldens).
- **CR 613.8 dependency for `SetCreatureTypes` vs `AddSubtypes`.** `SetTypeLine` already depends
  on `AddSubtypes`/`AddCardTypes` (`layers.rs:1348`). A creature-type SET and a *land*-subtype
  ADD target disjoint subtype sets, so no dependency is needed for the roster. But a
  hypothetical `AddSubtypes({some creature type})` co-resident with `SetCreatureTypes` would
  want the SET to win (apply after) — if such a card appears, add a `(SetCreatureTypes,
  AddSubtypes)` dependency arm. Decide and document; default is timestamp order.
- **`SetTypeLine` supertype wipe (accepted approximation vs CR-faithful).** Oko ships
  `SetTypeLine{supertypes: empty}`, dropping `Legendary`. Oracle says "loses all other card
  types" (NOT supertypes) → Legendary should survive. The optional `SetCardTypes` +
  `SetCreatureTypes` authoring path fixes this; `SetTypeLine` does not. Reviewer decides
  whether CR-faithfulness is worth Change 2.
- **Spell subtypes must be present on the cast spell's characteristics.** `spell_subtype_filter`
  reads `stack_object.characteristics.subtypes` (`abilities.rs:3368`). Confirm Aura/Equipment/
  Vehicle cards populate those subtypes on their `types` and that `enrich_spec_from_def`
  propagates them to the stack object. If a Vehicle card lacks the `Vehicle` subtype in its
  def, Sram will silently under-trigger.
- **Face-down composition (CR 708.2).** The 2/2-no-text override runs before the layer loop and
  empties abilities; a later `RemoveAllAbilities` is a no-op on the empty set. No conflict, but
  the test guards against a regression if the override is ever reordered.
- **No new mutable runtime fields** are introduced on `GameState`/`PlayerState`/`GameObject`,
  so the recurring PB-AC1/AC5 mutation-verified-field HIGH does not apply. The two hash-payload
  tests cover the new enum variant + trigger field.

---

## OOS-AC7 seeds (out of scope for this PB)

- **OOS-AC7-1** — `EffectAmount::PoisonDifference` ("poison counters equal to 9 minus target's
  current poison"). Blocks Vraska −9. 0–1 cards; low priority.
- **OOS-AC7-2** — Non-target "choose a creature you control" selection + grant keyword
  (`EffectTarget::ChosenCreatureYouControl` + `Effect::GrantKeyword`/ApplyContinuousEffect).
  Blocks Final Showdown mode 1. Possibly small cluster; medium.
- **OOS-AC7-3** — Frodo, Sauron's Bane cluster: conditional-on-own-subtype activated ability,
  self-scoped characteristic-override from an activated ability, granting a *triggered* ability
  via continuous effect, and Ring-tempt count condition. Multi-primitive; defer.
- **OOS-AC7-4** — If Change 2 (`SetCardTypes`) is NOT adopted: a future card needing partial
  creature-type replacement while preserving card types with the CR-faithful supertype
  behavior. Also: `SetCreatureTypes` currently has 0 fully-clean roster cards — revisit yield
  when W-PB3 authoring runs.
